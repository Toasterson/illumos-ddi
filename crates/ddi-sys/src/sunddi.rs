//! The core of `sys/sunddi.h`.
//!
//! This is the subset of the main DDI header that every leaf driver touches:
//! the `DDI_SUCCESS` and `DDI_FAILURE` return codes, device and instance
//! numbers, minor nodes, the soft-state table, driver-private data,
//! properties, and the two functions that move data across the user-kernel
//! boundary. Register access, interrupts and DMA live in their own modules
//! behind features.

use crate::devops::ddi_prop_op_t;
use crate::types::*;

/// The success return value of `dev_ops(9S)` entry points and of the
/// `ddi_*` functions that use this convention (`DDI_SUCCESS`).
pub const DDI_SUCCESS: c_int = 0;
/// The failure return value in the same convention (`DDI_FAILURE`).
pub const DDI_FAILURE: c_int = -1;

/// The node type string for pseudo devices (`DDI_PSEUDO`). Node types tell
/// `devfsadm` how to create `/dev` links for a minor node.
pub const DDI_PSEUDO: &core::ffi::CStr = c"ddi_pseudo";
/// The node type string for block devices (`DDI_NT_BLOCK`).
pub const DDI_NT_BLOCK: &core::ffi::CStr = c"ddi_block";
/// The node type string for block devices addressed by channel (`DDI_NT_BLOCK_CHAN`).
pub const DDI_NT_BLOCK_CHAN: &core::ffi::CStr = c"ddi_block:channel";
/// The node type string for CD-ROM devices (`DDI_NT_CD`).
pub const DDI_NT_CD: &core::ffi::CStr = c"ddi_block:cdrom";
/// The node type string for serial ports (`DDI_NT_SERIAL`).
pub const DDI_NT_SERIAL: &core::ffi::CStr = c"ddi_serial";
/// The node type string for network devices (`DDI_NT_NET`).
pub const DDI_NT_NET: &core::ffi::CStr = c"ddi_network";

/// A `dev_t` meaning "no device", used when creating properties that are not
/// tied to a minor node (`DDI_DEV_T_NONE`).
pub const DDI_DEV_T_NONE: dev_t = !0;
/// A `dev_t` meaning "any device", used when looking up properties without
/// regard to the minor node (`DDI_DEV_T_ANY`).
pub const DDI_DEV_T_ANY: dev_t = !0 - 1;

/// The property operation succeeded (`DDI_PROP_SUCCESS`).
pub const DDI_PROP_SUCCESS: c_int = 0;
/// The property does not exist (`DDI_PROP_NOT_FOUND`).
pub const DDI_PROP_NOT_FOUND: c_int = 1;
/// The property was explicitly undefined by an override (`DDI_PROP_UNDEFINED`).
pub const DDI_PROP_UNDEFINED: c_int = 2;
/// Memory for the property value could not be allocated (`DDI_PROP_NO_MEMORY`).
pub const DDI_PROP_NO_MEMORY: c_int = 3;
/// An argument to the property function was invalid (`DDI_PROP_INVAL_ARG`).
pub const DDI_PROP_INVAL_ARG: c_int = 4;
/// The caller's buffer is too small for the value (`DDI_PROP_BUF_TOO_SMALL`).
pub const DDI_PROP_BUF_TOO_SMALL: c_int = 5;
/// Look up the property on this node only, without consulting the parent
/// nodes (`DDI_PROP_DONTPASS`). Almost every driver lookup uses this flag.
pub const DDI_PROP_DONTPASS: uint_t = 0x0001;
/// The property function may block for memory (`DDI_PROP_CANSLEEP`).
pub const DDI_PROP_CANSLEEP: uint_t = 0x0002;

unsafe extern "C" {
    // --- device numbers (sys/mkdev.h) --------------------------------------
    /// `getmajor(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Extracts the major number from a `dev_t`.
    pub fn getmajor(dev: dev_t) -> major_t;
    /// `getminor(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Extracts the minor number from a `dev_t`.
    pub fn getminor(dev: dev_t) -> minor_t;
    /// `makedevice(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Combines a major and a minor number into a `dev_t`.
    pub fn makedevice(maj: major_t, min: minor_t) -> dev_t;

    // --- dev_info_t --------------------------------------------------------
    /// `ddi_get_instance(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the instance number of a device node. Instance numbers are
    /// stable across reboots for a given device.
    pub fn ddi_get_instance(dip: dev_info_t) -> c_int;
    /// `ddi_driver_name(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the name of the driver bound to a device node.
    pub fn ddi_driver_name(dip: dev_info_t) -> *const c_char;
    /// `ddi_driver_major(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the major number of the driver bound to a device node.
    pub fn ddi_driver_major(dip: dev_info_t) -> major_t;
    /// `ddi_get_parent(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the parent of a device node in the device tree.
    pub fn ddi_get_parent(dip: dev_info_t) -> dev_info_t;
    /// `ddi_report_dev(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Logs the line that announces an attached instance, in the form
    /// `rnull0 is /pseudo/rnull@0`. Conventionally called at the end of a
    /// successful `attach(9E)`.
    pub fn ddi_report_dev(dip: dev_info_t);
    /// `ddi_set_driver_private(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores one pointer of driver-private data in the device node.
    pub fn ddi_set_driver_private(dip: dev_info_t, data: *mut c_void);
    /// `ddi_get_driver_private(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Retrieves the pointer stored with [`ddi_set_driver_private`].
    pub fn ddi_get_driver_private(dip: dev_info_t) -> *mut c_void;
    /// `ddi_quiesce_not_needed(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A `quiesce(9E)` implementation for drivers that have no hardware to
    /// quiesce before a fast reboot. Always succeeds.
    pub fn ddi_quiesce_not_needed(dip: dev_info_t) -> c_int;
    /// `ddi_quiesce_not_supported(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A `quiesce(9E)` implementation for drivers that cannot quiesce their
    /// hardware. Always fails, which disables fast reboot while the driver is
    /// loaded.
    pub fn ddi_quiesce_not_supported(dip: dev_info_t) -> c_int;

    // --- minor nodes -------------------------------------------------------
    /// `ddi_create_minor_node(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates a minor node named `name` under the device node. `spec_type`
    /// is `S_IFCHR` or `S_IFBLK`, `node_type` is one of the `DDI_NT_*`
    /// strings, and `flag` is 0 for ordinary nodes. Returns `DDI_SUCCESS` or
    /// `DDI_FAILURE`.
    pub fn ddi_create_minor_node(
        dip: dev_info_t,
        name: *const c_char,
        spec_type: c_int,
        minor_num: minor_t,
        node_type: *const c_char,
        flag: c_int,
    ) -> c_int;
    /// `ddi_remove_minor_node(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the minor node named `name`, or every minor node of the device
    /// when `name` is NULL.
    pub fn ddi_remove_minor_node(dip: dev_info_t, name: *const c_char);

    // --- soft state --------------------------------------------------------
    /// `ddi_soft_state_init(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates a soft-state table whose items are `size` bytes each. The
    /// `n_items` argument is only a hint for the initial capacity; the table
    /// grows on demand. Called from `_init(9E)`.
    pub fn ddi_soft_state_init(state_p: *mut *mut c_void, size: size_t, n_items: size_t) -> c_int;
    /// `ddi_soft_state_fini(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Destroys a soft-state table. Called from `_fini(9E)`.
    pub fn ddi_soft_state_fini(state_p: *mut *mut c_void);
    /// `ddi_soft_state_zalloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates the zeroed item for index `item`, normally the instance
    /// number. Returns `DDI_SUCCESS` or `DDI_FAILURE`.
    pub fn ddi_soft_state_zalloc(state: *mut c_void, item: c_int) -> c_int;
    /// `ddi_soft_state_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the item for index `item`.
    pub fn ddi_soft_state_free(state: *mut c_void, item: c_int);
    /// `ddi_get_soft_state(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the item for index `item`, or NULL if none has been allocated.
    pub fn ddi_get_soft_state(state: *mut c_void, item: c_int) -> *mut c_void;

    // --- properties --------------------------------------------------------
    /// `ddi_prop_exists(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns non-zero if the named property exists.
    pub fn ddi_prop_exists(match_dev: dev_t, dip: dev_info_t, flags: uint_t, name: *const c_char) -> c_int;
    /// `ddi_prop_get_int(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the value of an integer property, or `defvalue` if the
    /// property does not exist or has the wrong type.
    pub fn ddi_prop_get_int(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        defvalue: c_int,
    ) -> c_int;
    /// `ddi_prop_get_int64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The 64-bit counterpart of [`ddi_prop_get_int`].
    pub fn ddi_prop_get_int64(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        defvalue: i64,
    ) -> i64;
    /// `ddi_prop_lookup_string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a string property in a buffer allocated by the kernel. The
    /// caller releases it with [`ddi_prop_free`]. Returns one of the
    /// `DDI_PROP_*` codes.
    pub fn ddi_prop_lookup_string(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        data: *mut *mut c_char,
    ) -> c_int;
    /// `ddi_prop_lookup_int_array(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns an integer array property in a buffer allocated by the
    /// kernel, with its element count in `nelements`. The caller releases
    /// the buffer with [`ddi_prop_free`].
    pub fn ddi_prop_lookup_int_array(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        data: *mut *mut c_int,
        nelements: *mut uint_t,
    ) -> c_int;
    /// `ddi_prop_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases a buffer returned by one of the `ddi_prop_lookup_*` functions.
    pub fn ddi_prop_free(data: *mut c_void);
    /// `ddi_prop_update_int(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates or replaces an integer property on the device node.
    pub fn ddi_prop_update_int(match_dev: dev_t, dip: dev_info_t, name: *const c_char, data: c_int) -> c_int;
    /// `ddi_prop_update_string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates or replaces a string property on the device node.
    pub fn ddi_prop_update_string(match_dev: dev_t, dip: dev_info_t, name: *const c_char, data: *const c_char)
    -> c_int;
    /// `ddi_prop_remove(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes one property from the device node.
    pub fn ddi_prop_remove(dev: dev_t, dip: dev_info_t, name: *const c_char) -> c_int;
    /// `ddi_prop_remove_all(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes every property the driver created on the device node.
    /// Conventionally called from `detach(9E)`.
    pub fn ddi_prop_remove_all(dip: dev_info_t);
    /// `ddi_prop_op(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The default implementation of `prop_op(9E)`. Its signature matches
    /// [`crate::devops::cb_prop_op_t`], so it can be placed in `cb_prop_op`
    /// directly, and most drivers do exactly that.
    pub fn ddi_prop_op(
        dev: dev_t,
        dip: dev_info_t,
        prop_op: ddi_prop_op_t,
        mod_flags: c_int,
        name: *mut c_char,
        valuep: caddr_t,
        lengthp: *mut c_int,
    ) -> c_int;

    // --- user memory -------------------------------------------------------
    /// `ddi_copyin(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies `cn` bytes from the caller's buffer `buf` into the kernel
    /// buffer `driverbuf`. `flags` is the `mode` argument of `ioctl(9E)`;
    /// when it contains `FKIOCTL` the source is already in kernel space and
    /// a plain copy is performed. Returns 0 on success and -1 on a fault.
    pub fn ddi_copyin(buf: *const c_void, driverbuf: *mut c_void, cn: size_t, flags: c_int) -> c_int;
    /// `ddi_copyout(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The counterpart of [`ddi_copyin`], copying from the kernel buffer to
    /// the caller's buffer. Returns 0 on success and -1 on a fault.
    pub fn ddi_copyout(driverbuf: *const c_void, buf: *mut c_void, cn: size_t, flags: c_int) -> c_int;

    // --- misc ------------------------------------------------------------
    /// `ddi_in_panic(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns non-zero while the system is panicking. Drivers use it to
    /// avoid blocking operations on the way to a crash dump.
    pub fn ddi_in_panic() -> c_int;
}
