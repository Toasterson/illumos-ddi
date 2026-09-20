//! `mutex(9F)` and `condvar(9F)` as owning types.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::context::Sleepable;
use crate::sys;

/// A `kmutex_t` of type `MUTEX_DRIVER` guarding a `T`.
///
/// The kernel requires `mutex_init(9F)` before use and `mutex_destroy(9F)`
/// after; both happen here. An unlocked adaptive mutex with no waiters is
/// a single zero word, so moving a `Mutex` between construction and its
/// final home (normally inside the driver's instance) is sound; it must
/// not be moved while locked, which the borrow held by [`MutexGuard`]
/// prevents.
pub struct Mutex<T> {
    raw: UnsafeCell<sys::kmutex_t>,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Create a `MUTEX_DRIVER` mutex with no interrupt cookie (never taken
    /// from a hardware interrupt handler).
    pub fn new(value: T) -> Mutex<T> {
        let m = Mutex { raw: UnsafeCell::new(sys::kmutex_t::UNINIT), data: UnsafeCell::new(value) };
        unsafe { sys::mutex_init(m.raw.get(), ptr::null(), sys::kmutex_type_t::MUTEX_DRIVER, ptr::null_mut()) };
        m
    }

    /// `mutex_enter(9F)`.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        unsafe { sys::mutex_enter(self.raw.get()) };
        MutexGuard { m: self }
    }

    /// `mutex_tryenter(9F)`.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if unsafe { sys::mutex_tryenter(self.raw.get()) } != 0 { Some(MutexGuard { m: self }) } else { None }
    }

    /// `mutex_owned(9F)`: whether the current thread holds the lock.
    pub fn is_held_by_current_thread(&self) -> bool {
        unsafe { sys::mutex_owned(self.raw.get()) != 0 }
    }

    /// Access without locking; requires `&mut`, so no one else can hold it.
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

/// Holds a [`Mutex`]; `mutex_exit(9F)` on drop.
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

/// A `kcondvar_t` of type `CV_DRIVER`.
pub struct CondVar {
    raw: UnsafeCell<sys::kcondvar_t>,
}

unsafe impl Send for CondVar {}
unsafe impl Sync for CondVar {}

impl CondVar {
    /// `cv_init(9F)`.
    pub fn new() -> CondVar {
        let cv = CondVar { raw: UnsafeCell::new(sys::kcondvar_t::UNINIT) };
        unsafe { sys::cv_init(cv.raw.get(), ptr::null(), sys::kcv_type_t::CV_DRIVER, ptr::null_mut()) };
        cv
    }

    /// `cv_wait(9F)`: atomically release the mutex behind `guard`, sleep,
    /// reacquire. Consuming and returning the guard encodes "the mutex must
    /// be held". Callers loop on their predicate.
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>, _ctx: &Sleepable) -> MutexGuard<'a, T> {
        unsafe { sys::cv_wait(self.raw.get(), guard.m.raw()) };
        guard
    }

    /// `cv_wait_sig(9F)`. Returns `None` if interrupted by a signal; the
    /// mutex is reacquired either way.
    pub fn wait_sig<'a, T>(&self, guard: MutexGuard<'a, T>, _ctx: &Sleepable) -> (MutexGuard<'a, T>, bool) {
        let r = unsafe { sys::cv_wait_sig(self.raw.get(), guard.m.raw()) };
        (guard, r != 0)
    }

    /// `cv_reltimedwait(9F)` with a relative timeout in ticks. `false` if
    /// the timeout expired first.
    pub fn wait_timeout_ticks<'a, T>(
        &self, guard: MutexGuard<'a, T>, ticks: sys::clock_t, _ctx: &Sleepable,
    ) -> (MutexGuard<'a, T>, bool) {
        let r = unsafe {
            sys::cv_reltimedwait(self.raw.get(), guard.m.raw(), ticks, sys::time_res_t::TR_CLOCK_TICK)
        };
        (guard, r >= 0)
    }

    /// `cv_signal(9F)`.
    pub fn notify_one(&self) {
        unsafe { sys::cv_signal(self.raw.get()) }
    }

    /// `cv_broadcast(9F)`.
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
