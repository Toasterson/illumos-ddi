//! Rings and groups, the `MAC_CAPAB_RINGS` capability, from
//! `sys/mac_provider.h` and `sys/mac.h`.
//!
//! A ring is the framework's abstraction of a hardware queue of descriptors,
//! and a group is a set of rings that share hardware filters on MAC address
//! or VLAN. A provider that advertises `MAC_CAPAB_RINGS` is asked through
//! `mc_getcapab(9E)` to fill in a [`mac_capab_rings_t`] once for receive and
//! once for transmit. The framework then calls `mr_gget(9E)` for every group
//! and `mr_rget(9E)` for every ring, and the provider describes each group
//! in a [`mac_group_info_t`] and each ring in a [`mac_ring_info_t`] with its
//! interrupt in a [`mac_intr_t`]. From then on the framework drives the
//! rings through the entry points stored in those structures.
//!
//! A provider using this capability replaces `mac_rx(9F)` with
//! `mac_rx_ring(9F)` and `mac_tx_update(9F)` with `mac_tx_ring_update(9F)`,
//! leaves `mc_unicst` unset when it has receive rings, and does not use
//! `mc_tx` when it has transmit rings. Every part of the capability is
//! Uncommitted, which means that its interface and binary layout may change
//! between releases.
//!
//! Where the manual pages and the header disagree, the header is followed.
//! `mac_ring_info(9S)` gives the first argument of `mri_tx` and `mri_poll`
//! as `mac_ring_driver_t`, and `mac_group_info(9S)` gives `mgi_addmac` and
//! `mgi_remmac` a trailing `flags` argument and a `mac_group_driver_t` first
//! argument, while the header declares a `void *` first argument and no
//! flags. The header's `mri_flags` constants, `MAC_RING_TX_SERIALIZE` and
//! `MAC_RING_RX_ENQUEUE`, are temporary workarounds for two sun4v drivers
//! and are documented nowhere, so they are not bound; neither are the
//! `mr_gaddring` and `mr_gremring` entry points for dynamic groups, whose
//! members are declared only so that the layout matches, nor the
//! `mac_group_add_ring`, `mac_group_rem_ring` and `mac_find_ring` functions.

use ddi_sys::{boolean_t, c_int, c_void, ddi_intr_handle_t, mblk_t, uint_t};

use crate::types::{mac_group_driver_t, mac_group_handle_t, mac_intr_handle_t, mac_ring_driver_t, mac_ring_handle_t};

/// `mac_capab_rings(9E)`. Uncommitted - This interface is still evolving.
///
/// Whether a ring or group transmits or receives, `mac_ring_type_t`. The
/// framework sets it in the `mr_type` member of [`mac_capab_rings_t`] and
/// passes it to `mr_rget(9E)` and `mr_gget(9E)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct mac_ring_type_t(pub c_int);
impl mac_ring_type_t {
    /// Receive rings (`MAC_RING_TYPE_RX`).
    pub const MAC_RING_TYPE_RX: Self = Self(1);
    /// Transmit rings (`MAC_RING_TYPE_TX`).
    pub const MAC_RING_TYPE_TX: Self = Self(2);
}

const _: () = assert!(core::mem::size_of::<mac_ring_type_t>() == 4);

/// `mac_capab_rings(9E)`. Uncommitted - This interface is still evolving.
///
/// How rings are assigned to groups, `mac_group_type_t`, which the provider
/// stores in the `mr_group_type` member of [`mac_capab_rings_t`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct mac_group_type_t(pub c_int);
impl mac_group_type_t {
    /// Each ring belongs to one fixed group. This is the value the manual
    /// tells providers to use (`MAC_GROUP_TYPE_STATIC`).
    pub const MAC_GROUP_TYPE_STATIC: Self = Self(1);
    /// Rings can be moved between groups at run time. The manual does not
    /// describe this mode for providers (`MAC_GROUP_TYPE_DYNAMIC`).
    pub const MAC_GROUP_TYPE_DYNAMIC: Self = Self(2);
}

const _: () = assert!(core::mem::size_of::<mac_group_type_t>() == 4);

/// `mr_rget(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that describes one ring,
/// `mac_get_ring_t`, stored in the `mr_rget` member of
/// [`mac_capab_rings_t`]. `driver` is the provider's private data, `rtype`
/// says whether the ring transmits or receives, and `group_index` and
/// `ring_index`, both counting from zero, identify the ring within its
/// group. The provider fills in the [`mac_ring_info_t`] at `infop` and keeps
/// the ring handle `rh`, which it needs for `mac_rx_ring(9F)` and
/// `mac_tx_ring_update(9F)`. The entry point has no return value.
///
/// It is called while the provider is inside `mac_register(9F)`, after the
/// `mr_gget(9E)` call for the ring's group.
pub type mac_get_ring_t = unsafe extern "C" fn(
    driver: *mut c_void,
    rtype: mac_ring_type_t,
    group_index: c_int,
    ring_index: c_int,
    infop: *mut mac_ring_info_t,
    rh: mac_ring_handle_t,
);

/// `mr_gget(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that describes one group,
/// `mac_get_group_t`, stored in the `mr_gget` member of
/// [`mac_capab_rings_t`]. `driver` is the provider's private data, `rtype`
/// says whether the group holds transmit or receive rings, and
/// `group_index` identifies the group, counting from zero to `mr_gnum`
/// minus one. The provider fills in the [`mac_group_info_t`] at `infop` and
/// keeps the group handle `gh`. The entry point has no return value.
///
/// It is called while the provider is inside `mac_register(9F)`.
pub type mac_get_group_t = unsafe extern "C" fn(
    driver: *mut c_void,
    rtype: mac_ring_type_t,
    group_index: c_int,
    infop: *mut mac_group_info_t,
    gh: mac_group_handle_t,
);

/// `mac_capab_rings(9E)`. Uncommitted - This interface is still evolving.
///
/// The data of the `MAC_CAPAB_RINGS` capability, `mac_capab_rings_t`. The
/// framework asks for it separately for receive and for transmit, and sets
/// `mr_type` to say which. If `mr_type` holds neither value, the provider
/// must return `B_FALSE` from `mc_getcapab(9E)`.
#[repr(C)]
pub struct mac_capab_rings_t {
    /// Whether this query is about receive or transmit rings. Set by the
    /// framework and not to be changed by the provider.
    pub mr_type: mac_ring_type_t,
    /// How rings are assigned to groups. Set it to
    /// [`mac_group_type_t::MAC_GROUP_TYPE_STATIC`].
    pub mr_group_type: mac_group_type_t,
    /// The total number of rings of this type the provider exposes, which
    /// may be fewer than the hardware has, for example because fewer
    /// interrupts were available.
    pub mr_rnum: uint_t,
    /// The total number of groups of this type the provider exposes. For
    /// transmit it may be zero, in which case each ring stands on its own.
    pub mr_gnum: uint_t,
    /// Describes one ring, see `mr_rget(9E)`.
    pub mr_rget: Option<mac_get_ring_t>,
    /// Describes one group, see `mr_gget(9E)`.
    pub mr_gget: Option<mac_get_group_t>,
    /// Adds a ring to a dynamic group. Not described by the manual; a
    /// provider with static groups leaves it `None`. The C type is
    /// `mac_group_add_ring_t`.
    pub mr_gaddring: Option<unsafe extern "C" fn(mac_group_driver_t, mac_ring_driver_t, mac_ring_type_t)>,
    /// Removes a ring from a dynamic group. Not described by the manual; a
    /// provider with static groups leaves it `None`. The C type is
    /// `mac_group_rem_ring_t`.
    pub mr_gremring: Option<unsafe extern "C" fn(mac_group_driver_t, mac_ring_driver_t, mac_ring_type_t)>,
}

// mac_capab_rings_t under LP64: two enumerations at 0 and 4, two uint_t at
// 8 and 12, then four pointers at 16, 24, 32 and 40; total 48.
const _: () = assert!(core::mem::size_of::<mac_capab_rings_t>() == 48);
const _: () = assert!(core::mem::offset_of!(mac_capab_rings_t, mr_type) == 0);
const _: () = assert!(core::mem::offset_of!(mac_capab_rings_t, mr_gnum) == 12);
const _: () = assert!(core::mem::offset_of!(mac_capab_rings_t, mr_rget) == 16);
const _: () = assert!(core::mem::offset_of!(mac_capab_rings_t, mr_gget) == 24);
const _: () = assert!(core::mem::offset_of!(mac_capab_rings_t, mr_gremring) == 40);

/// `mi_enable(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that re-enables interrupt generation
/// for one ring, `mac_intr_enable_t`, stored in the `mi_enable` member of
/// [`mac_intr_t`]. `driver` is the handle the provider stored in
/// `mi_handle`. The framework calls it when it stops polling the ring
/// through `mri_poll(9E)`.
///
/// It must act on the ring alone through device registers or control
/// messages, and must not be implemented with `ddi_intr_enable(9F)`,
/// because the underlying MSI or MSI-X interrupt may be shared with other
/// rings. It must be serialized with the ring's interrupt processing and
/// with `mri_poll(9E)` by the same per-ring mutex, or a packet may be
/// delivered twice. The entry point returns 0 on success and an error
/// number otherwise.
pub type mac_intr_enable_t = unsafe extern "C" fn(driver: mac_intr_handle_t) -> c_int;

/// `mi_disable(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that disables interrupt generation for
/// one ring, `mac_intr_disable_t`, stored in the `mi_disable` member of
/// [`mac_intr_t`]. `driver` is the handle the provider stored in
/// `mi_handle`. The framework calls it before it begins polling the ring
/// through `mri_poll(9E)`.
///
/// It must act on the ring alone through device registers or control
/// messages, and must not be implemented with `ddi_intr_disable(9F)`,
/// because the underlying MSI or MSI-X interrupt may be shared with other
/// rings. It must be serialized with the ring's interrupt processing and
/// with `mri_poll(9E)` by the same per-ring mutex, or a packet may be
/// delivered twice. The entry point returns 0 on success and an error
/// number otherwise.
pub type mac_intr_disable_t = unsafe extern "C" fn(driver: mac_intr_handle_t) -> c_int;

/// `mac_intr(9S)`. Interface stability: Uncommitted.
///
/// The description of one ring's interrupt, `mac_intr_t`, embedded as the
/// `mri_intr` member of `mac_ring_info(9S)` and the `mgi_intr` member of
/// `mac_group_info(9S)`. The framework uses its entry points to switch a
/// receive ring between interrupt-driven operation and polling. Although it
/// carries the DDI interrupt handle, it always describes the interrupt of a
/// single ring. The page gives the stability as Uncommitted, adding that the
/// interface is still evolving and that API and ABI stability is not
/// guaranteed.
#[repr(C)]
pub struct mac_intr_t {
    /// A value of the provider's choosing, usually a pointer to its ring,
    /// that the framework passes to `mi_enable` and `mi_disable`.
    pub mi_handle: mac_intr_handle_t,
    /// Enables interrupts for the ring, see `mi_enable(9E)`. Required for
    /// receive rings, optional for transmit rings.
    pub mi_enable: Option<mac_intr_enable_t>,
    /// Disables interrupts for the ring, see `mi_disable(9E)`. Required for
    /// receive rings, optional for transmit rings.
    pub mi_disable: Option<mac_intr_disable_t>,
    /// The DDI interrupt handle from `ddi_intr_alloc(9F)` that serves the
    /// ring. Set it only for MSI and MSI-X interrupts.
    pub mi_ddi_handle: ddi_intr_handle_t,
    /// Whether the DDI interrupt is shared with other rings. Not described
    /// by the manual; the framework maintains it, so the provider leaves it
    /// at `B_FALSE`.
    pub mi_ddi_shared: boolean_t,
}

// mac_intr_t under LP64: four pointers at 0, 8, 16 and 24, then the
// boolean_t mi_ddi_shared at 32 and tail padding to the 8-byte alignment;
// total 40.
const _: () = assert!(core::mem::size_of::<mac_intr_t>() == 40);
const _: () = assert!(core::mem::offset_of!(mac_intr_t, mi_handle) == 0);
const _: () = assert!(core::mem::offset_of!(mac_intr_t, mi_ddi_handle) == 24);
const _: () = assert!(core::mem::offset_of!(mac_intr_t, mi_ddi_shared) == 32);

/// `mri_start(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that starts one ring,
/// `mac_ring_start_t`, stored in the `mri_start` member of
/// [`mac_ring_info_t`]. `rh` is the handle the provider stored in
/// `mri_driver`, and `mr_gen` is the ring's generation number, which the
/// provider must save because `mac_rx_ring(9F)` drops every packet passed
/// with a different generation. The provider may allocate the ring's
/// buffers here rather than in `attach(9E)`. The framework starts a group
/// before any of its rings.
///
/// The entry point is required. It returns 0 on success and an error number
/// otherwise.
pub type mac_ring_start_t = unsafe extern "C" fn(rh: mac_ring_driver_t, mr_gen: u64) -> c_int;

/// `mri_stop(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that stops one ring, `mac_ring_stop_t`,
/// stored in the `mri_stop` member of [`mac_ring_info_t`]. `rh` is the
/// handle the provider stored in `mri_driver`. The framework stops every
/// ring of a group before the group itself. Because the entry point cannot
/// fail, a provider whose stop operation fails must mark the device as
/// faulted and recover from another context, for example with a reset.
pub type mac_ring_stop_t = unsafe extern "C" fn(rh: mac_ring_driver_t);

/// `mri_tx(9E)`. illumos DDI specific.
///
/// The signature of the transmit entry point of one ring,
/// `mac_ring_send_t`, stored in the `mri_tx` member of the union in
/// [`mac_ring_info_t`]. It follows the rules of `mc_tx(9E)`, except that
/// `driver` is the handle the provider stored in `mri_driver`, the provider
/// must transmit on that ring and no other, and it reports that resources
/// are available again with `mac_tx_ring_update(9F)` rather than
/// `mac_tx_update(9F)`. It returns NULL when it consumed the whole chain and
/// the unprocessed frames otherwise. The header declares the first argument
/// as `void *`, the manual page as `mac_ring_driver_t`.
pub type mac_ring_send_t = unsafe extern "C" fn(driver: *mut c_void, mp_chain: *mut mblk_t) -> *mut mblk_t;

/// `mri_poll(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that polls one receive ring,
/// `mac_ring_poll_t`, stored in the `mri_poll` member of the union in
/// [`mac_ring_info_t`]. `driver` is the handle the provider stored in
/// `mri_driver`. The provider processes the ring as its interrupt handler
/// would, but returns the frames as a chain linked through `b_next` instead
/// of passing them to `mac_rx_ring(9F)`. It stops once the total size of the
/// collected frames reaches `poll_bytes`, and returns early without waiting
/// when fewer frames are ready. The framework disables the ring's interrupt
/// with `mi_disable(9E)` before polling, and the same per-ring mutex must
/// serialize polling, interrupt processing and the enable and disable entry
/// points.
///
/// The entry point returns the chain of collected frames, or NULL when none
/// are available or an error occurred. The header declares the first
/// argument as `void *`, the manual page for the structure as
/// `mac_ring_driver_t`.
pub type mac_ring_poll_t = unsafe extern "C" fn(driver: *mut c_void, poll_bytes: c_int) -> *mut mblk_t;

/// `mri_stat(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that reads a statistic of one ring,
/// `mac_ring_stat_t`, stored in the `mri_stat` member of
/// [`mac_ring_info_t`]. `rh` is the handle the provider stored in
/// `mri_driver`, `stat` the statistic and `val` where the value goes. The
/// statistics have the meaning `mac(9E)` gives them but count only this
/// ring. Receive rings should support `MAC_STAT_IPACKETS` and
/// `MAC_STAT_RBYTES`, transmit rings `MAC_STAT_OPACKETS` and
/// `MAC_STAT_OBYTES`.
///
/// The entry point returns 0 after storing the value, `ENOTSUP` for an
/// unknown statistic, `EIO` for a transport or DMA fault, and `ECANCELLED`
/// when the device cannot currently service the request.
pub type mac_ring_stat_t = unsafe extern "C" fn(rh: mac_ring_driver_t, stat: uint_t, val: *mut u64) -> c_int;

/// `mac_ring_info(9S)`. Interface stability: Uncommitted.
///
/// The anonymous union `mrfunion` inside `mac_ring_info(9S)`, holding the
/// entry point that differs between transmit and receive rings. The header
/// reaches its two members through the macros `mri_tx` and `mri_poll`; in
/// Rust they are reached as `mrfunion.send` and `mrfunion.poll`. The type
/// has no C name of its own.
#[repr(C)]
#[derive(Clone, Copy)]
pub union mac_ring_info_mrfunion {
    /// The `mri_tx(9E)` entry point, set only on transmit rings, where it is
    /// required.
    pub send: Option<mac_ring_send_t>,
    /// The `mri_poll(9E)` entry point, set only on receive rings, where it
    /// is required.
    pub poll: Option<mac_ring_poll_t>,
}

const _: () = assert!(core::mem::size_of::<mac_ring_info_mrfunion>() == 8);

/// `mac_ring_info(9S)`. Interface stability: Uncommitted.
///
/// The description of one ring, `mac_ring_info_t`, which the provider fills
/// in during `mr_rget(9E)`. All members that are not functions are
/// required, as are `mri_start`, and `mri_tx` for transmit rings or
/// `mri_poll` for receive rings. The page gives the stability as
/// Uncommitted, adding that the interface is still evolving and that API and
/// ABI stability is not guaranteed.
#[repr(C)]
pub struct mac_ring_info_t {
    /// A value of the provider's choosing that identifies its ring. The
    /// framework passes it to every ring entry point.
    pub mri_driver: mac_ring_driver_t,
    /// Starts the ring and records its generation number, see
    /// `mri_start(9E)`. Required.
    pub mri_start: Option<mac_ring_start_t>,
    /// Stops the ring, see `mri_stop(9E)`. Optional.
    pub mri_stop: Option<mac_ring_stop_t>,
    /// The interrupt of the ring, filled in as `mac_intr(9S)` describes.
    pub mri_intr: mac_intr_t,
    /// The transmit or poll entry point, depending on the ring type.
    pub mrfunion: mac_ring_info_mrfunion,
    /// Reads a statistic of the ring, see `mri_stat(9E)`.
    pub mri_stat: Option<mac_ring_stat_t>,
    /// Flags for special properties of the ring. The header describes them
    /// as temporary workarounds used only by sun4v drivers and the manual
    /// does not mention them, so providers leave it at zero.
    pub mri_flags: uint_t,
}

// mac_ring_info_t under LP64: three pointers at 0, 8 and 16, the 40-byte
// mac_intr_t from 24 to 64, the pointer-sized union at 64, mri_stat at 72,
// mri_flags at 80 and tail padding to the 8-byte alignment; total 88.
const _: () = assert!(core::mem::size_of::<mac_ring_info_t>() == 88);
const _: () = assert!(core::mem::offset_of!(mac_ring_info_t, mri_driver) == 0);
const _: () = assert!(core::mem::offset_of!(mac_ring_info_t, mri_intr) == 24);
const _: () = assert!(core::mem::offset_of!(mac_ring_info_t, mrfunion) == 64);
const _: () = assert!(core::mem::offset_of!(mac_ring_info_t, mri_stat) == 72);
const _: () = assert!(core::mem::offset_of!(mac_ring_info_t, mri_flags) == 80);

/// `mgi_start(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the optional entry point that starts one group,
/// `mac_group_start_t`, stored in the `mgi_start` member of
/// [`mac_group_info_t`]. `gh` is the handle the provider stored in
/// `mgi_driver`. The framework calls it before starting any ring of the
/// group. The entry point returns 0 on success and an error number
/// otherwise.
pub type mac_group_start_t = unsafe extern "C" fn(gh: mac_group_driver_t) -> c_int;

/// `mgi_stop(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the optional entry point that stops one group,
/// `mac_group_stop_t`, stored in the `mgi_stop` member of
/// [`mac_group_info_t`]. `gh` is the handle the provider stored in
/// `mgi_driver`. The framework calls it after stopping every ring of the
/// group. Because it cannot fail, a provider whose stop operation fails
/// must mark the device as faulted and recover from another context.
pub type mac_group_stop_t = unsafe extern "C" fn(gh: mac_group_driver_t);

/// `mgi_addmac(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that adds a unicast MAC address filter
/// to a receive group, `mac_add_mac_addr_t`, stored in the `mgi_addmac`
/// member of [`mac_group_info_t`]. `driver` is the handle the provider stored
/// in `mgi_driver`, and `mac` points to the address, at least as long as
/// the plugin's address length. The filter applies to the group as a whole,
/// and the hardware should spread matching traffic over all rings of the
/// group. Filters of the same class combine as a logical OR, filters of
/// different classes as a logical AND. The framework does not replay
/// filters after a device reset, so the provider should remember them.
///
/// The entry point returns 0 once the filter is programmed and an error
/// number otherwise, such as `EEXIST` for a filter already present, `EINVAL`
/// for an address that is not unicast, `EIO` for a device error, `ENOSPC`
/// when no hardware filter is left, or `ENOTSUP` on hardware where filtering
/// cannot work at all.
pub type mac_add_mac_addr_t = unsafe extern "C" fn(driver: *mut c_void, mac: *const u8) -> c_int;

/// `mgi_remmac(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that removes a unicast MAC address
/// filter from a receive group, `mac_rem_mac_addr_t`, stored in the
/// `mgi_remmac` member of [`mac_group_info_t`]. `driver` is the handle the
/// provider stored in `mgi_driver`, and `mac` points to the address. The
/// entry point returns 0 once the filter is removed and an error number
/// otherwise, such as `ENOENT` for a filter that is not programmed or `EIO`
/// for a device error.
pub type mac_rem_mac_addr_t = unsafe extern "C" fn(driver: *mut c_void, mac: *const u8) -> c_int;

/// `mgi_addvlan(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that adds a VLAN filter to a receive
/// group, `mac_add_vlan_filter_t`, stored in the `mgi_addvlan` member of
/// [`mac_group_info_t`]. `gh` is the handle the provider stored in
/// `mgi_driver`, and `vlan` the VLAN identifier to accept. The framework
/// passes `MAC_VLAN_UNTAGGED`, which is 0xffff, when a client wants untagged
/// traffic. The entry point returns 0 once the filter is programmed and an
/// error number otherwise, as described for `mgi_addmac(9E)`.
pub type mac_add_vlan_filter_t = unsafe extern "C" fn(gh: mac_group_driver_t, vlan: u16) -> c_int;

/// `mgi_remvlan(9E)`. Uncommitted - This interface is still evolving.
///
/// The signature of the entry point that removes a VLAN filter from a
/// receive group, `mac_rem_vlan_filter_t`, stored in the `mgi_remvlan`
/// member of [`mac_group_info_t`]. `gh` is the handle the provider stored in
/// `mgi_driver`, and `vlan` the VLAN identifier. The entry point returns 0
/// once the filter is removed and an error number otherwise, such as
/// `ENOENT` for a filter that is not programmed.
pub type mac_rem_vlan_filter_t = unsafe extern "C" fn(gh: mac_group_driver_t, vlan: u16) -> c_int;

/// `mac_group_info(9S)`. Uncommitted - This interface is still evolving.
///
/// The description of one group, `mac_group_info_t`, which the provider
/// fills in during `mr_gget(9E)`. The members that are not functions are
/// required, `mgi_start` and `mgi_stop` are optional, and the filter entry
/// points must all be NULL for transmit groups. A receive group must have
/// some way to set a MAC address filter, which currently means providing
/// both `mgi_addmac` and `mgi_remmac`. Only the first receive group, the
/// default group, receives broadcast, multicast and promiscuous traffic.
#[repr(C)]
pub struct mac_group_info_t {
    /// A value of the provider's choosing that identifies its group. The
    /// framework passes it to every group entry point.
    pub mgi_driver: mac_group_driver_t,
    /// Starts the group, see `mgi_start(9E)`. Optional.
    pub mgi_start: Option<mac_group_start_t>,
    /// Stops the group, see `mgi_stop(9E)`. Optional.
    pub mgi_stop: Option<mac_group_stop_t>,
    /// The number of rings in the group. For a static group it is fixed.
    pub mgi_count: uint_t,
    /// An optional interrupt for the whole group. Not described by the
    /// manual page for the structure.
    pub mgi_intr: mac_intr_t,
    /// Adds a MAC address filter, see `mgi_addmac(9E)`. Receive groups only.
    pub mgi_addmac: Option<mac_add_mac_addr_t>,
    /// Removes a MAC address filter, see `mgi_remmac(9E)`. Required whenever
    /// `mgi_addmac` is set. Receive groups only.
    pub mgi_remmac: Option<mac_rem_mac_addr_t>,
    /// Adds a VLAN filter, see `mgi_addvlan(9E)`. Receive groups only.
    pub mgi_addvlan: Option<mac_add_vlan_filter_t>,
    /// Removes a VLAN filter, see `mgi_remvlan(9E)`. Required whenever
    /// `mgi_addvlan` is set. Receive groups only.
    pub mgi_remvlan: Option<mac_rem_vlan_filter_t>,
}

// mac_group_info_t under LP64: three pointers at 0, 8 and 16, mgi_count at
// 24 padded to 32, the 40-byte mac_intr_t from 32 to 72, then four pointers
// at 72, 80, 88 and 96; total 104.
const _: () = assert!(core::mem::size_of::<mac_group_info_t>() == 104);
const _: () = assert!(core::mem::offset_of!(mac_group_info_t, mgi_driver) == 0);
const _: () = assert!(core::mem::offset_of!(mac_group_info_t, mgi_count) == 24);
const _: () = assert!(core::mem::offset_of!(mac_group_info_t, mgi_intr) == 32);
const _: () = assert!(core::mem::offset_of!(mac_group_info_t, mgi_addmac) == 72);
const _: () = assert!(core::mem::offset_of!(mac_group_info_t, mgi_remvlan) == 96);

unsafe extern "C" {
    /// Listed in the `mac` mapfile. There is no manual page of its own.
    ///
    /// Gives the framework a new DDI interrupt handle `ddh` for the ring
    /// identified by `mrh`, replacing the `mi_ddi_handle` the provider
    /// originally stored in the ring's `mac_intr(9S)`. A provider calls it
    /// when the interrupt resource management framework has given it more
    /// MSI-X interrupts or taken some away, so that the framework can
    /// retarget its processing. Passing NULL for `ddh` resets the ring's
    /// interrupt. The function has no return value.
    ///
    /// The function enters the framework's perimeter for the device and may
    /// block. As `mac(9E)` requires for every MAC function other than the
    /// few it names, the provider must not hold its own locks while calling
    /// it.
    pub fn mac_ring_intr_set(mrh: mac_ring_handle_t, ddh: ddi_intr_handle_t);
}
