//! Device properties with the rules of `ddi_prop_lookup(9F)` and
//! `ddi_prop_update(9F)` in the types.
//!
//! A property is a named value on a device node. The driver's `.conf` file
//! is the usual source, which is how an administrator passes settings to a
//! driver, and the driver can publish values of its own for tools such as
//! `prtconf -v` to show. Every property function in the manual may only be
//! called from user or kernel context, so every method here that reaches the
//! kernel takes a [`Sleepable`] token.
//!
//! Lookups go through [`Props`], which a driver obtains from
//! [`DevInfo::props`]. A lookup returns `Ok(None)` when the property does
//! not exist, `Ok(Some(value))` when it does, and `Err` only when the kernel
//! reported a real problem such as a type mismatch or an invalid argument.
//! Values the kernel allocates come back as [`PropBuf`] or [`PropStrings`],
//! which release the buffer with `ddi_prop_free(9F)` when dropped.
//!
//! Updates create or replace a property on the node. The kernel copies the
//! value, so the driver's data need not outlive the call. Removing all of a
//! driver's properties in `detach(9E)` is the convention, and
//! [`Props::remove_all`] exists for exactly that.

use core::ffi::CStr;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr;

use crate::context::Sleepable;
use crate::device::{DevInfo, Minor};
use crate::sys;

/// What went wrong in a property call, as the `DDI_PROP_*` codes describe
/// it. `DDI_PROP_NOT_FOUND` and `DDI_PROP_UNDEFINED` never appear here;
/// the lookup methods turn them into `Ok(None)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropError {
    /// Memory for the value could not be allocated (`DDI_PROP_NO_MEMORY`).
    NoMemory,
    /// An argument was invalid, typically an empty name or a NULL device
    /// node (`DDI_PROP_INVAL_ARG`).
    InvalidArg,
    /// The property exists but holds a different type than the one
    /// requested (`DDI_PROP_CANNOT_DECODE`).
    WrongType,
    /// The value could not be encoded for storage (`DDI_PROP_CANNOT_ENCODE`).
    CannotEncode,
    /// A code this crate does not know.
    Other(i32),
}

impl PropError {
    /// Interprets a `DDI_PROP_*` return code, treating "not found" and
    /// "undefined" as the absence of a value rather than as an error.
    pub(crate) fn check(raw: i32) -> Result<bool, PropError> {
        match raw {
            sys::DDI_PROP_SUCCESS => Ok(true),
            sys::DDI_PROP_NOT_FOUND | sys::DDI_PROP_UNDEFINED => Ok(false),
            sys::DDI_PROP_NO_MEMORY => Err(PropError::NoMemory),
            sys::DDI_PROP_INVAL_ARG => Err(PropError::InvalidArg),
            sys::DDI_PROP_CANNOT_DECODE => Err(PropError::WrongType),
            sys::DDI_PROP_CANNOT_ENCODE => Err(PropError::CannotEncode),
            other => Err(PropError::Other(other)),
        }
    }

    /// Interprets the return code of an update, where anything but success
    /// is an error.
    fn check_update(raw: i32) -> Result<(), PropError> {
        match Self::check(raw) {
            Ok(true) => Ok(()),
            Ok(false) => Err(PropError::Other(raw)),
            Err(e) => Err(e),
        }
    }
}

impl fmt::Display for PropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropError::NoMemory => f.write_str("DDI_PROP_NO_MEMORY"),
            PropError::InvalidArg => f.write_str("DDI_PROP_INVAL_ARG"),
            PropError::WrongType => f.write_str("DDI_PROP_CANNOT_DECODE"),
            PropError::CannotEncode => f.write_str("DDI_PROP_CANNOT_ENCODE"),
            PropError::Other(c) => write!(f, "DDI_PROP_* code {c}"),
        }
    }
}

/// A view of the properties of a device node, with a fixed scope.
///
/// The scope is which minor device the properties belong to and whether the
/// lookup may continue to the parent nodes. The default from
/// [`DevInfo::props`] matches any minor and stays on the node itself, which
/// is what a leaf driver reading its `.conf` file wants.
#[derive(Clone, Copy)]
pub struct Props<'a> {
    dip: sys::dev_info_t,
    dev: sys::dev_t,
    flags: sys::uint_t,
    _node: PhantomData<&'a DevInfo>,
}

impl DevInfo {
    /// Returns the properties of this node. Lookups match any minor device
    /// and do not consult the parent nodes; use the methods on [`Props`] to
    /// change either.
    pub fn props(&self) -> Props<'_> {
        Props { dip: self.as_raw(), dev: sys::DDI_DEV_T_ANY, flags: sys::DDI_PROP_DONTPASS, _node: PhantomData }
    }
}

impl<'a> Props<'a> {
    /// Restricts the view to the properties of one minor device of this
    /// driver. Properties created through such a view are tied to that
    /// minor and are what `prop_op(9E)` reports for it.
    pub fn of_minor(self, minor: Minor) -> Props<'a> {
        let major = unsafe { sys::ddi_driver_major(self.dip) };
        Props { dev: unsafe { sys::makedevice(major, minor.0) }, ..self }
    }

    /// Lets lookups continue to the parent nodes when this node does not
    /// have the property, by clearing `DDI_PROP_DONTPASS`. Updates are
    /// unaffected; they always act on this node.
    pub fn inherited(self) -> Props<'a> {
        Props { flags: self.flags & !sys::DDI_PROP_DONTPASS, ..self }
    }

    /// Ignores properties that came from the firmware, by setting
    /// `DDI_PROP_NOTPROM`.
    pub fn without_firmware(self) -> Props<'a> {
        Props { flags: self.flags | sys::DDI_PROP_NOTPROM, ..self }
    }

    /// The `dev_t` that update calls use. It is `DDI_DEV_T_NONE` for the
    /// node itself and the minor's device number after [`Props::of_minor`].
    fn update_dev(&self) -> sys::dev_t {
        if self.dev == sys::DDI_DEV_T_ANY { sys::DDI_DEV_T_NONE } else { self.dev }
    }

    /// Reports whether the named property exists, using
    /// `ddi_prop_exists(9F)`.
    pub fn exists(&self, name: &CStr, _ctx: &Sleepable) -> bool {
        unsafe { sys::ddi_prop_exists(self.dev, self.dip, self.flags, name.as_ptr()) != 0 }
    }

    /// Reads an integer property, returning `default` when it does not
    /// exist or is not an integer, using `ddi_prop_get_int(9F)`. A property
    /// written without a value in a `.conf` file reads as 1.
    pub fn int_or(&self, name: &CStr, default: i32, _ctx: &Sleepable) -> i32 {
        unsafe { sys::ddi_prop_get_int(self.dev, self.dip, self.flags, name.as_ptr(), default) }
    }

    /// Reads a 64-bit integer property, returning `default` when it does
    /// not exist or is not a 64-bit integer, using `ddi_prop_get_int64(9F)`.
    pub fn int64_or(&self, name: &CStr, default: i64, _ctx: &Sleepable) -> i64 {
        unsafe { sys::ddi_prop_get_int64(self.dev, self.dip, self.flags, name.as_ptr(), default) }
    }

    /// Reads an integer property, distinguishing absence from any value,
    /// using `ddi_prop_lookup_int_array(9F)` and taking the first element.
    pub fn int(&self, name: &CStr, ctx: &Sleepable) -> Result<Option<i32>, PropError> {
        Ok(self.int_array(name, ctx)?.and_then(|a| a.first().copied()))
    }

    /// Reads a 64-bit integer property, distinguishing absence from any
    /// value, using `ddi_prop_lookup_int64_array(9F)`.
    pub fn int64(&self, name: &CStr, ctx: &Sleepable) -> Result<Option<i64>, PropError> {
        Ok(self.int64_array(name, ctx)?.and_then(|a| a.first().copied()))
    }

    /// Reads an integer array property with `ddi_prop_lookup_int_array(9F)`.
    pub fn int_array(&self, name: &CStr, _ctx: &Sleepable) -> Result<Option<PropBuf<i32>>, PropError> {
        let mut data: *mut i32 = ptr::null_mut();
        let mut n: u32 = 0;
        let r =
            unsafe { sys::ddi_prop_lookup_int_array(self.dev, self.dip, self.flags, name.as_ptr(), &mut data, &mut n) };
        Ok(PropError::check(r)?.then(|| PropBuf { data, len: n as usize }))
    }

    /// Reads a 64-bit integer array property with
    /// `ddi_prop_lookup_int64_array(9F)`.
    pub fn int64_array(&self, name: &CStr, _ctx: &Sleepable) -> Result<Option<PropBuf<i64>>, PropError> {
        let mut data: *mut i64 = ptr::null_mut();
        let mut n: u32 = 0;
        let r = unsafe {
            sys::ddi_prop_lookup_int64_array(self.dev, self.dip, self.flags, name.as_ptr(), &mut data, &mut n)
        };
        Ok(PropError::check(r)?.then(|| PropBuf { data, len: n as usize }))
    }

    /// Reads a byte array property with `ddi_prop_lookup_byte_array(9F)`.
    pub fn bytes(&self, name: &CStr, _ctx: &Sleepable) -> Result<Option<PropBuf<u8>>, PropError> {
        let mut data: *mut u8 = ptr::null_mut();
        let mut n: u32 = 0;
        let r = unsafe {
            sys::ddi_prop_lookup_byte_array(self.dev, self.dip, self.flags, name.as_ptr(), &mut data, &mut n)
        };
        Ok(PropError::check(r)?.then(|| PropBuf { data, len: n as usize }))
    }

    /// Reads a string property with `ddi_prop_lookup_string(9F)`.
    pub fn string(&self, name: &CStr, _ctx: &Sleepable) -> Result<Option<PropString>, PropError> {
        let mut data: *mut sys::c_char = ptr::null_mut();
        let r = unsafe { sys::ddi_prop_lookup_string(self.dev, self.dip, self.flags, name.as_ptr(), &mut data) };
        Ok(PropError::check(r)?.then(|| PropString { data }))
    }

    /// Reads a string array property with
    /// `ddi_prop_lookup_string_array(9F)`.
    pub fn strings(&self, name: &CStr, _ctx: &Sleepable) -> Result<Option<PropStrings>, PropError> {
        let mut data: *mut *mut sys::c_char = ptr::null_mut();
        let mut n: u32 = 0;
        let r = unsafe {
            sys::ddi_prop_lookup_string_array(self.dev, self.dip, self.flags, name.as_ptr(), &mut data, &mut n)
        };
        Ok(PropError::check(r)?.then(|| PropStrings { data, len: n as usize }))
    }

    /// Creates or replaces an integer property with `ddi_prop_update_int(9F)`.
    pub fn set_int(&self, name: &CStr, value: i32, _ctx: &Sleepable) -> Result<(), PropError> {
        PropError::check_update(unsafe { sys::ddi_prop_update_int(self.update_dev(), self.dip, name.as_ptr(), value) })
    }

    /// Creates or replaces a 64-bit integer property with
    /// `ddi_prop_update_int64(9F)`.
    pub fn set_int64(&self, name: &CStr, value: i64, _ctx: &Sleepable) -> Result<(), PropError> {
        PropError::check_update(unsafe {
            sys::ddi_prop_update_int64(self.update_dev(), self.dip, name.as_ptr(), value)
        })
    }

    /// Creates or replaces an integer array property with
    /// `ddi_prop_update_int_array(9F)`.
    pub fn set_int_array(&self, name: &CStr, value: &[i32], _ctx: &Sleepable) -> Result<(), PropError> {
        let n = u32::try_from(value.len()).map_err(|_| PropError::InvalidArg)?;
        PropError::check_update(unsafe {
            sys::ddi_prop_update_int_array(self.update_dev(), self.dip, name.as_ptr(), value.as_ptr(), n)
        })
    }

    /// Creates or replaces a 64-bit integer array property with
    /// `ddi_prop_update_int64_array(9F)`.
    pub fn set_int64_array(&self, name: &CStr, value: &[i64], _ctx: &Sleepable) -> Result<(), PropError> {
        let n = u32::try_from(value.len()).map_err(|_| PropError::InvalidArg)?;
        PropError::check_update(unsafe {
            sys::ddi_prop_update_int64_array(self.update_dev(), self.dip, name.as_ptr(), value.as_ptr(), n)
        })
    }

    /// Creates or replaces a byte array property with
    /// `ddi_prop_update_byte_array(9F)`.
    pub fn set_bytes(&self, name: &CStr, value: &[u8], _ctx: &Sleepable) -> Result<(), PropError> {
        let n = u32::try_from(value.len()).map_err(|_| PropError::InvalidArg)?;
        PropError::check_update(unsafe {
            sys::ddi_prop_update_byte_array(self.update_dev(), self.dip, name.as_ptr(), value.as_ptr(), n)
        })
    }

    /// Creates or replaces a string property with
    /// `ddi_prop_update_string(9F)`.
    pub fn set_string(&self, name: &CStr, value: &CStr, _ctx: &Sleepable) -> Result<(), PropError> {
        PropError::check_update(unsafe {
            sys::ddi_prop_update_string(self.update_dev(), self.dip, name.as_ptr(), value.as_ptr())
        })
    }

    /// Creates or replaces a string array property with
    /// `ddi_prop_update_string_array(9F)`. Up to sixteen strings are
    /// handled without allocating; longer arrays allocate the pointer table
    /// with `kmem_alloc(9F)`, which is why the sleepable token is needed
    /// here as much as anywhere.
    pub fn set_strings(&self, name: &CStr, value: &[&CStr], _ctx: &Sleepable) -> Result<(), PropError> {
        let n = u32::try_from(value.len()).map_err(|_| PropError::InvalidArg)?;
        let mut small: [*const sys::c_char; 16] = [ptr::null(); 16];
        if value.len() <= small.len() {
            for (slot, s) in small.iter_mut().zip(value) {
                *slot = s.as_ptr();
            }
            return PropError::check_update(unsafe {
                sys::ddi_prop_update_string_array(self.update_dev(), self.dip, name.as_ptr(), small.as_ptr(), n)
            });
        }
        let bytes = value.len() * core::mem::size_of::<*const sys::c_char>();
        let table = unsafe { sys::kmem_alloc(bytes, sys::KM_SLEEP) } as *mut *const sys::c_char;
        for (i, s) in value.iter().enumerate() {
            unsafe { table.add(i).write(s.as_ptr()) };
        }
        let r = unsafe { sys::ddi_prop_update_string_array(self.update_dev(), self.dip, name.as_ptr(), table, n) };
        unsafe { sys::kmem_free(table as *mut sys::c_void, bytes) };
        PropError::check_update(r)
    }

    /// Removes one property the driver created, with `ddi_prop_remove(9F)`.
    /// Returns whether it existed.
    pub fn remove(&self, name: &CStr, _ctx: &Sleepable) -> bool {
        unsafe { sys::ddi_prop_remove(self.update_dev(), self.dip, name.as_ptr()) == sys::DDI_PROP_SUCCESS }
    }

    /// Removes every property the driver created on the node, with
    /// `ddi_prop_remove_all(9F)`. Conventionally called from `detach(9E)`.
    pub fn remove_all(&self, _ctx: &Sleepable) {
        unsafe { sys::ddi_prop_remove_all(self.dip) }
    }

    /// Marks the property as undefined on this node, so that lookups stop
    /// here instead of continuing to the parents, with
    /// `ddi_prop_undefine(9F)`.
    pub fn undefine(&self, name: &CStr, _ctx: &Sleepable) -> Result<(), PropError> {
        PropError::check_update(unsafe {
            sys::ddi_prop_undefine(self.update_dev(), self.dip, sys::DDI_PROP_CANSLEEP as i32, name.as_ptr())
        })
    }
}

/// An array value the kernel allocated for a lookup. It dereferences to a
/// slice and is released with `ddi_prop_free(9F)` when dropped.
pub struct PropBuf<T> {
    data: *mut T,
    len: usize,
}

impl<T> PropBuf<T> {
    /// Takes ownership of an array the kernel allocated for a property
    /// lookup.
    ///
    /// # Safety
    /// `data` must come from a successful `ddi_prop_lookup_*` or
    /// `ldi_prop_lookup_*` call that reported `len` elements, and must not be
    /// freed elsewhere.
    pub(crate) unsafe fn from_raw(data: *mut T, len: usize) -> PropBuf<T> {
        PropBuf { data, len }
    }
}

unsafe impl<T: Send> Send for PropBuf<T> {}
unsafe impl<T: Sync> Sync for PropBuf<T> {}

impl<T> Deref for PropBuf<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        if self.len == 0 { &[] } else { unsafe { core::slice::from_raw_parts(self.data, self.len) } }
    }
}

impl<T> Drop for PropBuf<T> {
    fn drop(&mut self) {
        unsafe { sys::ddi_prop_free(self.data as *mut sys::c_void) }
    }
}

impl<T: fmt::Debug> fmt::Debug for PropBuf<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

/// A string value the kernel allocated for a lookup. It dereferences to a
/// `CStr` and is released with `ddi_prop_free(9F)` when dropped.
pub struct PropString {
    data: *mut sys::c_char,
}

impl PropString {
    /// Takes ownership of a string the kernel allocated for a property
    /// lookup.
    ///
    /// # Safety
    /// `data` must be a non-NULL string from a successful
    /// `ddi_prop_lookup_string` or `ldi_prop_lookup_string` call, and must not
    /// be freed elsewhere.
    pub(crate) unsafe fn from_raw(data: *mut sys::c_char) -> PropString {
        PropString { data }
    }
}

unsafe impl Send for PropString {}
unsafe impl Sync for PropString {}

impl Deref for PropString {
    type Target = CStr;
    fn deref(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.data) }
    }
}

impl Drop for PropString {
    fn drop(&mut self) {
        unsafe { sys::ddi_prop_free(self.data as *mut sys::c_void) }
    }
}

impl fmt::Debug for PropString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

/// A string array value the kernel allocated for a lookup. The strings are
/// reached by index or by iteration, and the whole value is released with
/// one `ddi_prop_free(9F)` when dropped.
pub struct PropStrings {
    data: *mut *mut sys::c_char,
    len: usize,
}

impl PropStrings {
    /// Takes ownership of a string array the kernel allocated for a
    /// property lookup.
    ///
    /// # Safety
    /// `data` must come from a successful `ddi_prop_lookup_string_array` or
    /// `ldi_prop_lookup_string_array` call that reported `len` strings, and
    /// must not be freed elsewhere.
    pub(crate) unsafe fn from_raw(data: *mut *mut sys::c_char, len: usize) -> PropStrings {
        PropStrings { data, len }
    }
}

unsafe impl Send for PropStrings {}
unsafe impl Sync for PropStrings {}

impl PropStrings {
    /// How many strings the property holds.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the property holds no strings at all.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The string at `i`, or `None` past the end.
    pub fn get(&self, i: usize) -> Option<&CStr> {
        (i < self.len).then(|| unsafe { CStr::from_ptr(*self.data.add(i)) })
    }

    /// Iterates over the strings in order.
    pub fn iter(&self) -> impl Iterator<Item = &CStr> + '_ {
        (0..self.len).map(move |i| unsafe { CStr::from_ptr(*self.data.add(i)) })
    }
}

impl Drop for PropStrings {
    fn drop(&mut self) {
        unsafe { sys::ddi_prop_free(self.data as *mut sys::c_void) }
    }
}

impl fmt::Debug for PropStrings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
