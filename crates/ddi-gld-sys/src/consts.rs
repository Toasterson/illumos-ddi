//! Return values and flag values from `sys/gld.h`, with the three DLPI media
//! types from `sys/dlpi.h` that `gld_mac_info(9S)` names.
//!
//! The return values are what the `gld(9E)` entry points give back to GLD.
//! The promiscuous mode and multicast flags are the arguments GLD passes to
//! `gldm_set_promiscuous` and `gldm_set_multicast`. The capability bits go
//! into `gldm_capabilities`, the link states are the argument of
//! `gld_linkstate(9F)`, the media types go into `gldm_type`, and the media
//! and duplex values go into the `glds_media` and `glds_duplex` members of
//! `gld_stats(9S)`. Each constant has the C type of the member or argument
//! it is used with.

use ddi_sys::c_int;

// --- entry point return values ---------------------------------------------

/// The entry point succeeded. `gldm_stop` and `gldm_ioctl` should always
/// return this value (`GLD_SUCCESS`).
pub const GLD_SUCCESS: c_int = 0;
/// There are not enough resources to carry out the request at this time.
/// Only `gldm_set_mac_addr`, `gldm_set_multicast`, `gldm_set_promiscuous`
/// and `gldm_send` may return this value, and a driver whose `gldm_send`
/// returns it must later call `gld_sched(9F)` (`GLD_NORESOURCES`).
pub const GLD_NORESOURCES: c_int = 1;
/// The requested function is not supported. Only `gldm_set_mac_addr`,
/// `gldm_set_multicast` and `gldm_set_promiscuous` may return this value
/// (`GLD_NOTSUPPORTED`).
pub const GLD_NOTSUPPORTED: c_int = 2;
/// The entry point detected an unsuitable argument, such as a bad multicast
/// address, a bad MAC address, or a bad packet or packet length
/// (`GLD_BADARG`).
pub const GLD_BADARG: c_int = 3;
/// `gldm_send` was called while there is no physical connection to a
/// network or link partner (`GLD_NOLINK`).
pub const GLD_NOLINK: c_int = 4;
/// The operation should be retried. The header defines this value, but
/// `gld(9E)` does not list it among the values an entry point may return
/// (`GLD_RETRY`).
pub const GLD_RETRY: c_int = 5;
/// The hardware failed (`GLD_FAILURE`).
pub const GLD_FAILURE: c_int = -1;

// --- promiscuous mode ------------------------------------------------------

/// Leave the promiscuous mode unchanged. GLD uses this value internally to
/// decide whether to call `gldm_set_promiscuous` at all and never passes it
/// to the driver, which should in any case treat an unrecognised value as
/// [`GLD_MAC_PROMISC_PHYS`] (`GLD_MAC_PROMISC_NOOP`).
pub const GLD_MAC_PROMISC_NOOP: c_int = -1;
/// Turn promiscuous mode off (`GLD_MAC_PROMISC_NONE`).
pub const GLD_MAC_PROMISC_NONE: c_int = 0;
/// Enable physical level promiscuous mode, so that every packet on the
/// medium is received (`GLD_MAC_PROMISC_PHYS`).
pub const GLD_MAC_PROMISC_PHYS: c_int = 1;
/// Receive every multicast packet on the medium. A device without a
/// multicast-only promiscuous mode should enter physical promiscuous mode
/// instead and return `GLD_SUCCESS`, and GLD filters out the packets that
/// are not wanted (`GLD_MAC_PROMISC_MULTI`).
pub const GLD_MAC_PROMISC_MULTI: c_int = 2;

// --- multicast -------------------------------------------------------------

/// Enable reception of the multicast address passed to
/// `gldm_set_multicast` (`GLD_MULTI_ENABLE`).
pub const GLD_MULTI_ENABLE: c_int = 1;
/// Allow the driver to disable reception of the multicast address passed to
/// `gldm_set_multicast` (`GLD_MULTI_DISABLE`).
pub const GLD_MULTI_DISABLE: c_int = 0;

// --- capabilities ----------------------------------------------------------

/// The device can report the state of its media link, and the driver will
/// call `gld_linkstate(9F)`. This is the only capability bit that
/// `gld_mac_info(9S)` documents (`GLD_CAP_LINKSTATE`).
pub const GLD_CAP_LINKSTATE: u32 = 0x0000_0001;
/// The device computes the IPv4 header checksum on transmit. GLD
/// advertises it to its clients in the hardware checksum capability it
/// returns for `DL_CAPABILITY_REQ`. The bit is honoured by GLD, but no
/// manual page documents it (`GLD_CAP_CKSUM_IPHDR`).
pub const GLD_CAP_CKSUM_IPHDR: u32 = 0x0000_0008;
/// The device computes a partial TCP or UDP checksum on transmit, starting
/// from an offset its client gives. GLD advertises it in the hardware
/// checksum capability. The bit is honoured by GLD, but no manual page
/// documents it (`GLD_CAP_CKSUM_PARTIAL`).
pub const GLD_CAP_CKSUM_PARTIAL: u32 = 0x0000_0010;
/// The device computes the full TCP or UDP checksum of IPv4 packets on
/// transmit. GLD advertises it in the hardware checksum capability. The bit
/// is honoured by GLD, but no manual page documents it
/// (`GLD_CAP_CKSUM_FULL_V4`).
pub const GLD_CAP_CKSUM_FULL_V4: u32 = 0x0000_0020;
/// The driver can safely transmit from memory that its client loans to it
/// instead of copying, so GLD advertises the zero-copy capability to its
/// clients and avoids loaned memory in the copies it keeps for loopback.
/// The bit is honoured by GLD, but no manual page documents it
/// (`GLD_CAP_ZEROCOPY`).
pub const GLD_CAP_ZEROCOPY: u32 = 0x0000_0040;
/// The device computes the full TCP or UDP checksum of IPv6 packets on
/// transmit. GLD advertises it in the hardware checksum capability. The bit
/// is honoured by GLD, but no manual page documents it
/// (`GLD_CAP_CKSUM_FULL_V6`).
pub const GLD_CAP_CKSUM_FULL_V6: u32 = 0x0000_0080;
/// The union of all checksum offload bits, which GLD tests to decide
/// whether the device offloads any checksum at all (`GLD_CAP_CKSUM_ANY`).
pub const GLD_CAP_CKSUM_ANY: u32 =
    GLD_CAP_CKSUM_IPHDR | GLD_CAP_CKSUM_PARTIAL | GLD_CAP_CKSUM_FULL_V4 | GLD_CAP_CKSUM_FULL_V6;

// --- link state ------------------------------------------------------------

/// The media link is unavailable (`GLD_LINKSTATE_DOWN`).
pub const GLD_LINKSTATE_DOWN: i32 = -1;
/// The state of the media link is unknown (`GLD_LINKSTATE_UNKNOWN`).
pub const GLD_LINKSTATE_UNKNOWN: i32 = 0;
/// The media link is up. The `gld(9F)` page describes this value with the
/// same words as [`GLD_LINKSTATE_DOWN`], which is a mistake in the page, and
/// GLD reports it to its clients as a link up notification
/// (`GLD_LINKSTATE_UP`).
pub const GLD_LINKSTATE_UP: i32 = 1;

// --- media types (gldm_type) -----------------------------------------------

/// IEEE 802.5 Token Passing Ring, a value for `gldm_type` defined in
/// `sys/dlpi.h`. The manual marks support for this media type as obsolete
/// and liable to be removed (`DL_TPR`).
pub const DL_TPR: u32 = 0x2;
/// IEEE 802.3 and Ethernet, a value for `gldm_type` defined in `sys/dlpi.h`
/// (`DL_ETHER`).
pub const DL_ETHER: u32 = 0x4;
/// ISO 9314-2 Fibre Distributed Data Interface, a value for `gldm_type`
/// defined in `sys/dlpi.h`. The manual marks support for this media type as
/// obsolete and liable to be removed (`DL_FDDI`).
pub const DL_FDDI: u32 = 0x08;

// --- media and connectors (glds_media) -------------------------------------

/// The driver cannot determine the media or connector type
/// (`GLDM_UNKNOWN`).
pub const GLDM_UNKNOWN: u32 = 0;
/// An attachment unit interface connector (`GLDM_AUI`).
pub const GLDM_AUI: u32 = 1;
/// A BNC connector for thin coaxial cable (`GLDM_BNC`).
pub const GLDM_BNC: u32 = 2;
/// Twisted pair wiring (`GLDM_TP`).
pub const GLDM_TP: u32 = 3;
/// Optical fibre (`GLDM_FIBER`).
pub const GLDM_FIBER: u32 = 4;
/// 100 megabit twisted pair Ethernet (`GLDM_100BT`).
pub const GLDM_100BT: u32 = 5;
/// 100VG-AnyLAN. The header defines this value, but `gld_stats(9S)` does not
/// list it among the supported media names (`GLDM_VGANYLAN`).
pub const GLDM_VGANYLAN: u32 = 6;
/// 10 megabit twisted pair Ethernet (`GLDM_10BT`).
pub const GLDM_10BT: u32 = 7;
/// A 4 megabit Token Ring (`GLDM_RING4`).
pub const GLDM_RING4: u32 = 8;
/// A 16 megabit Token Ring (`GLDM_RING16`).
pub const GLDM_RING16: u32 = 9;
/// A media independent interface to an external transceiver
/// (`GLDM_PHYMII`).
pub const GLDM_PHYMII: u32 = 10;
/// 100BASE-TX Ethernet (`GLDM_100BTX`).
pub const GLDM_100BTX: u32 = 11;
/// 100BASE-T4 Ethernet (`GLDM_100BT4`).
pub const GLDM_100BT4: u32 = 12;
/// IP over InfiniBand. The header defines this value, but `gld_stats(9S)`
/// does not list it among the supported media names (`GLDM_IB`).
pub const GLDM_IB: u32 = 14;

// --- duplex (glds_duplex) --------------------------------------------------

/// The duplex state of the interface is unknown (`GLD_DUPLEX_UNKNOWN`).
pub const GLD_DUPLEX_UNKNOWN: u32 = 0;
/// The interface runs in half duplex (`GLD_DUPLEX_HALF`).
pub const GLD_DUPLEX_HALF: u32 = 1;
/// The interface runs in full duplex (`GLD_DUPLEX_FULL`).
pub const GLD_DUPLEX_FULL: u32 = 2;
