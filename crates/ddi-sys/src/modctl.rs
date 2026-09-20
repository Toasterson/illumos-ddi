//! `sys/modctl.h`: loadable module linkage. `modlinkage(9S)`, `modldrv(9S)`,
//! `modlmisc(9S)`, `mod_install(9F)`, `mod_remove(9F)`, `mod_info(9F)`.

use crate::devops::dev_ops;
use crate::types::{c_char, c_int, c_void, mod_ops, modinfo};

/// `MODREV_1`: the only revision of the loadable module system.
pub const MODREV_1: c_int = 1;
/// `MODMAXLINKINFOLEN`: maximum length of `drv_linkinfo`, including NUL.
pub const MODMAXLINKINFOLEN: usize = 32;

/// `modldrv(9S)`: linkage for a device driver.
#[repr(C)]
pub struct modldrv {
    /// Always `&mod_driverops`.
    pub drv_modops: *mut mod_ops,
    /// Human-readable description shown by `modinfo(8)`.
    pub drv_linkinfo: *const c_char,
    /// The driver's `dev_ops(9S)`.
    pub drv_dev_ops: *mut dev_ops,
}

/// `modlmisc(9S)`: linkage for a miscellaneous (library) module.
#[repr(C)]
pub struct modlmisc {
    /// Always `&mod_miscops`.
    pub misc_modops: *mut mod_ops,
    /// Description string.
    pub misc_linkinfo: *const c_char,
}

/// `modlinkage(9S)`. `ml_linkage` is a NULL-terminated list; LP64 has room
/// for 7 entries.
#[repr(C)]
pub struct modlinkage {
    /// `MODREV_1`.
    pub ml_rev: c_int,
    /// Pointers to `modldrv`/`modlmisc`/… structures, NULL-terminated.
    pub ml_linkage: [*mut c_void; 7],
}

const _: () = assert!(core::mem::size_of::<modlinkage>() == 64);

unsafe extern "C" {
    /// `mod_driverops`: `mod_ops` for device drivers.
    pub static mod_driverops: mod_ops;
    /// `mod_miscops`: `mod_ops` for miscellaneous modules.
    pub static mod_miscops: mod_ops;

    /// `mod_install(9F)`. illumos DDI specific (illumos DDI). Returns 0 or errno.
    pub fn mod_install(modlp: *mut modlinkage) -> c_int;
    /// `mod_remove(9F)`. illumos DDI specific (illumos DDI). Returns 0 or errno.
    pub fn mod_remove(modlp: *mut modlinkage) -> c_int;
    /// `mod_info(9F)`. illumos DDI specific (illumos DDI).
    pub fn mod_info(modlp: *mut modlinkage, modinfop: *mut modinfo) -> c_int;
}
