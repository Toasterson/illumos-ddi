//! Fault management for leaf drivers, from `sys/ddifm.h`.
//!
//! A driver that takes part in the I/O fault management architecture
//! announces its capabilities with `ddi_fm_init(9F)` during `attach(9E)`,
//! and the framework answers with the subset its parent nexus can support.
//! From then on the driver has four jobs. It posts error reports with
//! `ddi_fm_ereport_post(9F)` when it detects a problem itself, it registers
//! an error handler with `ddi_fm_handler_register(9F)` so that the nexus can
//! ask it to inspect the device when a bus error is seen, it consults its
//! access and DMA handles with `ddi_fm_acc_err_get(9F)` and
//! `ddi_fm_dma_err_get(9F)` after transactions that were protected by
//! `DDI_FLAGERR_ACC`, `DDI_CAUTIOUS_ACC` or `DDI_DMA_FLAGERR`, and it tells
//! the framework how badly the service it provides was hit with
//! `ddi_fm_service_impact(9F)`. `ddi_fm_fini(9F)` undoes the initialisation
//! in `detach(9E)`.
//!
//! The one structure that ties this together is `ddi_fm_error(9S)`, which
//! the framework hands to the error handler and fills in from
//! `ddi_fm_acc_err_get`. It carries the error numeric association that
//! links the reports for one error propagation chain, the status the
//! handler is expected to return, a flag saying whether the error was the
//! expected result of a cautious access or a peek or poke, and the access or
//! DMA handle the nexus found to be affected.
//!
//! `fm_ena_generate` comes from `sys/fm/protocol.h` rather than from
//! `sys/ddifm.h`. It has no manual page but is listed in the `ddi` mapfile,
//! and it is what a driver calls to mint a fresh association when it starts
//! an error chain of its own. The `FM_ENA_FMT*` format values it takes, the
//! `FM_VERSION` payload name and `FM_EREPORT_VERS0` value that every report
//! must carry, and the `device.*` class names that `ddi_fm_ereport_post(9F)`
//! reserves for leaf drivers are declared alongside it because the manual
//! pages name them and a driver cannot post a report without them.
//!
//! The service impact values `DDI_SERVICE_LOST` and its siblings are the
//! members of the `ddi_fault_impact_t` enumeration in `sys/sunddi.h`. They
//! are declared here as plain integers because `ddi_fm_service_impact` takes
//! an `int` and because this is the module where a driver reaches for them.
//! Nothing from `sys/ddifm_impl.h` is bound; the `ndi_fm_*` nexus interfaces
//! and the `ddi_system_fmcap` tunable are left out for the same reason.

use crate::types::*;

/// `ddi_fm_error(9S)`. illumos DDI specific (illumos DDI).
///
/// The error status structure, `ddi_fm_error_t`. The framework passes a
/// pointer to one into the driver's error handler callback, where it can be
/// forwarded to `pci_ereport_post(9F)`, and it fills one in for the caller
/// of [`ddi_fm_acc_err_get`] and [`ddi_fm_dma_err_get`]. The manual page
/// documents the first six fields; the two `fme_bus_*` fields are part of
/// the header's definition and are used by nexus drivers to pass bus
/// specific detail down to `pci_ereport_post`.
///
/// The layout under LP64 is derived as follows. Three `int` fields occupy
/// bytes 0 to 11, then four bytes of padding bring `fme_ena` to its eight
/// byte alignment at offset 16. The three pointers follow at 24, 32 and 40,
/// `fme_bus_type` sits at 48, and four bytes of tail padding round the
/// structure up to its eight byte alignment, giving a size of 56.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_fm_error_t {
    /// The version of the structure, either [`DDI_FME_VER0`] or
    /// [`DDI_FME_VER1`]. Drivers always pass [`DDI_FME_VERSION`].
    pub fme_version: c_int,
    /// The status of the error handler callback or of the resource handle,
    /// one of [`DDI_FM_OK`], [`DDI_FM_FATAL`], [`DDI_FM_NONFATAL`] or
    /// [`DDI_FM_UNKNOWN`].
    pub fme_status: c_int,
    /// Whether the error was expected. [`DDI_FM_ERR_EXPECTED`] means it is
    /// the result of an access protected by `DDI_CAUTIOUS_ACC`, in which
    /// case `fme_acc_handle` is valid and the driver should report only
    /// errors unrelated to that access. [`DDI_FM_ERR_PEEK`] and
    /// [`DDI_FM_ERR_POKE`] mean a `ddi_peek(9F)` or `ddi_poke(9F)` caused
    /// it and are handled the same way. [`DDI_FM_ERR_UNEXPECTED`] means the
    /// driver must carry out the full range of error handling.
    pub fme_flag: c_int,
    /// The format 1 error numeric association for the error propagation
    /// chain this error belongs to. A driver posting a report from its
    /// error handler passes this value to [`ddi_fm_ereport_post`] so that
    /// the reports are correlated.
    pub fme_ena: u64,
    /// The access handle the error is associated with, or NULL. Valid when
    /// `fme_flag` is [`DDI_FM_ERR_EXPECTED`] and after `pci_ereport_post`
    /// finds a non-fatal error on a handle owned by this instance.
    pub fme_acc_handle: ddi_acc_handle_t,
    /// The DMA handle the error is associated with, or NULL, filled in the
    /// same circumstances as `fme_acc_handle`.
    pub fme_dma_handle: ddi_dma_handle_t,
    /// Optional bus specific error detail supplied by the nexus driver, or
    /// NULL. Its meaning depends on `fme_bus_type` and is private to the
    /// nexus and `pci_ereport_post`.
    pub fme_bus_specific: *mut c_void,
    /// The kind of bus that produced `fme_bus_specific`, either
    /// [`DDI_FME_BUS_TYPE_DFLT`] or [`DDI_FME_BUS_TYPE_PCI`].
    pub fme_bus_type: c_int,
}

const _: () = assert!(core::mem::size_of::<ddi_fm_error_t>() == 56);
const _: () = assert!(core::mem::align_of::<ddi_fm_error_t>() == 8);
const _: () = assert!(core::mem::offset_of!(ddi_fm_error_t, fme_version) == 0);
const _: () = assert!(core::mem::offset_of!(ddi_fm_error_t, fme_status) == 4);
const _: () = assert!(core::mem::offset_of!(ddi_fm_error_t, fme_flag) == 8);
const _: () = assert!(core::mem::offset_of!(ddi_fm_error_t, fme_ena) == 16);
const _: () = assert!(core::mem::offset_of!(ddi_fm_error_t, fme_acc_handle) == 24);
const _: () = assert!(core::mem::offset_of!(ddi_fm_error_t, fme_dma_handle) == 32);
const _: () = assert!(core::mem::offset_of!(ddi_fm_error_t, fme_bus_specific) == 40);
const _: () = assert!(core::mem::offset_of!(ddi_fm_error_t, fme_bus_type) == 48);

/// The error handler callback type registered with
/// [`ddi_fm_handler_register`], `ddi_err_func_t`. The framework calls it
/// with the device node, the error status structure describing the error
/// under investigation, and the private pointer the driver supplied at
/// registration. The handler inspects the device, posts a report for
/// anything it finds, and returns one of [`DDI_FM_OK`], [`DDI_FM_FATAL`],
/// [`DDI_FM_NONFATAL`] or [`DDI_FM_UNKNOWN`]. It may run in kernel,
/// interrupt or high-level interrupt context, so any lock it takes must have
/// been initialised with the cookie from [`ddi_fm_init`], and such a lock
/// must not be held while registers are accessed with `ddi_get8(9F)` and
/// its relatives. A NULL C function pointer is `None`.
pub type ddi_err_func_t =
    Option<unsafe extern "C" fn(dip: dev_info_t, error_status: *mut ddi_fm_error_t, impl_data: *const c_void) -> c_int>;

/// The error handler or handle status meaning no error affecting this
/// instance was detected (`DDI_FM_OK`).
pub const DDI_FM_OK: c_int = 0;
/// The status meaning an error considered fatal to the operational state of
/// the system was detected (`DDI_FM_FATAL`).
pub const DDI_FM_FATAL: c_int = -1;
/// The status meaning an error was detected that is not considered fatal to
/// the system; the affected handle is normally recorded in the error
/// structure (`DDI_FM_NONFATAL`).
pub const DDI_FM_NONFATAL: c_int = -2;
/// The status meaning an error was detected but its impact could not be
/// determined, which the framework treats as fatal unless another device
/// can evaluate it as non-fatal (`DDI_FM_UNKNOWN`).
pub const DDI_FM_UNKNOWN: c_int = -3;

/// The capability mask of a driver that takes no part in fault management,
/// which is the default for every driver (`DDI_FM_NOT_CAPABLE`).
pub const DDI_FM_NOT_CAPABLE: c_int = 0x00000000;
/// The capability bit saying the driver posts error report events when it
/// detects an error (`DDI_FM_EREPORT_CAPABLE`).
pub const DDI_FM_EREPORT_CAPABLE: c_int = 0x00000001;
/// The capability bit saying the driver checks its access handles for errors
/// after programmed I/O transactions (`DDI_FM_ACCCHK_CAPABLE`).
pub const DDI_FM_ACCCHK_CAPABLE: c_int = 0x00000002;
/// The capability bit saying the driver checks its DMA handles for errors
/// after DMA transactions (`DDI_FM_DMACHK_CAPABLE`).
pub const DDI_FM_DMACHK_CAPABLE: c_int = 0x00000004;
/// The capability bit saying the driver registers an error handler callback
/// (`DDI_FM_ERRCB_CAPABLE`).
pub const DDI_FM_ERRCB_CAPABLE: c_int = 0x00000008;

/// The `fme_flag` value meaning the error was not the result of any
/// deliberately protected operation and the driver must carry out full error
/// handling (`DDI_FM_ERR_UNEXPECTED`).
pub const DDI_FM_ERR_UNEXPECTED: c_int = 0;
/// The `fme_flag` value meaning the error resulted from an access protected
/// by `DDI_CAUTIOUS_ACC` (`DDI_FM_ERR_EXPECTED`).
pub const DDI_FM_ERR_EXPECTED: c_int = 1;
/// The `fme_flag` value meaning the error resulted from a `ddi_poke(9F)`
/// operation (`DDI_FM_ERR_POKE`).
pub const DDI_FM_ERR_POKE: c_int = 2;
/// The `fme_flag` value meaning the error resulted from a `ddi_peek(9F)`
/// operation (`DDI_FM_ERR_PEEK`).
pub const DDI_FM_ERR_PEEK: c_int = 3;

/// The first version of `ddi_fm_error_t`, without the bus specific fields
/// (`DDI_FME_VER0`).
pub const DDI_FME_VER0: c_int = 0;
/// The second version of `ddi_fm_error_t`, which added the bus specific
/// fields (`DDI_FME_VER1`).
pub const DDI_FME_VER1: c_int = 1;
/// The version a driver passes to the `ddi_fm_*_err_get` and
/// `ddi_fm_*_err_clear` functions; it is the current one, `DDI_FME_VER1`
/// (`DDI_FME_VERSION`).
pub const DDI_FME_VERSION: c_int = DDI_FME_VER1;
/// The `fme_bus_type` value meaning no bus specific detail is present
/// (`DDI_FME_BUS_TYPE_DFLT`).
pub const DDI_FME_BUS_TYPE_DFLT: c_int = 0;
/// The `fme_bus_type` value meaning `fme_bus_specific` was filled in by a
/// PCI, PCI-X or PCI Express nexus (`DDI_FME_BUS_TYPE_PCI`).
pub const DDI_FME_BUS_TYPE_PCI: c_int = 1;

/// The service impact meaning the device can no longer provide its service;
/// the device state becomes `DEVI_DEVICE_DOWN` (`DDI_SERVICE_LOST`).
pub const DDI_SERVICE_LOST: c_int = -32;
/// The service impact meaning the device works but at a reduced level, for
/// example after retries or at a lower speed; the device state becomes
/// `DEVI_DEVICE_DEGRADED` (`DDI_SERVICE_DEGRADED`).
pub const DDI_SERVICE_DEGRADED: c_int = -16;
/// The service impact meaning the error did not affect the service the
/// device provides (`DDI_SERVICE_UNAFFECTED`).
pub const DDI_SERVICE_UNAFFECTED: c_int = 0;
/// The service impact meaning service has been restored after an earlier
/// loss or degradation; the down or degraded state is cleared
/// (`DDI_SERVICE_RESTORED`).
pub const DDI_SERVICE_RESTORED: c_int = 16;

/// The error numeric association format that carries no generation, id or
/// time and is used only as a placeholder (`FM_ENA_FMT0`).
pub const FM_ENA_FMT0: c_uchar = 0;
/// The error numeric association format that encodes a generation number,
/// the processor id and a timestamp; this is the format the DDI expects in
/// `fme_ena` and the one a driver asks [`fm_ena_generate`] for
/// (`FM_ENA_FMT1`).
pub const FM_ENA_FMT1: c_uchar = 1;
/// The error numeric association format that encodes a generation number
/// and a timestamp without a processor id (`FM_ENA_FMT2`).
pub const FM_ENA_FMT2: c_uchar = 2;

/// The name of the mandatory version member of an error report payload,
/// from `sys/fm/protocol.h` (`FM_VERSION`).
pub const FM_VERSION: &core::ffi::CStr = c"version";
/// Version zero of the error report payload format, the value a driver
/// stores as a `DATA_TYPE_UINT8` under [`FM_VERSION`] (`FM_EREPORT_VERS0`).
pub const FM_EREPORT_VERS0: u8 = 0;
/// The current error report payload version, which is `FM_EREPORT_VERS0`
/// (`FM_EREPORT_VERSION`).
pub const FM_EREPORT_VERSION: u8 = FM_EREPORT_VERS0;

/// The error report class for a leaf device found in an invalid or
/// inconsistent state, such as corrupted ring indices or a register holding
/// a value it should not, from `sys/fm/io/ddi.h`
/// (`DDI_FM_DEVICE_INVAL_STATE`). The full class string a driver passes is
/// `device.inval_state`.
pub const DDI_FM_DEVICE_INVAL_STATE: &core::ffi::CStr = c"inval_state";
/// The error report class for a leaf device that timed out instead of
/// responding, for example after a reset or an enable
/// (`DDI_FM_DEVICE_NO_RESPONSE`). The full class string is
/// `device.no_response`.
pub const DDI_FM_DEVICE_NO_RESPONSE: &core::ffi::CStr = c"no_response";
/// The error report class for a leaf device whose data transmission has
/// stalled indefinitely (`DDI_FM_DEVICE_STALL`). The full class string is
/// `device.stall`.
pub const DDI_FM_DEVICE_STALL: &core::ffi::CStr = c"stall";
/// The error report class for a leaf device that raised too many
/// consecutive interrupts with no work to do (`DDI_FM_DEVICE_BADINT_LIMIT`).
/// The full class string is `device.badint_limit`.
pub const DDI_FM_DEVICE_BADINT_LIMIT: &core::ffi::CStr = c"badint_limit";
/// The error report class for a leaf device that reported an internal
/// correctable error of its own (`DDI_FM_DEVICE_INTERN_CORR`). The full
/// class string is `device.intern_corr`.
pub const DDI_FM_DEVICE_INTERN_CORR: &core::ffi::CStr = c"intern_corr";
/// The error report class for a leaf device that reported an internal
/// uncorrectable error of its own (`DDI_FM_DEVICE_INTERN_UNCORR`). The full
/// class string is `device.intern_uncorr`.
pub const DDI_FM_DEVICE_INTERN_UNCORR: &core::ffi::CStr = c"intern_uncorr";
/// The prefix shared by every leaf device error report class
/// (`DDI_FM_DEVICE`). The kernel joins it to the values above with a dot.
pub const DDI_FM_DEVICE: &core::ffi::CStr = c"device";

unsafe extern "C" {
    // --- registration ---------------------------------------------------------
    /// `ddi_fm_init(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Declares the fault management capabilities of the device node `dip`
    /// to the I/O fault management framework and allocates the resources
    /// they need. On entry the integer that `fm_capability` points to holds
    /// the bitwise or of the capabilities the driver wants, built from
    /// [`DDI_FM_EREPORT_CAPABLE`], [`DDI_FM_ACCCHK_CAPABLE`],
    /// [`DDI_FM_DMACHK_CAPABLE`] and [`DDI_FM_ERRCB_CAPABLE`], or
    /// [`DDI_FM_NOT_CAPABLE`] for none. Any capability the parent nexus
    /// cannot support has its bit cleared before the function returns, so
    /// the driver must read the value back and behave accordingly. The
    /// framework also creates the `fm-ereport-capable`, `fm-errcb-capable`,
    /// `fm-accchk-capable` and `fm-dmachk-capable` properties on the node,
    /// which `prtconf(8)` displays. A driver that lets the administrator
    /// choose the level through a `driver.conf(5)` property must read that
    /// property before calling this function.
    ///
    /// The interrupt block cookie for the error handler is stored through
    /// `ibcp`. Any mutex the error handler will take must be initialised
    /// with that cookie, because the handler can run at high-level interrupt
    /// priority. The function returns nothing and must be called from kernel
    /// context in `attach(9E)`.
    pub fn ddi_fm_init(dip: dev_info_t, fm_capability: *mut c_int, ibcp: *mut ddi_iblock_cookie_t);
    /// `ddi_fm_fini(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the resources that [`ddi_fm_init`] allocated for the device
    /// node `dip` and removes it from the fault management framework. Any
    /// error handler must have been unregistered first. It must be called
    /// from kernel context in `detach(9E)`.
    pub fn ddi_fm_fini(dip: dev_info_t);
    /// `ddi_fm_capable(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the capability mask currently in effect for the device node
    /// `dip`, which is the value that [`ddi_fm_init`] settled on after
    /// consulting the parent nexus. A driver uses it to decide whether to
    /// check handles or post reports without keeping its own copy. It must
    /// be called from kernel context.
    pub fn ddi_fm_capable(dip: dev_info_t) -> c_int;

    // --- reporting ------------------------------------------------------------
    /// `ddi_fm_ereport_post(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Queues an error report event for delivery to the fault manager
    /// daemon. `ereport_class` is the event class relative to the device;
    /// the classes reserved for any leaf driver are `device.inval_state`,
    /// `device.no_response`, `device.badint_limit`, `device.intern_corr`,
    /// `device.intern_uncorr` and `device.stall`, whose suffixes are
    /// declared as the `DDI_FM_DEVICE_*` constants in this module. `ena` is
    /// the format 1 error numeric association; when the report is posted
    /// from an error handler it is the `fme_ena` of the structure the
    /// handler received, and when the driver starts a new chain it is 0,
    /// in which case the framework generates one. `sflag` is `DDI_SLEEP` or
    /// `DDI_NOSLEEP` and says whether the caller can wait for memory and
    /// event channel resources.
    ///
    /// The variable arguments are name-value tuples describing the payload.
    /// Scalar members are given as a name, a `data_type_t` and a value;
    /// array members as a name, a `data_type_t`, an element count and a
    /// pointer. One tuple is mandatory and gives the payload version, with
    /// the name [`FM_VERSION`], the type `DATA_TYPE_UINT8` and the value
    /// [`FM_EREPORT_VERS0`]. Further tuples are recorded for diagnosis but
    /// not interpreted. The list is terminated by a NULL name.
    ///
    /// The function returns nothing and may be called from user, kernel or
    /// high-level interrupt context.
    pub fn ddi_fm_ereport_post(dip: dev_info_t, ereport_class: *const c_char, ena: u64, sflag: c_int, ...);
    /// `ddi_fm_service_impact(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Tells the framework how an error affected the service that the
    /// device node `dip` provides. `impact` is one of [`DDI_SERVICE_LOST`],
    /// [`DDI_SERVICE_DEGRADED`], [`DDI_SERVICE_UNAFFECTED`] or
    /// [`DDI_SERVICE_RESTORED`]. A loss moves the device to the
    /// `DEVI_DEVICE_DOWN` state, a degradation to `DEVI_DEVICE_DEGRADED`,
    /// and a restoration clears either. A driver that recovers from a handle
    /// error by resetting the device and retrying should still report
    /// `DDI_SERVICE_UNAFFECTED` so that the framework sees the whole story.
    ///
    /// The function returns nothing and may be called from user, kernel or
    /// high-level interrupt context.
    pub fn ddi_fm_service_impact(dip: dev_info_t, impact: c_int);

    // --- error handler --------------------------------------------------------
    /// `ddi_fm_handler_register(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers `error_handler` as the error handling callback for the
    /// device node `dip`, with `impl_data` passed back to it unchanged. The
    /// framework calls the handler when a bus error is detected in the path
    /// to the device. The handler is expected to check the device for
    /// outstanding errors, quiescing I/O if it must, to isolate the device
    /// where it can, and to post a report through [`ddi_fm_ereport_post`]
    /// for each error it finds. A PCI leaf driver can delegate the scan to
    /// `pci_ereport_post(9F)` and return the `fme_status` that it filled in.
    ///
    /// The handler can be called from kernel, interrupt or high-level
    /// interrupt context, so its locks must be initialised with the cookie
    /// from [`ddi_fm_init`] and must not be held across register accesses.
    /// The driver must have obtained [`DDI_FM_ERRCB_CAPABLE`] from
    /// `ddi_fm_init` first. The function returns nothing and must be called
    /// from kernel context in `attach(9E)`.
    pub fn ddi_fm_handler_register(dip: dev_info_t, error_handler: ddi_err_func_t, impl_data: *mut c_void);
    /// `ddi_fm_handler_unregister(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the error handling callback registered for the device node
    /// `dip`. After it returns the framework no longer calls the handler,
    /// so the driver may tear down the state the handler used. It must be
    /// called from kernel context in `detach(9E)`, before [`ddi_fm_fini`].
    pub fn ddi_fm_handler_unregister(dip: dev_info_t);

    // --- handle status --------------------------------------------------------
    /// `ddi_fm_acc_err_get(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Fills the structure that `error_status` points to with the error
    /// state of the access handle `acc_handle`, which came from
    /// `ddi_regs_map_setup(9F)`, `ddi_dma_mem_alloc(9F)` or a similar
    /// function. `version` is the structure version and must be
    /// [`DDI_FME_VERSION`]. If a fault affecting the mapping was captured
    /// during error handling by a bus or nexus driver, `fme_status` is set
    /// to something other than [`DDI_FM_OK`] and the remaining fields
    /// describe the error.
    ///
    /// A fault on an access handle means programmed I/O through it is no
    /// longer trustworthy; reads may return arbitrary values and writes may
    /// be lost. The fault may be transient, in which case the driver can
    /// clear it with [`ddi_fm_acc_err_clear`], reset the device and retry.
    /// If recovery fails the driver reports [`DDI_SERVICE_LOST`] through
    /// [`ddi_fm_service_impact`]; if it succeeds it still reports
    /// [`DDI_SERVICE_UNAFFECTED`]. The function returns nothing and may be
    /// called from user, kernel or high-level interrupt context.
    pub fn ddi_fm_acc_err_get(acc_handle: ddi_acc_handle_t, error_status: *mut ddi_fm_error_t, version: c_int);
    /// `ddi_fm_dma_err_get(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Fills the structure that `error_status` points to with the error
    /// state of the DMA handle `dma_handle`. `version` must be
    /// [`DDI_FME_VERSION`]. A fault on a DMA handle means an error has
    /// affected, or will affect, transfers between the device and the memory
    /// currently bound to the handle, or the memory most recently bound if
    /// it is unbound. The contents of that memory must be treated as
    /// indeterminate. Causes include a failed component in the DMA path and
    /// an invalid access by the device.
    ///
    /// The driver may recover by freeing the bound memory, resetting the
    /// device and retrying, then reporting the outcome through
    /// [`ddi_fm_service_impact`]. The function returns nothing and may be
    /// called from user, kernel or high-level interrupt context.
    pub fn ddi_fm_dma_err_get(dma_handle: ddi_dma_handle_t, error_status: *mut ddi_fm_error_t, version: c_int);
    /// `ddi_fm_acc_err_clear(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Clears the error state recorded on the access handle `acc_handle`,
    /// so that programmed I/O through the handle is once again considered
    /// trustworthy. `version` is the structure version, [`DDI_FME_VERSION`].
    /// A driver calls it as the first step of recovering from a transient
    /// fault reported by [`ddi_fm_acc_err_get`], before it resets the device
    /// and retries. The function returns nothing.
    pub fn ddi_fm_acc_err_clear(acc_handle: ddi_acc_handle_t, version: c_int);
    /// `ddi_fm_dma_err_clear(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Clears the error state recorded on the DMA handle `dma_handle`.
    /// `version` is the structure version, [`DDI_FME_VERSION`]. A driver
    /// calls it while recovering from a fault reported by
    /// [`ddi_fm_dma_err_get`], after it has dealt with the memory that was
    /// bound to the handle. The function returns nothing.
    pub fn ddi_fm_dma_err_clear(dma_handle: ddi_dma_handle_t, version: c_int);

    // --- error numeric association --------------------------------------------
    /// `fm_ena_generate`. No manual page; listed in `uts/common/mapfiles/ddi.mapfile`.
    ///
    /// Generates a fresh error numeric association in the format `format`,
    /// one of [`FM_ENA_FMT0`], [`FM_ENA_FMT1`] or [`FM_ENA_FMT2`], using
    /// `timestamp` as the time component, or the current high resolution
    /// time when `timestamp` is 0. The DDI works in format 1, which folds in
    /// the identifier of the processor the caller runs on and a generation
    /// count, so a driver that starts an error chain of its own outside an
    /// error handler calls `fm_ena_generate(0, FM_ENA_FMT1)` and passes the
    /// result to every [`ddi_fm_ereport_post`] belonging to that chain. It
    /// is declared in `sys/fm/protocol.h` and implemented in
    /// `uts/common/os/fm.c`. The association is returned.
    pub fn fm_ena_generate(timestamp: u64, format: c_uchar) -> u64;
}
