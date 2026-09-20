//! Programmed I/O: `ddi_regs_map_setup(9F)`, `ddi_get8(9F)` … `ddi_put64(9F)`,
//! `ddi_dev_regsize(9F)`, `ddi_dev_nregs(9F)`. Behind feature `regs`.

use crate::types::*;

unsafe extern "C" {
    /// `ddi_dev_nregs(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_dev_nregs(dip: dev_info_t, resultp: *mut c_int) -> c_int;
    /// `ddi_dev_regsize(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_dev_regsize(dip: dev_info_t, rnumber: uint_t, resultp: *mut off_t) -> c_int;
    /// `ddi_regs_map_setup(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_regs_map_setup(
        dip: dev_info_t, rnumber: uint_t, addrp: *mut caddr_t, offset: offset_t, len: offset_t,
        accattrp: *const ddi_device_acc_attr_t, handlep: *mut ddi_acc_handle_t,
    ) -> c_int;
    /// `ddi_regs_map_free(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_regs_map_free(handlep: *mut ddi_acc_handle_t);
    /// `ddi_get8(9F)` — Interface Stability: Committed.
    pub fn ddi_get8(handle: ddi_acc_handle_t, dev_addr: *const u8) -> u8;
    /// `ddi_get16(9F)`
    pub fn ddi_get16(handle: ddi_acc_handle_t, dev_addr: *const u16) -> u16;
    /// `ddi_get32(9F)`
    pub fn ddi_get32(handle: ddi_acc_handle_t, dev_addr: *const u32) -> u32;
    /// `ddi_get64(9F)`
    pub fn ddi_get64(handle: ddi_acc_handle_t, dev_addr: *const u64) -> u64;
    /// `ddi_put8(9F)`
    pub fn ddi_put8(handle: ddi_acc_handle_t, dev_addr: *mut u8, value: u8);
    /// `ddi_put16(9F)`
    pub fn ddi_put16(handle: ddi_acc_handle_t, dev_addr: *mut u16, value: u16);
    /// `ddi_put32(9F)`
    pub fn ddi_put32(handle: ddi_acc_handle_t, dev_addr: *mut u32, value: u32);
    /// `ddi_put64(9F)`
    pub fn ddi_put64(handle: ddi_acc_handle_t, dev_addr: *mut u64, value: u64);
    /// `ddi_check_acc_handle(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_check_acc_handle(handle: ddi_acc_handle_t) -> c_int;
}
