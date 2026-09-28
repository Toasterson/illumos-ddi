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
//!
//! Updates and removals act on the driver property list of the node, which
//! belongs to the driver bound to it: the kernel deletes that list when the
//! node's driver detaches (`detach_node` in `os/devcfg.c`), and other
//! drivers and the framework read their settings from it, for example
//! `pm-components` in `os/sunpm.c`. So only the view from the driver's own
//! node, [`OwnNode::props`], can update; the view from a [`DevInfo`] only
//! looks properties up. The two are the same type, [`Props`], with the
//! access as a type parameter ([`ReadOnly`] or [`ReadWrite`]). The own-node
//! guard applies too: a view from an [`OwnNode`] made for a foreign node
//! with the `unsafe` [`OwnNode::from_raw`] refuses every update with
//! [`PropError::NotOwnNode`] before the kernel is called.
//!
//! Writing properties of another node, as some nexus drivers do for their
//! children and a few leaf drivers for their parent, is out of scope: it
//! needs the raw `ddi::sys` functions until a capability for child nodes
//! exists.

use core::ffi::CStr;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr;

use crate::context::Sleepable;
use crate::device::{DevInfo, Minor, OwnNode};
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
    /// The view was made from an [`OwnNode`] that the own-node guard
    /// refuses, a node without a parent or bound to another driver, so the
    /// update was not attempted.
    NotOwnNode,
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
            PropError::NotOwnNode => f.write_str("property update refused for a node that is not the driver's own"),
            PropError::Other(c) => write!(f, "DDI_PROP_* code {c}"),
        }
    }
}

/// The access of a [`Props`] view from a [`DevInfo`]: lookups only.
#[derive(Clone, Copy, Debug)]
pub enum ReadOnly {}

/// The access of a [`Props`] view from the driver's [`OwnNode`]: lookups,
/// updates and removals.
#[derive(Clone, Copy, Debug)]
pub enum ReadWrite {}

/// A view of the properties of a device node, with a fixed scope.
///
/// The scope is which minor device the properties belong to and whether the
/// lookup may continue to the parent nodes. The default from
/// [`DevInfo::props`] matches any minor and stays on the node itself, which
/// is what a leaf driver reading its `.conf` file wants.
///
/// `A` is the access. A view from [`DevInfo::props`] is [`ReadOnly`]; only
/// the view from [`OwnNode::props`] is [`ReadWrite`] and has the update and
/// removal methods, as the [module documentation](self) explains. A
/// `DevInfo` cannot publish a property:
///
/// ```compile_fail,E0599
/// fn publish(dev: ddi::DevInfo, ctx: &ddi::Sleepable) {
///     let _ = dev.props().set_int(c"x", 1, ctx);
/// }
/// ```
///
/// The driver's own node can, and looks properties up the same way:
///
/// ```no_run
/// fn publish(node: &ddi::OwnNode, ctx: &ddi::Sleepable) {
///     let _ = node.props().set_int(c"x", 1, ctx);
///     let _ = node.dev().props().int(c"x", ctx);
/// }
/// ```
pub struct Props<'a, A = ReadOnly> {
    dip: sys::dev_info_t,
    dev: sys::dev_t,
    flags: sys::uint_t,
    /// Set for a [`ReadWrite`] view whose node the own-node guard refuses.
    refused: bool,
    _node: PhantomData<(&'a DevInfo, A)>,
}

impl<A> Clone for Props<'_, A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for Props<'_, A> {}

impl DevInfo {
    /// Returns the properties of this node, for lookups. They match any
    /// minor device and do not consult the parent nodes; use the methods on
    /// [`Props`] to change either. Updates need the view from
    /// [`OwnNode::props`].
    pub fn props(&self) -> Props<'_> {
        Props {
            dip: self.as_raw(),
            dev: sys::DDI_DEV_T_ANY,
            flags: sys::DDI_PROP_DONTPASS,
            refused: false,
            _node: PhantomData,
        }
    }
}

impl OwnNode {
    /// Returns the properties of the driver's own node, for lookups and for
    /// updates, with the same default scope as [`DevInfo::props`]. If the
    /// own-node guard refuses the node (only possible after the `unsafe`
    /// [`OwnNode::from_raw`]), every update and removal through the view
    /// fails with [`PropError::NotOwnNode`] without calling the kernel.
    pub fn props(&self) -> Props<'_, ReadWrite> {
        Props {
            dip: self.as_raw(),
            dev: sys::DDI_DEV_T_ANY,
            flags: sys::DDI_PROP_DONTPASS,
            refused: !self.is_plausibly_own(),
            _node: PhantomData,
        }
    }
}

impl<'a, A> Props<'a, A> {
    /// Restricts the view to the properties of one minor device of this
    /// driver. Properties created through such a view are tied to that
    /// minor and are what `prop_op(9E)` reports for it.
    pub fn of_minor(self, minor: Minor) -> Props<'a, A> {
        let major = unsafe { sys::ddi_driver_major(self.dip) };
        Props { dev: unsafe { sys::makedevice(major, minor.0) }, ..self }
    }

    /// Lets lookups continue to the parent nodes when this node does not
    /// have the property, by clearing `DDI_PROP_DONTPASS`. Updates are
    /// unaffected; they always act on this node.
    pub fn inherited(self) -> Props<'a, A> {
        Props { flags: self.flags & !sys::DDI_PROP_DONTPASS, ..self }
    }

    /// Ignores properties that came from the firmware, by setting
    /// `DDI_PROP_NOTPROM`.
    pub fn without_firmware(self) -> Props<'a, A> {
        Props { flags: self.flags | sys::DDI_PROP_NOTPROM, ..self }
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
}

impl Props<'_, ReadWrite> {
    /// The `dev_t` that update calls use. It is `DDI_DEV_T_NONE` for the
    /// node itself and the minor's device number after [`Props::of_minor`].
    fn update_dev(&self) -> sys::dev_t {
        if self.dev == sys::DDI_DEV_T_ANY { sys::DDI_DEV_T_NONE } else { self.dev }
    }

    /// The own-node guard of the updates.
    fn guard(&self) -> Result<(), PropError> {
        if self.refused { Err(PropError::NotOwnNode) } else { Ok(()) }
    }

    /// Creates or replaces an integer property with `ddi_prop_update_int(9F)`.
    pub fn set_int(&self, name: &CStr, value: i32, _ctx: &Sleepable) -> Result<(), PropError> {
        self.guard()?;
        PropError::check_update(unsafe { sys::ddi_prop_update_int(self.update_dev(), self.dip, name.as_ptr(), value) })
    }

    /// Creates or replaces a 64-bit integer property with
    /// `ddi_prop_update_int64(9F)`.
    pub fn set_int64(&self, name: &CStr, value: i64, _ctx: &Sleepable) -> Result<(), PropError> {
        self.guard()?;
        PropError::check_update(unsafe {
            sys::ddi_prop_update_int64(self.update_dev(), self.dip, name.as_ptr(), value)
        })
    }

    /// Creates or replaces an integer array property with
    /// `ddi_prop_update_int_array(9F)`.
    pub fn set_int_array(&self, name: &CStr, value: &[i32], _ctx: &Sleepable) -> Result<(), PropError> {
        self.guard()?;
        let n = u32::try_from(value.len()).map_err(|_| PropError::InvalidArg)?;
        PropError::check_update(unsafe {
            sys::ddi_prop_update_int_array(self.update_dev(), self.dip, name.as_ptr(), value.as_ptr(), n)
        })
    }

    /// Creates or replaces a 64-bit integer array property with
    /// `ddi_prop_update_int64_array(9F)`.
    pub fn set_int64_array(&self, name: &CStr, value: &[i64], _ctx: &Sleepable) -> Result<(), PropError> {
        self.guard()?;
        let n = u32::try_from(value.len()).map_err(|_| PropError::InvalidArg)?;
        PropError::check_update(unsafe {
            sys::ddi_prop_update_int64_array(self.update_dev(), self.dip, name.as_ptr(), value.as_ptr(), n)
        })
    }

    /// Creates or replaces a byte array property with
    /// `ddi_prop_update_byte_array(9F)`.
    pub fn set_bytes(&self, name: &CStr, value: &[u8], _ctx: &Sleepable) -> Result<(), PropError> {
        self.guard()?;
        let n = u32::try_from(value.len()).map_err(|_| PropError::InvalidArg)?;
        PropError::check_update(unsafe {
            sys::ddi_prop_update_byte_array(self.update_dev(), self.dip, name.as_ptr(), value.as_ptr(), n)
        })
    }

    /// Creates or replaces a string property with
    /// `ddi_prop_update_string(9F)`.
    pub fn set_string(&self, name: &CStr, value: &CStr, _ctx: &Sleepable) -> Result<(), PropError> {
        self.guard()?;
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
        self.guard()?;
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
    pub fn remove(&self, name: &CStr, _ctx: &Sleepable) -> Result<bool, PropError> {
        self.guard()?;
        PropError::check(unsafe { sys::ddi_prop_remove(self.update_dev(), self.dip, name.as_ptr()) })
    }

    /// Removes every property the driver created on the node, with
    /// `ddi_prop_remove_all(9F)`. Conventionally called from `detach(9E)`;
    /// the kernel does the same after a successful detach.
    pub fn remove_all(&self, _ctx: &Sleepable) -> Result<(), PropError> {
        self.guard()?;
        unsafe { sys::ddi_prop_remove_all(self.dip) };
        Ok(())
    }

    /// Marks the property as undefined on this node, so that lookups stop
    /// here instead of continuing to the parents, with
    /// `ddi_prop_undefine(9F)`.
    pub fn undefine(&self, name: &CStr, _ctx: &Sleepable) -> Result<(), PropError> {
        self.guard()?;
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
