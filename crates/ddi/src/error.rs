//! The two return-code spaces of the DDI.

use core::fmt;
use core::num::NonZeroI32;

use crate::sys;

/// A non-zero `errno` value, as returned by `cb_ops(9S)` entry points and
/// by `9F` functions that document errno returns.
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
    /// Wrap a raw errno; `None` for 0.
    pub const fn new(raw: i32) -> Option<Errno> {
        match NonZeroI32::new(raw) {
            Some(n) => Some(Errno(n)),
            None => None,
        }
    }

    /// The raw errno value.
    pub const fn get(self) -> i32 {
        self.0.get()
    }

    /// Interpret a C return value that is `0` on success or an errno.
    pub const fn check(raw: i32) -> core::result::Result<(), Errno> {
        match Errno::new(raw) {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }

    /// Convert a `Result` back into the C convention.
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

/// `DDI_FAILURE`, as returned by `dev_ops(9S)` entry points and by `9F`
/// functions documented to return `DDI_SUCCESS`/`DDI_FAILURE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct DdiError;

impl DdiError {
    /// Interpret a `DDI_SUCCESS`/`DDI_FAILURE` return.
    pub const fn check(raw: i32) -> core::result::Result<(), DdiError> {
        if raw == sys::DDI_SUCCESS { Ok(()) } else { Err(DdiError) }
    }

    /// Convert back to the C convention.
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

/// Result with [`Errno`] by default.
pub type Result<T, E = Errno> = core::result::Result<T, E>;
