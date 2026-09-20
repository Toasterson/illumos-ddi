//! Kernel logging without allocation.
//!
//! `cmn_err(9F)` takes a C format string and a variable argument list,
//! neither of which Rust formatting produces. The functions here format the
//! message with `core::fmt` into a fixed 256-byte buffer on the stack and
//! pass that buffer through a single `%s`. Kernel thread stacks are five
//! pages, so the buffer is deliberately small. Nothing here allocates, which
//! makes the macros usable from any context, including the panic handler.

use core::fmt::{self, Write};

use crate::sys;

/// The size of the formatting buffer in bytes, including the terminating NUL.
/// Longer messages are truncated.
pub const MSG_LEN: usize = 256;

/// The severity passed to `cmn_err(9F)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Level {
    /// Continues a previous message. No prefix is added and no newline is
    /// appended (`CE_CONT`).
    Cont = sys::CE_CONT,
    /// A notice, prefixed with `NOTICE:` (`CE_NOTE`).
    Note = sys::CE_NOTE,
    /// A warning, prefixed with `WARNING:` (`CE_WARN`).
    Warn = sys::CE_WARN,
}

/// A fixed-capacity, NUL-terminated buffer that implements `core::fmt::Write`.
pub struct MsgBuf {
    buf: [u8; MSG_LEN],
    len: usize,
}

impl MsgBuf {
    /// Creates an empty buffer.
    pub const fn new() -> MsgBuf {
        MsgBuf { buf: [0; MSG_LEN], len: 0 }
    }

    /// Returns a pointer to the NUL-terminated contents, suitable for passing
    /// to a C `%s`.
    pub fn as_cstr_ptr(&self) -> *const sys::c_char {
        self.buf.as_ptr() as *const sys::c_char
    }

    /// Returns the formatted bytes without the terminating NUL.
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

/// Formats the arguments and logs them through `cmn_err(9F)` at the given
/// level. The [`kwarn!`](crate::kwarn!), [`knote!`](crate::knote!) and [`kcont!`](crate::kcont!) macros are the usual way
/// to call this.
pub fn cmn_err(level: Level, args: fmt::Arguments<'_>) {
    let mut b = MsgBuf::new();
    let _ = b.write_fmt(args);
    unsafe { sys::cmn_err(level as i32, c"%s".as_ptr(), b.as_cstr_ptr()) }
}

/// Formats the arguments and logs them through `dev_err(9F)`, which prefixes
/// the message with the driver name and instance number.
///
/// # Safety
/// `dip` must be a valid `dev_info_t` of an attached instance. Drivers
/// should call [`crate::DevInfo::warn`] or [`crate::DevInfo::note`] instead,
/// which uphold this.
pub unsafe fn dev_err(dip: sys::dev_info_t, level: Level, args: fmt::Arguments<'_>) {
    let mut b = MsgBuf::new();
    let _ = b.write_fmt(args);
    unsafe { sys::dev_err(dip, level as i32, c"%s".as_ptr(), b.as_cstr_ptr()) }
}

/// Logs a warning through `cmn_err(9F)` at `CE_WARN`. Takes the same
/// arguments as `format!`.
#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => { $crate::log::cmn_err($crate::log::Level::Warn, format_args!($($arg)*)) };
}
/// Logs a notice through `cmn_err(9F)` at `CE_NOTE`. Takes the same
/// arguments as `format!`.
#[macro_export]
macro_rules! knote {
    ($($arg:tt)*) => { $crate::log::cmn_err($crate::log::Level::Note, format_args!($($arg)*)) };
}
/// Continues a message through `cmn_err(9F)` at `CE_CONT`. No newline is
/// appended, so include one in the format string when the message is
/// complete.
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
