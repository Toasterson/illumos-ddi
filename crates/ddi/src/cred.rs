//! `cred_t` as a borrowed handle.

use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::sys;

/// Credentials of the caller of an entry point. Valid for the duration of
/// that call only.
pub struct Cred<'a> {
    raw: NonNull<sys::cred_t>,
    _life: PhantomData<&'a sys::cred_t>,
}

impl<'a> Cred<'a> {
    /// Wrap a `cred_t *` handed in by the kernel.
    ///
    /// # Safety
    /// `p` must be a valid `cred_t` for lifetime `'a`.
    pub unsafe fn from_raw(p: *mut sys::cred_t) -> Cred<'a> {
        Cred { raw: unsafe { NonNull::new_unchecked(p) }, _life: PhantomData }
    }

    /// The raw pointer, for passing to `9F` functions.
    pub fn as_raw(&self) -> *mut sys::cred_t {
        self.raw.as_ptr()
    }

    /// `drv_priv(9F)`: does the caller hold `PRIV_SYS_DEVICES`?
    pub fn has_sys_devices_priv(&self) -> bool {
        unsafe { sys::drv_priv(self.raw.as_ptr()) == 0 }
    }

    /// `crgetuid(9F)`.
    pub fn uid(&self) -> u32 {
        unsafe { sys::crgetuid(self.raw.as_ptr()) }
    }

    /// `crgetzoneid(9F)`.
    pub fn zone_id(&self) -> i32 {
        unsafe { sys::crgetzoneid(self.raw.as_ptr()) }
    }
}
