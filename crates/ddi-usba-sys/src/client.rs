//! The USBA client driver interface from `sys/usb/usba.h` and
//! `sys/usb/usbai.h`.
//!
//! A USB client driver is an ordinary leaf driver whose device node was
//! created by the hub driver for a USB device or for one interface of it.
//! In `attach(9E)` it registers with the framework through
//! [`usb_client_attach`] and fetches the parsed descriptors of its device
//! through [`usb_get_dev_data`]. It finds the endpoints it needs with
//! [`usb_lookup_ep_data`], turns each into an extended endpoint descriptor
//! with [`usb_ep_xdescr_fill`] and opens a pipe to it with
//! [`usb_pipe_xopen`]. All I/O then happens by allocating a control, bulk,
//! interrupt or isochronous request, filling it in and handing it to the
//! matching `usb_pipe_*_xfer` function. Completion is reported either by
//! blocking the caller, when `USB_FLAGS_SLEEP` is passed, or through the
//! normal and exception callbacks stored in the request.
//!
//! This module binds every function on the `usb_*(9F)` pages, every
//! structure on the `usb_*(9S)` pages together with the structures of the
//! descriptor tree that `usb_client_dev_data(9S)` documents, the enumerations
//! of callback flags, completion reasons, request attributes, pipe states,
//! parse levels and reset levels, and the constants that those pages and the
//! header define for descriptor fields, device requests, class codes and
//! power levels. The C enumerations are declared as transparent newtypes
//! over `c_int` with associated constants, because several of them are bit
//! masks whose combinations are not enumerators.
//!
//! The descriptor structures in `usbai.h` are not packed. They are the
//! parsed form that the framework produces after converting the little
//! endian wire format to host order, and they keep the natural alignment of
//! their 16-bit members. `usb_cfg_descr_t` is therefore 10 bytes long where
//! the wire descriptor is 9, and `usb_ep_descr_t` is 8 bytes long where the
//! wire descriptor is 7. The `usba_hcdi(9E)` page warns explicitly that
//! these types must not be used to build raw descriptors. Raw class and
//! vendor specific descriptors are converted with [`usb_parse_data`].
//!
//! Several names in the header are left out on purpose. The descriptor
//! parsers `usb_parse_dev_descr`, `usb_parse_cfg_descr`,
//! `usb_parse_ia_descr`, `usb_parse_if_descr`, `usb_parse_ep_descr` and
//! `usb_parse_CV_cfg_descr` are declared in the private `usba_private.h`
//! and have no manual page; [`usb_parse_data`] is the documented way to
//! parse descriptors. `usb_register_event_cbs` and
//! `usb_unregister_event_cbs` are declared only in the private
//! `usbai_private.h` and have no manual page either. The interface
//! association descriptor `usb_ia_descr_t` has no page and no function takes
//! it. `usb_dev_str_t`, `usb_dev_driver_callback_t`,
//! `usb_register_dev_driver` and `usb_unregister_dev_driver` are marked in
//! the header as Contracted Project Private for VirtualBox device capture.
//! The Wireless USB and wire adapter constants, such as
//! `USB_DESCR_TYPE_WA`, `USB_REQ_SET_ENCRYPTION`, `USB_DEV_WUSB` and the
//! `USB_SUBCLS_WUSB_*` codes, belong to hardware support that illumos no
//! longer ships and are not documented on any page. The function-like
//! macros `USBA_MAKE_VER`, `USBA_GET_MAJOR`, `USBA_GET_MINOR`,
//! `USB_DEV_OS_PWR2USB_PWR`, `USB_DEV_PWRMASK` and `USB_DEV_PWRSTATE_OK`
//! have no symbol and are left to the safe crate. The USBA 1.0 compatibility
//! header `usba10.h` is obsolete and nothing from it is bound.

use ddi_sys::{
    boolean_t, c_char, c_int, c_uchar, c_uint, c_void, ddi_iblock_cookie_t, dev_info_t, mblk_t, size_t, uint_t,
};

opaque! {
    /// The structure behind a pipe handle, `struct usb_pipe_handle`. The
    /// header states that the pipe handle is opaque to the client driver.
    usb_pipe_handle;
    /// The structure behind the general opaque pointer `usb_opaque_t`. It
    /// has no definition anywhere; the type only carries client private data
    /// and requests through the framework.
    usb_opaque;
}

/// A handle to an open pipe, `usb_pipe_handle_t`. It is returned by
/// [`usb_pipe_xopen`] and [`usb_pipe_open`], and the handle of the default
/// control pipe is found in `dev_default_ph` of [`usb_client_dev_data`].
pub type usb_pipe_handle_t = *mut usb_pipe_handle;
/// A general opaque pointer, `usb_opaque_t`, used for client private data,
/// callback arguments and requests passed without their type.
pub type usb_opaque_t = *mut usb_opaque;
/// An absolute USB frame number, `usb_frame_number_t`. The framework counts
/// milliseconds since boot and uses that count as the logical frame number.
pub type usb_frame_number_t = u64;

// --- Versions and device states ---------------------------------------------

/// The major version of the USBA client interface described by the header
/// these bindings follow (`USBA_MAJOR_VER`).
pub const USBA_MAJOR_VER: c_uint = 2;
/// The minor version of the USBA client interface described by the header
/// these bindings follow (`USBA_MINOR_VER`).
pub const USBA_MINOR_VER: c_uint = 1;
/// The version argument that [`usb_client_attach`] expects from a driver
/// written against USBA 2.1. In C the macro is assembled from the
/// `USBDRV_MAJOR_VER` and `USBDRV_MINOR_VER` definitions of the driver as
/// the major version shifted left by eight bits combined with the minor
/// version. A driver using these bindings declares the version they were
/// written for, which is 2.1 (`USBDRV_VERSION`).
pub const USBDRV_VERSION: c_uint = (USBA_MAJOR_VER << 8) | USBA_MINOR_VER;

/// The device is online and usable. Client drivers track their device state
/// with these values, and states 0x80 to 0xff are left free for device
/// specific use (`USB_DEV_ONLINE`).
pub const USB_DEV_ONLINE: c_int = 1;
/// The device has been disconnected (`USB_DEV_DISCONNECTED`).
pub const USB_DEV_DISCONNECTED: c_int = 2;
/// The device has been suspended by a `DDI_SUSPEND` operation
/// (`USB_DEV_SUSPENDED`).
pub const USB_DEV_SUSPENDED: c_int = 3;
/// The device is powered down (`USB_DEV_PWRED_DOWN`).
pub const USB_DEV_PWRED_DOWN: c_int = 4;

// --- Return values ------------------------------------------------------------

/// The call succeeded (`USB_SUCCESS`).
pub const USB_SUCCESS: c_int = 0;
/// An unspecified error occurred in the framework or the host controller
/// driver (`USB_FAILURE`).
pub const USB_FAILURE: c_int = -1;
/// No resources were available (`USB_NO_RESOURCES`).
pub const USB_NO_RESOURCES: c_int = -2;
/// No bus bandwidth was available (`USB_NO_BANDWIDTH`).
pub const USB_NO_BANDWIDTH: c_int = -3;
/// The host controller driver does not support the function
/// (`USB_NOT_SUPPORTED`).
pub const USB_NOT_SUPPORTED: c_int = -4;
/// An error occurred on the pipe (`USB_PIPE_ERROR`).
pub const USB_PIPE_ERROR: c_int = -5;
/// The pipe handle passed is invalid (`USB_INVALID_PIPE`).
pub const USB_INVALID_PIPE: c_int = -6;
/// An isochronous request specified neither a start frame nor as soon as
/// possible scheduling (`USB_NO_FRAME_NUMBER`).
pub const USB_NO_FRAME_NUMBER: c_int = -7;
/// The starting frame of an isochronous request is not valid
/// (`USB_INVALID_START_FRAME`).
pub const USB_INVALID_START_FRAME: c_int = -8;
/// The host controller reported a hardware error (`USB_HC_HARDWARE_ERROR`).
pub const USB_HC_HARDWARE_ERROR: c_int = -9;
/// The request contained invalid values (`USB_INVALID_REQUEST`).
pub const USB_INVALID_REQUEST: c_int = -10;
/// The call was made from a context in which it is not allowed, typically
/// with `USB_FLAGS_SLEEP` from interrupt context (`USB_INVALID_CONTEXT`).
pub const USB_INVALID_CONTEXT: c_int = -11;
/// The version passed to the framework is not valid (`USB_INVALID_VERSION`).
pub const USB_INVALID_VERSION: c_int = -12;
/// The arguments are not valid (`USB_INVALID_ARGS`).
pub const USB_INVALID_ARGS: c_int = -13;
/// The operation is privileged and the caller does not own what it tried to
/// change (`USB_INVALID_PERM`).
pub const USB_INVALID_PERM: c_int = -14;
/// The device or pipe is busy (`USB_BUSY`).
pub const USB_BUSY: c_int = -15;

// --- Enumerations ----------------------------------------------------------------

/// `usb_callback_flags(9S)`. illumos DDI specific (illumos DDI).
///
/// The callback flags, `usb_cb_flags_t`. Before the framework calls an
/// exception callback it sets these flags to report errors found during the
/// recovery of a failed request and to summarise the recovery actions it
/// took. They supplement the transport error, which is found in the
/// completion reason of the request for transfers and in the `rval`
/// argument for command completion callbacks. The type is a bit mask, and
/// more than one flag may be set at the same time.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_cb_flags_t(pub c_int);
impl usb_cb_flags_t {
    /// No additional errors were found and no recovery action was taken
    /// (`USB_CB_NO_INFO`).
    pub const USB_CB_NO_INFO: Self = Self(0x00);
    /// A functional stall was cleared by the framework, which happens when
    /// the request has `USB_ATTRS_AUTOCLEARING` set (`USB_CB_STALL_CLEARED`).
    pub const USB_CB_STALL_CLEARED: Self = Self(0x01);
    /// A functional stall occurred during the transfer. It is usually caused
    /// by a hardware error, must be cleared explicitly, and is fatal if it
    /// cannot be cleared. The default control pipe never shows a functional
    /// stall (`USB_CB_FUNCTIONAL_STALL`).
    pub const USB_CB_FUNCTIONAL_STALL: Self = Self(0x02);
    /// A protocol stall occurred during the transfer. It is usually caused
    /// by an invalid or misunderstood command and is cleared automatically
    /// when the device receives its next command. Stalls on the default pipe
    /// are treated as protocol stalls (`USB_CB_PROTOCOL_STALL`).
    pub const USB_CB_PROTOCOL_STALL: Self = Self(0x04);
    /// The pipe was reset, either automatically through autoclearing or by
    /// an explicit call to [`usb_pipe_reset`]. The failed request and every
    /// pending request flushed by the reset carry this flag
    /// (`USB_CB_RESET_PIPE`).
    pub const USB_CB_RESET_PIPE: Self = Self(0x10);
    /// Resources for running the callback asynchronously could not be
    /// allocated. A callback that sees this flag must not block, and it may
    /// run out of order with respect to earlier requests
    /// (`USB_CB_ASYNC_REQ_FAILED`).
    pub const USB_CB_ASYNC_REQ_FAILED: Self = Self(0x80);
    /// Not enough resources were available for recovery to proceed
    /// (`USB_CB_NO_RESOURCES`).
    pub const USB_CB_NO_RESOURCES: Self = Self(0x100);
    /// A queued request was submitted to the host controller driver, which
    /// rejected it. The completion reason tells why (`USB_CB_SUBMIT_FAILED`).
    pub const USB_CB_SUBMIT_FAILED: Self = Self(0x200);
    /// The callback runs in interrupt context and must not block
    /// (`USB_CB_INTR_CONTEXT`).
    pub const USB_CB_INTR_CONTEXT: Self = Self(0x400);
}

/// `usb_completion_reason(9S)`. illumos DDI specific (illumos DDI).
///
/// The completion reason of a request, `usb_cr_t`. The host controller
/// driver sets exactly one of these values in the request before the
/// framework calls the normal or the exception callback. `USB_CR_OK` means
/// that the transfer completed without any error being detected.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_cr_t(pub c_int);
impl usb_cr_t {
    /// The transfer completed without errors (`USB_CR_OK`).
    pub const USB_CR_OK: Self = Self(0);
    /// A CRC error was detected (`USB_CR_CRC`).
    pub const USB_CR_CRC: Self = Self(1);
    /// A bit stuffing violation was detected (`USB_CR_BITSTUFFING`).
    pub const USB_CR_BITSTUFFING: Self = Self(2);
    /// The data toggle packet identifier did not match the expected value
    /// (`USB_CR_DATA_TOGGLE_MM`).
    pub const USB_CR_DATA_TOGGLE_MM: Self = Self(3);
    /// The endpoint indicated that it is stalled. With autoclearing enabled,
    /// the callback flags tell whether the stall was functional or a
    /// protocol stall (`USB_CR_STALL`).
    pub const USB_CR_STALL: Self = Self(4);
    /// The host controller timed out waiting for the device to respond
    /// (`USB_CR_DEV_NOT_RESP`).
    pub const USB_CR_DEV_NOT_RESP: Self = Self(5);
    /// The check bits on the packet identifier returned by the device were
    /// wrong (`USB_CR_PID_CHECKFAILURE`).
    pub const USB_CR_PID_CHECKFAILURE: Self = Self(6);
    /// The packet identifier received was not valid (`USB_CR_UNEXP_PID`).
    pub const USB_CR_UNEXP_PID: Self = Self(7);
    /// The device returned more data than the maximum packet size of the
    /// endpoint or the remaining buffer allowed (`USB_CR_DATA_OVERRUN`).
    pub const USB_CR_DATA_OVERRUN: Self = Self(8);
    /// The device returned less data than requested and the request did not
    /// allow short transfers (`USB_CR_DATA_UNDERRUN`).
    pub const USB_CR_DATA_UNDERRUN: Self = Self(9);
    /// The device sent data faster than the system could take it
    /// (`USB_CR_BUFFER_OVERRUN`).
    pub const USB_CR_BUFFER_OVERRUN: Self = Self(10);
    /// The host controller could not fetch data from the system fast enough
    /// to keep up with the USB data rate (`USB_CR_BUFFER_UNDERRUN`).
    pub const USB_CR_BUFFER_UNDERRUN: Self = Self(11);
    /// The timeout of a control, bulk or one-time interrupt request expired
    /// (`USB_CR_TIMEOUT`).
    pub const USB_CR_TIMEOUT: Self = Self(12);
    /// The host controller neither accessed nor processed the request
    /// (`USB_CR_NOT_ACCESSED`).
    pub const USB_CR_NOT_ACCESSED: Self = Self(13);
    /// No resources were available to continue servicing a periodic
    /// interrupt or isochronous request (`USB_CR_NO_RESOURCES`).
    pub const USB_CR_NO_RESOURCES: Self = Self(14);
    /// An unspecified error occurred in the framework or the host
    /// controller driver (`USB_CR_UNSPECIFIED_ERR`).
    pub const USB_CR_UNSPECIFIED_ERR: Self = Self(15);
    /// Polling on an interrupt IN or isochronous IN endpoint stopped, so the
    /// periodic request cannot continue (`USB_CR_STOPPED_POLLING`).
    pub const USB_CR_STOPPED_POLLING: Self = Self(16);
    /// The request was not started because its pipe was closing or closed
    /// (`USB_CR_PIPE_CLOSING`).
    pub const USB_CR_PIPE_CLOSING: Self = Self(17);
    /// The request was not started because its pipe was reset
    /// (`USB_CR_PIPE_RESET`).
    pub const USB_CR_PIPE_RESET: Self = Self(18);
    /// The request or command is not supported (`USB_CR_NOT_SUPPORTED`).
    pub const USB_CR_NOT_SUPPORTED: Self = Self(19);
    /// The request was not completed because its pipe went into an error
    /// state, stalled, was reset or was closed (`USB_CR_FLUSHED`).
    pub const USB_CR_FLUSHED: Self = Self(20);
    /// A general host controller hardware error prevented completion
    /// (`USB_CR_HC_HARDWARE_ERR`).
    pub const USB_CR_HC_HARDWARE_ERR: Self = Self(21);
}

/// The flags argument of the USBA functions, `usb_flags_t`. The only flag
/// most functions recognise is `USB_FLAGS_SLEEP`, which makes the call wait
/// for resources and, for many functions, for completion. Without it the
/// call must not block, and completion is reported through a callback.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_flags_t(pub c_int);
impl usb_flags_t {
    /// Do not block until resources are available (`USB_FLAGS_NOSLEEP`).
    pub const USB_FLAGS_NOSLEEP: Self = Self(0x0000);
    /// Block until resources are available (`USB_FLAGS_SLEEP`).
    pub const USB_FLAGS_SLEEP: Self = Self(0x0100);
    /// Bits reserved by the framework (`USB_FLAGS_RESERVED`).
    pub const USB_FLAGS_RESERVED: Self = Self(0xFE00);
}

/// `usb_get_dev_data(9F)`. illumos DDI specific (illumos DDI).
///
/// The parse level, `usb_reg_parse_lvl_t`. As an argument of
/// [`usb_get_dev_data`] it selects how much of the device the returned
/// descriptor tree should represent. In the `dev_parse_level` field of
/// [`usb_client_dev_data`] it reports what the tree actually represents,
/// which can differ from what was requested.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_reg_parse_lvl_t(pub c_int);
impl usb_reg_parse_lvl_t {
    /// Build no tree. `dev_n_cfg` is 0, `dev_cfg` and `dev_curr_cfg` are
    /// NULL and the other `dev_curr_*` fields are invalid
    /// (`USB_PARSE_LVL_NONE`).
    pub const USB_PARSE_LVL_NONE: Self = Self(0);
    /// Parse only the configured interface when the configuration number
    /// and interface properties are set, as when different interfaces are
    /// separate device instances. For a device node that represents the
    /// whole physical device this behaves like `USB_PARSE_LVL_ALL`
    /// (`USB_PARSE_LVL_IF`).
    pub const USB_PARSE_LVL_IF: Self = Self(1);
    /// Parse the entire configuration that contains the configured
    /// interface (`USB_PARSE_LVL_CFG`).
    pub const USB_PARSE_LVL_CFG: Self = Self(2);
    /// Parse the entire device with all of its configurations, even when
    /// the driver is bound to a single interface (`USB_PARSE_LVL_ALL`).
    pub const USB_PARSE_LVL_ALL: Self = Self(3);
}

/// `usb_pipe_get_state(9F)`. illumos DDI specific (illumos DDI).
///
/// The state of a pipe, `usb_pipe_state_t`, as reported by
/// [`usb_pipe_get_state`]. A control or bulk pipe is active as soon as it
/// is opened. An interrupt or isochronous pipe becomes active once polling
/// has been started and returns to idle when polling stops. After a device
/// error the pipe is in the error state until [`usb_pipe_reset`] is called.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_pipe_state_t(pub c_int);
impl usb_pipe_state_t {
    /// The pipe is closed (`USB_PIPE_STATE_CLOSED`).
    pub const USB_PIPE_STATE_CLOSED: Self = Self(0);
    /// Polling is stopped on an isochronous or interrupt IN pipe
    /// (`USB_PIPE_STATE_IDLE`).
    pub const USB_PIPE_STATE_IDLE: Self = Self(1);
    /// The pipe is active and can send and receive data. Polling is active
    /// for isochronous and interrupt pipes (`USB_PIPE_STATE_ACTIVE`).
    pub const USB_PIPE_STATE_ACTIVE: Self = Self(2);
    /// An error occurred and the client must call [`usb_pipe_reset`]. A
    /// client does not see this state when its requests set
    /// `USB_ATTRS_AUTOCLEARING` (`USB_PIPE_STATE_ERROR`).
    pub const USB_PIPE_STATE_ERROR: Self = Self(3);
    /// The pipe is being closed, requests are being drained and other
    /// cleanup is in progress (`USB_PIPE_STATE_CLOSING`).
    pub const USB_PIPE_STATE_CLOSING: Self = Self(4);
}

/// `usb_request_attributes(9S)`. illumos DDI specific (illumos DDI).
///
/// The request attributes, `usb_req_attrs_t`, which tell the framework how
/// to handle the execution of a request. They are stored in the
/// `*_attributes` field of each request structure, and more than one may be
/// set. `USB_ATTRS_ISOC_START_FRAME` and `USB_ATTRS_ISOC_XFER_ASAP` are
/// mutually exclusive.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_req_attrs_t(pub c_int);
impl usb_req_attrs_t {
    /// No attributes (`USB_ATTRS_NONE`).
    pub const USB_ATTRS_NONE: Self = Self(0);
    /// Accept transfers that return less data than requested without
    /// reporting an error. Only meaningful for control, bulk and interrupt
    /// IN requests (`USB_ATTRS_SHORT_XFER_OK`).
    pub const USB_ATTRS_SHORT_XFER_OK: Self = Self(0x01);
    /// Reset the pipe automatically when an error occurs, without trying to
    /// clear a stall. Pending requests are flushed unless the pipe is the
    /// default pipe (`USB_ATTRS_PIPE_RESET`).
    pub const USB_ATTRS_PIPE_RESET: Self = Self(0x02);
    /// Reset the pipe and clear functional stalls automatically when an
    /// error occurs. The value includes the bit of `USB_ATTRS_PIPE_RESET`.
    /// Autoclearing a stalled default pipe is not allowed, and neither is
    /// autoclearing a `USB_REQ_GET_STATUS` request on the default pipe
    /// (`USB_ATTRS_AUTOCLEARING`).
    pub const USB_ATTRS_AUTOCLEARING: Self = Self(0x12);
    /// Perform a single interrupt IN transfer instead of starting periodic
    /// polling (`USB_ATTRS_ONE_XFER`).
    pub const USB_ATTRS_ONE_XFER: Self = Self(0x100);
    /// Start an isochronous request at the frame number given in its
    /// `isoc_frame_no` field. Requests whose start frame has passed are
    /// rejected (`USB_ATTRS_ISOC_START_FRAME`).
    pub const USB_ATTRS_ISOC_START_FRAME: Self = Self(0x200);
    /// Start an isochronous request as soon as possible, on the frame after
    /// the last frame of the last queued request. The `isoc_frame_no` field
    /// is ignored (`USB_ATTRS_ISOC_XFER_ASAP`).
    pub const USB_ATTRS_ISOC_XFER_ASAP: Self = Self(0x400);
}

/// `usb_reset_device(9F)`. illumos DDI specific (illumos DDI).
///
/// The extent of a device reset requested with [`usb_reset_device`],
/// `usb_dev_reset_lvl_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_dev_reset_lvl_t(pub c_int);
impl usb_dev_reset_lvl_t {
    /// Reset the device, detach the current driver and start a new attach
    /// according to the updated compatible name. This level is meant for
    /// firmware downloads that change the descriptors and for other cases
    /// in which the device has to be enumerated again
    /// (`USB_RESET_LVL_REATTACH`).
    pub const USB_RESET_LVL_REATTACH: Self = Self(0);
    /// Reset the device and clear all error status. The state machines and
    /// registers of the device are cleared and must be initialised again by
    /// the driver, which remains attached. This level is meant for error
    /// recovery and for firmware downloads that do not change the
    /// descriptors (`USB_RESET_LVL_DEFAULT`).
    pub const USB_RESET_LVL_DEFAULT: Self = Self(1);
}

/// `usb_ep_xdescr(9S)`. illumos DDI Specific.
///
/// The flags of an extended endpoint descriptor, `usb_ep_xdescr_flags_t`.
/// Each flag tells whether a member of [`usb_ep_xdescr`] other than
/// `uex_ep` is present and valid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_ep_xdescr_flags_t(pub c_int);
impl usb_ep_xdescr_flags_t {
    /// A SuperSpeed endpoint companion descriptor is present and has been
    /// filled into `uex_ep_ss` (`USB_EP_XFLAGS_SS_COMP`).
    pub const USB_EP_XFLAGS_SS_COMP: Self = Self(1 << 0);
}

// --- Standard descriptors ------------------------------------------------------

/// `usb_dev_descr(9S)`. illumos DDI specific (illumos DDI).
///
/// The device descriptor, which defines attributes that apply to the whole
/// device. It is described in section 9.6.1 of the USB 2.0 specification.
/// The framework retrieves it from the device during enumeration, and a
/// client driver finds the parsed copy through the `dev_descr` field of the
/// data returned by [`usb_get_dev_data`]. The fields are in host byte
/// order, and the structure happens to have no padding, so its layout
/// matches the 18 bytes of the wire format.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct usb_dev_descr {
    /// The size of this descriptor in bytes.
    pub bLength: u8,
    /// The descriptor type, set to `USB_DESCR_TYPE_DEV`.
    pub bDescriptorType: u8,
    /// The release number of the USB specification that the device
    /// supports, in binary coded decimal.
    pub bcdUSB: u16,
    /// The class code of the device. `USB_CLASS_PER_INTERFACE` means that
    /// the class information is found at the interface level.
    pub bDeviceClass: u8,
    /// The subclass code, whose meaning is defined by the specification of
    /// the device class.
    pub bDeviceSubClass: u8,
    /// The protocol code, whose meaning is defined by the specification of
    /// the device class.
    pub bDeviceProtocol: u8,
    /// The maximum packet size of endpoint 0.
    pub bMaxPacketSize0: u8,
    /// The vendor ID.
    pub idVendor: u16,
    /// The product ID.
    pub idProduct: u16,
    /// The release number of the device in binary coded decimal.
    pub bcdDevice: u16,
    /// The index of the optional manufacturer string, valid when greater
    /// than 0.
    pub iManufacturer: u8,
    /// The index of the optional product string, valid when greater than 0.
    pub iProduct: u8,
    /// The index of the optional serial number string, valid when greater
    /// than 0.
    pub iSerialNumber: u8,
    /// The number of configurations the device offers.
    pub bNumConfigurations: u8,
}
/// The typedef name of [`usb_dev_descr`].
pub type usb_dev_descr_t = usb_dev_descr;

// usb_dev_descr: bytes at 0 and 1, bcdUSB at 2, four bytes from 4 to 7, the
// three 16-bit IDs at 8, 10 and 12, four bytes from 14 to 17. Alignment 2,
// no padding, 18 bytes.
const _: () = assert!(core::mem::size_of::<usb_dev_descr>() == 18);
const _: () = assert!(core::mem::align_of::<usb_dev_descr>() == 2);
const _: () = assert!(core::mem::offset_of!(usb_dev_descr, bLength) == 0);
const _: () = assert!(core::mem::offset_of!(usb_dev_descr, bcdUSB) == 2);
const _: () = assert!(core::mem::offset_of!(usb_dev_descr, idVendor) == 8);
const _: () = assert!(core::mem::offset_of!(usb_dev_descr, bcdDevice) == 12);
const _: () = assert!(core::mem::offset_of!(usb_dev_descr, bNumConfigurations) == 17);

/// `usb_dev_qlf_descr(9S)`. illumos DDI specific (illumos DDI).
///
/// The device qualifier descriptor, described in section 9.6.2 of the USB
/// 2.0 specification. A high speed capable device uses it to tell how the
/// fields of its device descriptor would change if it ran at its other
/// speed. When the device currently runs at full speed the descriptor
/// describes high speed operation, and the other way round.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct usb_dev_qlf_descr {
    /// The size of this descriptor in bytes.
    pub bLength: u8,
    /// The descriptor type, set to `USB_DESCR_TYPE_DEV_QLF`.
    pub bDescriptorType: u8,
    /// The release number of the USB specification in binary coded decimal.
    pub bcdUSB: u16,
    /// The device class code, as in [`usb_dev_descr`].
    pub bDeviceClass: u8,
    /// The device subclass code, defined by the specification of the class.
    pub bDeviceSubClass: u8,
    /// The protocol code, defined by the specification of the class.
    pub bDeviceProtocol: u8,
    /// The maximum packet size of endpoint 0 at the other speed.
    pub bMaxPacketSize0: u8,
    /// The number of configurations available at the other speed.
    pub bNumConfigurations: u8,
    /// Reserved.
    pub bReserved: u8,
}
/// The typedef name of [`usb_dev_qlf_descr`].
pub type usb_dev_qlf_descr_t = usb_dev_qlf_descr;

// usb_dev_qlf_descr: bytes at 0 and 1, bcdUSB at 2, six bytes from 4 to 9.
// Alignment 2, 10 bytes.
const _: () = assert!(core::mem::size_of::<usb_dev_qlf_descr>() == 10);
const _: () = assert!(core::mem::offset_of!(usb_dev_qlf_descr, bLength) == 0);
const _: () = assert!(core::mem::offset_of!(usb_dev_qlf_descr, bcdUSB) == 2);
const _: () = assert!(core::mem::offset_of!(usb_dev_qlf_descr, bReserved) == 9);

/// `usb_cfg_descr(9S)`. illumos DDI specific (illumos DDI).
///
/// The configuration descriptor, described in section 9.6.3 of the USB 2.0
/// specification. A configuration contains one or more interfaces, and its
/// descriptor acts as the header of the group of descriptors that describe
/// those interfaces and their endpoints. The framework retrieves all
/// configuration descriptors during enumeration, and a client driver finds
/// them in the descriptor tree returned by [`usb_get_dev_data`]. This is the
/// parsed form, so the natural alignment of `wTotalLength` adds one byte of
/// tail padding and the structure is 10 bytes long, where the wire format
/// is 9.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct usb_cfg_descr {
    /// The size of this descriptor in bytes.
    pub bLength: u8,
    /// The descriptor type, set to `USB_DESCR_TYPE_CFG`.
    pub bDescriptorType: u8,
    /// The total length of the data returned for this configuration,
    /// including this descriptor and all descriptors that belong to it.
    pub wTotalLength: u16,
    /// The number of interfaces in this configuration.
    pub bNumInterfaces: u8,
    /// The value that identifies this configuration, starting at 1.
    pub bConfigurationValue: u8,
    /// The index of the optional configuration string, valid when greater
    /// than 0.
    pub iConfiguration: u8,
    /// The characteristics of the configuration, a combination of
    /// `USB_CFG_ATTR_SELFPWR`, `USB_CFG_ATTR_REMOTE_WAKEUP` and
    /// `USB_CFG_ATTR_BAT_PWR`.
    pub bmAttributes: u8,
    /// The maximum power consumption in units of 2 mA.
    pub bMaxPower: u8,
}
/// The typedef name of [`usb_cfg_descr`].
pub type usb_cfg_descr_t = usb_cfg_descr;

// usb_cfg_descr: bytes at 0 and 1, wTotalLength at 2, five bytes from 4 to
// 8, one byte of tail padding for alignment 2, 10 bytes.
const _: () = assert!(core::mem::size_of::<usb_cfg_descr>() == 10);
const _: () = assert!(core::mem::align_of::<usb_cfg_descr>() == 2);
const _: () = assert!(core::mem::offset_of!(usb_cfg_descr, bLength) == 0);
const _: () = assert!(core::mem::offset_of!(usb_cfg_descr, wTotalLength) == 2);
const _: () = assert!(core::mem::offset_of!(usb_cfg_descr, bMaxPower) == 8);

/// `usb_other_speed_cfg_descr(9S)`. illumos DDI specific (illumos DDI).
///
/// The other speed configuration descriptor, described in section 9.6.4 of
/// the USB 2.0 specification. It tells how the configuration descriptor of
/// a high speed capable device would change if the device ran at its other
/// speed, from full to high speed or from high to full speed. Its layout is
/// identical to [`usb_cfg_descr`], including the byte of tail padding.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct usb_other_speed_cfg_descr {
    /// The size of this descriptor in bytes.
    pub bLength: u8,
    /// The descriptor type, set to `USB_DESCR_TYPE_OTHER_SPEED_CFG`.
    pub bDescriptorType: u8,
    /// The total length of the data returned for the other speed
    /// configuration, including all of its descriptors.
    pub wTotalLength: u16,
    /// The number of interfaces in the configuration.
    pub bNumInterfaces: u8,
    /// The value that identifies the other speed configuration, starting at
    /// 1.
    pub bConfigurationValue: u8,
    /// The index of the configuration string, valid when greater than 0. It
    /// can be passed to [`usb_get_string_descr`].
    pub iConfiguration: u8,
    /// The characteristics of the configuration, as in [`usb_cfg_descr`].
    pub bmAttributes: u8,
    /// The maximum power consumption in units of 2 mA.
    pub bMaxPower: u8,
}
/// The typedef name of [`usb_other_speed_cfg_descr`].
pub type usb_other_speed_cfg_descr_t = usb_other_speed_cfg_descr;

// usb_other_speed_cfg_descr: the same layout as usb_cfg_descr, 10 bytes.
const _: () = assert!(core::mem::size_of::<usb_other_speed_cfg_descr>() == 10);
const _: () = assert!(core::mem::offset_of!(usb_other_speed_cfg_descr, bLength) == 0);
const _: () = assert!(core::mem::offset_of!(usb_other_speed_cfg_descr, wTotalLength) == 2);
const _: () = assert!(core::mem::offset_of!(usb_other_speed_cfg_descr, bMaxPower) == 8);

/// `usb_if_descr(9S)`. illumos DDI specific (illumos DDI).
///
/// The interface descriptor, described in section 9.6.5 of the USB 2.0
/// specification. A configuration contains one or more interfaces, and an
/// interface contains one or more endpoints. A client driver finds the
/// parsed interface descriptors in the descriptor tree returned by
/// [`usb_get_dev_data`]. All fields are single bytes, so the layout equals
/// the 9 bytes of the wire format.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct usb_if_descr {
    /// The size of this descriptor in bytes.
    pub bLength: u8,
    /// The descriptor type, set to `USB_DESCR_TYPE_IF`.
    pub bDescriptorType: u8,
    /// The number of the interface, starting at 0.
    pub bInterfaceNumber: u8,
    /// The number of the alternate setting of this interface and its
    /// endpoints, starting at 0.
    pub bAlternateSetting: u8,
    /// The number of endpoints, not counting endpoint 0.
    pub bNumEndpoints: u8,
    /// The class code of the interface, one of the `USB_CLASS_*` values.
    pub bInterfaceClass: u8,
    /// The subclass code, defined by the specification of the interface
    /// class.
    pub bInterfaceSubClass: u8,
    /// The protocol code, defined by the specification of the interface
    /// class.
    pub bInterfaceProtocol: u8,
    /// The index of the optional string that describes the interface, valid
    /// when greater than 0. It can be passed to [`usb_get_string_descr`].
    pub iInterface: u8,
}
/// The typedef name of [`usb_if_descr`].
pub type usb_if_descr_t = usb_if_descr;

// usb_if_descr: nine single bytes, alignment 1, 9 bytes.
const _: () = assert!(core::mem::size_of::<usb_if_descr>() == 9);
const _: () = assert!(core::mem::align_of::<usb_if_descr>() == 1);
const _: () = assert!(core::mem::offset_of!(usb_if_descr, bLength) == 0);
const _: () = assert!(core::mem::offset_of!(usb_if_descr, iInterface) == 8);

/// `usb_ep_descr(9S)`. illumos DDI specific (illumos DDI).
///
/// The endpoint descriptor, described in section 9.6.6 of the USB 2.0
/// specification. An endpoint is a uniquely addressable part of a USB
/// device that is a source or a sink of data. A client driver finds the
/// parsed endpoint descriptors in the descriptor tree returned by
/// [`usb_get_dev_data`], usually through [`usb_lookup_ep_data`]. In the
/// parsed form `wMaxPacketSize` is aligned to two bytes, which adds a byte
/// of tail padding, so the structure is 8 bytes long where the wire format
/// is 7.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct usb_ep_descr {
    /// The size of this descriptor in bytes.
    pub bLength: u8,
    /// The descriptor type, set to `USB_DESCR_TYPE_EP`.
    pub bDescriptorType: u8,
    /// The address of the endpoint. `USB_EP_NUM_MASK` selects the endpoint
    /// number and `USB_EP_DIR_MASK` the direction.
    pub bEndpointAddress: u8,
    /// The attributes of the endpoint. `USB_EP_ATTR_MASK` selects the
    /// transfer type, and for isochronous endpoints `USB_EP_SYNC_MASK` and
    /// `USB_EP_USAGE_MASK` select the synchronisation and feedback type.
    pub bmAttributes: u8,
    /// The maximum packet size. For high speed isochronous and interrupt
    /// endpoints the bits under `USB_EP_MAX_XACTS_MASK` give the number of
    /// additional transactions per microframe.
    pub wMaxPacketSize: u16,
    /// The polling interval of interrupt and isochronous endpoints, or the
    /// NAK rate of high speed control and bulk endpoints.
    pub bInterval: u8,
}
/// The typedef name of [`usb_ep_descr`].
pub type usb_ep_descr_t = usb_ep_descr;

// usb_ep_descr: four bytes from 0 to 3, wMaxPacketSize at 4, bInterval at
// 6, one byte of tail padding for alignment 2, 8 bytes.
const _: () = assert!(core::mem::size_of::<usb_ep_descr>() == 8);
const _: () = assert!(core::mem::align_of::<usb_ep_descr>() == 2);
const _: () = assert!(core::mem::offset_of!(usb_ep_descr, bLength) == 0);
const _: () = assert!(core::mem::offset_of!(usb_ep_descr, wMaxPacketSize) == 4);
const _: () = assert!(core::mem::offset_of!(usb_ep_descr, bInterval) == 6);

/// `usb_string_descr(9S)`. illumos DDI specific (illumos DDI).
///
/// The string descriptor, described in section 9.6.7 of the USB 2.0
/// specification. Other descriptors refer to string descriptors through
/// their index fields. The header declares `bString` as a one element
/// array of bytes that stands for a variable length Unicode string, whereas
/// the manual page shows an array of 16-bit values. The layout follows the
/// header, which gives a 3 byte structure with alignment 1. Drivers
/// normally read strings with [`usb_get_string_descr`], which returns them
/// already converted.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct usb_string_descr {
    /// The size of this descriptor in bytes.
    pub bLength: u8,
    /// The descriptor type, set to `USB_DESCR_TYPE_STRING`.
    pub bDescriptorType: u8,
    /// The first byte of the variable length Unicode string.
    pub bString: [u8; 1],
}
/// The typedef name of [`usb_string_descr`].
pub type usb_string_descr_t = usb_string_descr;

// usb_string_descr: three single bytes, alignment 1, 3 bytes.
const _: () = assert!(core::mem::size_of::<usb_string_descr>() == 3);
const _: () = assert!(core::mem::offset_of!(usb_string_descr, bLength) == 0);
const _: () = assert!(core::mem::offset_of!(usb_string_descr, bString) == 2);

/// `usb_ep_ss_comp_descr(9S)`. illumos DDI Specific.
///
/// The SuperSpeed endpoint companion descriptor, formally defined in section
/// 9.6.7 of the USB 3.1 specification. USB 3.0 and newer devices return it
/// together with each standard endpoint descriptor to describe further
/// attributes of the endpoint; on its own it does not identify an endpoint.
/// It is available through the endpoint data returned by
/// [`usb_lookup_ep_data`], and [`usb_ep_xdescr_fill`] assembles it into the
/// form that [`usb_pipe_xopen`] requires.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct usb_ep_ss_comp_descr {
    /// The size of this descriptor, which is usually six.
    pub bLength: u8,
    /// The descriptor type, `USB_DESCR_TYPE_SS_EP_COMP`, whose value is 0x30.
    pub bDescriptorType: u8,
    /// The maximum number of packets the endpoint can send in one burst,
    /// minus one. Valid values range from 0 to 15, so a value of 0 means one
    /// packet and a value of 15 means sixteen.
    pub bMaxBurst: u8,
    /// Further attributes of the endpoint. The member is reserved and zero
    /// for control and interrupt endpoints. For a bulk endpoint bits 4 to 0
    /// give the maximum number of streams as a power of two, where 0 means
    /// that streams are not supported. For an isochronous endpoint bits 1 to
    /// 0, selected by `USB_EP_SS_COMP_ISOC_MULT_MASK`, give the Mult value
    /// used to compute the maximum number of packets per service interval.
    pub bmAttributes: u8,
    /// The total number of bytes the endpoint can transfer in one service
    /// interval. It is only valid for periodic endpoints, that is for
    /// isochronous and interrupt IN endpoints.
    pub wBytesPerInterval: u16,
}
/// The typedef name of [`usb_ep_ss_comp_descr`].
pub type usb_ep_ss_comp_descr_t = usb_ep_ss_comp_descr;

// usb_ep_ss_comp_descr: four bytes from 0 to 3, wBytesPerInterval at 4,
// alignment 2, 6 bytes.
const _: () = assert!(core::mem::size_of::<usb_ep_ss_comp_descr>() == 6);
const _: () = assert!(core::mem::align_of::<usb_ep_ss_comp_descr>() == 2);
const _: () = assert!(core::mem::offset_of!(usb_ep_ss_comp_descr, bLength) == 0);
const _: () = assert!(core::mem::offset_of!(usb_ep_ss_comp_descr, wBytesPerInterval) == 4);

/// `usb_ep_xdescr(9S)`. illumos DDI Specific.
///
/// The extended endpoint descriptor, a versioned structure that holds the
/// standard endpoint descriptor together with the companion descriptors that
/// newer revisions of the USB specification added. It spares client drivers
/// from changing whenever another companion descriptor appears. After
/// looking up the endpoint data with [`usb_lookup_ep_data`], a driver calls
/// [`usb_ep_xdescr_fill`] to fill in this structure and then passes it to
/// [`usb_pipe_xopen`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_ep_xdescr {
    /// The version of the structure, set to the value the driver passed to
    /// [`usb_ep_xdescr_fill`]. Drivers should ignore this member and must
    /// not modify it.
    pub uex_version: uint_t,
    /// Flags that tell which of the other members are present and valid.
    /// A driver must check them before it accesses any member other than
    /// `uex_ep`.
    pub uex_flags: usb_ep_xdescr_flags_t,
    /// The standard endpoint descriptor. It is always valid and has no flag.
    pub uex_ep: usb_ep_descr_t,
    /// The SuperSpeed endpoint companion descriptor. It is only valid when
    /// `USB_EP_XFLAGS_SS_COMP` is set in `uex_flags`.
    pub uex_ep_ss: usb_ep_ss_comp_descr_t,
}
/// The typedef name of [`usb_ep_xdescr`].
pub type usb_ep_xdescr_t = usb_ep_xdescr;

// usb_ep_xdescr: uex_version at 0, uex_flags at 4, uex_ep (8 bytes,
// alignment 2) at 8, uex_ep_ss (6 bytes, alignment 2) at 16 ending at 22,
// padded to the alignment of 4, 24 bytes.
const _: () = assert!(core::mem::size_of::<usb_ep_xdescr>() == 24);
const _: () = assert!(core::mem::align_of::<usb_ep_xdescr>() == 4);
const _: () = assert!(core::mem::offset_of!(usb_ep_xdescr, uex_version) == 0);
const _: () = assert!(core::mem::offset_of!(usb_ep_xdescr, uex_ep) == 8);
const _: () = assert!(core::mem::offset_of!(usb_ep_xdescr, uex_ep_ss) == 16);

/// The first version of the extended endpoint descriptor
/// (`USB_EP_XDESCR_VERSION_ONE`).
pub const USB_EP_XDESCR_VERSION_ONE: uint_t = 1;
/// The version of the extended endpoint descriptor that drivers should
/// always pass to [`usb_ep_xdescr_fill`] (`USB_EP_XDESCR_CURRENT_VERSION`).
pub const USB_EP_XDESCR_CURRENT_VERSION: uint_t = USB_EP_XDESCR_VERSION_ONE;

// --- Descriptor field values -------------------------------------------------

/// The index of the default configuration of a device with several
/// configurations. The index is not the configuration value
/// (`USB_DEV_DEFAULT_CONFIG_INDEX`).
pub const USB_DEV_DEFAULT_CONFIG_INDEX: uint_t = 0;
/// The configuration is self powered rather than using bus power
/// (`USB_CFG_ATTR_SELFPWR`).
pub const USB_CFG_ATTR_SELFPWR: u8 = 0x40;
/// The configuration supports remote wakeup (`USB_CFG_ATTR_REMOTE_WAKEUP`).
pub const USB_CFG_ATTR_REMOTE_WAKEUP: u8 = 0x20;
/// The configuration is battery powered (`USB_CFG_ATTR_BAT_PWR`).
pub const USB_CFG_ATTR_BAT_PWR: u8 = 0x10;

/// The mask of the endpoint number bits in `bEndpointAddress`
/// (`USB_EP_NUM_MASK`).
pub const USB_EP_NUM_MASK: u8 = 0x0F;
/// The mask of the direction bit in `bEndpointAddress` (`USB_EP_DIR_MASK`).
pub const USB_EP_DIR_MASK: u8 = 0x80;
/// The endpoint direction towards the device (`USB_EP_DIR_OUT`).
pub const USB_EP_DIR_OUT: u8 = 0x00;
/// The endpoint direction towards the host (`USB_EP_DIR_IN`).
pub const USB_EP_DIR_IN: u8 = 0x80;
/// The mask of the transfer type bits in `bmAttributes`
/// (`USB_EP_ATTR_MASK`).
pub const USB_EP_ATTR_MASK: u8 = 0x03;
/// The endpoint supports control transfers (`USB_EP_ATTR_CONTROL`).
pub const USB_EP_ATTR_CONTROL: u8 = 0x00;
/// The endpoint supports isochronous transfers (`USB_EP_ATTR_ISOCH`).
pub const USB_EP_ATTR_ISOCH: u8 = 0x01;
/// The endpoint supports bulk transfers (`USB_EP_ATTR_BULK`).
pub const USB_EP_ATTR_BULK: u8 = 0x02;
/// The endpoint supports interrupt transfers (`USB_EP_ATTR_INTR`).
pub const USB_EP_ATTR_INTR: u8 = 0x03;
/// The mask of the synchronisation type bits of an isochronous endpoint
/// (`USB_EP_SYNC_MASK`).
pub const USB_EP_SYNC_MASK: u8 = 0x0C;
/// The isochronous endpoint uses no synchronisation (`USB_EP_SYNC_NONE`).
pub const USB_EP_SYNC_NONE: u8 = 0x00;
/// The isochronous endpoint is asynchronous (`USB_EP_SYNC_ASYNC`).
pub const USB_EP_SYNC_ASYNC: u8 = 0x04;
/// The isochronous endpoint is adaptive (`USB_EP_SYNC_ADPT`).
pub const USB_EP_SYNC_ADPT: u8 = 0x08;
/// The isochronous endpoint is synchronous (`USB_EP_SYNC_SYNC`).
pub const USB_EP_SYNC_SYNC: u8 = 0x0C;
/// The mask of the feedback type bits of an isochronous endpoint
/// (`USB_EP_USAGE_MASK`).
pub const USB_EP_USAGE_MASK: u8 = 0x30;
/// The isochronous endpoint is a data endpoint (`USB_EP_USAGE_DATA`).
pub const USB_EP_USAGE_DATA: u8 = 0x00;
/// The isochronous endpoint is a feedback endpoint (`USB_EP_USAGE_FEED`).
pub const USB_EP_USAGE_FEED: u8 = 0x10;
/// The isochronous endpoint is an implicit feedback data endpoint
/// (`USB_EP_USAGE_IMPL`).
pub const USB_EP_USAGE_IMPL: u8 = 0x20;
/// The mask of the packet size bits in `wMaxPacketSize`
/// (`USB_EP_MAX_PKTSZ_MASK`).
pub const USB_EP_MAX_PKTSZ_MASK: u16 = 0x07FF;
/// The mask of the bits in `wMaxPacketSize` that give the number of
/// additional transactions per microframe of a high speed isochronous or
/// interrupt endpoint (`USB_EP_MAX_XACTS_MASK`).
pub const USB_EP_MAX_XACTS_MASK: u16 = 0x1800;
/// The shift that moves the additional transaction bits of `wMaxPacketSize`
/// to the lowest bits (`USB_EP_MAX_XACTS_SHIFT`).
pub const USB_EP_MAX_XACTS_SHIFT: c_uint = 11;
/// The smallest NAK rate of a high speed control endpoint
/// (`USB_EP_MIN_HIGH_CONTROL_INTRVL`).
pub const USB_EP_MIN_HIGH_CONTROL_INTRVL: u8 = 0;
/// The largest NAK rate of a high speed control endpoint
/// (`USB_EP_MAX_HIGH_CONTROL_INTRVL`).
pub const USB_EP_MAX_HIGH_CONTROL_INTRVL: u8 = 255;
/// The smallest NAK rate of a high speed bulk endpoint
/// (`USB_EP_MIN_HIGH_BULK_INTRVL`).
pub const USB_EP_MIN_HIGH_BULK_INTRVL: u8 = 0;
/// The largest NAK rate of a high speed bulk endpoint
/// (`USB_EP_MAX_HIGH_BULK_INTRVL`).
pub const USB_EP_MAX_HIGH_BULK_INTRVL: u8 = 255;
/// The smallest polling interval of a low speed interrupt endpoint
/// (`USB_EP_MIN_LOW_INTR_INTRVL`).
pub const USB_EP_MIN_LOW_INTR_INTRVL: u8 = 1;
/// The largest polling interval of a low speed interrupt endpoint
/// (`USB_EP_MAX_LOW_INTR_INTRVL`).
pub const USB_EP_MAX_LOW_INTR_INTRVL: u8 = 255;
/// The smallest polling interval of a full speed interrupt endpoint
/// (`USB_EP_MIN_FULL_INTR_INTRVL`).
pub const USB_EP_MIN_FULL_INTR_INTRVL: u8 = 1;
/// The largest polling interval of a full speed interrupt endpoint
/// (`USB_EP_MAX_FULL_INTR_INTRVL`).
pub const USB_EP_MAX_FULL_INTR_INTRVL: u8 = 255;
/// The smallest polling interval of a high speed interrupt endpoint. The
/// interval is two to the power of the value minus one
/// (`USB_EP_MIN_HIGH_INTR_INTRVL`).
pub const USB_EP_MIN_HIGH_INTR_INTRVL: u8 = 1;
/// The largest polling interval of a high speed interrupt endpoint, again
/// as an exponent (`USB_EP_MAX_HIGH_INTR_INTRVL`).
pub const USB_EP_MAX_HIGH_INTR_INTRVL: u8 = 16;
/// The smallest polling interval of a full speed isochronous endpoint, as an
/// exponent (`USB_EP_MIN_FULL_ISOCH_INTRVL`).
pub const USB_EP_MIN_FULL_ISOCH_INTRVL: u8 = 1;
/// The largest polling interval of a full speed isochronous endpoint, as an
/// exponent (`USB_EP_MAX_FULL_ISOCH_INTRVL`).
pub const USB_EP_MAX_FULL_ISOCH_INTRVL: u8 = 16;
/// The smallest polling interval of a high speed isochronous endpoint, as an
/// exponent (`USB_EP_MIN_HIGH_ISOCH_INTRVL`).
pub const USB_EP_MIN_HIGH_ISOCH_INTRVL: u8 = 1;
/// The largest polling interval of a high speed isochronous endpoint, as an
/// exponent (`USB_EP_MAX_HIGH_ISOCH_INTRVL`).
pub const USB_EP_MAX_HIGH_ISOCH_INTRVL: u8 = 16;
/// The maximum length of a string descriptor, and of the strings the
/// framework returns in the descriptor tree (`USB_MAXSTRINGLEN`).
pub const USB_MAXSTRINGLEN: c_int = 255;
/// The mask of the Mult bits in `bmAttributes` of the SuperSpeed companion
/// descriptor of an isochronous endpoint (`USB_EP_SS_COMP_ISOC_MULT_MASK`).
pub const USB_EP_SS_COMP_ISOC_MULT_MASK: u8 = 0x03;

// --- Descriptor tree -----------------------------------------------------------

/// `usb_client_dev_data(9S)`. illumos DDI specific (illumos DDI).
///
/// A configuration node of the descriptor tree, `usb_cfg_data_t`. The root
/// of the tree, `dev_cfg` in [`usb_client_dev_data`], is an array of these
/// nodes, one per configuration. The array index does not correspond to
/// the configuration value, which is found in the `bConfigurationValue`
/// field of `cfg_descr`. None of the arrays in the tree is NULL terminated.
#[repr(C)]
#[derive(Debug)]
pub struct usb_cfg_data {
    /// The parsed configuration descriptor.
    pub cfg_descr: usb_cfg_descr,
    /// The interfaces of this configuration, an array of `cfg_n_if`
    /// elements indexed by interface number. With `USB_PARSE_LVL_IF` only
    /// the interface that belongs to the current device node is present.
    pub cfg_if: *mut usb_if_data,
    /// The class and vendor specific descriptors that modify or extend the
    /// configuration, an array of `cfg_n_cvs` elements.
    pub cfg_cvs: *mut usb_cvs_data,
    /// The string that describes the configuration, in English ASCII.
    pub cfg_str: *mut c_char,
    /// The number of elements in `cfg_if`.
    pub cfg_n_if: uint_t,
    /// The number of elements in `cfg_cvs`.
    pub cfg_n_cvs: uint_t,
    /// The size of the string in `cfg_str`.
    pub cfg_strsize: uint_t,
}
/// The typedef name of [`usb_cfg_data`].
pub type usb_cfg_data_t = usb_cfg_data;

// usb_cfg_data: cfg_descr (10 bytes, alignment 2) at 0, padded to 16 for
// cfg_if, cfg_cvs at 24, cfg_str at 32, three uint_t at 40, 44 and 48,
// padded to the pointer alignment, 56 bytes.
const _: () = assert!(core::mem::size_of::<usb_cfg_data>() == 56);
const _: () = assert!(core::mem::offset_of!(usb_cfg_data, cfg_descr) == 0);
const _: () = assert!(core::mem::offset_of!(usb_cfg_data, cfg_if) == 16);
const _: () = assert!(core::mem::offset_of!(usb_cfg_data, cfg_str) == 32);
const _: () = assert!(core::mem::offset_of!(usb_cfg_data, cfg_strsize) == 48);

/// `usb_client_dev_data(9S)`. illumos DDI specific (illumos DDI).
///
/// An interface node of the descriptor tree, `usb_if_data_t`. An interface
/// can present itself in one of several alternate ways, and the node holds
/// the array of those alternates.
#[repr(C)]
#[derive(Debug)]
pub struct usb_if_data {
    /// The alternates of the interface, a sparse array of `if_n_alt`
    /// elements indexed by alternate setting.
    pub if_alt: *mut usb_alt_if_data,
    /// The number of elements in `if_alt`.
    pub if_n_alt: uint_t,
}
/// The typedef name of [`usb_if_data`].
pub type usb_if_data_t = usb_if_data;

// usb_if_data: if_alt at 0, if_n_alt at 8, padded to 16 bytes.
const _: () = assert!(core::mem::size_of::<usb_if_data>() == 16);
const _: () = assert!(core::mem::offset_of!(usb_if_data, if_alt) == 0);
const _: () = assert!(core::mem::offset_of!(usb_if_data, if_n_alt) == 8);

/// `usb_client_dev_data(9S)`. illumos DDI specific (illumos DDI).
///
/// An alternate interface node of the descriptor tree, `usb_alt_if_data_t`.
/// It holds the interface descriptor of the alternate, its string, its
/// endpoints and the class and vendor specific descriptors bound to it.
#[repr(C)]
#[derive(Debug)]
pub struct usb_alt_if_data {
    /// The parsed interface descriptor of this alternate.
    pub altif_descr: usb_if_descr_t,
    /// The endpoints of the alternate, an array of `altif_n_ep` elements.
    /// Unlike the interface and alternate arrays it is not sparse.
    pub altif_ep: *mut usb_ep_data,
    /// The class and vendor specific descriptors of this alternate, an
    /// array of `altif_n_cvs` elements.
    pub altif_cvs: *mut usb_cvs_data,
    /// The string that describes the alternate, in English ASCII.
    pub altif_str: *mut c_char,
    /// The number of elements in `altif_ep`.
    pub altif_n_ep: uint_t,
    /// The number of elements in `altif_cvs`.
    pub altif_n_cvs: uint_t,
    /// The size of the string in `altif_str`.
    pub altif_strsize: uint_t,
}
/// The typedef name of [`usb_alt_if_data`].
pub type usb_alt_if_data_t = usb_alt_if_data;

// usb_alt_if_data: altif_descr (9 bytes, alignment 1) at 0, padded to 16
// for altif_ep, altif_cvs at 24, altif_str at 32, three uint_t at 40, 44
// and 48, padded to 56 bytes.
const _: () = assert!(core::mem::size_of::<usb_alt_if_data>() == 56);
const _: () = assert!(core::mem::offset_of!(usb_alt_if_data, altif_descr) == 0);
const _: () = assert!(core::mem::offset_of!(usb_alt_if_data, altif_ep) == 16);
const _: () = assert!(core::mem::offset_of!(usb_alt_if_data, altif_str) == 32);
const _: () = assert!(core::mem::offset_of!(usb_alt_if_data, altif_strsize) == 48);

/// `usb_client_dev_data(9S)`. illumos DDI specific (illumos DDI).
///
/// An endpoint node of the descriptor tree, `usb_ep_data_t`. It holds the
/// endpoint descriptor, the SuperSpeed companion descriptor when the device
/// provided one, and the class and vendor specific descriptors of the
/// endpoint. [`usb_lookup_ep_data`] returns a pointer to such a node, and
/// [`usb_ep_xdescr_fill`] turns it into the descriptor that
/// [`usb_pipe_xopen`] takes.
#[repr(C)]
#[derive(Debug)]
pub struct usb_ep_data {
    /// The parsed endpoint descriptor.
    pub ep_descr: usb_ep_descr_t,
    /// The class and vendor specific descriptors that modify or extend this
    /// endpoint, an array of `ep_n_cvs` elements.
    pub ep_cvs: *mut usb_cvs_data,
    /// The number of elements in `ep_cvs`.
    pub ep_n_cvs: uint_t,
    /// Whether `ep_ss_comp` holds a valid SuperSpeed companion descriptor.
    pub ep_ss_valid: boolean_t,
    /// The SuperSpeed endpoint companion descriptor, valid when
    /// `ep_ss_valid` is true.
    pub ep_ss_comp: usb_ep_ss_comp_descr_t,
}
/// The typedef name of [`usb_ep_data`].
pub type usb_ep_data_t = usb_ep_data;

// usb_ep_data: ep_descr (8 bytes) at 0, ep_cvs at 8, ep_n_cvs at 16,
// ep_ss_valid at 20, ep_ss_comp (6 bytes, alignment 2) at 24 ending at 30,
// padded to the pointer alignment, 32 bytes.
const _: () = assert!(core::mem::size_of::<usb_ep_data>() == 32);
const _: () = assert!(core::mem::offset_of!(usb_ep_data, ep_descr) == 0);
const _: () = assert!(core::mem::offset_of!(usb_ep_data, ep_cvs) == 8);
const _: () = assert!(core::mem::offset_of!(usb_ep_data, ep_ss_valid) == 20);
const _: () = assert!(core::mem::offset_of!(usb_ep_data, ep_ss_comp) == 24);

/// `usb_client_dev_data(9S)`. illumos DDI specific (illumos DDI).
///
/// A class or vendor specific descriptor node of the descriptor tree,
/// `usb_cvs_data_t`. Such descriptors are grouped with the configuration,
/// interface or endpoint descriptor they immediately follow in the raw data
/// returned by the device. The node keeps the raw bytes, which a driver can
/// convert with [`usb_parse_data`].
#[repr(C)]
#[derive(Debug)]
pub struct usb_cvs_data {
    /// The raw data of the descriptor.
    pub cvs_buf: *mut c_uchar,
    /// The length of `cvs_buf` in bytes.
    pub cvs_buf_len: uint_t,
}
/// The typedef name of [`usb_cvs_data`].
pub type usb_cvs_data_t = usb_cvs_data;

// usb_cvs_data: cvs_buf at 0, cvs_buf_len at 8, padded to 16 bytes.
const _: () = assert!(core::mem::size_of::<usb_cvs_data>() == 16);
const _: () = assert!(core::mem::offset_of!(usb_cvs_data, cvs_buf) == 0);
const _: () = assert!(core::mem::offset_of!(usb_cvs_data, cvs_buf_len) == 8);

/// `usb_client_dev_data(9S)`. illumos DDI specific (illumos DDI).
///
/// The device configuration information that [`usb_get_dev_data`] returns
/// to a client driver, `usb_client_dev_data_t`. Most USBA functions need
/// information from it. Besides the handle of the default control pipe and
/// the interrupt cookie for mutexes, it holds the parsed device descriptor,
/// the identification strings of the device and the tree of parsed
/// descriptors. The descriptors in the tree have been adjusted to the host,
/// which may involve converting the byte order and adding padding. Each
/// client receives its own copy, although the default control pipe may be
/// shared among the drivers of a composite device. The structure is freed
/// with [`usb_free_dev_data`] or by passing it to [`usb_client_detach`].
#[repr(C)]
#[derive(Debug)]
pub struct usb_client_dev_data {
    /// The handle of the default control pipe, used mainly for control
    /// commands and device setup. A client driver cannot open or close this
    /// pipe itself.
    pub dev_default_ph: usb_pipe_handle_t,
    /// The interrupt block cookie for initialising mutexes that are used in
    /// callbacks running in interrupt context, that is callbacks that see
    /// `USB_CB_INTR_CONTEXT` in their flags. It is used in place of a cookie
    /// from `ddi_get_iblock_cookie(9F)`.
    pub dev_iblock_cookie: ddi_iblock_cookie_t,
    /// The parsed standard device descriptor, used to identify the type of
    /// the device.
    pub dev_descr: *mut usb_dev_descr,
    /// The manufacturer string of the device.
    pub dev_mfg: *mut c_char,
    /// The product string of the device.
    pub dev_product: *mut c_char,
    /// The serial number string of the device.
    pub dev_serial: *mut c_char,
    /// The parse level that the returned tree actually reflects. It can
    /// differ from the level requested from [`usb_get_dev_data`].
    pub dev_parse_level: usb_reg_parse_lvl_t,
    /// The root of the descriptor tree, an array of `dev_n_cfg`
    /// configuration nodes indexed by configuration index. With
    /// `USB_PARSE_LVL_CFG` or `USB_PARSE_LVL_IF` the one configuration that
    /// the system recognises is at index `USB_DEV_DEFAULT_CONFIG_INDEX`.
    pub dev_cfg: *mut usb_cfg_data,
    /// The number of configurations in `dev_cfg`. It may differ from the
    /// number of configurations in the device descriptor.
    pub dev_n_cfg: uint_t,
    /// The configuration node that was active when [`usb_get_dev_data`] was
    /// called.
    pub dev_curr_cfg: *mut usb_cfg_data,
    /// The first active interface in the tree that is under the control of
    /// the driver. It is always 0 when the driver controls the whole device.
    pub dev_curr_if: c_int,
}
/// The typedef name of [`usb_client_dev_data`].
pub type usb_client_dev_data_t = usb_client_dev_data;

// usb_client_dev_data: six pointers from 0 to 40, dev_parse_level at 48,
// dev_cfg at 56, dev_n_cfg at 64, dev_curr_cfg at 72, dev_curr_if at 80,
// padded to 88 bytes.
const _: () = assert!(core::mem::size_of::<usb_client_dev_data>() == 88);
const _: () = assert!(core::mem::offset_of!(usb_client_dev_data, dev_default_ph) == 0);
const _: () = assert!(core::mem::offset_of!(usb_client_dev_data, dev_iblock_cookie) == 8);
const _: () = assert!(core::mem::offset_of!(usb_client_dev_data, dev_parse_level) == 48);
const _: () = assert!(core::mem::offset_of!(usb_client_dev_data, dev_cfg) == 56);
const _: () = assert!(core::mem::offset_of!(usb_client_dev_data, dev_curr_cfg) == 72);
const _: () = assert!(core::mem::offset_of!(usb_client_dev_data, dev_curr_if) == 80);

/// The value that [`usb_parse_data`] returns when it fails, which is zero
/// bytes parsed (`USB_PARSE_ERROR`).
pub const USB_PARSE_ERROR: size_t = 0;
/// The language ID of US English, which is currently the only language ID
/// that [`usb_get_string_descr`] accepts. It is also the `wIndex` of
/// string descriptor requests (`USB_LANG_ID`).
pub const USB_LANG_ID: u16 = 0x0409;
/// The value [`usb_get_if_number`] returns when the driver is responsible
/// for the entire active configuration but its device node does not
/// correspond to an entire physical device (`USB_COMBINED_NODE`).
pub const USB_COMBINED_NODE: c_int = -1;
/// The value [`usb_get_if_number`] returns when the device node of the
/// driver corresponds to an entire physical device (`USB_DEVICE_NODE`).
pub const USB_DEVICE_NODE: c_int = -2;

// --- Pipes ----------------------------------------------------------------------

/// `usb_pipe_xopen(9F)`. illumos DDI specific (illumos DDI).
///
/// The pipe policy, `usb_pipe_policy_t`, a hint on how a pipe will be used
/// that must be passed to [`usb_pipe_xopen`] and [`usb_pipe_open`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct usb_pipe_policy {
    /// A hint of how many asynchronous operations that need their own kernel
    /// thread will be in progress on the pipe at the same time, that is the
    /// highest number of threads ever needed at once. Allow at least one for
    /// synchronous callback handling and as many as the expected parallelism
    /// of asynchronous calls to [`usb_pipe_close`], [`usb_set_cfg`],
    /// [`usb_set_alt_if`], [`usb_clr_feature`], [`usb_pipe_reset`],
    /// [`usb_pipe_drain_reqs`], [`usb_pipe_stop_intr_polling`] and
    /// [`usb_pipe_stop_isoc_polling`] requires. A value that is too small can
    /// deadlock the pipe.
    pub pp_max_async_reqs: c_uchar,
}
/// The typedef name of [`usb_pipe_policy`].
pub type usb_pipe_policy_t = usb_pipe_policy;

// usb_pipe_policy: a single byte.
const _: () = assert!(core::mem::size_of::<usb_pipe_policy>() == 1);
const _: () = assert!(core::mem::offset_of!(usb_pipe_policy, pp_max_async_reqs) == 0);

/// `usb_callback_flags(9S)`. illumos DDI specific (illumos DDI).
///
/// The command completion callback of [`usb_pipe_close`],
/// [`usb_pipe_reset`], [`usb_pipe_drain_reqs`], [`usb_set_cfg`],
/// [`usb_set_alt_if`] and [`usb_clr_feature`]. C spells the type out in each
/// prototype and gives it no name. The framework calls it when a command
/// issued without `USB_FLAGS_SLEEP` completes. `ph` is the pipe the command
/// was performed on, which is the default control pipe for the
/// configuration and feature commands, and the callback can use it to find
/// the private data of the pipe. `arg` is the callback argument passed with
/// the command. `rval` is the return status of the command, roughly the
/// equivalent of the completion reason of a transfer. `flags` gives further
/// information on how the command was executed and is worth checking when
/// `rval` indicates a failure. If `USB_CB_ASYNC_REQ_FAILED` is clear in
/// `flags` the callback runs in kernel context and may block; otherwise it
/// must not block.
pub type usb_cmd_completion_cb_t =
    unsafe extern "C" fn(ph: usb_pipe_handle_t, arg: usb_opaque_t, rval: c_int, flags: usb_cb_flags_t);

// --- Control requests --------------------------------------------------------

/// `usb_ctrl_req(9S)`. illumos DDI specific (illumos DDI).
///
/// The normal and exception callback of a control request. The framework
/// passes the pipe handle and the completed request, whose completion
/// reason and callback flags tell what happened. The callback owns the
/// request and has to free it. The rules on when a callback may block are
/// given on the `usb_callback_flags(9S)` page.
pub type usb_ctrl_req_cb_t = unsafe extern "C" fn(ph: usb_pipe_handle_t, req: *mut usb_ctrl_req);

/// `usb_ctrl_req(9S)`. illumos DDI specific (illumos DDI).
///
/// A control request, used to send commands to a device and to read its
/// status. Control pipes are described in section 5.5 of the USB 2.0
/// specification, and the format of the requests in section 9.3. A client
/// driver allocates the request with [`usb_alloc_ctrl_req`] and submits it
/// with [`usb_pipe_ctrl_xfer`]. The direction of the transfer follows from
/// `ctrl_bmRequestType`. All fields are set by the client except the data of
/// an IN request, which the client is responsible for freeing. Requests may
/// be reused, in which case the client has to reinitialise fields such as
/// the read and write pointers of the data. Control requests can be queued.
#[repr(C)]
#[derive(Debug)]
pub struct usb_ctrl_req {
    /// The characteristics of the request, a combination of one
    /// `USB_DEV_REQ_*` direction, one request type and one recipient.
    pub ctrl_bmRequestType: u8,
    /// The specific request, for example one of the standard `USB_REQ_*`
    /// requests.
    pub ctrl_bRequest: u8,
    /// A value that varies according to the request.
    pub ctrl_wValue: u16,
    /// An index or offset that varies according to the request.
    pub ctrl_wIndex: u16,
    /// The number of bytes to transfer in the data phase.
    pub ctrl_wLength: u16,
    /// The data of the data phase. It is allocated by the client for OUT
    /// requests and by the framework for IN requests.
    pub ctrl_data: *mut mblk_t,
    /// The time in seconds after which the framework retires the request. A
    /// value of 0 selects the default of 5 seconds.
    pub ctrl_timeout: uint_t,
    /// Private data of the client, which the framework does not touch.
    pub ctrl_client_private: usb_opaque_t,
    /// The attributes of this request. `USB_ATTRS_SHORT_XFER_OK`,
    /// `USB_ATTRS_AUTOCLEARING` and `USB_ATTRS_PIPE_RESET` are valid for
    /// control requests.
    pub ctrl_attributes: usb_req_attrs_t,
    /// The callback for normal completion, called when the request completes
    /// successfully. It may be NULL for synchronous requests.
    pub ctrl_cb: Option<usb_ctrl_req_cb_t>,
    /// The callback for exceptional completion, called when the request
    /// fails or when less data was transferred than requested without
    /// `USB_ATTRS_SHORT_XFER_OK`. With autoclearing, recovery is attempted
    /// first and the callback flags tell what was done.
    pub ctrl_exc_cb: Option<usb_ctrl_req_cb_t>,
    /// The overall completion status, set by the host controller driver.
    pub ctrl_completion_reason: usb_cr_t,
    /// The callback context and the recovery actions taken, set by the
    /// framework.
    pub ctrl_cb_flags: usb_cb_flags_t,
}
/// The typedef name of [`usb_ctrl_req`].
pub type usb_ctrl_req_t = usb_ctrl_req;

// usb_ctrl_req: two bytes at 0 and 1, three u16 at 2, 4 and 6, ctrl_data at
// 8, ctrl_timeout at 16, ctrl_client_private at 24, ctrl_attributes at 32,
// ctrl_cb at 40, ctrl_exc_cb at 48, ctrl_completion_reason at 56,
// ctrl_cb_flags at 60, 64 bytes.
const _: () = assert!(core::mem::size_of::<usb_ctrl_req>() == 64);
const _: () = assert!(core::mem::offset_of!(usb_ctrl_req, ctrl_bmRequestType) == 0);
const _: () = assert!(core::mem::offset_of!(usb_ctrl_req, ctrl_wLength) == 6);
const _: () = assert!(core::mem::offset_of!(usb_ctrl_req, ctrl_data) == 8);
const _: () = assert!(core::mem::offset_of!(usb_ctrl_req, ctrl_client_private) == 24);
const _: () = assert!(core::mem::offset_of!(usb_ctrl_req, ctrl_cb) == 40);
const _: () = assert!(core::mem::offset_of!(usb_ctrl_req, ctrl_cb_flags) == 60);

/// `usb_pipe_ctrl_xfer(9F)`. illumos DDI specific (illumos DDI).
///
/// The setup parameters of [`usb_pipe_ctrl_xfer_wait`], `usb_ctrl_setup_t`.
/// The fields correspond to the fields of the same name in
/// [`usb_ctrl_req`], and are described in section 9.3 of the USB 2.0
/// specification.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_ctrl_setup {
    /// The characteristics of the request, one direction, one request type
    /// and one recipient from the `USB_DEV_REQ_*` constants combined.
    pub bmRequestType: c_uchar,
    /// The request or command.
    pub bRequest: c_uchar,
    /// A value that varies according to the command.
    pub wValue: u16,
    /// A value that varies according to the command, typically an index or
    /// offset.
    pub wIndex: u16,
    /// The number of data bytes to transfer with the command, the same as
    /// the size of the data message block.
    pub wLength: u16,
    /// The attributes the request requires. `USB_ATTRS_AUTOCLEARING` is
    /// always enabled by the function.
    pub attrs: usb_req_attrs_t,
}
/// The typedef name of [`usb_ctrl_setup`].
pub type usb_ctrl_setup_t = usb_ctrl_setup;

// usb_ctrl_setup: two bytes at 0 and 1, three u16 at 2, 4 and 6, attrs at
// 8, alignment 4, 12 bytes.
const _: () = assert!(core::mem::size_of::<usb_ctrl_setup>() == 12);
const _: () = assert!(core::mem::offset_of!(usb_ctrl_setup, bmRequestType) == 0);
const _: () = assert!(core::mem::offset_of!(usb_ctrl_setup, attrs) == 8);

/// The descriptor type of a device descriptor, placed in the high byte of
/// `wValue` of a setup packet (`USB_DESCR_TYPE_SETUP_DEV`).
pub const USB_DESCR_TYPE_SETUP_DEV: u16 = 0x0100;
/// The setup packet form of the configuration descriptor type
/// (`USB_DESCR_TYPE_SETUP_CFG`).
pub const USB_DESCR_TYPE_SETUP_CFG: u16 = 0x0200;
/// The setup packet form of the string descriptor type
/// (`USB_DESCR_TYPE_SETUP_STRING`).
pub const USB_DESCR_TYPE_SETUP_STRING: u16 = 0x0300;
/// The setup packet form of the interface descriptor type
/// (`USB_DESCR_TYPE_SETUP_IF`).
pub const USB_DESCR_TYPE_SETUP_IF: u16 = 0x0400;
/// The setup packet form of the endpoint descriptor type
/// (`USB_DESCR_TYPE_SETUP_EP`).
pub const USB_DESCR_TYPE_SETUP_EP: u16 = 0x0500;
/// The setup packet form of the device qualifier descriptor type
/// (`USB_DESCR_TYPE_SETUP_DEV_QLF`).
pub const USB_DESCR_TYPE_SETUP_DEV_QLF: u16 = 0x0600;
/// The setup packet form of the other speed configuration descriptor type
/// (`USB_DESCR_TYPE_SETUP_OTHER_SPEED_CFG`).
pub const USB_DESCR_TYPE_SETUP_OTHER_SPEED_CFG: u16 = 0x0700;
/// The setup packet form of the interface power descriptor type
/// (`USB_DESCR_TYPE_SETUP_IF_PWR`).
pub const USB_DESCR_TYPE_SETUP_IF_PWR: u16 = 0x0800;
/// The descriptor type of a device descriptor (`USB_DESCR_TYPE_DEV`).
pub const USB_DESCR_TYPE_DEV: u8 = 0x01;
/// The descriptor type of a configuration descriptor (`USB_DESCR_TYPE_CFG`).
pub const USB_DESCR_TYPE_CFG: u8 = 0x02;
/// The descriptor type of a string descriptor (`USB_DESCR_TYPE_STRING`).
pub const USB_DESCR_TYPE_STRING: u8 = 0x03;
/// The descriptor type of an interface descriptor (`USB_DESCR_TYPE_IF`).
pub const USB_DESCR_TYPE_IF: u8 = 0x04;
/// The descriptor type of an endpoint descriptor (`USB_DESCR_TYPE_EP`).
pub const USB_DESCR_TYPE_EP: u8 = 0x05;
/// The descriptor type of a device qualifier descriptor
/// (`USB_DESCR_TYPE_DEV_QLF`).
pub const USB_DESCR_TYPE_DEV_QLF: u8 = 0x06;
/// The descriptor type of an other speed configuration descriptor
/// (`USB_DESCR_TYPE_OTHER_SPEED_CFG`).
pub const USB_DESCR_TYPE_OTHER_SPEED_CFG: u8 = 0x07;
/// The descriptor type of an interface power descriptor
/// (`USB_DESCR_TYPE_IF_PWR`).
pub const USB_DESCR_TYPE_IF_PWR: u8 = 0x08;
/// The descriptor type of an interface association descriptor
/// (`USB_DESCR_TYPE_IA`).
pub const USB_DESCR_TYPE_IA: u8 = 0x0B;
/// The descriptor type of a Binary Object Store descriptor
/// (`USB_DESCR_TYPE_BOS`).
pub const USB_DESCR_TYPE_BOS: u8 = 0x0f;
/// The descriptor type of a device capability descriptor inside a Binary
/// Object Store (`USB_DESCR_TYPE_DEV_CAPABILITY`).
pub const USB_DESCR_TYPE_DEV_CAPABILITY: u8 = 0x10;
/// The descriptor type of the hub class descriptor (`USB_DESCR_TYPE_HUB`).
pub const USB_DESCR_TYPE_HUB: u8 = 0x29;
/// The descriptor type of the SuperSpeed hub class descriptor
/// (`USB_DESCR_TYPE_SS_HUB`).
pub const USB_DESCR_TYPE_SS_HUB: u8 = 0x2A;
/// The descriptor type of the SuperSpeed endpoint companion descriptor
/// (`USB_DESCR_TYPE_SS_EP_COMP`).
pub const USB_DESCR_TYPE_SS_EP_COMP: u8 = 0x30;
/// The descriptor type of the SuperSpeedPlus isochronous endpoint companion
/// descriptor (`USB_DESCR_TYPE_SS_ISO_EP_COMP`).
pub const USB_DESCR_TYPE_SS_ISO_EP_COMP: u8 = 0x31;

/// The request transfers data from the host to the device
/// (`USB_DEV_REQ_HOST_TO_DEV`).
pub const USB_DEV_REQ_HOST_TO_DEV: u8 = 0x00;
/// The request transfers data from the device to the host
/// (`USB_DEV_REQ_DEV_TO_HOST`).
pub const USB_DEV_REQ_DEV_TO_HOST: u8 = 0x80;
/// The mask of the direction bit of `bmRequestType`
/// (`USB_DEV_REQ_DIR_MASK`).
pub const USB_DEV_REQ_DIR_MASK: u8 = 0x80;
/// A standard request defined by USB 2.0 for all devices
/// (`USB_DEV_REQ_TYPE_STANDARD`).
pub const USB_DEV_REQ_TYPE_STANDARD: u8 = 0x00;
/// A class specific request defined by USB 2.0 (`USB_DEV_REQ_TYPE_CLASS`).
pub const USB_DEV_REQ_TYPE_CLASS: u8 = 0x20;
/// A vendor specific request (`USB_DEV_REQ_TYPE_VENDOR`).
pub const USB_DEV_REQ_TYPE_VENDOR: u8 = 0x40;
/// The mask of the request type bits of `bmRequestType`
/// (`USB_DEV_REQ_TYPE_MASK`).
pub const USB_DEV_REQ_TYPE_MASK: u8 = 0x60;
/// The request is addressed to the device (`USB_DEV_REQ_RCPT_DEV`).
pub const USB_DEV_REQ_RCPT_DEV: u8 = 0x00;
/// The request is addressed to an interface (`USB_DEV_REQ_RCPT_IF`).
pub const USB_DEV_REQ_RCPT_IF: u8 = 0x01;
/// The request is addressed to an endpoint (`USB_DEV_REQ_RCPT_EP`).
pub const USB_DEV_REQ_RCPT_EP: u8 = 0x02;
/// The request is addressed to a recipient other than the device, an
/// interface or an endpoint (`USB_DEV_REQ_RCPT_OTHER`).
pub const USB_DEV_REQ_RCPT_OTHER: u8 = 0x03;
/// The mask of the recipient bits of `bmRequestType`
/// (`USB_DEV_REQ_RCPT_MASK`).
pub const USB_DEV_REQ_RCPT_MASK: u8 = 0x03;

/// The standard request that gets the status of a device, interface or
/// endpoint, section 9.4.5 (`USB_REQ_GET_STATUS`).
pub const USB_REQ_GET_STATUS: u8 = 0x00;
/// The standard request that clears the feature given in `wValue`, section
/// 9.4.1 (`USB_REQ_CLEAR_FEATURE`).
pub const USB_REQ_CLEAR_FEATURE: u8 = 0x01;
/// The standard request that sets the feature given in `wValue`, section
/// 9.4.9 (`USB_REQ_SET_FEATURE`).
pub const USB_REQ_SET_FEATURE: u8 = 0x03;
/// The standard request that sets the address given in `wValue`, section
/// 9.4.6 (`USB_REQ_SET_ADDRESS`).
pub const USB_REQ_SET_ADDRESS: u8 = 0x05;
/// The standard request that gets the descriptor selected by `wValue`,
/// section 9.4.3 (`USB_REQ_GET_DESCR`).
pub const USB_REQ_GET_DESCR: u8 = 0x06;
/// The standard request that sets the descriptor selected by `wValue`,
/// section 9.4.8 (`USB_REQ_SET_DESCR`).
pub const USB_REQ_SET_DESCR: u8 = 0x07;
/// The standard request that gets the current configuration, section 9.4.2
/// (`USB_REQ_GET_CFG`).
pub const USB_REQ_GET_CFG: u8 = 0x08;
/// The standard request that sets the current configuration, section 9.4.7.
/// Drivers must use [`usb_set_cfg`] instead, because only that function
/// updates the state of the framework (`USB_REQ_SET_CFG`).
pub const USB_REQ_SET_CFG: u8 = 0x09;
/// The standard request that gets the alternate setting of an interface,
/// section 9.4.4 (`USB_REQ_GET_IF`).
pub const USB_REQ_GET_IF: u8 = 0x0a;
/// The standard request that sets the alternate setting of an interface,
/// section 9.4.10 (`USB_REQ_SET_IF`).
pub const USB_REQ_SET_IF: u8 = 0x0b;
/// The standard request that sets and reports the synchronisation frame of
/// an endpoint, section 9.4.11 (`USB_REQ_SYNC_FRAME`).
pub const USB_REQ_SYNC_FRAME: u8 = 0x0c;
/// The standard request that sets the interface of the device side, from the
/// USB 3 specification (`USB_REQ_SET_INTERFACE_DS`).
pub const USB_REQ_SET_INTERFACE_DS: u8 = 0x17;

/// The feature selector that halts an endpoint (`USB_EP_HALT`).
pub const USB_EP_HALT: uint_t = 0x0000;
/// The feature selector for remote wakeup of a device
/// (`USB_DEV_REMOTE_WAKEUP`).
pub const USB_DEV_REMOTE_WAKEUP: uint_t = 0x0001;
/// The feature selector for the test mode of a device (`USB_DEV_TEST_MODE`).
pub const USB_DEV_TEST_MODE: uint_t = 0x0002;

/// The standard status request type (`USB_GET_STATUS_STANDARD`).
pub const USB_GET_STATUS_STANDARD: c_int = 0;
/// The Precision Time Measurement status request type
/// (`USB_GET_STATUS_PTM`).
pub const USB_GET_STATUS_PTM: c_int = 1;
/// The length in bytes of the data returned by a standard status request
/// (`USB_GET_STATUS_LEN`).
pub const USB_GET_STATUS_LEN: u16 = 2;
/// The length in bytes of the data returned by a Precision Time Measurement
/// status request (`USB_GET_STATUS_PTM_LEN`).
pub const USB_GET_STATUS_PTM_LEN: u16 = 4;
/// The device status bit that says the device is currently self powered
/// (`USB_DEV_SLF_PWRD_STATUS`).
pub const USB_DEV_SLF_PWRD_STATUS: u16 = 1;
/// The device status bit that says remote wakeup is enabled
/// (`USB_DEV_RWAKEUP_STATUS`).
pub const USB_DEV_RWAKEUP_STATUS: u16 = 2;
/// The device status bit that says the device is battery powered
/// (`USB_DEV_BAT_PWRD_STATUS`).
pub const USB_DEV_BAT_PWRD_STATUS: u16 = 4;
/// The endpoint status bit that says the endpoint is halted
/// (`USB_EP_HALT_STATUS`).
pub const USB_EP_HALT_STATUS: u16 = 1;
/// The status of an interface, which is always 0 (`USB_IF_STATUS`).
pub const USB_IF_STATUS: u16 = 0;

// --- Bulk, interrupt and isochronous requests ------------------------------------

/// `usb_bulk_req(9S)`. illumos DDI specific (illumos DDI).
///
/// The normal and exception callback of a bulk request. The framework
/// passes the pipe handle and the completed request.
pub type usb_bulk_req_cb_t = unsafe extern "C" fn(ph: usb_pipe_handle_t, req: *mut usb_bulk_req);

/// `usb_bulk_req(9S)`. illumos DDI specific (illumos DDI).
///
/// A bulk request, which transfers large amounts of data reliably but
/// without timing guarantees. Bulk transfers are described in section 5.8
/// of the USB 2.0 specification. A client driver allocates the request with
/// [`usb_alloc_bulk_req`] and submits it with [`usb_pipe_bulk_xfer`].
/// `bulk_len` must be greater than 0 and `bulk_data` must not be NULL.
/// Multiple bulk requests can be queued, but bulk pipes cannot be shared.
/// The host controller can only transfer a limited amount per request,
/// which [`usb_pipe_get_max_bulk_transfer_size`] reports, so larger
/// transfers have to be split into several requests.
#[repr(C)]
#[derive(Debug)]
pub struct usb_bulk_req {
    /// The number of bytes to transfer.
    pub bulk_len: uint_t,
    /// The data of the data phase. It is allocated by the client for OUT
    /// requests and by the framework for IN requests.
    pub bulk_data: *mut mblk_t,
    /// The transfer timeout in seconds. A value of 0 selects the default of
    /// 5 seconds.
    pub bulk_timeout: uint_t,
    /// Private data of the client, which the framework does not touch.
    pub bulk_client_private: usb_opaque_t,
    /// The attributes of the transfer. `USB_ATTRS_SHORT_XFER_OK`,
    /// `USB_ATTRS_AUTOCLEARING` and `USB_ATTRS_PIPE_RESET` are valid for
    /// bulk requests, and `USB_ATTRS_SHORT_XFER_OK` is illegal for OUT
    /// transfers.
    pub bulk_attributes: usb_req_attrs_t,
    /// The callback for normal completion.
    pub bulk_cb: Option<usb_bulk_req_cb_t>,
    /// The callback for exceptional completion.
    pub bulk_exc_cb: Option<usb_bulk_req_cb_t>,
    /// The overall completion status, set by the host controller driver.
    pub bulk_completion_reason: usb_cr_t,
    /// The callback context and the recovery actions taken, set by the
    /// framework.
    pub bulk_cb_flags: usb_cb_flags_t,
}
/// The typedef name of [`usb_bulk_req`].
pub type usb_bulk_req_t = usb_bulk_req;

// usb_bulk_req: bulk_len at 0, bulk_data at 8, bulk_timeout at 16,
// bulk_client_private at 24, bulk_attributes at 32, bulk_cb at 40,
// bulk_exc_cb at 48, bulk_completion_reason at 56, bulk_cb_flags at 60, 64
// bytes.
const _: () = assert!(core::mem::size_of::<usb_bulk_req>() == 64);
const _: () = assert!(core::mem::offset_of!(usb_bulk_req, bulk_len) == 0);
const _: () = assert!(core::mem::offset_of!(usb_bulk_req, bulk_data) == 8);
const _: () = assert!(core::mem::offset_of!(usb_bulk_req, bulk_client_private) == 24);
const _: () = assert!(core::mem::offset_of!(usb_bulk_req, bulk_cb) == 40);
const _: () = assert!(core::mem::offset_of!(usb_bulk_req, bulk_cb_flags) == 60);

/// `usb_intr_req(9S)`. illumos DDI specific (illumos DDI).
///
/// The normal and exception callback of an interrupt request. The framework
/// passes the pipe handle and the completed request. During periodic
/// polling each callback receives a new request cloned from the original.
pub type usb_intr_req_cb_t = unsafe extern "C" fn(ph: usb_pipe_handle_t, req: *mut usb_intr_req);

/// `usb_intr_req(9S)`. illumos DDI specific (illumos DDI).
///
/// An interrupt request, which transfers small amounts of data
/// infrequently but within bounded service periods, in either direction.
/// Interrupt transfers are described in section 5.7 of the USB 2.0
/// specification. A client driver allocates the request with
/// [`usb_alloc_intr_req`] and submits it with [`usb_pipe_intr_xfer`].
///
/// An IN request without `USB_ATTRS_ONE_XFER` starts continuous polling.
/// The client provides no data buffer; the framework allocates a new
/// request for each poll that returns data and passes it to the normal
/// callback, and the original request comes back through the exception
/// callback with `USB_CR_STOPPED_POLLING` once polling stops. An IN request
/// with `USB_ATTRS_ONE_XFER` performs a single transfer. An OUT request
/// carries its data, is written until the data is exhausted and results in
/// one callback; OUT requests can be queued. The manual page shows
/// `intr_len` as a `ushort_t`, but the header declares a `uint_t`, which
/// this binding follows.
#[repr(C)]
#[derive(Debug)]
pub struct usb_intr_req {
    /// For OUT requests the size of the whole transfer, for IN requests the
    /// size of the data returned per poll. The maximum is 8 KiB for low and
    /// full speed devices and 20 KiB for high speed devices.
    pub intr_len: uint_t,
    /// The data of the data phase. It is allocated by the client for OUT
    /// requests and must be NULL for IN requests, whose data the framework
    /// allocates.
    pub intr_data: *mut mblk_t,
    /// Private data of the client, which the framework does not touch.
    pub intr_client_private: usb_opaque_t,
    /// The timeout in seconds, only used with a one time poll. A value of 0
    /// selects the default of 5 seconds.
    pub intr_timeout: uint_t,
    /// The attributes of the request. `USB_ATTRS_SHORT_XFER_OK`,
    /// `USB_ATTRS_AUTOCLEARING`, `USB_ATTRS_PIPE_RESET` and
    /// `USB_ATTRS_ONE_XFER` are valid for interrupt requests.
    pub intr_attributes: usb_req_attrs_t,
    /// The callback for normal completion.
    pub intr_cb: Option<usb_intr_req_cb_t>,
    /// The callback for exceptional completion.
    pub intr_exc_cb: Option<usb_intr_req_cb_t>,
    /// The overall completion status, set by the host controller driver.
    pub intr_completion_reason: usb_cr_t,
    /// The callback context and the recovery actions taken, set by the
    /// framework.
    pub intr_cb_flags: usb_cb_flags_t,
}
/// The typedef name of [`usb_intr_req`].
pub type usb_intr_req_t = usb_intr_req;

// usb_intr_req: intr_len at 0, intr_data at 8, intr_client_private at 16,
// intr_timeout at 24, intr_attributes at 28, intr_cb at 32, intr_exc_cb at
// 40, intr_completion_reason at 48, intr_cb_flags at 52, padded to 56 bytes.
const _: () = assert!(core::mem::size_of::<usb_intr_req>() == 56);
const _: () = assert!(core::mem::offset_of!(usb_intr_req, intr_len) == 0);
const _: () = assert!(core::mem::offset_of!(usb_intr_req, intr_data) == 8);
const _: () = assert!(core::mem::offset_of!(usb_intr_req, intr_attributes) == 28);
const _: () = assert!(core::mem::offset_of!(usb_intr_req, intr_cb) == 32);
const _: () = assert!(core::mem::offset_of!(usb_intr_req, intr_cb_flags) == 52);

/// `usb_isoc_req(9S)`. illumos DDI specific (illumos DDI).
///
/// The descriptor of one isochronous packet, `usb_isoc_pkt_descr_t`. It
/// describes the packet transferred within one frame or microframe. The
/// client sets the length, and the framework reports the actual length and
/// the status. Packets whose status is not `USB_CR_OK` are counted in
/// `isoc_error_count` of the request. An array of these descriptors is
/// allocated together with the request by [`usb_alloc_isoc_req`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct usb_isoc_pkt_descr {
    /// The number of bytes to transfer in the frame, set by the client for
    /// every isochronous request.
    pub isoc_pkt_length: u16,
    /// The number of bytes actually sent or received in the frame, set by
    /// the host controller driver.
    pub isoc_pkt_actual_length: u16,
    /// The status of the packet, set by the host controller driver for IN
    /// and OUT requests.
    pub isoc_pkt_status: usb_cr_t,
}
/// The typedef name of [`usb_isoc_pkt_descr`].
pub type usb_isoc_pkt_descr_t = usb_isoc_pkt_descr;

// usb_isoc_pkt_descr: two u16 at 0 and 2, isoc_pkt_status at 4, 8 bytes.
const _: () = assert!(core::mem::size_of::<usb_isoc_pkt_descr>() == 8);
const _: () = assert!(core::mem::offset_of!(usb_isoc_pkt_descr, isoc_pkt_length) == 0);
const _: () = assert!(core::mem::offset_of!(usb_isoc_pkt_descr, isoc_pkt_status) == 4);

/// `usb_isoc_req(9S)`. illumos DDI specific (illumos DDI).
///
/// The normal and exception callback of an isochronous request. Normal
/// completion callbacks of isochronous requests run in interrupt context
/// and must never block.
pub type usb_isoc_req_cb_t = unsafe extern "C" fn(ph: usb_pipe_handle_t, req: *mut usb_isoc_req);

/// `usb_isoc_req(9S)`. illumos DDI specific (illumos DDI).
///
/// An isochronous request, which transfers large amounts of packetised data
/// with bounded service periods but without retries. A packet is tried
/// within a bounded time and not retried on failure, so errors never stop
/// the transfer and are recorded in the packet descriptors instead.
/// Isochronous transfers are described in section 5.6 of the USB 2.0
/// specification. A client driver allocates the request together with its
/// packet descriptors using [`usb_alloc_isoc_req`] and submits it with
/// [`usb_pipe_isoc_xfer`]. Exactly one of `USB_ATTRS_ISOC_START_FRAME` and
/// `USB_ATTRS_ISOC_XFER_ASAP` must be set. IN requests start continuous
/// polling, and both normal and exception callbacks must be given.
#[repr(C)]
#[derive(Debug)]
pub struct usb_isoc_req {
    /// The frame in which the request starts, used with
    /// `USB_ATTRS_ISOC_START_FRAME` to synchronise requests on different
    /// isochronous pipes. A value of 0 is an invalid frame number. It is
    /// ignored with `USB_ATTRS_ISOC_XFER_ASAP`.
    pub isoc_frame_no: usb_frame_number_t,
    /// The number of packets in this request, set by the client. It must
    /// not exceed the number of packet descriptors allocated.
    pub isoc_pkts_count: u16,
    /// The sum of the lengths of all packets. The header recommends setting
    /// it to 0, in which case the sum of the `isoc_pkt_length` fields is
    /// used and no check is applied.
    pub isoc_pkts_length: u16,
    /// The number of packets that completed with errors, set by the host
    /// controller driver.
    pub isoc_error_count: u16,
    /// The attributes of the request. `USB_ATTRS_ISOC_START_FRAME`,
    /// `USB_ATTRS_ISOC_XFER_ASAP` and `USB_ATTRS_SHORT_XFER_OK` are valid,
    /// and `USB_ATTRS_ONE_XFER` is not.
    pub isoc_attributes: usb_req_attrs_t,
    /// The data to transfer. For OUT requests the client allocates and sets
    /// it and the host controller driver frees and clears it on successful
    /// completion. For IN requests the host controller driver allocates it
    /// and the client frees it.
    pub isoc_data: *mut mblk_t,
    /// Private data of the client, which the framework does not touch.
    pub isoc_client_private: usb_opaque_t,
    /// The array of packet descriptors. The client must allocate it for
    /// every request and must also initialise it for OUT requests.
    pub isoc_pkt_descr: *mut usb_isoc_pkt_descr,
    /// The callback for normal completion. It runs in interrupt context and
    /// must not block.
    pub isoc_cb: Option<usb_isoc_req_cb_t>,
    /// The callback for exceptional completion, which signals that polling
    /// has stopped.
    pub isoc_exc_cb: Option<usb_isoc_req_cb_t>,
    /// The overall completion status, set by the host controller driver.
    pub isoc_completion_reason: usb_cr_t,
    /// The callback context and the recovery actions taken, set by the
    /// framework on an exception.
    pub isoc_cb_flags: usb_cb_flags_t,
}
/// The typedef name of [`usb_isoc_req`].
pub type usb_isoc_req_t = usb_isoc_req;

// usb_isoc_req: isoc_frame_no at 0, three u16 at 8, 10 and 12,
// isoc_attributes at 16, isoc_data at 24, isoc_client_private at 32,
// isoc_pkt_descr at 40, isoc_cb at 48, isoc_exc_cb at 56,
// isoc_completion_reason at 64, isoc_cb_flags at 68, 72 bytes.
const _: () = assert!(core::mem::size_of::<usb_isoc_req>() == 72);
const _: () = assert!(core::mem::offset_of!(usb_isoc_req, isoc_frame_no) == 0);
const _: () = assert!(core::mem::offset_of!(usb_isoc_req, isoc_attributes) == 16);
const _: () = assert!(core::mem::offset_of!(usb_isoc_req, isoc_data) == 24);
const _: () = assert!(core::mem::offset_of!(usb_isoc_req, isoc_pkt_descr) == 40);
const _: () = assert!(core::mem::offset_of!(usb_isoc_req, isoc_cb) == 48);
const _: () = assert!(core::mem::offset_of!(usb_isoc_req, isoc_cb_flags) == 68);

// --- Power management ----------------------------------------------------------

/// The operating system power level 0, which means no power
/// (`USB_DEV_OS_PWR_0`).
pub const USB_DEV_OS_PWR_0: c_int = 0;
/// The operating system power level 1 (`USB_DEV_OS_PWR_1`).
pub const USB_DEV_OS_PWR_1: c_int = 1;
/// The operating system power level 2 (`USB_DEV_OS_PWR_2`).
pub const USB_DEV_OS_PWR_2: c_int = 2;
/// The operating system power level 3, which means full power
/// (`USB_DEV_OS_PWR_3`).
pub const USB_DEV_OS_PWR_3: c_int = 3;
/// The operating system power level for a powered off device
/// (`USB_DEV_OS_PWR_OFF`).
pub const USB_DEV_OS_PWR_OFF: c_int = USB_DEV_OS_PWR_0;
/// The operating system power level for a device at full power
/// (`USB_DEV_OS_FULL_PWR`).
pub const USB_DEV_OS_FULL_PWR: c_int = USB_DEV_OS_PWR_3;
/// The USB power state D0, full power, expressed as an operating system
/// power level (`USB_DEV_PWR_D0`).
pub const USB_DEV_PWR_D0: c_int = USB_DEV_OS_FULL_PWR;
/// The USB power state D1 as the header defines it (`USB_DEV_PWR_D1`).
pub const USB_DEV_PWR_D1: c_int = 5;
/// The USB power state D2 as the header defines it (`USB_DEV_PWR_D2`).
pub const USB_DEV_PWR_D2: c_int = 6;
/// The USB power state D3, powered down, expressed as an operating system
/// power level (`USB_DEV_PWR_D3`).
pub const USB_DEV_PWR_D3: c_int = USB_DEV_OS_PWR_OFF;
/// The bit that [`usb_create_pm_components`] sets when the device supports
/// D0, which corresponds to `USB_DEV_OS_PWR_3` (`USB_DEV_OS_PWRMASK_D0`).
pub const USB_DEV_OS_PWRMASK_D0: uint_t = 1;
/// The bit for D1, which corresponds to `USB_DEV_OS_PWR_2`
/// (`USB_DEV_OS_PWRMASK_D1`).
pub const USB_DEV_OS_PWRMASK_D1: uint_t = 2;
/// The bit for D2, which corresponds to `USB_DEV_OS_PWR_1`
/// (`USB_DEV_OS_PWRMASK_D2`).
pub const USB_DEV_OS_PWRMASK_D2: uint_t = 4;
/// The bit for D3, which corresponds to `USB_DEV_OS_PWR_0`
/// (`USB_DEV_OS_PWRMASK_D3`).
pub const USB_DEV_OS_PWRMASK_D3: uint_t = 8;
/// The command of [`usb_handle_remote_wakeup`] that enables remote wakeup
/// (`USB_REMOTE_WAKEUP_ENABLE`).
pub const USB_REMOTE_WAKEUP_ENABLE: c_int = 1;
/// The command of [`usb_handle_remote_wakeup`] that disables remote wakeup
/// (`USB_REMOTE_WAKEUP_DISABLE`).
pub const USB_REMOTE_WAKEUP_DISABLE: c_int = 2;

/// `usb_register_hotplug_cbs(9F)`. illumos DDI specific (illumos DDI).
///
/// The disconnection and reconnection event handler registered with
/// [`usb_register_hotplug_cbs`]. It receives the device node of the device
/// that was disconnected or reconnected and always returns `USB_SUCCESS`.
/// It runs in kernel context. A handler that needs any DDI function other
/// than the `ddi_taskq_*` family should dispatch its work to a task queue
/// to avoid deadlocks.
pub type usb_hotplug_cb_t = unsafe extern "C" fn(dip: dev_info_t) -> c_int;

// --- Class codes ------------------------------------------------------------------

/// The class information is found at the interface level
/// (`USB_CLASS_PER_INTERFACE`).
pub const USB_CLASS_PER_INTERFACE: u8 = 0;
/// The audio interface class (`USB_CLASS_AUDIO`).
pub const USB_CLASS_AUDIO: u8 = 1;
/// The communication device class (`USB_CLASS_COMM`).
pub const USB_CLASS_COMM: u8 = 2;
/// The CDC control interface class, which has the same value as the
/// communication device class (`USB_CLASS_CDC_CTRL`).
pub const USB_CLASS_CDC_CTRL: u8 = 2;
/// The human interface device class (`USB_CLASS_HID`).
pub const USB_CLASS_HID: u8 = 3;
/// The physical interface class (`USB_CLASS_PHYSICAL`).
pub const USB_CLASS_PHYSICAL: u8 = 5;
/// The image interface class (`USB_CLASS_IMAGE`).
pub const USB_CLASS_IMAGE: u8 = 6;
/// The printer interface class (`USB_CLASS_PRINTER`).
pub const USB_CLASS_PRINTER: u8 = 7;
/// The mass storage interface class (`USB_CLASS_MASS_STORAGE`).
pub const USB_CLASS_MASS_STORAGE: u8 = 8;
/// The hub device class (`USB_CLASS_HUB`).
pub const USB_CLASS_HUB: u8 = 9;
/// The CDC data interface class (`USB_CLASS_CDC_DATA`).
pub const USB_CLASS_CDC_DATA: u8 = 10;
/// The chip and smart card interface class (`USB_CLASS_CCID`).
pub const USB_CLASS_CCID: u8 = 11;
/// The content security interface class (`USB_CLASS_SECURITY`).
pub const USB_CLASS_SECURITY: u8 = 13;
/// The video interface class (`USB_CLASS_VIDEO`).
pub const USB_CLASS_VIDEO: u8 = 14;
/// The diagnostic device class (`USB_CLASS_DIAG`).
pub const USB_CLASS_DIAG: u8 = 220;
/// The wireless controller class (`USB_CLASS_WIRELESS`).
pub const USB_CLASS_WIRELESS: u8 = 224;
/// The miscellaneous device class (`USB_CLASS_MISC`).
pub const USB_CLASS_MISC: u8 = 239;
/// The application specific interface class (`USB_CLASS_APP`).
pub const USB_CLASS_APP: u8 = 254;
/// The vendor specific class (`USB_CLASS_VENDOR_SPEC`).
pub const USB_CLASS_VENDOR_SPEC: u8 = 255;
/// The audio control subclass (`USB_SUBCLS_AUD_CONTROL`).
pub const USB_SUBCLS_AUD_CONTROL: u8 = 0x01;
/// The audio streaming subclass (`USB_SUBCLS_AUD_STREAMING`).
pub const USB_SUBCLS_AUD_STREAMING: u8 = 0x02;
/// The MIDI streaming subclass (`USB_SUBCLS_AUD_MIDI_STREAMING`).
pub const USB_SUBCLS_AUD_MIDI_STREAMING: u8 = 0x03;
/// The direct line control subclass of CDC (`USB_SUBCLS_CDCC_DIRECT_LINE`).
pub const USB_SUBCLS_CDCC_DIRECT_LINE: u8 = 0x01;
/// The abstract control subclass of CDC (`USB_SUBCLS_CDCC_ABSTRCT_CTRL`).
pub const USB_SUBCLS_CDCC_ABSTRCT_CTRL: u8 = 0x02;
/// The telephone control subclass of CDC (`USB_SUBCLS_CDCC_PHONE_CTRL`).
pub const USB_SUBCLS_CDCC_PHONE_CTRL: u8 = 0x03;
/// The multichannel control subclass of CDC for ISDN
/// (`USB_SUBCLS_CDCC_MULTCNL_ISDN`).
pub const USB_SUBCLS_CDCC_MULTCNL_ISDN: u8 = 0x04;
/// The CAPI control subclass of CDC for ISDN (`USB_SUBCLS_CDCC_ISDN`).
pub const USB_SUBCLS_CDCC_ISDN: u8 = 0x05;
/// The Ethernet networking control subclass of CDC
/// (`USB_SUBCLS_CDCC_ETHERNET`).
pub const USB_SUBCLS_CDCC_ETHERNET: u8 = 0x06;
/// The ATM networking control subclass of CDC
/// (`USB_SUBCLS_CDCC_ATM_NETWORK`).
pub const USB_SUBCLS_CDCC_ATM_NETWORK: u8 = 0x07;
/// The boot interface subclass of HID (`USB_SUBCLS_HID_1`).
pub const USB_SUBCLS_HID_1: u8 = 1;
/// The legacy keyboard protocol of HID (`USB_PROTO_HID_KEYBOARD`).
pub const USB_PROTO_HID_KEYBOARD: u8 = 0x01;
/// The legacy mouse protocol of HID (`USB_PROTO_HID_MOUSE`).
pub const USB_PROTO_HID_MOUSE: u8 = 0x02;
/// The printer subclass (`USB_SUBCLS_PRINTER_1`).
pub const USB_SUBCLS_PRINTER_1: u8 = 1;
/// The unidirectional printer protocol (`USB_PROTO_PRINTER_UNI`).
pub const USB_PROTO_PRINTER_UNI: u8 = 0x01;
/// The bidirectional printer protocol (`USB_PROTO_PRINTER_BI`).
pub const USB_PROTO_PRINTER_BI: u8 = 0x02;
/// The mass storage subclass of the reduced block command set, used for
/// flash devices (`USB_SUBCLS_MS_RBC_T10`).
pub const USB_SUBCLS_MS_RBC_T10: u8 = 0x1;
/// The mass storage subclass of SFF-8020i, used for CD-ROM drives
/// (`USB_SUBCLS_MS_SFF8020I`).
pub const USB_SUBCLS_MS_SFF8020I: u8 = 0x2;
/// The mass storage subclass of QIC-157, used for tape drives
/// (`USB_SUBCLS_MS_QIC_157`).
pub const USB_SUBCLS_MS_QIC_157: u8 = 0x3;
/// The mass storage subclass of USB floppy disk drives
/// (`USB_SUBCLS_MS_UFI`).
pub const USB_SUBCLS_MS_UFI: u8 = 0x4;
/// The mass storage subclass of SFF-8070i, used for floppy drives
/// (`USB_SUBCLS_MS_SFF8070I`).
pub const USB_SUBCLS_MS_SFF8070I: u8 = 0x5;
/// The mass storage subclass of transparent SCSI (`USB_SUBCLS_MS_SCSI`).
pub const USB_SUBCLS_MS_SCSI: u8 = 0x6;
/// The control, bulk and interrupt mass storage protocol with command
/// completion interrupt (`USB_PROTO_MS_CBI_WC`).
pub const USB_PROTO_MS_CBI_WC: u8 = 0x00;
/// The control, bulk and interrupt mass storage protocol (`USB_PROTO_MS_CBI`).
pub const USB_PROTO_MS_CBI: u8 = 0x01;
/// The ZIP mass storage protocol (`USB_PROTO_MS_ISD_1999_SILICN`).
pub const USB_PROTO_MS_ISD_1999_SILICN: u8 = 0x02;
/// The bulk only mass storage protocol (`USB_PROTO_MS_BULK_ONLY`).
pub const USB_PROTO_MS_BULK_ONLY: u8 = 0x50;
/// The full speed hub protocol (`USB_PROTO_HUB_FULL`).
pub const USB_PROTO_HUB_FULL: u8 = 0x00;
/// The high speed hub protocol with a single transaction translator
/// (`USB_PROTO_HUB_HIGH_STT`).
pub const USB_PROTO_HUB_HIGH_STT: u8 = 0x01;
/// The high speed hub protocol with multiple transaction translators
/// (`USB_PROTO_HUB_HIGH_MTT`).
pub const USB_PROTO_HUB_HIGH_MTT: u8 = 0x02;
/// The SuperSpeed hub protocol (`USB_PROTO_HUB_SUPER`).
pub const USB_PROTO_HUB_SUPER: u8 = 0x03;
/// The application specific firmware upgrade subclass
/// (`USB_SUBCLS_APP_FIRMWARE`).
pub const USB_SUBCLS_APP_FIRMWARE: u8 = 0x01;
/// The application specific IrDA bridge subclass (`USB_SUBCLS_APP_IRDA`).
pub const USB_SUBCLS_APP_IRDA: u8 = 0x02;
/// The application specific test and measurement subclass
/// (`USB_SUBCLS_APP_TEST`).
pub const USB_SUBCLS_APP_TEST: u8 = 0x03;
/// The video control subclass (`USB_SUBCLS_VIDEO_CONTROL`).
pub const USB_SUBCLS_VIDEO_CONTROL: u8 = 0x01;
/// The video streaming subclass (`USB_SUBCLS_VIDEO_STREAM`).
pub const USB_SUBCLS_VIDEO_STREAM: u8 = 0x02;
/// The video interface collection subclass (`USB_SUBCLS_VIDEO_COLLECTION`).
pub const USB_SUBCLS_VIDEO_COLLECTION: u8 = 0x03;

// --- Functions ------------------------------------------------------------------

unsafe extern "C" {
    /// `usb_client_attach(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers the driver of `dip` with the USBA framework. It must be
    /// called before any other USBA function and is usually followed by a
    /// call to [`usb_get_dev_data`]. `version` must be [`USBDRV_VERSION`],
    /// whose major part has to match the major version of the framework and
    /// whose minor part must not exceed its minor version. `flags` is not
    /// used.
    ///
    /// Returns `USB_SUCCESS` when the registration succeeded,
    /// `USB_INVALID_ARGS` when `dip` is NULL, `USB_INVALID_CONTEXT` when
    /// called from interrupt context or outside of attach,
    /// `USB_INVALID_VERSION` when the version is not valid and `USB_FAILURE`
    /// on any other internal error. The function may only be called from
    /// `attach(9E)`.
    pub fn usb_client_attach(dip: dev_info_t, version: uint_t, flags: usb_flags_t) -> c_int;
    /// `usb_client_detach(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unregisters the driver of `dip` from the USBA framework. When
    /// `dev_data` is not NULL it also releases all strings, descriptors and
    /// trees that [`usb_get_dev_data`] set up in it. It is the last USBA
    /// function a client calls before its `detach(9E)` completes, and it is
    /// not needed during a suspend operation. The function may be called
    /// only from `attach(9E)` or `detach(9E)`.
    pub fn usb_client_detach(dip: dev_info_t, dev_data: *mut usb_client_dev_data_t);

    /// `usb_get_dev_data(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Interrogates the device of `dip` and stores in `*dev_data` a pointer
    /// to its configuration information, described on the
    /// `usb_client_dev_data(9S)` page. `parse_level` selects how much of the
    /// device the returned descriptor tree represents, and `flags` is not
    /// used. Class and vendor specific descriptors are returned in their raw
    /// form.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_INVALID_ARGS` when `dip` or
    /// `dev_data` is NULL or the parse level is not valid,
    /// `USB_INVALID_CONTEXT` when called from interrupt context,
    /// `USB_INVALID_VERSION` when [`usb_client_attach`] was not called first
    /// and `USB_FAILURE` on bad descriptor information or another internal
    /// error. The function may be called from user or kernel context.
    pub fn usb_get_dev_data(
        dip: dev_info_t,
        dev_data: *mut *mut usb_client_dev_data_t,
        parse_level: usb_reg_parse_lvl_t,
        flags: usb_flags_t,
    ) -> c_int;
    /// `usb_free_dev_data(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Undoes what [`usb_get_dev_data`] set up, releasing the memory of all
    /// strings, descriptors and trees in `dev_data`. The function may be
    /// called from user, kernel or interrupt context.
    pub fn usb_free_dev_data(dip: dev_info_t, dev_data: *mut usb_client_dev_data_t);
    /// `usb_free_descr_tree(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the descriptor tree of `dev_data` while leaving the rest of the
    /// information intact, so that a driver can release the memory after it
    /// has copied the descriptors it needs. Afterwards `dev_cfg` and
    /// `dev_curr_cfg` are NULL, `dev_n_cfg` is 0, `dev_parse_level` is
    /// `USB_PARSE_LVL_NONE` and `dev_curr_if` is invalid. Nothing happens
    /// when `dip` or `dev_data` is NULL. The function may be called from
    /// user, kernel or interrupt context.
    pub fn usb_free_descr_tree(dip: dev_info_t, dev_data: *mut usb_client_dev_data_t);
    /// `usb_print_descr_tree(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A diagnostic aid that dumps the descriptor tree in `dev_data` to the
    /// console when the system was booted verbose. Returns `USB_SUCCESS`
    /// when the dump succeeded, `USB_INVALID_ARGS` when `dip` or `dev_data`
    /// is NULL, `USB_INVALID_CONTEXT` when called from interrupt context and
    /// `USB_FAILURE` on any other error. The function may be called from
    /// user or kernel context.
    pub fn usb_print_descr_tree(dip: dev_info_t, dev_data: *mut usb_client_dev_data_t) -> c_int;

    /// `usb_parse_data(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Parses raw little endian data, typically a class or vendor specific
    /// descriptor whose format the USB specification cannot define, into a
    /// structure in host byte order with the fields properly aligned.
    /// `format` is a NUL terminated string in which the letters `c`, `s`,
    /// `l` and `L` stand for quantities of 1, 2, 4 and 8 bytes, and a number
    /// before a letter repeats it, so that `"scc"` and `"s2c"` both describe
    /// a short followed by two bytes. `data` and `datalen` give the raw
    /// descriptor, and `structure` and `structlen` the destination buffer;
    /// the result is truncated to `structlen` bytes.
    ///
    /// Returns the size in bytes of the parsed result, or 0, which is
    /// `USB_PARSE_ERROR`, on failure. The function may be called from user,
    /// kernel or interrupt context.
    pub fn usb_parse_data(
        format: *mut c_char,
        data: *const c_uchar,
        datalen: size_t,
        structure: *mut c_void,
        structlen: size_t,
    ) -> size_t;
    /// `usb_lookup_ep_data(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the endpoint node from the descriptor tree in `dev_datap`,
    /// looking only at the current configuration that `dev_curr_cfg` points
    /// to. `interface` and `alternate` select the interface and its
    /// alternate setting. `type` is one of `USB_EP_ATTR_CONTROL`,
    /// `USB_EP_ATTR_ISOCH`, `USB_EP_ATTR_BULK` or `USB_EP_ATTR_INTR`, and
    /// `direction` is `USB_EP_DIR_OUT` or `USB_EP_DIR_IN`, which is ignored
    /// for bidirectional control endpoints. The function skips the first
    /// `skip` matching endpoints and returns the next one. The device is not
    /// accessed and no copy is made; the result points into the tree.
    ///
    /// Returns the tree node of the endpoint, or NULL when `dip` or
    /// `dev_datap` is NULL, no tree is present or the endpoint does not
    /// exist. The function may be called from user, kernel or interrupt
    /// context.
    pub fn usb_lookup_ep_data(
        dip: dev_info_t,
        dev_datap: *mut usb_client_dev_data_t,
        interface: uint_t,
        alternate: uint_t,
        skip: uint_t,
        r#type: uint_t,
        direction: uint_t,
    ) -> *mut usb_ep_data_t;
    /// `usb_get_string_descr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads the string descriptor with index `index` from the device of
    /// `dip` and stores it, converted and NUL terminated, in `buf`, which is
    /// `buflen` bytes long. Longer strings are truncated. `langid` must
    /// currently be `USB_LANG_ID`. The function accesses the device and
    /// blocks.
    ///
    /// Returns `USB_SUCCESS` when the string is in `buf`, `USB_INVALID_ARGS`
    /// when `dip` or `buf` is NULL or `index` or `buflen` is 0, and
    /// `USB_FAILURE` when the descriptor cannot be retrieved. The function
    /// may be called from user or kernel context.
    pub fn usb_get_string_descr(dip: dev_info_t, langid: u16, index: u8, buf: *mut c_char, buflen: size_t) -> c_int;
    /// `usb_ep_xdescr_fill(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Fills in the extended endpoint descriptor `ep_xdesc` from the
    /// endpoint data `ep_data`, which was obtained from
    /// [`usb_lookup_ep_data`]. Afterwards `ep_xdesc` holds every descriptor
    /// needed to open a pipe with [`usb_pipe_xopen`], including the
    /// companion descriptors that USB 3.0 added, and `ep_data` is no longer
    /// needed. `version` should always be `USB_EP_XDESCR_CURRENT_VERSION`.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_INVALID_ARGS` when the version
    /// is unknown or one of the pointers is not valid, and `USB_FAILURE` on
    /// an unknown error. The function is usually called from `attach(9E)`
    /// but may be called from user or kernel context.
    pub fn usb_ep_xdescr_fill(
        version: uint_t,
        dip: dev_info_t,
        ep_data: *mut usb_ep_data_t,
        ep_xdesc: *mut usb_ep_xdescr_t,
    ) -> c_int;

    /// `usb_get_addr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the current USB bus address of the device of `dip`, meant for
    /// debugging. The address is unique on one USB bus but may repeat when
    /// several host controllers are present, and it may change after the
    /// device is removed and inserted again. Returns 0 when `dip` is NULL.
    /// The function may be called from user, kernel or interrupt context.
    pub fn usb_get_addr(dip: dev_info_t) -> c_int;
    /// `usb_get_if_number(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the interface number the driver of `dip` is bound to. When the
    /// driver is responsible for the entire device the result is
    /// `USB_DEVICE_NODE` if the device node corresponds to the whole
    /// physical device, and `USB_COMBINED_NODE` if it is responsible for the
    /// entire active configuration without the node corresponding to the
    /// whole physical device. The function may be called from user or kernel
    /// context.
    pub fn usb_get_if_number(dip: dev_info_t) -> c_int;
    /// `usb_owns_device(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns `B_TRUE` when the driver of `dip` owns the entire device and
    /// `B_FALSE` when it owns only a particular interface. The function may
    /// be called from user or kernel context.
    pub fn usb_owns_device(dip: dev_info_t) -> boolean_t;

    /// `usb_pipe_get_state(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the state of the pipe `pipe_handle` in `*pipe_state`. `flags`
    /// is reserved and no flags are recognised. Returns `USB_SUCCESS` on
    /// success, `USB_INVALID_ARGS` when `pipe_state` is NULL and
    /// `USB_INVALID_PIPE` when `pipe_handle` is NULL. The function may be
    /// called from user, kernel or interrupt context.
    pub fn usb_pipe_get_state(
        pipe_handle: usb_pipe_handle_t,
        pipe_state: *mut usb_pipe_state_t,
        flags: usb_flags_t,
    ) -> c_int;
    /// `usb_pipe_open(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Opens a pipe, a logical connection, to the endpoint described by
    /// `ep` and stores its handle in `*pipe_handle`, or NULL on failure.
    /// This older form works only for devices up to USB 2.0; SuperSpeed
    /// devices need [`usb_pipe_xopen`], which also describes the arguments,
    /// the bandwidth rules and the return values. When the device of `dip`
    /// is at least a SuperSpeed device this function fails with
    /// `USB_FAILURE`.
    pub fn usb_pipe_open(
        dip: dev_info_t,
        ep: *mut usb_ep_descr_t,
        pipe_policy: *mut usb_pipe_policy_t,
        flags: usb_flags_t,
        pipe_handle: *mut usb_pipe_handle_t,
    ) -> c_int;
    /// `usb_pipe_xopen(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Opens a pipe, a logical connection, to the endpoint described by the
    /// extended descriptor `ep_xdesc`, which was filled in by
    /// [`usb_ep_xdescr_fill`], and stores an opaque handle to it in
    /// `*pipe_handle`, or NULL on failure. The transfer type of the pipe
    /// follows from `bmAttributes` of the endpoint descriptor. `pipe_policy`
    /// must be given and hints at how the pipe will be used. `flags` accepts
    /// only `USB_FLAGS_SLEEP`, which waits for memory. The default control
    /// pipe comes open from the hub and cannot be opened by a client, and
    /// all explicit opens are exclusive.
    ///
    /// The polling interval of periodic pipes, carried by `bInterval`, must
    /// be within range. Bandwidth for interrupt and isochronous pipes is
    /// allocated at open time, so the open fails when not enough bandwidth
    /// is left, and such pipes should only be opened when needed.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_NO_RESOURCES` or
    /// `USB_NO_BANDWIDTH` when resources or bandwidth are lacking,
    /// `USB_INVALID_CONTEXT` when called from an interrupt handler with
    /// `USB_FLAGS_SLEEP`, `USB_INVALID_ARGS` when `dip`, `pipe_handle` or
    /// `pipe_policy` is NULL, `USB_INVALID_PERM` when the endpoint is NULL,
    /// which stands for the default control pipe, `USB_NOT_SUPPORTED` for a
    /// periodic endpoint whose maximum packet size is 0,
    /// `USB_HC_HARDWARE_ERROR` when the host controller is in an error state
    /// and `USB_FAILURE` when the pipe is already open, the host controller
    /// is not operational or the polling interval is out of range. The
    /// function may be called from user or kernel context, and from
    /// interrupt context when `USB_FLAGS_SLEEP` is not set.
    pub fn usb_pipe_xopen(
        dip: dev_info_t,
        ep_xdesc: *mut usb_ep_xdescr_t,
        pipe_policy: *mut usb_pipe_policy_t,
        flags: usb_flags_t,
        pipe_handle: *mut usb_pipe_handle_t,
    ) -> c_int;
    /// `usb_pipe_close(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Closes the pipe `pipe_handle`, releases all related resources and
    /// frees the handle. Polling stops if the pipe is an interrupt IN or
    /// isochronous IN pipe. The function waits for pending requests to
    /// finish and after several seconds flushes the rest, whose exception
    /// callbacks see `USB_CR_FLUSHED`, while terminated periodic requests
    /// see `USB_CR_PIPE_CLOSING`. The default control pipe cannot be closed.
    ///
    /// With `USB_FLAGS_SLEEP` in `flags` the function waits for all cleanup
    /// to complete. Without it the close happens asynchronously in another
    /// thread, the function returns at once, and `cb`, which is then
    /// mandatory, is called with `cb_arg` when everything has completed. The
    /// status is only reported through the `rval` argument of the callback,
    /// which is `USB_INVALID_PIPE` for a closed or closing pipe,
    /// `USB_INVALID_ARGS` for NULL arguments, `USB_INVALID_CONTEXT` when
    /// called from interrupt context, `USB_INVALID_PERM` for the default
    /// control pipe and `USB_FAILURE` when asynchronous resources were
    /// lacking. Calling it with `USB_FLAGS_SLEEP` from a callback of the same
    /// pipe deadlocks. The function may be called from user or kernel
    /// context, but not from a callback running in interrupt context.
    pub fn usb_pipe_close(
        dip: dev_info_t,
        pipe_handle: usb_pipe_handle_t,
        flags: usb_flags_t,
        cb: Option<usb_cmd_completion_cb_t>,
        cb_arg: usb_opaque_t,
    );
    /// `usb_pipe_drain_reqs(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Waits until all requests pending on `pipe_handle` have completed and
    /// reports it synchronously or asynchronously. For a shared pipe, such
    /// as the default control pipe of a composite device, it only waits for
    /// the requests that belong to `dip`, which lets a close routine know
    /// that the default pipe, which a client cannot close, is clear. `time`
    /// is the maximum wait in seconds, and 0 means no timeout. With
    /// `USB_FLAGS_SLEEP` the function blocks; otherwise it returns at once
    /// and calls `cb` with `cb_arg` on completion.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_FAILURE` on timeout,
    /// `USB_INVALID_ARGS` when `dip` is NULL or no callback is given without
    /// `USB_FLAGS_SLEEP`, `USB_INVALID_CONTEXT` when called from callback
    /// context with `USB_FLAGS_SLEEP`, and `USB_INVALID_PIPE` when the pipe
    /// is not open or is closing. The function may be called from user or
    /// kernel context.
    pub fn usb_pipe_drain_reqs(
        dip: dev_info_t,
        pipe_handle: usb_pipe_handle_t,
        time: uint_t,
        flags: usb_flags_t,
        cb: Option<usb_cmd_completion_cb_t>,
        cb_arg: usb_opaque_t,
    ) -> c_int;
    /// `usb_pipe_reset(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Resets a pipe that is in an error state, or aborts the current
    /// request and clears the pipe. Polling stops on interrupt and
    /// isochronous pipes, all pending requests are removed with their
    /// exception callbacks called, and the pipe returns to the idle state.
    /// Stalls on requests without `USB_ATTRS_AUTOCLEARING` must be cleared
    /// this way. The default control pipe cannot be reset, and nothing is
    /// done on a closing pipe.
    ///
    /// With `USB_FLAGS_SLEEP` in `usb_flags` the function waits for the
    /// reset to complete; otherwise it queues the reset, returns at once and
    /// calls `cb` with `cb_arg` on completion, or gives no notification if
    /// `cb` is NULL. The status is reported through the `rval` argument of
    /// the callback. The function may be called from user or kernel context
    /// and from any callback without `USB_FLAGS_SLEEP`, but not with
    /// `USB_FLAGS_SLEEP` from a callback running in interrupt context.
    pub fn usb_pipe_reset(
        dip: dev_info_t,
        pipe_handle: usb_pipe_handle_t,
        usb_flags: usb_flags_t,
        cb: Option<usb_cmd_completion_cb_t>,
        cb_arg: usb_opaque_t,
    );
    /// `usb_pipe_set_private(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores `data` in the user private field of the pipe `pipe_handle`.
    /// The framework never uses the field, and drivers often keep their soft
    /// state there so that callbacks can find it. Returns `USB_SUCCESS` on
    /// success, `USB_INVALID_PIPE` when the handle is NULL or not valid or
    /// the pipe is closing or closed, and `USB_INVALID_PERM` for the default
    /// control pipe. The function may be called from user, kernel or
    /// interrupt context.
    pub fn usb_pipe_set_private(pipe_handle: usb_pipe_handle_t, data: usb_opaque_t) -> c_int;
    /// `usb_pipe_get_private(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the user private data stored in the pipe `pipe_handle` with
    /// [`usb_pipe_set_private`], or NULL when the handle is NULL or not
    /// valid or the pipe is closing or closed. The function may be called
    /// from user, kernel or interrupt context.
    pub fn usb_pipe_get_private(pipe_handle: usb_pipe_handle_t) -> usb_opaque_t;

    /// `usb_alloc_ctrl_req(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates a control request for the driver of `dip` and, when `len`
    /// is not 0, a message block of `len` bytes for its data. `flags`
    /// accepts only `USB_FLAGS_SLEEP`, which waits for resources. With valid
    /// arguments and `USB_FLAGS_SLEEP` outside interrupt context the call
    /// always succeeds. Returns the request, or NULL when `dip` is not
    /// valid, memory is lacking without `USB_FLAGS_SLEEP`, or
    /// `USB_FLAGS_SLEEP` was passed in interrupt context. The function may
    /// be called from kernel and user context, and from interrupt context
    /// without `USB_FLAGS_SLEEP`.
    pub fn usb_alloc_ctrl_req(dip: dev_info_t, len: size_t, flags: usb_flags_t) -> *mut usb_ctrl_req_t;
    /// `usb_free_ctrl_req(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the control request `reqp`, which may be NULL, together with
    /// its data block if it has one. The function may be called from
    /// kernel, user and interrupt context.
    pub fn usb_free_ctrl_req(reqp: *mut usb_ctrl_req_t);
    /// `usb_pipe_ctrl_xfer(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the framework to perform the control transfer `reqp` on the
    /// control pipe `pipe_handle`. The request goes to the host controller
    /// driver, which performs the transactions. With `USB_FLAGS_SLEEP` the
    /// call returns only when the transfer has completed; otherwise
    /// completion is reported through the callbacks in the request.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_INVALID_ARGS` when the request
    /// is NULL, `USB_INVALID_CONTEXT` when called from interrupt context
    /// with `USB_FLAGS_SLEEP`, `USB_INVALID_REQUEST` when the request was
    /// freed, has conflicting attributes, lacks callbacks for an
    /// asynchronous call or has no data space for a nonzero `ctrl_wLength`,
    /// `USB_INVALID_PIPE` when the handle is not valid or the pipe is
    /// closing, `USB_NO_RESOURCES` when resources are lacking,
    /// `USB_HC_HARDWARE_ERROR` when the host controller is in an error state
    /// and `USB_FAILURE` when an asynchronous transfer or internal operation
    /// failed or the pipe is in an unsuitable state. More detail may be
    /// found in the completion reason and callback flags of the request. The
    /// function may be called from kernel or user context, and from
    /// interrupt context without `USB_FLAGS_SLEEP`.
    pub fn usb_pipe_ctrl_xfer(pipe_handle: usb_pipe_handle_t, reqp: *mut usb_ctrl_req_t, flags: usb_flags_t) -> c_int;
    /// `usb_pipe_ctrl_xfer_wait(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A wrapper around [`usb_pipe_ctrl_xfer`] for simple synchronous control
    /// transactions, which allocates the request, performs the transfer on
    /// `pipe_handle` and frees the request again, with
    /// `USB_ATTRS_AUTOCLEARING` enabled. `setup` gives the setup parameters.
    /// `data` points to the message block with the data for an OUT
    /// transfer, or receives the data of an IN transfer, which the caller
    /// then has to free; it is ignored if NULL. `completion_reason` and
    /// `cb_flags` receive the completion status and the recovery flags and
    /// are ignored if NULL. No flags are recognised.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_INVALID_CONTEXT` when called
    /// from interrupt context, `USB_INVALID_ARGS` when `dip` is NULL, or any
    /// error of [`usb_pipe_ctrl_xfer`]. On failure the completion reason and
    /// callback flags help decide on recovery. The function may be called
    /// from kernel or user context.
    pub fn usb_pipe_ctrl_xfer_wait(
        pipe_handle: usb_pipe_handle_t,
        setup: *mut usb_ctrl_setup_t,
        data: *mut *mut mblk_t,
        completion_reason: *mut usb_cr_t,
        cb_flags: *mut usb_cb_flags_t,
        flags: usb_flags_t,
    ) -> c_int;
    /// `usb_get_status(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads the status of a device, interface or endpoint over the default
    /// control pipe `ph` and stores it in `*status`. `type` is
    /// `USB_DEV_REQ_RCPT_DEV`, `USB_DEV_REQ_RCPT_IF` or
    /// `USB_DEV_REQ_RCPT_EP`, and `what` is the interface or endpoint
    /// number, or 0 for the device. A device status contains
    /// `USB_DEV_SLF_PWRD_STATUS` and `USB_DEV_RWAKEUP_STATUS`, an endpoint
    /// status contains `USB_EP_HALT_STATUS`, and an interface status is
    /// always `USB_IF_STATUS`. The function always blocks, whatever `flags`
    /// says.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_INVALID_ARGS` when `status` or
    /// `dip` is NULL, `USB_INVALID_PIPE` when `ph` is NULL and `USB_FAILURE`
    /// otherwise. The function may be called from user or kernel context.
    pub fn usb_get_status(
        dip: dev_info_t,
        ph: usb_pipe_handle_t,
        r#type: uint_t,
        what: uint_t,
        status: *mut u16,
        flags: usb_flags_t,
    ) -> c_int;
    /// `usb_clr_feature(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Clears `feature` on a device, interface or endpoint. `type` is
    /// `USB_DEV_REQ_RCPT_DEV`, `USB_DEV_REQ_RCPT_IF` or
    /// `USB_DEV_REQ_RCPT_EP`, and `what` is the interface or endpoint
    /// number, or 0 for the device. The feature may be any device defined
    /// feature, including `USB_EP_HALT`, `USB_DEV_REMOTE_WAKEUP` and
    /// `USB_DEV_TEST_MODE`. With `USB_FLAGS_SLEEP` the call blocks until
    /// completion; otherwise it returns at once and calls `cb` with `cb_arg`
    /// when done.
    ///
    /// Returns `USB_SUCCESS` when the feature was cleared, `USB_INVALID_ARGS`
    /// when `dip` is NULL, `USB_INVALID_PIPE` when the pipe handle is NULL,
    /// `USB_INVALID_CONTEXT` when called from interrupt context with
    /// `USB_FLAGS_SLEEP`, and `USB_FAILURE` otherwise. The function may be
    /// called from user or kernel context, and from interrupt context
    /// without `USB_FLAGS_SLEEP`.
    pub fn usb_clr_feature(
        dip: dev_info_t,
        r#type: uint_t,
        feature: uint_t,
        what: uint_t,
        flags: usb_flags_t,
        cb: Option<usb_cmd_completion_cb_t>,
        cb_arg: usb_opaque_t,
    ) -> c_int;
    /// `usb_set_cfg(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets the configuration with index `cfg_index` in the descriptor tree,
    /// or the default configuration for `USB_DEV_DEFAULT_CONFIG_INDEX`. The
    /// device must be quiescent with every pipe except the default control
    /// pipe closed, and the driver must own the whole device exclusively,
    /// without child nodes operated by other drivers. This is the only
    /// supported way to change the configuration, because a raw
    /// `USB_REQ_SET_CFG` request does not update the framework. With
    /// `USB_FLAGS_SLEEP` the call blocks; otherwise it returns at once and
    /// calls `cb` with `cb_arg` on completion, passing the default pipe
    /// handle.
    ///
    /// Returns `USB_SUCCESS` when the configuration was set,
    /// `USB_INVALID_ARGS` when `dip` is NULL or no callback is given without
    /// `USB_FLAGS_SLEEP`, `USB_INVALID_CONTEXT` when called from interrupt
    /// context with `USB_FLAGS_SLEEP`, `USB_INVALID_PERM` when the caller
    /// does not own the whole device or has children, `USB_BUSY` when other
    /// pipes are open, `USB_INVALID_PIPE` for a bad or closing pipe, and
    /// `USB_FAILURE` for an illegal configuration. The function may be
    /// called from user or kernel context, and from interrupt context
    /// without `USB_FLAGS_SLEEP`.
    pub fn usb_set_cfg(
        dip: dev_info_t,
        cfg_index: uint_t,
        usb_flags: usb_flags_t,
        cb: Option<usb_cmd_completion_cb_t>,
        cb_arg: usb_opaque_t,
    ) -> c_int;
    /// `usb_get_cfg(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads the current configuration value of the device and stores it in
    /// `*cfgval`. The manual page shows `cfgval` as a plain `uint_t`, but
    /// the header and the description make it a pointer. The function
    /// ignores `usb_flags` and always blocks while it contacts the device.
    /// Returns `USB_SUCCESS` on success, `USB_INVALID_ARGS` when `cfgval` or
    /// `dip` is NULL and `USB_FAILURE` when the configuration cannot be
    /// read. The function may be called from user or kernel context.
    pub fn usb_get_cfg(dip: dev_info_t, cfgval: *mut uint_t, usb_flags: usb_flags_t) -> c_int;
    /// `usb_set_alt_if(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the device to select the alternate setting `alt_number` of the
    /// interface `interface`. All pipes except the default control pipe must
    /// be closed and quiescent, and the driver must be bound to the whole
    /// device, to the interface association that contains the interface, or
    /// to the interface itself. With `USB_FLAGS_SLEEP` the call blocks;
    /// otherwise it returns at once and calls `cb` with `cb_arg` on
    /// completion, passing the default pipe handle.
    ///
    /// Returns `USB_SUCCESS` when the alternate was set, `USB_INVALID_ARGS`
    /// when `dip` is NULL or no callback is given without `USB_FLAGS_SLEEP`,
    /// `USB_INVALID_PERM` when the driver does not own the interface,
    /// `USB_INVALID_CONTEXT` when called from interrupt context with
    /// `USB_FLAGS_SLEEP`, `USB_INVALID_PIPE` for a bad or closing pipe, and
    /// `USB_FAILURE` when the interface or alternate is not valid, pipes are
    /// open or an access error occurred. The function may be called from
    /// user or kernel context, and from interrupt context without
    /// `USB_FLAGS_SLEEP`.
    pub fn usb_set_alt_if(
        dip: dev_info_t,
        interface: uint_t,
        alt_number: uint_t,
        usb_flags: usb_flags_t,
        cb: Option<usb_cmd_completion_cb_t>,
        cb_arg: usb_opaque_t,
    ) -> c_int;
    /// `usb_get_alt_if(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the device for the current alternate setting of the interface
    /// `if_number` and stores it in `*alt_number`. The function ignores
    /// `flags` and always blocks. Returns `USB_SUCCESS` on success,
    /// `USB_INVALID_ARGS` when `alt_number` or `dip` is NULL,
    /// `USB_INVALID_CONTEXT` when called from interrupt context, and
    /// `USB_FAILURE` when the interface number is not valid or an access
    /// error occurred. The function may be called from user or kernel
    /// context.
    pub fn usb_get_alt_if(dip: dev_info_t, if_number: uint_t, alt_number: *mut uint_t, flags: usb_flags_t) -> c_int;

    /// `usb_alloc_bulk_req(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates a bulk request for the driver of `dip` and, when `len` is
    /// not 0, a message block of `len` bytes for its data. The rules for
    /// `flags`, the return value and the calling context are those of
    /// [`usb_alloc_ctrl_req`].
    pub fn usb_alloc_bulk_req(dip: dev_info_t, len: size_t, flags: usb_flags_t) -> *mut usb_bulk_req_t;
    /// `usb_free_bulk_req(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the bulk request `reqp`, which may be NULL, together with its
    /// data block if it has one. The function may be called from kernel,
    /// user and interrupt context.
    pub fn usb_free_bulk_req(reqp: *mut usb_bulk_req_t);
    /// `usb_pipe_bulk_xfer(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the framework to perform the bulk transfer `reqp` on the bulk
    /// pipe `pipe_handle`. With `USB_FLAGS_SLEEP` the call returns only when
    /// the transfer has completed; otherwise completion is reported through
    /// the callbacks in the request. The request must carry a message block
    /// for the data.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_INVALID_ARGS` when the request
    /// is NULL, `USB_INVALID_CONTEXT` when called from interrupt context
    /// with `USB_FLAGS_SLEEP`, `USB_INVALID_REQUEST` when the request was
    /// freed, has conflicting attributes, lacks callbacks for an
    /// asynchronous call or has no data for a nonzero length,
    /// `USB_INVALID_PIPE` for a bad or closing pipe, `USB_PIPE_ERROR` when
    /// the pipe is in the error state, `USB_NO_RESOURCES` when resources are
    /// lacking, `USB_HC_HARDWARE_ERROR` when the host controller is in an
    /// error state, and `USB_FAILURE` when the transfer failed, asked for
    /// more data than the maximum bulk transfer size, or the pipe is in an
    /// unsuitable state. The function may be called from kernel or user
    /// context, and from interrupt context without `USB_FLAGS_SLEEP`.
    pub fn usb_pipe_bulk_xfer(pipe_handle: usb_pipe_handle_t, reqp: *mut usb_bulk_req_t, flags: usb_flags_t) -> c_int;
    /// `usb_pipe_get_max_bulk_transfer_size(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores in `*size` the largest number of bytes the host controller
    /// driver can transfer in one bulk request, so that larger transfers can
    /// be split. The manual page shows `dip` without the pointer, but it is
    /// the usual `dev_info_t` pointer. Returns `USB_SUCCESS` on success,
    /// `USB_INVALID_ARGS` when `dip` or `size` is NULL and `USB_FAILURE`,
    /// with 0 stored in `*size`, when the size cannot be determined. The
    /// function may be called from user, kernel or interrupt context.
    pub fn usb_pipe_get_max_bulk_transfer_size(dip: dev_info_t, size: *mut size_t) -> c_int;

    /// `usb_alloc_intr_req(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates an interrupt request for the driver of `dip` and, when
    /// `len` is not 0, a message block of `len` bytes for its data. A
    /// request that starts IN polling should be allocated with a length of
    /// 0, because the framework allocates new requests for the data. The
    /// rules for `flags`, the return value and the calling context are those
    /// of [`usb_alloc_ctrl_req`].
    pub fn usb_alloc_intr_req(dip: dev_info_t, len: size_t, flags: usb_flags_t) -> *mut usb_intr_req_t;
    /// `usb_free_intr_req(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the interrupt request `reqp`, which may be NULL, together with
    /// its data block if it has one. Every request that polling returns has
    /// to be freed this way. The function may be called from kernel, user
    /// and interrupt context.
    pub fn usb_free_intr_req(reqp: *mut usb_intr_req_t);
    /// `usb_pipe_intr_xfer(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the framework to perform the interrupt transfer `req` on the
    /// interrupt pipe `pipe_handle`. An IN request without
    /// `USB_ATTRS_ONE_XFER` starts periodic polling at the rate of
    /// `bInterval`. Polling is always asynchronous, only one polling request
    /// can be active at a time, callbacks are required, and
    /// `USB_FLAGS_SLEEP` only waits for resources. Each poll with new data
    /// arrives in a new request allocated by the framework. The original
    /// request returns when polling ends or, with autoclearing, on an error;
    /// the callback can recognise it by a completion reason of
    /// `USB_CR_STOPPED_POLLING`, `USB_CR_PIPE_RESET`, `USB_CR_PIPE_CLOSING`
    /// or `USB_CR_NOT_SUPPORTED` and may restart polling from there. An IN
    /// request with `USB_ATTRS_ONE_XFER` performs one transfer, and with
    /// `USB_FLAGS_SLEEP` waits for it. OUT requests are single transfers
    /// that can be queued, and with `USB_FLAGS_SLEEP` wait for completion.
    /// Starting polling on a pipe that is already polled does nothing.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_INVALID_ARGS` when the request
    /// is NULL, `USB_INVALID_CONTEXT` when called from interrupt context
    /// with `USB_FLAGS_SLEEP`, `USB_INVALID_REQUEST` for a freed request,
    /// conflicting attributes, missing callbacks, data given for an IN
    /// request or missing for an OUT request, or a timeout on a polling
    /// request, `USB_INVALID_PIPE` for a bad or closing pipe,
    /// `USB_PIPE_ERROR` when the pipe is in the error state,
    /// `USB_NO_RESOURCES` when resources are lacking, `USB_HC_HARDWARE_ERROR`
    /// when the host controller is in an error state, and `USB_FAILURE` when
    /// the transfer failed, polling is already in progress or the pipe is in
    /// an unsuitable state. The function may be called from kernel or user
    /// context, and from interrupt context without `USB_FLAGS_SLEEP`.
    pub fn usb_pipe_intr_xfer(pipe_handle: usb_pipe_handle_t, req: *mut usb_intr_req_t, flags: usb_flags_t) -> c_int;
    /// `usb_pipe_stop_intr_polling(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stops polling on the interrupt IN pipe `pipe_handle` without closing
    /// it. Requests in progress complete through the normal callback, the
    /// pipe goes idle, and the original polling request comes back through
    /// the exception callback with `USB_CR_STOPPED_POLLING`. With
    /// `USB_FLAGS_SLEEP` the call waits until polling has stopped and all
    /// callbacks have completed. It returns nothing but fails, logging a
    /// message, when called with `USB_FLAGS_SLEEP` from interrupt context,
    /// for a bad or closing pipe, or for a pipe in the error state. The
    /// function may be called from kernel or user context, and from
    /// interrupt context without `USB_FLAGS_SLEEP`.
    pub fn usb_pipe_stop_intr_polling(pipe_handle: usb_pipe_handle_t, flags: usb_flags_t);

    /// `usb_alloc_isoc_req(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates an isochronous request for the driver of `dip` with
    /// `isoc_pkts_count` packet descriptors placed after the request itself,
    /// and, when `len` is not 0, a message block of `len` bytes for its
    /// data. `isoc_pkts_count` must be greater than 0. A request that starts
    /// IN polling should be allocated with a length of 0. The rules for
    /// `flags`, the return value and the calling context are those of
    /// [`usb_alloc_ctrl_req`].
    pub fn usb_alloc_isoc_req(
        dip: dev_info_t,
        isoc_pkts_count: uint_t,
        len: size_t,
        flags: usb_flags_t,
    ) -> *mut usb_isoc_req_t;
    /// `usb_free_isoc_req(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the isochronous request `usb_isoc_req`, which may be NULL,
    /// together with its data block and its array of packet descriptors.
    /// The function may be called from kernel, user and interrupt context.
    pub fn usb_free_isoc_req(usb_isoc_req: *mut usb_isoc_req_t);
    /// `usb_get_current_frame_number(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the current logical USB frame number, which in illumos is a
    /// running count of milliseconds since boot. Adding an offset of
    /// typically 4 to 10 frames gives a suitable start frame for an
    /// isochronous request with `USB_ATTRS_ISOC_START_FRAME`. Returns 0 when
    /// `dip` is NULL. The function may be called from user, kernel or
    /// interrupt context.
    pub fn usb_get_current_frame_number(dip: dev_info_t) -> usb_frame_number_t;
    /// `usb_get_max_pkts_per_isoc_request(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the largest number of isochronous packets per request that the
    /// host controller driver supports. Multiplied by `wMaxPacketSize` of the
    /// endpoint it gives the largest amount of data one isochronous request
    /// can carry. Returns 0 when `dip` is NULL. The function may be called
    /// from user, kernel or interrupt context.
    pub fn usb_get_max_pkts_per_isoc_request(dip: dev_info_t) -> uint_t;
    /// `usb_pipe_isoc_xfer(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the framework to perform the isochronous transfer `reqp` on the
    /// isochronous pipe `pipe_handle`. Each request consists of several
    /// packets that are transferred in consecutive frames, and the request
    /// is complete when all of them have been processed. All isochronous
    /// requests are asynchronous and need both callbacks; `USB_FLAGS_SLEEP`
    /// only waits for resources. Errors do not abort the transfer and are
    /// recorded in `isoc_pkt_status` of each packet. An OUT request is
    /// completed through the normal callback once all data has been sent.
    /// An IN request starts background polling, only one can be active at a
    /// time, the framework allocates a new request for each batch of data,
    /// and the original request returns through the exception callback when
    /// polling stops. Starting polling on a pipe that is already polled does
    /// nothing, and OUT requests can be queued.
    ///
    /// Returns `USB_SUCCESS` on success, `USB_INVALID_ARGS` when the request
    /// is NULL, `USB_INVALID_CONTEXT` when called from interrupt context
    /// with `USB_FLAGS_SLEEP`, `USB_INVALID_REQUEST` for a freed request,
    /// conflicting attributes, missing callbacks, missing packet descriptors
    /// or data, or a nonzero frame number with `USB_ATTRS_ISOC_XFER_ASAP`,
    /// `USB_NO_FRAME_NUMBER` when not exactly one of the start frame and as
    /// soon as possible attributes is set or the start frame is 0,
    /// `USB_INVALID_START_FRAME` when the start frame has passed,
    /// `USB_INVALID_PIPE` for a bad or closing pipe, `USB_PIPE_ERROR` when
    /// the pipe is in the error state, `USB_NO_RESOURCES` when resources are
    /// lacking, `USB_HC_HARDWARE_ERROR` when the host controller is in an
    /// error state, and `USB_FAILURE` when the transfer failed, asked for
    /// more data than the packet limit allows or the pipe is in an
    /// unsuitable state. The function may be called from kernel or user
    /// context, and from interrupt context without `USB_FLAGS_SLEEP`.
    pub fn usb_pipe_isoc_xfer(pipe_handle: usb_pipe_handle_t, reqp: *mut usb_isoc_req_t, flags: usb_flags_t) -> c_int;
    /// `usb_pipe_stop_isoc_polling(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stops polling on the isochronous IN pipe `pipe_handle` without
    /// closing it. Requests in progress complete through the normal
    /// callback, the pipe goes idle, and the original polling request comes
    /// back through the exception callback with `USB_CR_STOPPED_POLLING`.
    /// With `USB_FLAGS_SLEEP` the call waits until polling has stopped and
    /// all callbacks have completed. It returns nothing but fails, logging a
    /// message, when called with `USB_FLAGS_SLEEP` from interrupt context,
    /// for a bad or closing pipe, or for a pipe in the error state. The
    /// function may be called from kernel or user context, and from
    /// interrupt context without `USB_FLAGS_SLEEP`.
    pub fn usb_pipe_stop_isoc_polling(pipe_handle: usb_pipe_handle_t, flags: usb_flags_t);

    /// `usb_handle_remote_wakeup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Enables or disables remote wakeup on the device of `dip`, depending
    /// on whether `cmd` is `USB_REMOTE_WAKEUP_ENABLE` or
    /// `USB_REMOTE_WAKEUP_DISABLE`. The call can block. Returns
    /// `USB_SUCCESS` on success and `USB_FAILURE` when the device does not
    /// support remote wakeup or an internal error occurred. The function may
    /// be called from user or kernel context.
    pub fn usb_handle_remote_wakeup(dip: dev_info_t, cmd: c_int) -> c_int;
    /// `usb_create_pm_components(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates the power management component properties of `dip` for the
    /// standard USB power levels D0 to D3, updates its `pm-components`
    /// property, and stores in `*pwrstates` a mask of the power states the
    /// device supports, built from the `USB_DEV_OS_PWRMASK_*` bits. The USB
    /// power levels run opposite to the operating system levels, so D3 and
    /// `USB_DEV_OS_PWR_0` both mean powered down. Returns `USB_SUCCESS` when
    /// the system recognises the power management facilities of the device
    /// and `USB_FAILURE` on error. The function may be called from user or
    /// kernel context.
    pub fn usb_create_pm_components(dip: dev_info_t, pwrstates: *mut uint_t) -> c_int;
    /// `usb_register_hotplug_cbs(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers `disconnect_event_handler` and `reconnect_event_handler` to
    /// be called when the device of `dip` is removed or plugged in again.
    /// Returns `USB_SUCCESS` when the callbacks were registered and
    /// `USB_FAILURE` when an argument is NULL or the registration failed.
    /// The function may be called only from `attach(9E)`.
    pub fn usb_register_hotplug_cbs(
        dip: dev_info_t,
        disconnect_event_handler: Option<usb_hotplug_cb_t>,
        reconnect_event_handler: Option<usb_hotplug_cb_t>,
    ) -> c_int;
    /// `usb_unregister_hotplug_cbs(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unregisters the hotplug callbacks of `dip`, so that they no longer run
    /// when the device is removed or plugged in. The function may be called
    /// only from `detach(9E)`.
    pub fn usb_unregister_hotplug_cbs(dip: dev_info_t);
    /// `usb_reset_device(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Requests a hardware reset of the device of `dip`, for example to
    /// recover from an error state or to activate firmware uploaded into the
    /// RAM of the device. `reset_level` selects whether the driver stays
    /// attached or the device is enumerated again. For a reattach a helper
    /// thread is created; when called from `attach(9E)` it waits one second
    /// and until attach has completed before reattaching, and otherwise it
    /// reattaches immediately. Failures of the reattach are logged.
    ///
    /// Returns `USB_SUCCESS` when the reset succeeded or the reattach was
    /// started or is already in progress, `USB_FAILURE` when the parent hub
    /// is disconnected or suspended, the driver is detaching or the reset
    /// failed, `USB_INVALID_ARGS` for invalid arguments, `USB_INVALID_PERM`
    /// when the driver does not own the entire device, `USB_BUSY` when pipes
    /// other than the default control pipe are open for a default reset,
    /// and `USB_INVALID_CONTEXT` when a default reset is requested from
    /// interrupt context.
    pub fn usb_reset_device(dip: dev_info_t, reset_level: usb_dev_reset_lvl_t) -> c_int;
}
