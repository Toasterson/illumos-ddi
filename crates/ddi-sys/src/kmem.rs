//! The kernel memory allocator from `sys/kmem.h`.
//!
//! Three functions cover almost every driver's needs: `kmem_alloc(9F)`,
//! `kmem_zalloc(9F)` and `kmem_free(9F)`. The `KM_*` flags decide whether an
//! allocation may block waiting for memory, and that decision is the single
//! most important thing a caller has to get right. Blocking is only
//! permitted from kernel thread context, never from an interrupt handler.

use crate::types::{c_int, c_void, size_t};

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
}
