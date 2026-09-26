//! The host bus adapter side of SCSA, from `sys/scsi/impl/transport.h`.
//!
//! An HBA driver exports its transport entry points to the framework in a
//! [`scsi_hba_tran`] structure. It calls [`scsi_hba_init`] from `_init(9E)`
//! before `mod_install(9F)`, allocates the structure with
//! [`scsi_hba_tran_alloc`] in `attach(9E)`, fills in the `tran_*` vectors,
//! and registers it together with its DMA attributes through
//! [`scsi_hba_attach_setup`]. From then on the framework calls
//! `tran_tgt_init(9E)` for each target device that is created, and turns
//! the target drivers' `scsi_transport(9F)`, `scsi_init_pkt(9F)` and similar
//! calls into calls of `tran_start(9E)`, `tran_init_pkt(9E)` and the other
//! entry points. The HBA driver allocates packets for target drivers with
//! [`scsi_hba_pkt_alloc`] and reports their completion with
//! [`scsi_hba_pkt_comp`].
//!
//! Newer HBA drivers for fabrics such as SAS use complex addressing,
//! selected with [`SCSI_HBA_ADDR_COMPLEX`], and let the framework enumerate
//! their devices. With [`SCSI_HBA_HBA`] the HBA node only has initiator
//! port, or iport, children, which the driver creates with
//! [`scsi_hba_iport_register`] or through an iport map from
//! [`scsi_hba_iportmap_create`]. Each iport attaches to the same driver and
//! reports the targets behind it through a target map from
//! [`scsi_hba_tgtmap_create`].
//!
//! Everything here is declared in `sys/scsi/impl/transport.h`, which
//! `sys/scsi/scsi.h` includes. That header is not public as a whole. The
//! parts bound here are the ones that `scsi_hba_tran(9S)`, the `tran_*(9E)`
//! pages and the `scsi_hba_*(9F)` pages document, and the structure is
//! declared in full because HBA drivers fill it in. The iport and target
//! map functions are Evolving, as their manual pages say.
//!
//! The header also declares `scsi_hba_probe_pi`, `scsi_hba_ua_get`,
//! `scsi_hba_ua_get_reportdev`, `scsi_hba_open`, `scsi_hba_close`,
//! `scsi_hba_ioctl`, `scsi_hba_in_panic`, `scsi_hba_tran_size`,
//! `scsi_tran_ext_alloc`, `scsi_tran_ext_free`,
//! `scsi_hba_nodename_compatible_get`, `scsi_hba_nodename_compatible_free`,
//! `scsi_device_prop_update_inqstring`, `scsi_device_identity`,
//! `scsi_get_scsi_maxluns`, `scsi_get_scsi_options` and
//! `scsi_initialize_hba_interface`, and the flags `SCSI_HBA_TRAN_PHCI`,
//! `SCSI_HBA_ADDR_SPI`, `SCSI_HBA_SCSA_PHCI`, `SCSI_HBA_SCSA_TA` and
//! `SCSI_HBA_SCSA_FM`, the `SCSA_FLAVOR_*` values, `SCSI_HBA_MAX_IPORTS`
//! and the minor number macros. None of them has a manual page, so they are
//! not bound.

use ddi_sys::{
    boolean_t, buf, c_char, c_int, c_uchar, c_void, caddr_t, ddi_callback_id_t, ddi_dma_attr_t, ddi_event_cb_f,
    ddi_eventcookie_t, dev_info_t, kmem_cache_t, kmutex_t, modlinkage, opaque, uint_t,
};

use crate::device::{scsi_address, scsi_device};
use crate::pkt::{scsi_callback_t, scsi_pkt};

opaque! {
    /// `scsi_hba_iportmap_create(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// An iport map, `scsi_hba_iportmap_t`, which C declares as the private
    /// `struct __scsi_iportmap`. It is created by
    /// [`scsi_hba_iportmap_create`] and only ever handled through a pointer.
    scsi_hba_iportmap_t;
    /// `scsi_hba_tgtmap_create(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// A target map, `scsi_hba_tgtmap_t`, which C declares as the private
    /// `struct __scsi_tgtmap`. It is created by [`scsi_hba_tgtmap_create`]
    /// and only ever handled through a pointer.
    scsi_hba_tgtmap_t;
}

// --- entry points --------------------------------------------------------

/// `tran_tgt_init(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point the system calls when an instance of a
/// target device is being created, after its device node exists but before
/// its `probe(9E)` runs. `hba_dip` is the HBA's device node, `tgt_dip` the
/// target's, `tran` the HBA's transport structure and `sd` the target's
/// `scsi_device(9S)`. The HBA driver may initialise whatever it needs for
/// this target. If it registered with `SCSI_HBA_TRAN_CLONE`, `tran` is a
/// copy made for this target and the driver may set `tran_tgt_private`;
/// with complex addressing it stores its per-target data with
/// `scsi_device_hba_private_set(9F)` instead. The routine returns
/// `DDI_SUCCESS` if the HBA can support the target and prepared its
/// resources, or `DDI_FAILURE` otherwise, in which case the target's
/// initialisation stops, its `probe(9E)` is not called, and its device node
/// is destroyed.
pub type tran_tgt_init_t = unsafe extern "C" fn(
    hba_dip: dev_info_t,
    tgt_dip: dev_info_t,
    tran: *mut scsi_hba_tran,
    sd: *mut scsi_device,
) -> c_int;

/// `tran_tgt_probe(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that `scsi_probe(9F)` calls when a
/// target driver probes for its target from `probe(9E)` or `attach(9E)`.
/// `sd` describes the target and `waitfunc` is `NULL_FUNC` or `SLEEP_FUNC`,
/// saying whether the routine may wait for resources. The HBA driver may
/// point the vector straight at [`scsi_hba_probe`], or at a routine of its
/// own that calls `scsi_hba_probe` before or after other processing, as long
/// as the semantics of `scsi_probe(9F)`, including its `SCSIPROBE_*` return
/// values, are preserved. In C `waitfunc` is `int (*)(void)`; it is declared
/// as [`scsi_callback_t`] because it may hold the two sentinel values.
pub type tran_tgt_probe_t = unsafe extern "C" fn(sd: *mut scsi_device, waitfunc: scsi_callback_t) -> c_int;

/// `tran_tgt_free(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point the system calls when an instance of a
/// target device is being detached, after the target's `detach(9E)` has
/// returned successfully but before its device node is removed. The
/// arguments are those of `tran_tgt_init(9E)`. The HBA driver releases
/// whatever it allocated for this target in `tran_tgt_init(9E)` or
/// `tran_tgt_probe(9E)`.
pub type tran_tgt_free_t =
    unsafe extern "C" fn(hba_dip: dev_info_t, tgt_dip: dev_info_t, tran: *mut scsi_hba_tran, sd: *mut scsi_device);

/// `tran_start(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver calls
/// `scsi_transport(9F)`. It must start the command in `pkt` on the target
/// and logical unit that `ap` names. If `FLAG_NOINTR` is set in `pkt_flags`
/// it returns only once the command has completed and does not call
/// `pkt_comp`; otherwise it queues the command and returns at once, and the
/// completion routine is called later. It initialises `pkt_resid`,
/// `pkt_reason`, `pkt_statistics` and `pkt_state` and keeps them up to date,
/// and it performs automatic request sense when that is enabled and the
/// status area is large enough for a `scsi_arq_status(9S)`. It enforces
/// `pkt_time`, recovers from timeouts as the manual page describes, and
/// honours the `scsi-reset-delay` after a reset. It returns `TRAN_ACCEPT`,
/// `TRAN_BUSY`, `TRAN_BADPKT` or `TRAN_FATAL_ERROR`. It may be called from
/// user or interrupt context.
pub type tran_start_t = unsafe extern "C" fn(ap: *mut scsi_address, pkt: *mut scsi_pkt) -> c_int;

/// `tran_reset(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver calls
/// `scsi_reset(9F)`. It resets the bus if `level` is `RESET_ALL`, the target
/// named by `ap` if it is `RESET_TARGET`, or the logical unit named by `ap`
/// if it is `RESET_LUN`. Outstanding packets of the reset targets or logical
/// units get `pkt_reason` set to `CMD_RESET` and `STAT_BUS_RESET` or
/// `STAT_DEV_RESET` added to `pkt_statistics`; packets not yet active on the
/// bus get `CMD_RESET` and `STAT_ABORTED`. Support for `RESET_LUN` is
/// optional, and a driver that offers it must also provide the `lun-reset`
/// capability. The routine returns 1 on success and 0 on failure. It may be
/// called from user or interrupt context.
pub type tran_reset_t = unsafe extern "C" fn(ap: *mut scsi_address, level: c_int) -> c_int;

/// `tran_abort(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver calls
/// `scsi_abort(9F)`. It tries to abort the command `pkt` that was
/// transported to the HBA or, if `pkt` is NULL, every outstanding command of
/// the target and logical unit that `ap` names. Each packet that is aborted
/// gets `pkt_reason` set to `CMD_ABORTED` and `STAT_ABORTED` added to
/// `pkt_statistics`. The routine returns 1 on success or partial success and
/// 0 on failure. It may be called from user or interrupt context.
pub type tran_abort_t = unsafe extern "C" fn(ap: *mut scsi_address, pkt: *mut scsi_pkt) -> c_int;

/// `tran_getcap(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver calls
/// `scsi_ifgetcap(9F)`. It returns the current value of the capability
/// named by the NUL terminated string `cap`, for the target that `ap` names
/// if `whom` is non-zero, or -1 if the capability is not defined. The HBA
/// driver should match `cap` with [`scsi_hba_lookup_capstr`].
pub type tran_getcap_t = unsafe extern "C" fn(ap: *mut scsi_address, cap: *mut c_char, whom: c_int) -> c_int;

/// `tran_setcap(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver calls
/// `scsi_ifsetcap(9F)`. It sets the capability named by `cap` to `value`,
/// for the target that `ap` names if `whom` is non-zero or for all targets
/// if it is 0; the manual recommends that HBA drivers not support the
/// latter. It returns 1 if the capability was set, 0 if the HBA driver does
/// not support changing it, and -1 if it is not defined.
pub type tran_setcap_t =
    unsafe extern "C" fn(ap: *mut scsi_address, cap: *mut c_char, value: c_int, whom: c_int) -> c_int;

/// `tran_init_pkt(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver calls
/// `scsi_init_pkt(9F)`. If `pkt` is NULL, the HBA driver allocates a new
/// packet with [`scsi_hba_pkt_alloc`], using `cmdlen`, `statuslen` and
/// `tgtlen` for the sizes of the command block, the status block and the
/// target's private area. If `bp` is not NULL, it also allocates DMA
/// resources for the buffer, partially if `flags` contains
/// `PKT_DMA_PARTIAL`, reporting the unmapped remainder in `pkt_resid`; a
/// later call with the same packet and buffer moves the resources to the
/// next part of the transfer. `PKT_CONSISTENT` in `flags` means the buffer
/// came from `scsi_alloc_consistent_buf(9F)` and must be synchronised before
/// the completion callback. A `statuslen` of at least the size of
/// `scsi_arq_status(9S)` enables automatic request sense if the capability
/// is set. `callback` is `NULL_FUNC` or `SLEEP_FUNC` and `arg` is always
/// NULL. The routine returns the packet, or NULL on failure, in which case a
/// packet it allocated itself must be freed with [`scsi_hba_pkt_free`]
/// first. In C `callback` is `int (*)(caddr_t)`; it is declared as
/// [`scsi_callback_t`] because it may hold the two sentinel values.
pub type tran_init_pkt_t = unsafe extern "C" fn(
    ap: *mut scsi_address,
    pkt: *mut scsi_pkt,
    bp: *mut buf,
    cmdlen: c_int,
    statuslen: c_int,
    tgtlen: c_int,
    flags: c_int,
    callback: scsi_callback_t,
    arg: caddr_t,
) -> *mut scsi_pkt;

/// `tran_destroy_pkt(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver calls
/// `scsi_destroy_pkt(9F)`. It frees every resource that `tran_init_pkt(9E)`
/// allocated for `pkt`, including the packet itself.
pub type tran_destroy_pkt_t = unsafe extern "C" fn(ap: *mut scsi_address, pkt: *mut scsi_pkt);

/// `tran_dmafree(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver frees the
/// DMA resources of a packet without freeing the packet. It releases the DMA
/// resources that `tran_init_pkt(9E)` allocated for `pkt` but not the packet
/// itself, so the HBA driver must keep track of which packets have DMA
/// resources, since `tran_destroy_pkt(9E)` frees them as well. It is not
/// called for HBA drivers that provide `tran_setup_pkt(9E)`.
pub type tran_dmafree_t = unsafe extern "C" fn(ap: *mut scsi_address, pkt: *mut scsi_pkt);

/// `tran_sync_pkt(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver calls
/// `scsi_sync_pkt(9F)`. It synchronises the HBA's or device's view of the
/// data of `pkt`, typically with `ddi_dma_sync(9F)`, which may involve
/// flushing I/O caches or waiting for hardware buffers to drain. It is not
/// called for HBA drivers that provide `tran_setup_pkt(9E)`.
pub type tran_sync_pkt_t = unsafe extern "C" fn(ap: *mut scsi_address, pkt: *mut scsi_pkt);

/// `tran_reset_notify(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when a target driver calls
/// `scsi_reset_notify(9F)`. With `flag` set to `SCSI_RESET_NOTIFY` the HBA
/// driver records `callback` and `arg` for the target that `ap` names, and
/// calls `callback` with `arg` when the bus is reset. With
/// `SCSI_RESET_CANCEL` it removes that request. It returns `DDI_SUCCESS` if
/// the request was accepted or cancelled and `DDI_FAILURE` otherwise.
pub type tran_reset_notify_t = unsafe extern "C" fn(
    ap: *mut scsi_address,
    flag: c_int,
    callback: Option<unsafe extern "C" fn(arg: caddr_t)>,
    arg: caddr_t,
) -> c_int;

/// `tran_bus_reset(9E)`. illumos DDI.
///
/// The signature of the entry point called when a user asks for a bus reset
/// through the device control interfaces. It resets the bus of the HBA
/// `hba_dip` without resetting the targets; `level` is `RESET_BUS`. If the
/// hardware cannot do that without changing the state of the targets, the
/// driver leaves the vector empty or returns failure. It returns 1 on
/// success and 0 on failure.
pub type tran_bus_reset_t = unsafe extern "C" fn(hba_dip: dev_info_t, level: c_int) -> c_int;

/// `tran_quiesce(9E)`. illumos DDI.
///
/// The signature of the entry point called when a user asks to quiesce the
/// bus through the device control interfaces. It waits for all outstanding
/// commands on the HBA `hba_dip` to complete and blocks or queues new
/// requests. It returns 0 on success and non-zero on error.
pub type tran_quiesce_t = unsafe extern "C" fn(hba_dip: dev_info_t) -> c_int;

/// `tran_unquiesce(9E)`. illumos DDI.
///
/// The signature of the entry point called when a user asks to unquiesce
/// the bus through the device control interfaces. It lets I/O on the HBA
/// `hba_dip` resume. It returns 0 on success and non-zero on error.
pub type tran_unquiesce_t = unsafe extern "C" fn(hba_dip: dev_info_t) -> c_int;

/// `tran_setup_pkt(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that, together with
/// `tran_teardown_pkt(9E)`, replaces `tran_init_pkt(9E)` and
/// `tran_destroy_pkt(9E)`. The framework allocates the packet and calls
/// this routine to initialise the HBA specific parts of `pkt`. All members
/// that `scsi_pkt(9S)` documents are already set. If the HBA driver did not
/// preallocate the command and status blocks, it allocates them now. At
/// `tran_start(9E)` time it finds the DMA resources in `pkt_numcookies` and
/// `pkt_cookies`. `callback` is `NULL_FUNC` or `SLEEP_FUNC` and `arg` is
/// always NULL. The routine returns 0 on success and -1 on failure. A driver
/// that uses it must signal completion through `scsi_hba_pkt_comp(9F)`. In C
/// `callback` is `int (*)(caddr_t)`; it is declared as [`scsi_callback_t`]
/// because it may hold the two sentinel values.
pub type tran_setup_pkt_t = unsafe extern "C" fn(pkt: *mut scsi_pkt, callback: scsi_callback_t, arg: caddr_t) -> c_int;

/// `tran_teardown_pkt(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that frees all resources
/// `tran_setup_pkt(9E)` allocated for `pkt`.
pub type tran_teardown_pkt_t = unsafe extern "C" fn(pkt: *mut scsi_pkt);

/// `tran_pkt_constructor(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of an optional constructor for packets of an HBA driver
/// that uses `tran_setup_pkt(9E)`. It is called when the framework's packet
/// cache creates `pkt`, after `pkt_address`, `pkt_ha_private`, `pkt_cdbp`,
/// `pkt_private`, `pkt_scbp`, `pkt_cdblen`, `pkt_tgtlen` and `pkt_scblen`
/// have been set, and may for instance allocate a DMA handle. `tran` is the
/// HBA's transport structure and `kmflag` is `KM_SLEEP` or `KM_NOSLEEP`. The
/// restrictions of `kmem_cache_create(9F)` constructors apply. It returns 0
/// on success and -1 on failure.
pub type tran_pkt_constructor_t =
    unsafe extern "C" fn(pkt: *mut scsi_pkt, tran: *mut scsi_hba_tran, kmflag: c_int) -> c_int;

/// `tran_pkt_destructor(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the optional destructor that undoes what
/// `tran_pkt_constructor(9E)` did for `pkt` before the packet cache frees it.
pub type tran_pkt_destructor_t = unsafe extern "C" fn(pkt: *mut scsi_pkt, tran: *mut scsi_hba_tran);

// --- scsi_hba_tran(9S) ---------------------------------------------------

/// `scsi_hba_tran(9S)`. illumos architecture specific (illumos DDI).
///
/// The transport vector that an HBA driver exports to the framework, so
/// that HBA specific functions can be executed on behalf of target drivers.
/// The driver allocates it with [`scsi_hba_tran_alloc`], fills in its
/// private pointer and entry points, and registers it with
/// [`scsi_hba_attach_setup`]. Every `scsi_address(9S)` below the HBA points
/// to it, or to a per-target copy when the driver registered with
/// [`SCSI_HBA_TRAN_CLONE`].
///
/// The header marks everything from [`tran_hba_flags`](Self::tran_hba_flags)
/// onward as private to the implementation, except for the members that
/// `scsi_hba_tran(9S)` documents, which are
/// [`tran_interconnect_type`](Self::tran_interconnect_type), the four
/// `tran_setup_pkt(9E)` vectors and [`tran_hba_len`](Self::tran_hba_len).
/// The private members are declared for the sake of an exact layout. The
/// structure is always allocated by the framework, and drivers must not
/// depend on its size.
#[repr(C)]
pub struct scsi_hba_tran {
    /// The device node of the HBA that supplies this structure. It also
    /// refers to the HBA when the driver separates the initiator port
    /// function, and is used to manage DMA.
    pub tran_hba_dip: dev_info_t,
    /// A pointer the HBA driver may use to reach its soft state.
    pub tran_hba_private: *mut c_void,
    /// A pointer the HBA driver may use for per-target data. It is only
    /// usable with [`SCSI_HBA_TRAN_CLONE`], in which case the driver sets it
    /// in `tran_tgt_init(9E)`. Complex addressing is the preferred
    /// replacement.
    pub tran_tgt_private: *mut c_void,
    /// The target's `scsi_device(9S)` when the structure is a clone, and
    /// NULL otherwise.
    pub tran_sd: *mut scsi_device,
    /// Per-target initialisation, if necessary. See [`tran_tgt_init_t`].
    pub tran_tgt_init: Option<tran_tgt_init_t>,
    /// Per-target customisation of `scsi_probe(9F)`, if necessary. See
    /// [`tran_tgt_probe_t`].
    pub tran_tgt_probe: Option<tran_tgt_probe_t>,
    /// Per-target deallocation, if necessary. See [`tran_tgt_free_t`].
    pub tran_tgt_free: Option<tran_tgt_free_t>,
    /// Starts a command on the hardware. See [`tran_start_t`].
    pub tran_start: Option<tran_start_t>,
    /// Resets the bus, a target or a logical unit. See [`tran_reset_t`].
    pub tran_reset: Option<tran_reset_t>,
    /// Aborts one or all commands. See [`tran_abort_t`].
    pub tran_abort: Option<tran_abort_t>,
    /// Retrieves a capability. See [`tran_getcap_t`].
    pub tran_getcap: Option<tran_getcap_t>,
    /// Sets a capability. See [`tran_setcap_t`].
    pub tran_setcap: Option<tran_setcap_t>,
    /// Allocates a packet and its DMA resources. See [`tran_init_pkt_t`].
    pub tran_init_pkt: Option<tran_init_pkt_t>,
    /// Frees a packet allocated by `tran_init_pkt`. See
    /// [`tran_destroy_pkt_t`].
    pub tran_destroy_pkt: Option<tran_destroy_pkt_t>,
    /// Frees the DMA resources of a packet. Not called for drivers that
    /// provide `tran_setup_pkt`. See [`tran_dmafree_t`].
    pub tran_dmafree: Option<tran_dmafree_t>,
    /// Synchronises the data of a packet. Not called for drivers that
    /// provide `tran_setup_pkt`. See [`tran_sync_pkt_t`].
    pub tran_sync_pkt: Option<tran_sync_pkt_t>,
    /// Registers or cancels a bus reset notification. See
    /// [`tran_reset_notify_t`].
    pub tran_reset_notify: Option<tran_reset_notify_t>,
    /// Returns the bus address of a device. Private to the framework; it has
    /// no manual page.
    pub tran_get_bus_addr: Option<unsafe extern "C" fn(sd: *mut scsi_device, name: *mut c_char, len: c_int) -> c_int>,
    /// Returns the name of a device. Private to the framework; it has no
    /// manual page.
    pub tran_get_name: Option<unsafe extern "C" fn(sd: *mut scsi_device, name: *mut c_char, len: c_int) -> c_int>,
    /// Clears an auto contingent allegiance condition. Private to the
    /// framework; it has no manual page.
    pub tran_clear_aca: Option<unsafe extern "C" fn(ap: *mut scsi_address) -> c_int>,
    /// Clears the task set of a logical unit. Private to the framework; it
    /// has no manual page.
    pub tran_clear_task_set: Option<unsafe extern "C" fn(ap: *mut scsi_address) -> c_int>,
    /// Terminates a task. Private to the framework; it has no manual page.
    pub tran_terminate_task: Option<unsafe extern "C" fn(ap: *mut scsi_address, pkt: *mut scsi_pkt) -> c_int>,
    /// Looks up an NDI event cookie for a target. Private to the framework;
    /// it has no manual page.
    pub tran_get_eventcookie: Option<
        unsafe extern "C" fn(
            hba_dip: dev_info_t,
            tgt_dip: dev_info_t,
            name: *mut c_char,
            eventp: *mut ddi_eventcookie_t,
        ) -> c_int,
    >,
    /// Registers an NDI event callback for a target. Private to the
    /// framework; it has no manual page.
    pub tran_add_eventcall: Option<
        unsafe extern "C" fn(
            hba_dip: dev_info_t,
            tgt_dip: dev_info_t,
            event: ddi_eventcookie_t,
            callback: Option<ddi_event_cb_f>,
            arg: *mut c_void,
            cb_id: *mut ddi_callback_id_t,
        ) -> c_int,
    >,
    /// Removes an NDI event callback. Private to the framework; it has no
    /// manual page.
    pub tran_remove_eventcall: Option<unsafe extern "C" fn(devi: dev_info_t, cb_id: ddi_callback_id_t) -> c_int>,
    /// Posts an NDI event for a target. Private to the framework; it has no
    /// manual page.
    pub tran_post_event: Option<
        unsafe extern "C" fn(
            hba_dip: dev_info_t,
            tgt_dip: dev_info_t,
            event: ddi_eventcookie_t,
            bus_impldata: *mut c_void,
        ) -> c_int,
    >,
    /// Quiesces the bus. See [`tran_quiesce_t`].
    pub tran_quiesce: Option<tran_quiesce_t>,
    /// Lets I/O on the bus resume. See [`tran_unquiesce_t`].
    pub tran_unquiesce: Option<tran_unquiesce_t>,
    /// Resets the bus without resetting the targets. See
    /// [`tran_bus_reset_t`].
    pub tran_bus_reset: Option<tran_bus_reset_t>,
    /// The flags the HBA registered with. Private to the framework.
    pub tran_hba_flags: c_int,
    /// Obsolete and unused.
    pub tran_obs1: uint_t,
    /// Obsolete and unused.
    pub tran_obs2: c_uchar,
    /// Obsolete and unused.
    pub tran_obs3: c_uchar,
    /// Protects [`tran_open_flag`](Self::tran_open_flag). Private to the
    /// framework.
    pub tran_open_lock: kmutex_t,
    /// Which minor nodes of the HBA are open, used by the framework's own
    /// `open(9E)` implementation. Private to the framework.
    pub tran_open_flag: u64,
    /// The bus configuration entry point. Consolidation private and subject
    /// to change. Its `op` argument is a `ddi_bus_config_op_t`, an
    /// enumeration that has the size of an `int`.
    pub tran_bus_config: Option<
        unsafe extern "C" fn(
            hba_dip: dev_info_t,
            flag: uint_t,
            op: c_int,
            arg: *mut c_void,
            tgt_dipp: *mut dev_info_t,
        ) -> c_int,
    >,
    /// The bus unconfiguration entry point. Consolidation private and subject
    /// to change. Its `op` argument is a `ddi_bus_config_op_t`, an
    /// enumeration that has the size of an `int`.
    pub tran_bus_unconfig:
        Option<unsafe extern "C" fn(hba_dip: dev_info_t, flag: uint_t, op: c_int, arg: *mut c_void) -> c_int>,
    /// The bus power entry point. Consolidation private and subject to
    /// change. Its `op` argument is a `pm_bus_power_op_t`, an enumeration
    /// that has the size of an `int`.
    pub tran_bus_power: Option<
        unsafe extern "C" fn(
            dip: dev_info_t,
            impl_arg: *mut c_void,
            op: c_int,
            arg: *mut c_void,
            result: *mut c_void,
        ) -> c_int,
    >,
    /// The interconnect type of the transport, one of the `INTERCONNECT_*`
    /// values. It is what the `interconnect-type` capability reports.
    pub tran_interconnect_type: c_int,
    /// The optional packet constructor. See [`tran_pkt_constructor_t`].
    pub tran_pkt_constructor: Option<tran_pkt_constructor_t>,
    /// The optional packet destructor. See [`tran_pkt_destructor_t`].
    pub tran_pkt_destructor: Option<tran_pkt_destructor_t>,
    /// The framework's packet cache for drivers that use `tran_setup_pkt`.
    /// Private to the framework.
    pub tran_pkt_cache_ptr: *mut kmem_cache_t,
    /// The size of the HBA private area that `pkt_ha_private` points to in
    /// packets set up through `tran_setup_pkt(9E)`.
    pub tran_hba_len: uint_t,
    /// The optional packet initialisation entry point. See
    /// [`tran_setup_pkt_t`].
    pub tran_setup_pkt: Option<tran_setup_pkt_t>,
    /// Releases what `tran_setup_pkt` allocated. See
    /// [`tran_teardown_pkt_t`].
    pub tran_teardown_pkt: Option<tran_teardown_pkt_t>,
    /// A copy of the DMA attributes the HBA registered. Private to the
    /// framework.
    pub tran_dma_attr: ddi_dma_attr_t,
    /// Private data allocated by the framework for the HBA. Private to the
    /// framework.
    pub tran_extension: *mut c_void,
    /// The fault management capabilities of the HBA. The header allows a
    /// driver to set it before calling `scsi_hba_attach_setup(9F)`; if it
    /// does not, the framework supplies a default.
    pub tran_fm_capable: c_int,
    /// The device node of the initiator port, when the HBA driver separates
    /// the initiator port function from the HBA. Private to the framework.
    pub tran_iport_dip: dev_info_t,
    /// The iport map below the HBA. Private to the framework.
    pub tran_iportmap: *mut scsi_hba_iportmap_t,
    /// The target map below the initiator port. Private to the framework.
    pub tran_tgtmap: *mut scsi_hba_tgtmap_t,
}
/// The typedef name of [`scsi_hba_tran`], which the manual pages use.
pub type scsi_hba_tran_t = scsi_hba_tran;

// Layout on amd64 with LP64, as `sys/scsi/impl/transport.h` declares it
// without `SCSI_SIZE_CLEAN_VERIFY`. The four pointers tran_hba_dip to
// tran_sd occupy 0 to 32, and the 25 function pointers tran_tgt_init to
// tran_bus_reset occupy 32 to 232, with tran_start at 56, tran_init_pkt at
// 96, tran_reset_notify at 128 and tran_quiesce at 208. tran_hba_flags 4 at
// 232, tran_obs1 4 at 236, tran_obs2 1 at 240, tran_obs3 1 at 241 and 6
// bytes of padding, tran_open_lock 8 at 248, tran_open_flag 8 at 256,
// tran_bus_config 8 at 264, tran_bus_unconfig 8 at 272, tran_bus_power 8 at
// 280, tran_interconnect_type 4 at 288 and 4 bytes of padding,
// tran_pkt_constructor 8 at 296, tran_pkt_destructor 8 at 304,
// tran_pkt_cache_ptr 8 at 312, tran_hba_len 4 at 320 and 4 bytes of padding,
// tran_setup_pkt 8 at 328, tran_teardown_pkt 8 at 336, tran_dma_attr 80 at
// 344, tran_extension 8 at 424, tran_fm_capable 4 at 432 and 4 bytes of
// padding, tran_iport_dip 8 at 440, tran_iportmap 8 at 448, tran_tgtmap 8 at
// 456; total 464.
const _: () = assert!(core::mem::size_of::<scsi_hba_tran>() == 464);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_hba_dip) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_hba_private) == 8);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_sd) == 24);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_tgt_init) == 32);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_start) == 56);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_init_pkt) == 96);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_reset_notify) == 128);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_quiesce) == 208);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_bus_reset) == 224);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_hba_flags) == 232);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_obs3) == 241);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_open_lock) == 248);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_open_flag) == 256);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_bus_config) == 264);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_interconnect_type) == 288);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_pkt_constructor) == 296);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_pkt_cache_ptr) == 312);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_hba_len) == 320);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_setup_pkt) == 328);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_dma_attr) == 344);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_extension) == 424);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_fm_capable) == 432);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_iport_dip) == 440);
const _: () = assert!(core::mem::offset_of!(scsi_hba_tran, tran_tgtmap) == 456);

// --- flags ---------------------------------------------------------------

/// Clone the transport structure once for each target, so that
/// `tran_tgt_private` can hold per-target data. The use of this flag is
/// deprecated in favour of [`SCSI_HBA_ADDR_COMPLEX`] (`SCSI_HBA_TRAN_CLONE`).
pub const SCSI_HBA_TRAN_CLONE: c_int = 0x01;
/// Have the framework allocate the command descriptor block of packets set
/// up through `tran_setup_pkt(9E)` (`SCSI_HBA_TRAN_CDB`).
pub const SCSI_HBA_TRAN_CDB: c_int = 0x04;
/// Have the framework allocate the status block of packets set up through
/// `tran_setup_pkt(9E)` (`SCSI_HBA_TRAN_SCB`).
pub const SCSI_HBA_TRAN_SCB: c_int = 0x08;
/// The HBA driver only enumerates direct children that are iports. The
/// manual recommends it, together with [`SCSI_HBA_ADDR_COMPLEX`], for new
/// SAS HBA drivers (`SCSI_HBA_HBA`).
pub const SCSI_HBA_HBA: c_int = 0x10;
/// The HBA handles complex topologies such as SAS. Its `scsi_address(9S)`
/// structures become opaque, devices are identified by unit address
/// strings, and the functions of `scsi_address_device(9F)` become available
/// (`SCSI_HBA_ADDR_COMPLEX`).
pub const SCSI_HBA_ADDR_COMPLEX: c_int = 0x40;
/// Passed to [`scsi_hba_tran_alloc`], allows the allocation to sleep
/// (`SCSI_HBA_CANSLEEP`).
pub const SCSI_HBA_CANSLEEP: c_int = 0x01;

// --- target maps ---------------------------------------------------------

/// `scsi_hba_tgtmap_create(9F)`. Evolving - This interface is still evolving in illumos.
///
/// How addresses are reported into a target map, the C enumeration
/// `scsi_tgtmap_mode_t`. Its values are [`SCSI_TM_FULLSET`] and
/// [`SCSI_TM_PERADDR`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_tgtmap_mode_t(pub c_int);
/// The driver reports the full set of present devices each time, and the
/// map works out what was added and removed (`SCSI_TM_FULLSET`).
pub const SCSI_TM_FULLSET: scsi_tgtmap_mode_t = scsi_tgtmap_mode_t(0);
/// The driver reports each addition and removal itself with
/// [`scsi_hba_tgtmap_tgt_add`] and [`scsi_hba_tgtmap_tgt_remove`]
/// (`SCSI_TM_PERADDR`).
pub const SCSI_TM_PERADDR: scsi_tgtmap_mode_t = scsi_tgtmap_mode_t(1);

/// `scsi_hba_tgtmap_create(9F)`. Evolving - This interface is still evolving in illumos.
///
/// The kind of a target in a target map, the C enumeration
/// `scsi_tgtmap_tgt_type_t`. Its values are [`SCSI_TGT_SCSI_DEVICE`] and
/// [`SCSI_TGT_SMP_DEVICE`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_tgtmap_tgt_type_t(pub c_int);
/// Some form of SCSI device, whether parallel SCSI, SAS, SATA behind a
/// translation layer or an enclosure (`SCSI_TGT_SCSI_DEVICE`).
pub const SCSI_TGT_SCSI_DEVICE: scsi_tgtmap_tgt_type_t = scsi_tgtmap_tgt_type_t(0);
/// A SCSI Management Protocol device (`SCSI_TGT_SMP_DEVICE`).
pub const SCSI_TGT_SMP_DEVICE: scsi_tgtmap_tgt_type_t = scsi_tgtmap_tgt_type_t(1);

/// `scsi_hba_tgtmap_create(9F)`. Evolving - This interface is still evolving in illumos.
///
/// Why a target is being removed from a target map, the C enumeration
/// `scsi_tgtmap_deact_rsn_t`, which the deactivation callback receives.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_tgtmap_deact_rsn_t(pub c_int);
/// The device is gone (`SCSI_TGT_DEACT_RSN_GONE`).
pub const SCSI_TGT_DEACT_RSN_GONE: scsi_tgtmap_deact_rsn_t = scsi_tgtmap_deact_rsn_t(0);
/// The configuration callback failed (`SCSI_TGT_DEACT_RSN_CFG_FAIL`).
pub const SCSI_TGT_DEACT_RSN_CFG_FAIL: scsi_tgtmap_deact_rsn_t = scsi_tgtmap_deact_rsn_t(1);
/// The device was added and removed again during the stabilisation period
/// and is considered unstable (`SCSI_TGT_DEACT_RSN_UNSTBL`).
pub const SCSI_TGT_DEACT_RSN_UNSTBL: scsi_tgtmap_deact_rsn_t = scsi_tgtmap_deact_rsn_t(2);

/// `scsi_hba_tgtmap_create(9F)`. Evolving - This interface is still evolving in illumos.
///
/// The signature of the optional activation callback of a target map, whose
/// C typedef `scsi_tgt_activate_cb_t` is a pointer to such a function. It
/// is called in kernel context, asynchronously, when a new address has been
/// reported and before any driver is bound to it. `tgtmap_priv` is the value
/// passed to [`scsi_hba_tgtmap_create`], and `tgt_addr` and `tgt_type` are
/// those the address was reported with. `tgt_privp` initially points to the
/// `tgt_priv` value the address was reported with, and the callback may
/// change it; the deactivation callback receives the final value.
pub type scsi_tgt_activate_cb_t = unsafe extern "C" fn(
    tgtmap_priv: *mut c_void,
    tgt_addr: *mut c_char,
    tgt_type: scsi_tgtmap_tgt_type_t,
    tgt_privp: *mut *mut c_void,
);

/// `scsi_hba_tgtmap_create(9F)`. Evolving - This interface is still evolving in illumos.
///
/// The signature of the optional deactivation callback of a target map,
/// whose C typedef `scsi_tgt_deactivate_cb_t` is a pointer to such a
/// function. It is called in kernel context after the driver has been
/// detached from a device that was removed from the map. The first three
/// arguments are those of the activation callback, `tgt_priv` is the
/// private value as the activation callback left it, and `tgt_deact_rsn`
/// says why the device is being removed. The return value asks for the
/// address to be rediscovered and only matters for
/// [`SCSI_TGT_DEACT_RSN_CFG_FAIL`]. The manual advises returning `B_FALSE`
/// when in doubt, and notes that the system may take no action even for
/// `B_TRUE`.
pub type scsi_tgt_deactivate_cb_t = unsafe extern "C" fn(
    tgtmap_priv: *mut c_void,
    tgt_addr: *mut c_char,
    tgt_type: scsi_tgtmap_tgt_type_t,
    tgt_priv: *mut c_void,
    tgt_deact_rsn: scsi_tgtmap_deact_rsn_t,
) -> boolean_t;

unsafe extern "C" {
    // --- module and attach -----------------------------------------------

    /// `scsi_hba_init(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Registers an HBA driver with the system, so that it can accept
    /// configuration requests on behalf of SCSI target drivers. `modlp` is
    /// the driver's module linkage. The driver calls it from `_init(9E)`
    /// before `mod_install(9F)`, and calls [`scsi_hba_fini`] if
    /// `mod_install` fails. Returns 0 on success and non-zero on failure, in
    /// which case `_init` should return that value.
    pub fn scsi_hba_init(modlp: *mut modlinkage) -> c_int;
    /// `scsi_hba_fini(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Removes all the system references to the HBA driver that
    /// [`scsi_hba_init`] created. `modlp` is the driver's module linkage. The
    /// driver calls it from `_fini(9E)` once `mod_remove(9F)` has succeeded.
    pub fn scsi_hba_fini(modlp: *mut modlinkage);
    /// `scsi_hba_attach_setup(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Registers the DMA attributes `hba_dma_attr` and the transport vector
    /// `tran` for the HBA instance `hba_dip`; each instance may pass its own.
    /// `flags` is a combination of [`SCSI_HBA_TRAN_CLONE`],
    /// [`SCSI_HBA_TRAN_CDB`], [`SCSI_HBA_TRAN_SCB`], [`SCSI_HBA_HBA`] and
    /// [`SCSI_HBA_ADDR_COMPLEX`]. The function sets the `devo_bus_ops` of the
    /// driver's `dev_ops(9S)`, which the driver should leave NULL, and adds
    /// the properties `scsi-options`, `scsi-reset-delay` and
    /// `scsi-selection-timeout` to the node unless they already exist; the
    /// HBA driver should respect them. Returns `DDI_SUCCESS` or
    /// `DDI_FAILURE`. It should be called from `attach(9E)`.
    pub fn scsi_hba_attach_setup(
        hba_dip: dev_info_t,
        hba_dma_attr: *mut ddi_dma_attr_t,
        tran: *mut scsi_hba_tran,
        flags: c_int,
    ) -> c_int;
    /// `scsi_hba_detach(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Removes the references to the DMA attributes and the transport vector
    /// that [`scsi_hba_attach_setup`] registered for the HBA instance
    /// `hba_dip`. Returns `DDI_SUCCESS` or `DDI_FAILURE`. It should be called
    /// from `detach(9E)`.
    pub fn scsi_hba_detach(hba_dip: dev_info_t) -> c_int;
    /// `scsi_hba_tran_alloc(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Allocates a `scsi_hba_tran(9S)` structure for the HBA instance
    /// `hba_dip`. The HBA must use this structure to register its transport
    /// vectors with [`scsi_hba_attach_setup`]. The only flag is
    /// [`SCSI_HBA_CANSLEEP`], which allows the allocation to sleep; without
    /// it the call does not sleep and may fail. Returns the structure, or
    /// NULL if no memory was available. It may be called from user,
    /// interrupt or kernel context, but must not be allowed to sleep when
    /// called from an interrupt routine.
    pub fn scsi_hba_tran_alloc(hba_dip: dev_info_t, flags: c_int) -> *mut scsi_hba_tran;
    /// `scsi_hba_tran_free(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Frees a structure that [`scsi_hba_tran_alloc`] allocated. It may be
    /// called from user, interrupt or kernel context.
    pub fn scsi_hba_tran_free(tran: *mut scsi_hba_tran);
    /// `scsi_hba_probe(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// The default implementation of `tran_tgt_probe(9E)`, with the
    /// semantics and return values of `scsi_probe(9F)`. `sd` describes the
    /// target and `waitfunc` is `NULL_FUNC` or `SLEEP_FUNC`. An HBA driver
    /// may call it from its own `tran_tgt_probe(9E)` or point that vector at
    /// it directly, and should call it from nowhere else. In C `waitfunc` is
    /// `int (*)(void)`; it is declared as [`scsi_callback_t`] because it may
    /// hold the two sentinel values.
    pub fn scsi_hba_probe(sd: *mut scsi_device, waitfunc: scsi_callback_t) -> c_int;

    // --- packets ---------------------------------------------------------

    /// `scsi_hba_pkt_alloc(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Allocates a `scsi_pkt(9S)` for the HBA instance `hba_dip` and the
    /// target `ap`, together with a command block of `cmdlen` bytes, a status
    /// block of `statuslen` bytes, a private area of `tgtlen` bytes for the
    /// target driver and one of `hbalen` bytes for the HBA driver, and points
    /// `pkt_cdbp`, `pkt_scbp`, `pkt_private` and `pkt_ha_private` at them. It
    /// copies `ap` into `pkt_address`. HBA drivers must use it to allocate
    /// packets in `tran_init_pkt(9E)`. `callback` is `NULL_FUNC`, in which
    /// case the call does not sleep and may fail, or `SLEEP_FUNC`, and `arg`
    /// must be NULL. Returns the packet, or NULL if no memory was available.
    /// It may be called from user, interrupt or kernel context, but must not
    /// be allowed to sleep when called from an interrupt routine. In C
    /// `callback` is `int (*)(caddr_t)`; it is declared as
    /// [`scsi_callback_t`] because it may hold the two sentinel values.
    pub fn scsi_hba_pkt_alloc(
        hba_dip: dev_info_t,
        ap: *mut scsi_address,
        cmdlen: c_int,
        statuslen: c_int,
        tgtlen: c_int,
        hbalen: c_int,
        callback: scsi_callback_t,
        arg: caddr_t,
    ) -> *mut scsi_pkt;
    /// `scsi_hba_pkt_free(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Frees a packet that [`scsi_hba_pkt_alloc`] allocated for the target
    /// `ap`. It may be called from user, interrupt or kernel context.
    pub fn scsi_hba_pkt_free(ap: *mut scsi_address, pkt: *mut scsi_pkt);
    /// `scsi_hba_pkt_comp(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Signals the completion of `pkt` after the HBA driver has filled in its
    /// completion information. It is the recommended way to complete a
    /// packet and is mandatory for HBA drivers that use
    /// `tran_setup_pkt(9E)`, because it lets the framework observe and react
    /// to the completion. It always results in a call of the packet's
    /// `pkt_comp` routine, which may happen after this function returns and
    /// on another CPU. It may be called from user, interrupt or kernel
    /// context.
    pub fn scsi_hba_pkt_comp(pkt: *mut scsi_pkt);
    /// `scsi_hba_lookup_capstr(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Matches the NUL terminated string `capstr` against the known
    /// capability names and returns the index of the matching capability,
    /// one of the `SCSI_CAP_*` values, or -1 if the string is not a known
    /// capability. Several capabilities also accept a spelling with an
    /// underscore in place of the hyphen. HBA drivers use it in
    /// `tran_getcap(9E)` and `tran_setcap(9E)`. It may be called from user,
    /// interrupt or kernel context.
    pub fn scsi_hba_lookup_capstr(capstr: *mut c_char) -> c_int;

    // --- per device type properties --------------------------------------

    /// `scsi_get_device_type_scsi_options(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Looks up the `device-type-scsi-options-list` property of the HBA node
    /// `hba_dip`, which the HBA's `driver.conf(5)` file may set to pairs of
    /// an inquiry vendor and product string and the name of a property that
    /// holds `scsi-options` for such devices. If the inquiry data of the
    /// target `sd` matches one of the strings, the function returns the
    /// value of the corresponding property; otherwise it returns
    /// `default_scsi_options`. It may be called from kernel or interrupt
    /// context.
    pub fn scsi_get_device_type_scsi_options(
        hba_dip: dev_info_t,
        sd: *mut scsi_device,
        default_scsi_options: c_int,
    ) -> c_int;
    /// `scsi_get_device_type_string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Looks up the string array property `prop_name` of the HBA node
    /// `hba_dip`, which the HBA's `driver.conf(5)` file may set to pairs of an
    /// inquiry vendor and product string and an information string. If the
    /// inquiry data of the target `sd` matches one of the pairs, it returns a
    /// copy of the information string, whose meaning is up to the caller and
    /// which the caller frees with `kmem_free(9F)` using its length plus one.
    /// Otherwise it returns NULL. It may be called from kernel or interrupt
    /// context.
    pub fn scsi_get_device_type_string(
        prop_name: *mut c_char,
        hba_dip: dev_info_t,
        sd: *mut scsi_device,
    ) -> *mut c_char;

    // --- iports ----------------------------------------------------------

    /// `scsi_hba_iport_register(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Creates an iport named `port` as a child of the HBA node `dip`, bound
    /// to the same driver. The name must be unique for the parent and may
    /// be synthetic. It suits a fixed, static set of iports; a driver whose
    /// iports come and go with its phys should use an iport map instead.
    /// Returns `DDI_SUCCESS` or `DDI_FAILURE`. It is generally called from
    /// `attach(9E)` and may be called from user or kernel context.
    pub fn scsi_hba_iport_register(dip: dev_info_t, port: *mut c_char) -> c_int;
    /// `scsi_hba_iport_unit_address(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns the unit address that the iport `dip` was created with,
    /// through [`scsi_hba_iport_register`] or
    /// [`scsi_hba_iportmap_iport_add`], or NULL if `dip` is not an iport. A
    /// driver bound to both the HBA and its iports uses it in `attach(9E)` to
    /// tell them apart. It may be called from user, kernel or interrupt
    /// context.
    pub fn scsi_hba_iport_unit_address(dip: dev_info_t) -> *mut c_char;
    /// `scsi_hba_iport_exist(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns 1 if the node `dip` has any iport children added through
    /// [`scsi_hba_iport_register`] or [`scsi_hba_iportmap_iport_add`], and 0
    /// otherwise. It may be called from user or kernel context.
    pub fn scsi_hba_iport_exist(dip: dev_info_t) -> c_int;
    /// `scsi_hba_iport_find(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns the device node of the iport child of `pdip` whose unit
    /// address is `portnm`, or NULL if there is none. It may be called from
    /// user or kernel context.
    pub fn scsi_hba_iport_find(pdip: dev_info_t, portnm: *mut c_char) -> dev_info_t;

    // --- iport maps ------------------------------------------------------

    /// `scsi_hba_iportmap_create(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Creates an iport map for the HBA node `hba_dip` and stores it in
    /// `iportmapp`. `stable_usec` is how long in microseconds the map waits
    /// for changes to settle before reporting them, and `csync_usec` how
    /// long after creation the initial enumeration must be complete. Returns
    /// `DDI_SUCCESS` or `DDI_FAILURE`. It is generally called from
    /// `attach(9E)`.
    pub fn scsi_hba_iportmap_create(
        hba_dip: dev_info_t,
        csync_usec: c_int,
        stable_usec: c_int,
        iportmapp: *mut *mut scsi_hba_iportmap_t,
    ) -> c_int;
    /// `scsi_hba_iportmap_iport_add(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Adds the iport with the unit address `iport_addr` to `iportmap`,
    /// usually in response to a hotplug event. The address must identify the
    /// iport uniquely; with a phy map it is the unit address the phy map's
    /// callback provided, and otherwise it is usually the iport's world wide
    /// name. Drivers pass NULL as `iport_priv`. Returns `DDI_SUCCESS` or
    /// `DDI_FAILURE`. It should be called from kernel context.
    pub fn scsi_hba_iportmap_iport_add(
        iportmap: *mut scsi_hba_iportmap_t,
        iport_addr: *mut c_char,
        iport_priv: *mut c_void,
    ) -> c_int;
    /// `scsi_hba_iportmap_iport_remove(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Removes from `iportmap` the iport whose unit address `iport_addr` was
    /// passed to [`scsi_hba_iportmap_iport_add`]. Returns `DDI_SUCCESS` or
    /// `DDI_FAILURE`. It should be called from kernel context.
    pub fn scsi_hba_iportmap_iport_remove(iportmap: *mut scsi_hba_iportmap_t, iport_addr: *mut c_char) -> c_int;
    /// `scsi_hba_iportmap_destroy(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Destroys `iportmap`, detaching all its iports from the system through
    /// the driver's `detach(9E)`. It is generally called from `detach(9E)`.
    pub fn scsi_hba_iportmap_destroy(iportmap: *mut scsi_hba_iportmap_t);

    // --- target maps -----------------------------------------------------

    /// `scsi_hba_tgtmap_create(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Creates a target map for the iport `iport_dip` and stores it in
    /// `tgtmapp`; target maps are only supported on iports, not on the HBA
    /// node itself. `rpt_mode` selects full set or per address reporting.
    /// `stable_usec` is how long in microseconds the map waits for changes
    /// to settle before reporting them, and `csync_usec` how long after
    /// creation the initial enumeration must be complete. `tgtmap_priv` is
    /// passed to the optional callbacks `activate_cb` and `deactivate_cb`,
    /// which run when devices are added before a target driver attaches and
    /// when they are removed after it has detached. Returns `DDI_SUCCESS` or
    /// `DDI_FAILURE`. It is generally called from `attach(9E)` of an iport,
    /// and may also be called from user or kernel context.
    pub fn scsi_hba_tgtmap_create(
        iport_dip: dev_info_t,
        rpt_mode: scsi_tgtmap_mode_t,
        csync_usec: c_int,
        stable_usec: c_int,
        tgtmap_priv: *mut c_void,
        activate_cb: Option<scsi_tgt_activate_cb_t>,
        deactivate_cb: Option<scsi_tgt_deactivate_cb_t>,
        tgtmapp: *mut *mut scsi_hba_tgtmap_t,
    ) -> c_int;
    /// `scsi_hba_tgtmap_destroy(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Destroys `tgt_map`, first deactivating every device still present as
    /// though it had been removed. It is generally called from `detach(9E)`,
    /// and may also be called from user or kernel context.
    pub fn scsi_hba_tgtmap_destroy(tgt_map: *mut scsi_hba_tgtmap_t);
    /// `scsi_hba_tgtmap_set_begin(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Begins a new full set report on `tgtmap`. Returns `DDI_SUCCESS` or
    /// `DDI_FAILURE`. It may be called from user or kernel context but not
    /// from interrupt context.
    pub fn scsi_hba_tgtmap_set_begin(tgtmap: *mut scsi_hba_tgtmap_t) -> c_int;
    /// `scsi_hba_tgtmap_set_add(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Adds a device to the full set report in progress on `tgtmap`.
    /// `tgt_addr` is its address, usually a world wide name in unit address
    /// form, which also becomes the unit address of its `scsi_device(9S)`.
    /// `tgt_type` says whether it is a SCSI or an SMP device, and `tgt_priv`
    /// is passed to the callbacks. Returns `DDI_SUCCESS` or `DDI_FAILURE`. It
    /// may be called from user or kernel context but not from interrupt
    /// context.
    pub fn scsi_hba_tgtmap_set_add(
        tgtmap: *mut scsi_hba_tgtmap_t,
        tgt_type: scsi_tgtmap_tgt_type_t,
        tgt_addr: *mut c_char,
        tgt_priv: *mut c_void,
    ) -> c_int;
    /// `scsi_hba_tgtmap_set_end(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Ends the full set report in progress on `tgtmap`, after which the map
    /// works out which devices were added and removed. `flags` is reserved
    /// and must be 0. Returns `DDI_SUCCESS` or `DDI_FAILURE`. It may be called
    /// from user or kernel context but not from interrupt context.
    pub fn scsi_hba_tgtmap_set_end(tgtmap: *mut scsi_hba_tgtmap_t, flags: uint_t) -> c_int;
    /// `scsi_hba_tgtmap_set_flush(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Discards the full set report in progress on `tgtmap`. Returns
    /// `DDI_SUCCESS` or `DDI_FAILURE`. It may be called from user or kernel
    /// context but not from interrupt context.
    pub fn scsi_hba_tgtmap_set_flush(tgtmap: *mut scsi_hba_tgtmap_t) -> c_int;
    /// `scsi_hba_tgtmap_tgt_add(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Reports that the device at `tgt_addr` of kind `tgt_type` was added to
    /// the per address map `tgtmap`, with `tgt_priv` for the callbacks.
    /// Returns `DDI_SUCCESS` or `DDI_FAILURE`. It may be called from user or
    /// kernel context but not from interrupt context.
    pub fn scsi_hba_tgtmap_tgt_add(
        tgtmap: *mut scsi_hba_tgtmap_t,
        tgt_type: scsi_tgtmap_tgt_type_t,
        tgt_addr: *mut c_char,
        tgt_priv: *mut c_void,
    ) -> c_int;
    /// `scsi_hba_tgtmap_tgt_remove(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Reports that the device at `tgt_addr` of kind `tgt_type` was removed
    /// from the per address map `tgtmap`. Both must match the values used
    /// with [`scsi_hba_tgtmap_tgt_add`]. Returns `DDI_SUCCESS` or
    /// `DDI_FAILURE`. It may be called from user or kernel context but not
    /// from interrupt context.
    pub fn scsi_hba_tgtmap_tgt_remove(
        tgtmap: *mut scsi_hba_tgtmap_t,
        tgt_type: scsi_tgtmap_tgt_type_t,
        tgt_addr: *mut c_char,
    ) -> c_int;
    /// `scsi_hba_tgtmap_scan_luns(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Makes the framework send a REPORT LUNS command to the target at
    /// `tgt_addr` in `tgt_map`, creating logical units that have appeared
    /// and removing those that are gone. The target must have been reported
    /// to the map. It may be called from user or kernel context but not from
    /// interrupt context.
    pub fn scsi_hba_tgtmap_scan_luns(tgt_map: *mut scsi_hba_tgtmap_t, tgt_addr: *mut c_char);
}
