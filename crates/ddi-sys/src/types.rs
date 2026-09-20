//! Base types and opaque handles from `sys/types.h` and `sys/dditypes.h`.
//!
//! The first half of this module gives the C typedefs their Rust spelling so
//! that the other modules can declare prototypes that read like the header.
//! The second half declares the kernel structures a driver only ever sees
//! through a pointer, using the [`opaque!`] macro, together with the handle
//! typedefs built on them.

use core::ffi;

pub use ffi::{c_char, c_int, c_long, c_longlong, c_short, c_uchar, c_uint, c_ulong, c_ulonglong, c_ushort, c_void};

/// The C `uint_t`, an unsigned 32-bit integer.
pub type uint_t = c_uint;
/// The C `ulong_t`, an unsigned 64-bit integer under LP64.
pub type ulong_t = c_ulong;
/// The C `longlong_t`, a signed 64-bit integer.
pub type longlong_t = c_longlong;
/// The C `size_t`.
pub type size_t = usize;
/// The C `ssize_t`.
pub type ssize_t = isize;
/// The C `intptr_t`, an integer wide enough to hold a pointer.
pub type intptr_t = isize;
/// The C `uintptr_t`, an unsigned integer wide enough to hold a pointer.
pub type uintptr_t = usize;
/// The C `caddr_t`, a pointer to bytes of kernel or user memory.
pub type caddr_t = *mut c_char;
/// The C `daddr_t`, a disk block address. It is a `long`.
pub type daddr_t = c_long;
/// The C `offset_t`, a 64-bit file offset.
pub type offset_t = longlong_t;
/// The C `off_t`, which is 64 bits wide under LP64.
pub type off_t = c_long;
/// The C `dev_t`, a device number combining major and minor parts. It is a `ulong_t`.
pub type dev_t = ulong_t;
/// The C `major_t`, the major part of a device number.
pub type major_t = uint_t;
/// The C `minor_t`, the minor part of a device number.
pub type minor_t = uint_t;
/// The C `pri_t`, a scheduling priority.
pub type pri_t = c_short;
/// The C `clock_t`, a time interval in clock ticks. It is a `long`.
pub type clock_t = c_long;
/// The C `hrtime_t`, a high-resolution time in nanoseconds.
pub type hrtime_t = longlong_t;
/// The C `boolean_t`, with the values [`B_FALSE`] and [`B_TRUE`].
pub type boolean_t = c_int;
/// The false value of `boolean_t`.
pub const B_FALSE: boolean_t = 0;
/// The true value of `boolean_t`.
pub const B_TRUE: boolean_t = 1;

opaque! {
    /// The device node structure, `struct __dev_info`. Drivers receive it as
    /// a `dev_info_t` pointer in `attach(9E)` and pass it to most `ddi_*`
    /// functions. Its contents are private to the kernel.
    dev_info;
    /// The credentials of a caller, `cred_t`. Drivers query it through the
    /// functions in [`crate::cred`] and never look inside.
    cred_t;
    /// The `struct modinfo` that the kernel passes to `_info(9E)`. A driver
    /// only forwards it to `mod_info(9F)`.
    modinfo;
    /// A module operations vector, `struct mod_ops`. The kernel exports one
    /// per module kind, such as `mod_driverops` for device drivers.
    mod_ops;
    /// The nexus operations vector, `struct bus_ops`. Leaf drivers set the
    /// corresponding `dev_ops` field to NULL; this crate does not bind nexus
    /// interfaces.
    bus_ops;
    /// The STREAMS driver table, `struct streamtab`. It will be defined in a
    /// separate `ddi-streams-sys` crate; here it exists only so that
    /// `cb_ops.cb_str` has a pointer type.
    streamtab;
    /// The block I/O request structure, `buf(9S)`. It stays opaque until the
    /// block-device family is bound.
    buf;
    /// The poll head structure that `chpoll(9E)` fills in.
    pollhead;
    /// A process address space, `struct as`, as passed to `segmap(9E)`. The C
    /// name is a Rust keyword, hence the different spelling.
    addrspace;
    /// An asynchronous I/O request, `struct aio_req`, as passed to `aread(9E)`
    /// and `awrite(9E)`.
    aio_req;
    /// The structure behind a register access handle, `ddi_acc_handle_t`.
    __ddi_acc_handle;
    /// The structure behind a DMA handle, `ddi_dma_handle_t`.
    __ddi_dma_handle;
    /// The structure behind a periodic callback handle, `ddi_periodic_t`.
    __ddi_periodic;
    /// A task queue created by `ddi_taskq_create(9F)`.
    ddi_taskq_t;
    /// The structure behind a soft interrupt handle, `ddi_softint_handle_t`.
    __ddi_softint_handle;
    /// The structure behind an interrupt handle, `ddi_intr_handle_t`.
    __ddi_intr_handle;
}

/// A pointer to a device node. This is how `dev_info_t` is defined in C as well.
pub type dev_info_t = *mut dev_info;
/// A register access handle returned by `ddi_regs_map_setup(9F)`.
pub type ddi_acc_handle_t = *mut __ddi_acc_handle;
/// A DMA handle returned by `ddi_dma_alloc_handle(9F)`.
pub type ddi_dma_handle_t = *mut __ddi_dma_handle;
/// A handle for a periodic callback registered with `ddi_periodic_add(9F)`.
pub type ddi_periodic_t = *mut __ddi_periodic;
/// A soft interrupt handle returned by `ddi_intr_add_softint(9F)`.
pub type ddi_softint_handle_t = *mut __ddi_softint_handle;
/// An interrupt handle returned by `ddi_intr_alloc(9F)`.
pub type ddi_intr_handle_t = *mut __ddi_intr_handle;
/// A device mapping cookie for the `devmap(9E)` framework. `ddidevmap.h`
/// defines it as a plain `void *`.
pub type devmap_cookie_t = *mut c_void;

/// Describes how device registers are to be accessed, `ddi_device_acc_attr(9S)`.
/// A driver fills one in and passes it to `ddi_regs_map_setup(9F)`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_device_acc_attr_t {
    /// The structure version, either `DDI_DEVICE_ATTR_V0` or `DDI_DEVICE_ATTR_V1`.
    pub devacc_attr_version: c_ushort,
    /// The byte order of the device, one of `DDI_NEVERSWAP_ACC`,
    /// `DDI_STRUCTURE_LE_ACC` or `DDI_STRUCTURE_BE_ACC`.
    pub devacc_attr_endian_flags: c_uchar,
    /// How strictly accesses must be ordered, from `DDI_STRICTORDER_ACC` to
    /// `DDI_STORECACHING_OK_ACC`.
    pub devacc_attr_dataorder: c_uchar,
    /// How access errors are handled, one of `DDI_DEFAULT_ACC`,
    /// `DDI_FLAGERR_ACC` or `DDI_CAUTIOUS_ACC`. Only meaningful with version 1.
    pub devacc_attr_access: c_uchar,
}

/// Version 0 of the access attributes, without error handling (`DDI_DEVICE_ATTR_V0`).
pub const DDI_DEVICE_ATTR_V0: c_ushort = 0x0001;
/// Version 1 of the access attributes, with the access error field (`DDI_DEVICE_ATTR_V1`).
pub const DDI_DEVICE_ATTR_V1: c_ushort = 0x0002;
/// The device uses the host byte order; never swap (`DDI_NEVERSWAP_ACC`).
pub const DDI_NEVERSWAP_ACC: c_uchar = 0x00;
/// The device is little-endian (`DDI_STRUCTURE_LE_ACC`).
pub const DDI_STRUCTURE_LE_ACC: c_uchar = 0x01;
/// The device is big-endian (`DDI_STRUCTURE_BE_ACC`).
pub const DDI_STRUCTURE_BE_ACC: c_uchar = 0x02;
/// Accesses happen exactly in program order (`DDI_STRICTORDER_ACC`).
pub const DDI_STRICTORDER_ACC: c_uchar = 0x00;
/// Accesses may be reordered (`DDI_UNORDERED_OK_ACC`).
pub const DDI_UNORDERED_OK_ACC: c_uchar = 0x01;
/// Adjacent accesses may be merged (`DDI_MERGING_OK_ACC`).
pub const DDI_MERGING_OK_ACC: c_uchar = 0x02;
/// Loads may be cached (`DDI_LOADCACHING_OK_ACC`).
pub const DDI_LOADCACHING_OK_ACC: c_uchar = 0x03;
/// Stores may be cached (`DDI_STORECACHING_OK_ACC`).
pub const DDI_STORECACHING_OK_ACC: c_uchar = 0x04;
/// Access errors take the default action, which is to panic (`DDI_DEFAULT_ACC`).
pub const DDI_DEFAULT_ACC: c_uchar = 0x01;
/// Access errors are recorded in the handle for the driver to check (`DDI_FLAGERR_ACC`).
pub const DDI_FLAGERR_ACC: c_uchar = 0x02;
/// Every access is protected against faults at the cost of speed (`DDI_CAUTIOUS_ACC`).
pub const DDI_CAUTIOUS_ACC: c_uchar = 0x03;

const _: () = assert!(core::mem::size_of::<ddi_device_acc_attr_t>() == 6);
