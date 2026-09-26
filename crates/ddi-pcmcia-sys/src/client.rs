//! Client registration and the per client requests of `sys/cs.h` and
//! `sys/cs_types.h`.
//!
//! A PC Card driver instance becomes a Card Services client with
//! [`csx_RegisterClient`], which returns the [`client_handle_t`] that almost
//! every other function takes. Registration names the event handler and the
//! global event mask, but events only start to flow once
//! [`csx_RequestSocketMask`] has set the client event mask, which gives the
//! driver time to initialise the mutexes its handler needs. Afterwards
//! [`csx_SetEventMask`] and [`csx_GetEventMask`] change or read either mask,
//! [`csx_GetStatus`] reports the state of the card and socket, and
//! [`csx_MapLogSocket`] gives the physical socket number for messages and
//! minor node names. [`csx_DeregisterClient`] ends the registration once
//! every resource has been released.
//!
//! The module also holds the illumos extensions that help a driver fit
//! into the DDI. [`csx_CS_DDI_Info`] finds the device node and instance of a
//! client for `getinfo(9E)`, [`csx_MakeDeviceNode`] and
//! [`csx_RemoveDeviceNode`] create and remove minor nodes on the client's
//! behalf, and [`csx_GetFirstClient`] and [`csx_GetNextClient`] walk the
//! registered clients.
//!
//! The `CS_SOCK_STATUS_*` names are declared with the spelling of
//! `sys/cs.h`, which writes several of them with `CHNAGE` where the manual
//! page has `CHANGE`, and calls the card detect bit
//! [`CS_SOCK_STATUS_CARD_DETECT_CHNAGE`] where the page has
//! `CS_SOCK_STATUS_CARD_INSERTION_CHANGE`. The client type flags
//! `INFO_SOCKET_SERVICES` and `INFO_CSI_CLIENT` are used by Socket Services
//! and the Card Services interface driver, not by card drivers, and are
//! left out, as are the retired names `sockmask_t`, `struct devnode_desc`
//! and `REMOVAL_ALL_DEVICE_NODES`.

use ddi_sys::{c_char, c_void, ddi_iblock_cookie_t, dev_info_t};

use crate::event::{
    CS_EVENT_BATTERY_DEAD, CS_EVENT_BATTERY_LOW, CS_EVENT_CARD_INSERTION, CS_EVENT_CARD_LOCK, CS_EVENT_CARD_READY,
    CS_EVENT_EJECTION_REQUEST, CS_EVENT_INSERTION_REQUEST, CS_EVENT_WRITE_PROTECT, MODMAXNAMELEN,
    event_callback_args_t,
};

/// `csx_RegisterClient(9F)`. illumos DDI Specific (illumos DDI).
///
/// The handle of a registered client, a `uint32_t` in C.
/// [`csx_RegisterClient`] returns it, most other `csx_*` functions take it,
/// and [`csx_DeregisterClient`] ends it.
pub type client_handle_t = u32;

/// `csx_RegisterClient(9F)`. illumos DDI Specific (illumos DDI).
///
/// The function type that `sys/cs.h` uses for the callbacks it stores,
/// `int32_t (csfunction_t)(int32_t, ...)`. The header declares it variadic
/// to quiet `lint`, and the event handler and interrupt handler members of
/// [`client_reg_t`] and [`irq_req_t`](crate::io::irq_req_t) are pointers to
/// it. The real handlers have fixed signatures, such as
/// [`csx_event_handler_t`](crate::event::csx_event_handler_t), and are cast
/// to this type when they are stored.
pub type csfunction_t = unsafe extern "C" fn(arg: i32, ...) -> i32;

ddi_sys::opaque! {
    /// `csx_RegisterClient(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// The interrupt device cookie, `ddi_idevice_cookie(9S)`, that Card
    /// Services points to from [`client_reg_t::idev_cookie`] and
    /// [`irq_req_t::idev_cookie`](crate::io::irq_req_t::idev_cookie). Its
    /// page is obsolete along with the interrupt interfaces that used it, so
    /// neither `ddi_sys` nor this crate declares its members, and a driver
    /// has no use for them with the current interrupt interfaces.
    ddi_idevice_cookie_t;
}

// --- client_reg_t --------------------------------------------------------

/// `csx_RegisterClient(9F)`. illumos DDI Specific (illumos DDI).
///
/// Describes a client to [`csx_RegisterClient`]. The client sets its type,
/// its global event mask, its event handler and the data passed to it, the
/// Card Services version it expects, its device node and its driver name.
/// Card Services fills in the cookie pointers the event handler needs for
/// its mutexes.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct client_reg_t {
    /// The client type and card sharing flags. Exactly one of
    /// [`INFO_MEM_CLIENT`], [`INFO_MTD_CLIENT`] and [`INFO_IO_CLIENT`] must
    /// be set. If [`INFO_CARD_SHARE`] or [`INFO_CARD_EXCL`] is also set, the
    /// client receives `CS_EVENT_REGISTRATION_COMPLETE` once registration is
    /// complete and [`csx_RequestSocketMask`] has succeeded, and an
    /// artificial `CS_EVENT_CARD_INSERTION` if a card it can control is
    /// already in the socket.
    pub Attributes: u32,
    /// The client's global event mask, a combination of the `CS_EVENT_*`
    /// bits. Card Services notifies the client of the events in it.
    pub EventMask: u32,
    /// The arguments passed to the event handler. The client sets
    /// [`client_data`](event_callback_args_t::client_data), typically to its
    /// soft state pointer.
    pub event_callback_args: event_callback_args_t,
    /// The Card Services version the client expects, normally
    /// [`CS_VERSION`](crate::consts::CS_VERSION).
    pub Version: u32,
    /// The client's event handler, a
    /// [`csx_event_handler_t`](crate::event::csx_event_handler_t) cast to
    /// [`csfunction_t`].
    pub event_handler: Option<csfunction_t>,
    /// Set by Card Services to point at the interrupt block cookie that the
    /// client must use to initialise the mutex that protects high priority
    /// events in its event handler.
    pub iblk_cookie: *mut ddi_iblock_cookie_t,
    /// Set by Card Services to point at the interrupt device cookie for the
    /// same purpose. See [`ddi_idevice_cookie_t`].
    pub idev_cookie: *mut ddi_idevice_cookie_t,
    /// The client's device node, which the client must set.
    pub dip: dev_info_t,
    /// A name unique to the driver, which the client must copy in. It must
    /// be the same for every instance of the driver.
    pub driver_name: [c_char; MODMAXNAMELEN],
    /// Private to Card Services. The C member is named `priv`, which is a
    /// reserved word in Rust.
    pub priv_: *mut c_void,
}

// Layout on amd64 with LP64. Attributes 4 at 0, EventMask 4 at 4,
// event_callback_args 256 aligned to 8 at 8, which ends at 264, Version 4 at
// 264 and 4 bytes of padding, then the pointers event_handler at 272,
// iblk_cookie at 280, idev_cookie at 288 and dip at 296, driver_name 32 at
// 304 and priv 8 at 336; total 344.
const _: () = assert!(core::mem::size_of::<client_reg_t>() == 344);
const _: () = assert!(core::mem::offset_of!(client_reg_t, Attributes) == 0);
const _: () = assert!(core::mem::offset_of!(client_reg_t, event_callback_args) == 8);
const _: () = assert!(core::mem::offset_of!(client_reg_t, Version) == 264);
const _: () = assert!(core::mem::offset_of!(client_reg_t, event_handler) == 272);
const _: () = assert!(core::mem::offset_of!(client_reg_t, iblk_cookie) == 280);
const _: () = assert!(core::mem::offset_of!(client_reg_t, idev_cookie) == 288);
const _: () = assert!(core::mem::offset_of!(client_reg_t, dip) == 296);
const _: () = assert!(core::mem::offset_of!(client_reg_t, driver_name) == 304);
const _: () = assert!(core::mem::offset_of!(client_reg_t, priv_) == 336);

/// The client drives an I/O card (`INFO_IO_CLIENT`).
pub const INFO_IO_CLIENT: u32 = 0x0000_0002;
/// The client is a memory technology driver (`INFO_MTD_CLIENT`).
pub const INFO_MTD_CLIENT: u32 = 0x0000_0004;
/// The client drives a memory card (`INFO_MEM_CLIENT`).
pub const INFO_MEM_CLIENT: u32 = 0x0000_0008;
/// Asks for the artificial registration complete and card insertion
/// events. The header keeps it for compatibility with the PC Card standard
/// and treats it the same as [`INFO_CARD_EXCL`] (`INFO_CARD_SHARE`).
pub const INFO_CARD_SHARE: u32 = 0x0000_0020;
/// Asks for the artificial registration complete and card insertion
/// events. The header keeps it for compatibility with the PC Card standard
/// and treats it the same as [`INFO_CARD_SHARE`] (`INFO_CARD_EXCL`).
pub const INFO_CARD_EXCL: u32 = 0x0000_0040;

// --- get_status_t --------------------------------------------------------

/// `csx_GetStatus(9F)`. illumos DDI Specific (illumos DDI).
///
/// The status of a card and its socket as [`csx_GetStatus`] returns it.
/// The status bits share their values with the `CS_EVENT_*` bits where the
/// meaning is the same.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct get_status_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// What Card Services believes the current state of the card is, a
    /// combination of the `CS_STATUS_*` bits. [`CS_STATUS_CARD_INSERTED`]
    /// says whether the card belonging to this driver instance is inserted,
    /// not just any card. For an I/O card the state comes from the pin
    /// replacement register and, if present, the extended status register,
    /// and a bit those registers lack is simulated as not write protected,
    /// power good, ready, or not set.
    pub CardState: u32,
    /// The latched state of card and socket, a combination of the
    /// `CS_SOCK_STATUS_*` bits. It may differ from
    /// [`CardState`](Self::CardState), and clients should normally rely on
    /// that member instead.
    pub SocketState: u32,
    /// An illumos extension with the same bits as
    /// [`CardState`](Self::CardState), except that
    /// [`CS_STATUS_CARD_INSERTED`] is set whenever any card is in the
    /// socket.
    pub raw_CardState: u32,
}

// Layout on amd64 with LP64. Four uint32_t at 0, 4, 8 and 12; total 16.
const _: () = assert!(core::mem::size_of::<get_status_t>() == 16);
const _: () = assert!(core::mem::offset_of!(get_status_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(get_status_t, raw_CardState) == 12);

/// The card is write protected. It has the value of
/// [`CS_EVENT_WRITE_PROTECT`] (`CS_STATUS_WRITE_PROTECTED`).
pub const CS_STATUS_WRITE_PROTECTED: u32 = CS_EVENT_WRITE_PROTECT;
/// The card is locked. It has the value of [`CS_EVENT_CARD_LOCK`]
/// (`CS_STATUS_CARD_LOCKED`).
pub const CS_STATUS_CARD_LOCKED: u32 = CS_EVENT_CARD_LOCK;
/// An ejection request is in progress. It has the value of
/// [`CS_EVENT_EJECTION_REQUEST`] (`CS_STATUS_EJECTION_REQUEST`).
pub const CS_STATUS_EJECTION_REQUEST: u32 = CS_EVENT_EJECTION_REQUEST;
/// An insertion request is in progress. It has the value of
/// [`CS_EVENT_INSERTION_REQUEST`] (`CS_STATUS_INSERTION_REQUEST`).
pub const CS_STATUS_INSERTION_REQUEST: u32 = CS_EVENT_INSERTION_REQUEST;
/// The battery of the card is dead, from the BVD1 signal. It has the value
/// of [`CS_EVENT_BATTERY_DEAD`] (`CS_STATUS_BATTERY_DEAD`).
pub const CS_STATUS_BATTERY_DEAD: u32 = CS_EVENT_BATTERY_DEAD;
/// The battery of the card is low, from the BVD2 signal. It has the value
/// of [`CS_EVENT_BATTERY_LOW`] (`CS_STATUS_BATTERY_LOW`).
pub const CS_STATUS_BATTERY_LOW: u32 = CS_EVENT_BATTERY_LOW;
/// The card is ready. It has the value of [`CS_EVENT_CARD_READY`]
/// (`CS_STATUS_CARD_READY`).
pub const CS_STATUS_CARD_READY: u32 = CS_EVENT_CARD_READY;
/// The card is inserted. It has the value of [`CS_EVENT_CARD_INSERTION`]
/// (`CS_STATUS_CARD_INSERTED`).
pub const CS_STATUS_CARD_INSERTED: u32 = CS_EVENT_CARD_INSERTION;
/// The first reserved event status bit of the extended status register
/// (`CS_STATUS_RES_EVT1`).
pub const CS_STATUS_RES_EVT1: u32 = 0x0010_0000;
/// The second reserved event status bit of the extended status register
/// (`CS_STATUS_RES_EVT2`).
pub const CS_STATUS_RES_EVT2: u32 = 0x0020_0000;
/// The third reserved event status bit of the extended status register
/// (`CS_STATUS_RES_EVT3`).
pub const CS_STATUS_RES_EVT3: u32 = 0x0040_0000;
/// The card indicates a Vcc of 5.0 volts (`CS_STATUS_VCC_50`).
pub const CS_STATUS_VCC_50: u32 = 0x1000_0000;
/// The card indicates a Vcc of 3.3 volts (`CS_STATUS_VCC_33`).
pub const CS_STATUS_VCC_33: u32 = 0x2000_0000;
/// The card indicates a Vcc of X.X volts (`CS_STATUS_VCC_XX`).
pub const CS_STATUS_VCC_XX: u32 = 0x4000_0000;
/// The extended status register requests attention
/// (`CS_STATUS_REQ_ATTN`).
pub const CS_STATUS_REQ_ATTN: u32 = 0x8000_0000;

/// The write protect state has changed. It has the value of
/// [`CS_EVENT_WRITE_PROTECT`] (`CS_SOCK_STATUS_WRITE_PROTECT_CHANGE`).
pub const CS_SOCK_STATUS_WRITE_PROTECT_CHANGE: u32 = CS_EVENT_WRITE_PROTECT;
/// The card lock state has changed. It has the value of
/// [`CS_EVENT_CARD_LOCK`], and the manual page spells it
/// `CS_SOCK_STATUS_CARD_LOCK_CHANGE` (`CS_SOCK_STATUS_CARD_LOCK_CHNAGE`).
pub const CS_SOCK_STATUS_CARD_LOCK_CHNAGE: u32 = CS_EVENT_CARD_LOCK;
/// An ejection request is pending. It has the value of
/// [`CS_EVENT_EJECTION_REQUEST`] (`CS_SOCK_STATUS_EJECTION_PENDING`).
pub const CS_SOCK_STATUS_EJECTION_PENDING: u32 = CS_EVENT_EJECTION_REQUEST;
/// An insertion request is pending. It has the value of
/// [`CS_EVENT_INSERTION_REQUEST`] (`CS_SOCK_STATUS_INSERTION_PENDING`).
pub const CS_SOCK_STATUS_INSERTION_PENDING: u32 = CS_EVENT_INSERTION_REQUEST;
/// The battery dead state has changed. It has the value of
/// [`CS_EVENT_BATTERY_DEAD`], and the manual page spells it
/// `CS_SOCK_STATUS_BATTERY_DEAD_CHANGE`
/// (`CS_SOCK_STATUS_BATTERY_DEAD_CHNAGE`).
pub const CS_SOCK_STATUS_BATTERY_DEAD_CHNAGE: u32 = CS_EVENT_BATTERY_DEAD;
/// The battery low state has changed. It has the value of
/// [`CS_EVENT_BATTERY_LOW`], and the manual page spells it
/// `CS_SOCK_STATUS_BATTERY_LOW_CHANGE`
/// (`CS_SOCK_STATUS_BATTERY_LOW_CHNAGE`).
pub const CS_SOCK_STATUS_BATTERY_LOW_CHNAGE: u32 = CS_EVENT_BATTERY_LOW;
/// The ready state has changed. It has the value of
/// [`CS_EVENT_CARD_READY`] (`CS_SOCK_STATUS_CARD_READY_CHANGE`).
pub const CS_SOCK_STATUS_CARD_READY_CHANGE: u32 = CS_EVENT_CARD_READY;
/// The card detect state has changed. It has the value of
/// [`CS_EVENT_CARD_INSERTION`], and the manual page calls it
/// `CS_SOCK_STATUS_CARD_INSERTION_CHANGE`
/// (`CS_SOCK_STATUS_CARD_DETECT_CHNAGE`).
pub const CS_SOCK_STATUS_CARD_DETECT_CHNAGE: u32 = CS_EVENT_CARD_INSERTION;

// --- event masks ---------------------------------------------------------

/// `csx_SetEventMask(9F)`. illumos DDI Specific (illumos DDI).
///
/// The event mask that [`csx_SetEventMask`] sets or [`csx_GetEventMask`]
/// returns, and which of the client's two masks it is.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct sockevent_t {
    /// Which mask is meant, [`CONF_EVENT_MASK_GLOBAL`] for the client's
    /// global event mask or [`CONF_EVENT_MASK_CLIENT`] for its client event
    /// mask for the socket.
    pub Attributes: u32,
    /// The event mask to set, or the one returned, a combination of the
    /// `CS_EVENT_*` bits.
    pub EventMask: u32,
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
}

// Layout on amd64 with LP64. Three uint32_t at 0, 4 and 8; total 12.
const _: () = assert!(core::mem::size_of::<sockevent_t>() == 12);
const _: () = assert!(core::mem::offset_of!(sockevent_t, Attributes) == 0);
const _: () = assert!(core::mem::offset_of!(sockevent_t, Socket) == 8);

/// Selects the client's global event mask (`CONF_EVENT_MASK_GLOBAL`).
pub const CONF_EVENT_MASK_GLOBAL: u32 = 0x0000_0000;
/// Selects the client's client event mask for the socket, which requires an
/// earlier [`csx_RequestSocketMask`] (`CONF_EVENT_MASK_CLIENT`).
pub const CONF_EVENT_MASK_CLIENT: u32 = 0x0000_0001;

/// `csx_RequestSocketMask(9F)`. illumos DDI Specific (illumos DDI).
///
/// The client event mask that [`csx_RequestSocketMask`] sets.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct request_socket_mask_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// The client event mask, a combination of the `CS_EVENT_*` bits.
    pub EventMask: u32,
}

// Layout on amd64 with LP64. Two uint32_t at 0 and 4; total 8.
const _: () = assert!(core::mem::size_of::<request_socket_mask_t>() == 8);
const _: () = assert!(core::mem::offset_of!(request_socket_mask_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(request_socket_mask_t, EventMask) == 4);

/// `csx_RequestSocketMask(9F)`. illumos DDI Specific (illumos DDI).
///
/// The argument of [`csx_ReleaseSocketMask`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct release_socket_mask_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
}

// Layout on amd64 with LP64. One uint32_t at 0; total 4.
const _: () = assert!(core::mem::size_of::<release_socket_mask_t>() == 4);
const _: () = assert!(core::mem::offset_of!(release_socket_mask_t, Socket) == 0);

// --- clients and sockets -------------------------------------------------

/// `csx_GetFirstClient(9F)`. illumos DDI Specific (illumos DDI).
///
/// The request and result of [`csx_GetFirstClient`] and
/// [`csx_GetNextClient`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct get_firstnext_client_t {
    /// The socket to report on when [`Attributes`](Self::Attributes)
    /// includes [`CS_GET_FIRSTNEXT_CLIENT_SOCKET_ONLY`].
    pub Socket: u32,
    /// Which clients to return, [`CS_GET_FIRSTNEXT_CLIENT_ALL_CLIENTS`] or
    /// [`CS_GET_FIRSTNEXT_CLIENT_SOCKET_ONLY`].
    pub Attributes: u32,
    /// On return, the client handle of the driver found.
    pub client_handle: client_handle_t,
    /// On return, the number of clients.
    pub num_clients: u32,
}

// Layout on amd64 with LP64. Four uint32_t at 0, 4, 8 and 12; total 16.
const _: () = assert!(core::mem::size_of::<get_firstnext_client_t>() == 16);
const _: () = assert!(core::mem::offset_of!(get_firstnext_client_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(get_firstnext_client_t, num_clients) == 12);

/// Return information on all clients
/// (`CS_GET_FIRSTNEXT_CLIENT_ALL_CLIENTS`).
pub const CS_GET_FIRSTNEXT_CLIENT_ALL_CLIENTS: u32 = 0x0000_0001;
/// Return information only on the clients of the socket in
/// [`get_firstnext_client_t::Socket`]
/// (`CS_GET_FIRSTNEXT_CLIENT_SOCKET_ONLY`).
pub const CS_GET_FIRSTNEXT_CLIENT_SOCKET_ONLY: u32 = 0x0000_0002;

/// `csx_MapLogSocket(9F)`. illumos DDI Specific (illumos DDI).
///
/// The result of [`csx_MapLogSocket`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct map_log_socket_t {
    /// The logical socket. This implementation does not use it, and it may
    /// hold any value.
    pub LogSocket: u32,
    /// On return, the physical adapter number, which is always 0 in
    /// illumos.
    pub PhyAdapter: u32,
    /// On return, the physical socket number of the client. Drivers use it
    /// in messages and in minor node names.
    pub PhySocket: u32,
}

// Layout on amd64 with LP64. Three uint32_t at 0, 4 and 8; total 12.
const _: () = assert!(core::mem::size_of::<map_log_socket_t>() == 12);
const _: () = assert!(core::mem::offset_of!(map_log_socket_t, LogSocket) == 0);
const _: () = assert!(core::mem::offset_of!(map_log_socket_t, PhySocket) == 8);

/// `csx_CS_DDI_Info(9F)`. illumos DDI Specific (illumos DDI).
///
/// The request and result of [`csx_CS_DDI_Info`]. The client sets the
/// socket and its driver name, and on success Card Services fills in the
/// device node and instance of the matching driver instance.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cs_ddi_info_t {
    /// The physical socket number to look up. In a `getinfo(9E)` entry point
    /// it is typically extracted from the minor number in the argument.
    pub Socket: u32,
    /// The name of the client driver, the same string it registered with in
    /// [`client_reg_t::driver_name`].
    pub driver_name: *mut c_char,
    /// On return, the device node of the driver instance.
    pub dip: dev_info_t,
    /// On return, the instance number of the driver instance.
    pub instance: i32,
}

// Layout on amd64 with LP64. Socket 4 at 0 and 4 bytes of padding,
// driver_name 8 at 8, dip 8 at 16 and instance 4 at 24, which ends at 28
// and is padded to 32.
const _: () = assert!(core::mem::size_of::<cs_ddi_info_t>() == 32);
const _: () = assert!(core::mem::offset_of!(cs_ddi_info_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(cs_ddi_info_t, driver_name) == 8);
const _: () = assert!(core::mem::offset_of!(cs_ddi_info_t, dip) == 16);
const _: () = assert!(core::mem::offset_of!(cs_ddi_info_t, instance) == 24);

// --- device nodes --------------------------------------------------------

/// `csx_MakeDeviceNode(9F)`. illumos DDI Specific (illumos DDI).
///
/// Describes one minor node for [`csx_MakeDeviceNode`] or
/// [`csx_RemoveDeviceNode`]. The members mean the same as the parameters of
/// the same names of `ddi_create_minor_node(9F)` and
/// `ddi_remove_minor_node(9F)`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct devnode_desc_t {
    /// The name of the minor node.
    pub name: *mut c_char,
    /// The special file type, `S_IFBLK` or `S_IFCHR`.
    pub spec_type: i32,
    /// The minor number of the node.
    pub minor_num: i32,
    /// The node type, such as `DDI_NT_BLOCK`.
    pub node_type: *mut c_char,
}

// Layout on amd64 with LP64. name 8 at 0, spec_type 4 at 8, minor_num 4 at
// 12 and node_type 8 at 16; total 24.
const _: () = assert!(core::mem::size_of::<devnode_desc_t>() == 24);
const _: () = assert!(core::mem::offset_of!(devnode_desc_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(devnode_desc_t, spec_type) == 8);
const _: () = assert!(core::mem::offset_of!(devnode_desc_t, node_type) == 16);

/// `csx_MakeDeviceNode(9F)`. illumos DDI Specific (illumos DDI).
///
/// The request of [`csx_MakeDeviceNode`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct make_device_node_t {
    /// The operation, which must be [`CREATE_DEVICE_NODE`].
    pub Action: u32,
    /// The number of minor nodes to create.
    pub NumDevNodes: u32,
    /// An array of [`NumDevNodes`](Self::NumDevNodes) descriptions, which
    /// the client allocates and fills in.
    pub devnode_desc: *mut devnode_desc_t,
}

// Layout on amd64 with LP64. Action 4 at 0, NumDevNodes 4 at 4 and
// devnode_desc 8 at 8; total 16.
const _: () = assert!(core::mem::size_of::<make_device_node_t>() == 16);
const _: () = assert!(core::mem::offset_of!(make_device_node_t, Action) == 0);
const _: () = assert!(core::mem::offset_of!(make_device_node_t, devnode_desc) == 8);

/// `csx_MakeDeviceNode(9F)`. illumos DDI Specific (illumos DDI).
///
/// The request of [`csx_RemoveDeviceNode`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct remove_device_node_t {
    /// The operation, [`REMOVE_DEVICE_NODE`] or [`REMOVE_ALL_DEVICE_NODES`].
    pub Action: u32,
    /// The number of minor nodes to remove, or 0 with
    /// [`REMOVE_ALL_DEVICE_NODES`].
    pub NumDevNodes: u32,
    /// An array of [`NumDevNodes`](Self::NumDevNodes) descriptions, which
    /// the client allocates and fills in, or null with
    /// [`REMOVE_ALL_DEVICE_NODES`].
    pub devnode_desc: *mut devnode_desc_t,
}

// Layout on amd64 with LP64. Action 4 at 0, NumDevNodes 4 at 4 and
// devnode_desc 8 at 8; total 16.
const _: () = assert!(core::mem::size_of::<remove_device_node_t>() == 16);
const _: () = assert!(core::mem::offset_of!(remove_device_node_t, Action) == 0);
const _: () = assert!(core::mem::offset_of!(remove_device_node_t, devnode_desc) == 8);

/// Create the minor nodes described in the request
/// (`CREATE_DEVICE_NODE`).
pub const CREATE_DEVICE_NODE: u32 = 0x01;
/// Remove the minor nodes described in the request
/// (`REMOVE_DEVICE_NODE`).
pub const REMOVE_DEVICE_NODE: u32 = 0x02;
/// Remove all minor nodes of the client (`REMOVE_ALL_DEVICE_NODES`).
pub const REMOVE_ALL_DEVICE_NODES: u32 = 0x03;

unsafe extern "C" {
    // --- registration ------------------------------------------------------
    /// `csx_RegisterClient(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Registers a client with Card Services as described by the
    /// [`client_reg_t`] at `cr`, and stores a handle unique to the client in
    /// `ch`. The handle must be passed to [`csx_DeregisterClient`] when the
    /// client terminates. No events are delivered until
    /// [`csx_RequestSocketMask`] succeeds.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_ATTRIBUTE`](crate::consts::CS_BAD_ATTRIBUTE) when no client
    /// type or more than one was given,
    /// [`CS_OUT_OF_RESOURCE`](crate::consts::CS_OUT_OF_RESOURCE) when the
    /// client cannot be registered,
    /// [`CS_BAD_VERSION`](crate::consts::CS_BAD_VERSION) when the version is
    /// incompatible, [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) when the
    /// client has already registered for this socket, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_RegisterClient(ch: *mut client_handle_t, cr: *mut client_reg_t) -> i32;
    /// `csx_DeregisterClient(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Removes the client `ch` from the list of registered clients. The
    /// client must have released every resource it requested first. It
    /// should be prepared to receive callbacks until this call returns
    /// successfully.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_IN_USE`](crate::consts::CS_IN_USE) when resources have
    /// not been released, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_DeregisterClient(ch: client_handle_t) -> i32;

    // --- event masks -------------------------------------------------------
    /// `csx_RequestSocketMask(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Sets the client event mask of the client `ch` from the
    /// [`request_socket_mask_t`] at `sm` and enables delivery of events to
    /// its event handler. Events generated by the registration are delivered
    /// after this call, so the client can set up its event mutexes before
    /// the handler first runs. It must be called before
    /// [`csx_GetEventMask`] or [`csx_SetEventMask`] is used on the client
    /// event mask.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_IN_USE`](crate::consts::CS_IN_USE) when an earlier mask
    /// has not been released, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_RequestSocketMask(ch: client_handle_t, sm: *mut request_socket_mask_t) -> i32;
    /// `csx_RequestSocketMask(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Clears the client event mask of the client `ch`. The argument `rm`
    /// points to a [`release_socket_mask_t`].
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_BAD_SOCKET`](crate::consts::CS_BAD_SOCKET) when no mask
    /// was requested, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_ReleaseSocketMask(ch: client_handle_t, rm: *mut release_socket_mask_t) -> i32;
    /// `csx_SetEventMask(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Sets the global or client event mask of the client `ch`, as the
    /// [`sockevent_t`] at `se` describes. Setting the client event mask
    /// requires an earlier [`csx_RequestSocketMask`].
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_BAD_SOCKET`](crate::consts::CS_BAD_SOCKET) when
    /// [`csx_RequestSocketMask`] was not called for
    /// [`CONF_EVENT_MASK_CLIENT`], or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_SetEventMask(ch: client_handle_t, se: *mut sockevent_t) -> i32;
    /// `csx_SetEventMask(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Returns the global or client event mask of the client `ch` in the
    /// [`sockevent_t`] at `se`, whose
    /// [`Attributes`](sockevent_t::Attributes) member says which mask is
    /// wanted.
    ///
    /// The return values are those of [`csx_SetEventMask`]. It may be
    /// called from user or kernel context.
    pub fn csx_GetEventMask(ch: client_handle_t, se: *mut sockevent_t) -> i32;

    // --- status and sockets ------------------------------------------------
    /// `csx_GetStatus(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Returns the current status of the card of the client `ch` and of its
    /// socket in the [`get_status_t`] at `gs`.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_BAD_SOCKET`](crate::consts::CS_BAD_SOCKET) when the
    /// socket state could not be read, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It does not return
    /// [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the socket is empty.
    /// It may be called from user or kernel context.
    pub fn csx_GetStatus(ch: client_handle_t, gs: *mut get_status_t) -> i32;
    /// `csx_MapLogSocket(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Returns the physical socket number associated with the client `ch`
    /// in the [`map_log_socket_t`] at `ls`.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_MapLogSocket(ch: client_handle_t, ls: *mut map_log_socket_t) -> i32;
    /// `csx_GetFirstClient(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Returns information about the first installed card, or the first on
    /// one socket, in the [`get_firstnext_client_t`] at `fnc`, and starts a
    /// walk that [`csx_GetNextClient`] continues.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_BAD_SOCKET`](crate::consts::CS_BAD_SOCKET) for an
    /// invalid socket number, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when
    /// the socket is empty,
    /// [`CS_NO_MORE_ITEMS`](crate::consts::CS_NO_MORE_ITEMS), which the page
    /// explains as the driver not handling `CS_EVENT_CLIENT_INFO`, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_GetFirstClient(fnc: *mut get_firstnext_client_t) -> i32;
    /// `csx_GetFirstClient(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Returns information about the next installed card in the
    /// [`get_firstnext_client_t`] at `fnc`, continuing the walk that
    /// [`csx_GetFirstClient`] started.
    ///
    /// The return values are those of [`csx_GetFirstClient`]. It may be
    /// called from user or kernel context.
    pub fn csx_GetNextClient(fnc: *mut get_firstnext_client_t) -> i32;

    // --- DDI helpers -------------------------------------------------------
    /// `csx_CS_DDI_Info(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// An illumos extension for clients that implement `getinfo(9E)`. Given
    /// the socket number and driver name in the [`cs_ddi_info_t`] at `cdi`,
    /// it looks the driver up on that socket and, when found, fills in the
    /// device node and the instance number of the driver instance.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_SOCKET`](crate::consts::CS_BAD_SOCKET) when the client is
    /// not found on the socket, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_CS_DDI_Info(cdi: *mut cs_ddi_info_t) -> i32;
    /// `csx_MakeDeviceNode(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// An illumos extension that creates minor nodes on behalf of the client
    /// `ch`, as the [`make_device_node_t`] at `dn` describes.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_BAD_ATTRIBUTE`](crate::consts::CS_BAD_ATTRIBUTE) when an
    /// argument is invalid, [`CS_BAD_ARGS`](crate::consts::CS_BAD_ARGS) for
    /// an invalid action,
    /// [`CS_OUT_OF_RESOURCE`](crate::consts::CS_OUT_OF_RESOURCE) when a node
    /// could not be created, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_MakeDeviceNode(ch: client_handle_t, dn: *mut make_device_node_t) -> i32;
    /// `csx_MakeDeviceNode(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// An illumos extension that removes minor nodes on behalf of the client
    /// `ch`, either those the [`remove_device_node_t`] at `dn` describes or,
    /// with [`REMOVE_ALL_DEVICE_NODES`], all of the client's nodes.
    ///
    /// The return values are those of [`csx_MakeDeviceNode`]. It may be
    /// called from user or kernel context.
    pub fn csx_RemoveDeviceNode(ch: client_handle_t, dn: *mut remove_device_node_t) -> i32;
}
