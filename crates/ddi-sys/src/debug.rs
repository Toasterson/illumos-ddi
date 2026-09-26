//! Assertion support from `sys/debug.h`.
//!
//! In C, `ASSERT(9F)` and its `VERIFY` and `ASSERT3*` relatives are macros
//! that evaluate an expression and, when it is false, call one of the two
//! functions declared here with the expression text, the file name and the
//! line number, so that the kernel can log an `assertion failed` message
//! and panic. `ASSERT` compiles to nothing unless `DEBUG` is defined;
//! `VERIFY` is always live. A Rust driver has no use for either macro,
//! since `assert!`, `debug_assert!` and their `_eq` and `_ne` forms do the
//! same job and end in a Rust panic that the `ddi` crate's panic handler
//! turns into a kernel panic. The two externs exist for completeness and
//! because the `ddi` mapfile lists both of them. Neither is marked as
//! diverging, because the kernel lets a debugger set the `aok` variable to
//! make failed assertions return instead of panicking.

use crate::types::*;

unsafe extern "C" {
    /// `ASSERT(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// The function behind the `ASSERT` and `VERIFY` macros. It logs a
    /// message of the form `assertion failed: <a>, file: <f>, line: <l>`,
    /// where `a` is the text of the failed expression, `f` the source file
    /// and `l` the line number, and then panics the system unless the
    /// kernel variable `aok` has been set to let assertions continue. The
    /// macro that calls it can be used from user, interrupt, or kernel
    /// context.
    pub fn assfail(a: *const c_char, f: *const c_char, l: c_int);
    /// `ASSERT(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// The function behind the three-argument `ASSERT3S`, `ASSERT3U`,
    /// `ASSERT3P`, `ASSERT3B`, `ASSERT0` macros and their `VERIFY`
    /// counterparts, which compare two values with an operator. It logs the
    /// failed expression `a` together with the two operand values `lv` and
    /// `rv`, printed in hexadecimal around the operator text `op`, and the
    /// file `f` and line `l`, and then panics unless `aok` is set. The
    /// operands are passed as `uintmax_t`, which is a 64-bit unsigned
    /// integer under LP64.
    pub fn assfail3(a: *const c_char, lv: u64, op: *const c_char, rv: u64, f: *const c_char, l: c_int);
}
