//! The provider entry points, `mac_callbacks(9S)`, from `sys/mac_provider.h`.
//!
//! The MAC framework drives a provider entirely through the function
//! pointers in a [`mac_callbacks_t`]. Seven of them are required, namely
//! `mc_getstat`, `mc_start`, `mc_stop`, `mc_setpromisc`, `mc_multicst`,
//! `mc_unicst` and `mc_tx`, and `mac_register(9F)` fails if one is missing.
//! The others are optional, and the framework only looks at an optional slot
//! if the matching `MC_*` bit is set in `mc_callbacks`. This scheme lets the
//! framework add entry points without breaking providers built against an
//! older layout.
//!
//! The C header declares each entry point type as a pointer typedef. Here
//! each type is the function type itself, which cannot be NULL, and the
//! structure wraps it in `Option` so that an unused slot is `None`.
//!
//! The first argument of every entry point is the `m_driver` value the
//! provider stored in `mac_register(9S)`, usually its soft state. The manual
//! pages call this argument the private data "passed in via the m_pdata
//! member", which is a slip; it is `m_driver`. The framework makes no promise
//! about which entry points run concurrently, apart from those the
//! individual pages state, so a provider must protect its state with locks.
//! It may call `mac_hcksum_get(9F)`, `mac_hcksum_set(9F)`, `mac_lso_get(9F)`,
//! `mac_maxsdu_update(9F)` and the `mac_prop_info(9F)` functions while
//! holding its own locks, but no other MAC function, and in particular not
//! `mac_rx(9F)`, `mac_tx_update(9F)` or `mac_link_update(9F)`.
//!
//! The header also declares `mac_resources_t`, the type of a slot that was
//! removed from the structure long ago and whose place `mc_reserved` now
//! takes. It is not bound, and neither is the `MC_RESERVED` flag, which no
//! manual page describes.

use ddi_sys::{boolean_t, c_char, c_int, c_void, mblk_t, queue_t, uint_t};

use crate::capab::mac_capab_t;
use crate::props::mac_prop_id_t;
use crate::types::mac_prop_info_handle_t;

/// `mc_getstat(9E)`. illumos DDI specific.
///
/// The signature of the entry point that reads one statistic, `mac_getstat_t`.
/// The framework passes the provider's private data in `driver`, the
/// identifier of the statistic in `stat`, which is one of the
/// [`mac_driver_stat`](crate::stats::mac_driver_stat) values or, for an
/// Ethernet device, one of the [`ether_stat`](crate::stats::ether_stat)
/// values, and a pointer in `stat_value` through which the provider stores
/// the current value. Statistics are monotonic 64-bit counters that only
/// reset when the instance detaches, so a provider whose hardware counters
/// can overflow must accumulate them in its soft state.
///
/// The entry point returns 0 after storing the value. For a statistic it
/// does not recognize or support, it leaves `stat_value` untouched and
/// returns `ENOTSUP`. `EIO` reports a transport or DMA fault while reading
/// the device, and `ECANCELLED` a device that cannot currently service the
/// request. The provider is responsible for locking its statistics.
pub type mac_getstat_t = unsafe extern "C" fn(driver: *mut c_void, stat: uint_t, stat_value: *mut u64) -> c_int;

/// `mc_start(9E)`. illumos DDI specific.
///
/// The signature of the entry point that starts the device, `mac_start_t`.
/// The provider initializes the hardware so that it is ready to send and
/// receive, which for most devices means allocating DMA resources, enabling
/// the link and completing the programming of the device. The framework
/// calls it before any entry point that sends or receives data.
///
/// The framework guarantees that only one of `mc_start(9E)` and
/// `mc_stop(9E)` runs at a time for an instance, and that neither runs
/// concurrently with `attach(9E)` or `detach(9E)`, but other entry points
/// may. The entry point returns 0 on success and a positive error number
/// otherwise.
pub type mac_start_t = unsafe extern "C" fn(driver: *mut c_void) -> c_int;

/// `mc_stop(9E)`. illumos DDI specific.
///
/// The signature of the entry point that stops the device, `mac_stop_t`,
/// which is the opposite of `mc_start(9E)`. The provider tears down the
/// resources it allocated for operation and performs no further I/O once
/// the function has returned. The serialization guarantees of
/// `mc_start(9E)` apply here as well. The entry point has no return value.
pub type mac_stop_t = unsafe extern "C" fn(driver: *mut c_void);

/// `mc_setpromisc(9E)`. illumos DDI Specific.
///
/// The signature of the entry point that changes promiscuous mode,
/// `mac_setpromisc_t`. When `enable` is `B_TRUE`, the provider enables both
/// unicast and multicast promiscuous reception; when it is `B_FALSE`, it
/// disables both. A device should start with promiscuous mode disabled.
/// The entry point may run in parallel with others, so the provider must
/// lock its state.
///
/// The entry point returns 0 once the device is in the requested state, and
/// a positive error number otherwise, such as `EIO` for a device or
/// transport error.
pub type mac_setpromisc_t = unsafe extern "C" fn(driver: *mut c_void, enable: boolean_t) -> c_int;

/// `mc_multicst(9E)`. illumos DDI Specific.
///
/// The signature of the entry point that adds a multicast address to the
/// device's filters or removes one from them, `mac_multicst_t`. When `add`
/// is `B_TRUE` the provider adds the address in `mac`, otherwise it removes
/// it. The address is at least as long as the plugin's address length, six
/// bytes for Ethernet. The framework does the reference counting, so the
/// provider does not. The manual recommends that the provider keep its own
/// copy of the programmed addresses so that it can restore them after a
/// device reset or a resume.
///
/// The entry point returns 0 on success and a positive error number
/// otherwise. `EINVAL` means that the address is not valid, `EIO` a device
/// or transport error, `ENOENT` a request to remove an address that is not
/// programmed, and `ENOSPC` that no filter slot is left.
pub type mac_multicst_t = unsafe extern "C" fn(driver: *mut c_void, add: boolean_t, mac: *const u8) -> c_int;

/// `mc_unicst(9E)`. illumos DDI specific.
///
/// The signature of the entry point that replaces the primary unicast
/// address of the device, `mac_unicst_t`. The address in `mac` is exactly
/// as long as the plugin's address length, six bytes for Ethernet, and
/// overwrites the address currently programmed. The entry point supports a
/// single address; a device with several unicast filters exposes them
/// through the `MAC_CAPAB_RINGS` capability instead, and a provider that
/// implements that capability for receive rings must not implement this
/// entry point.
///
/// The entry point returns 0 after updating the filter. Otherwise it leaves
/// the address unchanged and returns an error number, such as `EINVAL` for
/// an address that is not a valid unicast address or `EIO` for a device or
/// transport error.
pub type mac_unicst_t = unsafe extern "C" fn(driver: *mut c_void, mac: *const u8) -> c_int;

/// `mc_tx(9E)`. illumos DDI specific.
///
/// The signature of the transmit entry point, `mac_tx_t`. The framework
/// passes a chain of frames in `mp_chain`. Each frame may span several
/// message blocks linked through `b_cont`, and frames are linked to each
/// other through `b_next`. For each frame the provider checks the offload
/// requests with `mac_hcksum_get(9F)` and `mac_lso_get(9F)` if it has
/// advertised those capabilities, and then either transmits the frame,
/// drops it with `freemsg(9F)`, or stops and returns it together with the
/// remaining frames because it has run out of resources.
///
/// The entry point returns NULL when it has consumed the whole chain.
/// Otherwise it returns the frames it did not process, and the framework
/// makes no further transmit calls until the provider calls
/// `mac_tx_update(9F)`, so a provider may only return frames when it will
/// later be told that resources are available again, typically by a
/// transmit completion interrupt. A frame that was bound for DMA must not be
/// freed before the hardware has sent it. The entry point may be called from
/// kernel or interrupt context, and concurrently with the provider's own
/// interrupt handling.
pub type mac_tx_t = unsafe extern "C" fn(driver: *mut c_void, mp_chain: *mut mblk_t) -> *mut mblk_t;

/// `mc_ioctl(9E)`. illumos DDI Specific.
///
/// The signature of the optional entry point for device specific I/O
/// control requests, `mac_ioctl_t`. Unlike `ioctl(9E)`, it receives a
/// STREAMS message. `wq` is the write-side queue the request arrived on, and
/// `mp` is the message whose data holds an `iocblk(9S)`. The provider
/// replies on the queue, normally with `miocack(9F)` or `miocnak(9F)`, and
/// the result travels in the reply rather than in a return value. Most
/// providers do not implement this entry point.
pub type mac_ioctl_t = unsafe extern "C" fn(driver: *mut c_void, wq: *mut queue_t, mp: *mut mblk_t);

/// `mc_getcapab(9E)`. illumos DDI specific.
///
/// The signature of the optional entry point that reports capabilities,
/// `mac_getcapab_t`. The framework asks about the capability `capab`, and
/// `cap_data` points to the data of that capability, whose type depends on
/// `capab`. If the provider supports the capability it fills in the data and
/// returns `B_TRUE`; for any unsupported or unknown capability it returns
/// `B_FALSE`, because the framework does ask about capabilities the provider
/// has never heard of.
///
/// Capabilities are generally queried once, and the answer must not change
/// afterwards. The manual recommends that every supported capability can be
/// disabled through a private property or a `driver.conf(5)` property, so
/// that hardware bugs can be worked around in the field. The entry point is
/// generally only called from kernel context.
pub type mac_getcapab_t =
    unsafe extern "C" fn(driver: *mut c_void, capab: mac_capab_t, cap_data: *mut c_void) -> boolean_t;

/// `mc_open(9E)`. illumos DDI specific.
///
/// The signature of the optional entry point called when the device node is
/// opened, `mac_open_t`. Standard providers neither need nor define it. The
/// framework serializes the calls to `mc_open(9E)` and `mc_close(9E)`. The
/// entry point returns 0 on success and a non-zero error number otherwise.
pub type mac_open_t = unsafe extern "C" fn(driver: *mut c_void) -> c_int;

/// `mc_close(9E)`. illumos DDI specific.
///
/// The signature of the optional entry point called when the device node is
/// closed, `mac_close_t`. Standard providers neither need nor define it. The
/// framework serializes the calls to `mc_open(9E)` and `mc_close(9E)`. The
/// entry point has no return value.
pub type mac_close_t = unsafe extern "C" fn(driver: *mut c_void);

/// `mc_setprop(9E)`. illumos DDI specific.
///
/// The signature of the optional entry point that sets a property,
/// `mac_set_prop_t`. The provider identifies the property by `pr_num`,
/// checks that `pr_valsize` is at least the size `mac(9E)` gives for that
/// property, and applies the new value in `pr_val`. For
/// [`mac_prop_id_t::MAC_PROP_PRIVATE`] it identifies the private property by
/// comparing the NUL-terminated `pr_name` with its own names using
/// `strcmp(9F)`. The provider must lock its state.
///
/// The entry point returns 0 on success and a positive error number
/// otherwise. `ENOTSUP` means that the property is unknown, unsupported or
/// not writable, `EINVAL` that the value is out of range, `EOVERFLOW` that
/// `pr_valsize` is too small, `EBUSY` that the property cannot be changed
/// while the device is started, and `ECANCELLED` that the device is in a
/// state such as suspended where it cannot handle the request.
pub type mac_set_prop_t = unsafe extern "C" fn(
    driver: *mut c_void,
    pr_name: *const c_char,
    pr_num: mac_prop_id_t,
    pr_valsize: uint_t,
    pr_val: *const c_void,
) -> c_int;

/// `mc_getprop(9E)`. illumos DDI specific.
///
/// The signature of the optional entry point that reads a property,
/// `mac_get_prop_t`. The provider identifies the property by `pr_num`,
/// checks that the `pr_valsize` bytes available at `pr_val` are at least the
/// size `mac(9E)` gives for that property, and copies the current value
/// there. For [`mac_prop_id_t::MAC_PROP_PRIVATE`] it identifies the private
/// property by comparing the NUL-terminated `pr_name` with its own names
/// using `strcmp(9F)`. Private properties are currently limited to string
/// values, because `dladm(8)` renders no other type correctly.
///
/// The entry point returns 0 on success and a positive error number
/// otherwise, `ENOTSUP` for an unknown or unsupported property and
/// `EOVERFLOW` when `pr_valsize` is too small. It is generally called from
/// kernel context.
pub type mac_get_prop_t = unsafe extern "C" fn(
    driver: *mut c_void,
    pr_name: *const c_char,
    pr_num: mac_prop_id_t,
    pr_valsize: uint_t,
    pr_val: *mut c_void,
) -> c_int;

/// `mc_propinfo(9E)`. illumos DDI specific.
///
/// The signature of the optional entry point that describes a property,
/// `mac_prop_info_t`. For the property identified by `pr_num`, or by
/// `pr_name` for [`mac_prop_id_t::MAC_PROP_PRIVATE`], the provider passes
/// `hdl` to the `mac_prop_info(9F)` functions to record the permissions with
/// `mac_prop_info_set_perm(9F)`, the default value with one of the
/// `mac_prop_info_set_default_*` functions, and for integer properties any
/// valid ranges with `mac_prop_info_set_range_uint32(9F)`. For an unknown
/// or unsupported property it simply returns without calling any of them.
/// The entry point has no return value.
pub type mac_prop_info_t = unsafe extern "C" fn(
    driver: *mut c_void,
    pr_name: *const c_char,
    pr_num: mac_prop_id_t,
    hdl: mac_prop_info_handle_t,
);

/// `mac_callbacks(9S)`. illumos DDI specific.
///
/// The table of entry points a GLDv3 provider implements, `mac_callbacks_t`.
/// A provider normally declares one statically and stores its address in
/// the `m_callbacks` member of `mac_register(9S)`; the framework keeps using
/// it for as long as the provider is registered.
///
/// The members `mc_getstat`, `mc_start`, `mc_stop`, `mc_setpromisc`,
/// `mc_multicst`, `mc_unicst` and `mc_tx` are required, except that a
/// provider implementing `MAC_CAPAB_RINGS` for receive rings must leave
/// `mc_unicst` unset and one implementing it for transmit rings must leave
/// `mc_tx` unset. The optional members are only used when the matching bit
/// is set in `mc_callbacks`. A provider that implements one of `mc_getprop`,
/// `mc_setprop` and `mc_propinfo` should implement all three, so that its
/// properties are fully visible to `dladm(8)`.
#[repr(C)]
pub struct mac_callbacks_t {
    /// The bitwise inclusive OR of the `MC_*` flags of the optional members
    /// that are set, for example [`MC_GETCAPAB`] `|` [`MC_PROPERTIES`].
    pub mc_callbacks: uint_t,
    /// Reads a statistic, see `mc_getstat(9E)`. Required.
    pub mc_getstat: Option<mac_getstat_t>,
    /// Starts the device, see `mc_start(9E)`. Required.
    pub mc_start: Option<mac_start_t>,
    /// Stops the device, see `mc_stop(9E)`. Required.
    pub mc_stop: Option<mac_stop_t>,
    /// Enables or disables promiscuous mode, see `mc_setpromisc(9E)`.
    /// Required.
    pub mc_setpromisc: Option<mac_setpromisc_t>,
    /// Adds or removes a multicast address, see `mc_multicst(9E)`. Required.
    pub mc_multicst: Option<mac_multicst_t>,
    /// Sets the primary unicast address, see `mc_unicst(9E)`. Required unless
    /// the provider implements `MAC_CAPAB_RINGS` for receive rings, in which
    /// case it must be `None`.
    pub mc_unicst: Option<mac_unicst_t>,
    /// Transmits a chain of frames, see `mc_tx(9E)`. Required unless the
    /// provider implements `MAC_CAPAB_RINGS` for transmit rings, in which
    /// case it is not used.
    pub mc_tx: Option<mac_tx_t>,
    /// Reserved. Must be NULL.
    pub mc_reserved: *mut c_void,
    /// Handles a device specific I/O control request, see `mc_ioctl(9E)`.
    /// Optional; used only when [`MC_IOCTL`] is set.
    pub mc_ioctl: Option<mac_ioctl_t>,
    /// Reports capabilities, see `mc_getcapab(9E)`. Optional; used only when
    /// [`MC_GETCAPAB`] is set.
    pub mc_getcapab: Option<mac_getcapab_t>,
    /// Called when the device node is opened, see `mc_open(9E)`. Optional;
    /// used only when [`MC_OPEN`] is set.
    pub mc_open: Option<mac_open_t>,
    /// Called when the device node is closed, see `mc_close(9E)`. Optional;
    /// used only when [`MC_CLOSE`] is set.
    pub mc_close: Option<mac_close_t>,
    /// Sets a property, see `mc_setprop(9E)`. Optional; used only when
    /// [`MC_SETPROP`] is set.
    pub mc_setprop: Option<mac_set_prop_t>,
    /// Reads a property, see `mc_getprop(9E)`. Optional; used only when
    /// [`MC_GETPROP`] is set.
    pub mc_getprop: Option<mac_get_prop_t>,
    /// Describes a property, see `mc_propinfo(9E)`. Optional; used only when
    /// [`MC_PROPINFO`] is set.
    pub mc_propinfo: Option<mac_prop_info_t>,
}

// mac_callbacks_t under LP64: mc_callbacks at 0 padded to 8, then fifteen
// pointers from 8 to 128, with mc_reserved as the eighth at 64 and
// mc_propinfo as the last at 120; total 128.
const _: () = assert!(core::mem::size_of::<mac_callbacks_t>() == 128);
const _: () = assert!(core::mem::offset_of!(mac_callbacks_t, mc_callbacks) == 0);
const _: () = assert!(core::mem::offset_of!(mac_callbacks_t, mc_getstat) == 8);
const _: () = assert!(core::mem::offset_of!(mac_callbacks_t, mc_tx) == 56);
const _: () = assert!(core::mem::offset_of!(mac_callbacks_t, mc_reserved) == 64);
const _: () = assert!(core::mem::offset_of!(mac_callbacks_t, mc_ioctl) == 72);
const _: () = assert!(core::mem::offset_of!(mac_callbacks_t, mc_propinfo) == 120);

/// The `mc_ioctl` member of `mac_callbacks(9S)` is set (`MC_IOCTL`).
pub const MC_IOCTL: uint_t = 0x0002;
/// The `mc_getcapab` member of `mac_callbacks(9S)` is set (`MC_GETCAPAB`).
pub const MC_GETCAPAB: uint_t = 0x0004;
/// The `mc_open` member of `mac_callbacks(9S)` is set (`MC_OPEN`).
pub const MC_OPEN: uint_t = 0x0008;
/// The `mc_close` member of `mac_callbacks(9S)` is set (`MC_CLOSE`).
pub const MC_CLOSE: uint_t = 0x0010;
/// The `mc_setprop` member of `mac_callbacks(9S)` is set (`MC_SETPROP`).
pub const MC_SETPROP: uint_t = 0x0020;
/// The `mc_getprop` member of `mac_callbacks(9S)` is set (`MC_GETPROP`).
pub const MC_GETPROP: uint_t = 0x0040;
/// The `mc_propinfo` member of `mac_callbacks(9S)` is set (`MC_PROPINFO`).
pub const MC_PROPINFO: uint_t = 0x0080;
/// The `mc_getprop`, `mc_setprop` and `mc_propinfo` members of
/// `mac_callbacks(9S)` are all set. It is the combination of
/// [`MC_SETPROP`], [`MC_GETPROP`] and [`MC_PROPINFO`] (`MC_PROPERTIES`).
pub const MC_PROPERTIES: uint_t = MC_SETPROP | MC_GETPROP | MC_PROPINFO;
