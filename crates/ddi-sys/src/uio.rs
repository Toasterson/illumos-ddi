//! The user I/O request structures from `sys/uio.h`.
//!
//! A `read(9E)` or `write(9E)` receives a `uio(9S)` that describes where the
//! caller's data lives, how much of it there is, and at which file offset.
//! `uiomove(9F)` is the one sanctioned way to move bytes between that
//! description and kernel memory; it updates the structure as it goes.
//! `ureadc(9F)` and `uwritec(9F)` do the same one character at a time, and
//! `uio_copyin(9F)` and `uio_copyout(9F)` copy a plain buffer to or from
//! whichever address space a `uio_seg_t` names.
//!
//! The header also declares `uiodup` and `uio_prefaultpages`, which have no
//! manual page and are not in the `ddi` mapfile, so they are not bound.

use crate::types::*;

/// One contiguous segment of a caller's buffer, `iovec(9S)`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct iovec {
    /// The address where the segment starts.
    pub iov_base: caddr_t,
    /// The length of the segment in bytes.
    pub iov_len: size_t,
}

/// The address space the I/O vectors point into, `uio_seg_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct uio_seg_t(pub c_int);
impl uio_seg_t {
    /// The vectors point into the calling process (`UIO_USERSPACE`).
    pub const UIO_USERSPACE: Self = Self(0);
    /// The vectors point into kernel memory (`UIO_SYSSPACE`).
    pub const UIO_SYSSPACE: Self = Self(1);
    /// The vectors point into user instruction space (`UIO_USERISPACE`).
    /// Rarely relevant to drivers.
    pub const UIO_USERISPACE: Self = Self(2);
}

/// The direction of a transfer, the `rwflag` argument of `uiomove(9F)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct uio_rw_t(pub c_int);
impl uio_rw_t {
    /// Data moves from the kernel to the caller, as in a `read(9E)` (`UIO_READ`).
    pub const UIO_READ: Self = Self(0);
    /// Data moves from the caller to the kernel, as in a `write(9E)` (`UIO_WRITE`).
    pub const UIO_WRITE: Self = Self(1);
}

/// A user I/O request, `uio(9S)`.
///
/// The fields the manual page documents are public. The two `lloff_t`
/// unions in the C definition are exposed through their 64-bit view as
/// `uio_loffset` and `uio_llimit`, which is how drivers use them.
#[repr(C)]
pub struct uio {
    /// The array of I/O vectors that make up the caller's buffer.
    pub uio_iov: *mut iovec,
    /// The number of entries in `uio_iov`.
    pub uio_iovcnt: c_int,
    /// The 64-bit file offset at which the transfer takes place (`uio_loffset`).
    pub uio_loffset: offset_t,
    /// The address space the vectors point into.
    pub uio_segflg: uio_seg_t,
    /// The open flags of the file, such as `FREAD`, `FWRITE` or `FNDELAY`.
    pub uio_fmode: u16,
    /// Extended flags. These are private to the kernel.
    pub uio_extflg: u16,
    /// The maximum byte offset the transfer may reach (`uio_llimit`).
    pub uio_llimit: offset_t,
    /// The number of bytes still to be transferred. `uiomove` decrements it.
    pub uio_resid: ssize_t,
}
/// The typedef name of [`uio`].
pub type uio_t = uio;

const _: () = assert!(core::mem::size_of::<uio>() == 48);
const _: () = assert!(core::mem::offset_of!(uio, uio_loffset) == 16);
const _: () = assert!(core::mem::offset_of!(uio, uio_resid) == 40);

unsafe extern "C" {
    /// `uiomove(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Moves up to `nbytes` between the kernel buffer at `addr` and the
    /// caller's vectors, in the direction given by `rwflag`, and advances the
    /// `uio` accordingly. Returns 0 or an errno, typically `EFAULT`.
    pub fn uiomove(addr: *mut c_void, nbytes: size_t, rwflag: uio_rw_t, uiop: *mut uio) -> c_int;
    /// Declared in `sys/uio.h` and exported by `genunix`. There is no
    /// manual page, and the `ddi` mapfile does not list it.
    ///
    /// Like [`uiomove`], but leaves the `uio` unchanged and reports the
    /// number of bytes copied through `cbytes`. It is bound despite the
    /// missing page because several drivers in the gate rely on it to peek
    /// at a request before committing to it. A manual page and a mapfile
    /// entry are the upstream fix.
    pub fn uiocopy(addr: *mut c_void, nbytes: size_t, rwflag: uio_rw_t, uiop: *mut uio, cbytes: *mut size_t) -> c_int;
    /// Declared in `sys/uio.h` and exported by `genunix`. There is no
    /// manual page, and the `ddi` mapfile does not list it.
    ///
    /// Advances the `uio` by `nbytes` without transferring anything. The
    /// safe crate's `Uio::skip` and `Uio::discard_all` are built on it,
    /// which is the reason it is bound. A manual page and a mapfile entry
    /// are the upstream fix.
    pub fn uioskip(uiop: *mut uio, nbytes: size_t);
    /// `ureadc(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Transfers the single character `c` into the address space described
    /// by the `uio` and updates the structure as [`uiomove`] would. Returns
    /// 0 on success and `EFAULT` on failure. May be called from user,
    /// interrupt or kernel context.
    pub fn ureadc(c: c_int, uio_p: *mut uio) -> c_int;
    /// `uwritec(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Removes the next character from the address space described by the
    /// `uio`, updates the structure as [`uiomove`] would, and returns that
    /// character. Returns -1 if the `uio` is empty or a fault occurred. May
    /// be called from user, interrupt or kernel context.
    pub fn uwritec(uio_p: *mut uio) -> c_int;
    /// `uio_copyin(9F)`. illumos DDI specific.
    ///
    /// Copies `n` bytes from `src`, which lies in the address space named by
    /// `seg`, to the kernel address `dst`. With `UIO_USERSPACE` or
    /// `UIO_USERISPACE` the copy behaves as `copyin(9F)` and may be called
    /// from user context only; with `UIO_SYSSPACE` both addresses are kernel
    /// addresses, the copy is performed with `kcopy(9F)`, and user or kernel
    /// context is acceptable. Any other value of `seg` panics the system.
    /// Returns 0 on success and -1 if a fault occurred.
    pub fn uio_copyin(src: *const c_void, dst: *mut c_void, n: size_t, seg: uio_seg_t) -> c_int;
    /// `uio_copyin(9F)`. illumos DDI specific.
    ///
    /// Copies `n` bytes from the kernel address `src` to `dst`, which lies
    /// in the address space named by `seg`. It is the counterpart of
    /// [`uio_copyin`] and follows the same rules about context and about
    /// invalid values of `seg`. The function is documented on the
    /// `uio_copyin(9F)` page as `uio_copyout`. Returns 0 on success and -1
    /// if a fault occurred.
    pub fn uio_copyout(src: *const c_void, dst: *mut c_void, n: size_t, seg: uio_seg_t) -> c_int;
}
