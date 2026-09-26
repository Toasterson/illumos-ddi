//! Task queues, the `ddi_taskq_*(9F)` family.
//!
//! A task queue is a small pool of kernel threads that run work on behalf of
//! a driver. Its most common use is to move work out of a context that may
//! not block, such as an interrupt handler or a [`Timeout`] closure, into
//! thread context, where it may. A driver creates a [`TaskQueue`] in
//! `attach(9E)`, keeps it in its instance state, and hands it closures.
//!
//! Every task is a closure that the queue owns until it runs. The closure is
//! moved into kernel memory when it is dispatched and is called exactly once
//! on one of the queue's threads, with a [`Sleepable`] token, because task
//! queue threads are ordinary kernel threads that may block. With a single
//! thread, tasks run in the order they were dispatched. With more than one
//! thread they may run in any order and at the same time, so tasks should
//! not depend on each other; a task that waits for a later task can
//! deadlock the queue.
//!
//! There are two ways to dispatch. [`TaskQueue::dispatch`] may wait for
//! memory and therefore needs a [`Sleepable`] token; it cannot fail for lack
//! of memory. [`TaskQueue::try_dispatch`] never blocks, may be called from
//! any context including interrupt handlers, and hands the closure back when
//! no memory is available right away.
//!
//! Dropping a `TaskQueue` destroys it with `ddi_taskq_destroy(9F)`, which
//! first waits for every task that has been dispatched. Because tasks are
//! `'static`, none of them can borrow from the instance that owns the queue.
//!
//! # Rules the compiler does not check
//!
//! Three operations wait for tasks, and a task that performs one of them on
//! its own queue waits for itself. A task must therefore never call
//! [`TaskQueue::wait`], [`TaskQueue::suspend`] or drop the `TaskQueue` that
//! it is running on. For the same reason, a thread must not wait for or drop
//! the queue while holding a lock that a pending task needs. Dropping the
//! queue must happen in user or kernel context, as the manual requires of
//! every function in the family.
//!
//! [`Timeout`]: crate::time::Timeout

use core::ffi::CStr;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::num::NonZeroU32;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::context::Sleepable;
use crate::device::DevInfo;
use crate::error::DdiError;
use crate::kmem::KBox;
use crate::sys;

/// A task queue created with `ddi_taskq_create(9F)` and destroyed with
/// `ddi_taskq_destroy(9F)` when dropped.
///
/// The queue is created on behalf of a device instance. The kernel names it
/// after the driver and the name given here and records the instance
/// number, so the queue can be told apart from those of other instances in
/// tools such as `mdb` and `kstat`.
pub struct TaskQueue {
    raw: NonNull<sys::ddi_taskq_t>,
    suspended: AtomicBool,
}

// SAFETY: a task queue is a kernel object designed to be used from many
// threads at once. The kernel serialises dispatch, wait and destroy
// internally, and suspension, whose resume must happen on the suspending
// thread, is guarded by the `suspended` flag and the thread-bound
// `Suspended` guard. The tasks themselves are required to be `Send`.
unsafe impl Send for TaskQueue {}
// SAFETY: see the `Send` implementation. Every method that takes `&self`
// is safe to call from several threads at the same time.
unsafe impl Sync for TaskQueue {}

impl TaskQueue {
    /// Creates a task queue with `threads` threads at the default priority,
    /// using `ddi_taskq_create(9F)` with `TASKQ_DEFAULTPRI`, which the manual
    /// recommends for drivers.
    ///
    /// The kernel builds the full name from the driver name and `name`. The
    /// manual allows only letters and digits in `name`; the kernel replaces
    /// other characters and shortens names that do not fit in 31 bytes. With
    /// one thread, tasks run in the order they were dispatched.
    pub fn new(dev: DevInfo, name: &CStr, threads: NonZeroU32, ctx: &Sleepable) -> Result<TaskQueue, DdiError> {
        Self::with_priority(dev, name, threads, sys::TASKQ_DEFAULTPRI, ctx)
    }

    /// Creates a task queue like [`TaskQueue::new`], but with the given
    /// thread priority instead of `TASKQ_DEFAULTPRI`. The manual asks
    /// drivers to use the default, so this is for the rare driver with a
    /// documented reason not to.
    pub fn with_priority(
        dev: DevInfo,
        name: &CStr,
        threads: NonZeroU32,
        priority: sys::pri_t,
        _ctx: &Sleepable,
    ) -> Result<TaskQueue, DdiError> {
        let threads = i32::try_from(threads.get()).unwrap_or(i32::MAX);
        // SAFETY: the token proves user or kernel context. The node is live
        // because the caller holds a `DevInfo`, the name is a valid C string
        // that the kernel copies, and `cflags` must be 0.
        let raw = unsafe { sys::ddi_taskq_create(dev.as_raw(), name.as_ptr(), threads, priority, 0) };
        let raw = NonNull::new(raw).ok_or(DdiError)?;
        Ok(TaskQueue { raw, suspended: AtomicBool::new(false) })
    }

    fn as_raw(&self) -> *mut sys::ddi_taskq_t {
        self.raw.as_ptr()
    }

    /// Queues `f` to run on one of the queue's threads, using
    /// `ddi_taskq_dispatch(9F)` with `DDI_SLEEP`.
    ///
    /// The closure is moved into memory allocated with `KM_SLEEP`, and the
    /// dispatch may wait for a task entry, which is why a [`Sleepable`] token
    /// is required. The manual guarantees that a `DDI_SLEEP` dispatch
    /// succeeds. Should the kernel report a failure anyway, the closure is
    /// dropped without running and `DdiError` is returned.
    pub fn dispatch<F: FnOnce(&Sleepable) + Send + 'static>(&self, f: F, ctx: &Sleepable) -> Result<(), DdiError> {
        let task = KBox::new(ManuallyDrop::new(f), ctx);
        let arg = KBox::into_raw(task);
        // SAFETY: the token proves user or kernel context. The trampoline
        // matches `F`, and it takes ownership of `arg` when it runs.
        let r = unsafe { sys::ddi_taskq_dispatch(self.as_raw(), task_trampoline::<F>, arg.cast(), sys::DDI_SLEEP) };
        DdiError::check(r).inspect_err(|_| {
            // SAFETY: the dispatch failed, so the kernel never saw `arg` and
            // it is still owned here.
            let mut task = unsafe { KBox::from_raw(arg) };
            // SAFETY: the closure is taken out exactly once, here.
            drop(unsafe { ManuallyDrop::take(&mut *task) });
        })
    }

    /// Queues `f` without blocking, using `ddi_taskq_dispatch(9F)` with
    /// `DDI_NOSLEEP`.
    ///
    /// The closure is moved into memory allocated with `KM_NOSLEEP`. Neither
    /// the allocation nor the dispatch waits, so this may be called from any
    /// context, including interrupt handlers and [`Timeout`] closures. When
    /// either one cannot get memory right away, the closure is handed back in
    /// `Err` so the caller can retry later or do the work some other way.
    ///
    /// [`Timeout`]: crate::time::Timeout
    pub fn try_dispatch<F: FnOnce(&Sleepable) + Send + 'static>(&self, f: F) -> Result<(), F> {
        let task = KBox::try_new_or_return(ManuallyDrop::new(f)).map_err(ManuallyDrop::into_inner)?;
        let arg = KBox::into_raw(task);
        // SAFETY: a DDI_NOSLEEP dispatch may be made from any context. The
        // trampoline matches `F`, and it takes ownership of `arg` when it
        // runs.
        let r = unsafe { sys::ddi_taskq_dispatch(self.as_raw(), task_trampoline::<F>, arg.cast(), sys::DDI_NOSLEEP) };
        if r == sys::DDI_SUCCESS {
            return Ok(());
        }
        // SAFETY: the dispatch failed, so the kernel never saw `arg` and it
        // is still owned here.
        let mut task = unsafe { KBox::from_raw(arg) };
        // SAFETY: the closure is taken out exactly once, here, and the box
        // then only frees the memory.
        Err(unsafe { ManuallyDrop::take(&mut *task) })
    }

    /// Waits until every task dispatched so far has finished, using
    /// `ddi_taskq_wait(9F)`. Tasks dispatched while waiting are not held
    /// back. A task must never call this on its own queue, because it would
    /// wait for itself. A thread that holds a [`Suspended`] guard for the
    /// queue must not call it either, because the pending tasks it waits for
    /// cannot start until the guard is dropped.
    pub fn wait(&self, _ctx: &Sleepable) {
        // SAFETY: the token proves user or kernel context, and the queue is
        // live for as long as `self` is.
        unsafe { sys::ddi_taskq_wait(self.as_raw()) }
    }

    /// Suspends the queue with `ddi_taskq_suspend(9F)` and returns a guard
    /// that resumes it with `ddi_taskq_resume(9F)` when dropped.
    ///
    /// While the guard exists, tasks dispatched after the call do not start.
    /// The manual does not promise that tasks already pending are held back,
    /// and tasks that are running finish normally. The call waits for
    /// running tasks, so a task must never suspend its own queue.
    ///
    /// The kernel keeps a lock held from the suspend until the resume, which
    /// has two consequences that the types enforce. The resume has to
    /// happen on the thread that suspended the queue, so the guard can
    /// neither be sent to nor shared with another thread. The queue cannot
    /// be suspended twice, so this returns `None` if it is already
    /// suspended. The guard also borrows the queue, which prevents the
    /// queue from being dropped while suspended, when destroying it would
    /// wait forever for tasks that cannot start.
    pub fn suspend(&self, _ctx: &Sleepable) -> Option<Suspended<'_>> {
        if self.suspended.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            return None;
        }
        // SAFETY: the token proves user or kernel context, and the flag
        // guarantees that no other suspension is in effect.
        unsafe { sys::ddi_taskq_suspend(self.as_raw()) };
        Some(Suspended { tq: self, _thread: PhantomData })
    }

    /// Reports whether the queue is suspended, using
    /// `ddi_taskq_suspended(9F)`. The manual intends it for assertions.
    pub fn is_suspended(&self, _ctx: &Sleepable) -> bool {
        // SAFETY: the token proves user or kernel context, and the queue is
        // live for as long as `self` is.
        unsafe { sys::ddi_taskq_suspended(self.as_raw()) != sys::B_FALSE }
    }
}

impl Drop for TaskQueue {
    /// Destroys the queue with `ddi_taskq_destroy(9F)`, which first waits
    /// for every dispatched task to finish. It must not be dropped by one of
    /// its own tasks, nor while holding a lock that a pending task needs.
    fn drop(&mut self) {
        // SAFETY: the queue came from ddi_taskq_create(9F) and is destroyed
        // exactly once. Ownership by value means no dispatch through this
        // queue can happen concurrently, and a `Suspended` guard cannot
        // outlive the borrow of the queue, so the queue is not suspended.
        unsafe { sys::ddi_taskq_destroy(self.as_raw()) }
    }
}

/// Proof that a [`TaskQueue`] is suspended. Dropping it resumes the queue
/// with `ddi_taskq_resume(9F)`.
///
/// The guard stays on the thread that created it, because the kernel
/// requires the resume to come from the thread that suspended the queue.
/// That thread was in user or kernel context when it suspended the queue,
/// which is also what the resume requires.
pub struct Suspended<'a> {
    tq: &'a TaskQueue,
    _thread: PhantomData<*mut ()>,
}

impl Suspended<'_> {
    /// Resumes the queue. This is the same as dropping the guard, spelled
    /// out for readability at the call site.
    pub fn resume(self) {
        drop(self);
    }
}

impl Drop for Suspended<'_> {
    /// Resumes the queue with `ddi_taskq_resume(9F)`.
    fn drop(&mut self) {
        // SAFETY: this guard exists only after a successful suspend on this
        // thread, and it is dropped exactly once.
        unsafe { sys::ddi_taskq_resume(self.tq.as_raw()) };
        self.tq.suspended.store(false, Ordering::Release);
    }
}

/// The function a task queue thread calls. It takes ownership of the boxed
/// closure, frees the box, and runs the closure with a [`Sleepable`] token,
/// because task queue threads are kernel threads that may block.
unsafe extern "C" fn task_trampoline<F: FnOnce(&Sleepable) + Send + 'static>(arg: *mut sys::c_void) {
    // SAFETY: `arg` came from `KBox::into_raw` in `dispatch` or
    // `try_dispatch`, and the kernel calls this function exactly once per
    // successful dispatch.
    let mut task = unsafe { KBox::from_raw(arg.cast::<ManuallyDrop<F>>()) };
    // SAFETY: the closure is taken out exactly once, here, and the box then
    // only frees the memory.
    let f = unsafe { ManuallyDrop::take(&mut *task) };
    drop(task);
    // SAFETY: this is the task queue glue, and task queue functions run in
    // kernel thread context, which the documentation of `Sleepable::assume`
    // names as one of the permitted places.
    let ctx = unsafe { Sleepable::assume() };
    f(&ctx);
}
