//! Device operations from `sys/devops.h`.
//!
//! Two structures connect a driver to the kernel. `dev_ops(9S)` holds the
//! autoconfiguration entry points such as `attach(9E)` and `detach(9E)`, and
//! `cb_ops(9S)` holds the character and block entry points such as `open(9E)`
//! and `read(9E)`. This module declares both, the function pointer type of
//! every slot, and the command enumerations the kernel passes to
//! `attach(9E)`, `detach(9E)` and `getinfo(9E)`.

use crate::types::*;
use crate::uio::uio;

/// The current revision of the `dev_ops` structure (`DEVO_REV`). A driver
/// sets `devo_rev` to this value.
pub const DEVO_REV: c_int = 4;
/// The current revision of the `cb_ops` structure (`CB_REV`). A driver sets
/// `cb_rev` to this value.
pub const CB_REV: c_int = 1;

/// The command passed to `getinfo(9E)`, `ddi_info_cmd_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_info_cmd_t(pub c_int);
impl ddi_info_cmd_t {
    /// Asks for the `dev_info_t` that owns a `dev_t`. The manual marks this
    /// command as deprecated, but the kernel still issues it.
    pub const DDI_INFO_DEVT2DEVINFO: Self = Self(0);
    /// Asks for the instance number that owns a `dev_t`.
    pub const DDI_INFO_DEVT2INSTANCE: Self = Self(1);
}

/// The command passed to `attach(9E)`, `ddi_attach_cmd_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_attach_cmd_t(pub c_int);
impl ddi_attach_cmd_t {
    /// Attach a new instance (`DDI_ATTACH`).
    pub const DDI_ATTACH: Self = Self(0);
    /// Resume an instance that was suspended (`DDI_RESUME`).
    pub const DDI_RESUME: Self = Self(1);
    /// Resume after a power management suspend (`DDI_PM_RESUME`). The kernel
    /// no longer delivers this command to drivers.
    pub const DDI_PM_RESUME: Self = Self(2);
}

/// The command passed to `detach(9E)`, `ddi_detach_cmd_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_detach_cmd_t(pub c_int);
impl ddi_detach_cmd_t {
    /// Detach the instance (`DDI_DETACH`).
    pub const DDI_DETACH: Self = Self(0);
    /// Suspend the instance before a system suspend (`DDI_SUSPEND`).
    pub const DDI_SUSPEND: Self = Self(1);
    /// Suspend for power management (`DDI_PM_SUSPEND`). No longer delivered.
    pub const DDI_PM_SUSPEND: Self = Self(2);
}

/// The command passed to the obsolete `devo_reset` entry point,
/// `ddi_reset_cmd_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_reset_cmd_t(pub c_int);
impl ddi_reset_cmd_t {
    /// Reset the device unconditionally (`DDI_RESET_FORCE`).
    pub const DDI_RESET_FORCE: Self = Self(0);
}

/// The operation requested of `prop_op(9E)`, `ddi_prop_op_t` from
/// `sys/ddipropdefs.h`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_prop_op_t(pub c_int);
impl ddi_prop_op_t {
    /// Return only the length of the property (`PROP_LEN`).
    pub const PROP_LEN: Self = Self(0);
    /// Return the length and copy the value into the caller's buffer
    /// (`PROP_LEN_AND_VAL_BUF`).
    pub const PROP_LEN_AND_VAL_BUF: Self = Self(1);
    /// Return the length and the value in a newly allocated buffer
    /// (`PROP_LEN_AND_VAL_ALLOC`).
    pub const PROP_LEN_AND_VAL_ALLOC: Self = Self(2);
    /// Report whether the property exists (`PROP_EXISTS`).
    pub const PROP_EXISTS: Self = Self(3);
}

/// The signature of `getinfo(9E)`.
pub type devo_getinfo_t =
    unsafe extern "C" fn(dip: dev_info_t, cmd: ddi_info_cmd_t, arg: *mut c_void, result: *mut *mut c_void) -> c_int;
/// The signature of `identify(9E)`. The entry point is obsolete and drivers
/// install a do-nothing stub.
pub type devo_identify_t = unsafe extern "C" fn(dip: dev_info_t) -> c_int;
/// The signature of `probe(9E)`.
pub type devo_probe_t = unsafe extern "C" fn(dip: dev_info_t) -> c_int;
/// The signature of `attach(9E)`.
pub type devo_attach_t = unsafe extern "C" fn(dip: dev_info_t, cmd: ddi_attach_cmd_t) -> c_int;
/// The signature of `detach(9E)`.
pub type devo_detach_t = unsafe extern "C" fn(dip: dev_info_t, cmd: ddi_detach_cmd_t) -> c_int;
/// The signature of the obsolete `devo_reset` entry point. Drivers install
/// a stub that returns `ENXIO`.
pub type devo_reset_t = unsafe extern "C" fn(dip: dev_info_t, cmd: ddi_reset_cmd_t) -> c_int;
/// The signature of `power(9E)`.
pub type devo_power_t = unsafe extern "C" fn(dip: dev_info_t, component: c_int, level: c_int) -> c_int;
/// The signature of `quiesce(9E)`.
pub type devo_quiesce_t = unsafe extern "C" fn(dip: dev_info_t) -> c_int;

/// The autoconfiguration entry points of a driver, `dev_ops(9S)`.
///
/// The kernel increments and decrements `devo_refcnt` while the driver is
/// in use, so a `dev_ops` has to live in writable memory rather than in a
/// read-only static.
#[repr(C)]
pub struct dev_ops {
    /// Always [`DEVO_REV`].
    pub devo_rev: c_int,
    /// A reference count owned by the kernel. Initialise it to 0 and never
    /// touch it afterwards.
    pub devo_refcnt: c_int,
    /// The `getinfo(9E)` entry point.
    pub devo_getinfo: Option<devo_getinfo_t>,
    /// The obsolete `identify(9E)` entry point. Install a stub returning 0.
    pub devo_identify: Option<devo_identify_t>,
    /// The `probe(9E)` entry point, or a stub returning 0 for self-identifying
    /// devices.
    pub devo_probe: Option<devo_probe_t>,
    /// The `attach(9E)` entry point.
    pub devo_attach: Option<devo_attach_t>,
    /// The `detach(9E)` entry point.
    pub devo_detach: Option<devo_detach_t>,
    /// The obsolete reset entry point. Install a stub returning `ENXIO`.
    pub devo_reset: Option<devo_reset_t>,
    /// The character and block entry points for a leaf driver.
    pub devo_cb_ops: *mut cb_ops,
    /// The nexus operations for a bus driver. Leaf drivers set this to NULL.
    pub devo_bus_ops: *mut bus_ops,
    /// The `power(9E)` entry point, or a stub for drivers without power
    /// management.
    pub devo_power: Option<devo_power_t>,
    /// The `quiesce(9E)` entry point. Drivers that have nothing to quiesce
    /// install `ddi_quiesce_not_needed`; drivers that cannot quiesce install
    /// `ddi_quiesce_not_supported`.
    pub devo_quiesce: Option<devo_quiesce_t>,
}

const _: () = assert!(core::mem::size_of::<dev_ops>() == 8 + 10 * 8);

/// The signature of `open(9E)`.
pub type cb_open_t = unsafe extern "C" fn(devp: *mut dev_t, flag: c_int, otyp: c_int, credp: *mut cred_t) -> c_int;
/// The signature of `close(9E)`.
pub type cb_close_t = unsafe extern "C" fn(dev: dev_t, flag: c_int, otyp: c_int, credp: *mut cred_t) -> c_int;
/// The signature of `strategy(9E)`.
pub type cb_strategy_t = unsafe extern "C" fn(bp: *mut buf) -> c_int;
/// The signature of `print(9E)`.
pub type cb_print_t = unsafe extern "C" fn(dev: dev_t, str: *mut c_char) -> c_int;
/// The signature of `dump(9E)`.
pub type cb_dump_t = unsafe extern "C" fn(dev: dev_t, addr: caddr_t, blkno: daddr_t, nblk: c_int) -> c_int;
/// The signature of `read(9E)`.
pub type cb_read_t = unsafe extern "C" fn(dev: dev_t, uiop: *mut uio, credp: *mut cred_t) -> c_int;
/// The signature of `write(9E)`.
pub type cb_write_t = unsafe extern "C" fn(dev: dev_t, uiop: *mut uio, credp: *mut cred_t) -> c_int;
/// The signature of `ioctl(9E)`.
pub type cb_ioctl_t = unsafe extern "C" fn(
    dev: dev_t,
    cmd: c_int,
    arg: intptr_t,
    mode: c_int,
    credp: *mut cred_t,
    rvalp: *mut c_int,
) -> c_int;
/// The signature of `devmap(9E)`.
pub type cb_devmap_t = unsafe extern "C" fn(
    dev: dev_t,
    dhp: devmap_cookie_t,
    off: offset_t,
    len: size_t,
    maplen: *mut size_t,
    model: uint_t,
) -> c_int;
/// The signature of the obsolete `mmap(9E)`.
pub type cb_mmap_t = unsafe extern "C" fn(dev: dev_t, off: off_t, prot: c_int) -> c_int;
/// The signature of `segmap(9E)`.
pub type cb_segmap_t = unsafe extern "C" fn(
    dev: dev_t,
    off: off_t,
    asp: *mut addrspace,
    addrp: *mut caddr_t,
    len: off_t,
    prot: c_uint,
    maxprot: c_uint,
    flags: c_uint,
    credp: *mut cred_t,
) -> c_int;
/// The signature of `chpoll(9E)`.
pub type cb_chpoll_t = unsafe extern "C" fn(
    dev: dev_t,
    events: c_short,
    anyyet: c_int,
    reventsp: *mut c_short,
    phpp: *mut *mut pollhead,
) -> c_int;
/// The signature of `prop_op(9E)`.
pub type cb_prop_op_t = unsafe extern "C" fn(
    dev: dev_t,
    dip: dev_info_t,
    prop_op: ddi_prop_op_t,
    mod_flags: c_int,
    name: *mut c_char,
    valuep: caddr_t,
    length: *mut c_int,
) -> c_int;
/// The signature of `aread(9E)`.
pub type cb_aread_t = unsafe extern "C" fn(dev: dev_t, aio: *mut aio_req, credp: *mut cred_t) -> c_int;
/// The signature of `awrite(9E)`.
pub type cb_awrite_t = unsafe extern "C" fn(dev: dev_t, aio: *mut aio_req, credp: *mut cred_t) -> c_int;

/// The character and block device entry points of a driver, `cb_ops(9S)`.
///
/// Slots a driver does not implement are filled with stubs. The `ddi` crate
/// provides a correctly typed stub for each slot.
#[repr(C)]
pub struct cb_ops {
    /// The `open(9E)` entry point.
    pub cb_open: Option<cb_open_t>,
    /// The `close(9E)` entry point.
    pub cb_close: Option<cb_close_t>,
    /// The `strategy(9E)` entry point, used by block devices only.
    pub cb_strategy: Option<cb_strategy_t>,
    /// The `print(9E)` entry point, used by block devices only.
    pub cb_print: Option<cb_print_t>,
    /// The `dump(9E)` entry point, used by block devices that can receive a
    /// crash dump.
    pub cb_dump: Option<cb_dump_t>,
    /// The `read(9E)` entry point.
    pub cb_read: Option<cb_read_t>,
    /// The `write(9E)` entry point.
    pub cb_write: Option<cb_write_t>,
    /// The `ioctl(9E)` entry point.
    pub cb_ioctl: Option<cb_ioctl_t>,
    /// The `devmap(9E)` entry point.
    pub cb_devmap: Option<cb_devmap_t>,
    /// The obsolete `mmap(9E)` entry point.
    pub cb_mmap: Option<cb_mmap_t>,
    /// The `segmap(9E)` entry point.
    pub cb_segmap: Option<cb_segmap_t>,
    /// The `chpoll(9E)` entry point, or `nochpoll` for devices that cannot be
    /// polled.
    pub cb_chpoll: Option<cb_chpoll_t>,
    /// The `prop_op(9E)` entry point. Most drivers install `ddi_prop_op`.
    pub cb_prop_op: Option<cb_prop_op_t>,
    /// The STREAMS table. Only STREAMS drivers set this; all others use NULL.
    pub cb_str: *mut streamtab,
    /// The driver flags from `sys/conf.h`, at least `D_MP`, usually combined
    /// with `D_64BIT` and `D_NEW`.
    pub cb_flag: c_int,
    /// Always [`CB_REV`].
    pub cb_rev: c_int,
    /// The `aread(9E)` entry point for asynchronous reads.
    pub cb_aread: Option<cb_aread_t>,
    /// The `awrite(9E)` entry point for asynchronous writes.
    pub cb_awrite: Option<cb_awrite_t>,
}

const _: () = assert!(core::mem::size_of::<cb_ops>() == 13 * 8 + 8 + 8 + 2 * 8);
