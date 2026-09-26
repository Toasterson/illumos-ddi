//! The `gld_mac_info(9S)` structure from `sys/gld.h`.
//!
//! The structure is the main data interface between a GLDv2 driver and GLD.
//! The driver allocates it with `gld_mac_alloc(9F)`, sets the members that
//! the manual page documents, and passes it to `gld_register(9F)`. The
//! header marks each of those members SET BY DRIVER and all others GLD
//! PRIVATE. The private members are declared here so that the layout
//! matches the kernel's, but a driver must neither read nor write them. The
//! manual adds that drivers can make no assumptions about the length of the
//! structure, which is why it is only ever allocated by GLD.

use core::mem::ManuallyDrop;

use ddi_sys::{c_char, c_uchar, caddr_t, ddi_iblock_cookie_t, dev_info_t, kmutex_t, krwlock_t};

use crate::entry::*;

/// The size of the private `reserved17` member, `GLD_PAD` in the header. It
/// is the space that 26 named kstats of 48 bytes each and 26 counters of 4
/// bytes each took up in the original layout, 1248 plus 104 bytes.
const GLD_PAD: usize = 26 * 48 + 26 * 4;

/// `gld_mac_info(9S)`. illumos architecture specific (illumos DDI).
///
/// The lock GLD keeps inside every `gld_mac_info(9S)` structure, the C union
/// `gld_lock_t`. It is private to GLD. The `reserved` arm keeps room for a
/// `kmutex_t`, and GLD uses the `gldl_rw_lock` arm as a reader-writer lock
/// around its calls into the driver. The union is declared only so that
/// the structure that contains it has the right layout.
#[repr(C)]
pub union gld_lock {
    /// Private to GLD. Space for a mutex, which is no longer used.
    pub reserved: ManuallyDrop<kmutex_t>,
    /// Private to GLD. The reader-writer lock GLD holds while it calls the
    /// driver's entry points.
    pub gldl_rw_lock: ManuallyDrop<krwlock_t>,
}

/// The typedef name of [`gld_lock`].
pub type gld_lock_t = gld_lock;

/// `gld_mac_info(9S)`. illumos architecture specific (illumos DDI).
///
/// The per-instance structure through which a GLDv2 driver describes its
/// device to GLD, the C `struct gld_mac_info` or `gld_mac_info_t`. It holds
/// the addresses, packet sizes and media type of the device, the pointers
/// to the driver's `gld(9E)` entry points, and a member for the driver's own
/// per-instance data.
///
/// The structure must be allocated with `gld_mac_alloc(9F)` and freed with
/// `gld_mac_free(9F)`. The driver must set the entry points and the other
/// members marked below as set by the driver before it calls
/// `gld_register(9F)`, and should not change them afterwards, because GLD
/// may use or cache their values at registration. `gldm_private` is the
/// exception and belongs to the driver at all times. The members that the
/// manual does not document are private to GLD and must be neither set nor
/// read by the driver.
#[repr(C)]
pub struct gld_mac_info {
    /// Private to GLD. The next structure on GLD's list.
    pub gldm_next: *mut gld_mac_info,
    /// Private to GLD. The previous structure on GLD's list.
    pub gldm_prev: *mut gld_mac_info,
    /// Private to GLD.
    pub reserved1: caddr_t,
    /// Private to GLD.
    pub reserved2: caddr_t,
    /// Private to GLD. The version of the driver interface.
    pub gldm_driver_version: u16,
    /// Private to GLD. The version of GLD.
    pub gldm_GLD_version: u16,
    /// Private to GLD. GLD's flags for this instance.
    pub gldm_GLD_flags: u16,
    /// Private to GLD. GLD's options for this instance, from the `GLDOPT_*`
    /// values in the private header `sys/gldpriv.h`.
    pub gldm_options: u16,
    /// Set by the driver. The `dev_info` node of this device.
    pub gldm_devinfo: dev_info_t,
    /// Set by the driver. Points to `gldm_addrlen` bytes holding the network
    /// physical address the vendor gave the device. The driver allocates the
    /// space and fills it in with the address read from the device.
    pub gldm_vendor_addr: *mut c_uchar,
    /// Set by the driver. Points to `gldm_addrlen` bytes holding the
    /// broadcast address used for transmission. The driver allocates the
    /// space and fills it in, normally with 0xFF-FF-FF-FF-FF-FF for Ethernet,
    /// Token Ring and FDDI.
    pub gldm_broadcast_addr: *mut c_uchar,
    /// Private to GLD. The lock GLD holds around its calls into the driver.
    pub gldm_lock: gld_lock_t,
    /// Set by the driver. The interrupt block cookie of the device's receive
    /// interrupt, from which `gld_recv(9F)` is called, as returned by
    /// `ddi_get_iblock_cookie(9F)`, `ddi_add_intr(9F)`,
    /// `ddi_get_soft_iblock_cookie(9F)` or `ddi_add_softintr(9F)`.
    pub gldm_cookie: ddi_iblock_cookie_t,
    /// Set by the driver. The number of bytes the device can transmit beyond
    /// `gldm_maxpkt`. A device that accepts a payload of up to 1522 bytes
    /// with a `gldm_maxpkt` of 1500 sets it to 22. GLD reports the value in
    /// its answer to the `DLIOCMARGININFO` ioctl. The manual page gives the
    /// type as `int`, the header as `uint32_t`.
    pub gldm_margin: u32,
    /// Private to GLD.
    pub reserved4: u32,
    /// Set by the driver. The maximum service data unit, the largest packet
    /// without the MAC header that the device can transmit. For Ethernet it
    /// is 1500.
    pub gldm_maxpkt: u32,
    /// Set by the driver. The minimum service data unit, the smallest packet
    /// without the MAC header that the device will transmit. It can be 0 if
    /// the driver handles any padding that is needed.
    pub gldm_minpkt: u32,
    /// Set by the driver. A short description of the device, used to
    /// identify it in system messages.
    pub gldm_ident: *mut c_char,
    /// Set by the driver. The type of device the driver handles, one of
    /// [`DL_ETHER`](crate::DL_ETHER), [`DL_TPR`](crate::DL_TPR) or
    /// [`DL_FDDI`](crate::DL_FDDI). GLD only works properly if it is set
    /// correctly. Support for Token Ring and FDDI is obsolete.
    pub gldm_type: u32,
    /// Private to GLD.
    pub reserved5: u32,
    /// Set by the driver. The length in bytes of the physical addresses the
    /// device handles, 6 for Ethernet, Token Ring and FDDI.
    pub gldm_addrlen: u32,
    /// Set by the driver. The length in bytes of the service access point
    /// address. GLD-based drivers must set it to -2, which says that two
    /// byte SAP values are supported and that the SAP follows the physical
    /// address in a DLSAP address.
    pub gldm_saplen: i32,
    /// Private to GLD. `ETHERADDRL` bytes of address storage.
    pub reserved7: [c_uchar; 6],
    /// Private to GLD. `ETHERADDRL` bytes of address storage.
    pub reserved8: [c_uchar; 6],
    /// Private to GLD. `ETHERADDRL` bytes of address storage.
    pub reserved9: [c_uchar; 6],
    /// Set by the driver. The physical point of attachment number of this
    /// instance, normally the instance number from `ddi_get_instance(9F)`.
    /// The C type is `t_uscalar_t`, a `uint32_t`.
    pub gldm_ppa: u32,
    /// Private to GLD.
    pub reserved10: i32,
    /// Set by the driver. The capabilities of the device as a combination of
    /// the `GLD_CAP_*` bits. A device that can report the state of its media
    /// link sets [`GLD_CAP_LINKSTATE`](crate::GLD_CAP_LINKSTATE).
    pub gldm_capabilities: u32,
    /// Private to GLD. The last link state the driver reported with
    /// `gld_linkstate(9F)`.
    pub gldm_linkstate: i32,
    /// Private to GLD.
    pub reserved11: u32,
    /// Private to GLD.
    pub reserved12: caddr_t,
    /// Private to GLD.
    pub reserved13: i32,
    /// Private to GLD.
    pub reserved14: u32,
    /// Private to GLD.
    pub reserved15: i32,
    /// Private to GLD. GLD's own per-instance state.
    pub gldm_mac_pvt: caddr_t,
    /// Private to GLD.
    pub reserved16: caddr_t,
    /// Private to GLD. Padding that keeps the layout of the original
    /// structure, `GLD_PAD` bytes.
    pub reserved17: [c_char; GLD_PAD],
    /// Private to GLD.
    pub reserved18: caddr_t,
    /// Private to the driver and never used or modified by GLD. It
    /// conventionally points to a per-instance data structure that the
    /// driver defines and allocates.
    pub gldm_private: caddr_t,
    /// Set by the driver. See [`gldm_reset_t`].
    pub gldm_reset: Option<gldm_reset_t>,
    /// Set by the driver. See [`gldm_start_t`].
    pub gldm_start: Option<gldm_start_t>,
    /// Set by the driver. See [`gldm_stop_t`].
    pub gldm_stop: Option<gldm_stop_t>,
    /// Set by the driver. See [`gldm_set_mac_addr_t`].
    pub gldm_set_mac_addr: Option<gldm_set_mac_addr_t>,
    /// Set by the driver. See [`gldm_send_t`].
    pub gldm_send: Option<gldm_send_t>,
    /// Set by the driver. See [`gldm_set_promiscuous_t`].
    pub gldm_set_promiscuous: Option<gldm_set_promiscuous_t>,
    /// Set by the driver. See [`gldm_get_stats_t`].
    pub gldm_get_stats: Option<gldm_get_stats_t>,
    /// Set by the driver, and may be NULL. See [`gldm_ioctl_t`].
    pub gldm_ioctl: Option<gldm_ioctl_t>,
    /// Set by the driver. See [`gldm_set_multicast_t`].
    pub gldm_set_multicast: Option<gldm_set_multicast_t>,
    /// Set by the driver. See [`gldm_intr_t`].
    pub gldm_intr: Option<gldm_intr_t>,
    /// Undocumented, and should be left NULL. See [`gldm_mctl_t`].
    pub gldm_mctl: Option<gldm_mctl_t>,
    /// Undocumented, and should be left NULL. See [`gldm_send_tagged_t`].
    pub gldm_send_tagged: Option<gldm_send_tagged_t>,
}

/// The typedef name of [`gld_mac_info`], which the manual pages use.
pub type gld_mac_info_t = gld_mac_info;

// Layout on amd64 with LP64 and _KERNEL defined, as sys/gld.h declares it.
// gld_lock_t is a union of an 8 byte kmutex_t and an 8 byte krwlock_t, so it
// is 8 bytes with 8 byte alignment.
const _: () = assert!(core::mem::size_of::<gld_lock_t>() == 8);

// gld_mac_info. The four pointers gldm_next to reserved2 occupy 0 to 32, the
// four uint16_t gldm_driver_version to gldm_options 32 to 40, the three
// pointers gldm_devinfo, gldm_vendor_addr and gldm_broadcast_addr 40 to 64,
// gldm_lock 8 at 64, gldm_cookie 8 at 72, gldm_margin, reserved4,
// gldm_maxpkt and gldm_minpkt 4 each from 80 to 96, gldm_ident 8 at 96,
// gldm_type, reserved5, gldm_addrlen and gldm_saplen 4 each from 104 to 120,
// reserved7, reserved8 and reserved9 6 bytes each from 120 to 138 and 2
// bytes of padding, gldm_ppa, reserved10, gldm_capabilities, gldm_linkstate
// and reserved11 4 each from 140 to 160, reserved12 8 at 160, reserved13,
// reserved14 and reserved15 4 each from 168 to 180 and 4 bytes of padding,
// gldm_mac_pvt 8 at 184, reserved16 8 at 192, reserved17 GLD_PAD, which is
// 26 times 48 plus 26 times 4 or 1352 bytes, from 200 to 1552, reserved18 8
// at 1552, gldm_private 8 at 1560, and the twelve function pointers
// gldm_reset to gldm_send_tagged from 1568 to 1664; total 1664. Checked with
// clang against the gate headers.
const _: () = assert!(core::mem::size_of::<gld_mac_info>() == 1664);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_next) == 0);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_driver_version) == 32);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_options) == 38);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_devinfo) == 40);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_lock) == 64);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_cookie) == 72);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_margin) == 80);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_ident) == 96);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_saplen) == 116);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, reserved9) == 132);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_ppa) == 140);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_capabilities) == 148);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, reserved12) == 160);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, reserved15) == 176);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_mac_pvt) == 184);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, reserved17) == 200);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, reserved18) == 1552);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_private) == 1560);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_reset) == 1568);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_send) == 1600);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_intr) == 1640);
const _: () = assert!(core::mem::offset_of!(gld_mac_info, gldm_send_tagged) == 1656);
