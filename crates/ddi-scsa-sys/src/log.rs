//! Messages and names from `sys/scsi/impl/services.h`.
//!
//! [`scsi_log`] prints a message about a SCSI device through `cmn_err(9F)`.
//! [`scsi_errmsg`] and [`scsi_vu_errmsg`] turn a failed packet and its
//! sense data into the standard multi line error report that SCSI target
//! drivers print, and the `scsi_*name` functions translate command codes,
//! device types, messages, completion reasons and sense keys into text.
//!
//! The prototypes and the two lookup structures live in
//! `sys/scsi/impl/services.h`, which `sys/scsi/scsi.h` includes. That header
//! is not public as a whole, but the parts bound here are documented by the
//! manual pages `scsi_log(9F)`, `scsi_errmsg(9F)`, `scsi_vu_errmsg(9F)`,
//! `scsi_cname(9F)` and `scsi_asc_key_strings(9S)`.
//!
//! The header also declares `scsi_cmd_name`, `scsi_esname`,
//! `scsi_asc_name`, `scsi_generic_errmsg`, the `scsi_state_bits` and
//! `sense_keys` variables, and the `SCSI_ERR_NONE` severity. None of them
//! has a manual page, so they are not bound. The `scsi_cmd_decode` macro is
//! another name for [`scsi_cname`].

use ddi_sys::{c_char, c_int, c_uchar, daddr_t, dev_info_t, uint_t, ushort_t};

use crate::device::scsi_device;
use crate::pkt::scsi_pkt;
use crate::sense::scsi_extended_sense;

/// Print the message on the console only, for debugging. It is passed as
/// the `level` of [`scsi_log`] in place of a `CE_*` level (`SCSI_DEBUG`).
pub const SCSI_DEBUG: uint_t = 0xDEB0_0000;

/// The error report shows the level "All" (`SCSI_ERR_ALL`).
pub const SCSI_ERR_ALL: c_int = 0;
/// The error report shows the level "Unknown" (`SCSI_ERR_UNKNOWN`).
pub const SCSI_ERR_UNKNOWN: c_int = 1;
/// The error report shows the level "Informational" (`SCSI_ERR_INFO`).
pub const SCSI_ERR_INFO: c_int = 2;
/// The error report shows the level "Recovered" (`SCSI_ERR_RECOVERED`).
pub const SCSI_ERR_RECOVERED: c_int = 3;
/// The error report shows the level "Retryable" (`SCSI_ERR_RETRYABLE`).
pub const SCSI_ERR_RETRYABLE: c_int = 4;
/// The error report shows the level "Fatal" (`SCSI_ERR_FATAL`).
pub const SCSI_ERR_FATAL: c_int = 5;

/// `scsi_errmsg(9F)`. illumos DDI specific (illumos DDI).
///
/// One entry of the table that maps SCSI command codes to their names for
/// [`scsi_errmsg`] and [`scsi_vu_errmsg`], `struct scsi_key_strings`. The
/// table is an array of these entries ending with one whose `key` is -1 and
/// whose `message` is NULL.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_key_strings {
    /// The command code, or -1 in the entry that ends the table.
    pub key: c_int,
    /// The name of the command, a NUL terminated string, or NULL in the
    /// entry that ends the table.
    pub message: *mut c_char,
}

// Layout on amd64 with LP64. key 4 at 0 and 4 bytes of padding, message 8 at
// 8; total 16.
const _: () = assert!(core::mem::size_of::<scsi_key_strings>() == 16);
const _: () = assert!(core::mem::offset_of!(scsi_key_strings, key) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_key_strings, message) == 8);

/// `scsi_asc_key_strings(9S)`. illumos DDI specific (illumos DDI).
///
/// One entry of a table that maps an additional sense code and qualifier
/// to a message, which [`scsi_vu_errmsg`] consults for vendor unique codes.
/// The manual page is named after the structure's old name; the header
/// calls it `struct scsi_asq_key_strings`. The table ends with an entry
/// whose `asc` is -1, stored as 0xffff in the unsigned member.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_asq_key_strings {
    /// The additional sense code, ASC.
    pub asc: ushort_t,
    /// The additional sense code qualifier, ASCQ.
    pub ascq: ushort_t,
    /// A NUL terminated string that describes the condition.
    pub message: *mut c_char,
}

// Layout on amd64 with LP64. asc 2 at 0, ascq 2 at 2 and 4 bytes of padding,
// message 8 at 8; total 16.
const _: () = assert!(core::mem::size_of::<scsi_asq_key_strings>() == 16);
const _: () = assert!(core::mem::offset_of!(scsi_asq_key_strings, asc) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_asq_key_strings, ascq) == 2);
const _: () = assert!(core::mem::offset_of!(scsi_asq_key_strings, message) == 8);

/// `scsi_vu_errmsg(9F)`. illumos DDI specific (illumos DDI).
///
/// The signature of the `decode_fru` function that [`scsi_vu_errmsg`]
/// takes, `char *(*)(struct scsi_device *, char *, int, uchar_t)` in C. It
/// is called after the whole of the sense information has been decoded,
/// with the device, a buffer, the length of that buffer and the field
/// replaceable unit code. It returns a string that describes the unit, which
/// may be written into the buffer, or NULL if it cannot decode the code.
pub type scsi_decode_fru_t =
    unsafe extern "C" fn(devp: *mut scsi_device, buf: *mut c_char, len: c_int, fru_code: c_uchar) -> *mut c_char;

unsafe extern "C" {
    /// `scsi_log(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Displays a message about a SCSI device through `cmn_err(9F)`, with the
    /// same formatting conversions. `dip` is the device node the message is
    /// about; if it is NULL, the message carries `drv_name` without a unit
    /// number or long name. `drv_name` is the short name by which the device
    /// is known, such as `sd`. `level` is `CE_PANIC`, `CE_WARN`, `CE_NOTE`,
    /// `CE_CONT` or [`SCSI_DEBUG`], the last of which prints debug messages
    /// on the console only. If the format starts with an exclamation mark
    /// the message goes only to the system buffer, if it starts with a caret
    /// it goes only to the console, and if it starts with a question mark
    /// and `level` is `CE_CONT` it always goes to the system buffer but to
    /// the console only when the system was booted verbose. It may be called
    /// from user, interrupt or kernel context.
    pub fn scsi_log(dip: dev_info_t, drv_name: *mut c_char, level: uint_t, fmt: *const c_char, ...);
    /// `scsi_errmsg(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Interprets the request sense data at `sensep` for the packet `pkt` of
    /// the device `devp` and prints the standard error report with
    /// [`scsi_log`], under the name `drv_name`. The first line is a
    /// `CE_WARN` and the rest are `CE_CONT`. `severity`, one of the
    /// `SCSI_ERR_*` values, selects the text shown as the error level.
    /// `blkno` is the block number of the original request and `err_blkno`
    /// the block where the error occurred. `cmdlist` is a table of
    /// [`scsi_key_strings`] that translates the command code in the packet
    /// into a name. `sensep` may be NULL, in which case no sense key or
    /// vendor information is shown. The driver decides from the severity of
    /// the failure whether to call it. It may be called from user, interrupt
    /// or kernel context.
    pub fn scsi_errmsg(
        devp: *mut scsi_device,
        pkt: *mut scsi_pkt,
        drv_name: *mut c_char,
        severity: c_int,
        blkno: daddr_t,
        err_blkno: daddr_t,
        cmdlist: *mut scsi_key_strings,
        sensep: *mut scsi_extended_sense,
    );
    /// `scsi_vu_errmsg(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`scsi_errmsg`], but able to decode vendor unique additional
    /// sense codes and field replaceable unit codes. The first eight
    /// arguments are those of `scsi_errmsg`. `asc_list` is a table of
    /// [`scsi_asq_key_strings`] ending with an `asc` of -1, which is searched
    /// for the sense code before the standard list is. `decode_fru` is called
    /// after the sense data has been decoded to describe the field
    /// replaceable unit, or is `None` if no special decoding is needed. The
    /// manual page shows an older argument list; the one here is the
    /// header's, which matches the page's parameter descriptions. It may be
    /// called from user, interrupt or kernel context.
    pub fn scsi_vu_errmsg(
        devp: *mut scsi_device,
        pkt: *mut scsi_pkt,
        drv_name: *mut c_char,
        severity: c_int,
        blkno: daddr_t,
        err_blkno: daddr_t,
        cmdlist: *mut scsi_key_strings,
        sensep: *mut scsi_extended_sense,
        asc_list: *mut scsi_asq_key_strings,
        decode_fru: Option<scsi_decode_fru_t>,
    );
    /// `scsi_cname(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the name of the SCSI command `cmd`, looked up in `cmdvec`, a
    /// NULL terminated array of strings whose first byte is a command code
    /// and whose remainder is the name of that command. If the command is
    /// not found, it returns a string that says so. It may be called from
    /// user, interrupt or kernel context.
    pub fn scsi_cname(cmd: c_uchar, cmdvec: *mut *mut c_char) -> *mut c_char;
    /// `scsi_dname(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the name of the peripheral device type `dtyp` from the inquiry
    /// data, such as direct access or sequential access, or a string that
    /// says the value is invalid. It may be called from user, interrupt or
    /// kernel context.
    pub fn scsi_dname(dtyp: c_int) -> *mut c_char;
    /// `scsi_mname(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the name of the SCSI message `msg`, or a string that says the
    /// value is invalid. It may be called from user, interrupt or kernel
    /// context.
    pub fn scsi_mname(msg: c_uchar) -> *mut c_char;
    /// `scsi_rname(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the name of the packet completion reason `reason`, one of the
    /// `CMD_*` values, or a string that says the value is invalid. It may be
    /// called from user, interrupt or kernel context.
    pub fn scsi_rname(reason: c_uchar) -> *mut c_char;
    /// `scsi_sname(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the name of the sense key `sense_key`, one of the `KEY_*`
    /// values, or a string that says the value is invalid. It may be called
    /// from user, interrupt or kernel context.
    pub fn scsi_sname(sense_key: c_uchar) -> *mut c_char;
}
