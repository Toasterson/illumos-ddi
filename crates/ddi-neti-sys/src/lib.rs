//! Raw bindings to the illumos network information (neti) and packet hook
//! interfaces.
//!
//! The network information framework, introduced in `netinfo(9F)`, lets a
//! kernel module work with the network layer protocols without linking
//! against IP. It can look up a protocol in an IP instance, walk its
//! interfaces, read their names, addresses and MTUs, look up routes, check
//! and inject packets, create kstats that belong to an IP instance, and be
//! told when IP instances are created and destroyed. Packet filters such as
//! IPFilter use it together with the packet hook framework, which lets a
//! module insert a callback into a networking event, for example into the
//! path of every packet received on a physical interface, and inspect,
//! change or drop the packet there.
//!
//! The two frameworks live in two kernel modules. The `net_*` functions
//! resolve from `misc/neti` and are declared in the [`neti`] module, which
//! follows `sys/neti.h`. The `hook_*` functions resolve from `misc/hook` and
//! are declared in the [`hook`] module, together with the event payloads
//! from `sys/hook_event.h`. The crate's `[package.metadata.kmod]` table lists
//! both modules, so that the link step records the dependencies. Neither
//! family has a mapfile in the gate. Every item of both modules is also
//! re-exported at the crate root.
//!
//! Every function and structure page of both families gives the interface
//! level "illumos DDI specific (illumos DDI)" and the interface stability
//! Committed, and `netinfo(9F)` itself gives only the stability. The manual
//! pages were written for an earlier revision of the interface and
//! disagree with the header on several prototypes and member lists. The
//! declarations follow the header, and each item says where the page
//! differs.
//!
//! What is left out is the provider side of both frameworks, which serves
//! the protocols that define families and events and has no manual pages,
//! the functions the header marks as internal to `misc/neti`, the private
//! structures, and the IP observability declarations. The module
//! documentation of [`neti`] and [`hook`] lists them by name. Nothing is
//! taken from `sys/neti_impl.h` or `sys/hook_impl.h`.
//!
//! The layout assertions in both modules were derived by hand for amd64
//! under the LP64 data model with `_KERNEL` defined.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

pub mod hook;
pub mod neti;

pub use hook::*;
pub use neti::*;
