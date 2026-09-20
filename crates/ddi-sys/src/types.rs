//! `sys/types.h`, `sys/dditypes.h`: base and opaque types.

use core::ffi;

pub use ffi::{c_char, c_int, c_long, c_longlong, c_short, c_uchar, c_uint, c_ulong, c_ulonglong, c_ushort, c_void};

/// `uint_t`
pub type uint_t = c_uint;
/// `ulong_t`
pub type ulong_t = c_ulong;
/// `longlong_t`
pub type longlong_t = c_longlong;
/// `size_t`
pub type size_t = usize;
/// `ssize_t`
pub type ssize_t = isize;
/// `intptr_t`
pub type intptr_t = isize;
/// `uintptr_t`
pub type uintptr_t = usize;
/// `caddr_t`: core address.
pub type caddr_t = *mut c_char;
/// `daddr_t`: disk address (`long`).
pub type daddr_t = c_long;
/// `offset_t`: 64-bit file offset.
pub type offset_t = longlong_t;
/// `off_t` under LP64.
pub type off_t = c_long;
/// `dev_t`: expanded device number (`ulong_t`).
pub type dev_t = ulong_t;
/// `major_t`
pub type major_t = uint_t;
/// `minor_t`
pub type minor_t = uint_t;
/// `pri_t`: scheduling priority.
pub type pri_t = c_short;
/// `clock_t`: relative time in ticks (`long`).
pub type clock_t = c_long;
/// `hrtime_t`: high-resolution time in nanoseconds.
pub type hrtime_t = longlong_t;
/// `boolean_t` (`B_FALSE`/`B_TRUE`).
pub type boolean_t = c_int;
/// `B_FALSE`
pub const B_FALSE: boolean_t = 0;
/// `B_TRUE`
pub const B_TRUE: boolean_t = 1;

opaque! {
    /// `struct __dev_info`; `dev_info_t` is a pointer to it. See `Intro(9E)`.
    dev_info;
    /// `cred_t`: user credentials, opaque to drivers. See `cred.h`, `drv_priv(9F)`.
    cred_t;
    /// `struct modinfo`, passed to `_info(9E)`.
    modinfo;
    /// `struct mod_ops` vtable (`mod_driverops`, `mod_miscops`, …).
    mod_ops;
    /// `struct bus_ops`: nexus operations, not bound in this crate.
    bus_ops;
    /// `struct streamtab`: bound by `ddi-streams-sys`; opaque here.
    streamtab;
    /// `struct buf`: `buf(9S)`. Opaque until the block-device family lands.
    buf;
    /// `struct pollhead`: `chpoll(9E)`.
    pollhead;
    /// `struct as`: address space, `segmap(9E)`.
    addrspace;
    /// `struct aio_req`: `aread(9E)`/`awrite(9E)`.
    aio_req;
    /// `struct __ddi_acc_handle`.
    __ddi_acc_handle;
    /// `struct __ddi_dma_handle`.
    __ddi_dma_handle;
    /// `struct __ddi_periodic`.
    __ddi_periodic;
    /// `struct ddi_taskq`.
    ddi_taskq_t;
    /// `struct __ddi_softinfo` behind `ddi_softint_handle_t`.
    __ddi_softint_handle;
    /// `struct __ddi_intr_handle`.
    __ddi_intr_handle;
}

/// `dev_info_t`
pub type dev_info_t = *mut dev_info;
/// `ddi_acc_handle_t`
pub type ddi_acc_handle_t = *mut __ddi_acc_handle;
/// `ddi_dma_handle_t`
pub type ddi_dma_handle_t = *mut __ddi_dma_handle;
/// `ddi_periodic_t`
pub type ddi_periodic_t = *mut __ddi_periodic;
/// `ddi_softint_handle_t`
pub type ddi_softint_handle_t = *mut __ddi_softint_handle;
/// `ddi_intr_handle_t`
pub type ddi_intr_handle_t = *mut __ddi_intr_handle;
/// `devmap_cookie_t` (`ddidevmap.h`: `typedef void *devmap_cookie_t`).
pub type devmap_cookie_t = *mut c_void;

/// `ddi_device_acc_attr(9S)`: device register access attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_device_acc_attr_t {
    /// `DDI_DEVICE_ATTR_V0` or `DDI_DEVICE_ATTR_V1`.
    pub devacc_attr_version: c_ushort,
    /// `DDI_NEVERSWAP_ACC`, `DDI_STRUCTURE_LE_ACC`, `DDI_STRUCTURE_BE_ACC`.
    pub devacc_attr_endian_flags: c_uchar,
    /// `DDI_STRICTORDER_ACC` … `DDI_STORECACHING_OK_ACC`.
    pub devacc_attr_dataorder: c_uchar,
    /// `DDI_DEFAULT_ACC`, `DDI_FLAGERR_ACC`, `DDI_CAUTIOUS_ACC` (V1 only).
    pub devacc_attr_access: c_uchar,
}

/// `DDI_DEVICE_ATTR_V0`
pub const DDI_DEVICE_ATTR_V0: c_ushort = 0x0001;
/// `DDI_DEVICE_ATTR_V1`
pub const DDI_DEVICE_ATTR_V1: c_ushort = 0x0002;
/// `DDI_NEVERSWAP_ACC`
pub const DDI_NEVERSWAP_ACC: c_uchar = 0x00;
/// `DDI_STRUCTURE_LE_ACC`
pub const DDI_STRUCTURE_LE_ACC: c_uchar = 0x01;
/// `DDI_STRUCTURE_BE_ACC`
pub const DDI_STRUCTURE_BE_ACC: c_uchar = 0x02;
/// `DDI_STRICTORDER_ACC`
pub const DDI_STRICTORDER_ACC: c_uchar = 0x00;
/// `DDI_UNORDERED_OK_ACC`
pub const DDI_UNORDERED_OK_ACC: c_uchar = 0x01;
/// `DDI_MERGING_OK_ACC`
pub const DDI_MERGING_OK_ACC: c_uchar = 0x02;
/// `DDI_LOADCACHING_OK_ACC`
pub const DDI_LOADCACHING_OK_ACC: c_uchar = 0x03;
/// `DDI_STORECACHING_OK_ACC`
pub const DDI_STORECACHING_OK_ACC: c_uchar = 0x04;
/// `DDI_DEFAULT_ACC`
pub const DDI_DEFAULT_ACC: c_uchar = 0x01;
/// `DDI_FLAGERR_ACC`
pub const DDI_FLAGERR_ACC: c_uchar = 0x02;
/// `DDI_CAUTIOUS_ACC`
pub const DDI_CAUTIOUS_ACC: c_uchar = 0x03;

const _: () = assert!(core::mem::size_of::<ddi_device_acc_attr_t>() == 6);
