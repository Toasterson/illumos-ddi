//! Device properties, `ddi_prop_*(9F)`, from `sys/sunddi.h` and `sys/ddipropdefs.h`.
//!
//! Properties are name and value pairs attached to a device node. They come
//! from three places. The driver's `.conf` file supplies them for each
//! instance it declares, the firmware or the parent nexus supplies them for
//! hardware it has enumerated, and the driver itself creates them at run time
//! with the `ddi_prop_update_*` functions. A lookup walks from the node
//! towards the root of the device tree unless `DDI_PROP_DONTPASS` is set,
//! which is what almost every driver wants.
//!
//! The values a property can hold are integers, 64-bit integers, strings and
//! byte arrays, each on its own or as an array. The lookup functions return
//! the value in memory the kernel allocates, which the caller releases with
//! `ddi_prop_free`. The `match_dev` argument ties a property to one minor
//! device; `DDI_DEV_T_ANY` matches any minor on lookup and `DDI_DEV_T_NONE`
//! creates a property that belongs to the node as a whole.
//!
//! The older `ddi_getprop`, `ddi_getlongprop`, `ddi_getlongprop_buf`,
//! `ddi_getproplen`, `ddi_prop_create` and `ddi_prop_modify` functions are
//! marked obsolete by their manual pages and are not declared here.
//! `ddi_prop_cache_invalidate` and `ddi_bus_prop_op` have no manual page and
//! are not in the mapfile, so they are left out as well.

use crate::devops::ddi_prop_op_t;
use crate::types::*;

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
/// The property exists but its encoded form could not be decoded into the
/// requested type (`DDI_PROP_CANNOT_DECODE`).
pub const DDI_PROP_CANNOT_DECODE: c_int = 6;
/// The value could not be encoded for storage (`DDI_PROP_CANNOT_ENCODE`).
pub const DDI_PROP_CANNOT_ENCODE: c_int = 7;
/// The end of the encoded property data was reached (`DDI_PROP_END_OF_DATA`).
pub const DDI_PROP_END_OF_DATA: c_int = 8;
/// The property was found in the Open Boot PROM 1275 encoding
/// (`DDI_PROP_FOUND_1275`).
pub const DDI_PROP_FOUND_1275: c_int = 255;

/// Look up the property on this node only, without consulting the parent
/// nodes (`DDI_PROP_DONTPASS`). Almost every driver lookup uses this flag.
pub const DDI_PROP_DONTPASS: uint_t = 0x0001;
/// The property function may block for memory (`DDI_PROP_CANSLEEP`).
pub const DDI_PROP_CANSLEEP: uint_t = 0x0002;
/// The property was defined by the system rather than by a driver
/// (`DDI_PROP_SYSTEM_DEF`). Drivers do not set this flag themselves.
pub const DDI_PROP_SYSTEM_DEF: uint_t = 0x0004;
/// Do not consult the firmware's properties, only those the driver and the
/// `.conf` file defined (`DDI_PROP_NOTPROM`).
pub const DDI_PROP_NOTPROM: uint_t = 0x0008;
/// The property function must not block for memory (`DDI_PROP_DONTSLEEP`).
pub const DDI_PROP_DONTSLEEP: uint_t = 0x0010;
/// The property was defined by the hardware (`DDI_PROP_HW_DEF`). Drivers do
/// not set this flag themselves.
pub const DDI_PROP_HW_DEF: uint_t = 0x0080;

/// The property holds one or more integers (`DDI_PROP_TYPE_INT`).
pub const DDI_PROP_TYPE_INT: uint_t = 0x0100;
/// The property holds one or more strings (`DDI_PROP_TYPE_STRING`).
pub const DDI_PROP_TYPE_STRING: uint_t = 0x0200;
/// The property holds bytes (`DDI_PROP_TYPE_BYTE`).
pub const DDI_PROP_TYPE_BYTE: uint_t = 0x0400;
/// The property holds a composite value (`DDI_PROP_TYPE_COMPOSITE`).
pub const DDI_PROP_TYPE_COMPOSITE: uint_t = 0x0800;
/// The property holds one or more 64-bit integers (`DDI_PROP_TYPE_INT64`).
pub const DDI_PROP_TYPE_INT64: uint_t = 0x1000;
/// Any of the property types (`DDI_PROP_TYPE_ANY`).
pub const DDI_PROP_TYPE_ANY: uint_t =
    DDI_PROP_TYPE_INT | DDI_PROP_TYPE_STRING | DDI_PROP_TYPE_BYTE | DDI_PROP_TYPE_COMPOSITE;
/// The mask that isolates the type bits of a flags word (`DDI_PROP_TYPE_MASK`).
pub const DDI_PROP_TYPE_MASK: uint_t = DDI_PROP_TYPE_ANY | DDI_PROP_TYPE_INT64;

unsafe extern "C" {
    /// `ddi_prop_exists(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether the named property exists on the node, or on one of
    /// its ancestors unless `flags` contains `DDI_PROP_DONTPASS`. Returns 1
    /// if it does and 0 if it does not. The function may be called from user
    /// or kernel context.
    pub fn ddi_prop_exists(match_dev: dev_t, dip: dev_info_t, flags: uint_t, name: *const c_char) -> c_int;
    /// `ddi_prop_get_int(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the value of an integer property, or `defvalue` if the
    /// property does not exist, has the wrong type, or cannot be decoded. A
    /// property that exists without a value is returned as 1, which is how
    /// boolean properties are written in a `.conf` file. The function may
    /// be called from user or kernel context.
    pub fn ddi_prop_get_int(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        defvalue: c_int,
    ) -> c_int;
    /// `ddi_prop_get_int(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The 64-bit counterpart of [`ddi_prop_get_int`]. Only properties that
    /// were created as 64-bit values match; a 32-bit integer property of the
    /// same name is not converted.
    pub fn ddi_prop_get_int64(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        defvalue: i64,
    ) -> i64;
    /// `ddi_prop_lookup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns an integer array property in a buffer that the kernel
    /// allocates, storing its address in `datap` and the element count in
    /// `nelementsp`. The caller releases the buffer with [`ddi_prop_free`].
    /// Returns `DDI_PROP_SUCCESS` or one of the other `DDI_PROP_*` codes.
    pub fn ddi_prop_lookup_int_array(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        datap: *mut *mut c_int,
        nelementsp: *mut uint_t,
    ) -> c_int;
    /// `ddi_prop_lookup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The 64-bit counterpart of [`ddi_prop_lookup_int_array`].
    pub fn ddi_prop_lookup_int64_array(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        datap: *mut *mut i64,
        nelementsp: *mut uint_t,
    ) -> c_int;
    /// `ddi_prop_lookup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a string array property as an array of pointers to
    /// NUL-terminated strings. The array and the strings are allocated
    /// together and released with one call to [`ddi_prop_free`] on the
    /// array.
    pub fn ddi_prop_lookup_string_array(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        datap: *mut *mut *mut c_char,
        nelementsp: *mut uint_t,
    ) -> c_int;
    /// `ddi_prop_lookup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a string property in a buffer allocated by the kernel. The
    /// caller releases it with [`ddi_prop_free`]. Returns one of the
    /// `DDI_PROP_*` codes.
    pub fn ddi_prop_lookup_string(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        datap: *mut *mut c_char,
    ) -> c_int;
    /// `ddi_prop_lookup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a byte array property in a buffer allocated by the kernel,
    /// with the byte count in `nelementsp`. The caller releases the buffer
    /// with [`ddi_prop_free`].
    pub fn ddi_prop_lookup_byte_array(
        match_dev: dev_t,
        dip: dev_info_t,
        flags: uint_t,
        name: *const c_char,
        datap: *mut *mut c_uchar,
        nelementsp: *mut uint_t,
    ) -> c_int;
    /// `ddi_prop_lookup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases a buffer returned by one of the `ddi_prop_lookup_*`
    /// functions.
    pub fn ddi_prop_free(data: *mut c_void);

    /// `ddi_prop_update(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates or replaces an integer array property on the device node.
    /// The `dev` argument is `DDI_DEV_T_NONE` for a property of the node
    /// itself, or the device number of the minor the property belongs to.
    /// The value is copied, so the caller's array need not outlive the call.
    /// Returns `DDI_PROP_SUCCESS` or one of the other `DDI_PROP_*` codes.
    /// The update functions may only be called from user or kernel context.
    pub fn ddi_prop_update_int_array(
        match_dev: dev_t,
        dip: dev_info_t,
        name: *const c_char,
        data: *const c_int,
        nelements: uint_t,
    ) -> c_int;
    /// `ddi_prop_update(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates or replaces an integer property on the device node.
    pub fn ddi_prop_update_int(match_dev: dev_t, dip: dev_info_t, name: *const c_char, data: c_int) -> c_int;
    /// `ddi_prop_update(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates or replaces a 64-bit integer array property on the device
    /// node.
    pub fn ddi_prop_update_int64_array(
        match_dev: dev_t,
        dip: dev_info_t,
        name: *const c_char,
        data: *const i64,
        nelements: uint_t,
    ) -> c_int;
    /// `ddi_prop_update(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates or replaces a 64-bit integer property on the device node.
    pub fn ddi_prop_update_int64(match_dev: dev_t, dip: dev_info_t, name: *const c_char, data: i64) -> c_int;
    /// `ddi_prop_update(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates or replaces a string array property on the device node from
    /// an array of `nelements` pointers to NUL-terminated strings.
    pub fn ddi_prop_update_string_array(
        match_dev: dev_t,
        dip: dev_info_t,
        name: *const c_char,
        data: *const *const c_char,
        nelements: uint_t,
    ) -> c_int;
    /// `ddi_prop_update(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates or replaces a string property on the device node.
    pub fn ddi_prop_update_string(match_dev: dev_t, dip: dev_info_t, name: *const c_char, data: *const c_char)
    -> c_int;
    /// `ddi_prop_update(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates or replaces a byte array property on the device node.
    pub fn ddi_prop_update_byte_array(
        match_dev: dev_t,
        dip: dev_info_t,
        name: *const c_char,
        data: *const c_uchar,
        nelements: uint_t,
    ) -> c_int;

    /// `ddi_prop_create(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes one property that the driver created on the device node.
    /// Returns `DDI_PROP_SUCCESS` or `DDI_PROP_NOT_FOUND`.
    pub fn ddi_prop_remove(dev: dev_t, dip: dev_info_t, name: *const c_char) -> c_int;
    /// `ddi_prop_create(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes every property the driver created on the device node.
    /// Conventionally called from `detach(9E)`.
    pub fn ddi_prop_remove_all(dip: dev_info_t);
    /// `ddi_prop_create(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Explicitly undefines a property, so that lookups on this node return
    /// `DDI_PROP_UNDEFINED` instead of consulting the parent nodes. The
    /// `flag` argument is 0 or `DDI_PROP_CANSLEEP`. A later
    /// [`ddi_prop_remove`] cancels the effect.
    pub fn ddi_prop_undefine(dev: dev_t, dip: dev_info_t, flag: c_int, name: *const c_char) -> c_int;

    /// `ddi_prop_op(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The default implementation of `prop_op(9E)`. Its signature matches
    /// [`crate::devops::cb_prop_op_t`], so it can be placed in `cb_prop_op`
    /// directly, and most drivers do exactly that. The meaning of `valuep`
    /// depends on `prop_op`. For `PROP_LEN` it is unused, for
    /// `PROP_LEN_AND_VAL_BUF` it points at the caller's buffer, and for
    /// `PROP_LEN_AND_VAL_ALLOC` it points at a pointer that receives a
    /// buffer the kernel allocates.
    pub fn ddi_prop_op(
        dev: dev_t,
        dip: dev_info_t,
        prop_op: ddi_prop_op_t,
        mod_flags: c_int,
        name: *mut c_char,
        valuep: caddr_t,
        lengthp: *mut c_int,
    ) -> c_int;
    /// `ddi_prop_op(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A `prop_op(9E)` helper for block drivers that answers the `Size` and
    /// `size` requests with `size64` bytes and the `Nblocks` and `nblocks`
    /// requests with that size divided by `DEV_BSIZE`, and delegates every
    /// other property to [`ddi_prop_op`]. This function has no manual page
    /// of its own; it is declared in `sys/sunddi.h` next to `ddi_prop_op`
    /// and used by the disk drivers in the gate.
    pub fn ddi_prop_op_size(
        dev: dev_t,
        dip: dev_info_t,
        prop_op: ddi_prop_op_t,
        mod_flags: c_int,
        name: *mut c_char,
        valuep: caddr_t,
        lengthp: *mut c_int,
        size64: u64,
    ) -> c_int;
    /// `ddi_prop_op(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`ddi_prop_op_size`], but reports block counts in units of
    /// `blksize` bytes and also answers the `blksize` property.
    pub fn ddi_prop_op_size_blksize(
        dev: dev_t,
        dip: dev_info_t,
        prop_op: ddi_prop_op_t,
        mod_flags: c_int,
        name: *mut c_char,
        valuep: caddr_t,
        lengthp: *mut c_int,
        size64: u64,
        blksize: uint_t,
    ) -> c_int;
    /// `ddi_prop_op(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A `prop_op(9E)` helper for block drivers that know their size in
    /// `DEV_BSIZE` blocks rather than in bytes. It answers the `Nblocks` and
    /// `nblocks` requests with `nblocks64`, derives `Size` and `size` from
    /// it, and delegates everything else to [`ddi_prop_op`].
    pub fn ddi_prop_op_nblocks(
        dev: dev_t,
        dip: dev_info_t,
        prop_op: ddi_prop_op_t,
        mod_flags: c_int,
        name: *mut c_char,
        valuep: caddr_t,
        lengthp: *mut c_int,
        nblocks64: u64,
    ) -> c_int;
    /// `ddi_prop_op(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`ddi_prop_op_nblocks`], but with blocks of `blksize` bytes.
    pub fn ddi_prop_op_nblocks_blksize(
        dev: dev_t,
        dip: dev_info_t,
        prop_op: ddi_prop_op_t,
        mod_flags: c_int,
        name: *mut c_char,
        valuep: caddr_t,
        lengthp: *mut c_int,
        nblocks64: u64,
        blksize: uint_t,
    ) -> c_int;
}
