//! SCSI command descriptor blocks from `sys/scsi/generic/commands.h` and
//! `sys/scsi/impl/commands.h`.
//!
//! A command descriptor block, or CDB, is the command a target driver sends
//! to a device. Its first byte is the operation code, whose top three bits
//! give the command group and thereby the length of the block. A target
//! driver writes the block into the buffer that `pkt_cdbp` of its
//! `scsi_pkt(9S)` points to, either by hand or with [`scsi_setup_cdb`],
//! which fills in the operation code, the address, the transfer length and,
//! for group 4, the additional data of a group 0, 1, 2, 4 or 5 block.
//!
//! The union [`scsi_cdb`] that [`scsi_setup_cdb`] takes is declared in
//! `sys/scsi/impl/commands.h`. That header is not public as a whole, but the
//! union is part of the documented prototype and is therefore bound here,
//! together with the group layouts it contains. The C declaration uses bit
//! fields in its second byte and in the control byte of each group; Rust has
//! no bit fields, so those bytes are declared whole and their documentation
//! names the bits, counted from the least significant with
//! `_BIT_FIELDS_LTOH`.
//!
//! The `SCMD_*` operation codes bound are the ones the manual pages name or
//! whose commands they describe, together with the group 4 and group 5 forms
//! of READ and WRITE that `scsi_setup_cdb(9F)` exists to build. The many
//! other codes in the header, the `SSVC_ACTION_*` service actions, the
//! `SD_GESN_*` definitions and the `SCSI_CMDS_KEY_STRINGS` initialiser have
//! no manual page. The `CDB_GROUPID` macro, the member macros such as
//! `scc_cmd` and `g0_addr0`, and the `FORMG0ADDR` family of macros are C
//! conveniences with no symbol. The `makecom_g0`, `makecom_g0_s`,
//! `makecom_g1` and `makecom_g5` functions are obsolete in favour of
//! [`scsi_setup_cdb`] and are not bound, and neither are the defect list and
//! capacity structures of `sys/scsi/impl/commands.h`, which no manual page
//! documents.

use ddi_sys::{c_int, c_uchar, uint_t};

// --- group lengths -------------------------------------------------------

/// The length in bytes of a group 0 command descriptor block
/// (`CDB_GROUP0`).
pub const CDB_GROUP0: c_int = 6;
/// The length in bytes of a group 1 command descriptor block
/// (`CDB_GROUP1`).
pub const CDB_GROUP1: c_int = 10;
/// The length in bytes of a group 2 command descriptor block
/// (`CDB_GROUP2`).
pub const CDB_GROUP2: c_int = 10;
/// The length in bytes of a group 4 command descriptor block
/// (`CDB_GROUP4`).
pub const CDB_GROUP4: c_int = 16;
/// The length in bytes of a group 5 command descriptor block
/// (`CDB_GROUP5`).
pub const CDB_GROUP5: c_int = 12;

// --- the command descriptor block ----------------------------------------

/// `scsi_setup_cdb(9F)`. illumos DDI specific (illumos DDI).
///
/// The group 0 part of a [`scsi_cdb`], `struct scsi_g0`, which follows the
/// first two bytes of a 6 byte block.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_g0 {
    /// The middle byte of the logical block address. The high five bits of
    /// the address are in the second byte of the block.
    pub addr1: c_uchar,
    /// The low byte of the logical block address.
    pub addr0: c_uchar,
    /// The transfer length, usually a block count.
    pub count0: c_uchar,
    /// The control byte. Bit 0 is `link`, set when another command follows,
    /// bit 1 is `flag`, set to interrupt when done, bit 2 is `naca`, normal
    /// auto contingent allegiance, bits 3 to 5 are reserved, and bits 6 and
    /// 7 are the vendor unique `vu_56` and `vu_57`.
    pub control: c_uchar,
}

/// `scsi_setup_cdb(9F)`. illumos DDI specific (illumos DDI).
///
/// The group 1 and group 2 part of a [`scsi_cdb`], `struct scsi_g1`, which
/// follows the first two bytes of a 10 byte block.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_g1 {
    /// The most significant byte of the logical block address.
    pub addr3: c_uchar,
    /// The second byte of the logical block address.
    pub addr2: c_uchar,
    /// The third byte of the logical block address.
    pub addr1: c_uchar,
    /// The least significant byte of the logical block address.
    pub addr0: c_uchar,
    /// A reserved byte, byte 6 of the block.
    pub rsvd1: c_uchar,
    /// The most significant byte of the transfer length.
    pub count1: c_uchar,
    /// The least significant byte of the transfer length.
    pub count0: c_uchar,
    /// The control byte. Bit 0 is `link`, bit 1 is `flag`, bit 2 is `naca`,
    /// bits 3 to 5 are reserved, and bits 6 and 7 are the vendor unique
    /// `vu_96` and `vu_97`.
    pub control: c_uchar,
}

/// `scsi_setup_cdb(9F)`. illumos DDI specific (illumos DDI).
///
/// The group 4 part of a [`scsi_cdb`], `struct scsi_g4`, which follows the
/// first two bytes of a 16 byte block.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_g4 {
    /// The most significant byte of the logical block address.
    pub addr3: c_uchar,
    /// The second byte of the logical block address.
    pub addr2: c_uchar,
    /// The third byte of the logical block address.
    pub addr1: c_uchar,
    /// The fourth byte of the logical block address.
    pub addr0: c_uchar,
    /// The most significant byte of the additional command data, which for
    /// the common 16 byte commands holds the low half of a 64-bit logical
    /// block address.
    pub addtl_cdb_data3: c_uchar,
    /// The second byte of the additional command data.
    pub addtl_cdb_data2: c_uchar,
    /// The third byte of the additional command data.
    pub addtl_cdb_data1: c_uchar,
    /// The least significant byte of the additional command data.
    pub addtl_cdb_data0: c_uchar,
    /// The most significant byte of the transfer length.
    pub count3: c_uchar,
    /// The second byte of the transfer length.
    pub count2: c_uchar,
    /// The third byte of the transfer length.
    pub count1: c_uchar,
    /// The least significant byte of the transfer length.
    pub count0: c_uchar,
    /// A reserved byte.
    pub rsvd1: c_uchar,
    /// The control byte. Bit 0 is `link`, bit 1 is `flag`, bit 2 is `naca`,
    /// bits 3 to 5 are reserved, and bits 6 and 7 are the vendor unique
    /// `vu_156` and `vu_157`.
    pub control: c_uchar,
}

/// `scsi_setup_cdb(9F)`. illumos DDI specific (illumos DDI).
///
/// The group 5 part of a [`scsi_cdb`], `struct scsi_g5`, which follows the
/// first two bytes of a 12 byte block.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_g5 {
    /// The most significant byte of the logical block address.
    pub addr3: c_uchar,
    /// The second byte of the logical block address.
    pub addr2: c_uchar,
    /// The third byte of the logical block address.
    pub addr1: c_uchar,
    /// The least significant byte of the logical block address.
    pub addr0: c_uchar,
    /// The most significant byte of the transfer length.
    pub count3: c_uchar,
    /// The second byte of the transfer length.
    pub count2: c_uchar,
    /// The third byte of the transfer length.
    pub count1: c_uchar,
    /// The least significant byte of the transfer length.
    pub count0: c_uchar,
    /// A reserved byte.
    pub rsvd1: c_uchar,
    /// The control byte. Bit 0 is `link`, bit 1 is `flag`, bit 2 is `naca`,
    /// bits 3 to 5 are reserved, and bits 6 and 7 are the vendor unique
    /// `vu_116` and `vu_117`.
    pub control: c_uchar,
}

/// `scsi_setup_cdb(9F)`. illumos DDI specific (illumos DDI).
///
/// The group specific part of a [`scsi_cdb`], the anonymous union that C
/// names `sg`, which starts at byte 2 of the block.
#[repr(C)]
#[derive(Clone, Copy)]
pub union scsi_cdb_sg {
    /// The 14 bytes as a plain array.
    pub scsi: [c_uchar; 14],
    /// The group 0 layout.
    pub g0: scsi_g0,
    /// The group 1 and group 2 layout.
    pub g1: scsi_g1,
    /// The group 4 layout.
    pub g4: scsi_g4,
    /// The group 5 layout.
    pub g5: scsi_g5,
}

/// `scsi_setup_cdb(9F)`. illumos DDI specific (illumos DDI).
///
/// The structured view of a [`scsi_cdb`], the anonymous structure that C
/// names `cdb_un`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct scsi_cdb_un {
    /// The operation code, byte 0 of the block. Its top three bits give the
    /// command group.
    pub cmd: c_uchar,
    /// Byte 1 of the block. Bits 0 to 4 are `tag`, which holds the high bits
    /// of the address in group 0, the relative address bit in group 1 and
    /// the fixed bit of tape commands, and bits 5 to 7 are `lun`, the
    /// logical unit, which SCSI-3 reserves.
    pub byte1: c_uchar,
    /// The rest of the block, laid out according to its group.
    pub sg: scsi_cdb_sg,
}

/// `scsi_setup_cdb(9F)`. illumos DDI specific (illumos DDI).
///
/// A SCSI command descriptor block, `union scsi_cdb`.
///
/// It is 16 bytes long, the size of the longest group, and can be viewed as
/// structured fields, as bytes or as four 32-bit words. [`scsi_setup_cdb`]
/// fills one in. The header declares it in `sys/scsi/impl/commands.h`; see
/// the module documentation for why it is bound.
#[repr(C)]
#[derive(Clone, Copy)]
pub union scsi_cdb {
    /// The block as structured fields.
    pub cdb_un: scsi_cdb_un,
    /// The block as an array of bytes.
    pub cdb_opaque: [c_uchar; 16],
    /// The block as an array of words.
    pub cdb_long: [uint_t; 4],
}

// Layout: the group structures are bytes only, 4, 8, 14 and 10 bytes long,
// so the sg union is 14 bytes aligned to 1, and cdb_un is cmd at 0, byte1 at
// 1 and sg at 2, 16 bytes aligned to 1. The word array makes scsi_cdb 16
// bytes aligned to 4.
const _: () = assert!(core::mem::size_of::<scsi_g0>() == 4);
const _: () = assert!(core::mem::offset_of!(scsi_g0, addr1) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_g0, control) == 3);
const _: () = assert!(core::mem::size_of::<scsi_g1>() == 8);
const _: () = assert!(core::mem::offset_of!(scsi_g1, addr3) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_g1, control) == 7);
const _: () = assert!(core::mem::size_of::<scsi_g4>() == 14);
const _: () = assert!(core::mem::offset_of!(scsi_g4, addr3) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_g4, count3) == 8);
const _: () = assert!(core::mem::offset_of!(scsi_g4, control) == 13);
const _: () = assert!(core::mem::size_of::<scsi_g5>() == 10);
const _: () = assert!(core::mem::offset_of!(scsi_g5, addr3) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_g5, control) == 9);
const _: () = assert!(core::mem::size_of::<scsi_cdb_sg>() == 14);
const _: () = assert!(core::mem::size_of::<scsi_cdb_un>() == 16);
const _: () = assert!(core::mem::offset_of!(scsi_cdb_un, cmd) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_cdb_un, byte1) == 1);
const _: () = assert!(core::mem::offset_of!(scsi_cdb_un, sg) == 2);
const _: () = assert!(core::mem::size_of::<scsi_cdb>() == 16);
const _: () = assert!(core::mem::align_of::<scsi_cdb>() == 4);

// --- operation codes -----------------------------------------------------

/// TEST UNIT READY, group 0 (`SCMD_TEST_UNIT_READY`).
pub const SCMD_TEST_UNIT_READY: c_uchar = 0x00;
/// REZERO UNIT, group 0 (`SCMD_REZERO_UNIT`).
pub const SCMD_REZERO_UNIT: c_uchar = 0x01;
/// REWIND for sequential access devices, the same code as
/// [`SCMD_REZERO_UNIT`] (`SCMD_REWIND`).
pub const SCMD_REWIND: c_uchar = 0x01;
/// REQUEST SENSE, group 0 (`SCMD_REQUEST_SENSE`).
pub const SCMD_REQUEST_SENSE: c_uchar = 0x03;
/// FORMAT UNIT, group 0 (`SCMD_FORMAT`).
pub const SCMD_FORMAT: c_uchar = 0x04;
/// REASSIGN BLOCKS, group 0 (`SCMD_REASSIGN_BLOCK`).
pub const SCMD_REASSIGN_BLOCK: c_uchar = 0x07;
/// READ(6), group 0 (`SCMD_READ`).
pub const SCMD_READ: c_uchar = 0x08;
/// WRITE(6), group 0 (`SCMD_WRITE`).
pub const SCMD_WRITE: c_uchar = 0x0a;
/// SEEK(6), group 0 (`SCMD_SEEK`).
pub const SCMD_SEEK: c_uchar = 0x0b;
/// WRITE FILEMARKS for sequential access devices, group 0
/// (`SCMD_WRITE_FILE_MARK`).
pub const SCMD_WRITE_FILE_MARK: c_uchar = 0x10;
/// SPACE for sequential access devices, group 0 (`SCMD_SPACE`).
pub const SCMD_SPACE: c_uchar = 0x11;
/// INQUIRY, group 0 (`SCMD_INQUIRY`).
pub const SCMD_INQUIRY: c_uchar = 0x12;
/// MODE SELECT(6), group 0 (`SCMD_MODE_SELECT`).
pub const SCMD_MODE_SELECT: c_uchar = 0x15;
/// RESERVE(6), group 0 (`SCMD_RESERVE`).
pub const SCMD_RESERVE: c_uchar = 0x16;
/// RELEASE(6), group 0 (`SCMD_RELEASE`).
pub const SCMD_RELEASE: c_uchar = 0x17;
/// COPY, group 0 (`SCMD_COPY`).
pub const SCMD_COPY: c_uchar = 0x18;
/// ERASE for sequential access devices, group 0 (`SCMD_ERASE`).
pub const SCMD_ERASE: c_uchar = 0x19;
/// MODE SENSE(6), group 0 (`SCMD_MODE_SENSE`).
pub const SCMD_MODE_SENSE: c_uchar = 0x1a;
/// START STOP UNIT, group 0 (`SCMD_START_STOP`).
pub const SCMD_START_STOP: c_uchar = 0x1b;
/// LOAD UNLOAD for sequential access devices, the same code as
/// [`SCMD_START_STOP`] (`SCMD_LOAD`).
pub const SCMD_LOAD: c_uchar = 0x1b;
/// PREVENT ALLOW MEDIUM REMOVAL, the door lock command, group 0
/// (`SCMD_DOORLOCK`).
pub const SCMD_DOORLOCK: c_uchar = 0x1E;
/// READ CAPACITY(10), group 1 (`SCMD_READ_CAPACITY`).
pub const SCMD_READ_CAPACITY: c_uchar = 0x25;
/// READ(10), group 1 (`SCMD_READ_G1`).
pub const SCMD_READ_G1: c_uchar = 0x28;
/// WRITE(10), group 1 (`SCMD_WRITE_G1`).
pub const SCMD_WRITE_G1: c_uchar = 0x2a;
/// VERIFY(10), group 1 (`SCMD_VERIFY`).
pub const SCMD_VERIFY: c_uchar = 0x2f;
/// SEARCH DATA HIGH, group 1 (`SCMD_SEARCH_HIGH`).
pub const SCMD_SEARCH_HIGH: c_uchar = 0x30;
/// SEARCH DATA EQUAL, group 1 (`SCMD_SEARCH_EQUAL`).
pub const SCMD_SEARCH_EQUAL: c_uchar = 0x31;
/// SEARCH DATA LOW, group 1 (`SCMD_SEARCH_LOW`).
pub const SCMD_SEARCH_LOW: c_uchar = 0x32;
/// READ DEFECT DATA(10), group 1 (`SCMD_READ_DEFECT_LIST`).
pub const SCMD_READ_DEFECT_LIST: c_uchar = 0x37;
/// COMPARE, group 1 (`SCMD_COMPARE`).
pub const SCMD_COMPARE: c_uchar = 0x39;
/// COPY AND VERIFY, group 1 (`SCMD_COPY_VERIFY`).
pub const SCMD_COPY_VERIFY: c_uchar = 0x3A;
/// WRITE BUFFER, group 1 (`SCMD_WRITE_BUFFER`).
pub const SCMD_WRITE_BUFFER: c_uchar = 0x3B;
/// READ(16), group 4 (`SCMD_READ_G4`).
pub const SCMD_READ_G4: c_uchar = 0x88;
/// WRITE(16), group 4 (`SCMD_WRITE_G4`).
pub const SCMD_WRITE_G4: c_uchar = 0x8a;
/// REPORT LUNS, group 5 (`SCMD_REPORT_LUNS`).
pub const SCMD_REPORT_LUNS: c_uchar = 0xA0;
/// READ(12), group 5 (`SCMD_READ_G5`).
pub const SCMD_READ_G5: c_uchar = 0xA8;
/// WRITE(12), group 5 (`SCMD_WRITE_G5`).
pub const SCMD_WRITE_G5: c_uchar = 0xAA;

unsafe extern "C" {
    /// `scsi_setup_cdb(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Initialises the group 0, 1, 2, 4 or 5 command descriptor block at
    /// `cdbp`. `cmd` becomes the operation code, and its group decides the
    /// layout of the rest. `addr` is the logical block address of the data,
    /// or 0 for commands without addressing information, such as a group 0
    /// READ for a sequential access device. `cnt` is the transfer length in
    /// the units of the device type, which is bytes for sequential devices
    /// and blocks for block devices. `othr_cdb_data` is the additional data
    /// of a group 4 command and must be 0 for the other groups. Unlike the
    /// obsolete `makecom` functions, it does not set the logical unit bits in
    /// byte 1, nor the fixed bit of sequential access commands. Returns 1 on
    /// success and 0 on failure. It may be called from user, interrupt or
    /// kernel context.
    pub fn scsi_setup_cdb(cdbp: *mut scsi_cdb, cmd: c_uchar, addr: uint_t, cnt: uint_t, othr_cdb_data: uint_t)
    -> c_int;
}
