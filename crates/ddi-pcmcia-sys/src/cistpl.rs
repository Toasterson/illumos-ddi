//! The structures that the tuple parsers fill in, from `sys/cis_handlers.h`
//! and `sys/cis.h`.
//!
//! Each `csx_Parse_CISTPL_*(9F)` function in [`crate::parse`] decodes one
//! kind of tuple into one of the structures declared here, and the generic
//! parser `csx_ParseTuple(9F)` fills in the union
//! [`cisparse_t`](crate::parse::cisparse_t) of all of them. The structures
//! hold the fields of the tuple widened to `uint32_t`, with arrays for
//! repeated parts and flag words saying which optional parts were present.
//!
//! Beside each structure are the constants that its manual page names for
//! its members. A few value sets are completed from the header where the
//! page leaves a value out, namely [`TPLBYTEORD_HIGH`], the `TPLFMTEDC_*`
//! error detection types, and the flags of [`cistpl_bar_t`] and
//! [`cistpl_longlink_cb_t`], which appear in the union of
//! `csx_ParseTuple(9F)` without a page of their own. The CIS interpreter
//! internals of the header, such as `cistpl_callout_t`, the `HANDTPL_*`
//! flags, the device speed and power description tables, the masks used
//! inside the `CISTPL_CFTABLE_ENTRY` handler and the unused
//! `cistpl_cftable_entry_stce_ev_t` and `cistpl_cftable_entry_stce_pd_t`,
//! are left out.
//!
//! C declares several members as unions or structures inside the parent
//! structure. Rust needs a name for each, so they are declared as separate
//! types named after the parent and the member, such as
//! [`cistpl_format_dev`] for the `dev` union of [`cistpl_format_t`].

use ddi_sys::{c_char, c_uchar, caddr_t};

use crate::tuple::CIS_MAX_TUPLE_DATA_LEN;

// --- CISTPL_DEVICE, CISTPL_DEVICE_A, CISTPL_DEVICE_OC, CISTPL_DEVICE_OA --

/// The number of devices a device information tuple can describe in a
/// [`cistpl_device_t`] (`CISTPL_DEVICE_MAX_DEVICES`).
pub const CISTPL_DEVICE_MAX_DEVICES: usize = 10;

/// `csx_Parse_CISTPL_DEVICE(9F)`. illumos DDI Specific (illumos DDI).
///
/// One device of a device information tuple, an element of
/// [`cistpl_device_t::devnode`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_device_node_t {
    /// Whether the device is writable, [`CISTPL_DEVICE_WPS`], and for the
    /// other conditions tuples the `CISTPL_DEVICE_OC_*` bits describing the
    /// Vcc at which the card can be operated and whether it uses MWAIT.
    pub flags: u32,
    /// The device speed as a device speed code, or
    /// [`CISTPL_DEVICE_SPEED_SIZE_IGNORE`] when the speed and size are to be
    /// ignored.
    pub speed: u32,
    /// The device speed in nanoseconds.
    pub nS_speed: u32,
    /// The device type, one of the `CISTPL_DEVICE_DTYPE_*` values. The C
    /// member is named `type`, which is a Rust keyword.
    pub type_: u32,
    /// The device size as a device size code, or
    /// [`CISTPL_DEVICE_SPEED_SIZE_IGNORE`] when it is to be ignored.
    pub size: u32,
    /// The device size in bytes.
    pub size_in_bytes: u32,
}

/// `csx_Parse_CISTPL_DEVICE(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_DEVICE`, `CISTPL_DEVICE_A`, `CISTPL_DEVICE_OC` or
/// `CISTPL_DEVICE_OA` tuple. These describe the speed, size, type and
/// address space layout of the devices of common or attribute memory, the
/// last two under a particular set of operating conditions.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_device_t {
    /// The number of devices found.
    pub num_devices: u32,
    /// The devices, of which the first
    /// [`num_devices`](Self::num_devices) are valid.
    pub devnode: [cistpl_device_node_t; CISTPL_DEVICE_MAX_DEVICES],
}

// Layout on amd64 with LP64. cistpl_device_node_t is six uint32_t at 0 to
// 20, 24 bytes. cistpl_device_t is num_devices 4 at 0 and devnode 10 * 24 at
// 4; total 244.
const _: () = assert!(core::mem::size_of::<cistpl_device_node_t>() == 24);
const _: () = assert!(core::mem::offset_of!(cistpl_device_node_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_device_node_t, size_in_bytes) == 20);
const _: () = assert!(core::mem::size_of::<cistpl_device_t>() == 244);
const _: () = assert!(core::mem::offset_of!(cistpl_device_t, num_devices) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_device_t, devnode) == 4);

/// The write protect switch bit of the device is set
/// (`CISTPL_DEVICE_WPS`).
pub const CISTPL_DEVICE_WPS: u32 = 0x0000_0001;
/// The device uses MWAIT. Only for the other conditions tuples
/// (`CISTPL_DEVICE_OC_MWAIT`).
pub const CISTPL_DEVICE_OC_MWAIT: u32 = 0x0001_0000;
/// The mask of the Vcc value. Only for the other conditions tuples
/// (`CISTPL_DEVICE_OC_Vcc_MASK`).
pub const CISTPL_DEVICE_OC_Vcc_MASK: u32 = 0x0006_0000;
/// Operation at 5.0 volts (`CISTPL_DEVICE_OC_Vcc5`).
pub const CISTPL_DEVICE_OC_Vcc5: u32 = 0x0000_0000;
/// Operation at 3.3 volts (`CISTPL_DEVICE_OC_Vcc33`).
pub const CISTPL_DEVICE_OC_Vcc33: u32 = 0x0002_0000;
/// Operation at X.X volts (`CISTPL_DEVICE_OC_VccXX`).
pub const CISTPL_DEVICE_OC_VccXX: u32 = 0x0004_0000;
/// Operation at Y.Y volts (`CISTPL_DEVICE_OC_VccYY`).
pub const CISTPL_DEVICE_OC_VccYY: u32 = 0x0006_0000;
/// The speed or size value meaning that both are to be ignored
/// (`CISTPL_DEVICE_SPEED_SIZE_IGNORE`).
pub const CISTPL_DEVICE_SPEED_SIZE_IGNORE: u32 = 0x0ff;

/// No device, a hole in the address space (`CISTPL_DEVICE_DTYPE_NULL`).
pub const CISTPL_DEVICE_DTYPE_NULL: u32 = 0x00;
/// Masked ROM (`CISTPL_DEVICE_DTYPE_ROM`).
pub const CISTPL_DEVICE_DTYPE_ROM: u32 = 0x01;
/// One time programmable ROM (`CISTPL_DEVICE_DTYPE_OTPROM`).
pub const CISTPL_DEVICE_DTYPE_OTPROM: u32 = 0x02;
/// UV EPROM (`CISTPL_DEVICE_DTYPE_EPROM`).
pub const CISTPL_DEVICE_DTYPE_EPROM: u32 = 0x03;
/// EEPROM (`CISTPL_DEVICE_DTYPE_EEPROM`).
pub const CISTPL_DEVICE_DTYPE_EEPROM: u32 = 0x04;
/// Flash memory (`CISTPL_DEVICE_DTYPE_FLASH`).
pub const CISTPL_DEVICE_DTYPE_FLASH: u32 = 0x05;
/// Static RAM (`CISTPL_DEVICE_DTYPE_SRAM`).
pub const CISTPL_DEVICE_DTYPE_SRAM: u32 = 0x06;
/// Dynamic RAM (`CISTPL_DEVICE_DTYPE_DRAM`).
pub const CISTPL_DEVICE_DTYPE_DRAM: u32 = 0x07;
/// A function specific memory address range
/// (`CISTPL_DEVICE_DTYPE_FUNCSPEC`).
pub const CISTPL_DEVICE_DTYPE_FUNCSPEC: u32 = 0x0d;
/// An extended type follows (`CISTPL_DEVICE_DTYPE_EXTEND`).
pub const CISTPL_DEVICE_DTYPE_EXTEND: u32 = 0x0e;

// --- CISTPL_CONFIG -------------------------------------------------------

/// The number of configuration register offsets a [`cistpl_config_t`] can
/// hold (`CISTPL_CONFIG_MAX_CONFIG_REGS`).
pub const CISTPL_CONFIG_MAX_CONFIG_REGS: usize = 128;

/// `csx_Parse_CISTPL_CONFIG(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_CONFIG` tuple, which describes the general
/// characteristics of a 16-bit card with I/O devices or a custom interface,
/// or of a card with unusual power requirements. It gives the location of
/// the configuration registers and the last valid configuration index. The
/// page notes that drivers should not use configurations beyond
/// [`last`](Self::last).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_config_t {
    /// Which configuration registers the card has, a combination of the
    /// `CONFIG_*_REG_PRESENT` bits of [`crate::config`].
    pub present: u32,
    /// The number of configuration registers present.
    pub nr: u32,
    /// The highest configuration register number present.
    pub hr: u32,
    /// The offset from the start of attribute memory of each configuration
    /// register. The entry of a register that is not present is undefined.
    pub regs: [u32; CISTPL_CONFIG_MAX_CONFIG_REGS],
    /// The offset from the start of attribute memory of the configuration
    /// register space.
    pub base: u32,
    /// The last valid configuration index of the card.
    pub last: u32,
}

// Layout on amd64 with LP64. present, nr and hr at 0, 4 and 8, regs 128 * 4
// at 12, base 4 at 524 and last 4 at 528; total 532.
const _: () = assert!(core::mem::size_of::<cistpl_config_t>() == 532);
const _: () = assert!(core::mem::offset_of!(cistpl_config_t, present) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_config_t, regs) == 12);
const _: () = assert!(core::mem::offset_of!(cistpl_config_t, base) == 524);
const _: () = assert!(core::mem::offset_of!(cistpl_config_t, last) == 528);

// --- CISTPL_VERS_1 and CISTPL_VERS_2 -------------------------------------

/// The number of product information strings a [`cistpl_vers_1_t`] can
/// hold (`CISTPL_VERS_1_MAX_PROD_STRINGS`).
pub const CISTPL_VERS_1_MAX_PROD_STRINGS: usize = 4;

/// `csx_Parse_CISTPL_VERS_1(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_VERS_1` tuple, which gives the level 1 version the card
/// complies with and the manufacturer's product information strings.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_vers_1_t {
    /// The major version number.
    pub major: u32,
    /// The minor version number.
    pub minor: u32,
    /// The number of product information strings.
    pub ns: u32,
    /// The product information strings, of which the first
    /// [`ns`](Self::ns) are valid. The header comment calls them pointers,
    /// but they are arrays of characters.
    pub pi: [[c_char; CIS_MAX_TUPLE_DATA_LEN]; CISTPL_VERS_1_MAX_PROD_STRINGS],
}

// Layout on amd64 with LP64. major, minor and ns at 0, 4 and 8, and pi
// 4 * 254 at 12, which ends at 1028; the alignment is 4, so the total is
// 1028.
const _: () = assert!(core::mem::size_of::<cistpl_vers_1_t>() == 1028);
const _: () = assert!(core::mem::offset_of!(cistpl_vers_1_t, major) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_vers_1_t, pi) == 12);

/// `csx_Parse_CISTPL_VERS_2(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_VERS_2` tuple, the level 2 information about the
/// logical organisation of the data on the card.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_vers_2_t {
    /// The version number.
    pub vers: u32,
    /// The level of compliance.
    pub comply: u32,
    /// The byte address of the first data byte on the card.
    pub dindex: u32,
    /// The two reserved bytes of the tuple. The manual page does not list
    /// this member.
    pub reserved: u32,
    /// Vendor specific byte 8.
    pub vspec8: u32,
    /// Vendor specific byte 9.
    pub vspec9: u32,
    /// The number of copies of the CIS present on the device.
    pub nhdr: u32,
    /// The vendor of the software that formatted the card.
    pub oem: [c_char; CIS_MAX_TUPLE_DATA_LEN],
    /// An informational message about the card.
    pub info: [c_char; CIS_MAX_TUPLE_DATA_LEN],
}

// Layout on amd64 with LP64. Seven uint32_t at 0 to 24, oem 254 at 28 and
// info 254 at 282, which ends at 536; the alignment is 4, so the total is
// 536.
const _: () = assert!(core::mem::size_of::<cistpl_vers_2_t>() == 536);
const _: () = assert!(core::mem::offset_of!(cistpl_vers_2_t, vers) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_vers_2_t, nhdr) == 24);
const _: () = assert!(core::mem::offset_of!(cistpl_vers_2_t, oem) == 28);
const _: () = assert!(core::mem::offset_of!(cistpl_vers_2_t, info) == 282);

// --- CISTPL_JEDEC_C and CISTPL_JEDEC_A -----------------------------------

/// The number of JEDEC identifiers a [`cistpl_jedec_t`] can hold
/// (`CISTPL_JEDEC_MAX_IDENTIFIERS`).
pub const CISTPL_JEDEC_MAX_IDENTIFIERS: usize = 4;

/// `csx_Parse_CISTPL_JEDEC_C(9F)`. illumos DDI Specific (illumos DDI).
///
/// One JEDEC identifier, an element of [`cistpl_jedec_t::jid`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct jedec_ident_t {
    /// The manufacturer identifier.
    pub id: u32,
    /// Manufacturer specific information.
    pub info: u32,
}

/// `csx_Parse_CISTPL_JEDEC_C(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_JEDEC_C` or `CISTPL_JEDEC_A` tuple. These optional
/// tuples give programming information for cards with programmable devices
/// in common or attribute memory.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_jedec_t {
    /// The number of JEDEC identifiers present.
    pub nid: u32,
    /// The identifiers, of which the first [`nid`](Self::nid) are valid.
    pub jid: [jedec_ident_t; CISTPL_JEDEC_MAX_IDENTIFIERS],
}

// Layout on amd64 with LP64. jedec_ident_t is id 4 at 0 and info 4 at 4, 8
// bytes. cistpl_jedec_t is nid 4 at 0 and jid 4 * 8 at 4; total 36.
const _: () = assert!(core::mem::size_of::<jedec_ident_t>() == 8);
const _: () = assert!(core::mem::offset_of!(jedec_ident_t, id) == 0);
const _: () = assert!(core::mem::offset_of!(jedec_ident_t, info) == 4);
const _: () = assert!(core::mem::size_of::<cistpl_jedec_t>() == 36);
const _: () = assert!(core::mem::offset_of!(cistpl_jedec_t, nid) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_jedec_t, jid) == 4);

// --- CISTPL_FORMAT and CISTPL_FORMAT_A -----------------------------------

/// `csx_Parse_CISTPL_FORMAT(9F)`. illumos DDI Specific (illumos DDI).
///
/// The disk-like view of a [`cistpl_format_dev`], which C declares as
/// `struct disk`. It is valid when [`cistpl_format_t::type_`] is
/// [`TPLFMTTYPE_DISK`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_format_disk {
    /// The block size.
    pub bksize: u32,
    /// The number of blocks.
    pub nblocks: u32,
    /// The location of the error detection code.
    pub edcloc: u32,
}

/// `csx_Parse_CISTPL_FORMAT(9F)`. illumos DDI Specific (illumos DDI).
///
/// The memory-like view of a [`cistpl_format_dev`], which C declares as
/// `struct mem`. It is valid when [`cistpl_format_t::type_`] is
/// [`TPLFMTTYPE_MEM`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_format_mem {
    /// Flags for the memory region, [`TPLFMTFLAGS_ADDR`] and
    /// [`TPLFMTFLAGS_AUTO`].
    pub flags: u32,
    /// Reserved.
    pub reserved: u32,
    /// The physical address of the region, valid with
    /// [`TPLFMTFLAGS_ADDR`].
    pub address: caddr_t,
    /// The location of the error detection code.
    pub edcloc: u32,
}

/// `csx_Parse_CISTPL_FORMAT(9F)`. illumos DDI Specific (illumos DDI).
///
/// The `dev` member of a [`cistpl_format_t`], an anonymous union in C,
/// holding the device specific part of the format.
#[repr(C)]
#[derive(Clone, Copy)]
pub union cistpl_format_dev {
    /// The format of a disk-like device.
    pub disk: cistpl_format_disk,
    /// The format of a memory-like device.
    pub mem: cistpl_format_mem,
}

/// `csx_Parse_CISTPL_FORMAT(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_FORMAT` tuple, which gives the data recording format of
/// a partition of a device. The header uses the same structure for
/// `CISTPL_FORMAT_A`, whose parser has no manual page.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct cistpl_format_t {
    /// The type of device, [`TPLFMTTYPE_DISK`], [`TPLFMTTYPE_MEM`] or
    /// [`TPLFMTTYPE_VS`]. The C member is named `type`, which is a Rust
    /// keyword.
    pub type_: u32,
    /// The length of the error detection code.
    pub edc_length: u32,
    /// The type of the error detection code, one of the `TPLFMTEDC_*`
    /// values.
    pub edc_type: u32,
    /// The offset of the first byte of data in the partition.
    pub offset: u32,
    /// The number of bytes of data in the partition.
    pub nbytes: u32,
    /// The device specific part, selected by [`type_`](Self::type_).
    pub dev: cistpl_format_dev,
}

// Layout on amd64 with LP64. cistpl_format_disk is three uint32_t, 12
// bytes. cistpl_format_mem is flags 4 at 0, reserved 4 at 4, address 8 at 8
// and edcloc 4 at 16, which ends at 20 and is padded to 24. The union
// cistpl_format_dev is therefore 24 bytes aligned to 8. cistpl_format_t is
// five uint32_t at 0 to 16, 4 bytes of padding and dev 24 at 24; total 48.
const _: () = assert!(core::mem::size_of::<cistpl_format_disk>() == 12);
const _: () = assert!(core::mem::offset_of!(cistpl_format_disk, bksize) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_format_disk, edcloc) == 8);
const _: () = assert!(core::mem::size_of::<cistpl_format_mem>() == 24);
const _: () = assert!(core::mem::offset_of!(cistpl_format_mem, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_format_mem, address) == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_format_mem, edcloc) == 16);
const _: () = assert!(core::mem::size_of::<cistpl_format_dev>() == 24);
const _: () = assert!(core::mem::size_of::<cistpl_format_t>() == 48);
const _: () = assert!(core::mem::offset_of!(cistpl_format_t, type_) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_format_t, nbytes) == 16);
const _: () = assert!(core::mem::offset_of!(cistpl_format_t, dev) == 24);

/// A disk-like format (`TPLFMTTYPE_DISK`).
pub const TPLFMTTYPE_DISK: u32 = 0x00;
/// A memory-like format (`TPLFMTTYPE_MEM`).
pub const TPLFMTTYPE_MEM: u32 = 0x01;
/// A vendor specific format (`TPLFMTTYPE_VS`).
pub const TPLFMTTYPE_VS: u32 = 0x80;
/// No error detection code (`TPLFMTEDC_NONE`).
pub const TPLFMTEDC_NONE: u32 = 0x00;
/// An arithmetic checksum (`TPLFMTEDC_CKSUM`).
pub const TPLFMTEDC_CKSUM: u32 = 0x01;
/// A 16-bit CRC (`TPLFMTEDC_CRC`).
pub const TPLFMTEDC_CRC: u32 = 0x02;
/// An arithmetic checksum over the whole partition (`TPLFMTEDC_PCC`).
pub const TPLFMTEDC_PCC: u32 = 0x03;
/// Vendor specific error checking (`TPLFMTEDC_VS`).
pub const TPLFMTEDC_VS: u32 = 0x80;
/// The address of a memory-like region is valid (`TPLFMTFLAGS_ADDR`).
pub const TPLFMTFLAGS_ADDR: u32 = 0x01;
/// The memory region is mapped automatically (`TPLFMTFLAGS_AUTO`).
pub const TPLFMTFLAGS_AUTO: u32 = 0x02;

// --- CISTPL_GEOMETRY, CISTPL_BYTEORDER, CISTPL_DATE, CISTPL_BATTERY ------

/// `csx_Parse_CISTPL_GEOMETRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_GEOMETRY` tuple, the geometry of a disk-like device.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_geometry_t {
    /// The number of sectors per track.
    pub spt: u32,
    /// The number of tracks per cylinder.
    pub tpc: u32,
    /// The number of cylinders.
    pub ncyl: u32,
}

// Layout on amd64 with LP64. Three uint32_t at 0, 4 and 8; total 12.
const _: () = assert!(core::mem::size_of::<cistpl_geometry_t>() == 12);
const _: () = assert!(core::mem::offset_of!(cistpl_geometry_t, spt) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_geometry_t, ncyl) == 8);

/// `csx_Parse_CISTPL_BYTEORDER(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_BYTEORDER` tuple, which appears only in the tuples of a
/// memory-like partition and gives the order of multiple byte data and how
/// bytes map into words on 16-bit cards.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_byteorder_t {
    /// The byte order of multiple byte numbers, [`TPLBYTEORD_LOW`],
    /// [`TPLBYTEORD_HIGH`] or [`TPLBYTEORD_VS`].
    pub order: u32,
    /// The byte mapping of 16-bit or wider cards, [`TPLBYTEMAP_LOW`],
    /// [`TPLBYTEMAP_HIGH`] or [`TPLBYTEMAP_VS`].
    pub map: u32,
}

// Layout on amd64 with LP64. Two uint32_t at 0 and 4; total 8.
const _: () = assert!(core::mem::size_of::<cistpl_byteorder_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_byteorder_t, order) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_byteorder_t, map) == 4);

/// Little-endian byte order (`TPLBYTEORD_LOW`).
pub const TPLBYTEORD_LOW: u32 = 0x00;
/// Big-endian byte order. The manual page does not list this value
/// (`TPLBYTEORD_HIGH`).
pub const TPLBYTEORD_HIGH: u32 = 0x01;
/// A vendor specific byte order, one of the values from 0x80 to 0xff
/// (`TPLBYTEORD_VS`).
pub const TPLBYTEORD_VS: u32 = 0x80;
/// Byte zero is the least significant byte (`TPLBYTEMAP_LOW`).
pub const TPLBYTEMAP_LOW: u32 = 0x00;
/// Byte zero is the most significant byte (`TPLBYTEMAP_HIGH`).
pub const TPLBYTEMAP_HIGH: u32 = 0x01;
/// A vendor specific byte mapping (`TPLBYTEMAP_VS`).
pub const TPLBYTEMAP_VS: u32 = 0x80;

/// `csx_Parse_CISTPL_DATE(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_DATE` tuple, the date and time at which the card was
/// formatted. The tuple is optional, and a card has at most one.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_date_t {
    /// The time at which the card was initialised.
    pub time: u32,
    /// The date on which the card was initialised.
    pub day: u32,
}

// Layout on amd64 with LP64. Two uint32_t at 0 and 4; total 8.
const _: () = assert!(core::mem::size_of::<cistpl_date_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_date_t, time) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_date_t, day) == 4);

/// `csx_Parse_CISTPL_BATTERY(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_BATTERY` tuple, which only cards with battery backed
/// storage have. It gives the date on which the battery was replaced and
/// the date on which it should be replaced again. A card has at most one.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_battery_t {
    /// The date on which the battery was last replaced.
    pub rday: u32,
    /// The date on which the battery should be replaced.
    pub xday: u32,
}

// Layout on amd64 with LP64. Two uint32_t at 0 and 4; total 8.
const _: () = assert!(core::mem::size_of::<cistpl_battery_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_battery_t, rday) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_battery_t, xday) == 4);

// --- CISTPL_ORG ----------------------------------------------------------

/// `csx_Parse_CISTPL_ORG(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_ORG` tuple, the organisation of the data in a
/// partition with a text description.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_org_t {
    /// The data organisation code. The C member is named `type`, which is
    /// a Rust keyword.
    pub type_: u32,
    /// A text description of the organisation.
    pub desc: [c_char; CIS_MAX_TUPLE_DATA_LEN],
}

// Layout on amd64 with LP64. type 4 at 0 and desc 254 at 4, which ends at
// 258 and is padded to 260.
const _: () = assert!(core::mem::size_of::<cistpl_org_t>() == 260);
const _: () = assert!(core::mem::offset_of!(cistpl_org_t, type_) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_org_t, desc) == 4);

// --- CISTPL_MANFID and CISTPL_FUNCID -------------------------------------

/// `csx_Parse_CISTPL_MANFID(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_MANFID` tuple, which identifies the manufacturer of the
/// card and the card itself.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_manfid_t {
    /// The manufacturer code that PCMCIA assigned.
    pub manf: u32,
    /// Manufacturer information, such as a part number or revision.
    pub card: u32,
}

// Layout on amd64 with LP64. Two uint32_t at 0 and 4; total 8.
const _: () = assert!(core::mem::size_of::<cistpl_manfid_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_manfid_t, manf) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_manfid_t, card) == 4);

/// `csx_Parse_CISTPL_FUNCID(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_FUNCID` tuple, which says what the card does and
/// whether system initialisation should configure it. Function specific
/// details follow in `CISTPL_FUNCE` tuples, parsed into
/// [`cistpl_funce_t`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_funcid_t {
    /// The function code, one of the `TPLFUNC_*` values.
    pub function: u32,
    /// The system initialisation mask, a combination of [`TPLINIT_POST`]
    /// and [`TPLINIT_ROM`].
    pub sysinit: u32,
}

// Layout on amd64 with LP64. Two uint32_t at 0 and 4; total 8.
const _: () = assert!(core::mem::size_of::<cistpl_funcid_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_funcid_t, function) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_funcid_t, sysinit) == 4);

/// A vendor specific multiple function card (`TPLFUNC_MULTI`).
pub const TPLFUNC_MULTI: u32 = 0x000;
/// A memory card (`TPLFUNC_MEMORY`).
pub const TPLFUNC_MEMORY: u32 = 0x001;
/// A serial I/O port (`TPLFUNC_SERIAL`).
pub const TPLFUNC_SERIAL: u32 = 0x002;
/// A parallel printer port (`TPLFUNC_PARALLEL`).
pub const TPLFUNC_PARALLEL: u32 = 0x003;
/// A fixed disk, silicon or removable (`TPLFUNC_FIXED`).
pub const TPLFUNC_FIXED: u32 = 0x004;
/// A video interface (`TPLFUNC_VIDEO`).
pub const TPLFUNC_VIDEO: u32 = 0x005;
/// A local area network adapter (`TPLFUNC_LAN`).
pub const TPLFUNC_LAN: u32 = 0x006;
/// Auto incrementing mass storage (`TPLFUNC_AIMS`).
pub const TPLFUNC_AIMS: u32 = 0x007;
/// A SCSI bridge (`TPLFUNC_SCSI`).
pub const TPLFUNC_SCSI: u32 = 0x008;
/// A security card (`TPLFUNC_SECURITY`).
pub const TPLFUNC_SECURITY: u32 = 0x009;
/// A vendor specific function (`TPLFUNC_VENDOR_SPECIFIC`).
pub const TPLFUNC_VENDOR_SPECIFIC: u32 = 0x0fe;
/// An unknown function. The PC Card standard does not define this value
/// (`TPLFUNC_UNKNOWN`).
pub const TPLFUNC_UNKNOWN: u32 = 0x0ff;
/// The power on self test should try to configure the card
/// (`TPLINIT_POST`).
pub const TPLINIT_POST: u32 = 0x01;
/// Map the ROM of the card during system initialisation (`TPLINIT_ROM`).
pub const TPLINIT_ROM: u32 = 0x02;

// --- CISTPL_FUNCE --------------------------------------------------------

/// `csx_Parse_CISTPL_FUNCE(9F)`. illumos DDI Specific (illumos DDI).
///
/// The serial port view of a [`cistpl_funce_data`], which C declares as
/// `struct serial`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_funce_serial {
    /// The UART in use, one of the `TPLFE_UA_*` values.
    pub ua: u32,
    /// The capabilities of the UART, a combination of the `TPLFE_UC_*`
    /// bits.
    pub uc: u32,
}

/// `csx_Parse_CISTPL_FUNCE(9F)`. illumos DDI Specific (illumos DDI).
///
/// The common modem interface view of a [`cistpl_funce_data`], which C
/// declares as `struct modem`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_funce_modem {
    /// The supported flow control methods, a combination of the
    /// `TPLFE_FC_*` bits.
    pub fc: u32,
    /// The size of the command buffer of the data circuit equipment.
    pub cb: u32,
    /// The size of the buffer from the data circuit equipment to its peer.
    pub eb: u32,
    /// The size of the buffer from the data terminal equipment to the data
    /// circuit equipment.
    pub tb: u32,
}

/// `csx_Parse_CISTPL_FUNCE(9F)`. illumos DDI Specific (illumos DDI).
///
/// The data modem view of a [`cistpl_funce_data`], which C declares as
/// `struct data_modem`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_funce_data_modem {
    /// The highest data rate.
    pub ud: u32,
    /// The modulation standards, a combination of the `TPLFE_MS_*` bits.
    pub ms: u32,
    /// The error correction and detection protocols and non-CCITT
    /// modulation, a combination of the `TPLFE_EM_*` bits.
    pub em: u32,
    /// The data compression protocols, a combination of the `TPLFE_DC_*`
    /// bits.
    pub dc: u32,
    /// The command protocols, a combination of the `TPLFE_CM_*` bits.
    pub cm: u32,
    /// The escape mechanisms, a combination of the `TPLFE_EX_*` bits.
    pub ex: u32,
    /// The standardised data encryption, reserved for future use and 0.
    pub dy: u32,
    /// Miscellaneous end user features, such as [`TPLFE_EF_CALLERID`].
    pub ef: u32,
    /// The number of country codes in [`cd`](Self::cd).
    pub ncd: u32,
    /// The CCITT country codes.
    pub cd: [c_uchar; 16],
}

/// `csx_Parse_CISTPL_FUNCE(9F)`. illumos DDI Specific (illumos DDI).
///
/// The fax modem view of a [`cistpl_funce_data`], which C declares as
/// `struct fax`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_funce_fax {
    /// The highest data rate between the data terminal equipment and the
    /// UART.
    pub uf: u32,
    /// The CCITT modulation standards, a combination of the `TPLFE_FM_*`
    /// bits.
    pub fm: u32,
    /// The standardised data encryption.
    pub fy: u32,
    /// The feature selection, a combination of the `TPLFE_FS_*` bits.
    pub fs: u32,
    /// The number of country codes in [`cf`](Self::cf).
    pub ncf: u32,
    /// The CCITT country codes.
    pub cf: [c_uchar; 16],
}

/// `csx_Parse_CISTPL_FUNCE(9F)`. illumos DDI Specific (illumos DDI).
///
/// The voice view of a [`cistpl_funce_data`], which C declares as
/// `struct voice`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_funce_voice {
    /// The highest data rate.
    pub uv: u32,
    /// The number of sampling rates in [`sr`](Self::sr).
    pub nsr: u32,
    /// The voice sampling rates, multiplied by 100.
    pub sr: [u32; 16],
    /// The number of sample sizes in [`ss`](Self::ss).
    pub nss: u32,
    /// The voice sample sizes, multiplied by 10.
    pub ss: [u32; 16],
    /// The number of compression methods in [`sc`](Self::sc).
    pub nsc: u32,
    /// The voice compression methods.
    pub sc: [u32; 16],
}

/// `csx_Parse_CISTPL_FUNCE(9F)`. illumos DDI Specific (illumos DDI).
///
/// The network adapter view of a [`cistpl_funce_data`], which C declares as
/// `struct lan`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_funce_lan {
    /// The network technology, one of the `TPLFE_LAN_TECH_*` values.
    pub tech: u32,
    /// The media bit or baud rate.
    pub speed: u32,
    /// The network media supported, one of the `TPLFE_LAN_MEDIA_*` values.
    pub media: u32,
    /// The open or closed connector standard.
    pub con: u32,
    /// The length of the station identifier in [`id`](Self::id).
    pub id_sz: u32,
    /// The station identifier, such as a MAC address.
    pub id: [c_uchar; 16],
}

/// `csx_Parse_CISTPL_FUNCE(9F)`. illumos DDI Specific (illumos DDI).
///
/// The `data` member of a [`cistpl_funce_t`], an anonymous union in C.
/// Which member is valid depends on the function the tuple extends and on
/// [`cistpl_funce_t::function`].
#[repr(C)]
#[derive(Clone, Copy)]
pub union cistpl_funce_data {
    /// A serial port.
    pub serial: cistpl_funce_serial,
    /// The common modem interface.
    pub modem: cistpl_funce_modem,
    /// Data modem services.
    pub data_modem: cistpl_funce_data_modem,
    /// Fax modem services.
    pub fax: cistpl_funce_fax,
    /// Voice services.
    pub voice: cistpl_funce_voice,
    /// A local area network adapter.
    pub lan: cistpl_funce_lan,
}

/// `csx_Parse_CISTPL_FUNCE(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_FUNCE` tuple, which gives details of a particular
/// function of the card. What it contains depends on the function that the
/// preceding `CISTPL_FUNCID` tuple names, which is why its parser takes
/// that function code as an argument.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct cistpl_funce_t {
    /// The kind of extended information, one of the `TPLFE_SUB_*` and
    /// `TPLFE_CAP_*` values.
    pub function: u32,
    /// A sub-category of the services the function provides, from 1 to 15.
    pub subfunction: u32,
    /// The extended information itself. See [`cistpl_funce_data`].
    pub data: cistpl_funce_data,
}

// Layout on amd64 with LP64. All members are uint32_t or uchar_t, so every
// structure is aligned to 4. cistpl_funce_serial is 8 bytes and
// cistpl_funce_modem 16. cistpl_funce_data_modem is nine uint32_t at 0 to
// 32 and cd 16 at 36; 52. cistpl_funce_fax is five uint32_t and cf 16 at
// 20; 36. cistpl_funce_voice is uv at 0, nsr at 4, sr 64 at 8, nss at 72, ss
// 64 at 76, nsc at 140 and sc 64 at 144; 208. cistpl_funce_lan is five
// uint32_t and id 16 at 20; 36. The union takes the largest, 208, and
// cistpl_funce_t is function 4 at 0, subfunction 4 at 4 and data 208 at 8;
// total 216.
const _: () = assert!(core::mem::size_of::<cistpl_funce_serial>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_serial, ua) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_serial, uc) == 4);
const _: () = assert!(core::mem::size_of::<cistpl_funce_modem>() == 16);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_modem, fc) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_modem, tb) == 12);
const _: () = assert!(core::mem::size_of::<cistpl_funce_data_modem>() == 52);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_data_modem, ud) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_data_modem, ncd) == 32);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_data_modem, cd) == 36);
const _: () = assert!(core::mem::size_of::<cistpl_funce_fax>() == 36);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_fax, uf) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_fax, cf) == 20);
const _: () = assert!(core::mem::size_of::<cistpl_funce_voice>() == 208);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_voice, uv) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_voice, sr) == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_voice, nss) == 72);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_voice, nsc) == 140);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_voice, sc) == 144);
const _: () = assert!(core::mem::size_of::<cistpl_funce_lan>() == 36);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_lan, tech) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_lan, id) == 20);
const _: () = assert!(core::mem::size_of::<cistpl_funce_data>() == 208);
const _: () = assert!(core::mem::size_of::<cistpl_funce_t>() == 216);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_t, function) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_funce_t, data) == 8);

/// Extended information about a serial port interface
/// (`TPLFE_SUB_SERIAL`).
pub const TPLFE_SUB_SERIAL: u32 = 0;
/// Extended information about a common modem interface
/// (`TPLFE_SUB_MODEM_COMMON`).
pub const TPLFE_SUB_MODEM_COMMON: u32 = 1;
/// Extended information about data modem services
/// (`TPLFE_SUB_MODEM_DATA`).
pub const TPLFE_SUB_MODEM_DATA: u32 = 2;
/// Extended information about fax modem services (`TPLFE_SUB_MODEM_FAX`).
pub const TPLFE_SUB_MODEM_FAX: u32 = 3;
/// Extended information about voice services (`TPLFE_SUB_VOICE`).
pub const TPLFE_SUB_VOICE: u32 = 4;
/// The capabilities of the data modem interface
/// (`TPLFE_CAP_MODEM_DATA`).
pub const TPLFE_CAP_MODEM_DATA: u32 = 5;
/// The capabilities of the fax modem interface (`TPLFE_CAP_MODEM_FAX`).
pub const TPLFE_CAP_MODEM_FAX: u32 = 6;
/// The capabilities of the voice modem interface
/// (`TPLFE_CAP_MODEM_VOICE`).
pub const TPLFE_CAP_MODEM_VOICE: u32 = 7;
/// The serial port interface for data modem services
/// (`TPLFE_CAP_SERIAL_DATA`).
pub const TPLFE_CAP_SERIAL_DATA: u32 = 8;
/// The serial port interface for fax modem services
/// (`TPLFE_CAP_SERIAL_FAX`).
pub const TPLFE_CAP_SERIAL_FAX: u32 = 9;
/// The serial port interface for voice modem services
/// (`TPLFE_CAP_SERIAL_VOICE`).
pub const TPLFE_CAP_SERIAL_VOICE: u32 = 10;

/// An Intel 8250 UART (`TPLFE_UA_8250`).
pub const TPLFE_UA_8250: u32 = 0;
/// An NS 16450 UART (`TPLFE_UA_16450`).
pub const TPLFE_UA_16450: u32 = 1;
/// An NS 16550 UART (`TPLFE_UA_16550`).
pub const TPLFE_UA_16550: u32 = 2;

/// Space parity is supported (`TPLFE_UC_PARITY_SPACE`).
pub const TPLFE_UC_PARITY_SPACE: u32 = 0x0001;
/// Mark parity is supported (`TPLFE_UC_PARITY_MARK`).
pub const TPLFE_UC_PARITY_MARK: u32 = 0x0002;
/// Odd parity is supported (`TPLFE_UC_PARITY_ODD`).
pub const TPLFE_UC_PARITY_ODD: u32 = 0x0004;
/// Even parity is supported (`TPLFE_UC_PARITY_EVEN`).
pub const TPLFE_UC_PARITY_EVEN: u32 = 0x0008;
/// 5-bit characters are supported (`TPLFE_UC_CS5`).
pub const TPLFE_UC_CS5: u32 = 0x0100;
/// 6-bit characters are supported (`TPLFE_UC_CS6`).
pub const TPLFE_UC_CS6: u32 = 0x0200;
/// 7-bit characters are supported (`TPLFE_UC_CS7`).
pub const TPLFE_UC_CS7: u32 = 0x0400;
/// 8-bit characters are supported (`TPLFE_UC_CS8`).
pub const TPLFE_UC_CS8: u32 = 0x0800;
/// One stop bit is supported (`TPLFE_UC_STOP_1`).
pub const TPLFE_UC_STOP_1: u32 = 0x1000;
/// One and a half stop bits are supported (`TPLFE_UC_STOP_15`).
pub const TPLFE_UC_STOP_15: u32 = 0x2000;
/// Two stop bits are supported (`TPLFE_UC_STOP_2`).
pub const TPLFE_UC_STOP_2: u32 = 0x4000;

/// Transmit XON/XOFF flow control (`TPLFE_FC_TX_XONOFF`).
pub const TPLFE_FC_TX_XONOFF: u32 = 0x01;
/// Receive XON/XOFF flow control (`TPLFE_FC_RX_XONOFF`).
pub const TPLFE_FC_RX_XONOFF: u32 = 0x02;
/// Transmit hardware flow control with CTS (`TPLFE_FC_TX_HW`).
pub const TPLFE_FC_TX_HW: u32 = 0x04;
/// Receive hardware flow control with RTS (`TPLFE_FC_RX_HW`).
pub const TPLFE_FC_RX_HW: u32 = 0x08;
/// Transparent flow control (`TPLFE_FC_TRANS`).
pub const TPLFE_FC_TRANS: u32 = 0x10;

/// 300 bps Bell 103 modulation (`TPLFE_MS_BELL103`).
pub const TPLFE_MS_BELL103: u32 = 0x0001;
/// 300 bps V.21 modulation (`TPLFE_MS_V21`).
pub const TPLFE_MS_V21: u32 = 0x0002;
/// 600 and 1200 bps V.23 modulation (`TPLFE_MS_V23`).
pub const TPLFE_MS_V23: u32 = 0x0004;
/// 1200 bps V.22A and V.22B modulation (`TPLFE_MS_V22AB`).
pub const TPLFE_MS_V22AB: u32 = 0x0008;
/// 2400 bps US Bell 212 modulation (`TPLFE_MS_BELL212`).
pub const TPLFE_MS_BELL212: u32 = 0x0010;
/// 2400 bps V.22bis modulation (`TPLFE_MS_V22BIS`).
pub const TPLFE_MS_V22BIS: u32 = 0x0020;
/// 2400 bps leased line V.26 modulation (`TPLFE_MS_V26`).
pub const TPLFE_MS_V26: u32 = 0x0040;
/// 2400 bps V.26bis modulation (`TPLFE_MS_V26BIS`).
pub const TPLFE_MS_V26BIS: u32 = 0x0080;
/// 4800 and 2400 bps leased line V.27bis modulation (`TPLFE_MS_V27BIS`).
pub const TPLFE_MS_V27BIS: u32 = 0x0100;
/// 9600, 7200 and 4800 bps leased line V.29 modulation (`TPLFE_MS_V29`).
pub const TPLFE_MS_V29: u32 = 0x0200;
/// V.32 modulation up to 9600 bps (`TPLFE_MS_V32`).
pub const TPLFE_MS_V32: u32 = 0x0400;
/// V.32bis modulation up to 14400 bps (`TPLFE_MS_V32BIS`).
pub const TPLFE_MS_V32BIS: u32 = 0x0800;
/// V.FAST modulation up to 28800 bps (`TPLFE_MS_VFAST`).
pub const TPLFE_MS_VFAST: u32 = 0x1000;

/// MNP levels 2 to 4 error correction (`TPLFE_EM_MNP`).
pub const TPLFE_EM_MNP: u32 = 0x01;
/// CCITT LAPM error correction, V.42 (`TPLFE_EM_V42`).
pub const TPLFE_EM_V42: u32 = 0x02;
/// CCITT V.42bis data compression. The manual page misspells it
/// `TPLFE_DC_V42BI` (`TPLFE_DC_V42BIS`).
pub const TPLFE_DC_V42BIS: u32 = 0x01;
/// MNP 5 data compression, which uses MNP 2, 3 or 4 (`TPLFE_DC_MNP5`).
pub const TPLFE_DC_MNP5: u32 = 0x02;

/// ANSI/EIA/TIA 602 action commands (`TPLFE_CM_AT1`).
pub const TPLFE_CM_AT1: u32 = 0x01;
/// ANSI/EIA/TIA 602 ACE/DCE interface parameters (`TPLFE_CM_AT2`).
pub const TPLFE_CM_AT2: u32 = 0x02;
/// ANSI/EIA/TIA 602 ACE parameters (`TPLFE_CM_AT3`).
pub const TPLFE_CM_AT3: u32 = 0x04;
/// MNP specific AT commands (`TPLFE_CM_MNP_AT`).
pub const TPLFE_CM_MNP_AT: u32 = 0x08;
/// V.25bis calling commands (`TPLFE_CM_V25BIS`).
pub const TPLFE_CM_V25BIS: u32 = 0x10;
/// V.25bis test procedures (`TPLFE_CM_V25A`).
pub const TPLFE_CM_V25A: u32 = 0x20;
/// DMCL command mode (`TPLFE_CM_DMCL`).
pub const TPLFE_CM_DMCL: u32 = 0x40;

/// A standardised BREAK escape is supported (`TPLFE_EX_BREAK`).
pub const TPLFE_EX_BREAK: u32 = 0x01;
/// The +++ sequence returns to command mode (`TPLFE_EX_PLUS`).
pub const TPLFE_EX_PLUS: u32 = 0x02;
/// A user defined escape character is supported (`TPLFE_EX_UD`).
pub const TPLFE_EX_UD: u32 = 0x04;
/// Caller ID is supported (`TPLFE_EF_CALLERID`).
pub const TPLFE_EF_CALLERID: u32 = 0x01;

/// 300 bps V.21 channel 2 fax modulation (`TPLFE_FM_V21C2`).
pub const TPLFE_FM_V21C2: u32 = 0x01;
/// 4800 and 2400 bps V.27ter fax modulation (`TPLFE_FM_V27TER`).
pub const TPLFE_FM_V27TER: u32 = 0x02;
/// 9600, 7200 and 4800 bps leased line V.29 fax modulation
/// (`TPLFE_FM_V29`).
pub const TPLFE_FM_V29: u32 = 0x04;
/// 14400, 12000, 9600 and 7200 bps V.17 fax modulation (`TPLFE_FM_V17`).
pub const TPLFE_FM_V17: u32 = 0x08;
/// 14400, 12000, 9600 and 7200 bps leased line V.33 fax modulation
/// (`TPLFE_FM_V33`).
pub const TPLFE_FM_V33: u32 = 0x10;

/// Group 2 (T.3) service class (`TPLFE_FS_T3`).
pub const TPLFE_FS_T3: u32 = 0x01;
/// Group 3 (T.4) service class (`TPLFE_FS_T4`).
pub const TPLFE_FS_T4: u32 = 0x02;
/// Group 4 (T.6) service class (`TPLFE_FS_T6`).
pub const TPLFE_FS_T6: u32 = 0x04;
/// Error correction mode (`TPLFE_FS_ECM`).
pub const TPLFE_FS_ECM: u32 = 0x08;
/// Voice requests are allowed (`TPLFE_FS_VOICEREQ`).
pub const TPLFE_FS_VOICEREQ: u32 = 0x10;
/// Polling is supported (`TPLFE_FS_POLLING`).
pub const TPLFE_FS_POLLING: u32 = 0x20;
/// File transfer is supported (`TPLFE_FS_FTP`).
pub const TPLFE_FS_FTP: u32 = 0x40;
/// Passwords are supported (`TPLFE_FS_PASSWORD`).
pub const TPLFE_FS_PASSWORD: u32 = 0x80;

/// ARCnet (`TPLFE_LAN_TECH_ARCNET`).
pub const TPLFE_LAN_TECH_ARCNET: u32 = 1;
/// Ethernet (`TPLFE_LAN_TECH_ETHERNET`).
pub const TPLFE_LAN_TECH_ETHERNET: u32 = 2;
/// Token Ring (`TPLFE_LAN_TECH_TOKENRING`).
pub const TPLFE_LAN_TECH_TOKENRING: u32 = 3;
/// LocalTalk (`TPLFE_LAN_TECH_LOCALTALK`).
pub const TPLFE_LAN_TECH_LOCALTALK: u32 = 4;
/// FDDI or CDDI (`TPLFE_LAN_TECH_FDDI`).
pub const TPLFE_LAN_TECH_FDDI: u32 = 5;
/// ATM (`TPLFE_LAN_TECH_ATM`).
pub const TPLFE_LAN_TECH_ATM: u32 = 6;
/// Wireless (`TPLFE_LAN_TECH_WIRELESS`).
pub const TPLFE_LAN_TECH_WIRELESS: u32 = 7;

/// A generic interface (`TPLFE_LAN_MEDIA_INHERENT`).
pub const TPLFE_LAN_MEDIA_INHERENT: u32 = 0;
/// Unshielded twisted pair (`TPLFE_LAN_MEDIA_UTP`).
pub const TPLFE_LAN_MEDIA_UTP: u32 = 1;
/// Shielded twisted pair (`TPLFE_LAN_MEDIA_STP`).
pub const TPLFE_LAN_MEDIA_STP: u32 = 2;
/// Thin coaxial cable (`TPLFE_LAN_MEDIA_THIN_COAX`).
pub const TPLFE_LAN_MEDIA_THIN_COAX: u32 = 3;
/// Thick coaxial cable (`TPLFE_LAN_MEDIA_THICK_COAX`).
pub const TPLFE_LAN_MEDIA_THICK_COAX: u32 = 4;
/// Fibre (`TPLFE_LAN_MEDIA_FIBER`).
pub const TPLFE_LAN_MEDIA_FIBER: u32 = 5;
/// Spread spectrum radio at 902 to 928 MHz (`TPLFE_LAN_MEDIA_SSR_902`).
pub const TPLFE_LAN_MEDIA_SSR_902: u32 = 6;
/// Spread spectrum radio at 2.4 GHz (`TPLFE_LAN_MEDIA_SSR_2_4`).
pub const TPLFE_LAN_MEDIA_SSR_2_4: u32 = 7;
/// Spread spectrum radio at 5.4 GHz (`TPLFE_LAN_MEDIA_SSR_5_4`).
pub const TPLFE_LAN_MEDIA_SSR_5_4: u32 = 8;
/// Diffuse infrared (`TPLFE_LAN_MEDIA_DIFFUSE_IR`).
pub const TPLFE_LAN_MEDIA_DIFFUSE_IR: u32 = 9;
/// Point to point infrared (`TPLFE_LAN_MEDIA_PTP_IR`).
pub const TPLFE_LAN_MEDIA_PTP_IR: u32 = 10;

// --- CISTPL_CFTABLE_ENTRY ------------------------------------------------

/// The number of I/O ranges a [`cistpl_cftable_entry_io_t`] can hold
/// (`CISTPL_CFTABLE_ENTRY_MAX_IO_RANGES`).
pub const CISTPL_CFTABLE_ENTRY_MAX_IO_RANGES: usize = 16;
/// The number of memory windows a [`cistpl_cftable_entry_mem_t`] can hold
/// (`CISTPL_CFTABLE_ENTRY_MAX_MEM_WINDOWS`).
pub const CISTPL_CFTABLE_ENTRY_MAX_MEM_WINDOWS: usize = 8;

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// The description of one power supply of a configuration, as it appears
/// three times in a [`cistpl_cftable_entry_pd_t`]. Each parameter has a
/// flags word whose low byte copies the parameter selection byte of the
/// tuple, a combination of the `CISTPL_CFTABLE_PD_*` selection bits such as
/// [`CISTPL_CFTABLE_PD_NOMV`], and whose higher bits are
/// [`CISTPL_CFTABLE_PD_EXISTS`], [`CISTPL_CFTABLE_PD_MUL10`],
/// [`CISTPL_CFTABLE_PD_NC_SLEEP`], [`CISTPL_CFTABLE_PD_ZERO`] and
/// [`CISTPL_CFTABLE_PD_NC`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_pwr_t {
    /// The nominal supply voltage.
    pub nomV: u32,
    /// The flags of [`nomV`](Self::nomV).
    pub nomV_flags: u32,
    /// The minimum supply voltage.
    pub minV: u32,
    /// The flags of [`minV`](Self::minV).
    pub minV_flags: u32,
    /// The maximum supply voltage.
    pub maxV: u32,
    /// The flags of [`maxV`](Self::maxV).
    pub maxV_flags: u32,
    /// The continuous supply current.
    pub staticI: u32,
    /// The flags of [`staticI`](Self::staticI).
    pub staticI_flags: u32,
    /// The maximum current required, averaged over one second.
    pub avgI: u32,
    /// The flags of [`avgI`](Self::avgI).
    pub avgI_flags: u32,
    /// The maximum current required, averaged over 10 ms.
    pub peakI: u32,
    /// The flags of [`peakI`](Self::peakI).
    pub peakI_flags: u32,
    /// The supply current required in power down mode.
    pub pdownI: u32,
    /// The flags of [`pdownI`](Self::pdownI).
    pub pdownI_flags: u32,
}

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// The power requirements of a configuration, the `pd` member of a
/// [`cistpl_cftable_entry_t`]. It is valid when the entry's flags include
/// [`CISTPL_CFTABLE_TPCE_FS_PWR`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_pd_t {
    /// Which descriptions are valid, a combination of
    /// [`CISTPL_CFTABLE_TPCE_FS_PWR_VCC`], [`CISTPL_CFTABLE_TPCE_FS_PWR_VPP1`]
    /// and [`CISTPL_CFTABLE_TPCE_FS_PWR_VPP2`].
    pub flags: u32,
    /// The Vcc power description.
    pub pd_vcc: cistpl_cftable_entry_pwr_t,
    /// The Vpp1 power description.
    pub pd_vpp1: cistpl_cftable_entry_pwr_t,
    /// The Vpp2 power description.
    pub pd_vpp2: cistpl_cftable_entry_pwr_t,
}

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// The timing of a configuration, the `speed` member of a
/// [`cistpl_cftable_entry_t`]. It is valid when the entry's flags include
/// [`CISTPL_CFTABLE_TPCE_FS_TD`]. Each time is given both in device speed
/// format and in nanoseconds.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_speed_t {
    /// Which timings are present, a combination of
    /// [`CISTPL_CFTABLE_TPCE_FS_TD_WAIT`], [`CISTPL_CFTABLE_TPCE_FS_TD_RDY`]
    /// and [`CISTPL_CFTABLE_TPCE_FS_TD_RSVD`].
    pub flags: u32,
    /// The maximum WAIT time in device speed format.
    pub wait: u32,
    /// The maximum WAIT time in nanoseconds.
    pub nS_wait: u32,
    /// The maximum READY/BUSY time in device speed format.
    pub rdybsy: u32,
    /// The maximum READY/BUSY time in nanoseconds.
    pub nS_rdybsy: u32,
    /// The maximum reserved time in device speed format.
    pub rsvd: u32,
    /// The maximum reserved time in nanoseconds.
    pub nS_rsvd: u32,
}

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// One I/O range of a configuration, an element of
/// [`cistpl_cftable_entry_io_t::range`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_io_range_t {
    /// The start address of the range.
    pub addr: u32,
    /// The length of the range.
    pub length: u32,
}

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// The I/O space of a configuration, the `io` member of a
/// [`cistpl_cftable_entry_t`]. It is valid when the entry's flags include
/// [`CISTPL_CFTABLE_TPCE_FS_IO`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_io_t {
    /// A direct copy of the TPCE_IO byte of the tuple, whose bus width bits
    /// are [`CISTPL_CFTABLE_TPCE_FS_IO_BUS8`] and
    /// [`CISTPL_CFTABLE_TPCE_FS_IO_BUS16`] under the mask
    /// [`CISTPL_CFTABLE_TPCE_FS_IO_BUS`], and which has
    /// [`CISTPL_CFTABLE_TPCE_FS_IO_RANGE`] set when address ranges follow.
    pub flags: u32,
    /// The number of I/O address lines the card decodes.
    pub addr_lines: u32,
    /// The number of I/O ranges in [`range`](Self::range).
    pub ranges: u32,
    /// The I/O ranges.
    pub range: [cistpl_cftable_entry_io_range_t; CISTPL_CFTABLE_ENTRY_MAX_IO_RANGES],
}

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// The interrupt use of a configuration, the `irq` member of a
/// [`cistpl_cftable_entry_t`]. It is valid when the entry's flags include
/// [`CISTPL_CFTABLE_TPCE_FS_IRQ`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_irq_t {
    /// A direct copy of the TPCE_IR byte of the tuple.
    pub flags: u32,
    /// A bit mask of the interrupts the card may use.
    pub irqs: u32,
}

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// One memory window of a configuration, an element of
/// [`cistpl_cftable_entry_mem_t::window`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_mem_window_t {
    /// The length of the window.
    pub length: u32,
    /// The card address of the window.
    pub card_addr: u32,
    /// The host address of the window.
    pub host_addr: u32,
}

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// The memory space of a configuration, the `mem` member of a
/// [`cistpl_cftable_entry_t`]. It is valid when the entry's flags include
/// [`CISTPL_CFTABLE_TPCE_FS_MEM`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_mem_t {
    /// The kind of memory description and whether host addresses are given,
    /// one of [`CISTPL_CFTABLE_TPCE_FS_MEM3`],
    /// [`CISTPL_CFTABLE_TPCE_FS_MEM2`] and [`CISTPL_CFTABLE_TPCE_FS_MEM1`],
    /// with [`CISTPL_CFTABLE_TPCE_FS_MEM_HOST`].
    pub flags: u32,
    /// The number of memory windows in [`window`](Self::window).
    pub windows: u32,
    /// The memory windows.
    pub window: [cistpl_cftable_entry_mem_window_t; CISTPL_CFTABLE_ENTRY_MAX_MEM_WINDOWS],
}

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// The miscellaneous features of a configuration, the `misc` member of a
/// [`cistpl_cftable_entry_t`]. It is valid when the entry's flags include
/// [`CISTPL_CFTABLE_TPCE_FS_MISC`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_misc_t {
    /// The features, a combination of the `CISTPL_CFTABLE_TPCE_MI_*` bits.
    pub flags: u32,
}

/// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_CFTABLE_ENTRY` tuple, which describes one possible
/// configuration of a 16-bit card. A card has one such tuple for each
/// configuration it supports, after its `CISTPL_CONFIG` tuple. The
/// [`flags`](Self::flags) member says which of the optional descriptions
/// are present.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_cftable_entry_t {
    /// Which descriptions are valid, a combination of
    /// [`CISTPL_CFTABLE_TPCE_DEFAULT`], [`CISTPL_CFTABLE_TPCE_IF`] and the
    /// `CISTPL_CFTABLE_TPCE_FS_*` presence bits.
    pub flags: u32,
    /// The interface description, valid when [`flags`](Self::flags)
    /// includes [`CISTPL_CFTABLE_TPCE_IF`]. The low four bits are the
    /// interface type under [`CISTPL_CFTABLE_TPCE_IF_MASK`], and the upper
    /// bits are the `CISTPL_CFTABLE_TPCE_IF_*` signal flags.
    pub ifc: u32,
    /// The value for the Pin Replacement Register.
    pub pin: u32,
    /// The configuration index number.
    pub index: u32,
    /// The power requirements.
    pub pd: cistpl_cftable_entry_pd_t,
    /// The timing.
    pub speed: cistpl_cftable_entry_speed_t,
    /// The I/O space.
    pub io: cistpl_cftable_entry_io_t,
    /// The interrupt use.
    pub irq: cistpl_cftable_entry_irq_t,
    /// The memory space.
    pub mem: cistpl_cftable_entry_mem_t,
    /// The miscellaneous features.
    pub misc: cistpl_cftable_entry_misc_t,
}

// Layout on amd64 with LP64. Every member is a uint32_t or an array or
// structure of them, so every structure is aligned to 4.
// cistpl_cftable_entry_pwr_t is fourteen uint32_t, 56 bytes.
// cistpl_cftable_entry_pd_t is flags 4 at 0, pd_vcc 56 at 4, pd_vpp1 56 at
// 60 and pd_vpp2 56 at 116; 172. cistpl_cftable_entry_speed_t is seven
// uint32_t, 28 bytes. cistpl_cftable_entry_io_range_t is 8 bytes and
// cistpl_cftable_entry_io_t is flags, addr_lines and ranges at 0 to 8 and
// range 16 * 8 at 12; 140. cistpl_cftable_entry_irq_t is 8 bytes.
// cistpl_cftable_entry_mem_window_t is 12 bytes and
// cistpl_cftable_entry_mem_t is flags and windows at 0 and 4 and window
// 8 * 12 at 8; 104. cistpl_cftable_entry_misc_t is 4 bytes.
// cistpl_cftable_entry_t is flags, ifc, pin and index at 0 to 12, pd 172 at
// 16, speed 28 at 188, io 140 at 216, irq 8 at 356, mem 104 at 364 and misc
// 4 at 468; total 472.
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_pwr_t>() == 56);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_pwr_t, nomV) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_pwr_t, pdownI_flags) == 52);
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_pd_t>() == 172);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_pd_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_pd_t, pd_vcc) == 4);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_pd_t, pd_vpp1) == 60);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_pd_t, pd_vpp2) == 116);
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_speed_t>() == 28);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_speed_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_speed_t, nS_rsvd) == 24);
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_io_range_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_io_range_t, addr) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_io_range_t, length) == 4);
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_io_t>() == 140);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_io_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_io_t, range) == 12);
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_irq_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_irq_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_irq_t, irqs) == 4);
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_mem_window_t>() == 12);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_mem_window_t, length) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_mem_window_t, host_addr) == 8);
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_mem_t>() == 104);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_mem_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_mem_t, window) == 8);
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_misc_t>() == 4);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_misc_t, flags) == 0);
const _: () = assert!(core::mem::size_of::<cistpl_cftable_entry_t>() == 472);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_t, pd) == 16);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_t, speed) == 188);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_t, io) == 216);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_t, irq) == 356);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_t, mem) == 364);
const _: () = assert!(core::mem::offset_of!(cistpl_cftable_entry_t, misc) == 468);

/// This is a default configuration (`CISTPL_CFTABLE_TPCE_DEFAULT`).
pub const CISTPL_CFTABLE_TPCE_DEFAULT: u32 = 0x0000_0100;
/// The interface configuration byte exists and
/// [`cistpl_cftable_entry_t::ifc`] is valid (`CISTPL_CFTABLE_TPCE_IF`).
pub const CISTPL_CFTABLE_TPCE_IF: u32 = 0x0000_0200;
/// Power information exists (`CISTPL_CFTABLE_TPCE_FS_PWR`).
pub const CISTPL_CFTABLE_TPCE_FS_PWR: u32 = 0x0000_1000;
/// Timing information exists (`CISTPL_CFTABLE_TPCE_FS_TD`).
pub const CISTPL_CFTABLE_TPCE_FS_TD: u32 = 0x0001_0000;
/// I/O information exists (`CISTPL_CFTABLE_TPCE_FS_IO`).
pub const CISTPL_CFTABLE_TPCE_FS_IO: u32 = 0x0010_0000;
/// Interrupt information exists (`CISTPL_CFTABLE_TPCE_FS_IRQ`).
pub const CISTPL_CFTABLE_TPCE_FS_IRQ: u32 = 0x0020_0000;
/// Memory space information exists (`CISTPL_CFTABLE_TPCE_FS_MEM`).
pub const CISTPL_CFTABLE_TPCE_FS_MEM: u32 = 0x0100_0000;
/// Miscellaneous information exists (`CISTPL_CFTABLE_TPCE_FS_MISC`).
pub const CISTPL_CFTABLE_TPCE_FS_MISC: u32 = 0x0200_0000;
/// An environment descriptor subtuple exists
/// (`CISTPL_CFTABLE_TPCE_FS_STCE_EV`).
pub const CISTPL_CFTABLE_TPCE_FS_STCE_EV: u32 = 0x0400_0000;
/// A physical device name subtuple exists
/// (`CISTPL_CFTABLE_TPCE_FS_STCE_PD`).
pub const CISTPL_CFTABLE_TPCE_FS_STCE_PD: u32 = 0x0800_0000;

/// A memory interface (`CISTPL_CFTABLE_TPCE_IF_MEMORY`).
pub const CISTPL_CFTABLE_TPCE_IF_MEMORY: u32 = 0x00;
/// An I/O and memory interface (`CISTPL_CFTABLE_TPCE_IF_IO_MEM`).
pub const CISTPL_CFTABLE_TPCE_IF_IO_MEM: u32 = 0x01;
/// Custom interface 0 (`CISTPL_CFTABLE_TPCE_IF_CUSTOM_0`).
pub const CISTPL_CFTABLE_TPCE_IF_CUSTOM_0: u32 = 0x04;
/// Custom interface 1 (`CISTPL_CFTABLE_TPCE_IF_CUSTOM_1`).
pub const CISTPL_CFTABLE_TPCE_IF_CUSTOM_1: u32 = 0x05;
/// Custom interface 2 (`CISTPL_CFTABLE_TPCE_IF_CUSTOM_2`).
pub const CISTPL_CFTABLE_TPCE_IF_CUSTOM_2: u32 = 0x06;
/// Custom interface 3 (`CISTPL_CFTABLE_TPCE_IF_CUSTOM_3`).
pub const CISTPL_CFTABLE_TPCE_IF_CUSTOM_3: u32 = 0x07;
/// The mask of the interface type (`CISTPL_CFTABLE_TPCE_IF_MASK`).
pub const CISTPL_CFTABLE_TPCE_IF_MASK: u32 = 0x0f;
/// BVD is active in the Pin Replacement Register
/// (`CISTPL_CFTABLE_TPCE_IF_BVD`).
pub const CISTPL_CFTABLE_TPCE_IF_BVD: u32 = 0x10;
/// Write protect is active in the Pin Replacement Register
/// (`CISTPL_CFTABLE_TPCE_IF_WP`).
pub const CISTPL_CFTABLE_TPCE_IF_WP: u32 = 0x20;
/// READY is active in the Pin Replacement Register
/// (`CISTPL_CFTABLE_TPCE_IF_RDY`).
pub const CISTPL_CFTABLE_TPCE_IF_RDY: u32 = 0x40;
/// WAIT is used for memory cycles (`CISTPL_CFTABLE_TPCE_IF_MWAIT`).
pub const CISTPL_CFTABLE_TPCE_IF_MWAIT: u32 = 0x80;

/// The Vcc power description is valid (`CISTPL_CFTABLE_TPCE_FS_PWR_VCC`).
pub const CISTPL_CFTABLE_TPCE_FS_PWR_VCC: u32 = 0x0000_0001;
/// The Vpp1 power description is valid
/// (`CISTPL_CFTABLE_TPCE_FS_PWR_VPP1`).
pub const CISTPL_CFTABLE_TPCE_FS_PWR_VPP1: u32 = 0x0000_0002;
/// The Vpp2 power description is valid
/// (`CISTPL_CFTABLE_TPCE_FS_PWR_VPP2`).
pub const CISTPL_CFTABLE_TPCE_FS_PWR_VPP2: u32 = 0x0000_0004;

/// The selection byte lists a nominal supply voltage
/// (`CISTPL_CFTABLE_PD_NOMV`).
pub const CISTPL_CFTABLE_PD_NOMV: u32 = 0x001;
/// The selection byte lists a minimum supply voltage
/// (`CISTPL_CFTABLE_PD_MINV`).
pub const CISTPL_CFTABLE_PD_MINV: u32 = 0x002;
/// The selection byte lists a maximum supply voltage
/// (`CISTPL_CFTABLE_PD_MAXV`).
pub const CISTPL_CFTABLE_PD_MAXV: u32 = 0x004;
/// The selection byte lists a continuous supply current
/// (`CISTPL_CFTABLE_PD_STATICI`).
pub const CISTPL_CFTABLE_PD_STATICI: u32 = 0x008;
/// The selection byte lists the maximum current averaged over one second
/// (`CISTPL_CFTABLE_PD_AVGI`).
pub const CISTPL_CFTABLE_PD_AVGI: u32 = 0x010;
/// The selection byte lists the maximum current averaged over 10 ms
/// (`CISTPL_CFTABLE_PD_PEAKI`).
pub const CISTPL_CFTABLE_PD_PEAKI: u32 = 0x020;
/// The selection byte lists the power down supply current
/// (`CISTPL_CFTABLE_PD_PDOWNI`).
pub const CISTPL_CFTABLE_PD_PDOWNI: u32 = 0x040;
/// The parameter exists (`CISTPL_CFTABLE_PD_EXISTS`).
pub const CISTPL_CFTABLE_PD_EXISTS: u32 = 0x0000_0100;
/// The value must be multiplied by 10 (`CISTPL_CFTABLE_PD_MUL10`).
pub const CISTPL_CFTABLE_PD_MUL10: u32 = 0x0000_0200;
/// No connection is required in sleep or power down mode
/// (`CISTPL_CFTABLE_PD_NC_SLEEP`).
pub const CISTPL_CFTABLE_PD_NC_SLEEP: u32 = 0x0000_1000;
/// A zero value is required (`CISTPL_CFTABLE_PD_ZERO`).
pub const CISTPL_CFTABLE_PD_ZERO: u32 = 0x0000_2000;
/// No connection is ever made (`CISTPL_CFTABLE_PD_NC`).
pub const CISTPL_CFTABLE_PD_NC: u32 = 0x0000_4000;

/// The WAIT timing exists (`CISTPL_CFTABLE_TPCE_FS_TD_WAIT`).
pub const CISTPL_CFTABLE_TPCE_FS_TD_WAIT: u32 = 0x0000_0001;
/// The READY/BUSY timing exists (`CISTPL_CFTABLE_TPCE_FS_TD_RDY`).
pub const CISTPL_CFTABLE_TPCE_FS_TD_RDY: u32 = 0x0000_0002;
/// The reserved timing exists (`CISTPL_CFTABLE_TPCE_FS_TD_RSVD`).
pub const CISTPL_CFTABLE_TPCE_FS_TD_RSVD: u32 = 0x0000_0004;

/// The mask of the bus width bits of the I/O description
/// (`CISTPL_CFTABLE_TPCE_FS_IO_BUS`).
pub const CISTPL_CFTABLE_TPCE_FS_IO_BUS: u32 = 0x060;
/// The card supports 8-bit I/O (`CISTPL_CFTABLE_TPCE_FS_IO_BUS8`).
pub const CISTPL_CFTABLE_TPCE_FS_IO_BUS8: u32 = 0x020;
/// The card supports 16-bit I/O (`CISTPL_CFTABLE_TPCE_FS_IO_BUS16`).
pub const CISTPL_CFTABLE_TPCE_FS_IO_BUS16: u32 = 0x040;
/// I/O address ranges exist (`CISTPL_CFTABLE_TPCE_FS_IO_RANGE`).
pub const CISTPL_CFTABLE_TPCE_FS_IO_RANGE: u32 = 0x080;

/// The memory space is given by space descriptors
/// (`CISTPL_CFTABLE_TPCE_FS_MEM3`).
pub const CISTPL_CFTABLE_TPCE_FS_MEM3: u32 = 0x0000_0001;
/// The host address equals the card address
/// (`CISTPL_CFTABLE_TPCE_FS_MEM2`).
pub const CISTPL_CFTABLE_TPCE_FS_MEM2: u32 = 0x0000_0002;
/// The card address is 0 and any host address may be used
/// (`CISTPL_CFTABLE_TPCE_FS_MEM1`).
pub const CISTPL_CFTABLE_TPCE_FS_MEM1: u32 = 0x0000_0004;
/// The space descriptors include host addresses
/// (`CISTPL_CFTABLE_TPCE_FS_MEM_HOST`).
pub const CISTPL_CFTABLE_TPCE_FS_MEM_HOST: u32 = 0x0000_0008;

/// The mask of the maximum number of twin cards
/// (`CISTPL_CFTABLE_TPCE_MI_MTC_MASK`).
pub const CISTPL_CFTABLE_TPCE_MI_MTC_MASK: u32 = 0x0000_0007;
/// Audio is supported on BVD2 (`CISTPL_CFTABLE_TPCE_MI_AUDIO`).
pub const CISTPL_CFTABLE_TPCE_MI_AUDIO: u32 = 0x0000_0008;
/// The storage is read only (`CISTPL_CFTABLE_TPCE_MI_READONLY`).
pub const CISTPL_CFTABLE_TPCE_MI_READONLY: u32 = 0x0000_0010;
/// The card is capable of power down (`CISTPL_CFTABLE_TPCE_MI_PWRDOWN`).
pub const CISTPL_CFTABLE_TPCE_MI_PWRDOWN: u32 = 0x0000_0020;
/// The mask of the DMA request signal (`CISTPL_CFTABLE_TPCE_MI_DRQ_MASK`).
pub const CISTPL_CFTABLE_TPCE_MI_DRQ_MASK: u32 = 0x0000_0c00;
/// DMA requests on SPKR (`CISTPL_CFTABLE_TPCE_MI_DRQ_SPK`).
pub const CISTPL_CFTABLE_TPCE_MI_DRQ_SPK: u32 = 0x0000_0400;
/// DMA requests on IOIS16 (`CISTPL_CFTABLE_TPCE_MI_DRQ_IOIS`).
pub const CISTPL_CFTABLE_TPCE_MI_DRQ_IOIS: u32 = 0x0000_0800;
/// DMA requests on INPACK (`CISTPL_CFTABLE_TPCE_MI_DRQ_INP`).
pub const CISTPL_CFTABLE_TPCE_MI_DRQ_INP: u32 = 0x0000_0c00;
/// A DMA width of 8 bits (`CISTPL_CFTABLE_TPCE_MI_DMA_8`).
pub const CISTPL_CFTABLE_TPCE_MI_DMA_8: u32 = 0x0000_0000;
/// A DMA width of 16 bits (`CISTPL_CFTABLE_TPCE_MI_DMA_16`).
pub const CISTPL_CFTABLE_TPCE_MI_DMA_16: u32 = 0x0000_1000;

// --- CISTPL_LINKTARGET, CISTPL_LONGLINK_A, CISTPL_LONGLINK_C -------------

/// `csx_Parse_CISTPL_LINKTARGET(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_LINKTARGET` tuple. Every chain other than the primary
/// one must start with this tuple, and its tag, the characters `CIS`,
/// verifies that the chain is valid.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_linktarget_t {
    /// The number of bytes in [`tpltg_tag`](Self::tpltg_tag).
    pub length: u32,
    /// The link target tag.
    pub tpltg_tag: [c_char; CIS_MAX_TUPLE_DATA_LEN],
}

// Layout on amd64 with LP64. length 4 at 0 and tpltg_tag 254 at 4, which
// ends at 258 and is padded to 260.
const _: () = assert!(core::mem::size_of::<cistpl_linktarget_t>() == 260);
const _: () = assert!(core::mem::offset_of!(cistpl_linktarget_t, length) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_linktarget_t, tpltg_tag) == 4);

/// `csx_Parse_CISTPL_LONGLINK_A(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_LONGLINK_A` or `CISTPL_LONGLINK_C` tuple, a link to a
/// chain in attribute or common memory.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_longlink_ac_t {
    /// The memory the link points into, [`CISTPL_LONGLINK_AC_AM`] or
    /// [`CISTPL_LONGLINK_AC_CM`].
    pub flags: u32,
    /// The offset of the target from the beginning of that address space.
    /// It is an offset, not a virtual address.
    pub tpll_addr: u32,
}

// Layout on amd64 with LP64. Two uint32_t at 0 and 4; total 8.
const _: () = assert!(core::mem::size_of::<cistpl_longlink_ac_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_ac_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_ac_t, tpll_addr) == 4);

/// The long link points into attribute memory (`CISTPL_LONGLINK_AC_AM`).
pub const CISTPL_LONGLINK_AC_AM: u32 = 0x0001;
/// The long link points into common memory (`CISTPL_LONGLINK_AC_CM`).
pub const CISTPL_LONGLINK_AC_CM: u32 = 0x0002;

// --- CISTPL_LONGLINK_MFC -------------------------------------------------

/// The largest number of functions a card can have, and the length of
/// [`cistpl_longlink_mfc_t::function`] (`CIS_MAX_FUNCTIONS`).
pub const CIS_MAX_FUNCTIONS: usize = 8;

/// `csx_Parse_CISTPL_LONGLINK_MFC(9F)`. illumos DDI Specific (illumos DDI).
///
/// Where the function specific CIS of one function starts, an element of
/// [`cistpl_longlink_mfc_t::function`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cis_function_t {
    /// The target address space, [`CISTPL_LONGLINK_MFC_TAS_AM`] or
    /// [`CISTPL_LONGLINK_MFC_TAS_CM`].
    pub tas: u32,
    /// The offset of the target in that space.
    pub addr: u32,
}

/// `csx_Parse_CISTPL_LONGLINK_MFC(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_LONGLINK_MFC` tuple, which gives the start of the
/// function specific CIS of each function of a multiple function card.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_longlink_mfc_t {
    /// The number of functions on the card.
    pub nfuncs: u32,
    /// The number of configuration register sets.
    pub nregs: u32,
    /// The start of the CIS of each function, of which the first
    /// [`nfuncs`](Self::nfuncs) are valid.
    pub function: [cis_function_t; CIS_MAX_FUNCTIONS],
}

// Layout on amd64 with LP64. cis_function_t is tas 4 at 0 and addr 4 at 4,
// 8 bytes. cistpl_longlink_mfc_t is nfuncs 4 at 0, nregs 4 at 4 and
// function 8 * 8 at 8; total 72.
const _: () = assert!(core::mem::size_of::<cis_function_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cis_function_t, tas) == 0);
const _: () = assert!(core::mem::offset_of!(cis_function_t, addr) == 4);
const _: () = assert!(core::mem::size_of::<cistpl_longlink_mfc_t>() == 72);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_mfc_t, nfuncs) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_mfc_t, function) == 8);

/// The CIS of the function is in attribute memory
/// (`CISTPL_LONGLINK_MFC_TAS_AM`).
pub const CISTPL_LONGLINK_MFC_TAS_AM: u32 = 0x00;
/// The CIS of the function is in common memory
/// (`CISTPL_LONGLINK_MFC_TAS_CM`).
pub const CISTPL_LONGLINK_MFC_TAS_CM: u32 = 0x01;

// --- CISTPL_LONGLINK_CB --------------------------------------------------

/// `csx_ParseTuple(9F)`. illumos DDI Specific (illumos DDI).
///
/// The configuration space view of a [`cistpl_longlink_cb_space`], an
/// anonymous structure named `cfg` in C.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_longlink_cb_cfg {
    /// The offset within configuration space.
    pub offset: u32,
}

/// `csx_ParseTuple(9F)`. illumos DDI Specific (illumos DDI).
///
/// The memory space view of a [`cistpl_longlink_cb_space`], an anonymous
/// structure named `mem` in C.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_longlink_cb_mem {
    /// The base address register, from 1 to 6.
    pub asi: u32,
    /// The offset within the space of that register.
    pub offset: u32,
}

/// `csx_ParseTuple(9F)`. illumos DDI Specific (illumos DDI).
///
/// The expansion ROM view of a [`cistpl_longlink_cb_space`], an anonymous
/// structure named `rom` in C.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_longlink_cb_rom {
    /// The image number.
    pub image: u32,
    /// The offset from the base of the image.
    pub offset: u32,
}

/// `csx_ParseTuple(9F)`. illumos DDI Specific (illumos DDI).
///
/// The `space` member of a [`cistpl_longlink_cb_t`], an anonymous union in
/// C. Which member is valid is given by [`cistpl_longlink_cb_t::flags`].
#[repr(C)]
#[derive(Clone, Copy)]
pub union cistpl_longlink_cb_space {
    /// The target in device dependent configuration space, valid with
    /// [`CISTPL_LONGLINK_CB_CFG`].
    pub cfg: cistpl_longlink_cb_cfg,
    /// The target in a memory space, valid with [`CISTPL_LONGLINK_CB_MEM`].
    pub mem: cistpl_longlink_cb_mem,
    /// The target in an expansion ROM, valid with
    /// [`CISTPL_LONGLINK_CB_ROM`].
    pub rom: cistpl_longlink_cb_rom,
}

/// `csx_ParseTuple(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_LONGLINK_CB` tuple, the start of the CIS chain of a
/// function of a CardBus card. It is a member of the union that
/// `csx_ParseTuple(9F)` fills in. `sys/cs.h` declares a parser
/// `csx_Parse_CISTPL_LONGLINK_CB` for it, but no manual page documents that
/// function and this crate does not bind it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct cistpl_longlink_cb_t {
    /// Which address space the target is in, [`CISTPL_LONGLINK_CB_CFG`],
    /// [`CISTPL_LONGLINK_CB_MEM`] or [`CISTPL_LONGLINK_CB_ROM`].
    pub flags: u32,
    /// The raw, unprocessed address value of the tuple.
    pub addr: u32,
    /// The target, decoded for its address space.
    pub space: cistpl_longlink_cb_space,
}

// Layout on amd64 with LP64. cistpl_longlink_cb_cfg is 4 bytes and
// cistpl_longlink_cb_mem and cistpl_longlink_cb_rom are 8, all aligned to
// 4, so the union is 8 bytes. cistpl_longlink_cb_t is flags 4 at 0, addr 4
// at 4 and space 8 at 8; total 16.
const _: () = assert!(core::mem::size_of::<cistpl_longlink_cb_cfg>() == 4);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_cb_cfg, offset) == 0);
const _: () = assert!(core::mem::size_of::<cistpl_longlink_cb_mem>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_cb_mem, asi) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_cb_mem, offset) == 4);
const _: () = assert!(core::mem::size_of::<cistpl_longlink_cb_rom>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_cb_rom, image) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_cb_rom, offset) == 4);
const _: () = assert!(core::mem::size_of::<cistpl_longlink_cb_space>() == 8);
const _: () = assert!(core::mem::size_of::<cistpl_longlink_cb_t>() == 16);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_cb_t, flags) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_longlink_cb_t, space) == 8);

/// The configuration space target is valid (`CISTPL_LONGLINK_CB_CFG`).
pub const CISTPL_LONGLINK_CB_CFG: u32 = 0x0001;
/// The memory space target is valid (`CISTPL_LONGLINK_CB_MEM`).
pub const CISTPL_LONGLINK_CB_MEM: u32 = 0x0002;
/// The expansion ROM target is valid (`CISTPL_LONGLINK_CB_ROM`).
pub const CISTPL_LONGLINK_CB_ROM: u32 = 0x0004;

// --- CISTPL_SPCL and CISTPL_SWIL -----------------------------------------

/// `csx_Parse_CISTPL_SPCL(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_SPCL` tuple, the special purpose tuple. An identifier
/// that PCMCIA or JEIDA assigns says what the data means, and a sequence
/// number allows a series of such tuples to carry more data than one tuple
/// can hold.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_spcl_t {
    /// The identifier, assigned by PCMCIA or JEIDA, of this series of
    /// tuples.
    pub id: u32,
    /// The sequence number of the data. [`CISTPL_SPCL_SEQ_END`] marks the
    /// last tuple of the sequence.
    pub seq: u32,
    /// The number of data bytes in [`data`](Self::data).
    pub bytes: u32,
    /// The data of this tuple.
    pub data: [c_uchar; CIS_MAX_TUPLE_DATA_LEN],
}

// Layout on amd64 with LP64. id, seq and bytes at 0, 4 and 8 and data 254 at
// 12, which ends at 266 and is padded to 268.
const _: () = assert!(core::mem::size_of::<cistpl_spcl_t>() == 268);
const _: () = assert!(core::mem::offset_of!(cistpl_spcl_t, id) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_spcl_t, data) == 12);

/// The bit of [`cistpl_spcl_t::seq`] that marks the last tuple of a
/// sequence (`CISTPL_SPCL_SEQ_END`).
pub const CISTPL_SPCL_SEQ_END: u32 = 0x080;

/// `csx_Parse_CISTPL_SWIL(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_SWIL` tuple, the software interleaving of data within a
/// partition.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_swil_t {
    /// The software interleave of the partition.
    pub intrlv: u32,
}

// Layout on amd64 with LP64. One uint32_t at 0; total 4.
const _: () = assert!(core::mem::size_of::<cistpl_swil_t>() == 4);
const _: () = assert!(core::mem::offset_of!(cistpl_swil_t, intrlv) == 0);

// --- CISTPL_BAR ----------------------------------------------------------

/// `csx_ParseTuple(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_BAR` tuple, which describes a base address register of
/// a CardBus card. It is a member of the union that `csx_ParseTuple(9F)`
/// fills in. `sys/cs.h` declares a parser `csx_Parse_CISTPL_BAR` for it,
/// but no manual page documents that function and this crate does not bind
/// it.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_bar_t {
    /// The attributes of the register, the register number under
    /// [`CISTPL_BAR_ASI_MASK`] combined with the other `CISTPL_BAR_*` bits.
    pub attributes: u32,
    /// The size of the register's space.
    pub size: u32,
}

// Layout on amd64 with LP64. Two uint32_t at 0 and 4; total 8.
const _: () = assert!(core::mem::size_of::<cistpl_bar_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cistpl_bar_t, attributes) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_bar_t, size) == 4);

/// The mask of the base address register number (`CISTPL_BAR_ASI_MASK`).
pub const CISTPL_BAR_ASI_MASK: u32 = 0x007;
/// Base address register 1 (`CISTPL_BAR_ASI_BAR_1`).
pub const CISTPL_BAR_ASI_BAR_1: u32 = 0x001;
/// Base address register 2 (`CISTPL_BAR_ASI_BAR_2`).
pub const CISTPL_BAR_ASI_BAR_2: u32 = 0x002;
/// Base address register 3 (`CISTPL_BAR_ASI_BAR_3`).
pub const CISTPL_BAR_ASI_BAR_3: u32 = 0x003;
/// Base address register 4 (`CISTPL_BAR_ASI_BAR_4`).
pub const CISTPL_BAR_ASI_BAR_4: u32 = 0x004;
/// Base address register 5 (`CISTPL_BAR_ASI_BAR_5`).
pub const CISTPL_BAR_ASI_BAR_5: u32 = 0x005;
/// Base address register 6 (`CISTPL_BAR_ASI_BAR_6`).
pub const CISTPL_BAR_ASI_BAR_6: u32 = 0x006;
/// Base address register 7, which shares its value with
/// [`CISTPL_BAR_ASI_EXP_ROM`] (`CISTPL_BAR_ASI_BAR_7`).
pub const CISTPL_BAR_ASI_BAR_7: u32 = 0x007;
/// The expansion ROM base address register (`CISTPL_BAR_ASI_EXP_ROM`).
pub const CISTPL_BAR_ASI_EXP_ROM: u32 = 0x007;
/// The register describes memory space (`CISTPL_BAR_AS_MEM`).
pub const CISTPL_BAR_AS_MEM: u32 = 0x000;
/// The register describes I/O space (`CISTPL_BAR_AS_IO`).
pub const CISTPL_BAR_AS_IO: u32 = 0x008;
/// The mask of the prefetch and cache bits
/// (`CISTPL_BAR_PREFETCH_CACHE_MASK`).
pub const CISTPL_BAR_PREFETCH_CACHE_MASK: u32 = 0x060;
/// The space is prefetchable but not cacheable (`CISTPL_BAR_PREFETCH`).
pub const CISTPL_BAR_PREFETCH: u32 = 0x020;
/// The space is prefetchable and cacheable (`CISTPL_BAR_PREFETCH_CACHE`).
pub const CISTPL_BAR_PREFETCH_CACHE: u32 = 0x040;
/// The space must be located within the first megabyte
/// (`CISTPL_BAR_BELOW_1MB`).
pub const CISTPL_BAR_BELOW_1MB: u32 = 0x080;

// --- CISTPL_DEVICEGEO and CISTPL_DEVICEGEO_A -----------------------------

/// The number of partitions a [`cistpl_devicegeo_t`] can describe
/// (`CISTPL_DEVICEGEO_MAX_PARTITIONS`).
pub const CISTPL_DEVICEGEO_MAX_PARTITIONS: usize = 42;

/// `csx_Parse_CISTPL_DEVICEGEO(9F)`. illumos DDI Specific (illumos DDI).
///
/// The geometry of one partition, an element of
/// [`cistpl_devicegeo_t::info`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_devicegeo_info_t {
    /// The card interface width in bytes.
    pub bus: u32,
    /// The minimum erase block size.
    pub ebs: u32,
    /// The minimum read block size.
    pub rbs: u32,
    /// The minimum write block size.
    pub wbs: u32,
    /// The segment partition subdivisions.
    pub part: u32,
    /// The hardware interleave.
    pub hwil: u32,
}

/// `csx_Parse_CISTPL_DEVICEGEO(9F)`. illumos DDI Specific (illumos DDI).
///
/// A parsed `CISTPL_DEVICEGEO` or `CISTPL_DEVICEGEO_A` tuple, the device
/// geometry of the partitions of common or attribute memory.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_devicegeo_t {
    /// The geometry of each partition.
    pub info: [cistpl_devicegeo_info_t; CISTPL_DEVICEGEO_MAX_PARTITIONS],
}

// Layout on amd64 with LP64. cistpl_devicegeo_info_t is six uint32_t at 0
// to 20, 24 bytes, and cistpl_devicegeo_t is 42 of them; total 1008.
const _: () = assert!(core::mem::size_of::<cistpl_devicegeo_info_t>() == 24);
const _: () = assert!(core::mem::offset_of!(cistpl_devicegeo_info_t, bus) == 0);
const _: () = assert!(core::mem::offset_of!(cistpl_devicegeo_info_t, hwil) == 20);
const _: () = assert!(core::mem::size_of::<cistpl_devicegeo_t>() == 42 * 24);
const _: () = assert!(core::mem::offset_of!(cistpl_devicegeo_t, info) == 0);

// --- tuple names ---------------------------------------------------------

/// The size of the tuple name in a [`cistpl_get_tuple_name_t`]
/// (`CIS_MAX_TUPLE_NAME_LEN`).
pub const CIS_MAX_TUPLE_NAME_LEN: usize = 40;

/// `csx_ParseTuple(9F)`. illumos DDI Specific (illumos DDI).
///
/// The name of a tuple, which `csx_ParseTuple(9F)` returns in its union
/// when the tuple's [`Attributes`](crate::tuple::tuple_t::Attributes)
/// include [`TUPLE_RETURN_NAME`](crate::tuple::TUPLE_RETURN_NAME).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cistpl_get_tuple_name_t {
    /// The name of the tuple.
    pub name: [c_char; CIS_MAX_TUPLE_NAME_LEN],
}

// Layout on amd64 with LP64. name 40 at 0; total 40.
const _: () = assert!(core::mem::size_of::<cistpl_get_tuple_name_t>() == 40);
const _: () = assert!(core::mem::offset_of!(cistpl_get_tuple_name_t, name) == 0);
