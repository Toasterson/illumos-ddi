//! `sys/conf.h`: `cb_flag` bits and the default entry-point stubs.

use crate::types::*;

/// `D_NEW`: new-style driver (historic, always set).
pub const D_NEW: c_int = 0x00;
/// `D_MP`: driver is MT-safe. Mandatory.
pub const D_MP: c_int = 0x20;
/// `D_64BIT`: driver supports 64-bit offsets and block numbers.
pub const D_64BIT: c_int = 0x200;
/// `D_DEVMAP`: driver uses the `devmap(9E)` framework.
pub const D_DEVMAP: c_int = 0x100;
/// `D_HOTPLUG`: driver supports hot-plug.
pub const D_HOTPLUG: c_int = 0x400;
/// `D_U64BIT`: driver supports unsigned 64-bit offsets.
pub const D_U64BIT: c_int = 0x1000;

unsafe extern "C" {
    /// `nulldev(9F)`. Architecture independent level 1 (DDI/DKI). Returns 0.
    /// Untyped in C; the safe crate provides per-slot stubs instead of
    /// casting this.
    pub fn nulldev() -> c_int;
    /// `nodev(9F)`. Architecture independent level 1 (DDI/DKI). Returns ENXIO.
    pub fn nodev() -> c_int;
    /// `nochpoll(9F)`. illumos DDI specific (illumos DDI). Returns ENXIO.
    pub fn nochpoll(
        dev: dev_t,
        events: c_short,
        anyyet: c_int,
        reventsp: *mut c_short,
        phpp: *mut *mut pollhead,
    ) -> c_int;
}
