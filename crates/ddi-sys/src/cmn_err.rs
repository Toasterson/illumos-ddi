//! `sys/cmn_err.h`: `cmn_err(9F)`, `vcmn_err(9F)`, `dev_err(9F)`, `panic(9F)`.

use crate::types::*;

/// `CE_CONT`: continuation.
pub const CE_CONT: c_int = 0;
/// `CE_NOTE`: notice.
pub const CE_NOTE: c_int = 1;
/// `CE_WARN`: warning.
pub const CE_WARN: c_int = 2;
/// `CE_PANIC`: panic the system.
pub const CE_PANIC: c_int = 3;
/// `CE_IGNORE`: print nothing.
pub const CE_IGNORE: c_int = 4;

unsafe extern "C" {
    /// `cmn_err(9F)`. Architecture independent level 1 (DDI/DKI).
    pub fn cmn_err(level: c_int, fmt: *const c_char, ...);
    /// `dev_err(9F)`. illumos DDI specific (illumos DDI). Prefixes the message
    /// with the driver name and instance.
    pub fn dev_err(dip: dev_info_t, level: c_int, fmt: *const c_char, ...);
    /// `panic(9F)`. illumos DDI specific (illumos DDI).
    pub fn panic(fmt: *const c_char, ...) -> !;
}
