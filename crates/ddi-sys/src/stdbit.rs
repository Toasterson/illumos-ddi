//! Bit utilities from `sys/stdbit.h`.
//!
//! C23 added a standard header of bit manipulation functions, and illumos
//! provides the same set to kernel modules, with `stdc_` prefixes, under
//! the fourteen `stdc_*(9F)` manual pages. Each operation exists in five
//! typed forms, suffixed `_uc`, `_us`, `_ui`, `_ul` and `_ull` for
//! `unsigned char`, `unsigned short`, `unsigned int`, `unsigned long` and
//! `unsigned long long`, and every one of the seventy is an exported
//! function that the `ddi` mapfile lists. The unsuffixed generic names
//! such as `stdc_count_ones` are `_Generic` macros in the header that
//! dispatch on the argument type; they are not symbols and are not
//! declared here. A Rust driver normally reaches for the inherent methods
//! on the integer types, such as `count_ones`, `leading_zeros`,
//! `next_power_of_two` and `is_power_of_two`, which compile to the same
//! instructions; these declarations exist for the sake of completeness and
//! of the mapfile.
//!
//! The manual pages for this family have no INTERFACE LEVEL section. In
//! its place they carry an INTERFACE STABILITY section reading Committed,
//! and that is what the first line of each doc comment quotes. Under LP64
//! `unsigned long` and `unsigned long long` are both 64 bits wide, so the
//! `_ul` and `_ull` forms differ only in their C type.

use crate::types::*;

unsafe extern "C" {
    /// `stdc_leading_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned char` `value`, starting
    /// at the most significant bit, and returns the count as an `unsigned int`.
    /// A value of zero yields the full bit width, 8. The manual notes that
    /// promoting an integer to a wider type adds leading zeros and so raises
    /// the result. It can be called from user, kernel, or interrupt context and
    /// cannot fail.
    pub fn stdc_leading_zeros_uc(value: c_uchar) -> c_uint;
    /// `stdc_leading_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned short` `value`,
    /// starting at the most significant bit, and returns the count as an
    /// `unsigned int`. A value of zero yields the full bit width, 16. The
    /// manual notes that promoting an integer to a wider type adds leading
    /// zeros and so raises the result. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_leading_zeros_us(value: c_ushort) -> c_uint;
    /// `stdc_leading_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned int` `value`, starting
    /// at the most significant bit, and returns the count as an `unsigned int`.
    /// A value of zero yields the full bit width, 32. The manual notes that
    /// promoting an integer to a wider type adds leading zeros and so raises
    /// the result. It can be called from user, kernel, or interrupt context and
    /// cannot fail.
    pub fn stdc_leading_zeros_ui(value: c_uint) -> c_uint;
    /// `stdc_leading_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned long` `value`, starting
    /// at the most significant bit, and returns the count as an `unsigned int`.
    /// A value of zero yields the full bit width, 64. The manual notes that
    /// promoting an integer to a wider type adds leading zeros and so raises
    /// the result. It can be called from user, kernel, or interrupt context and
    /// cannot fail.
    pub fn stdc_leading_zeros_ul(value: c_ulong) -> c_uint;
    /// `stdc_leading_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned long long` `value`,
    /// starting at the most significant bit, and returns the count as an
    /// `unsigned int`. A value of zero yields the full bit width, 64. The
    /// manual notes that promoting an integer to a wider type adds leading
    /// zeros and so raises the result. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_leading_zeros_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_leading_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned char` `value`, starting
    /// at the most significant bit, and returns the count as an `unsigned int`.
    /// A value with its top bit clear yields 0. It can be called from user,
    /// kernel, or interrupt context and cannot fail.
    pub fn stdc_leading_ones_uc(value: c_uchar) -> c_uint;
    /// `stdc_leading_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned short` `value`, starting
    /// at the most significant bit, and returns the count as an `unsigned int`.
    /// A value with its top bit clear yields 0. It can be called from user,
    /// kernel, or interrupt context and cannot fail.
    pub fn stdc_leading_ones_us(value: c_ushort) -> c_uint;
    /// `stdc_leading_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned int` `value`, starting
    /// at the most significant bit, and returns the count as an `unsigned int`.
    /// A value with its top bit clear yields 0. It can be called from user,
    /// kernel, or interrupt context and cannot fail.
    pub fn stdc_leading_ones_ui(value: c_uint) -> c_uint;
    /// `stdc_leading_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned long` `value`, starting
    /// at the most significant bit, and returns the count as an `unsigned int`.
    /// A value with its top bit clear yields 0. It can be called from user,
    /// kernel, or interrupt context and cannot fail.
    pub fn stdc_leading_ones_ul(value: c_ulong) -> c_uint;
    /// `stdc_leading_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned long long` `value`,
    /// starting at the most significant bit, and returns the count as an
    /// `unsigned int`. A value with its top bit clear yields 0. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_leading_ones_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_trailing_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned char` `value`, starting
    /// at the least significant bit, and returns the count as an `unsigned
    /// int`. A value of zero yields the full bit width, 8. It can be called
    /// from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_zeros_uc(value: c_uchar) -> c_uint;
    /// `stdc_trailing_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned short` `value`,
    /// starting at the least significant bit, and returns the count as an
    /// `unsigned int`. A value of zero yields the full bit width, 16. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_zeros_us(value: c_ushort) -> c_uint;
    /// `stdc_trailing_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned int` `value`, starting
    /// at the least significant bit, and returns the count as an `unsigned
    /// int`. A value of zero yields the full bit width, 32. It can be called
    /// from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_zeros_ui(value: c_uint) -> c_uint;
    /// `stdc_trailing_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned long` `value`, starting
    /// at the least significant bit, and returns the count as an `unsigned
    /// int`. A value of zero yields the full bit width, 64. It can be called
    /// from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_zeros_ul(value: c_ulong) -> c_uint;
    /// `stdc_trailing_zeros(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive zero bits in an `unsigned long long` `value`,
    /// starting at the least significant bit, and returns the count as an
    /// `unsigned int`. A value of zero yields the full bit width, 64. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_zeros_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_trailing_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned char` `value`, starting
    /// at the least significant bit, and returns the count as an `unsigned
    /// int`. A value with its lowest bit clear yields 0. It can be called from
    /// user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_ones_uc(value: c_uchar) -> c_uint;
    /// `stdc_trailing_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned short` `value`, starting
    /// at the least significant bit, and returns the count as an `unsigned
    /// int`. A value with its lowest bit clear yields 0. It can be called from
    /// user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_ones_us(value: c_ushort) -> c_uint;
    /// `stdc_trailing_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned int` `value`, starting
    /// at the least significant bit, and returns the count as an `unsigned
    /// int`. A value with its lowest bit clear yields 0. It can be called from
    /// user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_ones_ui(value: c_uint) -> c_uint;
    /// `stdc_trailing_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned long` `value`, starting
    /// at the least significant bit, and returns the count as an `unsigned
    /// int`. A value with its lowest bit clear yields 0. It can be called from
    /// user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_ones_ul(value: c_ulong) -> c_uint;
    /// `stdc_trailing_ones(9F)`. Interface stability: Committed.
    ///
    /// Counts the consecutive one bits in an `unsigned long long` `value`,
    /// starting at the least significant bit, and returns the count as an
    /// `unsigned int`. A value with its lowest bit clear yields 0. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_trailing_ones_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_first_leading_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned char`
    /// `value`, counting from the most significant bit, so that a clear top bit
    /// yields 1 and a value with no zero bit at all yields 0. The C standard
    /// numbers the most significant bit as index 0 and this function returns
    /// that index plus one; the manual's example is the 16-bit value 0x952b,
    /// whose first zero bit from the top is at index 1 and which therefore
    /// returns 2 in the `unsigned short` form. Promoting a value to a wider
    /// type introduces leading zeros and makes the function return 1. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_leading_zero_uc(value: c_uchar) -> c_uint;
    /// `stdc_first_leading_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned short`
    /// `value`, counting from the most significant bit, so that a clear top bit
    /// yields 1 and a value with no zero bit at all yields 0. The C standard
    /// numbers the most significant bit as index 0 and this function returns
    /// that index plus one; the manual's example is the 16-bit value 0x952b,
    /// whose first zero bit from the top is at index 1 and which therefore
    /// returns 2 in the `unsigned short` form. Promoting a value to a wider
    /// type introduces leading zeros and makes the function return 1. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_leading_zero_us(value: c_ushort) -> c_uint;
    /// `stdc_first_leading_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned int`
    /// `value`, counting from the most significant bit, so that a clear top bit
    /// yields 1 and a value with no zero bit at all yields 0. The C standard
    /// numbers the most significant bit as index 0 and this function returns
    /// that index plus one; the manual's example is the 16-bit value 0x952b,
    /// whose first zero bit from the top is at index 1 and which therefore
    /// returns 2 in the `unsigned short` form. Promoting a value to a wider
    /// type introduces leading zeros and makes the function return 1. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_leading_zero_ui(value: c_uint) -> c_uint;
    /// `stdc_first_leading_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned long`
    /// `value`, counting from the most significant bit, so that a clear top bit
    /// yields 1 and a value with no zero bit at all yields 0. The C standard
    /// numbers the most significant bit as index 0 and this function returns
    /// that index plus one; the manual's example is the 16-bit value 0x952b,
    /// whose first zero bit from the top is at index 1 and which therefore
    /// returns 2 in the `unsigned short` form. Promoting a value to a wider
    /// type introduces leading zeros and makes the function return 1. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_leading_zero_ul(value: c_ulong) -> c_uint;
    /// `stdc_first_leading_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned long
    /// long` `value`, counting from the most significant bit, so that a clear
    /// top bit yields 1 and a value with no zero bit at all yields 0. The C
    /// standard numbers the most significant bit as index 0 and this function
    /// returns that index plus one; the manual's example is the 16-bit value
    /// 0x952b, whose first zero bit from the top is at index 1 and which
    /// therefore returns 2 in the `unsigned short` form. Promoting a value to a
    /// wider type introduces leading zeros and makes the function return 1. It
    /// can be called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_leading_zero_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_first_leading_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned char`
    /// `value`, counting from the most significant bit, so that a set top bit
    /// yields 1 and a value of zero yields 0. The C standard numbers the most
    /// significant bit as index 0 and this function returns that index plus
    /// one; the manual's example is the 16-bit value 0x952b, whose top bit is
    /// set and which therefore returns 1 in the `unsigned short` form.
    /// Promoting a non-zero value to a wider type introduces leading zeros and
    /// raises the result. `ddi_fls(9F)` finds the same bit but numbers it from
    /// the least significant end. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_first_leading_one_uc(value: c_uchar) -> c_uint;
    /// `stdc_first_leading_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned short`
    /// `value`, counting from the most significant bit, so that a set top bit
    /// yields 1 and a value of zero yields 0. The C standard numbers the most
    /// significant bit as index 0 and this function returns that index plus
    /// one; the manual's example is the 16-bit value 0x952b, whose top bit is
    /// set and which therefore returns 1 in the `unsigned short` form.
    /// Promoting a non-zero value to a wider type introduces leading zeros and
    /// raises the result. `ddi_fls(9F)` finds the same bit but numbers it from
    /// the least significant end. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_first_leading_one_us(value: c_ushort) -> c_uint;
    /// `stdc_first_leading_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned int`
    /// `value`, counting from the most significant bit, so that a set top bit
    /// yields 1 and a value of zero yields 0. The C standard numbers the most
    /// significant bit as index 0 and this function returns that index plus
    /// one; the manual's example is the 16-bit value 0x952b, whose top bit is
    /// set and which therefore returns 1 in the `unsigned short` form.
    /// Promoting a non-zero value to a wider type introduces leading zeros and
    /// raises the result. `ddi_fls(9F)` finds the same bit but numbers it from
    /// the least significant end. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_first_leading_one_ui(value: c_uint) -> c_uint;
    /// `stdc_first_leading_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned long`
    /// `value`, counting from the most significant bit, so that a set top bit
    /// yields 1 and a value of zero yields 0. The C standard numbers the most
    /// significant bit as index 0 and this function returns that index plus
    /// one; the manual's example is the 16-bit value 0x952b, whose top bit is
    /// set and which therefore returns 1 in the `unsigned short` form.
    /// Promoting a non-zero value to a wider type introduces leading zeros and
    /// raises the result. `ddi_fls(9F)` finds the same bit but numbers it from
    /// the least significant end. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_first_leading_one_ul(value: c_ulong) -> c_uint;
    /// `stdc_first_leading_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned long
    /// long` `value`, counting from the most significant bit, so that a set top
    /// bit yields 1 and a value of zero yields 0. The C standard numbers the
    /// most significant bit as index 0 and this function returns that index
    /// plus one; the manual's example is the 16-bit value 0x952b, whose top bit
    /// is set and which therefore returns 1 in the `unsigned short` form.
    /// Promoting a non-zero value to a wider type introduces leading zeros and
    /// raises the result. `ddi_fls(9F)` finds the same bit but numbers it from
    /// the least significant end. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_first_leading_one_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_first_trailing_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned char`
    /// `value`, counting from the least significant bit, so that a clear low
    /// bit yields 1 and a value with every bit set yields 0. It can be called
    /// from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_trailing_zero_uc(value: c_uchar) -> c_uint;
    /// `stdc_first_trailing_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned short`
    /// `value`, counting from the least significant bit, so that a clear low
    /// bit yields 1 and a value with every bit set yields 0. It can be called
    /// from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_trailing_zero_us(value: c_ushort) -> c_uint;
    /// `stdc_first_trailing_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned int`
    /// `value`, counting from the least significant bit, so that a clear low
    /// bit yields 1 and a value with every bit set yields 0. It can be called
    /// from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_trailing_zero_ui(value: c_uint) -> c_uint;
    /// `stdc_first_trailing_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned long`
    /// `value`, counting from the least significant bit, so that a clear low
    /// bit yields 1 and a value with every bit set yields 0. It can be called
    /// from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_trailing_zero_ul(value: c_ulong) -> c_uint;
    /// `stdc_first_trailing_zero(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first zero bit in an `unsigned long
    /// long` `value`, counting from the least significant bit, so that a clear
    /// low bit yields 1 and a value with every bit set yields 0. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_first_trailing_zero_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_first_trailing_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned char`
    /// `value`, counting from the least significant bit, so that a set low bit
    /// yields 1 and a value of zero yields 0. This is the operation often
    /// called find first set, and `ddi_ffs(9F)` provides a signed equivalent.
    /// It can be called from user, kernel, or interrupt context and cannot
    /// fail.
    pub fn stdc_first_trailing_one_uc(value: c_uchar) -> c_uint;
    /// `stdc_first_trailing_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned short`
    /// `value`, counting from the least significant bit, so that a set low bit
    /// yields 1 and a value of zero yields 0. This is the operation often
    /// called find first set, and `ddi_ffs(9F)` provides a signed equivalent.
    /// It can be called from user, kernel, or interrupt context and cannot
    /// fail.
    pub fn stdc_first_trailing_one_us(value: c_ushort) -> c_uint;
    /// `stdc_first_trailing_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned int`
    /// `value`, counting from the least significant bit, so that a set low bit
    /// yields 1 and a value of zero yields 0. This is the operation often
    /// called find first set, and `ddi_ffs(9F)` provides a signed equivalent.
    /// It can be called from user, kernel, or interrupt context and cannot
    /// fail.
    pub fn stdc_first_trailing_one_ui(value: c_uint) -> c_uint;
    /// `stdc_first_trailing_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned long`
    /// `value`, counting from the least significant bit, so that a set low bit
    /// yields 1 and a value of zero yields 0. This is the operation often
    /// called find first set, and `ddi_ffs(9F)` provides a signed equivalent.
    /// It can be called from user, kernel, or interrupt context and cannot
    /// fail.
    pub fn stdc_first_trailing_one_ul(value: c_ulong) -> c_uint;
    /// `stdc_first_trailing_one(9F)`. Interface stability: Committed.
    ///
    /// Returns the one-based index of the first one bit in an `unsigned long
    /// long` `value`, counting from the least significant bit, so that a set
    /// low bit yields 1 and a value of zero yields 0. This is the operation
    /// often called find first set, and `ddi_ffs(9F)` provides a signed
    /// equivalent. It can be called from user, kernel, or interrupt context and
    /// cannot fail.
    pub fn stdc_first_trailing_one_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_count_zeros(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of zero bits in an `unsigned char` `value` as an
    /// `unsigned int`. The manual notes that promoting a value to a wider type
    /// fills the new positions with zeros and so raises the result. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_zeros_uc(value: c_uchar) -> c_uint;
    /// `stdc_count_zeros(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of zero bits in an `unsigned short` `value` as an
    /// `unsigned int`. The manual notes that promoting a value to a wider type
    /// fills the new positions with zeros and so raises the result. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_zeros_us(value: c_ushort) -> c_uint;
    /// `stdc_count_zeros(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of zero bits in an `unsigned int` `value` as an
    /// `unsigned int`. The manual notes that promoting a value to a wider type
    /// fills the new positions with zeros and so raises the result. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_zeros_ui(value: c_uint) -> c_uint;
    /// `stdc_count_zeros(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of zero bits in an `unsigned long` `value` as an
    /// `unsigned int`. The manual notes that promoting a value to a wider type
    /// fills the new positions with zeros and so raises the result. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_zeros_ul(value: c_ulong) -> c_uint;
    /// `stdc_count_zeros(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of zero bits in an `unsigned long long` `value` as an
    /// `unsigned int`. The manual notes that promoting a value to a wider type
    /// fills the new positions with zeros and so raises the result. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_zeros_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_count_ones(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of one bits in an `unsigned char` `value` as an
    /// `unsigned int`, an operation often called a population count. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_ones_uc(value: c_uchar) -> c_uint;
    /// `stdc_count_ones(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of one bits in an `unsigned short` `value` as an
    /// `unsigned int`, an operation often called a population count. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_ones_us(value: c_ushort) -> c_uint;
    /// `stdc_count_ones(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of one bits in an `unsigned int` `value` as an
    /// `unsigned int`, an operation often called a population count. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_ones_ui(value: c_uint) -> c_uint;
    /// `stdc_count_ones(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of one bits in an `unsigned long` `value` as an
    /// `unsigned int`, an operation often called a population count. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_ones_ul(value: c_ulong) -> c_uint;
    /// `stdc_count_ones(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of one bits in an `unsigned long long` `value` as an
    /// `unsigned int`, an operation often called a population count. It can be
    /// called from user, kernel, or interrupt context and cannot fail.
    pub fn stdc_count_ones_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_has_single_bit(9F)`. Interface stability: Committed.
    ///
    /// Returns true if exactly one bit of an `unsigned char` `value` is set,
    /// that is, if the value is a power of two, and false otherwise, including
    /// for zero. It can be called from user, kernel, or interrupt context and
    /// cannot fail.
    pub fn stdc_has_single_bit_uc(value: c_uchar) -> bool;
    /// `stdc_has_single_bit(9F)`. Interface stability: Committed.
    ///
    /// Returns true if exactly one bit of an `unsigned short` `value` is set,
    /// that is, if the value is a power of two, and false otherwise, including
    /// for zero. It can be called from user, kernel, or interrupt context and
    /// cannot fail.
    pub fn stdc_has_single_bit_us(value: c_ushort) -> bool;
    /// `stdc_has_single_bit(9F)`. Interface stability: Committed.
    ///
    /// Returns true if exactly one bit of an `unsigned int` `value` is set,
    /// that is, if the value is a power of two, and false otherwise, including
    /// for zero. It can be called from user, kernel, or interrupt context and
    /// cannot fail.
    pub fn stdc_has_single_bit_ui(value: c_uint) -> bool;
    /// `stdc_has_single_bit(9F)`. Interface stability: Committed.
    ///
    /// Returns true if exactly one bit of an `unsigned long` `value` is set,
    /// that is, if the value is a power of two, and false otherwise, including
    /// for zero. It can be called from user, kernel, or interrupt context and
    /// cannot fail.
    pub fn stdc_has_single_bit_ul(value: c_ulong) -> bool;
    /// `stdc_has_single_bit(9F)`. Interface stability: Committed.
    ///
    /// Returns true if exactly one bit of an `unsigned long long` `value` is
    /// set, that is, if the value is a power of two, and false otherwise,
    /// including for zero. It can be called from user, kernel, or interrupt
    /// context and cannot fail.
    pub fn stdc_has_single_bit_ull(value: c_ulonglong) -> bool;

    /// `stdc_bit_width(9F)`. Interface stability: Committed.
    ///
    /// Returns the minimum number of bits needed to represent an `unsigned
    /// char` `value`, which is one more than the base two logarithm of the
    /// value, or 0 when the value is 0. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_bit_width_uc(value: c_uchar) -> c_uint;
    /// `stdc_bit_width(9F)`. Interface stability: Committed.
    ///
    /// Returns the minimum number of bits needed to represent an `unsigned
    /// short` `value`, which is one more than the base two logarithm of the
    /// value, or 0 when the value is 0. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_bit_width_us(value: c_ushort) -> c_uint;
    /// `stdc_bit_width(9F)`. Interface stability: Committed.
    ///
    /// Returns the minimum number of bits needed to represent an `unsigned int`
    /// `value`, which is one more than the base two logarithm of the value, or
    /// 0 when the value is 0. It can be called from user, kernel, or interrupt
    /// context and cannot fail.
    pub fn stdc_bit_width_ui(value: c_uint) -> c_uint;
    /// `stdc_bit_width(9F)`. Interface stability: Committed.
    ///
    /// Returns the minimum number of bits needed to represent an `unsigned
    /// long` `value`, which is one more than the base two logarithm of the
    /// value, or 0 when the value is 0. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_bit_width_ul(value: c_ulong) -> c_uint;
    /// `stdc_bit_width(9F)`. Interface stability: Committed.
    ///
    /// Returns the minimum number of bits needed to represent an `unsigned long
    /// long` `value`, which is one more than the base two logarithm of the
    /// value, or 0 when the value is 0. It can be called from user, kernel, or
    /// interrupt context and cannot fail.
    pub fn stdc_bit_width_ull(value: c_ulonglong) -> c_uint;

    /// `stdc_bit_floor(9F)`. Interface stability: Committed.
    ///
    /// Returns the largest power of two that is not greater than an `unsigned
    /// char` `value`, or 0 when the value is 0. The result has the same type as
    /// the argument. It can be called from user, kernel, or interrupt context
    /// and cannot fail.
    pub fn stdc_bit_floor_uc(value: c_uchar) -> c_uchar;
    /// `stdc_bit_floor(9F)`. Interface stability: Committed.
    ///
    /// Returns the largest power of two that is not greater than an `unsigned
    /// short` `value`, or 0 when the value is 0. The result has the same type
    /// as the argument. It can be called from user, kernel, or interrupt
    /// context and cannot fail.
    pub fn stdc_bit_floor_us(value: c_ushort) -> c_ushort;
    /// `stdc_bit_floor(9F)`. Interface stability: Committed.
    ///
    /// Returns the largest power of two that is not greater than an `unsigned
    /// int` `value`, or 0 when the value is 0. The result has the same type as
    /// the argument. It can be called from user, kernel, or interrupt context
    /// and cannot fail.
    pub fn stdc_bit_floor_ui(value: c_uint) -> c_uint;
    /// `stdc_bit_floor(9F)`. Interface stability: Committed.
    ///
    /// Returns the largest power of two that is not greater than an `unsigned
    /// long` `value`, or 0 when the value is 0. The result has the same type as
    /// the argument. It can be called from user, kernel, or interrupt context
    /// and cannot fail.
    pub fn stdc_bit_floor_ul(value: c_ulong) -> c_ulong;
    /// `stdc_bit_floor(9F)`. Interface stability: Committed.
    ///
    /// Returns the largest power of two that is not greater than an `unsigned
    /// long long` `value`, or 0 when the value is 0. The result has the same
    /// type as the argument. It can be called from user, kernel, or interrupt
    /// context and cannot fail.
    pub fn stdc_bit_floor_ull(value: c_ulonglong) -> c_ulonglong;

    /// `stdc_bit_ceil(9F)`. Interface stability: Committed.
    ///
    /// Returns the smallest power of two that is not less than an `unsigned
    /// char` `value`. If that power of two does not fit in the type the result
    /// is 0; the manual's example is `stdc_bit_ceil_uc` applied to `UINT8_MAX`,
    /// whose next power of two, 0x100, does not fit in eight bits. The result
    /// has the same type as the argument. It can be called from user, kernel,
    /// or interrupt context and cannot fail.
    pub fn stdc_bit_ceil_uc(value: c_uchar) -> c_uchar;
    /// `stdc_bit_ceil(9F)`. Interface stability: Committed.
    ///
    /// Returns the smallest power of two that is not less than an `unsigned
    /// short` `value`. If that power of two does not fit in the type the result
    /// is 0; the manual's example is `stdc_bit_ceil_uc` applied to `UINT8_MAX`,
    /// whose next power of two, 0x100, does not fit in eight bits. The result
    /// has the same type as the argument. It can be called from user, kernel,
    /// or interrupt context and cannot fail.
    pub fn stdc_bit_ceil_us(value: c_ushort) -> c_ushort;
    /// `stdc_bit_ceil(9F)`. Interface stability: Committed.
    ///
    /// Returns the smallest power of two that is not less than an `unsigned
    /// int` `value`. If that power of two does not fit in the type the result
    /// is 0; the manual's example is `stdc_bit_ceil_uc` applied to `UINT8_MAX`,
    /// whose next power of two, 0x100, does not fit in eight bits. The result
    /// has the same type as the argument. It can be called from user, kernel,
    /// or interrupt context and cannot fail.
    pub fn stdc_bit_ceil_ui(value: c_uint) -> c_uint;
    /// `stdc_bit_ceil(9F)`. Interface stability: Committed.
    ///
    /// Returns the smallest power of two that is not less than an `unsigned
    /// long` `value`. If that power of two does not fit in the type the result
    /// is 0; the manual's example is `stdc_bit_ceil_uc` applied to `UINT8_MAX`,
    /// whose next power of two, 0x100, does not fit in eight bits. The result
    /// has the same type as the argument. It can be called from user, kernel,
    /// or interrupt context and cannot fail.
    pub fn stdc_bit_ceil_ul(value: c_ulong) -> c_ulong;
    /// `stdc_bit_ceil(9F)`. Interface stability: Committed.
    ///
    /// Returns the smallest power of two that is not less than an `unsigned
    /// long long` `value`. If that power of two does not fit in the type the
    /// result is 0; the manual's example is `stdc_bit_ceil_uc` applied to
    /// `UINT8_MAX`, whose next power of two, 0x100, does not fit in eight bits.
    /// The result has the same type as the argument. It can be called from
    /// user, kernel, or interrupt context and cannot fail.
    pub fn stdc_bit_ceil_ull(value: c_ulonglong) -> c_ulonglong;
}
