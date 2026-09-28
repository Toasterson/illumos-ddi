//! PCI configuration space.
//!
//! Every PCI function has a configuration space, a small address space
//! separate from its memory and I/O regions, where the header identifies
//! the device and the capability list describes its optional features. A
//! leaf driver reaches it through a handle from `pci_config_setup(9F)`, reads
//! and writes it by byte offset with `pci_config_get8(9F)`,
//! `pci_config_put8(9F)` and their wider relatives, and releases the handle
//! with `pci_config_teardown(9F)`. Configuration space is little-endian, and
//! the accessors translate to and from host order.
//!
//! [`PciConfig`] owns such a handle. Its accessors take a byte offset,
//! check it against the 4096 bytes of PCI Express configuration space and
//! against the alignment of the access width, and return a [`RegError`]
//! rather than passing a bad offset to the kernel, for the reasons given in
//! [`crate::regs`]. A conventional PCI function only has the first 256
//! bytes. Reads beyond that on such a function return whatever the platform
//! returns for an absent register, usually all ones, and are harmless.
//!
//! The readers for the header fields, such as [`PciConfig::vendor_id`], use
//! the fixed offsets from `sys/pci.h` and cannot fail.
//! [`PciConfig::capabilities`] walks the capability list.
//!
//! [`PciConfig::save_state`] and [`PciConfig::restore_state`] wrap
//! `pci_save_config_regs(9F)` and `pci_restore_config_regs(9F)`, which a
//! driver calls from `power(9E)` around a trip through the D3 power state,
//! and [`PciConfig::report_idle_speed`] wraps `pci_report_pmcap(9F)`.

use core::fmt;
use core::mem::size_of;

use crate::context::Sleepable;
use crate::device::{DevInfo, OwnNode};
use crate::error::DdiError;
use crate::regs::{RegError, RegValue};
use crate::sys;

/// The size of PCI Express configuration space in bytes. Offsets at or
/// above this value are refused.
pub const CONFIG_SPACE_SIZE: usize = 4096;

/// The size of conventional PCI configuration space in bytes, which is all
/// that a function without PCI Express extended configuration space has.
pub const LEGACY_CONFIG_SPACE_SIZE: usize = 256;

/// The class code of a function, from the three bytes at
/// `PCI_CONF_PROGCLASS`, `PCI_CONF_SUBCLASS` and `PCI_CONF_BASCLASS`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClassCode {
    /// The base class, such as 0x01 for mass storage or 0x02 for network.
    pub base: u8,
    /// The subclass within the base class.
    pub sub: u8,
    /// The programming interface within the subclass.
    pub prog_if: u8,
}

/// The lowest clock speed the device tolerates while idle, as
/// `pci_report_pmcap(9F)` reports it with `PCI_PM_IDLESPEED`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IdleSpeed {
    /// The device works at any idle clock speed, including a stopped clock
    /// (`PCI_PM_IDLESPEED_ANY`).
    Any,
    /// The clock must not be slowed at all (`PCI_PM_IDLESPEED_NONE`). This
    /// is what the framework assumes for a driver that never reports.
    None,
    /// The lowest tolerated speed in kilohertz, relative to a 33 MHz bus.
    KHz(u32),
}

/// A handle to the configuration space of one PCI function, created with
/// `pci_config_setup(9F)` and released with `pci_config_teardown(9F)` when
/// dropped.
pub struct PciConfig {
    handle: sys::ddi_acc_handle_t,
    dev: DevInfo,
}

// SAFETY: The handle is a plain value that any thread may use. C drivers
// read and write configuration space through one handle from their
// interrupt handlers and their other entry points at the same time, and the
// platform serialises the underlying configuration cycles itself.
// `pci_config_teardown(9F)` needs exclusive ownership, which `Drop` has.
unsafe impl Send for PciConfig {}
// SAFETY: See the `Send` implementation. Every method takes `&self`.
unsafe impl Sync for PciConfig {}

impl PciConfig {
    /// Obtains a configuration space handle for the driver's own attached
    /// node with `pci_config_setup(9F)`. The manual requires user or kernel
    /// context, hence the [`Sleepable`] token. The handle is released when
    /// the returned value is dropped.
    ///
    /// `pci_config_setup(9F)` maps register set 0 through the node's parent
    /// nexus, and the root node has none, so the kernel would dereference
    /// NULL. A node that the guard of [`OwnNode`] refuses fails before the
    /// kernel is called.
    pub fn setup(node: &OwnNode, _ctx: &Sleepable) -> Result<PciConfig, DdiError> {
        if !node.is_plausibly_own() {
            return Err(DdiError);
        }
        let mut handle: sys::ddi_acc_handle_t = core::ptr::null_mut();
        DdiError::check(unsafe { sys::pci_config_setup(node.as_raw(), &mut handle) })?;
        Ok(PciConfig { handle, dev: node.dev() })
    }

    /// Returns the raw access handle, for functions this crate does not wrap
    /// yet. The handle stays valid for as long as `self` exists.
    pub fn handle(&self) -> sys::ddi_acc_handle_t {
        self.handle
    }

    /// Checks that an access of width `T` at `off` lies inside configuration
    /// space and is aligned.
    fn offset<T: RegValue>(off: usize) -> Result<sys::off_t, RegError> {
        match off.checked_add(size_of::<T>()) {
            Some(end) if end <= CONFIG_SPACE_SIZE => {}
            _ => return Err(RegError::OutOfRange),
        }
        if !off.is_multiple_of(size_of::<T>()) {
            return Err(RegError::Misaligned);
        }
        // The offset is below 4096 and therefore fits.
        Ok(off as sys::off_t)
    }

    /// Reads a value of width `T` at byte offset `off`, using
    /// `pci_config_get8(9F)`, `pci_config_get16(9F)`, `pci_config_get32(9F)`
    /// or `pci_config_get64(9F)`.
    pub fn get<T: RegValue>(&self, off: usize) -> Result<T, RegError> {
        let o = Self::offset::<T>(off)?;
        // SAFETY: The handle came from `pci_config_setup(9F)` and the offset
        // was checked.
        Ok(unsafe { T::cfg_get(self.handle, o) })
    }

    /// Writes a value of width `T` at byte offset `off`, using
    /// `pci_config_put8(9F)`, `pci_config_put16(9F)`, `pci_config_put32(9F)`
    /// or `pci_config_put64(9F)`.
    pub fn put<T: RegValue>(&self, off: usize, value: T) -> Result<(), RegError> {
        let o = Self::offset::<T>(off)?;
        // SAFETY: As in `get`.
        unsafe { T::cfg_put(self.handle, o, value) };
        Ok(())
    }

    /// Reads a field at an offset the caller knows to be valid. Every
    /// caller passes either a `PCI_CONF_*` constant or an offset below 256
    /// taken from the capability list, and each is aligned to the width it
    /// is read with.
    fn header<T: RegValue>(&self, off: sys::off_t) -> T {
        debug_assert!(off >= 0 && (off as usize) < LEGACY_CONFIG_SPACE_SIZE);
        // SAFETY: The handle came from `pci_config_setup(9F)` and the offset
        // is valid and aligned, as described above.
        unsafe { T::cfg_get(self.handle, off) }
    }

    /// Returns the vendor identifier at `PCI_CONF_VENID`.
    pub fn vendor_id(&self) -> u16 {
        self.header(sys::PCI_CONF_VENID)
    }

    /// Returns the device identifier at `PCI_CONF_DEVID`.
    pub fn device_id(&self) -> u16 {
        self.header(sys::PCI_CONF_DEVID)
    }

    /// Returns the revision identifier at `PCI_CONF_REVID`.
    pub fn revision_id(&self) -> u8 {
        self.header(sys::PCI_CONF_REVID)
    }

    /// Returns the class code from `PCI_CONF_PROGCLASS`,
    /// `PCI_CONF_SUBCLASS` and `PCI_CONF_BASCLASS`.
    pub fn class_code(&self) -> ClassCode {
        ClassCode {
            base: self.header(sys::PCI_CONF_BASCLASS),
            sub: self.header(sys::PCI_CONF_SUBCLASS),
            prog_if: self.header(sys::PCI_CONF_PROGCLASS),
        }
    }

    /// Returns the header type at `PCI_CONF_HEADER`, including the
    /// multi-function bit `PCI_HEADER_MULTI`.
    pub fn header_type(&self) -> u8 {
        self.header(sys::PCI_CONF_HEADER)
    }

    /// Returns the subsystem vendor identifier at `PCI_CONF_SUBVENID`. The
    /// field exists in type 0 headers only; for a bridge the value is
    /// meaningless.
    pub fn subsystem_vendor_id(&self) -> u16 {
        self.header(sys::PCI_CONF_SUBVENID)
    }

    /// Returns the subsystem identifier at `PCI_CONF_SUBSYSID`, with the
    /// same caveat as [`PciConfig::subsystem_vendor_id`].
    pub fn subsystem_id(&self) -> u16 {
        self.header(sys::PCI_CONF_SUBSYSID)
    }

    /// Returns the command register at `PCI_CONF_COMM`, whose bits are the
    /// `PCI_COMM_*` constants.
    pub fn command(&self) -> u16 {
        self.header(sys::PCI_CONF_COMM)
    }

    /// Writes the command register at `PCI_CONF_COMM`. Drivers normally
    /// read the register, change the bits they own and write it back.
    pub fn set_command(&self, value: u16) {
        // SAFETY: The handle came from `pci_config_setup(9F)` and the
        // command register is an aligned 16-bit field of the header.
        unsafe { <u16 as crate::regs::sealed::Sealed>::cfg_put(self.handle, sys::PCI_CONF_COMM, value) }
    }

    /// Returns the status register at `PCI_CONF_STAT`, whose bits are the
    /// `PCI_STAT_*` constants.
    pub fn status(&self) -> u16 {
        self.header(sys::PCI_CONF_STAT)
    }

    /// Returns an iterator over the capability list, yielding the
    /// identifier and the configuration space offset of each entry.
    ///
    /// The list is only followed when `PCI_STAT_CAP` is set in the status
    /// register, and its head is taken from `PCI_CONF_CAP_PTR` for type 0
    /// and type 1 headers and from `PCI_CBUS_CAP_PTR` for CardBus bridges,
    /// as `pci_lcap_locate` does. The walk stops at a null pointer, at a
    /// pointer into the header below `PCI_CAP_PTR_OFF`, and at the first
    /// entry it has already visited, so a broken or looping list cannot keep
    /// it going. The PCI Express extended capabilities above offset 256 are
    /// a separate list and are not included.
    pub fn capabilities(&self) -> Capabilities<'_> {
        let mut next = 0u8;
        if self.status() & sys::PCI_STAT_CAP != 0 {
            let head = match self.header_type() & sys::PCI_HEADER_TYPE_M {
                sys::PCI_HEADER_ZERO | sys::PCI_HEADER_PPB => Some(sys::PCI_CONF_CAP_PTR),
                sys::PCI_HEADER_CARDBUS => Some(sys::PCI_CBUS_CAP_PTR),
                _ => None,
            };
            if let Some(h) = head {
                next = self.header::<u8>(h);
            }
        }
        Capabilities { cfg: self, next, seen: 0 }
    }

    /// Returns the offset of the first capability with identifier `id`,
    /// one of the `PCI_CAP_ID_*` constants, or `None` if the function does
    /// not have it. The capability's own registers start two bytes after
    /// the returned offset.
    pub fn find_capability(&self, id: u8) -> Option<u16> {
        self.capabilities().find(|&(cid, _)| cid == id).map(|(_, off)| off)
    }

    /// Saves the configuration registers of the function with
    /// `pci_save_config_regs(9F)`, so that [`PciConfig::restore_state`] can
    /// put them back after the device returns from D3. The manual requires
    /// user or kernel context.
    pub fn save_state(&self, _ctx: &Sleepable) -> Result<(), DdiError> {
        DdiError::check(unsafe { sys::pci_save_config_regs(self.dev.as_raw()) })
    }

    /// Restores the configuration registers saved by
    /// [`PciConfig::save_state`] with `pci_restore_config_regs(9F)`. This
    /// must happen before the device is touched in any other way after it
    /// leaves D3. The manual requires user or kernel context.
    pub fn restore_state(&self, _ctx: &Sleepable) -> Result<(), DdiError> {
        DdiError::check(unsafe { sys::pci_restore_config_regs(self.dev.as_raw()) })
    }

    /// Reports the lowest idle clock speed the device tolerates with
    /// `pci_report_pmcap(9F)` and `PCI_PM_IDLESPEED`. A single device that
    /// never reports keeps the whole bus from slowing its clock, so a driver
    /// whose device does not care should report [`IdleSpeed::Any`]. The
    /// report may be repeated whenever the answer changes, from any context.
    pub fn report_idle_speed(&self, speed: IdleSpeed) -> Result<(), DdiError> {
        let arg = match speed {
            IdleSpeed::Any => sys::PCI_PM_IDLESPEED_ANY,
            IdleSpeed::None => sys::PCI_PM_IDLESPEED_NONE,
            // The C interface passes the speed as an integer cast to a
            // pointer, so the pointer carries no provenance.
            IdleSpeed::KHz(k) => core::ptr::without_provenance_mut(k as usize),
        };
        DdiError::check(unsafe { sys::pci_report_pmcap(self.dev.as_raw(), sys::PCI_PM_IDLESPEED, arg) })
    }
}

macro_rules! fixed_width {
    ($t:ty, $get:ident, $put:ident, $gname:literal, $pname:literal) => {
        impl PciConfig {
            #[doc = concat!("Reads a `", stringify!($t), "` at byte offset `off` with `", $gname, "(9F)`.")]
            pub fn $get(&self, off: usize) -> Result<$t, RegError> {
                self.get::<$t>(off)
            }
            #[doc = concat!("Writes a `", stringify!($t), "` at byte offset `off` with `", $pname, "(9F)`.")]
            pub fn $put(&self, off: usize, value: $t) -> Result<(), RegError> {
                self.put::<$t>(off, value)
            }
        }
    };
}
fixed_width!(u8, get8, put8, "pci_config_get8", "pci_config_put8");
fixed_width!(u16, get16, put16, "pci_config_get16", "pci_config_put16");
fixed_width!(u32, get32, put32, "pci_config_get32", "pci_config_put32");
fixed_width!(u64, get64, put64, "pci_config_get64", "pci_config_put64");

impl Drop for PciConfig {
    fn drop(&mut self) {
        // SAFETY: The handle came from a successful `pci_config_setup(9F)`
        // and is released exactly once.
        unsafe { sys::pci_config_teardown(&mut self.handle) }
    }
}

impl fmt::Debug for PciConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PciConfig").field("dev", &self.dev).finish()
    }
}

/// An iterator over the capability list of a function, created by
/// [`PciConfig::capabilities`]. Each item is the capability identifier and
/// the configuration space offset of the entry.
pub struct Capabilities<'a> {
    cfg: &'a PciConfig,
    next: u8,
    /// One bit per four-byte slot of the 256-byte conventional space, set
    /// once the entry in that slot has been yielded.
    seen: u64,
}

impl Iterator for Capabilities<'_> {
    type Item = (u8, u16);

    fn next(&mut self) -> Option<(u8, u16)> {
        let off = self.next & sys::PCI_CAP_PTR_MASK;
        if (off as sys::off_t) < sys::PCI_CAP_PTR_OFF {
            return None;
        }
        let bit = 1u64 << (off / 4);
        if self.seen & bit != 0 {
            return None;
        }
        self.seen |= bit;
        let id: u8 = self.cfg.header(off as sys::off_t);
        self.next = self.cfg.header(off as sys::off_t + sys::PCI_CAP_NEXT_PTR);
        Some((id, u16::from(off)))
    }
}

impl fmt::Debug for Capabilities<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Capabilities").field("next", &self.next).finish()
    }
}
