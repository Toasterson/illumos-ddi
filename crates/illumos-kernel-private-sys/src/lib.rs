//! Raw bindings to private illumos kernel interfaces, pinned to one gate
//! commit.
//!
//! This is the private tier of `docs/DESIGN.md` section 4. Everything here
//! is a kernel interface without a section 9 manual page and outside every
//! mapfile: the gate may change it in any commit, and nothing but the gate
//! source defines it. The crate is therefore usable only with the Cargo
//! feature that names the gate commit its declarations were written and
//! checked against, `unstable-gate-cda5b9a49f` (illumos `cda5b9a49f`, the
//! kernel of OpenIndiana Hipster 2026.04 on `oibldofgate` and of the
//! Hephaistos test guests). Without the feature the crate is empty. A
//! consumer that enables it states which kernel it was built for; moving to
//! another gate commit means re-reading every cited line, re-running
//! `xtask kernel-check` against that kernel and renaming the feature.
//!
//! Instead of a manual page, the first line of each item's documentation
//! cites the gate file and line that define it, in the form
//! `` `name`. Private, `uts/common/os/fio.c:1362`. `` (paths relative to
//! `usr/src`). `xtask lint-man --gate <gate>` checks that the cited line
//! exists and mentions the name, so a citation cannot silently go stale
//! against a gate checkout. The local checkout used for the first check was
//! at `0fb1d5d8d3`, which has no change to any cited file since
//! `cda5b9a49f` (`git log cda5b9a49f..0fb1d5d8d3 -- <files>` is empty).
//!
//! `xtask kernel-check` covers the crate like every other `-sys` crate: the
//! layout assertions against the running kernel's CTF data, and every
//! symbol against the kernel's symbol table and the module list in
//! `Cargo.toml` (`fs/specfs` for [`specfs::makespecvp`]).
//!
//! What is here is what an ioctl needs to hand a kernel object to user
//! space as a file descriptor of the calling process, the mechanism of
//! `pipe(2)` (`uts/common/syscall/pipe.c`): a vnode for a device number of
//! the calling driver ([`specfs::makespecvp`]), opened like `open(2)` would
//! ([`vnode::fop_open`]), a file structure and descriptor ([`file::falloc`],
//! [`file::setf`], [`file::f_setfd_or`]), the undo path
//! ([`vnode::fop_close`], [`file::unfalloc`], [`vnode::vn_rele`]), and the
//! way back from a descriptor to the vnode ([`file::getf`],
//! [`file::releasef`]). The safe wrapper is `ddi::private::fd`, behind the
//! `ddi` crate feature of the same name.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

#[cfg(feature = "unstable-gate-cda5b9a49f")]
pub mod file;
#[cfg(feature = "unstable-gate-cda5b9a49f")]
pub mod specfs;
#[cfg(feature = "unstable-gate-cda5b9a49f")]
pub mod vnode;

/// The gate commit the declarations were checked against.
#[cfg(feature = "unstable-gate-cda5b9a49f")]
pub const GATE_COMMIT: &str = "cda5b9a49f";
