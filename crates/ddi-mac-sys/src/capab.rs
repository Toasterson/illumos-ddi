//! Capabilities, from `sys/mac_provider.h`, `sys/mac.h` and `sys/dlpi.h`.
//!
//! A capability is an optional hardware feature that a provider advertises
//! through its `mc_getcapab(9E)` entry point. The framework names the
//! capability with a [`mac_capab_t`] value and passes a pointer to data whose
//! type depends on the capability. The provider fills the data in and
//! returns `B_TRUE`, or returns `B_FALSE` for a capability it does not
//! support or does not know.
//!
//! `mac(9E)` describes two stable capabilities. `MAC_CAPAB_HCKSUM` takes a
//! `uint32_t` holding the `HCKSUM_*` flags declared here, which are defined
//! in `sys/dlpi.h`, and `MAC_CAPAB_LSO` takes a [`mac_capab_lso_t`]. It also
//! describes three evolving ones, whose manual pages carry their own
//! stability. `MAC_CAPAB_RINGS` is declared in [`crate::rings`], and
//! `MAC_CAPAB_TRANSCEIVER` and `MAC_CAPAB_LED` are declared here with their
//! structures and the two transceiver information functions.
//!
//! The header also defines private capabilities for the framework's own use,
//! namely `MAC_CAPAB_SHARES`, `MAC_CAPAB_MULTIFACTADDR`, `MAC_CAPAB_VNIC`,
//! `MAC_CAPAB_ANCHOR_VNIC`, `MAC_CAPAB_AGGR`, `MAC_CAPAB_NO_NATIVEVLAN`,
//! `MAC_CAPAB_NO_ZCOPY`, `MAC_CAPAB_LEGACY`, `MAC_CAPAB_VRRP` and
//! `MAC_CAPAB_OVERLAY`, together with the reserved values. No manual page
//! describes them, so they are not bound. A provider sees them only as
//! unknown capabilities, for which it returns `B_FALSE`.

use ddi_sys::{boolean_t, c_int, c_uint, c_void, off_t, size_t, uint_t};

use crate::types::mac_transceiver_info_t;

/// `mac(9E)`. illumos DDI specific.
///
/// The identifier of a capability, `mac_capab_t`, which the framework
/// passes to `mc_getcapab(9E)`. Each value is a distinct power of two. Only
/// the values that the manual describes are declared, and a provider must
/// answer `B_FALSE` for any other value it receives.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct mac_capab_t(pub c_uint);
impl mac_capab_t {
    /// Hardware checksum offload. The data is a `uint32_t` that the
    /// provider sets to the `HCKSUM_*` flags describing what it can compute
    /// on transmit (`MAC_CAPAB_HCKSUM`).
    pub const MAC_CAPAB_HCKSUM: Self = Self(0x0000_0001);
    /// Large send offload. The data is a [`mac_capab_lso_t`]
    /// (`MAC_CAPAB_LSO`).
    pub const MAC_CAPAB_LSO: Self = Self(0x0000_0008);
    /// Multiple rings and groups. The data is a
    /// [`mac_capab_rings_t`](crate::rings::mac_capab_rings_t), and the
    /// capability is described in `mac_capab_rings(9E)` (`MAC_CAPAB_RINGS`).
    pub const MAC_CAPAB_RINGS: Self = Self(0x0000_0010);
    /// Transceiver information. The data is a [`mac_capab_transceiver_t`],
    /// and the capability is described in `mac_capab_transceiver(9E)`
    /// (`MAC_CAPAB_TRANSCEIVER`).
    pub const MAC_CAPAB_TRANSCEIVER: Self = Self(0x0100_0000);
    /// Control of the device's LEDs. The data is a [`mac_capab_led_t`], and
    /// the capability is described in `mac_capab_led(9E)` (`MAC_CAPAB_LED`).
    pub const MAC_CAPAB_LED: Self = Self(0x0200_0000);
}

const _: () = assert!(core::mem::size_of::<mac_capab_t>() == 4);

/// The hardware can compute a partial one's complement checksum for TCP and
/// UDP over both IPv4 and IPv6, given the pseudo-header checksum that
/// `mac_hcksum_get(9F)` supplies. It does not imply support for other layer
/// 4 protocols or for the IPv4 header checksum. The flag is defined in
/// `sys/dlpi.h` (`HCKSUM_INET_PARTIAL`).
pub const HCKSUM_INET_PARTIAL: u32 = 0x02;
/// The hardware can compute the full layer 4 checksum of outgoing TCP and
/// UDP packets over IPv4 without a pseudo-header checksum. It does not imply
/// support for other layer 4 protocols or for the IPv4 header checksum. The
/// flag is defined in `sys/dlpi.h` (`HCKSUM_INET_FULL_V4`).
pub const HCKSUM_INET_FULL_V4: u32 = 0x04;
/// The hardware can compute the full layer 4 checksum of outgoing TCP and
/// UDP packets over IPv6 without a pseudo-header checksum. It does not imply
/// support for other layer 4 protocols. The flag is defined in `sys/dlpi.h`
/// (`HCKSUM_INET_FULL_V6`).
pub const HCKSUM_INET_FULL_V6: u32 = 0x08;
/// The hardware can compute the checksum of the IPv4 header itself. The flag
/// is defined in `sys/dlpi.h` (`HCKSUM_IPHDRCKSUM`).
pub const HCKSUM_IPHDRCKSUM: u32 = 0x10;

/// The hardware can segment TCP over IPv4, and the `lso_basic_tcp_ipv4`
/// member of [`mac_capab_lso_t`] is filled in (`LSO_TX_BASIC_TCP_IPV4`).
pub const LSO_TX_BASIC_TCP_IPV4: u32 = 0x01;
/// The hardware can segment TCP over IPv6 packets without extension
/// headers, and the `lso_basic_tcp_ipv6` member of [`mac_capab_lso_t`] is
/// filled in (`LSO_TX_BASIC_TCP_IPV6`).
pub const LSO_TX_BASIC_TCP_IPV6: u32 = 0x02;

/// `mac(9E)`. illumos DDI specific.
///
/// The limits of basic TCP segmentation offload over IPv4,
/// `lso_basic_tcp_ipv4_t`, which is the `lso_basic_tcp_ipv4` member of
/// [`mac_capab_lso_t`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct lso_basic_tcp_ipv4_t {
    /// The largest TCP payload, in bytes, that the hardware can segment in
    /// one operation. The C type is `t_uscalar_t`, a 32-bit unsigned integer
    /// under LP64.
    pub lso_max: u32,
}

const _: () = assert!(core::mem::size_of::<lso_basic_tcp_ipv4_t>() == 4);
const _: () = assert!(core::mem::offset_of!(lso_basic_tcp_ipv4_t, lso_max) == 0);

/// `mac(9E)`. illumos DDI specific.
///
/// The limits of basic TCP segmentation offload over IPv6,
/// `lso_basic_tcp_ipv6_t`, which is the `lso_basic_tcp_ipv6` member of
/// [`mac_capab_lso_t`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct lso_basic_tcp_ipv6_t {
    /// The largest TCP payload, in bytes, that the hardware can segment in
    /// one operation. The C type is `t_uscalar_t`, a 32-bit unsigned integer
    /// under LP64.
    pub lso_max: u32,
}

const _: () = assert!(core::mem::size_of::<lso_basic_tcp_ipv6_t>() == 4);
const _: () = assert!(core::mem::offset_of!(lso_basic_tcp_ipv6_t, lso_max) == 0);

/// `mac(9E)`. illumos DDI specific.
///
/// The data of the `MAC_CAPAB_LSO` capability, `mac_capab_lso_t`. The
/// provider sets `lso_flags` to the `LSO_TX_*` flags it supports and fills
/// in the member belonging to each flag it sets. The framework allocates
/// the structure, and new kinds of offload may only ever be appended to it,
/// so that a provider built against this layout keeps working when the
/// structure grows.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct mac_capab_lso_t {
    /// The bitwise inclusive OR of [`LSO_TX_BASIC_TCP_IPV4`] and
    /// [`LSO_TX_BASIC_TCP_IPV6`], saying which of the following members are
    /// valid. The C type is `t_uscalar_t`.
    pub lso_flags: u32,
    /// The limits for TCP over IPv4, valid when [`LSO_TX_BASIC_TCP_IPV4`] is
    /// set.
    pub lso_basic_tcp_ipv4: lso_basic_tcp_ipv4_t,
    /// The limits for TCP over IPv6, valid when [`LSO_TX_BASIC_TCP_IPV6`] is
    /// set.
    pub lso_basic_tcp_ipv6: lso_basic_tcp_ipv6_t,
}

// mac_capab_lso_t under LP64: three 32-bit members at 0, 4 and 8, with no
// pointers and no padding; total 12.
const _: () = assert!(core::mem::size_of::<mac_capab_lso_t>() == 12);
const _: () = assert!(core::mem::offset_of!(mac_capab_lso_t, lso_flags) == 0);
const _: () = assert!(core::mem::offset_of!(mac_capab_lso_t, lso_basic_tcp_ipv4) == 4);
const _: () = assert!(core::mem::offset_of!(mac_capab_lso_t, lso_basic_tcp_ipv6) == 8);

/// `mct_info(9E)`. Volatile - This interface is still evolving in illumos.
///
/// The signature of the `mct_info` member of [`mac_capab_transceiver_t`],
/// which describes one transceiver. The C header declares the member with
/// an inline function pointer type; the name of this alias is not a C name.
/// `driver` is the provider's private data, `id` the transceiver, counting
/// from zero to `mct_ntransceivers` minus one, and `infop` an opaque
/// structure that the provider fills in with
/// `mac_transceiver_info_set_present(9F)` and
/// `mac_transceiver_info_set_usable(9F)`.
///
/// The entry point is required when the capability is advertised. It returns
/// 0 on success and an error number otherwise, such as `EINVAL` for an
/// invalid `id`, `ENOTSUP` when this instance has no transceiver at all, or
/// `EIO` when reading the device failed. It is called from kernel context
/// and never from interrupt context.
pub type mct_info_t =
    unsafe extern "C" fn(driver: *mut c_void, id: uint_t, infop: *mut mac_transceiver_info_t) -> c_int;

/// `mct_read(9E)`. Volatile - This interface is still evolving in illumos.
///
/// The signature of the optional `mct_read` member of
/// [`mac_capab_transceiver_t`], which reads from a transceiver's i2c bus.
/// The C header declares the member with an inline function pointer type;
/// the name of this alias is not a C name. The provider reads up to `nbytes`
/// bytes from i2c page `page`, such as 0xa0 or 0xa2, of transceiver `id`,
/// starting at `offset` within the page, into `buf`, and stores the number
/// of bytes it read in `nread`. A short read is acceptable when the
/// transceiver does not allow reads of the requested region.
///
/// The entry point returns 0 on success and an error number otherwise, such
/// as `EINVAL` for an invalid transceiver, page or offset, or `EIO` when the
/// read failed. A provider whose hardware would be disrupted by such a
/// read, for example by a PHY reset, must not implement it. It is called
/// from kernel context and never from interrupt context, possibly at the
/// same time as normal I/O, so the provider should use separate locks.
pub type mct_read_t = unsafe extern "C" fn(
    driver: *mut c_void,
    id: uint_t,
    page: uint_t,
    buf: *mut c_void,
    nbytes: size_t,
    offset: off_t,
    nread: *mut size_t,
) -> c_int;

/// `mac_capab_transceiver(9E)`. Volatile - This interface is still evolving in illumos.
///
/// The data of the `MAC_CAPAB_TRANSCEIVER` capability,
/// `mac_capab_transceiver_t`, through which a provider with small form
/// factor pluggable transceivers, such as SFP, SFP28 or QSFP28 modules,
/// reports them to the system. The capability does not apply to ordinary
/// copper Ethernet PHYs.
#[repr(C)]
pub struct mac_capab_transceiver_t {
    /// Extension flags. The framework sets the extensions it knows, and the
    /// provider intersects them with those it supports. None are defined, so
    /// the provider sets it to 0.
    pub mct_flags: uint_t,
    /// The total number of transceivers the device can have, whether or not
    /// they are currently plugged in. It usually is one and must not change
    /// while the device is attached.
    pub mct_ntransceivers: uint_t,
    /// Describes one transceiver, see `mct_info(9E)`. Required.
    pub mct_info: Option<mct_info_t>,
    /// Reads from a transceiver's i2c bus, see `mct_read(9E)`. Optional.
    pub mct_read: Option<mct_read_t>,
}

// mac_capab_transceiver_t under LP64: two uint_t at 0 and 4, then two
// pointers at 8 and 16; total 24.
const _: () = assert!(core::mem::size_of::<mac_capab_transceiver_t>() == 24);
const _: () = assert!(core::mem::offset_of!(mac_capab_transceiver_t, mct_flags) == 0);
const _: () = assert!(core::mem::offset_of!(mac_capab_transceiver_t, mct_ntransceivers) == 4);
const _: () = assert!(core::mem::offset_of!(mac_capab_transceiver_t, mct_info) == 8);
const _: () = assert!(core::mem::offset_of!(mac_capab_transceiver_t, mct_read) == 16);

/// `mac_capab_led(9E)`. Evolving - This interface is still evolving.
///
/// A mode in which a provider can drive the LEDs of its device,
/// `mac_led_mode_t`. Each mode is a single bit, so that a provider can
/// report the modes it supports as a bitwise inclusive OR in `mcl_modes`,
/// while `mcl_set` always receives exactly one of them. The enumeration is
/// defined in `sys/mac.h`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct mac_led_mode_t(pub c_uint);
impl mac_led_mode_t {
    /// The device's own default behaviour, usually showing link status and
    /// activity (`MAC_LED_DEFAULT`).
    pub const MAC_LED_DEFAULT: Self = Self(1 << 0);
    /// All LEDs off (`MAC_LED_OFF`).
    pub const MAC_LED_OFF: Self = Self(1 << 1);
    /// An identification pattern, ideally a light blinking on and off at
    /// equal intervals, for example every 200 milliseconds, in a colour
    /// different from the normal link lighting if possible (`MAC_LED_IDENT`).
    pub const MAC_LED_IDENT: Self = Self(1 << 2);
    /// All LEDs on and solid (`MAC_LED_ON`).
    pub const MAC_LED_ON: Self = Self(1 << 3);
}

const _: () = assert!(core::mem::size_of::<mac_led_mode_t>() == 4);

/// `mcl_set(9E)`. Evolving - This interface is still evolving.
///
/// The signature of the `mcl_set` member of [`mac_capab_led_t`], which
/// changes how the provider drives its LEDs. The C header declares the
/// member with an inline function pointer type; the name of this alias is
/// not a C name. `driver` is the provider's private data, `mode` a single
/// mode that the provider should check it supports, and `flags` is reserved
/// and must be checked to be zero. On its first call the provider should
/// record the device's LED registers so that it can later restore the
/// default behaviour. Asking for the mode already in effect succeeds without
/// doing anything.
///
/// The entry point returns 0 once the mode is in effect and an error number
/// otherwise, such as `EINVAL` for non-zero `flags`, `ENOTSUP` for an
/// unsupported mode, or `EIO` when programming the device failed. The
/// framework never runs two calls at the same time, and calls it only from
/// user or kernel context, never from interrupt context.
pub type mcl_set_t = unsafe extern "C" fn(driver: *mut c_void, mode: mac_led_mode_t, flags: uint_t) -> c_int;

/// `mac_capab_led(9E)`. Evolving - This interface is still evolving.
///
/// The data of the `MAC_CAPAB_LED` capability, `mac_capab_led_t`, through
/// which a provider lets the system control the LEDs of its device, for
/// example so that an administrator can locate it. A provider that supports
/// no mode other than [`mac_led_mode_t::MAC_LED_DEFAULT`] should not
/// advertise the capability.
#[repr(C)]
pub struct mac_capab_led_t {
    /// Extension flags. The framework sets the extensions it knows, and the
    /// provider intersects them with those it supports. None are defined, so
    /// the provider sets it to 0.
    pub mcl_flags: uint_t,
    /// The bitwise inclusive OR of the modes the provider supports.
    pub mcl_modes: mac_led_mode_t,
    /// Changes the LED mode, see [`mcl_set_t`].
    pub mcl_set: Option<mcl_set_t>,
}

// mac_capab_led_t under LP64: mcl_flags at 0, the enumeration mcl_modes at
// 4, and the pointer mcl_set at 8; total 16.
const _: () = assert!(core::mem::size_of::<mac_capab_led_t>() == 16);
const _: () = assert!(core::mem::offset_of!(mac_capab_led_t, mcl_flags) == 0);
const _: () = assert!(core::mem::offset_of!(mac_capab_led_t, mcl_modes) == 4);
const _: () = assert!(core::mem::offset_of!(mac_capab_led_t, mcl_set) == 8);

unsafe extern "C" {
    /// `mac_transceiver_info_set_present(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Records in `infop` whether the transceiver being described is
    /// present, that is plugged into the device. A provider calls it from
    /// its `mct_info(9E)` entry point with `B_TRUE` or `B_FALSE`. A
    /// transceiver that is part of the device is always present. The
    /// function has no return value.
    ///
    /// It is called in kernel context while handling `mct_info(9E)`.
    pub fn mac_transceiver_info_set_present(infop: *mut mac_transceiver_info_t, present: boolean_t);

    /// `mac_transceiver_info_set_usable(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Records in `infop` whether the device can use the transceiver being
    /// described. A transceiver may be present but not usable, for example
    /// when the hardware and firmware support only certain transceivers. A
    /// provider calls it from its `mct_info(9E)` entry point with `B_TRUE`
    /// or `B_FALSE`, and must not call it for a transceiver that is not
    /// present. The function has no return value.
    ///
    /// It is called in kernel context while handling `mct_info(9E)`.
    pub fn mac_transceiver_info_set_usable(infop: *mut mac_transceiver_info_t, usable: boolean_t);
}
