//! `cmn_err(9F)` without allocation or varargs.
//!
//! Messages are formatted into a fixed 256-byte buffer (kernel stacks are
//! five pages) and passed through a single `%s`, so these macros are safe
//! from any context, including the panic handler.

use core::fmt::{self, Write};

use crate::sys;

/// Size of the formatting buffer, including the NUL.
pub const MSG_LEN: usize = 256;

/// `cmn_err(9F)` level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Level {
    /// `CE_CONT`: no prefix, no automatic newline.
    Cont = sys::CE_CONT,
    /// `CE_NOTE`
    Note = sys::CE_NOTE,
    /// `CE_WARN`
    Warn = sys::CE_WARN,
}

/// Fixed-capacity, NUL-terminated formatting buffer.
pub struct MsgBuf {
    buf: [u8; MSG_LEN],
    len: usize,
}

impl MsgBuf {
    /// Empty buffer.
    pub const fn new() -> MsgBuf {
        MsgBuf { buf: [0; MSG_LEN], len: 0 }
    }

    /// Pointer to the NUL-terminated contents.
    pub fn as_cstr_ptr(&self) -> *const sys::c_char {
        self.buf.as_ptr() as *const sys::c_char
    }

    /// Formatted bytes (without the NUL).
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl Default for MsgBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for MsgBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let room = MSG_LEN - 1 - self.len;
        let n = s.len().min(room);
        self.buf[self.len..self.len + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n;
        self.buf[self.len] = 0;
        // Truncate silently rather than fail: a partial message beats none.
        Ok(())
    }
}

/// Format and emit through `cmn_err(9F)`.
pub fn cmn_err(level: Level, args: fmt::Arguments<'_>) {
    let mut b = MsgBuf::new();
    let _ = b.write_fmt(args);
    unsafe { sys::cmn_err(level as i32, c"%s".as_ptr(), b.as_cstr_ptr()) }
}

/// Format and emit through `dev_err(9F)`, prefixed with driver and instance.
///
/// # Safety
/// `dip` must be a valid, attached `dev_info_t`. Prefer [`crate::DevInfo::warn`].
pub unsafe fn dev_err(dip: sys::dev_info_t, level: Level, args: fmt::Arguments<'_>) {
    let mut b = MsgBuf::new();
    let _ = b.write_fmt(args);
    unsafe { sys::dev_err(dip, level as i32, c"%s".as_ptr(), b.as_cstr_ptr()) }
}

/// `cmn_err(CE_WARN, …)`.
#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => { $crate::log::cmn_err($crate::log::Level::Warn, format_args!($($arg)*)) };
}
/// `cmn_err(CE_NOTE, …)`.
#[macro_export]
macro_rules! knote {
    ($($arg:tt)*) => { $crate::log::cmn_err($crate::log::Level::Note, format_args!($($arg)*)) };
}
/// `cmn_err(CE_CONT, …)`; include your own newline.
#[macro_export]
macro_rules! kcont {
    ($($arg:tt)*) => { $crate::log::cmn_err($crate::log::Level::Cont, format_args!($($arg)*)) };
}

#[cfg(feature = "panic-handler")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo<'_>) -> ! {
    let mut b = MsgBuf::new();
    let _ = write!(b, "rust panic: {info}");
    unsafe { sys::panic(c"%s".as_ptr(), b.as_cstr_ptr()) }
}
