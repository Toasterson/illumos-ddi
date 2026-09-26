//! Driver flags and default entry points from `sys/conf.h`.
//!
//! The `D_*` constants are combined into the `cb_flag` field of `cb_ops(9S)`
//! and describe what the driver supports. The three functions at the end are
//! the kernel's stand-ins for entry points a driver does not implement. The
//! `fmodsw(9S)` structure, which a STREAMS module hands to the kernel through
//! `modlstrmod(9S)`, is also defined in this header and so lives here.

use crate::poll::pollhead;
use crate::streams::streamtab;
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

/// The maximum length of a STREAMS module name, not counting the terminating
/// NUL (`FMNAMESZ`). The `f_name` field of [`fmodsw`] has room for one more
/// byte than this.
pub const FMNAMESZ: usize = 8;

/// `fmodsw(9S)`. illumos DDI specific (illumos DDI).
///
/// The declaration structure of a STREAMS module. Every STREAMS module
/// defines one and points at it from the `strmod_fmodsw` field of its
/// `modlstrmod(9S)`; when the module is loaded the kernel copies the
/// template into its class table. The name in `f_name` must match the
/// `mi_idname` of the module's `module_info(9S)` and should also match the
/// file name of the module binary. Every module sets `D_MP` in `f_flag` to
/// declare that it allows multiple threads of execution.
#[repr(C)]
pub struct fmodsw {
    /// The module name, NUL terminated, at most [`FMNAMESZ`] characters long.
    pub f_name: [c_char; FMNAMESZ + 1],
    /// The STREAMS table describing the module's queues.
    pub f_str: *mut streamtab,
    /// The driver flags, at least `D_MP`. See `mt-streams(9F)` for the rest.
    pub f_flag: c_int,
}

// f_name occupies bytes 0..9 and is padded to the 8-byte alignment of the
// pointer that follows it at offset 16. f_flag sits at 24 and the trailing
// padding brings the structure to 32 bytes.
const _: () = assert!(core::mem::size_of::<fmodsw>() == 32);
const _: () = assert!(core::mem::offset_of!(fmodsw, f_name) == 0);
const _: () = assert!(core::mem::offset_of!(fmodsw, f_str) == 16);
const _: () = assert!(core::mem::offset_of!(fmodsw, f_flag) == 24);

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
