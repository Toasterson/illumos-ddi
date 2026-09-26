//! Loadable module linkage from `sys/modctl.h`.
//!
//! Every kernel module exports `_init`, `_fini` and `_info`, and each of them
//! hands a `modlinkage(9S)` to `mod_install(9F)`, `mod_remove(9F)` or
//! `mod_info(9F)`. The linkage in turn points at one or more module-kind
//! structures such as `modldrv(9S)` for a device driver, `modlmisc(9S)` for
//! a library module or `modlstrmod(9S)` for a STREAMS module. The same
//! header also declares the `ddi_modopen(9F)` family, through which one
//! module loads another at run time and looks up its symbols.

use crate::conf::fmodsw;
use crate::devops::dev_ops;
use crate::types::{c_char, c_int, c_void, mod_ops, modinfo};

/// The revision of the loadable module system (`MODREV_1`). There has only
/// ever been one.
pub const MODREV_1: c_int = 1;
/// The maximum length of a `drv_linkinfo` string including its terminating
/// NUL (`MODMAXLINKINFOLEN`). Longer strings are truncated by `modinfo(8)`.
pub const MODMAXLINKINFOLEN: usize = 32;
/// The only mode `ddi_modopen(9F)` currently accepts (`KRTLD_MODE_FIRST`).
pub const KRTLD_MODE_FIRST: c_int = 0x0001;

opaque! {
    /// The structure behind a module handle returned by `ddi_modopen(9F)`.
    /// Its contents are private to the kernel runtime linker.
    __ddi_modhandle;
}

/// A handle on a module opened with `ddi_modopen(9F)`. It is passed to
/// `ddi_modsym(9F)` to look up symbols and to `ddi_modclose(9F)` to release
/// the module, and its value must not be interpreted by the caller.
pub type ddi_modhandle_t = *mut __ddi_modhandle;

/// `modldrv(9S)`. illumos DDI specific (illumos DDI).
///
/// The linkage structure for a device driver. It identifies the module as a
/// driver, gives `modinfo(8)` a description, and connects the driver's
/// `dev_ops(9S)` to the kernel.
#[repr(C)]
pub struct modldrv {
    /// Always points at [`mod_driverops`].
    pub drv_modops: *mut mod_ops,
    /// A short description of the module, shown by `modinfo(8)`.
    pub drv_linkinfo: *const c_char,
    /// The driver's `dev_ops(9S)` structure.
    pub drv_dev_ops: *mut dev_ops,
}

/// `modlmisc(9S)`. illumos DDI specific (illumos DDI).
///
/// The linkage structure for a miscellaneous module. These are library
/// modules that other modules depend on and that have no device nodes of
/// their own. The description string may be up to `MODMAXNAMELEN`
/// characters including the terminating NUL and usually names the module,
/// possibly with a version.
#[repr(C)]
pub struct modlmisc {
    /// Always points at [`mod_miscops`].
    pub misc_modops: *mut mod_ops,
    /// A short description of the module.
    pub misc_linkinfo: *const c_char,
}

/// `modlstrmod(9S)`. illumos DDI specific (illumos DDI).
///
/// The linkage structure for a loadable STREAMS module. It identifies the
/// module as a STREAMS module and points at the `fmodsw(9S)` template that
/// the kernel copies into its class table when the module is loaded.
#[repr(C)]
pub struct modlstrmod {
    /// Always points at [`mod_strmodops`].
    pub strmod_modops: *mut mod_ops,
    /// A short description of the module, at most `MODMAXNAMELEN` characters.
    pub strmod_linkinfo: *mut c_char,
    /// The `fmodsw(9S)` template describing the module's class entry.
    pub strmod_fmodsw: *mut fmodsw,
}

/// `modlinkage(9S)`. illumos DDI specific (illumos DDI).
///
/// The top-level linkage structure. Under LP64 it has room for seven
/// module-kind pointers, and the list is terminated by the first NULL.
#[repr(C)]
pub struct modlinkage {
    /// Always [`MODREV_1`].
    pub ml_rev: c_int,
    /// Pointers to `modldrv`, `modlmisc` or other module-kind structures,
    /// terminated by NULL.
    pub ml_linkage: [*mut c_void; 7],
}

// modldrv and modlstrmod are three pointers, modlmisc is two. modlinkage is
// an int padded to 8 followed by seven pointers.
const _: () = assert!(core::mem::size_of::<modldrv>() == 24);
const _: () = assert!(core::mem::offset_of!(modldrv, drv_modops) == 0);
const _: () = assert!(core::mem::offset_of!(modldrv, drv_dev_ops) == 16);
const _: () = assert!(core::mem::size_of::<modlmisc>() == 16);
const _: () = assert!(core::mem::offset_of!(modlmisc, misc_modops) == 0);
const _: () = assert!(core::mem::offset_of!(modlmisc, misc_linkinfo) == 8);
const _: () = assert!(core::mem::size_of::<modlstrmod>() == 24);
const _: () = assert!(core::mem::offset_of!(modlstrmod, strmod_modops) == 0);
const _: () = assert!(core::mem::offset_of!(modlstrmod, strmod_fmodsw) == 16);
const _: () = assert!(core::mem::size_of::<modlinkage>() == 64);
const _: () = assert!(core::mem::offset_of!(modlinkage, ml_rev) == 0);
const _: () = assert!(core::mem::offset_of!(modlinkage, ml_linkage) == 8);

unsafe extern "C" {
    /// The module operations vector for device drivers.
    pub static mod_driverops: mod_ops;
    /// The module operations vector for miscellaneous modules.
    pub static mod_miscops: mod_ops;
    /// The module operations vector for STREAMS modules.
    pub static mod_strmodops: mod_ops;

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
    /// `mod_install(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the name of the module that `modlp` belongs to, as the
    /// kernel recorded it when the module was loaded. The string belongs to
    /// the kernel and stays valid while the module is loaded.
    pub fn mod_modname(modlp: *mut modlinkage) -> *const c_char;

    /// `ddi_modopen(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Loads the kernel module named `modname` and makes it available to the
    /// caller, returning a handle for later `ddi_modsym` and `ddi_modclose`
    /// calls. The name has the form `[namespace/[dirspace/]]modulename`; each
    /// `namespace/` directory along the kernel module path is searched, and
    /// `misc/` is assumed when no namespace is given. The `mode` argument
    /// must currently be [`KRTLD_MODE_FIRST`]. Opening a module increments its
    /// reference count, and on the first load its `_init(9E)` is run before
    /// this function returns. The interface works best with `misc` modules
    /// that carry a single `modlmisc(9S)` linkage, though other module kinds
    /// can be opened too. Returns NULL if the module cannot be found,
    /// relocated or resolved, in which case an errno value is stored through
    /// `errnop` when that pointer is not NULL. May be called from user
    /// context only.
    pub fn ddi_modopen(modname: *const c_char, mode: c_int, errnop: *mut c_int) -> ddi_modhandle_t;
    /// `ddi_modsym(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Looks up the symbol `symname` within the module behind `handle`. The
    /// search is limited to that one module; the special handle values of
    /// the user-space `dlsym(3C)` are not supported. Returns the symbol's
    /// address, or NULL if it is not defined there, storing an errno value
    /// through `errnop` when that pointer is not NULL. Every address obtained
    /// this way becomes invalid once the handle is closed.
    pub fn ddi_modsym(handle: ddi_modhandle_t, symname: *const c_char, errnop: *mut c_int) -> *mut c_void;
    /// `ddi_modclose(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Drops the reference on the module behind `handle`. When the last
    /// reference goes away the module's `_fini(9E)` may be called and, if it
    /// succeeds, the module may be unloaded. Returns 0 on success and a
    /// non-zero value if the handle is invalid.
    pub fn ddi_modclose(handle: ddi_modhandle_t) -> c_int;
}
