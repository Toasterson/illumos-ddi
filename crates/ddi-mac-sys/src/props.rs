//! Properties, from `sys/mac.h`, `sys/mac_provider.h` and `sys/mac_ether.h`.
//!
//! Properties describe and configure a link, for example its state, speed,
//! MTU, flow control and which speeds auto-negotiation advertises.
//! Administrators see them through `dladm(8)`. A provider implements them in
//! three entry points of `mac_callbacks(9S)`. `mc_getprop(9E)` reads a
//! property, `mc_setprop(9E)` changes one, and `mc_propinfo(9E)` describes
//! one by calling the `mac_prop_info(9F)` functions declared here. Each
//! entry point receives the property as a [`mac_prop_id_t`], and private
//! properties arrive as [`mac_prop_id_t::MAC_PROP_PRIVATE`] together with
//! their name.
//!
//! The speed properties come in two families. Those with `EN` in the name
//! are read/write and say what the administrator allows the device to
//! advertise; those with `ADV` in the name are read-only and say what the
//! device currently advertises. The manual's rule for all three entry points
//! is to check first whether the hardware supports the speed at all, and to
//! return `ENOTSUP` if it does not.
//!
//! The value types that properties use are declared here as well, namely
//! [`link_state_t`], [`link_duplex_t`], [`link_flowctrl_t`], [`link_fec_t`]
//! and, for `MAC_PROP_MEDIA` on Ethernet, [`mac_ether_media_t`]. The header
//! also defines `link_tagmode_t`, the range structures `mac_propval_range_t`,
//! `mac_propval_uint32_range_t` and `mac_propval_str_range_t`, the flags
//! `MAC_PROP_MAP_KSTAT` and `MAC_PROP_FLAGS_RK`, and the function
//! `mac_prop_info_set_default_uint16`, which `mac_prop_info(9F)` documents
//! but which the header does not declare and the kernel does not define.
//! None of those is bound.

use ddi_sys::{c_char, c_int, c_uint};

use crate::types::mac_prop_info_handle_t;

/// `mac(9E)`. illumos DDI specific.
///
/// The identifier of a property, `mac_prop_id_t`, passed to
/// `mc_getprop(9E)`, `mc_setprop(9E)` and `mc_propinfo(9E)`. The header
/// calls every value public and adds new ones only at the end. `mac(9E)`
/// lists the subset a provider should handle, namely the private property,
/// duplex, speed, status, media, auto-negotiation, MTU, flow control,
/// forward error correction and the speed families. For any other value a
/// provider returns `ENOTSUP` from `mc_getprop(9E)` and `mc_setprop(9E)` and
/// does nothing in `mc_propinfo(9E)`. The size a provider checks
/// `pr_valsize` against is the size of the type given for each property.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct mac_prop_id_t(pub c_int);
impl mac_prop_id_t {
    /// A private property of the provider, one of the names listed in the
    /// `m_priv_props` member of `mac_register(9S)`. The entry point identifies
    /// it by comparing the `pr_name` argument with `strcmp(9F)`
    /// (`MAC_PROP_PRIVATE`).
    pub const MAC_PROP_PRIVATE: Self = Self(-1);
    /// The duplex state of the link as a `link_duplex_t`. It is read-only
    /// (`MAC_PROP_DUPLEX`).
    pub const MAC_PROP_DUPLEX: Self = Self(1);
    /// The current link speed in bits per second as a `uint64_t`, for example
    /// 100000000 for 100 Mbit/s. It is read-only (`MAC_PROP_SPEED`).
    pub const MAC_PROP_SPEED: Self = Self(2);
    /// The state of the link as a `link_state_t`. It is read-only
    /// (`MAC_PROP_STATUS`).
    pub const MAC_PROP_STATUS: Self = Self(3);
    /// Whether the device performs auto-negotiation, as a `uint8_t` that is
    /// non-zero for yes. It is read/write and should default to enabled. When
    /// it is set, the provider updates the hardware and restarts the link so
    /// that it negotiates again (`MAC_PROP_AUTONEG`).
    pub const MAC_PROP_AUTONEG: Self = Self(4);
    /// Whether auto-negotiation is enabled. It is not described by `mac(9E)`,
    /// which uses `MAC_PROP_AUTONEG` for this purpose, so a provider returns
    /// `ENOTSUP` for it (`MAC_PROP_EN_AUTONEG`).
    pub const MAC_PROP_EN_AUTONEG: Self = Self(5);
    /// The maximum transmission unit as a `uint32_t`, the largest payload
    /// without the provider's own headers and VLAN tags. It is read/write and
    /// should default to 1500 for Ethernet. To change it, the provider
    /// validates the value, calls `mac_maxsdu_update(9F)`, which may fail, and
    /// only then updates the hardware. A provider that cannot change it while
    /// started returns `EBUSY` (`MAC_PROP_MTU`).
    pub const MAC_PROP_MTU: Self = Self(6);
    /// The zone a link is assigned to. The framework or another part of the
    /// datalink layer handles it, and `mac(9E)` does not list it among the
    /// properties a provider handles, so a provider for which it has no meaning
    /// returns `ENOTSUP` (`MAC_PROP_ZONE`).
    pub const MAC_PROP_ZONE: Self = Self(7);
    /// The STREAMS modules pushed automatically on the link. The framework or
    /// another part of the datalink layer handles it, and `mac(9E)` does not
    /// list it among the properties a provider handles, so a provider for which
    /// it has no meaning returns `ENOTSUP` (`MAC_PROP_AUTOPUSH`).
    pub const MAC_PROP_AUTOPUSH: Self = Self(8);
    /// The pause frame configuration the device advertises, as a
    /// `link_flowctrl_t`. It is read/write. Reading it returns the
    /// configuration, not what was negotiated, and setting it updates the
    /// hardware and may restart auto-negotiation (`MAC_PROP_FLOWCTRL`).
    pub const MAC_PROP_FLOWCTRL: Self = Self(9);
    /// Whether 1 Gbit/s full-duplex support is advertised, as a `uint8_t` that
    /// is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_1000FDX_CAP`).
    pub const MAC_PROP_ADV_1000FDX_CAP: Self = Self(10);
    /// Whether 1 Gbit/s full-duplex support is enabled for advertisement, as a
    /// `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_1000FDX_CAP`).
    pub const MAC_PROP_EN_1000FDX_CAP: Self = Self(11);
    /// Whether 1 Gbit/s half-duplex support is advertised, as a `uint8_t` that
    /// is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_1000HDX_CAP`).
    pub const MAC_PROP_ADV_1000HDX_CAP: Self = Self(12);
    /// Whether 1 Gbit/s half-duplex support is enabled for advertisement, as a
    /// `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_1000HDX_CAP`).
    pub const MAC_PROP_EN_1000HDX_CAP: Self = Self(13);
    /// Whether 100 Mbit/s full-duplex support is advertised, as a `uint8_t`
    /// that is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_100FDX_CAP`).
    pub const MAC_PROP_ADV_100FDX_CAP: Self = Self(14);
    /// Whether 100 Mbit/s full-duplex support is enabled for advertisement, as
    /// a `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_100FDX_CAP`).
    pub const MAC_PROP_EN_100FDX_CAP: Self = Self(15);
    /// Whether 100 Mbit/s half-duplex support is advertised, as a `uint8_t`
    /// that is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_100HDX_CAP`).
    pub const MAC_PROP_ADV_100HDX_CAP: Self = Self(16);
    /// Whether 100 Mbit/s half-duplex support is enabled for advertisement, as
    /// a `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_100HDX_CAP`).
    pub const MAC_PROP_EN_100HDX_CAP: Self = Self(17);
    /// Whether 10 Mbit/s full-duplex support is advertised, as a `uint8_t` that
    /// is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_10FDX_CAP`).
    pub const MAC_PROP_ADV_10FDX_CAP: Self = Self(18);
    /// Whether 10 Mbit/s full-duplex support is enabled for advertisement, as a
    /// `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_10FDX_CAP`).
    pub const MAC_PROP_EN_10FDX_CAP: Self = Self(19);
    /// Whether 10 Mbit/s half-duplex support is advertised, as a `uint8_t` that
    /// is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_10HDX_CAP`).
    pub const MAC_PROP_ADV_10HDX_CAP: Self = Self(20);
    /// Whether 10 Mbit/s half-duplex support is enabled for advertisement, as a
    /// `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_10HDX_CAP`).
    pub const MAC_PROP_EN_10HDX_CAP: Self = Self(21);
    /// Whether 100BASE-T4 support is advertised, as a `uint8_t` that is
    /// non-zero for yes. It is read-only and reflects what the device currently
    /// advertises and has negotiated, which may differ from the matching enable
    /// property (`MAC_PROP_ADV_100T4_CAP`).
    pub const MAC_PROP_ADV_100T4_CAP: Self = Self(22);
    /// Whether 100BASE-T4 support is enabled for advertisement, as a `uint8_t`
    /// that is non-zero for yes. It is read/write, and setting it changes what
    /// the hardware advertises and restarts auto-negotiation if that is enabled
    /// (`MAC_PROP_EN_100T4_CAP`).
    pub const MAC_PROP_EN_100T4_CAP: Self = Self(23);
    /// The hop limit of an IP tunnel link. The framework or another part of the
    /// datalink layer handles it, and `mac(9E)` does not list it among the
    /// properties a provider handles, so a provider for which it has no meaning
    /// returns `ENOTSUP` (`MAC_PROP_IPTUN_HOPLIMIT`).
    pub const MAC_PROP_IPTUN_HOPLIMIT: Self = Self(24);
    /// The encapsulation limit of an IPv6 tunnel link. The framework or another
    /// part of the datalink layer handles it, and `mac(9E)` does not list it
    /// among the properties a provider handles, so a provider for which it has
    /// no meaning returns `ENOTSUP` (`MAC_PROP_IPTUN_ENCAPLIMIT`).
    pub const MAC_PROP_IPTUN_ENCAPLIMIT: Self = Self(25);
    /// The ESSID of a wireless link. It is handled by wireless providers built
    /// on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_ESSID`).
    pub const MAC_PROP_WL_ESSID: Self = Self(26);
    /// The BSSID of a wireless link. It is handled by wireless providers built
    /// on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_BSSID`).
    pub const MAC_PROP_WL_BSSID: Self = Self(27);
    /// The BSS type of a wireless link. It is handled by wireless providers
    /// built on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_BSSTYPE`).
    pub const MAC_PROP_WL_BSSTYPE: Self = Self(28);
    /// The link status of a wireless link. It is handled by wireless providers
    /// built on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_LINKSTATUS`).
    pub const MAC_PROP_WL_LINKSTATUS: Self = Self(29);
    /// The desired rates of a wireless link. It is handled by wireless
    /// providers built on the `mac_wifi` plugin, and `mac(9E)` does not
    /// describe it (`MAC_PROP_WL_DESIRED_RATES`).
    pub const MAC_PROP_WL_DESIRED_RATES: Self = Self(30);
    /// The supported rates of a wireless link. It is handled by wireless
    /// providers built on the `mac_wifi` plugin, and `mac(9E)` does not
    /// describe it (`MAC_PROP_WL_SUPPORTED_RATES`).
    pub const MAC_PROP_WL_SUPPORTED_RATES: Self = Self(31);
    /// The authentication mode of a wireless link. It is handled by wireless
    /// providers built on the `mac_wifi` plugin, and `mac(9E)` does not
    /// describe it (`MAC_PROP_WL_AUTH_MODE`).
    pub const MAC_PROP_WL_AUTH_MODE: Self = Self(32);
    /// The encryption setting of a wireless link. It is handled by wireless
    /// providers built on the `mac_wifi` plugin, and `mac(9E)` does not
    /// describe it (`MAC_PROP_WL_ENCRYPTION`).
    pub const MAC_PROP_WL_ENCRYPTION: Self = Self(33);
    /// The received signal strength of a wireless link. It is handled by
    /// wireless providers built on the `mac_wifi` plugin, and `mac(9E)` does
    /// not describe it (`MAC_PROP_WL_RSSI`).
    pub const MAC_PROP_WL_RSSI: Self = Self(34);
    /// The physical layer configuration of a wireless link. It is handled by
    /// wireless providers built on the `mac_wifi` plugin, and `mac(9E)` does
    /// not describe it (`MAC_PROP_WL_PHY_CONFIG`).
    pub const MAC_PROP_WL_PHY_CONFIG: Self = Self(35);
    /// The capabilities of a wireless link. It is handled by wireless providers
    /// built on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_CAPABILITY`).
    pub const MAC_PROP_WL_CAPABILITY: Self = Self(36);
    /// The WPA setting of a wireless link. It is handled by wireless providers
    /// built on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_WPA`).
    pub const MAC_PROP_WL_WPA: Self = Self(37);
    /// The scan results of a wireless link. It is handled by wireless providers
    /// built on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_SCANRESULTS`).
    pub const MAC_PROP_WL_SCANRESULTS: Self = Self(38);
    /// The power mode of a wireless link. It is handled by wireless providers
    /// built on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_POWER_MODE`).
    pub const MAC_PROP_WL_POWER_MODE: Self = Self(39);
    /// The radio state of a wireless link. It is handled by wireless providers
    /// built on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_RADIO`).
    pub const MAC_PROP_WL_RADIO: Self = Self(40);
    /// The list of networks seen by a wireless link. It is handled by wireless
    /// providers built on the `mac_wifi` plugin, and `mac(9E)` does not
    /// describe it (`MAC_PROP_WL_ESS_LIST`).
    pub const MAC_PROP_WL_ESS_LIST: Self = Self(41);
    /// The key table of a wireless link. It is handled by wireless providers
    /// built on the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_KEY_TAB`).
    pub const MAC_PROP_WL_KEY_TAB: Self = Self(42);
    /// Whether a wireless link creates an IBSS. It is handled by wireless
    /// providers built on the `mac_wifi` plugin, and `mac(9E)` does not
    /// describe it (`MAC_PROP_WL_CREATE_IBSS`).
    pub const MAC_PROP_WL_CREATE_IBSS: Self = Self(43);
    /// An optional information element of a wireless link. It is handled by
    /// wireless providers built on the `mac_wifi` plugin, and `mac(9E)` does
    /// not describe it (`MAC_PROP_WL_SETOPTIE`).
    pub const MAC_PROP_WL_SETOPTIE: Self = Self(44);
    /// The deletion of a key of a wireless link. It is handled by wireless
    /// providers built on the `mac_wifi` plugin, and `mac(9E)` does not
    /// describe it (`MAC_PROP_WL_DELKEY`).
    pub const MAC_PROP_WL_DELKEY: Self = Self(45);
    /// A key of a wireless link. It is handled by wireless providers built on
    /// the `mac_wifi` plugin, and `mac(9E)` does not describe it
    /// (`MAC_PROP_WL_KEY`).
    pub const MAC_PROP_WL_KEY: Self = Self(46);
    /// A management request of a wireless link. It is handled by wireless
    /// providers built on the `mac_wifi` plugin, and `mac(9E)` does not
    /// describe it (`MAC_PROP_WL_MLME`).
    pub const MAC_PROP_WL_MLME: Self = Self(47);
    /// The VLAN tagging mode of a link. The framework or another part of the
    /// datalink layer handles it, and `mac(9E)` does not list it among the
    /// properties a provider handles, so a provider for which it has no meaning
    /// returns `ENOTSUP` (`MAC_PROP_TAGMODE`).
    pub const MAC_PROP_TAGMODE: Self = Self(48);
    /// Whether 10 Gbit/s full-duplex support is advertised, as a `uint8_t` that
    /// is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_10GFDX_CAP`).
    pub const MAC_PROP_ADV_10GFDX_CAP: Self = Self(49);
    /// Whether 10 Gbit/s full-duplex support is enabled for advertisement, as a
    /// `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_10GFDX_CAP`).
    pub const MAC_PROP_EN_10GFDX_CAP: Self = Self(50);
    /// The port VLAN identifier of a link. The framework or another part of the
    /// datalink layer handles it, and `mac(9E)` does not list it among the
    /// properties a provider handles, so a provider for which it has no meaning
    /// returns `ENOTSUP` (`MAC_PROP_PVID`).
    pub const MAC_PROP_PVID: Self = Self(51);
    /// The learning limit of a bridged link. The framework or another part of
    /// the datalink layer handles it, and `mac(9E)` does not list it among the
    /// properties a provider handles, so a provider for which it has no meaning
    /// returns `ENOTSUP` (`MAC_PROP_LLIMIT`).
    pub const MAC_PROP_LLIMIT: Self = Self(52);
    /// The learning decay of a bridged link. The framework or another part of
    /// the datalink layer handles it, and `mac(9E)` does not list it among the
    /// properties a provider handles, so a provider for which it has no meaning
    /// returns `ENOTSUP` (`MAC_PROP_LDECAY`).
    pub const MAC_PROP_LDECAY: Self = Self(53);
    /// The resource settings of a MAC client. The framework or another part of
    /// the datalink layer handles it, and `mac(9E)` does not list it among the
    /// properties a provider handles, so a provider for which it has no meaning
    /// returns `ENOTSUP` (`MAC_PROP_RESOURCE`).
    pub const MAC_PROP_RESOURCE: Self = Self(54);
    /// The effective resource settings of a MAC client. The framework or
    /// another part of the datalink layer handles it, and `mac(9E)` does not
    /// list it among the properties a provider handles, so a provider for which
    /// it has no meaning returns `ENOTSUP` (`MAC_PROP_RESOURCE_EFF`).
    pub const MAC_PROP_RESOURCE_EFF: Self = Self(55);
    /// The range of receive rings a client can be given. The framework or
    /// another part of the datalink layer handles it, and `mac(9E)` does not
    /// list it among the properties a provider handles, so a provider for which
    /// it has no meaning returns `ENOTSUP` (`MAC_PROP_RXRINGSRANGE`).
    pub const MAC_PROP_RXRINGSRANGE: Self = Self(56);
    /// The range of transmit rings a client can be given. The framework or
    /// another part of the datalink layer handles it, and `mac(9E)` does not
    /// list it among the properties a provider handles, so a provider for which
    /// it has no meaning returns `ENOTSUP` (`MAC_PROP_TXRINGSRANGE`).
    pub const MAC_PROP_TXRINGSRANGE: Self = Self(57);
    /// The number of transmit rings still available to clients. The framework
    /// or another part of the datalink layer handles it, and `mac(9E)` does not
    /// list it among the properties a provider handles, so a provider for which
    /// it has no meaning returns `ENOTSUP` (`MAC_PROP_MAX_TX_RINGS_AVAIL`).
    pub const MAC_PROP_MAX_TX_RINGS_AVAIL: Self = Self(58);
    /// The number of receive rings still available to clients. The framework or
    /// another part of the datalink layer handles it, and `mac(9E)` does not
    /// list it among the properties a provider handles, so a provider for which
    /// it has no meaning returns `ENOTSUP` (`MAC_PROP_MAX_RX_RINGS_AVAIL`).
    pub const MAC_PROP_MAX_RX_RINGS_AVAIL: Self = Self(59);
    /// The number of clients that can still get a hardware receive group. The
    /// framework or another part of the datalink layer handles it, and
    /// `mac(9E)` does not list it among the properties a provider handles, so a
    /// provider for which it has no meaning returns `ENOTSUP`
    /// (`MAC_PROP_MAX_RXHWCLNT_AVAIL`).
    pub const MAC_PROP_MAX_RXHWCLNT_AVAIL: Self = Self(60);
    /// The number of clients that can still get a hardware transmit group. The
    /// framework or another part of the datalink layer handles it, and
    /// `mac(9E)` does not list it among the properties a provider handles, so a
    /// provider for which it has no meaning returns `ENOTSUP`
    /// (`MAC_PROP_MAX_TXHWCLNT_AVAIL`).
    pub const MAC_PROP_MAX_TXHWCLNT_AVAIL: Self = Self(61);
    /// The link mode of an InfiniBand link. The framework or another part of
    /// the datalink layer handles it, and `mac(9E)` does not list it among the
    /// properties a provider handles, so a provider for which it has no meaning
    /// returns `ENOTSUP` (`MAC_PROP_IB_LINKMODE`).
    pub const MAC_PROP_IB_LINKMODE: Self = Self(62);
    /// Whether promiscuous traffic to a VNIC is filtered. The framework or
    /// another part of the datalink layer handles it, and `mac(9E)` does not
    /// list it among the properties a provider handles, so a provider for which
    /// it has no meaning returns `ENOTSUP` (`MAC_PROP_VN_PROMISC_FILTERED`).
    pub const MAC_PROP_VN_PROMISC_FILTERED: Self = Self(63);
    /// The secondary MAC addresses of a VNIC. The framework or another part of
    /// the datalink layer handles it, and `mac(9E)` does not list it among the
    /// properties a provider handles, so a provider for which it has no meaning
    /// returns `ENOTSUP` (`MAC_PROP_SECONDARY_ADDRS`).
    pub const MAC_PROP_SECONDARY_ADDRS: Self = Self(64);
    /// Whether 40 Gbit/s full-duplex support is advertised, as a `uint8_t` that
    /// is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_40GFDX_CAP`).
    pub const MAC_PROP_ADV_40GFDX_CAP: Self = Self(65);
    /// Whether 40 Gbit/s full-duplex support is enabled for advertisement, as a
    /// `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_40GFDX_CAP`).
    pub const MAC_PROP_EN_40GFDX_CAP: Self = Self(66);
    /// Whether 100 Gbit/s full-duplex support is advertised, as a `uint8_t`
    /// that is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_100GFDX_CAP`).
    pub const MAC_PROP_ADV_100GFDX_CAP: Self = Self(67);
    /// Whether 100 Gbit/s full-duplex support is enabled for advertisement, as
    /// a `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_100GFDX_CAP`).
    pub const MAC_PROP_EN_100GFDX_CAP: Self = Self(68);
    /// Whether 2.5 Gbit/s full-duplex support is advertised, as a `uint8_t`
    /// that is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_2500FDX_CAP`).
    pub const MAC_PROP_ADV_2500FDX_CAP: Self = Self(69);
    /// Whether 2.5 Gbit/s full-duplex support is enabled for advertisement, as
    /// a `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_2500FDX_CAP`).
    pub const MAC_PROP_EN_2500FDX_CAP: Self = Self(70);
    /// Whether 5.0 Gbit/s full-duplex support is advertised, as a `uint8_t`
    /// that is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_5000FDX_CAP`).
    pub const MAC_PROP_ADV_5000FDX_CAP: Self = Self(71);
    /// Whether 5.0 Gbit/s full-duplex support is enabled for advertisement, as
    /// a `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_5000FDX_CAP`).
    pub const MAC_PROP_EN_5000FDX_CAP: Self = Self(72);
    /// Whether 25 Gbit/s full-duplex support is advertised, as a `uint8_t` that
    /// is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property. The manual does not list this speed, but it
    /// follows the same rules as the others (`MAC_PROP_ADV_25GFDX_CAP`).
    pub const MAC_PROP_ADV_25GFDX_CAP: Self = Self(73);
    /// Whether 25 Gbit/s full-duplex support is enabled for advertisement, as a
    /// `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled. The manual does not list this speed, but it follows the
    /// same rules as the others (`MAC_PROP_EN_25GFDX_CAP`).
    pub const MAC_PROP_EN_25GFDX_CAP: Self = Self(74);
    /// Whether 50 Gbit/s full-duplex support is advertised, as a `uint8_t` that
    /// is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property. The manual does not list this speed, but it
    /// follows the same rules as the others (`MAC_PROP_ADV_50GFDX_CAP`).
    pub const MAC_PROP_ADV_50GFDX_CAP: Self = Self(75);
    /// Whether 50 Gbit/s full-duplex support is enabled for advertisement, as a
    /// `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled. The manual does not list this speed, but it follows the
    /// same rules as the others (`MAC_PROP_EN_50GFDX_CAP`).
    pub const MAC_PROP_EN_50GFDX_CAP: Self = Self(76);
    /// The forward error correction codings the device advertises, as a
    /// `link_fec_t`. It is read/write and should default to `LINK_FEC_AUTO`. A
    /// provider returns `EINVAL` for a combination the hardware does not
    /// support (`MAC_PROP_EN_FEC_CAP`).
    pub const MAC_PROP_EN_FEC_CAP: Self = Self(77);
    /// The forward error correction coding negotiated on the link, as a
    /// `link_fec_t`. It is read-only (`MAC_PROP_ADV_FEC_CAP`).
    pub const MAC_PROP_ADV_FEC_CAP: Self = Self(78);
    /// The media currently in use on the link, as a `uint32_t` holding a value
    /// specific to the plugin, which for Ethernet is a `mac_ether_media_t`. It
    /// is read-only, is updated once auto-negotiation has completed, and should
    /// report unknown media rather than a guess (`MAC_PROP_MEDIA`).
    pub const MAC_PROP_MEDIA: Self = Self(79);
    /// Whether 200 Gbit/s full-duplex support is advertised, as a `uint8_t`
    /// that is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_200GFDX_CAP`).
    pub const MAC_PROP_ADV_200GFDX_CAP: Self = Self(80);
    /// Whether 200 Gbit/s full-duplex support is enabled for advertisement, as
    /// a `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_200GFDX_CAP`).
    pub const MAC_PROP_EN_200GFDX_CAP: Self = Self(81);
    /// Whether 400 Gbit/s full-duplex support is advertised, as a `uint8_t`
    /// that is non-zero for yes. It is read-only and reflects what the device
    /// currently advertises and has negotiated, which may differ from the
    /// matching enable property (`MAC_PROP_ADV_400GFDX_CAP`).
    pub const MAC_PROP_ADV_400GFDX_CAP: Self = Self(82);
    /// Whether 400 Gbit/s full-duplex support is enabled for advertisement, as
    /// a `uint8_t` that is non-zero for yes. It is read/write, and setting it
    /// changes what the hardware advertises and restarts auto-negotiation if
    /// that is enabled (`MAC_PROP_EN_400GFDX_CAP`).
    pub const MAC_PROP_EN_400GFDX_CAP: Self = Self(83);
}

const _: () = assert!(core::mem::size_of::<mac_prop_id_t>() == 4);

/// `mac(9E)`. illumos DDI specific.
///
/// The state of a link, `link_state_t`. It is the type of the
/// `MAC_PROP_STATUS` property and the argument of `mac_link_update(9F)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct link_state_t(pub c_int);
impl link_state_t {
    /// The state of the link is unknown, for example because `mc_start(9E)`
    /// has not been called yet. A provider that powers the link off or can no
    /// longer tell its state reports this value (`LINK_STATE_UNKNOWN`).
    pub const LINK_STATE_UNKNOWN: Self = Self(-1);
    /// The link is down, for example because of a negotiation or cable
    /// problem (`LINK_STATE_DOWN`).
    pub const LINK_STATE_DOWN: Self = Self(0);
    /// The link is up and auto-negotiation, if used, has completed, so
    /// traffic can flow (`LINK_STATE_UP`).
    pub const LINK_STATE_UP: Self = Self(1);
}

const _: () = assert!(core::mem::size_of::<link_state_t>() == 4);

/// `mac(9E)`. illumos DDI specific.
///
/// The duplex state of a link, `link_duplex_t`. It is the type of the
/// `MAC_PROP_DUPLEX` property and the value of the `ETHER_STAT_LINK_DUPLEX`
/// statistic.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct link_duplex_t(pub c_int);
impl link_duplex_t {
    /// The duplex state is unknown, for example because the link is down or
    /// has not negotiated a speed (`LINK_DUPLEX_UNKNOWN`).
    pub const LINK_DUPLEX_UNKNOWN: Self = Self(0);
    /// The link is half duplex, so traffic flows in one direction at a time
    /// (`LINK_DUPLEX_HALF`).
    pub const LINK_DUPLEX_HALF: Self = Self(1);
    /// The link is full duplex, so traffic flows in both directions at once
    /// (`LINK_DUPLEX_FULL`).
    pub const LINK_DUPLEX_FULL: Self = Self(2);
}

const _: () = assert!(core::mem::size_of::<link_duplex_t>() == 4);

/// `mac(9E)`. illumos DDI specific.
///
/// The Ethernet flow control configuration a device advertises,
/// `link_flowctrl_t`. It is the type of the `MAC_PROP_FLOWCTRL` property and
/// the argument of `mac_prop_info_set_default_link_flowctrl(9F)`. What is
/// actually in effect is subject to auto-negotiation.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct link_flowctrl_t(pub c_int);
impl link_flowctrl_t {
    /// No pause frames are generated or honoured (`LINK_FLOWCTRL_NONE`).
    pub const LINK_FLOWCTRL_NONE: Self = Self(0);
    /// Pause frames are honoured when received but never generated
    /// (`LINK_FLOWCTRL_RX`).
    pub const LINK_FLOWCTRL_RX: Self = Self(1);
    /// Pause frames are generated but not honoured when received
    /// (`LINK_FLOWCTRL_TX`).
    pub const LINK_FLOWCTRL_TX: Self = Self(2);
    /// Pause frames are both generated and honoured (`LINK_FLOWCTRL_BI`).
    pub const LINK_FLOWCTRL_BI: Self = Self(3);
}

const _: () = assert!(core::mem::size_of::<link_flowctrl_t>() == 4);

/// `mac(9E)`. illumos DDI specific.
///
/// A set of forward error correction codings, `link_fec_t`. Each value is a
/// single bit, and a property value may combine several of them with a
/// bitwise inclusive OR, except that `LINK_FEC_AUTO` stands alone. It is the
/// type of the `MAC_PROP_EN_FEC_CAP` and `MAC_PROP_ADV_FEC_CAP` properties
/// and the argument of `mac_prop_info_set_default_fec(9F)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct link_fec_t(pub c_uint);
impl link_fec_t {
    /// No forward error correction on the link (`LINK_FEC_NONE`).
    pub const LINK_FEC_NONE: Self = Self(1 << 0);
    /// The coding is chosen by auto-negotiation. It cannot be combined with
    /// the other values and is the default a provider should use
    /// (`LINK_FEC_AUTO`).
    pub const LINK_FEC_AUTO: Self = Self(1 << 1);
    /// The link may use Reed-Solomon coding (`LINK_FEC_RS`).
    pub const LINK_FEC_RS: Self = Self(1 << 2);
    /// The link may use BASE-R coding, also known as FireCode
    /// (`LINK_FEC_BASE_R`).
    pub const LINK_FEC_BASE_R: Self = Self(1 << 3);
}

const _: () = assert!(core::mem::size_of::<link_fec_t>() == 4);

/// `mac(9E)`. illumos DDI specific.
///
/// The media of an Ethernet link, `mac_ether_media_t`, from
/// `sys/mac_ether.h`. It is the value of the `MAC_PROP_MEDIA` property of an
/// Ethernet provider, stored in a `uint32_t`, and of the
/// `ETHER_STAT_XCVR_INUSE` statistic. The values generally follow the
/// physical medium dependent layers of IEEE 802.3. A provider reports the
/// media actually on the link rather than the electrical interface between
/// its MAC and PHY, and reports `ETHER_MEDIA_UNKNOWN` rather than guess, for
/// example when an SFP module does not reveal whether it is 40GBASE-SR4 or
/// 40GBASE-LR4.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct mac_ether_media_t(pub c_uint);
impl mac_ether_media_t {
    /// The driver cannot tell the media, because the link is in a state where
    /// it is unknown or because the hardware, firmware and driver cannot
    /// determine it. When no media is present and the link is down, use
    /// `ETHER_MEDIA_NONE` instead (`ETHER_MEDIA_UNKNOWN`).
    pub const ETHER_MEDIA_UNKNOWN: Self = Self(0);
    /// No specific media is in use, which is generally the case when the link
    /// is down (`ETHER_MEDIA_NONE`).
    pub const ETHER_MEDIA_NONE: Self = Self(1);
    /// Traditional 10 Mbit/s Ethernet over CAT-3 cabling, defined in 802.3i
    /// (`ETHER_MEDIA_10BASE_T`).
    pub const ETHER_MEDIA_10BASE_T: Self = Self(2);
    /// An uncommon half-duplex form of 100 Mbit/s Ethernet over four twisted
    /// pairs of CAT-3 cable, defined in 802.3u (`ETHER_MEDIA_100BASE_T4`).
    pub const ETHER_MEDIA_100BASE_T4: Self = Self(3);
    /// The general term for the 100BASE-TX and 100BASE-FX variants, for PHYs
    /// that do not tell them apart. A driver that can tell copper from fiber
    /// should use the more specific value (`ETHER_MEDIA_100BASE_X`).
    pub const ETHER_MEDIA_100BASE_X: Self = Self(4);
    /// An uncommon form of 100 Mbit/s Ethernet over two twisted pairs of CAT-3
    /// cable, defined in 802.3y. Some older drivers reported this value for
    /// 100BASE-TX, for which no value existed before
    /// (`ETHER_MEDIA_100BASE_T2`).
    pub const ETHER_MEDIA_100BASE_T2: Self = Self(5);
    /// The catch-all for 1 Gbit/s fiber operation, kept for compatibility with
    /// the generic information of traditional 802.3 PHYs. More specific
    /// information should be used when it is available
    /// (`ETHER_MEDIA_1000BASE_X`).
    pub const ETHER_MEDIA_1000BASE_X: Self = Self(6);
    /// Traditional 1 Gbit/s Ethernet over four twisted pairs of CAT-5 cable,
    /// defined in 802.3ab (`ETHER_MEDIA_1000BASE_T`).
    pub const ETHER_MEDIA_1000BASE_T: Self = Self(7);
    /// 1 Gbit/s Ethernet over a backplane, defined in 802.3ap
    /// (`ETHER_MEDIA_1000BASE_KX`).
    pub const ETHER_MEDIA_1000BASE_KX: Self = Self(8);
    /// A more recent form of 1 Gbit/s Ethernet over a single twisted pair
    /// (`ETHER_MEDIA_1000BASE_T1`).
    pub const ETHER_MEDIA_1000BASE_T1: Self = Self(9);
    /// An older form of 1 Gbit/s Ethernet over balanced copper cables, defined
    /// in 802.3z (`ETHER_MEDIA_1000BASE_CX`).
    pub const ETHER_MEDIA_1000BASE_CX: Self = Self(10);
    /// 1 Gbit/s Ethernet over a pair of multi-mode fibers, one for each
    /// direction (`ETHER_MEDIA_1000BASE_SX`).
    pub const ETHER_MEDIA_1000BASE_SX: Self = Self(11);
    /// 1 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction (`ETHER_MEDIA_1000BASE_LX`).
    pub const ETHER_MEDIA_1000BASE_LX: Self = Self(12);
    /// 1 Gbit/s Ethernet over a single bidirectional single-mode fiber
    /// (`ETHER_MEDIA_1000BASE_BX`).
    pub const ETHER_MEDIA_1000BASE_BX: Self = Self(13);
    /// 1 Gbit/s signalling defined by Cisco for chip-to-chip connections
    /// (`ETHER_MEDIA_1000_SGMII`).
    pub const ETHER_MEDIA_1000_SGMII: Self = Self(14);
    /// The most common form of 100 Mbit/s Ethernet, over two twisted pairs of
    /// CAT-5 cable, defined in 802.3u (`ETHER_MEDIA_100BASE_TX`).
    pub const ETHER_MEDIA_100BASE_TX: Self = Self(15);
    /// 100 Mbit/s Ethernet over multi-mode fiber, defined in 802.3u
    /// (`ETHER_MEDIA_100BASE_FX`).
    pub const ETHER_MEDIA_100BASE_FX: Self = Self(16);
    /// 100 Mbit/s signalling over the serial gigabit media-independent
    /// interface, generally used for chip-to-chip connections
    /// (`ETHER_MEDIA_100_SGMII`).
    pub const ETHER_MEDIA_100_SGMII: Self = Self(17);
    /// A more recent form of 10 Mbit/s Ethernet over a single twisted pair,
    /// defined in 802.3cg (`ETHER_MEDIA_10BASE_T1`).
    pub const ETHER_MEDIA_10BASE_T1: Self = Self(18);
    /// A more recent form of 100 Mbit/s Ethernet over a single twisted pair,
    /// defined in 802.3bw (`ETHER_MEDIA_100BASE_T1`).
    pub const ETHER_MEDIA_100BASE_T1: Self = Self(19);
    /// 2.5 Gbit/s Ethernet over four copper twisted pairs, defined in 802.3bz
    /// (`ETHER_MEDIA_2500BASE_T`).
    pub const ETHER_MEDIA_2500BASE_T: Self = Self(20);
    /// 2.5 Gbit/s Ethernet over a backplane, defined in 802.3cb
    /// (`ETHER_MEDIA_2500BASE_KX`).
    pub const ETHER_MEDIA_2500BASE_KX: Self = Self(21);
    /// A de facto standard that runs 1000BASE-X with a clock 2.5 times as fast
    /// (`ETHER_MEDIA_2500BASE_X`).
    pub const ETHER_MEDIA_2500BASE_X: Self = Self(22);
    /// 5.0 Gbit/s Ethernet over four copper twisted pairs, defined in 802.3bz
    /// (`ETHER_MEDIA_5000BASE_T`).
    pub const ETHER_MEDIA_5000BASE_T: Self = Self(23);
    /// 5.0 Gbit/s Ethernet over a backplane, defined in 802.3cb
    /// (`ETHER_MEDIA_5000BASE_KR`).
    pub const ETHER_MEDIA_5000BASE_KR: Self = Self(24);
    /// 10 Gbit/s Ethernet over four twisted pairs of CAT-6a cable, defined in
    /// 802.3an (`ETHER_MEDIA_10GBASE_T`).
    pub const ETHER_MEDIA_10GBASE_T: Self = Self(25);
    /// 10 Gbit/s Ethernet over a pair of multi-mode fibers, one for each
    /// direction, defined in 802.3ae (`ETHER_MEDIA_10GBASE_SR`).
    pub const ETHER_MEDIA_10GBASE_SR: Self = Self(26);
    /// 10 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, up to 10 km, defined in 802.3ae (`ETHER_MEDIA_10GBASE_LR`).
    pub const ETHER_MEDIA_10GBASE_LR: Self = Self(27);
    /// 10 Gbit/s Ethernet over a pair of multi-mode fibers, one for each
    /// direction, reaching up to 220 m, which is farther than 10GBASE-SR,
    /// defined in 802.3aq (`ETHER_MEDIA_10GBASE_LRM`).
    pub const ETHER_MEDIA_10GBASE_LRM: Self = Self(28);
    /// 10 Gbit/s Ethernet over a single lane backplane, defined in 802.3ap
    /// (`ETHER_MEDIA_10GBASE_KR`).
    pub const ETHER_MEDIA_10GBASE_KR: Self = Self(29);
    /// 10 Gbit/s Ethernet over a group of four shielded copper cables, defined
    /// in 802.3ak (`ETHER_MEDIA_10GBASE_CX4`).
    pub const ETHER_MEDIA_10GBASE_CX4: Self = Self(30);
    /// 10 Gbit/s Ethernet over a four lane backplane, defined in 802.3ap
    /// (`ETHER_MEDIA_10GBASE_KX4`).
    pub const ETHER_MEDIA_10GBASE_KX4: Self = Self(31);
    /// 10 Gbit/s signalling between a MAC and a PHY, the X attachment unit
    /// interface, sometimes also used between chips, defined in 802.3ae
    /// (`ETHER_MEDIA_10G_XAUI`).
    pub const ETHER_MEDIA_10G_XAUI: Self = Self(32);
    /// 10 Gbit/s Ethernet over a short-range active optical cable compatible
    /// with SFP+, defined in SFF-8431 (`ETHER_MEDIA_10GBASE_AOC`).
    pub const ETHER_MEDIA_10GBASE_AOC: Self = Self(33);
    /// 10 Gbit/s Ethernet over a single lane copper cable with an active
    /// component that reaches farther than 10GBASE-CR, defined in SFF-8431
    /// (`ETHER_MEDIA_10GBASE_ACC`).
    pub const ETHER_MEDIA_10GBASE_ACC: Self = Self(34);
    /// 10 Gbit/s Ethernet over a passive copper cable compatible with SFP,
    /// sometimes called 10GSFP+Cu passive, defined in SFF-8431
    /// (`ETHER_MEDIA_10GBASE_CR`).
    pub const ETHER_MEDIA_10GBASE_CR: Self = Self(35);
    /// 10 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, up to 30 km, defined in 802.3ae (`ETHER_MEDIA_10GBASE_ER`).
    pub const ETHER_MEDIA_10GBASE_ER: Self = Self(36);
    /// 10 Gbit/s signalling between a MAC and an SFP transceiver, defined in
    /// SFF-8431 (`ETHER_MEDIA_10G_SFI`).
    pub const ETHER_MEDIA_10G_SFI: Self = Self(37);
    /// 10 Gbit/s signalling between a MAC and an XFP transceiver, defined in
    /// INF-8077i (`ETHER_MEDIA_10G_XFI`).
    pub const ETHER_MEDIA_10G_XFI: Self = Self(38);
    /// 25 Gbit/s Ethernet over four twisted pairs of CAT-8 cable, defined in
    /// 802.3bq (`ETHER_MEDIA_25GBASE_T`).
    pub const ETHER_MEDIA_25GBASE_T: Self = Self(39);
    /// 25 Gbit/s Ethernet over a pair of multi-mode fibers, one for each
    /// direction, defined in 802.3by (`ETHER_MEDIA_25GBASE_SR`).
    pub const ETHER_MEDIA_25GBASE_SR: Self = Self(40);
    /// 25 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, up to 10 km, defined in 802.3cc (`ETHER_MEDIA_25GBASE_LR`).
    pub const ETHER_MEDIA_25GBASE_LR: Self = Self(41);
    /// 25 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, up to 30 km, defined in 802.3cc (`ETHER_MEDIA_25GBASE_ER`).
    pub const ETHER_MEDIA_25GBASE_ER: Self = Self(42);
    /// 25 Gbit/s Ethernet over a single lane backplane, defined in 802.3by
    /// (`ETHER_MEDIA_25GBASE_KR`).
    pub const ETHER_MEDIA_25GBASE_KR: Self = Self(43);
    /// 25 Gbit/s Ethernet over a single lane copper cable, generally with an
    /// SFP28 connector, defined in 802.3by (`ETHER_MEDIA_25GBASE_CR`).
    pub const ETHER_MEDIA_25GBASE_CR: Self = Self(44);
    /// 25 Gbit/s Ethernet over a short-range active optical cable compatible
    /// with SFP28, loosely defined by SFF-8402 and often based on 25GBASE-SR
    /// (`ETHER_MEDIA_25GBASE_AOC`).
    pub const ETHER_MEDIA_25GBASE_AOC: Self = Self(45);
    /// 25 Gbit/s Ethernet over a single lane copper cable with an active
    /// component that reaches farther than 25GBASE-CR, loosely defined by
    /// SFF-8402 (`ETHER_MEDIA_25GBASE_ACC`).
    pub const ETHER_MEDIA_25GBASE_ACC: Self = Self(46);
    /// 25 Gbit/s signalling between a MAC and a PHY or between chips, defined
    /// in 802.3by (`ETHER_MEDIA_25G_AUI`).
    pub const ETHER_MEDIA_25G_AUI: Self = Self(47);
    /// 40 Gbit/s Ethernet over four twisted pairs of CAT-8 cable, defined in
    /// 802.3bq (`ETHER_MEDIA_40GBASE_T`).
    pub const ETHER_MEDIA_40GBASE_T: Self = Self(48);
    /// 40 Gbit/s Ethernet over four lanes of twinaxial copper cable at 10
    /// Gbit/s each, generally with a QSFP+ connector, defined in 802.3ba
    /// (`ETHER_MEDIA_40GBASE_CR4`).
    pub const ETHER_MEDIA_40GBASE_CR4: Self = Self(49);
    /// 40 Gbit/s Ethernet over four lanes of a copper backplane at 10 Gbit/s
    /// each, defined in 802.3ba (`ETHER_MEDIA_40GBASE_KR4`).
    pub const ETHER_MEDIA_40GBASE_KR4: Self = Self(50);
    /// 40 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing four 10 Gbit/s signals by wavelength, up to 10
    /// km, defined in 802.3ba (`ETHER_MEDIA_40GBASE_LR4`).
    pub const ETHER_MEDIA_40GBASE_LR4: Self = Self(51);
    /// 40 Gbit/s Ethernet over four pairs of multi-mode fiber, each at 10
    /// Gbit/s, with one fiber of each pair transmitting and the other
    /// receiving, generally with a QSFP+ connector, defined in 802.3ba
    /// (`ETHER_MEDIA_40GBASE_SR4`).
    pub const ETHER_MEDIA_40GBASE_SR4: Self = Self(52);
    /// 40 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing four 10 Gbit/s signals by wavelength, generally
    /// with a QSFP+ connector, up to 40 km, defined in 802.3bm
    /// (`ETHER_MEDIA_40GBASE_ER4`).
    pub const ETHER_MEDIA_40GBASE_ER4: Self = Self(53);
    /// 40 Gbit/s Ethernet over one pair of multi-mode fibers, one for each
    /// direction, multiplexing four 10 Gbit/s signals by wavelength, generally
    /// with a QSFP+ connector, defined by a multi-source agreement
    /// (`ETHER_MEDIA_40GBASE_LM4`).
    pub const ETHER_MEDIA_40GBASE_LM4: Self = Self(54);
    /// 40 Gbit/s Ethernet over a QSFP+ cable with built-in optical transceivers
    /// and four electrical lanes at 10 Gbit/s each
    /// (`ETHER_MEDIA_40GBASE_AOC4`).
    pub const ETHER_MEDIA_40GBASE_AOC4: Self = Self(55);
    /// 40 Gbit/s Ethernet over four copper lanes at 10 Gbit/s each with an
    /// active component that 40GBASE-CR4 lacks (`ETHER_MEDIA_40GBASE_ACC4`).
    pub const ETHER_MEDIA_40GBASE_ACC4: Self = Self(56);
    /// 40 Gbit/s signalling over four lanes between a MAC and a PHY or between
    /// chips, defined in 802.3ba (`ETHER_MEDIA_40G_XLAUI`).
    pub const ETHER_MEDIA_40G_XLAUI: Self = Self(57);
    /// 40 Gbit/s signalling over four lanes between a chip and a module,
    /// generally a QSFP+ device, defined in 802.3ba (`ETHER_MEDIA_40G_XLPPI`).
    pub const ETHER_MEDIA_40G_XLPPI: Self = Self(58);
    /// 50 Gbit/s Ethernet over a two lane copper backplane at 25 Gbit/s per
    /// lane, defined by the 25G and 50G Ethernet consortium and never an IEEE
    /// standard (`ETHER_MEDIA_50GBASE_KR2`).
    pub const ETHER_MEDIA_50GBASE_KR2: Self = Self(59);
    /// 50 Gbit/s Ethernet over two lanes of twinaxial copper cable at 25 Gbit/s
    /// each, generally with a QSFP+ connector, defined by the 25G and 50G
    /// Ethernet consortium (`ETHER_MEDIA_50GBASE_CR2`).
    pub const ETHER_MEDIA_50GBASE_CR2: Self = Self(60);
    /// 50 Gbit/s Ethernet over multi-mode fiber pairs at 25 Gbit/s each, with
    /// one fiber of each pair transmitting and the other receiving, generally
    /// with a QSFP+ connector, defined by the 25G and 50G Ethernet consortium
    /// (`ETHER_MEDIA_50GBASE_SR2`).
    pub const ETHER_MEDIA_50GBASE_SR2: Self = Self(61);
    /// 50 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing two 25 Gbit/s signals by wavelength, defined by
    /// the 25G and 50G Ethernet consortium (`ETHER_MEDIA_50GBASE_LR2`).
    pub const ETHER_MEDIA_50GBASE_LR2: Self = Self(62);
    /// 50 Gbit/s Ethernet, generally over a QSFP+ cable with built-in optical
    /// transceivers and two electrical lanes at 25 Gbit/s each
    /// (`ETHER_MEDIA_50GBASE_AOC2`).
    pub const ETHER_MEDIA_50GBASE_AOC2: Self = Self(63);
    /// 50 Gbit/s Ethernet over two twinaxial copper lanes at 25 Gbit/s each
    /// with an active component that 50GBASE-CR2 lacks
    /// (`ETHER_MEDIA_50GBASE_ACC2`).
    pub const ETHER_MEDIA_50GBASE_ACC2: Self = Self(64);
    /// 50 Gbit/s Ethernet over a single lane backplane, defined in 802.3cd
    /// (`ETHER_MEDIA_50GBASE_KR`).
    pub const ETHER_MEDIA_50GBASE_KR: Self = Self(65);
    /// 50 Gbit/s Ethernet over a single lane twinaxial copper cable, generally
    /// with an SFP56 interface, defined in 802.3cd (`ETHER_MEDIA_50GBASE_CR`).
    pub const ETHER_MEDIA_50GBASE_CR: Self = Self(66);
    /// 50 Gbit/s Ethernet over a pair of multi-mode fibers, one for each
    /// direction, defined in 802.3cd (`ETHER_MEDIA_50GBASE_SR`).
    pub const ETHER_MEDIA_50GBASE_SR: Self = Self(67);
    /// 50 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, up to 10 km, defined in 802.3cd (`ETHER_MEDIA_50GBASE_LR`).
    pub const ETHER_MEDIA_50GBASE_LR: Self = Self(68);
    /// 50 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, up to 2 km, defined in 802.3cd (`ETHER_MEDIA_50GBASE_FR`).
    pub const ETHER_MEDIA_50GBASE_FR: Self = Self(69);
    /// 50 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, up to 40 km, defined in 802.3cd (`ETHER_MEDIA_50GBASE_ER`).
    pub const ETHER_MEDIA_50GBASE_ER: Self = Self(70);
    /// 50 Gbit/s Ethernet over a short-range active optical cable, generally
    /// compatible with SFP56, whose electrical interface uses 25 Gbit/s PAM4
    /// signalling (`ETHER_MEDIA_50GBASE_AOC`).
    pub const ETHER_MEDIA_50GBASE_AOC: Self = Self(71);
    /// 50 Gbit/s Ethernet over a single lane twinaxial cable, generally
    /// compatible with SFP56, with an active component such as a retimer or
    /// redriver that 50GBASE-CR lacks (`ETHER_MEDIA_50GBASE_ACC`).
    pub const ETHER_MEDIA_50GBASE_ACC: Self = Self(72);
    /// 100 Gbit/s Ethernet over ten lanes of shielded twinaxial copper cable at
    /// 10 Gbit/s each, defined in 802.3ba (`ETHER_MEDIA_100GBASE_CR10`).
    pub const ETHER_MEDIA_100GBASE_CR10: Self = Self(73);
    /// 100 Gbit/s Ethernet over ten pairs of multi-mode fiber, each at 10
    /// Gbit/s, with one fiber of each pair transmitting and the other receiving
    /// (`ETHER_MEDIA_100GBASE_SR10`).
    pub const ETHER_MEDIA_100GBASE_SR10: Self = Self(74);
    /// 100 Gbit/s Ethernet over four pairs of multi-mode fiber, each at 25
    /// Gbit/s, with one fiber of each pair transmitting and the other
    /// receiving, defined in 802.3bm (`ETHER_MEDIA_100GBASE_SR4`).
    pub const ETHER_MEDIA_100GBASE_SR4: Self = Self(75);
    /// 100 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing four 25 Gbit/s signals by wavelength, generally
    /// with a QSFP28 connector, up to 10 km, defined in 802.3ba
    /// (`ETHER_MEDIA_100GBASE_LR4`).
    pub const ETHER_MEDIA_100GBASE_LR4: Self = Self(76);
    /// 100 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing four 25 Gbit/s signals by wavelength, generally
    /// with a QSFP28 connector, up to 40 km, defined in 802.3ba
    /// (`ETHER_MEDIA_100GBASE_ER4`).
    pub const ETHER_MEDIA_100GBASE_ER4: Self = Self(77);
    /// 100 Gbit/s Ethernet over a four lane copper backplane at 25 Gbit/s per
    /// lane, defined in 802.3bj (`ETHER_MEDIA_100GBASE_KR4`).
    pub const ETHER_MEDIA_100GBASE_KR4: Self = Self(78);
    /// 100 Gbit/s Ethernet over a four lane twinaxial copper cable at 25 Gbit/s
    /// per lane, generally with a QSFP28 connector, defined in 802.3bj
    /// (`ETHER_MEDIA_100GBASE_CR4`).
    pub const ETHER_MEDIA_100GBASE_CR4: Self = Self(79);
    /// 100 Gbit/s signalling for chip-to-chip and chip-to-module connections,
    /// defined in 802.3bm (`ETHER_MEDIA_100GBASE_CAUI4`).
    pub const ETHER_MEDIA_100GBASE_CAUI4: Self = Self(80);
    /// 100 Gbit/s Ethernet over an active optical cable with short-range
    /// optical transceivers and four electrical lanes at 25 Gbit/s each, most
    /// commonly with a QSFP28 connector (`ETHER_MEDIA_100GBASE_AOC4`).
    pub const ETHER_MEDIA_100GBASE_AOC4: Self = Self(81);
    /// 100 Gbit/s Ethernet over a four lane twinaxial copper cable with an
    /// active component such as a retimer or redriver that 100GBASE-CR4 lacks
    /// (`ETHER_MEDIA_100GBASE_ACC4`).
    pub const ETHER_MEDIA_100GBASE_ACC4: Self = Self(82);
    /// 100 Gbit/s Ethernet over a two lane copper backplane at 50 Gbit/s per
    /// lane, defined in 802.3cd (`ETHER_MEDIA_100GBASE_KR2`).
    pub const ETHER_MEDIA_100GBASE_KR2: Self = Self(83);
    /// 100 Gbit/s Ethernet over a two lane twinaxial copper cable at 50 Gbit/s
    /// per lane, defined in 802.3cd (`ETHER_MEDIA_100GBASE_CR2`).
    pub const ETHER_MEDIA_100GBASE_CR2: Self = Self(84);
    /// 100 Gbit/s Ethernet over two pairs of multi-mode fiber, each at 50
    /// Gbit/s, with one fiber of each pair transmitting and the other
    /// receiving, defined in 802.3cd (`ETHER_MEDIA_100GBASE_SR2`).
    pub const ETHER_MEDIA_100GBASE_SR2: Self = Self(85);
    /// 100 Gbit/s Ethernet over a single lane copper backplane, defined in
    /// 802.3ck (`ETHER_MEDIA_100GBASE_KR`).
    pub const ETHER_MEDIA_100GBASE_KR: Self = Self(86);
    /// 100 Gbit/s Ethernet over a single lane twinaxial copper cable, generally
    /// with an SFP112 connector, defined in 802.3ck
    /// (`ETHER_MEDIA_100GBASE_CR`).
    pub const ETHER_MEDIA_100GBASE_CR: Self = Self(87);
    /// 100 Gbit/s Ethernet over a pair of multi-mode fibers, one for each
    /// direction, up to 60 to 100 m depending on the fiber type, defined in
    /// 802.3db (`ETHER_MEDIA_100GBASE_SR`).
    pub const ETHER_MEDIA_100GBASE_SR: Self = Self(88);
    /// 100 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, meant for use with a parallel DR4 or DR8 interface, up to 500
    /// m, defined in 802.3cd (`ETHER_MEDIA_100GBASE_DR`).
    pub const ETHER_MEDIA_100GBASE_DR: Self = Self(89);
    /// 100 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, up to 10 km, defined in 802.3cu (`ETHER_MEDIA_100GBASE_LR`).
    pub const ETHER_MEDIA_100GBASE_LR: Self = Self(90);
    /// 100 Gbit/s Ethernet over a pair of single-mode fibers, one for each
    /// direction, up to 2 km, defined in 802.3cu (`ETHER_MEDIA_100GBASE_FR`).
    pub const ETHER_MEDIA_100GBASE_FR: Self = Self(91);
    /// 200 Gbit/s signalling over four lanes at 50 Gbit/s each for chip-to-chip
    /// and chip-to-module connections, defined in 802.3bs
    /// (`ETHER_MEDIA_200GAUI_4`).
    pub const ETHER_MEDIA_200GAUI_4: Self = Self(92);
    /// 200 Gbit/s Ethernet over a four lane passive twinaxial copper cable at
    /// 50 Gbit/s per lane, generally with a QSFP56 connector, defined in
    /// 802.3cd (`ETHER_MEDIA_200GBASE_CR4`).
    pub const ETHER_MEDIA_200GBASE_CR4: Self = Self(93);
    /// 200 Gbit/s Ethernet over four lanes of a copper backplane at 50 Gbit/s
    /// each, defined in 802.3cd (`ETHER_MEDIA_200GBASE_KR4`).
    pub const ETHER_MEDIA_200GBASE_KR4: Self = Self(94);
    /// 200 Gbit/s Ethernet over four pairs of multi-mode fiber, each at 50
    /// Gbit/s, with one fiber of each pair transmitting and the other
    /// receiving, defined in 802.3cd (`ETHER_MEDIA_200GBASE_SR4`).
    pub const ETHER_MEDIA_200GBASE_SR4: Self = Self(95);
    /// 200 Gbit/s Ethernet over four pairs of single-mode fiber at 50 Gbit/s
    /// each, with one fiber of each pair transmitting and the other receiving,
    /// defined in 802.3bs (`ETHER_MEDIA_200GBASE_DR4`).
    pub const ETHER_MEDIA_200GBASE_DR4: Self = Self(96);
    /// 200 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing four 50 Gbit/s signals by wavelength, generally
    /// with a QSFP56 connector, up to 2 km, defined in 802.3bs
    /// (`ETHER_MEDIA_200GBASE_FR4`).
    pub const ETHER_MEDIA_200GBASE_FR4: Self = Self(97);
    /// 200 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing four 50 Gbit/s signals by wavelength, generally
    /// with a QSFP56 connector, up to 10 km, defined in 802.3bs
    /// (`ETHER_MEDIA_200GBASE_LR4`).
    pub const ETHER_MEDIA_200GBASE_LR4: Self = Self(98);
    /// 200 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing four 50 Gbit/s signals by wavelength, generally
    /// with a QSFP56 connector, up to 40 km, defined in 802.3bs
    /// (`ETHER_MEDIA_200GBASE_ER4`).
    pub const ETHER_MEDIA_200GBASE_ER4: Self = Self(99);
    /// 200 Gbit/s signalling over two lanes at 100 Gbit/s each for chip-to-chip
    /// and chip-to-module connections, defined in 802.3ck
    /// (`ETHER_MEDIA_200GAUI_2`).
    pub const ETHER_MEDIA_200GAUI_2: Self = Self(100);
    /// 200 Gbit/s Ethernet over two lanes of a copper backplane at 100 Gbit/s
    /// each, defined in 802.3ck (`ETHER_MEDIA_200GBASE_KR2`).
    pub const ETHER_MEDIA_200GBASE_KR2: Self = Self(101);
    /// 200 Gbit/s Ethernet over a two lane passive twinaxial copper cable at
    /// 100 Gbit/s per lane, defined in 802.3ck (`ETHER_MEDIA_200GBASE_CR2`).
    pub const ETHER_MEDIA_200GBASE_CR2: Self = Self(102);
    /// 200 Gbit/s Ethernet over two pairs of multi-mode fiber, each at 100
    /// Gbit/s, with one fiber of each pair transmitting and the other
    /// receiving, defined in 802.3db (`ETHER_MEDIA_200GBASE_SR2`).
    pub const ETHER_MEDIA_200GBASE_SR2: Self = Self(103);
    /// 400 Gbit/s signalling over eight lanes at 50 Gbit/s each for
    /// chip-to-chip and chip-to-module connections, defined in 802.3bs
    /// (`ETHER_MEDIA_400GAUI_8`).
    pub const ETHER_MEDIA_400GAUI_8: Self = Self(104);
    /// 400 Gbit/s Ethernet over eight lanes of a copper backplane at 50 Gbit/s
    /// each, defined by the 25/50 Gigabit Ethernet Consortium
    /// (`ETHER_MEDIA_400GBASE_KR8`).
    pub const ETHER_MEDIA_400GBASE_KR8: Self = Self(105);
    /// 400 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing eight 50 Gbit/s signals by wavelength, generally
    /// with a QSFP-DD connector, up to 2 km, defined in 802.3bs. The manual
    /// page gives the speed as 200 Gbit/s, which is a slip
    /// (`ETHER_MEDIA_400GBASE_FR8`).
    pub const ETHER_MEDIA_400GBASE_FR8: Self = Self(106);
    /// 400 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing eight 50 Gbit/s signals by wavelength, generally
    /// with a QSFP-DD connector, up to 10 km, defined in 802.3bs. The manual
    /// page gives the speed as 200 Gbit/s, which is a slip
    /// (`ETHER_MEDIA_400GBASE_LR8`).
    pub const ETHER_MEDIA_400GBASE_LR8: Self = Self(107);
    /// 400 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing eight 50 Gbit/s signals by wavelength, generally
    /// with a QSFP-DD connector, up to 40 km, defined in 802.3cn. The manual
    /// page gives the speed as 200 Gbit/s, which is a slip
    /// (`ETHER_MEDIA_400GBASE_ER8`).
    pub const ETHER_MEDIA_400GBASE_ER8: Self = Self(108);
    /// 400 Gbit/s signalling over four lanes at 100 Gbit/s each for
    /// chip-to-chip and chip-to-module connections, defined in 802.3ck
    /// (`ETHER_MEDIA_400GAUI_4`).
    pub const ETHER_MEDIA_400GAUI_4: Self = Self(109);
    /// 400 Gbit/s Ethernet over four lanes of a copper backplane at 100 Gbit/s
    /// each, defined in 802.3ck (`ETHER_MEDIA_400GBASE_KR4`).
    pub const ETHER_MEDIA_400GBASE_KR4: Self = Self(110);
    /// 400 Gbit/s Ethernet over a four lane passive twinaxial copper cable at
    /// 100 Gbit/s per lane, generally with a QSFP112 connector, defined in
    /// 802.3ck. The manual page describes it as a 200 Gbit/s two lane cable,
    /// which is a slip (`ETHER_MEDIA_400GBASE_CR4`).
    pub const ETHER_MEDIA_400GBASE_CR4: Self = Self(111);
    /// 400 Gbit/s Ethernet over four pairs of multi-mode fiber, each at 100
    /// Gbit/s, with one fiber of each pair transmitting and the other
    /// receiving, defined in 802.3db (`ETHER_MEDIA_400GBASE_SR4`).
    pub const ETHER_MEDIA_400GBASE_SR4: Self = Self(112);
    /// 400 Gbit/s Ethernet over four pairs of single-mode fiber at 100 Gbit/s
    /// each, with one fiber of each pair transmitting and the other receiving,
    /// up to 500 m, defined in 802.3bs (`ETHER_MEDIA_400GBASE_DR4`).
    pub const ETHER_MEDIA_400GBASE_DR4: Self = Self(113);
    /// 400 Gbit/s Ethernet over one pair of single-mode fibers, one for each
    /// direction, multiplexing four 100 Gbit/s signals by wavelength, generally
    /// with a QSFP112 connector, up to 2 km, defined in 802.3cu
    /// (`ETHER_MEDIA_400GBASE_FR4`).
    pub const ETHER_MEDIA_400GBASE_FR4: Self = Self(114);
}

const _: () = assert!(core::mem::size_of::<mac_ether_media_t>() == 4);

/// The property can be read. It is one of the permission bits passed to
/// `mac_prop_info_set_perm(9F)` (`MAC_PROP_PERM_READ`).
pub const MAC_PROP_PERM_READ: u8 = 0x01;
/// The property can be written. It is one of the permission bits passed to
/// `mac_prop_info_set_perm(9F)`, and it rarely makes sense without
/// [`MAC_PROP_PERM_READ`] (`MAC_PROP_PERM_WRITE`).
pub const MAC_PROP_PERM_WRITE: u8 = 0x10;
/// The property can be read and written, the combination of
/// [`MAC_PROP_PERM_READ`] and [`MAC_PROP_PERM_WRITE`] (`MAC_PROP_PERM_RW`).
pub const MAC_PROP_PERM_RW: u8 = MAC_PROP_PERM_READ | MAC_PROP_PERM_WRITE;

unsafe extern "C" {
    /// `mac_prop_info_set_default_fec(9F)`. illumos DDI specific.
    ///
    /// Records `fec` as the default value of the property described by
    /// `hdl`, for properties of type [`link_fec_t`] such as
    /// `MAC_PROP_EN_FEC_CAP` and `MAC_PROP_ADV_FEC_CAP`. The default is the
    /// value a property has after `mac_register(9F)`. A later call replaces
    /// the value of an earlier one. The function has no return value; the
    /// framework makes sure there is room for the value and propagates any
    /// error itself.
    ///
    /// It is generally called on the handle passed to `mc_propinfo(9E)`, and
    /// works in user and kernel context. A provider may call it while holding
    /// its own locks.
    pub fn mac_prop_info_set_default_fec(hdl: mac_prop_info_handle_t, fec: link_fec_t);

    /// `mac_prop_info_set_default_link_flowctrl(9F)`. illumos DDI specific.
    ///
    /// Records `fctl` as the default value of the flow control property
    /// described by `hdl`, which is `MAC_PROP_FLOWCTRL`. A later call replaces
    /// the value of an earlier one. The function has no return value.
    ///
    /// It is generally called on the handle passed to `mc_propinfo(9E)`, and
    /// works in user and kernel context. A provider may call it while holding
    /// its own locks.
    pub fn mac_prop_info_set_default_link_flowctrl(hdl: mac_prop_info_handle_t, fctl: link_flowctrl_t);

    /// `mac_prop_info_set_default_str(9F)`. illumos DDI specific.
    ///
    /// Records the NUL-terminated string `str` as the default value of the
    /// string property described by `hdl`, which is how private properties
    /// usually report their defaults. For portability the string should
    /// contain only alphanumeric ASCII characters. The string is copied, and
    /// a later call replaces the value of an earlier one. If the string does
    /// not fit, the framework reports the error itself. The function has no
    /// return value.
    ///
    /// It is generally called on the handle passed to `mc_propinfo(9E)`, and
    /// works in user and kernel context. A provider may call it while holding
    /// its own locks.
    pub fn mac_prop_info_set_default_str(hdl: mac_prop_info_handle_t, str: *const c_char);

    /// `mac_prop_info_set_default_uint8(9F)`. illumos DDI specific.
    ///
    /// Records `u8` as the default value of the 8-bit unsigned property
    /// described by `hdl`, for example one of the speed properties. A later
    /// call replaces the value of an earlier one. The function has no return
    /// value.
    ///
    /// It is generally called on the handle passed to `mc_propinfo(9E)`, and
    /// works in user and kernel context. A provider may call it while holding
    /// its own locks.
    pub fn mac_prop_info_set_default_uint8(hdl: mac_prop_info_handle_t, u8: u8);

    /// `mac_prop_info_set_default_uint32(9F)`. illumos DDI specific.
    ///
    /// Records `u32` as the default value of the 32-bit unsigned property
    /// described by `hdl`, for example `MAC_PROP_MTU`. A later call replaces
    /// the value of an earlier one. The function has no return value.
    ///
    /// It is generally called on the handle passed to `mc_propinfo(9E)`, and
    /// works in user and kernel context. A provider may call it while holding
    /// its own locks. The function has a manual page but is not listed in the
    /// `mac` mapfile.
    pub fn mac_prop_info_set_default_uint32(hdl: mac_prop_info_handle_t, u32: u32);

    /// `mac_prop_info_set_default_uint64(9F)`. illumos DDI specific.
    ///
    /// Records `u64` as the default value of the 64-bit unsigned property
    /// described by `hdl`. The manual page names this function without
    /// describing it separately, and documents a 16-bit variant in its place
    /// that the kernel does not provide. A later call replaces the value of
    /// an earlier one. The function has no return value.
    ///
    /// It is generally called on the handle passed to `mc_propinfo(9E)`, and
    /// works in user and kernel context. A provider may call it while holding
    /// its own locks. The function is not listed in the `mac` mapfile.
    pub fn mac_prop_info_set_default_uint64(hdl: mac_prop_info_handle_t, u64: u64);

    /// `mac_prop_info_set_perm(9F)`. illumos DDI specific.
    ///
    /// Records the permissions of the property described by `hdl`. The
    /// `perm` argument is [`MAC_PROP_PERM_READ`] or [`MAC_PROP_PERM_RW`] in
    /// practice. The permissions express what the provider and device
    /// support, not the privileges of the user, and should not change for a
    /// given device. A later call replaces the value of an earlier one. The
    /// function has no return value.
    ///
    /// It is generally called on the handle passed to `mc_propinfo(9E)`, and
    /// works in user and kernel context. A provider may call it while holding
    /// its own locks.
    pub fn mac_prop_info_set_perm(hdl: mac_prop_info_handle_t, perm: u8);

    /// `mac_prop_info_set_range_uint32(9F)`. illumos DDI specific.
    ///
    /// Adds the range from `low` to `high`, generally inclusive, to the valid
    /// values of the integer property described by `hdl`. Each call adds
    /// another range, so disjoint ranges can be described by calling it
    /// several times. A device that supports jumbo frames up to 9600 bytes,
    /// for example, describes `MAC_PROP_MTU` with the range 1500 to 9600. The
    /// function has no return value; if there is no room for another range,
    /// the framework reports the error itself.
    ///
    /// It is generally called on the handle passed to `mc_propinfo(9E)`, and
    /// works in user and kernel context. A provider may call it while holding
    /// its own locks.
    pub fn mac_prop_info_set_range_uint32(hdl: mac_prop_info_handle_t, low: u32, high: u32);
}
