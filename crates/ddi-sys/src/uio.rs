//! The user I/O request structures from `sys/uio.h`.
//!
//! A `read(9E)` or `write(9E)` receives a `uio(9S)` that describes where the
//! caller's data lives, how much of it there is, and at which file offset.
//! `uiomove(9F)` is the one sanctioned way to move bytes between that
//! description and kernel memory; it updates the structure as it goes.

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
    /// `uiocopy(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`uiomove`], but leaves the `uio` unchanged and reports the
    /// number of bytes copied through `cbytes`.
    pub fn uiocopy(addr: *mut c_void, nbytes: size_t, rwflag: uio_rw_t, uiop: *mut uio, cbytes: *mut size_t) -> c_int;
    /// `uioskip(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Advances the `uio` by `nbytes` without transferring anything.
    pub fn uioskip(uiop: *mut uio, nbytes: size_t);
}
