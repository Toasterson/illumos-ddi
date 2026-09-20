//! The two return-code conventions of the DDI as two types.
//!
//! Entry points in `cb_ops(9S)` and most functions that deal with user
//! requests report failure with an errno value. Entry points in `dev_ops(9S)`
//! and the `ddi_*` functions that manage device state report `DDI_SUCCESS`
//! or `DDI_FAILURE`. Mixing the two up is a classic mistake in C drivers.
//! [`Errno`] and [`DdiError`] make it a type error instead.

use core::fmt;
use core::num::NonZeroI32;

use crate::sys;

/// A non-zero errno value.
///
/// This is what `cb_ops(9S)` entry points return, and what `9F` functions
/// such as `uiomove` and `ddi_copyin` report. The associated constants cover
/// the values a driver commonly needs; [`Errno::new`] accepts any other.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Errno(NonZeroI32);

macro_rules! errnos {
    ($($name:ident),* $(,)?) => {
        impl Errno {
            $(
                #[doc = concat!("`", stringify!($name), "`")]
                pub const $name: Errno = Errno(match NonZeroI32::new(sys::errno::$name) {
                    Some(n) => n,
                    None => panic!("errno constant is zero"),
                });
            )*
            fn name(self) -> Option<&'static str> {
                match self.0.get() {
                    $(sys::errno::$name => Some(stringify!($name)),)*
                    _ => None,
                }
            }
        }
    };
}
errnos!(
    EPERM,
    ENOENT,
    ESRCH,
    EINTR,
    EIO,
    ENXIO,
    E2BIG,
    EBADF,
    EAGAIN,
    ENOMEM,
    EACCES,
    EFAULT,
    EBUSY,
    EEXIST,
    ENODEV,
    ENOTDIR,
    EINVAL,
    ENFILE,
    EMFILE,
    ENOTTY,
    EFBIG,
    ENOSPC,
    ESPIPE,
    EROFS,
    EPIPE,
    ERANGE,
    ENOMSG,
    ENOTSUP,
    EOVERFLOW,
    ECANCELED,
    ETIME,
    ETIMEDOUT,
    EPROTO,
    ENOBUFS,
    EALREADY,
    EINPROGRESS,
);

impl Errno {
    /// Wraps a raw errno value. Returns `None` for 0, which is not an error.
    pub const fn new(raw: i32) -> Option<Errno> {
        match NonZeroI32::new(raw) {
            Some(n) => Some(Errno(n)),
            None => None,
        }
    }

    /// Returns the raw errno value.
    pub const fn get(self) -> i32 {
        self.0.get()
    }

    /// Interprets a C return value that is 0 on success and an errno
    /// otherwise.
    pub const fn check(raw: i32) -> core::result::Result<(), Errno> {
        match Errno::new(raw) {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }

    /// Converts a `Result` back into the C convention, 0 for `Ok` and the
    /// errno for `Err`.
    pub fn into_c(r: core::result::Result<(), Errno>) -> i32 {
        match r {
            Ok(()) => 0,
            Err(e) => e.get(),
        }
    }
}

impl fmt::Debug for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(n) => write!(f, "{n}({})", self.get()),
            None => write!(f, "errno({})", self.get()),
        }
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// The failure of an operation in the `DDI_SUCCESS` and `DDI_FAILURE`
/// convention.
///
/// This is what `dev_ops(9S)` entry points return and what `9F` functions
/// such as `ddi_create_minor_node` report. The convention carries no reason,
/// which is why this type has no payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct DdiError;

impl DdiError {
    /// Interprets a C return value in the `DDI_SUCCESS` and `DDI_FAILURE`
    /// convention.
    pub const fn check(raw: i32) -> core::result::Result<(), DdiError> {
        if raw == sys::DDI_SUCCESS { Ok(()) } else { Err(DdiError) }
    }

    /// Converts a `Result` back into the C convention.
    pub const fn into_c(r: core::result::Result<(), DdiError>) -> i32 {
        match r {
            Ok(()) => sys::DDI_SUCCESS,
            Err(DdiError) => sys::DDI_FAILURE,
        }
    }
}

impl fmt::Display for DdiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DDI_FAILURE")
    }
}

/// A `Result` whose error type defaults to [`Errno`].
pub type Result<T, E = Errno> = core::result::Result<T, E>;
