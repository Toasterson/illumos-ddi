//! Synchronisation primitives from `sys/ksynch.h`, which forwards to
//! `sys/mutex.h`, `sys/condvar.h`, `sys/rwlock.h` and `sys/semaphore.h`.
//!
//! All four types are documented as opaque, but their sizes are part of the
//! ABI because drivers embed them in their own structures. Under LP64 a
//! `kmutex_t` is 8 bytes, a `kcondvar_t` 2 bytes, a `krwlock_t` 8 bytes and a
//! `ksema_t` 16 bytes. The constant assertions below pin those numbers.
//!
//! The module binds the complete `mutex(9F)`, `condvar(9F)`, `rwlock(9F)`
//! and `semaphore(9F)` pages. The headers declare a few more functions, such
//! as `sema_held`, `rw_lock_held`, `rw_read_held`, `rw_write_held`,
//! `rw_owner`, `mutex_owner`, `cv_wait_sig_swap` and `cv_timedwait_hires`,
//! that neither a manual page nor the `ddi` mapfile names. They are left out
//! on purpose. The `MUTEX_HELD`, `RW_LOCK_HELD`, `RW_READ_HELD`,
//! `RW_WRITE_HELD` and `SEMA_HELD` macros are provided by the `ddi` crate.

use crate::types::*;

/// A kernel mutex, `kmutex_t`. Initialise it with [`mutex_init`] before use.
#[repr(C)]
pub struct kmutex_t {
    _opaque: [*mut c_void; 1],
}
/// A kernel condition variable, `kcondvar_t`. Initialise it with [`cv_init`].
#[repr(C)]
pub struct kcondvar_t {
    _opaque: c_ushort,
}
/// A kernel reader-writer lock, `krwlock_t`. Initialise it with [`rw_init`].
#[repr(C)]
pub struct krwlock_t {
    _opaque: [*mut c_void; 1],
}
/// A kernel counting semaphore, `ksema_t`. Initialise it with [`sema_init`].
#[repr(C)]
pub struct ksema_t {
    _opaque: [*mut c_void; 2],
}

const _: () = assert!(core::mem::size_of::<kmutex_t>() == 8);
const _: () = assert!(core::mem::size_of::<kcondvar_t>() == 2);
const _: () = assert!(core::mem::size_of::<krwlock_t>() == 8);
const _: () = assert!(core::mem::size_of::<ksema_t>() == 16);

impl kmutex_t {
    /// Zeroed storage to hand to `mutex_init(9F)`. Not a usable mutex by
    /// itself.
    pub const UNINIT: Self = Self { _opaque: [core::ptr::null_mut()] };
}
impl kcondvar_t {
    /// Zeroed storage to hand to `cv_init(9F)`.
    pub const UNINIT: Self = Self { _opaque: 0 };
}
impl krwlock_t {
    /// Zeroed storage to hand to `rw_init(9F)`.
    pub const UNINIT: Self = Self { _opaque: [core::ptr::null_mut()] };
}
impl ksema_t {
    /// Zeroed storage to hand to `sema_init(9F)`. Not a usable semaphore by
    /// itself.
    pub const UNINIT: Self = Self { _opaque: [core::ptr::null_mut(), core::ptr::null_mut()] };
}

/// The kind of mutex requested from `mutex_init(9F)`, `kmutex_type_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct kmutex_type_t(pub c_int);
impl kmutex_type_t {
    /// Spins while the owner runs and blocks otherwise (`MUTEX_ADAPTIVE`).
    pub const MUTEX_ADAPTIVE: Self = Self(0);
    /// Blocks interrupts and spins (`MUTEX_SPIN`).
    pub const MUTEX_SPIN: Self = Self(1);
    /// The kind every driver passes (`MUTEX_DRIVER`). `mutex_init` turns it
    /// into an adaptive or a spin mutex depending on whether an interrupt
    /// cookie is supplied.
    pub const MUTEX_DRIVER: Self = Self(4);
    /// The kernel's own default, identical to adaptive (`MUTEX_DEFAULT`).
    /// Drivers use `MUTEX_DRIVER` instead.
    pub const MUTEX_DEFAULT: Self = Self(6);
}

/// The kind of condition variable requested from `cv_init(9F)`, `kcv_type_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct kcv_type_t(pub c_int);
impl kcv_type_t {
    /// The kernel's default (`CV_DEFAULT`).
    pub const CV_DEFAULT: Self = Self(0);
    /// The kind drivers pass (`CV_DRIVER`).
    pub const CV_DRIVER: Self = Self(1);
}

/// The kind of reader-writer lock requested from `rw_init(9F)`, `krw_type_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct krw_type_t(pub c_int);
impl krw_type_t {
    /// The kind drivers pass (`RW_DRIVER`).
    pub const RW_DRIVER: Self = Self(2);
    /// The kernel's default (`RW_DEFAULT`).
    pub const RW_DEFAULT: Self = Self(4);
}

/// The mode in which `rw_enter(9F)` takes a reader-writer lock, `krw_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct krw_t(pub c_int);
impl krw_t {
    /// Take the lock exclusively (`RW_WRITER`).
    pub const RW_WRITER: Self = Self(0);
    /// Take the lock shared with other readers (`RW_READER`).
    pub const RW_READER: Self = Self(1);
    /// Take the lock shared, ignoring waiting writers (`RW_READER_STARVEWRITER`).
    pub const RW_READER_STARVEWRITER: Self = Self(2);
}

/// The kind of semaphore requested from `sema_init(9F)`, `ksema_type_t`.
/// The manual page says that only `SEMA_DRIVER` is supported for drivers.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ksema_type_t(pub c_int);
impl ksema_type_t {
    /// The kernel's default (`SEMA_DEFAULT`).
    pub const SEMA_DEFAULT: Self = Self(0);
    /// The kind drivers pass (`SEMA_DRIVER`).
    pub const SEMA_DRIVER: Self = Self(1);
}

/// The time resolution hint for `cv_reltimedwait(9F)`, `time_res_t`. It
/// tells the kernel how much slack it may add to the timeout to coalesce
/// wakeups.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct time_res_t(pub c_int);
impl time_res_t {
    /// Nanosecond precision (`TR_NANOSEC`).
    pub const TR_NANOSEC: Self = Self(0);
    /// Microsecond precision (`TR_MICROSEC`).
    pub const TR_MICROSEC: Self = Self(1);
    /// Millisecond precision (`TR_MILLISEC`).
    pub const TR_MILLISEC: Self = Self(2);
    /// Second precision (`TR_SEC`).
    pub const TR_SEC: Self = Self(3);
    /// Clock tick precision (`TR_CLOCK_TICK`).
    pub const TR_CLOCK_TICK: Self = Self(4);
}

unsafe extern "C" {
    /// `mutex_init(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Initialises a mutex. `name` is unused and may be NULL. For
    /// `MUTEX_DRIVER`, `arg` is the interrupt priority cookie of the
    /// interrupt handler that will take the mutex, or NULL if no interrupt
    /// handler does. It is an error to initialise a mutex more than once.
    /// May be called from user or kernel context only.
    pub fn mutex_init(mp: *mut kmutex_t, name: *const c_char, mtype: kmutex_type_t, arg: *mut c_void);
    /// `mutex_destroy(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the resources of a mutex. The mutex must not be held, and
    /// must be destroyed before the memory containing it is freed. May be
    /// called from user or kernel context only.
    pub fn mutex_destroy(mp: *mut kmutex_t);
    /// `mutex_enter(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Acquires the mutex, blocking or spinning as its kind dictates. If the
    /// calling thread already holds the mutex the system panics, because
    /// mutexes are not recursive.
    pub fn mutex_enter(mp: *mut kmutex_t);
    /// `mutex_exit(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the mutex and unblocks one waiting thread, if any.
    pub fn mutex_exit(mp: *mut kmutex_t);
    /// `mutex_tryenter(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Acquires the mutex if it is free and returns non-zero; otherwise
    /// returns 0 without blocking.
    pub fn mutex_tryenter(mp: *mut kmutex_t) -> c_int;
    /// `mutex_owned(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns non-zero if the current thread holds the mutex, or, when that
    /// cannot be determined, if any thread holds it. Intended for
    /// assertions only.
    pub fn mutex_owned(mp: *mut kmutex_t) -> c_int;

    /// `cv_init(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Initialises a condition variable. `name` and `arg` are unused and may
    /// be NULL. `ctype` is `CV_DRIVER` for drivers.
    pub fn cv_init(cvp: *mut kcondvar_t, name: *const c_char, ctype: kcv_type_t, arg: *mut c_void);
    /// `cv_destroy(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the resources of a condition variable. No thread may be
    /// waiting on it.
    pub fn cv_destroy(cvp: *mut kcondvar_t);
    /// `cv_wait(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the mutex, sleeps until signalled, and reacquires the mutex
    /// before returning. The mutex must be held on entry. Spurious wakeups
    /// are possible, so callers loop on their condition. The manual advises
    /// against calling it from interrupt context and forbids high-level
    /// interrupt context.
    pub fn cv_wait(cvp: *mut kcondvar_t, mp: *mut kmutex_t);
    /// `cv_wait_sig(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`cv_wait`], but a pending signal ends the wait early. Returns 0
    /// if a signal interrupted the wait and non-zero otherwise. The mutex is
    /// reacquired in both cases. When the thread cannot receive signals, as
    /// during a `close(9E)` caused by process exit, it behaves as `cv_wait`.
    pub fn cv_wait_sig(cvp: *mut kcondvar_t, mp: *mut kmutex_t) -> c_int;
    /// `cv_timedwait(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`cv_wait`], but gives up at the absolute `lbolt` value
    /// `timeout`, which is expressed in clock ticks since boot as returned
    /// by `ddi_get_lbolt(9F)`. Returns -1 if the timeout expired and a
    /// positive value otherwise.
    pub fn cv_timedwait(cvp: *mut kcondvar_t, mp: *mut kmutex_t, timeout: clock_t) -> clock_t;
    /// `cv_timedwait_sig(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Combines [`cv_timedwait`] and [`cv_wait_sig`]. The thread sleeps
    /// until it is signalled, until the absolute `lbolt` value `timeout` is
    /// reached, or until a signal is pending for it. Returns -1 if the
    /// timeout expired, 0 if a signal ended the wait, and a positive value
    /// if the condition variable was signalled or the thread woke early for
    /// another reason. The mutex is reacquired before returning in every
    /// case. When the thread cannot receive signals it behaves as
    /// `cv_timedwait`.
    pub fn cv_timedwait_sig(cvp: *mut kcondvar_t, mp: *mut kmutex_t, timeout: clock_t) -> clock_t;
    /// `cv_reltimedwait(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`cv_wait`], but gives up after `delta` clock ticks. The `res`
    /// argument tells the kernel how precise the timeout needs to be;
    /// because the wait is measured in clock ticks, only `TR_SEC` and
    /// `TR_CLOCK_TICK` have an effect. Returns -1 if the timeout expired.
    pub fn cv_reltimedwait(cvp: *mut kcondvar_t, mp: *mut kmutex_t, delta: clock_t, res: time_res_t) -> clock_t;
    /// `cv_reltimedwait_sig(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Combines [`cv_reltimedwait`] and [`cv_wait_sig`]. The thread sleeps
    /// until it is signalled, until `delta` clock ticks have passed, or
    /// until a signal is pending for it. `res` gives the requested
    /// precision of the timeout. Returns -1 if the timeout expired, 0 if a
    /// signal ended the wait, and a positive value otherwise. The mutex is
    /// reacquired before returning in every case. When the thread cannot
    /// receive signals it behaves as `cv_reltimedwait`.
    pub fn cv_reltimedwait_sig(cvp: *mut kcondvar_t, mp: *mut kmutex_t, delta: clock_t, res: time_res_t) -> clock_t;
    /// `cv_signal(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Wakes one thread waiting on the condition variable. The caller need
    /// not hold the associated mutex, although holding it makes scheduling
    /// predictable.
    pub fn cv_signal(cvp: *mut kcondvar_t);
    /// `cv_broadcast(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Wakes every thread waiting on the condition variable.
    pub fn cv_broadcast(cvp: *mut kcondvar_t);

    /// `rw_init(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Initialises a reader-writer lock. `name` is unused and may be NULL.
    /// `rwtype` is `RW_DRIVER` for drivers. If the lock is taken by an
    /// interrupt handler, `arg` is the interrupt priority cookie of that
    /// handler; otherwise it is NULL. May be called from user context only.
    pub fn rw_init(rwlp: *mut krwlock_t, name: *const c_char, rwtype: krw_type_t, arg: *mut c_void);
    /// `rw_destroy(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the resources of a reader-writer lock. It must not be held.
    /// May be called from user context only.
    pub fn rw_destroy(rwlp: *mut krwlock_t);
    /// `rw_enter(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Acquires the lock in the given mode, blocking as necessary. A writer
    /// blocks while any thread holds the lock. A reader blocks while a
    /// writer holds the lock or is waiting for it, unless the mode is
    /// `RW_READER_STARVEWRITER`, which ignores waiting writers. A thread
    /// must not acquire a lock as `RW_READER` that it already holds, since
    /// a waiting writer would then deadlock it.
    pub fn rw_enter(rwlp: *mut krwlock_t, rw: krw_t);
    /// `rw_exit(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the lock, whichever mode it was taken in, and may wake one
    /// or more waiting threads.
    pub fn rw_exit(rwlp: *mut krwlock_t);
    /// `rw_tryenter(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Acquires the lock in the given mode if that is possible without
    /// blocking and returns non-zero; otherwise returns 0.
    pub fn rw_tryenter(rwlp: *mut krwlock_t, rw: krw_t) -> c_int;
    /// `rw_downgrade(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts a held writer lock into a reader lock without releasing it.
    /// One or more waiting readers may be unblocked.
    pub fn rw_downgrade(rwlp: *mut krwlock_t);
    /// `rw_tryupgrade(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Attempts to convert a held reader lock into a writer lock. The
    /// upgrade succeeds only if no other thread holds the lock and no thread
    /// is waiting to write. Returns non-zero on success; on failure the
    /// reader lock is still held.
    pub fn rw_tryupgrade(rwlp: *mut krwlock_t) -> c_int;
    /// `rw_read_locked(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns non-zero if the calling thread holds the lock for reading and
    /// 0 if it holds the lock for writing. The caller must hold the lock;
    /// the system may panic otherwise. Intended for assertions.
    pub fn rw_read_locked(rwlp: *mut krwlock_t) -> c_int;

    /// `sema_init(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Initialises a counting semaphore with the initial value `val`. `name`
    /// is obsolete and should be NULL, `stype` is `SEMA_DRIVER`, and `arg`
    /// should be NULL. The caller provides the storage, but the kernel may
    /// allocate more, so [`sema_destroy`] must be called before that storage
    /// is freed. May be called from user or kernel context only.
    pub fn sema_init(sp: *mut ksema_t, val: uint_t, name: *const c_char, stype: ksema_type_t, arg: *mut c_void);
    /// `sema_destroy(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases any resources that [`sema_init`] allocated. May be called
    /// from user or kernel context only.
    pub fn sema_destroy(sp: *mut ksema_t);
    /// `sema_p(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Atomically decrements the semaphore. If its value is zero the calling
    /// thread blocks until another thread calls [`sema_v`]. The manual
    /// advises against calling it from interrupt context, because lower
    /// priority interrupts are not serviced during the wait, and forbids
    /// high-level interrupt context.
    pub fn sema_p(sp: *mut ksema_t);
    /// `sema_v(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Atomically increments the semaphore and wakes a thread blocked in
    /// [`sema_p`], if any. It must not be called from high-level interrupt
    /// context.
    pub fn sema_v(sp: *mut ksema_t);
    /// `sema_p_sig(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`sema_p`], but if the value is zero and a signal is pending for
    /// the thread it returns 1 without decrementing the semaphore. Returns 0
    /// when the semaphore was decremented.
    pub fn sema_p_sig(sp: *mut ksema_t) -> c_int;
    /// `sema_tryp(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Decrements the semaphore only if its value is greater than zero and
    /// never blocks. Returns non-zero if the semaphore was decremented and 0
    /// if its value was zero.
    pub fn sema_tryp(sp: *mut ksema_t) -> c_int;
}
