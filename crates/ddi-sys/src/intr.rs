//! Interrupts from `sys/ddi_intr.h`, the `ddi_intr_*(9F)` family, together
//! with the generic callback interface `ddi_cb_register(9F)` from
//! `sys/sunddi.h` that interrupt resource management depends on.
//!
//! A driver discovers what kinds of interrupt its device and the host can
//! deliver with [`ddi_intr_get_supported_types`], asks how many of a kind
//! exist and are currently on offer with [`ddi_intr_get_nintrs`] and
//! [`ddi_intr_get_navail`], and then allocates handles with
//! [`ddi_intr_alloc`]. For each handle it reads the priority with
//! [`ddi_intr_get_pri`], initialises the locks its handler will take with
//! that priority through [`DDI_INTR_PRI`], attaches the handler with
//! [`ddi_intr_add_handler`], and finally turns the interrupt on with
//! [`ddi_intr_enable`] or [`ddi_intr_block_enable`]. Teardown runs the same
//! steps in reverse. Soft interrupts, which a driver triggers itself to move
//! work out of a high level handler, use the parallel
//! [`ddi_intr_add_softint`] family and their own handle type.
//!
//! Every function in this module that reports an outcome uses the extended
//! return convention of `sys/sunddi.h`. Success is `DDI_SUCCESS` from
//! [`crate::sunddi`], implementation specific failure is `DDI_FAILURE`, and
//! the more specific negative codes [`DDI_EAGAIN`], [`DDI_EINVAL`],
//! [`DDI_ENOTSUP`], [`DDI_EPENDING`] and [`DDI_EALREADY`] are declared here
//! because the interrupt pages are where the manual documents them. The
//! DMA module refers to the same constants rather than declaring its own.
//! The header does not define a `DDI_INTR_SUCCESS`; the interrupt functions
//! return `DDI_SUCCESS` like everything else.
//!
//! The old interrupt interfaces that predate handles are not bound. The
//! pages for `ddi_add_intr(9F)`, `ddi_add_softintr(9F)`,
//! `ddi_dev_nintrs(9F)`, `ddi_intr_hilevel(9F)`,
//! `ddi_get_iblock_cookie(9F)` and `ddi_get_soft_iblock_cookie(9F)` are all
//! marked obsolete in their INTERFACE LEVEL sections, and the header itself
//! says to use the functions above instead. Only the
//! [`ddi_iblock_cookie_t`] type
//! survives, in the types module, because the lock initialisation functions
//! still accept it.
//!
//! The `ddi_cb_action_t`, `ddi_cb_flags_t` and `ddi_cb_func_t` types are
//! spelled out in `sys/ddi_impldefs.h` rather than in `sys/sunddi.h`, but
//! they are the documented public interface of `ddi_cb_register(9F)` and the
//! prototypes in `sys/sunddi.h` depend on them, so they are declared here
//! with their documented values.

use crate::types::*;

// --- extended return codes (sys/sunddi.h) --------------------------------

/// Not enough interrupt resources were available to satisfy the request
/// (`DDI_EAGAIN`).
pub const DDI_EAGAIN: c_int = -3;
/// The request or one of its arguments was invalid (`DDI_EINVAL`).
pub const DDI_EINVAL: c_int = -4;
/// The device or the platform does not support the operation
/// (`DDI_ENOTSUP`).
pub const DDI_ENOTSUP: c_int = -5;
/// An earlier operation or event is still pending, for example a soft
/// interrupt that has been triggered but not yet serviced (`DDI_EPENDING`).
pub const DDI_EPENDING: c_int = -6;
/// The operation is already in progress or has already been done, for
/// example registering a second callback for the same device
/// (`DDI_EALREADY`).
pub const DDI_EALREADY: c_int = -7;
/// The requested interrupt does not exist, returned by the allocation and
/// enumeration functions when the device has no interrupt of the requested
/// type (`DDI_INTR_NOTFOUND`).
pub const DDI_INTR_NOTFOUND: c_int = 1;

// --- handler return values -----------------------------------------------

/// The value an interrupt handler returns when the interrupt belonged to
/// its device and it serviced it (`DDI_INTR_CLAIMED`).
pub const DDI_INTR_CLAIMED: uint_t = 1;
/// The value an interrupt handler returns when the interrupt was not for
/// its device, so that the framework can offer it to other handlers on a
/// shared line (`DDI_INTR_UNCLAIMED`).
pub const DDI_INTR_UNCLAIMED: uint_t = 0;

// --- interrupt types -----------------------------------------------------

/// A legacy line-based interrupt, or a PCI INTx pin (`DDI_INTR_TYPE_FIXED`).
pub const DDI_INTR_TYPE_FIXED: c_int = 0x1;
/// A PCI message signalled interrupt, of which a device may have up to 32
/// vectors that must be allocated as a power of two (`DDI_INTR_TYPE_MSI`).
pub const DDI_INTR_TYPE_MSI: c_int = 0x2;
/// An extended PCI message signalled interrupt, of which a device may have
/// up to 2048 independently maskable vectors (`DDI_INTR_TYPE_MSIX`).
pub const DDI_INTR_TYPE_MSIX: c_int = 0x4;

// --- priorities ----------------------------------------------------------

/// The lowest priority a hardware interrupt can have (`DDI_INTR_PRI_MIN`).
pub const DDI_INTR_PRI_MIN: uint_t = 1;
/// The highest priority a hardware interrupt can have (`DDI_INTR_PRI_MAX`).
pub const DDI_INTR_PRI_MAX: uint_t = 12;
/// The lowest priority a soft interrupt can be given
/// (`DDI_INTR_SOFTPRI_MIN`).
pub const DDI_INTR_SOFTPRI_MIN: c_int = 1;
/// The highest priority a soft interrupt can be given
/// (`DDI_INTR_SOFTPRI_MAX`).
pub const DDI_INTR_SOFTPRI_MAX: c_int = 9;
/// The soft interrupt priority to use when a driver has no reason to prefer
/// another; it is the lowest one (`DDI_INTR_SOFTPRI_DEFAULT`).
pub const DDI_INTR_SOFTPRI_DEFAULT: c_int = DDI_INTR_SOFTPRI_MIN;

// --- capability flags ----------------------------------------------------

/// The interrupt is level triggered. For fixed interrupts the host may
/// report this together with the edge flag when both are possible, and a
/// driver may select it with [`ddi_intr_set_cap`] (`DDI_INTR_FLAG_LEVEL`).
pub const DDI_INTR_FLAG_LEVEL: c_int = 0x0001;
/// The interrupt is edge triggered. Never reported for MSI or MSI-X. A
/// driver may select it with [`ddi_intr_set_cap`] (`DDI_INTR_FLAG_EDGE`).
pub const DDI_INTR_FLAG_EDGE: c_int = 0x0002;
/// The interrupt can be masked by the device, the host bridge or the host,
/// so [`ddi_intr_set_mask`] and [`ddi_intr_clr_mask`] work. Read only
/// (`DDI_INTR_FLAG_MASKABLE`).
pub const DDI_INTR_FLAG_MASKABLE: c_int = 0x0010;
/// The interrupt has a pending bit that [`ddi_intr_get_pending`] can read.
/// Read only (`DDI_INTR_FLAG_PENDING`).
pub const DDI_INTR_FLAG_PENDING: c_int = 0x0020;
/// All interrupts of this type must be enabled and disabled together with
/// [`ddi_intr_block_enable`] and [`ddi_intr_block_disable`] and cannot be
/// masked one at a time. Read only (`DDI_INTR_FLAG_BLOCK`).
pub const DDI_INTR_FLAG_BLOCK: c_int = 0x0100;

// --- allocation behaviour ------------------------------------------------

/// [`ddi_intr_alloc`] succeeds if it can allocate at least one interrupt
/// and reports how many it got (`DDI_INTR_ALLOC_NORMAL`).
pub const DDI_INTR_ALLOC_NORMAL: c_int = 0;
/// [`ddi_intr_alloc`] succeeds only if it can allocate exactly the number
/// of interrupts asked for (`DDI_INTR_ALLOC_STRICT`).
pub const DDI_INTR_ALLOC_STRICT: c_int = 1;

// --- types ---------------------------------------------------------------

/// The type of a hardware or soft interrupt handler, `ddi_intr_handler_t`.
///
/// The handler receives the two arguments that were registered with it by
/// [`ddi_intr_add_handler`] or, for soft interrupts, the first argument
/// from [`ddi_intr_add_softint`] and the second from
/// [`ddi_intr_trigger_softint`]. It returns [`DDI_INTR_CLAIMED`] if the
/// interrupt was for its device and [`DDI_INTR_UNCLAIMED`] otherwise. The C
/// header declares this as a function type rather than a pointer type, and
/// the prototypes that take it let it decay to a pointer, which is what the
/// Rust `fn` type is.
pub type ddi_intr_handler_t = unsafe extern "C" fn(arg1: caddr_t, arg2: caddr_t) -> uint_t;

opaque! {
    /// The structure behind a callback handle, `struct __ddi_cb`. The
    /// public handle type is a pointer to a pointer to it.
    __ddi_cb;
}

/// A handle for a callback registered with [`ddi_cb_register`],
/// `ddi_cb_handle_t`. `sys/sunddi.h` declares it as `struct __ddi_cb **`.
pub type ddi_cb_handle_t = *mut *mut __ddi_cb;

/// The kind of event a registered callback is being told about,
/// `ddi_cb_action_t`. This is a C enumeration and is passed by value.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_cb_action_t(pub c_int);

/// More interrupts have become available to the driver. The `cbarg`
/// argument of the callback, cast to an integer, is how many. The driver
/// may allocate that many more with [`ddi_intr_alloc`] (`DDI_CB_INTR_ADD`).
pub const DDI_CB_INTR_ADD: ddi_cb_action_t = ddi_cb_action_t(0);
/// Fewer interrupts are available to the driver. The `cbarg` argument of
/// the callback, cast to an integer, is how many it has lost. The driver
/// must release that many with [`ddi_intr_free`] before it returns
/// (`DDI_CB_INTR_REMOVE`).
pub const DDI_CB_INTR_REMOVE: ddi_cb_action_t = ddi_cb_action_t(1);

/// The set of callback classes a driver registers for, `ddi_cb_flags_t`.
/// This is a C enumeration used as a bit mask and is passed by value.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_cb_flags_t(pub c_int);

/// The driver takes part in interrupt resource management and accepts
/// [`DDI_CB_INTR_ADD`] and [`DDI_CB_INTR_REMOVE`] notices at any time after
/// it has attached (`DDI_CB_FLAG_INTR`).
pub const DDI_CB_FLAG_INTR: ddi_cb_flags_t = ddi_cb_flags_t(0x1);

/// The type of a callback registered with [`ddi_cb_register`],
/// `ddi_cb_func_t`.
///
/// The framework calls it with the device node, the action, an argument
/// whose meaning depends on the action, and the two arguments given at
/// registration. For the interrupt actions `cbarg` is an integer count
/// disguised as a pointer. The callback returns `DDI_SUCCESS` when it has
/// handled the action, [`DDI_ENOTSUP`] when it does not know the action, and
/// `DDI_FAILURE` when it failed while handling it.
pub type ddi_cb_func_t = unsafe extern "C" fn(
    dip: dev_info_t,
    action: ddi_cb_action_t,
    cbarg: *mut c_void,
    arg1: *mut c_void,
    arg2: *mut c_void,
) -> c_int;

/// `DDI_INTR_PRI`. illumos DDI specific (illumos DDI).
///
/// Converts an interrupt priority, as returned by [`ddi_intr_get_pri`] or
/// [`ddi_intr_get_softint_pri`], into the untyped pointer that
/// `mutex_init(9F)`, `rw_init(9F)` and `cv_init(9F)` take as their last
/// argument. The C macro is `(void *)((uintptr_t)(pri))`; the value is the
/// priority itself, not the address of anything, which is why the pointer
/// is created without provenance. A lock initialised this way blocks the
/// interrupt while it is held, which is what lets an interrupt handler and
/// the rest of the driver share it.
pub const fn DDI_INTR_PRI(pri: uint_t) -> *mut c_void {
    core::ptr::without_provenance_mut(pri as uintptr_t)
}

unsafe extern "C" {
    // --- discovery ---------------------------------------------------------
    /// `ddi_intr_get_supported_types(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports, as a bit mask stored through `typesp`, which of
    /// [`DDI_INTR_TYPE_FIXED`], [`DDI_INTR_TYPE_MSI`] and
    /// [`DDI_INTR_TYPE_MSIX`] both the device and every bridge between it
    /// and the host support. A driver picks one of the reported types and
    /// uses it for every later call. Returns `DDI_SUCCESS`, [`DDI_EINVAL`]
    /// for bad arguments, or [`DDI_INTR_NOTFOUND`] when the device has no
    /// interrupts at all. May be called from user or kernel context but not
    /// from interrupt context.
    pub fn ddi_intr_get_supported_types(dip: dev_info_t, typesp: *mut c_int) -> c_int;
    /// `ddi_intr_get_nintrs(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores through `nintrsp` how many interrupts of the given `type` the
    /// device itself supports. This is the upper bound on what
    /// [`ddi_intr_alloc`] can ever hand out and on the argument of
    /// [`ddi_intr_set_nreq`]. Returns `DDI_SUCCESS`, [`DDI_EINVAL`],
    /// [`DDI_INTR_NOTFOUND`] when the device supports none of that type
    /// rather than storing zero, or `DDI_FAILURE`.
    /// [`ddi_intr_get_supported_types`] must have been called first. May be
    /// called from user or kernel context but not from interrupt context.
    pub fn ddi_intr_get_nintrs(dip: dev_info_t, r#type: c_int, nintrsp: *mut c_int) -> c_int;
    /// `ddi_intr_get_navail(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores through `navailp` how many interrupts of the given `type` the
    /// system is currently willing to give this device. The host may make
    /// fewer available than the device supports, so this is the number a
    /// driver should pass to [`ddi_intr_alloc`]. Returns the same codes as
    /// [`ddi_intr_get_nintrs`] and has the same calling rules.
    pub fn ddi_intr_get_navail(dip: dev_info_t, r#type: c_int, navailp: *mut c_int) -> c_int;

    // --- allocation --------------------------------------------------------
    /// `ddi_intr_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates `count` interrupts of the given `type`, starting at the
    /// relative vector `inum`, and stores one handle for each in the array
    /// `h_array`, which the caller must have sized for `count` entries. For
    /// fixed interrupts `inum` is the index into the device's interrupt
    /// property. For MSI it is a vector number from 0 to 31 and `count` must
    /// be a power of two; for MSI-X it is a vector number from 0 to 2047. The
    /// number actually allocated is stored through `actualp`. With
    /// [`DDI_INTR_ALLOC_STRICT`] as `behavior` the call fails unless all
    /// `count` interrupts could be allocated, and then reports through
    /// `actualp` how many were available; with [`DDI_INTR_ALLOC_NORMAL`] it
    /// succeeds with whatever it got, as long as that is at least one.
    /// Returns `DDI_SUCCESS`, [`DDI_EAGAIN`] when resources ran short,
    /// [`DDI_EINVAL`] for bad arguments including a `count` above the
    /// supported number, [`DDI_INTR_NOTFOUND`] when the interrupt does not
    /// exist, or `DDI_FAILURE`. Kernel context only, never from an
    /// interrupt handler.
    pub fn ddi_intr_alloc(
        dip: dev_info_t,
        h_array: *mut ddi_intr_handle_t,
        r#type: c_int,
        inum: c_int,
        count: c_int,
        actualp: *mut c_int,
        behavior: c_int,
    ) -> c_int;
    /// `ddi_intr_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the interrupt vector, the system resources and the handle
    /// itself for one interrupt allocated by [`ddi_intr_alloc`] or produced
    /// by [`ddi_intr_dup_handler`]. It must be called once per handle, after
    /// the interrupt has been disabled and, for handles that had a handler
    /// added directly, after [`ddi_intr_remove_handler`]. The handle must
    /// not be used afterwards. Returns `DDI_SUCCESS`, [`DDI_EINVAL`] or
    /// `DDI_FAILURE`. Kernel context only, never from an interrupt handler.
    pub fn ddi_intr_free(h: ddi_intr_handle_t) -> c_int;

    // --- capabilities ------------------------------------------------------
    /// `ddi_intr_get_cap(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores through `flagsp` the capability flags of the interrupt, a
    /// combination of [`DDI_INTR_FLAG_LEVEL`], [`DDI_INTR_FLAG_EDGE`],
    /// [`DDI_INTR_FLAG_MASKABLE`], [`DDI_INTR_FLAG_PENDING`] and
    /// [`DDI_INTR_FLAG_BLOCK`]. A driver consults these before deciding
    /// whether it may mask, poll or block enable the interrupt. Returns
    /// `DDI_SUCCESS`, [`DDI_EINVAL`], [`DDI_ENOTSUP`] or `DDI_FAILURE`. May be
    /// called from user or kernel context but not from interrupt context.
    pub fn ddi_intr_get_cap(h: ddi_intr_handle_t, flagsp: *mut c_int) -> c_int;
    /// `ddi_intr_set_cap(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Selects the trigger mode of a fixed interrupt whose device supports
    /// both, by passing [`DDI_INTR_FLAG_LEVEL`] or [`DDI_INTR_FLAG_EDGE`] in
    /// `flags`. No other flag can be set. The call is only accepted after
    /// [`ddi_intr_alloc`] and before [`ddi_intr_add_handler`], and fails
    /// at any other time. Whether it does anything at all depends on the
    /// device and the platform. Returns `DDI_SUCCESS`, [`DDI_EINVAL`],
    /// [`DDI_ENOTSUP`] or `DDI_FAILURE`. May be called from user or kernel
    /// context but not from interrupt context.
    pub fn ddi_intr_set_cap(h: ddi_intr_handle_t, flags: c_int) -> c_int;

    // --- priorities --------------------------------------------------------
    /// `ddi_intr_get_hilevel_pri(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the lowest priority at which an interrupt counts as high
    /// level, that is, one that the scheduler does not block. A driver
    /// compares the value from [`ddi_intr_get_pri`] against it before
    /// calling [`ddi_intr_add_handler`]. A handler at or above this level
    /// cannot use any service that manipulates thread or process state,
    /// so it must do as little as possible, typically move data into a
    /// buffer protected by a mutex initialised with its priority and then
    /// call [`ddi_intr_trigger_softint`] to have a soft interrupt do the
    /// rest. The header declares the return type as `uint_t` although the
    /// manual page shows `int`. May be called from user or kernel context
    /// but not from interrupt context.
    pub fn ddi_intr_get_hilevel_pri() -> uint_t;
    /// `ddi_intr_get_pri(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores through `prip` the current software priority of the interrupt,
    /// a small integer between [`DDI_INTR_PRI_MIN`] and
    /// [`DDI_INTR_PRI_MAX`]. This value, wrapped by [`DDI_INTR_PRI`], is
    /// what `mutex_init(9F)` and `rw_init(9F)` need for locks that an
    /// interrupt handler will take. It may be called at any time, even
    /// after a handler has been added. Returns `DDI_SUCCESS`, [`DDI_EINVAL`],
    /// [`DDI_ENOTSUP`] or `DDI_FAILURE`. Kernel context only, never from an
    /// interrupt handler.
    pub fn ddi_intr_get_pri(h: ddi_intr_handle_t, prip: *mut uint_t) -> c_int;
    /// `ddi_intr_set_pri(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Changes the software priority of the interrupt to `pri`, which must
    /// lie within the supported range. The call is only accepted while no
    /// handler is attached to the handle, that is, before
    /// [`ddi_intr_add_handler`] or after [`ddi_intr_remove_handler`]; at any
    /// other time it returns `DDI_FAILURE`. It is documented on the
    /// `ddi_intr_get_pri(9F)` page and is absent from the `ddi` mapfile.
    /// Returns `DDI_SUCCESS`, [`DDI_EINVAL`], [`DDI_ENOTSUP`] or
    /// `DDI_FAILURE`. Kernel context only, never from an interrupt handler.
    pub fn ddi_intr_set_pri(h: ddi_intr_handle_t, pri: uint_t) -> c_int;

    // --- handlers ----------------------------------------------------------
    /// `ddi_intr_add_handler(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Associates the handler `inthandler` with the allocated interrupt
    /// handle `h`. When the interrupt fires the framework calls the handler
    /// with `arg1` and `arg2`, in that order, and the handler answers
    /// [`DDI_INTR_CLAIMED`] or [`DDI_INTR_UNCLAIMED`]. The call must come
    /// after [`ddi_intr_alloc`] and before [`ddi_intr_enable`]; the
    /// interrupt does not fire until it has been enabled. Returns
    /// `DDI_SUCCESS`, [`DDI_EINVAL`] or `DDI_FAILURE`. Kernel context only,
    /// never from an interrupt handler.
    pub fn ddi_intr_add_handler(
        h: ddi_intr_handle_t,
        inthandler: ddi_intr_handler_t,
        arg1: *mut c_void,
        arg2: *mut c_void,
    ) -> c_int;
    /// `ddi_intr_remove_handler(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Breaks the association made by [`ddi_intr_add_handler`]. The
    /// interrupt must have been disabled first. Unloadable drivers call it
    /// from `detach(9E)`. If the handle was duplicated with
    /// [`ddi_intr_dup_handler`], every duplicate must have been disabled and
    /// freed before this call, or it fails with `DDI_FAILURE`. Returns
    /// `DDI_SUCCESS`, [`DDI_EINVAL`] or `DDI_FAILURE`. Kernel context only,
    /// never from an interrupt handler.
    pub fn ddi_intr_remove_handler(h: ddi_intr_handle_t) -> c_int;
    /// `ddi_intr_dup_handler(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// For MSI-X only. Makes the unallocated device vector `vector` share
    /// the address, data, handler and handler arguments of the primary
    /// handle `org`, and stores the resulting handle through `dup`. The
    /// primary must already have had its handler added and been enabled,
    /// and the vector must not have been allocated with [`ddi_intr_alloc`].
    /// This lets a driver that was granted fewer vectors than its device has
    /// still direct every device vector somewhere. A duplicate starts out
    /// disabled and supports only [`ddi_intr_enable`], [`ddi_intr_disable`],
    /// [`ddi_intr_set_mask`], [`ddi_intr_clr_mask`],
    /// [`ddi_intr_get_pending`] and [`ddi_intr_free`]; it needs no
    /// [`ddi_intr_remove_handler`]. Returns `DDI_SUCCESS`, [`DDI_EINVAL`]
    /// when the arguments are bad, when `org` is itself a duplicate or when
    /// the device does not support MSI-X, or `DDI_FAILURE`. Kernel context
    /// only, never from an interrupt handler.
    pub fn ddi_intr_dup_handler(org: ddi_intr_handle_t, vector: c_int, dup: *mut ddi_intr_handle_t) -> c_int;

    // --- enable and disable ------------------------------------------------
    /// `ddi_intr_enable(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Turns on the interrupt, after which it can fire and reach the
    /// handler. Enabling clears any mask bit the system or the device held
    /// on it. It must be called after [`ddi_intr_alloc`],
    /// [`ddi_intr_add_handler`] and the initialisation of every lock the
    /// handler takes. Returns `DDI_SUCCESS`, [`DDI_EINVAL`] or
    /// `DDI_FAILURE`. Kernel context only, never from an interrupt handler.
    pub fn ddi_intr_enable(h: ddi_intr_handle_t) -> c_int;
    /// `ddi_intr_disable(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Turns off the interrupt. It must be called before
    /// [`ddi_intr_remove_handler`] and [`ddi_intr_free`]. Returns
    /// `DDI_SUCCESS`, [`DDI_EINVAL`] or `DDI_FAILURE`. Kernel context only,
    /// never from an interrupt handler.
    pub fn ddi_intr_disable(h: ddi_intr_handle_t) -> c_int;
    /// `ddi_intr_block_enable(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Turns on all `count` interrupts in `h_array` at once. It is only
    /// available when [`ddi_intr_get_cap`] reports [`DDI_INTR_FLAG_BLOCK`],
    /// which is the case for MSI when the device lacks per vector masking,
    /// and `count` must be at least one. The same preconditions as
    /// [`ddi_intr_enable`] apply to every handle. Returns `DDI_SUCCESS`,
    /// [`DDI_EINVAL`] or `DDI_FAILURE`. Kernel context only, never from an
    /// interrupt handler.
    pub fn ddi_intr_block_enable(h_array: *mut ddi_intr_handle_t, count: c_int) -> c_int;
    /// `ddi_intr_block_disable(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Turns off all `count` interrupts in `h_array` at once. It is the
    /// counterpart of [`ddi_intr_block_enable`] and should be used whenever
    /// that function was used to enable them, before the handlers are
    /// removed and the handles freed. Returns `DDI_SUCCESS`, [`DDI_EINVAL`]
    /// or `DDI_FAILURE`. Kernel context only, never from an interrupt
    /// handler.
    pub fn ddi_intr_block_disable(h_array: *mut ddi_intr_handle_t, count: c_int) -> c_int;

    // --- masking and pending -----------------------------------------------
    /// `ddi_intr_set_mask(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Masks the interrupt at the device, the host bridge or the host,
    /// whichever supports it, so that new occurrences are held back while
    /// interrupts already in flight may still be delivered. It is only
    /// valid when [`ddi_intr_get_cap`] reports [`DDI_INTR_FLAG_MASKABLE`]
    /// and only while the interrupt is enabled; otherwise [`DDI_EINVAL`]
    /// comes back. Returns `DDI_SUCCESS`, [`DDI_EINVAL`], [`DDI_ENOTSUP`] or
    /// `DDI_FAILURE`. May be called from any context.
    pub fn ddi_intr_set_mask(h: ddi_intr_handle_t) -> c_int;
    /// `ddi_intr_clr_mask(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes a mask set by [`ddi_intr_set_mask`]. It must follow a
    /// matching set call and cannot clear a mask the operating system
    /// itself has temporarily applied. There is no need to call it after
    /// [`ddi_intr_enable`]. Returns `DDI_SUCCESS`, [`DDI_EINVAL`] when the
    /// interrupt is not enabled, [`DDI_ENOTSUP`] or `DDI_FAILURE`. May be
    /// called from any context.
    pub fn ddi_intr_clr_mask(h: ddi_intr_handle_t) -> c_int;
    /// `ddi_intr_get_pending(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a non-zero value through `pendingp` if the interrupt is
    /// pending. It works only when [`ddi_intr_get_cap`] reports
    /// [`DDI_INTR_FLAG_PENDING`]; otherwise it stores zero and returns
    /// [`DDI_ENOTSUP`]. The handle must already be allocated. Returns
    /// `DDI_SUCCESS`, [`DDI_EINVAL`], [`DDI_ENOTSUP`] or `DDI_FAILURE`. May be
    /// called from user or kernel context but not from interrupt context.
    pub fn ddi_intr_get_pending(h: ddi_intr_handle_t, pendingp: *mut c_int) -> c_int;

    // --- interrupt resource management -----------------------------------
    /// `ddi_intr_set_nreq(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Tells the interrupt resource manager that this instance would like
    /// `nreq` interrupts in total, counting the ones it already holds. The
    /// value must be between one and what [`ddi_intr_get_nintrs`] reported.
    /// The framework answers later through the callback registered with
    /// [`ddi_cb_register`], sending [`DDI_CB_INTR_ADD`] when the driver may
    /// allocate more or [`DDI_CB_INTR_REMOVE`] when it must give some back.
    /// The call is refused unless the driver already consumes interrupts
    /// and has a callback registered with [`DDI_CB_FLAG_INTR`]. Returns
    /// `DDI_SUCCESS`, [`DDI_EINVAL`] for an illegal `nreq`, [`DDI_ENOTSUP`]
    /// when there is no callback or the platform has no interrupt pools,
    /// or `DDI_FAILURE`. Kernel context only, never from an interrupt
    /// handler. The manual page marks this interface as Private stability.
    pub fn ddi_intr_set_nreq(dip: dev_info_t, nreq: c_int) -> c_int;

    // --- soft interrupts ---------------------------------------------------
    /// `ddi_intr_add_softint(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers `handler` as a soft interrupt handler at priority
    /// `soft_pri`, which lies between [`DDI_INTR_SOFTPRI_MIN`] and
    /// [`DDI_INTR_SOFTPRI_MAX`] or is [`DDI_INTR_SOFTPRI_DEFAULT`], and
    /// stores the new soft interrupt handle through `h`. The handler is
    /// later called with `arg1` and with whatever second argument
    /// [`ddi_intr_trigger_softint`] supplies. The priority is also the
    /// value to initialise the handler's locks with, and the soft interrupt
    /// cannot be triggered until those locks exist. Because another driver
    /// may trigger a soft interrupt at the same level, a handler must check
    /// a flag of its own before assuming it has work, and return
    /// [`DDI_INTR_UNCLAIMED`] when it does not. Returns `DDI_SUCCESS`,
    /// [`DDI_EAGAIN`], [`DDI_EINVAL`] or `DDI_FAILURE`. May be called from
    /// user or kernel context but not from interrupt context.
    pub fn ddi_intr_add_softint(
        dip: dev_info_t,
        h: *mut ddi_softint_handle_t,
        soft_pri: c_int,
        handler: ddi_intr_handler_t,
        arg1: *mut c_void,
    ) -> c_int;
    /// `ddi_intr_remove_softint(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unregisters a soft interrupt handler. After this returns the soft
    /// interrupt can no longer be triggered, although a trigger already in
    /// progress may still be delivered. A driver must remove all of its
    /// soft interrupts before it can be unloaded, or kernel resources leak.
    /// Returns `DDI_SUCCESS`, [`DDI_EINVAL`] or `DDI_FAILURE`. May be called
    /// from user or kernel context but not from interrupt context.
    pub fn ddi_intr_remove_softint(h: ddi_softint_handle_t) -> c_int;
    /// `ddi_intr_trigger_softint(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Schedules the soft interrupt handler to run, passing it `arg2` as its
    /// second argument. Only one trigger can be outstanding; while one is
    /// pending, further triggers and their `arg2` are dropped and the call
    /// returns [`DDI_EPENDING`]. Before triggering, the driver should have
    /// recorded in its own state that there is work to do, since the
    /// handler cannot otherwise tell whether the trigger was its own.
    /// Returns `DDI_SUCCESS`, [`DDI_EINVAL`], [`DDI_EPENDING`] or
    /// `DDI_FAILURE`. Despite the CONTEXT section of the page, its purpose
    /// is to be called from a high level interrupt handler.
    pub fn ddi_intr_trigger_softint(h: ddi_softint_handle_t, arg2: *mut c_void) -> c_int;
    /// `ddi_intr_get_softint_pri(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores through `soft_prip` the priority the soft interrupt was
    /// registered with, for use with [`DDI_INTR_PRI`] when initialising
    /// locks. Returns `DDI_SUCCESS`, [`DDI_EINVAL`] or `DDI_FAILURE`. May be
    /// called from user or kernel context but not from interrupt context.
    pub fn ddi_intr_get_softint_pri(h: ddi_softint_handle_t, soft_prip: *mut uint_t) -> c_int;
    /// `ddi_intr_set_softint_pri(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Changes the priority of an existing soft interrupt to `soft_pri`,
    /// which must lie within the soft priority range. Locks initialised
    /// with the old priority are not adjusted. Returns `DDI_SUCCESS`,
    /// [`DDI_EINVAL`] or `DDI_FAILURE`. May be called from user or kernel
    /// context but not from interrupt context.
    pub fn ddi_intr_set_softint_pri(h: ddi_softint_handle_t, soft_pri: uint_t) -> c_int;

    // --- generic callbacks (sys/sunddi.h) ----------------------------------
    /// `ddi_cb_register(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Installs `cbfunc` as the callback that the framework invokes when an
    /// event of one of the classes in `flags` concerns the device. At
    /// present the only class is [`DDI_CB_FLAG_INTR`], interrupt resource
    /// management, whose events are [`DDI_CB_INTR_ADD`] and
    /// [`DDI_CB_INTR_REMOVE`]. The callback receives `arg1` and `arg2`
    /// unchanged. A handle for the registration is stored through
    /// `ret_hdlp`. Only one callback may be registered per device; a
    /// second attempt fails with [`DDI_EALREADY`]. A driver that registers
    /// for interrupt events need not allocate every interrupt it is
    /// offered, but it must never hold more than are currently available.
    /// Returns `DDI_SUCCESS`, [`DDI_EINVAL`] or [`DDI_EALREADY`]. Kernel
    /// context only, never from an interrupt handler. The manual page
    /// marks this interface as Private stability.
    pub fn ddi_cb_register(
        dip: dev_info_t,
        flags: ddi_cb_flags_t,
        cbfunc: ddi_cb_func_t,
        arg1: *mut c_void,
        arg2: *mut c_void,
        ret_hdlp: *mut ddi_cb_handle_t,
    ) -> c_int;
    /// `ddi_cb_unregister(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes a callback installed by [`ddi_cb_register`] so that no
    /// further events are delivered. A driver does this in `detach(9E)`,
    /// after it has freed its interrupts. Returns `DDI_SUCCESS` or
    /// [`DDI_EINVAL`] for an invalid handle. Kernel context only, never
    /// from an interrupt handler.
    pub fn ddi_cb_unregister(hdl: ddi_cb_handle_t) -> c_int;
}
