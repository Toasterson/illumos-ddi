//! Mutexes, condition variables, reader-writer locks and semaphores as
//! owning types.
//!
//! Every synchronisation object of the DDI has to be initialised before use
//! and destroyed afterwards, and most of them come with rules about who may
//! hold what when. The types here take care of the first part in their
//! constructors and destructors and express as much of the second part as
//! they can in their signatures.
//!
//! [`Mutex`] wraps a `kmutex_t` from `mutex(9F)` and owns the data it
//! protects. The only way to reach the data is through a [`MutexGuard`],
//! which calls `mutex_exit(9F)` when it is dropped.
//!
//! [`CondVar`] wraps a `kcondvar_t` from `condvar(9F)`. Its wait methods
//! consume a [`MutexGuard`] and return it, which is how the rule that the
//! mutex must be held across a wait becomes something the compiler checks.
//! Every wait may block, so every wait takes a [`Sleepable`] token. The
//! timed and interruptible waits report why they returned with a
//! [`WaitResult`].
//!
//! [`RwLock`] wraps a `krwlock_t` from `rwlock(9F)` and hands out
//! [`RwLockReadGuard`] and [`RwLockWriteGuard`]. A write guard can be
//! downgraded to a read guard, and a read guard can try to upgrade to a
//! write guard.
//!
//! [`Semaphore`] wraps a `ksema_t` from `semaphore(9F)`. It protects no
//! data, because a semaphore is a counter rather than a lock, and it is not
//! owned by the thread that decremented it.
//!
//! The rules for interrupt handlers are these:
//!
//! - A mutex or reader-writer lock that an interrupt handler acquires must
//!   be created with that handler's priority, using [`Mutex::with_priority`]
//!   or [`RwLock::with_priority`] with the [`IntrPriority`] reported for the
//!   interrupt. A lock created with [`Mutex::new`] or [`RwLock::new`] must
//!   never be acquired from interrupt context, because the handler could
//!   then block on a thread that it has itself interrupted.
//! - A mutex created with the priority of a high-level interrupt, one at or
//!   above the level that `ddi_intr_get_hilevel_pri(9F)` reports, becomes a spin
//!   mutex that raises the processor priority while it is held. It is the
//!   only kind of lock a high-level handler may take. A reader-writer lock
//!   always blocks when it is contended and must never be taken by a
//!   high-level handler, whatever priority it was created with.
//! - Waiting on a condition variable and decrementing a semaphore with
//!   [`Semaphore::acquire`] block, so they require a [`Sleepable`] token,
//!   which interrupt handlers do not receive. Signalling a condition
//!   variable, releasing a semaphore and the `try_` methods never block and
//!   are available everywhere.
//!
//! None of the locks here are recursive. Acquiring a [`Mutex`] that the
//! current thread already holds panics the system, and acquiring an
//! [`RwLock`] for reading that the current thread already holds for reading
//! can deadlock against a waiting writer, as `rwlock(9F)` explains. Those
//! are logic errors that the types do not prevent.
//!
//! The kernel objects are small and do not point to themselves while they
//! are unlocked and have no waiters, so all of these types may be moved
//! between their construction and their final home, normally a field of the
//! driver's instance. While a lock is held or a thread is waiting, the
//! guard or the waiting call borrows the object, so it cannot be moved or
//! dropped. The guards are neither `Send` nor transferable in any other
//! way, because illumos requires that the thread that acquired a mutex or a
//! write lock is the thread that releases it.

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::time::Duration;

use crate::context::Sleepable;
use crate::error::Errno;
use crate::intr::IntrPriority;
use crate::sys;

/// A `kmutex_t` of kind `MUTEX_DRIVER` that guards a `T`.
///
/// The constructor calls `mutex_init(9F)` and `Drop` calls
/// `mutex_destroy(9F)`, so a driver cannot forget either. An unlocked
/// adaptive mutex with no waiters is a single zero word, and an unlocked
/// spin mutex holds only its lock bytes and its priority, which is why a
/// `Mutex` may be moved between its construction and its final home,
/// normally a field of the driver's instance. It must not be moved while
/// locked, and the borrow held by a [`MutexGuard`] makes that impossible.
pub struct Mutex<T> {
    raw: UnsafeCell<sys::kmutex_t>,
    data: UnsafeCell<T>,
}

// SAFETY: The mutex owns its `T`, so sending the mutex sends the `T`, which
// requires `T: Send`. The kernel mutex itself has no thread affinity while
// it is unlocked, and it cannot be moved while locked.
unsafe impl<T: Send> Send for Mutex<T> {}
// SAFETY: Shared access to the mutex only yields access to the `T` through
// a guard, and `mutex_enter(9F)` ensures that one guard exists at a time.
// That hands the `T` from thread to thread, which requires `T: Send` but
// not `T: Sync`.
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a `MUTEX_DRIVER` mutex without an interrupt priority, using
    /// `mutex_init(9F)` with a NULL argument. Such a mutex is adaptive and
    /// must never be acquired from an interrupt handler. A mutex that an
    /// interrupt handler takes is created with [`Mutex::with_priority`]
    /// instead.
    pub fn new(value: T) -> Mutex<T> {
        Self::init(value, ptr::null_mut())
    }

    /// Creates a `MUTEX_DRIVER` mutex that the interrupt handler of
    /// priority `pri` may acquire. The priority is converted with
    /// `DDI_INTR_PRI` and passed to `mutex_init(9F)`, as the manual
    /// requires for every mutex that an interrupt handler uses.
    ///
    /// For a normal interrupt the result is an adaptive mutex like the one
    /// [`Mutex::new`] creates. For a high-level interrupt, whose priority is
    /// at least the one `ddi_intr_get_hilevel_pri(9F)` reports, the kernel
    /// creates a spin mutex that raises the processor priority to `pri` for
    /// as long as it is held. Such a mutex should be held only briefly.
    pub fn with_priority(value: T, pri: IntrPriority) -> Mutex<T> {
        Self::init(value, pri.cookie())
    }

    fn init(value: T, cookie: sys::ddi_iblock_cookie_t) -> Mutex<T> {
        let m = Mutex { raw: UnsafeCell::new(sys::kmutex_t::UNINIT), data: UnsafeCell::new(value) };
        // SAFETY: The storage is zeroed and has not been initialised before.
        // The cookie is either NULL or a value from `DDI_INTR_PRI`, which
        // are the two forms `mutex_init(9F)` accepts for `MUTEX_DRIVER`.
        unsafe { sys::mutex_init(m.raw.get(), ptr::null(), sys::kmutex_type_t::MUTEX_DRIVER, cookie) };
        m
    }

    /// Acquires the mutex with `mutex_enter(9F)` and returns a guard that
    /// releases it when dropped.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // SAFETY: The mutex was initialised in the constructor and is not
        // destroyed before `self` is dropped.
        unsafe { sys::mutex_enter(self.raw.get()) };
        MutexGuard { m: self, _not_send: PhantomData }
    }

    /// Tries to acquire the mutex with `mutex_tryenter(9F)` without blocking.
    /// Returns `None` if another thread holds it.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        // SAFETY: As in `lock`.
        if unsafe { sys::mutex_tryenter(self.raw.get()) } != 0 {
            Some(MutexGuard { m: self, _not_send: PhantomData })
        } else {
            None
        }
    }

    /// Reports whether the current thread holds the mutex, using
    /// `mutex_owned(9F)`. Intended for assertions.
    pub fn is_held_by_current_thread(&self) -> bool {
        // SAFETY: As in `lock`.
        unsafe { sys::mutex_owned(self.raw.get()) != 0 }
    }

    /// Gives access to the data without taking the lock. This requires a
    /// mutable reference to the mutex, which proves that no guard exists.
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    /// Returns the address of the `kmutex_t`, for kernel structures such as
    /// the `ks_lock` field of a `kstat(9S)` that refer to a mutex by
    /// pointer.
    pub(crate) fn raw(&self) -> *mut sys::kmutex_t {
        self.raw.get()
    }
}

impl<T> Drop for Mutex<T> {
    fn drop(&mut self) {
        // SAFETY: `&mut self` proves that no guard exists, so the mutex is
        // not held, and it is destroyed exactly once.
        unsafe { sys::mutex_destroy(self.raw.get()) }
    }
}

/// Proof that a [`Mutex`] is held. Dereferences to the guarded data and
/// calls `mutex_exit(9F)` when dropped.
///
/// A guard is not `Send`, because `mutex_exit(9F)` must be called by the
/// thread that acquired the mutex.
pub struct MutexGuard<'a, T> {
    m: &'a Mutex<T>,
    _not_send: PhantomData<*mut ()>,
}

// SAFETY: A shared reference to the guard only gives out `&T`, so sharing
// it between threads is sound whenever sharing `&T` is.
unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: The guard proves that the mutex is held by this thread.
        unsafe { &*self.m.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: The guard proves that the mutex is held by this thread,
        // and `&mut self` proves that no other reference into the data
        // derived from this guard exists.
        unsafe { &mut *self.m.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: The guard proves that this thread holds the mutex.
        unsafe { sys::mutex_exit(self.m.raw()) }
    }
}

/// Why a timed or interruptible wait on a [`CondVar`] returned.
///
/// The value comes straight from the return convention of `condvar(9F)`.
/// None of the outcomes says anything reliable about the condition the
/// caller is waiting for. A thread may be woken without a call to
/// `cv_signal(9F)` or `cv_broadcast(9F)`, and the condition may have become
/// true just as the timeout expired, so the caller always checks its
/// condition again and uses the outcome only to decide whether to keep
/// waiting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub enum WaitResult {
    /// The thread was woken by `cv_signal(9F)`, by `cv_broadcast(9F)` or
    /// prematurely for another reason. The kernel reports this with a
    /// positive return value.
    Woken,
    /// The timeout expired. The kernel reports this with -1.
    TimedOut,
    /// A signal is pending for the thread, and the wait ended so that the
    /// entry point can return `EINTR` to the caller. The kernel reports
    /// this with 0. Only the `_sig` variants of the wait return it.
    Interrupted,
}

impl WaitResult {
    /// Decodes the return value of the `_sig` variants, where -1 means the
    /// timeout expired, 0 means a signal is pending and a positive value
    /// means the thread was woken.
    fn from_sig(r: sys::clock_t) -> WaitResult {
        match r {
            r if r < 0 => WaitResult::TimedOut,
            0 => WaitResult::Interrupted,
            _ => WaitResult::Woken,
        }
    }

    /// Decodes the return value of the variants that ignore signals, where
    /// -1 means the timeout expired and anything else means the thread was
    /// woken.
    fn from_timed(r: sys::clock_t) -> WaitResult {
        if r < 0 { WaitResult::TimedOut } else { WaitResult::Woken }
    }

    /// Reports whether the timeout expired.
    pub fn timed_out(self) -> bool {
        self == WaitResult::TimedOut
    }

    /// Reports whether a pending signal ended the wait.
    pub fn interrupted(self) -> bool {
        self == WaitResult::Interrupted
    }
}

/// Converts a duration into clock ticks for `cv_reltimedwait(9F)`.
///
/// The duration is rounded up to whole microseconds and converted with
/// `drv_usectohz(9F)`, which rounds up to whole ticks, so a wait never ends
/// before the requested time has passed. A zero duration becomes zero
/// ticks, which the kernel treats as an expired timeout. Durations beyond
/// roughly 146 years are clamped, so that neither `drv_usectohz` nor the
/// kernel's conversion of ticks back into nanoseconds can overflow.
fn duration_to_ticks(d: Duration) -> sys::clock_t {
    // The kernel converts the tick count back to nanoseconds in a signed
    // 64-bit value. Half of that range, expressed in microseconds, leaves
    // room for the rounding in both directions.
    const MAX_USEC: u128 = (i64::MAX as u128 / 2) / 1000;
    let usec = d.as_nanos().div_ceil(1000).min(MAX_USEC) as sys::clock_t;
    // SAFETY: `drv_usectohz(9F)` has no preconditions and may be called
    // from any context.
    unsafe { sys::drv_usectohz(usec) }
}

/// A `kcondvar_t` of kind `CV_DRIVER`.
///
/// The constructor calls `cv_init(9F)` and `Drop` calls `cv_destroy(9F)`.
/// A condition variable is always used together with a [`Mutex`]. Each wait
/// method takes the guard of that mutex, releases the mutex while the
/// thread sleeps, and hands the guard back once the mutex has been
/// reacquired. The manual advises against waiting in interrupt context and
/// forbids it in high-level interrupt context, so every wait requires a
/// [`Sleepable`] token. Every wait can also end without a matching
/// `cv_signal(9F)`, so callers wait in a loop that checks their condition.
pub struct CondVar {
    raw: UnsafeCell<sys::kcondvar_t>,
}

// SAFETY: A condition variable holds no data of its own. Waiting and
// signalling from any thread is what it is for, and it cannot be moved or
// destroyed while a thread waits on it, because the wait borrows it.
unsafe impl Send for CondVar {}
// SAFETY: As above; all operations on a shared `CondVar` are the kernel's
// own thread-safe `cv_*` functions.
unsafe impl Sync for CondVar {}

impl CondVar {
    /// Creates a condition variable with `cv_init(9F)`.
    pub fn new() -> CondVar {
        let cv = CondVar { raw: UnsafeCell::new(sys::kcondvar_t::UNINIT) };
        // SAFETY: The storage is zeroed and has not been initialised before.
        unsafe { sys::cv_init(cv.raw.get(), ptr::null(), sys::kcv_type_t::CV_DRIVER, ptr::null_mut()) };
        cv
    }

    /// Waits with `cv_wait(9F)`. The mutex behind `guard` is released,
    /// the thread sleeps until signalled, and the mutex is reacquired before
    /// the guard is returned. Taking and returning the guard is how the
    /// requirement that the mutex be held becomes part of the signature.
    /// Wakeups can be spurious, so callers loop on their condition.
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>, _ctx: &Sleepable) -> MutexGuard<'a, T> {
        // SAFETY: The condition variable is initialised, and the guard
        // proves that this thread holds the mutex.
        unsafe { sys::cv_wait(self.raw.get(), guard.m.raw()) };
        guard
    }

    /// Waits with `cv_wait_sig(9F)`, which lets a pending signal end the
    /// wait early. The boolean is `false` if a signal interrupted the wait.
    /// The mutex is reacquired in both cases.
    ///
    /// A thread that cannot receive signals, for example one running
    /// `close(9E)` because its process is exiting, waits as with
    /// [`CondVar::wait`].
    pub fn wait_sig<'a, T>(&self, guard: MutexGuard<'a, T>, _ctx: &Sleepable) -> (MutexGuard<'a, T>, bool) {
        // SAFETY: As in `wait`.
        let r = unsafe { sys::cv_wait_sig(self.raw.get(), guard.m.raw()) };
        (guard, r != 0)
    }

    /// Waits with `cv_timedwait(9F)` until the thread is woken or the clock
    /// reaches `deadline`, whichever happens first.
    ///
    /// The deadline is an absolute time in clock ticks since boot, the
    /// value `ddi_get_lbolt(9F)` returns. The usual pattern computes it
    /// once, before the loop that checks the condition, so that spurious
    /// wakeups do not extend the total wait. A deadline that has already
    /// passed makes the call return [`WaitResult::TimedOut`] at once
    /// without releasing the mutex. The result is never
    /// [`WaitResult::Interrupted`].
    pub fn wait_until<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        deadline: sys::clock_t,
        _ctx: &Sleepable,
    ) -> (MutexGuard<'a, T>, WaitResult) {
        // SAFETY: As in `wait`.
        let r = unsafe { sys::cv_timedwait(self.raw.get(), guard.m.raw(), deadline) };
        (guard, WaitResult::from_timed(r))
    }

    /// Waits with `cv_timedwait_sig(9F)` until the thread is woken, the
    /// clock reaches `deadline`, or a signal is pending for the thread.
    ///
    /// The deadline is interpreted as in [`CondVar::wait_until`]. A thread
    /// that cannot receive signals waits as with `wait_until`.
    pub fn wait_until_sig<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        deadline: sys::clock_t,
        _ctx: &Sleepable,
    ) -> (MutexGuard<'a, T>, WaitResult) {
        // SAFETY: As in `wait`.
        let r = unsafe { sys::cv_timedwait_sig(self.raw.get(), guard.m.raw(), deadline) };
        (guard, WaitResult::from_sig(r))
    }

    /// Waits with `cv_reltimedwait(9F)` until the thread is woken or
    /// `timeout` has passed, whichever happens first.
    ///
    /// The timeout is converted to clock ticks with `drv_usectohz(9F)`,
    /// rounding up, and passed with the resolution `TR_CLOCK_TICK`, which is
    /// what the manual recommends. A zero timeout makes the call return
    /// [`WaitResult::TimedOut`] at once without releasing the mutex. The
    /// result is never [`WaitResult::Interrupted`].
    ///
    /// Each call starts a new timeout. A caller that loops on its condition
    /// and wants to bound the total time uses [`CondVar::wait_until`]
    /// instead.
    pub fn wait_timeout<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        timeout: Duration,
        _ctx: &Sleepable,
    ) -> (MutexGuard<'a, T>, WaitResult) {
        let ticks = duration_to_ticks(timeout);
        // SAFETY: As in `wait`.
        let r = unsafe { sys::cv_reltimedwait(self.raw.get(), guard.m.raw(), ticks, sys::time_res_t::TR_CLOCK_TICK) };
        (guard, WaitResult::from_timed(r))
    }

    /// Waits with `cv_reltimedwait_sig(9F)` until the thread is woken,
    /// `timeout` has passed, or a signal is pending for the thread.
    ///
    /// The timeout is converted as in [`CondVar::wait_timeout`]. A thread
    /// that cannot receive signals waits as with `wait_timeout`.
    pub fn wait_timeout_sig<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        timeout: Duration,
        _ctx: &Sleepable,
    ) -> (MutexGuard<'a, T>, WaitResult) {
        let ticks = duration_to_ticks(timeout);
        // SAFETY: As in `wait`.
        let r =
            unsafe { sys::cv_reltimedwait_sig(self.raw.get(), guard.m.raw(), ticks, sys::time_res_t::TR_CLOCK_TICK) };
        (guard, WaitResult::from_sig(r))
    }

    /// Waits with `cv_reltimedwait(9F)` for at most `ticks` clock ticks. The
    /// boolean is `false` if the timeout expired before a signal arrived.
    ///
    /// This is the tick-based form of [`CondVar::wait_timeout`], for callers
    /// that already have a tick count.
    pub fn wait_timeout_ticks<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        ticks: sys::clock_t,
        _ctx: &Sleepable,
    ) -> (MutexGuard<'a, T>, bool) {
        // SAFETY: As in `wait`.
        let r = unsafe { sys::cv_reltimedwait(self.raw.get(), guard.m.raw(), ticks, sys::time_res_t::TR_CLOCK_TICK) };
        (guard, r >= 0)
    }

    /// Wakes one waiting thread with `cv_signal(9F)`.
    pub fn notify_one(&self) {
        // SAFETY: The condition variable is initialised.
        unsafe { sys::cv_signal(self.raw.get()) }
    }

    /// Wakes every waiting thread with `cv_broadcast(9F)`.
    pub fn notify_all(&self) {
        // SAFETY: The condition variable is initialised.
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
        // SAFETY: `&mut self` proves that no thread is waiting, because a
        // wait borrows the condition variable, and it is destroyed once.
        unsafe { sys::cv_destroy(self.raw.get()) }
    }
}

/// A `krwlock_t` of kind `RW_DRIVER` that guards a `T`.
///
/// Many threads may hold the lock for reading at the same time, and they
/// share the data through [`RwLockReadGuard`]. One thread at a time may
/// hold it for writing, with exclusive access through [`RwLockWriteGuard`].
/// A thread asking for a read lock waits not only while a writer holds the
/// lock but also while a writer is waiting for it, so writers are not
/// starved.
///
/// The constructor calls `rw_init(9F)` and `Drop` calls `rw_destroy(9F)`.
/// The manual allows both only in user context. Neither the constructor nor
/// the destructor demands a [`Sleepable`] token, for symmetry with
/// [`Mutex`] and because a destructor cannot take an argument, so a driver
/// creates and drops its locks in thread context such as `attach(9E)` and
/// `detach(9E)`.
///
/// Acquiring the lock may block. An interrupt handler may acquire it only
/// if the lock was created with [`RwLock::with_priority`] and that
/// handler's priority, and a high-level interrupt handler may never acquire
/// it. The `try_` methods never block.
///
/// An unlocked reader-writer lock without waiters is a single zero word,
/// which is why an `RwLock` may be moved until it is first locked. While it
/// is locked, the borrow held by the guard prevents a move.
pub struct RwLock<T> {
    raw: UnsafeCell<sys::krwlock_t>,
    data: UnsafeCell<T>,
}

// SAFETY: The lock owns its `T`, so sending the lock sends the `T`. The
// kernel object has no thread affinity while it is unlocked, and it cannot
// be moved while locked.
unsafe impl<T: Send> Send for RwLock<T> {}
// SAFETY: A shared lock gives several threads `&T` at once through read
// guards, which requires `T: Sync`, and hands `&mut T` from thread to
// thread through write guards, which requires `T: Send`.
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    /// Creates an `RW_DRIVER` lock that no interrupt handler uses, calling
    /// `rw_init(9F)` with a NULL argument.
    pub fn new(value: T) -> RwLock<T> {
        Self::init(value, ptr::null_mut())
    }

    /// Creates an `RW_DRIVER` lock that the interrupt handler of priority
    /// `pri` may acquire. The priority is converted with `DDI_INTR_PRI` and
    /// passed to `rw_init(9F)`, as the manual requires for a lock used by an
    /// interrupt handler.
    ///
    /// `pri` must not be a high-level priority. A reader-writer lock blocks
    /// when it is contended, and a high-level interrupt handler must never
    /// block.
    pub fn with_priority(value: T, pri: IntrPriority) -> RwLock<T> {
        Self::init(value, pri.cookie())
    }

    fn init(value: T, cookie: sys::ddi_iblock_cookie_t) -> RwLock<T> {
        let l = RwLock { raw: UnsafeCell::new(sys::krwlock_t::UNINIT), data: UnsafeCell::new(value) };
        // SAFETY: The storage is zeroed and has not been initialised before.
        // The cookie is either NULL or a value from `DDI_INTR_PRI`, which
        // are the two forms `rw_init(9F)` documents.
        unsafe { sys::rw_init(l.raw.get(), ptr::null(), sys::krw_type_t::RW_DRIVER, cookie) };
        l
    }

    /// Acquires the lock for reading with `rw_enter(9F)` and `RW_READER`,
    /// blocking while a writer holds the lock or waits for it.
    ///
    /// A thread must not acquire a read lock that it already holds. If a
    /// writer started waiting in between, the second acquisition waits for
    /// the writer, which waits for the first read lock, and the two threads
    /// deadlock.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        // SAFETY: The lock was initialised in the constructor and is not
        // destroyed before `self` is dropped.
        unsafe { sys::rw_enter(self.raw.get(), sys::krw_t::RW_READER) };
        RwLockReadGuard { l: self, _not_send: PhantomData }
    }

    /// Acquires the lock for writing with `rw_enter(9F)` and `RW_WRITER`,
    /// blocking while any other thread holds it.
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        // SAFETY: As in `read`.
        unsafe { sys::rw_enter(self.raw.get(), sys::krw_t::RW_WRITER) };
        RwLockWriteGuard { l: self, _not_send: PhantomData }
    }

    /// Tries to acquire the lock for reading with `rw_tryenter(9F)` without
    /// blocking. Returns `None` if a writer holds the lock or waits for it.
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        // SAFETY: As in `read`.
        if unsafe { sys::rw_tryenter(self.raw.get(), sys::krw_t::RW_READER) } != 0 {
            Some(RwLockReadGuard { l: self, _not_send: PhantomData })
        } else {
            None
        }
    }

    /// Tries to acquire the lock for writing with `rw_tryenter(9F)` without
    /// blocking. Returns `None` if any other thread holds the lock.
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        // SAFETY: As in `read`.
        if unsafe { sys::rw_tryenter(self.raw.get(), sys::krw_t::RW_WRITER) } != 0 {
            Some(RwLockWriteGuard { l: self, _not_send: PhantomData })
        } else {
            None
        }
    }

    /// Gives access to the data without taking the lock. This requires a
    /// mutable reference to the lock, which proves that no guard exists.
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

impl<T> Drop for RwLock<T> {
    fn drop(&mut self) {
        // SAFETY: `&mut self` proves that no guard exists, so the lock is
        // not held, and it is destroyed exactly once.
        unsafe { sys::rw_destroy(self.raw.get()) }
    }
}

/// Proof that an [`RwLock`] is held for reading. Dereferences to the shared
/// data and calls `rw_exit(9F)` when dropped.
///
/// A guard is not `Send`, so the lock is released by the thread that
/// acquired it.
pub struct RwLockReadGuard<'a, T> {
    l: &'a RwLock<T>,
    _not_send: PhantomData<*mut ()>,
}

// SAFETY: A shared reference to the guard only gives out `&T`, so sharing
// it between threads is sound whenever sharing `&T` is.
unsafe impl<T: Sync> Sync for RwLockReadGuard<'_, T> {}

impl<'a, T> RwLockReadGuard<'a, T> {
    /// Tries to convert the read lock into a write lock with
    /// `rw_tryupgrade(9F)` without releasing it.
    ///
    /// The upgrade succeeds only if no other thread holds the lock and no
    /// thread is waiting to write. On failure the read lock is still held,
    /// and the read guard is handed back in `Err`. A caller that must have
    /// the write lock then drops the read guard, calls [`RwLock::write`]
    /// and checks again whatever it read, because another writer may have
    /// run in between.
    pub fn try_upgrade(self) -> Result<RwLockWriteGuard<'a, T>, RwLockReadGuard<'a, T>> {
        // SAFETY: The guard proves that this thread holds the lock for
        // reading, which is what `rw_tryupgrade(9F)` requires.
        if unsafe { sys::rw_tryupgrade(self.l.raw.get()) } != 0 {
            // The lock is now held for writing. The read guard must not run
            // its destructor, because the single `rw_exit(9F)` that is due
            // belongs to the write guard.
            let this = ManuallyDrop::new(self);
            Ok(RwLockWriteGuard { l: this.l, _not_send: PhantomData })
        } else {
            Err(self)
        }
    }
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: The guard proves that the lock is held for reading, so no
        // writer can hold a mutable reference to the data.
        unsafe { &*self.l.data.get() }
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: The guard proves that this thread holds the lock.
        unsafe { sys::rw_exit(self.l.raw.get()) }
    }
}

/// Proof that an [`RwLock`] is held for writing. Dereferences to the data
/// with exclusive access and calls `rw_exit(9F)` when dropped.
///
/// A guard is not `Send`, because the lock must be released by the thread
/// that acquired it.
pub struct RwLockWriteGuard<'a, T> {
    l: &'a RwLock<T>,
    _not_send: PhantomData<*mut ()>,
}

// SAFETY: A shared reference to the guard only gives out `&T`, so sharing
// it between threads is sound whenever sharing `&T` is.
unsafe impl<T: Sync> Sync for RwLockWriteGuard<'_, T> {}

impl<'a, T> RwLockWriteGuard<'a, T> {
    /// Converts the write lock into a read lock with `rw_downgrade(9F)`
    /// without releasing it. Readers that were waiting may proceed, but no
    /// writer can slip in between, so the data the caller has just written
    /// is still what it reads.
    pub fn downgrade(self) -> RwLockReadGuard<'a, T> {
        // SAFETY: The guard proves that this thread holds the lock for
        // writing, which is what `rw_downgrade(9F)` requires.
        unsafe { sys::rw_downgrade(self.l.raw.get()) };
        // The single `rw_exit(9F)` that is due now belongs to the read
        // guard, so the write guard must not run its destructor.
        let this = ManuallyDrop::new(self);
        RwLockReadGuard { l: this.l, _not_send: PhantomData }
    }
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: The guard proves that this thread holds the lock for
        // writing, so no other reference to the data exists elsewhere.
        unsafe { &*self.l.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: As in `deref`, and `&mut self` proves that no other
        // reference derived from this guard exists.
        unsafe { &mut *self.l.data.get() }
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: The guard proves that this thread holds the lock.
        unsafe { sys::rw_exit(self.l.raw.get()) }
    }
}

/// A counting semaphore, `ksema_t` of kind `SEMA_DRIVER`.
///
/// A semaphore holds a count. [`Semaphore::acquire`] decrements it and
/// blocks while it is zero, and [`Semaphore::release`] increments it and
/// wakes a blocked thread. Unlike a mutex, a semaphore has no owner. One
/// thread may acquire it and another release it, which is what makes it
/// useful for counting free resources or for handing a completion from an
/// interrupt handler to a waiting thread. For the same reason there is no
/// guard; a driver pairs the calls itself.
///
/// The constructor calls `sema_init(9F)` and `Drop` calls
/// `sema_destroy(9F)`. The manual allows both only in user or kernel
/// context. As with [`Mutex`], neither demands a [`Sleepable`] token, so a
/// driver creates and drops its semaphores in thread context such as
/// `attach(9E)` and `detach(9E)`.
///
/// No semaphore function may be called from a high-level interrupt handler.
/// A semaphore that such a handler needs to signal is the wrong tool; a
/// spin [`Mutex`] and a soft interrupt are the usual answer.
///
/// An idle semaphore holds only its count and an empty queue pointer, so it
/// may be moved until a thread first waits on it. While a thread waits, it
/// borrows the semaphore, which prevents a move.
pub struct Semaphore {
    raw: UnsafeCell<sys::ksema_t>,
}

// SAFETY: A semaphore holds no data besides its count, and every operation
// on it is one of the kernel's own thread-safe `sema_*` functions. It has no
// owner, so there is no thread affinity to preserve.
unsafe impl Send for Semaphore {}
// SAFETY: As above.
unsafe impl Sync for Semaphore {}

impl Semaphore {
    /// Creates a semaphore with the initial count `count`, using
    /// `sema_init(9F)` with `SEMA_DRIVER`.
    pub fn new(count: u32) -> Semaphore {
        let s = Semaphore { raw: UnsafeCell::new(sys::ksema_t::UNINIT) };
        // SAFETY: The storage is zeroed and has not been initialised before.
        // The name is NULL and the argument NULL, as the manual asks.
        unsafe { sys::sema_init(s.raw.get(), count, ptr::null(), sys::ksema_type_t::SEMA_DRIVER, ptr::null_mut()) };
        s
    }

    /// Decrements the count with `sema_p(9F)`, blocking while it is zero.
    ///
    /// The manual advises against calling this from interrupt context,
    /// because lower priority interrupts are not serviced during the wait,
    /// so it requires a [`Sleepable`] token.
    pub fn acquire(&self, _ctx: &Sleepable) {
        // SAFETY: The semaphore was initialised in the constructor and is
        // not destroyed before `self` is dropped.
        unsafe { sys::sema_p(self.raw.get()) }
    }

    /// Decrements the count with `sema_p_sig(9F)`, blocking while it is zero
    /// unless a signal is pending for the thread.
    ///
    /// Returns `Err(Errno::EINTR)` if a signal ended the wait, in which case
    /// the count was not decremented and the caller must not release it.
    /// The error is meant to be returned from the entry point, so that the
    /// system call reports the interruption to the process. A thread that
    /// cannot receive signals waits as with [`Semaphore::acquire`].
    pub fn acquire_sig(&self, _ctx: &Sleepable) -> Result<(), Errno> {
        // SAFETY: As in `acquire`.
        if unsafe { sys::sema_p_sig(self.raw.get()) } == 0 { Ok(()) } else { Err(Errno::EINTR) }
    }

    /// Decrements the count with `sema_tryp(9F)` if it is greater than zero,
    /// without blocking. Returns `true` if the count was decremented.
    pub fn try_acquire(&self) -> bool {
        // SAFETY: As in `acquire`.
        unsafe { sys::sema_tryp(self.raw.get()) != 0 }
    }

    /// Increments the count with `sema_v(9F)` and wakes one thread blocked
    /// in [`Semaphore::acquire`], if there is one. This never blocks and
    /// may be called from a normal interrupt handler, but not from a
    /// high-level one.
    pub fn release(&self) {
        // SAFETY: As in `acquire`.
        unsafe { sys::sema_v(self.raw.get()) }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        // SAFETY: `&mut self` proves that no thread is blocked on the
        // semaphore, because a waiting thread borrows it, and it is
        // destroyed exactly once.
        unsafe { sys::sema_destroy(self.raw.get()) }
    }
}
