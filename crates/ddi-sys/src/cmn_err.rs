//! Kernel logging from `sys/cmn_err.h`.
//!
//! `cmn_err(9F)` is the kernel's `printf`. Messages go to the console and to
//! the system log, prefixed according to their level. `dev_err(9F)` does the
//! same and adds the driver name and instance number in front. `panic(9F)`
//! stops the system.

use crate::types::*;

/// Continues a previous message without prefix or automatic newline (`CE_CONT`).
pub const CE_CONT: c_int = 0;
/// A notice; the message is prefixed with `NOTICE:` (`CE_NOTE`).
pub const CE_NOTE: c_int = 1;
/// A warning; the message is prefixed with `WARNING:` (`CE_WARN`).
pub const CE_WARN: c_int = 2;
/// The message is printed and the system panics (`CE_PANIC`).
pub const CE_PANIC: c_int = 3;
/// The message is discarded (`CE_IGNORE`).
pub const CE_IGNORE: c_int = 4;

unsafe extern "C" {
    /// `cmn_err(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Formats and logs a message at the given level. A newline is appended
    /// automatically unless the level is `CE_CONT`.
    pub fn cmn_err(level: c_int, fmt: *const c_char, ...);
    /// `dev_err(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`cmn_err`], but the message is prefixed with the name and
    /// instance number of the device `dip` refers to.
    pub fn dev_err(dip: dev_info_t, level: c_int, fmt: *const c_char, ...);
    /// `panic(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Logs the message and halts the system for a crash dump. It does not
    /// return.
    pub fn panic(fmt: *const c_char, ...) -> !;
}
