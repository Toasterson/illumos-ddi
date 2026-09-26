//! The SCSI packet from `sys/scsi/scsi_pkt.h` and `sys/scsi/scsi_resource.h`.
//!
//! A `scsi_pkt(9S)` carries one command from a target driver to the host
//! bus adapter driver and brings the outcome back. The target driver never
//! allocates one itself. It asks `scsi_init_pkt(9F)` for a packet, which
//! passes the request to the HBA driver's `tran_init_pkt(9E)` or
//! `tran_setup_pkt(9E)` entry point, fills in the command descriptor block,
//! sets [`scsi_pkt::pkt_comp`], [`scsi_pkt::pkt_flags`] and
//! [`scsi_pkt::pkt_time`], and hands the packet to `scsi_transport(9F)`.
//! When the HBA driver is done with the command it records the outcome in
//! [`scsi_pkt::pkt_reason`], [`scsi_pkt::pkt_state`],
//! [`scsi_pkt::pkt_statistics`] and [`scsi_pkt::pkt_resid`] and calls the
//! completion routine.
//!
//! This module declares the structure, the `FLAG_*` values of `pkt_flags`,
//! the `CMD_*` completion reasons, the `STATE_*` and `STAT_*` bits, the
//! `TRAN_*` values that `scsi_transport(9F)` and `tran_start(9E)` return,
//! and the `PKT_*` flags of `scsi_init_pkt(9F)`. It also declares the
//! callback argument that all SCSA allocation functions share, because in C
//! that argument is a function pointer that may also hold the integers 0
//! and 1.
//!
//! Only the flags that `scsi_pkt(9S)` documents are declared. The header
//! also defines `FLAG_ACA`, which it marks internal, and `FLAG_SILENT`,
//! `FLAG_DIAGNOSE`, `FLAG_ISOLATE`, `FLAG_TLR`, `FLAG_NOQUEUE`,
//! `FLAG_IMMEDIATE_CB`, `FLAG_PKT_PATH_INSTANCE` and `FLAG_PKT_COMP_CALLED`,
//! which no manual page describes and which the framework or particular
//! drivers use among themselves. `PKT_CONSISTENT_OLD` is a compatibility
//! value for old binaries. The `CMD_REASON_ASCII` initialiser is a C macro
//! for building tables and has no symbol.

use ddi_sys::{c_int, c_uchar, c_void, caddr_t, ddi_dma_cookie_t, ddi_dma_handle_t, off_t, size_t, ssize_t, uint_t};

use crate::device::scsi_address;

// --- the shared callback argument ----------------------------------------

/// `scsi_init_pkt(9F)`. illumos DDI specific (illumos DDI).
///
/// The callback argument of the SCSA allocation functions. In C this
/// argument is a function pointer. For `scsi_init_pkt(9F)`,
/// `scsi_alloc_consistent_buf(9F)`, `scsi_hba_pkt_alloc(9F)`,
/// `tran_init_pkt(9E)` and `tran_setup_pkt(9E)` its type is
/// `int (*)(caddr_t)`, and for `scsi_probe(9F)`, `scsi_hba_probe(9F)` and
/// `tran_tgt_probe(9E)` it is `int (*)(void)`. In either case it may hold
/// the address of a function or one of the two sentinels [`NULL_FUNC`] and
/// [`SLEEP_FUNC`], which are the integers 0 and 1 cast to the pointer type.
/// A Rust `fn` value cannot hold 1, so the argument is declared as this
/// pointer sized wrapper, which has the same representation as the C
/// pointer. To pass a real function, cast it to a pointer, as in
/// `scsi_callback_t(my_callback as *const c_void)`. The two function
/// signatures are [`scsi_callback_fn_t`] and [`scsi_waitfunc_fn_t`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_callback_t(pub *const c_void);

/// Do not wait for resources. The function fails at once when resources are
/// short and arranges no callback (`NULL_FUNC`).
pub const NULL_FUNC: scsi_callback_t = scsi_callback_t(core::ptr::null());
/// Wait until resources are available. A call made with this value may
/// sleep and must therefore not come from interrupt context (`SLEEP_FUNC`).
pub const SLEEP_FUNC: scsi_callback_t = scsi_callback_t(core::ptr::without_provenance(1));

/// `scsi_init_pkt(9F)`. illumos DDI specific (illumos DDI).
///
/// The signature of a resource callback that takes an argument, which is
/// `int (*)(caddr_t)` in C. When an allocation that was given such a function fails for lack of
/// resources, the system calls the function with the caller's argument once
/// resources may have become available. The function tries the allocation
/// again. It returns 0 if it failed once more, in which case the system
/// keeps it on the list and calls it again later, or 1 if it succeeded or no
/// longer wants to be called. It must not block or call routines that
/// block.
pub type scsi_callback_fn_t = unsafe extern "C" fn(arg: caddr_t) -> c_int;

/// `scsi_probe(9F)`. illumos DDI specific (illumos DDI).
///
/// The signature of a resource callback without an argument, which is
/// `int (*)(void)` in C and appears as the `waitfunc` of `scsi_probe(9F)`,
/// `scsi_hba_probe(9F)` and `tran_tgt_probe(9E)`. The manual pages only ever pass [`NULL_FUNC`] or [`SLEEP_FUNC`] in this
/// position. The return convention of a real function is the same as for
/// [`scsi_callback_fn_t`].
pub type scsi_waitfunc_fn_t = unsafe extern "C" fn() -> c_int;

// --- scsi_pkt(9S) --------------------------------------------------------

/// `scsi_pkt(9S)`. illumos DDI specific (illumos DDI).
///
/// The packet that describes one SCSI command on its way from a target
/// driver through the host bus adapter driver to the device and back. It is
/// allocated by `scsi_init_pkt(9F)`, which calls into the HBA driver. The
/// target driver fills in some members and passes the packet to
/// `scsi_transport(9F)`. The HBA driver fills in others as the command is
/// processed and, when the command completes or can go no further, calls
/// the completion routine in [`pkt_comp`](Self::pkt_comp) with the packet
/// as its argument. From the members of the packet the target driver can
/// then tell whether the command succeeded.
///
/// The manual warns that drivers must not depend on the size of this
/// structure, must not embed it in their own structures, and must not
/// allocate it except through the functions made for the purpose, such as
/// `scsi_init_pkt(9F)` and `scsi_hba_pkt_alloc(9F)`. The members after
/// [`pkt_reason`](Self::pkt_reason) may only be used by an HBA driver that
/// allocated the packet with `scsi_hba_pkt_alloc(9F)` or through
/// `tran_setup_pkt(9E)`. The layout assertions below exist to catch a
/// mistake in this declaration, not to invite drivers to allocate packets.
#[repr(C)]
pub struct scsi_pkt {
    /// An opaque pointer that the HBA driver uses to reach its private data
    /// for the request. `scsi_hba_pkt_alloc(9F)` points it to an area of the
    /// size the HBA asked for.
    pub pkt_ha_private: *mut c_void,
    /// The destination of the packet. `scsi_init_pkt(9F)` copies it from the
    /// address the target driver passes, and it records the route and the
    /// recipient of the request.
    pub pkt_address: scsi_address,
    /// Reserved for the target driver. The HBA driver does not change it.
    /// If the target driver asked for a private area it points there;
    /// otherwise the target driver may use it to store its own pointer,
    /// typically the `buf(9S)` of the request.
    pub pkt_private: *mut c_void,
    /// The command completion routine, or `None`. The HBA driver calls it
    /// with the packet as its argument once it has taken the command as far
    /// as it can. It is called from interrupt context and must not sleep or
    /// call any function that might sleep. HBA drivers call it by way of
    /// `scsi_hba_pkt_comp(9F)` rather than directly.
    pub pkt_comp: Option<unsafe extern "C" fn(pkt: *mut scsi_pkt)>,
    /// How the target driver wants the command executed, a combination of
    /// the `FLAG_*` values such as [`FLAG_NOINTR`] and [`FLAG_STAG`].
    pub pkt_flags: uint_t,
    /// The time in seconds that the command may take, counted from when it
    /// is transmitted on the bus. Zero means that no timeout is required.
    pub pkt_time: c_int,
    /// Points to the status completion block. It holds a
    /// [`scsi_status`](crate::status::scsi_status) or, when automatic
    /// request sense is enabled and [`pkt_state`](Self::pkt_state) includes
    /// [`STATE_ARQ_DONE`], a
    /// [`scsi_arq_status`](crate::status::scsi_arq_status) that also
    /// contains the sense data.
    pub pkt_scbp: *mut c_uchar,
    /// Points to the command descriptor block, a kernel addressable buffer
    /// of the length requested from `scsi_init_pkt(9F)`.
    pub pkt_cdbp: *mut c_uchar,
    /// The residual count. After a transfer it is the number of data bytes
    /// that were not transferred. After `scsi_init_pkt(9F)` with
    /// [`PKT_DMA_PARTIAL`] it is the number of bytes for which DMA resources
    /// could not be allocated.
    pub pkt_resid: ssize_t,
    /// The states the command went through, a combination of the `STATE_*`
    /// bits such as [`STATE_GOT_BUS`] and [`STATE_XFERRED_DATA`].
    pub pkt_state: uint_t,
    /// Transport statistics for the command, a combination of the `STAT_*`
    /// bits such as [`STAT_BUS_RESET`] and [`STAT_TIMEOUT`].
    pub pkt_statistics: uint_t,
    /// Why the completion routine was called, one of the `CMD_*` values
    /// such as [`CMD_CMPLT`] and [`CMD_TIMEOUT`].
    pub pkt_reason: c_uchar,
    /// The length of the buffer at [`pkt_cdbp`](Self::pkt_cdbp). Only valid
    /// in an HBA driver that allocated the packet with
    /// `scsi_hba_pkt_alloc(9F)` or through `tran_setup_pkt(9E)`.
    pub pkt_cdblen: uint_t,
    /// The length of the target driver's private area at
    /// [`pkt_private`](Self::pkt_private). The same restriction as for
    /// [`pkt_cdblen`](Self::pkt_cdblen) applies.
    pub pkt_tgtlen: uint_t,
    /// The length of the buffer at [`pkt_scbp`](Self::pkt_scbp). The same
    /// restriction as for [`pkt_cdblen`](Self::pkt_cdblen) applies.
    pub pkt_scblen: uint_t,
    /// The DMA handle of the packet. Private to the framework; it is only
    /// set up when the HBA driver uses `tran_setup_pkt(9E)`.
    pub pkt_handle: ddi_dma_handle_t,
    /// The number of DMA cookies at [`pkt_cookies`](Self::pkt_cookies), or
    /// zero if the packet has no DMA resources. Only valid in an HBA driver
    /// that uses `tran_setup_pkt(9E)`.
    pub pkt_numcookies: uint_t,
    /// The offset of the current DMA window. Private to the framework.
    pub pkt_dma_offset: off_t,
    /// The length of the current DMA window. Private to the framework.
    pub pkt_dma_len: size_t,
    /// The DMA flags in use, such as `DDI_DMA_READ` and `DDI_DMA_WRITE`.
    /// Only valid in an HBA driver that uses `tran_setup_pkt(9E)`.
    pub pkt_dma_flags: uint_t,
    /// The array of [`pkt_numcookies`](Self::pkt_numcookies) DMA cookies
    /// that describe the data transfer. Only valid in an HBA driver that
    /// uses `tran_setup_pkt(9E)`.
    pub pkt_cookies: *mut ddi_dma_cookie_t,
    /// The instance of the multipathing transport path the packet takes.
    /// Private to the framework.
    pub pkt_path_instance: c_int,
    /// Temporary storage for the current stage of the packet. Private to the
    /// framework.
    pub pkt_stmp: *mut c_void,
}

// Layout on amd64 with LP64, as `sys/scsi/scsi_pkt.h` declares it under
// `_KERNEL` without `SCSI_SIZE_CLEAN_VERIFY`. pkt_ha_private 8 at 0,
// pkt_address 16 at 8, pkt_private 8 at 24, pkt_comp 8 at 32, pkt_flags 4 at
// 40, pkt_time 4 at 44, pkt_scbp 8 at 48, pkt_cdbp 8 at 56, pkt_resid 8 at
// 64, pkt_state 4 at 72, pkt_statistics 4 at 76, pkt_reason 1 at 80 and 3
// bytes of padding, pkt_cdblen 4 at 84, pkt_tgtlen 4 at 88, pkt_scblen 4 at
// 92, pkt_handle 8 at 96, pkt_numcookies 4 at 104 and 4 bytes of padding,
// pkt_dma_offset 8 at 112, pkt_dma_len 8 at 120, pkt_dma_flags 4 at 128 and
// 4 bytes of padding, pkt_cookies 8 at 136, pkt_path_instance 4 at 144 and 4
// bytes of padding, pkt_stmp 8 at 152; total 160.
const _: () = assert!(core::mem::size_of::<scsi_pkt>() == 160);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_ha_private) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_address) == 8);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_private) == 24);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_comp) == 32);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_flags) == 40);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_time) == 44);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_scbp) == 48);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_cdbp) == 56);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_resid) == 64);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_state) == 72);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_reason) == 80);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_cdblen) == 84);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_handle) == 96);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_numcookies) == 104);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_dma_offset) == 112);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_dma_flags) == 128);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_cookies) == 136);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_path_instance) == 144);
const _: () = assert!(core::mem::offset_of!(scsi_pkt, pkt_stmp) == 152);

// --- pkt_flags -----------------------------------------------------------

/// Run the command without a completion callback. The command is complete
/// when `scsi_transport(9F)` returns, and the HBA driver must not call
/// `pkt_comp` for it (`FLAG_NOINTR`).
pub const FLAG_NOINTR: uint_t = 0x0001;
/// Run the command without allowing the target to disconnect
/// (`FLAG_NODISCON`).
pub const FLAG_NODISCON: uint_t = 0x0002;
/// Run the command without parity checking (`FLAG_NOPARITY`).
pub const FLAG_NOPARITY: uint_t = 0x0008;
/// Run the command as a head of queue tagged command (`FLAG_HTAG`).
pub const FLAG_HTAG: uint_t = 0x1000;
/// Run the command as an ordered queue tagged command (`FLAG_OTAG`).
pub const FLAG_OTAG: uint_t = 0x2000;
/// Run the command as a simple queue tagged command (`FLAG_STAG`).
pub const FLAG_STAG: uint_t = 0x4000;
/// The mask of the three tag attribute flags [`FLAG_HTAG`], [`FLAG_OTAG`]
/// and [`FLAG_STAG`] (`FLAG_TAGMASK`).
pub const FLAG_TAGMASK: uint_t = FLAG_HTAG | FLAG_OTAG | FLAG_STAG;
/// The packet is a request sense command run on behalf of a failed packet
/// (`FLAG_SENSING`).
pub const FLAG_SENSING: uint_t = 0x0400;
/// Place the command at the head of the HBA driver's queue (`FLAG_HEAD`).
pub const FLAG_HEAD: uint_t = 0x8000;
/// Before transporting the command, the HBA should renegotiate wide mode
/// and synchronous transfer speed with the target. The manual recommends
/// it before Request Sense and Inquiry commands and warns that setting it
/// on every packet severely reduces performance
/// (`FLAG_RENEGOTIATE_WIDE_SYNC`).
pub const FLAG_RENEGOTIATE_WIDE_SYNC: uint_t = 0x0100_0000;

// --- pkt_reason ----------------------------------------------------------

/// No transport errors; the command completed normally (`CMD_CMPLT`).
pub const CMD_CMPLT: c_uchar = 0;
/// The transport stopped with an abnormal state (`CMD_INCOMPLETE`).
pub const CMD_INCOMPLETE: c_uchar = 1;
/// A DMA direction error occurred (`CMD_DMA_DERR`).
pub const CMD_DMA_DERR: c_uchar = 2;
/// An unspecified transport error occurred (`CMD_TRAN_ERR`).
pub const CMD_TRAN_ERR: c_uchar = 3;
/// A bus or device reset destroyed the command (`CMD_RESET`).
pub const CMD_RESET: c_uchar = 4;
/// The command was aborted on request (`CMD_ABORTED`).
pub const CMD_ABORTED: c_uchar = 5;
/// The command timed out (`CMD_TIMEOUT`).
pub const CMD_TIMEOUT: c_uchar = 6;
/// A data overrun occurred (`CMD_DATA_OVR`).
pub const CMD_DATA_OVR: c_uchar = 7;
/// A command overrun occurred (`CMD_CMD_OVR`).
pub const CMD_CMD_OVR: c_uchar = 8;
/// A status overrun occurred (`CMD_STS_OVR`).
pub const CMD_STS_OVR: c_uchar = 9;
/// The message received was not Command Complete (`CMD_BADMSG`).
pub const CMD_BADMSG: c_uchar = 10;
/// The target refused to go to the Message Out phase (`CMD_NOMSGOUT`).
pub const CMD_NOMSGOUT: c_uchar = 11;
/// The Extended Identify message was rejected (`CMD_XID_FAIL`).
pub const CMD_XID_FAIL: c_uchar = 12;
/// The Initiator Detected Error message was rejected (`CMD_IDE_FAIL`).
pub const CMD_IDE_FAIL: c_uchar = 13;
/// The Abort message was rejected (`CMD_ABORT_FAIL`).
pub const CMD_ABORT_FAIL: c_uchar = 14;
/// The Reject message was rejected (`CMD_REJECT_FAIL`).
pub const CMD_REJECT_FAIL: c_uchar = 15;
/// The No Operation message was rejected (`CMD_NOP_FAIL`).
pub const CMD_NOP_FAIL: c_uchar = 16;
/// The Message Parity Error message was rejected (`CMD_PER_FAIL`).
pub const CMD_PER_FAIL: c_uchar = 17;
/// The Bus Device Reset message was rejected (`CMD_BDR_FAIL`).
pub const CMD_BDR_FAIL: c_uchar = 18;
/// The Identify message was rejected (`CMD_ID_FAIL`).
pub const CMD_ID_FAIL: c_uchar = 19;
/// An unexpected bus free phase occurred (`CMD_UNX_BUS_FREE`).
pub const CMD_UNX_BUS_FREE: c_uchar = 20;
/// The target rejected the tag message (`CMD_TAG_REJECT`).
pub const CMD_TAG_REJECT: c_uchar = 21;
/// The command was terminated on request. The header defines this reason
/// but `scsi_pkt(9S)` does not list it; HBA drivers report it for commands
/// ended by a terminate task request (`CMD_TERMINATED`).
pub const CMD_TERMINATED: c_uchar = 22;
/// The device does not support transport layer retries. The header defines
/// this reason but `scsi_pkt(9S)` does not list it (`CMD_TLR_OFF`).
pub const CMD_TLR_OFF: c_uchar = 23;
/// The device has been removed (`CMD_DEV_GONE`).
pub const CMD_DEV_GONE: c_uchar = 24;

// --- pkt_state -----------------------------------------------------------

/// Bus arbitration succeeded (`STATE_GOT_BUS`).
pub const STATE_GOT_BUS: uint_t = 0x01;
/// The target was successfully selected (`STATE_GOT_TARGET`).
pub const STATE_GOT_TARGET: uint_t = 0x02;
/// The command was successfully sent (`STATE_SENT_CMD`).
pub const STATE_SENT_CMD: uint_t = 0x04;
/// A data transfer took place (`STATE_XFERRED_DATA`).
pub const STATE_XFERRED_DATA: uint_t = 0x08;
/// Status was received (`STATE_GOT_STATUS`).
pub const STATE_GOT_STATUS: uint_t = 0x10;
/// The command ended in a check condition and the HBA driver ran an
/// automatic request sense command, whose outcome is in the
/// [`scsi_arq_status`](crate::status::scsi_arq_status) at `pkt_scbp`
/// (`STATE_ARQ_DONE`).
pub const STATE_ARQ_DONE: uint_t = 0x20;
/// The command was issued with [`PKT_XARQ`], ended in a check condition,
/// and the HBA driver successfully requested and returned the extra sense
/// data. The sense data starts at `sts_sensedata` and its residual, based on
/// the `statuslen` given to `scsi_init_pkt(9F)`, is in `sts_rqpkt_resid`
/// (`STATE_XARQ_DONE`).
pub const STATE_XARQ_DONE: uint_t = 0x40;

// --- pkt_statistics ------------------------------------------------------

/// The device disconnected during the command (`STAT_DISCON`).
pub const STAT_DISCON: uint_t = 0x1;
/// The command did a synchronous data transfer (`STAT_SYNC`).
pub const STAT_SYNC: uint_t = 0x2;
/// A SCSI parity error occurred (`STAT_PERR`).
pub const STAT_PERR: uint_t = 0x4;
/// The bus was reset (`STAT_BUS_RESET`).
pub const STAT_BUS_RESET: uint_t = 0x8;
/// The device was reset (`STAT_DEV_RESET`).
pub const STAT_DEV_RESET: uint_t = 0x10;
/// The command was aborted (`STAT_ABORTED`).
pub const STAT_ABORTED: uint_t = 0x20;
/// The command timed out (`STAT_TIMEOUT`).
pub const STAT_TIMEOUT: uint_t = 0x40;
/// The command was terminated. The header defines this bit but
/// `scsi_pkt(9S)` does not list it; it accompanies [`CMD_TERMINATED`]
/// (`STAT_TERMINATED`).
pub const STAT_TERMINATED: uint_t = 0x80;

// --- transport return values ---------------------------------------------

/// The packet was accepted by the transport layer (`TRAN_ACCEPT`).
pub const TRAN_ACCEPT: c_int = 1;
/// The packet could not be accepted because a packet was already in
/// progress for the target and logical unit, or the HBA queue or the target
/// device queue was full. The target driver may try again later
/// (`TRAN_BUSY`).
pub const TRAN_BUSY: c_int = 0;
/// The packet was rejected, for instance because its DMA count exceeded the
/// maximum of the DMA engine (`TRAN_BADPKT`).
pub const TRAN_BADPKT: c_int = -1;
/// A fatal error occurred in the transport layer and the HBA cannot accept
/// any packets (`TRAN_FATAL_ERROR`).
pub const TRAN_FATAL_ERROR: c_int = -2;

/// The timeout in seconds that `scsi_poll(9F)` uses for a packet whose
/// `pkt_time` is zero (`SCSI_POLL_TIMEOUT`).
pub const SCSI_POLL_TIMEOUT: c_int = 60;

// --- scsi_init_pkt(9F) flags ---------------------------------------------

/// The data buffer was allocated with `scsi_alloc_consistent_buf(9F)`. The
/// HBA driver then guarantees that the transfer is synchronized before the
/// completion callback runs (`PKT_CONSISTENT`).
pub const PKT_CONSISTENT: c_int = 0x0001;
/// The target driver accepts a partial DMA mapping. `pkt_resid` then tells
/// how many bytes are not yet mapped, and further calls to
/// `scsi_init_pkt(9F)` with the same packet and buffer move the resources
/// to the next part of the transfer (`PKT_DMA_PARTIAL`).
pub const PKT_DMA_PARTIAL: c_int = 0x040000;
/// Ask the HBA to return more than the default 20 bytes of sense data. The
/// `statuslen` passed to `scsi_init_pkt(9F)` must then be the size of
/// [`scsi_arq_status`](crate::status::scsi_arq_status) plus the number of
/// extra bytes wanted (`PKT_XARQ`).
pub const PKT_XARQ: c_int = 0x080000;
