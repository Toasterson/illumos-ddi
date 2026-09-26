//! Raw bindings to the illumos USB architecture, USBA.
//!
//! USBA is the framework through which illumos drives USB. It sits between
//! two kinds of drivers, and this crate binds the interface of each in its
//! own module.
//!
//! The [`client`] module is the client driver interface of
//! `sys/usb/usba.h` and `sys/usb/usbai.h`, the committed `usb_*(9F)` and
//! `usb_*(9S)` family. A client driver attaches to a device node that the
//! hub driver created for a USB device or for one of its interfaces,
//! registers with the framework, reads the parsed descriptors of its device,
//! opens pipes to endpoints, and exchanges control, bulk, interrupt and
//! isochronous requests over them. It never touches the host controller
//! directly.
//!
//! The [`hcdi`] module is the host controller driver interface of
//! `sys/usb/usba/hcdi.h` and `sys/usb/usba/hubdi.h`, the `usba_hcdi*(9E)`,
//! `usba_*(9F)` and `usba_*(9S)` family. A host controller driver fills in
//! an operations vector, registers it with the framework, binds a root hub,
//! and then carries out the pipe operations and transfers that the
//! framework passes down on behalf of the clients. Every page of this
//! family marks its interface as Volatile and private to illumos USB host
//! controller drivers, and warns that it may change at any time.
//!
//! All functions are implemented by the `misc/usba` kernel module rather
//! than by `genunix`, so a driver that uses this crate must be linked with a
//! dependency on that module. The crate records it in
//! `[package.metadata.kmod]`, and the link tool turns it into the `-N
//! misc/usba` flag.
//!
//! The crate follows the rules of `ddi-sys`, which are laid out in section 4
//! of `docs/DESIGN.md`. It contains declarations only. Every function,
//! structure and entry point type names its manual page and quotes the
//! interface level of that page. Structure layouts are pinned by constant
//! assertions derived by hand for amd64 with the LP64 data model and
//! `_KERNEL` defined, and the C enumerations are transparent newtypes with
//! associated constants. Each module lists in its own documentation what the
//! headers declare but the crate leaves out, and why.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

#[macro_use]
extern crate ddi_sys;

pub mod client;
pub mod hcdi;

pub use client::*;
pub use hcdi::*;
