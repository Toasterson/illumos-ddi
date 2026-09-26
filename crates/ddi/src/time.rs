//! Clock ticks, high-resolution time, waiting, and callbacks that run later.
//!
//! The kernel measures time in two units. The clock tick is the unit of
//! `lbolt`, of `delay(9F)` and of `timeout(9F)`, and its length depends on
//! the configured clock rate. The nanosecond is the unit of `gethrtime(9F)`
//! and of `ddi_periodic_add(9F)`. A driver should never assume a tick rate,
//! which is why [`Ticks`] is a separate type that is converted to and from
//! microseconds and [`Duration`] only through `drv_usectohz(9F)` and
//! `drv_hztousec(9F)`. [`HrTime`] is a reading of the high-resolution clock,
//! and the difference of two readings is a [`Duration`].
//!
//! There are two ways to wait. [`delay`] puts the thread to sleep for a
//! number of ticks and therefore needs a [`Sleepable`] token. [`busy_wait`]
//! spins on the processor with `drv_usecwait(9F)` and is allowed in any
//! context, which makes it the right choice for the few microseconds a device
//! needs to notice a register write, and the wrong choice for anything
//! longer.
//!
//! There are two ways to have code run later. A [`Timeout`] runs a closure
//! once after a number of ticks, and a [`Periodic`] runs a closure over and
//! over at a fixed interval. Both own the closure and the registration with
//! the kernel. Dropping either cancels the registration, waits for a
//! callback that is already running to finish, and only then frees the
//! closure, so the closure is never freed while the kernel can still call
//! it.
//!
//! # Dropping a timer waits for its callback
//!
//! `untimeout(9F)` and `ddi_periodic_delete(9F)` do not return while the
//! callback is running on another processor. That makes dropping a
//! [`Timeout`] or a [`Periodic`] safe with respect to memory, and it is also
//! the one rule in this module that the compiler cannot check. A thread
//! that drops a timer while it holds a lock that the timer's callback takes
//! will deadlock if the callback has already started and is waiting for that
//! lock. The same is true of any other wait the callback may be stuck in on
//! behalf of the dropping thread. Release such locks before dropping the
//! timer. For example, take the timer out of the state guarded by the mutex
//! with `Option::take`, release the guard, and drop the timer afterwards.
//!
//! Both functions also block, so a timer must be dropped in user or kernel
//! context. A [`Periodic`] must in addition never be dropped from inside its
//! own callback; `ddi_periodic_delete(9F)` panics the system when that
//! happens.

use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;
use core::ops::Sub;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, Ordering};
use core::time::Duration;

use crate::context::{Atomic, Sleepable};
use crate::kmem::KBox;
use crate::sys;

/// Microseconds as the `clock_t` the `drv_*` functions take. Values that do
/// not fit are saturated to the largest `clock_t`, which the kernel treats
/// as "as long as possible" anyway.
fn clock_from_u64(n: u64) -> sys::clock_t {
    sys::clock_t::try_from(n).unwrap_or(sys::clock_t::MAX)
}

/// Converts a duration to whole microseconds, rounding a fractional
/// microsecond up so that a wait is never shorter than requested.
fn micros_rounded_up(d: Duration) -> u64 {
    let whole = d.as_micros();
    let whole = if !d.subsec_nanos().is_multiple_of(1_000) { whole.saturating_add(1) } else { whole };
    u64::try_from(whole).unwrap_or(u64::MAX)
}

/// A number of system clock ticks.
///
/// This is the unit of `lbolt`, of `delay(9F)`, of `timeout(9F)` and of the
/// timed condition variable waits. The number of ticks in a second depends on
/// the kernel's configured clock rate, so conversions from and to real time
/// go through `drv_usectohz(9F)` and `drv_hztousec(9F)` rather than through
/// a constant. A `Ticks` value is never negative.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ticks(sys::clock_t);

impl Ticks {
    /// Zero ticks.
    pub const ZERO: Ticks = Ticks(0);

    /// Wraps a raw number of ticks. Values larger than the largest
    /// `clock_t` are saturated to it.
    pub const fn new(ticks: u64) -> Ticks {
        if ticks > sys::clock_t::MAX as u64 { Ticks(sys::clock_t::MAX) } else { Ticks(ticks as sys::clock_t) }
    }

    /// Returns the number of ticks.
    pub const fn get(self) -> u64 {
        self.0 as u64
    }

    /// Returns the number of ticks as the `clock_t` that the `9F` functions
    /// take.
    pub const fn as_raw(self) -> sys::clock_t {
        self.0
    }

    /// Converts microseconds to ticks with `drv_usectohz(9F)`. The kernel
    /// rounds up, so any positive number of microseconds gives at least one
    /// tick, and a result too large for a `clock_t` saturates.
    pub fn from_micros(micros: u64) -> Ticks {
        // SAFETY: drv_usectohz(9F) is a pure computation that may be called
        // from any context.
        let t = unsafe { sys::drv_usectohz(clock_from_u64(micros)) };
        Ticks(t.max(0))
    }

    /// Converts a duration to ticks with `drv_usectohz(9F)`. A fractional
    /// microsecond is rounded up before the conversion, and the kernel
    /// rounds up again to whole ticks, so the result never stands for less
    /// time than `d`.
    pub fn from_duration(d: Duration) -> Ticks {
        Ticks::from_micros(micros_rounded_up(d))
    }

    /// Converts the ticks to microseconds with `drv_hztousec(9F)`. A result
    /// too large for a `clock_t` saturates.
    pub fn as_micros(self) -> u64 {
        // SAFETY: drv_hztousec(9F) is a pure computation that may be called
        // from any context.
        let us = unsafe { sys::drv_hztousec(self.0) };
        us.max(0) as u64
    }

    /// Converts the ticks to a duration with `drv_hztousec(9F)`.
    pub fn as_duration(self) -> Duration {
        Duration::from_micros(self.as_micros())
    }

    /// Adds two tick counts, saturating at the largest `clock_t`.
    pub const fn saturating_add(self, rhs: Ticks) -> Ticks {
        Ticks(self.0.saturating_add(rhs.0))
    }

    /// Subtracts `rhs`, giving zero if `rhs` is the larger value. This is
    /// how the ticks elapsed between two readings of [`lbolt`] are
    /// computed.
    pub const fn saturating_sub(self, rhs: Ticks) -> Ticks {
        let d = self.0.saturating_sub(rhs.0);
        if d < 0 { Ticks(0) } else { Ticks(d) }
    }
}

impl From<Duration> for Ticks {
    /// Converts with [`Ticks::from_duration`].
    fn from(d: Duration) -> Ticks {
        Ticks::from_duration(d)
    }
}

/// Returns the number of clock ticks since the system booted, using
/// `ddi_get_lbolt64(9F)`. The 64-bit counter does not wrap during the life
/// of a system. It may be read from any context.
pub fn lbolt() -> Ticks {
    // SAFETY: ddi_get_lbolt64(9F) only reads a counter and may be called
    // from any context.
    let t = unsafe { sys::ddi_get_lbolt64() };
    Ticks(t.max(0))
}

/// A reading of the high-resolution clock, in nanoseconds from an arbitrary
/// origin.
///
/// The clock behind `gethrtime(9F)` is monotonic and is not affected by
/// changes to the time of day, which makes it the right clock for measuring
/// how long something took. Subtracting an earlier reading from a later one
/// gives a [`Duration`]. A reading says nothing about the wall-clock time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HrTime(sys::hrtime_t);

impl HrTime {
    /// Reads the clock with `gethrtime(9F)`, which may be called from any
    /// context.
    pub fn now() -> HrTime {
        // SAFETY: gethrtime(9F) has no context restrictions.
        HrTime(unsafe { sys::gethrtime() })
    }

    /// Wraps a raw `hrtime_t`, for example one found in a kernel structure.
    pub const fn from_nanos(nanos: sys::hrtime_t) -> HrTime {
        HrTime(nanos)
    }

    /// Returns the raw `hrtime_t` in nanoseconds.
    pub const fn as_nanos(self) -> sys::hrtime_t {
        self.0
    }

    /// Returns the time that passed between `earlier` and `self`, or zero if
    /// `earlier` is in fact the later reading.
    pub fn duration_since(self, earlier: HrTime) -> Duration {
        let d = self.0.saturating_sub(earlier.0);
        Duration::from_nanos(d.max(0) as u64)
    }

    /// Returns the time that has passed since this reading was taken.
    pub fn elapsed(self) -> Duration {
        HrTime::now().duration_since(self)
    }

    /// Returns the reading `d` after this one, or `None` if it cannot be
    /// represented.
    pub fn checked_add(self, d: Duration) -> Option<HrTime> {
        let n = sys::hrtime_t::try_from(d.as_nanos()).ok()?;
        self.0.checked_add(n).map(HrTime)
    }
}

impl Sub for HrTime {
    type Output = Duration;

    /// Returns the time between the two readings, with the same saturation
    /// at zero as [`HrTime::duration_since`].
    fn sub(self, rhs: HrTime) -> Duration {
        self.duration_since(rhs)
    }
}

/// Sleeps for the given number of ticks with `delay(9F)`.
///
/// The thread gives up the processor, so this needs a [`Sleepable`] token.
/// The manual recommends `cv_timedwait(9F)` instead for threads that wait on
/// behalf of processes with real-time constraints, because `delay` is built
/// on `timeout(9F)` and inherits its exposure to priority inversion. To wait
/// for real time rather than ticks, convert with [`Ticks::from_duration`].
pub fn delay(ticks: Ticks, _ctx: &Sleepable) {
    // SAFETY: the token proves that the caller is in user or kernel context,
    // which is what delay(9F) requires.
    unsafe { sys::delay(ticks.as_raw()) }
}

/// Spins for at least the given time with `drv_usecwait(9F)`.
///
/// The processor is not given up, so this may be called from any context,
/// including interrupt handlers. A fractional microsecond is rounded up. The
/// wait may last longer than requested but never shorter. Because the
/// processor does nothing useful in the meantime, this is meant for waits of
/// a few microseconds, such as giving a device time to act on a register
/// write. Longer waits in thread context should use [`delay`].
pub fn busy_wait(d: Duration) {
    // SAFETY: drv_usecwait(9F) may be called from any context.
    unsafe { sys::drv_usecwait(clock_from_u64(micros_rounded_up(d))) }
}

/// The closure has not been claimed by anyone yet.
const PENDING: u8 = 0;
/// The callback has taken the closure and is running it or has run it.
const FIRED: u8 = 1;
/// The owner has taken the closure back after cancelling the timeout.
const CANCELLED: u8 = 2;

/// The allocation shared between a [`Timeout`] and its callback. The state
/// decides which of the two takes the closure out, so that it is consumed or
/// dropped exactly once. The allocation itself is always freed by the owner.
struct TimeoutInner<F> {
    state: AtomicU8,
    f: UnsafeCell<ManuallyDrop<F>>,
}

/// A closure that the kernel runs once, a number of ticks from now, through
/// `timeout(9F)`.
///
/// The closure runs in the context of a callout, which the manual says must
/// obey the restrictions of a soft interrupt handler, so it receives an
/// [`Atomic`] token and may not block. The usual pattern is for the closure
/// to set a flag under a [`Mutex`](crate::sync::Mutex) and signal a
/// [`CondVar`](crate::sync::CondVar), or to hand longer work to a
/// [`TaskQueue`](crate::taskq::TaskQueue) with
/// [`try_dispatch`](crate::taskq::TaskQueue::try_dispatch).
///
/// Dropping a `Timeout` cancels it with `untimeout(9F)`. If the closure has
/// not started, it is dropped without running. If it is running on another
/// processor, the drop waits for it to finish. That wait is why a `Timeout`
/// must never be dropped while holding a lock that the closure takes; see
/// the [module documentation](self) for the details and a way to structure
/// the code. Dropping a `Timeout` from inside its own closure is allowed;
/// the kernel recognises the case and does not wait.
///
/// Like every callback that the kernel calls by address, the closure's code
/// must still be loaded when the timeout fires, so a `Timeout` must not be
/// leaked with `core::mem::forget` in a module that may be unloaded.
pub struct Timeout<F: FnOnce(&Atomic) + Send + 'static> {
    id: sys::timeout_id_t,
    inner: NonNull<TimeoutInner<F>>,
}

// SAFETY: the closure is `Send`, and nothing else in a `Timeout` is tied to
// a thread; the timeout identifier may be passed to untimeout(9F) from any
// thread.
unsafe impl<F: FnOnce(&Atomic) + Send + 'static> Send for Timeout<F> {}
// SAFETY: a shared reference only allows reading the atomic state through
// `has_fired`. The closure is never reachable through `&Timeout`, so it
// does not have to be `Sync`.
unsafe impl<F: FnOnce(&Atomic) + Send + 'static> Sync for Timeout<F> {}

impl<F: FnOnce(&Atomic) + Send + 'static> Timeout<F> {
    /// Schedules `f` to run once after `after` ticks, using `timeout(9F)`.
    ///
    /// The closure is placed in memory allocated with `KM_SLEEP`, which is
    /// why a [`Sleepable`] token is required. Use [`Timeout::try_new`] from
    /// interrupt context. The interval is approximate; the kernel runs the
    /// closure close to, but not exactly at, the requested time.
    pub fn new(after: Ticks, f: F, ctx: &Sleepable) -> Timeout<F> {
        let inner =
            KBox::new(TimeoutInner { state: AtomicU8::new(PENDING), f: UnsafeCell::new(ManuallyDrop::new(f)) }, ctx);
        Self::arm(inner, after)
    }

    /// Schedules `f` like [`Timeout::new`], but allocates with `KM_NOSLEEP`
    /// so that it may be called from any context, including interrupt
    /// handlers. If memory is not available right away, the closure is
    /// handed back in `Err`.
    pub fn try_new(after: Ticks, f: F) -> Result<Timeout<F>, F> {
        match KBox::try_new_or_return(TimeoutInner {
            state: AtomicU8::new(PENDING),
            f: UnsafeCell::new(ManuallyDrop::new(f)),
        }) {
            Ok(inner) => Ok(Self::arm(inner, after)),
            Err(inner) => Err(ManuallyDrop::into_inner(inner.f.into_inner())),
        }
    }

    fn arm(inner: KBox<TimeoutInner<F>>, after: Ticks) -> Timeout<F> {
        // The pointer stays valid until `stop` reconstitutes the box, which
        // happens only after untimeout(9F) has returned.
        let inner = KBox::into_raw(inner);
        // SAFETY: timeout(9F) may be called from any context. The trampoline
        // matches `F`, and `inner` stays allocated until the callback can no
        // longer run.
        let id = unsafe { sys::timeout(timeout_trampoline::<F>, inner.cast(), after.as_raw()) };
        // SAFETY: `into_raw` never returns a null pointer.
        Timeout { id, inner: unsafe { NonNull::new_unchecked(inner) } }
    }

    /// Reports whether the kernel has started the closure. Once this returns
    /// `true`, the closure has run or is running, and dropping the `Timeout`
    /// no longer prevents anything.
    pub fn has_fired(&self) -> bool {
        // SAFETY: `inner` is valid for as long as `self` exists.
        let inner = unsafe { self.inner.as_ref() };
        inner.state.load(Ordering::Acquire) == FIRED
    }

    /// Cancels the timeout with `untimeout(9F)` and returns the closure if it
    /// had not started. `None` means the closure ran or is running on the
    /// current thread. The rule about locks from the
    /// [module documentation](self) applies, because this waits for a
    /// closure running on another processor exactly like dropping does.
    pub fn cancel(self) -> Option<F> {
        let mut this = ManuallyDrop::new(self);
        // SAFETY: `this` is never used or dropped again.
        unsafe { this.stop() }
    }

    /// Cancels the kernel's request, frees the allocation, and returns the
    /// closure if the callback never claimed it.
    ///
    /// # Safety
    /// Must be called exactly once, after which `self` must not be used.
    unsafe fn stop(&mut self) -> Option<F> {
        // untimeout(9F) does not return until the callback has been
        // cancelled or has finished running. The one exception is a call
        // from inside the callback itself, and in that case the callback has
        // already taken the closure and no longer touches the allocation.
        // SAFETY: `id` came from timeout(9F). Calling untimeout for a
        // timeout that has already run is harmless according to the manual.
        unsafe { sys::untimeout(self.id) };
        // SAFETY: the pointer came from `KBox::into_raw` in `arm`, and after
        // untimeout returned the kernel holds no further reference to it.
        let inner = unsafe { KBox::from_raw(self.inner.as_ptr()) };
        let f = if inner.state.compare_exchange(PENDING, CANCELLED, Ordering::AcqRel, Ordering::Acquire).is_ok() {
            // SAFETY: the state moved from PENDING, so the callback never
            // took the closure, and it can no longer run.
            Some(unsafe { ManuallyDrop::take(&mut *inner.f.get()) })
        } else {
            None
        };
        // The box only frees the memory, because the closure sits in a
        // `ManuallyDrop`.
        drop(inner);
        f
    }
}

impl<F: FnOnce(&Atomic) + Send + 'static> Drop for Timeout<F> {
    /// Cancels the timeout with `untimeout(9F)`, waiting for a closure that
    /// is running on another processor, and drops the closure if it never
    /// ran. Do not drop a `Timeout` while holding a lock that its closure
    /// takes.
    fn drop(&mut self) {
        // SAFETY: `drop` runs once and `self` is not used afterwards.
        drop(unsafe { self.stop() });
    }
}

/// The function `timeout(9F)` calls. It claims the closure, moves it out of
/// the allocation, and runs it with an [`Atomic`] token because the manual
/// gives timeout callbacks the restrictions of a soft interrupt handler.
unsafe extern "C" fn timeout_trampoline<F: FnOnce(&Atomic) + Send + 'static>(arg: *mut sys::c_void) {
    let f = {
        // SAFETY: `arg` is the `TimeoutInner<F>` passed to timeout(9F) in
        // `Timeout::arm`. The owner frees it only after untimeout(9F) has
        // returned, which waits for this function, unless the owner is
        // dropped by the closure below. That happens after the last access
        // to the allocation in this block.
        let inner = unsafe { &*arg.cast::<TimeoutInner<F>>() };
        if inner.state.compare_exchange(PENDING, FIRED, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return;
        }
        // SAFETY: the state moved from PENDING to FIRED here, so no one else
        // takes the closure.
        unsafe { ManuallyDrop::take(&mut *inner.f.get()) }
    };
    // SAFETY: this is the soft interrupt glue for timeout(9F) callbacks.
    let ctx = unsafe { Atomic::assume() };
    f(&ctx);
}

/// The resolution `ddi_periodic_add(9F)` works in, 10 milliseconds.
const PERIODIC_RESOLUTION_NS: u64 = 10_000_000;

/// A closure that the kernel runs repeatedly at a fixed interval, through
/// `ddi_periodic_add(9F)` at level `DDI_IPL_0`.
///
/// At level 0 the manual says that the callback is invoked in kernel
/// context. In the implementation it runs on a thread of a system task queue
/// that all periodic callbacks of level 0 share. Blocking is therefore
/// allowed and the closure receives a [`Sleepable`] token, so it may take
/// locks, allocate with `KM_SLEEP` and wait on condition variables. It
/// should still return promptly. A long wait holds up the periodic callbacks
/// of other drivers, and an interval that expires while the closure is still
/// running is skipped rather than queued. The interrupt levels 1 to 10 of
/// `ddi_periodic_add(9F)` are not offered here.
///
/// The kernel never runs the closure concurrently with itself, which is why
/// it may be `FnMut` and need only be `Send`, not `Sync`.
///
/// Dropping a `Periodic` stops it with `ddi_periodic_delete(9F)`, which
/// waits for a running invocation to finish before it returns. After that
/// the closure is dropped. The rule from the [module documentation](self)
/// applies here too. Never drop a `Periodic` while holding a lock the
/// closure takes, and never while the closure is waiting for something the
/// dropping thread would do next. Dropping a `Periodic` from inside its own closure panics
/// the system, and dropping it must happen in user or kernel context.
pub struct Periodic<F: FnMut(&Sleepable) + Send + 'static> {
    handle: sys::ddi_periodic_t,
    f: NonNull<UnsafeCell<F>>,
}

// SAFETY: the closure is `Send`. It is only called by the kernel and dropped
// by the owner after ddi_periodic_delete(9F) has returned, and the handle may
// be deleted from any thread in user or kernel context.
unsafe impl<F: FnMut(&Sleepable) + Send + 'static> Send for Periodic<F> {}
// SAFETY: a shared reference to a `Periodic` gives no access to anything, so
// sharing one between threads cannot cause a data race.
unsafe impl<F: FnMut(&Sleepable) + Send + 'static> Sync for Periodic<F> {}

impl<F: FnMut(&Sleepable) + Send + 'static> Periodic<F> {
    /// Starts calling `f` every `interval`, using `ddi_periodic_add(9F)` at
    /// level `DDI_IPL_0`.
    ///
    /// The manual supports only intervals that are non-zero multiples of 10
    /// milliseconds, so `interval` is rounded up to the next multiple, and
    /// an interval shorter than 10 milliseconds becomes 10 milliseconds. The
    /// interval is a lower bound on the time between two calls. The first
    /// call happens roughly one interval from now.
    ///
    /// `ddi_periodic_add(9F)` may only be called from user or kernel
    /// context, and the closure is allocated with `KM_SLEEP`, which is why a
    /// [`Sleepable`] token is required.
    pub fn new(interval: Duration, f: F, ctx: &Sleepable) -> Periodic<F> {
        let ns = u64::try_from(interval.as_nanos()).unwrap_or(u64::MAX);
        let ns = ns.max(1).div_ceil(PERIODIC_RESOLUTION_NS).saturating_mul(PERIODIC_RESOLUTION_NS);
        let ns = sys::hrtime_t::try_from(ns).unwrap_or(sys::hrtime_t::MAX);
        let f = KBox::into_raw(KBox::new(UnsafeCell::new(f), ctx));
        // SAFETY: the token proves user or kernel context. The trampoline
        // matches `F`, and the closure stays allocated until
        // ddi_periodic_delete(9F) has returned in `drop`.
        let handle = unsafe { sys::ddi_periodic_add(periodic_trampoline::<F>, f.cast(), ns, sys::DDI_IPL_0) };
        // SAFETY: `into_raw` never returns a null pointer.
        Periodic { handle, f: unsafe { NonNull::new_unchecked(f) } }
    }
}

impl<F: FnMut(&Sleepable) + Send + 'static> Drop for Periodic<F> {
    /// Stops the callbacks with `ddi_periodic_delete(9F)`, waiting for a
    /// running invocation to finish, and then drops the closure. Do not drop
    /// a `Periodic` while holding a lock that its closure takes, and never
    /// from inside the closure itself.
    fn drop(&mut self) {
        // SAFETY: the handle came from ddi_periodic_add(9F) and is deleted
        // exactly once. The function blocks until no invocation is running
        // and guarantees that none starts afterwards.
        unsafe { sys::ddi_periodic_delete(self.handle) };
        // SAFETY: the pointer came from `KBox::into_raw` in `new`, and the
        // kernel no longer refers to it.
        drop(unsafe { KBox::from_raw(self.f.as_ptr()) });
    }
}

/// The function `ddi_periodic_add(9F)` calls. Level 0 callbacks run in
/// kernel thread context, so the closure receives a [`Sleepable`] token.
unsafe extern "C" fn periodic_trampoline<F: FnMut(&Sleepable) + Send + 'static>(arg: *mut sys::c_void) {
    // SAFETY: `arg` is the `UnsafeCell<F>` passed to ddi_periodic_add(9F) in
    // `Periodic::new`, which stays allocated until ddi_periodic_delete(9F)
    // has returned. The kernel never runs two invocations of one periodic
    // callback at the same time, and the owner never touches the closure
    // while the callback is registered, so this is the only reference.
    let f = unsafe { &mut *(*arg.cast::<UnsafeCell<F>>()).get() };
    // SAFETY: this is the glue for a DDI_IPL_0 periodic callback, which the
    // manual documents as running in kernel context.
    let ctx = unsafe { Sleepable::assume() };
    f(&ctx);
}
