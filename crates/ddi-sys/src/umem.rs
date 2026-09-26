//! Page-aligned kernel memory and locked user memory, the `ddi_umem_*(9F)`
//! family from `sys/sunddi.h` and `sys/ddidevmap.h`.
//!
//! There are two ways for a driver to obtain memory that user space can
//! reach. `ddi_umem_alloc(9F)` allocates page-aligned kernel memory that the
//! driver may later export with `devmap_umem_setup(9F)`, and
//! `ddi_umem_lock(9F)` pins the physical pages behind a range of the current
//! process's address space so that a device can read or write them directly.
//! Both hand back a `ddi_umem_cookie_t` that names the memory in the calls
//! that follow, including `ddi_umem_iosetup(9F)`, which builds a `buf(9S)`
//! for DMA to or from locked pages.

use crate::bio::buf;
use crate::types::*;

/// The cookie that names memory obtained from `ddi_umem_alloc(9F)` or locked
/// by `ddi_umem_lock(9F)`. The header defines it as a plain `void *`, and its
/// contents are private to the kernel.
pub type ddi_umem_cookie_t = *mut c_void;

/// The allocation may sleep until memory is available and is guaranteed to
/// succeed (`DDI_UMEM_SLEEP`). Not permitted in interrupt context.
pub const DDI_UMEM_SLEEP: c_int = 0x0;
/// The allocation must not sleep and may return NULL (`DDI_UMEM_NOSLEEP`).
pub const DDI_UMEM_NOSLEEP: c_int = 0x01;
/// Allocate pageable memory instead of the locked memory that is the default
/// (`DDI_UMEM_PAGEABLE`). Pageable memory can be swapped out.
pub const DDI_UMEM_PAGEABLE: c_int = 0x02;
/// The locked pages will be read by the device, as in a disk write or a
/// network send (`DDI_UMEMLOCK_READ`).
pub const DDI_UMEMLOCK_READ: c_int = 0x01;
/// The locked pages will be written by the device, as in a disk read or a
/// network receive (`DDI_UMEMLOCK_WRITE`).
pub const DDI_UMEMLOCK_WRITE: c_int = 0x02;

unsafe extern "C" {
    /// `ddi_umem_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates page-aligned kernel memory that can be used in the kernel
    /// and exported to user space. The amount allocated is `size` rounded up
    /// to a multiple of the system page size, and the memory is zero-filled.
    /// The `flag` argument is `DDI_UMEM_SLEEP` or `DDI_UMEM_NOSLEEP`,
    /// optionally combined with `DDI_UMEM_PAGEABLE` to ask for pageable rather
    /// than locked memory. A cookie describing the allocation is stored
    /// through `cookiep` for later use with `devmap_umem_setup(9F)` and
    /// [`ddi_umem_free`]. Returns a pointer to the memory, or NULL when
    /// `DDI_UMEM_NOSLEEP` was given and memory is not available. With
    /// `DDI_UMEM_NOSLEEP` it may be called from any context; with
    /// `DDI_UMEM_SLEEP` only from user or kernel context.
    pub fn ddi_umem_alloc(size: size_t, flag: c_int, cookiep: *mut ddi_umem_cookie_t) -> *mut c_void;
    /// `ddi_umem_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the whole buffer named by a cookie obtained from
    /// [`ddi_umem_alloc`]. May be called from any context.
    pub fn ddi_umem_free(cookie: ddi_umem_cookie_t);
    /// `ddi_umem_lock(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Locks down the physical pages, including I/O pages, behind the current
    /// process's virtual address range from `addr` to `addr + size`, and
    /// stores a cookie representing them through `cookie`. Both the address
    /// and the length must be multiples of the system page size. The `flags`
    /// argument must be exactly one of `DDI_UMEMLOCK_READ` or
    /// `DDI_UMEMLOCK_WRITE`, stating whether the device will read or write
    /// the pages. The cookie can be turned into a `buf(9S)` with
    /// [`ddi_umem_iosetup`] or exported with `devmap_umem_setup(9F)`. While
    /// the pages are locked the process may not `exec(2)` or `fork(2)`, and
    /// the driver must make sure they are unlocked once the application has
    /// closed the device. Returns 0 on success or an errno value: `EFAULT`
    /// when the process has no mapping there, `EACCES` when it lacks
    /// permission, `ENOMEM` when a locked-memory limit would be exceeded,
    /// `EAGAIN` when resources are temporarily short, and `EINVAL` when the
    /// range is not page aligned. May be called from user context only.
    pub fn ddi_umem_lock(addr: caddr_t, size: size_t, flags: c_int, cookie: *mut ddi_umem_cookie_t) -> c_int;
    /// `ddi_umem_unlock(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unlocks the pages named by a cookie obtained from [`ddi_umem_lock`].
    /// May be called from user, kernel or interrupt context.
    pub fn ddi_umem_unlock(cookie: ddi_umem_cookie_t);
    /// `ddi_umem_iosetup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Builds a `buf(9S)` that covers `len` bytes starting `off` bytes into
    /// the locked memory named by `cookie`, ready to be handed to
    /// `ddi_dma_buf_bind_handle(9F)`. Several buffers may be active on the
    /// same cookie at once and may overlap. The `direction` argument is
    /// `B_READ` or `B_WRITE` from the device's point of view, which is the
    /// opposite sense of the `DDI_UMEMLOCK_*` flag the cookie was locked with,
    /// so pages locked with `DDI_UMEMLOCK_READ` are set up with `B_WRITE`.
    /// The `dev` and `blkno` arguments are recorded in the buffer, and
    /// `iodone` names the `biodone(9F)` routine the driver wants called when
    /// the transfer completes. The `sleepflag` argument is `DDI_UMEM_SLEEP`
    /// or `DDI_UMEM_NOSLEEP`. Returns the initialised buffer, or NULL if no
    /// space is available. The buffer is released with `freerbuf(9F)` after
    /// the I/O has completed. With `DDI_UMEM_NOSLEEP` it may be called from
    /// any context; with `DDI_UMEM_SLEEP` only from user or kernel context.
    pub fn ddi_umem_iosetup(
        cookie: ddi_umem_cookie_t,
        off: off_t,
        len: size_t,
        direction: c_int,
        dev: dev_t,
        blkno: daddr_t,
        iodone: Option<unsafe extern "C" fn(bp: *mut buf) -> c_int>,
        sleepflag: c_int,
    ) -> *mut buf;
}
