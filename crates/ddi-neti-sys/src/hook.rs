//! The packet hook interface from `sys/hook.h` and `sys/hook_event.h`.
//!
//! A hook is a callback that a kernel module inserts into a networking
//! event, such as a packet arriving on a physical interface or an interface
//! being plumbed. The consumer allocates a `hook_t(9S)` with
//! `hook_alloc(9F)`, fills in the callback, its name and its argument, and
//! registers it on a protocol and event with `net_hook_register(9F)` from
//! the [`crate::neti`] module. Every time the event fires, the protocol
//! calls the hook with a token for the event, a `hook_data_t` that points
//! to a [`hook_pkt_event_t`] for packet events or to a
//! [`hook_nic_event_t`] for interface events, and the consumer's argument.
//! `hook_alloc(9F)` and `hook_free(9F)` resolve from `misc/hook`.
//!
//! The provider side of the framework, the `hook_family_t` and
//! `hook_event_t` structures, the `HOOK_RDONLY` flag and the
//! `HOOK_INIT`, `HOOK_FAMILY_INIT` and `HOOK_EVENT_INIT` macros, is not
//! bound. Those items serve the network protocols that define families and
//! events, have no manual page, and are only used with provider functions
//! that have none either. The `hook_pkt_observe_t` and `ipobs_hook_type_t`
//! declarations in `sys/hook_event.h` belong to IP observability and the
//! `hook_nic_event_int` structure is internal to IP, so they are left out
//! too. Nothing is taken from `sys/hook_impl.h`.

use crate::neti::{lif_if_t, net_handle_t, phy_if_t};
use ddi_sys::{c_char, c_int, c_void, mblk_t, size_t, uint_t, uintptr_t};

/// The version of the hook interface that this crate describes. It must be
/// passed to `hook_alloc(9F)`, which stores it in the `h_version` member of
/// the new hook (`HOOK_VERSION`).
pub const HOOK_VERSION: c_int = 1;

/// The data a hook receives about the event that fired it, `hook_data_t`.
/// It is an integer wide enough to hold a pointer, and for the events of
/// the IP protocols it holds a pointer to a [`hook_pkt_event_t`] or a
/// [`hook_nic_event_t`].
pub type hook_data_t = uintptr_t;

ddi_sys::opaque! {
    /// The kernel's record of an event, `struct hook_event_int`, from the
    /// private `sys/hook_impl.h`. Hooks only see it through a
    /// [`hook_event_token_t`].
    hook_event_int;
    /// The kernel's record of a registered hook, `struct hook_int`, from the
    /// private `sys/hook_impl.h`. It only appears through a
    /// [`hook_token_t`].
    hook_int;
}

/// A token that identifies the event a hook is being called for,
/// `hook_event_token_t`. The kernel passes it as the first argument of every
/// hook callback. It is a pointer to a private structure.
pub type hook_event_token_t = *mut hook_event_int;
/// A token that identifies a registered hook inside the kernel,
/// `hook_token_t`. It is a pointer to a private structure and no documented
/// function takes or returns one.
pub type hook_token_t = *mut hook_int;

/// `hook_t(9S)`. illumos DDI specific (illumos DDI).
///
/// The signature of a hook callback, stored in the `h_func` member of a
/// [`hook_t`]. The C header calls the type `hook_func_t`. The first argument
/// is the token of the event that fired, the second describes the event and
/// usually points to a [`hook_pkt_event_t`] or a [`hook_nic_event_t`], and
/// the third is the `h_arg` value the consumer registered.
///
/// The return value tells the protocol whether the packet should still be
/// considered valid. A callback that has changed the packet so that it must
/// no longer be processed, for example by freeing it with `freemsg(9F)`,
/// returns 1. Otherwise it returns 0. Once a callback returns non-zero, the
/// kernel stops calling the remaining hooks on the event.
pub type hook_func_t = unsafe extern "C" fn(token: hook_event_token_t, info: hook_data_t, arg: *mut c_void) -> c_int;

/// `net_event_notify_register(9F)`. illumos DDI specific (illumos DDI).
///
/// The command passed as the first argument of a [`hook_notify_fn_t`]
/// callback, `hook_notify_cmd_t`. It says whether the object named by the
/// other arguments has just been added or is being removed. The C type is an
/// enumeration.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct hook_notify_cmd_t(pub c_int);
impl hook_notify_cmd_t {
    /// No change. The manual pages only name the two values below as
    /// commands a callback receives (`HN_NONE`).
    pub const HN_NONE: Self = Self(0);
    /// The named hook, event, protocol or instance has been added
    /// (`HN_REGISTER`).
    pub const HN_REGISTER: Self = Self(1);
    /// The named hook, event, protocol or instance is being removed
    /// (`HN_UNREGISTER`).
    pub const HN_UNREGISTER: Self = Self(2);
}

/// `hook_t(9S)`. illumos DDI specific (illumos DDI).
///
/// Where a hook should be inserted among the other hooks on an event,
/// `hook_hint_t`, as stored in the `h_hint` member of a [`hook_t`]. A hint
/// only applies at the time of insertion and is not a rule that keeps the
/// hook in that place for its lifetime. The C type is an enumeration.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct hook_hint_t(pub c_int);
impl hook_hint_t {
    /// Insert the hook wherever convenient (`HH_NONE`).
    pub const HH_NONE: Self = Self(0);
    /// Place the hook first on the list of hooks. Only one hook on an event
    /// can hold this position (`HH_FIRST`).
    pub const HH_FIRST: Self = Self(1);
    /// Place the hook last on the list of hooks. Only one hook on an event
    /// can hold this position (`HH_LAST`).
    pub const HH_LAST: Self = Self(2);
    /// Place the hook before another hook, whose name `h_hintvalue` points
    /// to (`HH_BEFORE`).
    pub const HH_BEFORE: Self = Self(3);
    /// Place the hook after another hook, whose name `h_hintvalue` points to
    /// (`HH_AFTER`).
    pub const HH_AFTER: Self = Self(4);
}

/// `hook_t(9S)`. illumos DDI specific (illumos DDI).
///
/// A callback that is to be inserted into a networking event. The structure
/// must be allocated with `hook_alloc(9F)` and released with
/// `hook_free(9F)`, so that its size can change between versions of the
/// interface. The consumer fills in the members other than `h_version` and
/// passes it to `net_hook_register(9F)`. Using one structure for several
/// registrations is possible but discouraged.
///
/// The manual page gives `h_flags` the type `int` and does not mention
/// `h_version`. The declaration here follows the header.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hook_s {
    /// The version of the structure, set by `hook_alloc(9F)` from its
    /// argument. Consumers do not change it.
    pub h_version: c_int,
    /// The callback to invoke when the event fires. It must not be `None`
    /// when the hook is registered.
    pub h_func: Option<hook_func_t>,
    /// The name of the hook, which represents its owner. No two hooks on an
    /// event, and no two hooks on the same network stack, may share a name.
    pub h_name: *mut c_char,
    /// Extra properties of the hook. Currently unused and must be 0.
    pub h_flags: uint_t,
    /// The kind of insertion hint in `h_hintvalue`.
    pub h_hint: hook_hint_t,
    /// The value that goes with `h_hint`. For [`hook_hint_t::HH_BEFORE`]
    /// and [`hook_hint_t::HH_AFTER`] it is a pointer to the name of another
    /// hook. It is unused for the other hints.
    pub h_hintvalue: uintptr_t,
    /// A value of the consumer's choosing that is passed back as the third
    /// argument of `h_func`.
    pub h_arg: *mut c_void,
}
/// The typedef name of [`hook_s`], by which the manual page knows it.
pub type hook_t = hook_s;

// hook_s under LP64: h_version 4 at 0, 4 bytes of padding, h_func 8 at 8,
// h_name 8 at 16, h_flags 4 at 24, h_hint 4 at 28, h_hintvalue 8 at 32,
// h_arg 8 at 40; total 48.
const _: () = assert!(core::mem::size_of::<hook_s>() == 48);
const _: () = assert!(core::mem::offset_of!(hook_s, h_version) == 0);
const _: () = assert!(core::mem::offset_of!(hook_s, h_func) == 8);
const _: () = assert!(core::mem::offset_of!(hook_s, h_name) == 16);
const _: () = assert!(core::mem::offset_of!(hook_s, h_flags) == 24);
const _: () = assert!(core::mem::offset_of!(hook_s, h_hint) == 28);
const _: () = assert!(core::mem::offset_of!(hook_s, h_hintvalue) == 32);
const _: () = assert!(core::mem::offset_of!(hook_s, h_arg) == 40);

/// `net_event_notify_register(9F)`. illumos DDI specific (illumos DDI).
///
/// The signature of a notification callback, `hook_notify_fn_t`, as
/// registered with `net_event_notify_register(9F)`,
/// `net_protocol_notify_register(9F)` or
/// `net_instance_notify_register(9F)`. The kernel calls it when a hook,
/// protocol or instance is added or removed. The first argument is either
/// [`hook_notify_cmd_t::HN_REGISTER`] or
/// [`hook_notify_cmd_t::HN_UNREGISTER`], and the second is the argument that
/// was given at registration.
///
/// The meaning of the three names depends on the registration. For event
/// notifications they are the name of the protocol, the name of the event
/// and the name of the hook being added or removed. For protocol
/// notifications they are the name of the protocol, NULL, and the name of
/// the protocol being added or removed. For instance notifications they are
/// the network instance identifier written as a string, NULL, and the name
/// of the instance being added or removed.
///
/// The memory the names point to must be treated as read-only, and the
/// callback must not block other events. The manual pages do not define
/// the return value, and the kernel ignores it.
pub type hook_notify_fn_t = unsafe extern "C" fn(
    command: hook_notify_cmd_t,
    arg: *mut c_void,
    name1: *const c_char,
    name2: *const c_char,
    name3: *const c_char,
) -> c_int;

/// `hook_pkt_event(9S)`. illumos DDI specific (illumos DDI).
///
/// The description of a packet that the IP protocols pass to a hook for the
/// packet events `NH_PHYSICAL_IN`, `NH_PHYSICAL_OUT`, `NH_FORWARDING`,
/// `NH_LOOPBACK_IN` and `NH_LOOPBACK_OUT`, through the `hook_data_t`
/// argument of the callback. A callback may only modify `hpe_hdr`, `hpe_mp`
/// and `hpe_mb`. Which of the interface members are meaningful depends on
/// the event. Inbound events set `hpe_ifp`, outbound events set `hpe_ofp`,
/// and `NH_FORWARDING` sets both.
///
/// The manual page lists the first member as `hne_family`, gives
/// `hpe_flags` the type `uint32_t`, calls the inbound and outbound events
/// `NH_PRE_ROUTING` and `NH_POST_ROUTING`, and omits `hpe_reserved`. The
/// declaration here follows the header, whose event names are those in
/// [`crate::neti`].
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hook_pkt_event {
    /// The protocol the packet belongs to. It matches the handle returned by
    /// `net_protocol_lookup(9F)` and can be passed to the other `net_*`
    /// functions.
    pub hpe_protocol: net_handle_t,
    /// The inbound interface of the packet, for packets that come into the
    /// system or are forwarded.
    pub hpe_ifp: phy_if_t,
    /// The outbound interface of the packet, for packets that are
    /// transmitted or forwarded.
    pub hpe_ofp: phy_if_t,
    /// A pointer to the start of the network protocol header within the
    /// message block `hpe_mb`.
    pub hpe_hdr: *mut c_void,
    /// A pointer to the pointer to the first message block of the packet. A
    /// callback that replaces the packet stores the new chain through it.
    pub hpe_mp: *mut *mut mblk_t,
    /// The message block that contains `hpe_hdr`.
    pub hpe_mb: *mut mblk_t,
    /// Additional properties of the packet, a combination of
    /// [`HPE_MULTICAST`] and [`HPE_BROADCAST`].
    pub hpe_flags: c_int,
    /// Reserved for future use.
    pub hpe_reserved: [*mut c_void; 2],
}
/// The typedef name of [`hook_pkt_event`].
pub type hook_pkt_event_t = hook_pkt_event;

// hook_pkt_event under LP64: six pointer sized members from hpe_protocol at
// 0 to hpe_mb at 40, hpe_flags 4 at 48, 4 bytes of padding, hpe_reserved 16
// at 56; total 72.
const _: () = assert!(core::mem::size_of::<hook_pkt_event>() == 72);
const _: () = assert!(core::mem::offset_of!(hook_pkt_event, hpe_protocol) == 0);
const _: () = assert!(core::mem::offset_of!(hook_pkt_event, hpe_hdr) == 24);
const _: () = assert!(core::mem::offset_of!(hook_pkt_event, hpe_mb) == 40);
const _: () = assert!(core::mem::offset_of!(hook_pkt_event, hpe_flags) == 48);
const _: () = assert!(core::mem::offset_of!(hook_pkt_event, hpe_reserved) == 56);

/// The packet was recognised as a multicast packet by the link layer. This
/// bit and [`HPE_BROADCAST`] are never set together, and currently only
/// packets on the physical inbound event carry it (`HPE_MULTICAST`).
pub const HPE_MULTICAST: c_int = 0x01;
/// The packet was recognised as a broadcast packet by the link layer. This
/// bit and [`HPE_MULTICAST`] are never set together, and currently only
/// packets on the physical inbound event carry it (`HPE_BROADCAST`).
pub const HPE_BROADCAST: c_int = 0x02;

/// `hook_nic_event(9S)`. illumos DDI specific (illumos DDI).
///
/// The kind of interface event described by a [`hook_nic_event_t`],
/// `nic_event_t`. The C type is an enumeration. The manual page describes
/// the first five values, and the header adds the last three.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct nic_event_t(pub c_int);
impl nic_event_t {
    /// An interface has just been created (`NE_PLUMB`).
    pub const NE_PLUMB: Self = Self(1);
    /// An interface has just been destroyed and no more events will be
    /// received for it (`NE_UNPLUMB`).
    pub const NE_UNPLUMB: Self = Self(2);
    /// An interface has changed state to up and may now generate packet
    /// events (`NE_UP`).
    pub const NE_UP: Self = Self(3);
    /// An interface has changed state to down and will no longer generate
    /// packet events (`NE_DOWN`).
    pub const NE_DOWN: Self = Self(4);
    /// An address on an interface has changed. `hne_lif` names the logical
    /// interface concerned, and `hne_data` points to a socket address of
    /// `hne_datalen` bytes that holds the new address
    /// (`NE_ADDRESS_CHANGE`).
    pub const NE_ADDRESS_CHANGE: Self = Self(5);
    /// A logical interface has changed state to up. The manual page does not
    /// describe this event (`NE_LIF_UP`).
    pub const NE_LIF_UP: Self = Self(6);
    /// A logical interface has changed state to down. The manual page does
    /// not describe this event (`NE_LIF_DOWN`).
    pub const NE_LIF_DOWN: Self = Self(7);
    /// The interface index of an interface has changed. The manual page does
    /// not describe this event (`NE_IFINDEX_CHANGE`).
    pub const NE_IFINDEX_CHANGE: Self = Self(8);
}

/// Extra data about an interface event, `nic_event_data_t`. It is a plain
/// `void *` whose meaning depends on the event, or NULL if there is none.
pub type nic_event_data_t = *mut c_void;

/// `hook_nic_event(9S)`. illumos DDI specific (illumos DDI).
///
/// The description of an event on a network interface, passed to hooks on
/// the `NH_NIC_EVENTS` event through the `hook_data_t` argument of the
/// callback. A callback may not change any of its members.
///
/// The manual page lists the first two members as `hne_family` and
/// `pkt_private`. The declaration here follows the header, which calls them
/// `hne_protocol` and `hne_nic`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hook_nic_event {
    /// The protocol that owns the interface. It can be passed to the other
    /// `net_*` functions.
    pub hne_protocol: net_handle_t,
    /// The physical interface the event belongs to.
    pub hne_nic: phy_if_t,
    /// The logical interface the event belongs to, if any.
    pub hne_lif: lif_if_t,
    /// The kind of event.
    pub hne_event: nic_event_t,
    /// A pointer to extra data about the event, or NULL if there is none.
    pub hne_data: nic_event_data_t,
    /// The size in bytes of the data `hne_data` points to, which can be 0.
    pub hne_datalen: size_t,
}
/// The typedef name of [`hook_nic_event`].
pub type hook_nic_event_t = hook_nic_event;

// hook_nic_event under LP64: hne_protocol 8 at 0, hne_nic 8 at 8, hne_lif 8
// at 16, hne_event 4 at 24, 4 bytes of padding, hne_data 8 at 32,
// hne_datalen 8 at 40; total 48.
const _: () = assert!(core::mem::size_of::<hook_nic_event>() == 48);
const _: () = assert!(core::mem::offset_of!(hook_nic_event, hne_protocol) == 0);
const _: () = assert!(core::mem::offset_of!(hook_nic_event, hne_event) == 24);
const _: () = assert!(core::mem::offset_of!(hook_nic_event, hne_data) == 32);
const _: () = assert!(core::mem::offset_of!(hook_nic_event, hne_datalen) == 40);

unsafe extern "C" {
    /// `hook_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates a [`hook_t`] structure, zeroed apart from its version, and
    /// returns a pointer to it for the caller to fill in. `version` must
    /// always be [`HOOK_VERSION`]. The allocation sleeps until memory is
    /// available.
    ///
    /// The function returns a pointer to the new structure on success and
    /// NULL on failure. It may be called from user or kernel context.
    pub fn hook_alloc(version: c_int) -> *mut hook_t;

    /// `hook_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees a [`hook_t`] structure that `hook_alloc(9F)` allocated. It only
    /// frees the structure itself, so memory the caller allocated for
    /// `h_name` or `h_arg` must be freed separately. The hook must no longer
    /// be registered.
    ///
    /// The function may be called from user, kernel or interrupt context.
    pub fn hook_free(hook: *mut hook_t);
}
