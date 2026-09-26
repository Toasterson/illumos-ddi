//! String and memory functions from `sys/systm.h` and `sys/sunddi.h`.
//!
//! The kernel carries its own copies of the familiar C string and memory
//! routines, since a driver cannot link against libc. `string(9F)`
//! documents the null-terminated string family, `memchr(9F)` the
//! length-bounded memory family, and `bcopy(9F)`, `bcmp(9F)`, `bzero(9F)`
//! and `kcopy(9F)` the older byte-oriented routines. `sprintf(9F)` covers
//! the four functions that format into a caller-supplied buffer and
//! `stoi(9F)` the two decimal conversions. Every prototype below follows
//! the header rather than the manual where the two differ; in particular
//! `strchr`, `strrchr` and `memchr` return a mutable pointer, which is what
//! the C prototypes say when the header is not being compiled as C++.
//!
//! The `ddi` mapfile misspells one of its entries as `strrch`. The exported
//! symbol is `strrchr` and that is what is declared here. The mapfile also
//! lists `memcpy` and `memset`, which `memchr(9F)` documents together with
//! `memcmp` and `memmove`, so all five are bound under that page.
//!
//! `strstr`, `strpbrk`, `strsep` and `ovbcopy` are declared by the same
//! headers but have no section 9 manual page and do not appear in the
//! mapfile, so they are not bound. `ddi_strdup`, which `string(9F)` also
//! documents, and `swab(9F)` are bound elsewhere in the crate. The
//! `va_list` type that `vsprintf` and `vsnprintf` take is declared in
//! [the `cmn_err` module](mod@crate::cmn_err).

use crate::cmn_err::va_list;
use crate::types::*;

unsafe extern "C" {
    // --- string(9F) ---------------------------------------------------------
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Compares the strings `s1` and `s2` the way [`strcmp`] does, but
    /// ignores differences in case between upper and lower case letters.
    /// The comparison assumes the ASCII character set. It returns an
    /// integer greater than, equal to, or less than zero according to
    /// whether `s1` is greater than, equal to, or less than `s2`. It can be
    /// called from user, interrupt, or kernel context.
    pub fn strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The case-insensitive counterpart of [`strncmp`]. It compares at most
    /// `n` bytes of `s1` and `s2`, treating upper and lower case ASCII
    /// letters as equal, and returns an integer greater than, equal to, or
    /// less than zero with the same meaning as `strcmp`. It can be called
    /// from user, interrupt, or kernel context.
    pub fn strncasecmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Appends a copy of the string `s2`, including its terminating null
    /// character, to the end of the string `s1`. The first character of
    /// `s2` overwrites the null character that ended `s1`. The result is
    /// null-terminated and a pointer to it, which is `s1`, is returned. No
    /// check is made that the buffer behind `s1` is large enough, and the
    /// behaviour is undefined if the two strings overlap. It can be called
    /// from user, interrupt, or kernel context.
    pub fn strcat(s1: *mut c_char, s2: *const c_char) -> *mut c_char;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Appends at most `n` characters of `s2` to the end of `s1`, not
    /// counting any terminating null character, and then appends a null
    /// character. The first character of `s2` overwrites the null that
    /// ended `s1`. It returns `s1`. As with [`strcat`], overlapping strings
    /// give undefined results. It can be called from user, interrupt, or
    /// kernel context.
    pub fn strncat(s1: *mut c_char, s2: *const c_char, n: size_t) -> *mut c_char;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Appends at most `dstsize - strlen(dst) - 1` characters of `src` to
    /// `dst`, where `dstsize` is the full size of the buffer behind `dst`.
    /// If `dst` held a null-terminated string that fit in `dstsize` bytes
    /// on entry, it holds a null-terminated string that fits in `dstsize`
    /// bytes on return, with the first character of `src` having replaced
    /// the old terminator. If the string in `dst` was already longer than
    /// `dstsize` bytes, `dst` is not changed. The return value is the
    /// smaller of `dstsize` and `strlen(dst)` on entry, plus `strlen(src)`;
    /// a result greater than or equal to `dstsize` means the output was
    /// truncated. Overlapping arguments give undefined results. It can be
    /// called from user, interrupt, or kernel context.
    pub fn strlcat(dst: *mut c_char, src: *const c_char, dstsize: size_t) -> size_t;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a pointer to the first occurrence of `chr`, converted to a
    /// `char`, in the string `s`, or a null pointer if the character does
    /// not occur. The terminating null character counts as part of the
    /// string, so searching for zero finds the end of the string. The
    /// header returns a `char *` when compiled as C, which is why the
    /// return type is mutable here even though the argument is not. It can
    /// be called from user, interrupt, or kernel context.
    pub fn strchr(s: *const c_char, chr: c_int) -> *mut c_char;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a pointer to the last occurrence of `chr`, converted to a
    /// `char`, in the string `s`, or a null pointer if the character does
    /// not occur. The terminating null character counts as part of the
    /// string. This is the symbol the `ddi` mapfile lists as `strrch`. It
    /// can be called from user, interrupt, or kernel context.
    pub fn strrchr(s: *const c_char, chr: c_int) -> *mut c_char;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Compares the strings `s1` and `s2` byte by byte according to the
    /// machine's character ordering. It returns an integer greater than,
    /// equal to, or less than zero according to whether `s1` is greater
    /// than, equal to, or less than `s2`. The sign of a non-zero result is
    /// the sign of the difference between the first pair of bytes that
    /// differ. It can be called from user, interrupt, or kernel context.
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Makes the same comparison as [`strcmp`] but looks at no more than
    /// `n` bytes. Bytes after a null byte are not compared. It returns an
    /// integer greater than, equal to, or less than zero with the same
    /// meaning as `strcmp`. It can be called from user, interrupt, or
    /// kernel context.
    pub fn strncmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies the string `src` to `dst`, including the terminating null
    /// character, and stops after that null has been copied. It returns
    /// `dst`. The manual stresses that no check is made for overflow of
    /// the destination array, and that overlapping arguments give undefined
    /// results. It can be called from user, interrupt, or kernel context.
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies exactly `n` bytes from `src` to `dst`, truncating `src` if it
    /// is longer or padding `dst` with null characters if it is shorter.
    /// The result is not null-terminated if the length of `src` is `n` or
    /// more. It returns `dst`. Overlapping arguments give undefined
    /// results. It can be called from user, interrupt, or kernel context.
    pub fn strncpy(dst: *mut c_char, src: *const c_char, n: size_t) -> *mut c_char;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies at most `dstsize - 1` characters from `src` to `dst`, where
    /// `dstsize` is the full size of the buffer behind `dst`, truncating
    /// `src` if necessary. The result is always null-terminated. It returns
    /// `strlen(src)`, so a result greater than or equal to `dstsize` means
    /// the output was truncated. Overlapping arguments give undefined
    /// results. It can be called from user, interrupt, or kernel context.
    pub fn strlcpy(dst: *mut c_char, src: *const c_char, dstsize: size_t) -> size_t;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the memory behind the string `s`. That memory must be exactly
    /// `strlen(s) + 1` bytes long and must have been allocated, directly or
    /// indirectly, by `kmem_alloc(9F)` or `kmem_zalloc(9F)`; the strings
    /// returned by [`strdup`] and `ddi_strdup` qualify. It can be called
    /// from user, interrupt, or kernel context.
    pub fn strfree(s: *mut c_char);
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the length of the initial segment of `s1` that consists
    /// entirely of characters drawn from `s2`. It can be called from user,
    /// interrupt, or kernel context.
    pub fn strspn(s1: *const c_char, s2: *const c_char) -> size_t;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a pointer to a newly allocated copy of the string `s1`. The
    /// space comes from `kmem_alloc(9F)` and the copy can be released with
    /// [`strfree`] or with `kmem_free(9F)` and a length of
    /// `strlen(s1) + 1`. It behaves like `ddi_strdup` called with
    /// `KM_SLEEP`, which means it may sleep until memory is available and
    /// always succeeds. Because it can sleep it may only be called from
    /// user or kernel context, never from interrupt context.
    pub fn strdup(s1: *const c_char) -> *mut c_char;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the number of bytes in the string `s`, not counting the
    /// terminating null character. It can be called from user, interrupt,
    /// or kernel context.
    pub fn strlen(s: *const c_char) -> size_t;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the smaller of `n` and the number of bytes in `s` before its
    /// terminating null character. It never examines more than `n` bytes,
    /// so it is safe on buffers that might not be terminated. It can be
    /// called from user, interrupt, or kernel context.
    pub fn strnlen(s: *const c_char, n: size_t) -> size_t;
    /// `string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Splits the string `s1` into tokens separated by runs of one or more
    /// of the characters in `s2`. On the first call `s1` points at the
    /// string to scan and the value behind `lasts` is ignored; the function
    /// returns a pointer to the first token, writes a null character just
    /// after it, and stores its position in the variable that `lasts`
    /// points to. On later calls `s1` is a null pointer and `lasts` is
    /// passed back unchanged, so each call yields the next token. The
    /// separator string may differ between calls. When no token remains a
    /// null pointer is returned. The function modifies `s1`. It can be
    /// called from user, interrupt, or kernel context.
    pub fn strtok_r(s1: *mut c_char, s2: *const c_char, lasts: *mut *mut c_char) -> *mut c_char;

    // --- bcopy(9F), bcmp(9F), bzero(9F), kcopy(9F) ---------------------------
    /// `bcopy(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Copies `bcount` bytes from the kernel address `from` to the kernel
    /// address `to`. If the two ranges overlap the copy still runs but the
    /// result may not be what was intended. Both addresses must lie in
    /// kernel space; no range checking is done, and the manual warns that a
    /// bad address can corrupt the system unpredictably. It must never be
    /// used to move data to or from a user buffer, because it makes no
    /// provision for page faults and the system panics if the user pages
    /// are not resident; `ddi_copyin(9F)` and `ddi_copyout(9F)` exist for
    /// that. It can be called from user, interrupt, or kernel context.
    pub fn bcopy(from: *const c_void, to: *mut c_void, bcount: size_t);
    /// `bcmp(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Compares the two byte arrays `s1` and `s2` of length `len`. It
    /// returns 0 if the arrays are identical and 1 if they are not. Unlike
    /// [`strcmp`] it does not stop at a null byte. It can be called from
    /// user, interrupt, or kernel context.
    pub fn bcmp(s1: *const c_void, s2: *const c_void, len: size_t) -> c_int;
    /// `bzero(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Fills `bytes` bytes of memory starting at the virtual address `addr`
    /// with zeros. The range must lie in kernel space; no range checking is
    /// done. It can be called from user, interrupt, or kernel context.
    pub fn bzero(addr: *mut c_void, bytes: size_t);
    /// `kcopy(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Copies `count` bytes from the kernel address `from` to the kernel
    /// address `to`, performing the same copy as [`bcopy`] but guarding
    /// against faults for its duration. A fault taken during the copy is
    /// first handled as an ordinary page fault, so a copy that touches
    /// pageable kernel memory succeeds when the fault can be resolved;
    /// one that cannot be resolved makes the function return an error
    /// where `bcopy` would panic. It returns 0 on success and otherwise an
    /// error number describing the fault, most commonly `EFAULT`, which
    /// differs from `copyin(9F)` and `copyout(9F)` returning -1. Both
    /// addresses must be kernel addresses; it must not be used for user
    /// memory, and overlapping ranges give undefined behaviour. It can be
    /// called from user or kernel context.
    pub fn kcopy(from: *const c_void, to: *mut c_void, count: size_t) -> c_int;

    // --- memchr(9F) ---------------------------------------------------------
    /// `memchr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a pointer to the first occurrence of `c`, converted to an
    /// `unsigned char`, among the first `n` bytes of the memory area `s`,
    /// each byte likewise being read as an `unsigned char`, or a null
    /// pointer if it does not occur. The header returns a `void *` when
    /// compiled as C, hence the mutable return type. It can be called from
    /// user, interrupt, or kernel context.
    pub fn memchr(s: *const c_void, c: c_int, n: size_t) -> *mut c_void;
    /// `memchr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Compares the first `n` bytes of `s1` and `s2`, each byte being read
    /// as an `unsigned char`, and returns an integer less than, equal to,
    /// or greater than zero according to whether `s1` is lexicographically
    /// less than, equal to, or greater than `s2`. It can be called from
    /// user, interrupt, or kernel context.
    pub fn memcmp(s1: *const c_void, s2: *const c_void, n: size_t) -> c_int;
    /// `memchr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies `n` bytes from the memory area `s2` to `s1` and returns `s1`.
    /// The behaviour is undefined if the two areas overlap; use
    /// [`memmove`] in that case. It does not check for overflow of the
    /// receiving area. It can be called from user, interrupt, or kernel
    /// context. The `ddi` mapfile lists it.
    pub fn memcpy(s1: *mut c_void, s2: *const c_void, n: size_t) -> *mut c_void;
    /// `memchr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies `n` bytes from the memory area `s2` to `s1` and returns `s1`.
    /// Copying between areas that overlap takes place correctly, which is
    /// the difference from [`memcpy`]; the manual notes that `memcpy` may
    /// be faster when the caller knows the areas are disjoint. It can be
    /// called from user, interrupt, or kernel context.
    pub fn memmove(s1: *mut c_void, s2: *const c_void, n: size_t) -> *mut c_void;
    /// `memchr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets the first `n` bytes of the memory area `s` to the value `c`,
    /// converted to an `unsigned char`, and returns `s`. It can be called
    /// from user, interrupt, or kernel context. The `ddi` mapfile lists it.
    pub fn memset(s: *mut c_void, c: c_int, n: size_t) -> *mut c_void;

    // --- sprintf(9F) --------------------------------------------------------
    /// `sprintf(9F)`. illumos DDI specific.
    ///
    /// Formats its arguments under the control of `format` and places the
    /// output, followed by a null byte, in consecutive bytes starting at
    /// `s`. It is the caller's responsibility to make sure enough storage
    /// is available. The format language is the kernel's, as described by
    /// the manual and shared with `cmn_err(9F)`; it supports the `d`, `o`,
    /// `x`, `u`, `c`, `s`, `p` and `%` conversions, the `%b` bit-name
    /// conversion, and the `h`, `hh`, `l` and `ll` length modifiers. It
    /// returns `s`. It can be called from user, kernel, interrupt, or
    /// high-level interrupt context.
    pub fn sprintf(s: *mut c_char, format: *const c_char, ...) -> *mut c_char;
    /// `sprintf(9F)`. illumos DDI specific.
    ///
    /// Identical to [`sprintf`] except for the added argument `n`, the
    /// size of the buffer behind `s`. If `n` is 0 nothing is written and
    /// `s` may be a null pointer. Otherwise bytes beyond the first `n - 1`
    /// are discarded rather than written and a null byte is placed after
    /// the bytes that were written. It returns the number of bytes that
    /// would have been written had `n` been large enough, not counting the
    /// terminating null, so a result of `n` or more signals truncation. It
    /// can be called from user, kernel, interrupt, or high-level interrupt
    /// context.
    pub fn snprintf(s: *mut c_char, n: size_t, format: *const c_char, ...) -> size_t;
    /// `sprintf(9F)`. illumos DDI specific.
    ///
    /// The same as [`sprintf`] except that the arguments consumed by the
    /// conversion specifications come from the variable argument list `ap`
    /// rather than from the call itself. The manual requires that `ap` be
    /// initialised with `va_start(9F)` and cleaned up with `va_end(9F)`
    /// after each traversal. It returns `s`. It can be called from user,
    /// kernel, interrupt, or high-level interrupt context.
    pub fn vsprintf(s: *mut c_char, format: *const c_char, ap: va_list) -> *mut c_char;
    /// `sprintf(9F)`. illumos DDI specific.
    ///
    /// The same as [`snprintf`] except that the arguments come from the
    /// variable argument list `ap`, subject to the same `va_start(9F)` and
    /// `va_end(9F)` rules as [`vsprintf`]. It returns the number of bytes
    /// that would have been written had `n` been large enough, not counting
    /// the terminating null. It can be called from user, kernel,
    /// interrupt, or high-level interrupt context.
    pub fn vsnprintf(s: *mut c_char, n: size_t, format: *const c_char, ap: va_list) -> size_t;

    // --- stoi(9F) -----------------------------------------------------------
    /// `stoi(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the integer value of the string of decimal digits that
    /// begins at the address the variable behind `str` points to, and
    /// advances that variable to point at the last character examined. No
    /// overflow checking is done, and only positive numbers are handled; a
    /// leading minus sign is not understood. It can be called from user,
    /// interrupt, or kernel context.
    pub fn stoi(str: *mut *mut c_char) -> c_int;
    /// `stoi(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the unsigned long `num` into a null-terminated string of
    /// decimal digits stored at `s`. No bounds checking is done; the caller
    /// must make sure the buffer is large enough for the longest possible
    /// result and its terminator. The manual describes only `stoi` under
    /// CONTEXT; `numtos` touches nothing but its arguments.
    pub fn numtos(num: ulong_t, s: *mut c_char);
}
