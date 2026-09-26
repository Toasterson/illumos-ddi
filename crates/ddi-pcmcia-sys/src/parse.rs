//! The tuple parsers from `sys/cs.h` and the union they share from
//! `sys/cis_handlers.h`.
//!
//! Once `csx_GetFirstTuple(9F)` or `csx_GetNextTuple(9F)` has positioned a
//! [`tuple_t`] on a standard tuple, a driver has Card Services decode it
//! rather than picking apart the raw bytes itself. There is one parser per
//! kind of tuple, each filling in the matching structure of
//! [`crate::cistpl`], and the generic [`csx_ParseTuple`], which fills in
//! the union [`cisparse_t`]. Each specific parser is a small function in
//! `cs_stubs.c` that stores its tuple code in
//! [`DesiredTuple`](crate::tuple::tuple_t::DesiredTuple) and then calls the
//! generic parser, so the [`tuple_t`] passed in is modified.
//!
//! Every parser returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS) when the
//! tuple was parsed, [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) when
//! the client handle is invalid,
//! [`CS_UNKNOWN_TUPLE`](crate::consts::CS_UNKNOWN_TUPLE) when the parser
//! does not know how to parse the tuple,
//! [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when there is no card in the
//! socket, [`CS_NO_CIS`](crate::consts::CS_NO_CIS) when the card has no
//! Card Information Structure, and
//! [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION) when
//! no PCMCIA hardware is installed. The generic parser may also return
//! [`CS_BAD_CIS`](crate::consts::CS_BAD_CIS) for a generic parser error.
//! Every parser may be called from user or kernel context.
//!
//! `sys/cs.h` also declares `csx_Parse_CISTPL_FORMAT_A`,
//! `csx_Parse_CISTPL_BAR` and `csx_Parse_CISTPL_LONGLINK_CB`, which no
//! manual page documents and which are not bound. Their tuples can still be
//! parsed with [`csx_ParseTuple`].

use crate::cistpl::*;
use crate::client::client_handle_t;
use crate::tuple::tuple_t;

// --- cisparse_t ----------------------------------------------------------

/// `csx_ParseTuple(9F)`. illumos DDI Specific (illumos DDI).
///
/// The union of all tuple parsing structures, which [`csx_ParseTuple`]
/// fills in for whatever tuple the [`tuple_t`] it is given points at.
/// `sys/cis_handlers.h` also gives the union a second, shorter name for
/// each member, such as `config` for `cistpl_config` and `cftable` for
/// `cistpl_cftable_entry`, marked for legacy support and for removal. The
/// manual page lists only the long names, and only those are declared; the
/// aliases do not change the layout.
#[repr(C)]
#[derive(Clone, Copy)]
pub union cisparse_t {
    /// A parsed `CISTPL_CONFIG` tuple.
    pub cistpl_config: cistpl_config_t,
    /// A parsed device information tuple.
    pub cistpl_device: cistpl_device_t,
    /// A parsed `CISTPL_VERS_1` tuple.
    pub cistpl_vers_1: cistpl_vers_1_t,
    /// A parsed `CISTPL_VERS_2` tuple.
    pub cistpl_vers_2: cistpl_vers_2_t,
    /// A parsed JEDEC identifier tuple.
    pub cistpl_jedec: cistpl_jedec_t,
    /// A parsed data recording format tuple.
    pub cistpl_format: cistpl_format_t,
    /// A parsed `CISTPL_GEOMETRY` tuple.
    pub cistpl_geometry: cistpl_geometry_t,
    /// A parsed `CISTPL_BYTEORDER` tuple.
    pub cistpl_byteorder: cistpl_byteorder_t,
    /// A parsed `CISTPL_DATE` tuple.
    pub cistpl_date: cistpl_date_t,
    /// A parsed `CISTPL_BATTERY` tuple.
    pub cistpl_battery: cistpl_battery_t,
    /// A parsed `CISTPL_ORG` tuple.
    pub cistpl_org: cistpl_org_t,
    /// A parsed `CISTPL_MANFID` tuple.
    pub cistpl_manfid: cistpl_manfid_t,
    /// A parsed `CISTPL_FUNCID` tuple.
    pub cistpl_funcid: cistpl_funcid_t,
    /// A parsed `CISTPL_FUNCE` tuple.
    pub cistpl_funce: cistpl_funce_t,
    /// A parsed `CISTPL_CFTABLE_ENTRY` tuple.
    pub cistpl_cftable_entry: cistpl_cftable_entry_t,
    /// A parsed `CISTPL_LINKTARGET` tuple.
    pub cistpl_linktarget: cistpl_linktarget_t,
    /// A parsed `CISTPL_LONGLINK_A` or `CISTPL_LONGLINK_C` tuple.
    pub cistpl_longlink_ac: cistpl_longlink_ac_t,
    /// A parsed `CISTPL_LONGLINK_MFC` tuple.
    pub cistpl_longlink_mfc: cistpl_longlink_mfc_t,
    /// A parsed `CISTPL_SPCL` tuple.
    pub cistpl_spcl: cistpl_spcl_t,
    /// A parsed `CISTPL_SWIL` tuple.
    pub cistpl_swil: cistpl_swil_t,
    /// A parsed `CISTPL_BAR` tuple.
    pub cistpl_bar: cistpl_bar_t,
    /// A parsed device geometry tuple.
    pub cistpl_devicegeo: cistpl_devicegeo_t,
    /// A parsed `CISTPL_LONGLINK_CB` tuple.
    pub cistpl_longlink_cb: cistpl_longlink_cb_t,
    /// The name of the tuple, returned when the tuple's attributes include
    /// [`TUPLE_RETURN_NAME`](crate::tuple::TUPLE_RETURN_NAME).
    pub cistpl_get_tuple_name: cistpl_get_tuple_name_t,
}

// Layout on amd64 with LP64. The largest member is cistpl_vers_1_t at 1028
// bytes, and cistpl_format_t, which holds a pointer, makes the alignment 8,
// so the union is padded from 1028 to 1032.
const _: () = assert!(core::mem::size_of::<cisparse_t>() == 1032);
const _: () = assert!(core::mem::align_of::<cisparse_t>() == 8);

unsafe extern "C" {
    /// `csx_ParseTuple(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// The generic tuple parser. It parses the tuple that the [`tuple_t`] at
    /// `tu`, returned by `csx_GetFirstTuple(9F)` or `csx_GetNextTuple(9F)`
    /// for the client `ch`, points at, and stores the result in the member
    /// of the [`cisparse_t`] at `cp` that matches the tuple. The argument
    /// `ef` carries extended tuple data that some tuples need, such as the
    /// function code for `CISTPL_FUNCE`. The page declares it as a
    /// `cisdata_t`, while `sys/cs.h` and the definition in `cs_stubs.c` take
    /// a `uint32_t`, which the declaration here follows. The return values
    /// are those listed in the module documentation, including
    /// [`CS_BAD_CIS`](crate::consts::CS_BAD_CIS), and the function may be
    /// called from user or kernel context.
    pub fn csx_ParseTuple(ch: client_handle_t, tu: *mut tuple_t, cp: *mut cisparse_t, ef: u32) -> i32;

    /// `csx_Parse_CISTPL_BATTERY(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Battery Replacement Date tuple, `CISTPL_BATTERY`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the
    /// dates of the last and the next battery replacement in the
    /// [`cistpl_battery_t`] at `cb`. The return values are those listed in
    /// the module documentation, and the function may be called from user
    /// or kernel context.
    pub fn csx_Parse_CISTPL_BATTERY(ch: client_handle_t, tu: *mut tuple_t, cb: *mut cistpl_battery_t) -> i32;
    /// `csx_Parse_CISTPL_BYTEORDER(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Byte Order tuple, `CISTPL_BYTEORDER`, that the [`tuple_t`]
    /// at `tu` points at for the client `ch`, and stores the byte order of
    /// multiple byte data and the byte mapping of a memory-like partition in
    /// the [`cistpl_byteorder_t`] at `cbo`. The return values are those
    /// listed in the module documentation, and the function may be called
    /// from user or kernel context.
    pub fn csx_Parse_CISTPL_BYTEORDER(ch: client_handle_t, tu: *mut tuple_t, cbo: *mut cistpl_byteorder_t) -> i32;
    /// `csx_Parse_CISTPL_CFTABLE_ENTRY(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the 16-bit Card Configuration Table Entry tuple,
    /// `CISTPL_CFTABLE_ENTRY`, that the [`tuple_t`] at `tu` points at for the
    /// client `ch`, and stores the interface, power, timing, I/O, interrupt,
    /// memory and miscellaneous requirements of that configuration in the
    /// [`cistpl_cftable_entry_t`] at `cft`. The return values are those
    /// listed in the module documentation, and the function may be called
    /// from user or kernel context.
    pub fn csx_Parse_CISTPL_CFTABLE_ENTRY(
        ch: client_handle_t,
        tu: *mut tuple_t,
        cft: *mut cistpl_cftable_entry_t,
    ) -> i32;
    /// `csx_Parse_CISTPL_CONFIG(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Configuration tuple, `CISTPL_CONFIG`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the
    /// configuration registers present, their offsets and base, and the last
    /// valid configuration index in the [`cistpl_config_t`] at `cc`. The
    /// return values are those listed in the module documentation, and the
    /// function may be called from user or kernel context.
    pub fn csx_Parse_CISTPL_CONFIG(ch: client_handle_t, tu: *mut tuple_t, cc: *mut cistpl_config_t) -> i32;
    /// `csx_Parse_CISTPL_DATE(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Card Initialization Date tuple, `CISTPL_DATE`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the time
    /// and date at which the card was formatted in the [`cistpl_date_t`] at
    /// `cd`. The return values are those listed in the module
    /// documentation, and the function may be called from user or kernel
    /// context.
    pub fn csx_Parse_CISTPL_DATE(ch: client_handle_t, tu: *mut tuple_t, cd: *mut cistpl_date_t) -> i32;
    /// `csx_Parse_CISTPL_DEVICE(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the 5 volt Device Information tuple for common memory,
    /// `CISTPL_DEVICE`, that the [`tuple_t`] at `tu` points at for the client
    /// `ch`, and stores the speed, size, type and write protection of each
    /// device in the [`cistpl_device_t`] at `cd`. The return values are those
    /// listed in the module documentation, and the function may be called
    /// from user or kernel context.
    pub fn csx_Parse_CISTPL_DEVICE(ch: client_handle_t, tu: *mut tuple_t, cd: *mut cistpl_device_t) -> i32;
    /// `csx_Parse_CISTPL_DEVICE(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the 5 volt Device Information tuple for attribute memory,
    /// `CISTPL_DEVICE_A`, that the [`tuple_t`] at `tu` points at for the
    /// client `ch`, and stores the speed, size, type and write protection of
    /// each device in the [`cistpl_device_t`] at `cd`. The return values are
    /// those listed in the module documentation, and the function may be
    /// called from user or kernel context.
    pub fn csx_Parse_CISTPL_DEVICE_A(ch: client_handle_t, tu: *mut tuple_t, cd: *mut cistpl_device_t) -> i32;
    /// `csx_Parse_CISTPL_DEVICE(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Other Condition Device Information tuple for attribute
    /// memory, `CISTPL_DEVICE_OA`, that the [`tuple_t`] at `tu` points at for
    /// the client `ch`, and stores the devices under that set of operating
    /// conditions, including the `CISTPL_DEVICE_OC_*` Vcc and MWAIT flags, in
    /// the [`cistpl_device_t`] at `cd`. The return values are those listed in
    /// the module documentation, and the function may be called from user
    /// or kernel context.
    pub fn csx_Parse_CISTPL_DEVICE_OA(ch: client_handle_t, tu: *mut tuple_t, cd: *mut cistpl_device_t) -> i32;
    /// `csx_Parse_CISTPL_DEVICE(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Other Condition Device Information tuple for common
    /// memory, `CISTPL_DEVICE_OC`, that the [`tuple_t`] at `tu` points at for
    /// the client `ch`, and stores the devices under that set of operating
    /// conditions, including the `CISTPL_DEVICE_OC_*` Vcc and MWAIT flags, in
    /// the [`cistpl_device_t`] at `cd`. The return values are those listed in
    /// the module documentation, and the function may be called from user
    /// or kernel context.
    pub fn csx_Parse_CISTPL_DEVICE_OC(ch: client_handle_t, tu: *mut tuple_t, cd: *mut cistpl_device_t) -> i32;
    /// `csx_Parse_CISTPL_DEVICEGEO(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Device Geometry tuple for common memory,
    /// `CISTPL_DEVICEGEO`, that the [`tuple_t`] at `tp` points at for the
    /// client `ch`, and stores the interface width, erase, read and write
    /// block sizes, partition subdivisions and interleave of each partition
    /// in the [`cistpl_devicegeo_t`] at `pt`. The return values are those
    /// listed in the module documentation, and the function may be called
    /// from user or kernel context.
    pub fn csx_Parse_CISTPL_DEVICEGEO(ch: client_handle_t, tp: *mut tuple_t, pt: *mut cistpl_devicegeo_t) -> i32;
    /// `csx_Parse_CISTPL_DEVICEGEO_A(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Device Geometry tuple for attribute memory,
    /// `CISTPL_DEVICEGEO_A`, that the [`tuple_t`] at `tp` points at for the
    /// client `ch`, and stores the geometry of each attribute memory
    /// partition in the [`cistpl_devicegeo_t`] at `pt`. The return values
    /// are those listed in the module documentation, and the function may be
    /// called from user or kernel context.
    pub fn csx_Parse_CISTPL_DEVICEGEO_A(ch: client_handle_t, tp: *mut tuple_t, pt: *mut cistpl_devicegeo_t) -> i32;
    /// `csx_Parse_CISTPL_FORMAT(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Data Recording Format tuple, `CISTPL_FORMAT`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the
    /// format type, error detection code and extent of the partition, with
    /// its disk-like or memory-like details, in the [`cistpl_format_t`] at
    /// `pt`. The return values are those listed in the module
    /// documentation, and the function may be called from user or kernel
    /// context.
    pub fn csx_Parse_CISTPL_FORMAT(ch: client_handle_t, tu: *mut tuple_t, pt: *mut cistpl_format_t) -> i32;
    /// `csx_Parse_CISTPL_FUNCE(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Function Extension tuple, `CISTPL_FUNCE`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the
    /// serial, modem, fax, voice or network details it gives in the
    /// [`cistpl_funce_t`] at `cf`. Because the meaning of the tuple depends
    /// on the function it extends, `fid` must hold the function code from
    /// the preceding `CISTPL_FUNCID` tuple, one of the `TPLFUNC_*` values.
    /// The return values are those listed in the module documentation, and
    /// the function may be called from user or kernel context.
    pub fn csx_Parse_CISTPL_FUNCE(ch: client_handle_t, tu: *mut tuple_t, cf: *mut cistpl_funce_t, fid: u32) -> i32;
    /// `csx_Parse_CISTPL_FUNCID(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Function Identification tuple, `CISTPL_FUNCID`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the
    /// function code of the card and its system initialisation mask in the
    /// [`cistpl_funcid_t`] at `cf`. The return values are those listed in
    /// the module documentation, and the function may be called from user
    /// or kernel context.
    pub fn csx_Parse_CISTPL_FUNCID(ch: client_handle_t, tu: *mut tuple_t, cf: *mut cistpl_funcid_t) -> i32;
    /// `csx_Parse_CISTPL_GEOMETRY(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Geometry tuple, `CISTPL_GEOMETRY`, that the [`tuple_t`] at
    /// `tu` points at for the client `ch`, and stores the sectors per track,
    /// tracks per cylinder and number of cylinders of a disk-like device in
    /// the [`cistpl_geometry_t`] at `pt`. The return values are those listed
    /// in the module documentation, and the function may be called from user
    /// or kernel context.
    pub fn csx_Parse_CISTPL_GEOMETRY(ch: client_handle_t, tu: *mut tuple_t, pt: *mut cistpl_geometry_t) -> i32;
    /// `csx_Parse_CISTPL_JEDEC_C(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the JEDEC Identifier tuple for common memory,
    /// `CISTPL_JEDEC_C`, that the [`tuple_t`] at `tu` points at for the client
    /// `ch`, and stores the manufacturer identifiers and information of the
    /// programmable devices in the [`cistpl_jedec_t`] at `cj`. The return
    /// values are those listed in the module documentation, and the function
    /// may be called from user or kernel context.
    pub fn csx_Parse_CISTPL_JEDEC_C(ch: client_handle_t, tu: *mut tuple_t, cj: *mut cistpl_jedec_t) -> i32;
    /// `csx_Parse_CISTPL_JEDEC_C(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the JEDEC Identifier tuple for attribute memory,
    /// `CISTPL_JEDEC_A`, that the [`tuple_t`] at `tu` points at for the client
    /// `ch`, and stores the manufacturer identifiers and information of the
    /// programmable devices in the [`cistpl_jedec_t`] at `cj`. The return
    /// values are those listed in the module documentation, and the function
    /// may be called from user or kernel context.
    pub fn csx_Parse_CISTPL_JEDEC_A(ch: client_handle_t, tu: *mut tuple_t, cj: *mut cistpl_jedec_t) -> i32;
    /// `csx_Parse_CISTPL_LINKTARGET(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Link Target tuple, `CISTPL_LINKTARGET`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the tag
    /// that validates a secondary tuple chain and its length in the
    /// [`cistpl_linktarget_t`] at `pt`. The return values are those listed
    /// in the module documentation, and the function may be called from user
    /// or kernel context.
    pub fn csx_Parse_CISTPL_LINKTARGET(ch: client_handle_t, tu: *mut tuple_t, pt: *mut cistpl_linktarget_t) -> i32;
    /// `csx_Parse_CISTPL_LONGLINK_A(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Long Link tuple to attribute memory, `CISTPL_LONGLINK_A`,
    /// that the [`tuple_t`] at `tu` points at for the client `ch`, and stores
    /// the address space and offset of the linked chain in the
    /// [`cistpl_longlink_ac_t`] at `pt`. The return values are those listed
    /// in the module documentation, and the function may be called from user
    /// or kernel context.
    pub fn csx_Parse_CISTPL_LONGLINK_A(ch: client_handle_t, tu: *mut tuple_t, pt: *mut cistpl_longlink_ac_t) -> i32;
    /// `csx_Parse_CISTPL_LONGLINK_A(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Long Link tuple to common memory, `CISTPL_LONGLINK_C`, that
    /// the [`tuple_t`] at `tu` points at for the client `ch`, and stores the
    /// address space and offset of the linked chain in the
    /// [`cistpl_longlink_ac_t`] at `pt`. The return values are those listed
    /// in the module documentation, and the function may be called from user
    /// or kernel context.
    pub fn csx_Parse_CISTPL_LONGLINK_C(ch: client_handle_t, tu: *mut tuple_t, pt: *mut cistpl_longlink_ac_t) -> i32;
    /// `csx_Parse_CISTPL_LONGLINK_MFC(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Multi-Function tuple, `CISTPL_LONGLINK_MFC`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the
    /// number of functions and configuration register sets and the start of
    /// the CIS of each function in the [`cistpl_longlink_mfc_t`] at `pt`.
    /// The return values are those listed in the module documentation, and
    /// the function may be called from user or kernel context.
    pub fn csx_Parse_CISTPL_LONGLINK_MFC(ch: client_handle_t, tu: *mut tuple_t, pt: *mut cistpl_longlink_mfc_t) -> i32;
    /// `csx_Parse_CISTPL_MANFID(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Manufacturer Identification tuple, `CISTPL_MANFID`, that
    /// the [`tuple_t`] at `tu` points at for the client `ch`, and stores the
    /// PCMCIA manufacturer code and the manufacturer's card number in the
    /// [`cistpl_manfid_t`] at `cm`. The return values are those listed in the
    /// module documentation, and the function may be called from user or
    /// kernel context.
    pub fn csx_Parse_CISTPL_MANFID(ch: client_handle_t, tu: *mut tuple_t, cm: *mut cistpl_manfid_t) -> i32;
    /// `csx_Parse_CISTPL_ORG(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Data Organization tuple, `CISTPL_ORG`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the data
    /// organisation code and its text description in the [`cistpl_org_t`] at
    /// `pt`. The return values are those listed in the module documentation,
    /// and the function may be called from user or kernel context.
    pub fn csx_Parse_CISTPL_ORG(ch: client_handle_t, tu: *mut tuple_t, pt: *mut cistpl_org_t) -> i32;
    /// `csx_Parse_CISTPL_SPCL(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Special Purpose tuple, `CISTPL_SPCL`, that the [`tuple_t`]
    /// at `tu` points at for the client `ch`, and stores its identifier,
    /// sequence number and data in the [`cistpl_spcl_t`] at `csp`. The
    /// return values are those listed in the module documentation, and the
    /// function may be called from user or kernel context.
    pub fn csx_Parse_CISTPL_SPCL(ch: client_handle_t, tu: *mut tuple_t, csp: *mut cistpl_spcl_t) -> i32;
    /// `csx_Parse_CISTPL_SWIL(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Software Interleaving tuple, `CISTPL_SWIL`, that the
    /// [`tuple_t`] at `tu` points at for the client `ch`, and stores the
    /// software interleave of the partition in the [`cistpl_swil_t`] at
    /// `pt`. The return values are those listed in the module documentation,
    /// and the function may be called from user or kernel context.
    pub fn csx_Parse_CISTPL_SWIL(ch: client_handle_t, tu: *mut tuple_t, pt: *mut cistpl_swil_t) -> i32;
    /// `csx_Parse_CISTPL_VERS_1(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Level-1 Version/Product Information tuple,
    /// `CISTPL_VERS_1`, that the [`tuple_t`] at `tu` points at for the client
    /// `ch`, and stores the version the card complies with and the
    /// manufacturer's product information strings in the
    /// [`cistpl_vers_1_t`] at `cv1`. The return values are those listed in
    /// the module documentation, and the function may be called from user or
    /// kernel context.
    pub fn csx_Parse_CISTPL_VERS_1(ch: client_handle_t, tu: *mut tuple_t, cv1: *mut cistpl_vers_1_t) -> i32;
    /// `csx_Parse_CISTPL_VERS_2(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Parses the Level-2 Version and Information tuple, `CISTPL_VERS_2`,
    /// that the [`tuple_t`] at `tu` points at for the client `ch`, and stores
    /// the version, compliance level, location of the first data byte,
    /// number of CIS copies and the formatting vendor and informational
    /// strings in the [`cistpl_vers_2_t`] at `cv2`. The return values are
    /// those listed in the module documentation, and the function may be
    /// called from user or kernel context.
    pub fn csx_Parse_CISTPL_VERS_2(ch: client_handle_t, tu: *mut tuple_t, cv2: *mut cistpl_vers_2_t) -> i32;
}
