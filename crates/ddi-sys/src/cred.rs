//! Credentials from `sys/cred.h`.
//!
//! The kernel hands a `cred_t` to most entry points. The structure itself is
//! opaque to drivers (see [`crate::types::cred_t`]); these functions are the
//! documented way to ask questions about it.

use crate::types::{c_int, cred_t};

unsafe extern "C" {
    /// `drv_priv(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns 0 if the credential carries the `PRIV_SYS_DEVICES` privilege
    /// and `EPERM` otherwise. This is the check a driver performs before an
    /// ioctl that changes device state.
    pub fn drv_priv(cr: *mut cred_t) -> c_int;
    /// `ddi_get_cred(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the credential of the current thread.
    pub fn ddi_get_cred() -> *mut cred_t;
    /// `crgetuid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the effective user ID of the credential.
    pub fn crgetuid(cr: *const cred_t) -> crate::types::uint_t;
    /// `crgetzoneid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the ID of the zone the credential belongs to.
    pub fn crgetzoneid(cr: *const cred_t) -> c_int;
}
