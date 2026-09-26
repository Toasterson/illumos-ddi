//! Name-value pair lists with the rules of `nvlist_alloc(9F)` in the types.
//!
//! An `nvlist_t` is an ordered list of named, typed values. A driver meets
//! one when it posts an ereport to the fault management framework, when it
//! exchanges structured data with a user program through an ioctl, and when
//! another kernel subsystem hands it a description of something. The kernel
//! manages all of the memory behind a list. Every string, array and nested
//! list that a lookup returns points into the list and stays valid only as
//! long as the list does and only until the pair it belongs to is removed or
//! replaced.
//!
//! A typed lookup matches on the name and the type together, because that
//! is how `nvlist_lookup_boolean(9F)` and its siblings search. Looking up a
//! `u32` pair with `lookup_u64` therefore reports the pair as absent,
//! `Ok(None)`, rather than as a type error. This was confirmed in a running
//! kernel. [`NvListRef::lookup_pair`] finds a pair by name alone and reports
//! its real type.
//!
//! # Ownership
//!
//! [`NvList`] owns a list. It is created with `nvlist_alloc(9F)`, with
//! `nvlist_dup(9F)` or with `nvlist_unpack(9F)`, and it calls
//! `nvlist_free(9F)` when dropped. [`NvListRef`] borrows a list that someone
//! else owns. That is how a driver sees a nested list, which belongs to the
//! list that contains it, a list the kernel passes in, and an [`NvList`] it
//! wants to read through a shared view. Strings, arrays, pairs and nested
//! lists returned by a lookup borrow from the list they came from. The
//! methods that change a list take `&mut self`, so the borrow checker rejects
//! a change while anything obtained from the list is still in use. That is
//! how the manual's rule that a pair must not be used after it has been
//! removed or replaced becomes something the compiler checks.
//!
//! # Allocation policy
//!
//! The `kmflag` given to `nvlist_alloc(9F)` is not only used for the list
//! header. It is stored in the list and governs every allocation made on
//! behalf of the list from then on, and every add function allocates. The
//! rule of `kmem_alloc(9F)` that `KM_SLEEP` may not be used in interrupt
//! context therefore applies to each add, not only to the creation. For that
//! reason the policy is a type parameter of [`NvList`] and there are two
//! kinds of list.
//!
//! A [`KmSleep`] list comes from [`NvList::new`], which requires a
//! [`Sleepable`] token. Its add methods require the token again, at every
//! call. A list created in `attach(9E)` could otherwise be stored and filled
//! later from an interrupt handler, where a `KM_SLEEP` allocation is not
//! allowed. In exchange the adds cannot fail for lack of memory.
//!
//! A [`KmNoSleep`] list comes from [`NvList::try_new`], which needs no token.
//! Its add methods need no token either and may be called from any context,
//! but each of them may fail with `ENOMEM`.
//!
//! Methods that do not allocate, which are the lookups, the iteration, the
//! removal and the size computation, need no token for either kind.
//! Duplicating, packing and unpacking allocate under a policy chosen by the
//! call rather than by the list, so each has a variant that takes the token
//! and a `try_` variant that does not.
//!
//! # Names and lookups
//!
//! Every list this module creates with `nvlist_alloc(9F)` uses
//! `NV_UNIQUE_NAME`, so adding a pair replaces any pair with the same name,
//! whatever its type. The lookup functions only work on lists with unique
//! names and report `ENOTSUP` otherwise. A list that comes from
//! [`NvList::unpack`] or from the kernel keeps the flags it was created with,
//! so a lookup on such a list may return that error.
//!
//! A lookup returns `Ok(Some(value))` when the list holds a pair with the
//! given name and the requested type, `Ok(None)` when it does not, and `Err`
//! for any other error. The `nvlist_lookup_*` functions match on name and
//! type together, so a pair with the right name and a different type counts
//! as absent.
//!
//! # Threads
//!
//! The manual allows many readers of a list at once. The implementation,
//! however, stores an iteration hint inside the list every time
//! `nvlist_next_nvpair(9F)` is called, so two threads iterating over the same
//! list write the same field. [`NvList`] is therefore `Send` and not `Sync`.
//! A driver that shares a list between threads puts it in a
//! [`Mutex`](crate::sync::Mutex).

use core::ffi::CStr;
use core::fmt;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::ptr::{self, NonNull};

use crate::context::Sleepable;
use crate::error::Errno;
use crate::sys;

mod sealed {
    pub trait Sealed {}
}

/// The allocation policy of an [`NvList`], fixed when the list is created.
///
/// The trait is sealed. [`KmSleep`] and [`KmNoSleep`] are its only
/// implementations.
pub trait AllocPolicy: sealed::Sealed {
    /// The `kmflag` value that the list was created with and that every
    /// allocation on its behalf uses.
    const KMFLAG: i32;
}

/// The policy of a list created with `KM_SLEEP`. Allocations on behalf of
/// the list may block and never fail, so every method that allocates
/// requires a [`Sleepable`] token.
pub enum KmSleep {}

/// The policy of a list created with `KM_NOSLEEP`. Allocations on behalf of
/// the list never block and may fail with `ENOMEM`, so the list may be
/// changed from any context.
pub enum KmNoSleep {}

impl sealed::Sealed for KmSleep {}
impl sealed::Sealed for KmNoSleep {}

impl AllocPolicy for KmSleep {
    const KMFLAG: i32 = sys::KM_SLEEP;
}

impl AllocPolicy for KmNoSleep {
    const KMFLAG: i32 = sys::KM_NOSLEEP;
}

/// The encoding used by [`NvListRef::pack`] and its relatives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NvEncoding {
    /// A straight copy of the in-memory representation, for a consumer on
    /// the same host, such as a user program reading the result of an ioctl
    /// (`NV_ENCODE_NATIVE`).
    Native,
    /// XDR encoding, which does not depend on the byte order or the word
    /// size of the host (`NV_ENCODE_XDR`).
    Xdr,
}

impl NvEncoding {
    fn raw(self) -> i32 {
        match self {
            NvEncoding::Native => sys::NV_ENCODE_NATIVE,
            NvEncoding::Xdr => sys::NV_ENCODE_XDR,
        }
    }
}

/// Interprets the return value of an `nvlist_lookup_*` function. 0 means the
/// pair was found, `ENOENT` means it was not, and anything else is an error.
fn found(raw: i32) -> Result<bool, Errno> {
    match raw {
        0 => Ok(true),
        sys::errno::ENOENT => Ok(false),
        e => Err(Errno::new(e).unwrap_or(Errno::EINVAL)),
    }
}

/// Builds a slice from an array the kernel returned, which may be NULL when
/// it has no elements.
///
/// # Safety
/// When `len` is not zero, `data` must point to `len` initialised elements
/// that stay valid and unchanged for `'a`.
unsafe fn array<'a, T>(data: *const T, len: sys::uint_t) -> &'a [T] {
    if len == 0 || data.is_null() { &[] } else { unsafe { core::slice::from_raw_parts(data, len as usize) } }
}

/// Converts a slice length into the `nelem` argument of an add function.
fn nelem(len: usize) -> Result<sys::uint_t, Errno> {
    sys::uint_t::try_from(len).map_err(|_| Errno::EINVAL)
}

/// A list of name-value pairs that this value owns.
///
/// The list is created with `NV_UNIQUE_NAME`, so adding a pair replaces any
/// pair with the same name. It is released with `nvlist_free(9F)` when the
/// value is dropped, which is allowed in any context.
///
/// The type parameter is the allocation policy described in the
/// [module documentation](self). `NvList` on its own means a [`KmSleep`]
/// list. All the lookups of [`NvListRef`] are also available directly on an
/// `NvList`, and [`NvList::as_list`] gives the borrowed view itself.
pub struct NvList<P: AllocPolicy = KmSleep> {
    raw: NonNull<sys::nvlist_t>,
    _policy: PhantomData<P>,
}

// The list is memory owned by this value and nothing in the kernel ties it to
// the thread that created it, so ownership may move to another thread. It is
// not `Sync` because reading through `&NvList` includes iteration, which
// writes an iteration hint inside the list.
unsafe impl<P: AllocPolicy> Send for NvList<P> {}

impl<P: AllocPolicy> NvList<P> {
    /// Allocates an empty list with `nvlist_alloc(9F)` under this policy.
    fn alloc() -> Result<Self, Errno> {
        let mut raw: *mut sys::nvlist_t = ptr::null_mut();
        Errno::check(unsafe { sys::nvlist_alloc(&mut raw, sys::NV_UNIQUE_NAME, P::KMFLAG) })?;
        NonNull::new(raw).map(|raw| NvList { raw, _policy: PhantomData }).ok_or(Errno::ENOMEM)
    }

    /// Unpacks a buffer under this policy.
    fn unpack_with(buf: &[u8]) -> Result<Self, Errno> {
        let mut raw: *mut sys::nvlist_t = ptr::null_mut();
        // nvlist_unpack only reads the buffer, although its prototype takes
        // a mutable pointer.
        Errno::check(unsafe { sys::nvlist_unpack(buf.as_ptr() as *mut sys::c_char, buf.len(), &mut raw, P::KMFLAG) })?;
        NonNull::new(raw).map(|raw| NvList { raw, _policy: PhantomData }).ok_or(Errno::EINVAL)
    }

    /// Takes ownership of a list created elsewhere. The list is freed with
    /// `nvlist_free(9F)` when the returned value is dropped.
    ///
    /// # Safety
    /// `raw` must be a valid list that the caller owns and that nothing else
    /// will free or use afterwards. Its allocation policy must match `P`,
    /// that is, it must have been created with `KM_SLEEP` for a [`KmSleep`]
    /// list and with `KM_NOSLEEP` for a [`KmNoSleep`] list, because the add
    /// methods rely on that when they decide whether a token is needed.
    pub unsafe fn from_raw(raw: *mut sys::nvlist_t) -> Self {
        NvList { raw: unsafe { NonNull::new_unchecked(raw) }, _policy: PhantomData }
    }

    /// Gives up ownership and returns the raw pointer, for example to hand
    /// the list to a kernel function that frees it. The list is not freed
    /// by this crate afterwards.
    pub fn into_raw(self) -> *mut sys::nvlist_t {
        ManuallyDrop::new(self).raw.as_ptr()
    }

    /// Returns the raw pointer, for `9F` functions that are not wrapped yet.
    /// The list remains owned by this value.
    pub fn as_raw(&self) -> *mut sys::nvlist_t {
        self.raw.as_ptr()
    }

    /// Returns a borrowed view of the list, which is what
    /// [`NvList::add_nvlist`] and other functions that accept any list take.
    pub fn as_list(&self) -> NvListRef<'_> {
        NvListRef { raw: self.raw, _life: PhantomData }
    }

    /// Removes every pair called `name`, whatever its type, with
    /// `nvlist_remove_all(9F)`. Returns whether any pair was removed.
    /// Removal frees memory and allocates none, so it needs no token.
    pub fn remove(&mut self, name: &CStr) -> Result<bool, Errno> {
        found(unsafe { sys::nvlist_remove_all(self.raw.as_ptr(), name.as_ptr()) })
    }

    fn add_flag_impl(&mut self, name: &CStr) -> Result<(), Errno> {
        Errno::check(unsafe { sys::nvlist_add_boolean(self.raw.as_ptr(), name.as_ptr()) })
    }

    fn add_bool_impl(&mut self, name: &CStr, value: bool) -> Result<(), Errno> {
        let v = if value { sys::B_TRUE } else { sys::B_FALSE };
        Errno::check(unsafe { sys::nvlist_add_boolean_value(self.raw.as_ptr(), name.as_ptr(), v) })
    }

    fn add_string_impl(&mut self, name: &CStr, value: &CStr) -> Result<(), Errno> {
        Errno::check(unsafe { sys::nvlist_add_string(self.raw.as_ptr(), name.as_ptr(), value.as_ptr()) })
    }

    fn add_bytes_impl(&mut self, name: &CStr, value: &[u8]) -> Result<(), Errno> {
        let n = nelem(value.len())?;
        // The add functions copy the array and never write to it, although
        // their prototypes take a mutable pointer.
        Errno::check(unsafe {
            sys::nvlist_add_byte_array(self.raw.as_ptr(), name.as_ptr(), value.as_ptr() as *mut u8, n)
        })
    }

    fn add_u32_array_impl(&mut self, name: &CStr, value: &[u32]) -> Result<(), Errno> {
        let n = nelem(value.len())?;
        Errno::check(unsafe {
            sys::nvlist_add_uint32_array(self.raw.as_ptr(), name.as_ptr(), value.as_ptr() as *mut u32, n)
        })
    }

    fn add_u64_array_impl(&mut self, name: &CStr, value: &[u64]) -> Result<(), Errno> {
        let n = nelem(value.len())?;
        Errno::check(unsafe {
            sys::nvlist_add_uint64_array(self.raw.as_ptr(), name.as_ptr(), value.as_ptr() as *mut u64, n)
        })
    }

    fn add_strings_impl(&mut self, name: &CStr, value: &[&CStr]) -> Result<(), Errno> {
        let n = nelem(value.len())?;
        let mut small: [*mut sys::c_char; 16] = [ptr::null_mut(); 16];
        if value.len() <= small.len() {
            for (slot, s) in small.iter_mut().zip(value) {
                *slot = s.as_ptr() as *mut sys::c_char;
            }
            return Errno::check(unsafe {
                sys::nvlist_add_string_array(self.raw.as_ptr(), name.as_ptr(), small.as_ptr(), n)
            });
        }
        // The pointer table is allocated under the list's own policy, so a
        // KM_NOSLEEP list never blocks here either.
        let bytes = value.len() * size_of::<*mut sys::c_char>();
        let table = unsafe { sys::kmem_alloc(bytes, P::KMFLAG) } as *mut *mut sys::c_char;
        if table.is_null() {
            return Err(Errno::ENOMEM);
        }
        for (i, s) in value.iter().enumerate() {
            unsafe { table.add(i).write(s.as_ptr() as *mut sys::c_char) };
        }
        let r = unsafe { sys::nvlist_add_string_array(self.raw.as_ptr(), name.as_ptr(), table, n) };
        unsafe { sys::kmem_free(table as *mut sys::c_void, bytes) };
        Errno::check(r)
    }

    fn add_nvlist_impl(&mut self, name: &CStr, value: NvListRef<'_>) -> Result<(), Errno> {
        Errno::check(unsafe { sys::nvlist_add_nvlist(self.raw.as_ptr(), name.as_ptr(), value.raw.as_ptr()) })
    }

    fn merge_impl(&mut self, other: NvListRef<'_>) -> Result<(), Errno> {
        Errno::check(unsafe { sys::nvlist_merge(self.raw.as_ptr(), other.raw.as_ptr(), P::KMFLAG) })
    }
}

impl<P: AllocPolicy> Drop for NvList<P> {
    fn drop(&mut self) {
        unsafe { sys::nvlist_free(self.raw.as_ptr()) }
    }
}

impl NvList<KmSleep> {
    /// Allocates an empty list with `nvlist_alloc(9F)`, `NV_UNIQUE_NAME` and
    /// `KM_SLEEP`. The allocation may block and cannot fail, which is why a
    /// [`Sleepable`] token is required. Every later addition to the list
    /// requires the token as well.
    pub fn new(_ctx: &Sleepable) -> NvList<KmSleep> {
        match Self::alloc() {
            Ok(l) => l,
            // nvlist_alloc(9F) only fails with ENOMEM, which KM_SLEEP rules
            // out, or with EINVAL for a NULL result pointer.
            Err(e) => panic!("nvlist_alloc(KM_SLEEP) failed with {e}"),
        }
    }

    /// Unpacks a buffer produced by `nvlist_pack(9F)` into a new list, with
    /// `nvlist_unpack(9F)` and `KM_SLEEP`. The encoding is read from the
    /// buffer, and the buffer may come from an untrusted source such as a
    /// user program; a malformed buffer is reported as `EFAULT` or `EINVAL`.
    /// The new list keeps the uniqueness flag it was packed with, which may
    /// not be `NV_UNIQUE_NAME`.
    pub fn unpack(buf: &[u8], _ctx: &Sleepable) -> Result<NvList<KmSleep>, Errno> {
        Self::unpack_with(buf)
    }
}

impl NvList<KmNoSleep> {
    /// Allocates an empty list with `nvlist_alloc(9F)`, `NV_UNIQUE_NAME` and
    /// `KM_NOSLEEP`. This may be called from any context and fails with
    /// `ENOMEM` when memory is not available at once. Later additions to the
    /// list follow the same policy.
    pub fn try_new() -> Result<NvList<KmNoSleep>, Errno> {
        Self::alloc()
    }

    /// Unpacks a buffer produced by `nvlist_pack(9F)` into a new list, with
    /// `nvlist_unpack(9F)` and `KM_NOSLEEP`. Apart from the allocation
    /// policy it behaves like [`NvList::unpack`].
    pub fn try_unpack(buf: &[u8]) -> Result<NvList<KmNoSleep>, Errno> {
        Self::unpack_with(buf)
    }
}

/// Generates the add methods of one allocation policy. The optional argument
/// is the token parameter that the policy requires.
macro_rules! adds {
    ($policy:literal $(, $ctx:ident : $ctx_ty:ty)?) => {
        /// Adds a pair of type `DATA_TYPE_BOOLEAN` with
        /// `nvlist_add_boolean(9F)`. Such a pair has no value; its presence
        /// alone means true.
        #[doc = $policy]
        pub fn add_flag(&mut self, name: &CStr $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            self.add_flag_impl(name)
        }

        /// Adds a `DATA_TYPE_BOOLEAN_VALUE` pair with
        /// `nvlist_add_boolean_value(9F)`.
        #[doc = $policy]
        pub fn add_bool(&mut self, name: &CStr, value: bool $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            self.add_bool_impl(name, value)
        }

        /// Adds a `DATA_TYPE_UINT8` pair with `nvlist_add_uint8(9F)`.
        #[doc = $policy]
        pub fn add_u8(&mut self, name: &CStr, value: u8 $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            Errno::check(unsafe { sys::nvlist_add_uint8(self.raw.as_ptr(), name.as_ptr(), value) })
        }

        /// Adds a `DATA_TYPE_UINT16` pair with `nvlist_add_uint16(9F)`.
        #[doc = $policy]
        pub fn add_u16(&mut self, name: &CStr, value: u16 $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            Errno::check(unsafe { sys::nvlist_add_uint16(self.raw.as_ptr(), name.as_ptr(), value) })
        }

        /// Adds a `DATA_TYPE_UINT32` pair with `nvlist_add_uint32(9F)`.
        #[doc = $policy]
        pub fn add_u32(&mut self, name: &CStr, value: u32 $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            Errno::check(unsafe { sys::nvlist_add_uint32(self.raw.as_ptr(), name.as_ptr(), value) })
        }

        /// Adds a `DATA_TYPE_UINT64` pair with `nvlist_add_uint64(9F)`.
        #[doc = $policy]
        pub fn add_u64(&mut self, name: &CStr, value: u64 $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            Errno::check(unsafe { sys::nvlist_add_uint64(self.raw.as_ptr(), name.as_ptr(), value) })
        }

        /// Adds a `DATA_TYPE_INT8` pair with `nvlist_add_int8(9F)`.
        #[doc = $policy]
        pub fn add_i8(&mut self, name: &CStr, value: i8 $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            Errno::check(unsafe { sys::nvlist_add_int8(self.raw.as_ptr(), name.as_ptr(), value) })
        }

        /// Adds a `DATA_TYPE_INT16` pair with `nvlist_add_int16(9F)`.
        #[doc = $policy]
        pub fn add_i16(&mut self, name: &CStr, value: i16 $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            Errno::check(unsafe { sys::nvlist_add_int16(self.raw.as_ptr(), name.as_ptr(), value) })
        }

        /// Adds a `DATA_TYPE_INT32` pair with `nvlist_add_int32(9F)`.
        #[doc = $policy]
        pub fn add_i32(&mut self, name: &CStr, value: i32 $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            Errno::check(unsafe { sys::nvlist_add_int32(self.raw.as_ptr(), name.as_ptr(), value) })
        }

        /// Adds a `DATA_TYPE_INT64` pair with `nvlist_add_int64(9F)`.
        #[doc = $policy]
        pub fn add_i64(&mut self, name: &CStr, value: i64 $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            Errno::check(unsafe { sys::nvlist_add_int64(self.raw.as_ptr(), name.as_ptr(), value) })
        }

        /// Adds a `DATA_TYPE_STRING` pair with `nvlist_add_string(9F)`. The
        /// list keeps its own copy of the string.
        #[doc = $policy]
        pub fn add_string(&mut self, name: &CStr, value: &CStr $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            self.add_string_impl(name, value)
        }

        /// Adds a `DATA_TYPE_BYTE_ARRAY` pair with
        /// `nvlist_add_byte_array(9F)`. The list keeps its own copy of the
        /// bytes. An array with more than `u32::MAX` elements is refused with
        /// `EINVAL`.
        #[doc = $policy]
        pub fn add_bytes(&mut self, name: &CStr, value: &[u8] $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            self.add_bytes_impl(name, value)
        }

        /// Adds a `DATA_TYPE_UINT32_ARRAY` pair with
        /// `nvlist_add_uint32_array(9F)`. The list keeps its own copy of the
        /// array.
        #[doc = $policy]
        pub fn add_u32_array(&mut self, name: &CStr, value: &[u32] $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            self.add_u32_array_impl(name, value)
        }

        /// Adds a `DATA_TYPE_UINT64_ARRAY` pair with
        /// `nvlist_add_uint64_array(9F)`. The list keeps its own copy of the
        /// array.
        #[doc = $policy]
        pub fn add_u64_array(&mut self, name: &CStr, value: &[u64] $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            self.add_u64_array_impl(name, value)
        }

        /// Adds a `DATA_TYPE_STRING_ARRAY` pair with
        /// `nvlist_add_string_array(9F)`. The list keeps its own copies of
        /// the strings. Up to sixteen strings are passed without an
        /// allocation of this crate's own; a longer array needs a temporary
        /// pointer table from `kmem_alloc(9F)`, allocated under the list's
        /// policy.
        #[doc = $policy]
        pub fn add_strings(&mut self, name: &CStr, value: &[&CStr] $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            self.add_strings_impl(name, value)
        }

        /// Adds a `DATA_TYPE_NVLIST` pair with `nvlist_add_nvlist(9F)`. The
        /// list keeps a deep copy of `value`, so `value` may be dropped or
        /// changed afterwards without affecting this list.
        #[doc = $policy]
        pub fn add_nvlist(&mut self, name: &CStr, value: NvListRef<'_> $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            self.add_nvlist_impl(name, value)
        }

        /// Adds a copy of every pair of `other` to this list with
        /// `nvlist_merge(9F)`. Because this list has unique names, a pair of
        /// `other` replaces a pair of this list with the same name.
        #[doc = $policy]
        pub fn merge(&mut self, other: NvListRef<'_> $(, $ctx: $ctx_ty)?) -> Result<(), Errno> {
            self.merge_impl(other)
        }
    };
}

impl NvList<KmSleep> {
    adds!(
        "\n\nThe list was created with `KM_SLEEP`, so the allocation may block \
         and a [`Sleepable`] token is required. Lack of memory cannot make the \
         call fail; an error means an invalid argument.",
        _ctx: &Sleepable
    );
}

impl NvList<KmNoSleep> {
    adds!(
        "\n\nThe list was created with `KM_NOSLEEP`, so the call never blocks, \
         may be made from any context, and fails with `ENOMEM` when memory is \
         not available at once."
    );
}

/// Generates the methods that read a list. They are shared between
/// [`NvListRef`], where the results borrow for the view's lifetime, and
/// [`NvList`], where they borrow for as long as the list is borrowed.
macro_rules! reads {
    ($l:lifetime, [$($g:tt)*], $recv:ty) => {
        /// Reports whether the list holds a `DATA_TYPE_BOOLEAN` pair called
        /// `name`, with `nvlist_lookup_boolean(9F)`. Such a pair has no value,
        /// so presence alone means true.
        pub fn lookup_flag<$($g)*>(self: $recv, name: &CStr) -> Result<bool, Errno> {
            found(unsafe { sys::nvlist_lookup_boolean(self.raw.as_ptr(), name.as_ptr()) })
        }

        /// Looks up a `DATA_TYPE_BOOLEAN_VALUE` pair with
        /// `nvlist_lookup_boolean_value(9F)`.
        pub fn lookup_bool<$($g)*>(self: $recv, name: &CStr) -> Result<Option<bool>, Errno> {
            let mut v: sys::boolean_t = sys::B_FALSE;
            let r = unsafe { sys::nvlist_lookup_boolean_value(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then_some(v != sys::B_FALSE))
        }

        /// Looks up a `DATA_TYPE_UINT8` pair with `nvlist_lookup_uint8(9F)`.
        pub fn lookup_u8<$($g)*>(self: $recv, name: &CStr) -> Result<Option<u8>, Errno> {
            let mut v = 0;
            let r = unsafe { sys::nvlist_lookup_uint8(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then_some(v))
        }

        /// Looks up a `DATA_TYPE_UINT16` pair with `nvlist_lookup_uint16(9F)`.
        pub fn lookup_u16<$($g)*>(self: $recv, name: &CStr) -> Result<Option<u16>, Errno> {
            let mut v = 0;
            let r = unsafe { sys::nvlist_lookup_uint16(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then_some(v))
        }

        /// Looks up a `DATA_TYPE_UINT32` pair with `nvlist_lookup_uint32(9F)`.
        pub fn lookup_u32<$($g)*>(self: $recv, name: &CStr) -> Result<Option<u32>, Errno> {
            let mut v = 0;
            let r = unsafe { sys::nvlist_lookup_uint32(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then_some(v))
        }

        /// Looks up a `DATA_TYPE_UINT64` pair with `nvlist_lookup_uint64(9F)`.
        pub fn lookup_u64<$($g)*>(self: $recv, name: &CStr) -> Result<Option<u64>, Errno> {
            let mut v = 0;
            let r = unsafe { sys::nvlist_lookup_uint64(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then_some(v))
        }

        /// Looks up a `DATA_TYPE_INT8` pair with `nvlist_lookup_int8(9F)`.
        pub fn lookup_i8<$($g)*>(self: $recv, name: &CStr) -> Result<Option<i8>, Errno> {
            let mut v = 0;
            let r = unsafe { sys::nvlist_lookup_int8(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then_some(v))
        }

        /// Looks up a `DATA_TYPE_INT16` pair with `nvlist_lookup_int16(9F)`.
        pub fn lookup_i16<$($g)*>(self: $recv, name: &CStr) -> Result<Option<i16>, Errno> {
            let mut v = 0;
            let r = unsafe { sys::nvlist_lookup_int16(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then_some(v))
        }

        /// Looks up a `DATA_TYPE_INT32` pair with `nvlist_lookup_int32(9F)`.
        pub fn lookup_i32<$($g)*>(self: $recv, name: &CStr) -> Result<Option<i32>, Errno> {
            let mut v = 0;
            let r = unsafe { sys::nvlist_lookup_int32(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then_some(v))
        }

        /// Looks up a `DATA_TYPE_INT64` pair with `nvlist_lookup_int64(9F)`.
        pub fn lookup_i64<$($g)*>(self: $recv, name: &CStr) -> Result<Option<i64>, Errno> {
            let mut v = 0;
            let r = unsafe { sys::nvlist_lookup_int64(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then_some(v))
        }

        /// Looks up a `DATA_TYPE_STRING` pair with
        /// `nvlist_lookup_string(9F)`. The string lives inside the list and
        /// is borrowed from it.
        pub fn lookup_string<$($g)*>(self: $recv, name: &CStr) -> Result<Option<&$l CStr>, Errno> {
            let mut v: *mut sys::c_char = ptr::null_mut();
            let r = unsafe { sys::nvlist_lookup_string(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then(|| unsafe { CStr::from_ptr(v) }))
        }

        /// Looks up a `DATA_TYPE_BYTE_ARRAY` pair with
        /// `nvlist_lookup_byte_array(9F)`. The bytes live inside the list and
        /// are borrowed from it.
        pub fn lookup_bytes<$($g)*>(self: $recv, name: &CStr) -> Result<Option<&$l [u8]>, Errno> {
            let mut v: *mut u8 = ptr::null_mut();
            let mut n: sys::uint_t = 0;
            let r = unsafe { sys::nvlist_lookup_byte_array(self.raw.as_ptr(), name.as_ptr(), &mut v, &mut n) };
            Ok(found(r)?.then(|| unsafe { array(v, n) }))
        }

        /// Looks up a `DATA_TYPE_UINT32_ARRAY` pair with
        /// `nvlist_lookup_uint32_array(9F)`. The array lives inside the list
        /// and is borrowed from it.
        pub fn lookup_u32_array<$($g)*>(self: $recv, name: &CStr) -> Result<Option<&$l [u32]>, Errno> {
            let mut v: *mut u32 = ptr::null_mut();
            let mut n: sys::uint_t = 0;
            let r = unsafe { sys::nvlist_lookup_uint32_array(self.raw.as_ptr(), name.as_ptr(), &mut v, &mut n) };
            Ok(found(r)?.then(|| unsafe { array(v, n) }))
        }

        /// Looks up a `DATA_TYPE_UINT64_ARRAY` pair with
        /// `nvlist_lookup_uint64_array(9F)`. The array lives inside the list
        /// and is borrowed from it.
        pub fn lookup_u64_array<$($g)*>(self: $recv, name: &CStr) -> Result<Option<&$l [u64]>, Errno> {
            let mut v: *mut u64 = ptr::null_mut();
            let mut n: sys::uint_t = 0;
            let r = unsafe { sys::nvlist_lookup_uint64_array(self.raw.as_ptr(), name.as_ptr(), &mut v, &mut n) };
            Ok(found(r)?.then(|| unsafe { array(v, n) }))
        }

        /// Looks up a `DATA_TYPE_STRING_ARRAY` pair with
        /// `nvlist_lookup_string_array(9F)`. The strings live inside the list
        /// and are borrowed from it.
        pub fn lookup_strings<$($g)*>(self: $recv, name: &CStr) -> Result<Option<NvStrings<$l>>, Errno> {
            let mut v: *mut *mut sys::c_char = ptr::null_mut();
            let mut n: sys::uint_t = 0;
            let r = unsafe { sys::nvlist_lookup_string_array(self.raw.as_ptr(), name.as_ptr(), &mut v, &mut n) };
            Ok(found(r)?.then(|| NvStrings { data: unsafe { array(v, n) } }))
        }

        /// Looks up a `DATA_TYPE_NVLIST` pair with
        /// `nvlist_lookup_nvlist(9F)`. The nested list is stored inside this
        /// one, so the result borrows from it rather than being a copy.
        pub fn lookup_nvlist<$($g)*>(self: $recv, name: &CStr) -> Result<Option<NvListRef<$l>>, Errno> {
            let mut v: *mut sys::nvlist_t = ptr::null_mut();
            let r = unsafe { sys::nvlist_lookup_nvlist(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then(|| NonNull::new(v).map(|raw| NvListRef { raw, _life: PhantomData })).flatten())
        }

        /// Looks up the pair called `name`, whatever its type, with
        /// `nvlist_lookup_nvpair(9F)`. This is also the way to test whether
        /// a name is present at all.
        pub fn lookup_pair<$($g)*>(self: $recv, name: &CStr) -> Result<Option<NvPair<$l>>, Errno> {
            let mut v: *mut sys::nvpair_t = ptr::null_mut();
            let r = unsafe { sys::nvlist_lookup_nvpair(self.raw.as_ptr(), name.as_ptr(), &mut v) };
            Ok(found(r)?.then(|| NonNull::new(v).map(|raw| NvPair { raw, _life: PhantomData })).flatten())
        }

        /// Returns an iterator over the pairs of the list in the order they
        /// were added, using `nvlist_next_nvpair(9F)`.
        pub fn pairs<$($g)*>(self: $recv) -> NvPairs<$l> {
            NvPairs { list: self.raw, cur: ptr::null_mut(), _life: PhantomData }
        }

        /// Makes a deep copy of the list with `nvlist_dup(9F)` and
        /// `KM_SLEEP`. The copy keeps the uniqueness flag of the original and
        /// is a [`KmSleep`] list whatever the policy of the original.
        pub fn dup<$($g)*>(self: $recv, _ctx: &Sleepable) -> Result<NvList<KmSleep>, Errno> {
            let mut raw: *mut sys::nvlist_t = ptr::null_mut();
            Errno::check(unsafe { sys::nvlist_dup(self.raw.as_ptr(), &mut raw, sys::KM_SLEEP) })?;
            NonNull::new(raw).map(|raw| NvList { raw, _policy: PhantomData }).ok_or(Errno::ENOMEM)
        }

        /// Makes a deep copy of the list with `nvlist_dup(9F)` and
        /// `KM_NOSLEEP`. It may be called from any context, fails with
        /// `ENOMEM` when memory is not available at once, and produces a
        /// [`KmNoSleep`] list.
        pub fn try_dup<$($g)*>(self: $recv) -> Result<NvList<KmNoSleep>, Errno> {
            let mut raw: *mut sys::nvlist_t = ptr::null_mut();
            Errno::check(unsafe { sys::nvlist_dup(self.raw.as_ptr(), &mut raw, sys::KM_NOSLEEP) })?;
            NonNull::new(raw).map(|raw| NvList { raw, _policy: PhantomData }).ok_or(Errno::ENOMEM)
        }

        /// Returns the number of bytes that packing the list with `encoding`
        /// produces, using `nvlist_size(9F)`. This allocates nothing.
        pub fn packed_size<$($g)*>(self: $recv, encoding: NvEncoding) -> Result<usize, Errno> {
            let mut size: usize = 0;
            Errno::check(unsafe { sys::nvlist_size(self.raw.as_ptr(), &mut size, encoding.raw()) })?;
            Ok(size)
        }

        /// Packs the list into a new buffer with `nvlist_pack(9F)` and
        /// `KM_SLEEP`. The kernel allocates the buffer with `kmem_alloc(9F)`,
        /// and the returned [`PackedNvList`] frees it with `kmem_free(9F)`
        /// when dropped. An encoding error is reported as `EFAULT`.
        pub fn pack<$($g)*>(self: $recv, encoding: NvEncoding, _ctx: &Sleepable) -> Result<PackedNvList, Errno> {
            PackedNvList::pack(self.raw, encoding, sys::KM_SLEEP)
        }

        /// Packs the list like [`pack`](NvListRef::pack), but allocates the
        /// buffer with `KM_NOSLEEP`. It may be called from any context and
        /// fails with `ENOMEM` when memory is not available at once.
        pub fn try_pack<$($g)*>(self: $recv, encoding: NvEncoding) -> Result<PackedNvList, Errno> {
            PackedNvList::pack(self.raw, encoding, sys::KM_NOSLEEP)
        }
    };
}

/// A list of name-value pairs that someone else owns.
///
/// This is the view of a nested list, of a list the kernel hands to the
/// driver, and of an [`NvList`] through [`NvList::as_list`]. Everything a
/// lookup returns borrows for `'a`, the lifetime of the owner. The view can
/// only read the list; changing a list requires the owning [`NvList`].
///
/// The type is `Copy` but neither `Send` nor `Sync`, because iteration writes
/// an iteration hint inside the list.
#[derive(Clone, Copy)]
pub struct NvListRef<'a> {
    raw: NonNull<sys::nvlist_t>,
    _life: PhantomData<&'a sys::nvlist_t>,
}

impl<'a> NvListRef<'a> {
    /// Wraps a list pointer handed in by the kernel.
    ///
    /// # Safety
    /// `raw` must point to a valid list that stays valid for `'a`. Nothing
    /// else may change the list, or iterate over it from another thread,
    /// during `'a`.
    pub unsafe fn from_raw(raw: *mut sys::nvlist_t) -> NvListRef<'a> {
        NvListRef { raw: unsafe { NonNull::new_unchecked(raw) }, _life: PhantomData }
    }

    /// Returns the raw pointer, for `9F` functions that are not wrapped yet.
    pub fn as_raw(&self) -> *mut sys::nvlist_t {
        self.raw.as_ptr()
    }

    reads!('a, [], &Self);
}

impl<P: AllocPolicy> NvList<P> {
    reads!('s, ['s], &'s Self);
}

impl<'a, P: AllocPolicy> From<&'a NvList<P>> for NvListRef<'a> {
    fn from(l: &'a NvList<P>) -> NvListRef<'a> {
        l.as_list()
    }
}

impl<'a> IntoIterator for NvListRef<'a> {
    type Item = NvPair<'a>;
    type IntoIter = NvPairs<'a>;
    fn into_iter(self) -> NvPairs<'a> {
        self.pairs()
    }
}

impl<'a, P: AllocPolicy> IntoIterator for &'a NvList<P> {
    type Item = NvPair<'a>;
    type IntoIter = NvPairs<'a>;
    fn into_iter(self) -> NvPairs<'a> {
        self.pairs()
    }
}

impl fmt::Debug for NvListRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.pairs().map(|p| (p.name(), p.value()))).finish()
    }
}

impl<P: AllocPolicy> fmt::Debug for NvList<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_list(), f)
    }
}

/// An iterator over the pairs of a list, built on `nvlist_next_nvpair(9F)`.
pub struct NvPairs<'a> {
    list: NonNull<sys::nvlist_t>,
    cur: *mut sys::nvpair_t,
    _life: PhantomData<&'a sys::nvlist_t>,
}

impl<'a> Iterator for NvPairs<'a> {
    type Item = NvPair<'a>;
    fn next(&mut self) -> Option<NvPair<'a>> {
        // The list cannot change while it is borrowed for 'a, so the pair
        // passed back in is still a member, as nvlist_next_nvpair requires.
        let next = unsafe { sys::nvlist_next_nvpair(self.list.as_ptr(), self.cur) };
        self.cur = next;
        NonNull::new(next).map(|raw| NvPair { raw, _life: PhantomData })
    }
}

/// One name-value pair inside a list, borrowed from the list.
#[derive(Clone, Copy)]
pub struct NvPair<'a> {
    raw: NonNull<sys::nvpair_t>,
    _life: PhantomData<&'a sys::nvlist_t>,
}

impl<'a> NvPair<'a> {
    /// Returns the raw pointer, for `9F` functions that are not wrapped yet.
    pub fn as_raw(&self) -> *mut sys::nvpair_t {
        self.raw.as_ptr()
    }

    /// Returns the name of the pair, using `nvpair_name(9F)`.
    pub fn name(&self) -> &'a CStr {
        unsafe { CStr::from_ptr(sys::nvpair_name(self.raw.as_ptr())) }
    }

    /// Returns the type of the value, using `nvpair_type(9F)`.
    pub fn data_type(&self) -> sys::data_type_t {
        unsafe { sys::nvpair_type(self.raw.as_ptr()) }
    }

    /// Returns the value of the pair. The type is read with
    /// `nvpair_type(9F)` and the value with the matching function from
    /// `nvpair_value_byte(9F)`. Types that [`NvValue`] does not cover come
    /// back as [`NvValue::Other`].
    pub fn value(&self) -> NvValue<'a> {
        let p = self.raw.as_ptr();
        let ty = self.data_type();

        macro_rules! scalar {
            ($f:ident, $variant:ident, $init:expr) => {{
                let mut v = $init;
                if unsafe { sys::$f(p, &mut v) } == 0 { NvValue::$variant(v) } else { NvValue::Other(ty) }
            }};
        }
        macro_rules! slice {
            ($f:ident, $variant:ident, $t:ty) => {{
                let mut v: *mut $t = ptr::null_mut();
                let mut n: sys::uint_t = 0;
                if unsafe { sys::$f(p, &mut v, &mut n) } == 0 {
                    NvValue::$variant(unsafe { array(v, n) })
                } else {
                    NvValue::Other(ty)
                }
            }};
        }

        match ty {
            sys::data_type_t::DATA_TYPE_BOOLEAN => NvValue::Flag,
            sys::data_type_t::DATA_TYPE_BOOLEAN_VALUE => {
                let mut v: sys::boolean_t = sys::B_FALSE;
                if unsafe { sys::nvpair_value_boolean_value(p, &mut v) } == 0 {
                    NvValue::Bool(v != sys::B_FALSE)
                } else {
                    NvValue::Other(ty)
                }
            }
            sys::data_type_t::DATA_TYPE_BYTE => scalar!(nvpair_value_byte, Byte, 0),
            sys::data_type_t::DATA_TYPE_UINT8 => scalar!(nvpair_value_uint8, U8, 0),
            sys::data_type_t::DATA_TYPE_UINT16 => scalar!(nvpair_value_uint16, U16, 0),
            sys::data_type_t::DATA_TYPE_UINT32 => scalar!(nvpair_value_uint32, U32, 0),
            sys::data_type_t::DATA_TYPE_UINT64 => scalar!(nvpair_value_uint64, U64, 0),
            sys::data_type_t::DATA_TYPE_INT8 => scalar!(nvpair_value_int8, I8, 0),
            sys::data_type_t::DATA_TYPE_INT16 => scalar!(nvpair_value_int16, I16, 0),
            sys::data_type_t::DATA_TYPE_INT32 => scalar!(nvpair_value_int32, I32, 0),
            sys::data_type_t::DATA_TYPE_INT64 => scalar!(nvpair_value_int64, I64, 0),
            sys::data_type_t::DATA_TYPE_HRTIME => scalar!(nvpair_value_hrtime, HrTime, 0),
            sys::data_type_t::DATA_TYPE_STRING => {
                let mut v: *mut sys::c_char = ptr::null_mut();
                if unsafe { sys::nvpair_value_string(p, &mut v) } == 0 && !v.is_null() {
                    NvValue::String(unsafe { CStr::from_ptr(v) })
                } else {
                    NvValue::Other(ty)
                }
            }
            sys::data_type_t::DATA_TYPE_NVLIST => {
                let mut v: *mut sys::nvlist_t = ptr::null_mut();
                if unsafe { sys::nvpair_value_nvlist(p, &mut v) } == 0 {
                    match NonNull::new(v) {
                        Some(raw) => NvValue::NvList(NvListRef { raw, _life: PhantomData }),
                        None => NvValue::Other(ty),
                    }
                } else {
                    NvValue::Other(ty)
                }
            }
            sys::data_type_t::DATA_TYPE_BYTE_ARRAY => slice!(nvpair_value_byte_array, Bytes, u8),
            sys::data_type_t::DATA_TYPE_UINT8_ARRAY => slice!(nvpair_value_uint8_array, U8Array, u8),
            sys::data_type_t::DATA_TYPE_UINT16_ARRAY => slice!(nvpair_value_uint16_array, U16Array, u16),
            sys::data_type_t::DATA_TYPE_UINT32_ARRAY => slice!(nvpair_value_uint32_array, U32Array, u32),
            sys::data_type_t::DATA_TYPE_UINT64_ARRAY => slice!(nvpair_value_uint64_array, U64Array, u64),
            sys::data_type_t::DATA_TYPE_INT8_ARRAY => slice!(nvpair_value_int8_array, I8Array, i8),
            sys::data_type_t::DATA_TYPE_INT16_ARRAY => slice!(nvpair_value_int16_array, I16Array, i16),
            sys::data_type_t::DATA_TYPE_INT32_ARRAY => slice!(nvpair_value_int32_array, I32Array, i32),
            sys::data_type_t::DATA_TYPE_INT64_ARRAY => slice!(nvpair_value_int64_array, I64Array, i64),
            sys::data_type_t::DATA_TYPE_STRING_ARRAY => {
                let mut v: *mut *mut sys::c_char = ptr::null_mut();
                let mut n: sys::uint_t = 0;
                if unsafe { sys::nvpair_value_string_array(p, &mut v, &mut n) } == 0 {
                    NvValue::Strings(NvStrings { data: unsafe { array(v, n) } })
                } else {
                    NvValue::Other(ty)
                }
            }
            other => NvValue::Other(other),
        }
    }
}

impl fmt::Debug for NvPair<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NvPair").field(&self.name()).field(&self.value()).finish()
    }
}

/// The value of a pair, as returned by [`NvPair::value`]. Every variant
/// that carries data borrows it from the list.
#[derive(Clone, Copy, Debug)]
pub enum NvValue<'a> {
    /// A `DATA_TYPE_BOOLEAN` pair, which has no value; its presence means
    /// true.
    Flag,
    /// A `DATA_TYPE_BOOLEAN_VALUE` pair.
    Bool(bool),
    /// A `DATA_TYPE_BYTE` pair. It holds an unsigned byte like
    /// [`NvValue::U8`] but is a distinct type to the nvpair library.
    Byte(u8),
    /// A `DATA_TYPE_UINT8` pair.
    U8(u8),
    /// A `DATA_TYPE_UINT16` pair.
    U16(u16),
    /// A `DATA_TYPE_UINT32` pair.
    U32(u32),
    /// A `DATA_TYPE_UINT64` pair.
    U64(u64),
    /// A `DATA_TYPE_INT8` pair.
    I8(i8),
    /// A `DATA_TYPE_INT16` pair.
    I16(i16),
    /// A `DATA_TYPE_INT32` pair.
    I32(i32),
    /// A `DATA_TYPE_INT64` pair.
    I64(i64),
    /// A `DATA_TYPE_HRTIME` pair, a high-resolution time in nanoseconds.
    HrTime(i64),
    /// A `DATA_TYPE_STRING` pair.
    String(&'a CStr),
    /// A `DATA_TYPE_BYTE_ARRAY` pair.
    Bytes(&'a [u8]),
    /// A `DATA_TYPE_UINT8_ARRAY` pair.
    U8Array(&'a [u8]),
    /// A `DATA_TYPE_UINT16_ARRAY` pair.
    U16Array(&'a [u16]),
    /// A `DATA_TYPE_UINT32_ARRAY` pair.
    U32Array(&'a [u32]),
    /// A `DATA_TYPE_UINT64_ARRAY` pair.
    U64Array(&'a [u64]),
    /// A `DATA_TYPE_INT8_ARRAY` pair.
    I8Array(&'a [i8]),
    /// A `DATA_TYPE_INT16_ARRAY` pair.
    I16Array(&'a [i16]),
    /// A `DATA_TYPE_INT32_ARRAY` pair.
    I32Array(&'a [i32]),
    /// A `DATA_TYPE_INT64_ARRAY` pair.
    I64Array(&'a [i64]),
    /// A `DATA_TYPE_STRING_ARRAY` pair.
    Strings(NvStrings<'a>),
    /// A `DATA_TYPE_NVLIST` pair, a nested list stored inside the outer one.
    NvList(NvListRef<'a>),
    /// A pair of a type this enumeration does not cover, such as a boolean
    /// array or an array of lists. The raw accessors in `sys` can read it
    /// through [`NvPair::as_raw`].
    Other(sys::data_type_t),
}

/// An array of strings stored inside a list, as returned by
/// [`NvListRef::lookup_strings`] and [`NvValue::Strings`].
#[derive(Clone, Copy)]
pub struct NvStrings<'a> {
    data: &'a [*mut sys::c_char],
}

impl<'a> NvStrings<'a> {
    /// Returns how many strings the array holds.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Reports whether the array holds no strings at all.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the string at `i`, or `None` past the end.
    pub fn get(&self, i: usize) -> Option<&'a CStr> {
        self.data.get(i).map(|&s| unsafe { CStr::from_ptr(s) })
    }

    /// Returns an iterator over the strings in order.
    pub fn iter(&self) -> impl Iterator<Item = &'a CStr> + 'a {
        self.data.iter().map(|&s| unsafe { CStr::from_ptr(s) })
    }
}

impl fmt::Debug for NvStrings<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// A packed list in a buffer the kernel allocated for `nvlist_pack(9F)`.
///
/// When `nvlist_pack(9F)` is given no buffer, it allocates one with
/// `kmem_alloc(9F)` under the `kmflag` of the call and reports its length.
/// This value dereferences to those bytes and returns the buffer with
/// `kmem_free(9F)` and that length when dropped, which is allowed in any
/// context. The usual destination of the bytes is a user program, reached
/// through `ddi_copyout(9F)` in an ioctl.
pub struct PackedNvList {
    ptr: NonNull<u8>,
    len: usize,
}

// The buffer is plain kernel memory owned by this value, which only hands
// out shared access to it.
unsafe impl Send for PackedNvList {}
unsafe impl Sync for PackedNvList {}

impl PackedNvList {
    fn pack(list: NonNull<sys::nvlist_t>, encoding: NvEncoding, kmflag: i32) -> Result<PackedNvList, Errno> {
        let mut buf: *mut sys::c_char = ptr::null_mut();
        let mut len: usize = 0;
        Errno::check(unsafe { sys::nvlist_pack(list.as_ptr(), &mut buf, &mut len, encoding.raw(), kmflag) })?;
        let ptr = NonNull::new(buf as *mut u8).ok_or(Errno::ENOMEM)?;
        Ok(PackedNvList { ptr, len })
    }

    /// Returns the number of bytes in the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Reports whether the buffer is empty, which a successful pack never
    /// produces because the encoding always starts with a header.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Deref for PackedNvList {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for PackedNvList {
    fn drop(&mut self) {
        unsafe { sys::kmem_free(self.ptr.as_ptr() as *mut sys::c_void, self.len) }
    }
}

impl fmt::Debug for PackedNvList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackedNvList").field("len", &self.len).finish()
    }
}
