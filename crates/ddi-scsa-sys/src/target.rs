//! The target driver side of SCSA, from `sys/scsi/conf/device.h`,
//! `sys/scsi/conf/autoconf.h`, `sys/scsi/scsi_ctl.h`,
//! `sys/scsi/scsi_resource.h`, `sys/scsi/scsi_pkt.h` and the documented
//! part of `sys/scsi/impl/services.h`.
//!
//! A SCSI target driver, such as a disk or tape driver, finds its
//! `scsi_device(9S)` with `ddi_get_driver_private(9F)` and checks for its
//! device with [`scsi_probe`] from `probe(9E)` or `attach(9E)`. It learns
//! and sets transport features through [`scsi_ifgetcap`] and
//! [`scsi_ifsetcap`]. For each command it allocates a packet with
//! [`scsi_init_pkt`], which also maps the data buffer for DMA, fills in the
//! command descriptor block and hands the packet to [`scsi_transport`], or
//! to [`scsi_poll`] when it must wait. The completion routine in the packet
//! examines the outcome, and [`scsi_destroy_pkt`] releases the packet. Error
//! recovery uses [`scsi_abort`], [`scsi_reset`] and [`scsi_reset_notify`].
//! Every one of these calls reaches the HBA driver through the
//! `scsi_hba_tran(9S)` vector in the device's address.
//!
//! `scsi_probe` is declared in `sys/scsi/conf/device.h`, the control
//! functions in `sys/scsi/scsi_ctl.h`, the resource functions in
//! `sys/scsi/scsi_resource.h` and `scsi_poll` in
//! `sys/scsi/impl/services.h`, which is bound only as far as its manual
//! pages document it.
//!
//! Several functions of these headers are obsolete and not bound.
//! `scsi_slave` and `scsi_unslave` were replaced by [`scsi_probe`] and
//! [`scsi_unprobe`]. `scsi_pktalloc`, `scsi_resalloc`, `scsi_pktfree`,
//! `scsi_resfree` and `scsi_dmaget` were replaced by [`scsi_init_pkt`] and
//! [`scsi_destroy_pkt`], and `get_pktiopb` and `free_pktiopb` by
//! [`scsi_alloc_consistent_buf`] and [`scsi_free_consistent_buf`].
//! `scsi_dmafree` shares its page with `scsi_dmaget`, and that page marks
//! both obsolete, so it is left out as well even though `tran_dmafree(9E)`
//! still refers to it. The functions `scsi_clear_task_set`,
//! `scsi_terminate_task`, `scsi_clear_aca`, `scsi_ua_get`,
//! `scsi_ua_get_reportdev`, `scsi_pkt_allocated_correctly`, `scsi_pkt2bp`,
//! `scsi_pkt_size` and `scsi_size_clean`, the structure
//! `scsi_pkt_cache_wrapper`, the constants `SCSI_NO_QUIESCE`,
//! `SCSIPROBE_ASCII`, `SCSI_DEFAULT_RESET_DELAY`,
//! `SCSI_DEFAULT_SELECTION_TIMEOUT` and `SCSI_ENUMERATION_*`, and the
//! variables `scsi_options`, `scsi_reset_delay` and their relatives have no
//! manual page and are not bound either. The `watch` and fault management
//! interfaces of `sys/scsi/scsi_watch.h` and `sys/scsi/scsi_fm.h` have no
//! manual page.

use ddi_sys::{buf, c_char, c_int, caddr_t, size_t, uint_t};

use crate::device::{scsi_address, scsi_device};
use crate::pkt::{scsi_callback_t, scsi_pkt};

// --- scsi_probe(9F) results ----------------------------------------------

/// The device exists and its inquiry data is valid (`SCSIPROBE_EXISTS`).
pub const SCSIPROBE_EXISTS: c_int = 0;
/// The device exists, but its inquiry data is not valid, as for a non-CCS
/// device (`SCSIPROBE_NONCCS`).
pub const SCSIPROBE_NONCCS: c_int = 1;
/// The device does not respond to an INQUIRY command (`SCSIPROBE_NORESP`).
pub const SCSIPROBE_NORESP: c_int = 2;
/// No space was available for the structures (`SCSIPROBE_NOMEM`).
pub const SCSIPROBE_NOMEM: c_int = 3;
/// The polled command failed (`SCSIPROBE_FAILURE`).
pub const SCSIPROBE_FAILURE: c_int = 4;
/// The device exists but is currently busy (`SCSIPROBE_BUSY`).
pub const SCSIPROBE_BUSY: c_int = 5;
/// No space was available for the structures, but a callback request has
/// been queued (`SCSIPROBE_NOMEM_CB`).
pub const SCSIPROBE_NOMEM_CB: c_int = 6;

// --- reset levels and notification ---------------------------------------

/// Reset the bus, the host adapter and everything on it (`RESET_ALL`).
pub const RESET_ALL: c_int = 0;
/// Reset the target that the address names (`RESET_TARGET`).
pub const RESET_TARGET: c_int = 1;
/// Reset the bus only, not the targets. It is the level that
/// `tran_bus_reset(9E)` receives (`RESET_BUS`).
pub const RESET_BUS: c_int = 2;
/// Reset the logical unit that the address names. It may fail if the device
/// or the HBA does not support the LOGICAL UNIT RESET message, and it is
/// only passed to the HBA when the `lun-reset` capability is 1
/// (`RESET_LUN`).
pub const RESET_LUN: c_int = 3;
/// Register a bus reset notification with [`scsi_reset_notify`]
/// (`SCSI_RESET_NOTIFY`).
pub const SCSI_RESET_NOTIFY: c_int = 0x01;
/// Cancel a bus reset notification with [`scsi_reset_notify`]
/// (`SCSI_RESET_CANCEL`).
pub const SCSI_RESET_CANCEL: c_int = 0x02;

unsafe extern "C" {
    // --- probing ---------------------------------------------------------

    /// `scsi_probe(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds out whether the target and logical unit of `sd` are present,
    /// using the INQUIRY command and retrying it as appropriate. On success
    /// it allocates a `scsi_inquiry(9S)`, fills it in, stores its address in
    /// `sd_inq` and returns [`SCSIPROBE_EXISTS`]. The other results are
    /// [`SCSIPROBE_NONCCS`] for a non-CCS device, [`SCSIPROBE_NORESP`],
    /// [`SCSIPROBE_NOMEM`], [`SCSIPROBE_NOMEM_CB`], [`SCSIPROBE_FAILURE`] and
    /// [`SCSIPROBE_BUSY`]. `waitfunc` is `NULL_FUNC` not to wait for
    /// resources or `SLEEP_FUNC` to wait indefinitely. [`scsi_unprobe`]
    /// undoes its effect. It is normally called from the target driver's
    /// `probe(9E)` or `attach(9E)` and must not be called from interrupt
    /// context, because it may sleep. In C `waitfunc` is `int (*)(void)`; it
    /// is declared as [`scsi_callback_t`] because it holds one of the two
    /// sentinel values.
    pub fn scsi_probe(sd: *mut scsi_device, waitfunc: scsi_callback_t) -> c_int;
    /// `scsi_unprobe(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the resources allocated on the driver's behalf while
    /// [`scsi_probe`] probed the device `sd`. The inquiry data is no longer
    /// freed here but by the framework. It must not be called from interrupt
    /// context.
    pub fn scsi_unprobe(sd: *mut scsi_device);

    // --- packets ---------------------------------------------------------

    /// `scsi_init_pkt(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the transport layer to allocate and initialise a packet for a
    /// command to the target at `ap`, possibly with a data transfer. If
    /// `pktp` is NULL, a new `scsi_pkt(9S)` is allocated through the HBA
    /// driver, with a command block of `cmdlen` bytes, a status block of
    /// `statuslen` bytes and a private area of `privatelen` bytes that
    /// `pkt_private` then points to. A `statuslen` of 1 returns the status
    /// byte, and the size of a `scsi_arq_status(9S)` returns automatic
    /// request sense data as well. If `bp` is not NULL and has a byte count,
    /// DMA resources are allocated for it. `flags` is a combination of
    /// `PKT_CONSISTENT`, required for buffers from
    /// [`scsi_alloc_consistent_buf`], `PKT_DMA_PARTIAL`, to accept a partial
    /// mapping, and `PKT_XARQ`, to ask for extra sense data. When DMA
    /// resources are allocated, `pkt_resid` gives the number of bytes that
    /// could not be mapped; with a partial mapping the driver calls again
    /// with the same packet and buffer, in which case the lengths are
    /// ignored, to move to the next part of the transfer. `callback` is
    /// `NULL_FUNC` not to wait, `SLEEP_FUNC` to wait, or a function that is
    /// called with `arg` when resources may have become available. Returns
    /// the packet, which is `pktp` if that was not NULL, or NULL if the
    /// packet or the DMA resources could not be allocated. With `SLEEP_FUNC`
    /// it may only be called from user context, and otherwise from user,
    /// interrupt or kernel context. The callback must not block. In C
    /// `callback` is `int (*)(caddr_t)`; it is declared as
    /// [`scsi_callback_t`] because it may hold the two sentinel values.
    pub fn scsi_init_pkt(
        ap: *mut scsi_address,
        pktp: *mut scsi_pkt,
        bp: *mut buf,
        cmdlen: c_int,
        statuslen: c_int,
        privatelen: c_int,
        flags: c_int,
        callback: scsi_callback_t,
        arg: caddr_t,
    ) -> *mut scsi_pkt;
    /// `scsi_destroy_pkt(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases a packet and all its resources, typically at the end of a
    /// transfer. The data is synchronised to memory, the DMA resources are
    /// freed, and then `pktp` itself is freed. It may be called from user,
    /// interrupt or kernel context.
    pub fn scsi_destroy_pkt(pktp: *mut scsi_pkt);
    /// `scsi_sync_pkt(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Synchronises the CPU's or the device's view of the data of `pktp`,
    /// which may involve flushing caches or waiting for write buffers to
    /// drain. [`scsi_init_pkt`] and [`scsi_destroy_pkt`] already synchronise
    /// implicitly. It is needed when a packet is reused, before
    /// [`scsi_transport`] for a transfer to the device, and after completion
    /// but before the data is read for a transfer from the device. It may be
    /// called from user, interrupt or kernel context.
    pub fn scsi_sync_pkt(pktp: *mut scsi_pkt);
    /// `scsi_alloc_consistent_buf(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates a `buf(9S)` and a data buffer of `datalen` bytes in
    /// consistent memory for DMA to or from the target at `ap`. If `bp` is
    /// NULL, a new buffer header is allocated with `getrbuf(9F)`, and if
    /// `datalen` is not zero, the data buffer is allocated with
    /// `ddi_dma_mem_alloc(9F)`. `bflags` is `B_READ` or `B_WRITE`. A packet
    /// for this buffer must be created with `PKT_CONSISTENT`, so that the HBA
    /// synchronises the transfer before the completion callback. `callback`
    /// and `arg` work as for [`scsi_init_pkt`]. Returns the buffer, or NULL
    /// if resources were not available. With `SLEEP_FUNC` it may only be
    /// called from user context, and otherwise from user, interrupt or kernel
    /// context. In C `callback` is `int (*)(caddr_t)`; it is declared as
    /// [`scsi_callback_t`] because it may hold the two sentinel values.
    pub fn scsi_alloc_consistent_buf(
        ap: *mut scsi_address,
        bp: *mut buf,
        datalen: size_t,
        bflags: uint_t,
        callback: scsi_callback_t,
        arg: caddr_t,
    ) -> *mut buf;
    /// `scsi_free_consistent_buf(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees a buffer header and its data buffer that
    /// [`scsi_alloc_consistent_buf`] allocated. It may be called from user,
    /// interrupt or kernel context.
    pub fn scsi_free_consistent_buf(bp: *mut buf);

    // --- transport -------------------------------------------------------

    /// `scsi_transport(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the HBA driver to send the command in `pkt` to the device its
    /// `pkt_address` names. The packet must have been obtained from
    /// [`scsi_init_pkt`]. The function does not wait for the command to
    /// complete unless `FLAG_NOINTR` is set; when the command completes, the
    /// HBA calls the packet's `pkt_comp` routine. Returns `TRAN_ACCEPT` if
    /// the packet was accepted, `TRAN_BUSY` if a queue was full or a packet
    /// was already in progress for the target, `TRAN_BADPKT` if the packet
    /// was rejected, for example because its DMA count was too large, and
    /// `TRAN_FATAL_ERROR` if a fatal error occurred in the transport layer.
    /// It may be called from user, interrupt or kernel context.
    pub fn scsi_transport(pkt: *mut scsi_pkt) -> c_int;
    /// `scsi_poll(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Runs the command in `pkt` to completion before returning, unlike
    /// [`scsi_transport`]. If `pkt_time` is 0, it uses
    /// [`SCSI_POLL_TIMEOUT`](crate::pkt::SCSI_POLL_TIMEOUT) seconds instead
    /// to avoid hanging the system. Returns 0 if the command completed
    /// successfully and -1 if it failed. It may be called from user,
    /// interrupt or kernel context, but not from a `timeout(9F)` routine
    /// running in the context of a thread.
    pub fn scsi_poll(pkt: *mut scsi_pkt) -> c_int;

    // --- control ---------------------------------------------------------

    /// `scsi_abort(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Terminates the command `pkt`, which has been transported to the HBA
    /// driver for the target at `ap`, or all outstanding commands of that
    /// target if `pkt` is NULL. Aborted packets get `pkt_reason` set to
    /// `CMD_ABORTED` and `STAT_ABORTED` added to `pkt_statistics`. Returns 1
    /// on success and 0 on failure. It may be called from user, interrupt or
    /// kernel context.
    pub fn scsi_abort(ap: *mut scsi_address, pkt: *mut scsi_pkt) -> c_int;
    /// `scsi_reset(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the HBA driver to reset the bus if `level` is [`RESET_ALL`], the
    /// target at `ap` if it is [`RESET_TARGET`], or the logical unit at `ap`
    /// if it is [`RESET_LUN`]. Commands outstanding at the time are aborted
    /// and completed with `pkt_reason` set to `CMD_RESET` and
    /// `STAT_BUS_RESET` or `STAT_DEV_RESET` in `pkt_statistics`. A command
    /// that another thread is passing to [`scsi_transport`] at the same
    /// moment may not have reached the HBA and is then not aborted. Returns
    /// 1 on success and 0 on failure. It may be called from user, interrupt
    /// or kernel context.
    pub fn scsi_reset(ap: *mut scsi_address, level: c_int) -> c_int;
    /// `scsi_reset_notify(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers or cancels a request to be told of bus resets on the bus of
    /// the target at `ap`, whether the HBA or another initiator caused them.
    /// With `flag` set to [`SCSI_RESET_NOTIFY`], `callback` is called with
    /// `arg` after a reset; with [`SCSI_RESET_CANCEL`], the request is
    /// cancelled. The `reset-notification` capability tells whether the HBA
    /// supports it. Returns `DDI_SUCCESS` if the request was accepted or
    /// cancelled, and `DDI_FAILURE` if the transport does not support it or
    /// no request was registered. The manual page shows a `void` return,
    /// but the header returns an `int`, as here. It may be called from user,
    /// interrupt or kernel context.
    pub fn scsi_reset_notify(
        ap: *mut scsi_address,
        flag: c_int,
        callback: Option<unsafe extern "C" fn(arg: caddr_t)>,
        arg: caddr_t,
    ) -> c_int;
    /// `scsi_ifgetcap(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the current value of the transport capability named by the
    /// NUL terminated string `cap`, such as `auto-rqsense`, `tagged-qing`
    /// or `max-cdb-length`, or -1 if the capability is not defined. If `whom`
    /// is 0 the question concerns all targets, and otherwise only the target
    /// at `ap`. The `geometry` capability returns the heads per cylinder in
    /// the upper 16 bits and the sectors per track in the lower 16 bits, or
    /// -1 if geometry does not apply. It may be called from user, interrupt
    /// or kernel context.
    pub fn scsi_ifgetcap(ap: *mut scsi_address, cap: *mut c_char, whom: c_int) -> c_int;
    /// `scsi_ifsetcap(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets the transport capability named by `cap` to `value`, for all
    /// targets if `whom` is 0 and otherwise only for the target at `ap`. A
    /// target driver sets `sector-size` and `total-sectors` so that the HBA
    /// can break up transfers and compute a geometry. Returns 1 if the
    /// capability was set, 0 if it cannot be changed, and -1 if it is not
    /// defined or could not be set. It may be called from user, interrupt or
    /// kernel context.
    pub fn scsi_ifsetcap(ap: *mut scsi_address, cap: *mut c_char, value: c_int, whom: c_int) -> c_int;
}
