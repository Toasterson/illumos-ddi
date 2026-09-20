//! Loadable module linkage from `sys/modctl.h`.
//!
//! Every kernel module exports `_init`, `_fini` and `_info`, and each of them
//! hands a `modlinkage(9S)` to `mod_install(9F)`, `mod_remove(9F)` or
//! `mod_info(9F)`. The linkage in turn points at one or more module-kind
//! structures such as `modldrv(9S)` for a device driver.

use crate::devops::dev_ops;
use crate::types::{c_char, c_int, c_void, mod_ops, modinfo};

/// The revision of the loadable module system (`MODREV_1`). There has only
/// ever been one.
pub const MODREV_1: c_int = 1;
/// The maximum length of a `drv_linkinfo` string including its terminating
/// NUL (`MODMAXLINKINFOLEN`). Longer strings are truncated by `modinfo(8)`.
pub const MODMAXLINKINFOLEN: usize = 32;

/// The linkage structure for a device driver, `modldrv(9S)`.
#[repr(C)]
pub struct modldrv {
    /// Always points at [`mod_driverops`].
    pub drv_modops: *mut mod_ops,
    /// A short description of the module, shown by `modinfo(8)`.
    pub drv_linkinfo: *const c_char,
    /// The driver's `dev_ops(9S)` structure.
    pub drv_dev_ops: *mut dev_ops,
}

/// The linkage structure for a miscellaneous module, `modlmisc(9S)`. These
/// are library modules that other modules depend on and that have no
/// device nodes of their own.
#[repr(C)]
pub struct modlmisc {
    /// Always points at [`mod_miscops`].
    pub misc_modops: *mut mod_ops,
    /// A short description of the module.
    pub misc_linkinfo: *const c_char,
}

/// The top-level linkage structure, `modlinkage(9S)`. Under LP64 it has
/// room for seven module-kind pointers, and the list is terminated by the
/// first NULL.
#[repr(C)]
pub struct modlinkage {
    /// Always [`MODREV_1`].
    pub ml_rev: c_int,
    /// Pointers to `modldrv`, `modlmisc` or other module-kind structures,
    /// terminated by NULL.
    pub ml_linkage: [*mut c_void; 7],
}

const _: () = assert!(core::mem::size_of::<modlinkage>() == 64);

unsafe extern "C" {
    /// The module operations vector for device drivers.
    pub static mod_driverops: mod_ops;
    /// The module operations vector for miscellaneous modules.
    pub static mod_miscops: mod_ops;

    /// `mod_install(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers the module with the kernel. Called from `_init(9E)`.
    /// Returns 0 on success or an errno value.
    pub fn mod_install(modlp: *mut modlinkage) -> c_int;
    /// `mod_remove(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unregisters the module. Called from `_fini(9E)`. Returns 0 on
    /// success or an errno value, typically `EBUSY` while the module is still
    /// in use.
    pub fn mod_remove(modlp: *mut modlinkage) -> c_int;
    /// `mod_info(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Fills in the `modinfo` structure for `modinfo(8)`. Called from
    /// `_info(9E)`.
    pub fn mod_info(modlp: *mut modlinkage, modinfop: *mut modinfo) -> c_int;
}
