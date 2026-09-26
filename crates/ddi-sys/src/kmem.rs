//! The kernel memory allocator from `sys/kmem.h`.
//!
//! Three functions cover almost every driver's needs: `kmem_alloc(9F)`,
//! `kmem_zalloc(9F)` and `kmem_free(9F)`. The `KM_*` flags decide whether an
//! allocation may block waiting for memory, and that decision is the single
//! most important thing a caller has to get right. Blocking is only
//! permitted from kernel thread context, never from an interrupt handler.
//!
//! The second half of the module is the object cache interface of
//! `kmem_cache_create(9F)`. A cache hands out objects of one size that keep
//! their constructed state between uses, so that a driver pays for
//! `mutex_init(9F)` and similar setup once per object rather than once per
//! allocation. The cache structure itself is opaque; the callbacks a driver
//! registers are declared as function pointer types here.
//!
//! The header also declares `kmem_alloc_tryhard`, `kmem_rezalloc`,
//! `kmem_cache_stat`, `kmem_cache_reap_soon` and `kmem_cache_reap_active`.
//! None of them has a manual page and none is in the `ddi` mapfile, so they
//! are not bound. The `POINTER_IS_VALID` and `POINTER_INVALIDATE` macros
//! that help a move callback recognise its own objects are provided by the
//! `ddi` crate.

use crate::types::{c_char, c_int, c_void, size_t};
use crate::vmem::vmem_t;

/// The allocation may block until memory is available and is guaranteed to
/// succeed (`KM_SLEEP`). Not permitted in interrupt context.
pub const KM_SLEEP: c_int = 0x0000;
/// The allocation must not block and may return NULL (`KM_NOSLEEP`).
pub const KM_NOSLEEP: c_int = 0x0001;
/// The system panics if the allocation cannot be satisfied (`KM_PANIC`).
pub const KM_PANIC: c_int = 0x0002;
/// The allocation may block and may draw on the reserve pool (`KM_PUSHPAGE`).
pub const KM_PUSHPAGE: c_int = 0x0004;
/// Combined with `KM_NOSLEEP`, lowers the priority of the allocation
/// (`KM_NORMALPRI`).
pub const KM_NORMALPRI: c_int = 0x0008;
/// A non-blocking allocation at normal priority (`KM_NOSLEEP_LAZY`).
pub const KM_NOSLEEP_LAZY: c_int = KM_NOSLEEP | KM_NORMALPRI;

/// The cache should not touch the objects it holds, for example because
/// they are not ordinary memory (`KMC_NOTOUCH`). The manual page requires
/// drivers to pass 0 as `cflags`, and the header describes these flags as
/// private and evolving; they are declared so that the argument can be
/// spelled out, not as an invitation to use them.
pub const KMC_NOTOUCH: c_int = 0x00010000;
/// The cache is excluded from the allocator's debugging features
/// (`KMC_NODEBUG`). Subject to the same caveat as [`KMC_NOTOUCH`].
pub const KMC_NODEBUG: c_int = 0x00020000;
/// The cache has no per-CPU magazine layer (`KMC_NOMAGAZINE`). Subject to
/// the same caveat as [`KMC_NOTOUCH`].
pub const KMC_NOMAGAZINE: c_int = 0x00040000;
/// The cache keeps its buffer control structures inline rather than in a
/// hash table (`KMC_NOHASH`). Subject to the same caveat as [`KMC_NOTOUCH`].
pub const KMC_NOHASH: c_int = 0x00080000;
/// The cache is a quantum cache used by a vmem arena (`KMC_QCACHE`).
/// Subject to the same caveat as [`KMC_NOTOUCH`].
pub const KMC_QCACHE: c_int = 0x00100000;
/// The cache is filled with objects ahead of demand (`KMC_PREFILL`).
/// Subject to the same caveat as [`KMC_NOTOUCH`].
pub const KMC_PREFILL: c_int = 0x00800000;

opaque! {
    /// An object cache created by `kmem_cache_create(9F)`, `kmem_cache_t`.
    /// The header says explicitly that the structure is opaque to clients.
    kmem_cache_t;
}

/// The response of a move callback registered with
/// `kmem_cache_set_move(9F)`, `kmem_cbrc_t`. It tells the allocator which
/// of the two objects it passed to the callback may now be freed.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct kmem_cbrc_t(pub c_int);
impl kmem_cbrc_t {
    /// The client moved the object, so the allocator frees the old one
    /// (`KMEM_CBRC_YES`).
    pub const KMEM_CBRC_YES: Self = Self(0);
    /// The client refused to move the object, so the allocator frees the
    /// unused destination (`KMEM_CBRC_NO`).
    pub const KMEM_CBRC_NO: Self = Self(1);
    /// The client is using the object and cannot move it now, but it is
    /// likely to become movable soon; the allocator frees the unused
    /// destination (`KMEM_CBRC_LATER`).
    pub const KMEM_CBRC_LATER: Self = Self(2);
    /// The client no longer needs the object, so the allocator frees both
    /// the old object and the destination (`KMEM_CBRC_DONT_NEED`).
    pub const KMEM_CBRC_DONT_NEED: Self = Self(3);
    /// The client does not know about the object, either because it has
    /// just been allocated, or because it is about to be or has already been
    /// freed; the allocator frees the destination and evicts the old object
    /// from the magazine layer (`KMEM_CBRC_DONT_KNOW`).
    pub const KMEM_CBRC_DONT_KNOW: Self = Self(4);
}

/// The constructor callback of `kmem_cache_create(9F)`.
///
/// It turns the plain memory at `buf` into a constructed object. `user_arg`
/// is the `private` argument of `kmem_cache_create`, and `kmflags` are the
/// flags of the allocation that triggered construction, which the
/// constructor must forward to any allocation it makes itself. It returns 0
/// on success and -1 if it could not allocate memory under `KM_NOSLEEP`.
pub type kmem_constructor_t = unsafe extern "C" fn(buf: *mut c_void, user_arg: *mut c_void, kmflags: c_int) -> c_int;
/// The destructor callback of `kmem_cache_create(9F)`.
///
/// It reverses the constructor on the object at `buf`. `user_arg` is the
/// `private` argument of `kmem_cache_create`. The allocator may invoke it on
/// any object under its control at any time, so it must be immediately
/// reversible by the constructor.
pub type kmem_destructor_t = unsafe extern "C" fn(buf: *mut c_void, user_arg: *mut c_void);
/// The reclaim callback of `kmem_cache_create(9F)`.
///
/// The allocator calls it with the `private` argument when the system is
/// short of memory. The manual page says drivers should pass NULL.
pub type kmem_reclaim_t = unsafe extern "C" fn(user_arg: *mut c_void);
/// The move callback registered with `kmem_cache_set_move(9F)`.
///
/// The allocator asks the client to move the object at `old` into the
/// freshly constructed object at `new`. `bufsize` is the object size, for
/// callbacks shared between caches, and `user_arg` is the `private`
/// argument of `kmem_cache_create`. The client must not free either object
/// itself; the returned [`kmem_cbrc_t`] tells the allocator what to free.
pub type kmem_move_t =
    unsafe extern "C" fn(old: *mut c_void, new: *mut c_void, bufsize: size_t, user_arg: *mut c_void) -> kmem_cbrc_t;

unsafe extern "C" {
    /// `kmem_alloc(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Allocates `size` bytes of uninitialised kernel memory. The result is
    /// at least 8-byte aligned; the manual page says explicitly that no
    /// larger alignment may be assumed.
    pub fn kmem_alloc(size: size_t, kmflags: c_int) -> *mut c_void;
    /// `kmem_zalloc(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Like [`kmem_alloc`], but the memory is zeroed.
    pub fn kmem_zalloc(size: size_t, kmflags: c_int) -> *mut c_void;
    /// `kmem_free(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Releases memory obtained from [`kmem_alloc`] or [`kmem_zalloc`]. The
    /// `size` argument must be the same value that was passed at allocation;
    /// the allocator does not remember it.
    pub fn kmem_free(buf: *mut c_void, size: size_t);

    /// `kmem_cache_create(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates a cache of objects of `bufsize` bytes each, aligned on an
    /// `align` boundary, or on the allocator's default when `align` is 0.
    /// `name` names the cache in the `kstat(9S)` of class `kmem_cache` that
    /// the function creates; names longer than 31 characters are truncated.
    /// `constructor` and `destructor` convert plain memory into objects and
    /// back, and either may be NULL. `reclaim` and `vmp` should be NULL and
    /// `cflags` should be 0 for drivers. `private` is passed unchanged to
    /// every callback. Returns a pointer to the cache. The function may
    /// block for memory and must not be called from interrupt context; the
    /// manual suggests calling it from `_init(9E)` or `attach(9E)`.
    pub fn kmem_cache_create(
        name: *mut c_char,
        bufsize: size_t,
        align: size_t,
        constructor: Option<kmem_constructor_t>,
        destructor: Option<kmem_destructor_t>,
        reclaim: Option<kmem_reclaim_t>,
        private: *mut c_void,
        vmp: *mut vmem_t,
        cflags: c_int,
    ) -> *mut kmem_cache_t;
    /// `kmem_cache_set_move(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers a callback that the allocator may use to move objects out
    /// of sparsely used pages so that those pages can be reclaimed. It is
    /// called in the same context as [`kmem_cache_create`], immediately
    /// after it and before any object is allocated from the cache. The
    /// callback runs in a single global thread, so at most one move is in
    /// progress at any time.
    pub fn kmem_cache_set_move(cp: *mut kmem_cache_t, r#move: Option<kmem_move_t>);
    /// `kmem_cache_destroy(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Destroys the cache and releases all of its resources. Every object
    /// allocated from it must already have been freed. The destructor may
    /// run on cached objects during this call. Must not be called from
    /// interrupt context.
    pub fn kmem_cache_destroy(cp: *mut kmem_cache_t);
    /// `kmem_cache_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Takes an object from the cache in its constructed state. `kmflag` is
    /// `KM_SLEEP` or `KM_NOSLEEP`, optionally combined with `KM_PUSHPAGE`
    /// for allocations tied to a single I/O operation that is guaranteed to
    /// complete. Returns NULL only for a non-blocking request that could
    /// not be satisfied. May be called from interrupt context only with
    /// `KM_NOSLEEP`. The constructor may run in the caller's context.
    pub fn kmem_cache_alloc(cp: *mut kmem_cache_t, kmflag: c_int) -> *mut c_void;
    /// `kmem_cache_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns an object to the cache. The object must be in its
    /// constructed state, which the allocator cannot verify. May be called
    /// from user, kernel or interrupt context; the destructor may run in the
    /// caller's context.
    pub fn kmem_cache_free(cp: *mut kmem_cache_t, obj: *mut c_void);
    /// `kmem_cache_create(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Tells the allocator that the object `obj`, which the move callback
    /// previously refused to move with `KMEM_CBRC_NO` or `KMEM_CBRC_LATER`,
    /// has become movable. The function is documented on the
    /// `kmem_cache_create(9F)` page as `kmem_cache_move_notify`. Calling it
    /// is optional; the allocator will otherwise retry on its own schedule.
    pub fn kmem_cache_move_notify(cp: *mut kmem_cache_t, obj: *mut c_void);
}
