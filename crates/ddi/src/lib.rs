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
//! The driver's own attached node is an [`OwnNode`], which only the
//! [`module!`] glue creates and which is neither `Copy` nor `Clone`. DMA
//! handles, interrupts, register mappings, PCI configuration handles,
//! minor nodes, property updates and LDI identifiers need it, so safe code
//! cannot create them for another node. [`DevInfo`] stays for queries.
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
//! [`Props`] reads device properties and, from the driver's own node,
//! writes them. Its lookups return
//! `Ok(None)` for a missing property and hand back kernel-allocated values
//! that free themselves, and every call demands the sleepable token because
//! the manual restricts the property functions to user and kernel context.
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
pub mod dma;
pub mod error;
pub mod intr;
pub mod ioctl;
pub mod kmem;
pub mod kstat;
pub mod ldi;
pub mod log;
pub mod module;
pub mod nvlist;
pub mod pci;
pub mod poll;
#[cfg(feature = "unstable-gate-cda5b9a49f")]
pub mod private;
pub mod prop;
pub mod regs;
pub mod sync;
pub mod taskq;
pub mod time;
pub mod uio;

pub use context::{Atomic, Sleepable};
pub use cred::Cred;
pub use device::{DevInfo, Instance, Minor, MinorKind, MinorNode, NodeType, OpenFlags, OpenType, OwnNode};
pub use dma::{
    DmaAttr, DmaBinding, DmaBuffer, DmaCache, DmaCookie, DmaCookies, DmaDirection, DmaError, DmaHandle, DmaMode,
};
pub use error::{DdiError, Errno, Result};
pub use intr::{
    Claimed, InterruptHandler, Interrupts, IntrCaps, IntrError, IntrKind, IntrPriority, IntrTypes, SoftInterrupt,
    SoftInterruptHandler,
};
pub use ioctl::{DataModel, IoctlArg, IoctlCmd};
pub use kmem::KBox;
pub use kstat::{IoKstat, KstatDataType, KstatId, NamedKstat, TypedKstat};
pub use ldi::{LdiFlags, LdiHandle, LdiIdent, LdiOpenType, LdiPropFlags};
pub use module::{CharDevice, DevmapHandle, Driver};
pub use nvlist::{
    AllocPolicy, KmNoSleep, KmSleep, NvEncoding, NvList, NvListRef, NvPair, NvStrings, NvValue, PackedNvList,
};
pub use pci::{ClassCode, IdleSpeed, PciConfig};
pub use poll::{PollEvents, PollHead, PollRegistration};
pub use prop::{PropBuf, PropError, PropString, PropStrings, Props, ReadOnly, ReadWrite};
pub use regs::{AccessAttr, DataOrder, Endian, FaultMode, Reg, RegBlock, RegError, RegMap, RegValue, RepMode};
pub use sync::{CondVar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, Semaphore, WaitResult};
pub use taskq::TaskQueue;
pub use time::{HrTime, Periodic, Ticks, Timeout};
pub use uio::Uio;

/// The names a driver normally needs, for a single `use ddi::prelude::*;`.
pub mod prelude {
    pub use crate::context::{Atomic, Sleepable};
    pub use crate::cred::Cred;
    pub use crate::device::{DevInfo, Instance, Minor, MinorKind, MinorNode, NodeType, OpenFlags, OpenType, OwnNode};
    pub use crate::dma::{DmaAttr, DmaBinding, DmaBuffer, DmaCache, DmaDirection, DmaError, DmaHandle, DmaMode};
    pub use crate::error::{DdiError, Errno, Result};
    pub use crate::intr::{
        Claimed, InterruptHandler, Interrupts, IntrError, IntrKind, IntrPriority, SoftInterrupt, SoftInterruptHandler,
    };
    pub use crate::ioctl::{DataModel, IoctlArg, IoctlCmd};
    pub use crate::kmem::KBox;
    pub use crate::kstat::{IoKstat, KstatDataType, KstatId, NamedKstat, TypedKstat};
    pub use crate::ldi::{LdiFlags, LdiHandle, LdiIdent};
    pub use crate::module::{CharDevice, DevmapHandle, Driver};
    pub use crate::nvlist::{NvEncoding, NvList, NvListRef};
    pub use crate::pci::PciConfig;
    pub use crate::poll::{PollEvents, PollHead, PollRegistration};
    pub use crate::prop::{PropBuf, PropError, PropString, PropStrings, Props};
    pub use crate::regs::{AccessAttr, DataOrder, Endian, FaultMode, Reg, RegBlock, RegError, RegMap, RepMode};
    pub use crate::sync::{
        CondVar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, Semaphore, WaitResult,
    };
    pub use crate::taskq::TaskQueue;
    pub use crate::time::{HrTime, Periodic, Ticks, Timeout};
    pub use crate::uio::Uio;
    pub use crate::{kcont, knote, kwarn, module, named_kstat};
}
