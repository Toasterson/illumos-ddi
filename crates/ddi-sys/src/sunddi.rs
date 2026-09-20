//! `sys/sunddi.h` core subset: return codes, instance and device numbers,
//! minor nodes, soft state, driver-private data, properties, copyin/out.

use crate::devops::ddi_prop_op_t;
use crate::types::*;

/// `DDI_SUCCESS`
pub const DDI_SUCCESS: c_int = 0;
/// `DDI_FAILURE`
pub const DDI_FAILURE: c_int = -1;

/// `DDI_PSEUDO`: node type for pseudo devices in `ddi_create_minor_node(9F)`.
pub const DDI_PSEUDO: &core::ffi::CStr = c"ddi_pseudo";
/// `DDI_NT_BLOCK`: block device node type.
pub const DDI_NT_BLOCK: &core::ffi::CStr = c"ddi_block";
/// `DDI_NT_BLOCK_CHAN`: block device with channel node type.
pub const DDI_NT_BLOCK_CHAN: &core::ffi::CStr = c"ddi_block:channel";
/// `DDI_NT_CD`: CD-ROM node type.
pub const DDI_NT_CD: &core::ffi::CStr = c"ddi_block:cdrom";
/// `DDI_NT_SERIAL`: serial port node type.
pub const DDI_NT_SERIAL: &core::ffi::CStr = c"ddi_serial";
/// `DDI_NT_NET`: network device node type.
pub const DDI_NT_NET: &core::ffi::CStr = c"ddi_network";

/// `DDI_DEV_T_NONE`
pub const DDI_DEV_T_NONE: dev_t = !0;
/// `DDI_DEV_T_ANY`
pub const DDI_DEV_T_ANY: dev_t = !0 - 1;

/// `DDI_PROP_SUCCESS`
pub const DDI_PROP_SUCCESS: c_int = 0;
/// `DDI_PROP_NOT_FOUND`
pub const DDI_PROP_NOT_FOUND: c_int = 1;
/// `DDI_PROP_UNDEFINED`
pub const DDI_PROP_UNDEFINED: c_int = 2;
/// `DDI_PROP_NO_MEMORY`
pub const DDI_PROP_NO_MEMORY: c_int = 3;
/// `DDI_PROP_INVAL_ARG`
pub const DDI_PROP_INVAL_ARG: c_int = 4;
/// `DDI_PROP_BUF_TOO_SMALL`
pub const DDI_PROP_BUF_TOO_SMALL: c_int = 5;
/// `DDI_PROP_DONTPASS`: do not pass the lookup to the parent.
pub const DDI_PROP_DONTPASS: uint_t = 0x0001;
/// `DDI_PROP_CANSLEEP`
pub const DDI_PROP_CANSLEEP: uint_t = 0x0002;

unsafe extern "C" {
    // --- device numbers (sys/mkdev.h) --------------------------------------
    /// `getmajor(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn getmajor(dev: dev_t) -> major_t;
    /// `getminor(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn getminor(dev: dev_t) -> minor_t;
    /// `makedevice(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn makedevice(maj: major_t, min: minor_t) -> dev_t;

    // --- dev_info_t --------------------------------------------------------
    /// `ddi_get_instance(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_get_instance(dip: dev_info_t) -> c_int;
    /// `ddi_driver_name(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_driver_name(dip: dev_info_t) -> *const c_char;
    /// `ddi_driver_major(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_driver_major(dip: dev_info_t) -> major_t;
    /// `ddi_get_parent(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_get_parent(dip: dev_info_t) -> dev_info_t;
    /// `ddi_report_dev(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_report_dev(dip: dev_info_t);
    /// `ddi_set_driver_private(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_set_driver_private(dip: dev_info_t, data: *mut c_void);
    /// `ddi_get_driver_private(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_get_driver_private(dip: dev_info_t) -> *mut c_void;
    /// `ddi_quiesce_not_needed(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_quiesce_not_needed(dip: dev_info_t) -> c_int;
    /// `ddi_quiesce_not_supported(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_quiesce_not_supported(dip: dev_info_t) -> c_int;

    // --- minor nodes -------------------------------------------------------
    /// `ddi_create_minor_node(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_create_minor_node(
        dip: dev_info_t, name: *const c_char, spec_type: c_int, minor_num: minor_t, node_type: *const c_char, flag: c_int,
    ) -> c_int;
    /// `ddi_remove_minor_node(9F)` — illumos DDI specific (illumos DDI).
    /// `name == NULL` removes all minor nodes of `dip`.
    pub fn ddi_remove_minor_node(dip: dev_info_t, name: *const c_char);

    // --- soft state --------------------------------------------------------
    /// `ddi_soft_state_init(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_soft_state_init(state_p: *mut *mut c_void, size: size_t, n_items: size_t) -> c_int;
    /// `ddi_soft_state_fini(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_soft_state_fini(state_p: *mut *mut c_void);
    /// `ddi_soft_state_zalloc(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_soft_state_zalloc(state: *mut c_void, item: c_int) -> c_int;
    /// `ddi_soft_state_free(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_soft_state_free(state: *mut c_void, item: c_int);
    /// `ddi_get_soft_state(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_get_soft_state(state: *mut c_void, item: c_int) -> *mut c_void;

    // --- properties --------------------------------------------------------
    /// `ddi_prop_exists(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_exists(match_dev: dev_t, dip: dev_info_t, flags: uint_t, name: *const c_char) -> c_int;
    /// `ddi_prop_get_int(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_get_int(match_dev: dev_t, dip: dev_info_t, flags: uint_t, name: *const c_char, defvalue: c_int) -> c_int;
    /// `ddi_prop_get_int64(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_get_int64(match_dev: dev_t, dip: dev_info_t, flags: uint_t, name: *const c_char, defvalue: i64) -> i64;
    /// `ddi_prop_lookup_string(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_lookup_string(match_dev: dev_t, dip: dev_info_t, flags: uint_t, name: *const c_char, data: *mut *mut c_char) -> c_int;
    /// `ddi_prop_lookup_int_array(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_lookup_int_array(match_dev: dev_t, dip: dev_info_t, flags: uint_t, name: *const c_char, data: *mut *mut c_int, nelements: *mut uint_t) -> c_int;
    /// `ddi_prop_free(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_free(data: *mut c_void);
    /// `ddi_prop_update_int(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_update_int(match_dev: dev_t, dip: dev_info_t, name: *const c_char, data: c_int) -> c_int;
    /// `ddi_prop_update_string(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_update_string(match_dev: dev_t, dip: dev_info_t, name: *const c_char, data: *const c_char) -> c_int;
    /// `ddi_prop_remove(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_remove(dev: dev_t, dip: dev_info_t, name: *const c_char) -> c_int;
    /// `ddi_prop_remove_all(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_prop_remove_all(dip: dev_info_t);
    /// `ddi_prop_op(9F)` — illumos DDI specific (illumos DDI). The default
    /// `prop_op(9E)` implementation; signature matches `cb_prop_op_t`.
    pub fn ddi_prop_op(
        dev: dev_t, dip: dev_info_t, prop_op: ddi_prop_op_t, mod_flags: c_int, name: *mut c_char, valuep: caddr_t, lengthp: *mut c_int,
    ) -> c_int;

    // --- user memory -------------------------------------------------------
    /// `ddi_copyin(9F)` — illumos DDI specific (illumos DDI). `flags` is the
    /// ioctl `mode`; honours `FKIOCTL`. Returns 0 or -1.
    pub fn ddi_copyin(buf: *const c_void, driverbuf: *mut c_void, cn: size_t, flags: c_int) -> c_int;
    /// `ddi_copyout(9F)` — illumos DDI specific (illumos DDI). Returns 0 or -1.
    pub fn ddi_copyout(driverbuf: *const c_void, buf: *mut c_void, cn: size_t, flags: c_int) -> c_int;

    // --- misc ------------------------------------------------------------
    /// `ddi_in_panic(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_in_panic() -> c_int;
}
