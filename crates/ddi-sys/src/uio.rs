//! `sys/uio.h`: `uio(9S)`, `iovec(9S)`, `uiomove(9F)`.

use crate::types::*;

/// `iovec(9S)`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct iovec {
    /// Base address of the segment.
    pub iov_base: caddr_t,
    /// Length of the segment.
    pub iov_len: size_t,
}

/// `uio_seg_t`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct uio_seg_t(pub c_int);
impl uio_seg_t {
    /// `UIO_USERSPACE`
    pub const UIO_USERSPACE: Self = Self(0);
    /// `UIO_SYSSPACE`
    pub const UIO_SYSSPACE: Self = Self(1);
    /// `UIO_USERISPACE`
    pub const UIO_USERISPACE: Self = Self(2);
}

/// `uio_rw_t`, direction argument of `uiomove(9F)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct uio_rw_t(pub c_int);
impl uio_rw_t {
    /// `UIO_READ`: kernel → user (a `read(9E)`).
    pub const UIO_READ: Self = Self(0);
    /// `UIO_WRITE`: user → kernel (a `write(9E)`).
    pub const UIO_WRITE: Self = Self(1);
}

/// `uio(9S)`. Fields the man page documents are public; the two `lloff_t`
/// unions are exposed as their 64-bit view (`uio_loffset`, `uio_llimit`).
#[repr(C)]
pub struct uio {
    /// Array of I/O vectors.
    pub uio_iov: *mut iovec,
    /// Number of vectors.
    pub uio_iovcnt: c_int,
    /// `uio_loffset`: 64-bit file offset.
    pub uio_loffset: offset_t,
    /// Address space of the vectors.
    pub uio_segflg: uio_seg_t,
    /// File mode flags (`FREAD`, `FWRITE`, …).
    pub uio_fmode: u16,
    /// Extended flags; private.
    pub uio_extflg: u16,
    /// `uio_llimit`: maximum byte offset.
    pub uio_llimit: offset_t,
    /// Residual byte count.
    pub uio_resid: ssize_t,
}
/// `uio_t`
pub type uio_t = uio;

const _: () = assert!(core::mem::size_of::<uio>() == 48);
const _: () = assert!(core::mem::offset_of!(uio, uio_loffset) == 16);
const _: () = assert!(core::mem::offset_of!(uio, uio_resid) == 40);

unsafe extern "C" {
    /// `uiomove(9F)`. Architecture independent level 1 (DDI/DKI).
    pub fn uiomove(addr: *mut c_void, nbytes: size_t, rwflag: uio_rw_t, uiop: *mut uio) -> c_int;
    /// `uiocopy(9F)`. illumos DDI specific (illumos DDI).
    pub fn uiocopy(addr: *mut c_void, nbytes: size_t, rwflag: uio_rw_t, uiop: *mut uio, cbytes: *mut size_t) -> c_int;
    /// `uioskip(9F)`. illumos DDI specific (illumos DDI).
    pub fn uioskip(uiop: *mut uio, nbytes: size_t);
}
