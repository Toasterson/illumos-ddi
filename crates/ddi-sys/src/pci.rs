//! PCI configuration space access, power management reporting and error
//! reporting, from the `pci_*` subset of `sys/sunddi.h` together with the
//! register layout constants from `sys/pci.h` and `pci_lcap_locate` from
//! `sys/pci_cap.h`.
//!
//! A PCI leaf driver obtains a handle to its function's configuration space
//! with `pci_config_setup(9F)` during `attach(9E)` and reads and writes the
//! space through it with the `pci_config_get8(9F)` and `pci_config_put8(9F)`
//! families, which take a byte offset rather than an address and translate
//! between the little-endian layout of configuration space and host order.
//! `pci_config_teardown(9F)` releases the handle in `detach(9E)`. Around
//! that core sit three smaller interfaces. `pci_save_config_regs(9F)` and
//! `pci_restore_config_regs(9F)` preserve the header across a trip through
//! the D3 power state in `power(9E)`. `pci_report_pmcap(9F)` tells the
//! framework whether the device tolerates a slowed clock while idle.
//! `pci_ereport_setup(9F)`, `pci_ereport_post(9F)` and
//! `pci_ereport_teardown(9F)` let a driver that has declared
//! `DDI_FM_EREPORT_CAPABLE` through [`crate::fm::ddi_fm_init`] delegate the
//! scanning of the PCI status registers and the posting of the resulting
//! error reports to the framework from inside its error handler.
//!
//! `pci_lcap_locate` walks the capability list in conventional
//! configuration space for a capability identifier. It is the only function
//! from `sys/pci_cap.h` that appears in the `ddi` mapfile, and the header
//! has no manual page at all, so its behaviour is described here from the
//! source in `uts/common/io/pci_cap.c`. Its siblings `pci_xcap_locate`,
//! `pci_htcap_locate`, `pci_cap_probe`, `pci_cap_get`, `pci_cap_put` and
//! `pci_cap_read` are neither in the mapfile nor documented and are left
//! out, as are the `PCI_CAP_GET*` and `PCI_CAP_PUT*` macros built on them.
//!
//! The constants cover what a leaf driver needs to read its own header. The
//! type 0 register offsets from `PCI_CONF_VENID` to `PCI_CONF_MAX_L` are
//! all here, as are the type 1 bridge header offsets, the command and status
//! bits, the header type values, the base address register decode masks,
//! the capability identifiers with the layout of a capability entry, and
//! the power management capability and control registers that
//! `pci_report_pmcap` and `power(9E)` implementations consult. The offsets
//! are typed `off_t` so that they can be passed straight to
//! `pci_config_get8` and its relatives; the bit masks carry the width of the
//! register they describe.
//!
//! The rest of `sys/pci.h` is deliberately absent. It consists of the class
//! and subclass code tables, the bridge control and window decoding bits,
//! the MSI, MSI-X, PCI-X, HyperTransport and vendor capability layouts, the
//! PCI Express and hot plug registers, the type 2 CardBus header, and the
//! address encoding used in `reg` and `assigned-addresses` properties. Those
//! belong to nexus drivers, to the PCI Express framework, or to feature
//! specific crates, and binding them here would triple the size of this
//! module without serving a leaf driver. Nothing from `sys/pci_impl.h` or
//! `sys/pcie_impl.h` is bound.

use crate::fm::ddi_fm_error_t;
use crate::types::*;

// --- type 0 configuration header ----------------------------------------------

/// The byte offset of the two byte vendor identifier (`PCI_CONF_VENID`).
pub const PCI_CONF_VENID: off_t = 0x0;
/// The byte offset of the two byte device identifier (`PCI_CONF_DEVID`).
pub const PCI_CONF_DEVID: off_t = 0x2;
/// The byte offset of the two byte command register, whose bits are the
/// `PCI_COMM_*` values (`PCI_CONF_COMM`).
pub const PCI_CONF_COMM: off_t = 0x4;
/// The byte offset of the two byte status register, whose bits are the
/// `PCI_STAT_*` values (`PCI_CONF_STAT`).
pub const PCI_CONF_STAT: off_t = 0x6;
/// The byte offset of the one byte revision identifier (`PCI_CONF_REVID`).
pub const PCI_CONF_REVID: off_t = 0x8;
/// The byte offset of the one byte programming interface code
/// (`PCI_CONF_PROGCLASS`).
pub const PCI_CONF_PROGCLASS: off_t = 0x9;
/// The byte offset of the one byte subclass code (`PCI_CONF_SUBCLASS`).
pub const PCI_CONF_SUBCLASS: off_t = 0xA;
/// The byte offset of the one byte base class code (`PCI_CONF_BASCLASS`).
pub const PCI_CONF_BASCLASS: off_t = 0xB;
/// The byte offset of the one byte cache line size register
/// (`PCI_CONF_CACHE_LINESZ`).
pub const PCI_CONF_CACHE_LINESZ: off_t = 0xC;
/// The byte offset of the one byte latency timer (`PCI_CONF_LATENCY_TIMER`).
pub const PCI_CONF_LATENCY_TIMER: off_t = 0xD;
/// The byte offset of the one byte header type register, whose values are
/// the `PCI_HEADER_*` constants (`PCI_CONF_HEADER`).
pub const PCI_CONF_HEADER: off_t = 0xE;
/// The byte offset of the one byte built-in self test register
/// (`PCI_CONF_BIST`).
pub const PCI_CONF_BIST: off_t = 0xF;
/// The byte offset of the four byte base address register 0
/// (`PCI_CONF_BASE0`).
pub const PCI_CONF_BASE0: off_t = 0x10;
/// The byte offset of the four byte base address register 1
/// (`PCI_CONF_BASE1`).
pub const PCI_CONF_BASE1: off_t = 0x14;
/// The byte offset of the four byte base address register 2
/// (`PCI_CONF_BASE2`).
pub const PCI_CONF_BASE2: off_t = 0x18;
/// The byte offset of the four byte base address register 3
/// (`PCI_CONF_BASE3`).
pub const PCI_CONF_BASE3: off_t = 0x1c;
/// The byte offset of the four byte base address register 4
/// (`PCI_CONF_BASE4`).
pub const PCI_CONF_BASE4: off_t = 0x20;
/// The byte offset of the four byte base address register 5
/// (`PCI_CONF_BASE5`).
pub const PCI_CONF_BASE5: off_t = 0x24;
/// The byte offset of the four byte CardBus CIS pointer (`PCI_CONF_CIS`).
pub const PCI_CONF_CIS: off_t = 0x28;
/// The byte offset of the two byte subsystem vendor identifier
/// (`PCI_CONF_SUBVENID`).
pub const PCI_CONF_SUBVENID: off_t = 0x2c;
/// The byte offset of the two byte subsystem identifier
/// (`PCI_CONF_SUBSYSID`).
pub const PCI_CONF_SUBSYSID: off_t = 0x2e;
/// The byte offset of the four byte expansion ROM base address register
/// (`PCI_CONF_ROM`).
pub const PCI_CONF_ROM: off_t = 0x30;
/// The byte offset of the one byte capabilities pointer, valid when
/// [`PCI_STAT_CAP`] is set in the status register (`PCI_CONF_CAP_PTR`).
pub const PCI_CONF_CAP_PTR: off_t = 0x34;
/// The byte offset of the one byte interrupt line register
/// (`PCI_CONF_ILINE`).
pub const PCI_CONF_ILINE: off_t = 0x3c;
/// The byte offset of the one byte interrupt pin register
/// (`PCI_CONF_IPIN`).
pub const PCI_CONF_IPIN: off_t = 0x3d;
/// The byte offset of the one byte minimum grant register
/// (`PCI_CONF_MIN_G`).
pub const PCI_CONF_MIN_G: off_t = 0x3e;
/// The byte offset of the one byte maximum latency register; this is the
/// last register of the type 0 header (`PCI_CONF_MAX_L`).
pub const PCI_CONF_MAX_L: off_t = 0x3f;

/// The number of base address registers in a type 0 header
/// (`PCI_BASE_NUM`).
pub const PCI_BASE_NUM: c_int = 6;
/// The size in bytes of one base address register (`PCI_BASE_SIZE`).
pub const PCI_BASE_SIZE: c_int = 4;
/// The size in bytes of the conventional configuration header, which is
/// also the extent of what `pci_config_get8` can reach on a non-PCI Express
/// bus (`PCI_CONF_HDR_SIZE`).
pub const PCI_CONF_HDR_SIZE: c_int = 256;

// --- type 1 configuration header (PCI to PCI bridge) --------------------------

/// The byte offset in a type 1 header of the primary bus number
/// (`PCI_BCNF_PRIBUS`).
pub const PCI_BCNF_PRIBUS: off_t = 0x18;
/// The byte offset in a type 1 header of the secondary bus number
/// (`PCI_BCNF_SECBUS`).
pub const PCI_BCNF_SECBUS: off_t = 0x19;
/// The byte offset in a type 1 header of the subordinate bus number
/// (`PCI_BCNF_SUBBUS`).
pub const PCI_BCNF_SUBBUS: off_t = 0x1a;
/// The byte offset in a type 1 header of the secondary latency timer
/// (`PCI_BCNF_LATENCY_TIMER`).
pub const PCI_BCNF_LATENCY_TIMER: off_t = 0x1b;
/// The byte offset in a type 1 header of the low byte of the I/O window
/// base (`PCI_BCNF_IO_BASE_LOW`).
pub const PCI_BCNF_IO_BASE_LOW: off_t = 0x1c;
/// The byte offset in a type 1 header of the low byte of the I/O window
/// limit (`PCI_BCNF_IO_LIMIT_LOW`).
pub const PCI_BCNF_IO_LIMIT_LOW: off_t = 0x1d;
/// The byte offset in a type 1 header of the two byte secondary status
/// register (`PCI_BCNF_SEC_STATUS`).
pub const PCI_BCNF_SEC_STATUS: off_t = 0x1e;
/// The byte offset in a type 1 header of the two byte memory window base
/// (`PCI_BCNF_MEM_BASE`).
pub const PCI_BCNF_MEM_BASE: off_t = 0x20;
/// The byte offset in a type 1 header of the two byte memory window limit
/// (`PCI_BCNF_MEM_LIMIT`).
pub const PCI_BCNF_MEM_LIMIT: off_t = 0x22;
/// The byte offset in a type 1 header of the two byte low half of the
/// prefetchable memory window base (`PCI_BCNF_PF_BASE_LOW`).
pub const PCI_BCNF_PF_BASE_LOW: off_t = 0x24;
/// The byte offset in a type 1 header of the two byte low half of the
/// prefetchable memory window limit (`PCI_BCNF_PF_LIMIT_LOW`).
pub const PCI_BCNF_PF_LIMIT_LOW: off_t = 0x26;
/// The byte offset in a type 1 header of the four byte high half of the
/// prefetchable memory window base (`PCI_BCNF_PF_BASE_HIGH`).
pub const PCI_BCNF_PF_BASE_HIGH: off_t = 0x28;
/// The byte offset in a type 1 header of the four byte high half of the
/// prefetchable memory window limit (`PCI_BCNF_PF_LIMIT_HIGH`).
pub const PCI_BCNF_PF_LIMIT_HIGH: off_t = 0x2c;
/// The byte offset in a type 1 header of the two byte high half of the I/O
/// window base (`PCI_BCNF_IO_BASE_HI`).
pub const PCI_BCNF_IO_BASE_HI: off_t = 0x30;
/// The byte offset in a type 1 header of the two byte high half of the I/O
/// window limit (`PCI_BCNF_IO_LIMIT_HI`).
pub const PCI_BCNF_IO_LIMIT_HI: off_t = 0x32;
/// The byte offset in a type 1 header of the one byte capabilities pointer
/// (`PCI_BCNF_CAP_PTR`).
pub const PCI_BCNF_CAP_PTR: off_t = 0x34;
/// The byte offset in a type 1 header of the four byte expansion ROM base
/// address register (`PCI_BCNF_ROM`).
pub const PCI_BCNF_ROM: off_t = 0x38;
/// The byte offset in a type 1 header of the one byte interrupt line
/// register (`PCI_BCNF_ILINE`).
pub const PCI_BCNF_ILINE: off_t = 0x3c;
/// The byte offset in a type 1 header of the one byte interrupt pin
/// register (`PCI_BCNF_IPIN`).
pub const PCI_BCNF_IPIN: off_t = 0x3d;
/// The byte offset in a type 1 header of the two byte bridge control
/// register (`PCI_BCNF_BCNTRL`).
pub const PCI_BCNF_BCNTRL: off_t = 0x3e;
/// The number of base address registers in a type 1 header
/// (`PCI_BCNF_BASE_NUM`).
pub const PCI_BCNF_BASE_NUM: c_int = 0x2;
/// The byte offset in a type 2 CardBus header of the one byte capabilities
/// pointer; declared here because `pci_lcap_locate` consults it
/// (`PCI_CBUS_CAP_PTR`).
pub const PCI_CBUS_CAP_PTR: off_t = 0x14;

// --- command register ---------------------------------------------------------

/// The command bit that enables the function to respond to I/O space
/// accesses (`PCI_COMM_IO`).
pub const PCI_COMM_IO: u16 = 0x1;
/// The command bit that enables the function to respond to memory space
/// accesses (`PCI_COMM_MAE`).
pub const PCI_COMM_MAE: u16 = 0x2;
/// The command bit that allows the function to act as a bus master, which
/// it must be to perform DMA (`PCI_COMM_ME`).
pub const PCI_COMM_ME: u16 = 0x4;
/// The command bit that enables monitoring of special cycles
/// (`PCI_COMM_SPEC_CYC`).
pub const PCI_COMM_SPEC_CYC: u16 = 0x8;
/// The command bit that permits the memory write and invalidate command
/// (`PCI_COMM_MEMWR_INVAL`).
pub const PCI_COMM_MEMWR_INVAL: u16 = 0x10;
/// The command bit that enables VGA palette snooping
/// (`PCI_COMM_PALETTE_SNOOP`).
pub const PCI_COMM_PALETTE_SNOOP: u16 = 0x20;
/// The command bit that enables the function to act on detected parity
/// errors (`PCI_COMM_PARITY_DETECT`).
pub const PCI_COMM_PARITY_DETECT: u16 = 0x40;
/// The command bit that enables address and data stepping
/// (`PCI_COMM_WAIT_CYC_ENAB`).
pub const PCI_COMM_WAIT_CYC_ENAB: u16 = 0x80;
/// The command bit that enables assertion of the system error signal
/// (`PCI_COMM_SERR_ENABLE`).
pub const PCI_COMM_SERR_ENABLE: u16 = 0x100;
/// The command bit that allows fast back to back transactions to other
/// agents (`PCI_COMM_BACK2BACK_ENAB`).
pub const PCI_COMM_BACK2BACK_ENAB: u16 = 0x200;
/// The command bit that disables assertion of the legacy INTx interrupt
/// signal, set by the framework when MSI or MSI-X is in use
/// (`PCI_COMM_INTX_DISABLE`).
pub const PCI_COMM_INTX_DISABLE: u16 = 0x400;

// --- status register ----------------------------------------------------------

/// The status bit saying the function is ready for configuration accesses
/// immediately after reset (`PCI_STAT_READY`).
pub const PCI_STAT_READY: u16 = 0x1;
/// The status bit saying the function is asserting its INTx interrupt
/// signal (`PCI_STAT_INTR`).
pub const PCI_STAT_INTR: u16 = 0x8;
/// The status bit saying the function implements a capability list, which
/// makes [`PCI_CONF_CAP_PTR`] valid (`PCI_STAT_CAP`).
pub const PCI_STAT_CAP: u16 = 0x10;
/// The status bit saying the function is capable of 66 MHz operation
/// (`PCI_STAT_66MHZ`).
pub const PCI_STAT_66MHZ: u16 = 0x20;
/// The status bit saying user definable features are supported
/// (`PCI_STAT_UDF`).
pub const PCI_STAT_UDF: u16 = 0x40;
/// The status bit saying the function can accept fast back to back
/// transactions (`PCI_STAT_FBBC`).
pub const PCI_STAT_FBBC: u16 = 0x80;
/// The status bit recording a master data parity error
/// (`PCI_STAT_S_PERROR`).
pub const PCI_STAT_S_PERROR: u16 = 0x100;
/// The mask of the two device select timing bits, whose values are
/// [`PCI_STAT_DEVSELT_FAST`], [`PCI_STAT_DEVSELT_MEDIUM`] and
/// [`PCI_STAT_DEVSELT_SLOW`] (`PCI_STAT_DEVSELT`).
pub const PCI_STAT_DEVSELT: u16 = 0x600;
/// The status bit recording that the function signalled a target abort
/// (`PCI_STAT_S_TARG_AB`).
pub const PCI_STAT_S_TARG_AB: u16 = 0x800;
/// The status bit recording that the function received a target abort
/// (`PCI_STAT_R_TARG_AB`).
pub const PCI_STAT_R_TARG_AB: u16 = 0x1000;
/// The status bit recording that the function received a master abort
/// (`PCI_STAT_R_MAST_AB`).
pub const PCI_STAT_R_MAST_AB: u16 = 0x2000;
/// The status bit recording that the function signalled a system error
/// (`PCI_STAT_S_SYSERR`).
pub const PCI_STAT_S_SYSERR: u16 = 0x4000;
/// The status bit recording that the function detected a parity error
/// (`PCI_STAT_PERROR`).
pub const PCI_STAT_PERROR: u16 = 0x8000;
/// The device select timing value for fast decoding
/// (`PCI_STAT_DEVSELT_FAST`).
pub const PCI_STAT_DEVSELT_FAST: u16 = 0x0000;
/// The device select timing value for medium decoding
/// (`PCI_STAT_DEVSELT_MEDIUM`).
pub const PCI_STAT_DEVSELT_MEDIUM: u16 = 0x0200;
/// The device select timing value for slow decoding
/// (`PCI_STAT_DEVSELT_SLOW`).
pub const PCI_STAT_DEVSELT_SLOW: u16 = 0x0400;

// --- header type register -------------------------------------------------------

/// The header type bit saying the device has several functions
/// (`PCI_HEADER_MULTI`).
pub const PCI_HEADER_MULTI: u8 = 0x80;
/// The header type value of an ordinary device, with six base address
/// registers (`PCI_HEADER_ZERO`).
pub const PCI_HEADER_ZERO: u8 = 0x00;
/// The header type value of a PCI to PCI bridge (`PCI_HEADER_ONE`).
pub const PCI_HEADER_ONE: u8 = 0x01;
/// The header type value of a CardBus bridge (`PCI_HEADER_TWO`).
pub const PCI_HEADER_TWO: u8 = 0x02;
/// The header type value of a PCI to PCI bridge, the same as
/// `PCI_HEADER_ONE` (`PCI_HEADER_PPB`).
pub const PCI_HEADER_PPB: u8 = PCI_HEADER_ONE;
/// The header type value of a CardBus bridge, the same as `PCI_HEADER_TWO`
/// (`PCI_HEADER_CARDBUS`).
pub const PCI_HEADER_CARDBUS: u8 = PCI_HEADER_TWO;
/// The mask that isolates the header type from the multi-function bit
/// (`PCI_HEADER_TYPE_M`).
pub const PCI_HEADER_TYPE_M: u8 = 0x7f;

// --- base address registers -------------------------------------------------------

/// The base address register bit that distinguishes I/O space from memory
/// space (`PCI_BASE_SPACE_M`).
pub const PCI_BASE_SPACE_M: u32 = 0x1;
/// The value of the space bit for a base address register that decodes I/O
/// space (`PCI_BASE_SPACE_IO`).
pub const PCI_BASE_SPACE_IO: u32 = 0x1;
/// The value of the space bit for a base address register that decodes
/// memory space (`PCI_BASE_SPACE_MEM`).
pub const PCI_BASE_SPACE_MEM: u32 = 0x0;
/// The memory type field value for a 32-bit base address register
/// (`PCI_BASE_TYPE_MEM`).
pub const PCI_BASE_TYPE_MEM: u32 = 0x0;
/// The memory type field value for a register that must be placed below one
/// megabyte, no longer produced by current hardware (`PCI_BASE_TYPE_LOW`).
pub const PCI_BASE_TYPE_LOW: u32 = 0x2;
/// The memory type field value for a 64-bit base address register, which
/// occupies this register and the next (`PCI_BASE_TYPE_ALL`).
pub const PCI_BASE_TYPE_ALL: u32 = 0x4;
/// The reserved memory type field value (`PCI_BASE_TYPE_RES`).
pub const PCI_BASE_TYPE_RES: u32 = 0x6;
/// The mask that isolates the memory type field (`PCI_BASE_TYPE_M`).
pub const PCI_BASE_TYPE_M: u32 = 0x00000006;
/// The base address register bit saying the memory is prefetchable
/// (`PCI_BASE_PREF_M`).
pub const PCI_BASE_PREF_M: u32 = 0x00000008;
/// The mask that isolates the address from a 32-bit memory base address
/// register (`PCI_BASE_M_ADDR_M`).
pub const PCI_BASE_M_ADDR_M: u32 = 0xfffffff0;
/// The mask that isolates the address from the pair of registers forming a
/// 64-bit memory base address (`PCI_BASE_M_ADDR64_M`).
pub const PCI_BASE_M_ADDR64_M: u64 = 0xfffffffffffffff0;
/// The mask that isolates the address from an I/O base address register
/// (`PCI_BASE_IO_ADDR_M`).
pub const PCI_BASE_IO_ADDR_M: u32 = 0xfffffffe;
/// The mask that isolates the address from the expansion ROM base address
/// register (`PCI_BASE_ROM_ADDR_M`).
pub const PCI_BASE_ROM_ADDR_M: u32 = 0xfffff800;
/// The expansion ROM base address register bit that enables decoding of
/// the ROM (`PCI_BASE_ROM_ENABLE`).
pub const PCI_BASE_ROM_ENABLE: u32 = 0x00000001;

// --- capability list ------------------------------------------------------------

/// The offset within a capability entry of the one byte pointer to the
/// next entry; the identifier is at offset 0 (`PCI_CAP_NEXT_PTR`).
pub const PCI_CAP_NEXT_PTR: off_t = 0x1;
/// The offset within a capability entry at which the capability's own
/// registers begin (`PCI_CAP_ID_REGS_OFF`).
pub const PCI_CAP_ID_REGS_OFF: off_t = 0x2;
/// The largest number of entries a well formed capability list can hold;
/// `pci_lcap_locate` stops after this many to survive a looped list
/// (`PCI_CAP_MAX_PTR`).
pub const PCI_CAP_MAX_PTR: u16 = 0x30;
/// The lowest offset at which a capability entry may legally start, since
/// everything below it belongs to the header (`PCI_CAP_PTR_OFF`).
pub const PCI_CAP_PTR_OFF: off_t = 0x40;
/// The mask applied to a capability pointer, whose low two bits are
/// reserved because entries are aligned to four bytes (`PCI_CAP_PTR_MASK`).
pub const PCI_CAP_PTR_MASK: u8 = 0xFC;
/// The next pointer value that ends the capability list; also what
/// `pci_lcap_locate` stores when it finds nothing (`PCI_CAP_NEXT_PTR_NULL`).
pub const PCI_CAP_NEXT_PTR_NULL: u16 = 0x0;

/// The capability identifier of the power management capability
/// (`PCI_CAP_ID_PM`).
pub const PCI_CAP_ID_PM: u8 = 0x1;
/// The capability identifier of the accelerated graphics port capability
/// (`PCI_CAP_ID_AGP`).
pub const PCI_CAP_ID_AGP: u8 = 0x2;
/// The capability identifier of the vital product data capability
/// (`PCI_CAP_ID_VPD`).
pub const PCI_CAP_ID_VPD: u8 = 0x3;
/// The capability identifier of the slot identification capability
/// (`PCI_CAP_ID_SLOT_ID`).
pub const PCI_CAP_ID_SLOT_ID: u8 = 0x4;
/// The capability identifier of the message signalled interrupts
/// capability (`PCI_CAP_ID_MSI`).
pub const PCI_CAP_ID_MSI: u8 = 0x5;
/// The capability identifier of the CompactPCI hot swap capability
/// (`PCI_CAP_ID_cPCI_HS`).
pub const PCI_CAP_ID_cPCI_HS: u8 = 0x6;
/// The capability identifier of the PCI-X capability (`PCI_CAP_ID_PCIX`).
pub const PCI_CAP_ID_PCIX: u8 = 0x7;
/// The capability identifier of the HyperTransport capability
/// (`PCI_CAP_ID_HT`).
pub const PCI_CAP_ID_HT: u8 = 0x8;
/// The capability identifier of a vendor specific capability
/// (`PCI_CAP_ID_VS`).
pub const PCI_CAP_ID_VS: u8 = 0x9;
/// The capability identifier of the debug port capability
/// (`PCI_CAP_ID_DEBUG_PORT`).
pub const PCI_CAP_ID_DEBUG_PORT: u8 = 0xA;
/// The capability identifier of the CompactPCI central resource control
/// capability (`PCI_CAP_ID_cPCI_CRC`).
pub const PCI_CAP_ID_cPCI_CRC: u8 = 0xB;
/// The capability identifier of the standard hot plug controller
/// capability (`PCI_CAP_ID_PCI_HOTPLUG`).
pub const PCI_CAP_ID_PCI_HOTPLUG: u8 = 0xC;
/// The capability identifier of the bridge subsystem identifier capability
/// (`PCI_CAP_ID_P2P_SUBSYS`).
pub const PCI_CAP_ID_P2P_SUBSYS: u8 = 0xD;
/// The capability identifier of the AGP 8x capability
/// (`PCI_CAP_ID_AGP_8X`).
pub const PCI_CAP_ID_AGP_8X: u8 = 0xE;
/// The capability identifier of the secure device capability
/// (`PCI_CAP_ID_SECURE_DEV`).
pub const PCI_CAP_ID_SECURE_DEV: u8 = 0xF;
/// The capability identifier of the PCI Express capability
/// (`PCI_CAP_ID_PCI_E`).
pub const PCI_CAP_ID_PCI_E: u8 = 0x10;
/// The capability identifier of the MSI-X capability (`PCI_CAP_ID_MSI_X`).
pub const PCI_CAP_ID_MSI_X: u8 = 0x11;
/// The capability identifier of the SATA index and data configuration
/// capability (`PCI_CAP_ID_SATA`).
pub const PCI_CAP_ID_SATA: u8 = 0x12;
/// The capability identifier of the function level reset capability
/// (`PCI_CAP_ID_FLR`).
pub const PCI_CAP_ID_FLR: u8 = 0x13;
/// The capability identifier of the enhanced allocation capability
/// (`PCI_CAP_ID_EA`).
pub const PCI_CAP_ID_EA: u8 = 0x14;
/// The capability identifier of the flattening portal bridge capability
/// (`PCI_CAP_ID_FPB`).
pub const PCI_CAP_ID_FPB: u8 = 0x15;

// --- power management capability ------------------------------------------------

/// The offset within the power management capability entry of the two byte
/// capabilities register, whose bits are the `PCI_PMCAP_*` values
/// (`PCI_PMCAP`).
pub const PCI_PMCAP: off_t = 0x2;
/// The offset within the power management capability entry of the two byte
/// control and status register, whose bits are the `PCI_PMCSR_*` values
/// (`PCI_PMCSR`).
pub const PCI_PMCSR: off_t = 0x4;
/// The offset within the power management capability entry of the one byte
/// bridge support extensions register (`PCI_PMCSR_BSE`).
pub const PCI_PMCSR_BSE: off_t = 0x6;
/// The offset within the power management capability entry of the one byte
/// data register (`PCI_PMDATA`).
pub const PCI_PMDATA: off_t = 0x7;

/// The capabilities register version field value for revision 1.0 of the
/// PCI power management specification (`PCI_PMCAP_VER_1_0`).
pub const PCI_PMCAP_VER_1_0: u16 = 0x1;
/// The version field value for revision 1.1 (`PCI_PMCAP_VER_1_1`).
pub const PCI_PMCAP_VER_1_1: u16 = 0x2;
/// The version field value for revision 1.2 (`PCI_PMCAP_VER_1_2`).
pub const PCI_PMCAP_VER_1_2: u16 = 0x3;
/// The mask that isolates the version field (`PCI_PMCAP_VER_MASK`).
pub const PCI_PMCAP_VER_MASK: u16 = 0x7;
/// The capabilities bit saying the function needs the PCI clock to
/// generate a power management event (`PCI_PMCAP_PME_CLOCK`).
pub const PCI_PMCAP_PME_CLOCK: u16 = 0x8;
/// The capabilities bit saying the function needs device specific
/// initialisation after a return to D0 (`PCI_PMCAP_DSI`).
pub const PCI_PMCAP_DSI: u16 = 0x20;
/// The auxiliary current field value for a self powered function that
/// draws nothing from the 3.3 volt auxiliary supply
/// (`PCI_PMCAP_AUX_CUR_SELF`).
pub const PCI_PMCAP_AUX_CUR_SELF: u16 = 0x0;
/// The auxiliary current field value for 55 milliamperes
/// (`PCI_PMCAP_AUX_CUR_55mA`).
pub const PCI_PMCAP_AUX_CUR_55mA: u16 = 0x40;
/// The auxiliary current field value for 100 milliamperes
/// (`PCI_PMCAP_AUX_CUR_100mA`).
pub const PCI_PMCAP_AUX_CUR_100mA: u16 = 0x80;
/// The auxiliary current field value for 160 milliamperes
/// (`PCI_PMCAP_AUX_CUR_160mA`).
pub const PCI_PMCAP_AUX_CUR_160mA: u16 = 0xc0;
/// The auxiliary current field value for 220 milliamperes
/// (`PCI_PMCAP_AUX_CUR_220mA`).
pub const PCI_PMCAP_AUX_CUR_220mA: u16 = 0x100;
/// The auxiliary current field value for 270 milliamperes
/// (`PCI_PMCAP_AUX_CUR_270mA`).
pub const PCI_PMCAP_AUX_CUR_270mA: u16 = 0x140;
/// The auxiliary current field value for 320 milliamperes
/// (`PCI_PMCAP_AUX_CUR_320mA`).
pub const PCI_PMCAP_AUX_CUR_320mA: u16 = 0x180;
/// The auxiliary current field value for 375 milliamperes
/// (`PCI_PMCAP_AUX_CUR_375mA`).
pub const PCI_PMCAP_AUX_CUR_375mA: u16 = 0x1c0;
/// The mask that isolates the auxiliary current field
/// (`PCI_PMCAP_AUX_CUR_MASK`).
pub const PCI_PMCAP_AUX_CUR_MASK: u16 = 0x1c0;
/// The capabilities bit saying the function supports the D1 power state
/// (`PCI_PMCAP_D1`).
pub const PCI_PMCAP_D1: u16 = 0x200;
/// The capabilities bit saying the function supports the D2 power state
/// (`PCI_PMCAP_D2`).
pub const PCI_PMCAP_D2: u16 = 0x400;
/// The capabilities bit saying the function can raise a power management
/// event from D0 (`PCI_PMCAP_D0_PME`).
pub const PCI_PMCAP_D0_PME: u16 = 0x800;
/// The capabilities bit saying the function can raise a power management
/// event from D1 (`PCI_PMCAP_D1_PME`).
pub const PCI_PMCAP_D1_PME: u16 = 0x1000;
/// The capabilities bit saying the function can raise a power management
/// event from D2 (`PCI_PMCAP_D2_PME`).
pub const PCI_PMCAP_D2_PME: u16 = 0x2000;
/// The capabilities bit saying the function can raise a power management
/// event from D3hot (`PCI_PMCAP_D3HOT_PME`).
pub const PCI_PMCAP_D3HOT_PME: u16 = 0x4000;
/// The capabilities bit saying the function can raise a power management
/// event from D3cold (`PCI_PMCAP_D3COLD_PME`).
pub const PCI_PMCAP_D3COLD_PME: u16 = 0x8000;
/// The mask that isolates the five power management event support bits
/// (`PCI_PMCAP_PME_MASK`).
pub const PCI_PMCAP_PME_MASK: u16 = 0xf800;

/// The control and status register power state value for D0, the fully
/// operational state (`PCI_PMCSR_D0`).
pub const PCI_PMCSR_D0: u16 = 0x0;
/// The power state value for D1 (`PCI_PMCSR_D1`).
pub const PCI_PMCSR_D1: u16 = 0x1;
/// The power state value for D2 (`PCI_PMCSR_D2`).
pub const PCI_PMCSR_D2: u16 = 0x2;
/// The power state value for D3hot, the lowest state reachable with power
/// still applied; drivers save the header with `pci_save_config_regs`
/// before entering it (`PCI_PMCSR_D3HOT`).
pub const PCI_PMCSR_D3HOT: u16 = 0x3;
/// The mask that isolates the power state field (`PCI_PMCSR_STATE_MASK`).
pub const PCI_PMCSR_STATE_MASK: u16 = 0x3;
/// The control and status bit saying the function keeps its state across a
/// D3hot to D0 transition, so no soft reset occurs
/// (`PCI_PMCSR_NOSOFTRESET`).
pub const PCI_PMCSR_NOSOFTRESET: u16 = 0x8;
/// The control and status bit that enables assertion of the power
/// management event signal (`PCI_PMCSR_PME_EN`).
pub const PCI_PMCSR_PME_EN: u16 = 0x100;
/// The data select value that makes the data register report power
/// consumed in D0 (`PCI_PMCSR_DSEL_D0_PWR_C`).
pub const PCI_PMCSR_DSEL_D0_PWR_C: u16 = 0x0;
/// The data select value for power consumed in D1
/// (`PCI_PMCSR_DSEL_D1_PWR_C`).
pub const PCI_PMCSR_DSEL_D1_PWR_C: u16 = 0x200;
/// The data select value for power consumed in D2
/// (`PCI_PMCSR_DSEL_D2_PWR_C`).
pub const PCI_PMCSR_DSEL_D2_PWR_C: u16 = 0x400;
/// The data select value for power consumed in D3
/// (`PCI_PMCSR_DSEL_D3_PWR_C`).
pub const PCI_PMCSR_DSEL_D3_PWR_C: u16 = 0x600;
/// The data select value for power dissipated in D0
/// (`PCI_PMCSR_DSEL_D0_PWR_D`).
pub const PCI_PMCSR_DSEL_D0_PWR_D: u16 = 0x800;
/// The data select value for power dissipated in D1
/// (`PCI_PMCSR_DSEL_D1_PWR_D`).
pub const PCI_PMCSR_DSEL_D1_PWR_D: u16 = 0xa00;
/// The data select value for power dissipated in D2
/// (`PCI_PMCSR_DSEL_D2_PWR_D`).
pub const PCI_PMCSR_DSEL_D2_PWR_D: u16 = 0xc00;
/// The data select value for power dissipated in D3
/// (`PCI_PMCSR_DSEL_D3_PWR_D`).
pub const PCI_PMCSR_DSEL_D3_PWR_D: u16 = 0xe00;
/// The data select value for the power consumed by the function's common
/// logic (`PCI_PMCSR_DSEL_COM_C`).
pub const PCI_PMCSR_DSEL_COM_C: u16 = 0x1000;
/// The mask that isolates the data select field (`PCI_PMCSR_DSEL_MASK`).
pub const PCI_PMCSR_DSEL_MASK: u16 = 0x1e00;
/// The data scale value meaning the scale is unknown
/// (`PCI_PMCSR_DSCL_UNKNOWN`).
pub const PCI_PMCSR_DSCL_UNKNOWN: u16 = 0x0;
/// The data scale value meaning the data register is in tenths of a watt
/// (`PCI_PMCSR_DSCL_1_BY_10`).
pub const PCI_PMCSR_DSCL_1_BY_10: u16 = 0x2000;
/// The data scale value meaning hundredths of a watt
/// (`PCI_PMCSR_DSCL_1_BY_100`).
pub const PCI_PMCSR_DSCL_1_BY_100: u16 = 0x4000;
/// The data scale value meaning thousandths of a watt
/// (`PCI_PMCSR_DSCL_1_BY_1000`).
pub const PCI_PMCSR_DSCL_1_BY_1000: u16 = 0x6000;
/// The mask that isolates the data scale field (`PCI_PMCSR_DSCL_MASK`).
pub const PCI_PMCSR_DSCL_MASK: u16 = 0x6000;
/// The control and status bit, cleared by writing one, that records an
/// asserted power management event (`PCI_PMCSR_PME_STAT`).
pub const PCI_PMCSR_PME_STAT: u16 = 0x8000;
/// The bridge support extensions bit saying a bridge in D3hot puts its
/// secondary bus into B2 rather than B3 (`PCI_PMCSR_BSE_B2_B3`).
pub const PCI_PMCSR_BSE_B2_B3: u8 = 0x40;
/// The bridge support extensions bit saying bus power and clock control is
/// enabled (`PCI_PMCSR_BSE_BPCC_EN`).
pub const PCI_PMCSR_BSE_BPCC_EN: u8 = 0x80;

// --- pci_report_pmcap -------------------------------------------------------------

/// The capability that `pci_report_pmcap(9F)` accepts, naming the lowest
/// clock speed the device tolerates while idle on a 33 MHz bus
/// (`PCI_PM_IDLESPEED`). The accompanying argument is the speed in
/// kilohertz cast to a pointer, or one of the two special values below.
pub const PCI_PM_IDLESPEED: c_int = 0x1;
/// The `PCI_PM_IDLESPEED` argument saying the device tolerates any idle
/// clock speed; it is the C expression `(void *)-1`
/// (`PCI_PM_IDLESPEED_ANY`).
pub const PCI_PM_IDLESPEED_ANY: *mut c_void = usize::MAX as *mut c_void;
/// The `PCI_PM_IDLESPEED` argument saying the device cannot tolerate the
/// clock slowing at all, which is also what the framework assumes for a
/// driver that never calls `pci_report_pmcap`; it is the C expression
/// `(void *)-2` (`PCI_PM_IDLESPEED_NONE`).
pub const PCI_PM_IDLESPEED_NONE: *mut c_void = (usize::MAX - 1) as *mut c_void;

unsafe extern "C" {
    // --- configuration space handle ----------------------------------------------
    /// `pci_config_setup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets up the resources needed to access the PCI configuration space of
    /// the function that the device node `dip` represents and stores the
    /// resulting access handle through `handle`. The handle is what every
    /// other function in this module that takes a handle expects, and it is
    /// distinct from any register mapping handle the driver holds. The
    /// function returns `DDI_SUCCESS` when the handle was created and
    /// `DDI_FAILURE` when the resources could not be allocated. It must be
    /// called from user or kernel context, and is normally called once in
    /// `attach(9E)`.
    pub fn pci_config_setup(dip: dev_info_t, handle: *mut ddi_acc_handle_t) -> c_int;
    /// `pci_config_teardown(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the resources behind the configuration space handle that
    /// `handle` points to, which must have come from [`pci_config_setup`].
    /// The handle is invalid afterwards. It may be called from any context
    /// and is normally called in `detach(9E)`.
    pub fn pci_config_teardown(handle: *mut ddi_acc_handle_t);

    // --- single datum accessors ------------------------------------------------------
    /// `pci_config_get8(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads one byte from the configuration space that `handle` gives
    /// access to, at byte `offset` from the start of the space. The
    /// `PCI_CONF_*` constants in this module name the offsets of the header
    /// registers. Configuration space is little-endian, so the wider
    /// variants translate to host order; for a single byte there is nothing
    /// to translate. The value read is returned. [`pci_config_setup`] must
    /// have been called first.
    ///
    /// The page names `pci_config_getb`, `pci_config_getw`,
    /// `pci_config_getl` and `pci_config_getll` as the obsolete spellings
    /// that `pci_config_get8`, `pci_config_get16`, `pci_config_get32` and
    /// `pci_config_get64` replace. The function may be called from user,
    /// kernel or interrupt context.
    pub fn pci_config_get8(handle: ddi_acc_handle_t, offset: off_t) -> u8;
    /// `pci_config_get16(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads a 16-bit value from configuration space at byte `offset`,
    /// which must be a multiple of two, through `handle`, translating from
    /// the little-endian layout of the space to host order. The value read
    /// is returned. This is the replacement for the obsolete
    /// `pci_config_getw`. It may be called from user, kernel or interrupt
    /// context.
    pub fn pci_config_get16(handle: ddi_acc_handle_t, offset: off_t) -> u16;
    /// `pci_config_get32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads a 32-bit value from configuration space at byte `offset`,
    /// which must be a multiple of four, through `handle`, translating from
    /// little-endian to host order. The value read is returned. This is the
    /// replacement for the obsolete `pci_config_getl`. It may be called
    /// from user, kernel or interrupt context.
    pub fn pci_config_get32(handle: ddi_acc_handle_t, offset: off_t) -> u32;
    /// `pci_config_get64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads a 64-bit value from configuration space at byte `offset`,
    /// which must be a multiple of eight, through `handle`, translating
    /// from little-endian to host order. The value read is returned. This
    /// is the replacement for the obsolete `pci_config_getll`. It may be
    /// called from user, kernel or interrupt context.
    pub fn pci_config_get64(handle: ddi_acc_handle_t, offset: off_t) -> u64;
    /// `pci_config_put8(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Writes the byte `value` to configuration space at byte `offset`
    /// through `handle`. The wider variants translate from host order to
    /// the little-endian layout of the space before writing. Nothing is
    /// returned. [`pci_config_setup`] must have been called first.
    ///
    /// The page names `pci_config_putb`, `pci_config_putw`,
    /// `pci_config_putl` and `pci_config_putll` as the obsolete spellings
    /// that `pci_config_put8`, `pci_config_put16`, `pci_config_put32` and
    /// `pci_config_put64` replace. The function may be called from user,
    /// kernel or interrupt context.
    pub fn pci_config_put8(handle: ddi_acc_handle_t, offset: off_t, value: u8);
    /// `pci_config_put16(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Writes the 16-bit `value` to configuration space at byte `offset`,
    /// which must be a multiple of two, through `handle`, translating from
    /// host order to little-endian. This is the replacement for the
    /// obsolete `pci_config_putw`. It may be called from user, kernel or
    /// interrupt context.
    pub fn pci_config_put16(handle: ddi_acc_handle_t, offset: off_t, value: u16);
    /// `pci_config_put32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Writes the 32-bit `value` to configuration space at byte `offset`,
    /// which must be a multiple of four, through `handle`, translating from
    /// host order to little-endian. This is the replacement for the
    /// obsolete `pci_config_putl`. It may be called from user, kernel or
    /// interrupt context.
    pub fn pci_config_put32(handle: ddi_acc_handle_t, offset: off_t, value: u32);
    /// `pci_config_put64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Writes the 64-bit `value` to configuration space at byte `offset`,
    /// which must be a multiple of eight, through `handle`, translating
    /// from host order to little-endian. This is the replacement for the
    /// obsolete `pci_config_putll`. It may be called from user, kernel or
    /// interrupt context.
    pub fn pci_config_put64(handle: ddi_acc_handle_t, offset: off_t, value: u64);

    // --- capability list -----------------------------------------------------------------
    /// `pci_lcap_locate`. No manual page; listed in `uts/common/mapfiles/ddi.mapfile`.
    ///
    /// Searches the capability list in the conventional configuration space
    /// reachable through `h` for the capability whose identifier is `id`,
    /// one of the `PCI_CAP_ID_*` constants. It first checks
    /// [`PCI_STAT_CAP`] in the status register, then picks the list head
    /// from [`PCI_CONF_CAP_PTR`], [`PCI_BCNF_CAP_PTR`] or
    /// [`PCI_CBUS_CAP_PTR`] according to the header type, and follows the
    /// next pointers for at most [`PCI_CAP_MAX_PTR`] entries. When a match
    /// is found its offset is stored through `base_p` and `DDI_SUCCESS` is
    /// returned; the capability's registers then start at that offset plus
    /// [`PCI_CAP_ID_REGS_OFF`]. When the function has no capability list,
    /// the header type is unrecognised, or the identifier is absent,
    /// [`PCI_CAP_NEXT_PTR_NULL`] is stored through `base_p` in the last case
    /// and `DDI_FAILURE` is returned.
    ///
    /// `sys/pci_cap.h` has no manual page; this description follows the
    /// implementation in `uts/common/io/pci_cap.c`. The function reads
    /// configuration space through `pci_config_get8` and
    /// `pci_config_get16` and inherits their calling context rules.
    pub fn pci_lcap_locate(h: ddi_acc_handle_t, id: u8, base_p: *mut u16) -> c_int;

    // --- power management --------------------------------------------------------------------
    /// `pci_report_pmcap(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports a power management capability of the device node `dip` that
    /// goes beyond what the PCI power management specification describes.
    /// The only capability defined is [`PCI_PM_IDLESPEED`], for which `arg`
    /// is the lowest clock speed in kilohertz that the device tolerates
    /// while idle on a 33 MHz bus, cast to a pointer, or
    /// [`PCI_PM_IDLESPEED_ANY`] when any speed is acceptable, or
    /// [`PCI_PM_IDLESPEED_NONE`] when the clock must not slow at all. A
    /// driver that never calls this function is treated as having reported
    /// `PCI_PM_IDLESPEED_NONE`, and one such device keeps the whole bus
    /// from being power managed. The capability may be reported again
    /// whenever it changes.
    ///
    /// The function returns `DDI_SUCCESS` when the report was accepted and
    /// `DDI_FAILURE` when `cap` or `arg` is invalid. It may be called from
    /// user, kernel or interrupt context.
    pub fn pci_report_pmcap(dip: dev_info_t, cap: c_int, arg: *mut c_void) -> c_int;
    /// `pci_save_config_regs(9F)`. illumos DDI-specific (illumos DDI).
    ///
    /// Copies the configuration registers of the device node `dip` into
    /// memory that persists across a power transition, so that they can be
    /// put back by [`pci_restore_config_regs`]. A driver calls it from its
    /// `power(9E)` entry point before moving the device to D3, because the
    /// header contents are lost in that state. The function returns
    /// `DDI_SUCCESS` when the registers were saved and `DDI_FAILURE`
    /// otherwise. It must be called from user or kernel context.
    pub fn pci_save_config_regs(dip: dev_info_t) -> c_int;
    /// `pci_restore_config_regs(9F)`. illumos DDI-specific (illumos DDI).
    ///
    /// Writes back the configuration registers of the device node `dip`
    /// that [`pci_save_config_regs`] saved. A driver calls it from
    /// `power(9E)` after bringing the device out of D3 and before touching
    /// it in any other way, since the base address registers and command
    /// register must be restored before the device can be reached. The
    /// function returns `DDI_SUCCESS` when the registers were restored and
    /// `DDI_FAILURE` otherwise. It must be called from user or kernel
    /// context.
    pub fn pci_restore_config_regs(dip: dev_info_t) -> c_int;

    // --- error reporting -------------------------------------------------------------------------
    /// `pci_ereport_setup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Initialises error report generation for the device node `dip` and
    /// sets up the resources that [`pci_ereport_post`] will use to reach
    /// the PCI, PCI-X or PCI Express configuration space of the function.
    /// The driver must already have obtained at least
    /// `DDI_FM_EREPORT_CAPABLE` from [`crate::fm::ddi_fm_init`] for this
    /// node. The function returns nothing and must be called from user or
    /// kernel context, normally in `attach(9E)`.
    pub fn pci_ereport_setup(dip: dev_info_t);
    /// `pci_ereport_teardown(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases the resources that [`pci_ereport_setup`] allocated for the
    /// device node `dip`. It returns nothing and must be called from user or
    /// kernel context, normally in `detach(9E)` before
    /// [`crate::fm::ddi_fm_fini`].
    pub fn pci_ereport_teardown(dip: dev_info_t);
    /// `pci_ereport_post(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Scans the error registers of the function that `dip` represents and
    /// posts an error report for each PCI, PCI-X or PCI Express error it
    /// finds. On a conventional PCI bus that covers detected parity errors,
    /// master data parity errors, target aborts, master aborts, system
    /// errors and discard timeouts. The function must only be called from
    /// the error handler registered with
    /// [`crate::fm::ddi_fm_handler_register`], and `derr` must be the error
    /// status the handler received, because it may carry bus specific
    /// detail from the nexus. Reports are generated only when `fme_flag` is
    /// `DDI_FM_ERR_UNEXPECTED`; for the expected, peek and poke flags the
    /// registers are cleared without reporting.
    ///
    /// If `status` is not NULL the contents of the configuration status
    /// register are stored through it, or `0xffff` if the register could
    /// not be read. On return `fme_status` in `derr` holds `DDI_FM_OK`,
    /// `DDI_FM_FATAL`, `DDI_FM_NONFATAL` or `DDI_FM_UNKNOWN`, and for a
    /// non-fatal error `fme_acc_handle` or `fme_dma_handle` typically
    /// names the affected handle. The usual error handler returns that
    /// status directly. The function returns nothing and may be called from
    /// any context.
    pub fn pci_ereport_post(dip: dev_info_t, derr: *mut ddi_fm_error_t, status: *mut u16);
}
