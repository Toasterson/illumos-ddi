//! Capability indices, interconnect types, `scsi-options` bits and the SCSI
//! minor node types.
//!
//! The capability indices and interconnect types come from
//! `sys/scsi/impl/services.h`, bound here because `scsi_hba_lookup_capstr(9F)`
//! returns the former and `scsi_hba_tran(9S)` refers to the latter. The
//! `scsi-options` bits come from `sys/scsi/conf/autoconf.h` and are the ones
//! `scsi_hba_attach_setup(9F)` documents. The node types come from
//! `sys/sunddi.h`; the core `ddi-sys` crate declares the general ones, such
//! as `DDI_NT_BLOCK_CHAN`, and the ones used by SCSI target, HBA and SMP
//! drivers are declared here.
//!
//! Target drivers name capabilities with strings such as `auto-rqsense`,
//! which the manual pages list but which the headers define only inside the
//! `SCSI_CAP_ASCII` initialiser, so there is no constant to bind. The
//! `INTERCONNECT_*_STR` names, the `SCSI_VERSION_*` values of the
//! `scsi-version` capability, the `SCSI_OPTIONS_LINK`,
//! `SCSI_OPTIONS_NLUNS_*` and `SCSI_DEBUG_*` bits, and the remaining node
//! types of `sys/sunddi.h` are not documented by the SCSA pages and are not
//! bound.

use core::ffi::CStr;

use ddi_sys::c_int;

// --- capability indices --------------------------------------------------

/// The index of the `dma-max` capability, the largest DMA transfer the host
/// adapter supports (`SCSI_CAP_DMA_MAX`).
pub const SCSI_CAP_DMA_MAX: c_int = 0;
/// The index of the `msg-out` capability (`SCSI_CAP_MSG_OUT`).
pub const SCSI_CAP_MSG_OUT: c_int = 1;
/// The index of the `disconnect` capability (`SCSI_CAP_DISCONNECT`).
pub const SCSI_CAP_DISCONNECT: c_int = 2;
/// The index of the `synchronous` capability (`SCSI_CAP_SYNCHRONOUS`).
pub const SCSI_CAP_SYNCHRONOUS: c_int = 3;
/// The index of the `wide-xfer` capability (`SCSI_CAP_WIDE_XFER`).
pub const SCSI_CAP_WIDE_XFER: c_int = 4;
/// The index of the `parity` capability (`SCSI_CAP_PARITY`).
pub const SCSI_CAP_PARITY: c_int = 5;
/// The index of the `initiator-id` capability, the host's bus address
/// (`SCSI_CAP_INITIATOR_ID`).
pub const SCSI_CAP_INITIATOR_ID: c_int = 6;
/// The index of the `untagged-qing` capability (`SCSI_CAP_UNTAGGED_QING`).
pub const SCSI_CAP_UNTAGGED_QING: c_int = 7;
/// The index of the `tagged-qing` capability (`SCSI_CAP_TAGGED_QING`).
pub const SCSI_CAP_TAGGED_QING: c_int = 8;
/// The index of the `auto-rqsense` capability, automatic request sense on
/// check conditions (`SCSI_CAP_ARQ`).
pub const SCSI_CAP_ARQ: c_int = 9;
/// The index of the `linked-cmds` capability (`SCSI_CAP_LINKED_CMDS`).
pub const SCSI_CAP_LINKED_CMDS: c_int = 10;
/// The index of the `sector-size` capability, the granularity of DMA
/// breakup that the target driver sets (`SCSI_CAP_SECTOR_SIZE`).
pub const SCSI_CAP_SECTOR_SIZE: c_int = 11;
/// The index of the `total-sectors` capability, the capacity that the
/// target driver sets before asking for the geometry
/// (`SCSI_CAP_TOTAL_SECTORS`).
pub const SCSI_CAP_TOTAL_SECTORS: c_int = 12;
/// The index of the `geometry` capability (`SCSI_CAP_GEOMETRY`).
pub const SCSI_CAP_GEOMETRY: c_int = 13;
/// The index of the `reset-notification` capability
/// (`SCSI_CAP_RESET_NOTIFICATION`).
pub const SCSI_CAP_RESET_NOTIFICATION: c_int = 14;
/// The index of the `qfull-retries` capability (`SCSI_CAP_QFULL_RETRIES`).
pub const SCSI_CAP_QFULL_RETRIES: c_int = 15;
/// The index of the `qfull-retry-interval` capability
/// (`SCSI_CAP_QFULL_RETRY_INTERVAL`).
pub const SCSI_CAP_QFULL_RETRY_INTERVAL: c_int = 16;
/// The index of the `scsi-version` capability (`SCSI_CAP_SCSI_VERSION`).
pub const SCSI_CAP_SCSI_VERSION: c_int = 17;
/// The index of the `interconnect-type` capability, one of the
/// `INTERCONNECT_*` values (`SCSI_CAP_INTERCONNECT_TYPE`).
pub const SCSI_CAP_INTERCONNECT_TYPE: c_int = 18;
/// The index of the `lun-reset` capability (`SCSI_CAP_LUN_RESET`).
pub const SCSI_CAP_LUN_RESET: c_int = 19;
/// The index of the `max-cdb-length` capability (`SCSI_CAP_CDB_LEN`).
pub const SCSI_CAP_CDB_LEN: c_int = 20;
/// The index of the `dma-max-arch` capability, the largest DMA transfer
/// that the host adapter and the system architecture together support. It
/// cannot be set (`SCSI_CAP_DMA_MAX_ARCH`).
pub const SCSI_CAP_DMA_MAX_ARCH: c_int = 21;
/// The index of the `tran-layer-retries` capability
/// (`SCSI_CAP_TRAN_LAYER_RETRIES`).
pub const SCSI_CAP_TRAN_LAYER_RETRIES: c_int = 22;

// --- interconnect types --------------------------------------------------

/// Parallel SCSI (`INTERCONNECT_PARALLEL`).
pub const INTERCONNECT_PARALLEL: c_int = 1;
/// Fibre Channel with a private loop or a hard physical address
/// (`INTERCONNECT_FIBRE`).
pub const INTERCONNECT_FIBRE: c_int = 2;
/// IEEE 1394 (`INTERCONNECT_1394`).
pub const INTERCONNECT_1394: c_int = 3;
/// Serial Storage Architecture, no longer supported (`INTERCONNECT_SSA`).
pub const INTERCONNECT_SSA: c_int = 4;
/// Fibre Channel with a soft physical address or a switched fabric
/// (`INTERCONNECT_FABRIC`).
pub const INTERCONNECT_FABRIC: c_int = 5;
/// USB (`INTERCONNECT_USB`).
pub const INTERCONNECT_USB: c_int = 6;
/// ATAPI (`INTERCONNECT_ATAPI`).
pub const INTERCONNECT_ATAPI: c_int = 7;
/// iSCSI (`INTERCONNECT_ISCSI`).
pub const INTERCONNECT_ISCSI: c_int = 8;
/// SCSI RDMA Protocol over InfiniBand (`INTERCONNECT_IBSRP`).
pub const INTERCONNECT_IBSRP: c_int = 9;
/// SATA (`INTERCONNECT_SATA`).
pub const INTERCONNECT_SATA: c_int = 10;
/// SAS (`INTERCONNECT_SAS`).
pub const INTERCONNECT_SAS: c_int = 11;

// --- scsi-options --------------------------------------------------------

/// If not set, the HBA should not grant disconnect privileges to target
/// devices (`SCSI_OPTIONS_DR`).
pub const SCSI_OPTIONS_DR: c_int = 0x8;
/// If not set, the HBA should not operate the bus in synchronous transfer
/// mode (`SCSI_OPTIONS_SYNC`).
pub const SCSI_OPTIONS_SYNC: c_int = 0x20;
/// If not set, the HBA should not operate in parity mode
/// (`SCSI_OPTIONS_PARITY`).
pub const SCSI_OPTIONS_PARITY: c_int = 0x40;
/// If not set, the HBA should not operate in tagged command queueing mode
/// (`SCSI_OPTIONS_TAG`).
pub const SCSI_OPTIONS_TAG: c_int = 0x80;
/// If not set, the HBA should not operate the bus in FAST SCSI mode
/// (`SCSI_OPTIONS_FAST`).
pub const SCSI_OPTIONS_FAST: c_int = 0x100;
/// If not set, the HBA should not operate the bus in WIDE SCSI mode
/// (`SCSI_OPTIONS_WIDE`).
pub const SCSI_OPTIONS_WIDE: c_int = 0x200;
/// If not set, the HBA should not operate the bus in FAST20 mode
/// (`SCSI_OPTIONS_FAST20`).
pub const SCSI_OPTIONS_FAST20: c_int = 0x400;
/// If not set, the HBA should not operate the bus in FAST40 mode
/// (`SCSI_OPTIONS_FAST40`).
pub const SCSI_OPTIONS_FAST40: c_int = 0x800;
/// If not set, the HBA should not operate the bus in FAST80 mode
/// (`SCSI_OPTIONS_FAST80`).
pub const SCSI_OPTIONS_FAST80: c_int = 0x1000;
/// If not set, the HBA should not operate the bus in FAST160 mode
/// (`SCSI_OPTIONS_FAST160`).
pub const SCSI_OPTIONS_FAST160: c_int = 0x2000;
/// If not set, the HBA should not operate the bus in FAST320 mode
/// (`SCSI_OPTIONS_FAST320`).
pub const SCSI_OPTIONS_FAST320: c_int = 0x4000;
/// If not set, the HBA should not use quick arbitration and selection
/// (`SCSI_OPTIONS_QAS`).
pub const SCSI_OPTIONS_QAS: c_int = 0x100000;

// --- minor node types ----------------------------------------------------

/// The node type of a CD-ROM drive on a SCSI style channel, for
/// `ddi_create_minor_node(9F)` (`DDI_NT_CD_CHAN`).
pub const DDI_NT_CD_CHAN: &CStr = c"ddi_block:cdrom:channel";
/// The node type of a disk named by its world wide name
/// (`DDI_NT_BLOCK_WWN`).
pub const DDI_NT_BLOCK_WWN: &CStr = c"ddi_block:wwn";
/// The node type of a disk on a SAS fabric (`DDI_NT_BLOCK_SAS`).
pub const DDI_NT_BLOCK_SAS: &CStr = c"ddi_block:sas";
/// The node type of a tape drive (`DDI_NT_TAPE`).
pub const DDI_NT_TAPE: &CStr = c"ddi_byte:tape";
/// The node type of a SCSI enclosure services device
/// (`DDI_NT_SCSI_ENCLOSURE`).
pub const DDI_NT_SCSI_ENCLOSURE: &CStr = c"ddi_enclosure:scsi";
/// The node type of a SAS SMP device (`DDI_NT_SMP`).
pub const DDI_NT_SMP: &CStr = c"ddi_sas_smp";
/// The node type of the device control node of a SCSI HBA
/// (`DDI_NT_SCSI_NEXUS`).
pub const DDI_NT_SCSI_NEXUS: &CStr = c"ddi_ctl:devctl:scsi";
/// The node type of a SCSI attachment point for dynamic reconfiguration
/// (`DDI_NT_SCSI_ATTACHMENT_POINT`).
pub const DDI_NT_SCSI_ATTACHMENT_POINT: &CStr = c"ddi_ctl:attachment_point:scsi";
