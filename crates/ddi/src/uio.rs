//! The user I/O request of `read(9E)` and `write(9E)`.
//!
//! A [`Uio`] is the only way a driver moves data to or from the caller of a
//! read or write. Every transfer goes through `uiomove(9F)`, which handles
//! user and kernel address spaces alike and updates the request as it goes.

use crate::error::Errno;
use crate::sys;

/// A borrowed `uio_t`.
///
/// The direction of the transfer is fixed by the entry point. In `read(9E)`
/// data flows to the caller through [`Uio::copy_out`]; in `write(9E)` it
/// flows from the caller through [`Uio::copy_in`]. Both methods exist on the
/// same type because the underlying `uiomove(9F)` takes the direction as an
/// argument, but a driver only ever needs one of them per entry point.
pub struct Uio<'a> {
    raw: &'a mut sys::uio_t,
}

impl<'a> Uio<'a> {
    /// Wraps a `uio_t` pointer handed in by the kernel.
    ///
    /// # Safety
    /// `p` must be valid and must not be accessed by anyone else for the
    /// lifetime `'a`.
    pub unsafe fn from_raw(p: *mut sys::uio_t) -> Uio<'a> {
        Uio { raw: unsafe { &mut *p } }
    }

    /// Returns the number of bytes the caller still wants transferred.
    pub fn resid(&self) -> usize {
        self.raw.uio_resid.max(0) as usize
    }

    /// Returns the current file offset of the request.
    pub fn offset(&self) -> i64 {
        self.raw.uio_loffset
    }

    /// Returns the open flags of the file, such as `FREAD`, `FWRITE` or
    /// `FNDELAY`.
    pub fn fmode(&self) -> u16 {
        self.raw.uio_fmode
    }

    /// Copies `src` to the caller, as a `read(9E)` does. At most
    /// [`resid`](Uio::resid) bytes are copied; the return value says how
    /// many were.
    pub fn copy_out(&mut self, src: &[u8]) -> Result<usize, Errno> {
        let n = src.len().min(self.resid());
        if n == 0 {
            return Ok(0);
        }
        let r = unsafe { sys::uiomove(src.as_ptr() as *mut sys::c_void, n, sys::uio_rw_t::UIO_READ, self.raw) };
        Errno::check(r).map(|()| n)
    }

    /// Fills `dst` from the caller, as a `write(9E)` does. At most
    /// [`resid`](Uio::resid) bytes are copied; the return value says how
    /// many were.
    pub fn copy_in(&mut self, dst: &mut [u8]) -> Result<usize, Errno> {
        let n = dst.len().min(self.resid());
        if n == 0 {
            return Ok(0);
        }
        let r = unsafe { sys::uiomove(dst.as_mut_ptr() as *mut sys::c_void, n, sys::uio_rw_t::UIO_WRITE, self.raw) };
        Errno::check(r).map(|()| n)
    }

    /// Consumes `n` bytes of the request without transferring them, using
    /// `uioskip(9F)`.
    pub fn skip(&mut self, n: usize) {
        let n = n.min(self.resid());
        unsafe { sys::uioskip(self.raw, n) }
    }

    /// Consumes the whole request without transferring anything, which is
    /// what a null device does with writes. Returns the number of bytes
    /// consumed.
    pub fn discard_all(&mut self) -> usize {
        let n = self.resid();
        self.skip(n);
        n
    }
}
