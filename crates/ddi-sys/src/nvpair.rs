//! Name-value pair lists from `sys/nvpair.h`.
//!
//! An `nvlist_t` is an ordered list of named, typed values. Drivers use it
//! to describe devices to the fault management framework, to exchange
//! structured data with user space through ioctls, and to carry properties
//! between kernel subsystems. Every value in a list is an `nvpair_t` with a
//! name, a [`data_type_t`] and either a scalar, a string, a nested list or
//! an array of one of those. The kernel manages all of the memory behind a
//! list; the driver only ever holds pointers into it, and those pointers
//! stay valid until `nvlist_free(9F)` is called on the list that owns them.
//!
//! The seven section 9F pages that document the family are
//! `nvlist_alloc(9F)`, `nvlist_add_boolean(9F)`, `nvlist_lookup_boolean(9F)`,
//! `nvlist_lookup_nvpair(9F)`, `nvlist_next_nvpair(9F)`, `nvlist_remove(9F)`
//! and `nvpair_value_byte(9F)`. Each page covers a whole group of functions,
//! and the doc comment of every function below names the page that covers
//! it. All of the pages that carry an INTERFACE LEVEL section give it as
//! illumos DDI specific. The `nvlist_lookup_nvpair(9F)` page has no such
//! section and instead carries an Interface Stability attribute of
//! Committed, which is what its two functions quote.
//!
//! The `fnvlist_*` and `fnvpair_*` functions are the force wrappers from
//! `usr/src/common/nvpair/fnvpair.c`. They call the plain functions with
//! `NV_UNIQUE_NAME`, `NV_ENCODE_NATIVE` and `KM_SLEEP`, and they `VERIFY`
//! that the operation succeeded, so a failure panics the system instead of
//! returning an error. In return the lookup and value functions can hand
//! back the value directly rather than through an out pointer. These
//! functions have no section 9 manual page; they are documented in
//! `libnvpair(3LIB)`. They are compiled into `genunix` from
//! `uts/common/Makefile.files` and the gate exports them to drivers through
//! `uts/common/mapfiles/fnvlist.mapfile`, which the `ltc430x` and `pca953x`
//! drivers list in their `MAPFILES` alongside `ddi`. A driver that uses
//! them therefore needs no module dependency beyond `genunix`.
//!
//! `nvlist_t` and `nvpair_t` are declared opaque even though `sys/nvpair.h`
//! spells out their headers. Those headers describe the packed on-disk and
//! on-wire encoding, and the live in-memory list hangs off a private
//! pointer that only `nvpair.c` knows how to follow. A driver never
//! constructs or inspects either structure; it obtains them from
//! `nvlist_alloc(9F)`, `nvlist_unpack(9F)` and the iteration functions and
//! passes them straight back. Keeping them opaque means the crate cannot
//! encourage a driver to depend on a layout it has no business reading.
//!
//! `nv_alloc_t` and `nv_alloc_ops_t`, by contrast, are declared with their
//! full layout. A driver that wants the fixed-buffer allocator from the
//! `nvlist_alloc(9F)` page has to provide storage for an `nv_alloc_t`, so
//! its size is part of the interface, and a driver that supplies its own
//! allocator has to fill in an `nv_alloc_ops_t`. Both layouts are pinned by
//! constant assertions below; under LP64 an `nv_alloc_t` is 16 bytes and an
//! `nv_alloc_ops_t` is 40 bytes.
//!
//! The kernel is built without floating point, so `DATA_TYPE_DOUBLE`,
//! `nvlist_add_double`, `nvlist_lookup_double` and `nvpair_value_double`
//! exist only in the user-space library and are not declared here. The
//! `NV_ALIGN`, `NVP_*` and `NVL_*` macros read inside the opaque structures
//! and are likewise left out.

use crate::types::*;

opaque! {
    /// A list of name-value pairs, `nvlist_t`. Drivers obtain one from
    /// `nvlist_alloc(9F)` or `nvlist_unpack(9F)` and release it with
    /// `nvlist_free(9F)`. The public header shows a four-field header for
    /// the packed encoding, but the live list is reached through a private
    /// pointer inside it, so a driver gains nothing from the layout and the
    /// crate keeps it opaque.
    nvlist_t;
    /// A single name-value pair inside an `nvlist_t`, `nvpair_t`. Drivers
    /// receive one from `nvlist_next_nvpair(9F)` or `nvlist_lookup_nvpair(9F)`
    /// and read it with `nvpair_name(9F)`, `nvpair_type(9F)` and the
    /// `nvpair_value_*` functions. The name and value are stored inline
    /// after a small header, and only `nvpair.c` knows the alignment rules,
    /// so the crate keeps it opaque.
    nvpair_t;
    /// The amd64 register save area that a C `va_list` points at, `struct
    /// __va_list_tag`. The `nv_ao_init` slot of [`nv_alloc_ops_t`] receives a
    /// `__va_list`, which the compiler defines as a one-element array of this
    /// structure, and an array parameter decays to a pointer to its element.
    /// Stable Rust cannot walk a variable argument list, so the type is only
    /// ever passed through.
    __va_list_tag;
}

/// The version of the nvpair implementation stored in every packed list
/// (`NV_VERSION`).
pub const NV_VERSION: c_int = 0;

/// Pack a list with a straight copy of the native in-memory representation,
/// suitable for another consumer on the same host (`NV_ENCODE_NATIVE`).
pub const NV_ENCODE_NATIVE: c_int = 0;
/// Pack a list with XDR encoding, suitable for sending to another host
/// (`NV_ENCODE_XDR`).
pub const NV_ENCODE_XDR: c_int = 1;

/// Every pair in the list has a unique name; adding a pair removes any
/// existing pair with the same name first (`NV_UNIQUE_NAME`). Passed as the
/// `nvflag` argument of `nvlist_alloc(9F)` and persistent across packing,
/// unpacking and duplication.
pub const NV_UNIQUE_NAME: uint_t = 0x1;
/// Every combination of name and data type in the list is unique; adding a
/// pair removes any existing pair with the same name and type first
/// (`NV_UNIQUE_NAME_TYPE`). Passed as the `nvflag` argument of
/// `nvlist_alloc(9F)`.
pub const NV_UNIQUE_NAME_TYPE: uint_t = 0x2;

/// A missing pair is not an error for `nvlist_lookup_pairs(9F)`; the
/// corresponding output locations are left untouched (`NV_FLAG_NOENTOK`).
pub const NV_FLAG_NOENTOK: c_int = 0x1;

/// The type of the value in a name-value pair, `data_type_t`. Returned by
/// `nvpair_type(9F)` and passed to `nvlist_remove(9F)` and
/// `nvlist_lookup_pairs(9F)`. The C enumeration contains the value `-1`, so
/// its underlying type is a signed `int`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct data_type_t(pub c_int);
impl data_type_t {
    /// Matches any type in interfaces that accept a wildcard
    /// (`DATA_TYPE_DONTCARE`).
    pub const DATA_TYPE_DONTCARE: Self = Self(-1);
    /// The type of a pair that has not been initialised (`DATA_TYPE_UNKNOWN`).
    pub const DATA_TYPE_UNKNOWN: Self = Self(0);
    /// A boolean whose truth is the presence of the pair itself; it carries
    /// no value (`DATA_TYPE_BOOLEAN`).
    pub const DATA_TYPE_BOOLEAN: Self = Self(1);
    /// A single `uchar_t` (`DATA_TYPE_BYTE`).
    pub const DATA_TYPE_BYTE: Self = Self(2);
    /// A single `int16_t` (`DATA_TYPE_INT16`).
    pub const DATA_TYPE_INT16: Self = Self(3);
    /// A single `uint16_t` (`DATA_TYPE_UINT16`).
    pub const DATA_TYPE_UINT16: Self = Self(4);
    /// A single `int32_t` (`DATA_TYPE_INT32`).
    pub const DATA_TYPE_INT32: Self = Self(5);
    /// A single `uint32_t` (`DATA_TYPE_UINT32`).
    pub const DATA_TYPE_UINT32: Self = Self(6);
    /// A single `int64_t` (`DATA_TYPE_INT64`).
    pub const DATA_TYPE_INT64: Self = Self(7);
    /// A single `uint64_t` (`DATA_TYPE_UINT64`).
    pub const DATA_TYPE_UINT64: Self = Self(8);
    /// A NUL-terminated string (`DATA_TYPE_STRING`).
    pub const DATA_TYPE_STRING: Self = Self(9);
    /// An array of `uchar_t` (`DATA_TYPE_BYTE_ARRAY`).
    pub const DATA_TYPE_BYTE_ARRAY: Self = Self(10);
    /// An array of `int16_t` (`DATA_TYPE_INT16_ARRAY`).
    pub const DATA_TYPE_INT16_ARRAY: Self = Self(11);
    /// An array of `uint16_t` (`DATA_TYPE_UINT16_ARRAY`).
    pub const DATA_TYPE_UINT16_ARRAY: Self = Self(12);
    /// An array of `int32_t` (`DATA_TYPE_INT32_ARRAY`).
    pub const DATA_TYPE_INT32_ARRAY: Self = Self(13);
    /// An array of `uint32_t` (`DATA_TYPE_UINT32_ARRAY`).
    pub const DATA_TYPE_UINT32_ARRAY: Self = Self(14);
    /// An array of `int64_t` (`DATA_TYPE_INT64_ARRAY`).
    pub const DATA_TYPE_INT64_ARRAY: Self = Self(15);
    /// An array of `uint64_t` (`DATA_TYPE_UINT64_ARRAY`).
    pub const DATA_TYPE_UINT64_ARRAY: Self = Self(16);
    /// An array of NUL-terminated strings (`DATA_TYPE_STRING_ARRAY`).
    pub const DATA_TYPE_STRING_ARRAY: Self = Self(17);
    /// A single `hrtime_t` (`DATA_TYPE_HRTIME`).
    pub const DATA_TYPE_HRTIME: Self = Self(18);
    /// A nested `nvlist_t` (`DATA_TYPE_NVLIST`).
    pub const DATA_TYPE_NVLIST: Self = Self(19);
    /// An array of nested `nvlist_t` pointers (`DATA_TYPE_NVLIST_ARRAY`).
    pub const DATA_TYPE_NVLIST_ARRAY: Self = Self(20);
    /// A single `boolean_t` that carries an explicit true or false value
    /// (`DATA_TYPE_BOOLEAN_VALUE`).
    pub const DATA_TYPE_BOOLEAN_VALUE: Self = Self(21);
    /// A single `int8_t` (`DATA_TYPE_INT8`).
    pub const DATA_TYPE_INT8: Self = Self(22);
    /// A single `uint8_t` (`DATA_TYPE_UINT8`).
    pub const DATA_TYPE_UINT8: Self = Self(23);
    /// An array of `boolean_t` (`DATA_TYPE_BOOLEAN_ARRAY`).
    pub const DATA_TYPE_BOOLEAN_ARRAY: Self = Self(24);
    /// An array of `int8_t` (`DATA_TYPE_INT8_ARRAY`).
    pub const DATA_TYPE_INT8_ARRAY: Self = Self(25);
    /// An array of `uint8_t` (`DATA_TYPE_UINT8_ARRAY`).
    pub const DATA_TYPE_UINT8_ARRAY: Self = Self(26);
}

/// The operations of a pluggable nvlist allocator, `nv_alloc_ops_t`, as
/// described in the Creating Pluggable Allocators section of
/// `nvlist_alloc(9F)`.
///
/// A driver that supplies its own allocator fills one of these in and
/// passes it to [`nv_alloc_init`]. The `nv_ao_alloc` and `nv_ao_free` slots
/// are required; the other three may be `None`. The first argument of every
/// operation is the `nv_alloc_t` that was initialised with this vector, and
/// an operation finds its private state in that structure's `nva_arg`
/// member. Most drivers never build one of these and instead use the
/// kernel's [`nv_fixed_ops`], [`nv_alloc_sleep`] or [`nv_alloc_nosleep`].
#[repr(C)]
pub struct nv_alloc_ops_t {
    /// Called once from [`nv_alloc_init`] with the variable arguments that
    /// were passed to it. It is responsible for filling in `nva_arg`.
    /// Optional.
    pub nv_ao_init: Option<unsafe extern "C" fn(nva: *mut nv_alloc_t, valist: *mut __va_list_tag) -> c_int>,
    /// Called from [`nv_alloc_fini`] to release whatever `nv_ao_init` set
    /// up. Optional.
    pub nv_ao_fini: Option<unsafe extern "C" fn(nva: *mut nv_alloc_t)>,
    /// Allocates `sz` bytes for the framework and returns a pointer to them,
    /// or NULL if it cannot. Required.
    pub nv_ao_alloc: Option<unsafe extern "C" fn(nva: *mut nv_alloc_t, sz: size_t) -> *mut c_void>,
    /// Frees a block of `sz` bytes previously returned by `nv_ao_alloc`;
    /// `sz` must match the original request exactly. Required.
    pub nv_ao_free: Option<unsafe extern "C" fn(nva: *mut nv_alloc_t, buf: *mut c_void, sz: size_t)>,
    /// Called from [`nv_alloc_reset`] to return `nva_arg` to the state that
    /// `nv_ao_init` established. Optional.
    pub nv_ao_reset: Option<unsafe extern "C" fn(nva: *mut nv_alloc_t)>,
}

/// An allocator handle for the `nvlist_x*` functions, `nv_alloc_t`.
///
/// The structure binds an operations vector to a private argument. A driver
/// provides the storage, initialises it with [`nv_alloc_init`], passes its
/// address to [`nvlist_xalloc`] and the other `nvlist_x*` functions, and
/// tears it down with [`nv_alloc_fini`]. The example in `nvlist_alloc(9F)`
/// obtains the storage from `kmem_alloc(9F)` with `sizeof (nv_alloc_t)`,
/// which is why the layout rather than an opaque declaration is given here.
#[repr(C)]
pub struct nv_alloc_t {
    /// The operations vector this allocator uses.
    pub nva_ops: *const nv_alloc_ops_t,
    /// Private state for the operations, filled in by their `nv_ao_init`.
    pub nva_arg: *mut c_void,
}

const _: () = assert!(core::mem::size_of::<nv_alloc_t>() == 16);
const _: () = assert!(core::mem::size_of::<nv_alloc_ops_t>() == 40);

unsafe extern "C" {
    // --- allocators exported by genunix ------------------------------------

    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The operations vector of the fixed-buffer allocator, `nv_fixed_ops`.
    /// The variable is a pointer to a constant vector inside `genunix`. It
    /// is passed to [`nv_alloc_init`] together with the address and size of
    /// a caller-provided buffer, after which every allocation the framework
    /// makes on behalf of that `nv_alloc_t` is carved out of the buffer.
    /// Memory released through `nvlist_free(9F)` or pair removal is not
    /// reclaimed until [`nv_alloc_reset`] resets the buffer. Because it never
    /// calls `kmem_alloc(9F)`, the fixed-buffer allocator may be used in
    /// interrupt context. The symbol is described in the manual page but
    /// does not appear in `ddi.mapfile`.
    pub static nv_fixed_ops: *const nv_alloc_ops_t;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A ready-made allocator, `nv_alloc_sleep`, that draws from
    /// `kmem_alloc(9F)` with `KM_SLEEP`. Passing it to [`nvlist_xalloc`]
    /// gives the same behaviour as [`nvlist_alloc`] with `KM_SLEEP`, which
    /// means it may block and must not be used from interrupt context. The
    /// variable is a pointer to a structure inside `genunix` and is only
    /// declared when the header is included by kernel code. The symbol is
    /// described in the manual page but does not appear in `ddi.mapfile`.
    pub static nv_alloc_sleep: *mut nv_alloc_t;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A ready-made allocator, `nv_alloc_nosleep`, that draws from
    /// `kmem_alloc(9F)` with `KM_NOSLEEP`. Passing it to [`nvlist_xalloc`]
    /// gives the same behaviour as [`nvlist_alloc`] with `KM_NOSLEEP`, so
    /// allocations can fail with `ENOMEM` but never block, and it is safe in
    /// interrupt context. The symbol is described in the manual page but
    /// does not appear in `ddi.mapfile`.
    pub static nv_alloc_nosleep: *mut nv_alloc_t;

    // --- nvlist_alloc(9F): list management ---------------------------------

    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates a new, empty name-value pair list and stores its address
    /// through `nvlp`. The `nvflag` argument is zero, [`NV_UNIQUE_NAME`] or
    /// [`NV_UNIQUE_NAME_TYPE`] and decides whether later additions replace
    /// existing pairs with the same name, the same name and type, or never
    /// replace anything. The flag is persistent and survives packing,
    /// unpacking and duplication. `kmflag` is `KM_SLEEP` or `KM_NOSLEEP` and
    /// governs every allocation made on behalf of the list from then on.
    /// Returns 0 on success, `EINVAL` for a bad argument and `ENOMEM` if
    /// memory could not be obtained. May be called from interrupt context
    /// only with `KM_NOSLEEP`; from user or kernel context either flag is
    /// valid.
    pub fn nvlist_alloc(nvlp: *mut *mut nvlist_t, nvflag: uint_t, kmflag: c_int) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees a list and every pair, string, array and nested list inside
    /// it. The memory goes back to whichever allocator the list was created
    /// with. Passing NULL is permitted and does nothing. After this call any
    /// pointer that was obtained from the list, whether an `nvpair_t`, a
    /// string or an array, is invalid.
    pub fn nvlist_free(nvl: *mut nvlist_t);
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Computes the smallest contiguous buffer that can hold `nvl` packed
    /// with the given `encoding`, which is [`NV_ENCODE_NATIVE`] or
    /// [`NV_ENCODE_XDR`], and stores the number of bytes through `size`.
    /// Returns 0 on success or `EINVAL` for a bad argument or an unsupported
    /// encoding. It allocates nothing and may be called in any context.
    pub fn nvlist_size(nvl: *mut nvlist_t, size: *mut size_t, encoding: c_int) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Packs `nvl` into contiguous memory using `encoding`. If the pointer
    /// at `*bufp` is not NULL it is taken to be a caller-supplied buffer of
    /// `*buflen` bytes, which must be 8-byte aligned, and `kmflag` is
    /// ignored. If `*bufp` is NULL the kernel allocates a buffer with the
    /// policy in `kmflag`, stores its address through `bufp` and its length
    /// through `buflen`, and the caller later frees it with `kmem_free(9F)`
    /// using that length. Returns 0 on success, `EINVAL` for a bad argument,
    /// `ENOMEM` if memory could not be obtained, `EFAULT` for an encoding
    /// error and `ENOTSUP` for an unsupported encoding. May be called from
    /// interrupt context only with `KM_NOSLEEP`.
    pub fn nvlist_pack(
        nvl: *mut nvlist_t,
        bufp: *mut *mut c_char,
        buflen: *mut size_t,
        encoding: c_int,
        kmflag: c_int,
    ) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unpacks the `buflen` bytes at `buf`, which must hold a list packed
    /// by [`nvlist_pack`] or its equivalents, into a new searchable list and
    /// stores the list's address through `nvlp`. The encoding is detected
    /// from the packed header. Memory for the list comes from
    /// `kmem_alloc(9F)` with the policy in `kmflag`, and the caller releases
    /// it with [`nvlist_free`]. Returns 0 on success, `EINVAL`, `ENOMEM`,
    /// `EFAULT` for a decoding error or `ENOTSUP` for an unsupported
    /// encoding. May be called from interrupt context only with
    /// `KM_NOSLEEP`.
    pub fn nvlist_unpack(buf: *mut c_char, buflen: size_t, nvlp: *mut *mut nvlist_t, kmflag: c_int) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Makes a deep copy of `nvl`, including nested lists, and stores the
    /// address of the copy through `nvlp`. The copy keeps the uniqueness
    /// flag of the original and is allocated with the policy in `kmflag`.
    /// Returns 0 on success, `EINVAL` for a bad argument or `ENOMEM` if
    /// memory could not be obtained. May be called from interrupt context
    /// only with `KM_NOSLEEP`.
    pub fn nvlist_dup(nvl: *mut nvlist_t, nvlp: *mut *mut nvlist_t, kmflag: c_int) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a copy of every pair in `nvl` to `dst`. Where `dst` was created
    /// with [`NV_UNIQUE_NAME`], a pair in `dst` with the same name as an
    /// incoming pair is replaced; where it was created with
    /// [`NV_UNIQUE_NAME_TYPE`], only a pair with the same name and type is
    /// replaced; otherwise every incoming pair is appended. `kmflag` gives
    /// the allocation policy for the copies. Returns 0 on success, `EINVAL`
    /// or `ENOMEM`.
    pub fn nvlist_merge(dst: *mut nvlist_t, nvl: *mut nvlist_t, kmflag: c_int) -> c_int;
    /// `nvlist_nvflag`. Listed in `ddi.mapfile`; declared in `sys/nvpair.h` without a manual page.
    ///
    /// Returns the persistent flags of a list, that is the `nvflag` value
    /// that was given to [`nvlist_alloc`] or inherited through unpacking or
    /// duplication. The result is zero, [`NV_UNIQUE_NAME`] or
    /// [`NV_UNIQUE_NAME_TYPE`]. It reads a header field and may be called in
    /// any context.
    pub fn nvlist_nvflag(nvl: *mut nvlist_t) -> uint_t;

    // --- nvlist_alloc(9F): pluggable allocators ----------------------------

    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates a list exactly as [`nvlist_alloc`] does but obtains memory
    /// from the allocator `nva` rather than from `kmem_alloc(9F)` with a
    /// flag. The allocator is remembered and used for every later operation
    /// on the list, including pair addition and [`nvlist_free`]. The
    /// [`nv_alloc_sleep`] and [`nv_alloc_nosleep`] allocators reproduce the
    /// `KM_SLEEP` and `KM_NOSLEEP` behaviour; an allocator initialised with
    /// [`nv_fixed_ops`] carves the list out of a caller-provided buffer.
    /// Returns 0 on success, `EINVAL` or `ENOMEM`. May be called from
    /// interrupt context only if the allocator does not sleep, either
    /// because it is the default allocator with `KM_NOSLEEP` or because it
    /// uses memory that was set aside in advance.
    pub fn nvlist_xalloc(nvlp: *mut *mut nvlist_t, nvflag: uint_t, nva: *mut nv_alloc_t) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Packs `nvl` exactly as [`nvlist_pack`] does but, when `*bufp` is NULL
    /// and a buffer has to be allocated, takes it from `nva` instead of from
    /// `kmem_alloc(9F)`. A buffer obtained this way is released through the
    /// same allocator's free operation with the length stored through
    /// `buflen`. Returns 0, `EINVAL`, `ENOMEM`, `EFAULT` or `ENOTSUP`. The
    /// interrupt context rule of [`nvlist_xalloc`] applies.
    pub fn nvlist_xpack(
        nvl: *mut nvlist_t,
        bufp: *mut *mut c_char,
        buflen: *mut size_t,
        encoding: c_int,
        nva: *mut nv_alloc_t,
    ) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unpacks a packed list exactly as [`nvlist_unpack`] does but builds
    /// the new list from the allocator `nva`, which is then remembered as
    /// the list's allocator. Returns 0, `EINVAL`, `ENOMEM`, `EFAULT` or
    /// `ENOTSUP`. The interrupt context rule of [`nvlist_xalloc`] applies.
    pub fn nvlist_xunpack(buf: *mut c_char, buflen: size_t, nvlp: *mut *mut nvlist_t, nva: *mut nv_alloc_t) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Duplicates a list exactly as [`nvlist_dup`] does but builds the copy
    /// from the allocator `nva`. Returns 0 on success, `EINVAL` or
    /// `ENOMEM`. The interrupt context rule of [`nvlist_xalloc`] applies.
    pub fn nvlist_xdup(nvl: *mut nvlist_t, nvlp: *mut *mut nvlist_t, nva: *mut nv_alloc_t) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the allocator that was recorded for `nvl` when it was
    /// created, or NULL if `nvl` is NULL or has no private data. For a list
    /// made with [`nvlist_alloc`] this is one of the kernel's own
    /// allocators; for a list made with [`nvlist_xalloc`] it is the `nva`
    /// that was passed in. A driver uses it to allocate further lists or
    /// buffers from the same source.
    pub fn nvlist_lookup_nv_alloc(nvl: *mut nvlist_t) -> *mut nv_alloc_t;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Initialises the allocator handle `nva` with the operations vector
    /// `nvo`. The `nva_ops` member is set to `nvo`, `nva_arg` is cleared,
    /// and if the vector has an `nv_ao_init` operation it is called with a
    /// `va_list` of the remaining arguments and its return value becomes the
    /// return value of this function; otherwise 0 is returned. For the
    /// fixed-buffer allocator the call is `nv_alloc_init(nva, nv_fixed_ops,
    /// bufptr, sz)` where `bufptr` is a `void *` and `sz` a `size_t`. The
    /// symbol is described in the manual page but does not appear in
    /// `ddi.mapfile`.
    pub fn nv_alloc_init(nva: *mut nv_alloc_t, nvo: *const nv_alloc_ops_t, ...) -> c_int;
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the allocator's private state to what [`nv_alloc_init`]
    /// established by calling the vector's `nv_ao_reset` operation. When
    /// the vector has no such operation the call does nothing. For the
    /// fixed-buffer allocator it makes the whole buffer available again,
    /// which is the only way that allocator ever reclaims memory. The symbol
    /// is described in the manual page but does not appear in
    /// `ddi.mapfile`.
    pub fn nv_alloc_reset(nva: *mut nv_alloc_t);
    /// `nvlist_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Tears down the allocator handle by calling the vector's `nv_ao_fini`
    /// operation, if there is one. It does not free the storage of `nva`
    /// itself, nor, for the fixed-buffer allocator, the buffer that was
    /// handed to [`nv_alloc_init`]; both remain the caller's to release. The
    /// symbol is described in the manual page but does not appear in
    /// `ddi.mapfile`.
    pub fn nv_alloc_fini(nva: *mut nv_alloc_t);

    // --- nvlist_add_boolean(9F) --------------------------------------------

    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a copy of the pair `nvp`, which normally comes from iterating
    /// another list, to `nvl` under the same name, type and value. Memory
    /// follows the policy that `nvl` was created with, and the uniqueness
    /// flag of `nvl` decides whether an existing pair with the same name, or
    /// the same name and type, is removed first. Returns 0 on success,
    /// `EINVAL` or `ENOMEM`. Only one thread may modify a list at a time,
    /// and the caller provides that synchronisation. May be called from
    /// interrupt context only if the list's allocator does not sleep.
    pub fn nvlist_add_nvpair(nvl: *mut nvlist_t, nvp: *mut nvpair_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_BOOLEAN` named `name`. Such a pair has
    /// no value; its presence in the list is what means true. The manual
    /// page marks this function as deprecated in favour of
    /// [`nvlist_add_boolean_value`], but it remains exported and is still
    /// widely used. The uniqueness, return value and context rules of
    /// [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_boolean(nvl: *mut nvlist_t, name: *const c_char) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_BOOLEAN_VALUE` named `name` holding
    /// the explicit `boolean_t` value `val`, which may be `B_TRUE` or
    /// `B_FALSE`. The uniqueness, return value and context rules of
    /// [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_boolean_value(nvl: *mut nvlist_t, name: *const c_char, val: boolean_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_BYTE` named `name` holding the
    /// `uchar_t` value `val`. The uniqueness, return value and context rules
    /// of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_byte(nvl: *mut nvlist_t, name: *const c_char, val: c_uchar) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_INT8` named `name` holding the
    /// `int8_t` value `val`. The uniqueness, return value and context rules
    /// of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_int8(nvl: *mut nvlist_t, name: *const c_char, val: i8) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_UINT8` named `name` holding the
    /// `uint8_t` value `val`. The uniqueness, return value and context rules
    /// of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_uint8(nvl: *mut nvlist_t, name: *const c_char, val: u8) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_INT16` named `name` holding the
    /// `int16_t` value `val`. The uniqueness, return value and context rules
    /// of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_int16(nvl: *mut nvlist_t, name: *const c_char, val: i16) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_UINT16` named `name` holding the
    /// `uint16_t` value `val`. The uniqueness, return value and context
    /// rules of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_uint16(nvl: *mut nvlist_t, name: *const c_char, val: u16) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_INT32` named `name` holding the
    /// `int32_t` value `val`. The uniqueness, return value and context rules
    /// of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_int32(nvl: *mut nvlist_t, name: *const c_char, val: i32) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_UINT32` named `name` holding the
    /// `uint32_t` value `val`. The uniqueness, return value and context
    /// rules of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_uint32(nvl: *mut nvlist_t, name: *const c_char, val: u32) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_INT64` named `name` holding the
    /// `int64_t` value `val`. The uniqueness, return value and context rules
    /// of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_int64(nvl: *mut nvlist_t, name: *const c_char, val: i64) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_UINT64` named `name` holding the
    /// `uint64_t` value `val`. The uniqueness, return value and context
    /// rules of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_uint64(nvl: *mut nvlist_t, name: *const c_char, val: u64) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_STRING` named `name` holding a copy of
    /// the NUL-terminated string `val`. The header declares `val` as `const
    /// char *`; the manual page's `char *` is the older spelling of the
    /// same thing. The list owns the copy, so the caller's string need not
    /// outlive the call. The uniqueness, return value and context rules of
    /// [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_string(nvl: *mut nvlist_t, name: *const c_char, val: *const c_char) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_NVLIST` named `name` holding a deep
    /// copy of the list `val`. The caller keeps ownership of `val` and may
    /// free it afterwards. The uniqueness, return value and context rules of
    /// [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_nvlist(nvl: *mut nvlist_t, name: *const c_char, val: *mut nvlist_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_BOOLEAN_ARRAY` named `name` holding a
    /// copy of the `nelem` `boolean_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_boolean_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut boolean_t,
        nelem: uint_t,
    ) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_BYTE_ARRAY` named `name` holding a
    /// copy of the `nelem` `uchar_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_byte_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut c_uchar, nelem: uint_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_INT8_ARRAY` named `name` holding a
    /// copy of the `nelem` `int8_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_int8_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut i8, nelem: uint_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_UINT8_ARRAY` named `name` holding a
    /// copy of the `nelem` `uint8_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_uint8_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut u8, nelem: uint_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_INT16_ARRAY` named `name` holding a
    /// copy of the `nelem` `int16_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_int16_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut i16, nelem: uint_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_UINT16_ARRAY` named `name` holding a
    /// copy of the `nelem` `uint16_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_uint16_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut u16, nelem: uint_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_INT32_ARRAY` named `name` holding a
    /// copy of the `nelem` `int32_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_int32_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut i32, nelem: uint_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_UINT32_ARRAY` named `name` holding a
    /// copy of the `nelem` `uint32_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_uint32_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut u32, nelem: uint_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_INT64_ARRAY` named `name` holding a
    /// copy of the `nelem` `int64_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_int64_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut i64, nelem: uint_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_UINT64_ARRAY` named `name` holding a
    /// copy of the `nelem` `uint64_t` values starting at `val`. The
    /// uniqueness, return value and context rules of [`nvlist_add_nvpair`]
    /// apply.
    pub fn nvlist_add_uint64_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut u64, nelem: uint_t) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_STRING_ARRAY` named `name` holding
    /// copies of the `nelem` NUL-terminated strings whose pointers start at
    /// `val`. The header declares `val` as `char *const *`, a pointer to
    /// constant pointers to mutable characters, and that is what the
    /// binding takes. The uniqueness, return value and context rules of
    /// [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_string_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *const *mut c_char,
        nelem: uint_t,
    ) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_NVLIST_ARRAY` named `name` holding
    /// deep copies of the `nelem` lists whose pointers start at `val`. The
    /// caller keeps ownership of the source lists. The uniqueness, return
    /// value and context rules of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_nvlist_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut nvlist_t,
        nelem: uint_t,
    ) -> c_int;
    /// `nvlist_add_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Adds a pair of type `DATA_TYPE_HRTIME` named `name` holding the
    /// `hrtime_t` value `val`. The function is not listed in the manual
    /// page's NAME line but is declared in the header alongside the other
    /// scalar additions and exported through `ddi.mapfile`. The uniqueness,
    /// return value and context rules of [`nvlist_add_nvpair`] apply.
    pub fn nvlist_add_hrtime(nvl: *mut nvlist_t, name: *const c_char, val: hrtime_t) -> c_int;

    // --- nvlist_remove(9F) -------------------------------------------------

    /// `nvlist_remove(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the first pair in `nvl` whose name is `name` and whose type
    /// is `type_`. The pair's memory goes back to the list's allocator and
    /// any pointer previously obtained from that pair becomes invalid.
    /// Returns 0 on success, `EINVAL` for a bad argument or `ENOENT` if no
    /// pair matched. Only one thread may modify a list at a time. May be
    /// called from user, interrupt or kernel context.
    pub fn nvlist_remove(nvl: *mut nvlist_t, name: *const c_char, type_: data_type_t) -> c_int;
    /// `nvlist_remove(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes every pair in `nvl` whose name is `name`, whatever its type.
    /// Returns 0 on success, `EINVAL` for a bad argument or `ENOENT` if no
    /// pair matched. Only one thread may modify a list at a time. May be
    /// called from user, interrupt or kernel context.
    pub fn nvlist_remove_all(nvl: *mut nvlist_t, name: *const c_char) -> c_int;
    /// `nvlist_remove_nvpair`. Listed in `ddi.mapfile`; declared in `sys/nvpair.h` without a manual page.
    ///
    /// Removes the specific pair `nvp`, which must have been obtained from
    /// `nvl` by iteration or lookup, unlinks it from the list and frees it.
    /// It is the natural companion of [`nvlist_next_nvpair`] when a driver
    /// filters a list in place, but the caller must fetch the next pair
    /// before removing the current one. Returns 0 on success or `EINVAL` if
    /// either argument is NULL or `nvp` is not in `nvl`.
    pub fn nvlist_remove_nvpair(nvl: *mut nvlist_t, nvp: *mut nvpair_t) -> c_int;

    // --- nvlist_lookup_boolean(9F) -----------------------------------------

    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether `nvl` contains a pair of type `DATA_TYPE_BOOLEAN`
    /// named `name`. There is no value to return, so success alone means
    /// the boolean is true. Returns 0 if the pair exists, `ENOENT` if it
    /// does not, `EINVAL` for a bad argument, and `ENOTSUP` if the list was
    /// not created with [`NV_UNIQUE_NAME`] or [`NV_UNIQUE_NAME_TYPE`], since
    /// such a list may hold several pairs with the same name. Many readers
    /// may search a list at once as long as nobody modifies it. May be
    /// called from user, interrupt or kernel context.
    pub fn nvlist_lookup_boolean(nvl: *mut nvlist_t, name: *const c_char) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_BOOLEAN_VALUE` named `name` and
    /// stores its `boolean_t` value through `val`. The return values and
    /// context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_boolean_value(nvl: *mut nvlist_t, name: *const c_char, val: *mut boolean_t) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_BYTE` named `name` and stores its
    /// `uchar_t` value through `val`. The return values and context rules of
    /// [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_byte(nvl: *mut nvlist_t, name: *const c_char, val: *mut c_uchar) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_INT8` named `name` and stores its
    /// `int8_t` value through `val`. The return values and context rules of
    /// [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_int8(nvl: *mut nvlist_t, name: *const c_char, val: *mut i8) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_UINT8` named `name` and stores its
    /// `uint8_t` value through `val`. The return values and context rules of
    /// [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_uint8(nvl: *mut nvlist_t, name: *const c_char, val: *mut u8) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_INT16` named `name` and stores its
    /// `int16_t` value through `val`. The return values and context rules of
    /// [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_int16(nvl: *mut nvlist_t, name: *const c_char, val: *mut i16) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_UINT16` named `name` and stores
    /// its `uint16_t` value through `val`. The return values and context
    /// rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_uint16(nvl: *mut nvlist_t, name: *const c_char, val: *mut u16) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_INT32` named `name` and stores its
    /// `int32_t` value through `val`. The return values and context rules of
    /// [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_int32(nvl: *mut nvlist_t, name: *const c_char, val: *mut i32) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_UINT32` named `name` and stores
    /// its `uint32_t` value through `val`. The return values and context
    /// rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_uint32(nvl: *mut nvlist_t, name: *const c_char, val: *mut u32) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_INT64` named `name` and stores its
    /// `int64_t` value through `val`. The return values and context rules of
    /// [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_int64(nvl: *mut nvlist_t, name: *const c_char, val: *mut i64) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_UINT64` named `name` and stores
    /// its `uint64_t` value through `val`. The return values and context
    /// rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_uint64(nvl: *mut nvlist_t, name: *const c_char, val: *mut u64) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_STRING` named `name` and stores a
    /// pointer to its NUL-terminated string through `val`. The string is
    /// owned by the list and stays valid until [`nvlist_free`] is called on
    /// `nvl` or the pair is removed or replaced. The return values and
    /// context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_string(nvl: *mut nvlist_t, name: *const c_char, val: *mut *mut c_char) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_NVLIST` named `name` and stores a
    /// pointer to the nested list through `val`. The nested list is owned
    /// by `nvl`, must not be freed separately, and stays valid until
    /// [`nvlist_free`] is called on `nvl`. The return values and context
    /// rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_nvlist(nvl: *mut nvlist_t, name: *const c_char, val: *mut *mut nvlist_t) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_BOOLEAN_ARRAY` named `name`,
    /// stores a pointer to the first `boolean_t` element through `val` and
    /// the element count through `nelem`, which is a `uint_t`. The array is
    /// owned by the list and stays valid until [`nvlist_free`] is called on
    /// `nvl`. The return values and context rules of
    /// [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_boolean_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut boolean_t,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_BYTE_ARRAY` named `name`, stores a
    /// pointer to the first `uchar_t` element through `val` and the element
    /// count through `nelem`. The array is owned by the list. The return
    /// values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_byte_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut c_uchar,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_INT8_ARRAY` named `name`, stores a
    /// pointer to the first `int8_t` element through `val` and the element
    /// count through `nelem`. The array is owned by the list. The return
    /// values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_int8_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut i8,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_UINT8_ARRAY` named `name`, stores
    /// a pointer to the first `uint8_t` element through `val` and the
    /// element count through `nelem`. The array is owned by the list. The
    /// return values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_uint8_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut u8,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_INT16_ARRAY` named `name`, stores
    /// a pointer to the first `int16_t` element through `val` and the
    /// element count through `nelem`. The array is owned by the list. The
    /// return values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_int16_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut i16,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_UINT16_ARRAY` named `name`, stores
    /// a pointer to the first `uint16_t` element through `val` and the
    /// element count through `nelem`. The array is owned by the list. The
    /// return values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_uint16_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut u16,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_INT32_ARRAY` named `name`, stores
    /// a pointer to the first `int32_t` element through `val` and the
    /// element count through `nelem`. The array is owned by the list. The
    /// return values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_int32_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut i32,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_UINT32_ARRAY` named `name`, stores
    /// a pointer to the first `uint32_t` element through `val` and the
    /// element count through `nelem`. The array is owned by the list. The
    /// return values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_uint32_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut u32,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_INT64_ARRAY` named `name`, stores
    /// a pointer to the first `int64_t` element through `val` and the
    /// element count through `nelem`. The array is owned by the list. The
    /// return values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_int64_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut i64,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_UINT64_ARRAY` named `name`, stores
    /// a pointer to the first `uint64_t` element through `val` and the
    /// element count through `nelem`. The array is owned by the list. The
    /// return values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_uint64_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut u64,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_STRING_ARRAY` named `name`, stores
    /// a pointer to an array of `char *` string pointers through `val` and
    /// the number of strings through `nelem`. Both the pointer array and
    /// the strings are owned by the list and stay valid until
    /// [`nvlist_free`] is called on `nvl`. The return values and context
    /// rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_string_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut *mut c_char,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_NVLIST_ARRAY` named `name`, stores
    /// a pointer to an array of `nvlist_t *` through `val` and the number
    /// of lists through `nelem`. The nested lists are owned by `nvl` and
    /// must not be freed separately. The return values and context rules of
    /// [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_nvlist_array(
        nvl: *mut nvlist_t,
        name: *const c_char,
        val: *mut *mut *mut nvlist_t,
        nelem: *mut uint_t,
    ) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Finds the pair of type `DATA_TYPE_HRTIME` named `name` and stores
    /// its `hrtime_t` value through `val`. The function is not listed in the
    /// manual page's NAME line but is declared in the header alongside the
    /// other scalar lookups and exported through `ddi.mapfile`. The return
    /// values and context rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_hrtime(nvl: *mut nvlist_t, name: *const c_char, val: *mut hrtime_t) -> c_int;
    /// `nvlist_lookup_boolean(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Looks up several pairs in one call. After `flag` comes a variable
    /// argument list terminated by a NULL name. Each entry is a name, a
    /// [`data_type_t`], and then, for every type other than
    /// `DATA_TYPE_BOOLEAN`, the address at which to store the value, and,
    /// for array types only, the address of a `uint_t` to receive the
    /// element count. If `flag` contains [`NV_FLAG_NOENTOK`] a missing pair
    /// is not an error and its output locations are left untouched;
    /// otherwise the first missing pair makes the call fail with `ENOENT`.
    /// Returns 0 on success, `EINVAL`, `ENOENT` or `ENOTSUP`. The context
    /// rules of [`nvlist_lookup_boolean`] apply.
    pub fn nvlist_lookup_pairs(nvl: *mut nvlist_t, flag: c_int, ...) -> c_int;

    // --- nvlist_lookup_nvpair(9F) ------------------------------------------

    /// `nvlist_lookup_nvpair(9F)`. Interface stability: Committed.
    ///
    /// Finds the pair named `name` regardless of its type and stores a
    /// pointer to it through `nvp`. The pair is owned by the list and stays
    /// valid until it is removed, replaced or the list is freed. The
    /// function works only on lists created with [`NV_UNIQUE_NAME`], since
    /// without that flag a name may not identify a single pair. Returns 0
    /// on success, `EINVAL` for a bad argument, `ENOENT` if there is no such
    /// pair and `ENOTSUP` if the list was not created with
    /// [`NV_UNIQUE_NAME`]. The manual page has no INTERFACE LEVEL section;
    /// its ATTRIBUTES table gives an Interface Stability of Committed, which
    /// is what the first line quotes. The page states no context
    /// restriction and the function allocates nothing.
    pub fn nvlist_lookup_nvpair(nvl: *mut nvlist_t, name: *const c_char, nvp: *mut *mut nvpair_t) -> c_int;
    /// `nvlist_lookup_nvpair(9F)`. Interface stability: Committed.
    ///
    /// Reports whether any pair named `name` exists in `nvl`, whatever its
    /// type. Unlike [`nvlist_lookup_nvpair`] it works on every kind of
    /// list, because it only has to find one match. Returns `B_TRUE` if a
    /// pair exists and `B_FALSE` otherwise, including when either argument
    /// is NULL. The manual page has no INTERFACE LEVEL section; its
    /// ATTRIBUTES table gives an Interface Stability of Committed. The
    /// symbol is described in the manual page but does not appear in
    /// `ddi.mapfile`.
    pub fn nvlist_exists(nvl: *mut nvlist_t, name: *const c_char) -> boolean_t;

    // --- nvlist_next_nvpair(9F) --------------------------------------------

    /// `nvlist_next_nvpair(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Walks the list in insertion order. When `nvpair` is NULL the first
    /// pair is returned; otherwise the pair after `nvpair` is returned, and
    /// NULL means `nvpair` was the last one. A driver loops from NULL until
    /// NULL comes back, reading each pair with [`nvpair_name`],
    /// [`nvpair_type`] and the `nvpair_value_*` functions. A pair that has
    /// been removed from the list, or replaced by an addition on a list
    /// created with [`NV_UNIQUE_NAME`] or [`NV_UNIQUE_NAME_TYPE`], must not
    /// be passed back in. May be called from user, interrupt or kernel
    /// context.
    pub fn nvlist_next_nvpair(nvl: *mut nvlist_t, nvpair: *mut nvpair_t) -> *mut nvpair_t;
    /// `nvlist_prev_nvpair`. Listed in `ddi.mapfile`; declared in `sys/nvpair.h` without a manual page.
    ///
    /// Walks the list backwards. When `nvpair` is NULL the last pair is
    /// returned; otherwise the pair before `nvpair` is returned, and NULL
    /// means `nvpair` was the first one or is not a member of `nvl`. It is
    /// the mirror image of [`nvlist_next_nvpair`] and follows the same rule
    /// that a removed or replaced pair must not be passed back in.
    pub fn nvlist_prev_nvpair(nvl: *mut nvlist_t, nvpair: *mut nvpair_t) -> *mut nvpair_t;
    /// `nvlist_next_nvpair(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the NUL-terminated name of a pair. The string lives inside
    /// the pair and stays valid until the pair is removed, replaced or its
    /// list is freed. May be called from user, interrupt or kernel context.
    pub fn nvpair_name(nvpair: *mut nvpair_t) -> *mut c_char;
    /// `nvlist_next_nvpair(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the [`data_type_t`] of a pair's value. A driver walking a
    /// list of unknown shape uses the result to choose which
    /// `nvpair_value_*` function to call. May be called from user, interrupt
    /// or kernel context.
    pub fn nvpair_type(nvpair: *mut nvpair_t) -> data_type_t;
    /// `nvpair_type_is_array`. Listed in `ddi.mapfile`; declared in `sys/nvpair.h` without a manual page.
    ///
    /// Returns 1 if the pair's type is one of the twelve array types, from
    /// `DATA_TYPE_BYTE_ARRAY` through `DATA_TYPE_NVLIST_ARRAY`, and 0 for a
    /// scalar, a string, a nested list or a boolean. It only inspects the
    /// pair header and may be called in any context.
    pub fn nvpair_type_is_array(nvpair: *mut nvpair_t) -> c_int;

    // --- nvpair_value_byte(9F) ---------------------------------------------

    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `boolean_t` of a pair of type `DATA_TYPE_BOOLEAN_VALUE`
    /// through `val`. There is no `nvpair_value_boolean`, because a pair of
    /// type `DATA_TYPE_BOOLEAN` has no value and its presence alone means
    /// true. Returns 0 on success or `EINVAL` if an argument is NULL or the
    /// pair's type does not match. A pair that has been removed from or
    /// replaced in its list must not be read. May be called from user,
    /// interrupt or kernel context.
    pub fn nvpair_value_boolean_value(nvpair: *mut nvpair_t, val: *mut boolean_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `uchar_t` of a pair of type `DATA_TYPE_BYTE` through
    /// `val`. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_byte(nvpair: *mut nvpair_t, val: *mut c_uchar) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `int8_t` of a pair of type `DATA_TYPE_INT8` through
    /// `val`. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_int8(nvpair: *mut nvpair_t, val: *mut i8) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `uint8_t` of a pair of type `DATA_TYPE_UINT8` through
    /// `val`. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_uint8(nvpair: *mut nvpair_t, val: *mut u8) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `int16_t` of a pair of type `DATA_TYPE_INT16` through
    /// `val`. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_int16(nvpair: *mut nvpair_t, val: *mut i16) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `uint16_t` of a pair of type `DATA_TYPE_UINT16` through
    /// `val`. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_uint16(nvpair: *mut nvpair_t, val: *mut u16) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `int32_t` of a pair of type `DATA_TYPE_INT32` through
    /// `val`. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_int32(nvpair: *mut nvpair_t, val: *mut i32) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `uint32_t` of a pair of type `DATA_TYPE_UINT32` through
    /// `val`. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_uint32(nvpair: *mut nvpair_t, val: *mut u32) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `int64_t` of a pair of type `DATA_TYPE_INT64` through
    /// `val`. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_int64(nvpair: *mut nvpair_t, val: *mut i64) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `uint64_t` of a pair of type `DATA_TYPE_UINT64` through
    /// `val`. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_uint64(nvpair: *mut nvpair_t, val: *mut u64) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the NUL-terminated string of a pair of type
    /// `DATA_TYPE_STRING` through `val`. The string is owned by the list
    /// and stays valid until [`nvlist_free`] is called on the list the pair
    /// came from. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_string(nvpair: *mut nvpair_t, val: *mut *mut c_char) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the nested list of a pair of type
    /// `DATA_TYPE_NVLIST` through `val`. The nested list is owned by the
    /// outer list and must not be freed separately. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_nvlist(nvpair: *mut nvpair_t, val: *mut *mut nvlist_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `boolean_t` of a pair of type
    /// `DATA_TYPE_BOOLEAN_ARRAY` through `val` and the element count
    /// through `nelem`, which is a `uint_t`. The array is owned by the list
    /// and stays valid until [`nvlist_free`] is called on the list the pair
    /// came from. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_boolean_array(nvpair: *mut nvpair_t, val: *mut *mut boolean_t, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `uchar_t` of a pair of type
    /// `DATA_TYPE_BYTE_ARRAY` through `val` and the element count through
    /// `nelem`. The array is owned by the list. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_byte_array(nvpair: *mut nvpair_t, val: *mut *mut c_uchar, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `int8_t` of a pair of type
    /// `DATA_TYPE_INT8_ARRAY` through `val` and the element count through
    /// `nelem`. The array is owned by the list. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_int8_array(nvpair: *mut nvpair_t, val: *mut *mut i8, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `uint8_t` of a pair of type
    /// `DATA_TYPE_UINT8_ARRAY` through `val` and the element count through
    /// `nelem`. The array is owned by the list. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_uint8_array(nvpair: *mut nvpair_t, val: *mut *mut u8, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `int16_t` of a pair of type
    /// `DATA_TYPE_INT16_ARRAY` through `val` and the element count through
    /// `nelem`. The array is owned by the list. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_int16_array(nvpair: *mut nvpair_t, val: *mut *mut i16, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `uint16_t` of a pair of type
    /// `DATA_TYPE_UINT16_ARRAY` through `val` and the element count through
    /// `nelem`. The array is owned by the list. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_uint16_array(nvpair: *mut nvpair_t, val: *mut *mut u16, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `int32_t` of a pair of type
    /// `DATA_TYPE_INT32_ARRAY` through `val` and the element count through
    /// `nelem`. The array is owned by the list. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_int32_array(nvpair: *mut nvpair_t, val: *mut *mut i32, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `uint32_t` of a pair of type
    /// `DATA_TYPE_UINT32_ARRAY` through `val` and the element count through
    /// `nelem`. The array is owned by the list. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_uint32_array(nvpair: *mut nvpair_t, val: *mut *mut u32, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `int64_t` of a pair of type
    /// `DATA_TYPE_INT64_ARRAY` through `val` and the element count through
    /// `nelem`. The array is owned by the list. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_int64_array(nvpair: *mut nvpair_t, val: *mut *mut i64, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to the first `uint64_t` of a pair of type
    /// `DATA_TYPE_UINT64_ARRAY` through `val` and the element count through
    /// `nelem`. The array is owned by the list. The return values and
    /// context rules of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_uint64_array(nvpair: *mut nvpair_t, val: *mut *mut u64, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to an array of `char *` string pointers of a pair
    /// of type `DATA_TYPE_STRING_ARRAY` through `val` and the number of
    /// strings through `nelem`. The pointer array and the strings are owned
    /// by the list. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_string_array(nvpair: *mut nvpair_t, val: *mut *mut *mut c_char, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores a pointer to an array of `nvlist_t *` of a pair of type
    /// `DATA_TYPE_NVLIST_ARRAY` through `val` and the number of lists
    /// through `nelem`. The nested lists are owned by the outer list and
    /// must not be freed separately. The return values and context rules of
    /// [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_nvlist_array(nvpair: *mut nvpair_t, val: *mut *mut *mut nvlist_t, nelem: *mut uint_t) -> c_int;
    /// `nvpair_value_byte(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the `hrtime_t` of a pair of type `DATA_TYPE_HRTIME` through
    /// `val`. The function is not listed in the manual page's NAME line but
    /// is declared in the header alongside the other scalar accessors and
    /// exported through `ddi.mapfile`. The return values and context rules
    /// of [`nvpair_value_boolean_value`] apply.
    pub fn nvpair_value_hrtime(nvpair: *mut nvpair_t, val: *mut hrtime_t) -> c_int;

    // --- fnvlist.mapfile: force wrappers, list management ------------------

    /// `fnvlist_alloc`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Allocates a list with [`NV_UNIQUE_NAME`] and `KM_SLEEP` and returns
    /// it. The underlying [`nvlist_alloc`] cannot fail with those
    /// arguments, and the wrapper verifies that it did not. Because the
    /// allocation may block it must be called from kernel thread context,
    /// never from an interrupt handler.
    pub fn fnvlist_alloc() -> *mut nvlist_t;
    /// `fnvlist_free`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Frees a list. It simply forwards to [`nvlist_free`] and, like it,
    /// accepts NULL. It exists so that code using the force wrappers reads
    /// uniformly.
    pub fn fnvlist_free(nvl: *mut nvlist_t);
    /// `fnvlist_size`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the number of bytes needed to pack `nvl` with
    /// [`NV_ENCODE_NATIVE`]. The wrapper verifies that [`nvlist_size`]
    /// succeeded, so an invalid list panics rather than returning an
    /// error.
    pub fn fnvlist_size(nvl: *mut nvlist_t) -> size_t;
    /// `fnvlist_pack`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Packs `nvl` with [`NV_ENCODE_NATIVE`] into a buffer that the kernel
    /// allocates with `KM_SLEEP`, stores the buffer's length through
    /// `sizep` and returns its address. The caller releases the buffer with
    /// [`fnvlist_pack_free`] and that same length. The wrapper verifies that
    /// [`nvlist_pack`] succeeded. Must be called from kernel thread context
    /// because the allocation may block.
    pub fn fnvlist_pack(nvl: *mut nvlist_t, sizep: *mut size_t) -> *mut c_char;
    /// `fnvlist_pack_free`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Frees a buffer returned by [`fnvlist_pack`]. In the kernel it is a
    /// call to `kmem_free(9F)` with the `pack` address and the `size` that
    /// [`fnvlist_pack`] reported, so the two must match exactly.
    pub fn fnvlist_pack_free(pack: *mut c_char, size: size_t);
    /// `fnvlist_unpack`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Unpacks the `buflen` bytes at `buf` into a new list allocated with
    /// `KM_SLEEP` and returns it. The wrapper verifies that
    /// [`nvlist_unpack`] succeeded, so a malformed buffer panics the system;
    /// a driver that receives packed data from user space should use
    /// [`nvlist_unpack`] and check the result instead. Must be called from
    /// kernel thread context.
    pub fn fnvlist_unpack(buf: *mut c_char, buflen: size_t) -> *mut nvlist_t;
    /// `fnvlist_dup`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns a deep copy of `nvl` allocated with `KM_SLEEP`. The wrapper
    /// verifies that [`nvlist_dup`] succeeded. Must be called from kernel
    /// thread context.
    pub fn fnvlist_dup(nvl: *mut nvlist_t) -> *mut nvlist_t;
    /// `fnvlist_merge`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds copies of every pair in `src` to `dst` with `KM_SLEEP`,
    /// replacing existing pairs according to the uniqueness flag of `dst`
    /// as [`nvlist_merge`] describes. The wrapper verifies that the merge
    /// succeeded. Must be called from kernel thread context.
    pub fn fnvlist_merge(dst: *mut nvlist_t, src: *mut nvlist_t);
    /// `fnvlist_num_pairs`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the number of pairs in `nvl` by walking it with
    /// [`nvlist_next_nvpair`]. It allocates nothing and may be called in
    /// any context in which the list may be read.
    pub fn fnvlist_num_pairs(nvl: *mut nvlist_t) -> size_t;

    // --- fnvlist.mapfile: force wrappers, addition -------------------------

    /// `fnvlist_add_boolean`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_BOOLEAN` named `name`, whose
    /// presence alone means true. The wrapper verifies that
    /// [`nvlist_add_boolean`] succeeded. Because the list's allocation
    /// policy is followed, and a list from [`fnvlist_alloc`] uses
    /// `KM_SLEEP`, it must in that case be called from kernel thread
    /// context. The same rule applies to every `fnvlist_add_*` function.
    pub fn fnvlist_add_boolean(nvl: *mut nvlist_t, name: *const c_char);
    /// `fnvlist_add_boolean_value`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_BOOLEAN_VALUE` named `name` holding
    /// `val`. The wrapper verifies that [`nvlist_add_boolean_value`]
    /// succeeded. The context rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_boolean_value(nvl: *mut nvlist_t, name: *const c_char, val: boolean_t);
    /// `fnvlist_add_byte`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_BYTE` named `name` holding the
    /// `uchar_t` value `val`. The wrapper verifies that [`nvlist_add_byte`]
    /// succeeded. The context rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_byte(nvl: *mut nvlist_t, name: *const c_char, val: c_uchar);
    /// `fnvlist_add_int8`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_INT8` named `name` holding `val`. The
    /// wrapper verifies that [`nvlist_add_int8`] succeeded. The context rule
    /// of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_int8(nvl: *mut nvlist_t, name: *const c_char, val: i8);
    /// `fnvlist_add_uint8`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_UINT8` named `name` holding `val`.
    /// The wrapper verifies that [`nvlist_add_uint8`] succeeded. The context
    /// rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_uint8(nvl: *mut nvlist_t, name: *const c_char, val: u8);
    /// `fnvlist_add_int16`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_INT16` named `name` holding `val`.
    /// The wrapper verifies that [`nvlist_add_int16`] succeeded. The context
    /// rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_int16(nvl: *mut nvlist_t, name: *const c_char, val: i16);
    /// `fnvlist_add_uint16`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_UINT16` named `name` holding `val`.
    /// The wrapper verifies that [`nvlist_add_uint16`] succeeded. The
    /// context rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_uint16(nvl: *mut nvlist_t, name: *const c_char, val: u16);
    /// `fnvlist_add_int32`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_INT32` named `name` holding `val`.
    /// The wrapper verifies that [`nvlist_add_int32`] succeeded. The context
    /// rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_int32(nvl: *mut nvlist_t, name: *const c_char, val: i32);
    /// `fnvlist_add_uint32`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_UINT32` named `name` holding `val`.
    /// The wrapper verifies that [`nvlist_add_uint32`] succeeded. The
    /// context rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_uint32(nvl: *mut nvlist_t, name: *const c_char, val: u32);
    /// `fnvlist_add_int64`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_INT64` named `name` holding `val`.
    /// The wrapper verifies that [`nvlist_add_int64`] succeeded. The context
    /// rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_int64(nvl: *mut nvlist_t, name: *const c_char, val: i64);
    /// `fnvlist_add_uint64`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_UINT64` named `name` holding `val`.
    /// The wrapper verifies that [`nvlist_add_uint64`] succeeded. The
    /// context rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_uint64(nvl: *mut nvlist_t, name: *const c_char, val: u64);
    /// `fnvlist_add_string`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_STRING` named `name` holding a copy
    /// of the NUL-terminated string `val`. The wrapper verifies that
    /// [`nvlist_add_string`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_string(nvl: *mut nvlist_t, name: *const c_char, val: *const c_char);
    /// `fnvlist_add_nvlist`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_NVLIST` named `name` holding a deep
    /// copy of the list `val`, which the caller still owns afterwards. The
    /// wrapper verifies that [`nvlist_add_nvlist`] succeeded. The context
    /// rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_nvlist(nvl: *mut nvlist_t, name: *const c_char, val: *mut nvlist_t);
    /// `fnvlist_add_nvpair`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a copy of the pair `pair`, taken from another list, to `nvl`.
    /// The wrapper verifies that [`nvlist_add_nvpair`] succeeded. The
    /// context rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_nvpair(nvl: *mut nvlist_t, pair: *mut nvpair_t);
    /// `fnvlist_add_boolean_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_BOOLEAN_ARRAY` named `name` holding a
    /// copy of the `n` `boolean_t` values at `val`. The wrapper verifies
    /// that [`nvlist_add_boolean_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_boolean_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut boolean_t, n: uint_t);
    /// `fnvlist_add_byte_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_BYTE_ARRAY` named `name` holding a
    /// copy of the `n` `uchar_t` values at `val`. The wrapper verifies that
    /// [`nvlist_add_byte_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_byte_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut c_uchar, n: uint_t);
    /// `fnvlist_add_int8_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_INT8_ARRAY` named `name` holding a
    /// copy of the `n` `int8_t` values at `val`. The wrapper verifies that
    /// [`nvlist_add_int8_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_int8_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut i8, n: uint_t);
    /// `fnvlist_add_uint8_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_UINT8_ARRAY` named `name` holding a
    /// copy of the `n` `uint8_t` values at `val`. The wrapper verifies that
    /// [`nvlist_add_uint8_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_uint8_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut u8, n: uint_t);
    /// `fnvlist_add_int16_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_INT16_ARRAY` named `name` holding a
    /// copy of the `n` `int16_t` values at `val`. The wrapper verifies that
    /// [`nvlist_add_int16_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_int16_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut i16, n: uint_t);
    /// `fnvlist_add_uint16_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_UINT16_ARRAY` named `name` holding a
    /// copy of the `n` `uint16_t` values at `val`. The wrapper verifies
    /// that [`nvlist_add_uint16_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_uint16_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut u16, n: uint_t);
    /// `fnvlist_add_int32_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_INT32_ARRAY` named `name` holding a
    /// copy of the `n` `int32_t` values at `val`. The wrapper verifies that
    /// [`nvlist_add_int32_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_int32_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut i32, n: uint_t);
    /// `fnvlist_add_uint32_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_UINT32_ARRAY` named `name` holding a
    /// copy of the `n` `uint32_t` values at `val`. The wrapper verifies
    /// that [`nvlist_add_uint32_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_uint32_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut u32, n: uint_t);
    /// `fnvlist_add_int64_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_INT64_ARRAY` named `name` holding a
    /// copy of the `n` `int64_t` values at `val`. The wrapper verifies that
    /// [`nvlist_add_int64_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_int64_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut i64, n: uint_t);
    /// `fnvlist_add_uint64_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_UINT64_ARRAY` named `name` holding a
    /// copy of the `n` `uint64_t` values at `val`. The wrapper verifies
    /// that [`nvlist_add_uint64_array`] succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_uint64_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut u64, n: uint_t);
    /// `fnvlist_add_string_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_STRING_ARRAY` named `name` holding
    /// copies of the `n` strings whose pointers start at `val`, declared as
    /// `char *const *` just as for [`nvlist_add_string_array`]. The wrapper
    /// verifies that the addition succeeded. The context rule of
    /// [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_string_array(nvl: *mut nvlist_t, name: *const c_char, val: *const *mut c_char, n: uint_t);
    /// `fnvlist_add_nvlist_array`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Adds a pair of type `DATA_TYPE_NVLIST_ARRAY` named `name` holding
    /// deep copies of the `n` lists whose pointers start at `val`. The
    /// wrapper verifies that [`nvlist_add_nvlist_array`] succeeded. The
    /// context rule of [`fnvlist_add_boolean`] applies.
    pub fn fnvlist_add_nvlist_array(nvl: *mut nvlist_t, name: *const c_char, val: *mut *mut nvlist_t, n: uint_t);

    // --- fnvlist.mapfile: force wrappers, removal --------------------------

    /// `fnvlist_remove`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Removes every pair named `name` from `nvl` by way of
    /// [`nvlist_remove_all`] and verifies that at least one was found, so
    /// asking to remove a name that is not present panics the system.
    pub fn fnvlist_remove(nvl: *mut nvlist_t, name: *const c_char);
    /// `fnvlist_remove_nvpair`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Removes the specific pair `pair`, which must belong to `nvl`, by way
    /// of [`nvlist_remove_nvpair`] and verifies that the removal succeeded.
    pub fn fnvlist_remove_nvpair(nvl: *mut nvlist_t, pair: *mut nvpair_t);

    // --- fnvlist.mapfile: force wrappers, lookup ---------------------------

    /// `fnvlist_lookup_nvpair`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the pair named `name`, whatever its type, by way of
    /// [`nvlist_lookup_nvpair`]. The wrapper verifies that the lookup
    /// succeeded, so a missing name or a list without [`NV_UNIQUE_NAME`]
    /// panics the system. It allocates nothing.
    pub fn fnvlist_lookup_nvpair(nvl: *mut nvlist_t, name: *const c_char) -> *mut nvpair_t;
    /// `fnvlist_lookup_boolean`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns `B_TRUE` if a pair of type `DATA_TYPE_BOOLEAN` named `name`
    /// is present and `B_FALSE` otherwise. Unlike the other force lookups
    /// it never panics on a missing pair, since absence is the false value;
    /// it simply reports whether [`nvlist_lookup_boolean`] returned 0.
    pub fn fnvlist_lookup_boolean(nvl: *mut nvlist_t, name: *const c_char) -> boolean_t;
    /// `fnvlist_lookup_boolean_value`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `boolean_t` of the pair of type
    /// `DATA_TYPE_BOOLEAN_VALUE` named `name`. The wrapper verifies that
    /// [`nvlist_lookup_boolean_value`] succeeded, so a missing pair or a
    /// type mismatch panics the system. The same holds for every
    /// `fnvlist_lookup_*` function below.
    pub fn fnvlist_lookup_boolean_value(nvl: *mut nvlist_t, name: *const c_char) -> boolean_t;
    /// `fnvlist_lookup_byte`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uchar_t` of the pair of type `DATA_TYPE_BYTE` named
    /// `name`, verifying that [`nvlist_lookup_byte`] succeeded.
    pub fn fnvlist_lookup_byte(nvl: *mut nvlist_t, name: *const c_char) -> c_uchar;
    /// `fnvlist_lookup_int8`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `int8_t` of the pair of type `DATA_TYPE_INT8` named
    /// `name`, verifying that [`nvlist_lookup_int8`] succeeded.
    pub fn fnvlist_lookup_int8(nvl: *mut nvlist_t, name: *const c_char) -> i8;
    /// `fnvlist_lookup_int16`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `int16_t` of the pair of type `DATA_TYPE_INT16` named
    /// `name`, verifying that [`nvlist_lookup_int16`] succeeded.
    pub fn fnvlist_lookup_int16(nvl: *mut nvlist_t, name: *const c_char) -> i16;
    /// `fnvlist_lookup_int32`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `int32_t` of the pair of type `DATA_TYPE_INT32` named
    /// `name`, verifying that [`nvlist_lookup_int32`] succeeded.
    pub fn fnvlist_lookup_int32(nvl: *mut nvlist_t, name: *const c_char) -> i32;
    /// `fnvlist_lookup_int64`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `int64_t` of the pair of type `DATA_TYPE_INT64` named
    /// `name`, verifying that [`nvlist_lookup_int64`] succeeded.
    pub fn fnvlist_lookup_int64(nvl: *mut nvlist_t, name: *const c_char) -> i64;
    /// `fnvlist_lookup_uint8`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uint8_t` of the pair of type `DATA_TYPE_UINT8` named
    /// `name`, verifying that [`nvlist_lookup_uint8`] succeeded.
    pub fn fnvlist_lookup_uint8(nvl: *mut nvlist_t, name: *const c_char) -> u8;
    /// `fnvlist_lookup_uint16`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uint16_t` of the pair of type `DATA_TYPE_UINT16` named
    /// `name`, verifying that [`nvlist_lookup_uint16`] succeeded.
    pub fn fnvlist_lookup_uint16(nvl: *mut nvlist_t, name: *const c_char) -> u16;
    /// `fnvlist_lookup_uint32`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uint32_t` of the pair of type `DATA_TYPE_UINT32` named
    /// `name`, verifying that [`nvlist_lookup_uint32`] succeeded.
    pub fn fnvlist_lookup_uint32(nvl: *mut nvlist_t, name: *const c_char) -> u32;
    /// `fnvlist_lookup_uint64`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uint64_t` of the pair of type `DATA_TYPE_UINT64` named
    /// `name`, verifying that [`nvlist_lookup_uint64`] succeeded.
    pub fn fnvlist_lookup_uint64(nvl: *mut nvlist_t, name: *const c_char) -> u64;
    /// `fnvlist_lookup_string`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns a pointer to the string of the pair of type
    /// `DATA_TYPE_STRING` named `name`, verifying that
    /// [`nvlist_lookup_string`] succeeded. The string is owned by the list
    /// and stays valid until the list is freed.
    pub fn fnvlist_lookup_string(nvl: *mut nvlist_t, name: *const c_char) -> *mut c_char;
    /// `fnvlist_lookup_nvlist`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the nested list of the pair of type `DATA_TYPE_NVLIST` named
    /// `name`, verifying that [`nvlist_lookup_nvlist`] succeeded. The nested
    /// list is owned by `nvl` and must not be freed separately.
    pub fn fnvlist_lookup_nvlist(nvl: *mut nvlist_t, name: *const c_char) -> *mut nvlist_t;

    // --- fnvlist.mapfile: force wrappers, pair values ----------------------

    /// `fnvpair_value_boolean_value`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `boolean_t` of a pair of type `DATA_TYPE_BOOLEAN_VALUE`.
    /// The wrapper verifies that [`nvpair_value_boolean_value`] succeeded,
    /// so a pair of any other type panics the system. The same holds for
    /// every `fnvpair_value_*` function below. None of them allocates.
    pub fn fnvpair_value_boolean_value(nvp: *mut nvpair_t) -> boolean_t;
    /// `fnvpair_value_byte`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uchar_t` of a pair of type `DATA_TYPE_BYTE`, verifying
    /// that [`nvpair_value_byte`] succeeded.
    pub fn fnvpair_value_byte(nvp: *mut nvpair_t) -> c_uchar;
    /// `fnvpair_value_int8`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `int8_t` of a pair of type `DATA_TYPE_INT8`, verifying
    /// that [`nvpair_value_int8`] succeeded.
    pub fn fnvpair_value_int8(nvp: *mut nvpair_t) -> i8;
    /// `fnvpair_value_int16`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `int16_t` of a pair of type `DATA_TYPE_INT16`, verifying
    /// that [`nvpair_value_int16`] succeeded.
    pub fn fnvpair_value_int16(nvp: *mut nvpair_t) -> i16;
    /// `fnvpair_value_int32`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `int32_t` of a pair of type `DATA_TYPE_INT32`, verifying
    /// that [`nvpair_value_int32`] succeeded.
    pub fn fnvpair_value_int32(nvp: *mut nvpair_t) -> i32;
    /// `fnvpair_value_int64`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `int64_t` of a pair of type `DATA_TYPE_INT64`, verifying
    /// that [`nvpair_value_int64`] succeeded.
    pub fn fnvpair_value_int64(nvp: *mut nvpair_t) -> i64;
    /// `fnvpair_value_uint8`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uint8_t` of a pair of type `DATA_TYPE_UINT8`, verifying
    /// that [`nvpair_value_uint8`] succeeded.
    pub fn fnvpair_value_uint8(nvp: *mut nvpair_t) -> u8;
    /// `fnvpair_value_uint16`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uint16_t` of a pair of type `DATA_TYPE_UINT16`,
    /// verifying that [`nvpair_value_uint16`] succeeded.
    pub fn fnvpair_value_uint16(nvp: *mut nvpair_t) -> u16;
    /// `fnvpair_value_uint32`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uint32_t` of a pair of type `DATA_TYPE_UINT32`,
    /// verifying that [`nvpair_value_uint32`] succeeded.
    pub fn fnvpair_value_uint32(nvp: *mut nvpair_t) -> u32;
    /// `fnvpair_value_uint64`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the `uint64_t` of a pair of type `DATA_TYPE_UINT64`,
    /// verifying that [`nvpair_value_uint64`] succeeded.
    pub fn fnvpair_value_uint64(nvp: *mut nvpair_t) -> u64;
    /// `fnvpair_value_string`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns a pointer to the string of a pair of type
    /// `DATA_TYPE_STRING`, verifying that [`nvpair_value_string`]
    /// succeeded. The string is owned by the list the pair came from.
    pub fn fnvpair_value_string(nvp: *mut nvpair_t) -> *mut c_char;
    /// `fnvpair_value_nvlist`. Listed in `fnvlist.mapfile`; documented in `libnvpair(3LIB)`.
    ///
    /// Returns the nested list of a pair of type `DATA_TYPE_NVLIST`,
    /// verifying that [`nvpair_value_nvlist`] succeeded. The nested list is
    /// owned by the outer list and must not be freed separately.
    pub fn fnvpair_value_nvlist(nvp: *mut nvpair_t) -> *mut nvlist_t;
}
