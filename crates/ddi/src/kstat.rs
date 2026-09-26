//! Kernel statistics that a driver publishes through `kstat(9S)`.
//!
//! A kstat is a named block of data that user programs such as `kstat(8)`
//! and `iostat(8)` read through `/dev/kstat`. The kernel identifies each
//! kstat by a module name, an instance number and a name, and files it under
//! a class such as `misc`, `net` or `disk`. [`KstatId`] carries those four
//! values.
//!
//! In C, publishing a kstat is a sequence of steps whose order matters. The
//! driver calls `kstat_create(9F)`, names each entry of the data section
//! with `kstat_named_init(9F)`, points `ks_lock` at a mutex if it wants
//! consistent snapshots, calls `kstat_install(9F)` to make the kstat
//! visible, updates the data under the mutex from then on, and calls
//! `kstat_delete(9F)` without holding the mutex before the mutex and the
//! driver's state go away. The types in this module do all of that in their
//! constructors and destructors, so the order cannot be got wrong.
//!
//! Each kstat owns a [`Mutex`] in memory from `kmem_alloc(9F)` and stores
//! its address in `ks_lock`. The kernel takes that mutex whenever a reader
//! takes a snapshot, and the driver takes it by calling `lock`, which is
//! the only way to reach the data. The guard that `lock` returns is
//! therefore proof that no snapshot is in progress, and a reader never sees
//! a half-updated set of counters. Dropping the kstat calls
//! `kstat_delete(9F)` first, which waits for readers to finish, and only
//! then destroys the mutex. The borrow held by the guard makes it impossible
//! to drop the kstat while the mutex is held, which `kstat_delete(9F)` would
//! punish with a system panic.
//!
//! The mutex is an ordinary adaptive mutex unless the kstat is created with
//! a `with_priority` constructor. A driver that updates its statistics from
//! an interrupt handler must use `with_priority` with that handler's
//! [`IntrPriority`], for the reasons given in [`crate::sync`].
//!
//! Three kinds of kstat are offered.
//!
//! - [`NamedKstat`] is a named kstat, `KSTAT_TYPE_NAMED`, whose entries are
//!   given as a list of names and types when it is created and addressed by
//!   index afterwards. Each access checks the index and the type.
//! - [`TypedKstat`] is a named kstat whose entries are the fields of a
//!   structure declared with [`named_kstat!`](crate::named_kstat!). The
//!   driver updates them by name, and the types are checked by the compiler.
//! - [`IoKstat`] is an I/O kstat, `KSTAT_TYPE_IO`, with the byte and
//!   operation counters and the queue accounting of `kstat_queue(9F)`.
//!
//! String entries, `KSTAT_DATA_STRING`, are not offered. The kernel keeps
//! only a pointer to the string, and a sound interface would have to tie
//! the lifetime of the string to the kstat, which the simple interfaces here
//! cannot. The obsolete floating point types and the kernel's private
//! `KSTAT_DATA_LONG` and `KSTAT_DATA_ULONG` encodings are not offered
//! either. The kstats are never virtual, persistent or writable, and they
//! have no `ks_update(9E)` or `ks_snapshot(9E)` callback. The kernel
//! allocates and zeroes the data section, and user programs can only read
//! it.
//!
//! Creating a kstat may allocate memory and `kstat_create(9F)` may only be
//! called from user or kernel context, so every constructor requires a
//! [`Sleepable`] token. A driver creates its kstats in `attach(9E)` and
//! keeps them in its instance, so that they disappear in `detach(9E)`.
//! When `kstat_create(9F)` fails it returns NULL without saying why. The
//! usual reasons are a kstat with the same module, instance and name that
//! already exists, an invalid argument, and a lack of memory, and the kernel
//! logs a warning for each. The constructors report the failure as
//! [`DdiError`], which likewise carries no reason, rather than guessing at
//! an errno.
//!
//! A named kstat for a pseudo device might look like this.
//!
//! ```ignore
//! ddi::named_kstat! {
//!     /// The statistics of one rnull instance.
//!     pub struct RNullStats {
//!         pub opens: u64,
//!         pub bytes_discarded: u64 = "discarded",
//!     }
//! }
//!
//! let ks = TypedKstat::<RNullStats>::new(
//!     KstatId { module: c"rnull", instance: dev.instance(), name: c"stats", class: c"misc" },
//!     ctx,
//! )?;
//! ks.lock().opens.inc();
//! ```

use core::ffi::{CStr, c_char};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use crate::context::Sleepable;
use crate::device::Instance;
use crate::error::DdiError;
use crate::intr::IntrPriority;
use crate::kmem::KBox;
use crate::sync::{Mutex, MutexGuard};
use crate::sys;

/// The four values that identify a kstat and file it in a class.
///
/// `kstat_create(9F)` copies all four strings, so they need only live for
/// the duration of the call. Only the first 30 bytes of each string are
/// significant, which is `KSTAT_STRLEN` minus the terminating NUL; longer
/// strings are truncated.
#[derive(Clone, Copy, Debug)]
pub struct KstatId<'a> {
    /// The name of the providing module, conventionally the driver name as
    /// [`DevInfo::driver_name`](crate::DevInfo::driver_name) reports it.
    pub module: &'a CStr,
    /// The instance number of the provider, conventionally the device
    /// instance as [`DevInfo::instance`](crate::DevInfo::instance) reports
    /// it.
    pub instance: Instance,
    /// The name of the kstat, unique for the module and instance.
    pub name: &'a CStr,
    /// The class of the kstat, a general category such as `misc`, `net`,
    /// `disk` or `controller`.
    pub class: &'a CStr,
}

/// The type of one entry of a named kstat, one of the `KSTAT_DATA_*` values
/// that `kstat_named_init(9F)` accepts.
///
/// Only the fixed-size types that a driver can update without further
/// obligations are listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KstatDataType {
    /// Sixteen bytes of characters, for example a short identifier or a
    /// 128-bit value (`KSTAT_DATA_CHAR`).
    Char,
    /// A signed 32-bit integer (`KSTAT_DATA_INT32`).
    Int32,
    /// An unsigned 32-bit integer (`KSTAT_DATA_UINT32`).
    Uint32,
    /// A signed 64-bit integer (`KSTAT_DATA_INT64`).
    Int64,
    /// An unsigned 64-bit integer, the usual type for counters
    /// (`KSTAT_DATA_UINT64`).
    Uint64,
}

impl KstatDataType {
    /// Returns the `KSTAT_DATA_*` value.
    pub const fn raw(self) -> u8 {
        match self {
            KstatDataType::Char => sys::KSTAT_DATA_CHAR,
            KstatDataType::Int32 => sys::KSTAT_DATA_INT32,
            KstatDataType::Uint32 => sys::KSTAT_DATA_UINT32,
            KstatDataType::Int64 => sys::KSTAT_DATA_INT64,
            KstatDataType::Uint64 => sys::KSTAT_DATA_UINT64,
        }
    }
}

mod sealed {
    use crate::sys;

    /// The conversions between a Rust value and the matching member of the
    /// `value` union of a `kstat_named(9S)`. Kept private so that the set
    /// of types stays exactly the one this module has checked.
    pub trait Sealed: Copy {
        fn load(v: &sys::kstat_named_value) -> Self;
        fn store(v: &mut sys::kstat_named_value, x: Self);
    }
}

/// A Rust type that can be the value of a named kstat entry.
///
/// It is implemented for `i32`, `u32`, `i64`, `u64` and `[u8; 16]`, which
/// correspond to the variants of [`KstatDataType`], and cannot be
/// implemented outside this crate.
pub trait KstatValue: sealed::Sealed {
    /// The `KSTAT_DATA_*` type that `kstat_named_init(9F)` records for an
    /// entry holding this type.
    const TYPE: KstatDataType;
}

macro_rules! kstat_values {
    ($($t:ty => $member:ident, $kind:ident;)*) => {
        $(
            impl sealed::Sealed for $t {
                fn load(v: &sys::kstat_named_value) -> Self {
                    // SAFETY: Every member of the union is plain data of at
                    // most 16 bytes, the kernel zeroes the data section when
                    // it creates it, and every bit pattern is a valid
                    // integer, so reading any member is defined.
                    unsafe { v.$member }
                }
                fn store(v: &mut sys::kstat_named_value, x: Self) {
                    v.$member = x;
                }
            }
            impl KstatValue for $t {
                const TYPE: KstatDataType = KstatDataType::$kind;
            }
        )*
    };
}

kstat_values! {
    i32 => i32, Int32;
    u32 => ui32, Uint32;
    i64 => i64, Int64;
    u64 => ui64, Uint64;
}

impl sealed::Sealed for [u8; 16] {
    fn load(v: &sys::kstat_named_value) -> Self {
        // SAFETY: As for the integer types; the character array covers the
        // whole union.
        unsafe { v.c }.map(|b| b as u8)
    }
    fn store(v: &mut sys::kstat_named_value, x: Self) {
        v.c = x.map(|b| b as c_char);
    }
}

impl KstatValue for [u8; 16] {
    const TYPE: KstatDataType = KstatDataType::Char;
}

/// One entry of a named kstat, a `kstat_named(9S)` whose value is a `T`.
///
/// A driver never creates one. It reaches the entries of a kstat through
/// the guard that [`TypedKstat::lock`] or [`NamedKstat::lock`] returns, so
/// every access happens with the kstat's mutex held. The type has the same
/// layout as `kstat_named_t`, which is what lets a structure of such
/// entries overlay the data section that `kstat_create(9F)` allocates.
#[repr(transparent)]
pub struct KstatNamed<T: KstatValue> {
    raw: sys::kstat_named_t,
    _value: PhantomData<T>,
}

impl<T: KstatValue> KstatNamed<T> {
    /// Returns the name that `kstat_named_init(9F)` stored in the entry.
    pub fn name(&self) -> &CStr {
        // SAFETY: `c_char` and `u8` have the same size and alignment, and
        // the array is initialised.
        let bytes: &[u8; sys::KSTAT_STRLEN] = unsafe { &*(&raw const self.raw.name).cast() };
        // `kstat_named_init(9F)` always terminates the name, but a missing
        // NUL is handled rather than trusted.
        CStr::from_bytes_until_nul(bytes).unwrap_or(c"")
    }

    /// Returns the current value.
    pub fn get(&self) -> T {
        T::load(&self.raw.value)
    }

    /// Replaces the value.
    pub fn set(&mut self, value: T) {
        T::store(&mut self.raw.value, value)
    }
}

macro_rules! counter_ops {
    ($($t:ty),*) => {
        $(
            impl KstatNamed<$t> {
                /// Adds `delta` to the value, wrapping around on overflow
                /// as the counters of C drivers do.
                pub fn add(&mut self, delta: $t) {
                    let v = self.get();
                    self.set(v.wrapping_add(delta));
                }

                /// Adds one to the value, wrapping around on overflow.
                pub fn inc(&mut self) {
                    self.add(1)
                }
            }
        )*
    };
}

counter_ops!(i32, u32, i64, u64);

/// A structure whose fields are the entries of a named kstat.
///
/// Implement it with [`named_kstat!`](crate::named_kstat!), which declares
/// the structure and the list of names in one go, rather than by hand.
///
/// # Safety
///
/// `Self` must be `#[repr(C)]` and consist of exactly `FIELDS.len()`
/// fields, each of type [`KstatNamed<T>`] for some [`KstatValue`] `T`,
/// and nothing else. The field at position `i` must have
/// `T::TYPE == FIELDS[i].1`. [`TypedKstat`] relies on this to treat the
/// data section of the kstat as a `Self`.
pub unsafe trait KstatFields: Sized {
    /// The name and type of each field, in declaration order. The names are
    /// what `kstat(8)` shows; only the first 30 bytes of each are
    /// significant.
    const FIELDS: &'static [(&'static CStr, KstatDataType)];
}

/// Declares a structure of named kstat entries and implements
/// [`KstatFields`](crate::kstat::KstatFields) for it.
///
/// Each field is written as a name, a colon and one of the value types
/// `u64`, `i64`, `u32`, `i32` or `[u8; 16]`. The macro turns the type into
/// [`KstatNamed`](crate::kstat::KstatNamed) of that type and makes the
/// structure `#[repr(C)]`. The kstat entry takes the field's name unless a
/// string literal follows the type after an equals sign, which is useful
/// for names that are not Rust identifiers. A name longer than 30 bytes is
/// rejected at compile time.
///
/// ```ignore
/// ddi::named_kstat! {
///     /// The statistics of one instance.
///     pub struct Stats {
///         pub opens: u64,
///         pub errors: u32,
///         pub link_speed: u64 = "link-speed",
///     }
/// }
/// ```
#[macro_export]
macro_rules! named_kstat {
    (@name $field:ident) => {
        $crate::named_kstat!(@cstr ::core::concat!(::core::stringify!($field), "\0"))
    };
    (@name $field:ident $kname:literal) => {
        $crate::named_kstat!(@cstr ::core::concat!($kname, "\0"))
    };
    (@cstr $s:expr) => {{
        const S: &str = $s;
        ::core::assert!(S.len() <= $crate::sys::KSTAT_STRLEN, "a kstat name may be at most 30 bytes long");
        match ::core::ffi::CStr::from_bytes_with_nul(S.as_bytes()) {
            ::core::result::Result::Ok(c) => c,
            ::core::result::Result::Err(_) => ::core::panic!("a kstat name must not contain a NUL byte"),
        }
    }};
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$fmeta:meta])*
                $fvis:vis $field:ident : $ty:ty $(= $kname:literal)?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(C)]
        $vis struct $name {
            $(
                $(#[$fmeta])*
                $fvis $field: $crate::kstat::KstatNamed<$ty>,
            )*
        }

        // SAFETY: The structure is `#[repr(C)]` and consists of exactly one
        // `KstatNamed` field per entry of `FIELDS`, in the same order, and
        // each entry's type is taken from the field's value type.
        unsafe impl $crate::kstat::KstatFields for $name {
            const FIELDS: &'static [(&'static ::core::ffi::CStr, $crate::kstat::KstatDataType)] = &[
                $(
                    (
                        $crate::named_kstat!(@name $field $($kname)?),
                        <$ty as $crate::kstat::KstatValue>::TYPE,
                    ),
                )*
            ];
        }
    };
}

/// The part every kstat type shares. It owns the kstat header and the
/// mutex whose address is stored in `ks_lock`, and it deletes the kstat
/// before it destroys the mutex.
struct RawKstat {
    ksp: NonNull<sys::kstat_t>,
    // Declared after `ksp` and dropped only after `Drop::drop` has deleted
    // the kstat, so the kernel never sees a `ks_lock` that points at freed
    // memory.
    lock: KBox<Mutex<()>>,
}

impl RawKstat {
    /// Creates the kstat with `kstat_create(9F)` and points `ks_lock` at a
    /// new mutex. The kstat is not installed yet.
    fn create(
        id: &KstatId<'_>,
        ks_type: u8,
        ndata: u32,
        pri: Option<IntrPriority>,
        ctx: &Sleepable,
    ) -> Result<RawKstat, DdiError> {
        let lock = KBox::new(
            match pri {
                Some(p) => Mutex::with_priority((), p),
                None => Mutex::new(()),
            },
            ctx,
        );
        // SAFETY: All four strings are NUL-terminated, and the kernel copies
        // them. The flags are 0, which asks for a kstat whose data section
        // the kernel allocates.
        let ksp = unsafe {
            sys::kstat_create(id.module.as_ptr(), id.instance.0, id.name.as_ptr(), id.class.as_ptr(), ks_type, ndata, 0)
        };
        let ksp = NonNull::new(ksp).ok_or(DdiError)?;
        let raw = RawKstat { ksp, lock };
        // SAFETY: `kstat_create(9F)` returned a valid header that is not
        // installed, so no reader looks at it yet, and the manual allows the
        // driver to set `ks_lock` until `kstat_install(9F)`. The mutex lives
        // in its own allocation, so its address is stable for as long as
        // `raw` exists, and `raw` deletes the kstat before freeing it.
        unsafe { (&raw mut (*ksp.as_ptr()).ks_lock).write(raw.lock.raw()) };
        if raw.data().is_null() {
            // Not expected for a kstat that is not virtual, but the data
            // section is about to be written through, so it is checked.
            return Err(DdiError);
        }
        Ok(raw)
    }

    /// Returns the data section.
    fn data(&self) -> *mut sys::c_void {
        // SAFETY: The header is valid until `Drop` deletes it, and the
        // kernel does not change `ks_data` of a kstat that is not virtual.
        unsafe { (&raw const (*self.ksp.as_ptr()).ks_data).read() }
    }

    /// Makes the kstat visible with `kstat_install(9F)`.
    fn install(&self) {
        // SAFETY: The header is valid, and the caller has initialised the
        // data section.
        unsafe { sys::kstat_install(self.ksp.as_ptr()) }
    }

    /// Takes the mutex that the kernel also takes around every snapshot.
    fn enter(&self) -> MutexGuard<'_, ()> {
        self.lock.lock()
    }
}

impl Drop for RawKstat {
    fn drop(&mut self) {
        // SAFETY: The header came from `kstat_create(9F)` and is deleted
        // exactly once. `&mut self` proves that no guard exists, so the
        // current thread does not hold `ks_lock`, which `kstat_delete(9F)`
        // requires. The mutex is dropped after this returns, when the
        // kernel no longer refers to it.
        unsafe { sys::kstat_delete(self.ksp.as_ptr()) }
    }
}

// SAFETY: The kstat header belongs to this value from `kstat_create(9F)`
// until `kstat_delete(9F)`, which may be called from any thread. The data
// section is only touched with the mutex held.
unsafe impl Send for RawKstat {}
// SAFETY: Shared access only reaches the data section through `enter`,
// which serialises all access with the kernel's own readers.
unsafe impl Sync for RawKstat {}

/// Creates a named kstat, names its entries and installs it.
fn create_named(
    id: &KstatId<'_>,
    entries: &[(&CStr, KstatDataType)],
    pri: Option<IntrPriority>,
    ctx: &Sleepable,
) -> Result<RawKstat, DdiError> {
    let ndata = u32::try_from(entries.len()).map_err(|_| DdiError)?;
    let raw = RawKstat::create(id, sys::KSTAT_TYPE_NAMED, ndata, pri, ctx)?;
    let data = raw.data().cast::<sys::kstat_named_t>();
    for (i, (name, ty)) in entries.iter().enumerate() {
        // SAFETY: The data section holds `ndata` entries, and `i` is below
        // `ndata`. The kstat is not installed, so no reader touches it. The
        // kernel copies the name.
        unsafe { sys::kstat_named_init(data.add(i), name.as_ptr(), ty.raw()) };
    }
    raw.install();
    Ok(raw)
}

/// A named kstat whose entries are given at run time and addressed by
/// index.
///
/// [`NamedKstat::new`] creates the kstat with `kstat_create(9F)` and
/// `KSTAT_TYPE_NAMED`, one entry per element of the list it is given, names
/// the entries with `kstat_named_init(9F)` and installs the kstat with
/// `kstat_install(9F)`. [`NamedKstat::lock`] takes the kstat's mutex and
/// returns a [`NamedKstatGuard`] through which the entries are read and
/// updated. Dropping the kstat deletes it with `kstat_delete(9F)`.
///
/// Every access through the guard checks that the index exists and that
/// the entry has the type the method expects. When the entries are known at
/// compile time, [`TypedKstat`] does those checks in the compiler instead.
pub struct NamedKstat {
    raw: RawKstat,
    len: usize,
}

impl NamedKstat {
    /// Creates and installs a named kstat with one entry per element of
    /// `entries`, each given as a name and a type. All entries start at
    /// zero. The mutex is an adaptive mutex, so the kstat must not be
    /// updated from an interrupt handler.
    ///
    /// Only the first 30 bytes of each name are significant. Returns
    /// [`DdiError`] if `kstat_create(9F)` fails.
    pub fn new(id: KstatId<'_>, entries: &[(&CStr, KstatDataType)], ctx: &Sleepable) -> Result<NamedKstat, DdiError> {
        Ok(NamedKstat { raw: create_named(&id, entries, None, ctx)?, len: entries.len() })
    }

    /// Like [`NamedKstat::new`], but the kstat's mutex is created with the
    /// priority `pri`, so that the interrupt handler of that priority may
    /// update the kstat.
    pub fn with_priority(
        id: KstatId<'_>,
        entries: &[(&CStr, KstatDataType)],
        pri: IntrPriority,
        ctx: &Sleepable,
    ) -> Result<NamedKstat, DdiError> {
        Ok(NamedKstat { raw: create_named(&id, entries, Some(pri), ctx)?, len: entries.len() })
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Reports whether the kstat has no entries.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Takes the kstat's mutex and returns a guard through which the
    /// entries can be read and updated. Readers of the kstat wait until
    /// the guard is dropped, so they see either all or none of the updates
    /// made through one guard.
    pub fn lock(&self) -> NamedKstatGuard<'_> {
        let lock = self.raw.enter();
        NamedKstatGuard { _lock: lock, data: self.raw.data().cast(), len: self.len }
    }
}

/// Access to the entries of a [`NamedKstat`] with its mutex held. The
/// mutex is released when the guard is dropped.
///
/// The `set_` and `add_` methods panic if `index` is out of range or the
/// entry at `index` was declared with a different type, in the same way
/// that indexing a slice out of range panics. [`NamedKstatGuard::entry`] is
/// the form that returns `None` instead.
pub struct NamedKstatGuard<'a> {
    _lock: MutexGuard<'a, ()>,
    data: *mut sys::kstat_named_t,
    len: usize,
}

impl NamedKstatGuard<'_> {
    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Reports whether the kstat has no entries.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the entry at `index` if it exists and holds a `T`.
    pub fn entry<T: KstatValue>(&mut self, index: usize) -> Option<&mut KstatNamed<T>> {
        if index >= self.len {
            return None;
        }
        // SAFETY: The data section holds `len` entries, and `index` is
        // below `len`. The guard holds the mutex that every reader takes,
        // so no other access to the entry happens while the reference
        // lives, and `&mut self` prevents a second reference through this
        // guard. `KstatNamed<T>` is a transparent wrapper around
        // `kstat_named_t`.
        let e = unsafe { &mut *self.data.add(index).cast::<KstatNamed<T>>() };
        if e.raw.data_type == T::TYPE.raw() { Some(e) } else { None }
    }

    /// Returns the value of the entry at `index` if it exists and holds a
    /// `T`.
    pub fn get<T: KstatValue>(&mut self, index: usize) -> Option<T> {
        self.entry::<T>(index).map(|e| e.get())
    }

    fn expect<T: KstatValue>(&mut self, index: usize) -> &mut KstatNamed<T> {
        let len = self.len;
        match self.entry::<T>(index) {
            Some(e) => e,
            None if index >= len => panic!("kstat entry {index} out of range for {len} entries"),
            None => panic!("kstat entry {index} is not of type {:?}", T::TYPE),
        }
    }

    /// Sets the `KSTAT_DATA_UINT64` entry at `index` to `value`.
    pub fn set_u64(&mut self, index: usize, value: u64) {
        self.expect::<u64>(index).set(value)
    }

    /// Adds `delta` to the `KSTAT_DATA_UINT64` entry at `index`, wrapping
    /// around on overflow.
    pub fn add_u64(&mut self, index: usize, delta: u64) {
        self.expect::<u64>(index).add(delta)
    }

    /// Sets the `KSTAT_DATA_INT64` entry at `index` to `value`.
    pub fn set_i64(&mut self, index: usize, value: i64) {
        self.expect::<i64>(index).set(value)
    }

    /// Adds `delta`, which may be negative, to the `KSTAT_DATA_INT64` entry
    /// at `index`, wrapping around on overflow.
    pub fn add_i64(&mut self, index: usize, delta: i64) {
        self.expect::<i64>(index).add(delta)
    }

    /// Sets the `KSTAT_DATA_UINT32` entry at `index` to `value`.
    pub fn set_u32(&mut self, index: usize, value: u32) {
        self.expect::<u32>(index).set(value)
    }

    /// Adds `delta` to the `KSTAT_DATA_UINT32` entry at `index`, wrapping
    /// around on overflow.
    pub fn add_u32(&mut self, index: usize, delta: u32) {
        self.expect::<u32>(index).add(delta)
    }

    /// Sets the `KSTAT_DATA_INT32` entry at `index` to `value`.
    pub fn set_i32(&mut self, index: usize, value: i32) {
        self.expect::<i32>(index).set(value)
    }

    /// Adds `delta`, which may be negative, to the `KSTAT_DATA_INT32` entry
    /// at `index`, wrapping around on overflow.
    pub fn add_i32(&mut self, index: usize, delta: i32) {
        self.expect::<i32>(index).add(delta)
    }

    /// Sets the `KSTAT_DATA_CHAR` entry at `index` to `value`.
    pub fn set_chars(&mut self, index: usize, value: [u8; 16]) {
        self.expect::<[u8; 16]>(index).set(value)
    }
}

/// A named kstat whose entries are the fields of a structure `S` declared
/// with [`named_kstat!`](crate::named_kstat!).
///
/// [`TypedKstat::new`] creates the kstat with `kstat_create(9F)` and
/// `KSTAT_TYPE_NAMED`, one entry per field of `S`, names the entries with
/// `kstat_named_init(9F)` and installs the kstat with `kstat_install(9F)`.
/// [`TypedKstat::lock`] takes the kstat's mutex and returns a guard that
/// dereferences to `S`, so the driver updates its statistics as
/// `ks.lock().opens.inc()`. Dropping the kstat deletes it with
/// `kstat_delete(9F)`.
pub struct TypedKstat<S: KstatFields> {
    raw: RawKstat,
    _fields: PhantomData<S>,
}

// SAFETY: The contract of `KstatFields` restricts `S` to `KstatNamed`
// fields, which are plain data. They live in the kernel's data section and
// are only reached through a guard that holds the mutex, so no two threads
// ever access them at once.
unsafe impl<S: KstatFields> Send for TypedKstat<S> {}
// SAFETY: As above.
unsafe impl<S: KstatFields> Sync for TypedKstat<S> {}

impl<S: KstatFields> TypedKstat<S> {
    const LAYOUT_OK: () = assert!(
        size_of::<S>() == S::FIELDS.len() * size_of::<sys::kstat_named_t>()
            && align_of::<S>() <= align_of::<sys::kstat_named_t>(),
        "the layout of a KstatFields type does not match its FIELDS"
    );

    /// Creates and installs a named kstat with one entry per field of `S`.
    /// All entries start at zero. The mutex is an adaptive mutex, so the
    /// kstat must not be updated from an interrupt handler.
    ///
    /// Returns [`DdiError`] if `kstat_create(9F)` fails.
    pub fn new(id: KstatId<'_>, ctx: &Sleepable) -> Result<TypedKstat<S>, DdiError> {
        let () = Self::LAYOUT_OK;
        Ok(TypedKstat { raw: create_named(&id, S::FIELDS, None, ctx)?, _fields: PhantomData })
    }

    /// Like [`TypedKstat::new`], but the kstat's mutex is created with the
    /// priority `pri`, so that the interrupt handler of that priority may
    /// update the kstat.
    pub fn with_priority(id: KstatId<'_>, pri: IntrPriority, ctx: &Sleepable) -> Result<TypedKstat<S>, DdiError> {
        let () = Self::LAYOUT_OK;
        Ok(TypedKstat { raw: create_named(&id, S::FIELDS, Some(pri), ctx)?, _fields: PhantomData })
    }

    /// Takes the kstat's mutex and returns a guard that dereferences to the
    /// structure of entries. Readers of the kstat wait until the guard is
    /// dropped, so they see either all or none of the updates made through
    /// one guard.
    pub fn lock(&self) -> TypedKstatGuard<'_, S> {
        let lock = self.raw.enter();
        // SAFETY: `create_named` checked that the data section is not NULL.
        let data = unsafe { NonNull::new_unchecked(self.raw.data().cast::<S>()) };
        TypedKstatGuard { _lock: lock, data }
    }
}

/// Access to the entries of a [`TypedKstat`] with its mutex held.
/// Dereferences to the structure of entries and releases the mutex when
/// dropped.
pub struct TypedKstatGuard<'a, S: KstatFields> {
    _lock: MutexGuard<'a, ()>,
    data: NonNull<S>,
}

impl<S: KstatFields> Deref for TypedKstatGuard<'_, S> {
    type Target = S;
    fn deref(&self) -> &S {
        // SAFETY: The data section holds one `kstat_named_t` per entry of
        // `S::FIELDS`, named with those types, which by the contract of
        // `KstatFields` is the layout of `S`. The kernel allocated it with
        // 8-byte alignment, which `LAYOUT_OK` shows is enough. The guard
        // holds the mutex that every reader takes.
        unsafe { self.data.as_ref() }
    }
}

impl<S: KstatFields> DerefMut for TypedKstatGuard<'_, S> {
    fn deref_mut(&mut self) -> &mut S {
        // SAFETY: As in `deref`, and `&mut self` proves that no other
        // reference derived from this guard exists.
        unsafe { self.data.as_mut() }
    }
}

/// An I/O kstat, `KSTAT_TYPE_IO`, as disk and other block drivers publish
/// for `iostat(8)`.
///
/// The data section is a single `kstat_io(9S)`. [`IoKstat::lock`] takes the
/// kstat's mutex and returns an [`IoKstatGuard`], through which the driver
/// counts completed transfers and records every movement of a request
/// between the wait queue and the run queue with the `kstat_queue(9F)`
/// functions. The manual requires that those transitions happen with
/// `ks_lock` held, which the guard guarantees, and that every transition
/// is recorded, which remains the driver's responsibility. Dropping the
/// kstat deletes it with `kstat_delete(9F)`.
pub struct IoKstat {
    raw: RawKstat,
}

impl IoKstat {
    /// Creates and installs an I/O kstat. All counters start at zero. The
    /// mutex is an adaptive mutex, so the kstat must not be updated from an
    /// interrupt handler. The class is conventionally `disk`.
    ///
    /// Returns [`DdiError`] if `kstat_create(9F)` fails.
    pub fn new(id: KstatId<'_>, ctx: &Sleepable) -> Result<IoKstat, DdiError> {
        Self::create(id, None, ctx)
    }

    /// Like [`IoKstat::new`], but the kstat's mutex is created with the
    /// priority `pri`, so that the interrupt handler of that priority may
    /// update the counters.
    pub fn with_priority(id: KstatId<'_>, pri: IntrPriority, ctx: &Sleepable) -> Result<IoKstat, DdiError> {
        Self::create(id, Some(pri), ctx)
    }

    fn create(id: KstatId<'_>, pri: Option<IntrPriority>, ctx: &Sleepable) -> Result<IoKstat, DdiError> {
        let raw = RawKstat::create(&id, sys::KSTAT_TYPE_IO, 1, pri, ctx)?;
        // The kernel zeroed the single `kstat_io`, which is its correct
        // initial state, so there is nothing to initialise.
        raw.install();
        Ok(IoKstat { raw })
    }

    /// Takes the kstat's mutex and returns a guard through which the
    /// counters and queues are updated.
    pub fn lock(&self) -> IoKstatGuard<'_> {
        let lock = self.raw.enter();
        // SAFETY: `RawKstat::create` checked that the data section is not
        // NULL.
        let data = unsafe { NonNull::new_unchecked(self.raw.data().cast::<sys::kstat_io_t>()) };
        IoKstatGuard { _lock: lock, data }
    }
}

/// Access to the data of an [`IoKstat`] with its mutex held. The mutex is
/// released when the guard is dropped.
///
/// The queue methods correspond to the `kstat_queue(9F)` functions. That
/// manual page allows them in user and kernel context only, so each of them
/// requires a [`Sleepable`] token. The counter methods are plain
/// arithmetic and are available wherever the guard is.
pub struct IoKstatGuard<'a> {
    _lock: MutexGuard<'a, ()>,
    data: NonNull<sys::kstat_io_t>,
}

impl IoKstatGuard<'_> {
    fn io(&mut self) -> *mut sys::kstat_io_t {
        self.data.as_ptr()
    }

    /// Returns the current statistics.
    pub fn stats(&self) -> &sys::kstat_io_t {
        // SAFETY: The data section is a single `kstat_io`, allocated and
        // zeroed by the kernel, and the guard holds the mutex that every
        // reader takes.
        unsafe { self.data.as_ref() }
    }

    /// Counts one completed read of `bytes` bytes, adding to `nread` and
    /// `reads`. The manual advises updating the counters at the end of
    /// service, just before `biodone(9F)`.
    pub fn record_read(&mut self, bytes: u64) {
        // SAFETY: As in `stats`; `&mut self` makes the access exclusive.
        let io = unsafe { self.data.as_mut() };
        io.nread = io.nread.wrapping_add(bytes);
        io.reads = io.reads.wrapping_add(1);
    }

    /// Counts one completed write of `bytes` bytes, adding to `nwritten`
    /// and `writes`.
    pub fn record_write(&mut self, bytes: u64) {
        // SAFETY: As in `record_read`.
        let io = unsafe { self.data.as_mut() };
        io.nwritten = io.nwritten.wrapping_add(bytes);
        io.writes = io.writes.wrapping_add(1);
    }

    /// Records with `kstat_waitq_enter(9F)` that a request has arrived and
    /// joined the wait queue.
    pub fn waitq_enter(&mut self, _ctx: &Sleepable) {
        // SAFETY: The pointer is the kstat's own `kstat_io`, and the guard
        // holds `ks_lock`, as the manual requires.
        unsafe { sys::kstat_waitq_enter(self.io()) }
    }

    /// Records with `kstat_waitq_exit(9F)` that a request has left the wait
    /// queue.
    pub fn waitq_exit(&mut self, _ctx: &Sleepable) {
        // SAFETY: As in `waitq_enter`.
        unsafe { sys::kstat_waitq_exit(self.io()) }
    }

    /// Records with `kstat_runq_enter(9F)` that a request has entered
    /// service and joined the run queue.
    pub fn runq_enter(&mut self, _ctx: &Sleepable) {
        // SAFETY: As in `waitq_enter`.
        unsafe { sys::kstat_runq_enter(self.io()) }
    }

    /// Records with `kstat_runq_exit(9F)` that a request has finished
    /// service and left the run queue.
    pub fn runq_exit(&mut self, _ctx: &Sleepable) {
        // SAFETY: As in `waitq_enter`.
        unsafe { sys::kstat_runq_exit(self.io()) }
    }

    /// Moves a request from the wait queue to the run queue with
    /// `kstat_waitq_to_runq(9F)`, which is the same as
    /// [`IoKstatGuard::waitq_exit`] followed by
    /// [`IoKstatGuard::runq_enter`] with a single timestamp.
    pub fn waitq_to_runq(&mut self, _ctx: &Sleepable) {
        // SAFETY: As in `waitq_enter`.
        unsafe { sys::kstat_waitq_to_runq(self.io()) }
    }

    /// Moves a request from the run queue back to the wait queue with
    /// `kstat_runq_back_to_waitq(9F)`, for example when writes are
    /// throttled.
    pub fn runq_back_to_waitq(&mut self, _ctx: &Sleepable) {
        // SAFETY: As in `waitq_enter`.
        unsafe { sys::kstat_runq_back_to_waitq(self.io()) }
    }
}
