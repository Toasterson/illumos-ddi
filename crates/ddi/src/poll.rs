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
//!   head cannot be freed while registered. `pollhead_clean(9F)` may only be
//!   called from user or kernel context, so a `PollHead` must be dropped
//!   there (from `close(9E)`, a task queue, or another thread-context entry
//!   point), never from an interrupt handler.
//! - A registered head must stay alive at least until the file it was
//!   registered for is closed: the poll framework links the poller into the
//!   head after `chpoll(9E)` has returned, while it holds the file open.
//!   Keep the head in the per-file or per-object state that `close(9E)`
//!   frees.
//! - No mutex that `chpoll` takes may be held across [`PollHead::wakeup`].
//!   The types cannot express this; release such locks first.
//! - [`PollHead::wakeup`] may be called from any context, interrupt
//!   handlers included.

use core::cell::UnsafeCell;
use core::marker::PhantomData;

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
    raw: KBox<UnsafeCell<sys::pollhead>>,
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
        PollHead { raw: KBox::new(Self::zeroed(), ctx) }
    }

    /// Allocates a zeroed pollhead with `KM_NOSLEEP`, from any context.
    pub fn try_new() -> Result<PollHead, Errno> {
        Ok(PollHead { raw: KBox::try_new(Self::zeroed())? })
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

impl Drop for PollHead {
    /// Wakes the remaining pollers with `POLLERR` and removes them with
    /// `pollhead_clean(9F)` before the memory is freed, as `chpoll(9E)`
    /// asks. User or kernel context only.
    fn drop(&mut self) {
        // SAFETY: the pollhead is live; after pollhead_clean the kernel
        // holds no reference to it and the box frees it.
        unsafe {
            sys::pollwakeup(self.raw.get(), sys::POLLERR);
            sys::pollhead_clean(self.raw.get());
        }
    }
}

/// Where [`CharDevice::chpoll`](crate::CharDevice::chpoll) registers its
/// [`PollHead`]; the glue passes it to the kernel through `phpp` when the
/// algorithm of `chpoll(9E)` needs one.
pub struct PollRegistration<'a> {
    head: *mut sys::pollhead,
    _call: PhantomData<&'a mut ()>,
}

impl PollRegistration<'_> {
    pub(crate) fn new() -> Self {
        PollRegistration { head: core::ptr::null_mut(), _call: PhantomData }
    }

    /// Registers `head`. The head must stay alive until the polled file is
    /// closed (see the [module documentation](self)); its `Drop` cleans it
    /// in any case.
    pub fn register(&mut self, head: &PollHead) {
        self.head = head.as_raw();
    }

    pub(crate) fn raw(&self) -> *mut sys::pollhead {
        self.head
    }
}
