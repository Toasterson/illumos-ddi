//! Raw bindings to the GLDv3 MAC provider interface of the illumos kernel.
//!
//! The MAC framework, also called GLDv3, is the layer of the illumos
//! networking stack that sits between network device drivers and everything
//! above them, from DLPI consumers to IP, VNICs, link aggregations and
//! bridges. A driver that plugs into it is called a MAC provider. The
//! provider does not create device nodes, implement `open(9E)` or speak DLPI
//! itself; the framework does all of that. Instead the provider registers
//! each device instance with `mac_register(9F)`, handing over a table of
//! entry points, `mac_callbacks(9S)`, through which the framework starts and
//! stops the device, transmits frames, programs address filters, reads
//! statistics and manages properties. In the other direction the provider
//! calls into the framework to deliver received frames, to report link state
//! changes and to lift transmit back pressure. `mac(9E)` gives the overview,
//! and every structure and function here names its own manual page.
//!
//! This crate declares that interface as the headers `sys/mac_provider.h`,
//! `sys/mac.h` and `sys/mac_ether.h` define it for kernel consumers on
//! amd64, together with the few flags from `sys/dlpi.h` and `sys/pattr.h`
//! that the manual pages of this family document. It follows the rules of
//! the `ddi-sys` crate, which are laid out in section 4 of `docs/DESIGN.md`.
//! Each item names its manual page and interface level in the first line of
//! its documentation, structures are pinned by layout assertions, C
//! enumerations are transparent wrappers with associated constants, and the
//! crate contains declarations only. It is split into modules by topic, and
//! everything is re-exported at the crate root.
//!
//! The functions live in the kernel module `misc/mac`, not in `genunix`, so
//! a provider must be linked with a dependency on that module. The
//! `[package.metadata.kmod]` table of this crate records it, together with
//! the `mac` mapfile of the gate, which lists the symbols a provider may
//! use. A few functions with manual pages are missing from that mapfile;
//! their documentation says so.
//!
//! The interface is versioned. A provider passes the version it was built
//! against, [`MAC_VERSION`], to `mac_alloc(9F)`, which refuses versions the
//! running kernel does not support, and the optional parts of
//! `mac_callbacks(9S)` are announced with `MC_*` flags so that the table can
//! grow without breaking older providers. The core of the interface is
//! illumos DDI specific. The ring, group and interrupt structures of
//! `MAC_CAPAB_RINGS` are Uncommitted, the LED capability is Evolving and the
//! transceiver capability is Volatile, so their layouts may change between
//! releases.
//!
//! Nothing is taken from `sys/mac_impl.h`, `sys/mac_client_impl.h`,
//! `sys/mac_soft_ring.h` or other private headers, and the client side of
//! the framework in `sys/mac_client.h` is out of scope. Every layout
//! assumption is for amd64 with the LP64 data model and `_KERNEL` defined.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

pub mod callbacks;
pub mod capab;
pub mod datapath;
pub mod props;
pub mod register;
pub mod rings;
pub mod stats;
pub mod types;

pub use callbacks::*;
pub use capab::*;
pub use datapath::*;
pub use props::*;
pub use register::*;
pub use rings::*;
pub use stats::*;
pub use types::*;
