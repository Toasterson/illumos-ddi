//! The network information interface from `sys/neti.h`.
//!
//! The network information interface, described in `netinfo(9F)`, gives a
//! kernel module access to data and functions of a network layer protocol
//! implementation without linking against that implementation. A consumer
//! first finds the protocol with `net_protocol_lookup(9F)`, giving the
//! identifier of an IP instance and a protocol name such as [`NHF_INET`],
//! and receives a [`net_handle_t`]. With that handle it can walk the
//! physical and logical interfaces the protocol owns, ask for interface
//! names, addresses and MTUs, look up routes, check checksums, inject
//! packets with `net_inject(9F)` and register packet hooks with
//! `net_hook_register(9F)`. When it no longer needs the handle it gives it
//! back with `net_protocol_release(9F)`. A protocol need not implement every
//! function, and one it does not implement returns -1.
//!
//! Every zone with an exclusive IP stack has its own IP instance, identified
//! by a [`netid_t`]. A module that wants to act in every instance registers
//! a `net_instance_t(9S)` with `net_instance_register(9F)`, and the kernel
//! then calls its create, shutdown and destroy callbacks as instances come
//! and go. Protocols, events and instances can also be watched with the
//! `net_*_notify_register` functions.
//!
//! Every function here resolves from `misc/neti`. Every page in the family
//! gives the interface level "illumos DDI specific (illumos DDI)" and the
//! interface stability Committed.
//!
//! The manual pages and the header disagree in many places, because the
//! pages were written for an earlier revision of the interface. Where they
//! do, the declarations here follow the header, since that is what the
//! kernel was compiled against, and the doc comment of each item says what
//! differs.
//!
//! Several declarations of the header are not bound. The functions a
//! protocol uses to make itself known, namely `net_protocol_register`,
//! `net_protocol_unregister`, `net_family_register`, `net_family_shutdown`,
//! `net_family_unregister`, `net_event_register`, `net_event_shutdown` and
//! `net_event_unregister`, have no manual page and take the unbound
//! provider structures of `sys/hook.h`. The accessors `net_getlifzone` and
//! `net_getlifflags` have no manual page either. `neti_init`, `neti_fini`,
//! `net_getnetistackbyid`, `net_getnetstackidbynetid`,
//! `net_getnetidbynetstackid` and `net_getzoneidbynetid` are marked in the
//! header as internal to the module. The structures `net_protocol_t`,
//! `struct net_data`, `injection_t`, `net_instance_int_t` and
//! `neti_stack_t` are private and only `struct net_data` appears here, as
//! an opaque type behind [`net_handle_t`]. The `NET_IS_HCK_*` macros are
//! simple tests of the result of `net_ispartialchecksum(9F)` and are left to
//! the safe crate, together with the `MAP_IPIF_ID` and `UNMAP_IPIF_ID`
//! macros, which are internal to IP. Nothing is taken from
//! `sys/neti_impl.h`.
//!
//! The `net_netidtozonid(9F)` page documents a function that the kernel
//! does not export under that name. The function that does the job is
//! `net_getzoneidbynetid`, which the header marks as internal, so neither is
//! declared. The `net_inject_alloc(9F)` page refers to a symbol
//! `NETI_VERSION`, which does not exist; [`NETINFO_VERSION`] is meant.

use crate::hook::{hook_notify_fn_t, hook_t};
use ddi_sys::{c_char, c_int, c_uchar, c_void, id_t, intptr_t, kstat_t, mblk_t, size_t, uintptr_t, ulong_t, zoneid_t};

/// The version of the network information interface that this crate
/// describes. It must be passed to `net_instance_alloc(9F)`, which refuses
/// any other value, and to `net_inject_alloc(9F)` (`NETINFO_VERSION`).
pub const NETINFO_VERSION: c_int = 1;

/// The protocol name of IPv4, to be passed to `net_protocol_lookup(9F)`
/// (`NHF_INET`).
pub const NHF_INET: &core::ffi::CStr = c"NHF_INET";
/// The protocol name of IPv6, to be passed to `net_protocol_lookup(9F)`
/// (`NHF_INET6`).
pub const NHF_INET6: &core::ffi::CStr = c"NHF_INET6";
/// The protocol name under which IP registers its ARP packet events
/// (`NHF_ARP`).
pub const NHF_ARP: &core::ffi::CStr = c"NHF_ARP";
/// The protocol name under which the `viona` driver of bhyve registers the
/// packet events of its virtual network interfaces (`NHF_VIONA`).
pub const NHF_VIONA: &core::ffi::CStr = c"NHF_VIONA";

/// The name of the event for packets received from a physical interface.
/// The hooks receive a `hook_pkt_event(9S)` with `hpe_ifp` set
/// (`NH_PHYSICAL_IN`).
pub const NH_PHYSICAL_IN: &core::ffi::CStr = c"PHYSICAL_IN";
/// The name of the event for packets about to be sent out of a physical
/// interface. The hooks receive a `hook_pkt_event(9S)` with `hpe_ofp` set
/// (`NH_PHYSICAL_OUT`).
pub const NH_PHYSICAL_OUT: &core::ffi::CStr = c"PHYSICAL_OUT";
/// The name of the event for packets being forwarded from one interface to
/// another. The hooks receive a `hook_pkt_event(9S)` with both `hpe_ifp`
/// and `hpe_ofp` set (`NH_FORWARDING`).
pub const NH_FORWARDING: &core::ffi::CStr = c"FORWARDING";
/// The name of the event for packets received on the loopback path
/// (`NH_LOOPBACK_IN`).
pub const NH_LOOPBACK_IN: &core::ffi::CStr = c"LOOPBACK_IN";
/// The name of the event for packets sent on the loopback path
/// (`NH_LOOPBACK_OUT`).
pub const NH_LOOPBACK_OUT: &core::ffi::CStr = c"LOOPBACK_OUT";
/// The name of the event for changes to network interfaces. The hooks
/// receive a `hook_nic_event(9S)` (`NH_NIC_EVENTS`).
pub const NH_NIC_EVENTS: &core::ffi::CStr = c"NIC_EVENTS";
/// The name of the event IP uses to pass copies of packets to observability
/// consumers. Its payload, `hook_pkt_observe_t`, has no manual page and is
/// not bound (`NH_OBSERVE`).
pub const NH_OBSERVE: &core::ffi::CStr = c"OBSERVING";

/// The packet is not marked for hardware checksum calculation. This is the
/// value 0 that `net_ispartialchecksum(9F)` returns in that case
/// (`NET_HCK_NONE`).
pub const NET_HCK_NONE: c_int = 0x00;
/// The hardware calculates the complete layer 3 checksum
/// (`NET_HCK_L3_FULL`).
pub const NET_HCK_L3_FULL: c_int = 0x01;
/// The hardware completes a partial layer 3 checksum (`NET_HCK_L3_PART`).
pub const NET_HCK_L3_PART: c_int = 0x02;
/// The hardware calculates the complete layer 4 checksum
/// (`NET_HCK_L4_FULL`).
pub const NET_HCK_L4_FULL: c_int = 0x10;
/// The hardware completes a partial layer 4 checksum (`NET_HCK_L4_PART`).
pub const NET_HCK_L4_PART: c_int = 0x20;

/// A physical network interface as seen by a protocol, `phy_if_t`. It is an
/// unsigned integer wide enough to hold a pointer. The value 0 means that
/// there is no interface, and the value that is all ones, -1 in C, means
/// that the protocol does not support the call that returned it.
pub type phy_if_t = uintptr_t;
/// A logical interface on a physical interface, `lif_if_t`. It is a signed
/// integer wide enough to hold a pointer. Logical interfaces are numbered
/// from 1, so that 0 can mean the end of a list.
pub type lif_if_t = intptr_t;
/// The identifier of an IP instance, `netid_t`. It is an `id_t`, which is a
/// plain `int`. The value -1 means that there is no instance.
pub type netid_t = id_t;

/// `net_getlifaddr(9F)`. illumos DDI specific (illumos DDI).
///
/// The kind of address `net_getlifaddr(9F)` is asked for, `net_ifaddr_t`.
/// The caller passes an array of these values together with an array of
/// socket address structures of the same length. The C type is an
/// enumeration.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct net_ifaddr_t(pub c_int);
impl net_ifaddr_t {
    /// The network address of the logical interface (`NA_ADDRESS`).
    pub const NA_ADDRESS: Self = Self(1);
    /// The address of the remote host of a point to point interface
    /// (`NA_PEER`).
    pub const NA_PEER: Self = Self(2);
    /// The broadcast address of an interface that supports broadcast
    /// (`NA_BROADCAST`).
    pub const NA_BROADCAST: Self = Self(3);
    /// The netmask of the logical interface (`NA_NETMASK`).
    pub const NA_NETMASK: Self = Self(4);
}

/// `net_inject(9F)`. illumos DDI specific (illumos DDI).
///
/// How `net_inject(9F)` delivers a packet, `inject_t`. The C type is an
/// enumeration.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct inject_t(pub c_int);
impl inject_t {
    /// Queue the packet for delivery up into the kernel as if it had been
    /// received on the interface `ni_physical`. The destination address is
    /// ignored. The packet passes through the physical inbound hooks again
    /// (`NI_QUEUE_IN`).
    pub const NI_QUEUE_IN: Self = Self(1);
    /// Queue the packet for delivery out of the kernel as if it had been
    /// sent on a raw socket. The destination address and the interface are
    /// ignored. The packet passes through the physical outbound hooks again
    /// (`NI_QUEUE_OUT`).
    pub const NI_QUEUE_OUT: Self = Self(2);
    /// Send the packet out of the interface `ni_physical` at once, with no
    /// processing beyond adding the link layer header. `ni_addr` may name
    /// the next hop for link layer address resolution (`NI_DIRECT_OUT`).
    pub const NI_DIRECT_OUT: Self = Self(3);
}

/// A socket address family, `sa_family_t`, from `sys/socket_impl.h`. It is
/// a 16-bit unsigned integer.
pub type sa_family_t = u16;

/// A generic socket address, `struct sockaddr`, from `sys/socket_impl.h`.
/// The functions here take a pointer to one in place of the address of a
/// specific family, such as `struct sockaddr_in` for IPv4 or
/// `struct sockaddr_in6` for IPv6, and the caller casts accordingly.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct sockaddr {
    /// The address family, such as `AF_INET` or `AF_INET6`.
    pub sa_family: sa_family_t,
    /// The first bytes of the address itself.
    pub sa_data: [c_char; 14],
}

// sockaddr: sa_family 2 at 0, sa_data 14 at 2; total 16 with an alignment
// of 2.
const _: () = assert!(core::mem::size_of::<sockaddr>() == 16);
const _: () = assert!(core::mem::offset_of!(sockaddr, sa_family) == 0);
const _: () = assert!(core::mem::offset_of!(sockaddr, sa_data) == 2);

/// Storage large enough and aligned enough for any socket address,
/// `struct sockaddr_storage`, from `sys/socket_impl.h`. It is embedded in
/// `net_inject_t(9S)`. Only `ss_family` has a meaning of its own; the rest
/// is filled in through a cast to the address structure of the family.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct sockaddr_storage {
    /// The address family, which says how the rest is to be read.
    pub ss_family: sa_family_t,
    /// Padding up to the alignment member.
    pub _ss_pad1: [c_char; 6],
    /// A `double`, which gives the structure its 8 byte alignment. The C
    /// header calls the type `sockaddr_maxalign_t`.
    pub _ss_align: f64,
    /// Padding up to the full size of 256 bytes.
    pub _ss_pad2: [c_char; 240],
}

// sockaddr_storage under LP64: ss_family 2 at 0, _ss_pad1 6 at 2, _ss_align
// 8 at 8, _ss_pad2 240 at 16; total 256 with an alignment of 8. The pad
// sizes follow from _SS_MAXSIZE 256 and _SS_ALIGNSIZE 8.
const _: () = assert!(core::mem::size_of::<sockaddr_storage>() == 256);
const _: () = assert!(core::mem::align_of::<sockaddr_storage>() == 8);
const _: () = assert!(core::mem::offset_of!(sockaddr_storage, ss_family) == 0);
const _: () = assert!(core::mem::offset_of!(sockaddr_storage, _ss_align) == 8);
const _: () = assert!(core::mem::offset_of!(sockaddr_storage, _ss_pad2) == 16);

/// `net_inject_t(9S)`. illumos DDI specific (illumos DDI).
///
/// The description of a packet to be transmitted with `net_inject(9F)`,
/// where transmitting includes sending the packet up into the system as
/// well as out of it. It should be allocated with `net_inject_alloc(9F)` and
/// freed with `net_inject_free(9F)`.
///
/// The manual page lists only `ni_packet`, `ni_addr` and `ni_physical`. The
/// header adds `ni_version` and `ni_netid` in front of them.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct net_inject {
    /// The version of the structure, set by `net_inject_alloc(9F)` from its
    /// argument.
    pub ni_version: c_int,
    /// The IP instance the packet belongs to.
    pub ni_netid: netid_t,
    /// The first message block of the packet.
    pub ni_packet: *mut mblk_t,
    /// The address of the next hop, only needed with
    /// [`inject_t::NI_DIRECT_OUT`]. Its family says whether it is to be read
    /// as a `struct sockaddr_in` for IPv4 or a `struct sockaddr_in6` for
    /// IPv6.
    pub ni_addr: sockaddr_storage,
    /// The physical interface on which the packet is to be injected.
    pub ni_physical: phy_if_t,
}
/// The typedef name of [`struct@net_inject`].
pub type net_inject_t = net_inject;

// net_inject under LP64: ni_version 4 at 0, ni_netid 4 at 4, ni_packet 8 at
// 8, ni_addr 256 at 16 (aligned to 8), ni_physical 8 at 272; total 280.
const _: () = assert!(core::mem::size_of::<net_inject>() == 280);
const _: () = assert!(core::mem::offset_of!(net_inject, ni_version) == 0);
const _: () = assert!(core::mem::offset_of!(net_inject, ni_netid) == 4);
const _: () = assert!(core::mem::offset_of!(net_inject, ni_packet) == 8);
const _: () = assert!(core::mem::offset_of!(net_inject, ni_addr) == 16);
const _: () = assert!(core::mem::offset_of!(net_inject, ni_physical) == 272);

ddi_sys::opaque! {
    /// The kernel's record of a registered protocol, `struct net_data`. It
    /// is declared in `sys/neti.h` but private to the framework, and a
    /// consumer only ever holds a [`net_handle_t`] that points to one. The
    /// manual pages call the handle `net_data_t`.
    net_data;
}

/// A handle for a network protocol in one IP instance, `net_handle_t`. It
/// is returned by `net_protocol_lookup(9F)` and `net_protocol_walk(9F)` and
/// holds a reference that `net_protocol_release(9F)` gives back. Many
/// manual pages spell the type `net_data_t`.
pub type net_handle_t = *mut net_data;

/// `net_instance_t(9S)`. illumos DDI specific (illumos DDI).
///
/// The signature of the `nin_create` callback of a [`net_instance_t`]. The
/// kernel calls it when a new IP instance is created, and for every
/// instance that already exists when the structure is registered. The
/// argument identifies the instance. The callback returns a pointer to the
/// module's state for that instance, which the kernel stores and later
/// passes to `nin_shutdown` and `nin_destroy`. It may not return NULL. The C
/// structure declares the member inline, without a type name.
pub type nin_create_t = unsafe extern "C" fn(netid: netid_t) -> *mut c_void;

/// `net_instance_t(9S)`. illumos DDI specific (illumos DDI).
///
/// The signature of the `nin_destroy` callback of a [`net_instance_t`]. The
/// kernel calls it when an IP instance is being destroyed, after
/// `nin_shutdown`. The arguments are the identifier of the instance and the
/// pointer `nin_create` returned for it, and the callback frees that state.
/// The C structure declares the member inline, without a type name.
pub type nin_destroy_t = unsafe extern "C" fn(netid: netid_t, arg: *mut c_void);

/// `net_instance_t(9S)`. illumos DDI specific (illumos DDI).
///
/// The signature of the `nin_shutdown` callback of a [`net_instance_t`]. The
/// kernel calls it when an IP instance is being shut down, before
/// `nin_destroy`. The arguments are the identifier of the instance and the
/// pointer `nin_create` returned for it. The C structure declares the
/// member inline, without a type name.
pub type nin_shutdown_t = unsafe extern "C" fn(netid: netid_t, arg: *mut c_void);

/// `net_instance_t(9S)`. illumos DDI specific (illumos DDI).
///
/// A set of callbacks to be called when IP instances are created, shut down
/// and destroyed. A module allocates it with `net_instance_alloc(9F)`, fills
/// in the name and the callbacks, and registers it with
/// `net_instance_register(9F)`. The value `nin_create` returns for an
/// instance is stored by the kernel and passed back as the second argument
/// of `nin_shutdown` and `nin_destroy`, and the `netid_t` passed to each
/// callback identifies the instance.
///
/// The manual page does not list `nin_version`. The page's name line calls
/// the structure a packet event structure passed through to hooks, which is
/// a copying error in the page.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct net_instance_s {
    /// The version of the structure, set by `net_instance_alloc(9F)`.
    pub nin_version: c_int,
    /// The name of the owner of the instance. It must be unique among the
    /// registered sets of callbacks.
    pub nin_name: *mut c_char,
    /// Called when a new IP instance is created.
    pub nin_create: Option<nin_create_t>,
    /// Called when an IP instance is being destroyed.
    pub nin_destroy: Option<nin_destroy_t>,
    /// Called when an IP instance is being shut down, before `nin_destroy`.
    pub nin_shutdown: Option<nin_shutdown_t>,
}
/// The typedef name of [`net_instance_s`], by which the manual page knows it.
pub type net_instance_t = net_instance_s;

// net_instance_s under LP64: nin_version 4 at 0, 4 bytes of padding,
// nin_name 8 at 8, nin_create 8 at 16, nin_destroy 8 at 24, nin_shutdown 8
// at 32; total 40.
const _: () = assert!(core::mem::size_of::<net_instance_s>() == 40);
const _: () = assert!(core::mem::offset_of!(net_instance_s, nin_version) == 0);
const _: () = assert!(core::mem::offset_of!(net_instance_s, nin_name) == 8);
const _: () = assert!(core::mem::offset_of!(net_instance_s, nin_create) == 16);
const _: () = assert!(core::mem::offset_of!(net_instance_s, nin_shutdown) == 32);

unsafe extern "C" {
    /// `net_protocol_lookup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Locates the implementation of the network protocol named `protocol`
    /// in the IP instance `id` and takes a reference on it. `protocol` must
    /// be the name of a registered protocol, such as [`NHF_INET`] for IPv4
    /// or [`NHF_INET6`] for IPv6. The handle stays valid until it is passed
    /// to `net_protocol_release(9F)`.
    ///
    /// The function returns NULL if it knows nothing about the protocol, and
    /// otherwise a handle for use with the other functions of the
    /// framework. It may be called from user, kernel or interrupt context.
    pub fn net_protocol_lookup(id: netid_t, protocol: *const c_char) -> net_handle_t;

    /// `net_protocol_release(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Tells the framework that the caller no longer needs the handle
    /// `info`, which `net_protocol_lookup(9F)` or `net_protocol_walk(9F)`
    /// returned, and drops the reference it held.
    ///
    /// The page says that the function returns 0 on success, 1 if it has
    /// been called too many times and -1 if the value passed in is unknown.
    /// The current implementation always returns 0. It may be called from
    /// user, kernel or interrupt context.
    pub fn net_protocol_release(info: net_handle_t) -> c_int;

    /// `net_protocol_walk(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Steps through the network protocols registered in the IP instance
    /// `netid`. The first call passes NULL as `info`, and each later call
    /// passes the handle the previous call returned. The function releases
    /// the handle it is given and takes a reference on the one it returns,
    /// so a caller that stops before the end must pass the last handle to
    /// `net_protocol_release(9F)`.
    ///
    /// The page shows a single `net_data_t` argument. The header adds the
    /// instance identifier in front of it.
    ///
    /// The function returns NULL at the end of the list and otherwise a
    /// handle that can be used with the other functions of the framework.
    /// It may be called from user, kernel or interrupt context.
    pub fn net_protocol_walk(netid: netid_t, info: net_handle_t) -> net_handle_t;

    /// `net_protocol_notify_register(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers `callback` to be called when a protocol is added to or
    /// removed from the IP instance of `family`, a handle from
    /// `net_protocol_lookup(9F)`. Several callbacks may be registered, and
    /// each is called with `arg`, the command, the name of the protocol,
    /// NULL, and the name of the protocol being added or removed. The
    /// callback must treat the names as read-only and must not block other
    /// protocols.
    ///
    /// The page's synopsis declares `callback` as a pointer to a
    /// `hook_notify_fn_t`. The header takes the function pointer itself.
    ///
    /// The function returns 0 on success and `EEXIST` if the callback is
    /// already registered. It may be called from user or kernel context.
    pub fn net_protocol_notify_register(family: net_handle_t, callback: hook_notify_fn_t, arg: *mut c_void) -> c_int;

    /// `net_protocol_notify_unregister(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes `callback` from the functions that
    /// `net_protocol_notify_register(9F)` registered for `family`. The
    /// page's name line calls the function `net_instance_protocol_unregister`,
    /// which is a mistake in the page.
    ///
    /// The function returns 0 on success and an error number if the callback
    /// cannot be found. It may be called from user or kernel context.
    pub fn net_protocol_notify_unregister(family: net_handle_t, callback: hook_notify_fn_t) -> c_int;

    /// `net_getnetid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the identifier of the IP instance to which the protocol handle
    /// `net` belongs. `net` is a handle from `net_protocol_lookup(9F)`.
    ///
    /// `sys/neti.h` does not declare this function, but `misc/neti` defines
    /// and exports it with this signature. The page shows the argument as a
    /// `net_data_t`, which is the same handle.
    ///
    /// The function returns the instance identifier, or -1 if the protocol
    /// is no longer attached to an instance. It may be called from user,
    /// kernel or interrupt context.
    pub fn net_getnetid(net: net_handle_t) -> netid_t;

    /// `net_zoneidtonetid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Maps the running zone `zone` to the identifier of the IP instance that
    /// supports its network functions. For a zone that shares an IP
    /// instance, that is the instance of the global zone.
    ///
    /// The function returns -1 if no mapping took place, and otherwise the
    /// instance identifier the zone currently uses. It may be called from
    /// user, kernel or interrupt context.
    pub fn net_zoneidtonetid(zone: zoneid_t) -> netid_t;

    /// `net_getifname(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies the name of the physical interface `ifp` of the protocol `net`
    /// into `buffer`, which is `buflen` bytes long. The name is always
    /// terminated with a NUL and is truncated if the buffer is too small.
    /// `ifp` comes from `net_phylookup(9F)` or `net_phygetnext(9F)`.
    ///
    /// The function returns 0 on success, 1 on failure and -1 if the
    /// protocol does not support it. It may be called from user, kernel or
    /// interrupt context.
    pub fn net_getifname(net: net_handle_t, ifp: phy_if_t, buffer: *mut c_char, buflen: size_t) -> c_int;

    /// `net_getmtu(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the current MTU of an interface of the protocol `net`. `ifp`
    /// is the physical interface and `lif` a logical interface on it. IP
    /// treats 0 and 1 alike as the first logical interface. The MTU is not
    /// guaranteed to stay constant, so the result should not be cached.
    ///
    /// The page shows only the protocol and the physical interface. The
    /// header adds the logical interface.
    ///
    /// The function returns the MTU, or -1 if the protocol does not support
    /// the call. The IP implementation returns 0 if the interface cannot be
    /// found. The function may be called from user, kernel or interrupt
    /// context.
    pub fn net_getmtu(net: net_handle_t, ifp: phy_if_t, lif: lif_if_t) -> c_int;

    /// `net_getpmtuenabled(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Tells whether path MTU discovery is enabled for the protocol `net`.
    ///
    /// The function returns 1 if it is enabled, 0 if it is disabled and -1
    /// if the protocol does not support the call. It may be called from
    /// user, kernel or interrupt context.
    pub fn net_getpmtuenabled(net: net_handle_t) -> c_int;

    /// `net_getlifaddr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Retrieves addresses of the logical interface `lif` on the physical
    /// interface `ifp` of the protocol `net`. `type_` points to an array of
    /// `nelem` [`net_ifaddr_t`] values that say which addresses are wanted,
    /// and `storage` points to an array of `nelem` socket address
    /// structures of the protocol's family, such as `struct sockaddr_in` for
    /// IPv4, into which the addresses are copied in the same order.
    ///
    /// The page shows a single type and a single `struct sockaddr` pointer.
    /// The header takes the count, the array of types and a `void` pointer
    /// to the array of addresses, and the prototype here follows the header.
    ///
    /// The function returns 0 on success, 1 on failure and -1 if the
    /// protocol does not support it. It may be called from user, kernel or
    /// interrupt context.
    pub fn net_getlifaddr(
        net: net_handle_t,
        ifp: phy_if_t,
        lif: lif_if_t,
        nelem: c_int,
        type_: *mut net_ifaddr_t,
        storage: *mut c_void,
    ) -> c_int;

    /// `net_phygetnext(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Steps through the physical interfaces that the protocol `net` owns.
    /// The first call passes 0 as `ifp`, and each later call passes the
    /// value the previous call returned. A returned value identifies an
    /// interface that existed at the time of the call and is only
    /// guaranteed to be unique within the protocol.
    ///
    /// The function returns the next interface, 0 once the last one has
    /// been passed, or -1 if the protocol does not support the call. It may
    /// be called from user, kernel or interrupt context.
    pub fn net_phygetnext(net: net_handle_t, ifp: phy_if_t) -> phy_if_t;

    /// `net_phylookup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Looks up the physical interface called `name` in the protocol `net`.
    /// A returned value identifies an interface that existed at the time of
    /// the call and is only guaranteed to be unique within the protocol.
    ///
    /// The function returns a value greater than 0 for the interface, 0 if
    /// no interface of that name is known, or -1 if the protocol does not
    /// support the call. It may be called from user, kernel or interrupt
    /// context.
    pub fn net_phylookup(net: net_handle_t, name: *const c_char) -> phy_if_t;

    /// `net_lifgetnext(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Steps through the logical interfaces on the physical interface `ifp`
    /// of the protocol `net`. The first call passes 0 as `lif`, and each
    /// later call passes the value the previous call returned. A returned
    /// value identifies a logical interface that existed at the time of the
    /// call and is only guaranteed to be unique within the physical
    /// interface and its protocol. The page spells the type of `lif` as
    /// `net_if_t`.
    ///
    /// The function returns the next logical interface, 0 once the last one
    /// has been passed, or -1 if the protocol does not support the call. It
    /// may be called from user, kernel or interrupt context.
    pub fn net_lifgetnext(net: net_handle_t, ifp: phy_if_t, lif: lif_if_t) -> lif_if_t;

    /// `net_routeto(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Tells which physical interface packets for `address` would leave
    /// through, according to the routing tables of the protocol `net`. If
    /// `nexthop` is not NULL, the address of the next hop router is stored
    /// there. Both addresses are of the protocol's family, cast to a
    /// generic socket address.
    ///
    /// The function returns the interface, 0 if no route to the address is
    /// found, or -1 if the protocol does not support the call. It may be
    /// called from user, kernel or interrupt context.
    pub fn net_routeto(net: net_handle_t, address: *mut sockaddr, nexthop: *mut sockaddr) -> phy_if_t;

    /// `net_ispartialchecksum(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Looks at the message block `mb`, which holds a packet of the protocol
    /// `net`, to find out whether its headers carry only partial checksums
    /// because the hardware is to complete them.
    ///
    /// The function returns 0 if the packet does not contain partial
    /// checksums and -1 if the protocol does not support the call. If the
    /// packet is marked for hardware checksum calculation, it returns a
    /// combination of [`NET_HCK_L3_FULL`], [`NET_HCK_L3_PART`],
    /// [`NET_HCK_L4_FULL`] and [`NET_HCK_L4_PART`]. It may be called from
    /// user, kernel or interrupt context.
    pub fn net_ispartialchecksum(net: net_handle_t, mb: *mut mblk_t) -> c_int;

    /// `net_isvalidchecksum(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Verifies the layer 3 checksum of the packet in the message block
    /// `mb`, which belongs to the protocol `net`. The page says that for
    /// TCP, UDP and ICMP over IPv4 or IPv6 the layer 4 checksum is verified
    /// as well. Where possible the function relies on the fields that
    /// hardware checksum offload fills in rather than recomputing the sums.
    ///
    /// The page describes the return values as 0 if the packet does not
    /// contain partial checksums, 1 if it does, and -1 if the protocol does
    /// not support the call, which appears to be copied from
    /// `net_ispartialchecksum(9F)`. The IP implementation returns 1 when the
    /// checksums it examines are valid and 0 when they are not. The function
    /// may be called from user, kernel or interrupt context.
    pub fn net_isvalidchecksum(net: net_handle_t, mb: *mut mblk_t) -> c_int;

    /// `net_inject(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Delivers the network layer packet described by `packet` either into
    /// the kernel or onto the network, using the protocol `net` and the
    /// method `style`. `packet` usually comes from `net_inject_alloc(9F)`,
    /// and its `ni_packet` member must point to the message block chain of
    /// the packet in every case.
    ///
    /// With [`inject_t::NI_QUEUE_IN`] and [`inject_t::NI_QUEUE_OUT`] the
    /// packet is not processed at once. It is added to a list and delivered
    /// later on a different thread and in a different context, and it passes
    /// through the physical inbound or outbound hooks again. With
    /// [`inject_t::NI_DIRECT_OUT`] the packet is sent out of `ni_physical`
    /// immediately, using `ni_addr` as the next hop.
    ///
    /// The function returns 0 if the packet was queued or sent, 1 if it
    /// could not be, and -1 if the protocol does not support the call. It
    /// may be called from user, kernel or interrupt context.
    pub fn net_inject(net: net_handle_t, style: inject_t, packet: *mut net_inject_t) -> c_int;

    /// `net_inject_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates a zeroed [`net_inject_t`] and stores `version` in it. The
    /// page says that `version` must always be `NETI_VERSION`, a symbol that
    /// does not exist, and [`NETINFO_VERSION`] is what callers pass. The
    /// allocation does not sleep.
    ///
    /// The function returns a pointer to the new structure on success and
    /// NULL on failure. It may be called from user, kernel or interrupt
    /// context.
    pub fn net_inject_alloc(version: c_int) -> *mut net_inject_t;

    /// `net_inject_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees a [`net_inject_t`] that `net_inject_alloc(9F)` allocated. It does
    /// not free the packet the structure refers to.
    ///
    /// The function may be called from user, kernel or interrupt context.
    pub fn net_inject_free(inject: *mut net_inject_t);

    /// `net_hook_register(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers the callback described by `hook` on the event named `event`
    /// of the protocol `info`, so that it is called each time the event
    /// fires. `hook` must come from `hook_alloc(9F)` and have `h_func` set,
    /// `h_name` must be unique on the event and within the network stack,
    /// `h_flags` must be 0, and `h_hint` and `h_hintvalue` say where the hook
    /// is to be inserted. Using one `hook_t` for several registrations is
    /// possible but not encouraged.
    ///
    /// The function returns 0 on success. It returns `ENOMEM` if memory ran
    /// out, `ENXIO` if the event cannot be found, `EEXIST` if a hook of the
    /// same name is already on the event, `ESRCH` if a before or after hint
    /// names a hook that is not there, `EBUSY` if the hint conflicts with
    /// another hook, for example a second [`crate::hook::hook_hint_t::HH_FIRST`], and
    /// `ESHUTDOWN` if the network stack of `info` is being torn down. It may
    /// be called from user or kernel context.
    pub fn net_hook_register(info: net_handle_t, event: *mut c_char, hook: *mut hook_t) -> c_int;

    /// `net_hook_unregister(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the hook `hook`, which `net_hook_register(9F)` registered on
    /// the event named `event` of the protocol `net`, so that it is no
    /// longer called. The caller may free the hook with `hook_free(9F)`
    /// afterwards.
    ///
    /// The page shows only the protocol handle and a hook token. The header
    /// takes the event name and the `hook_t` pointer as well, which is what
    /// the kernel uses.
    ///
    /// The function returns 0 on success and an error number describing the
    /// problem otherwise, such as `ENXIO` if the protocol has no events. It
    /// may be called from user, kernel or interrupt context.
    pub fn net_hook_unregister(net: net_handle_t, event: *mut c_char, hook: *mut hook_t) -> c_int;

    /// `net_event_notify_register(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers `callback` to be called when a hook is added to or removed
    /// from the event named `event` of the protocol `family`, a handle from
    /// `net_protocol_lookup(9F)`. Several callbacks may be registered, and
    /// each is called with `arg`, the command, the name of the protocol, the
    /// name of the event and the name of the hook. The callback must treat
    /// the names as read-only and must not block other events. The names of
    /// hooks are a private interface unless documented otherwise, and those
    /// of IPFilter are public but uncommitted.
    ///
    /// The page's synopsis declares `callback` as a pointer to a
    /// `hook_notify_fn_t`. The header takes the function pointer itself.
    ///
    /// The function returns 0 on success and `EEXIST` if the callback is
    /// already registered. It may be called from user or kernel context.
    pub fn net_event_notify_register(
        family: net_handle_t,
        event: *mut c_char,
        callback: hook_notify_fn_t,
        arg: *mut c_void,
    ) -> c_int;

    /// `net_event_notify_unregister(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes `callback` from the functions that
    /// `net_event_notify_register(9F)` registered for the event named
    /// `event` of the protocol `family`.
    ///
    /// The function returns 0 on success and an error number, such as
    /// `ESRCH`, if the callback cannot be found. It may be called from user or kernel context.
    pub fn net_event_notify_unregister(family: net_handle_t, event: *mut c_char, callback: hook_notify_fn_t) -> c_int;

    /// `net_instance_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates a zeroed [`net_instance_t`] and stores `version` in it.
    /// `version` must always be [`NETINFO_VERSION`]. The allocation sleeps
    /// until memory is available.
    ///
    /// The function returns a pointer to the new structure on success and
    /// NULL on failure, which includes being passed any other version. It
    /// may be called from user or kernel context.
    pub fn net_instance_alloc(version: c_int) -> *mut net_instance_t;

    /// `net_instance_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees a [`net_instance_t`] that `net_instance_alloc(9F)` allocated.
    /// The structure must no longer be registered.
    ///
    /// The function may be called from user, kernel or interrupt context.
    pub fn net_instance_free(net_instance: *mut net_instance_t);

    /// `net_instance_register(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Records the callbacks in `instances`, which must come from
    /// `net_instance_alloc(9F)`, to be called when IP instances are created,
    /// shut down and destroyed. The `nin_create` callback is also called for
    /// every IP instance that already exists.
    ///
    /// The function returns `DDI_SUCCESS` on success and `DDI_FAILURE` if a
    /// set of callbacks with the same name is already registered. It may be
    /// called from user or kernel context.
    pub fn net_instance_register(instances: *mut net_instance_t) -> c_int;

    /// `net_instance_unregister(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the callbacks that `net_instance_register(9F)` recorded for
    /// `instances`. Before it returns, the kernel calls the shutdown and then
    /// the destroy callback for every IP instance the set was created for.
    ///
    /// The page's synopsis declares the function as returning `void`, while
    /// its text and the header give it an `int` result. The function
    /// returns `DDI_SUCCESS` on success and `DDI_FAILURE` if `instances` is
    /// not registered. It may be called from user, kernel or interrupt
    /// context.
    pub fn net_instance_unregister(instances: *mut net_instance_t) -> c_int;

    /// `net_instance_notify_register(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers `callback` to be called when a set of instance callbacks is
    /// added to or removed from the IP instance `netid`, which comes from an
    /// instance callback or from `net_zoneidtonetid(9F)`. Several callbacks
    /// may be registered, and each is called with `arg`, the command, the
    /// instance identifier written as a string, NULL, and the name of the
    /// instance being added or removed. The callback must treat the names as
    /// read-only and must not block other events.
    ///
    /// The page's synopsis declares `callback` as a pointer to a
    /// `hook_notify_fn_t`. The header takes the function pointer itself.
    ///
    /// The function returns 0 on success and `EEXIST` if the callback is
    /// already registered. It may be called from user or kernel context.
    pub fn net_instance_notify_register(netid: netid_t, callback: hook_notify_fn_t, arg: *mut c_void) -> c_int;

    /// `net_instance_notify_unregister(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes `callback` from the functions that
    /// `net_instance_notify_register(9F)` registered for the IP instance
    /// `netid`.
    ///
    /// The function returns 0 on success and an error number if the callback
    /// cannot be found. It may be called from user or kernel context.
    pub fn net_instance_notify_unregister(netid: netid_t, callback: hook_notify_fn_t) -> c_int;

    /// `net_kstat_create(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates and initialises a `kstat(9S)` for the IP instance `netid`,
    /// so that the statistic is visible in the zone that uses that instance.
    /// The other arguments are those of `kstat_create(9F)`. `module` is the
    /// name of the providing module, `instance` its instance number or 0,
    /// `name` the name of the kstat, `class` its general class such as
    /// `net`, `type_` one of the `KSTAT_TYPE_*` values, `ndata` the number
    /// of data records and `ks_flag` a combination of the `KSTAT_FLAG_*`
    /// values. The kstat must then be installed with `kstat_install(9F)`.
    ///
    /// The function returns a pointer to the new kstat, or NULL on failure,
    /// which includes an unknown instance. It may be called from user or
    /// kernel context.
    pub fn net_kstat_create(
        netid: netid_t,
        module: *mut c_char,
        instance: c_int,
        name: *mut c_char,
        class: *mut c_char,
        type_: c_uchar,
        ndata: ulong_t,
        ks_flag: c_uchar,
    ) -> *mut kstat_t;

    /// `net_kstat_delete(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the kstat `ksp`, which `net_kstat_create(9F)` created for the
    /// IP instance `netid`, from the system and frees its resources, as
    /// `kstat_delete(9F)` does for an ordinary kstat.
    ///
    /// The page's return value section describes a pointer result, but the
    /// function returns nothing. It may be called from user, kernel or
    /// interrupt context.
    pub fn net_kstat_delete(netid: netid_t, ksp: *mut kstat_t);
}
