//! Task queues from `sys/sunddi.h`, the `ddi_taskq_*(9F)` family.
//!
//! A task queue is a pool of kernel threads that run functions on behalf of
//! a driver. Its main use is to move work out of interrupt context, where
//! blocking is not allowed, into thread context, where it is.

use crate::types::*;

/// The dispatch may block until a task entry is available (`DDI_SLEEP`).
pub const DDI_SLEEP: uint_t = 0;
/// The dispatch must not block and may fail (`DDI_NOSLEEP`).
pub const DDI_NOSLEEP: uint_t = 1;
/// The default scheduling priority for task queue threads (`TASKQ_DEFAULTPRI`).
pub const TASKQ_DEFAULTPRI: pri_t = -1;

/// The function type that `ddi_taskq_dispatch(9F)` runs.
pub type ddi_taskq_fn_t = unsafe extern "C" fn(arg: *mut c_void);

unsafe extern "C" {
    /// `ddi_taskq_create(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates a task queue with `nthreads` threads at priority `pri` on
    /// behalf of the device `dip`. `cflags` is reserved and must be 0.
    /// Returns NULL on failure.
    pub fn ddi_taskq_create(
        dip: dev_info_t,
        name: *const c_char,
        nthreads: c_int,
        pri: pri_t,
        cflags: uint_t,
    ) -> *mut ddi_taskq_t;
    /// `ddi_taskq_destroy(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Waits for all dispatched tasks to finish and destroys the queue.
    pub fn ddi_taskq_destroy(tq: *mut ddi_taskq_t);
    /// `ddi_taskq_dispatch(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Queues `func` to be called with `arg` on one of the queue's threads.
    /// Returns `DDI_SUCCESS` or, with `DDI_NOSLEEP`, `DDI_FAILURE` when no
    /// task entry could be allocated.
    pub fn ddi_taskq_dispatch(tq: *mut ddi_taskq_t, func: ddi_taskq_fn_t, arg: *mut c_void, dflags: uint_t) -> c_int;
    /// `ddi_taskq_wait(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Blocks until every task dispatched so far has completed.
    pub fn ddi_taskq_wait(tq: *mut ddi_taskq_t);
    /// `ddi_taskq_suspend(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stops the queue from starting new tasks. Running tasks complete.
    pub fn ddi_taskq_suspend(tq: *mut ddi_taskq_t);
    /// `ddi_taskq_suspended(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether the queue is currently suspended.
    pub fn ddi_taskq_suspended(tq: *mut ddi_taskq_t) -> boolean_t;
    /// `ddi_taskq_resume(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Lets a suspended queue run tasks again.
    pub fn ddi_taskq_resume(tq: *mut ddi_taskq_t);
}
