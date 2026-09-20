//! `ddi_taskq_create(9F)` family from `sys/sunddi.h`.

use crate::types::*;

/// `DDI_SLEEP`: dispatch may block.
pub const DDI_SLEEP: uint_t = 0;
/// `DDI_NOSLEEP`: dispatch may not block; may fail.
pub const DDI_NOSLEEP: uint_t = 1;
/// `TASKQ_DEFAULTPRI`
pub const TASKQ_DEFAULTPRI: pri_t = -1;

/// Task function for `ddi_taskq_dispatch(9F)`.
pub type ddi_taskq_fn_t = unsafe extern "C" fn(arg: *mut c_void);

unsafe extern "C" {
    /// `ddi_taskq_create(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_taskq_create(dip: dev_info_t, name: *const c_char, nthreads: c_int, pri: pri_t, cflags: uint_t) -> *mut ddi_taskq_t;
    /// `ddi_taskq_destroy(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_taskq_destroy(tq: *mut ddi_taskq_t);
    /// `ddi_taskq_dispatch(9F)` — illumos DDI specific (illumos DDI). Returns `DDI_SUCCESS`/`DDI_FAILURE`.
    pub fn ddi_taskq_dispatch(tq: *mut ddi_taskq_t, func: ddi_taskq_fn_t, arg: *mut c_void, dflags: uint_t) -> c_int;
    /// `ddi_taskq_wait(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_taskq_wait(tq: *mut ddi_taskq_t);
    /// `ddi_taskq_suspend(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_taskq_suspend(tq: *mut ddi_taskq_t);
    /// `ddi_taskq_suspended(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_taskq_suspended(tq: *mut ddi_taskq_t) -> boolean_t;
    /// `ddi_taskq_resume(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_taskq_resume(tq: *mut ddi_taskq_t);
}
