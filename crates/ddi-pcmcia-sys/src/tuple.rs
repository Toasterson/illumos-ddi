//! The tuple walk over the Card Information Structure, from `sys/cs.h` and
//! `sys/cis.h`.
//!
//! A card describes itself in its Card Information Structure, the CIS,
//! which is a linked list of tuples called a CIS chain. Each tuple has a
//! one byte type code, one of the `CISTPL_*` values declared here, and a one
//! byte link, the offset of the next tuple. A card may have more than one
//! chain. A multiple function card that follows the PC Card 95
//! MultiFunction Metaformat has a global CIS and a function specific CIS for
//! each function, and illumos returns the tuples of the global CIS followed
//! by those of the caller's own function, so a driver need not care which
//! chain a tuple is in.
//!
//! A driver finds a tuple with [`csx_GetFirstTuple`] and
//! [`csx_GetNextTuple`], either the next tuple of any type or the next of a
//! particular type. It then either reads the raw bytes with
//! [`csx_GetTupleData`] or, for a standard tuple, has Card Services decode
//! it with one of the parsers in [`crate::parse`]. [`csx_ValidateCIS`]
//! checks the CIS as a whole. All of these share one [`tuple_t`], which
//! carries the position in the CIS from one call to the next.
//!
//! Every tuple code of `sys/cis.h` is declared, including the vendor
//! specific codes from `0x80` to `0x8f`. The CIS interpreter internals of
//! the header, such as `cisptr_t`, the `CISTPLF_*` list flags and the
//! `GET_*` macros, are private to Card Services and left out.

use crate::client::client_handle_t;

/// `tuple(9S)`. illumos DDI Specific (illumos DDI).
///
/// One byte of CIS data, an `unsigned char` in C. Tuple codes, tuple links
/// and the raw tuple body are all of this type.
pub type cisdata_t = u8;

ddi_sys::opaque! {
    /// `tuple(9S)`. illumos DDI Specific (illumos DDI).
    ///
    /// An entry in the list of tuples that Card Services keeps for each
    /// card, `cistpl_t`. The [`tuple_t`] members that point to it are
    /// private to Card Services, and so is its layout.
    cistpl_t;
}

/// The largest size of the data body of a tuple (`CIS_MAX_TUPLE_DATA_LEN`).
pub const CIS_MAX_TUPLE_DATA_LEN: usize = 254;

// --- tuple_t -------------------------------------------------------------

/// `tuple(9S)`. illumos DDI Specific (illumos DDI).
///
/// The basic structure through which a driver reads the Card Information
/// Structure of its card. [`csx_GetFirstTuple`] and [`csx_GetNextTuple`]
/// fill it in with the tuple found and the position reached,
/// [`csx_GetTupleData`] adds the raw body, and the parsers of
/// [`crate::parse`] decode the tuple it points at. Because all these
/// functions share the structure, members a call does not use must be left
/// as the previous call set them.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct tuple_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// Which tuples to return, a combination of [`TUPLE_RETURN_LINK`],
    /// [`TUPLE_RETURN_IGNORED_TUPLES`] and [`TUPLE_RETURN_NAME`]. It is set
    /// before [`csx_GetFirstTuple`] and must not be changed before
    /// [`csx_GetTupleData`].
    pub Attributes: u32,
    /// The tuple code to look for, or [`RETURN_FIRST_TUPLE`] or
    /// [`RETURN_NEXT_TUPLE`] for the first or next tuple of any type. It must
    /// not be changed before [`csx_GetTupleData`].
    pub DesiredTuple: cisdata_t,
    /// The offset within the tuple body at which [`csx_GetTupleData`]
    /// starts, which allows a partial read. It must only be set before
    /// that call.
    pub TupleOffset: cisdata_t,
    /// Private to Card Services.
    pub Flags: u32,
    /// Private to Card Services.
    pub LinkOffset: *mut cistpl_t,
    /// Private to Card Services.
    pub CISOffset: *mut cistpl_t,
    /// The size of the buffer in which [`csx_GetTupleData`] returns the raw
    /// body. It may be larger than the body. Card Services ignores any value
    /// the client puts here.
    pub TupleDataMax: cisdata_t,
    /// The size of the tuple body, that is the number of body bytes that
    /// [`csx_GetTupleData`] returned.
    pub TupleDataLen: cisdata_t,
    /// The raw contents of the tuple body, as returned by
    /// [`csx_GetTupleData`].
    pub TupleData: [cisdata_t; CIS_MAX_TUPLE_DATA_LEN],
    /// The type code of the tuple found by [`csx_GetFirstTuple`] or
    /// [`csx_GetNextTuple`].
    pub TupleCode: cisdata_t,
    /// The link of the tuple found, the offset to the next tuple, which is
    /// also the size of the tuple body.
    pub TupleLink: cisdata_t,
}

// Layout on amd64 with LP64. Socket 4 at 0, Attributes 4 at 4, DesiredTuple
// 1 at 8, TupleOffset 1 at 9 and 2 bytes of padding, Flags 4 at 12,
// LinkOffset 8 at 16, CISOffset 8 at 24, TupleDataMax 1 at 32, TupleDataLen
// 1 at 33, TupleData 254 at 34, TupleCode 1 at 288 and TupleLink 1 at 289,
// which ends at 290 and is padded to 296 for the pointers.
const _: () = assert!(core::mem::size_of::<tuple_t>() == 296);
const _: () = assert!(core::mem::offset_of!(tuple_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(tuple_t, DesiredTuple) == 8);
const _: () = assert!(core::mem::offset_of!(tuple_t, Flags) == 12);
const _: () = assert!(core::mem::offset_of!(tuple_t, LinkOffset) == 16);
const _: () = assert!(core::mem::offset_of!(tuple_t, CISOffset) == 24);
const _: () = assert!(core::mem::offset_of!(tuple_t, TupleDataMax) == 32);
const _: () = assert!(core::mem::offset_of!(tuple_t, TupleData) == 34);
const _: () = assert!(core::mem::offset_of!(tuple_t, TupleCode) == 288);
const _: () = assert!(core::mem::offset_of!(tuple_t, TupleLink) == 289);

/// Return link tuples too. Without it [`CISTPL_NULL`],
/// [`CISTPL_LONGLINK_A`], [`CISTPL_LONGLINK_C`], [`CISTPL_LONGLINK_CB`],
/// [`CISTPL_LONGLINK_MFC`], [`CISTPL_LINKTARGET`], [`CISTPL_NO_LINK`] and
/// [`CISTPL_END`] are skipped (`TUPLE_RETURN_LINK`).
pub const TUPLE_RETURN_LINK: u32 = 0x0000_0002;
/// Return ignored tuples too. Ignored tuples are those in the global CIS of
/// a multiple function card that a function specific CIS repeats
/// (`TUPLE_RETURN_IGNORED_TUPLES`).
pub const TUPLE_RETURN_IGNORED_TUPLES: u32 = 0x0001_0000;
/// Have `csx_ParseTuple(9F)` return the name of the tuple
/// (`TUPLE_RETURN_NAME`).
pub const TUPLE_RETURN_NAME: u32 = 0x0002_0000;

/// The value of [`tuple_t::DesiredTuple`] that returns the very first tuple
/// of the CIS, whatever its type. It shares the value of [`CISTPL_END`],
/// which is never returned (`RETURN_FIRST_TUPLE`).
pub const RETURN_FIRST_TUPLE: cisdata_t = 0xff;
/// The value of [`tuple_t::DesiredTuple`] that returns the very next tuple
/// of the CIS, whatever its type. It shares the value of [`CISTPL_END`],
/// which is never returned (`RETURN_NEXT_TUPLE`).
pub const RETURN_NEXT_TUPLE: cisdata_t = 0xff;

// --- cisinfo_t -----------------------------------------------------------

/// `csx_ValidateCIS(9F)`. illumos DDI Specific (illumos DDI).
///
/// The result of [`csx_ValidateCIS`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct cisinfo_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// The number of valid tuple chains found in the CIS. Zero means that
    /// the CIS is not valid.
    pub Chains: u32,
    /// An illumos extension giving the total number of tuples on all chains
    /// of the CIS.
    pub Tuples: u32,
}

// Layout on amd64 with LP64. Three uint32_t at 0, 4 and 8; total 12.
const _: () = assert!(core::mem::size_of::<cisinfo_t>() == 12);
const _: () = assert!(core::mem::offset_of!(cisinfo_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(cisinfo_t, Tuples) == 8);

// --- tuple codes ---------------------------------------------------------

/// The null tuple, which is ignored. It is a link tuple (`CISTPL_NULL`).
pub const CISTPL_NULL: cisdata_t = 0x00;
/// Device information for common memory (`CISTPL_DEVICE`).
pub const CISTPL_DEVICE: cisdata_t = 0x01;
/// A long link to the next tuple chain of a CardBus card
/// (`CISTPL_LONGLINK_CB`).
pub const CISTPL_LONGLINK_CB: cisdata_t = 0x02;
/// The configuration of a CardBus card (`CISTPL_CONFIG_CB`).
pub const CISTPL_CONFIG_CB: cisdata_t = 0x04;
/// A configuration table entry of a CardBus card
/// (`CISTPL_CFTABLE_ENTRY_CB`).
pub const CISTPL_CFTABLE_ENTRY_CB: cisdata_t = 0x05;
/// The start of the function specific CIS of each function of a multiple
/// function card (`CISTPL_LONGLINK_MFC`).
pub const CISTPL_LONGLINK_MFC: cisdata_t = 0x06;
/// A CardBus base address register definition (`CISTPL_BAR`).
pub const CISTPL_BAR: cisdata_t = 0x07;
/// Checksum control (`CISTPL_CHECKSUM`).
pub const CISTPL_CHECKSUM: cisdata_t = 0x10;
/// A long link to attribute memory (`CISTPL_LONGLINK_A`).
pub const CISTPL_LONGLINK_A: cisdata_t = 0x11;
/// A long link to common memory (`CISTPL_LONGLINK_C`).
pub const CISTPL_LONGLINK_C: cisdata_t = 0x12;
/// The link target that starts every secondary chain
/// (`CISTPL_LINKTARGET`).
pub const CISTPL_LINKTARGET: cisdata_t = 0x13;
/// No link to a further chain (`CISTPL_NO_LINK`).
pub const CISTPL_NO_LINK: cisdata_t = 0x14;
/// Level 1 version and product information (`CISTPL_VERS_1`).
pub const CISTPL_VERS_1: cisdata_t = 0x15;
/// An alternate language string (`CISTPL_ALTSTR`).
pub const CISTPL_ALTSTR: cisdata_t = 0x16;
/// Device information for attribute memory (`CISTPL_DEVICE_A`).
pub const CISTPL_DEVICE_A: cisdata_t = 0x17;
/// JEDEC programming information for common memory (`CISTPL_JEDEC_C`).
pub const CISTPL_JEDEC_C: cisdata_t = 0x18;
/// JEDEC programming information for attribute memory (`CISTPL_JEDEC_A`).
pub const CISTPL_JEDEC_A: cisdata_t = 0x19;
/// The configuration of a 16-bit card (`CISTPL_CONFIG`).
pub const CISTPL_CONFIG: cisdata_t = 0x1a;
/// A configuration table entry of a 16-bit card (`CISTPL_CFTABLE_ENTRY`).
pub const CISTPL_CFTABLE_ENTRY: cisdata_t = 0x1b;
/// Device information for common memory under other operating conditions
/// (`CISTPL_DEVICE_OC`).
pub const CISTPL_DEVICE_OC: cisdata_t = 0x1c;
/// Device information for attribute memory under other operating
/// conditions (`CISTPL_DEVICE_OA`).
pub const CISTPL_DEVICE_OA: cisdata_t = 0x1d;
/// The device geometry of common memory (`CISTPL_DEVICEGEO`).
pub const CISTPL_DEVICEGEO: cisdata_t = 0x1e;
/// The device geometry of attribute memory (`CISTPL_DEVICEGEO_A`).
pub const CISTPL_DEVICEGEO_A: cisdata_t = 0x1f;
/// Manufacturer identification (`CISTPL_MANFID`).
pub const CISTPL_MANFID: cisdata_t = 0x20;
/// Function identification (`CISTPL_FUNCID`).
pub const CISTPL_FUNCID: cisdata_t = 0x21;
/// A function extension (`CISTPL_FUNCE`).
pub const CISTPL_FUNCE: cisdata_t = 0x22;
/// Software interleave (`CISTPL_SWIL`).
pub const CISTPL_SWIL: cisdata_t = 0x23;
/// Level 2 version information (`CISTPL_VERS_2`).
pub const CISTPL_VERS_2: cisdata_t = 0x40;
/// The data recording format of common memory (`CISTPL_FORMAT`).
pub const CISTPL_FORMAT: cisdata_t = 0x41;
/// The geometry of a disk-like device (`CISTPL_GEOMETRY`).
pub const CISTPL_GEOMETRY: cisdata_t = 0x42;
/// Byte order (`CISTPL_BYTEORDER`).
pub const CISTPL_BYTEORDER: cisdata_t = 0x43;
/// The card initialisation date (`CISTPL_DATE`).
pub const CISTPL_DATE: cisdata_t = 0x44;
/// The battery replacement date (`CISTPL_BATTERY`).
pub const CISTPL_BATTERY: cisdata_t = 0x45;
/// Data organisation (`CISTPL_ORG`).
pub const CISTPL_ORG: cisdata_t = 0x46;
/// The data recording format of attribute memory (`CISTPL_FORMAT_A`).
pub const CISTPL_FORMAT_A: cisdata_t = 0x47;
/// Vendor specific tuple 0x80 (`CISTPL_VEND_SPEC_80`).
pub const CISTPL_VEND_SPEC_80: cisdata_t = 0x80;
/// Vendor specific tuple 0x81 (`CISTPL_VEND_SPEC_81`).
pub const CISTPL_VEND_SPEC_81: cisdata_t = 0x81;
/// Vendor specific tuple 0x82 (`CISTPL_VEND_SPEC_82`).
pub const CISTPL_VEND_SPEC_82: cisdata_t = 0x82;
/// Vendor specific tuple 0x83 (`CISTPL_VEND_SPEC_83`).
pub const CISTPL_VEND_SPEC_83: cisdata_t = 0x83;
/// Vendor specific tuple 0x84 (`CISTPL_VEND_SPEC_84`).
pub const CISTPL_VEND_SPEC_84: cisdata_t = 0x84;
/// Vendor specific tuple 0x85 (`CISTPL_VEND_SPEC_85`).
pub const CISTPL_VEND_SPEC_85: cisdata_t = 0x85;
/// Vendor specific tuple 0x86 (`CISTPL_VEND_SPEC_86`).
pub const CISTPL_VEND_SPEC_86: cisdata_t = 0x86;
/// Vendor specific tuple 0x87 (`CISTPL_VEND_SPEC_87`).
pub const CISTPL_VEND_SPEC_87: cisdata_t = 0x87;
/// Vendor specific tuple 0x88 (`CISTPL_VEND_SPEC_88`).
pub const CISTPL_VEND_SPEC_88: cisdata_t = 0x88;
/// Vendor specific tuple 0x89 (`CISTPL_VEND_SPEC_89`).
pub const CISTPL_VEND_SPEC_89: cisdata_t = 0x89;
/// Vendor specific tuple 0x8a (`CISTPL_VEND_SPEC_8a`).
pub const CISTPL_VEND_SPEC_8a: cisdata_t = 0x8a;
/// Vendor specific tuple 0x8b (`CISTPL_VEND_SPEC_8b`).
pub const CISTPL_VEND_SPEC_8b: cisdata_t = 0x8b;
/// Vendor specific tuple 0x8c (`CISTPL_VEND_SPEC_8c`).
pub const CISTPL_VEND_SPEC_8c: cisdata_t = 0x8c;
/// Vendor specific tuple 0x8d (`CISTPL_VEND_SPEC_8d`).
pub const CISTPL_VEND_SPEC_8d: cisdata_t = 0x8d;
/// Vendor specific tuple 0x8e (`CISTPL_VEND_SPEC_8e`).
pub const CISTPL_VEND_SPEC_8e: cisdata_t = 0x8e;
/// Vendor specific tuple 0x8f (`CISTPL_VEND_SPEC_8f`).
pub const CISTPL_VEND_SPEC_8f: cisdata_t = 0x8f;
/// The special purpose tuple, whose contents depend on its header
/// (`CISTPL_SPCL`).
pub const CISTPL_SPCL: cisdata_t = 0x90;
/// The end of a chain. Card Services never returns it as a found tuple,
/// which is why its value doubles as [`RETURN_FIRST_TUPLE`] (`CISTPL_END`).
pub const CISTPL_END: cisdata_t = 0xff;

unsafe extern "C" {
    /// `csx_GetFirstTuple(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Finds the first tuple of the type in
    /// [`DesiredTuple`](tuple_t::DesiredTuple), or the very first tuple for
    /// [`RETURN_FIRST_TUPLE`], in the CIS of the card of the client `ch`,
    /// and records its code, link and position in the [`tuple_t`] at `tu`.
    /// Members the call does not use must not be initialised by the client.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the socket is
    /// empty, [`CS_NO_CIS`](crate::consts::CS_NO_CIS) when the card has no
    /// CIS, [`CS_NO_MORE_ITEMS`](crate::consts::CS_NO_MORE_ITEMS) when no
    /// such tuple exists, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_GetFirstTuple(ch: client_handle_t, tu: *mut tuple_t) -> i32;
    /// `csx_GetFirstTuple(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Finds the next tuple of the type in
    /// [`DesiredTuple`](tuple_t::DesiredTuple), or the very next tuple for
    /// [`RETURN_NEXT_TUPLE`], after the position recorded in the
    /// [`tuple_t`] at `tu` by an earlier call, and records it there.
    ///
    /// The return values are those of [`csx_GetFirstTuple`]. It may be
    /// called from user or kernel context.
    pub fn csx_GetNextTuple(ch: client_handle_t, tu: *mut tuple_t) -> i32;
    /// `csx_GetTupleData(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Copies the body of the tuple that [`csx_GetFirstTuple`] or
    /// [`csx_GetNextTuple`] found into [`TupleData`](tuple_t::TupleData) of
    /// the [`tuple_t`] at `tu`, starting at
    /// [`TupleOffset`](tuple_t::TupleOffset), and sets
    /// [`TupleDataLen`](tuple_t::TupleDataLen).
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_BAD_ARGS`](crate::consts::CS_BAD_ARGS) when the data
    /// left by the earlier call is corrupt,
    /// [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the socket is empty,
    /// [`CS_NO_CIS`](crate::consts::CS_NO_CIS) when the card has no CIS,
    /// [`CS_NO_MORE_ITEMS`](crate::consts::CS_NO_MORE_ITEMS) when the tuple
    /// could not be read from the card, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_GetTupleData(ch: client_handle_t, tu: *mut tuple_t) -> i32;
    /// `csx_ValidateCIS(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Validates the Card Information Structure of the card of the client
    /// `ch` and reports the number of chains and tuples in the
    /// [`cisinfo_t`] at `ci`.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_NO_CIS`](crate::consts::CS_NO_CIS) when the card has no CIS or
    /// the CIS is invalid, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the
    /// socket is empty, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_ValidateCIS(ch: client_handle_t, ci: *mut cisinfo_t) -> i32;
}
