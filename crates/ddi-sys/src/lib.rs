//! Raw bindings to the core of the illumos kernel Driver Developer Interface.
//!
//! This crate is the bottom layer of the `illumos-ddi` project. It declares
//! the C types, constants and functions of the DDI/DKI exactly as the kernel
//! defines them, so that the safe `ddi` crate can build on a foundation that
//! is easy to audit against the manual. The rules it follows are laid out in
//! section 4 of `docs/DESIGN.md`; the short version is this.
//!
//! Every item here corresponds to a header under `usr/src/uts/common/sys/`
//! and to a manual page in section 9. Each function's doc comment names that
//! page and quotes its interface level, in the form `` `kmem_alloc(9F)`.
//! Architecture independent level 1 (DDI/DKI). ``
//!
//! Nothing is taken from `*_impl.h`, `*_impldefs.h`, `esunddi.h` or
//! `ddi_obsolete.h`. Those headers are private to the kernel, and the last
//! one is compiled out of every module in the gate anyway.
//!
//! The crate contains declarations only. There are no `impl` blocks beyond
//! derived traits and a few `const` initialisers, no allocation, and no
//! wrappers. Anything that adds behaviour belongs in the `ddi` crate.
//!
//! Function pointers inside `9S` structures are declared as
//! `Option<unsafe extern "C" fn>` with their exact C signatures, so that a
//! NULL slot is `None` rather than an invalid function pointer.
//!
//! Types whose layout the kernel keeps to itself are declared with the
//! [`opaque!`] macro and only ever appear behind raw pointers.
//!
//! Structure layouts are pinned by constant assertions in each module today,
//! and will be pinned by `tests/layout_generated.rs`, produced from the
//! running kernel's CTF data by `xtask layout-check`, once that tool exists.
//!
//! All layout assumptions are for amd64 with the LP64 data model.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

#[macro_use]
mod macros;

pub mod cmn_err;
pub mod conf;
pub mod cred;
pub mod devops;
pub mod errno;
pub mod file;
pub mod ioccom;
pub mod kmem;
pub mod ksynch;
pub mod modctl;
pub mod model;
pub mod stat;
pub mod sunddi;
pub mod taskq;
pub mod time;
pub mod types;
pub mod uio;

#[cfg(feature = "regs")]
pub mod regs;

pub use cmn_err::*;
pub use conf::*;
pub use cred::*;
pub use devops::*;
pub use errno::*;
pub use file::*;
pub use ioccom::*;
pub use kmem::*;
pub use ksynch::*;
pub use modctl::*;
pub use model::*;
pub use stat::*;
pub use sunddi::*;
pub use taskq::*;
pub use time::*;
pub use types::*;
pub use uio::*;

#[cfg(feature = "regs")]
pub use regs::*;
