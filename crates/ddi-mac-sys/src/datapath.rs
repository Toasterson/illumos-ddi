//! The data path and link notifications, from `sys/mac_provider.h` and
//! `sys/pattr.h`.
//!
//! These are the functions a registered provider calls while it runs. It
//! delivers received frames with [`mac_rx`], or with [`mac_rx_ring`] once it
//! uses rings, lifts transmit back pressure with [`mac_tx_update`] or
//! [`mac_tx_ring_update`], and reports changes of the link state and of the
//! maximum SDU with [`mac_link_update`] and [`mac_maxsdu_update`]. With the
//! exception of `mac_maxsdu_update`, none of these may be called while the
//! provider holds locks of its own, because the framework may call back into
//! the provider.
//!
//! The offload metadata functions are declared here as well. A provider that
//! advertises `MAC_CAPAB_HCKSUM` reads the checksum requests of each
//! outgoing frame with [`mac_hcksum_get`] and records what the hardware
//! verified on each incoming frame with [`mac_hcksum_set`], and one that
//! advertises `MAC_CAPAB_LSO` reads the segmentation request of each
//! outgoing frame with [`mac_lso_get`]. All three act on the first message
//! block of a single frame, that is on a block reached through `b_next` or
//! handed over by the framework, never on one reached through `b_cont`. The
//! `HCK_*` and `HW_LSO` flags they use are defined in `sys/pattr.h`.
//! Finally, [`mac_ether_offload_info`] parses the headers of an Ethernet
//! frame for providers whose hardware needs the header lengths to perform
//! an offload.
//!
//! The header declares further functions that neither have a manual page
//! nor appear in the `mac` mapfile, among them `mac_hcksum_clone`,
//! `mac_partial_offload_info`, `mac_ether_l2_info`, `mac_link_redo`,
//! `mac_unicst_update`, `mac_capab_update`, `mac_maxsdu_update2`,
//! `mac_sdu_get`, `mac_margin_update` and the `mac_margin_*`,
//! `mac_*_refresh`, `mactype_*` and bridging functions. They are not bound.

use ddi_sys::{c_uint, mblk_t, size_t, uint_t};

use crate::props::link_state_t;
use crate::types::{mac_handle_t, mac_resource_handle_t, mac_ring_handle_t};

/// On transmit, the hardware must compute the IPv4 header checksum, and no
/// other value is filled in. The flag is defined in `sys/pattr.h` and shares
/// its value with [`HCK_IPV4_HDRCKSUM_OK`] (`HCK_IPV4_HDRCKSUM`).
pub const HCK_IPV4_HDRCKSUM: u32 = 0x01;
/// On receive, the hardware has verified that the IPv4 header checksum is
/// correct, so the stack need not check it. The flag is defined in
/// `sys/pattr.h` and shares its value with [`HCK_IPV4_HDRCKSUM`]
/// (`HCK_IPV4_HDRCKSUM_OK`).
pub const HCK_IPV4_HDRCKSUM_OK: u32 = 0x01;
/// On transmit, the hardware must compute the partial one's complement
/// checksum from the `start`, `stuff` and `end` offsets the system supplies.
/// On receive, the hardware has computed a partial checksum, which the
/// provider passes as `value` together with the `start` and `end` offsets it
/// covers. The flag is defined in `sys/pattr.h` (`HCK_PARTIALCKSUM`).
pub const HCK_PARTIALCKSUM: u32 = 0x02;
/// On transmit, the hardware must compute the full layer 4 checksum, and no
/// other value is filled in. On receive, the hardware has computed the full
/// layer 4 checksum, which the provider passes as `value` for the stack to
/// verify. The flag is defined in `sys/pattr.h` (`HCK_FULLCKSUM`).
pub const HCK_FULLCKSUM: u32 = 0x04;
/// On receive, the hardware has computed the full layer 4 checksum and
/// verified that it is correct, so the stack need not check it. It is not
/// used on transmit. The flag is defined in `sys/pattr.h`
/// (`HCK_FULLCKSUM_OK`).
pub const HCK_FULLCKSUM_OK: u32 = 0x08;
/// On transmit, the hardware must perform large send offload, segmenting
/// the frame with the maximum segment size that `mac_lso_get(9F)` reports.
/// The flag is defined in `sys/pattr.h` (`HW_LSO`).
pub const HW_LSO: u32 = 0x10;

unsafe extern "C" {
    /// `mac_rx(9F)`. illumos DDI specific.
    ///
    /// Delivers the frames a provider has received to the rest of the
    /// system. `mh` is the handle from `mac_register(9F)`, `mrh` is reserved
    /// and must be NULL, and `mp_chain` is a chain of one or more frames
    /// linked through `b_next`, whose ownership passes to the framework. A
    /// provider typically calls it once at the end of its interrupt handler,
    /// batching all the frames it is willing to process, rather than once
    /// per frame. A provider that has negotiated `MAC_CAPAB_RINGS` with
    /// receive groups must use [`mac_rx_ring`] instead. The function has no
    /// return value.
    ///
    /// It can be called from user, kernel or interrupt context. The provider
    /// must not hold any of its own locks while calling it.
    pub fn mac_rx(mh: mac_handle_t, mrh: mac_resource_handle_t, mp_chain: *mut mblk_t);

    /// `mac_rx_ring(9F)`. illumos DDI specific.
    ///
    /// Delivers the frames received on one ring, for a provider that has
    /// negotiated `MAC_CAPAB_RINGS` with receive groups. `mh` is the handle
    /// from `mac_register(9F)`, `mring` the ring handle received in
    /// `mr_rget(9E)`, `mp_chain` the frames linked through `b_next`, and
    /// `mr_gen` the generation number received in `mri_start(9E)` for that
    /// ring. If `mr_gen` does not match what the framework expects, every
    /// frame is dropped. A provider that processed several rings in one
    /// interrupt calls it once per ring. While the framework is polling a
    /// ring through `mri_poll(9E)`, the provider returns the frames from
    /// that entry point instead of calling this function. The function has
    /// no return value, and its interface is Uncommitted.
    ///
    /// The provider must not hold any of its own locks while calling it.
    pub fn mac_rx_ring(mh: mac_handle_t, mring: mac_ring_handle_t, mp_chain: *mut mblk_t, mr_gen: u64);

    /// `mac_tx_update(9F)`. illumos DDI specific.
    ///
    /// Tells the framework that the device identified by `mh` can transmit
    /// again. A provider calls it only after its `mc_tx(9E)` entry point has
    /// returned frames for lack of resources, typically when a transmit
    /// completion interrupt has freed descriptors, and the framework makes
    /// no further transmit calls until it does. A provider that has
    /// negotiated `MAC_CAPAB_RINGS` with transmit groups must use
    /// [`mac_tx_ring_update`] instead. The function has no return value.
    ///
    /// It may be called from user, kernel or interrupt context. The provider
    /// must not hold any of its own locks while calling it.
    pub fn mac_tx_update(mh: mac_handle_t);

    /// `mac_tx_ring_update(9F)`. illumos DDI specific.
    ///
    /// Tells the framework that the transmit ring `mrh`, the handle received
    /// in `mr_rget(9E)`, of the device `mh` can transmit again after its
    /// `mri_tx(9E)` entry point returned frames. In every other respect it
    /// behaves like [`mac_tx_update`], which a provider with transmit rings
    /// must not use. The function has no return value, and its interface is
    /// Uncommitted.
    ///
    /// The provider must not hold any of its own locks while calling it.
    pub fn mac_tx_ring_update(mh: mac_handle_t, mrh: mac_ring_handle_t);

    /// `mac_link_update(9F)`. illumos DDI specific.
    ///
    /// Tells the framework that the state of the link of the device `mh` has
    /// changed to `link`. A provider calls it whenever it notices a change,
    /// usually from a link state interrupt, and not when the state is
    /// unchanged. When it powers the link off or can no longer tell its
    /// state, it reports [`link_state_t::LINK_STATE_UNKNOWN`]. Link
    /// aggregations and other features depend on these notifications. The
    /// function has no return value.
    ///
    /// It may be called from user, kernel or interrupt context. The provider
    /// must not hold any of its own locks while calling it.
    pub fn mac_link_update(mh: mac_handle_t, link: link_state_t);

    /// `mac_maxsdu_update(9F)`. illumos DDI specific.
    ///
    /// Tells the framework that the largest payload the device `mh` can
    /// transmit, its SDU, is now `sdu`. The value excludes the device's own
    /// headers and any margin such as a VLAN tag. A provider calls it from
    /// `mc_setprop(9E)` when `MAC_PROP_MTU` is set, before changing the
    /// hardware, and applies the new MTU only if the call succeeds. The
    /// framework has already checked that the new MTU suits the virtual links
    /// on top of the device.
    ///
    /// Returns 0 on success and an error number otherwise, `EINVAL` meaning
    /// that `sdu` is below the device's minimum SDU. It is one of the few MAC
    /// functions a provider may call while holding its own locks.
    pub fn mac_maxsdu_update(mh: mac_handle_t, sdu: uint_t) -> ddi_sys::c_int;

    /// `mac_hcksum_get(9F)`. illumos DDI specific.
    ///
    /// Reads the checksum offload requests attached to the outgoing frame
    /// whose first message block is `mp`, for a provider that advertises
    /// `MAC_CAPAB_HCKSUM`. It stores the `HCK_*` flags through `flags`, which
    /// is what the provider must check first. For
    /// [`HCK_PARTIALCKSUM`] it also stores the offsets from the start of the
    /// layer 3 header of the first byte covered by the checksum through
    /// `start`, of the checksum field itself through `stuff`, and of the last
    /// byte covered through `end`. For [`HCK_PARTIALCKSUM`] and
    /// [`HCK_FULLCKSUM`] it stores the pseudo-header checksum through
    /// `value`. Each pointer may be NULL if the caller does not need that
    /// value. Which flags can appear depends on the `HCKSUM_*` flags the
    /// provider advertised, and the partial and full flags are mutually
    /// exclusive. The function has no return value.
    ///
    /// It may be called from any context, and while the provider holds its
    /// own locks.
    pub fn mac_hcksum_get(
        mp: *const mblk_t,
        start: *mut u32,
        stuff: *mut u32,
        end: *mut u32,
        value: *mut u32,
        flags: *mut u32,
    );

    /// `mac_hcksum_set(9F)`. illumos DDI specific.
    ///
    /// Attaches to the received frame whose first message block is `mp` the
    /// checksum work the hardware has done, for a provider that advertises
    /// `MAC_CAPAB_HCKSUM` and whose hardware checks received checksums.
    /// `flags` is the bitwise inclusive OR of [`HCK_IPV4_HDRCKSUM_OK`] and at
    /// most one of [`HCK_PARTIALCKSUM`], [`HCK_FULLCKSUM`] and
    /// [`HCK_FULLCKSUM_OK`]. For a partial checksum, `value` is the checksum
    /// and `start` and `end` the offsets it covers from the layer 3 header;
    /// for a full checksum to be verified by the stack, `value` is the
    /// checksum. Arguments the flags do not call for are passed as 0. A
    /// provider with no checksum information for a frame simply does not
    /// call it. The function has no return value.
    ///
    /// It may be called from any context, and while the provider holds its
    /// own locks.
    pub fn mac_hcksum_set(mp: *mut mblk_t, start: u32, stuff: u32, end: u32, value: u32, flags: u32);

    /// `mac_lso_get(9F)`. illumos DDI specific.
    ///
    /// Reads the large send offload request attached to the outgoing frame
    /// whose first message block is `mp`, for a provider that advertises
    /// `MAC_CAPAB_LSO`. It stores the flags through `flags`, where
    /// [`HW_LSO`] means that the hardware must segment the frame, and in that
    /// case stores the maximum segment size through `mss`. The function has
    /// no return value.
    ///
    /// It may be called from user, kernel or interrupt context, and while
    /// the provider holds its own locks.
    pub fn mac_lso_get(mp: *mut mblk_t, mss: *mut u32, flags: *mut u32);

    /// Listed in the `mac` mapfile. There is no manual page of its own.
    ///
    /// Parses the Ethernet, IP and TCP or UDP headers of the frame whose
    /// first message block is `mp` and describes them in the structure at
    /// `meoi`, which it clears first. The total length of the frame is
    /// always filled in, and the flags in `meoi_flags` say how far parsing
    /// got. Each of [`MEOI_L2INFO_SET`](mac_ether_offload_flags_t::MEOI_L2INFO_SET),
    /// [`MEOI_L3INFO_SET`](mac_ether_offload_flags_t::MEOI_L3INFO_SET) and
    /// [`MEOI_L4INFO_SET`](mac_ether_offload_flags_t::MEOI_L4INFO_SET)
    /// promises that the matching fields are valid and that the message is
    /// long enough to contain the parsed headers; the fields of a layer that
    /// was not parsed stay zero. Providers use it when the hardware needs the
    /// header lengths to perform checksum or segmentation offload. The
    /// header describes it as an evolving interface private to illumos. The
    /// function has no return value.
    pub fn mac_ether_offload_info(mp: *mut mblk_t, meoi: *mut mac_ether_offload_info_t);
}

/// Used by `mac_ether_offload_info`, which is listed in the `mac` mapfile.
/// There is no manual page for either.
///
/// The flags of a [`mac_ether_offload_info_t`],
/// `mac_ether_offload_flags_t`, saying which of its fields are valid and
/// what parsing found. Each value is a single bit, and the field holds their
/// bitwise inclusive OR. The header describes the enumeration as part of a
/// provisional, evolving interface private to illumos.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct mac_ether_offload_flags_t(pub c_uint);
impl mac_ether_offload_flags_t {
    /// The Ethernet header was parsed, so `meoi_l2hlen` and `meoi_l3proto`
    /// are valid (`MEOI_L2INFO_SET`).
    pub const MEOI_L2INFO_SET: Self = Self(1 << 0);
    /// The layer 3 header was parsed, so `meoi_l3hlen` and `meoi_l4proto` are
    /// valid (`MEOI_L3INFO_SET`).
    pub const MEOI_L3INFO_SET: Self = Self(1 << 1);
    /// The layer 4 header was parsed, so `meoi_l4hlen` is valid
    /// (`MEOI_L4INFO_SET`).
    pub const MEOI_L4INFO_SET: Self = Self(1 << 2);
    /// The Ethernet header carries a VLAN tag (`MEOI_VLAN_TAGGED`).
    pub const MEOI_VLAN_TAGGED: Self = Self(1 << 3);
    /// The layer 3 header says that more fragments of this packet follow
    /// (`MEOI_L3_FRAG_MORE`).
    pub const MEOI_L3_FRAG_MORE: Self = Self(1 << 4);
    /// The packet is a fragment that lies after the start of the original
    /// data, so it holds no layer 4 header (`MEOI_L3_FRAG_OFFSET`).
    pub const MEOI_L3_FRAG_OFFSET: Self = Self(1 << 5);
}

const _: () = assert!(core::mem::size_of::<mac_ether_offload_flags_t>() == 4);

/// Filled in by `mac_ether_offload_info`, which is listed in the `mac`
/// mapfile. There is no manual page for either.
///
/// The description of the headers of an Ethernet frame,
/// `mac_ether_offload_info_t`. The header describes it as part of a
/// provisional, evolving interface private to illumos, and notes that the
/// userland `mac_test` program keeps a copy of it.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct mac_ether_offload_info_t {
    /// Which of the following fields are valid, and what parsing found.
    pub meoi_flags: mac_ether_offload_flags_t,
    /// The total length of the message in bytes. It is always valid.
    pub meoi_len: size_t,
    /// The length of the Ethernet header, including any VLAN tag.
    pub meoi_l2hlen: u8,
    /// The Ethertype of the layer 3 protocol.
    pub meoi_l3proto: u16,
    /// The length of the layer 3 header, including IPv6 extension headers.
    pub meoi_l3hlen: u16,
    /// The protocol number of the layer 4 protocol.
    pub meoi_l4proto: u8,
    /// The length of the layer 4 header.
    pub meoi_l4hlen: u8,
}

// mac_ether_offload_info_t under LP64: the enumeration meoi_flags at 0
// padded to 8, the size_t meoi_len at 8, meoi_l2hlen at 16, meoi_l3proto at
// 18, meoi_l3hlen at 20, meoi_l4proto at 22 and meoi_l4hlen at 23, with the
// size rounded up to the 8-byte alignment; total 24.
const _: () = assert!(core::mem::size_of::<mac_ether_offload_info_t>() == 24);
const _: () = assert!(core::mem::offset_of!(mac_ether_offload_info_t, meoi_flags) == 0);
const _: () = assert!(core::mem::offset_of!(mac_ether_offload_info_t, meoi_len) == 8);
const _: () = assert!(core::mem::offset_of!(mac_ether_offload_info_t, meoi_l2hlen) == 16);
const _: () = assert!(core::mem::offset_of!(mac_ether_offload_info_t, meoi_l3proto) == 18);
const _: () = assert!(core::mem::offset_of!(mac_ether_offload_info_t, meoi_l4hlen) == 23);
