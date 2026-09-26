//! Device identifiers, the `ddi_devid_*(9F)` family from `sys/sunddi.h` and
//! `sys/dditypes.h`.
//!
//! A device ID is a persistent, globally unique name for a piece of storage,
//! derived from firmware data such as a SCSI world wide name or a serial
//! number, or fabricated by the kernel when the hardware offers nothing
//! better. Kernel modules use device IDs to find a device again regardless
//! of where it is plugged in or what logical name it currently has. A driver
//! builds one with `ddi_devid_init(9F)`, registers it against its device
//! node in `attach(9E)` with `ddi_devid_register(9F)`, and unregisters and
//! frees it again in `detach(9E)`. The string forms produced by
//! `ddi_devid_str_encode(9F)` are what appear in `/dev` paths and in
//! configuration files.

use crate::sunddi::ushort_t;
use crate::types::*;

opaque! {
    /// The structure behind a device ID, `struct __ddi_devid`. Its contents
    /// are private to the kernel; a driver only compares, copies and
    /// serialises device IDs through the `ddi_devid_*` functions.
    __ddi_devid;
}

/// A pointer to a device ID. This is how `ddi_devid_t` is defined in C as
/// well.
pub type ddi_devid_t = *mut __ddi_devid;

/// No device ID type (`DEVID_NONE`).
pub const DEVID_NONE: ushort_t = 0;
/// A world wide name associated with a SCSI-3 device (`DEVID_SCSI3_WWN`).
pub const DEVID_SCSI3_WWN: ushort_t = 1;
/// A vendor ID and serial number from a SCSI device (`DEVID_SCSI_SERIAL`).
/// Only to be used when known to be unique; otherwise a fabricated ID is
/// required.
pub const DEVID_SCSI_SERIAL: ushort_t = 2;
/// A fabricated device ID built from the host ID and a timestamp
/// (`DEVID_FAB`).
pub const DEVID_FAB: ushort_t = 3;
/// The device ID of another device, for layered drivers (`DEVID_ENCAP`).
pub const DEVID_ENCAP: ushort_t = 4;
/// A serial number from an ATA device (`DEVID_ATA_SERIAL`).
pub const DEVID_ATA_SERIAL: ushort_t = 5;
/// A T10 vendor ID from a SCSI-3 vital product data page (`DEVID_SCSI3_VPD_T10`).
pub const DEVID_SCSI3_VPD_T10: ushort_t = 6;
/// An EUI-64 identifier from a SCSI-3 vital product data page (`DEVID_SCSI3_VPD_EUI`).
pub const DEVID_SCSI3_VPD_EUI: ushort_t = 7;
/// An NAA identifier from a SCSI-3 vital product data page (`DEVID_SCSI3_VPD_NAA`).
pub const DEVID_SCSI3_VPD_NAA: ushort_t = 8;
/// An NVMe namespace identifier (`DEVID_NVME_NSID`).
pub const DEVID_NVME_NSID: ushort_t = 9;
/// An NVMe EUI-64 identifier (`DEVID_NVME_EUI64`).
pub const DEVID_NVME_EUI64: ushort_t = 10;
/// An NVMe namespace globally unique identifier (`DEVID_NVME_NGUID`).
pub const DEVID_NVME_NGUID: ushort_t = 11;
/// The highest defined device ID type (`DEVID_MAXTYPE`).
pub const DEVID_MAXTYPE: ushort_t = 11;

unsafe extern "C" {
    /// `ddi_devid_init(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates and initialises an opaque device ID of type `devid_type` for
    /// the device node `dip`, storing it through `ret_devid`. For the types
    /// `DEVID_SCSI3_WWN`, `DEVID_SCSI_SERIAL` and `DEVID_ENCAP` the caller
    /// passes `nbytes` bytes of identifying data in `id`; for `DEVID_FAB` the
    /// `id` pointer must be NULL and `nbytes` zero, and the kernel fabricates
    /// an ID from the host ID and a timestamp. The function does not store the
    /// ID anywhere; when the ID is not derived from device firmware the driver
    /// is responsible for keeping it on some reliable store. The memory must
    /// be released with [`ddi_devid_free`]. Returns `DDI_SUCCESS`, or
    /// `DDI_FAILURE` when memory is short or the type is invalid.
    pub fn ddi_devid_init(
        dip: dev_info_t,
        devid_type: ushort_t,
        nbytes: ushort_t,
        id: *mut c_void,
        ret_devid: *mut ddi_devid_t,
    ) -> c_int;
    /// `ddi_devid_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees a device ID returned by [`ddi_devid_init`], [`ddi_devid_get`] or
    /// [`ddi_devid_str_decode`].
    pub fn ddi_devid_free(devid: ddi_devid_t);
    /// `ddi_devid_register(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers `devid` with the DDI framework as the device ID of the node
    /// `dip`. Drivers register their device IDs at attach time. Returns
    /// `DDI_SUCCESS`, or `DDI_FAILURE` when the node already has a device ID
    /// or the ID is invalid.
    pub fn ddi_devid_register(dip: dev_info_t, devid: ddi_devid_t) -> c_int;
    /// `ddi_devid_unregister(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the device ID from the node `dip`. Drivers call it when the
    /// device is being detached. It does not free the ID; the driver still
    /// has to call [`ddi_devid_free`].
    pub fn ddi_devid_unregister(dip: dev_info_t);
    /// `ddi_devid_valid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Validates a device ID. A driver must use it on any fabricated device
    /// ID that it has read back from a device before trusting it. Returns
    /// `DDI_SUCCESS` for a valid ID and `DDI_FAILURE` otherwise.
    pub fn ddi_devid_valid(devid: ddi_devid_t) -> c_int;
    /// `ddi_devid_get(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a copy of the device ID registered for the node `dip` through
    /// `ret_devid`. The copy is newly allocated and must be released with
    /// [`ddi_devid_free`]. Returns `DDI_SUCCESS`, or `DDI_FAILURE` when no
    /// device ID is registered for the node.
    pub fn ddi_devid_get(dip: dev_info_t, ret_devid: *mut ddi_devid_t) -> c_int;
    /// `ddi_devid_sizeof(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the number of bytes allocated for `devid`. When called with
    /// NULL it returns the number of bytes that must be allocated and
    /// initialised before the size of a complete device ID can be determined.
    pub fn ddi_devid_sizeof(devid: ddi_devid_t) -> size_t;
    /// `ddi_devid_compare(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Compares two device IDs byte by byte. Returns -1 if the first sorts
    /// before the second, 0 if they are equal and 1 if the first sorts after
    /// the second.
    pub fn ddi_devid_compare(id1: ddi_devid_t, id2: ddi_devid_t) -> c_int;
    /// `ddi_devid_str_encode(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Encodes a device ID and a minor name into a NUL terminated ASCII
    /// string. When both are given, a slash separates the ID from the minor
    /// name; when `minor_name` is NULL only the ID is encoded; and when
    /// `devid` is NULL the special string `id0` is produced. The result
    /// cannot be compared with `strcmp` to test for equality of device IDs.
    /// Returns the string, which must be released with
    /// [`ddi_devid_str_free`], or NULL when the ID cannot be encoded.
    pub fn ddi_devid_str_encode(devid: ddi_devid_t, minor_name: *mut c_char) -> *mut c_char;
    /// `ddi_devid_str_decode(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Decodes a string produced by [`ddi_devid_str_encode`] or by the
    /// user-space `devid_str_encode(3DEVID)`, allocating and returning the
    /// device ID through `devidp` and the minor name through `minor_namep`.
    /// For the special string `id0` both results are NULL. A non-NULL device
    /// ID is released with [`ddi_devid_free`] and a non-NULL minor name with
    /// [`ddi_devid_str_free`]. Returns `DDI_SUCCESS`, or `DDI_FAILURE` when
    /// the string is not valid.
    pub fn ddi_devid_str_decode(
        devidstr: *mut c_char,
        devidp: *mut ddi_devid_t,
        minor_namep: *mut *mut c_char,
    ) -> c_int;
    /// `ddi_devid_str_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees a string returned by [`ddi_devid_str_encode`] or a minor name
    /// returned by [`ddi_devid_str_decode`]. The manual page shows an `int`
    /// return; the header declares the function `void`, and the header is
    /// followed here.
    pub fn ddi_devid_str_free(devidstr: *mut c_char);
}
