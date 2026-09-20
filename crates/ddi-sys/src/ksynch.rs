//! `sys/ksynch.h` → `sys/mutex.h`, `sys/condvar.h`, `sys/rwlock.h`,
//! `sys/semaphore.h`: synchronisation primitives.
//!
//! All four types are documented as opaque; their sizes are part of the ABI
//! (`kmutex_t` 8, `kcondvar_t` 2, `krwlock_t` 8, `ksema_t` 16 on LP64).

use crate::types::*;

/// `kmutex_t`
#[repr(C)]
pub struct kmutex_t {
    _opaque: [*mut c_void; 1],
}
/// `kcondvar_t`
#[repr(C)]
pub struct kcondvar_t {
    _opaque: c_ushort,
}
/// `krwlock_t`
#[repr(C)]
pub struct krwlock_t {
    _opaque: [*mut c_void; 1],
}
/// `ksema_t`
#[repr(C)]
pub struct ksema_t {
    _opaque: [*mut c_void; 2],
}

const _: () = assert!(core::mem::size_of::<kmutex_t>() == 8);
const _: () = assert!(core::mem::size_of::<kcondvar_t>() == 2);
const _: () = assert!(core::mem::size_of::<krwlock_t>() == 8);
const _: () = assert!(core::mem::size_of::<ksema_t>() == 16);

impl kmutex_t {
    /// All-zero storage for `mutex_init(9F)` to initialise.
    pub const UNINIT: Self = Self { _opaque: [core::ptr::null_mut()] };
}
impl kcondvar_t {
    /// All-zero storage for `cv_init(9F)` to initialise.
    pub const UNINIT: Self = Self { _opaque: 0 };
}
impl krwlock_t {
    /// All-zero storage for `rw_init(9F)` to initialise.
    pub const UNINIT: Self = Self { _opaque: [core::ptr::null_mut()] };
}

/// `kmutex_type_t`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct kmutex_type_t(pub c_int);
impl kmutex_type_t {
    /// `MUTEX_ADAPTIVE`
    pub const MUTEX_ADAPTIVE: Self = Self(0);
    /// `MUTEX_SPIN`
    pub const MUTEX_SPIN: Self = Self(1);
    /// `MUTEX_DRIVER`: what drivers pass; converted using the interrupt cookie.
    pub const MUTEX_DRIVER: Self = Self(4);
    /// `MUTEX_DEFAULT`
    pub const MUTEX_DEFAULT: Self = Self(6);
}

/// `kcv_type_t`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct kcv_type_t(pub c_int);
impl kcv_type_t {
    /// `CV_DEFAULT`
    pub const CV_DEFAULT: Self = Self(0);
    /// `CV_DRIVER`
    pub const CV_DRIVER: Self = Self(1);
}

/// `krw_type_t`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct krw_type_t(pub c_int);
impl krw_type_t {
    /// `RW_DRIVER`
    pub const RW_DRIVER: Self = Self(2);
    /// `RW_DEFAULT`
    pub const RW_DEFAULT: Self = Self(4);
}

/// `krw_t`: lock mode for `rw_enter(9F)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct krw_t(pub c_int);
impl krw_t {
    /// `RW_WRITER`
    pub const RW_WRITER: Self = Self(0);
    /// `RW_READER`
    pub const RW_READER: Self = Self(1);
    /// `RW_READER_STARVEWRITER`
    pub const RW_READER_STARVEWRITER: Self = Self(2);
}

/// `time_res_t` for `cv_reltimedwait(9F)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct time_res_t(pub c_int);
impl time_res_t {
    /// `TR_NANOSEC`
    pub const TR_NANOSEC: Self = Self(0);
    /// `TR_MICROSEC`
    pub const TR_MICROSEC: Self = Self(1);
    /// `TR_MILLISEC`
    pub const TR_MILLISEC: Self = Self(2);
    /// `TR_SEC`
    pub const TR_SEC: Self = Self(3);
    /// `TR_CLOCK_TICK`
    pub const TR_CLOCK_TICK: Self = Self(4);
}

unsafe extern "C" {
    /// `mutex_init(9F)` — Architecture independent level 1 (DDI/DKI).
    /// `arg` is the interrupt priority cookie for `MUTEX_DRIVER`, else NULL.
    pub fn mutex_init(mp: *mut kmutex_t, name: *const c_char, mtype: kmutex_type_t, arg: *mut c_void);
    /// `mutex_destroy(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn mutex_destroy(mp: *mut kmutex_t);
    /// `mutex_enter(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn mutex_enter(mp: *mut kmutex_t);
    /// `mutex_exit(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn mutex_exit(mp: *mut kmutex_t);
    /// `mutex_tryenter(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn mutex_tryenter(mp: *mut kmutex_t) -> c_int;
    /// `mutex_owned(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn mutex_owned(mp: *mut kmutex_t) -> c_int;

    /// `cv_init(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn cv_init(cvp: *mut kcondvar_t, name: *const c_char, ctype: kcv_type_t, arg: *mut c_void);
    /// `cv_destroy(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn cv_destroy(cvp: *mut kcondvar_t);
    /// `cv_wait(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn cv_wait(cvp: *mut kcondvar_t, mp: *mut kmutex_t);
    /// `cv_wait_sig(9F)` — Architecture independent level 1 (DDI/DKI).
    /// Returns 0 if interrupted by a signal.
    pub fn cv_wait_sig(cvp: *mut kcondvar_t, mp: *mut kmutex_t) -> c_int;
    /// `cv_timedwait(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn cv_timedwait(cvp: *mut kcondvar_t, mp: *mut kmutex_t, timeout: clock_t) -> clock_t;
    /// `cv_reltimedwait(9F)` — illumos DDI specific (illumos DDI).
    pub fn cv_reltimedwait(cvp: *mut kcondvar_t, mp: *mut kmutex_t, delta: clock_t, res: time_res_t) -> clock_t;
    /// `cv_signal(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn cv_signal(cvp: *mut kcondvar_t);
    /// `cv_broadcast(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn cv_broadcast(cvp: *mut kcondvar_t);

    /// `rw_init(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn rw_init(rwlp: *mut krwlock_t, name: *const c_char, rwtype: krw_type_t, arg: *mut c_void);
    /// `rw_destroy(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn rw_destroy(rwlp: *mut krwlock_t);
    /// `rw_enter(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn rw_enter(rwlp: *mut krwlock_t, rw: krw_t);
    /// `rw_exit(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn rw_exit(rwlp: *mut krwlock_t);
    /// `rw_tryenter(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn rw_tryenter(rwlp: *mut krwlock_t, rw: krw_t) -> c_int;
    /// `rw_downgrade(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn rw_downgrade(rwlp: *mut krwlock_t);
    /// `rw_tryupgrade(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn rw_tryupgrade(rwlp: *mut krwlock_t) -> c_int;
    /// `rw_read_locked(9F)` — Architecture independent level 1 (DDI/DKI).
    pub fn rw_read_locked(rwlp: *mut krwlock_t) -> c_int;
}
