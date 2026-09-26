//! Raw bindings to the illumos GLDv2 network driver interface.
//!
//! GLD, the Generic LAN Driver, is a loadable kernel module that implements
//! the Data Link Provider Interface, DLPI, and the STREAMS side of a network
//! driver on behalf of a device-specific driver. The device-specific driver
//! only deals with its hardware. It describes itself in a `gld_mac_info(9S)`
//! structure, fills in the `gldm_*` entry points that `gld(9E)` documents,
//! and hands received packets to GLD with `gld_recv(9F)`. GLD in turn
//! implements `open(9E)`, `close(9E)`, the STREAMS `put(9E)` and `srv(9E)`
//! routines and `getinfo(9E)`, answers the DLPI primitives, keeps the
//! statistics that the driver reports through `gld_stats(9S)`, and calls the
//! driver's entry points to start and stop the device, to transmit, and to
//! set addresses, multicast filters and promiscuous modes.
//!
//! This interface is GLDv2. It is the predecessor of GLDv3, the MAC
//! framework in the `misc/mac` module, which replaced it as the network
//! driver interface of illumos. GLDv2 is still shipped for a few older
//! drivers; in the gate the Chelsio `chxge` driver is still linked against
//! it. New drivers should not use it. They should use the MAC interface
//! through the `ddi-mac-sys` crate instead.
//!
//! A GLDv2 driver works as follows. Its `attach(9E)` routine allocates a
//! [`gld_mac_info_t`] with [`gld_mac_alloc`], fills in the members that
//! `gld_mac_info(9S)` marks as set by the driver, including the entry point
//! vectors from [`entry`], and links the instance to GLD with
//! [`gld_register`]. The read side `qinit(9S)` of the driver's `streamtab`
//! names [`gld_rsrv`], [`gld_open`] and [`gld_close`], the write side names
//! [`gld_wput`] and [`gld_wsrv`], and the `dev_ops(9S)` structure names
//! [`gld_getinfo`] as its `getinfo(9E)` routine, as `gld(4D)` requires. The
//! interrupt handler either is [`gld_intr`] with the `gld_mac_info_t` as its
//! argument, or calls the driver's own `gldm_intr` routine, and passes each
//! received packet to [`gld_recv`]. A driver whose `gldm_send` routine
//! returned [`GLD_NORESOURCES`] calls [`gld_sched`] once resources are free
//! again, and a driver that sets [`GLD_CAP_LINKSTATE`] reports link changes
//! with [`gld_linkstate`]. The `detach(9E)` routine calls
//! [`gld_unregister`] and, if that succeeds, [`gld_mac_free`].
//!
//! The functions live in the `misc/gld` kernel module, not in `genunix`, so
//! a driver that uses this crate must be linked with `-N misc/gld`, as
//! `gld(4D)` states. The crate's `Cargo.toml` records that dependency in its
//! `[package.metadata.kmod]` table for the link tool.
//!
//! The declarations follow `sys/gld.h`. The rules of section 4 of
//! `docs/DESIGN.md` apply. Every function, structure and entry point type
//! names its manual page and quotes its interface level. Structures are
//! declared in full with `#[repr(C)]`, including the members private to
//! GLD, because the kernel allocates them and the driver reads and writes
//! them in place, and their layouts are pinned by assertions derived for
//! amd64 with LP64 and `_KERNEL` defined. The module [`consts`] holds the
//! return values and flag values, [`entry`] the signatures of the `gld(9E)`
//! entry points, [`mac_info`] the `gld_mac_info(9S)` structure, [`stats`]
//! the `gld_stats(9S)` structure and [`service`] the functions of
//! `gld(9F)`, together with the STREAMS and `getinfo(9E)` routines that
//! `gld(4D)` tells drivers to install. Everything is re-exported at the
//! crate root. There is no code, only declarations.
//!
//! Several things in `sys/gld.h` are left out because no section 9 page
//! documents them. They are the function `gld_recv_tagged` and the tagged
//! transmit path it belongs to, the `VTAG_SIZE` and `VLAN_*` constants, the
//! `GLD_MAKE_TCI`, `GLD_MAKE_VTAG`, `GLD_TCI2VTAG`, `GLD_MK_*` and
//! `GLD_VTAG_*` macros, and the size helpers `GLD_STATS_SIZE_ORIG`,
//! `GLD_KSTAT_SIZE_ORIG` and `GLD_PAD`, the last of which is only used to
//! size a private member. The `glds_dot3_*`, `glds_dot5_*` and `glds_fddi_*`
//! names on the `gld_stats(9S)` page are C macros that expand to members of
//! [`gld_stats`] or of its media specific union, and they have no Rust
//! spelling of their own; each member's documentation says which name maps
//! to it. The `GLDOPT_*` and `GLD_OPT_*` options are defined in the private
//! header `sys/gldpriv.h` and are not part of the driver interface, so they
//! are not bound either.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

pub mod consts;
pub mod entry;
pub mod mac_info;
pub mod service;
pub mod stats;

pub use consts::*;
pub use entry::*;
pub use mac_info::*;
pub use service::*;
pub use stats::*;
