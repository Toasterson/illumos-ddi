//! The core of `sys/sunddi.h`.
//!
//! This is the part of the main DDI header that every leaf driver touches:
//! the `DDI_SUCCESS` and `DDI_FAILURE` return codes, device and instance
//! numbers, the names and state of a device node, minor nodes, the
//! soft-state table, driver-private data, the functions that move data
//! across the user-kernel boundary, and a set of small utilities for
//! strings, bit scanning, page arithmetic, critical sections, process
//! identity and signalling. Properties, register access, interrupts, DMA,
//! power management, user memory, events and device identifiers live in
//! their own modules.

use crate::types::*;

/// The success return value of `dev_ops(9S)` entry points and of the
/// `ddi_*` functions that use this convention (`DDI_SUCCESS`).
pub const DDI_SUCCESS: c_int = 0;
/// The failure return value in the same convention (`DDI_FAILURE`).
pub const DDI_FAILURE: c_int = -1;

/// The node type string for pseudo devices (`DDI_PSEUDO`). Node types tell
/// `devfsadm` how to create `/dev` links for a minor node.
pub const DDI_PSEUDO: &core::ffi::CStr = c"ddi_pseudo";
/// The node type string for serial ports (`DDI_NT_SERIAL`).
pub const DDI_NT_SERIAL: &core::ffi::CStr = c"ddi_serial";
/// The node type string for built-in serial ports on the motherboard (`DDI_NT_SERIAL_MB`).
pub const DDI_NT_SERIAL_MB: &core::ffi::CStr = c"ddi_serial:mb";
/// The node type string for dial-out serial ports (`DDI_NT_SERIAL_DO`).
pub const DDI_NT_SERIAL_DO: &core::ffi::CStr = c"ddi_serial:dialout";
/// The node type string for dial-out ports of the built-in serial ports (`DDI_NT_SERIAL_MB_DO`).
pub const DDI_NT_SERIAL_MB_DO: &core::ffi::CStr = c"ddi_serial:dialout,mb";
/// The node type string for the LOMlite2 console port (`DDI_NT_SERIAL_LOMCON`).
pub const DDI_NT_SERIAL_LOMCON: &core::ffi::CStr = c"ddi_serial:lomcon";
/// The node type string for hard disks (`DDI_NT_BLOCK`).
pub const DDI_NT_BLOCK: &core::ffi::CStr = c"ddi_block";
/// The node type string for disks addressed by controller channel (`DDI_NT_BLOCK_CHAN`).
pub const DDI_NT_BLOCK_CHAN: &core::ffi::CStr = c"ddi_block:channel";
/// The node type string for disks addressed by world wide name (`DDI_NT_BLOCK_WWN`).
pub const DDI_NT_BLOCK_WWN: &core::ffi::CStr = c"ddi_block:wwn";
/// The node type string for CD-ROM and other optical drives (`DDI_NT_CD`).
pub const DDI_NT_CD: &core::ffi::CStr = c"ddi_block:cdrom";
/// The node type string for optical drives addressed by channel, as SCSI drives are (`DDI_NT_CD_CHAN`).
pub const DDI_NT_CD_CHAN: &core::ffi::CStr = c"ddi_block:cdrom:channel";
/// The node type string for floppy disks (`DDI_NT_FD`).
pub const DDI_NT_FD: &core::ffi::CStr = c"ddi_block:diskette";
/// The node type string for storage enclosures (`DDI_NT_ENCLOSURE`).
pub const DDI_NT_ENCLOSURE: &core::ffi::CStr = c"ddi_enclosure";
/// The node type string for SCSI storage enclosures (`DDI_NT_SCSI_ENCLOSURE`).
pub const DDI_NT_SCSI_ENCLOSURE: &core::ffi::CStr = c"ddi_enclosure:scsi";
/// The node type string for SAS disks (`DDI_NT_BLOCK_SAS`).
pub const DDI_NT_BLOCK_SAS: &core::ffi::CStr = c"ddi_block:sas";
/// The node type string for disks driven through the blkdev framework (`DDI_NT_BLOCK_BLKDEV`).
pub const DDI_NT_BLOCK_BLKDEV: &core::ffi::CStr = c"ddi_block:blkdev";
/// The node type string for Xen virtual disks (`DDI_NT_BLOCK_XVMD`).
pub const DDI_NT_BLOCK_XVMD: &core::ffi::CStr = c"ddi_block:xvmd";
/// The node type string for Xen virtual optical drives (`DDI_NT_CD_XVMD`).
pub const DDI_NT_CD_XVMD: &core::ffi::CStr = c"ddi_block:cdrom:xvmd";
/// The node type string for tape drives (`DDI_NT_TAPE`).
pub const DDI_NT_TAPE: &core::ffi::CStr = c"ddi_byte:tape";
/// The node type string for DLPI network devices (`DDI_NT_NET`).
pub const DDI_NT_NET: &core::ffi::CStr = c"ddi_network";
/// The node type string for wireless network devices (`DDI_NT_NET_WIFI`).
pub const DDI_NT_NET_WIFI: &core::ffi::CStr = c"ddi_network:wifi";
/// The node type string for display devices (`DDI_NT_DISPLAY`).
pub const DDI_NT_DISPLAY: &core::ffi::CStr = c"ddi_display";
/// The node type string for DRM display devices (`DDI_NT_DISPLAY_DRM`).
pub const DDI_NT_DISPLAY_DRM: &core::ffi::CStr = c"ddi_display:drm";
/// The node type string for audio devices (`DDI_NT_AUDIO`).
pub const DDI_NT_AUDIO: &core::ffi::CStr = c"ddi_audio";
/// The node type string for mice (`DDI_NT_MOUSE`).
pub const DDI_NT_MOUSE: &core::ffi::CStr = c"ddi_mouse";
/// The node type string for keyboards (`DDI_NT_KEYBOARD`).
pub const DDI_NT_KEYBOARD: &core::ffi::CStr = c"ddi_keyboard";
/// The node type string for parallel ports (`DDI_NT_PARALLEL`).
pub const DDI_NT_PARALLEL: &core::ffi::CStr = c"ddi_parallel";
/// The node type string for printers (`DDI_NT_PRINTER`).
pub const DDI_NT_PRINTER: &core::ffi::CStr = c"ddi_printer";
/// The node type string for the generic USB driver (`DDI_NT_UGEN`).
pub const DDI_NT_UGEN: &core::ffi::CStr = c"ddi_generic:usb";
/// The node type string for SAS SMP targets (`DDI_NT_SMP`).
pub const DDI_NT_SMP: &core::ffi::CStr = c"ddi_sas_smp";
/// The node type string for EEPROM devices (`DDI_NT_EEPROM`).
pub const DDI_NT_EEPROM: &core::ffi::CStr = c"ddi_eeprom";
/// The node type string for the devctl node of nexus drivers (`DDI_NT_NEXUS`).
pub const DDI_NT_NEXUS: &core::ffi::CStr = c"ddi_ctl:devctl";
/// The node type string for the devctl node of the NVMe nexus (`DDI_NT_NVME_NEXUS`).
pub const DDI_NT_NVME_NEXUS: &core::ffi::CStr = c"ddi_ctl:devctl:nvme";
/// The node type string for the devctl node of SCSI nexus drivers (`DDI_NT_SCSI_NEXUS`).
pub const DDI_NT_SCSI_NEXUS: &core::ffi::CStr = c"ddi_ctl:devctl:scsi";
/// The node type string for the devctl node of SATA nexus drivers (`DDI_NT_SATA_NEXUS`).
pub const DDI_NT_SATA_NEXUS: &core::ffi::CStr = c"ddi_ctl:devctl:sata";
/// The node type string for the devctl node of InfiniBand nexus drivers (`DDI_NT_IB_NEXUS`).
pub const DDI_NT_IB_NEXUS: &core::ffi::CStr = c"ddi_ctl:devctl:ib";
/// The node type string for generic hot-plug attachment points (`DDI_NT_ATTACHMENT_POINT`).
pub const DDI_NT_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point";
/// The node type string for NVMe attachment points (`DDI_NT_NVME_ATTACHMENT_POINT`).
pub const DDI_NT_NVME_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:nvme";
/// The node type string for SCSI attachment points (`DDI_NT_SCSI_ATTACHMENT_POINT`).
pub const DDI_NT_SCSI_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:scsi";
/// The node type string for SATA attachment points (`DDI_NT_SATA_ATTACHMENT_POINT`).
pub const DDI_NT_SATA_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:sata";
/// The node type string for SD card attachment points (`DDI_NT_SDCARD_ATTACHMENT_POINT`).
pub const DDI_NT_SDCARD_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:sdcard";
/// The node type string for PCI hot-plug attachment points (`DDI_NT_PCI_ATTACHMENT_POINT`).
pub const DDI_NT_PCI_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:pci";
/// The node type string for system board attachment points (`DDI_NT_SBD_ATTACHMENT_POINT`).
pub const DDI_NT_SBD_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:sbd";
/// The node type string for Fibre Channel attachment points (`DDI_NT_FC_ATTACHMENT_POINT`).
pub const DDI_NT_FC_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:fc";
/// The node type string for USB attachment points (`DDI_NT_USB_ATTACHMENT_POINT`).
pub const DDI_NT_USB_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:usb";
/// The node type string for Fibre Channel fabric disks (`DDI_NT_BLOCK_FABRIC`).
pub const DDI_NT_BLOCK_FABRIC: &core::ffi::CStr = c"ddi_block:fabric";
/// The node type string for InfiniBand attachment points (`DDI_NT_IB_ATTACHMENT_POINT`).
pub const DDI_NT_IB_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:ib";
/// The node type string for smart card reader slots (`DDI_NT_CCID_ATTACHMENT_POINT`).
pub const DDI_NT_CCID_ATTACHMENT_POINT: &core::ffi::CStr = c"ddi_ctl:attachment_point:ccid";
/// The node type string for asynchronous audio and video devices (`DDI_NT_AV_ASYNC`).
pub const DDI_NT_AV_ASYNC: &core::ffi::CStr = c"ddi_av:async";
/// The node type string for isochronous audio and video devices (`DDI_NT_AV_ISOCH`).
pub const DDI_NT_AV_ISOCH: &core::ffi::CStr = c"ddi_av:isoch";
/// The node type string for the agpgart pseudo device (`DDI_NT_AGP_PSEUDO`).
pub const DDI_NT_AGP_PSEUDO: &core::ffi::CStr = c"ddi_agp:pseudo";
/// The node type string for AGP master devices (`DDI_NT_AGP_MASTER`).
pub const DDI_NT_AGP_MASTER: &core::ffi::CStr = c"ddi_agp:master";
/// The node type string for AGP target devices (`DDI_NT_AGP_TARGET`).
pub const DDI_NT_AGP_TARGET: &core::ffi::CStr = c"ddi_agp:target";
/// The node type string for the on-CPU GART of AMD64 processors (`DDI_NT_AGP_CPUGART`).
pub const DDI_NT_AGP_CPUGART: &core::ffi::CStr = c"ddi_agp:cpugart";
/// The node type string for the register access tool node (`DDI_NT_REGACC`).
pub const DDI_NT_REGACC: &core::ffi::CStr = c"ddi_tool_reg";
/// The node type string for the interrupt control tool node (`DDI_NT_INTRCTL`).
pub const DDI_NT_INTRCTL: &core::ffi::CStr = c"ddi_tool_intr";
/// The node type string for CPU temperature sensors, as used by `ksensor_create(9F)` (`DDI_NT_SENSOR_TEMP_CPU`).
pub const DDI_NT_SENSOR_TEMP_CPU: &core::ffi::CStr = c"ddi_sensor:temperature:cpu";
/// The node type string for chipset (PCH) temperature sensors, as used by `ksensor_create(9F)` (`DDI_NT_SENSOR_TEMP_PCH`).
pub const DDI_NT_SENSOR_TEMP_PCH: &core::ffi::CStr = c"ddi_sensor:temperature:pch";
/// The node type string for GPIO controllers (`DDI_NT_GPIO_CTRL`).
pub const DDI_NT_GPIO_CTRL: &core::ffi::CStr = c"ddi_gpio:controller";
/// The node type string for dedicated-purpose I/O pins (`DDI_NT_GPIO_DPIO`).
pub const DDI_NT_GPIO_DPIO: &core::ffi::CStr = c"ddi_gpio:dpio";
/// The node type string for I2C controllers (`DDI_NT_I2C_CTRL`).
pub const DDI_NT_I2C_CTRL: &core::ffi::CStr = c"ddi_i2c:controller";
/// The node type string for I2C ports (`DDI_NT_I2C_PORT`).
pub const DDI_NT_I2C_PORT: &core::ffi::CStr = c"ddi_i2c:port";
/// The node type string for I2C devices (`DDI_NT_I2C_DEV`).
pub const DDI_NT_I2C_DEV: &core::ffi::CStr = c"ddi_i2c:device";
/// The node type string for I2C multiplexers (`DDI_NT_I2C_MUX`).
pub const DDI_NT_I2C_MUX: &core::ffi::CStr = c"ddi_i2c:mux";

/// The C `pid_t`, a process ID. It is an `int`.
pub type pid_t = c_int;
/// The C `kt_did_t`, the unique identifier of a kernel thread. It is a
/// `uint64_t`.
pub type kt_did_t = u64;
/// The C `time_t`, a time in seconds. It is a `long`.
pub type time_t = c_long;
/// The C `ushort_t`, an unsigned 16-bit integer.
pub type ushort_t = c_ushort;
/// The C `u_longlong_t`, an unsigned 64-bit integer.
pub type u_longlong_t = c_ulonglong;

/// The state of a device as returned by `ddi_get_devstate(9F)`,
/// `ddi_devstate_t`. The values are ordered but not consecutive so that
/// further states can be interpolated later.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ddi_devstate_t(pub c_int);
impl ddi_devstate_t {
    /// The device is offline; the driver is not attached and will not be
    /// attached automatically, and the device cannot be used until it is
    /// brought online (`DDI_DEVSTATE_OFFLINE`).
    pub const DDI_DEVSTATE_OFFLINE: Self = Self(-32);
    /// The device is online but unusable because of a fault
    /// (`DDI_DEVSTATE_DOWN`).
    pub const DDI_DEVSTATE_DOWN: Self = Self(-16);
    /// The bus the device sits on has been quiesced; this is not a fault, but
    /// no operation should be performed on the device until the bus is
    /// unquiesced (`DDI_DEVSTATE_QUIESCED`).
    pub const DDI_DEVSTATE_QUIESCED: Self = Self(0);
    /// The device is online but can only provide partial or degraded service
    /// because of a fault (`DDI_DEVSTATE_DEGRADED`).
    pub const DDI_DEVSTATE_DEGRADED: Self = Self(16);
    /// The device is online and fully operational (`DDI_DEVSTATE_UP`).
    pub const DDI_DEVSTATE_UP: Self = Self(32);
}

/// The impact of a reported fault on the service a device provides,
/// `ddi_fault_impact_t`, passed to `ddi_dev_report_fault(9F)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ddi_fault_impact_t(pub c_int);
impl ddi_fault_impact_t {
    /// Total loss of service; the driver cannot perform the normal functions
    /// of its hardware (`DDI_SERVICE_LOST`).
    pub const DDI_SERVICE_LOST: Self = Self(-32);
    /// Partial or degraded service; the driver may need repeated attempts or
    /// run below its configured speed, and an alternative device should be
    /// used if one exists (`DDI_SERVICE_DEGRADED`).
    pub const DDI_SERVICE_DEGRADED: Self = Self(-16);
    /// Service is currently unaffected; used to report recovered errors for
    /// predictive failure analysis (`DDI_SERVICE_UNAFFECTED`).
    pub const DDI_SERVICE_UNAFFECTED: Self = Self(0);
    /// Normal service has resumed after an earlier report of lost or degraded
    /// service, so any previously reported fault no longer exists
    /// (`DDI_SERVICE_RESTORED`).
    pub const DDI_SERVICE_RESTORED: Self = Self(16);
}

/// The location of a reported fault relative to the hardware the driver
/// controls, `ddi_fault_location_t`, passed to `ddi_dev_report_fault(9F)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ddi_fault_location_t(pub c_int);
impl ddi_fault_location_t {
    /// The fault lies in the path between the driver and the device, for
    /// example because the device is unplugged or the bus is faulty; this is
    /// the value to use when the device does not respond or
    /// `ddi_check_acc_handle(9F)` fails (`DDI_DATAPATH_FAULT`).
    pub const DDI_DATAPATH_FAULT: Self = Self(-32);
    /// The fault lies in the device itself, which is present and accessible
    /// but not working correctly (`DDI_DEVICE_FAULT`).
    pub const DDI_DEVICE_FAULT: Self = Self(-16);
    /// The fault is external to the device, such as a cable fault reported
    /// by an Ethernet driver (`DDI_EXTERNAL_FAULT`).
    pub const DDI_EXTERNAL_FAULT: Self = Self(0);
}

/// One-byte data words for `ddi_device_copy(9F)` and `ddi_device_zero(9F)`
/// (`DDI_DATA_SZ01_ACC`).
pub const DDI_DATA_SZ01_ACC: uint_t = 1;
/// Two-byte data words for `ddi_device_copy(9F)` and `ddi_device_zero(9F)`
/// (`DDI_DATA_SZ02_ACC`).
pub const DDI_DATA_SZ02_ACC: uint_t = 2;
/// Four-byte data words for `ddi_device_copy(9F)` and `ddi_device_zero(9F)`
/// (`DDI_DATA_SZ04_ACC`).
pub const DDI_DATA_SZ04_ACC: uint_t = 4;
/// Eight-byte data words for `ddi_device_copy(9F)` and `ddi_device_zero(9F)`
/// (`DDI_DATA_SZ08_ACC`).
pub const DDI_DATA_SZ08_ACC: uint_t = 8;

/// Asks `drv_getparm(9F)` for the time in seconds, stored as a `time_t`
/// (`TIME`).
pub const TIME: uint_t = 1;
/// Asks `drv_getparm(9F)` for the process table token of the caller, stored
/// as a pointer (`UPROCP`).
pub const UPROCP: uint_t = 2;
/// Asks `drv_getparm(9F)` for the process group ID of the caller, which
/// determines which processes receive a hangup or break signal detected by
/// a driver, stored as a `pid_t` (`PPGRP`).
pub const PPGRP: uint_t = 3;
/// Asks `drv_getparm(9F)` for the value of `lbolt`, the count of clock ticks
/// since boot, stored as a `clock_t` (`LBOLT`). The counter is allowed to
/// wrap, so only differences between two readings are meaningful.
pub const LBOLT: uint_t = 4;
/// Asks `drv_getparm(9F)` for the process ID of the caller, stored as a
/// `pid_t` (`PPID`).
pub const PPID: uint_t = 11;
/// Asks `drv_getparm(9F)` for the session ID of the caller, stored as a
/// `pid_t` (`PSID`).
pub const PSID: uint_t = 12;
/// Asks `drv_getparm(9F)` for a pointer to the caller's credential
/// structure (`UCRED`).
pub const UCRED: uint_t = 13;

unsafe extern "C" {
    // --- device numbers (sys/mkdev.h) --------------------------------------
    /// `getmajor(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Extracts the major number from a `dev_t`.
    pub fn getmajor(dev: dev_t) -> major_t;
    /// `getminor(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Extracts the minor number from a `dev_t`.
    pub fn getminor(dev: dev_t) -> minor_t;
    /// `makedevice(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Combines a major and a minor number into a `dev_t`.
    pub fn makedevice(maj: major_t, min: minor_t) -> dev_t;

    // --- dev_info_t --------------------------------------------------------
    /// `ddi_get_instance(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the instance number of a device node. Instance numbers are
    /// stable across reboots for a given device.
    pub fn ddi_get_instance(dip: dev_info_t) -> c_int;
    /// `ddi_driver_name(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the name of the driver bound to a device node.
    pub fn ddi_driver_name(dip: dev_info_t) -> *const c_char;
    /// `ddi_driver_major(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the major number of the driver bound to a device node.
    pub fn ddi_driver_major(dip: dev_info_t) -> major_t;
    /// `ddi_get_parent(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the parent of a device node in the device tree.
    pub fn ddi_get_parent(dip: dev_info_t) -> dev_info_t;
    /// `ddi_report_dev(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Logs the line that announces an attached instance, in the form
    /// `rnull0 is /pseudo/rnull@0`. Conventionally called at the end of a
    /// successful `attach(9E)`.
    pub fn ddi_report_dev(dip: dev_info_t);
    /// `ddi_set_driver_private(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores one pointer of driver-private data in the device node.
    pub fn ddi_set_driver_private(dip: dev_info_t, data: *mut c_void);
    /// `ddi_get_driver_private(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Retrieves the pointer stored with [`ddi_set_driver_private`].
    pub fn ddi_get_driver_private(dip: dev_info_t) -> *mut c_void;
    /// Listed in the `ddi` mapfile. There is no manual page of its own.
    ///
    /// A `quiesce(9E)` implementation for drivers that have no hardware to
    /// quiesce before a fast reboot. It always succeeds. Pseudo drivers put
    /// it in `devo_quiesce` so that their presence does not disable fast
    /// reboot.
    pub fn ddi_quiesce_not_needed(dip: dev_info_t) -> c_int;
    /// Listed in the `ddi` mapfile. There is no manual page of its own.
    ///
    /// A `quiesce(9E)` implementation for drivers that cannot quiesce their
    /// hardware. Always fails, which disables fast reboot while the driver is
    /// loaded.
    pub fn ddi_quiesce_not_supported(dip: dev_info_t) -> c_int;

    // --- minor nodes -------------------------------------------------------
    /// `ddi_create_minor_node(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates a minor node named `name` under the device node. `spec_type`
    /// is `S_IFCHR` or `S_IFBLK`, `node_type` is one of the `DDI_NT_*`
    /// strings, and `flag` is 0 for ordinary nodes. Returns `DDI_SUCCESS` or
    /// `DDI_FAILURE`.
    pub fn ddi_create_minor_node(
        dip: dev_info_t,
        name: *const c_char,
        spec_type: c_int,
        minor_num: minor_t,
        node_type: *const c_char,
        flag: c_int,
    ) -> c_int;
    /// `ddi_remove_minor_node(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the minor node named `name`, or every minor node of the device
    /// when `name` is NULL.
    pub fn ddi_remove_minor_node(dip: dev_info_t, name: *const c_char);

    // --- soft state --------------------------------------------------------
    /// `ddi_soft_state_init(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Creates a soft-state table whose items are `size` bytes each. The
    /// `n_items` argument is only a hint for the initial capacity; the table
    /// grows on demand. Called from `_init(9E)`.
    pub fn ddi_soft_state_init(state_p: *mut *mut c_void, size: size_t, n_items: size_t) -> c_int;
    /// `ddi_soft_state_fini(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Destroys a soft-state table. Called from `_fini(9E)`.
    pub fn ddi_soft_state_fini(state_p: *mut *mut c_void);
    /// `ddi_soft_state_zalloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates the zeroed item for index `item`, normally the instance
    /// number. Returns `DDI_SUCCESS` or `DDI_FAILURE`.
    pub fn ddi_soft_state_zalloc(state: *mut c_void, item: c_int) -> c_int;
    /// `ddi_soft_state_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the item for index `item`.
    pub fn ddi_soft_state_free(state: *mut c_void, item: c_int);
    /// `ddi_get_soft_state(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the item for index `item`, or NULL if none has been allocated.
    pub fn ddi_get_soft_state(state: *mut c_void, item: c_int) -> *mut c_void;

    // --- user memory -------------------------------------------------------
    /// `ddi_copyin(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies `cn` bytes from the caller's buffer `buf` into the kernel
    /// buffer `driverbuf`. `flags` is the `mode` argument of `ioctl(9E)`;
    /// when it contains `FKIOCTL` the source is already in kernel space and
    /// a plain copy is performed. Returns 0 on success and -1 on a fault.
    pub fn ddi_copyin(buf: *const c_void, driverbuf: *mut c_void, cn: size_t, flags: c_int) -> c_int;
    /// `ddi_copyout(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The counterpart of [`ddi_copyin`], copying from the kernel buffer to
    /// the caller's buffer. Returns 0 on success and -1 on a fault.
    pub fn ddi_copyout(driverbuf: *const c_void, buf: *mut c_void, cn: size_t, flags: c_int) -> c_int;

    // --- misc ------------------------------------------------------------
    /// `ddi_in_panic(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns non-zero while the system is panicking. Drivers use it to
    /// avoid blocking operations on the way to a crash dump.
    pub fn ddi_in_panic() -> c_int;

    // --- node names --------------------------------------------------------
    /// `ddi_binding_name(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the driver binding name of the device node, the name used to
    /// select a driver for the device. It is typically derived from the
    /// node's `name` or `compatible` property and may be a driver alias or
    /// the driver name itself. May be called from user, kernel or interrupt
    /// context.
    pub fn ddi_binding_name(dip: dev_info_t) -> *mut c_char;
    /// `ddi_get_name(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A synonym for [`ddi_binding_name`], documented on the same page. It
    /// returns the driver binding name of the device node and may be called
    /// from user, kernel or interrupt context.
    pub fn ddi_get_name(dip: dev_info_t) -> *mut c_char;
    /// `ddi_node_name(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the device node name stored in the node, which is the `name`
    /// property rather than the binding name. May be called from user,
    /// kernel or interrupt context.
    pub fn ddi_node_name(dip: dev_info_t) -> *mut c_char;
    /// `ddi_name_to_major`. Not documented in section 9; exported by
    /// `uts/common/mapfiles/ddi.mapfile`.
    ///
    /// Returns the major number of the driver called `name`, or the
    /// `(major_t)-1` value that `ddipropdefs.h` names `DDI_MAJOR_T_NONE`
    /// when no such driver is known to the system.
    pub fn ddi_name_to_major(name: *const c_char) -> major_t;
    /// `ddi_root_node(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the root node of the device information tree. May be called
    /// from user, kernel or interrupt context.
    pub fn ddi_root_node() -> dev_info_t;

    // --- device state and faults ------------------------------------------
    /// `ddi_get_devstate(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the state of the device, derived from the configuration
    /// operations performed on it or on its bus and from any fault reports
    /// relating to it. A driver should check its own state at each major
    /// entry point and before committing resources to an operation; if the
    /// device is down it should clean up and return an error such as `EIO`,
    /// and if the bus is quiesced it may return `EAGAIN` or queue the
    /// operation until the bus is unquiesced. May be called from user,
    /// kernel or interrupt context.
    pub fn ddi_get_devstate(dip: dev_info_t) -> ddi_devstate_t;
    /// `ddi_dev_report_fault(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports a hardware fault through the standard mechanism, so that a
    /// system with fault management can respond, for example by failing
    /// over to another device or scheduling a replacement. The `impact`
    /// argument states how the fault affects the service the driver
    /// provides, `location` states where the fault lies relative to the
    /// driver's hardware, and `message` describes it. When a device returns
    /// detectably bad data the driver should first check the access or DMA
    /// handle and report a datapath fault rather than a device fault if the
    /// handle is at fault. The call may update the device state reported by
    /// [`ddi_get_devstate`]. A driver that finds its device already down
    /// should not report another fault, and the function is only for
    /// hardware problems, never for software conditions such as resource
    /// exhaustion. May be called from user, kernel or interrupt context.
    pub fn ddi_dev_report_fault(
        dip: dev_info_t,
        impact: ddi_fault_impact_t,
        location: ddi_fault_location_t,
        message: *const c_char,
    );
    /// `ddi_dev_is_sid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether the device is self-identifying, that is, able to tell
    /// the system unequivocally that it exists. Drivers that support both a
    /// self-identifying and a non-self-identifying variant of a device use
    /// it in `probe(9E)` to decide whether probing is needed at all. Returns
    /// `DDI_SUCCESS` when the device is self-identifying and `DDI_FAILURE`
    /// otherwise. May be called from user, kernel or interrupt context.
    pub fn ddi_dev_is_sid(dip: dev_info_t) -> c_int;
    /// `ddi_slaveonly(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether the bus, or the part of the bus, that the device is
    /// installed on prevents it from becoming a DMA master, that is, whether
    /// the device sits in a slave-access-only slot. Returns `DDI_SUCCESS`
    /// when it does and `DDI_FAILURE` when it does not. May be called from
    /// user, kernel or interrupt context.
    pub fn ddi_slaveonly(dip: dev_info_t) -> c_int;

    // --- power (legacy) ---------------------------------------------------
    /// `ddi_dev_is_needed(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Tells the system that component `cmpt` of the device is needed at
    /// power level `level`, which must be non-zero, sets the component to
    /// that level and brings every device that depends on it to its normal
    /// level. The page marks the interface obsolete in its attributes and
    /// directs drivers to `pm_raise_power(9F)` and `pm_lower_power(9F)`
    /// instead. The call may re-enter the driver, so holding driver locks
    /// across it can deadlock. Returns `DDI_SUCCESS` when the level was set
    /// and `DDI_FAILURE` otherwise. May be called from user or kernel
    /// context.
    pub fn ddi_dev_is_needed(dip: dev_info_t, cmpt: c_int, level: c_int) -> c_int;
    /// `ddi_removing_power(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether the pending `detach(9E)` call with the `DDI_SUSPEND`
    /// command is likely to result in power being removed from the device.
    /// A driver whose hardware would be damaged by an abrupt power loss can
    /// refuse the suspend in that case. A true result does not guarantee
    /// that power will actually be removed, since the system suspend may
    /// still fail. Returns 1 when power might be removed and 0 when it will
    /// not.
    pub fn ddi_removing_power(dip: dev_info_t) -> c_int;
    /// `ddi_power`. Not documented in section 9; exported by
    /// `uts/common/mapfiles/ddi.mapfile`.
    ///
    /// Sets component `cmpt` of the device to power level `level` through
    /// the device's parent nexus. Returns `DDI_SUCCESS` or `DDI_FAILURE`.
    /// The header declares it next to [`ddi_dev_is_needed`]; the `pm_*`
    /// functions in [`crate::pm`] are the documented interfaces.
    pub fn ddi_power(dip: dev_info_t, cmpt: c_int, level: c_int) -> c_int;

    // --- thread and process identity -------------------------------------
    /// `ddi_can_receive_sig(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether the current thread can receive signals sent with
    /// `kill(2)`. When the result is `B_FALSE` the thread is not associated
    /// with a user process or is in the middle of exiting, and any call to
    /// `cv_wait_sig(9F)`, `cv_timedwait_sig(9F)` or `qwait_sig(9F)` behaves
    /// like its uninterruptible counterpart. Drivers that may block
    /// indefinitely use it to decide whether a timeout is needed to avoid
    /// creating an unkillable thread. The function does not check whether
    /// signals are currently blocked. May be called from user, kernel or
    /// interrupt context.
    pub fn ddi_can_receive_sig() -> boolean_t;
    /// `ddi_get_pid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the process ID of the current process. It can be used to
    /// restrict an operation to one process or to decide whether a device
    /// context belongs to the caller. May be called from user context only.
    pub fn ddi_get_pid() -> pid_t;
    /// `ddi_get_kt_did(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a unique 64-bit identifier for the current thread. It always
    /// succeeds. May be called from user, kernel or interrupt context, but
    /// not from a high-level interrupt handler.
    pub fn ddi_get_kt_did() -> kt_did_t;
    /// `ddi_ncpus_expected(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the number of CPUs the system is expected to have once it
    /// has finished booting. A driver that attaches while only the boot
    /// processor is running, such as the driver for the boot device, would
    /// otherwise size interrupt vectors, queues and worker threads for a
    /// single CPU. Once further processors are running the function returns
    /// the number present. The value is an estimate for sizing purposes; the
    /// number of CPUs can change through hotplug and not every present CPU
    /// is necessarily online. Always succeeds and never returns less than 1.
    /// May be called from user, kernel or interrupt context.
    pub fn ddi_ncpus_expected() -> uint_t;
    /// `proc_ref(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns an unambiguous reference to the current process for later
    /// signalling with [`proc_signal`]. The handle stays unique even after
    /// the process dies. Because the reference commits system resources, it
    /// must be released with [`proc_unref`] as soon as it is no longer
    /// needed, exactly once per call. May be called from user context only.
    pub fn proc_ref() -> *mut c_void;
    /// `proc_unref(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases a process reference obtained from [`proc_ref`]. May be
    /// called from user, kernel or interrupt context.
    pub fn proc_unref(pref: *mut c_void);
    /// `proc_signal(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sends signal `sig` to the process referenced by `pref`. The signals a
    /// driver may send are `SIGHUP`, `SIGINT`, `SIGQUIT`, `SIGPOLL`,
    /// `SIGKILL`, `SIGWINCH` and `SIGURG`. Returns 0 when the process
    /// existed and the signal was sent, and -1 when the process no longer
    /// exists, in which case the caller should drop the reference with
    /// [`proc_unref`]. May be called from user, kernel or interrupt context.
    pub fn proc_signal(pref: *mut c_void, sig: c_int) -> c_int;
    /// `drv_getparm(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Reads one item of kernel state selected by `parm`, which is one of
    /// [`LBOLT`], [`PPGRP`], [`UPROCP`], [`PPID`], [`PSID`], [`TIME`] or
    /// [`UCRED`], and stores it through `value_p`, which must point at
    /// suitably sized and aligned storage for that item; the function checks
    /// neither. The interface has been superseded by `ddi_get_lbolt(9F)`,
    /// `ddi_get_time(9F)` and `ddi_get_pid(9F)`. Returns 0 on success and
    /// -1 when `parm` names nothing that may be read, in which case the
    /// stored value is undefined. May be called from user, kernel or
    /// interrupt context for `LBOLT` and `TIME`, and from user context only
    /// for the process-related items.
    pub fn drv_getparm(parm: uint_t, value_p: *mut c_void) -> c_int;

    // --- critical sections and bit scanning --------------------------------
    /// `ddi_enter_critical(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the system to guarantee, as far as it can, that the current
    /// thread will be neither preempted nor interrupted until the matching
    /// [`ddi_exit_critical`] call. It does not stop other processors and
    /// cannot stop the hardware from performing interrupt acknowledge
    /// cycles. Between the two calls the driver may not call any function
    /// outside itself, and code that could loop forever inside the region
    /// may crash the machine. Returns an opaque value that must be passed to
    /// `ddi_exit_critical`. May be called from user, kernel or interrupt
    /// context.
    pub fn ddi_enter_critical() -> c_uint;
    /// `ddi_exit_critical(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Ends the critical region opened by [`ddi_enter_critical`]. The
    /// argument is the value that call returned.
    pub fn ddi_exit_critical(ddic: c_uint);
    /// `ddi_ffs(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the position of the least significant set bit in `mask`,
    /// counting from 1 for the least significant bit up to 64 for the most
    /// significant, or 0 when no bit is set. May be called from user,
    /// kernel or interrupt context.
    pub fn ddi_ffs(mask: c_long) -> c_int;
    /// `ddi_fls(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the position of the most significant set bit in `mask`,
    /// counting from 1 for the least significant bit up to 64 for the most
    /// significant, or 0 when no bit is set. May be called from user,
    /// kernel or interrupt context.
    pub fn ddi_fls(mask: c_long) -> c_int;

    // --- strings -----------------------------------------------------------
    /// `ddi_strdup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a newly allocated copy of the string `str`, documented on the
    /// `string(9F)` page. The `flag` argument is `KM_SLEEP` or `KM_NOSLEEP`
    /// and decides whether the allocation may block; with `KM_NOSLEEP` the
    /// result may be NULL. The copy is released with `strfree(9F)`.
    pub fn ddi_strdup(str: *const c_char, flag: c_int) -> *mut c_char;
    /// `ddi_strtol(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the initial portion of `str` to a `long` and stores it
    /// through `result`. Leading white space is skipped, an optional sign is
    /// accepted, and `base` selects the radix: 0 lets a `0x` or `0X` prefix
    /// select hexadecimal and a leading `0` octal, while a value from 2 to
    /// 36 fixes the radix and, for 16, allows the `0x` prefix. When `endptr`
    /// is not NULL the address of the first unconverted character is stored
    /// through it, or the address of `str` itself when nothing could be
    /// converted. Returns 0 on success, `EINVAL` when `base` is invalid, and
    /// `ERANGE` when the value does not fit; in both error cases `result` is
    /// left unchanged. May be called from user, kernel or interrupt context.
    pub fn ddi_strtol(str: *const c_char, endptr: *mut *mut c_char, base: c_int, result: *mut c_long) -> c_int;
    /// `ddi_strtoul(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The unsigned counterpart of [`ddi_strtol`], converting to an
    /// `unsigned long` with the same rules for white space, sign, radix and
    /// `endptr`. Returns 0 on success, `EINVAL` when `base` is invalid, and
    /// `ERANGE` when the value does not fit. May be called from user, kernel
    /// or interrupt context.
    pub fn ddi_strtoul(str: *const c_char, endptr: *mut *mut c_char, base: c_int, result: *mut c_ulong) -> c_int;
    /// `ddi_strtoll(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the initial portion of `str` to a `longlong_t` and stores it
    /// through `result`, with the same rules for white space, sign, radix and
    /// `endptr` as [`ddi_strtol`]. Returns 0 on success and `EINVAL` when
    /// `base` is invalid, in which case `result` is left unchanged. May be
    /// called from user, kernel or interrupt context.
    pub fn ddi_strtoll(str: *const c_char, endptr: *mut *mut c_char, base: c_int, result: *mut longlong_t) -> c_int;
    /// `ddi_strtoull(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the initial portion of `str` to a `u_longlong_t` and stores
    /// it through `result`, with the same rules for white space, sign, radix
    /// and `endptr` as [`ddi_strtol`]. Returns 0 on success and `EINVAL`
    /// when `base` is invalid, in which case `result` is left unchanged. May
    /// be called from user, kernel or interrupt context.
    pub fn ddi_strtoull(str: *const c_char, endptr: *mut *mut c_char, base: c_int, result: *mut u_longlong_t) -> c_int;
    /// `swab(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Copies `nbytes` bytes, rounded down to a whole number of 16-bit
    /// halfwords, from `src` to `dst`, swapping the two bytes of each
    /// halfword as it goes. When `dst` is the same as `src` the buffer is
    /// swapped in place. May be called from user, kernel or interrupt
    /// context.
    pub fn swab(src: *mut c_void, dst: *mut c_void, nbytes: size_t);

    // --- page arithmetic --------------------------------------------------
    /// `ddi_btop(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts a byte count to the number of memory pages it covers,
    /// rounding down when the count is not a page multiple. The conversion
    /// is performed by the parent nexus, so it reflects the page size of
    /// whatever address translation the bus provides rather than that of
    /// the main system MMU alone, which is why these functions are preferred
    /// over `btop(9F)` and its relatives. There are no error returns. May be
    /// called from user, kernel or interrupt context.
    pub fn ddi_btop(dip: dev_info_t, bytes: c_ulong) -> c_ulong;
    /// `ddi_btopr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`ddi_btop`], but rounds up when the byte count is not a page
    /// multiple. May be called from user, kernel or interrupt context.
    pub fn ddi_btopr(dip: dev_info_t, bytes: c_ulong) -> c_ulong;
    /// `ddi_ptob(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts a page count to the number of bytes it covers, using the
    /// page size of the parent nexus as [`ddi_btop`] does. Calling it with a
    /// page count of 1 yields the page size. May be called from user, kernel
    /// or interrupt context.
    pub fn ddi_ptob(dip: dev_info_t, pages: c_ulong) -> c_ulong;

    // --- entry point helpers ----------------------------------------------
    /// `ddi_mmap_get_model(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the data model the current thread expects, for use with
    /// `ddi_model_convert_from(9F)` inside the `mmap(9E)` entry point, where
    /// no `mode` argument carries the model bits. Returns
    /// `DDI_MODEL_ILP32` or `DDI_MODEL_LP64`, or `DDI_FAILURE` when not
    /// called from `mmap(9E)`, which is the only place it may be called
    /// from. The header declares the return type as `model_t`.
    pub fn ddi_mmap_get_model() -> crate::model::model_t;
    /// `ddi_no_info(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// A `getinfo(9E)` implementation that always returns `DDI_FAILURE`. It
    /// is meant for drivers that provide no `cb_ops(9S)`, and for network
    /// drivers that only offer DLPI-2 services, which can install it in the
    /// `devo_getinfo` slot of their `dev_ops(9S)`.
    pub fn ddi_no_info(
        dip: dev_info_t,
        infocmd: crate::devops::ddi_info_cmd_t,
        arg: *mut c_void,
        result: *mut *mut c_void,
    ) -> c_int;

    // --- device memory ----------------------------------------------------
    /// `ddi_device_copy(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies `bytecount` bytes from the device address `src_addr` to the
    /// device address `dest_addr`, going through the access handles of both
    /// so that byte order is translated when the two devices disagree. The
    /// `dev_datasz` argument is one of the `DDI_DATA_SZ*_ACC` values and
    /// gives the size of each data word, which must be the same on both
    /// sides. The `src_advcnt` and `dest_advcnt` arguments give the number
    /// of data words by which the respective address advances after each
    /// access: 0 reuses the same address every time, a positive value moves
    /// forward and a negative value moves backward. Returns `DDI_SUCCESS`,
    /// or `DDI_FAILURE` when `bytecount` is not a multiple of the data word
    /// size. May be called from user, kernel or interrupt context.
    pub fn ddi_device_copy(
        src_handle: ddi_acc_handle_t,
        src_addr: caddr_t,
        src_advcnt: ssize_t,
        dest_handle: ddi_acc_handle_t,
        dest_addr: caddr_t,
        dest_advcnt: ssize_t,
        bytecount: size_t,
        dev_datasz: uint_t,
    ) -> c_int;
    /// `ddi_device_zero(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Writes `bytecount` bytes of zeroes to the device register or memory
    /// at `dev_addr` through the access handle. The `dev_datasz` argument
    /// is one of the `DDI_DATA_SZ*_ACC` values and gives the size of each
    /// data word, and `dev_advcnt` gives the number of data words by which
    /// the address advances after each access, with 0 reusing the same
    /// address, a positive value moving forward and a negative value moving
    /// backward. Returns `DDI_SUCCESS`, or `DDI_FAILURE` when `bytecount` is
    /// not a multiple of the data word size. May be called from user, kernel
    /// or interrupt context.
    pub fn ddi_device_zero(
        handle: ddi_acc_handle_t,
        dev_addr: caddr_t,
        bytecount: size_t,
        dev_advcnt: ssize_t,
        dev_datasz: uint_t,
    ) -> c_int;
}
