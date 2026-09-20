//! `sys/kmem.h`: kernel memory allocator. `kmem_alloc(9F)`, `kmem_zalloc(9F)`,
//! `kmem_free(9F)`.

use crate::types::{c_int, c_void, size_t};

/// `KM_SLEEP`: may block; success guaranteed. Not from interrupt context.
pub const KM_SLEEP: c_int = 0x0000;
/// `KM_NOSLEEP`: may not block; may return NULL.
pub const KM_NOSLEEP: c_int = 0x0001;
/// `KM_PANIC`: panic on failure.
pub const KM_PANIC: c_int = 0x0002;
/// `KM_PUSHPAGE`: may block; may use reserve.
pub const KM_PUSHPAGE: c_int = 0x0004;
/// `KM_NORMALPRI`: with `KM_NOSLEEP`, lower priority allocation.
pub const KM_NORMALPRI: c_int = 0x0008;
/// `KM_NOSLEEP_LAZY`
pub const KM_NOSLEEP_LAZY: c_int = KM_NOSLEEP | KM_NORMALPRI;

unsafe extern "C" {
    /// `kmem_alloc(9F)`. Architecture independent level 1 (DDI/DKI).
    /// Memory is at least 8-byte aligned; no larger alignment may be assumed.
    pub fn kmem_alloc(size: size_t, kmflags: c_int) -> *mut c_void;
    /// `kmem_zalloc(9F)`. Architecture independent level 1 (DDI/DKI).
    pub fn kmem_zalloc(size: size_t, kmflags: c_int) -> *mut c_void;
    /// `kmem_free(9F)`. Architecture independent level 1 (DDI/DKI).
    /// `size` must equal the size passed at allocation.
    pub fn kmem_free(buf: *mut c_void, size: size_t);
}
