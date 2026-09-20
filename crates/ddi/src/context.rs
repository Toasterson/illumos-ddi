//! Tokens that describe the execution context.
//!
//! The manual page for `kmem_alloc(9F)` says that a `KM_SLEEP` allocation
//! must not be made from interrupt context, and `cv_wait(9F)`, `delay(9F)`
//! and the other blocking functions carry the same rule. In C this is a
//! matter of discipline. Here the rule is a value. Entry points that the
//! kernel calls from thread context receive a [`Sleepable`], and interrupt
//! handlers receive an [`Atomic`]. A driver cannot construct either token,
//! and neither is `Send`, so a `Sleepable` obtained in `attach(9E)` cannot
//! be stored away and reused inside an interrupt handler.

use core::marker::PhantomData;

/// Proof that the current context may block. Blocking includes sleeping for
/// memory, waiting on a condition variable and calling `delay(9F)`.
pub struct Sleepable(PhantomData<*mut ()>);

/// Proof that the current context is an interrupt handler or some other
/// context that may not block. Only `KM_NOSLEEP` allocation and the
/// non-blocking primitives are available with this token.
pub struct Atomic(PhantomData<*mut ()>);

impl Sleepable {
    /// Asserts that the caller is in a context where blocking is allowed.
    ///
    /// # Safety
    /// Only the framework glue should call this, and only at entry points the
    /// DDI documents as running in kernel thread context, such as
    /// `attach(9E)`, `open(9E)`, `read(9E)`, `ioctl(9E)` and task queue
    /// functions. A driver that calls it has defeated the purpose of the
    /// token.
    #[inline]
    pub unsafe fn assume() -> Self {
        Self(PhantomData)
    }
}

impl Atomic {
    /// Asserts that the caller is in a context that may not block.
    ///
    /// # Safety
    /// Only the interrupt and soft interrupt glue should call this.
    #[inline]
    pub unsafe fn assume() -> Self {
        Self(PhantomData)
    }
}
