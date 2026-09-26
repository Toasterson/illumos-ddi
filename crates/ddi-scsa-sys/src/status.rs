//! SCSI status from `sys/scsi/generic/status.h`.
//!
//! When a command completes, the status completion block that `pkt_scbp`
//! points to holds either the plain status byte, declared here as
//! [`scsi_status`], or, if automatic request sense was enabled and the
//! packet's `pkt_state` includes `STATE_ARQ_DONE`, a [`scsi_arq_status`]
//! that adds the outcome of the request sense command and the sense data.
//! The `STATUS_*` constants give the status byte values as a whole.
//!
//! `SECMDS_STATUS_SIZE` is the size of [`scsi_arq_status`] under another
//! name and `STATUS_SIZE` in `sys/scsi/impl/status.h` is an implementation
//! detail; neither is bound.

use ddi_sys::{c_uchar, uint_t};

use crate::sense::scsi_extended_sense;

/// `scsi_status(9S)`. illumos DDI specific (illumos DDI).
///
/// The SCSI status byte that the target sends to the initiator in the
/// status phase at the end of each command. In C it is a structure of eight
/// one bit fields; Rust has no bit fields, so the byte is declared as a
/// whole. For most purposes a driver compares the byte, masked with
/// [`STATUS_MASK`], with the `STATUS_*` values.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_status {
    /// The status byte. Counting from the least significant bit with
    /// `_BIT_FIELDS_LTOH`, bit 0 is `sts_vu0`, which is vendor unique, bit 1
    /// is `sts_chk`, check condition, bit 2 is `sts_cm`, condition met, bit
    /// 3 is `sts_busy`, the device is busy or reserved, bit 4 is `sts_is`,
    /// intermediate status sent, bit 5 is `sts_vu6`, which the header also
    /// names `sts_scsi2` and which is the SCSI-2 modifier bit, bit 6 is
    /// `sts_vu7`, which is vendor unique, and bit 7 is the reserved
    /// `sts_resvd`. The manual page counts the bits from the other end but
    /// describes the same byte. `sts_chk` means a contingent allegiance
    /// condition occurred. `sts_is` with `sts_busy` means a reservation
    /// conflict, `sts_scsi2` with `sts_chk` means command terminated, and
    /// `sts_scsi2` with `sts_busy` means the target's command queue is full.
    pub sts_byte0: c_uchar,
}

// Layout: one byte of bit fields; size 1, alignment 1.
const _: () = assert!(core::mem::size_of::<scsi_status>() == 1);
const _: () = assert!(core::mem::offset_of!(scsi_status, sts_byte0) == 0);

/// `scsi_arq_status(9S)`. illumos DDI specific (illumos DDI).
///
/// The status completion block of a packet for which automatic request
/// sense was performed. When auto request sense has been enabled with
/// `scsi_ifsetcap(9F)` and the `auto-rqsense` capability, the target driver
/// allocates a status area of at least this size in its packets, and on a
/// check condition the transport layer runs a request sense command
/// itself, so that the sense information is not lost. This structure then
/// supplies the status of the original command, the transport information
/// of the request sense command, and the sense data.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_arq_status {
    /// The status of the original command. If it shows a check condition,
    /// the transport layer may have run an automatic request sense command.
    pub sts_status: scsi_status,
    /// The status of the request sense command.
    pub sts_rqpkt_status: scsi_status,
    /// The completion reason of the request sense command, one of the
    /// `CMD_*` values. Anything other than `CMD_CMPLT` means the request
    /// sense command did not complete normally.
    pub sts_rqpkt_reason: c_uchar,
    /// The residual count of the request sense transfer, the number of
    /// bytes that were not transferred. The command requests
    /// [`SENSE_LENGTH`](crate::sense::SENSE_LENGTH) bytes, or more for a
    /// packet allocated with `PKT_XARQ`.
    pub sts_rqpkt_resid: c_uchar,
    /// The states the request sense command went through, a combination of
    /// the `STATE_*` bits.
    pub sts_rqpkt_state: uint_t,
    /// The transport statistics of the request sense command, a combination
    /// of the `STAT_*` bits.
    pub sts_rqpkt_statistics: uint_t,
    /// The sense data, valid if the request sense command completed
    /// normally. With `PKT_XARQ` the sense data continues past the end of
    /// this member into the extra space the target driver allocated.
    pub sts_sensedata: scsi_extended_sense,
}

// Layout on amd64. sts_status 1 at 0, sts_rqpkt_status 1 at 1,
// sts_rqpkt_reason 1 at 2, sts_rqpkt_resid 1 at 3, sts_rqpkt_state 4 at 4,
// sts_rqpkt_statistics 4 at 8, sts_sensedata 20 at 12 (it is aligned to 1);
// total 32, a multiple of the alignment of 4.
const _: () = assert!(core::mem::size_of::<scsi_arq_status>() == 32);
const _: () = assert!(core::mem::offset_of!(scsi_arq_status, sts_status) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_arq_status, sts_rqpkt_status) == 1);
const _: () = assert!(core::mem::offset_of!(scsi_arq_status, sts_rqpkt_reason) == 2);
const _: () = assert!(core::mem::offset_of!(scsi_arq_status, sts_rqpkt_resid) == 3);
const _: () = assert!(core::mem::offset_of!(scsi_arq_status, sts_rqpkt_state) == 4);
const _: () = assert!(core::mem::offset_of!(scsi_arq_status, sts_rqpkt_statistics) == 8);
const _: () = assert!(core::mem::offset_of!(scsi_arq_status, sts_sensedata) == 12);

/// The mask that selects the meaningful bits of the status byte, leaving
/// out the vendor unique and reserved bits. The header defines it but
/// `scsi_status(9S)` does not list it (`STATUS_MASK`).
pub const STATUS_MASK: c_uchar = 0x3E;
/// The target successfully completed the command (`STATUS_GOOD`).
pub const STATUS_GOOD: c_uchar = 0x00;
/// A contingent allegiance condition occurred, and sense data describes it
/// (`STATUS_CHECK`).
pub const STATUS_CHECK: c_uchar = 0x02;
/// The requested operation was satisfied (`STATUS_MET`).
pub const STATUS_MET: c_uchar = 0x04;
/// The target is busy. The recommended recovery is to issue the command
/// again later (`STATUS_BUSY`).
pub const STATUS_BUSY: c_uchar = 0x08;
/// Returned for every successfully completed command in a series of linked
/// commands except the last (`STATUS_INTERMEDIATE`).
pub const STATUS_INTERMEDIATE: c_uchar = 0x10;
/// The SCSI-2 modifier bit (`STATUS_SCSI2`).
pub const STATUS_SCSI2: c_uchar = 0x20;
/// The combination of [`STATUS_MET`] and [`STATUS_INTERMEDIATE`]
/// (`STATUS_INTERMEDIATE_MET`).
pub const STATUS_INTERMEDIATE_MET: c_uchar = 0x14;
/// The combination of [`STATUS_INTERMEDIATE`] and [`STATUS_BUSY`], returned
/// when an initiator tries to access a logical unit, or an extent within
/// one, that is reserved for another initiator
/// (`STATUS_RESERVATION_CONFLICT`).
pub const STATUS_RESERVATION_CONFLICT: c_uchar = 0x18;
/// The combination of [`STATUS_SCSI2`] and [`STATUS_CHECK`], returned when
/// the target ends the current I/O process after receiving a terminate I/O
/// process message (`STATUS_TERMINATED`).
pub const STATUS_TERMINATED: c_uchar = 0x22;
/// The combination of [`STATUS_SCSI2`] and [`STATUS_BUSY`], returned when
/// the target's command queue is full (`STATUS_QFULL`).
pub const STATUS_QFULL: c_uchar = 0x28;
/// An auto contingent allegiance condition is active. The header defines
/// this value but `scsi_status(9S)` does not list it (`STATUS_ACA_ACTIVE`).
pub const STATUS_ACA_ACTIVE: c_uchar = 0x30;
/// The command was aborted by another initiator. The header defines this
/// value but `scsi_status(9S)` does not list it (`STATUS_TASK_ABORT`).
pub const STATUS_TASK_ABORT: c_uchar = 0x40;
