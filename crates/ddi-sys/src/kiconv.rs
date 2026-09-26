//! Character set conversion and Unicode helpers from `sys/kiconv.h` and
//! `sys/u8_textprep.h`, both of which `sys/sunddi.h` includes.
//!
//! The kernel offers three related services for drivers and file systems
//! that must deal with text in foreign encodings. The `kiconv` family is
//! the in-kernel counterpart of `iconv(3C)`; a conversion descriptor is
//! opened for a pair of codeset names, used on buffers with `kiconv(9F)`,
//! and closed again, while `kiconvstr(9F)` performs a one-shot conversion
//! without a descriptor. The `u8` functions validate, compare and
//! normalise UTF-8 text according to a chosen version of the Unicode
//! Standard. The `uconv` functions convert between UTF-8, UTF-16 and
//! UTF-32 with control over byte order and byte order marks.
//!
//! The descriptor type `kiconv_t` is a pointer to a structure whose fields
//! are private to the kernel, so it is declared as an opaque type here.
//! Its failure value is `(kiconv_t)-1`, a pointer with every bit set,
//! which a caller compares against rather than against NULL.

use crate::types::*;

opaque! {
    /// The structure behind a code conversion descriptor, `kiconv_data_t`.
    /// It records the conversion module handle and its operations vector,
    /// neither of which a driver examines.
    kiconv_data_t;
}

/// A code conversion descriptor, `kiconv_t`, returned by [`kiconv_open`]
/// and consumed by [`kiconv`] and [`kiconv_close`]. In C it is a pointer to
/// `kiconv_data_t`. The value `(kiconv_t)-1` signals that a descriptor
/// could not be allocated.
pub type kiconv_t = *mut kiconv_data_t;

/// A null byte in the input does not stop [`kiconvstr`]; conversion runs
/// until the whole input length has been consumed (`KICONV_IGNORE_NULL`).
pub const KICONV_IGNORE_NULL: c_int = 0x0001;
/// Illegal or incomplete input characters do not stop [`kiconvstr`] but
/// are treated as non-identical conversions (`KICONV_REPLACE_INVALID`).
pub const KICONV_REPLACE_INVALID: c_int = 0x0002;

/// [`u8_strcmp`] compares case-sensitively, which is also the default when
/// no case option is given (`U8_STRCMP_CS`).
pub const U8_STRCMP_CS: c_int = 0x00000001;
/// [`u8_strcmp`] compares the Unicode upper case forms of both strings
/// (`U8_STRCMP_CI_UPPER`).
pub const U8_STRCMP_CI_UPPER: c_int = 0x00000002;
/// [`u8_strcmp`] compares the Unicode lower case forms of both strings
/// (`U8_STRCMP_CI_LOWER`).
pub const U8_STRCMP_CI_LOWER: c_int = 0x00000004;
/// Apply canonical decomposition; a building block of the normalisation
/// form flags (`U8_CANON_DECOMP`).
pub const U8_CANON_DECOMP: c_int = 0x00000010;
/// Apply compatibility decomposition; a building block of the
/// normalisation form flags (`U8_COMPAT_DECOMP`).
pub const U8_COMPAT_DECOMP: c_int = 0x00000020;
/// Apply canonical composition after decomposition; a building block of
/// the normalisation form flags (`U8_CANON_COMP`).
pub const U8_CANON_COMP: c_int = 0x00000040;
/// [`u8_strcmp`] normalises both strings to Unicode Normalization Form D
/// before comparing (`U8_STRCMP_NFD`).
pub const U8_STRCMP_NFD: c_int = U8_CANON_DECOMP;
/// [`u8_strcmp`] normalises both strings to Unicode Normalization Form C
/// before comparing (`U8_STRCMP_NFC`).
pub const U8_STRCMP_NFC: c_int = U8_CANON_DECOMP | U8_CANON_COMP;
/// [`u8_strcmp`] normalises both strings to Unicode Normalization Form KD
/// before comparing (`U8_STRCMP_NFKD`).
pub const U8_STRCMP_NFKD: c_int = U8_COMPAT_DECOMP;
/// [`u8_strcmp`] normalises both strings to Unicode Normalization Form KC
/// before comparing (`U8_STRCMP_NFKC`).
pub const U8_STRCMP_NFKC: c_int = U8_COMPAT_DECOMP | U8_CANON_COMP;
/// [`u8_textprep_str`] maps lower case characters to upper case where a
/// mapping exists (`U8_TEXTPREP_TOUPPER`).
pub const U8_TEXTPREP_TOUPPER: c_int = U8_STRCMP_CI_UPPER;
/// [`u8_textprep_str`] maps upper case characters to lower case where a
/// mapping exists (`U8_TEXTPREP_TOLOWER`).
pub const U8_TEXTPREP_TOLOWER: c_int = U8_STRCMP_CI_LOWER;
/// [`u8_textprep_str`] applies Unicode Normalization Form D
/// (`U8_TEXTPREP_NFD`).
pub const U8_TEXTPREP_NFD: c_int = U8_STRCMP_NFD;
/// [`u8_textprep_str`] applies Unicode Normalization Form C
/// (`U8_TEXTPREP_NFC`).
pub const U8_TEXTPREP_NFC: c_int = U8_STRCMP_NFC;
/// [`u8_textprep_str`] applies Unicode Normalization Form KD
/// (`U8_TEXTPREP_NFKD`).
pub const U8_TEXTPREP_NFKD: c_int = U8_STRCMP_NFKD;
/// [`u8_textprep_str`] applies Unicode Normalization Form KC
/// (`U8_TEXTPREP_NFKC`).
pub const U8_TEXTPREP_NFKC: c_int = U8_STRCMP_NFKC;
/// A null byte in the input does not stop [`u8_textprep_str`]; preparation
/// runs until the whole input length has been consumed
/// (`U8_TEXTPREP_IGNORE_NULL`).
pub const U8_TEXTPREP_IGNORE_NULL: c_int = 0x00010000;
/// Illegal or incomplete characters do not stop [`u8_textprep_str`] but are
/// passed through as if no preparation were needed
/// (`U8_TEXTPREP_IGNORE_INVALID`).
pub const U8_TEXTPREP_IGNORE_INVALID: c_int = 0x00020000;
/// [`u8_textprep_str`] must not sleep while allocating working memory
/// (`U8_TEXTPREP_NOWAIT`). The header defines it; the manual does not
/// describe it.
pub const U8_TEXTPREP_NOWAIT: c_int = 0x00040000;
/// Use the Unicode 3.2.0 character data (`U8_UNICODE_320`). The version
/// argument of the `u8` functions is a `size_t`.
pub const U8_UNICODE_320: size_t = 0;
/// Use the Unicode 5.0.0 character data (`U8_UNICODE_500`).
pub const U8_UNICODE_500: size_t = 1;
/// Use the newest Unicode character data the kernel has, which is
/// currently 5.0.0 (`U8_UNICODE_LATEST`).
pub const U8_UNICODE_LATEST: size_t = U8_UNICODE_500;
/// [`u8_validate`] checks up to `n` bytes and possibly several characters
/// rather than stopping after the first (`U8_VALIDATE_ENTIRE`).
pub const U8_VALIDATE_ENTIRE: c_int = 0x00100000;
/// [`u8_validate`] also rejects any of the strings supplied in its `list`
/// argument (`U8_VALIDATE_CHECK_ADDITIONAL`).
pub const U8_VALIDATE_CHECK_ADDITIONAL: c_int = 0x00200000;
/// [`u8_validate`] restricts the valid coding space to U+0000 through
/// U+FFFF instead of the full range up to U+10FFFF
/// (`U8_VALIDATE_UCS2_RANGE`).
pub const U8_VALIDATE_UCS2_RANGE: c_int = 0x00400000;
/// The value the header reserves for an illegal character in internal
/// tables (`U8_ILLEGAL_CHAR`).
pub const U8_ILLEGAL_CHAR: c_int = -1;
/// The value the header reserves for a character outside the coding space
/// in internal tables (`U8_OUT_OF_RANGE_CHAR`).
pub const U8_OUT_OF_RANGE_CHAR: c_int = -2;

/// The input to a `uconv` function is in big-endian byte order
/// (`UCONV_IN_BIG_ENDIAN`).
pub const UCONV_IN_BIG_ENDIAN: c_int = 0x0001;
/// The output of a `uconv` function should be in big-endian byte order
/// (`UCONV_OUT_BIG_ENDIAN`).
pub const UCONV_OUT_BIG_ENDIAN: c_int = 0x0002;
/// The input is in the byte order of the running system
/// (`UCONV_IN_SYSTEM_ENDIAN`).
pub const UCONV_IN_SYSTEM_ENDIAN: c_int = 0x0004;
/// The output should be in the byte order of the running system
/// (`UCONV_OUT_SYSTEM_ENDIAN`).
pub const UCONV_OUT_SYSTEM_ENDIAN: c_int = 0x0008;
/// The input is in little-endian byte order (`UCONV_IN_LITTLE_ENDIAN`).
pub const UCONV_IN_LITTLE_ENDIAN: c_int = 0x0010;
/// The output should be in little-endian byte order
/// (`UCONV_OUT_LITTLE_ENDIAN`).
pub const UCONV_OUT_LITTLE_ENDIAN: c_int = 0x0020;
/// A null or U+0000 character does not stop the conversion; the whole
/// input length is consumed (`UCONV_IGNORE_NULL`).
pub const UCONV_IGNORE_NULL: c_int = 0x0040;
/// A byte order mark at the start of the input is interpreted as such and
/// dictates the byte order of what follows, instead of being treated as a
/// zero width no-break space (`UCONV_IN_ACCEPT_BOM`).
pub const UCONV_IN_ACCEPT_BOM: c_int = 0x0080;
/// The output starts with a byte order mark when it is UTF-16 or UTF-32
/// (`UCONV_OUT_EMIT_BOM`).
pub const UCONV_OUT_EMIT_BOM: c_int = 0x0100;

unsafe extern "C" {
    /// `kiconv_open(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a code conversion descriptor for converting text from the
    /// codeset named by `fromcode` to the one named by `tocode`. The
    /// supported conversions are between UTF-8 and a fixed list of legacy
    /// codesets, which the manual enumerates together with their aliases;
    /// the names are compared case-insensitively. For state-dependent
    /// encodings the descriptor starts in its initial shift state and is
    /// ready for immediate use with [`kiconv`]. It stays valid until
    /// [`kiconv_close`] releases it. If the conversion is not supported or
    /// the descriptor cannot be allocated for any other reason, the value
    /// `(kiconv_t)-1` is returned. The manual allows it to be called from
    /// user context only.
    pub fn kiconv_open(tocode: *const c_char, fromcode: *const c_char) -> kiconv_t;
    /// `kiconv(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the bytes in the buffer that the variable behind `inbuf`
    /// points to, of which `inbytesleft` remain, into the codeset the
    /// descriptor `cd` was opened for, writing the result to the buffer
    /// that the variable behind `outbuf` points to, which has
    /// `outbytesleft` bytes free. All four variables are advanced or
    /// decremented to reflect how much was consumed and produced.
    /// Conversion stops before an invalid input sequence, before an
    /// incomplete character or shift sequence at the end of the input, or
    /// before input that would overflow the output, and in those cases the
    /// variable behind `errno` is set to `EILSEQ`, `EINVAL` or `E2BIG`
    /// respectively; `EBADF` means `cd` is not an open descriptor. A call
    /// with `inbuf` null, or pointing to a null pointer, resets a
    /// state-dependent descriptor to its initial shift state and writes
    /// any reset sequence to the output if room is available. The function
    /// returns the number of non-identical conversions performed, or
    /// `(size_t)-1` on error. It can be called from user or interrupt
    /// context.
    pub fn kiconv(
        cd: kiconv_t,
        inbuf: *mut *mut c_char,
        inbytesleft: *mut size_t,
        outbuf: *mut *mut c_char,
        outbytesleft: *mut size_t,
        errno: *mut c_int,
    ) -> size_t;
    /// `kiconv_close(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the descriptor `cd` and every resource that
    /// [`kiconv_open`] allocated for it. It returns 0 on success and
    /// `EBADF` if the descriptor is invalid. It can be called from user or
    /// interrupt context.
    pub fn kiconv_close(cd: kiconv_t) -> c_int;
    /// `kiconvstr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the `inlen` bytes at `inarray` from the codeset named by
    /// `fromcode` to the one named by `tocode`, writing the result into the
    /// `outlen` bytes at `outarray`, without the caller having to open a
    /// descriptor. The two length variables are decremented to show how
    /// much remains unconverted and how much output space is left. By
    /// default a null byte in the input stops the conversion regardless of
    /// `inlen`, and an illegal or incomplete input character stops it with
    /// `errno` set to `EILSEQ` or `EINVAL`; the [`KICONV_IGNORE_NULL`] and
    /// [`KICONV_REPLACE_INVALID`] bits in `flag` change those two
    /// behaviours. Output that would overflow stops the conversion with
    /// `E2BIG`, and an unsupported conversion yields `EBADF`. The function
    /// returns the number of non-identical conversions performed, or
    /// `(size_t)-1` on error. It can be called from user or interrupt
    /// context.
    pub fn kiconvstr(
        tocode: *const c_char,
        fromcode: *const c_char,
        inarray: *mut c_char,
        inlen: *mut size_t,
        outarray: *mut c_char,
        outlen: *mut size_t,
        flag: c_int,
        errno: *mut c_int,
    ) -> size_t;

    /// `u8_validate(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Validates the UTF-8 text at `u8str`, examining no more than `n`
    /// bytes, and reports how many bytes make up the valid characters it
    /// found. By default it looks only at the first character; with
    /// [`U8_VALIDATE_ENTIRE`] in `flag` it checks up to `n` bytes across
    /// several characters. With [`U8_VALIDATE_CHECK_ADDITIONAL`] it also
    /// rejects the text if it contains any of the null-terminated strings
    /// in the null-terminated array `list`, and with
    /// [`U8_VALIDATE_UCS2_RANGE`] it restricts the coding space to
    /// U+0000 through U+FFFF. It returns 0 if `u8str` is a null pointer,
    /// the number of bytes constituting the validated characters on
    /// success, or -1 on failure, in which case the variable behind
    /// `errno` may be set to `EILSEQ` for an illegal byte, `EINVAL` for an
    /// incomplete character, `EBADF` for a match against `list`, or
    /// `ERANGE` for a character outside the coding space. The manual gives
    /// no CONTEXT section for this function.
    pub fn u8_validate(u8str: *mut c_char, n: size_t, list: *mut *mut c_char, flag: c_int, errno: *mut c_int) -> c_int;
    /// `u8_strcmp(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Compares the UTF-8 strings `s1` and `s2` byte by byte after any
    /// preprocessing that `flag` requests, looking at no more than `n`
    /// bytes, or until both terminate if `n` is 0. `flag` combines at most
    /// one of [`U8_STRCMP_CS`], [`U8_STRCMP_CI_UPPER`] and
    /// [`U8_STRCMP_CI_LOWER`] with at most one of the normalisation form
    /// flags; when both are present, case folding happens before
    /// normalisation. `unicode_version` selects the character data, one of
    /// [`U8_UNICODE_320`], [`U8_UNICODE_500`] or [`U8_UNICODE_LATEST`].
    /// The result is an integer greater than, equal to, or less than zero
    /// according to whether `s1` is greater than, equal to, or less than
    /// `s2`. If an illegal or incomplete character is met, the variable
    /// behind `errno` is set to `EILSEQ` or `EINVAL` but the comparison is
    /// still carried out on the resulting strings; `EBADF` reports
    /// conflicting options and `ERANGE` an unsupported version. It can be
    /// called from user or interrupt context.
    pub fn u8_strcmp(
        s1: *const c_char,
        s2: *const c_char,
        n: size_t,
        flag: c_int,
        unicode_version: size_t,
        errno: *mut c_int,
    ) -> c_int;
    /// `u8_textprep_str(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Prepares the `inlen` bytes of UTF-8 text at `inarray` according to
    /// `flag` and writes the result into the `outlen` bytes at `outarray`,
    /// decrementing both length variables to show what remains unconsumed
    /// and how much output space is left. `flag` combines at most one of
    /// [`U8_TEXTPREP_TOUPPER`] and [`U8_TEXTPREP_TOLOWER`] with at most
    /// one of the `U8_TEXTPREP_NF*` normalisation forms, case folding being
    /// applied first; with no option set the bytes are simply copied.
    /// [`U8_TEXTPREP_IGNORE_NULL`] lets a null byte pass instead of ending
    /// the input, and [`U8_TEXTPREP_IGNORE_INVALID`] passes illegal or
    /// incomplete characters through instead of stopping. `unicode_version`
    /// selects the character data as for [`u8_strcmp`]. Without
    /// `U8_TEXTPREP_IGNORE_INVALID` the function returns 0 on success;
    /// with it, the number of illegal or incomplete characters found. On
    /// error it returns `(size_t)-1` and sets the variable behind `errno`
    /// to `E2BIG`, `EBADF`, `EILSEQ`, `EINVAL` or `ERANGE`. It can be
    /// called from user or interrupt context.
    pub fn u8_textprep_str(
        inarray: *mut c_char,
        inlen: *mut size_t,
        outarray: *mut c_char,
        outlen: *mut size_t,
        flag: c_int,
        unicode_version: size_t,
        errno: *mut c_int,
    ) -> size_t;

    /// `uconv_u16tou32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads UTF-16 code units from `utf16str` until it meets U+0000 or
    /// has read the number of 16-bit units stored in `utf16len`, converts
    /// them to UTF-32, and stores the result at `utf32str`, which has room
    /// for the number of 32-bit units stored in `utf32len`. On success
    /// both length variables are updated to the number of units consumed
    /// and produced; on failure they are left unchanged. `flag` combines
    /// the `UCONV_IN_*` and `UCONV_OUT_*` byte order selections, the
    /// system order being the default, with [`UCONV_IGNORE_NULL`],
    /// [`UCONV_IN_ACCEPT_BOM`] and [`UCONV_OUT_EMIT_BOM`]. It returns 0 on
    /// success, or `EILSEQ` for an illegal or out of range character,
    /// `E2BIG` if the output is too small, `EINVAL` for an incomplete
    /// character at the end of the input, or `EBADF` for conflicting byte
    /// order flags. It can be called from user or interrupt context.
    pub fn uconv_u16tou32(
        utf16str: *const u16,
        utf16len: *mut size_t,
        utf32str: *mut u32,
        utf32len: *mut size_t,
        flag: c_int,
    ) -> c_int;
    /// `uconv_u16tou32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads UTF-16 code units from `utf16str` until it meets U+0000 or
    /// has read the number of 16-bit units stored in `utf16len`, converts
    /// them to UTF-8, and stores the result in the `utf8len` bytes at
    /// `utf8str`. Any byte order flag given for the UTF-8 side is
    /// ignored. The length variables, `flag` and the return values behave
    /// as described for [`uconv_u16tou32`]. It can be called from user or
    /// interrupt context.
    pub fn uconv_u16tou8(
        utf16str: *const u16,
        utf16len: *mut size_t,
        utf8str: *mut c_uchar,
        utf8len: *mut size_t,
        flag: c_int,
    ) -> c_int;
    /// `uconv_u16tou32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads UTF-32 code units from `utf32str` until it meets U+0000 or
    /// has read the number of 32-bit units stored in `utf32len`, converts
    /// them to UTF-16, and stores the result in the `utf16len` 16-bit units
    /// at `utf16str`. A character outside the Basic Multilingual Plane
    /// becomes a surrogate pair, so the output may need more units than
    /// the input. The length variables, `flag` and the return values
    /// behave as described for [`uconv_u16tou32`]. It can be called from
    /// user or interrupt context.
    pub fn uconv_u32tou16(
        utf32str: *const u32,
        utf32len: *mut size_t,
        utf16str: *mut u16,
        utf16len: *mut size_t,
        flag: c_int,
    ) -> c_int;
    /// `uconv_u16tou32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads UTF-32 code units from `utf32str` until it meets U+0000 or
    /// has read the number of 32-bit units stored in `utf32len`, converts
    /// them to UTF-8, and stores the result in the `utf8len` bytes at
    /// `utf8str`. Any byte order flag given for the UTF-8 side is ignored.
    /// The length variables, `flag` and the return values behave as
    /// described for [`uconv_u16tou32`]. It can be called from user or
    /// interrupt context.
    pub fn uconv_u32tou8(
        utf32str: *const u32,
        utf32len: *mut size_t,
        utf8str: *mut c_uchar,
        utf8len: *mut size_t,
        flag: c_int,
    ) -> c_int;
    /// `uconv_u16tou32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads UTF-8 bytes from `utf8str` until it meets a null byte or has
    /// read the number of bytes stored in `utf8len`, converts them to
    /// UTF-16, and stores the result in the `utf16len` 16-bit units at
    /// `utf16str`. Any byte order flag given for the UTF-8 side is
    /// ignored. The length variables, `flag` and the return values behave
    /// as described for [`uconv_u16tou32`]. It can be called from user or
    /// interrupt context.
    pub fn uconv_u8tou16(
        utf8str: *const c_uchar,
        utf8len: *mut size_t,
        utf16str: *mut u16,
        utf16len: *mut size_t,
        flag: c_int,
    ) -> c_int;
    /// `uconv_u16tou32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads UTF-8 bytes from `utf8str` until it meets a null byte or has
    /// read the number of bytes stored in `utf8len`, converts them to
    /// UTF-32, and stores the result in the `utf32len` 32-bit units at
    /// `utf32str`. Any byte order flag given for the UTF-8 side is
    /// ignored. The length variables, `flag` and the return values behave
    /// as described for [`uconv_u16tou32`]. It can be called from user or
    /// interrupt context.
    pub fn uconv_u8tou32(
        utf8str: *const c_uchar,
        utf8len: *mut size_t,
        utf32str: *mut u32,
        utf32len: *mut size_t,
        flag: c_int,
    ) -> c_int;
}
