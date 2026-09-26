//! Polling support from `sys/poll.h`.
//!
//! A driver that implements `chpoll(9E)` keeps one `struct pollhead` per
//! pollable minor device inside its own state, hands its address back to
//! the kernel from `chpoll`, and calls `pollwakeup(9F)` on it whenever one
//! of the events it reported as not yet ready becomes ready. Before the
//! state that holds the `pollhead` is freed, the driver calls
//! `pollhead_clean(9F)` so that the kernel drops any references it still
//! holds. The `POLL*` constants are the event bits that `chpoll` receives
//! in its `events` argument and returns through `reventsp`.
//!
//! `sys/poll.h` defines `struct pollhead` under `_KERNEL`, right after it
//! includes `sys/poll_impl.h`. The list element it points to, `struct
//! polldat`, is defined only in `sys/poll_impl.h` and stays opaque here.
//! Because `chpoll(9E)` requires drivers to embed a `pollhead` by value,
//! its layout is public by necessity even though it borders on the private
//! implementation; the header itself says that the size cannot change for
//! DDI compliance. This is the one place where the crate takes a layout
//! from the edge of an `_impl` header, and the size is asserted below.

use crate::types::*;

/// The file descriptor is readable (`POLLIN`).
pub const POLLIN: c_short = 0x0001;
/// High priority data is available on the file descriptor (`POLLPRI`).
pub const POLLPRI: c_short = 0x0002;
/// The file descriptor is writable without blocking (`POLLOUT`).
pub const POLLOUT: c_short = 0x0004;
/// Normal data is readable (`POLLRDNORM`).
pub const POLLRDNORM: c_short = 0x0040;
/// Normal data is writable; the same bit as [`POLLOUT`] (`POLLWRNORM`).
pub const POLLWRNORM: c_short = POLLOUT;
/// Out-of-band data is readable (`POLLRDBAND`).
pub const POLLRDBAND: c_short = 0x0080;
/// Out-of-band data is writable (`POLLWRBAND`).
pub const POLLWRBAND: c_short = 0x0100;
/// The read side of the connection has hung up (`POLLRDHUP`).
pub const POLLRDHUP: c_short = 0x4000;
/// An error condition exists on the file descriptor. It is never requested
/// in `events` and only ever reported in `revents` (`POLLERR`).
pub const POLLERR: c_short = 0x0008;
/// The file descriptor has been hung up. Reported in `revents` only
/// (`POLLHUP`).
pub const POLLHUP: c_short = 0x0010;
/// The `pollfd` entry was invalid. Reported in `revents` only (`POLLNVAL`).
pub const POLLNVAL: c_short = 0x0020;
/// The file descriptor is polled in edge-triggered mode through `/dev/poll`
/// or `epoll(7)`. It is only ever set in `events`, never in `revents`, and
/// tells a `chpoll(9E)` implementation that it must always register its
/// `pollhead` (`POLLET`).
pub const POLLET: c_short = 0x2000;

opaque! {
    /// One registration of a poller on a `pollhead`, `struct polldat`. It is
    /// defined in `sys/poll_impl.h` and drivers only ever see the pointer
    /// at the head of the list.
    polldat;
}

/// The head of the list of threads polling a device, `struct pollhead`.
///
/// A driver embeds one of these in its per-minor state, initialised to
/// zero, and never touches the fields; it passes the address to the kernel
/// from `chpoll(9E)` and to [`pollwakeup`] and [`pollhead_clean`]. The
/// structure is defined in `sys/poll.h` under `_KERNEL`, next to the
/// private `sys/poll_impl.h` machinery it points into. The header says only
/// `ph_list` is used, and that the two padding fields exist because the size
/// is part of the DDI and cannot change. The layout is public here only
/// because `chpoll(9E)` obliges drivers to embed the structure.
#[repr(C)]
pub struct pollhead {
    /// The list of pollers registered on this head. Owned by the kernel.
    pub ph_list: *mut polldat,
    /// Unused padding kept for binary compatibility.
    pub ph_pad1: *mut c_void,
    /// Unused padding kept for binary compatibility.
    pub ph_pad2: c_short,
}
/// The typedef name of [`pollhead`].
pub type pollhead_t = pollhead;

// Two pointers (16 bytes) followed by a short, padded to the 8-byte
// alignment of the structure, giving 24 bytes.
const _: () = assert!(core::mem::size_of::<pollhead>() == 24);
const _: () = assert!(core::mem::offset_of!(pollhead, ph_list) == 0);
const _: () = assert!(core::mem::offset_of!(pollhead, ph_pad1) == 8);
const _: () = assert!(core::mem::offset_of!(pollhead, ph_pad2) == 16);

unsafe extern "C" {
    /// `pollwakeup(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Wakes the threads waiting on `php` for the event `event`, which is
    /// one of the `POLL*` bits. A driver calls it for each occurrence of an
    /// event on the minor device the `pollhead` belongs to; the `pollhead`
    /// is usually part of the driver's private state for that minor device.
    /// May be called from user, interrupt or kernel context.
    pub fn pollwakeup(php: *mut pollhead, event: c_short);
    /// `pollhead_clean(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Tells the kernel that the `pollhead` at `php` is about to be
    /// deallocated, usually from `close(9E)` before the soft state
    /// containing it is freed with `ddi_soft_state_free(9F)`. The kernel
    /// removes every poller registered on it. Generally called from the
    /// context of `close(9E)`, but it may be called from user or kernel
    /// context.
    pub fn pollhead_clean(php: *mut pollhead);
}
