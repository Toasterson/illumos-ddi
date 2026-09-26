//! Atomic operations and memory barriers from `sys/atomic.h`.
//!
//! These are the kernel's primitive atomic memory operations, implemented
//! in assembly for each architecture and exported from `unix`. They act on
//! a location that several threads or an interrupt handler may touch at
//! once, and guarantee that each operation happens as a whole. The manual
//! groups them into eight families, one page each, for increment,
//! decrement, add, bitwise or, bitwise and, compare and swap, swap, and
//! exclusive bit set and clear, plus `membar_ops(9F)` for the four memory
//! barriers. The umbrella page `atomic_ops(9F)` lists the families.
//!
//! Rust code should normally not call these. `core::sync::atomic` provides
//! the same operations as inherent methods on `AtomicU32`, `AtomicUsize`
//! and the rest, with explicit memory orderings, and they compile to the
//! same instructions with no call overhead. The declarations exist for
//! structures that are shared with C code and must be updated with the
//! kernel's own routines to preserve whatever ordering the C side relies
//! on, and for the sake of the `ddi` mapfile, which lists
//! `atomic_inc_32_nv`, `atomic_dec_32_nv` and the four barriers.
//!
//! The C prototypes declare the target as a `volatile` pointer. Rust has no
//! `volatile` qualifier on pointer types, so every target is a plain
//! `*mut`; the volatility lives in the callee. The manual pages for this
//! family have no INTERFACE LEVEL section. In its place their ATTRIBUTES
//! table gives an Interface Stability of Committed, and that is what the
//! first line of each doc comment quotes.
//!
//! The `atomic_inc(9F)` and `atomic_dec(9F)` pages also list
//! `atomic_inc_ptr`, `atomic_inc_ptr_nv`, `atomic_dec_ptr` and
//! `atomic_dec_ptr_nv`. Neither the header nor the assembly defines those
//! four, so they are not declared here. The `atomic_add_ptr`,
//! `atomic_cas_ptr` and `atomic_swap_ptr` functions do exist and are
//! bound; on amd64 they share their code with the `ulong` forms. The
//! header also defines the aliases `atomic_add_ip`, `atomic_add_ip_nv`
//! and `casip` as macros for the `long` forms, which a Rust caller can
//! simply spell out.

use core::ffi::c_schar;

use crate::types::*;

unsafe extern "C" {
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint8_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_inc_8(target: *mut u8);
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uchar_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_inc_uchar(target: *mut c_uchar);
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint16_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_inc_16(target: *mut u16);
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `ushort_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_inc_ushort(target: *mut c_ushort);
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint32_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_inc_32(target: *mut u32);
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_inc_uint(target: *mut uint_t);
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `ulong_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_inc_ulong(target: *mut ulong_t);
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint64_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_inc_64(target: *mut u64);
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint8_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_inc_8_nv(target: *mut u8) -> u8;
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uchar_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_inc_uchar_nv(target: *mut c_uchar) -> c_uchar;
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint16_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_inc_16_nv(target: *mut u16) -> u16;
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `ushort_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_inc_ushort_nv(target: *mut c_ushort) -> c_ushort;
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint32_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_inc_32_nv(target: *mut u32) -> u32;
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_inc_uint_nv(target: *mut uint_t) -> uint_t;
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `ulong_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_inc_ulong_nv(target: *mut ulong_t) -> ulong_t;
    /// `atomic_inc(9F)`. Interface stability: Committed.
    ///
    /// Increments by one the `uint64_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_inc_64_nv(target: *mut u64) -> u64;

    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint8_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_dec_8(target: *mut u8);
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uchar_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_dec_uchar(target: *mut c_uchar);
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint16_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_dec_16(target: *mut u16);
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `ushort_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_dec_ushort(target: *mut c_ushort);
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint32_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_dec_32(target: *mut u32);
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_dec_uint(target: *mut uint_t);
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `ulong_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_dec_ulong(target: *mut ulong_t);
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint64_t` that `target` points to, as a single
    /// atomic operation. Nothing is returned; use the `_nv` form to obtain the
    /// new value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_dec_64(target: *mut u64);
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint8_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_dec_8_nv(target: *mut u8) -> u8;
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uchar_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_dec_uchar_nv(target: *mut c_uchar) -> c_uchar;
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint16_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_dec_16_nv(target: *mut u16) -> u16;
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `ushort_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_dec_ushort_nv(target: *mut c_ushort) -> c_ushort;
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint32_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_dec_32_nv(target: *mut u32) -> u32;
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_dec_uint_nv(target: *mut uint_t) -> uint_t;
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `ulong_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_dec_ulong_nv(target: *mut ulong_t) -> ulong_t;
    /// `atomic_dec(9F)`. Interface stability: Committed.
    ///
    /// Decrements by one the `uint64_t` that `target` points to, as a single
    /// atomic operation, and returns the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_dec_64_nv(target: *mut u64) -> u64;

    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int8_t` `delta` to the `uint8_t` that `target` points to, as a
    /// single atomic operation. The addend is signed, so a negative delta
    /// subtracts. Nothing is returned; use the `_nv` form to obtain the new
    /// value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_add_8(target: *mut u8, delta: i8);
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `signed char` `delta` to the `uchar_t` that `target` points to,
    /// as a single atomic operation. The addend is signed, so a negative delta
    /// subtracts. Nothing is returned; use the `_nv` form to obtain the new
    /// value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_add_char(target: *mut c_uchar, delta: c_schar);
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int16_t` `delta` to the `uint16_t` that `target` points to, as
    /// a single atomic operation. The addend is signed, so a negative delta
    /// subtracts. Nothing is returned; use the `_nv` form to obtain the new
    /// value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_add_16(target: *mut u16, delta: i16);
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `short` `delta` to the `ushort_t` that `target` points to, as a
    /// single atomic operation. The addend is signed, so a negative delta
    /// subtracts. Nothing is returned; use the `_nv` form to obtain the new
    /// value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_add_short(target: *mut c_ushort, delta: c_short);
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int32_t` `delta` to the `uint32_t` that `target` points to, as
    /// a single atomic operation. The addend is signed, so a negative delta
    /// subtracts. Nothing is returned; use the `_nv` form to obtain the new
    /// value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_add_32(target: *mut u32, delta: i32);
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int` `delta` to the `uint_t` that `target` points to, as a
    /// single atomic operation. The addend is signed, so a negative delta
    /// subtracts. Nothing is returned; use the `_nv` form to obtain the new
    /// value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_add_int(target: *mut uint_t, delta: c_int);
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `ssize_t` `delta`, in bytes, to the pointer that `target`
    /// points to, as a single atomic operation. The addend is signed, so a
    /// negative delta subtracts. Nothing is returned; use the `_nv` form to
    /// obtain the new value. It can be called from user, interrupt, or kernel
    /// context.
    pub fn atomic_add_ptr(target: *mut c_void, delta: ssize_t);
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `long` `delta` to the `ulong_t` that `target` points to, as a
    /// single atomic operation. The addend is signed, so a negative delta
    /// subtracts. Nothing is returned; use the `_nv` form to obtain the new
    /// value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_add_long(target: *mut ulong_t, delta: c_long);
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int64_t` `delta` to the `uint64_t` that `target` points to, as
    /// a single atomic operation. The addend is signed, so a negative delta
    /// subtracts. Nothing is returned; use the `_nv` form to obtain the new
    /// value. It can be called from user, interrupt, or kernel context.
    pub fn atomic_add_64(target: *mut u64, delta: i64);
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int8_t` `delta` to the `uint8_t` that `target` points to, as a
    /// single atomic operation, and returns the new value. It can be called
    /// from user, interrupt, or kernel context.
    pub fn atomic_add_8_nv(target: *mut u8, delta: i8) -> u8;
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `signed char` `delta` to the `uchar_t` that `target` points to,
    /// as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_add_char_nv(target: *mut c_uchar, delta: c_schar) -> c_uchar;
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int16_t` `delta` to the `uint16_t` that `target` points to, as
    /// a single atomic operation, and returns the new value. It can be called
    /// from user, interrupt, or kernel context.
    pub fn atomic_add_16_nv(target: *mut u16, delta: i16) -> u16;
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `short` `delta` to the `ushort_t` that `target` points to, as a
    /// single atomic operation, and returns the new value. It can be called
    /// from user, interrupt, or kernel context.
    pub fn atomic_add_short_nv(target: *mut c_ushort, delta: c_short) -> c_ushort;
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int32_t` `delta` to the `uint32_t` that `target` points to, as
    /// a single atomic operation, and returns the new value. It can be called
    /// from user, interrupt, or kernel context.
    pub fn atomic_add_32_nv(target: *mut u32, delta: i32) -> u32;
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int` `delta` to the `uint_t` that `target` points to, as a
    /// single atomic operation, and returns the new value. It can be called
    /// from user, interrupt, or kernel context.
    pub fn atomic_add_int_nv(target: *mut uint_t, delta: c_int) -> uint_t;
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `ssize_t` `delta`, in bytes, to the pointer that `target`
    /// points to, as a single atomic operation, and returns the new value. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_add_ptr_nv(target: *mut c_void, delta: ssize_t) -> *mut c_void;
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `long` `delta` to the `ulong_t` that `target` points to, as a
    /// single atomic operation, and returns the new value. It can be called
    /// from user, interrupt, or kernel context.
    pub fn atomic_add_long_nv(target: *mut ulong_t, delta: c_long) -> ulong_t;
    /// `atomic_add(9F)`. Interface stability: Committed.
    ///
    /// Adds the `int64_t` `delta` to the `uint64_t` that `target` points to, as
    /// a single atomic operation, and returns the new value. It can be called
    /// from user, interrupt, or kernel context.
    pub fn atomic_add_64_nv(target: *mut u64, delta: i64) -> u64;

    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint8_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_or_8(target: *mut u8, bits: u8);
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uchar_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_or_uchar(target: *mut c_uchar, bits: c_uchar);
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint16_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_or_16(target: *mut u16, bits: u16);
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `ushort_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_or_ushort(target: *mut c_ushort, bits: c_ushort);
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint32_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_or_32(target: *mut u32, bits: u32);
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_or_uint(target: *mut uint_t, bits: uint_t);
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `ulong_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_or_ulong(target: *mut ulong_t, bits: ulong_t);
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint64_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_or_64(target: *mut u64, bits: u64);
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint8_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_or_8_nv(target: *mut u8, bits: u8) -> u8;
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uchar_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_or_uchar_nv(target: *mut c_uchar, bits: c_uchar) -> c_uchar;
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint16_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_or_16_nv(target: *mut u16, bits: u16) -> u16;
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `ushort_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_or_ushort_nv(target: *mut c_ushort, bits: c_ushort) -> c_ushort;
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint32_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_or_32_nv(target: *mut u32, bits: u32) -> u32;
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_or_uint_nv(target: *mut uint_t, bits: uint_t) -> uint_t;
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `ulong_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_or_ulong_nv(target: *mut ulong_t, bits: ulong_t) -> ulong_t;
    /// `atomic_or(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise or of `bits` into the `uint64_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_or_64_nv(target: *mut u64, bits: u64) -> u64;

    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint8_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_and_8(target: *mut u8, bits: u8);
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uchar_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_and_uchar(target: *mut c_uchar, bits: c_uchar);
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint16_t` that `target`
    /// points to, as a single atomic operation. Nothing is returned; use the
    /// `_nv` form to obtain the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_and_16(target: *mut u16, bits: u16);
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `ushort_t` that `target`
    /// points to, as a single atomic operation. Nothing is returned; use the
    /// `_nv` form to obtain the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_and_ushort(target: *mut c_ushort, bits: c_ushort);
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint32_t` that `target`
    /// points to, as a single atomic operation. Nothing is returned; use the
    /// `_nv` form to obtain the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_and_32(target: *mut u32, bits: u32);
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_and_uint(target: *mut uint_t, bits: uint_t);
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `ulong_t` that `target` points
    /// to, as a single atomic operation. Nothing is returned; use the `_nv`
    /// form to obtain the new value. It can be called from user, interrupt, or
    /// kernel context.
    pub fn atomic_and_ulong(target: *mut ulong_t, bits: ulong_t);
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint64_t` that `target`
    /// points to, as a single atomic operation. Nothing is returned; use the
    /// `_nv` form to obtain the new value. It can be called from user,
    /// interrupt, or kernel context.
    pub fn atomic_and_64(target: *mut u64, bits: u64);
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint8_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_and_8_nv(target: *mut u8, bits: u8) -> u8;
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uchar_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_and_uchar_nv(target: *mut c_uchar, bits: c_uchar) -> c_uchar;
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint16_t` that `target`
    /// points to, as a single atomic operation, and returns the new value. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_and_16_nv(target: *mut u16, bits: u16) -> u16;
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `ushort_t` that `target`
    /// points to, as a single atomic operation, and returns the new value. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_and_ushort_nv(target: *mut c_ushort, bits: c_ushort) -> c_ushort;
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint32_t` that `target`
    /// points to, as a single atomic operation, and returns the new value. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_and_32_nv(target: *mut u32, bits: u32) -> u32;
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_and_uint_nv(target: *mut uint_t, bits: uint_t) -> uint_t;
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `ulong_t` that `target` points
    /// to, as a single atomic operation, and returns the new value. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_and_ulong_nv(target: *mut ulong_t, bits: ulong_t) -> ulong_t;
    /// `atomic_and(9F)`. Interface stability: Committed.
    ///
    /// Performs a bitwise and of `bits` with the `uint64_t` that `target`
    /// points to, as a single atomic operation, and returns the new value. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_and_64_nv(target: *mut u64, bits: u64) -> u64;

    /// `atomic_cas(9F)`. Interface stability: Committed.
    ///
    /// Compares the `uint8_t` that `target` points to with `cmp` and, if the
    /// two are equal, stores `newval` in its place, all as a single atomic
    /// operation. It returns the value that `target` held before the call
    /// whether or not the store happened, so a caller detects success by
    /// comparing the result with `cmp`. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_cas_8(target: *mut u8, cmp: u8, newval: u8) -> u8;
    /// `atomic_cas(9F)`. Interface stability: Committed.
    ///
    /// Compares the `uchar_t` that `target` points to with `cmp` and, if the
    /// two are equal, stores `newval` in its place, all as a single atomic
    /// operation. It returns the value that `target` held before the call
    /// whether or not the store happened, so a caller detects success by
    /// comparing the result with `cmp`. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_cas_uchar(target: *mut c_uchar, cmp: c_uchar, newval: c_uchar) -> c_uchar;
    /// `atomic_cas(9F)`. Interface stability: Committed.
    ///
    /// Compares the `uint16_t` that `target` points to with `cmp` and, if the
    /// two are equal, stores `newval` in its place, all as a single atomic
    /// operation. It returns the value that `target` held before the call
    /// whether or not the store happened, so a caller detects success by
    /// comparing the result with `cmp`. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_cas_16(target: *mut u16, cmp: u16, newval: u16) -> u16;
    /// `atomic_cas(9F)`. Interface stability: Committed.
    ///
    /// Compares the `ushort_t` that `target` points to with `cmp` and, if the
    /// two are equal, stores `newval` in its place, all as a single atomic
    /// operation. It returns the value that `target` held before the call
    /// whether or not the store happened, so a caller detects success by
    /// comparing the result with `cmp`. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_cas_ushort(target: *mut c_ushort, cmp: c_ushort, newval: c_ushort) -> c_ushort;
    /// `atomic_cas(9F)`. Interface stability: Committed.
    ///
    /// Compares the `uint32_t` that `target` points to with `cmp` and, if the
    /// two are equal, stores `newval` in its place, all as a single atomic
    /// operation. It returns the value that `target` held before the call
    /// whether or not the store happened, so a caller detects success by
    /// comparing the result with `cmp`. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_cas_32(target: *mut u32, cmp: u32, newval: u32) -> u32;
    /// `atomic_cas(9F)`. Interface stability: Committed.
    ///
    /// Compares the `uint_t` that `target` points to with `cmp` and, if the two
    /// are equal, stores `newval` in its place, all as a single atomic
    /// operation. It returns the value that `target` held before the call
    /// whether or not the store happened, so a caller detects success by
    /// comparing the result with `cmp`. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_cas_uint(target: *mut uint_t, cmp: uint_t, newval: uint_t) -> uint_t;
    /// `atomic_cas(9F)`. Interface stability: Committed.
    ///
    /// Compares the pointer that `target` points to with `cmp` and, if the two
    /// are equal, stores `newval` in its place, all as a single atomic
    /// operation. It returns the value that `target` held before the call
    /// whether or not the store happened, so a caller detects success by
    /// comparing the result with `cmp`. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_cas_ptr(target: *mut c_void, cmp: *mut c_void, newval: *mut c_void) -> *mut c_void;
    /// `atomic_cas(9F)`. Interface stability: Committed.
    ///
    /// Compares the `ulong_t` that `target` points to with `cmp` and, if the
    /// two are equal, stores `newval` in its place, all as a single atomic
    /// operation. It returns the value that `target` held before the call
    /// whether or not the store happened, so a caller detects success by
    /// comparing the result with `cmp`. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_cas_ulong(target: *mut ulong_t, cmp: ulong_t, newval: ulong_t) -> ulong_t;
    /// `atomic_cas(9F)`. Interface stability: Committed.
    ///
    /// Compares the `uint64_t` that `target` points to with `cmp` and, if the
    /// two are equal, stores `newval` in its place, all as a single atomic
    /// operation. It returns the value that `target` held before the call
    /// whether or not the store happened, so a caller detects success by
    /// comparing the result with `cmp`. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_cas_64(target: *mut u64, cmp: u64, newval: u64) -> u64;

    /// `atomic_swap(9F)`. Interface stability: Committed.
    ///
    /// Replaces the `uint8_t` that `target` points to with `newval`, as a
    /// single atomic operation, and returns the value that was there before. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_swap_8(target: *mut u8, newval: u8) -> u8;
    /// `atomic_swap(9F)`. Interface stability: Committed.
    ///
    /// Replaces the `uchar_t` that `target` points to with `newval`, as a
    /// single atomic operation, and returns the value that was there before. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_swap_uchar(target: *mut c_uchar, newval: c_uchar) -> c_uchar;
    /// `atomic_swap(9F)`. Interface stability: Committed.
    ///
    /// Replaces the `uint16_t` that `target` points to with `newval`, as a
    /// single atomic operation, and returns the value that was there before. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_swap_16(target: *mut u16, newval: u16) -> u16;
    /// `atomic_swap(9F)`. Interface stability: Committed.
    ///
    /// Replaces the `ushort_t` that `target` points to with `newval`, as a
    /// single atomic operation, and returns the value that was there before. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_swap_ushort(target: *mut c_ushort, newval: c_ushort) -> c_ushort;
    /// `atomic_swap(9F)`. Interface stability: Committed.
    ///
    /// Replaces the `uint32_t` that `target` points to with `newval`, as a
    /// single atomic operation, and returns the value that was there before. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_swap_32(target: *mut u32, newval: u32) -> u32;
    /// `atomic_swap(9F)`. Interface stability: Committed.
    ///
    /// Replaces the `uint_t` that `target` points to with `newval`, as a single
    /// atomic operation, and returns the value that was there before. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_swap_uint(target: *mut uint_t, newval: uint_t) -> uint_t;
    /// `atomic_swap(9F)`. Interface stability: Committed.
    ///
    /// Replaces the pointer that `target` points to with `newval`, as a single
    /// atomic operation, and returns the value that was there before. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_swap_ptr(target: *mut c_void, newval: *mut c_void) -> *mut c_void;
    /// `atomic_swap(9F)`. Interface stability: Committed.
    ///
    /// Replaces the `ulong_t` that `target` points to with `newval`, as a
    /// single atomic operation, and returns the value that was there before. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_swap_ulong(target: *mut ulong_t, newval: ulong_t) -> ulong_t;
    /// `atomic_swap(9F)`. Interface stability: Committed.
    ///
    /// Replaces the `uint64_t` that `target` points to with `newval`, as a
    /// single atomic operation, and returns the value that was there before. It
    /// can be called from user, interrupt, or kernel context.
    pub fn atomic_swap_64(target: *mut u64, newval: u64) -> u64;

    /// `atomic_bits(9F)`. Interface stability: Committed.
    ///
    /// Sets bit number `bit` of the `ulong_t` that `target` points to, as a
    /// single atomic operation, but only if it was clear. It returns 0 if the
    /// bit was clear and has now been set, and -1 if it was already set, which
    /// lets a caller claim a slot exclusively. Bits are numbered from zero up
    /// to one less than the number of bits in a `long`; a number outside that
    /// range gives an undefined result. It can be called from user, interrupt,
    /// or kernel context.
    pub fn atomic_set_long_excl(target: *mut ulong_t, bit: uint_t);
    /// `atomic_bits(9F)`. Interface stability: Committed.
    ///
    /// Clears bit number `bit` of the `ulong_t` that `target` points to, as a
    /// single atomic operation, but only if it was set. It returns 0 if the bit
    /// was set and has now been cleared, and -1 if it was already clear. Bits
    /// are numbered from zero up to one less than the number of bits in a
    /// `long`; a number outside that range gives an undefined result. It can be
    /// called from user, interrupt, or kernel context.
    pub fn atomic_clear_long_excl(target: *mut ulong_t, bit: uint_t);

    /// `membar_ops(9F)`. Interface stability: Committed.
    ///
    /// A general memory barrier for lock entry. It is placed after the memory
    /// operation that acquires a lock so that the lock really protects its
    /// data; no store issued after the barrier becomes visible, and no load
    /// issued after it is resolved, before the acquisition itself is globally
    /// visible. It is used together with [`membar_exit`] to let a region of
    /// code run in relaxed store order while preserving ordering at the lock
    /// boundaries. It can be called from user, interrupt, or kernel context.
    pub fn membar_enter();
    /// `membar_ops(9F)`. Interface stability: Committed.
    ///
    /// A general memory barrier for lock exit. It is placed before the memory
    /// operation that releases a lock so that every load and store issued
    /// before the barrier is resolved before the release becomes visible. It
    /// can be called from user, interrupt, or kernel context.
    pub fn membar_exit();
    /// `membar_ops(9F)`. Interface stability: Committed.
    ///
    /// Arranges for every store issued before this point to reach global
    /// visibility before any store that follows. A producer updates a data
    /// item, issues this barrier, and then sets the flag that announces the
    /// item, so that the flag can never be seen before the data. It can be
    /// called from user, interrupt, or kernel context.
    pub fn membar_producer();
    /// `membar_ops(9F)`. Interface stability: Committed.
    ///
    /// Arranges for every load issued before this point to complete before any
    /// load that follows. A consumer reads the flag that announces a data item,
    /// issues this barrier, and then reads the item, so that the data is never
    /// sampled before the flag has been seen. It can be called from user,
    /// interrupt, or kernel context.
    pub fn membar_consumer();
}
