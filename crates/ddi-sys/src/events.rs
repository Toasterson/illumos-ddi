//! NDI event cookies and handlers, and system event logging, from
//! `sys/sunddi.h`, `sys/dditypes.h`, `sys/sysevent.h` and
//! `sys/sysevent/eventdefs.h`.
//!
//! Two unrelated event mechanisms share this module. The first is the NDI
//! event service, through which a nexus driver tells its children about
//! things such as bus resets and device removals. A leaf driver asks for a
//! cookie naming an event with `ddi_get_eventcookie(9F)`, subscribes a
//! callback with `ddi_add_event_handler(9F)`, and unsubscribes it again with
//! `ddi_remove_event_handler(9F)` before `detach(9E)` completes. The second
//! is the system event facility, through which `ddi_log_sysevent(9F)` queues
//! a class and subclass, optionally with an attribute list, for delivery to
//! the user-space `syseventd` daemon.

use crate::nvpair::nvlist_t;
use crate::types::*;

opaque! {
    /// The structure behind an event cookie, `struct ddi_event_cookie`. Its
    /// contents are private to the NDI event service.
    ddi_event_cookie;
    /// The structure behind a callback registration,
    /// `struct ddi_event_callbacks`. Its contents are private to the NDI
    /// event service.
    ddi_event_callbacks;
}

/// A cookie naming one NDI event, returned by `ddi_get_eventcookie(9F)`.
pub type ddi_eventcookie_t = *mut ddi_event_cookie;
/// The identifier of a registered callback, returned by
/// `ddi_add_event_handler(9F)` and passed to `ddi_remove_event_handler(9F)`.
pub type ddi_callback_id_t = *mut ddi_event_callbacks;
/// The signature of an NDI event callback, `ddi_event_cb_f`. The kernel
/// calls it with the node that registered the handler, the cookie of the
/// event that occurred, the `arg` pointer given at registration, and a
/// pointer to bus-specific data defined by the nexus that raised the event.
pub type ddi_event_cb_f =
    unsafe extern "C" fn(dip: dev_info_t, cookie: ddi_eventcookie_t, arg: *mut c_void, impldata: *mut c_void);

/// `sysevent_id_t`. illumos DDI specific (illumos DDI).
///
/// The unique identifier of a queued system event, filled in by
/// `ddi_log_sysevent(9F)` when the caller asks for it. Its two fields
/// together are sufficient to identify an event.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct sysevent_id_t {
    /// The sequence number of the event.
    pub eid_seq: u64,
    /// The timestamp of the event, in the units of `gethrtime(9F)`.
    pub eid_ts: hrtime_t,
}

// Two 8-byte fields.
const _: () = assert!(core::mem::size_of::<sysevent_id_t>() == 16);
const _: () = assert!(core::mem::offset_of!(sysevent_id_t, eid_seq) == 0);
const _: () = assert!(core::mem::offset_of!(sysevent_id_t, eid_ts) == 8);

/// The vendor string that drivers shipped with the system pass to
/// `ddi_log_sysevent(9F)` (`DDI_VENDOR_SUNW`). Third-party drivers use an
/// enduring identifier of their own, such as a stock symbol.
pub const DDI_VENDOR_SUNW: &core::ffi::CStr = c"SUNW";

/// Not enough memory to queue the event (`DDI_ENOMEM`). Never returned when
/// the caller passed `DDI_SLEEP`.
pub const DDI_ENOMEM: c_int = 1;
/// The system event queue is full (`DDI_EBUSY`). Never returned when the
/// caller passed `DDI_SLEEP`.
pub const DDI_EBUSY: c_int = 2;
/// The `syseventd` daemon is not responding, so events cannot be queued or
/// delivered at this time (`DDI_ETRANSPORT`). Possible even with `DDI_SLEEP`.
pub const DDI_ETRANSPORT: c_int = 3;
/// `DDI_SLEEP` was passed from interrupt context (`DDI_ECONTEXT`).
pub const DDI_ECONTEXT: c_int = 4;

/// The NDI event a nexus raises when a child node is removed
/// (`DDI_DEVI_REMOVE_EVENT`).
pub const DDI_DEVI_REMOVE_EVENT: &core::ffi::CStr = c"DDI:DEVI_REMOVE";
/// The NDI event a nexus raises when a child node is inserted
/// (`DDI_DEVI_INSERT_EVENT`).
pub const DDI_DEVI_INSERT_EVENT: &core::ffi::CStr = c"DDI:DEVI_INSERT";
/// The NDI event a nexus raises after a bus reset (`DDI_DEVI_BUS_RESET_EVENT`).
pub const DDI_DEVI_BUS_RESET_EVENT: &core::ffi::CStr = c"DDI:DEVI_BUS_RESET";
/// The NDI event a nexus raises after a device reset
/// (`DDI_DEVI_DEVICE_RESET_EVENT`).
pub const DDI_DEVI_DEVICE_RESET_EVENT: &core::ffi::CStr = c"DDI:DEVI_DEVICE_RESET";

/// The system event class for a device being added (`EC_DEV_ADD`).
pub const EC_DEV_ADD: &core::ffi::CStr = c"EC_dev_add";
/// The system event class for a device being removed (`EC_DEV_REMOVE`).
pub const EC_DEV_REMOVE: &core::ffi::CStr = c"EC_dev_remove";
/// The system event class for a device tree branch changing (`EC_DEV_BRANCH`).
pub const EC_DEV_BRANCH: &core::ffi::CStr = c"EC_dev_branch";
/// The system event class for a device status change (`EC_DEV_STATUS`).
pub const EC_DEV_STATUS: &core::ffi::CStr = c"EC_dev_status";
/// The system event class for dynamic reconfiguration (`EC_DR`).
pub const EC_DR: &core::ffi::CStr = c"EC_dr";
/// The system event class for environmental monitoring (`EC_ENV`).
pub const EC_ENV: &core::ffi::CStr = c"EC_env";
/// The system event class for fault management error reports (`EC_FM`).
pub const EC_FM: &core::ffi::CStr = c"EC_fm";
/// The system event class for events private to a platform (`EC_PLATFORM`).
pub const EC_PLATFORM: &core::ffi::CStr = c"EC_platform";
/// The system event class for datalink events (`EC_DATALINK`).
pub const EC_DATALINK: &core::ffi::CStr = c"EC_datalink";
/// The system event class for PCI Express events (`EC_PCIE`).
pub const EC_PCIE: &core::ffi::CStr = c"EC_pcie";
/// The subclass of `EC_DEV_ADD` and `EC_DEV_REMOVE` for a disk (`ESC_DISK`).
pub const ESC_DISK: &core::ffi::CStr = c"disk";
/// The subclass of `EC_DEV_ADD` and `EC_DEV_REMOVE` for a network interface
/// (`ESC_NETWORK`).
pub const ESC_NETWORK: &core::ffi::CStr = c"network";
/// The subclass of `EC_DEV_ADD` and `EC_DEV_REMOVE` for a printer (`ESC_PRINTER`).
pub const ESC_PRINTER: &core::ffi::CStr = c"printer";
/// The subclass of `EC_DEV_ADD` and `EC_DEV_REMOVE` for a lofi device (`ESC_LOFI`).
pub const ESC_LOFI: &core::ffi::CStr = c"lofi";
/// The subclass of `EC_DEV_BRANCH` for a branch being added (`ESC_DEV_BRANCH_ADD`).
pub const ESC_DEV_BRANCH_ADD: &core::ffi::CStr = c"ESC_dev_branch_add";
/// The subclass of `EC_DEV_BRANCH` for a branch being removed
/// (`ESC_DEV_BRANCH_REMOVE`).
pub const ESC_DEV_BRANCH_REMOVE: &core::ffi::CStr = c"ESC_dev_branch_remove";
/// The subclass of `EC_DEV_STATUS` for a dynamic LUN expansion (`ESC_DEV_DLE`).
pub const ESC_DEV_DLE: &core::ffi::CStr = c"ESC_dev_dle";
/// The subclass of `EC_DEV_STATUS` for an eject request (`ESC_DEV_EJECT_REQUEST`).
pub const ESC_DEV_EJECT_REQUEST: &core::ffi::CStr = c"ESC_dev_eject_request";
/// The subclass of `EC_DR` for an attachment point state change
/// (`ESC_DR_AP_STATE_CHANGE`).
pub const ESC_DR_AP_STATE_CHANGE: &core::ffi::CStr = c"ESC_dr_ap_state_change";
/// The subclass of `EC_DR` for a reconfiguration request (`ESC_DR_REQ`).
pub const ESC_DR_REQ: &core::ffi::CStr = c"ESC_dr_req";
/// The subclass of `EC_DR` for a target state change
/// (`ESC_DR_TARGET_STATE_CHANGE`).
pub const ESC_DR_TARGET_STATE_CHANGE: &core::ffi::CStr = c"ESC_dr_target_state_change";
/// The subclass of `EC_FM` for an error report (`ESC_FM_ERROR`).
pub const ESC_FM_ERROR: &core::ffi::CStr = c"ESC_FM_error";
/// The subclass of `EC_FM` for a replayed error report (`ESC_FM_ERROR_REPLAY`).
pub const ESC_FM_ERROR_REPLAY: &core::ffi::CStr = c"ESC_FM_error_replay";
/// The subclass of `EC_PCIE` for a link state change (`ESC_PCIE_LINK_STATE`).
pub const ESC_PCIE_LINK_STATE: &core::ffi::CStr = c"ESC_pcie_link_state";

unsafe extern "C" {
    /// `ddi_get_eventcookie(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the device tree for the cookie of the event called `name` on
    /// behalf of the child node `dip`. The request travels up the tree until
    /// a nexus driver satisfies it or the root is reached, and the cookie is
    /// stored through `event_cookiep`. The cookie is then used to register or
    /// unregister a callback handler or to post the event. Returns
    /// `DDI_SUCCESS` when a cookie was returned, and `DDI_FAILURE` when no
    /// nexus in the node's ancestry services that event. May be called from
    /// user and kernel context only.
    pub fn ddi_get_eventcookie(dip: dev_info_t, name: *mut c_char, event_cookiep: *mut ddi_eventcookie_t) -> c_int;
    /// `ddi_add_event_handler(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Subscribes `handler` to the event named by `cookie` on behalf of the
    /// node `dip`. When the event occurs the kernel calls the handler with
    /// the node, the cookie, the `arg` pointer given here, which is usually
    /// the driver's soft state, and a pointer to event-specific data supplied
    /// by the nexus. A driver instance may register several handlers for one
    /// event or one handler for several events; the order in which handlers
    /// run is undefined. The registration identifier stored through `id`
    /// must be kept for the later [`ddi_remove_event_handler`] call, and
    /// every handler must be removed before `detach(9E)` completes. Returns
    /// `DDI_SUCCESS`, or `DDI_FAILURE` when resources are short or the cookie
    /// is bad. Both this function and the handler run in user or kernel
    /// context only.
    pub fn ddi_add_event_handler(
        dip: dev_info_t,
        event: ddi_eventcookie_t,
        handler: ddi_event_cb_f,
        arg: *mut c_void,
        id: *mut ddi_callback_id_t,
    ) -> c_int;
    /// `ddi_remove_event_handler(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the callback handler identified by `id`, which was returned by
    /// [`ddi_add_event_handler`]. Once removed the handler is no longer
    /// invoked when the event occurs. Returns `DDI_SUCCESS` or `DDI_FAILURE`.
    /// May be called from user and kernel context only.
    pub fn ddi_remove_event_handler(id: ddi_callback_id_t) -> c_int;
    /// `ddi_log_sysevent(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Generates a system event of the given class and subclass on behalf of
    /// the driver and queues it for delivery to the user-space `syseventd`
    /// daemon. The publisher string is built from `vendor` and the driver
    /// name in the form `vendor:kern:driver-name`; drivers shipped with the
    /// system pass [`DDI_VENDOR_SUNW`]. The `attr_list` argument is an
    /// `nvlist_t` of name-value attributes, or NULL when the event carries
    /// none; the supported value types are the scalar and array forms of
    /// byte, 16-bit, 32-bit and 64-bit integers plus strings. When `eidp` is
    /// not NULL the sequence number and timestamp of the queued event are
    /// stored there. The `sleep_flag` argument is `DDI_SLEEP` to wait for
    /// resources or `DDI_NOSLEEP` to fail instead. Returns `DDI_SUCCESS`, or
    /// one of [`DDI_ENOMEM`], [`DDI_EBUSY`], [`DDI_ETRANSPORT`] and
    /// [`DDI_ECONTEXT`]. May be called from user, kernel or interrupt
    /// context, except that `DDI_SLEEP` is not permitted in interrupt context.
    pub fn ddi_log_sysevent(
        dip: dev_info_t,
        vendor: *mut c_char,
        class_name: *mut c_char,
        subclass_name: *mut c_char,
        attr_list: *mut nvlist_t,
        eidp: *mut sysevent_id_t,
        sleep_flag: c_int,
    ) -> c_int;
}
