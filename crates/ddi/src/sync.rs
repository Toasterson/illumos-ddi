//! Mutexes and condition variables as owning types.
//!
//! The kernel's `kmutex_t` and `kcondvar_t` have to be initialised before
//! use and destroyed afterwards, and the mutex has to be held whenever the
//! condition variable is waited on. The types here do the first two in
//! their constructors and destructors and encode the third in the signature
//! of [`CondVar::wait`].

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::context::Sleepable;
use crate::sys;

/// A `kmutex_t` of kind `MUTEX_DRIVER` that guards a `T`.
///
/// The constructor calls `mutex_init(9F)` and `Drop` calls
/// `mutex_destroy(9F)`, so a driver cannot forget either. An unlocked
/// adaptive mutex with no waiters is a single zero word, which is why a
/// `Mutex` may be moved between its construction and its final home,
/// normally a field of the driver's instance. It must not be moved while
/// locked, and the borrow held by a [`MutexGuard`] makes that impossible.
pub struct Mutex<T> {
    raw: UnsafeCell<sys::kmutex_t>,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a `MUTEX_DRIVER` mutex without an interrupt cookie. Such a
    /// mutex must never be taken from a hardware interrupt handler; the
    /// variant that can be will arrive with the `intr` feature.
    pub fn new(value: T) -> Mutex<T> {
        let m = Mutex { raw: UnsafeCell::new(sys::kmutex_t::UNINIT), data: UnsafeCell::new(value) };
        unsafe { sys::mutex_init(m.raw.get(), ptr::null(), sys::kmutex_type_t::MUTEX_DRIVER, ptr::null_mut()) };
        m
    }

    /// Acquires the mutex with `mutex_enter(9F)` and returns a guard that
    /// releases it when dropped.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        unsafe { sys::mutex_enter(self.raw.get()) };
        MutexGuard { m: self }
    }

    /// Tries to acquire the mutex with `mutex_tryenter(9F)` without blocking.
    /// Returns `None` if another thread holds it.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if unsafe { sys::mutex_tryenter(self.raw.get()) } != 0 { Some(MutexGuard { m: self }) } else { None }
    }

    /// Reports whether the current thread holds the mutex, using
    /// `mutex_owned(9F)`. Intended for assertions.
    pub fn is_held_by_current_thread(&self) -> bool {
        unsafe { sys::mutex_owned(self.raw.get()) != 0 }
    }

    /// Gives access to the data without taking the lock. This requires a
    /// mutable reference to the mutex, which proves that no guard exists.
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    fn raw(&self) -> *mut sys::kmutex_t {
        self.raw.get()
    }
}

impl<T> Drop for Mutex<T> {
    fn drop(&mut self) {
        unsafe { sys::mutex_destroy(self.raw.get()) }
    }
}

/// Proof that a [`Mutex`] is held. Dereferences to the guarded data and
/// calls `mutex_exit(9F)` when dropped.
pub struct MutexGuard<'a, T> {
    m: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.m.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.m.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { sys::mutex_exit(self.m.raw()) }
    }
}

/// A `kcondvar_t` of kind `CV_DRIVER`.
///
/// The constructor calls `cv_init(9F)` and `Drop` calls `cv_destroy(9F)`.
pub struct CondVar {
    raw: UnsafeCell<sys::kcondvar_t>,
}

unsafe impl Send for CondVar {}
unsafe impl Sync for CondVar {}

impl CondVar {
    /// Creates a condition variable with `cv_init(9F)`.
    pub fn new() -> CondVar {
        let cv = CondVar { raw: UnsafeCell::new(sys::kcondvar_t::UNINIT) };
        unsafe { sys::cv_init(cv.raw.get(), ptr::null(), sys::kcv_type_t::CV_DRIVER, ptr::null_mut()) };
        cv
    }

    /// Waits with `cv_wait(9F)`. The mutex behind `guard` is released,
    /// the thread sleeps until signalled, and the mutex is reacquired before
    /// the guard is returned. Taking and returning the guard is how the
    /// requirement that the mutex be held becomes part of the signature.
    /// Wakeups can be spurious, so callers loop on their condition.
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>, _ctx: &Sleepable) -> MutexGuard<'a, T> {
        unsafe { sys::cv_wait(self.raw.get(), guard.m.raw()) };
        guard
    }

    /// Waits with `cv_wait_sig(9F)`, which lets a pending signal end the
    /// wait early. The boolean is `false` if a signal interrupted the wait.
    /// The mutex is reacquired in both cases.
    pub fn wait_sig<'a, T>(&self, guard: MutexGuard<'a, T>, _ctx: &Sleepable) -> (MutexGuard<'a, T>, bool) {
        let r = unsafe { sys::cv_wait_sig(self.raw.get(), guard.m.raw()) };
        (guard, r != 0)
    }

    /// Waits with `cv_reltimedwait(9F)` for at most `ticks` clock ticks. The
    /// boolean is `false` if the timeout expired before a signal arrived.
    pub fn wait_timeout_ticks<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        ticks: sys::clock_t,
        _ctx: &Sleepable,
    ) -> (MutexGuard<'a, T>, bool) {
        let r = unsafe { sys::cv_reltimedwait(self.raw.get(), guard.m.raw(), ticks, sys::time_res_t::TR_CLOCK_TICK) };
        (guard, r >= 0)
    }

    /// Wakes one waiting thread with `cv_signal(9F)`.
    pub fn notify_one(&self) {
        unsafe { sys::cv_signal(self.raw.get()) }
    }

    /// Wakes every waiting thread with `cv_broadcast(9F)`.
    pub fn notify_all(&self) {
        unsafe { sys::cv_broadcast(self.raw.get()) }
    }
}

impl Default for CondVar {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CondVar {
    fn drop(&mut self) {
        unsafe { sys::cv_destroy(self.raw.get()) }
    }
}
