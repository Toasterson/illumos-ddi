//! `sys/cred.h`: credentials. `cred_t` is opaque; see `types`.

use crate::types::{c_int, cred_t};

unsafe extern "C" {
    /// `drv_priv(9F)` — illumos DDI specific (illumos DDI). 0 if the
    /// credential carries `PRIV_SYS_DEVICES`, else `EPERM`.
    pub fn drv_priv(cr: *mut cred_t) -> c_int;
    /// `ddi_get_cred(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_get_cred() -> *mut cred_t;
    /// `crgetuid(9F)` — illumos DDI specific (illumos DDI).
    pub fn crgetuid(cr: *const cred_t) -> crate::types::uint_t;
    /// `crgetzoneid(9F)` — illumos DDI specific (illumos DDI).
    pub fn crgetzoneid(cr: *const cred_t) -> c_int;
}
