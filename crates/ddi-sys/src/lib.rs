//! Raw bindings to the core illumos kernel DDI/DKI.
//!
//! Rules of this crate (see `docs/DESIGN.md` §4):
//!
//! - Every item corresponds to a header in `usr/src/uts/common/sys/` and to a
//!   section 9 manual page, named in its doc comment as `name(9X)`.
//! - Nothing from `*_impl.h`, `*_impldefs.h`, `esunddi.h` or
//!   `ddi_obsolete.h` appears here.
//! - No logic. No `impl` blocks beyond derives. No allocation.
//! - Function pointers in `9S` structures are `Option<unsafe extern "C" fn>`
//!   with exact signatures; NULL is `None`.
//! - Kernel-opaque types are declared with [`opaque!`] and only ever used
//!   behind raw pointers.
//! - Struct layouts are pinned by `tests/layout_generated.rs`, produced from
//!   the running kernel's CTF by `xtask layout-check` (M1).
//!
//! Layout assumptions are amd64 / LP64 only.
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
