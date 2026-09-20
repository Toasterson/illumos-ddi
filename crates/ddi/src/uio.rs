//! `uio(9S)` as the only path to user memory in `read(9E)`/`write(9E)`.

use crate::error::Errno;
use crate::sys;

/// A borrowed `uio_t`. Direction is fixed by the entry point: in `read(9E)`
/// data flows to the user (`copy_out`), in `write(9E)` from the user
/// (`copy_in`); the framework passes the right wrapper.
pub struct Uio<'a> {
    raw: &'a mut sys::uio_t,
}

impl<'a> Uio<'a> {
    /// Wrap a kernel-provided `uio_t *`.
    ///
    /// # Safety
    /// `p` must be valid and exclusively borrowed for `'a`.
    pub unsafe fn from_raw(p: *mut sys::uio_t) -> Uio<'a> {
        Uio { raw: unsafe { &mut *p } }
    }

    /// Bytes remaining in the request.
    pub fn resid(&self) -> usize {
        self.raw.uio_resid.max(0) as usize
    }

    /// Current file offset.
    pub fn offset(&self) -> i64 {
        self.raw.uio_loffset
    }

    /// Open-mode flags (`FREAD`, `FWRITE`, `FNDELAY`, …) of the file.
    pub fn fmode(&self) -> u16 {
        self.raw.uio_fmode
    }

    /// Move `src` to the caller (`uiomove(9F)` with `UIO_READ`). Copies at
    /// most `resid()` bytes; returns how many.
    pub fn copy_out(&mut self, src: &[u8]) -> Result<usize, Errno> {
        let n = src.len().min(self.resid());
        if n == 0 {
            return Ok(0);
        }
        let r = unsafe { sys::uiomove(src.as_ptr() as *mut sys::c_void, n, sys::uio_rw_t::UIO_READ, self.raw) };
        Errno::check(r).map(|()| n)
    }

    /// Fill `dst` from the caller (`uiomove(9F)` with `UIO_WRITE`). Copies at
    /// most `resid()` bytes; returns how many.
    pub fn copy_in(&mut self, dst: &mut [u8]) -> Result<usize, Errno> {
        let n = dst.len().min(self.resid());
        if n == 0 {
            return Ok(0);
        }
        let r = unsafe { sys::uiomove(dst.as_mut_ptr() as *mut sys::c_void, n, sys::uio_rw_t::UIO_WRITE, self.raw) };
        Errno::check(r).map(|()| n)
    }

    /// Consume `n` bytes without transferring them (`uioskip(9F)`).
    pub fn skip(&mut self, n: usize) {
        let n = n.min(self.resid());
        unsafe { sys::uioskip(self.raw, n) }
    }

    /// Consume everything, as a null device does.
    pub fn discard_all(&mut self) -> usize {
        let n = self.resid();
        self.skip(n);
        n
    }
}
