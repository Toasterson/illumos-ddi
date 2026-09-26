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
//! The crate covers everything the `ddi` mapfile in the gate lists, and
//! every section 9F family whose symbols resolve from `genunix` or `unix`
//! without a module dependency. Families that need their own `-N`
//! dependency, such as MAC, SCSA or USBA, get their own crate. There are no
//! Cargo features; declarations cost nothing to compile and the mapfile
//! check at link time is what tells a driver which symbols it really uses.
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
//! Structure layouts are pinned by constant assertions in each module, and
//! were checked against the running kernel's CTF data with `mdb -k` on
//! amd64. All layout assumptions are for amd64 with the LP64 data model.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

#[macro_use]
mod macros;

pub mod atomic;
pub mod avl;
pub mod bio;
pub mod bitext;
pub mod cmn_err;
pub mod conf;
pub mod cred;
pub mod debug;
pub mod devid;
pub mod devmap;
pub mod devops;
pub mod dma;
#[cfg(target_arch = "x86_64")]
pub mod dmae;
pub mod errno;
pub mod events;
pub mod file;
pub mod firmload;
pub mod fm;
pub mod id_space;
pub mod intr;
pub mod ioccom;
pub mod kiconv;
pub mod kmem;
pub mod kstat;
pub mod ksynch;
pub mod ldi;
pub mod list;
pub mod map;
pub mod modctl;
pub mod model;
pub mod nvpair;
pub mod pci;
pub mod pm;
pub mod poll;
pub mod prop;
pub mod regs;
pub mod stat;
pub mod stdbit;
pub mod streams;
pub mod string;
pub mod sunddi;
pub mod sysmacros;
pub mod taskq;
pub mod time;
pub mod types;
pub mod ufm;
pub mod uio;
pub mod umem;
pub mod vmem;

pub use atomic::*;
pub use avl::*;
pub use bio::*;
pub use bitext::*;
pub use cmn_err::*;
pub use conf::*;
pub use cred::*;
pub use debug::*;
pub use devid::*;
pub use devmap::*;
pub use devops::*;
pub use dma::*;
#[cfg(target_arch = "x86_64")]
pub use dmae::*;
pub use errno::*;
pub use events::*;
pub use file::*;
pub use firmload::*;
pub use fm::*;
pub use id_space::*;
pub use intr::*;
pub use ioccom::*;
pub use kiconv::*;
pub use kmem::*;
pub use kstat::*;
pub use ksynch::*;
pub use ldi::*;
pub use list::*;
pub use map::*;
pub use modctl::*;
pub use model::*;
pub use nvpair::*;
pub use pci::*;
pub use pm::*;
pub use poll::*;
pub use prop::*;
pub use regs::*;
pub use stat::*;
pub use stdbit::*;
pub use streams::*;
pub use string::*;
pub use sunddi::*;
pub use sysmacros::*;
pub use taskq::*;
pub use time::*;
pub use types::*;
pub use ufm::*;
pub use uio::*;
pub use umem::*;
pub use vmem::*;
