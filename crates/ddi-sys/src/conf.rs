//! Driver flags and default entry points from `sys/conf.h`.
//!
//! The `D_*` constants are combined into the `cb_flag` field of `cb_ops(9S)`
//! and describe what the driver supports. The three functions at the end are
//! the kernel's stand-ins for entry points a driver does not implement.

use crate::types::*;

/// Marks a new-style driver (`D_NEW`). Historical; always set.
pub const D_NEW: c_int = 0x00;
/// The driver is safe for multithreaded access (`D_MP`). Every driver must set this.
pub const D_MP: c_int = 0x20;
/// The driver handles 64-bit offsets and block numbers (`D_64BIT`).
pub const D_64BIT: c_int = 0x200;
/// The driver uses the `devmap(9E)` framework for memory mapping (`D_DEVMAP`).
pub const D_DEVMAP: c_int = 0x100;
/// The driver supports hot-plugging (`D_HOTPLUG`).
pub const D_HOTPLUG: c_int = 0x400;
/// The driver handles unsigned 64-bit offsets (`D_U64BIT`).
pub const D_U64BIT: c_int = 0x1000;

unsafe extern "C" {
    /// `nulldev(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns 0. In C this function has no fixed prototype and is cast into
    /// whichever slot needs a do-nothing entry point. Rust cannot do that
    /// soundly, so the `ddi` crate provides a correctly typed stub for every
    /// slot instead. The declaration is kept here for completeness.
    pub fn nulldev() -> c_int;
    /// `nodev(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns `ENXIO`. The same caveat as for [`nulldev`] applies.
    pub fn nodev() -> c_int;
    /// `nochpoll(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns `ENXIO`. Unlike the other two, this one has the exact
    /// signature of `chpoll(9E)` and can be placed in `cb_chpoll` directly.
    pub fn nochpoll(
        dev: dev_t,
        events: c_short,
        anyyet: c_int,
        reventsp: *mut c_short,
        phpp: *mut *mut pollhead,
    ) -> c_int;
}
