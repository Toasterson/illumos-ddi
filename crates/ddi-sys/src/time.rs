//! Time and timing functions from `sys/sunddi.h`, `sys/ddi_periodic.h` and
//! `sys/time.h`.
//!
//! The kernel keeps time in two units. `lbolt` counts clock ticks since
//! boot, and `hrtime_t` counts nanoseconds. The `drv_*` conversion functions
//! move between microseconds and ticks so that a driver does not have to
//! know the tick rate.

use crate::types::*;

/// The callback type for `ddi_periodic_add(9F)`.
pub type ddi_periodic_fn_t = unsafe extern "C" fn(arg: *mut c_void);

/// The lowest interrupt priority level, used with `ddi_periodic_add(9F)` for
/// callbacks that run in kernel thread context (`DDI_IPL_0`).
pub const DDI_IPL_0: c_int = 0;

unsafe extern "C" {
    /// `ddi_get_lbolt(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the number of clock ticks since boot as a `clock_t`.
    pub fn ddi_get_lbolt() -> clock_t;
    /// `ddi_get_lbolt64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the number of clock ticks since boot as a 64-bit value that
    /// does not wrap.
    pub fn ddi_get_lbolt64() -> i64;
    /// `ddi_get_time(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the current time in seconds since the Unix epoch.
    pub fn ddi_get_time() -> c_long;
    /// `drv_usectohz(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Converts microseconds to clock ticks, rounding up so that the result
    /// is never zero for a positive input.
    pub fn drv_usectohz(microsecs: clock_t) -> clock_t;
    /// `drv_hztousec(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Converts clock ticks to microseconds.
    pub fn drv_hztousec(hz: clock_t) -> clock_t;
    /// `drv_usecwait(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Spins for the given number of microseconds. It never sleeps, so it is
    /// safe in interrupt context, and it should be used only for very short
    /// waits.
    pub fn drv_usecwait(microsecs: clock_t);
    /// `delay(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Sleeps for the given number of clock ticks. It must not be called
    /// from interrupt context.
    pub fn delay(ticks: clock_t);
    /// `gethrtime(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the high-resolution time in nanoseconds from an arbitrary
    /// origin. It is monotonic and suitable for measuring intervals.
    pub fn gethrtime() -> hrtime_t;
    /// `ddi_periodic_add(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Arranges for `func` to be called with `arg` every `interval`
    /// nanoseconds at interrupt level `level`. The returned handle is passed
    /// to [`ddi_periodic_delete`] to stop the callbacks.
    pub fn ddi_periodic_add(
        func: ddi_periodic_fn_t,
        arg: *mut c_void,
        interval: hrtime_t,
        level: c_int,
    ) -> ddi_periodic_t;
    /// `ddi_periodic_delete(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cancels a periodic callback. It waits for any running invocation to
    /// finish before returning, so it must not be called from the callback
    /// itself.
    pub fn ddi_periodic_delete(request: ddi_periodic_t);
}
