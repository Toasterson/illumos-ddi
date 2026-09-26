//! Statistics, from `sys/mac.h`, `sys/mac_ether.h` and `sys/mac_provider.h`.
//!
//! The framework reads a provider's statistics one at a time through
//! `mc_getstat(9E)`, and a ring's through `mri_stat(9E)`, naming each by a
//! `uint_t` identifier. Identifiers from [`MAC_STAT_MIN`] up to
//! [`MACTYPE_STAT_MIN`] are the general statistics of [`mac_driver_stat`],
//! based on the MIB-II counters of RFC 1213 and RFC 1573, which every
//! provider maintains. Identifiers from [`MACTYPE_STAT_MIN`] upwards are
//! defined by the MAC plugin; for Ethernet they are the [`ether_stat`]
//! values, based on RFC 1643 and the MII and GMII registers. Identifiers
//! below [`MAC_STAT_MIN`] belong to the framework itself, and the
//! `enum mac_mod_stat` that names them is not bound.
//!
//! The C header declares the statistics as the untagged enumerations
//! `enum mac_driver_stat` and `enum ether_stat`, without typedefs, and the
//! entry points receive them as a plain `uint_t`. Here each enumeration is a
//! transparent wrapper around `uint_t` named after the C tag, so an entry
//! point compares its argument with, for example,
//! `mac_driver_stat::MAC_STAT_RBYTES.0`. There is no `mac_stat_t` in the
//! headers.
//!
//! Counters are monotonic 64-bit values that only reset when the instance
//! detaches, and a provider that cannot support a statistic returns
//! `ENOTSUP` for it. Additional statistics of its own a provider exports
//! with `kstat_create(9F)`. The `MAC_STAT_ISACOUNTER` and
//! `ETHER_STAT_ISACOUNTER` macros of the headers have no function form and
//! are not bound.

use ddi_sys::uint_t;

/// The first identifier of the general statistics that each provider
/// maintains. Smaller identifiers belong to the framework (`MAC_STAT_MIN`).
pub const MAC_STAT_MIN: uint_t = 1000;
/// The first identifier of the statistics defined by a MAC plugin, such as
/// the Ethernet statistics (`MACTYPE_STAT_MIN`).
pub const MACTYPE_STAT_MIN: uint_t = 2000;

/// `mac(9E)`. illumos DDI specific.
///
/// The general device statistics, `enum mac_driver_stat`, which the
/// framework passes to `mc_getstat(9E)` and `mri_stat(9E)` as a `uint_t`.
/// They are based on the MIB-II statistics of RFC 1213 and RFC 1573. The
/// header forbids reordering them and only ever appends to the list.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct mac_driver_stat(pub uint_t);
impl mac_driver_stat {
    /// The current speed of the device in bits per second (`MAC_STAT_IFSPEED`).
    pub const MAC_STAT_IFSPEED: Self = Self(1000);
    /// The number of multicast packets received (`MAC_STAT_MULTIRCV`).
    pub const MAC_STAT_MULTIRCV: Self = Self(1001);
    /// The number of broadcast packets received (`MAC_STAT_BRDCSTRCV`).
    pub const MAC_STAT_BRDCSTRCV: Self = Self(1002);
    /// The number of multicast packets transmitted (`MAC_STAT_MULTIXMT`).
    pub const MAC_STAT_MULTIXMT: Self = Self(1003);
    /// The number of broadcast packets transmitted. The manual page says
    /// received, which is a slip (`MAC_STAT_BRDCSTXMT`).
    pub const MAC_STAT_BRDCSTXMT: Self = Self(1004);
    /// The number of packets the hardware discarded for lack of receive buffers
    /// (`MAC_STAT_NORCVBUF`).
    pub const MAC_STAT_NORCVBUF: Self = Self(1005);
    /// The number of errors detected on input (`MAC_STAT_IERRORS`).
    pub const MAC_STAT_IERRORS: Self = Self(1006);
    /// The number of received packets discarded because their protocol was
    /// unknown (`MAC_STAT_UNKNOWNS`).
    pub const MAC_STAT_UNKNOWNS: Self = Self(1007);
    /// The number of outgoing packets dropped for lack of transmit buffers
    /// (`MAC_STAT_NOXMTBUF`).
    pub const MAC_STAT_NOXMTBUF: Self = Self(1008);
    /// The number of outgoing packets that resulted in errors
    /// (`MAC_STAT_OERRORS`).
    pub const MAC_STAT_OERRORS: Self = Self(1009);
    /// The number of collisions the transmitter encountered
    /// (`MAC_STAT_COLLISIONS`).
    pub const MAC_STAT_COLLISIONS: Self = Self(1010);
    /// The number of bytes received, whatever the packet type
    /// (`MAC_STAT_RBYTES`).
    pub const MAC_STAT_RBYTES: Self = Self(1011);
    /// The number of packets received, whatever the packet type
    /// (`MAC_STAT_IPACKETS`).
    pub const MAC_STAT_IPACKETS: Self = Self(1012);
    /// The number of bytes transmitted, whatever the packet type
    /// (`MAC_STAT_OBYTES`).
    pub const MAC_STAT_OBYTES: Self = Self(1013);
    /// The number of packets transmitted, whatever the packet type
    /// (`MAC_STAT_OPACKETS`).
    pub const MAC_STAT_OPACKETS: Self = Self(1014);
    /// The number of packets dropped because they were smaller than the
    /// smallest packet the device handles (`MAC_STAT_UNDERFLOWS`).
    pub const MAC_STAT_UNDERFLOWS: Self = Self(1015);
    /// The number of packets dropped because they were larger than the largest
    /// packet the device handles (`MAC_STAT_OVERFLOWS`).
    pub const MAC_STAT_OVERFLOWS: Self = Self(1016);
}

const _: () = assert!(core::mem::size_of::<mac_driver_stat>() == 4);

/// The number of general device statistics, from `MAC_STAT_IFSPEED` to
/// `MAC_STAT_OVERFLOWS` inclusive (`MAC_NSTAT`).
pub const MAC_NSTAT: uint_t = 1016 - MAC_STAT_MIN + 1;
const _: () = assert!(MAC_NSTAT == 17);

/// `mac(9E)`. illumos DDI specific.
///
/// The Ethernet statistics, `enum ether_stat` from `sys/mac_ether.h`, which
/// the framework passes to `mc_getstat(9E)` of an Ethernet provider as a
/// `uint_t`. They are based on RFC 1643 and the MII and GMII registers, and
/// many are defined in IEEE 802.3. The statistics about capabilities report
/// 1 for yes and 0 for no. Those with `CAP` in the name describe the
/// hardware alone, ignoring the enable properties, while those with `ADV`
/// describe what the device currently advertises and those with `LP` what
/// the link partner advertises. The header forbids reordering them and only
/// ever appends to the list.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ether_stat(pub uint_t);
impl ether_stat {
    /// The number of alignment errors, meaning received frames that were not a
    /// whole number of octets long and failed the frame check sequence
    /// (`ETHER_STAT_ALIGN_ERRORS`).
    pub const ETHER_STAT_ALIGN_ERRORS: Self = Self(2000);
    /// The number of received frames whose frame check sequence failed
    /// (`ETHER_STAT_FCS_ERRORS`).
    pub const ETHER_STAT_FCS_ERRORS: Self = Self(2001);
    /// The number of frames transmitted successfully after exactly one
    /// collision (`ETHER_STAT_FIRST_COLLISIONS`).
    pub const ETHER_STAT_FIRST_COLLISIONS: Self = Self(2002);
    /// The number of frames transmitted successfully after more than one
    /// collision (`ETHER_STAT_MULTI_COLLISIONS`).
    pub const ETHER_STAT_MULTI_COLLISIONS: Self = Self(2003);
    /// The number of SQE errors, whose conditions IEEE 802.3 defines
    /// (`ETHER_STAT_SQE_ERRORS`).
    pub const ETHER_STAT_SQE_ERRORS: Self = Self(2004);
    /// The number of frames whose transmission was deferred because the medium
    /// was busy (`ETHER_STAT_DEFER_XMTS`).
    pub const ETHER_STAT_DEFER_XMTS: Self = Self(2005);
    /// The number of collisions detected late in a transmission
    /// (`ETHER_STAT_TX_LATE_COLLISIONS`).
    pub const ETHER_STAT_TX_LATE_COLLISIONS: Self = Self(2006);
    /// The number of frames that could not be sent because of excessive
    /// collisions (`ETHER_STAT_EX_COLLISIONS`).
    pub const ETHER_STAT_EX_COLLISIONS: Self = Self(2007);
    /// The number of internal MAC errors while processing and transmitting a
    /// frame (`ETHER_STAT_MACXMT_ERRORS`).
    pub const ETHER_STAT_MACXMT_ERRORS: Self = Self(2008);
    /// The number of times carrier sense was lost or not asserted
    /// (`ETHER_STAT_CARRIER_ERRORS`).
    pub const ETHER_STAT_CARRIER_ERRORS: Self = Self(2009);
    /// The number of received frames longer than the largest frame the device
    /// supports (`ETHER_STAT_TOOLONG_ERRORS`).
    pub const ETHER_STAT_TOOLONG_ERRORS: Self = Self(2010);
    /// The number of internal MAC errors while receiving and processing a frame
    /// (`ETHER_STAT_MACRCV_ERRORS`).
    pub const ETHER_STAT_MACRCV_ERRORS: Self = Self(2011);
    /// The address of the MII or GMII transceiver (`ETHER_STAT_XCVR_ADDR`).
    pub const ETHER_STAT_XCVR_ADDR: Self = Self(2012);
    /// The identifier of the MII or GMII transceiver (`ETHER_STAT_XCVR_ID`).
    pub const ETHER_STAT_XCVR_ID: Self = Self(2013);
    /// The kind of transceiver in use, as a `mac_ether_media_t` value. The
    /// values are compatible with the older `XCVR_*` constants
    /// (`ETHER_STAT_XCVR_INUSE`).
    pub const ETHER_STAT_XCVR_INUSE: Self = Self(2014);
    /// Whether the device supports 1 Gbit/s full-duplex operation, judged from
    /// the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_1000FDX`).
    pub const ETHER_STAT_CAP_1000FDX: Self = Self(2015);
    /// Whether the device supports 1 Gbit/s half-duplex operation, judged from
    /// the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_1000HDX`).
    pub const ETHER_STAT_CAP_1000HDX: Self = Self(2016);
    /// Whether the device supports 100 Mbit/s full-duplex operation, judged
    /// from the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_100FDX`).
    pub const ETHER_STAT_CAP_100FDX: Self = Self(2017);
    /// Whether the device supports 100 Mbit/s half-duplex operation, judged
    /// from the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_100HDX`).
    pub const ETHER_STAT_CAP_100HDX: Self = Self(2018);
    /// Whether the device supports 10 Mbit/s full-duplex operation, judged from
    /// the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_10FDX`).
    pub const ETHER_STAT_CAP_10FDX: Self = Self(2019);
    /// Whether the device supports 10 Mbit/s half-duplex operation, judged from
    /// the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_10HDX`).
    pub const ETHER_STAT_CAP_10HDX: Self = Self(2020);
    /// Whether the device can receive pause frames (`ETHER_STAT_CAP_ASMPAUSE`).
    pub const ETHER_STAT_CAP_ASMPAUSE: Self = Self(2021);
    /// Whether the device can transmit pause frames (`ETHER_STAT_CAP_PAUSE`).
    pub const ETHER_STAT_CAP_PAUSE: Self = Self(2022);
    /// Whether the device can perform link auto-negotiation
    /// (`ETHER_STAT_CAP_AUTONEG`).
    pub const ETHER_STAT_CAP_AUTONEG: Self = Self(2023);
    /// Whether the device is advertising support for 1 Gbit/s full-duplex
    /// operation (`ETHER_STAT_ADV_CAP_1000FDX`).
    pub const ETHER_STAT_ADV_CAP_1000FDX: Self = Self(2024);
    /// Whether the device is advertising support for 1 Gbit/s half-duplex
    /// operation (`ETHER_STAT_ADV_CAP_1000HDX`).
    pub const ETHER_STAT_ADV_CAP_1000HDX: Self = Self(2025);
    /// Whether the device is advertising support for 100 Mbit/s full-duplex
    /// operation (`ETHER_STAT_ADV_CAP_100FDX`).
    pub const ETHER_STAT_ADV_CAP_100FDX: Self = Self(2026);
    /// Whether the device is advertising support for 100 Mbit/s half-duplex
    /// operation (`ETHER_STAT_ADV_CAP_100HDX`).
    pub const ETHER_STAT_ADV_CAP_100HDX: Self = Self(2027);
    /// Whether the device is advertising support for 10 Mbit/s full-duplex
    /// operation (`ETHER_STAT_ADV_CAP_10FDX`).
    pub const ETHER_STAT_ADV_CAP_10FDX: Self = Self(2028);
    /// Whether the device is advertising support for 10 Mbit/s half-duplex
    /// operation (`ETHER_STAT_ADV_CAP_10HDX`).
    pub const ETHER_STAT_ADV_CAP_10HDX: Self = Self(2029);
    /// Whether the device is advertising that it can receive pause frames
    /// (`ETHER_STAT_ADV_CAP_ASMPAUSE`).
    pub const ETHER_STAT_ADV_CAP_ASMPAUSE: Self = Self(2030);
    /// Whether the device is advertising that it can generate pause frames
    /// (`ETHER_STAT_ADV_CAP_PAUSE`).
    pub const ETHER_STAT_ADV_CAP_PAUSE: Self = Self(2031);
    /// Whether the device is advertising support for auto-negotiation
    /// (`ETHER_STAT_ADV_CAP_AUTONEG`).
    pub const ETHER_STAT_ADV_CAP_AUTONEG: Self = Self(2032);
    /// Whether the link partner supports 1 Gbit/s full-duplex operation
    /// (`ETHER_STAT_LP_CAP_1000FDX`).
    pub const ETHER_STAT_LP_CAP_1000FDX: Self = Self(2033);
    /// Whether the link partner supports 1 Gbit/s half-duplex operation
    /// (`ETHER_STAT_LP_CAP_1000HDX`).
    pub const ETHER_STAT_LP_CAP_1000HDX: Self = Self(2034);
    /// Whether the link partner supports 100 Mbit/s full-duplex operation
    /// (`ETHER_STAT_LP_CAP_100FDX`).
    pub const ETHER_STAT_LP_CAP_100FDX: Self = Self(2035);
    /// Whether the link partner supports 100 Mbit/s half-duplex operation
    /// (`ETHER_STAT_LP_CAP_100HDX`).
    pub const ETHER_STAT_LP_CAP_100HDX: Self = Self(2036);
    /// Whether the link partner supports 10 Mbit/s full-duplex operation
    /// (`ETHER_STAT_LP_CAP_10FDX`).
    pub const ETHER_STAT_LP_CAP_10FDX: Self = Self(2037);
    /// Whether the link partner supports 10 Mbit/s half-duplex operation
    /// (`ETHER_STAT_LP_CAP_10HDX`).
    pub const ETHER_STAT_LP_CAP_10HDX: Self = Self(2038);
    /// Whether the link partner can receive pause frames
    /// (`ETHER_STAT_LP_CAP_ASMPAUSE`).
    pub const ETHER_STAT_LP_CAP_ASMPAUSE: Self = Self(2039);
    /// Whether the link partner can transmit pause frames
    /// (`ETHER_STAT_LP_CAP_PAUSE`).
    pub const ETHER_STAT_LP_CAP_PAUSE: Self = Self(2040);
    /// Whether the link partner can perform link auto-negotiation
    /// (`ETHER_STAT_LP_CAP_AUTONEG`).
    pub const ETHER_STAT_LP_CAP_AUTONEG: Self = Self(2041);
    /// Whether the link is currently configured to accept pause frames
    /// (`ETHER_STAT_LINK_ASMPAUSE`).
    pub const ETHER_STAT_LINK_ASMPAUSE: Self = Self(2042);
    /// Whether the link is currently configured to generate pause frames
    /// (`ETHER_STAT_LINK_PAUSE`).
    pub const ETHER_STAT_LINK_PAUSE: Self = Self(2043);
    /// Whether the current link state is the result of auto-negotiation
    /// (`ETHER_STAT_LINK_AUTONEG`).
    pub const ETHER_STAT_LINK_AUTONEG: Self = Self(2044);
    /// The current duplex state of the link, using the `link_duplex_t` values
    /// of `MAC_PROP_DUPLEX` (`ETHER_STAT_LINK_DUPLEX`).
    pub const ETHER_STAT_LINK_DUPLEX: Self = Self(2045);
    /// The number of received frames shorter than the smallest frame the device
    /// supports (`ETHER_STAT_TOOSHORT_ERRORS`).
    pub const ETHER_STAT_TOOSHORT_ERRORS: Self = Self(2046);
    /// Whether the device can detect a remote fault in its link partner
    /// (`ETHER_STAT_CAP_REMFAULT`).
    pub const ETHER_STAT_CAP_REMFAULT: Self = Self(2047);
    /// Whether the device is advertising that it can detect faults in its link
    /// partner (`ETHER_STAT_ADV_REMFAULT`).
    pub const ETHER_STAT_ADV_REMFAULT: Self = Self(2048);
    /// Whether the link partner can detect a remote fault. The manual page
    /// spells the name `ETHER_STAT_LP_CAP_REMFAULT`, which the header does not
    /// define (`ETHER_STAT_LP_REMFAULT`).
    pub const ETHER_STAT_LP_REMFAULT: Self = Self(2049);
    /// The number of received frames that were longer than the largest packet
    /// and also failed the frame check sequence (`ETHER_STAT_JABBER_ERRORS`).
    pub const ETHER_STAT_JABBER_ERRORS: Self = Self(2050);
    /// Whether the device supports 100BASE-T4 operation, judged from the
    /// hardware alone and not from what is enabled (`ETHER_STAT_CAP_100T4`).
    pub const ETHER_STAT_CAP_100T4: Self = Self(2051);
    /// Whether the device is advertising support for 100BASE-T4 operation
    /// (`ETHER_STAT_ADV_CAP_100T4`).
    pub const ETHER_STAT_ADV_CAP_100T4: Self = Self(2052);
    /// Whether the link partner supports 100BASE-T4 operation
    /// (`ETHER_STAT_LP_CAP_100T4`).
    pub const ETHER_STAT_LP_CAP_100T4: Self = Self(2053);
    /// Whether the device supports 10 Gbit/s full-duplex operation, judged from
    /// the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_10GFDX`).
    pub const ETHER_STAT_CAP_10GFDX: Self = Self(2054);
    /// Whether the device is advertising support for 10 Gbit/s full-duplex
    /// operation (`ETHER_STAT_ADV_CAP_10GFDX`).
    pub const ETHER_STAT_ADV_CAP_10GFDX: Self = Self(2055);
    /// Whether the link partner supports 10 Gbit/s full-duplex operation
    /// (`ETHER_STAT_LP_CAP_10GFDX`).
    pub const ETHER_STAT_LP_CAP_10GFDX: Self = Self(2056);
    /// Whether the device supports 40 Gbit/s full-duplex operation, judged from
    /// the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_40GFDX`).
    pub const ETHER_STAT_CAP_40GFDX: Self = Self(2057);
    /// Whether the device is advertising support for 40 Gbit/s full-duplex
    /// operation (`ETHER_STAT_ADV_CAP_40GFDX`).
    pub const ETHER_STAT_ADV_CAP_40GFDX: Self = Self(2058);
    /// Whether the link partner supports 40 Gbit/s full-duplex operation
    /// (`ETHER_STAT_LP_CAP_40GFDX`).
    pub const ETHER_STAT_LP_CAP_40GFDX: Self = Self(2059);
    /// Whether the device supports 100 Gbit/s full-duplex operation, judged
    /// from the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_100GFDX`).
    pub const ETHER_STAT_CAP_100GFDX: Self = Self(2060);
    /// Whether the device is advertising support for 100 Gbit/s full-duplex
    /// operation (`ETHER_STAT_ADV_CAP_100GFDX`).
    pub const ETHER_STAT_ADV_CAP_100GFDX: Self = Self(2061);
    /// Whether the link partner supports 100 Gbit/s full-duplex operation
    /// (`ETHER_STAT_LP_CAP_100GFDX`).
    pub const ETHER_STAT_LP_CAP_100GFDX: Self = Self(2062);
    /// Whether the device supports 2.5 Gbit/s full-duplex operation, judged
    /// from the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_2500FDX`).
    pub const ETHER_STAT_CAP_2500FDX: Self = Self(2063);
    /// Whether the device is advertising support for 2.5 Gbit/s full-duplex
    /// operation (`ETHER_STAT_ADV_CAP_2500FDX`).
    pub const ETHER_STAT_ADV_CAP_2500FDX: Self = Self(2064);
    /// Whether the link partner supports 2.5 Gbit/s full-duplex operation
    /// (`ETHER_STAT_LP_CAP_2500FDX`).
    pub const ETHER_STAT_LP_CAP_2500FDX: Self = Self(2065);
    /// Whether the device supports 5.0 Gbit/s full-duplex operation, judged
    /// from the hardware alone and not from what is enabled
    /// (`ETHER_STAT_CAP_5000FDX`).
    pub const ETHER_STAT_CAP_5000FDX: Self = Self(2066);
    /// Whether the device is advertising support for 5.0 Gbit/s full-duplex
    /// operation (`ETHER_STAT_ADV_CAP_5000FDX`).
    pub const ETHER_STAT_ADV_CAP_5000FDX: Self = Self(2067);
    /// Whether the link partner supports 5.0 Gbit/s full-duplex operation
    /// (`ETHER_STAT_LP_CAP_5000FDX`).
    pub const ETHER_STAT_LP_CAP_5000FDX: Self = Self(2068);
    /// Whether the device supports 25 Gbit/s full-duplex operation, judged from
    /// the hardware alone and not from what is enabled. The manual does not
    /// list this speed, but the statistic follows the same rules as the others
    /// (`ETHER_STAT_CAP_25GFDX`).
    pub const ETHER_STAT_CAP_25GFDX: Self = Self(2069);
    /// Whether the device is advertising support for 25 Gbit/s full-duplex
    /// operation. The manual does not list this speed, but the statistic
    /// follows the same rules as the others (`ETHER_STAT_ADV_CAP_25GFDX`).
    pub const ETHER_STAT_ADV_CAP_25GFDX: Self = Self(2070);
    /// Whether the link partner supports 25 Gbit/s full-duplex operation. The
    /// manual does not list this speed, but the statistic follows the same
    /// rules as the others (`ETHER_STAT_LP_CAP_25GFDX`).
    pub const ETHER_STAT_LP_CAP_25GFDX: Self = Self(2071);
    /// Whether the device supports 50 Gbit/s full-duplex operation, judged from
    /// the hardware alone and not from what is enabled. The manual does not
    /// list this speed, but the statistic follows the same rules as the others
    /// (`ETHER_STAT_CAP_50GFDX`).
    pub const ETHER_STAT_CAP_50GFDX: Self = Self(2072);
    /// Whether the device is advertising support for 50 Gbit/s full-duplex
    /// operation. The manual does not list this speed, but the statistic
    /// follows the same rules as the others (`ETHER_STAT_ADV_CAP_50GFDX`).
    pub const ETHER_STAT_ADV_CAP_50GFDX: Self = Self(2073);
    /// Whether the link partner supports 50 Gbit/s full-duplex operation. The
    /// manual does not list this speed, but the statistic follows the same
    /// rules as the others (`ETHER_STAT_LP_CAP_50GFDX`).
    pub const ETHER_STAT_LP_CAP_50GFDX: Self = Self(2074);
    /// Whether the device supports 200 Gbit/s full-duplex operation, judged
    /// from the hardware alone and not from what is enabled. The manual does
    /// not list this speed, but the statistic follows the same rules as the
    /// others (`ETHER_STAT_CAP_200GFDX`).
    pub const ETHER_STAT_CAP_200GFDX: Self = Self(2075);
    /// Whether the device is advertising support for 200 Gbit/s full-duplex
    /// operation. The manual does not list this speed, but the statistic
    /// follows the same rules as the others (`ETHER_STAT_ADV_CAP_200GFDX`).
    pub const ETHER_STAT_ADV_CAP_200GFDX: Self = Self(2076);
    /// Whether the link partner supports 200 Gbit/s full-duplex operation. The
    /// manual does not list this speed, but the statistic follows the same
    /// rules as the others (`ETHER_STAT_LP_CAP_200GFDX`).
    pub const ETHER_STAT_LP_CAP_200GFDX: Self = Self(2077);
    /// Whether the device supports 400 Gbit/s full-duplex operation, judged
    /// from the hardware alone and not from what is enabled. The manual does
    /// not list this speed, but the statistic follows the same rules as the
    /// others (`ETHER_STAT_CAP_400GFDX`).
    pub const ETHER_STAT_CAP_400GFDX: Self = Self(2078);
    /// Whether the device is advertising support for 400 Gbit/s full-duplex
    /// operation. The manual does not list this speed, but the statistic
    /// follows the same rules as the others (`ETHER_STAT_ADV_CAP_400GFDX`).
    pub const ETHER_STAT_ADV_CAP_400GFDX: Self = Self(2079);
    /// Whether the link partner supports 400 Gbit/s full-duplex operation. The
    /// manual does not list this speed, but the statistic follows the same
    /// rules as the others (`ETHER_STAT_LP_CAP_400GFDX`).
    pub const ETHER_STAT_LP_CAP_400GFDX: Self = Self(2080);
}

const _: () = assert!(core::mem::size_of::<ether_stat>() == 4);

/// The number of Ethernet statistics, from `ETHER_STAT_ALIGN_ERRORS` to
/// `ETHER_STAT_LP_CAP_400GFDX` inclusive (`ETHER_NSTAT`).
pub const ETHER_NSTAT: uint_t = 2080 - MACTYPE_STAT_MIN + 1;
const _: () = assert!(ETHER_NSTAT == 81);

/// The older value of `ETHER_STAT_XCVR_INUSE` for an undefined transceiver.
/// It equals `ETHER_MEDIA_UNKNOWN`, which new providers use instead
/// (`XCVR_UNDEFINED`).
pub const XCVR_UNDEFINED: uint_t = 0;
/// The older value of `ETHER_STAT_XCVR_INUSE` for no transceiver. It equals
/// `ETHER_MEDIA_NONE` (`XCVR_NONE`).
pub const XCVR_NONE: uint_t = 1;
/// The older value of `ETHER_STAT_XCVR_INUSE` for 10 Mbit/s. It equals
/// `ETHER_MEDIA_10BASE_T` (`XCVR_10`).
pub const XCVR_10: uint_t = 2;
/// The older value of `ETHER_STAT_XCVR_INUSE` for 100BASE-T4. It equals
/// `ETHER_MEDIA_100BASE_T4` (`XCVR_100T4`).
pub const XCVR_100T4: uint_t = 3;
/// The older value of `ETHER_STAT_XCVR_INUSE` for 100BASE-X. It equals
/// `ETHER_MEDIA_100BASE_X` (`XCVR_100X`).
pub const XCVR_100X: uint_t = 4;
/// The older value of `ETHER_STAT_XCVR_INUSE` for 100BASE-T2. It equals
/// `ETHER_MEDIA_100BASE_T2` (`XCVR_100T2`).
pub const XCVR_100T2: uint_t = 5;
/// The older value of `ETHER_STAT_XCVR_INUSE` for 1000BASE-X. It equals
/// `ETHER_MEDIA_1000BASE_X` (`XCVR_1000X`).
pub const XCVR_1000X: uint_t = 6;
/// The older value of `ETHER_STAT_XCVR_INUSE` for 1000BASE-T. It equals
/// `ETHER_MEDIA_1000BASE_T` (`XCVR_1000T`).
pub const XCVR_1000T: uint_t = 7;
