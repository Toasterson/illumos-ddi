//! The upgradeable firmware module framework from `sys/ddi_ufm.h`.
//!
//! An upgradeable firmware module, or UFM, is a persistent firmware image
//! that lives on a device: a disk's controller firmware, a NIC's option ROM,
//! a motherboard's BIOS or a CPU's microcode. The UFM framework lets a
//! driver report what images its device carries, how many slots each image
//! has, which slot is active and what version is in it, and optionally lets
//! the system read the raw payload of a slot. User programs reach this data
//! through the `ufm(4D)` driver; the device driver never marshals anything
//! itself.
//!
//! A driver registers in `attach(9E)` by filling in a `ddi_ufm_ops_t`
//! vector and calling `ddi_ufm_init(9F)`, then calls `ddi_ufm_update(9F)`
//! once it is ready to answer queries and again whenever the firmware
//! state may have changed. The entry points, described in `ddi_ufm(9E)`,
//! are called serially on a single handle from kernel context. Inside the
//! fill callbacks the driver describes an image or a slot with the setters
//! from `ddi_ufm_image(9F)` and `ddi_ufm_slot(9F)`; the image and slot
//! structures themselves are opaque. In `detach(9E)` the driver calls
//! `ddi_ufm_fini(9F)`, which blocks until any outstanding operation has
//! finished.
//!
//! Every manual page in this family carries the interface level "Evolving",
//! meaning that API and ABI stability is not guaranteed. The level is quoted
//! verbatim on each item. Nevertheless every one of the UFM functions bound
//! here, `ddi_ufm_init`, `ddi_ufm_update`, `ddi_ufm_fini`,
//! `ddi_ufm_image_set_desc`, `ddi_ufm_image_set_nslots`,
//! `ddi_ufm_image_set_misc`, `ddi_ufm_slot_set_version`,
//! `ddi_ufm_slot_set_attrs`, `ddi_ufm_slot_set_misc` and
//! `ddi_ufm_slot_set_imgsize`, appears in the `ddi` mapfile under
//! `uts/common/mapfiles/`, so a driver may link against them like any other
//! DDI symbol.
//!
//! The ioctl request structures and the `DDI_UFM_NV_*` nvlist key names in
//! the header describe the user interface of `ufm(4D)` and are not part of
//! the driver interface, so they are not declared here. Nothing is taken
//! from `ddi_ufm_impl.h`.
//!
//! The two C enumerations `ddi_ufm_cap_t` and `ddi_ufm_attr_t` are bit
//! fields that drivers combine with bitwise OR, so they are declared as
//! plain `c_int` aliases with their enumerators as constants rather than as
//! newtypes.

use crate::nvpair::nvlist_t;
use crate::types::*;

/// The current version of the UFM interface (`DDI_UFM_CURRENT_VERSION`).
/// Drivers in the gate pass this to `ddi_ufm_init(9F)`.
pub const DDI_UFM_CURRENT_VERSION: uint_t = 1;
/// The first version of the UFM interface (`DDI_UFM_VERSION_ONE`). Drivers
/// that need to bind to a specific revision pass this instead.
pub const DDI_UFM_VERSION_ONE: uint_t = 1;

/// The capabilities a driver reports from its `ddi_ufm_op_getcaps` entry
/// point, `ddi_ufm_cap_t`. A bitwise OR of the `DDI_UFM_CAP_*` values. The
/// C type is an enumeration.
pub type ddi_ufm_cap_t = c_int;
/// The driver can report image and slot information and implements the
/// fill image and fill slot entry points (`DDI_UFM_CAP_REPORT`). Every UFM
/// capable driver must set this.
pub const DDI_UFM_CAP_REPORT: ddi_ufm_cap_t = 1 << 0;
/// The driver can read a binary firmware payload from the device and
/// implements the read image entry point (`DDI_UFM_CAP_READIMG`).
pub const DDI_UFM_CAP_READIMG: ddi_ufm_cap_t = 1 << 1;

/// The attributes of a firmware slot, `ddi_ufm_attr_t`, passed to
/// `ddi_ufm_slot_set_attrs(9F)` as a bitwise OR of the `DDI_UFM_ATTR_*`
/// values. The C type is an enumeration.
pub type ddi_ufm_attr_t = c_int;
/// The image in the slot can be read, whether or not the driver currently
/// supports doing so (`DDI_UFM_ATTR_READABLE`).
pub const DDI_UFM_ATTR_READABLE: ddi_ufm_attr_t = 1 << 0;
/// The image in the slot can be updated, whether or not the driver
/// currently supports doing so (`DDI_UFM_ATTR_WRITEABLE`).
pub const DDI_UFM_ATTR_WRITEABLE: ddi_ufm_attr_t = 1 << 1;
/// The slot holds the firmware that is currently running
/// (`DDI_UFM_ATTR_ACTIVE`). Only one slot of an image should be active.
pub const DDI_UFM_ATTR_ACTIVE: ddi_ufm_attr_t = 1 << 2;
/// The slot holds no firmware image at present (`DDI_UFM_ATTR_EMPTY`).
pub const DDI_UFM_ATTR_EMPTY: ddi_ufm_attr_t = 1 << 3;
/// Every defined slot attribute bit combined (`DDI_UFM_ATTR_MAX`).
pub const DDI_UFM_ATTR_MAX: ddi_ufm_attr_t =
    DDI_UFM_ATTR_READABLE | DDI_UFM_ATTR_WRITEABLE | DDI_UFM_ATTR_ACTIVE | DDI_UFM_ATTR_EMPTY;

opaque! {
    /// The per-device UFM handle, `struct ddi_ufm_handle`. It is created by
    /// `ddi_ufm_init(9F)` and passed to every entry point. Its contents are
    /// private to the kernel.
    ddi_ufm_handle;
    /// A firmware image being described, `struct ddi_ufm_image`. The
    /// kernel passes one to the fill image entry point and the driver fills
    /// it through `ddi_ufm_image(9F)`.
    ddi_ufm_image;
    /// A firmware slot being described, `struct ddi_ufm_slot`. The kernel
    /// passes one to the fill slot entry point and the driver fills it
    /// through `ddi_ufm_slot(9F)`.
    ddi_ufm_slot;
}

/// The typedef name of [`ddi_ufm_handle`]. In C the typedef names the
/// structure itself, so functions take a pointer to it.
pub type ddi_ufm_handle_t = ddi_ufm_handle;
/// The typedef name of [`ddi_ufm_image`].
pub type ddi_ufm_image_t = ddi_ufm_image;
/// The typedef name of [`ddi_ufm_slot`].
pub type ddi_ufm_slot_t = ddi_ufm_slot;

/// `ddi_ufm_op_nimages(9E)`. Evolving - This interface is evolving still in illumos.
///
/// The signature of the optional entry point that reports how many
/// distinct firmware images the device carries. `uhp` is the handle from
/// `ddi_ufm_init(9F)` and `drv_arg` the private value the driver passed to
/// it. The driver stores the count, which may be zero, through `nimgp` and
/// returns 0, or an error number such as `EIO` if the device could not be
/// asked. A driver with a single image may leave the slot `None`, in which
/// case the system assumes one image. Images are numbered from zero.
pub type ddi_ufm_op_nimages_t =
    unsafe extern "C" fn(uhp: *mut ddi_ufm_handle_t, drv_arg: *mut c_void, nimgp: *mut uint_t) -> c_int;
/// `ddi_ufm_op_fill_image(9E)`. Evolving - This interface is evolving still in illumos.
///
/// The signature of the entry point that describes image number `imgno`.
/// The driver must call `ddi_ufm_image_set_desc(9F)` with a human readable
/// description of the image's purpose and `ddi_ufm_image_set_nslots(9F)`
/// with the number of slots the image has, and may attach ancillary data
/// with `ddi_ufm_image_set_misc(9F)`. It returns 0 once the image is
/// described, `EINVAL` if `imgno` is unknown, `EIO` if the device could not
/// be asked or `ENOMEM` if memory ran out.
pub type ddi_ufm_op_fill_image_t = unsafe extern "C" fn(
    uhp: *mut ddi_ufm_handle_t,
    drv_arg: *mut c_void,
    imgno: uint_t,
    imgp: *mut ddi_ufm_image_t,
) -> c_int;
/// `ddi_ufm_op_fill_slot(9E)`. Evolving - This interface is evolving still in illumos.
///
/// The signature of the entry point that describes slot number `slotno` of
/// image number `imgno`. The driver calls `ddi_ufm_slot_set_version(9F)`
/// with the vendor style version string of the firmware in the slot and
/// `ddi_ufm_slot_set_attrs(9F)` with the `DDI_UFM_ATTR_*` bits that apply.
/// A driver that supports reading images should also report the image size
/// with `ddi_ufm_slot_set_imgsize(9F)`, and any driver may attach ancillary
/// data with `ddi_ufm_slot_set_misc(9F)`. It returns 0 once the slot is
/// described, `EINVAL` if the image or slot is unknown, `EIO` if the device
/// could not be asked or `ENOMEM` if memory ran out. The manual page's copy
/// of the operations vector shows an extra image pointer argument for this
/// entry point; the header, which is what the kernel calls through, does
/// not have one, and the prototype here follows the header.
pub type ddi_ufm_op_fill_slot_t = unsafe extern "C" fn(
    uhp: *mut ddi_ufm_handle_t,
    drv_arg: *mut c_void,
    imgno: uint_t,
    slotno: uint_t,
    slotp: *mut ddi_ufm_slot_t,
) -> c_int;
/// `ddi_ufm_op_getcaps(9E)`. Evolving - This interface is evolving still in illumos.
///
/// The signature of the entry point that reports which UFM capabilities the
/// driver instance supports. The driver stores a bitwise OR of the
/// `DDI_UFM_CAP_*` values through `caps`, which must include
/// `DDI_UFM_CAP_REPORT`, and returns 0, or `EIO` or `ENOMEM` on failure.
pub type ddi_ufm_op_getcaps_t =
    unsafe extern "C" fn(uhp: *mut ddi_ufm_handle_t, drv_arg: *mut c_void, caps: *mut ddi_ufm_cap_t) -> c_int;
/// `ddi_ufm_op_readimg(9E)`. Evolving - This interface is evolving still in illumos.
///
/// The signature of the optional entry point that reads a binary firmware
/// payload. The driver copies up to `len` bytes starting `offset` bytes
/// into the image in slot `slotno` of image `imgno` into `buf`, stores the
/// number of bytes actually read through `nreadp` and returns 0. It returns
/// `EINVAL` if the image or slot is unknown or the requested range is
/// outside the image or violates an alignment restriction, `EIO` if the
/// device could not be read, and `ENOTSUP` if reading is not supported for
/// this device or slot. Only drivers that report `DDI_UFM_CAP_READIMG` need
/// to implement it.
pub type ddi_ufm_op_readimg_t = unsafe extern "C" fn(
    uhp: *mut ddi_ufm_handle_t,
    drv_arg: *mut c_void,
    imgno: uint_t,
    slotno: uint_t,
    len: u64,
    offset: u64,
    buf: *mut c_void,
    nreadp: *mut u64,
) -> c_int;

/// `ddi_ufm(9E)`. Evolving - This interface is evolving still in illumos.
///
/// The UFM operations vector a driver fills in and passes to
/// `ddi_ufm_init(9F)`. The fill image, fill slot and get capabilities entry
/// points are required; the count images and read image entry points are
/// optional and may be `None`. The kernel calls the entry points from
/// kernel context, and never more than one at a time on a given handle,
/// though other driver entry points may run concurrently and the driver
/// must lock its own state accordingly.
#[repr(C)]
pub struct ddi_ufm_ops {
    /// The optional `ddi_ufm_op_nimages(9E)` entry point.
    pub ddi_ufm_op_nimages: Option<ddi_ufm_op_nimages_t>,
    /// The `ddi_ufm_op_fill_image(9E)` entry point.
    pub ddi_ufm_op_fill_image: Option<ddi_ufm_op_fill_image_t>,
    /// The `ddi_ufm_op_fill_slot(9E)` entry point.
    pub ddi_ufm_op_fill_slot: Option<ddi_ufm_op_fill_slot_t>,
    /// The `ddi_ufm_op_getcaps(9E)` entry point.
    pub ddi_ufm_op_getcaps: Option<ddi_ufm_op_getcaps_t>,
    /// The optional `ddi_ufm_op_readimg(9E)` entry point.
    pub ddi_ufm_op_readimg: Option<ddi_ufm_op_readimg_t>,
}
/// The typedef name of [`ddi_ufm_ops`].
pub type ddi_ufm_ops_t = ddi_ufm_ops;

// Layout: five function pointers of 8 bytes each; total 40.
const _: () = assert!(core::mem::size_of::<ddi_ufm_ops>() == 40);
const _: () = assert!(core::mem::offset_of!(ddi_ufm_ops, ddi_ufm_op_readimg) == 32);

unsafe extern "C" {
    /// `ddi_ufm_init(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Registers the device instance `dip` with the UFM subsystem. `version`
    /// is the interface revision the driver was written against, normally
    /// `DDI_UFM_CURRENT_VERSION`, and `ops` is the driver's operations
    /// vector. `drv_arg`, usually the driver's soft state pointer, is handed
    /// back to every entry point. On success the function returns 0 and
    /// stores the new handle through `ufmpp`. No entry point is called until
    /// the driver has also called [`ddi_ufm_update`]. Usually called from
    /// `attach(9E)`; may be called from user or kernel context. The manual
    /// page gives `version` as `int`; the header declares it `uint_t`, which
    /// is the prototype bound here.
    pub fn ddi_ufm_init(
        dip: dev_info_t,
        version: uint_t,
        ops: *mut ddi_ufm_ops_t,
        ufmpp: *mut *mut ddi_ufm_handle_t,
        drv_arg: *mut c_void,
    ) -> c_int;
    /// `ddi_ufm_update(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Tells the UFM subsystem that the driver is ready to answer queries
    /// and that any information it has cached about this device's firmware
    /// should be discarded. A driver calls it once at the end of
    /// `attach(9E)` after [`ddi_ufm_init`], and again whenever the firmware
    /// or the device's capabilities may have changed, for example after a
    /// reset or an upgrade. It need not know that anything actually changed.
    /// May be called from any context with any locks held, except from a
    /// high level interrupt handler above lock level.
    pub fn ddi_ufm_update(ufmp: *mut ddi_ufm_handle_t);
    /// `ddi_ufm_fini(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Unregisters the device from the UFM subsystem and frees the handle.
    /// The function blocks until any UFM operation in progress has
    /// finished, so the driver must not hold any lock its entry points need
    /// across the call. When it returns, no further entry point will be
    /// called and the handle is invalid. Usually called from `detach(9E)`;
    /// may be called from user or kernel context.
    pub fn ddi_ufm_fini(ufmp: *mut ddi_ufm_handle_t);
    /// `ddi_ufm_image_set_desc(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Sets the description of the image `uip` to the NUL-terminated string
    /// `description`, which is intended for administrators and should
    /// convey what the image is for. The string is copied. Must be called
    /// only from within the `ddi_ufm_op_fill_image(9E)` entry point, where
    /// it is mandatory.
    pub fn ddi_ufm_image_set_desc(uip: *mut ddi_ufm_image_t, description: *const c_char);
    /// `ddi_ufm_image_set_nslots(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Sets the number of firmware slots the image `uip` has. Slots are
    /// numbered from zero to `nslots - 1` in later calls to the fill slot
    /// entry point. Must be called only from within the
    /// `ddi_ufm_op_fill_image(9E)` entry point, where it is mandatory.
    pub fn ddi_ufm_image_set_nslots(uip: *mut ddi_ufm_image_t, nslots: uint_t);
    /// `ddi_ufm_image_set_misc(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Attaches the nvlist `nvl` of device specific key and value pairs to
    /// the image `uip`. The driver allocates the list with
    /// `nvlist_alloc(9F)`; once passed in it belongs to the UFM subsystem
    /// and the driver must neither modify nor free it. Must be called only
    /// from within the `ddi_ufm_op_fill_image(9E)` entry point.
    pub fn ddi_ufm_image_set_misc(uip: *mut ddi_ufm_image_t, nvl: *mut nvlist_t);
    /// `ddi_ufm_slot_set_version(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Sets the version of the firmware in the slot `usp` to the
    /// NUL-terminated ASCII string `version`, which should be readable by an
    /// administrator and follow the vendor's own format. The string is
    /// copied. Must be called only from within the
    /// `ddi_ufm_op_fill_slot(9E)` entry point.
    pub fn ddi_ufm_slot_set_version(usp: *mut ddi_ufm_slot_t, version: *const c_char);
    /// `ddi_ufm_slot_set_attrs(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Sets the attributes of the slot `usp` to `attrs`, a bitwise OR of the
    /// `DDI_UFM_ATTR_*` values. Must be called only from within the
    /// `ddi_ufm_op_fill_slot(9E)` entry point.
    pub fn ddi_ufm_slot_set_attrs(usp: *mut ddi_ufm_slot_t, attrs: ddi_ufm_attr_t);
    /// `ddi_ufm_slot_set_misc(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Attaches the nvlist `nvl` of device specific key and value pairs to
    /// the slot `usp`, for example details of when or how the firmware was
    /// built or installed. The driver allocates the list with
    /// `nvlist_alloc(9F)`; once passed in it belongs to the UFM subsystem
    /// and the driver must neither modify nor free it. Must be called only
    /// from within the `ddi_ufm_op_fill_slot(9E)` entry point.
    pub fn ddi_ufm_slot_set_misc(usp: *mut ddi_ufm_slot_t, nvl: *mut nvlist_t);
    /// `ddi_ufm_slot_set_imgsize(9F)`. Evolving - This interface is evolving still in illumos.
    ///
    /// Records that the firmware image in the slot `usp` is `len` bytes
    /// long, which is the size of the image rather than of the slot.
    /// Consumers use it to know how much to read through the read image
    /// entry point. Must be called only from within the
    /// `ddi_ufm_op_fill_slot(9E)` entry point.
    pub fn ddi_ufm_slot_set_imgsize(usp: *mut ddi_ufm_slot_t, len: u64);
}
