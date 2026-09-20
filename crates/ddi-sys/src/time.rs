//! Time-related `9F` functions from `sys/sunddi.h`, `sys/ddi_periodic.h`,
//! `sys/time.h`.

use crate::types::*;

/// `ddi_periodic_add(9F)` handler.
pub type ddi_periodic_fn_t = unsafe extern "C" fn(arg: *mut c_void);

/// `DDI_IPL_0` … `DDI_IPL_10`: interrupt level for `ddi_periodic_add(9F)`.
pub const DDI_IPL_0: c_int = 0;

unsafe extern "C" {
    /// `ddi_get_lbolt(9F)`. illumos DDI specific (illumos DDI).
    pub fn ddi_get_lbolt() -> clock_t;
    /// `ddi_get_lbolt64(9F)`. illumos DDI specific (illumos DDI).
    pub fn ddi_get_lbolt64() -> i64;
    /// `ddi_get_time(9F)`. illumos DDI specific (illumos DDI).
    pub fn ddi_get_time() -> c_long;
    /// `drv_usectohz(9F)`. Architecture independent level 1 (DDI/DKI).
    pub fn drv_usectohz(microsecs: clock_t) -> clock_t;
    /// `drv_hztousec(9F)`. Architecture independent level 1 (DDI/DKI).
    pub fn drv_hztousec(hz: clock_t) -> clock_t;
    /// `drv_usecwait(9F)`. Architecture independent level 1 (DDI/DKI). Busy-waits.
    pub fn drv_usecwait(microsecs: clock_t);
    /// `delay(9F)`. Architecture independent level 1 (DDI/DKI). Sleeps; not from interrupt context.
    pub fn delay(ticks: clock_t);
    /// `gethrtime(9F)`. illumos DDI specific (illumos DDI).
    pub fn gethrtime() -> hrtime_t;
    /// `ddi_periodic_add(9F)`. illumos DDI specific (illumos DDI).
    pub fn ddi_periodic_add(
        func: ddi_periodic_fn_t,
        arg: *mut c_void,
        interval: hrtime_t,
        level: c_int,
    ) -> ddi_periodic_t;
    /// `ddi_periodic_delete(9F)`. illumos DDI specific (illumos DDI).
    pub fn ddi_periodic_delete(request: ddi_periodic_t);
}
