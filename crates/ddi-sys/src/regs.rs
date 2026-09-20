//! Programmed I/O to device registers, behind the `regs` feature.
//!
//! A driver maps a register set with `ddi_regs_map_setup(9F)` and then reads
//! and writes it through the returned handle with the `ddi_get*` and
//! `ddi_put*` functions. Going through the handle rather than dereferencing
//! the address directly is what lets the framework apply the byte order and
//! ordering rules from the access attributes, and what makes fault
//! management possible.

use crate::types::*;

unsafe extern "C" {
    /// `ddi_dev_nregs(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports how many register sets the device has.
    pub fn ddi_dev_nregs(dip: dev_info_t, resultp: *mut c_int) -> c_int;
    /// `ddi_dev_regsize(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports the size in bytes of register set `rnumber`.
    pub fn ddi_dev_regsize(dip: dev_info_t, rnumber: uint_t, resultp: *mut off_t) -> c_int;
    /// `ddi_regs_map_setup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Maps `len` bytes of register set `rnumber` starting at `offset`,
    /// using the access attributes in `accattrp`. A length of 0 maps the
    /// whole set. On success the mapped address is stored in `addrp` and the
    /// access handle in `handlep`.
    pub fn ddi_regs_map_setup(
        dip: dev_info_t,
        rnumber: uint_t,
        addrp: *mut caddr_t,
        offset: offset_t,
        len: offset_t,
        accattrp: *const ddi_device_acc_attr_t,
        handlep: *mut ddi_acc_handle_t,
    ) -> c_int;
    /// `ddi_regs_map_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unmaps a register set and invalidates the handle.
    pub fn ddi_regs_map_free(handlep: *mut ddi_acc_handle_t);
    /// `ddi_get8(9F)`. illumos DDI specific (illumos DDI), Committed.
    ///
    /// Reads one byte from the mapped address `dev_addr`. The `ddi_get16`,
    /// `ddi_get32` and `ddi_get64` variants read wider values and apply the
    /// byte order from the access attributes.
    pub fn ddi_get8(handle: ddi_acc_handle_t, dev_addr: *const u8) -> u8;
    /// `ddi_get16(9F)`. Reads a 16-bit value; see [`ddi_get8`].
    pub fn ddi_get16(handle: ddi_acc_handle_t, dev_addr: *const u16) -> u16;
    /// `ddi_get32(9F)`. Reads a 32-bit value; see [`ddi_get8`].
    pub fn ddi_get32(handle: ddi_acc_handle_t, dev_addr: *const u32) -> u32;
    /// `ddi_get64(9F)`. Reads a 64-bit value; see [`ddi_get8`].
    pub fn ddi_get64(handle: ddi_acc_handle_t, dev_addr: *const u64) -> u64;
    /// `ddi_put8(9F)`. Writes one byte to the mapped address `dev_addr`. The
    /// wider variants apply the byte order from the access attributes.
    pub fn ddi_put8(handle: ddi_acc_handle_t, dev_addr: *mut u8, value: u8);
    /// `ddi_put16(9F)`. Writes a 16-bit value; see [`ddi_put8`].
    pub fn ddi_put16(handle: ddi_acc_handle_t, dev_addr: *mut u16, value: u16);
    /// `ddi_put32(9F)`. Writes a 32-bit value; see [`ddi_put8`].
    pub fn ddi_put32(handle: ddi_acc_handle_t, dev_addr: *mut u32, value: u32);
    /// `ddi_put64(9F)`. Writes a 64-bit value; see [`ddi_put8`].
    pub fn ddi_put64(handle: ddi_acc_handle_t, dev_addr: *mut u64, value: u64);
    /// `ddi_check_acc_handle(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether any access through the handle has faulted. Only
    /// meaningful when the handle was set up with `DDI_FLAGERR_ACC`.
    pub fn ddi_check_acc_handle(handle: ddi_acc_handle_t) -> c_int;
}
