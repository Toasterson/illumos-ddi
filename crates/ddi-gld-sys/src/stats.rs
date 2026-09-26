//! The `gld_stats(9S)` structure from `sys/gld.h`.
//!
//! GLD passes a `gld_stats` structure to the driver's `gldm_get_stats`
//! entry point, and the driver fills in the counters it keeps. GLD then
//! reports them, under the kstat names that `gld(4D)` lists, together with
//! the counters it maintains itself. Most members count how often an event
//! happened. The exceptions are `glds_speed`, `glds_media` and
//! `glds_duplex`, which describe the current state of the interface.
//!
//! The media specific counters live in the union [`media_stats`] inside the
//! structure. The manual page names them with the C macros
//! `glds_dot3_*`, `glds_dot5_*` and `glds_fddi_*`, which expand to members
//! of that union or, in three cases, to general members of the structure.
//! Rust has no such aliases, so each member's documentation says which
//! manual name refers to it. The manual adds that drivers can make no
//! assumptions about the length of the structure, and that the members it
//! does not document are private to GLD.

use ddi_sys::ulong_t;

/// `gld_stats(9S)`. illumos architecture specific (illumos DDI).
///
/// The Ethernet counters of the media specific statistics, the C `struct
/// dot3stat`, a subset of the Dot3Stats of RFC 1643. A driver for a
/// `DL_ETHER` device fills them in. They are reached through the `dot3` arm
/// of [`media_stats`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct dot3stat {
    /// Frames sent after exactly one collision, reported as
    /// `first_collisions`. The manual calls it `glds_dot3_first_coll`.
    pub first_coll: u32,
    /// Frames sent after more than one collision, reported as
    /// `multi_collisions`. The manual calls it `glds_dot3_multi_coll`.
    pub multi_coll: u32,
    /// SQE test errors, reported as `sqe_errors`. The manual calls it
    /// `glds_dot3_sqe_error`.
    pub sqe_error: u32,
    /// Transmissions that failed because of an internal MAC error, reported
    /// as `macxmt_errors`. The manual calls it `glds_dot3_mac_xmt_error`.
    pub mac_xmt_error: u32,
    /// Received frames that exceeded the maximum frame size, reported as
    /// `toolong_errors`. The manual calls it `glds_dot3_frame_too_long`.
    pub frame_too_long: u32,
    /// Receptions that failed because of an internal MAC error, reported as
    /// `macrcv_errors`. The manual calls it `glds_dot3_mac_rcv_error`.
    pub mac_rcv_error: u32,
}

/// `gld_stats(9S)`. illumos architecture specific (illumos DDI).
///
/// The Token Ring counters of the media specific statistics, the C `struct
/// dot5stat`, a subset of the Dot5Stats of RFC 1748. A driver for a
/// `DL_TPR` device fills them in. They are reached through the `dot5` arm
/// of [`media_stats`]. Support for Token Ring is obsolete and may be
/// removed.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct dot5stat {
    /// ACE errors, reported as `ace_errors`. The manual calls it
    /// `glds_dot5_ace_error`.
    pub ace_error: u32,
    /// Internal errors, reported as `internal_errors`. The manual calls it
    /// `glds_dot5_internal_error`.
    pub internal_error: u32,
    /// Lost frame errors, reported as `lost_frame_errors`. The manual calls
    /// it `glds_dot5_lost_frame_error`.
    pub lost_frame_error: u32,
    /// Frame copied errors, reported as `frame_copied_errors`. The manual
    /// calls it `glds_dot5_frame_copied_error`.
    pub frame_copied_error: u32,
    /// Token errors, reported as `token_errors`. The manual calls it
    /// `glds_dot5_token_error`.
    pub token_error: u32,
    /// Frequency errors, reported as `freq_errors`. The manual calls it
    /// `glds_dot5_freq_error`.
    pub freq_error: u32,
}

/// `gld_stats(9S)`. illumos architecture specific (illumos DDI).
///
/// The FDDI counters of the media specific statistics, the C `struct
/// fddistat`, a subset of RFC 1512. A driver for a `DL_FDDI` device fills
/// them in. They are reached through the `fddi` arm of [`media_stats`].
/// Support for FDDI is obsolete and may be removed.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct fddistat {
    /// MAC errors, reported as `mac_errors`. The manual calls it
    /// `glds_fddi_mac_error`.
    pub mac_error: u32,
    /// Lost MAC frames, reported as `mac_lost_errors`. The manual calls it
    /// `glds_fddi_mac_lost`.
    pub mac_lost: u32,
    /// MAC tokens, reported as `mac_tokens`. The manual calls it
    /// `glds_fddi_mac_token`.
    pub mac_token: u32,
    /// Expirations of the valid transmission timer, reported as
    /// `mac_tvx_expired`. The manual calls it `glds_fddi_mac_tvx_expired`.
    pub mac_tvx_expired: u32,
    /// Late MAC events, reported as `mac_late`. The manual calls it
    /// `glds_fddi_mac_late`.
    pub mac_late: u32,
    /// Ring operational events, reported as `mac_ring_ops`. The manual calls
    /// it `glds_fddi_mac_ring_op`.
    pub mac_ring_op: u32,
}

/// `gld_stats(9S)`. illumos architecture specific (illumos DDI).
///
/// The media specific counters inside [`gld_stats`], the C union
/// `media_stats` or `media_stats_t`. The arm to use follows from the
/// `gldm_type` of the device, `dot3` for Ethernet, `dot5` for Token Ring
/// and `fddi` for FDDI. The `pad` arm fixes the size of the union at 64
/// bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub union media_stats {
    /// The Ethernet counters.
    pub dot3: dot3stat,
    /// The Token Ring counters.
    pub dot5: dot5stat,
    /// The FDDI counters.
    pub fddi: fddistat,
    /// Padding that reserves room for more counters.
    pub pad: [u32; 16],
}

/// The typedef name of [`media_stats`].
pub type media_stats_t = media_stats;

/// `gld_stats(9S)`. illumos architecture specific (illumos DDI).
///
/// The statistics a GLDv2 driver returns from its `gldm_get_stats` entry
/// point, the C `struct gld_stats`. GLD passes a pointer to it, the driver
/// fills in the counters it keeps, and GLD reports them under the names
/// that `gld(4D)` describes. The members documented for all media types are
/// `glds_speed`, `glds_media`, `glds_intr`, `glds_norcvbuf`, `glds_errrcv`,
/// `glds_errxmt`, `glds_missed`, `glds_underflow` and `glds_overflow`. The
/// Ethernet specific members are `glds_frame`, `glds_crc`, `glds_duplex`,
/// `glds_nocarrier`, `glds_collisions`, `glds_excoll`, `glds_xmtlatecoll`,
/// `glds_defer` and `glds_short`, together with the `dot3` counters in
/// `glds_media_specific`. The remaining members are maintained by GLD
/// itself or are reserved, and the driver should leave them alone.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gld_stats {
    /// Maintained by GLD. Multicast packets sent, `ifOutMulticastPkts`.
    pub glds_multixmt: ulong_t,
    /// Maintained by GLD. Multicast packets received, `ifInMulticastPkts`.
    pub glds_multircv: ulong_t,
    /// Maintained by GLD. Broadcast packets sent, `ifOutBroadcastPkts`.
    pub glds_brdcstxmt: ulong_t,
    /// Maintained by GLD. Broadcast packets received, `ifInBroadcastPkts`.
    pub glds_brdcstrcv: ulong_t,
    /// Maintained by GLD. Received packets discarded because of flow control
    /// upstream.
    pub glds_blocked: u32,
    /// Reserved.
    pub glds_reserved1: u32,
    /// Reserved.
    pub glds_reserved2: u32,
    /// Reserved.
    pub glds_reserved3: u32,
    /// Reserved.
    pub glds_reserved4: u32,
    /// Set by the driver for all media. Packets that could not be sent
    /// because of errors, reported as `oerrors`.
    pub glds_errxmt: u32,
    /// Set by the driver for all media. Packets received with errors,
    /// reported as `ierrors`.
    pub glds_errrcv: u32,
    /// Set by an Ethernet driver. Collisions, reported as `collisions`.
    pub glds_collisions: u32,
    /// Set by an Ethernet driver. Frames that were not sent because of
    /// excessive collisions, reported as `ex_collisions`.
    pub glds_excoll: u32,
    /// Set by an Ethernet driver. Transmissions deferred because the medium
    /// was busy, reported as `defer_xmts`.
    pub glds_defer: u32,
    /// Set by an Ethernet driver. Frames received with alignment errors,
    /// reported as `align_errors`. For Token Ring the manual calls this
    /// member `glds_dot5_burst_error`, reported as `burst_errors`.
    pub glds_frame: u32,
    /// Set by an Ethernet driver. Frames received with a bad frame check
    /// sequence, reported as `fcs_errors`. For Token Ring the manual calls
    /// this member `glds_dot5_line_error`, reported as `line_errors`.
    pub glds_crc: u32,
    /// Set by the driver for all media. Receiver overflows, reported as
    /// `oflo`.
    pub glds_overflow: u32,
    /// Set by the driver for all media. Transmitter underflows, reported as
    /// `uflo`.
    pub glds_underflow: u32,
    /// Set by an Ethernet driver. Frames received that were too short,
    /// reported as `runt_errors`.
    pub glds_short: u32,
    /// Set by the driver for all media. Frames the hardware dropped on
    /// receive, reported as `missed`.
    pub glds_missed: u32,
    /// Set by an Ethernet driver. Late collisions on transmit, reported as
    /// `tx_late_collisions`.
    pub glds_xmtlatecoll: u32,
    /// Set by an Ethernet driver. Carrier sense errors on transmit, reported
    /// as `carrier_errors`. For Token Ring the manual calls this member
    /// `glds_dot5_signal_loss`, reported as `signal_losses`.
    pub glds_nocarrier: u32,
    /// Maintained by GLD. Packets discarded on transmit for lack of buffers,
    /// `ifOutDiscards`.
    pub glds_noxmtbuf: u32,
    /// Set by the driver for all media. Received packets dropped for lack of
    /// buffers, reported as `norcvbuf`.
    pub glds_norcvbuf: u32,
    /// Set by the driver for all media. Interrupts, reported as `intr`.
    pub glds_intr: u32,
    /// Maintained by GLD. Transmissions retried after `gldm_send` returned
    /// `GLD_NORESOURCES`.
    pub glds_xmtretry: u32,
    /// Maintained by GLD. Packets sent, as a 64-bit counter.
    pub glds_pktxmt64: u64,
    /// Maintained by GLD. Packets received, as a 64-bit counter.
    pub glds_pktrcv64: u64,
    /// Maintained by GLD. Bytes sent, `ifHCOutOctets`.
    pub glds_bytexmt64: u64,
    /// Maintained by GLD. Bytes received, `ifHCInOctets`.
    pub glds_bytercv64: u64,
    /// Set by the driver for all media. An estimate of the current bandwidth
    /// of the interface in bits per second, reported as `ifspeed`. An
    /// interface whose bandwidth does not vary, or which cannot estimate it,
    /// gives the nominal bandwidth.
    pub glds_speed: u64,
    /// Set by an Ethernet driver. The current duplex state, one of
    /// [`GLD_DUPLEX_HALF`](crate::GLD_DUPLEX_HALF),
    /// [`GLD_DUPLEX_FULL`](crate::GLD_DUPLEX_FULL) or
    /// [`GLD_DUPLEX_UNKNOWN`](crate::GLD_DUPLEX_UNKNOWN), reported as
    /// `duplex`.
    pub glds_duplex: u32,
    /// Set by the driver for all media. The type of media or connector the
    /// hardware uses, one of the `GLDM_*` values, reported as `media`.
    pub glds_media: u32,
    /// Maintained by GLD. Packets received for an unknown protocol,
    /// `ifInUnknownProtos`.
    pub glds_unknowns: u32,
    /// Reserved.
    pub reserved: [u32; 19],
    /// The media specific counters. See [`media_stats`].
    pub glds_media_specific: media_stats_t,
    /// Maintained by GLD. Packets sent with a bad length or format.
    pub glds_xmtbadinterp: u32,
    /// Maintained by GLD. Packets received with a bad length or format.
    pub glds_rcvbadinterp: u32,
    /// Maintained by GLD. Received packets GLD itself dropped for lack of
    /// buffers.
    pub glds_gldnorcvbuf: u32,
}

// Layout on amd64 with LP64 and _KERNEL defined, as sys/gld.h declares it.
// dot3stat, dot5stat and fddistat each hold six uint32_t, 24 bytes, and the
// union media_stats_t is as large as its pad arm of sixteen uint32_t, 64
// bytes with 4 byte alignment.
const _: () = assert!(core::mem::size_of::<dot3stat>() == 24);
const _: () = assert!(core::mem::offset_of!(dot3stat, first_coll) == 0);
const _: () = assert!(core::mem::offset_of!(dot3stat, mac_rcv_error) == 20);
const _: () = assert!(core::mem::size_of::<dot5stat>() == 24);
const _: () = assert!(core::mem::offset_of!(dot5stat, ace_error) == 0);
const _: () = assert!(core::mem::offset_of!(dot5stat, freq_error) == 20);
const _: () = assert!(core::mem::size_of::<fddistat>() == 24);
const _: () = assert!(core::mem::offset_of!(fddistat, mac_error) == 0);
const _: () = assert!(core::mem::offset_of!(fddistat, mac_ring_op) == 20);
const _: () = assert!(core::mem::size_of::<media_stats_t>() == 64);

// gld_stats. The four ulong_t glds_multixmt to glds_brdcstrcv occupy 0 to
// 32, the 22 uint32_t glds_blocked to glds_xmtretry 32 to 120 with
// glds_errxmt at 52, the five uint64_t glds_pktxmt64 to glds_speed 120 to
// 160, glds_duplex, glds_media and glds_unknowns 4 each from 160 to 172,
// reserved 76 bytes from 172 to 248, glds_media_specific 64 from 248 to
// 312, and glds_xmtbadinterp, glds_rcvbadinterp and glds_gldnorcvbuf 4 each
// from 312 to 324, padded to the 8 byte alignment of the structure; total
// 328. Checked with clang against the gate headers.
const _: () = assert!(core::mem::size_of::<gld_stats>() == 328);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_multixmt) == 0);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_brdcstrcv) == 24);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_blocked) == 32);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_errxmt) == 52);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_xmtretry) == 116);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_pktxmt64) == 120);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_speed) == 152);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_duplex) == 160);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_unknowns) == 168);
const _: () = assert!(core::mem::offset_of!(gld_stats, reserved) == 172);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_media_specific) == 248);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_xmtbadinterp) == 312);
const _: () = assert!(core::mem::offset_of!(gld_stats, glds_gldnorcvbuf) == 320);
