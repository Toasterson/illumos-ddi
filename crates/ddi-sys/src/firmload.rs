//! Loading firmware images from the file system, `firmload(9F)`, from
//! `sys/firmload.h`.
//!
//! A driver that needs to push a firmware blob into its device opens the
//! image by module and file name, asks for its size, reads it into a buffer
//! and closes it again. The kernel looks for the file under
//! `/kernel/firmware/<module>/` and the other module directories. The
//! functions live in `genunix`, so no module dependency is needed. They
//! read from the file system and therefore block; they may only be called
//! from kernel context, never from an interrupt handler.

use crate::types::*;

opaque! {
    /// The structure behind a firmware handle. It is private to the kernel.
    firmware_handle;
}

/// A handle to an open firmware image, `firmware_handle_t`.
pub type firmware_handle_t = *mut firmware_handle;

unsafe extern "C" {
    /// `firmware_open(9F)`. Interface stability: Committed.
    ///
    /// Opens the firmware file `fname` that belongs to the module
    /// `drvname`, and stores a handle for it through `fhp`. Returns 0 on
    /// success or an errno, typically `ENOENT` when no such file exists.
    pub fn firmware_open(drvname: *const c_char, fname: *const c_char, fhp: *mut firmware_handle_t) -> c_int;
    /// `firmware_open(9F)`. Interface stability: Committed.
    ///
    /// Closes a handle returned by [`firmware_open`] and releases the file.
    /// Returns 0 on success or an errno.
    pub fn firmware_close(fh: firmware_handle_t) -> c_int;
    /// `firmware_open(9F)`. Interface stability: Committed.
    ///
    /// Returns the size of the open firmware image in bytes.
    pub fn firmware_get_size(fh: firmware_handle_t) -> off_t;
    /// `firmware_open(9F)`. Interface stability: Committed.
    ///
    /// Reads `len` bytes starting at `offset` of the image into `buf`.
    /// Returns 0 when the whole range was read, or an errno.
    pub fn firmware_read(fh: firmware_handle_t, offset: off_t, buf: *mut c_void, len: size_t) -> c_int;
}
