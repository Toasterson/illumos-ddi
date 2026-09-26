//! Kernel logging from `sys/cmn_err.h`.
//!
//! `cmn_err(9F)` is the kernel's `printf`. Messages go to the console and to
//! the system log, prefixed according to their level. `dev_err(9F)` does the
//! same and adds the driver name and instance number in front. `zcmn_err`
//! directs the log copy of a message to a particular zone. Each of the
//! three has a `v` variant that takes a C `va_list` instead of a variable
//! argument list, for callers that are themselves variadic and forward
//! their arguments. `panic` stops the system.

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

/// The C `va_list` as it appears in a function parameter.
///
/// On amd64 the compiler's `__builtin_va_list`, which `sys/va_list.h`
/// names `__va_list`, is an array of one structure. An array type in a
/// parameter position decays to a pointer to its first element, so at the
/// ABI level every function below receives a pointer, and a raw pointer is
/// the correct FFI spelling of the parameter. The pointee is left as
/// `c_void` because Rust code has no reason to look inside. A Rust caller
/// cannot construct a value of this type on stable Rust anyway, since the
/// language offers no way to build a C variable argument list; the only
/// realistic source of one is a C function that received it and passes it
/// on.
pub type va_list = *mut c_void;

unsafe extern "C" {
    /// `cmn_err(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Formats and logs a message at the given level. A newline is appended
    /// automatically unless the level is `CE_CONT`.
    pub fn cmn_err(level: c_int, fmt: *const c_char, ...);
    /// `dev_err(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Like [`cmn_err`], but the message is prefixed with the name and
    /// instance number of the device `dip` refers to.
    pub fn dev_err(dip: dev_info_t, level: c_int, fmt: *const c_char, ...);
    /// `cmn_err(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Identical to [`cmn_err`] except that its last argument, `ap`, is a
    /// pointer to a variable argument list rather than the arguments
    /// themselves. `ap` holds the arguments consumed by the conversion
    /// specifications in `fmt`. The manual requires that it be initialised
    /// with `va_start(9F)` and released with `va_end(9F)` after each
    /// traversal; several traversals, each bracketed that way, are
    /// permitted. Like the rest of the family it can be called from user,
    /// kernel, interrupt, or high-level interrupt context.
    pub fn vcmn_err(level: c_int, fmt: *const c_char, ap: va_list);
    /// `cmn_err(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// The combination of [`vcmn_err`] and [`dev_err`]. The `dip`, `level`
    /// and `fmt` arguments are treated as `dev_err` treats them, so the
    /// message is prefixed with the driver name and instance number taken
    /// from the device node, while the trailing `ap` is handled the way
    /// `vcmn_err` handles it. The header declares this one with a `const`
    /// format string, unlike `dev_err`.
    pub fn vdev_err(dip: dev_info_t, level: c_int, fmt: *const c_char, ap: va_list);
    /// `cmn_err(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Identical to [`cmn_err`] apart from its first argument. `zoneid` is
    /// the numeric identifier of the zone whose system log should receive
    /// the message; it is a `zoneid_t`, which `sys/types.h` defines as an
    /// `int`. The zone only matters when the message goes to the log, in
    /// which case the `syslogd(8)` running in that zone processes it instead
    /// of the one in the global zone. The console output is unaffected. A
    /// driver can obtain the zone of a caller from its credentials with
    /// `crgetzoneid(9F)`.
    pub fn zcmn_err(zoneid: c_int, level: c_int, fmt: *const c_char, ...);
    /// `cmn_err(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Identical to [`vcmn_err`] apart from its first argument, which names
    /// the zone whose log receives the message exactly as it does for
    /// [`zcmn_err`]. The `ap` argument follows the same `va_start(9F)` and
    /// `va_end(9F)` rules as `vcmn_err`.
    pub fn vzcmn_err(zoneid: c_int, level: c_int, fmt: *const c_char, ap: va_list);
    /// `panic`. Listed in the `ddi` mapfile; it has no manual page of its
    /// own, and `cmn_err(9F)` describes the same action under `CE_PANIC`.
    ///
    /// Logs the message and halts the system for a crash dump. It does not
    /// return.
    pub fn panic(fmt: *const c_char, ...) -> !;
}
