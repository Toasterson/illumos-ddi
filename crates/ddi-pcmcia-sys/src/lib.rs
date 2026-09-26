//! Raw bindings to the illumos PC Card services interface, `csx_*(9F)`.
//!
//! PC Card, which began life as PCMCIA, is the credit card sized expansion
//! bus of 1990s laptops. A card carries a Card Information Structure, or
//! CIS, in its attribute memory. The CIS is a chain of tuples that describe
//! what the card is, which configurations it supports, and which memory,
//! I/O and interrupt resources each configuration needs. Card Services is
//! the layer of software, standardised by PCMCIA and JEIDA, that sits
//! between the socket adapter drivers and the drivers of the cards
//! themselves. A card driver never touches the socket hardware. It
//! registers with Card Services as a client, receives insertion, removal
//! and other events through a callback, reads the tuples of its card,
//! requests windows, I/O ranges and an interrupt, and finally asks Card
//! Services to apply a configuration. It then reads and writes the card
//! through access handles that Card Services gave it.
//!
//! The hardware has been obsolete for a long time, and the illumos gate no
//! longer ships a driver for any PC Card. The only caller of these
//! functions left in the gate is the `pcmcia` nexus driver itself. The
//! interface is still there, though. The `csx_*` entry points are small
//! stubs in `uts/common/pcmcia/cs/cs_stubs.c`, which is linked into
//! `genunix`, and they forward every call to the Card Services module once
//! it has registered itself. A driver that uses this crate therefore needs no
//! `-N` module dependency, and the crate's `[package.metadata.kmod]`
//! table lists none. When no Card Services module is loaded, every
//! function returns [`CS_UNSUPPORTED_FUNCTION`], which the manual pages
//! describe as "No PCMCIA hardware installed".
//!
//! The crate follows the headers that `sys/pccard.h` includes, which the
//! build installs as `sys/pctypes.h`, `sys/cs_types.h`, `sys/cis.h`,
//! `sys/cis_handlers.h` and `sys/cs.h` from `uts/common/pcmcia/sys/`. It
//! is split by task. [`client`] holds registration, event and socket masks,
//! card status and the other per client requests. [`event`] holds the
//! event codes, the `csx_event_handler(9E)` callback and the text
//! conversion functions. [`window`] holds memory windows and the speed and
//! size conversions that go with them, [`io`] holds I/O ranges and
//! interrupts, and [`config`] holds the configuration of the card and
//! socket. [`tuple`](mod@tuple) holds the tuple walk over the CIS and the
//! tuple codes, [`cistpl`] the structures that the tuple parsers fill in,
//! and [`parse`] the parsers themselves. [`access`] holds the common access functions
//! that read and write the card through a handle, and [`consts`] the
//! return codes. Everything is re-exported at the crate root.
//!
//! The rules of section 4 of `docs/DESIGN.md` apply. Every function and
//! structure names its manual page and quotes its interface level.
//! Structures are declared in full with `#[repr(C)]` and pinned by layout
//! assertions derived by hand for amd64 with LP64 and `_KERNEL`, and
//! checked by compiling the gate headers with clang for the same target.
//! Types the kernel keeps private are declared with `ddi_sys::opaque!`.
//! There is no code, only declarations.
//!
//! Several things are left out. The header declares `csx_CS_Sys_Ctl`,
//! `csx_GetClientInfo`, `csx_GetCardServicesInfo`,
//! `csx_GetConfigurationInfo`, `csx_GetPhysicalAdapterInfo`,
//! `csx_GetPhysAddr`, `csx_Parse_CISTPL_FORMAT_A`, `csx_Parse_CISTPL_BAR`
//! and `csx_Parse_CISTPL_LONGLINK_CB`, and `cs_stubs.c` defines
//! `csx_register_cardservices` for the Card Services module, but no manual
//! page documents any of them, so they are not bound, and neither are the
//! structures that only they take. Everything in `cs_priv.h`, `cs_stubs.h`,
//! `cs_strings.h` and `cis_protos.h` is private to the Card Services
//! implementation, and so are the CIS interpreter internals in `cis.h` and
//! `cis_handlers.h`, such as `cisptr_t`, `cistpl_callout_t` and the
//! `HANDTPL_*` flags. The manual pages mention `CS_FAILURE`,
//! `CONF_VSOVERRIDE` and `CONFIG_ESR_REG_PRESENT`, but no header defines
//! them, so they cannot be bound. The flags that `cs.h` marks as private
//! to Card Services, reserved for compatibility and rejected, or due for
//! retirement are left out as well. C macros that only compute values,
//! such as `CS_CLIENT_INFO_MAKE_DATE`, `GET_CLIENT_INFO_SUBSVC` and the
//! `SCR_*` helpers, have no symbol and belong in a safe crate.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

pub mod access;
pub mod cistpl;
pub mod client;
pub mod config;
pub mod consts;
pub mod event;
pub mod io;
pub mod parse;
pub mod tuple;
pub mod window;

pub use access::*;
pub use cistpl::*;
pub use client::*;
pub use config::*;
pub use consts::*;
pub use event::*;
pub use io::*;
pub use parse::*;
pub use tuple::*;
pub use window::*;
