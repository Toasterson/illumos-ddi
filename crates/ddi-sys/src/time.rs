//! Time and timing functions from `sys/sunddi.h`, `sys/ddi_periodic.h`,
//! `sys/systm.h` and `sys/time.h`.
//!
//! The kernel keeps time in two units. `lbolt` counts clock ticks since
//! boot, and `hrtime_t` counts nanoseconds. The `drv_*` conversion functions
//! move between microseconds and ticks so that a driver does not have to
//! know the tick rate. `timeout(9F)` schedules a one-shot callback in ticks,
//! and `ddi_periodic_add(9F)` schedules a repeating one in nanoseconds.
//!
//! `gethrestime` and `gethrestime_sec` are declared in `sys/time.h` but have
//! no section 9 manual page and are not in the `ddi` mapfile, so they are
//! not bound; `ddi_get_time(9F)` covers the same need for drivers.

use crate::types::*;

/// A time in seconds and nanoseconds, `timestruc_t`. `sys/time_impl.h`
/// defines it as the SVR4 name for `struct timespec`. The DDI uses it for
/// absolute and relative timeouts, for example in `ldi_getmsg(9F)`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct timestruc_t {
    /// Whole seconds, a `time_t`, which is a `long` under LP64.
    pub tv_sec: c_long,
    /// Nanoseconds within the second, from 0 to 999999999.
    pub tv_nsec: c_long,
}

// Two longs under LP64.
const _: () = assert!(core::mem::size_of::<timestruc_t>() == 16);
const _: () = assert!(core::mem::offset_of!(timestruc_t, tv_nsec) == 8);

/// The callback type for `ddi_periodic_add(9F)`.
pub type ddi_periodic_fn_t = unsafe extern "C" fn(arg: *mut c_void);

/// The callback type for `timeout(9F)`. The function receives the `arg`
/// that was passed to `timeout` and must obey the same restrictions as a
/// soft interrupt handler.
pub type timeout_fn_t = unsafe extern "C" fn(arg: *mut c_void);

/// The opaque identifier returned by `timeout(9F)` and passed to
/// `untimeout(9F)`, `timeout_id_t`. It is a `void *` in `sys/types.h`. A
/// pending timeout never has the value 0, so drivers conventionally store 0
/// to mean that no timeout is outstanding.
pub type timeout_id_t = *mut c_void;

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
    /// `gethrtime(9F)`. No interface level given.
    ///
    /// Returns the high-resolution time in nanoseconds from an arbitrary
    /// origin. It is monotonic and suitable for measuring intervals.
    pub fn gethrtime() -> hrtime_t;
    /// `timeout(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Schedules `func` to be called with `arg` after `ticks` clock ticks
    /// have elapsed. The interval is a close approximation rather than a
    /// guarantee. The callback must adhere to the same restrictions as a
    /// soft interrupt handler. Returns an opaque non-zero identifier that
    /// can be passed to [`untimeout`] to cancel the request. May be called
    /// from user, interrupt or kernel context. Because `timeout` is subject
    /// to priority inversion, drivers waiting on behalf of processes with
    /// real-time constraints should use `cv_timedwait(9F)` instead of
    /// `delay(9F)`, which is built on it.
    pub fn timeout(func: timeout_fn_t, arg: *mut c_void, ticks: clock_t) -> timeout_id_t;
    /// `untimeout(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Cancels a pending `timeout` request. It does not return until the
    /// callback has been cancelled or has finished running, so no lock that
    /// the callback acquires may be held across the call. It is harmless to
    /// call it for a timeout that is running on another CPU or that has
    /// already completed. Returns -1 if the identifier is not found and a
    /// value greater than or equal to 0 otherwise. May be called from user,
    /// interrupt or kernel context.
    pub fn untimeout(id: timeout_id_t) -> clock_t;
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
