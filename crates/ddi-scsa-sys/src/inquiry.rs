//! SCSI inquiry data from `sys/scsi/generic/inquiry.h`.
//!
//! `scsi_probe(9F)` sends an INQUIRY command to a logical unit and stores
//! the answer in a [`scsi_inquiry`] that `sd_inq` of the `scsi_device(9S)`
//! points to. The first byte combines the peripheral qualifier, one of the
//! `DPQ_*` values, with the peripheral device type, one of the `DTYPE_*`
//! values, and is what a target driver's `probe(9E)` usually examines.
//!
//! The header's `DTYPE_ASCII` initialiser, the `TPGS_FAILOVER_*` values,
//! the vital product data structures `vpd_hdr` and `vpd_desc`, and the
//! `PM_CAPABLE_*` flags have no manual page and are not bound. Neither are
//! `SUN_INQSIZE` and the `INQUIRY_*` property names of
//! `sys/scsi/impl/inquiry.h`; the first is the size of [`scsi_inquiry`].

use ddi_sys::{c_char, c_uchar};

/// `scsi_inquiry(9S)`. illumos DDI specific (illumos DDI).
///
/// A version descriptor of the inquiry data, the C `struct inq_vd` nested in
/// [`scsi_inquiry`]. The eight descriptors in bytes 58 to 73 each name a standard that the
/// device claims to conform to, as a big endian 16-bit code.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct inq_vd {
    /// The most significant byte of the version descriptor.
    pub inq_vd_msb: c_uchar,
    /// The least significant byte of the version descriptor.
    pub inq_vd_lsb: c_uchar,
}

const _: () = assert!(core::mem::size_of::<inq_vd>() == 2);
const _: () = assert!(core::mem::offset_of!(inq_vd, inq_vd_lsb) == 1);

/// `scsi_inquiry(9S)`. illumos DDI specific (illumos DDI).
///
/// The standard inquiry data of a SCSI device. It consists of 36 required
/// bytes followed by a variable number of vendor specific parameters, and
/// the header pads it to 132 bytes so that some vendor specific data is
/// reachable through `sd_inq`. `scsi_probe(9F)` allocates and fills it in.
///
/// The C declaration uses bit fields in several bytes. Rust has no bit
/// fields, so those bytes are declared whole and named after their offset,
/// and their documentation names the bits, counted from the least
/// significant with `_BIT_FIELDS_LTOH`. The manual page lists the members in
/// the opposite order within each byte. It marks `inq_acc`, `inq_3pc` and
/// `inq_protect` as unstable, and the header marks several other bits as
/// deprecated by later SCSI standards.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_inquiry {
    /// The peripheral qualifier in bits 5 to 7, one of the `DPQ_*` values,
    /// and the peripheral device type in bits 0 to 4, one of the `DTYPE_*`
    /// values. [`DTYPE_MASK`] and [`DPQ_MASK`] separate them.
    pub inq_dtype: c_uchar,
    /// Bits 0 to 6 are `inq_qual`, the device type qualifier, and bit 7 is
    /// `inq_rmb`, which is set when the medium is removable.
    pub inq_byte1: c_uchar,
    /// Bits 0 to 2 are `inq_ansi`, the ANSI version, bits 3 to 5 are
    /// `inq_ecma`, the ECMA version, and bits 6 and 7 are `inq_iso`, the ISO
    /// version.
    pub inq_byte2: c_uchar,
    /// Bits 0 to 3 are `inq_rdf`, the response data format, one of the
    /// `RDF_*` values. Bit 4 is `inq_hisup`, set when the target uses the
    /// hierarchical addressing model for its logical units, bit 5 is
    /// `inq_normaca`, set when the device supports setting the NACA bit in
    /// a command descriptor block, bit 6 is the deprecated `inq_trmiop`, set
    /// when the device supports the TERMINATE I/O PROCESS message, and bit
    /// 7 is the deprecated `inq_aenc`, set when the device supports
    /// asynchronous event notification.
    pub inq_byte3: c_uchar,
    /// The additional length, the number of bytes of parameters that follow.
    pub inq_len: c_uchar,
    /// Bit 0 is `inq_protect`, set when the logical unit supports protection
    /// information. Bits 1 and 2 are the reserved `inq_5_1` and `inq_5_2`.
    /// Bit 3 is `inq_3pc`, set when the device supports third party copy
    /// commands. Bits 4 and 5 are `inq_tpgs`, which report implicit and
    /// explicit asymmetric logical unit access; the manual page calls them
    /// `inq_tpgsi` and `inq_tpgse`. Bit 6 is `inq_acc`, set when the logical
    /// unit contains an access controls coordinator, and bit 7 is
    /// `inq_sccs`, set when the device contains an embedded storage array
    /// controller.
    pub inq_byte5: c_uchar,
    /// Bit 0 is `inq_addr16`, set when a parallel SCSI device supports 16-bit
    /// wide addresses, and bit 1 is the deprecated `inq_addr32`. Bit 2 is the
    /// deprecated `inq_ackqreqq`. Bit 3 is the deprecated `inq_mchngr`, set
    /// when the device is embedded in or attached to a medium changer. Bit 4
    /// is `inq_dualp`, set when the device has two or more ports. Bit 5 is
    /// the deprecated `inq_port`. Bit 6 is `inq_encserv`, set when the device
    /// contains an embedded enclosure services component, and bit 7 is the
    /// deprecated `inq_bque`, which reports basic queueing.
    pub inq_byte6: c_uchar,
    /// Bit 0 is the deprecated `inq_sftre`, set when the device responds to a
    /// reset condition with the soft reset alternative. Bit 1 is
    /// `inq_cmdque`, set when the device supports tagged command queueing.
    /// Bit 2 is the deprecated `inq_trandis`. Bit 3 is the deprecated
    /// `inq_linked`, set when the device supports linked commands. Bit 4 is
    /// `inq_sync`, set when a parallel SCSI device supports synchronous
    /// transfers, and bit 5 is `inq_wbus16`, set when it supports 16-bit wide
    /// data transfers. Bits 6 and 7 are the deprecated `inq_wbus32` and
    /// `inq_reladdr`.
    pub inq_byte7: c_uchar,
    /// Eight bytes of ASCII data identifying the vendor of the product.
    pub inq_vid: [c_char; 8],
    /// Sixteen bytes of ASCII data identifying the product, as defined by the
    /// vendor.
    pub inq_pid: [c_char; 16],
    /// Four bytes of ASCII data giving the revision level, as defined by the
    /// vendor.
    pub inq_revision: [c_char; 4],
    /// Bytes 36 to 47, reserved by the standard. On hard disks qualified by
    /// Sun they hold a manufacturing date code and serial number.
    pub inq_serial: [c_char; 12],
    /// Reserved byte 48.
    pub __inq_48: c_uchar,
    /// Reserved byte 49.
    pub __inq_49: c_uchar,
    /// Reserved byte 50.
    pub __inq_50: c_uchar,
    /// Reserved byte 51.
    pub __inq_51: c_uchar,
    /// Reserved byte 52.
    pub __inq_52: c_uchar,
    /// Reserved byte 53.
    pub __inq_53: c_uchar,
    /// Reserved byte 54.
    pub __inq_54: c_uchar,
    /// Reserved byte 55.
    pub __inq_55: c_uchar,
    /// Byte 56, whose meaning is specific to SPI-3 and which is reserved for
    /// other protocols. Bit 0 is `inq_ius`, set when the device supports
    /// information unit transfers, bit 1 is `inq_qas`, set when the target
    /// port supports quick arbitration and selection, bits 2 and 3 are
    /// `inq_clk`, the clocking of the target port, and bits 4 to 7 are
    /// reserved.
    pub inq_byte56: c_uchar,
    /// Reserved byte 57.
    pub __inq_57: c_uchar,
    /// The eight version descriptors in bytes 58 to 73.
    pub inq_vd: [inq_vd; 8],
    /// Bytes 74 to 131. Bytes 74 to 95 are reserved and the rest are vendor
    /// specific parameters.
    pub __inq_74_127: [c_uchar; 58],
}

// Layout: bytes only, so no padding. inq_dtype at 0, the bit field bytes at
// 1, 2 and 3, inq_len at 4, bit field bytes at 5, 6 and 7, inq_vid 8 at 8,
// inq_pid 16 at 16, inq_revision 4 at 32, inq_serial 12 at 36, eight reserved
// bytes at 48 to 55, inq_byte56 at 56, __inq_57 at 57, inq_vd 16 at 58,
// __inq_74_127 58 at 74; total 132.
const _: () = assert!(core::mem::size_of::<scsi_inquiry>() == 132);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, inq_dtype) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, inq_len) == 4);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, inq_byte7) == 7);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, inq_vid) == 8);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, inq_pid) == 16);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, inq_revision) == 32);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, inq_serial) == 36);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, __inq_48) == 48);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, inq_byte56) == 56);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, inq_vd) == 58);
const _: () = assert!(core::mem::offset_of!(scsi_inquiry, __inq_74_127) == 74);

// --- peripheral device types ---------------------------------------------

/// A direct access device, such as a magnetic disk (`DTYPE_DIRECT`).
pub const DTYPE_DIRECT: c_uchar = 0x00;
/// A sequential access device, such as a magnetic tape
/// (`DTYPE_SEQUENTIAL`).
pub const DTYPE_SEQUENTIAL: c_uchar = 0x01;
/// A printer device (`DTYPE_PRINTER`).
pub const DTYPE_PRINTER: c_uchar = 0x02;
/// A processor device (`DTYPE_PROCESSOR`).
pub const DTYPE_PROCESSOR: c_uchar = 0x03;
/// A write once device, such as some optical disks (`DTYPE_WORM`).
pub const DTYPE_WORM: c_uchar = 0x04;
/// A read only direct access device, such as a CD-ROM (`DTYPE_RODIRECT`).
pub const DTYPE_RODIRECT: c_uchar = 0x05;
/// A scanner device, now obsolete in the standard (`DTYPE_SCANNER`).
pub const DTYPE_SCANNER: c_uchar = 0x06;
/// An optical memory device, such as some optical disks (`DTYPE_OPTICAL`).
pub const DTYPE_OPTICAL: c_uchar = 0x07;
/// A medium changer device, such as a jukebox (`DTYPE_CHANGER`).
pub const DTYPE_CHANGER: c_uchar = 0x08;
/// A communications device, now obsolete in the standard (`DTYPE_COMM`).
pub const DTYPE_COMM: c_uchar = 0x09;
/// An array controller device, such as a RAID controller
/// (`DTYPE_ARRAY_CTRL`).
pub const DTYPE_ARRAY_CTRL: c_uchar = 0x0C;
/// An enclosure services device (`DTYPE_ESI`).
pub const DTYPE_ESI: c_uchar = 0x0D;
/// A simplified direct access device (`DTYPE_RBC`).
pub const DTYPE_RBC: c_uchar = 0x0E;
/// An optical card reader or writer (`DTYPE_OCRW`).
pub const DTYPE_OCRW: c_uchar = 0x0F;
/// A bridge controller. The manual page calls it `DTYPE_BRIDGE`, which the
/// header does not define (`DTYPE_BCC`).
pub const DTYPE_BCC: c_uchar = 0x10;
/// An object based storage device (`DTYPE_OSD`).
pub const DTYPE_OSD: c_uchar = 0x11;
/// An automation drive interface. The header defines this type but the
/// manual page does not list it (`DTYPE_ADC`).
pub const DTYPE_ADC: c_uchar = 0x12;
/// A well known logical unit. The header defines this type but the manual
/// page does not list it (`DTYPE_WELLKNOWN`).
pub const DTYPE_WELLKNOWN: c_uchar = 0x1E;
/// An unknown device type, or no device (`DTYPE_UNKNOWN`).
pub const DTYPE_UNKNOWN: c_uchar = 0x1F;
/// The mask that isolates the peripheral device type in `inq_dtype`
/// (`DTYPE_MASK`).
pub const DTYPE_MASK: c_uchar = 0x1F;

// --- peripheral qualifiers -----------------------------------------------

/// The mask that isolates the peripheral qualifier in `inq_dtype`
/// (`DPQ_MASK`).
pub const DPQ_MASK: c_uchar = 0x60;
/// A device of the given type is connected to this logical unit, or the
/// target cannot tell whether one is. It does not imply that the device is
/// ready for access (`DPQ_POSSIBLE`).
pub const DPQ_POSSIBLE: c_uchar = 0x00;
/// The target can support a device of the given type on this logical unit,
/// but none is connected (`DPQ_SUPPORTED`).
pub const DPQ_SUPPORTED: c_uchar = 0x20;
/// The target cannot support a device on this logical unit. The device type
/// is then [`DTYPE_UNKNOWN`] (`DPQ_NEVER`).
pub const DPQ_NEVER: c_uchar = 0x60;
/// The qualifier is vendor unique (`DPQ_VUNIQ`).
pub const DPQ_VUNIQ: c_uchar = 0x80;
/// The combination of [`DPQ_NEVER`] and [`DTYPE_UNKNOWN`], which says that
/// no device is present at this logical unit (`DTYPE_NOTPRESENT`).
pub const DTYPE_NOTPRESENT: c_uchar = DPQ_NEVER | DTYPE_UNKNOWN;

// --- response data formats -----------------------------------------------

/// The inquiry data makes no conformance claim, as for SCSI-1
/// (`RDF_LEVEL0`).
pub const RDF_LEVEL0: c_uchar = 0x00;
/// The inquiry data complies with the CCS pseudo specification, now
/// obsolete (`RDF_CCS`).
pub const RDF_CCS: c_uchar = 0x01;
/// The inquiry data complies with SCSI-2, now obsolete (`RDF_SCSI2`).
pub const RDF_SCSI2: c_uchar = 0x02;
/// The inquiry data complies with SPC, ANSI INCITS 301-1997
/// (`RDF_SCSI_SPC`).
pub const RDF_SCSI_SPC: c_uchar = 0x03;
/// The inquiry data complies with SPC-2, ANSI INCITS 351-2001
/// (`RDF_SCSI_SPC2`).
pub const RDF_SCSI_SPC2: c_uchar = 0x04;
/// The inquiry data complies with SPC-3, ANSI INCITS 408-2005
/// (`RDF_SCSI_SPC3`).
pub const RDF_SCSI_SPC3: c_uchar = 0x05;
/// The inquiry data complies with SPC-4 (`RDF_SCSI_SPC4`).
pub const RDF_SCSI_SPC4: c_uchar = 0x06;
