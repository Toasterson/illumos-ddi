//! SCSI sense data from `sys/scsi/generic/sense.h` and the functions of
//! `sys/scsi/impl/sense.h`.
//!
//! When a command ends in a check condition, the target has sense data that
//! describes what went wrong. The target driver obtains it either through
//! automatic request sense, in which case it arrives in the
//! [`scsi_arq_status`](crate::status::scsi_arq_status) of the packet, or by
//! sending a REQUEST SENSE command itself. Sense data comes in two formats.
//! The older fixed format is laid out as [`scsi_extended_sense`], and the
//! newer descriptor format consists of a header followed by a list of
//! descriptors.
//!
//! The functions in this module read either format, so a driver does not
//! have to tell them apart itself. [`scsi_validate_sense`] checks a buffer
//! and reports its format, [`scsi_sense_key`], [`scsi_sense_asc`] and
//! [`scsi_sense_ascq`] extract the three values that classify an error,
//! [`scsi_ext_sense_fields`] and its two companions locate or decode the
//! other fields, and [`scsi_find_sense_descr`] finds a particular descriptor
//! in descriptor format data. Their prototypes are in
//! `sys/scsi/impl/sense.h`, which `sys/scsi/generic/sense.h` includes; that
//! part of the header is public in practice because the manual pages
//! document it.
//!
//! The descriptor format structures of the header, `scsi_descr_sense_hdr`,
//! `scsi_information_sense_descr`, `scsi_cmd_specific_sense_descr`,
//! `scsi_sk_specific_sense_descr` with its union `scsi_sk_specific_data`,
//! `scsi_fru_sense_descr`, `scsi_stream_cmd_sense_descr`,
//! `scsi_block_cmd_sense_descr`, `scsi_ata_status_ret_sense_descr`,
//! `scsi_vendor_specific_sense_descr` and `scsi_descr_template`, have no
//! manual page, and neither do the old `scsi_sense` structure, the
//! vendor specific `SUN_KEY_*` sense keys, `MAX_SENSE_LENGTH` and the
//! `SCSI_IS_DESCR_SENSE` macro. They are left to the safe crate or to a
//! later revision of the manual.

use ddi_sys::{boolean_t, c_int, c_uchar, size_t};

/// `scsi_extended_sense(9S)`. illumos DDI specific (illumos DDI).
///
/// Sense data in fixed format, as returned by a successful REQUEST SENSE
/// command for error codes 0x70, current errors, and 0x71, deferred errors.
/// SCSI-2 compliant targets return at least the first 18 bytes. The
/// structure is part of `scsi_device(9S)`, where `sd_sense` may point to
/// one, and of `scsi_arq_status(9S)`. The C declaration uses bit fields in
/// its first and third bytes; Rust has no bit fields, so those bytes are
/// declared whole and their documentation names the bits, counted from the
/// least significant with `_BIT_FIELDS_LTOH`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_extended_sense {
    /// Bits 0 to 3 are `es_code`, the error code, which is 0x0 for current
    /// and 0x1 for deferred errors in fixed format, bits 4 to 6 are
    /// `es_class`, which is always 0x7, and bit 7 is `es_valid`, which is set
    /// when the information bytes hold valid information.
    pub es_byte0: c_uchar,
    /// The number of the current segment descriptor, if the REQUEST SENSE
    /// command answers a COPY, COMPARE or COPY AND VERIFY command.
    pub es_segnum: c_uchar,
    /// Bits 0 to 3 are `es_key`, the sense key, one of the `KEY_*` values,
    /// bit 4 is reserved, bit 5 is `es_ili`, which is set when the requested
    /// logical block length did not match the length of the block on the
    /// medium, bit 6 is `es_eom`, which is set when an end of medium
    /// condition exists on a sequential access or printer device, and bit 7
    /// is `es_filmk`, which is set when a sequential access device read a
    /// file mark or set mark.
    pub es_byte2: c_uchar,
    /// Information byte 1, the most significant. The four information bytes
    /// are specific to the device type or the command.
    pub es_info_1: c_uchar,
    /// Information byte 2.
    pub es_info_2: c_uchar,
    /// Information byte 3.
    pub es_info_3: c_uchar,
    /// Information byte 4, the least significant.
    pub es_info_4: c_uchar,
    /// The number of additional sense bytes that follow.
    pub es_add_len: c_uchar,
    /// Command specific information, whose meaning depends on the command
    /// that was executed.
    pub es_cmd_info: [c_uchar; 4],
    /// The additional sense code, ASC, which gives further information about
    /// the condition reported in the sense key.
    pub es_add_code: c_uchar,
    /// The additional sense code qualifier, ASCQ, which gives detail about
    /// the additional sense code.
    pub es_qual_code: c_uchar,
    /// The field replaceable unit code, FRU, which identifies a device
    /// specific unit that has failed.
    pub es_fru_code: c_uchar,
    /// Sense key specific information. It is defined when bit 7 of its first
    /// byte, the sense key specific valid bit, is 1, and reserved for sense
    /// keys not defined above.
    pub es_skey_specific: [c_uchar; 3],
    /// The first two additional sense bytes. The header notes that more may
    /// follow in a larger buffer and that this member rounds the structure
    /// to 20 bytes.
    pub es_add_info: [c_uchar; 2],
}

// Layout: bytes only, so no padding. es_byte0 at 0, es_segnum at 1,
// es_byte2 at 2, es_info_1 to es_info_4 at 3 to 6, es_add_len at 7,
// es_cmd_info 4 at 8, es_add_code at 12, es_qual_code at 13, es_fru_code at
// 14, es_skey_specific 3 at 15, es_add_info 2 at 18; total 20.
const _: () = assert!(core::mem::size_of::<scsi_extended_sense>() == 20);
const _: () = assert!(core::mem::offset_of!(scsi_extended_sense, es_byte0) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_extended_sense, es_byte2) == 2);
const _: () = assert!(core::mem::offset_of!(scsi_extended_sense, es_add_len) == 7);
const _: () = assert!(core::mem::offset_of!(scsi_extended_sense, es_cmd_info) == 8);
const _: () = assert!(core::mem::offset_of!(scsi_extended_sense, es_add_code) == 12);
const _: () = assert!(core::mem::offset_of!(scsi_extended_sense, es_skey_specific) == 15);
const _: () = assert!(core::mem::offset_of!(scsi_extended_sense, es_add_info) == 18);

/// The number of sense bytes that an automatic request sense command asks
/// for, the size of [`scsi_extended_sense`] rounded up to a multiple of the
/// size of an `int`. `scsi_arq_status(9S)` refers to it (`SENSE_LENGTH`).
pub const SENSE_LENGTH: size_t = 20;
/// The error class of extended sense data, the value of the `es_class` bits
/// (`CLASS_EXTENDED_SENSE`).
pub const CLASS_EXTENDED_SENSE: c_uchar = 0x7;

/// Fixed format sense data for a current error, a value of the `es_code`
/// bits (`CODE_FMT_FIXED_CURRENT`).
pub const CODE_FMT_FIXED_CURRENT: c_uchar = 0x0;
/// Fixed format sense data for a deferred error, a value of the `es_code`
/// bits (`CODE_FMT_FIXED_DEFERRED`).
pub const CODE_FMT_FIXED_DEFERRED: c_uchar = 0x1;
/// Descriptor format sense data for a current error, a value of the
/// `es_code` bits (`CODE_FMT_DESCR_CURRENT`).
pub const CODE_FMT_DESCR_CURRENT: c_uchar = 0x2;
/// Descriptor format sense data for a deferred error, a value of the
/// `es_code` bits (`CODE_FMT_DESCR_DEFERRED`).
pub const CODE_FMT_DESCR_DEFERRED: c_uchar = 0x3;
/// Vendor specific sense data, a value of the `es_code` bits
/// (`CODE_FMT_VENDOR_SPECIFIC`).
pub const CODE_FMT_VENDOR_SPECIFIC: c_uchar = 0xF;

// --- sense keys ----------------------------------------------------------

/// There is no specific sense key information to report (`KEY_NO_SENSE`).
pub const KEY_NO_SENSE: u8 = 0x00;
/// The last command completed successfully after some recovery action by
/// the target (`KEY_RECOVERABLE_ERROR`).
pub const KEY_RECOVERABLE_ERROR: u8 = 0x01;
/// The addressed logical unit cannot be accessed (`KEY_NOT_READY`).
pub const KEY_NOT_READY: u8 = 0x02;
/// The command ended with a non-recovered error that was probably caused by
/// a flaw in the medium or an error in the recorded data
/// (`KEY_MEDIUM_ERROR`).
pub const KEY_MEDIUM_ERROR: u8 = 0x03;
/// The target detected a non-recoverable hardware failure while performing
/// the command or during a self test (`KEY_HARDWARE_ERROR`).
pub const KEY_HARDWARE_ERROR: u8 = 0x04;
/// There was an illegal parameter in the command descriptor block or in the
/// parameter data of the command (`KEY_ILLEGAL_REQUEST`).
pub const KEY_ILLEGAL_REQUEST: u8 = 0x05;
/// The removable medium may have been changed or the target has been reset
/// (`KEY_UNIT_ATTENTION`).
pub const KEY_UNIT_ATTENTION: u8 = 0x06;
/// A command that reads or writes the medium was attempted on a block that
/// is protected from that operation (`KEY_WRITE_PROTECT`).
pub const KEY_WRITE_PROTECT: u8 = 0x07;
/// The SCSI-2 name of [`KEY_WRITE_PROTECT`] (`KEY_DATA_PROTECT`).
pub const KEY_DATA_PROTECT: u8 = KEY_WRITE_PROTECT;
/// A write once or sequential access device found blank medium or an end of
/// data indication while reading, or a write once device found non-blank
/// medium while writing (`KEY_BLANK_CHECK`).
pub const KEY_BLANK_CHECK: u8 = 0x08;
/// A vendor specific condition is being reported (`KEY_VENDOR_UNIQUE`).
pub const KEY_VENDOR_UNIQUE: u8 = 0x09;
/// A COPY, COMPARE or COPY AND VERIFY command was aborted
/// (`KEY_COPY_ABORTED`).
pub const KEY_COPY_ABORTED: u8 = 0x0A;
/// The target aborted the command (`KEY_ABORTED_COMMAND`).
pub const KEY_ABORTED_COMMAND: u8 = 0x0B;
/// A SEARCH DATA command satisfied an equal comparison (`KEY_EQUAL`).
pub const KEY_EQUAL: u8 = 0x0C;
/// A buffered peripheral device reached the end of a partition and data
/// that was not written to the medium may remain in the buffer
/// (`KEY_VOLUME_OVERFLOW`).
pub const KEY_VOLUME_OVERFLOW: u8 = 0x0D;
/// The source data did not match the data read from the medium
/// (`KEY_MISCOMPARE`).
pub const KEY_MISCOMPARE: u8 = 0x0E;
/// The reserved sense key. The manual page calls it `KEY_RESERVE` and
/// describes it as the target being reserved by another initiator, but the
/// header spells it with a final D (`KEY_RESERVED`).
pub const KEY_RESERVED: u8 = 0x0F;

// --- scsi_validate_sense(9F) ---------------------------------------------

/// The response code is not recognized, or there is not enough sense data
/// to contain the sense key, ASC and ASCQ (`SENSE_UNUSABLE`).
pub const SENSE_UNUSABLE: c_int = 0;
/// The buffer holds usable sense data in fixed format
/// (`SENSE_FIXED_FORMAT`).
pub const SENSE_FIXED_FORMAT: c_int = 1;
/// The buffer holds usable sense data in descriptor format
/// (`SENSE_DESCR_FORMAT`).
pub const SENSE_DESCR_FORMAT: c_int = 2;
/// The buffer provided for the request was too small to hold all the sense
/// data (`SNS_BUF_OVERFLOW`).
pub const SNS_BUF_OVERFLOW: c_int = 1;
/// The sense data describes an error that occurred while processing a
/// command that had already completed successfully, such as a cached write
/// that could not be committed to the medium (`SNS_BUF_DEFERRED`).
pub const SNS_BUF_DEFERRED: c_int = 2;

// --- descriptor types ----------------------------------------------------

/// The information descriptor (`DESCR_INFORMATION`).
pub const DESCR_INFORMATION: c_int = 0x00;
/// The command specific information descriptor (`DESCR_COMMAND_SPECIFIC`).
pub const DESCR_COMMAND_SPECIFIC: c_int = 0x01;
/// The sense key specific descriptor (`DESCR_SENSE_KEY_SPECIFIC`).
pub const DESCR_SENSE_KEY_SPECIFIC: c_int = 0x02;
/// The field replaceable unit descriptor (`DESCR_FRU`).
pub const DESCR_FRU: c_int = 0x03;
/// The stream commands descriptor, with the file mark, end of medium and
/// incorrect length flags (`DESCR_STREAM_COMMANDS`).
pub const DESCR_STREAM_COMMANDS: c_int = 0x04;
/// The block commands descriptor, with the incorrect length flag
/// (`DESCR_BLOCK_COMMANDS`).
pub const DESCR_BLOCK_COMMANDS: c_int = 0x05;
/// The object identification descriptor of object based storage devices
/// (`DESCR_OSD_OID`).
pub const DESCR_OSD_OID: c_int = 0x06;
/// The response integrity check value descriptor of object based storage
/// devices (`DESCR_OSD_RESP_INTEGRITY`).
pub const DESCR_OSD_RESP_INTEGRITY: c_int = 0x07;
/// The attribute identification descriptor of object based storage devices
/// (`DESCR_OSD_ATTR_ID`).
pub const DESCR_OSD_ATTR_ID: c_int = 0x08;
/// The ATA status return descriptor, which the header defines but
/// `scsi_find_sense_descr(9F)` does not list (`DESCR_ATA_STATUS_RETURN`).
pub const DESCR_ATA_STATUS_RETURN: c_int = 0x09;

unsafe extern "C" {
    /// `scsi_validate_sense(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Examines the `sense_buf_len` bytes of sense data at `sense_buffer`,
    /// which is expected in wire format starting at the response code, and
    /// reports its format. It returns [`SENSE_UNUSABLE`] if the response code
    /// is not recognized or if there is not enough data to include the sense
    /// key, ASC and ASCQ, [`SENSE_FIXED_FORMAT`] for usable fixed format
    /// data, and [`SENSE_DESCR_FORMAT`] for usable descriptor format data.
    /// It also stores additional properties of the data in `flags`, a
    /// combination of [`SNS_BUF_OVERFLOW`] and [`SNS_BUF_DEFERRED`]. Drivers
    /// call it before using the other sense functions on a buffer. It may be
    /// called from user or interrupt context.
    pub fn scsi_validate_sense(sense_buffer: *mut u8, sense_buf_len: c_int, flags: *mut c_int) -> c_int;
    /// `scsi_sense_key(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the sense key from the sense data at `sense_buffer`, which is
    /// expected in wire format starting at the response code and may be in
    /// fixed or descriptor format. The value is one of the `KEY_*`
    /// constants. Drivers should check the buffer with
    /// [`scsi_validate_sense`] first. It may be called from user or interrupt
    /// context.
    pub fn scsi_sense_key(sense_buffer: *mut u8) -> u8;
    /// `scsi_sense_asc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the additional sense code, ASC, from the sense data at
    /// `sense_buffer`, in either fixed or descriptor format. Drivers should
    /// check the buffer with [`scsi_validate_sense`] first. It may be called
    /// from user or interrupt context.
    pub fn scsi_sense_asc(sense_buffer: *mut u8) -> u8;
    /// `scsi_sense_ascq(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the additional sense code qualifier, ASCQ, from the sense data
    /// at `sense_buffer`, in either fixed or descriptor format. Drivers
    /// should check the buffer with [`scsi_validate_sense`] first. It may be
    /// called from user or interrupt context.
    pub fn scsi_sense_ascq(sense_buffer: *mut u8) -> u8;
    /// `scsi_ext_sense_fields(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Locates the extended fields in the `sense_buf_len` bytes of sense data
    /// at `sense_buffer`, in either fixed or descriptor format. Each of the
    /// remaining arguments is an output parameter that may be NULL if the
    /// caller is not interested in that field. For each one that is not
    /// NULL, the function stores a pointer into the sense buffer if the field
    /// is present and valid, and NULL otherwise. `information` receives the
    /// INFORMATION field, `cmd_spec_info` the COMMAND SPECIFIC INFORMATION
    /// field, `fru_code` the FIELD REPLACEABLE UNIT CODE, `sk_specific` the
    /// SENSE KEY SPECIFIC field, and `stream_flags` the byte that holds the
    /// ILI, EOM and FILEMARK flags. The information and command specific
    /// fields are 4 bytes long in fixed format and 8 bytes long in
    /// descriptor format, which [`scsi_validate_sense`] tells apart. The
    /// manual page shows `sense_buf_len` as a pointer, but the header
    /// declares it as an `int`, as here. It may be called from user or
    /// interrupt context.
    pub fn scsi_ext_sense_fields(
        sense_buffer: *mut u8,
        sense_buf_len: c_int,
        information: *mut *mut u8,
        cmd_spec_info: *mut *mut u8,
        fru_code: *mut *mut u8,
        sk_specific: *mut *mut u8,
        stream_flags: *mut *mut u8,
    );
    /// `scsi_sense_info_uint64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads the INFORMATION field of the `sense_buf_len` bytes of sense data
    /// at `sense_buffer` as a 64-bit integer and stores it in `information`.
    /// In fixed format the field is 32 bits wide and the upper 32 bits of the
    /// result are 0. Whether the field holds an integer, such as the logical
    /// block address of a failed disk request, depends on the ASC and ASCQ.
    /// Returns `B_TRUE` if the field is present and valid and `B_FALSE`
    /// otherwise. It may be called from user or interrupt context.
    pub fn scsi_sense_info_uint64(sense_buffer: *mut u8, sense_buf_len: c_int, information: *mut u64) -> boolean_t;
    /// `scsi_sense_cmdspecific_uint64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads the COMMAND SPECIFIC INFORMATION field of the `sense_buf_len`
    /// bytes of sense data at `sense_buffer` as a 64-bit integer and stores it
    /// in `cmd_spec_info`. In fixed format the field is 32 bits wide and the
    /// upper 32 bits of the result are 0. Returns `B_TRUE` if the field is
    /// present and valid and `B_FALSE` otherwise. It may be called from user
    /// or interrupt context.
    pub fn scsi_sense_cmdspecific_uint64(
        sense_buffer: *mut u8,
        sense_buf_len: c_int,
        cmd_spec_info: *mut u64,
    ) -> boolean_t;
    /// `scsi_find_sense_descr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Searches the `sense_buf_len` bytes of descriptor format sense data at
    /// `sense_buffer` for a descriptor of type `req_descr_type`, which may be
    /// any value and is usually one of the `DESCR_*` constants. Before
    /// returning, it verifies that the whole descriptor lies within the
    /// buffer. Returns a pointer to the descriptor, or NULL if there is no
    /// descriptor of that type. Drivers should check with
    /// [`scsi_validate_sense`] that the buffer holds valid descriptor format
    /// data first. It may be called from user or interrupt context.
    pub fn scsi_find_sense_descr(sense_buffer: *mut u8, sense_buf_len: c_int, req_descr_type: c_int) -> *mut u8;
}
