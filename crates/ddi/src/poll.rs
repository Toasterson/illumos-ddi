//! Polling a character device: `chpoll(9E)`, `pollwakeup(9F)` and
//! `pollhead_clean(9F)`.
//!
//! A driver that lets `poll(2)` wait on its minor devices keeps a
//! [`PollHead`] for each pollable object, answers
//! [`CharDevice::chpoll`](crate::CharDevice::chpoll) with the events that
//! are ready and, when the manual's algorithm asks for it, registers the
//! head in the [`PollRegistration`] the glue passes in; it calls
//! [`PollHead::wakeup`] whenever one of those events becomes ready. The
//! module is declared with `ops: [char, chpoll]` or
//! `ops: [char, devmap, chpoll]`.
//!
//! The rules of `chpoll(9E)` and their shape here:
//!
//! - The `pollhead` must be zero-initialised and must not move while
//!   pollers are registered on it. [`PollHead`] keeps it in its own
//!   [`KBox`], so the handle may move freely.
//! - Before its memory goes, the driver wakes the pollers with `POLLERR`
//!   and calls `pollhead_clean(9F)`. [`PollHead`]'s `Drop` does both, so a
//!   head cannot be freed while registered.
//! - `pollhead_clean(9F)` may only be called from user or kernel context.
//!   [`PollHead::release`] takes the [`Sleepable`] token; an implicit drop
//!   in interrupt context is refused at run time where the private tier is
//!   built in (`servicing_interrupt`): the head is leaked, with a warning,
//!   rather than cleaned there ([`leaked_heads`] counts them).
//! - The poll framework links the poller into the registered head after
//!   `chpoll(9E)` has returned. The head must therefore outlive the call by
//!   more than the call: [`CharDevice::chpoll`](crate::CharDevice::chpoll)
//!   receives the instance as `&'s self` and a `PollRegistration<'s>`, and
//!   [`PollRegistration::register`] only accepts a head borrowed for `'s`,
//!   that is one the instance owns, which lives until detach, after every
//!   file is closed. A head owned by something shorter-lived (a per-file or
//!   per-object head) needs the `unsafe`
//!   [`PollRegistration::register_unchecked`], whose caller guarantees that
//!   the head is not dropped before the polled file's `close(9E)`. Nothing
//!   at run time can see the window in the poll framework, so there is no
//!   guard for that path (`docs/DESIGN.md` section 5.12).
//! - No mutex that `chpoll` takes may be held across [`PollHead::wakeup`].
//!   The types cannot express this; release such locks first.
//! - [`PollHead::wakeup`] may be called from any context, interrupt
//!   handlers included.
//!
//! A head that is not borrowed from the instance cannot be registered in
//! safe code:
//!
//! ```compile_fail,E0597
//! fn chpoll_local<'s>(reg: &mut ddi::PollRegistration<'s>, ctx: &ddi::Sleepable) {
//!     let head = ddi::PollHead::new(ctx);
//!     reg.register(&head);
//! }
//! ```
//!
//! A head borrowed for the registration's lifetime can:
//!
//! ```no_run
//! fn chpoll_owned<'s>(head: &'s ddi::PollHead, reg: &mut ddi::PollRegistration<'s>) {
//!     reg.register(head);
//! }
//! ```

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::context::Sleepable;
use crate::error::Errno;
use crate::kmem::KBox;
use crate::sys;

/// A set of `poll(2)` event bits, as `chpoll(9E)` receives and returns
/// them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PollEvents(pub i16);

impl PollEvents {
    /// No events.
    pub const NONE: PollEvents = PollEvents(0);
    /// `POLLIN`: readable.
    pub const IN: PollEvents = PollEvents(sys::POLLIN);
    /// `POLLPRI`: high-priority data.
    pub const PRI: PollEvents = PollEvents(sys::POLLPRI);
    /// `POLLOUT`: writable.
    pub const OUT: PollEvents = PollEvents(sys::POLLOUT);
    /// `POLLRDNORM`: normal data readable.
    pub const RDNORM: PollEvents = PollEvents(sys::POLLRDNORM);
    /// `POLLRDBAND`: priority band data readable.
    pub const RDBAND: PollEvents = PollEvents(sys::POLLRDBAND);
    /// `POLLWRBAND`: priority band data writable.
    pub const WRBAND: PollEvents = PollEvents(sys::POLLWRBAND);
    /// `POLLERR`: an error; reported, never requested.
    pub const ERR: PollEvents = PollEvents(sys::POLLERR);
    /// `POLLHUP`: hung up; reported, never requested.
    pub const HUP: PollEvents = PollEvents(sys::POLLHUP);
    /// `POLLNVAL`: the descriptor is invalid; reported, never requested.
    pub const NVAL: PollEvents = PollEvents(sys::POLLNVAL);
    /// `POLLET`: edge-triggered polling was requested; the head must always
    /// be registered.
    pub const ET: PollEvents = PollEvents(sys::POLLET);

    /// Combines two sets.
    pub const fn union(self, other: PollEvents) -> PollEvents {
        PollEvents(self.0 | other.0)
    }
    /// The events in both sets.
    pub const fn intersection(self, other: PollEvents) -> PollEvents {
        PollEvents(self.0 & other.0)
    }
    /// Reports whether any event of `other` is in the set.
    pub const fn intersects(self, other: PollEvents) -> bool {
        self.0 & other.0 != 0
    }
    /// Reports whether the set is empty.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// A `struct pollhead` in memory of its own, cleaned before it is freed.
/// See the [module documentation](self).
pub struct PollHead {
    raw: ManuallyDrop<KBox<UnsafeCell<sys::pollhead>>>,
}

// SAFETY: the pollhead is only handed to pollwakeup(9F) and
// pollhead_clean(9F), which lock the kernel's poll structures themselves
// and may be called from any thread.
unsafe impl Send for PollHead {}
// SAFETY: as above; a shared PollHead only allows pollwakeup and taking
// its address for the poll framework.
unsafe impl Sync for PollHead {}

impl PollHead {
    fn zeroed() -> UnsafeCell<sys::pollhead> {
        UnsafeCell::new(sys::pollhead { ph_list: core::ptr::null_mut(), ph_pad1: core::ptr::null_mut(), ph_pad2: 0 })
    }

    /// Allocates a zeroed pollhead with `KM_SLEEP`.
    pub fn new(ctx: &Sleepable) -> PollHead {
        PollHead { raw: ManuallyDrop::new(KBox::new(Self::zeroed(), ctx)) }
    }

    /// Allocates a zeroed pollhead with `KM_NOSLEEP`, from any context.
    pub fn try_new() -> Result<PollHead, Errno> {
        Ok(PollHead { raw: ManuallyDrop::new(KBox::try_new(Self::zeroed())?) })
    }

    /// Wakes the remaining pollers with `POLLERR` and removes them with
    /// `pollhead_clean(9F)`, then frees the head, in thread context (the
    /// token). This is what dropping does; the token makes the context a
    /// type.
    pub fn release(self, _ctx: &Sleepable) {
        drop(self)
    }

    /// Wakes the pollers waiting for any of `events` with `pollwakeup(9F)`.
    /// Any context; no mutex that `chpoll(9E)` takes may be held.
    pub fn wakeup(&self, events: PollEvents) {
        // SAFETY: a live, zero-initialised pollhead that does not move.
        unsafe { sys::pollwakeup(self.raw.get(), events.0) }
    }

    fn as_raw(&self) -> *mut sys::pollhead {
        self.raw.get()
    }
}

/// Heads dropped in interrupt context and leaked rather than cleaned.
static LEAKED: AtomicU64 = AtomicU64::new(0);

/// Returns how many [`PollHead`]s were dropped in interrupt context and
/// leaked instead of being cleaned there. Always 0 without the private
/// tier, which provides the check.
pub fn leaked_heads() -> u64 {
    LEAKED.load(Ordering::Relaxed)
}

/// Reports whether the caller runs in interrupt context, where the private
/// tier can tell; `false` otherwise.
fn in_interrupt() -> bool {
    #[cfg(feature = "unstable-gate-cda5b9a49f")]
    {
        // SAFETY: servicing_interrupt may be called from any context.
        unsafe { illumos_kernel_private_sys::disp::servicing_interrupt() != 0 }
    }
    #[cfg(not(feature = "unstable-gate-cda5b9a49f"))]
    {
        false
    }
}

impl Drop for PollHead {
    /// Wakes the remaining pollers with `POLLERR` and removes them with
    /// `pollhead_clean(9F)` before the memory is freed, as `chpoll(9E)`
    /// asks. User or kernel context only; in interrupt context (detected
    /// with the private tier) the head is leaked with a warning instead.
    fn drop(&mut self) {
        if in_interrupt() {
            LEAKED.fetch_add(1, Ordering::Relaxed);
            crate::kwarn!("ddi::poll: a PollHead was dropped in interrupt context; leaking it");
            // The box is not dropped: the pollhead stays valid for pollers.
            return;
        }
        // SAFETY: the pollhead is live; after pollhead_clean the kernel
        // holds no reference to it, and the box is dropped once, here.
        unsafe {
            sys::pollwakeup(self.raw.get(), sys::POLLERR);
            sys::pollhead_clean(self.raw.get());
            ManuallyDrop::drop(&mut self.raw);
        }
    }
}

/// Where [`CharDevice::chpoll`](crate::CharDevice::chpoll) registers its
/// [`PollHead`]; the glue passes it to the kernel through `phpp` when the
/// algorithm of `chpoll(9E)` needs one. `'s` is the lifetime of the
/// instance the call borrows; see the [module documentation](self).
pub struct PollRegistration<'s> {
    head: *mut sys::pollhead,
    _heads: PhantomData<&'s PollHead>,
}

impl<'s> PollRegistration<'s> {
    pub(crate) fn new() -> Self {
        PollRegistration { head: core::ptr::null_mut(), _heads: PhantomData }
    }

    /// Registers `head`, which the instance owns: it lives for `'s`, until
    /// the device detaches, after every file is closed.
    pub fn register(&mut self, head: &'s PollHead) {
        self.head = head.as_raw();
    }

    /// Registers a head that does not live as long as the instance.
    ///
    /// # Safety
    /// The head must not be dropped before `close(9E)` of the file being
    /// polled: the poll framework links the poller into it after
    /// `chpoll(9E)` has returned, holding the file open meanwhile. A head
    /// kept in per-object state that only `close(9E)` of the object's own
    /// minor frees satisfies this.
    pub unsafe fn register_unchecked(&mut self, head: &PollHead) {
        self.head = head.as_raw();
    }

    pub(crate) fn raw(&self) -> *mut sys::pollhead {
        self.head
    }
}
