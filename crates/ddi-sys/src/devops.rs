//! `sys/devops.h`: `dev_ops(9S)`, `cb_ops(9S)` and the command enumerations
//! passed to `attach(9E)`, `detach(9E)`, `getinfo(9E)`.

use crate::types::*;
use crate::uio::uio;

/// `DEVO_REV`: current `dev_ops` revision.
pub const DEVO_REV: c_int = 4;
/// `CB_REV`: current `cb_ops` revision.
pub const CB_REV: c_int = 1;

/// `ddi_info_cmd_t`, argument to `getinfo(9E)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_info_cmd_t(pub c_int);
impl ddi_info_cmd_t {
    /// Convert a `dev_t` to a `dev_info_t` (deprecated but still issued).
    pub const DDI_INFO_DEVT2DEVINFO: Self = Self(0);
    /// Convert a `dev_t` to an instance number.
    pub const DDI_INFO_DEVT2INSTANCE: Self = Self(1);
}

/// `ddi_attach_cmd_t`, argument to `attach(9E)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_attach_cmd_t(pub c_int);
impl ddi_attach_cmd_t {
    /// `DDI_ATTACH`
    pub const DDI_ATTACH: Self = Self(0);
    /// `DDI_RESUME`
    pub const DDI_RESUME: Self = Self(1);
    /// `DDI_PM_RESUME` (not delivered to drivers today)
    pub const DDI_PM_RESUME: Self = Self(2);
}

/// `ddi_detach_cmd_t`, argument to `detach(9E)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_detach_cmd_t(pub c_int);
impl ddi_detach_cmd_t {
    /// `DDI_DETACH`
    pub const DDI_DETACH: Self = Self(0);
    /// `DDI_SUSPEND`
    pub const DDI_SUSPEND: Self = Self(1);
    /// `DDI_PM_SUSPEND`
    pub const DDI_PM_SUSPEND: Self = Self(2);
}

/// `ddi_reset_cmd_t`, argument to `devo_reset`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_reset_cmd_t(pub c_int);
impl ddi_reset_cmd_t {
    /// `DDI_RESET_FORCE`
    pub const DDI_RESET_FORCE: Self = Self(0);
}

/// `ddi_prop_op_t` (`sys/ddipropdefs.h`), argument to `prop_op(9E)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_prop_op_t(pub c_int);
impl ddi_prop_op_t {
    /// `PROP_LEN`
    pub const PROP_LEN: Self = Self(0);
    /// `PROP_LEN_AND_VAL_BUF`
    pub const PROP_LEN_AND_VAL_BUF: Self = Self(1);
    /// `PROP_LEN_AND_VAL_ALLOC`
    pub const PROP_LEN_AND_VAL_ALLOC: Self = Self(2);
    /// `PROP_EXISTS`
    pub const PROP_EXISTS: Self = Self(3);
}

/// `getinfo(9E)`
pub type devo_getinfo_t =
    unsafe extern "C" fn(dip: dev_info_t, cmd: ddi_info_cmd_t, arg: *mut c_void, result: *mut *mut c_void) -> c_int;
/// `identify(9E)` (obsolete; use `nulldev`)
pub type devo_identify_t = unsafe extern "C" fn(dip: dev_info_t) -> c_int;
/// `probe(9E)`
pub type devo_probe_t = unsafe extern "C" fn(dip: dev_info_t) -> c_int;
/// `attach(9E)`
pub type devo_attach_t = unsafe extern "C" fn(dip: dev_info_t, cmd: ddi_attach_cmd_t) -> c_int;
/// `detach(9E)`
pub type devo_detach_t = unsafe extern "C" fn(dip: dev_info_t, cmd: ddi_detach_cmd_t) -> c_int;
/// `devo_reset` (obsolete; use `nodev`)
pub type devo_reset_t = unsafe extern "C" fn(dip: dev_info_t, cmd: ddi_reset_cmd_t) -> c_int;
/// `power(9E)`
pub type devo_power_t = unsafe extern "C" fn(dip: dev_info_t, component: c_int, level: c_int) -> c_int;
/// `quiesce(9E)`
pub type devo_quiesce_t = unsafe extern "C" fn(dip: dev_info_t) -> c_int;

/// `dev_ops(9S)`: device operations. The kernel writes `devo_refcnt`, so a
/// `dev_ops` must live in writable memory.
#[repr(C)]
pub struct dev_ops {
    /// `DEVO_REV`
    pub devo_rev: c_int,
    /// Managed by the kernel; initialise to 0.
    pub devo_refcnt: c_int,
    /// `getinfo(9E)`
    pub devo_getinfo: Option<devo_getinfo_t>,
    /// `identify(9E)`, obsolete: `nulldev`
    pub devo_identify: Option<devo_identify_t>,
    /// `probe(9E)`
    pub devo_probe: Option<devo_probe_t>,
    /// `attach(9E)`
    pub devo_attach: Option<devo_attach_t>,
    /// `detach(9E)`
    pub devo_detach: Option<devo_detach_t>,
    /// obsolete: `nodev`
    pub devo_reset: Option<devo_reset_t>,
    /// `cb_ops(9S)` for leaf drivers.
    pub devo_cb_ops: *mut cb_ops,
    /// `bus_ops(9S)` for nexus drivers, else NULL.
    pub devo_bus_ops: *mut bus_ops,
    /// `power(9E)`
    pub devo_power: Option<devo_power_t>,
    /// `quiesce(9E)`; `ddi_quiesce_not_needed` or `ddi_quiesce_not_supported`.
    pub devo_quiesce: Option<devo_quiesce_t>,
}

const _: () = assert!(core::mem::size_of::<dev_ops>() == 8 + 10 * 8);

/// `open(9E)`
pub type cb_open_t = unsafe extern "C" fn(devp: *mut dev_t, flag: c_int, otyp: c_int, credp: *mut cred_t) -> c_int;
/// `close(9E)`
pub type cb_close_t = unsafe extern "C" fn(dev: dev_t, flag: c_int, otyp: c_int, credp: *mut cred_t) -> c_int;
/// `strategy(9E)`
pub type cb_strategy_t = unsafe extern "C" fn(bp: *mut buf) -> c_int;
/// `print(9E)`
pub type cb_print_t = unsafe extern "C" fn(dev: dev_t, str: *mut c_char) -> c_int;
/// `dump(9E)`
pub type cb_dump_t = unsafe extern "C" fn(dev: dev_t, addr: caddr_t, blkno: daddr_t, nblk: c_int) -> c_int;
/// `read(9E)`
pub type cb_read_t = unsafe extern "C" fn(dev: dev_t, uiop: *mut uio, credp: *mut cred_t) -> c_int;
/// `write(9E)`
pub type cb_write_t = unsafe extern "C" fn(dev: dev_t, uiop: *mut uio, credp: *mut cred_t) -> c_int;
/// `ioctl(9E)`
pub type cb_ioctl_t = unsafe extern "C" fn(
    dev: dev_t,
    cmd: c_int,
    arg: intptr_t,
    mode: c_int,
    credp: *mut cred_t,
    rvalp: *mut c_int,
) -> c_int;
/// `devmap(9E)`
pub type cb_devmap_t = unsafe extern "C" fn(
    dev: dev_t,
    dhp: devmap_cookie_t,
    off: offset_t,
    len: size_t,
    maplen: *mut size_t,
    model: uint_t,
) -> c_int;
/// `mmap(9E)` (obsolete)
pub type cb_mmap_t = unsafe extern "C" fn(dev: dev_t, off: off_t, prot: c_int) -> c_int;
/// `segmap(9E)`
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
/// `chpoll(9E)`
pub type cb_chpoll_t = unsafe extern "C" fn(
    dev: dev_t,
    events: c_short,
    anyyet: c_int,
    reventsp: *mut c_short,
    phpp: *mut *mut pollhead,
) -> c_int;
/// `prop_op(9E)`
pub type cb_prop_op_t = unsafe extern "C" fn(
    dev: dev_t,
    dip: dev_info_t,
    prop_op: ddi_prop_op_t,
    mod_flags: c_int,
    name: *mut c_char,
    valuep: caddr_t,
    length: *mut c_int,
) -> c_int;
/// `aread(9E)`
pub type cb_aread_t = unsafe extern "C" fn(dev: dev_t, aio: *mut aio_req, credp: *mut cred_t) -> c_int;
/// `awrite(9E)`
pub type cb_awrite_t = unsafe extern "C" fn(dev: dev_t, aio: *mut aio_req, credp: *mut cred_t) -> c_int;

/// `cb_ops(9S)`: character/block entry points.
#[repr(C)]
pub struct cb_ops {
    /// `open(9E)`
    pub cb_open: Option<cb_open_t>,
    /// `close(9E)`
    pub cb_close: Option<cb_close_t>,
    /// `strategy(9E)` (block)
    pub cb_strategy: Option<cb_strategy_t>,
    /// `print(9E)` (block)
    pub cb_print: Option<cb_print_t>,
    /// `dump(9E)` (block)
    pub cb_dump: Option<cb_dump_t>,
    /// `read(9E)`
    pub cb_read: Option<cb_read_t>,
    /// `write(9E)`
    pub cb_write: Option<cb_write_t>,
    /// `ioctl(9E)`
    pub cb_ioctl: Option<cb_ioctl_t>,
    /// `devmap(9E)`
    pub cb_devmap: Option<cb_devmap_t>,
    /// `mmap(9E)`
    pub cb_mmap: Option<cb_mmap_t>,
    /// `segmap(9E)`
    pub cb_segmap: Option<cb_segmap_t>,
    /// `chpoll(9E)`
    pub cb_chpoll: Option<cb_chpoll_t>,
    /// `prop_op(9E)`
    pub cb_prop_op: Option<cb_prop_op_t>,
    /// STREAMS drivers only.
    pub cb_str: *mut streamtab,
    /// `D_MP | D_64BIT | D_NEW | …` (`sys/conf.h`).
    pub cb_flag: c_int,
    /// `CB_REV`
    pub cb_rev: c_int,
    /// `aread(9E)`
    pub cb_aread: Option<cb_aread_t>,
    /// `awrite(9E)`
    pub cb_awrite: Option<cb_awrite_t>,
}

const _: () = assert!(core::mem::size_of::<cb_ops>() == 13 * 8 + 8 + 8 + 2 * 8);
