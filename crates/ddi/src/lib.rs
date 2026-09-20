//! The safe layer for writing illumos kernel drivers in Rust.
//!
//! This crate sits on top of the raw bindings in `ddi-sys` and is the only
//! crate a driver needs to depend on. Its purpose is to take the rules that
//! the section 9 manual pages state in prose and give them a shape the
//! compiler can check. Section 5 of `docs/DESIGN.md` explains the reasoning
//! at length; the main ideas are these.
//!
//! The DDI has two unrelated return conventions. `cb_ops` entry points return
//! an errno, `dev_ops` entry points return `DDI_SUCCESS` or `DDI_FAILURE`.
//! [`Errno`] and [`DdiError`] keep them apart at the type level.
//!
//! Some kernel functions may block and some contexts may not. A [`Sleepable`]
//! token is proof that blocking is allowed. The framework hands one to every
//! entry point that runs in thread context, and [`KBox::new`] and
//! [`CondVar::wait`] demand one.
//!
//! [`Mutex`] and [`CondVar`] own their data, call `mutex_init(9F)` and
//! `cv_init(9F)` when created and the matching destroy functions when dropped.
//! A [`MinorNode`] calls `ddi_create_minor_node(9F)` when created and
//! `ddi_remove_minor_node(9F)` when dropped.
//!
//! [`Uio`] and [`IoctlArg`] are the only ways to touch user memory, and both
//! go through the DDI copy functions so that kernel callers through LDI are
//! handled correctly.
//!
//! The [`module!`] macro turns a [`Driver`] implementation, and optionally a
//! [`CharDevice`] implementation, into the `_init`, `_fini` and `_info`
//! symbols and the `modlinkage`, `dev_ops` and `cb_ops` structures the kernel
//! expects.
#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

pub use ddi_sys as sys;

pub mod context;
pub mod cred;
pub mod device;
pub mod error;
pub mod ioctl;
pub mod kmem;
pub mod log;
pub mod module;
pub mod sync;
pub mod uio;

pub use context::Sleepable;
pub use cred::Cred;
pub use device::{DevInfo, Instance, Minor, MinorKind, MinorNode, NodeType, OpenFlags, OpenType};
pub use error::{DdiError, Errno, Result};
pub use ioctl::{DataModel, IoctlArg, IoctlCmd};
pub use kmem::KBox;
pub use module::{CharDevice, Driver};
pub use sync::{CondVar, Mutex, MutexGuard};
pub use uio::Uio;

/// The names a driver normally needs, for a single `use ddi::prelude::*;`.
pub mod prelude {
    pub use crate::context::Sleepable;
    pub use crate::cred::Cred;
    pub use crate::device::{DevInfo, Instance, Minor, MinorKind, MinorNode, NodeType, OpenFlags, OpenType};
    pub use crate::error::{DdiError, Errno, Result};
    pub use crate::ioctl::{DataModel, IoctlArg, IoctlCmd};
    pub use crate::kmem::KBox;
    pub use crate::module::{CharDevice, Driver};
    pub use crate::sync::{CondVar, Mutex, MutexGuard};
    pub use crate::uio::Uio;
    pub use crate::{kcont, knote, kwarn, module};
}
