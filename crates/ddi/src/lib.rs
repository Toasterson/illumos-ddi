//! Safe driver-development layer over the illumos DDI.
//!
//! The rules the section 9 man pages state in prose are expressed here as
//! types (see `docs/DESIGN.md` §5):
//!
//! - [`Errno`] versus [`DdiError`]: `cb_ops` and `dev_ops` return codes are
//!   different integer spaces and different types.
//! - [`Sleepable`]: proof that the current context may block. Handed to
//!   entry points by the framework; required by [`KBox::new`] and
//!   [`CondVar::wait`].
//! - [`Mutex`], [`CondVar`]: own their data, initialise through
//!   `mutex_init(9F)`/`cv_init(9F)`, destroy on drop.
//! - [`MinorNode`]: `ddi_create_minor_node(9F)` on creation,
//!   `ddi_remove_minor_node(9F)` on drop.
//! - [`Uio`], [`IoctlArg`]: the only way to touch user memory.
//! - [`module!`]: emits `_init`/`_fini`/`_info`, `modlinkage`, `dev_ops` and
//!   `cb_ops` from a [`Driver`] (and optionally [`CharDevice`]) impl.
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

/// Everything a driver normally needs.
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
