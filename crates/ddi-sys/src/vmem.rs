//! Virtual memory arenas from `sys/vmem.h`.
//!
//! A vmem arena manages a range of integers. Usually the integers are
//! virtual addresses, but with `VMC_IDENTIFIER` they can be arbitrary
//! identifiers, which is how `id_space(9F)` is implemented. Arenas form a
//! hierarchy in which a child imports spans from its parent through an
//! allocation function and returns them through a free function. Drivers
//! reach for vmem when they need to carve up a private address range, such
//! as a window into device memory, with alignment and boundary constraints
//! that `kmem_alloc(9F)` cannot express.
//!
//! The `vmem_create(9F)`, `vmem_alloc(9F)`, `vmem_add(9F)`,
//! `vmem_contains(9F)` and `vmem_walk(9F)` pages give their interface level
//! as `illumos DDI specific` and carry no separate stability section in the
//! gate used for these bindings; each function below quotes that level
//! verbatim. Callers should nonetheless treat the interface as less settled
//! than the core DDI, since the header notes that several flags are
//! restricted to the kernel itself. Those restricted flags, `VM_MEMLOAD`,
//! `VM_NORELOC`, `VM_ABORT`, `VMC_XALLOC` and `VMC_XALIGN`, and the private
//! segment types `VMEM_SPAN`, `VMEM_ROTOR`, `VMEM_WALKER` and
//! `VMEM_REENTRANT`, are not bound. `vmem_init`, `vmem_update` and
//! `vmem_qcache_reap` have no manual page and are not bound either.
//!
//! In C the three callback typedefs are function types and the prototypes
//! take pointers to them. Here they are function pointer types, and the
//! parameters that may be NULL are wrapped in `Option`.

use crate::types::*;

/// The allocation may block until space is available (`VM_SLEEP`). It has
/// the same value as `KM_SLEEP`.
pub const VM_SLEEP: c_int = 0x00000000;
/// The allocation must not block and may fail (`VM_NOSLEEP`). It has the
/// same value as `KM_NOSLEEP`.
pub const VM_NOSLEEP: c_int = 0x00000001;
/// The system panics if the allocation cannot be satisfied (`VM_PANIC`).
pub const VM_PANIC: c_int = 0x00000002;
/// The allocation may draw on the reserve pool (`VM_PUSHPAGE`).
pub const VM_PUSHPAGE: c_int = 0x00000004;
/// Combined with `VM_NOSLEEP`, lowers the priority of the allocation
/// (`VM_NORMALPRI`).
pub const VM_NORMALPRI: c_int = 0x00000008;
/// The mask of the flags that must match the `KM_*` values (`VM_KMFLAGS`).
pub const VM_KMFLAGS: c_int = 0x000000ff;
/// Take the segment from the smallest free segment that can satisfy the
/// request (`VM_BESTFIT`).
pub const VM_BESTFIT: c_int = 0x00000100;
/// Take the segment from the first free segment found that can satisfy the
/// request (`VM_FIRSTFIT`).
pub const VM_FIRSTFIT: c_int = 0x00000200;
/// Take the segment after the one previously allocated, which gives
/// sequential behaviour useful for identifier arenas (`VM_NEXTFIT`).
pub const VM_NEXTFIT: c_int = 0x00000400;
/// Prefer higher addresses. It has no effect together with `VM_NEXTFIT`
/// (`VM_ENDALLOC`).
pub const VM_ENDALLOC: c_int = 0x00004000;
/// The mask of all per-allocation flags (`VM_FLAGS`).
pub const VM_FLAGS: c_int = 0x0000FFFF;

/// The arena's import function may populate the arena's own metadata
/// (`VMC_POPULATOR`). Used by the kernel's bootstrap arenas.
pub const VMC_POPULATOR: c_int = 0x00010000;
/// The arena must not use quantum caches (`VMC_NO_QCACHE`).
pub const VMC_NO_QCACHE: c_int = 0x00020000;
/// The arena manages arbitrary integer identifiers rather than memory
/// (`VMC_IDENTIFIER`).
pub const VMC_IDENTIFIER: c_int = 0x00040000;
/// The arena may use the alternate memory reserved for crash dumps
/// (`VMC_DUMPSAFE`).
pub const VMC_DUMPSAFE: c_int = 0x00200000;
/// The mask of all arena creation flags (`VMC_FLAGS`).
pub const VMC_FLAGS: c_int = 0xFFFF0000u32 as c_int;

/// The segment type of allocated segments, for `vmem_walk(9F)` and
/// `vmem_size` (`VMEM_ALLOC`).
pub const VMEM_ALLOC: c_int = 0x01;
/// The segment type of free segments, for `vmem_walk(9F)` and `vmem_size`
/// (`VMEM_FREE`).
pub const VMEM_FREE: c_int = 0x02;

opaque! {
    /// A vmem arena, `vmem_t`. The header declares `struct vmem` without a
    /// body, so the layout is private to the kernel.
    vmem_t;
}

/// The import function of `vmem_create(9F)`, `vmem_alloc_t`.
///
/// The arena calls it to obtain `size` bytes from the source arena `vmp`
/// with the flags `vmflag`, and expects a pointer to the imported span or
/// NULL.
pub type vmem_alloc_t = unsafe extern "C" fn(vmp: *mut vmem_t, size: size_t, vmflag: c_int) -> *mut c_void;
/// The release function of `vmem_create(9F)` and `vmem_xcreate(9F)`,
/// `vmem_free_t`.
///
/// The arena calls it to return the span of `size` bytes at `vaddr` to the
/// source arena `vmp`.
pub type vmem_free_t = unsafe extern "C" fn(vmp: *mut vmem_t, vaddr: *mut c_void, size: size_t);
/// The import function of `vmem_xcreate(9F)`, `vmem_ximport_t`.
///
/// It differs from [`vmem_alloc_t`] in that the requested size is passed
/// through a pointer, which the function may increase but never decrease,
/// and in that the desired alignment `align` of the span is supplied.
pub type vmem_ximport_t =
    unsafe extern "C" fn(vmp: *mut vmem_t, size: *mut size_t, align: size_t, vmflag: c_int) -> *mut c_void;
/// The callback of `vmem_walk(9F)`.
///
/// It receives the `arg` given to `vmem_walk`, the base address of a
/// segment and its size.
pub type vmem_walk_fn_t = unsafe extern "C" fn(arg: *mut c_void, vaddr: *mut c_void, size: size_t);

unsafe extern "C" {
    /// `vmem_create(9F)`. illumos DDI specific.
    ///
    /// Creates an arena named `name` covering `size` bytes starting at
    /// `base`, with the allocation granularity `quantum`, which must be a
    /// power of two. `afunc` and `ffunc` import spans from and return them
    /// to the `source` arena; all three may be NULL for an arena that does
    /// not import. The arena caches allocations of integer multiples of
    /// `quantum` up to `qcache_max`, which may be 0. `vmflag` carries the
    /// `VMC_*` creation flags and the `VM_SLEEP` or `VM_NOSLEEP` choice.
    /// Returns the arena or NULL. May be called from user or kernel context.
    pub fn vmem_create(
        name: *const c_char,
        base: *mut c_void,
        size: size_t,
        quantum: size_t,
        afunc: Option<vmem_alloc_t>,
        ffunc: Option<vmem_free_t>,
        source: *mut vmem_t,
        qcache_max: size_t,
        vmflag: c_int,
    ) -> *mut vmem_t;
    /// `vmem_create(9F)`. illumos DDI specific.
    ///
    /// Like [`vmem_create`], but the import function is a
    /// [`vmem_ximport_t`] that may increase the size of the span it imports
    /// and that is told the desired alignment. Documented on the
    /// `vmem_create(9F)` page as `vmem_xcreate`. Returns the arena or NULL.
    /// May be called from user or kernel context.
    pub fn vmem_xcreate(
        name: *const c_char,
        base: *mut c_void,
        size: size_t,
        quantum: size_t,
        afunc: Option<vmem_ximport_t>,
        ffunc: Option<vmem_free_t>,
        source: *mut vmem_t,
        qcache_max: size_t,
        vmflag: c_int,
    ) -> *mut vmem_t;
    /// `vmem_create(9F)`. illumos DDI specific.
    ///
    /// Destroys an arena. Documented on the `vmem_create(9F)` page as
    /// `vmem_destroy`. May be called from user or kernel context.
    pub fn vmem_destroy(vmp: *mut vmem_t);
    /// `vmem_alloc(9F)`. illumos DDI specific.
    ///
    /// Allocates a segment of `size` bytes from the arena. `vmflag` must
    /// contain `VM_SLEEP` or `VM_NOSLEEP` and may name an allocation policy
    /// with `VM_BESTFIT`, `VM_FIRSTFIT` or `VM_NEXTFIT`, optionally combined
    /// with `VM_ENDALLOC`; the default policy is instant fit. Returns the
    /// base of the segment, or NULL for a non-blocking request that could
    /// not be satisfied. For an identifier arena the returned pointer value
    /// is itself the identifier. May be called from interrupt context only
    /// with `VM_NOSLEEP`.
    pub fn vmem_alloc(vmp: *mut vmem_t, size: size_t, vmflag: c_int) -> *mut c_void;
    /// `vmem_alloc(9F)`. illumos DDI specific.
    ///
    /// Allocates a segment of `size` bytes with additional constraints. The
    /// segment is aligned to `align_arg`, or to the arena's quantum when
    /// that is 0, starts `phase` bytes past the alignment boundary, does not
    /// cross a `nocross` boundary, and lies between `minaddr` and `maxaddr`.
    /// Such segments are uncached and must be freed with [`vmem_xfree`].
    /// Documented on the `vmem_alloc(9F)` page as `vmem_xalloc`. Returns
    /// the base of the segment or NULL. May be called from interrupt
    /// context only with `VM_NOSLEEP`.
    pub fn vmem_xalloc(
        vmp: *mut vmem_t,
        size: size_t,
        align_arg: size_t,
        phase: size_t,
        nocross: size_t,
        minaddr: *mut c_void,
        maxaddr: *mut c_void,
        vmflag: c_int,
    ) -> *mut c_void;
    /// `vmem_alloc(9F)`. illumos DDI specific.
    ///
    /// Frees the segment of `size` bytes at `vaddr` that [`vmem_alloc`]
    /// returned. Documented on the `vmem_alloc(9F)` page as `vmem_free`.
    pub fn vmem_free(vmp: *mut vmem_t, vaddr: *mut c_void, size: size_t);
    /// `vmem_alloc(9F)`. illumos DDI specific.
    ///
    /// Frees the segment of `size` bytes at `vaddr` that [`vmem_xalloc`]
    /// returned, bypassing the quantum caches. Documented on the
    /// `vmem_alloc(9F)` page as `vmem_xfree`.
    pub fn vmem_xfree(vmp: *mut vmem_t, vaddr: *mut c_void, size: size_t);
    /// `vmem_add(9F)`. illumos DDI specific.
    ///
    /// Adds the span of `size` bytes at `vaddr` to the arena, making it
    /// available to later allocations. `vmflag` must contain `VM_SLEEP` or
    /// `VM_NOSLEEP`. Returns `vaddr` on success and NULL on failure. May be
    /// called from interrupt context only with `VM_NOSLEEP`.
    pub fn vmem_add(vmp: *mut vmem_t, vaddr: *mut c_void, size: size_t, vmflag: c_int) -> *mut c_void;
    /// `vmem_contains(9F)`. illumos DDI specific.
    ///
    /// Reports whether the segment of `size` bytes at `vaddr` lies within
    /// the arena. The manual page shows `size` as a pointer; the header
    /// declares it as a plain `size_t`, which is what is bound here.
    /// Returns non-zero if the segment exists in the arena and 0 otherwise.
    /// May be called from user or kernel context.
    pub fn vmem_contains(vmp: *mut vmem_t, vaddr: *mut c_void, size: size_t) -> c_int;
    /// `vmem_walk(9F)`. illumos DDI specific.
    ///
    /// Calls `func` with `arg` and the base and size of every segment in the
    /// arena whose type matches `typemask`, a combination of `VMEM_ALLOC`
    /// and `VMEM_FREE`. May be called from user or kernel context.
    pub fn vmem_walk(vmp: *mut vmem_t, typemask: c_int, func: vmem_walk_fn_t, arg: *mut c_void);
    /// `vmem_walk(9F)`. illumos DDI specific.
    ///
    /// Returns the total size of every segment in the arena whose type
    /// matches `typemask`. Documented on the `vmem_walk(9F)` page as
    /// `vmem_size`. May be called from user or kernel context.
    pub fn vmem_size(vmp: *mut vmem_t, typemask: c_int) -> size_t;
}
