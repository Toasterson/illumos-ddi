//! Execution-context tokens.
//!
//! `kmem_alloc(9F)`: "KM_SLEEP … the caller must not be in interrupt
//! context". `cv_wait(9F)`, `delay(9F)` and friends have the same rule. The
//! rule becomes a value: entry points that the kernel calls from a thread
//! context receive a [`Sleepable`]; interrupt handlers receive an
//! [`Atomic`]. Neither can be created by a driver, neither is `Send`, so a
//! `Sleepable` cannot be stashed and reused where it is not valid.

use core::marker::PhantomData;

/// Proof that the current context may block (sleep for memory, wait on a
/// condition variable, `delay(9F)`).
pub struct Sleepable(PhantomData<*mut ()>);

/// Proof that the current context is an interrupt handler or otherwise
/// may not block. Only `KM_NOSLEEP` allocation and non-blocking primitives
/// are available.
pub struct Atomic(PhantomData<*mut ()>);

impl Sleepable {
    /// Assert that the caller is in a context where blocking is allowed.
    ///
    /// # Safety
    /// Only the framework glue should call this, at entry points the DDI
    /// documents as running in kernel thread context (`attach(9E)`,
    /// `open(9E)`, `read(9E)`, `ioctl(9E)`, taskq functions, …).
    #[inline]
    pub unsafe fn assume() -> Self {
        Self(PhantomData)
    }
}

impl Atomic {
    /// Assert that the caller is in a non-blocking context.
    ///
    /// # Safety
    /// Only interrupt and softint glue should call this.
    #[inline]
    pub unsafe fn assume() -> Self {
        Self(PhantomData)
    }
}
