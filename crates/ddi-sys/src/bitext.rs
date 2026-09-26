//! Bit field extraction and insertion from `sys/bitext.h`.
//!
//! Hardware registers are usually described as a series of bit fields, and
//! the classical way to pick one out is a shift followed by a mask. These
//! functions do exactly that for 8, 16, 32 and 64-bit values, with the
//! field named by an inclusive range of bit positions rather than by a
//! hand-computed mask, so that a register definition in a driver can read
//! like the data sheet. `bitx*` extracts a field, `bitset*` replaces one,
//! and `bitdel64` removes one and closes the gap.
//!
//! Their manual pages mark the interface level as Volatile, meaning the
//! interface is still evolving and neither API nor ABI stability is
//! promised. They are bound anyway because the `ddi` mapfile lists all
//! nine of them, and the first line of each doc comment quotes the level as
//! the page states it. A Rust driver that wants the same operations with
//! no ABI exposure can compute them inline instead.

use crate::types::*;

unsafe extern "C" {
    /// `bitx8(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Extracts the bits of the 8-bit `value` between positions `low` and
    /// `high` inclusive and returns them shifted right so that the field
    /// starts at bit 0. Both positions must lie in the range 0 to 7 and
    /// `high` must not be less than `low`. It can be called from user,
    /// kernel, or interrupt context.
    pub fn bitx8(value: u8, high: uint_t, low: uint_t) -> u8;
    /// `bitx16(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Extracts the bits of the 16-bit `value` between positions `low` and
    /// `high` inclusive and returns them shifted right so that the field
    /// starts at bit 0. Both positions must lie in the range 0 to 15 and
    /// `high` must not be less than `low`. It can be called from user,
    /// kernel, or interrupt context.
    pub fn bitx16(value: u16, high: uint_t, low: uint_t) -> u16;
    /// `bitx32(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Extracts the bits of the 32-bit `value` between positions `low` and
    /// `high` inclusive and returns them shifted right so that the field
    /// starts at bit 0. Both positions must lie in the range 0 to 31 and
    /// `high` must not be less than `low`. It can be called from user,
    /// kernel, or interrupt context.
    pub fn bitx32(value: u32, high: uint_t, low: uint_t) -> u32;
    /// `bitx64(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Extracts the bits of the 64-bit `value` between positions `low` and
    /// `high` inclusive and returns them shifted right so that the field
    /// starts at bit 0. Both positions must lie in the range 0 to 63 and
    /// `high` must not be less than `low`. It can be called from user,
    /// kernel, or interrupt context.
    pub fn bitx64(value: u64, high: uint_t, low: uint_t) -> u64;

    /// `bitset8(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Returns a copy of the 8-bit `base` in which the bits between
    /// positions `low` and `high` inclusive have been cleared and then
    /// replaced by `value` shifted left by `low`. Neither position may
    /// exceed 7, `low` may not be greater than `high`, and `value` must fit
    /// in the width of the field. Neither argument is modified. The manual
    /// gives no CONTEXT section for this family.
    pub fn bitset8(base: u8, high: uint_t, low: uint_t, value: u8) -> u8;
    /// `bitset16(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Returns a copy of the 16-bit `base` in which the bits between
    /// positions `low` and `high` inclusive have been cleared and then
    /// replaced by `value` shifted left by `low`. Neither position may
    /// exceed 15, `low` may not be greater than `high`, and `value` must
    /// fit in the width of the field. Neither argument is modified. The
    /// manual gives no CONTEXT section for this family.
    pub fn bitset16(base: u16, high: uint_t, low: uint_t, value: u16) -> u16;
    /// `bitset32(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Returns a copy of the 32-bit `base` in which the bits between
    /// positions `low` and `high` inclusive have been cleared and then
    /// replaced by `value` shifted left by `low`. Neither position may
    /// exceed 31, `low` may not be greater than `high`, and `value` must
    /// fit in the width of the field. Neither argument is modified. The
    /// manual gives no CONTEXT section for this family.
    pub fn bitset32(base: u32, high: uint_t, low: uint_t, value: u32) -> u32;
    /// `bitset64(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Returns a copy of the 64-bit `base` in which the bits between
    /// positions `low` and `high` inclusive have been cleared and then
    /// replaced by `value` shifted left by `low`. Neither position may
    /// exceed 63, `low` may not be greater than `high`, and `value` must
    /// fit in the width of the field. Neither argument is modified. The
    /// manual gives no CONTEXT section for this family.
    pub fn bitset64(base: u64, high: uint_t, low: uint_t, value: u64) -> u64;

    /// `bitdel64(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Removes the bits between positions `low` and `high` inclusive from
    /// the 64-bit `value` and shifts every higher bit down so that what was
    /// above the field now starts at `low`. The manual's example deletes
    /// bits 4 through 7 of 0x3d3 and obtains 0x33. `low` may not be greater
    /// than `high` nor than 63. It returns the modified integer. The manual
    /// gives no CONTEXT section for this function.
    pub fn bitdel64(value: u64, high: uint_t, low: uint_t) -> u64;
}
