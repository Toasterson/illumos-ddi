//! Unit conversion and comparison helpers documented under `sys/ddi.h`.
//!
//! Five names in this family have manual pages that prescribe a function
//! signature. Three of them are bound here as real functions.
//! `sys/param.h` defines `btop`, `btopr` and `ptob` as macros over
//! `PAGESHIFT`; `sys/ddi.h` then undefines those macros and declares the
//! three as external functions, which `uts/common/os/ddi.c` implements
//! with the same shifts. A driver that includes `sys/ddi.h` therefore
//! calls exported symbols, and those are what this module binds. No page
//! size constant is baked in here; the kernel's own value is used at run
//! time, which on amd64 is 4096 bytes.
//!
//! `min(9F)` and `max(9F)` are not bound. `sys/ddi.h` shadows the
//! functions with type-generic macros, and `uts/common/os/subr.c` defines
//! them, but the running kernel checked with `xtask kernel-check` (illumos
//! `cda5b9a49f`, OpenIndiana Hipster 2026.04) does not export either
//! symbol, so a module that called them would fail to load. A Rust driver
//! uses `core::cmp::min` and `core::cmp::max`, which is also what the `MIN`
//! and `MAX` macros of `sys/sysmacros.h` amount to. None of the five names
//! appears in the `ddi` mapfile; `ddi_btop(9F)` and `ddi_ptob(9F)`, which
//! do, are bound elsewhere in the crate.

use crate::types::*;

unsafe extern "C" {
    /// `btop(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns the number of whole memory pages contained in `numbytes`
    /// bytes, rounding down when the byte count is not a multiple of the
    /// page size. With a page size of 2048, the manual's example, both
    /// `btop(4096)` and `btop(4097)` return 2, and `btop(0)` returns 0.
    /// There are no invalid inputs and no error return. It can be called
    /// from user, interrupt, or kernel context.
    pub fn btop(numbytes: c_ulong) -> c_ulong;
    /// `btopr(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns the number of memory pages needed to hold `numbytes` bytes,
    /// rounding up to the next whole page. With a page size of 2048, the
    /// manual's example, `btopr(4096)` returns 2 and `btopr(4097)` returns
    /// 3. There are no invalid inputs and no error return. It can be called
    /// from user, interrupt, or kernel context.
    pub fn btopr(numbytes: c_ulong) -> c_ulong;
    /// `ptob(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns the number of bytes contained in `numpages` memory pages.
    /// With a page size of 2048, the manual's example, `ptob(2)` returns
    /// 4096, and `ptob(0)` returns 0. No overflow check is made when the
    /// byte count does not fit in an `unsigned long`; the high order bits
    /// are simply lost. It can be called from user, interrupt, or kernel
    /// context.
    pub fn ptob(numpages: c_ulong) -> c_ulong;
}
