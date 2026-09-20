//! The caller's credentials as a borrowed handle.

use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::sys;

/// The credentials of the process or thread that called the current entry
/// point. The handle is valid for the duration of that call only, which the
/// lifetime parameter enforces.
pub struct Cred<'a> {
    raw: NonNull<sys::cred_t>,
    _life: PhantomData<&'a sys::cred_t>,
}

impl<'a> Cred<'a> {
    /// Wraps a `cred_t` pointer handed in by the kernel.
    ///
    /// # Safety
    /// `p` must point to a `cred_t` that stays valid for the lifetime `'a`.
    pub unsafe fn from_raw(p: *mut sys::cred_t) -> Cred<'a> {
        Cred { raw: unsafe { NonNull::new_unchecked(p) }, _life: PhantomData }
    }

    /// Returns the raw pointer, for `9F` functions that are not wrapped yet.
    pub fn as_raw(&self) -> *mut sys::cred_t {
        self.raw.as_ptr()
    }

    /// Reports whether the caller holds the `PRIV_SYS_DEVICES` privilege,
    /// using `drv_priv(9F)`. This is the check to perform before an ioctl
    /// that changes device state.
    pub fn has_sys_devices_priv(&self) -> bool {
        unsafe { sys::drv_priv(self.raw.as_ptr()) == 0 }
    }

    /// Returns the caller's effective user ID, using `crgetuid(9F)`.
    pub fn uid(&self) -> u32 {
        unsafe { sys::crgetuid(self.raw.as_ptr()) }
    }

    /// Returns the ID of the caller's zone, using `crgetzoneid(9F)`.
    pub fn zone_id(&self) -> i32 {
        unsafe { sys::crgetzoneid(self.raw.as_ptr()) }
    }
}
