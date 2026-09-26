//! Raw bindings to the illumos SCSI architecture, SCSA.
//!
//! SCSA separates the drivers of SCSI devices from the drivers of the host
//! bus adapters, or HBAs, that connect them. A target driver, such as `sd`
//! for disks or `st` for tapes, knows the command set of a device class and
//! nothing about the controller. It describes each command in a
//! `scsi_pkt(9S)` and hands it to the framework with `scsi_transport(9F)`.
//! An HBA driver knows its controller and nothing about device classes. It
//! registers a `scsi_hba_tran(9S)` vector of `tran_*(9E)` entry points,
//! and the framework calls those entry points to allocate packets, start
//! commands, reset, abort and query capabilities on behalf of the target
//! drivers. A `scsi_address(9S)`, which every packet and every
//! `scsi_device(9S)` contains, points to the HBA's vector, and that is how
//! a call from one side reaches the other.
//!
//! The crate is split along the same line. [`target`] holds what a target
//! driver calls and [`hba`] what an HBA driver implements and calls,
//! including the Evolving iport and target map interfaces for SAS style
//! HBAs, and [`sas`] holds the Evolving SAS phy maps. The data that passes
//! between them is in [`pkt`] for the packet and its flags and completion
//! codes, [`device`] for addresses and devices, [`cdb`] for command
//! descriptor blocks, [`status`] for status bytes, [`sense`] for sense data
//! and [`inquiry`] for inquiry data. [`log`] holds the message and name
//! functions, and [`consts`] the capability indices, interconnect types,
//! `scsi-options` bits and minor node types. Everything is re-exported at
//! the crate root.
//!
//! The functions live in the `misc/scsi` kernel module, not in `genunix`,
//! so a driver that uses this crate must be linked with `-N misc/scsi`. The
//! crate's `Cargo.toml` records that dependency in its
//! `[package.metadata.kmod]` table for the link tool.
//!
//! The declarations follow `sys/scsi/scsi.h` and the headers it includes.
//! Most of SCSA's documented interface is declared under
//! `sys/scsi/impl/`, in `transport.h`, `services.h`, `sense.h`,
//! `commands.h` and `scsi_sas.h`, which are not public as a whole. From
//! those headers only what a section 9 page documents is bound, and each
//! module says which parts it takes. The rules of section 4 of
//! `docs/DESIGN.md` apply. Every function and structure names its manual
//! page and quotes its interface level. Structures are declared in full
//! with `#[repr(C)]` and pinned by layout assertions derived for amd64 with
//! LP64 and `_BIT_FIELDS_LTOH`. Bit fields, which Rust lacks, are declared
//! as their underlying bytes with the bits named in the documentation. Types
//! the kernel keeps private are declared with `ddi_sys::opaque!`. There is
//! no code, only declarations.
//!
//! Several things are left out. The obsolete functions `scsi_slave`,
//! `scsi_unslave`, `scsi_pktalloc`, `scsi_pktfree`, `scsi_resalloc`,
//! `scsi_resfree`, `scsi_dmaget`, `scsi_dmafree`, `makecom_g0`,
//! `makecom_g0_s`, `makecom_g1`, `makecom_g5`, `get_pktiopb` and
//! `free_pktiopb` are not bound; their manual pages mark them obsolete and
//! name the replacements. `scsi_dmafree` is among them because it shares
//! the obsolete page of `scsi_dmaget`, although `tran_dmafree(9E)` still
//! mentions it. Functions, structures and constants that appear in the
//! headers without a manual page are not bound either; the module
//! documentation lists them. C macros that only compute values, such as
//! `SCSI_IS_DESCR_SENSE`, `CDB_GROUPID`, `FORMG0ADDR` and the member
//! aliases `a_target` and `scc_cmd`, have no symbol and belong in the safe
//! crate.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

pub mod cdb;
pub mod consts;
pub mod device;
pub mod hba;
pub mod inquiry;
pub mod log;
pub mod pkt;
pub mod sas;
pub mod sense;
pub mod status;
pub mod target;

pub use cdb::*;
pub use consts::*;
pub use device::*;
pub use hba::*;
pub use inquiry::*;
pub use log::*;
pub use pkt::*;
pub use sas::*;
pub use sense::*;
pub use status::*;
pub use target::*;
