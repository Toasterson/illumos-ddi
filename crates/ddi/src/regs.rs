//! Programmed I/O to device registers through an access handle.
//!
//! A C driver maps a register set with `ddi_regs_map_setup(9F)`, receives a
//! base address and an access handle, and then reads and writes registers
//! by passing the handle and the base address plus an offset to
//! `ddi_get32(9F)`, `ddi_put32(9F)` and their relatives. Going through the
//! handle rather than dereferencing the address is what lets the framework
//! apply the byte order and ordering rules the driver asked for, and it is
//! where access faults are recorded for `ddi_check_acc_handle(9F)` to find.
//!
//! This module keeps that model and adds the checks a C driver has to make
//! by hand. The rules are these.
//!
//! - [`AccessAttr`] describes the byte order, the ordering and the fault
//!   handling of a mapping and produces the `ddi_device_acc_attr(9S)`
//!   structure, choosing the structure version itself.
//! - [`RegMap::map`] maps a register set and remembers how many bytes were
//!   mapped. Dropping the [`RegMap`] calls `ddi_regs_map_free(9F)`.
//! - Every access names a byte offset from the start of the mapping. The
//!   offset is checked against the mapped length and the resulting device
//!   address is checked for the natural alignment of the access width
//!   before anything reaches the kernel.
//! - [`RegBlock`] and [`Reg`] move both checks to compile time for drivers
//!   that describe their registers as constants.
//!
//! An access outside the mapping would touch whatever lies behind it, which
//! may be another device's registers or an unmapped page, and a misaligned
//! access to device memory faults on some platforms. Either outcome can take
//! the system down. Panicking on a bad offset would take it down as well,
//! because a Rust panic in the kernel is a system panic. The byte-offset
//! accessors therefore return a [`RegError`] instead, and the caller decides
//! what a bad offset means for its device. Drivers that want infallible
//! accessors use [`RegBlock`], where an offset outside the block or a
//! misaligned offset fails to build rather than failing at run time.
//!
//! What the register contains is not checked, and cannot be. Writing the
//! wrong value to a register is a hardware bug and not a memory safety bug,
//! which is why the accessors are safe functions.
//!
//! The functions behind the accessors may be called from user, kernel or
//! interrupt context, so none of them requires a context token. Mapping and
//! unmapping must happen in user or kernel context, which is why
//! [`RegMap::map`] requires a [`Sleepable`] token. A [`RegMap`] is normally
//! stored in the driver's instance state and dropped in `detach(9E)`; it
//! must not be dropped from an interrupt handler.

use core::fmt;
use core::marker::PhantomData;
use core::mem::size_of;

use crate::context::Sleepable;
use crate::device::DevInfo;
use crate::error::DdiError;
use crate::sys;

/// The byte order of the device, which is the `devacc_attr_endian_flags`
/// member of `ddi_device_acc_attr(9S)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Endian {
    /// The data is transferred without any byte swapping
    /// (`DDI_NEVERSWAP_ACC`). This is the choice for byte streams and for
    /// devices whose registers are already in host order.
    NeverSwap,
    /// The device registers are little-endian (`DDI_STRUCTURE_LE_ACC`). The
    /// framework swaps bytes when the host is big-endian. PCI devices are
    /// almost always little-endian.
    Little,
    /// The device registers are big-endian (`DDI_STRUCTURE_BE_ACC`). The
    /// framework swaps bytes when the host is little-endian.
    Big,
}

/// How the processor may order, merge and cache the accesses, which is the
/// `devacc_attr_dataorder` member of `ddi_device_acc_attr(9S)`.
///
/// Each value permits everything the values before it permit. The manual
/// describes all of them as advisory, so the system may always order more
/// strictly than asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DataOrder {
    /// Accesses are issued in program order (`DDI_STRICTORDER_ACC`). This is
    /// the default and the right choice for control registers.
    Strict,
    /// Accesses may be reordered (`DDI_UNORDERED_OK_ACC`).
    Unordered,
    /// Accesses to consecutive locations may also be merged into wider ones
    /// (`DDI_MERGING_OK_ACC`).
    Merging,
    /// Loaded data may also be cached and reused until the next store
    /// (`DDI_LOADCACHING_OK_ACC`).
    LoadCaching,
    /// Stored data may also be held in the cache and written to the device
    /// later (`DDI_STORECACHING_OK_ACC`).
    StoreCaching,
}

/// What the system does when an access through the handle faults, which is
/// the `devacc_attr_access` member of `ddi_device_acc_attr(9S)`.
///
/// The two modes other than [`FaultMode::Default`] only take effect when the
/// driver has declared `DDI_FM_ACCCHK_CAPABLE` with `ddi_fm_init(9F)`
/// before mapping. Otherwise the framework silently falls back to the
/// default behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FaultMode {
    /// The system takes its default action on a fault, which may be a panic
    /// (`DDI_DEFAULT_ACC`).
    Default,
    /// The driver is hardened. The system should record a fault in the
    /// handle rather than panic, and the driver promises to look for it
    /// with [`RegMap::check`] or `ddi_fm_acc_err_get(9F)` (`DDI_FLAGERR_ACC`).
    FlagErrors,
    /// A fault is expected and every access is protected against it, at a
    /// considerable cost (`DDI_CAUTIOUS_ACC`). The accessors then take
    /// kernel mutexes internally, so this mode is meant for probing a device
    /// that may not be present, not for the data path.
    Cautious,
}

/// The access attributes of a mapping, which become the
/// `ddi_device_acc_attr(9S)` structure passed to `ddi_regs_map_setup(9F)`.
///
/// [`AccessAttr::new`] starts from strict ordering and the default fault
/// handling, and the other methods change one member at a time.
///
/// ```ignore
/// const ATTR: AccessAttr = AccessAttr::new(Endian::Little).order(DataOrder::Strict);
/// ```
///
/// The structure version is chosen from the fault mode. With
/// [`FaultMode::Default`] it is `DDI_DEVICE_ATTR_V0`, which has no fault
/// member. With either of the other modes it is `DDI_DEVICE_ATTR_V1`, which
/// is the version that makes the framework read that member. This is the
/// same choice `pci_config_setup(9F)` makes internally, and it keeps a
/// driver that declared fault management capabilities from having its
/// default mappings reported as a capability mismatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AccessAttr {
    endian: Endian,
    order: DataOrder,
    fault: FaultMode,
}

impl AccessAttr {
    /// Creates attributes with the given byte order, strict ordering and
    /// the default fault handling.
    pub const fn new(endian: Endian) -> AccessAttr {
        AccessAttr { endian, order: DataOrder::Strict, fault: FaultMode::Default }
    }

    /// Returns these attributes with a different ordering rule.
    pub const fn order(self, order: DataOrder) -> AccessAttr {
        AccessAttr { order, ..self }
    }

    /// Returns these attributes with a different fault mode.
    pub const fn fault_mode(self, fault: FaultMode) -> AccessAttr {
        AccessAttr { fault, ..self }
    }

    /// Returns the byte order.
    pub const fn endian(&self) -> Endian {
        self.endian
    }

    /// Returns the ordering rule.
    pub const fn data_order(&self) -> DataOrder {
        self.order
    }

    /// Returns the fault mode.
    pub const fn fault(&self) -> FaultMode {
        self.fault
    }

    /// Builds the `ddi_device_acc_attr(9S)` structure. Other parts of the
    /// DDI take the same structure, `ddi_dma_mem_alloc(9F)` among them, which
    /// is why this is public.
    pub const fn to_raw(&self) -> sys::ddi_device_acc_attr_t {
        let (version, access) = match self.fault {
            FaultMode::Default => (sys::DDI_DEVICE_ATTR_V0, sys::DDI_DEFAULT_ACC),
            FaultMode::FlagErrors => (sys::DDI_DEVICE_ATTR_V1, sys::DDI_FLAGERR_ACC),
            FaultMode::Cautious => (sys::DDI_DEVICE_ATTR_V1, sys::DDI_CAUTIOUS_ACC),
        };
        sys::ddi_device_acc_attr_t {
            devacc_attr_version: version,
            devacc_attr_endian_flags: match self.endian {
                Endian::NeverSwap => sys::DDI_NEVERSWAP_ACC,
                Endian::Little => sys::DDI_STRUCTURE_LE_ACC,
                Endian::Big => sys::DDI_STRUCTURE_BE_ACC,
            },
            devacc_attr_dataorder: match self.order {
                DataOrder::Strict => sys::DDI_STRICTORDER_ACC,
                DataOrder::Unordered => sys::DDI_UNORDERED_OK_ACC,
                DataOrder::Merging => sys::DDI_MERGING_OK_ACC,
                DataOrder::LoadCaching => sys::DDI_LOADCACHING_OK_ACC,
                DataOrder::StoreCaching => sys::DDI_STORECACHING_OK_ACC,
            },
            devacc_attr_access: access,
        }
    }
}

pub(crate) mod sealed {
    use crate::sys;

    /// The per-width entry points behind [`super::RegValue`]. The trait is
    /// sealed so that only the four widths the DDI provides can implement
    /// it, and so that its unsafe methods stay out of the public interface.
    pub trait Sealed: Copy {
        /// Calls `ddi_get*(9F)`.
        ///
        /// # Safety
        /// `addr` must lie within the mapping behind `h` and be aligned.
        unsafe fn get(h: sys::ddi_acc_handle_t, addr: *const Self) -> Self;
        /// Calls `ddi_put*(9F)`.
        ///
        /// # Safety
        /// As for [`Sealed::get`].
        unsafe fn put(h: sys::ddi_acc_handle_t, addr: *mut Self, v: Self);
        /// Calls `ddi_rep_get*(9F)`.
        ///
        /// # Safety
        /// Every device address the call touches must lie within the mapping
        /// behind `h`, and `host` must be valid for `n` writes.
        unsafe fn rep_get(h: sys::ddi_acc_handle_t, host: *mut Self, dev: *const Self, n: usize, flags: sys::uint_t);
        /// Calls `ddi_rep_put*(9F)`.
        ///
        /// # Safety
        /// As for [`Sealed::rep_get`], with `host` valid for `n` reads.
        unsafe fn rep_put(h: sys::ddi_acc_handle_t, host: *const Self, dev: *mut Self, n: usize, flags: sys::uint_t);
        /// Calls `pci_config_get*(9F)`.
        ///
        /// # Safety
        /// `h` must come from `pci_config_setup(9F)` and `off` must be an
        /// aligned offset within configuration space.
        unsafe fn cfg_get(h: sys::ddi_acc_handle_t, off: sys::off_t) -> Self;
        /// Calls `pci_config_put*(9F)`.
        ///
        /// # Safety
        /// As for [`Sealed::cfg_get`].
        unsafe fn cfg_put(h: sys::ddi_acc_handle_t, off: sys::off_t, v: Self);
    }

    macro_rules! width {
        ($t:ty, $get:ident, $put:ident, $rget:ident, $rput:ident, $cget:ident, $cput:ident) => {
            impl Sealed for $t {
                #[inline]
                unsafe fn get(h: sys::ddi_acc_handle_t, addr: *const Self) -> Self {
                    unsafe { sys::$get(h, addr) }
                }
                #[inline]
                unsafe fn put(h: sys::ddi_acc_handle_t, addr: *mut Self, v: Self) {
                    unsafe { sys::$put(h, addr, v) }
                }
                #[inline]
                unsafe fn rep_get(
                    h: sys::ddi_acc_handle_t,
                    host: *mut Self,
                    dev: *const Self,
                    n: usize,
                    flags: sys::uint_t,
                ) {
                    unsafe { sys::$rget(h, host, dev, n, flags) }
                }
                #[inline]
                unsafe fn rep_put(
                    h: sys::ddi_acc_handle_t,
                    host: *const Self,
                    dev: *mut Self,
                    n: usize,
                    flags: sys::uint_t,
                ) {
                    unsafe { sys::$rput(h, host, dev, n, flags) }
                }
                #[inline]
                unsafe fn cfg_get(h: sys::ddi_acc_handle_t, off: sys::off_t) -> Self {
                    unsafe { sys::$cget(h, off) }
                }
                #[inline]
                unsafe fn cfg_put(h: sys::ddi_acc_handle_t, off: sys::off_t, v: Self) {
                    unsafe { sys::$cput(h, off, v) }
                }
            }
            impl super::RegValue for $t {}
        };
    }
    width!(u8, ddi_get8, ddi_put8, ddi_rep_get8, ddi_rep_put8, pci_config_get8, pci_config_put8);
    width!(u16, ddi_get16, ddi_put16, ddi_rep_get16, ddi_rep_put16, pci_config_get16, pci_config_put16);
    width!(u32, ddi_get32, ddi_put32, ddi_rep_get32, ddi_rep_put32, pci_config_get32, pci_config_put32);
    width!(u64, ddi_get64, ddi_put64, ddi_rep_get64, ddi_rep_put64, pci_config_get64, pci_config_put64);
}

/// A register width the DDI can access, which is one of `u8`, `u16`, `u32`
/// and `u64`.
///
/// The generic accessors such as [`RegMap::get`] use it to pick between
/// `ddi_get8(9F)`, `ddi_get16(9F)`, `ddi_get32(9F)` and `ddi_get64(9F)`.
/// The trait is sealed, because the DDI has no other widths.
pub trait RegValue: sealed::Sealed + fmt::Debug + Send + Sync + 'static {}

/// Why a register access was refused before it reached the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RegError {
    /// The access would extend past the end of the mapping, or the offset
    /// calculation overflowed.
    OutOfRange,
    /// The device address is not a multiple of the access width.
    Misaligned,
}

impl fmt::Display for RegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegError::OutOfRange => f.write_str("register access outside the mapping"),
            RegError::Misaligned => f.write_str("misaligned register access"),
        }
    }
}

/// Whether a repeated access walks through device memory or stays on one
/// register, which is the `flags` argument of `ddi_rep_get8(9F)` and its
/// relatives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RepMode {
    /// The device address advances by one element after each access, so a
    /// region of device memory is copied (`DDI_DEV_AUTOINCR`). Every element
    /// of the region must lie within the mapping.
    Increment,
    /// Every access goes to the same device address, which is how a FIFO
    /// data register is drained or filled (`DDI_DEV_NO_AUTOINCR`). Only that
    /// one register must lie within the mapping.
    Fixed,
}

impl RepMode {
    fn flags(self) -> sys::uint_t {
        match self {
            RepMode::Increment => sys::DDI_DEV_AUTOINCR,
            RepMode::Fixed => sys::DDI_DEV_NO_AUTOINCR,
        }
    }
}

impl DevInfo {
    /// Returns the number of register sets the node has, using
    /// `ddi_dev_nregs(9F)`. The register sets are numbered from zero. The
    /// call fails when the node has no registers at all.
    pub fn nregs(&self) -> Result<usize, DdiError> {
        let mut n: sys::c_int = 0;
        DdiError::check(unsafe { sys::ddi_dev_nregs(self.as_raw(), &mut n) })?;
        usize::try_from(n).map_err(|_| DdiError)
    }

    /// Returns the size in bytes of register set `rnumber`, using
    /// `ddi_dev_regsize(9F)`. The call fails when the set does not exist.
    pub fn regsize(&self, rnumber: u32) -> Result<usize, DdiError> {
        let mut size: sys::off_t = 0;
        DdiError::check(unsafe { sys::ddi_dev_regsize(self.as_raw(), rnumber, &mut size) })?;
        usize::try_from(size).map_err(|_| DdiError)
    }
}

/// A mapped register set, created with `ddi_regs_map_setup(9F)` and
/// released with `ddi_regs_map_free(9F)` when dropped.
///
/// All accesses go through the access handle with the `ddi_get*(9F)`,
/// `ddi_put*(9F)`, `ddi_rep_get*(9F)` and `ddi_rep_put*(9F)` functions, so
/// the byte order and ordering rules of the [`AccessAttr`] apply to every
/// one of them. Offsets are in bytes from the start of the mapping, which is
/// the register set offset given to [`RegMap::map`], and each access checks
/// its offset and alignment first as the module documentation describes.
pub struct RegMap {
    handle: sys::ddi_acc_handle_t,
    base: *mut u8,
    len: usize,
}

// SAFETY: The handle and the base address are plain values that any thread
// may use. The DDI lets a driver use one access handle from its interrupt
// handler and from its other entry points at the same time, which every C
// driver with registers does. The accessors do not modify the handle, except
// to record a fault, and the cautious accessors serialise internally with the
// handle's own mutexes. `ddi_regs_map_free(9F)` needs exclusive ownership,
// which `Drop` has.
unsafe impl Send for RegMap {}
// SAFETY: See the `Send` implementation. Every method takes `&self` and is
// one of the accessors the DDI allows to run concurrently.
unsafe impl Sync for RegMap {}

impl RegMap {
    /// Maps `len` bytes of register set `rnumber` of `dev`, starting `offset`
    /// bytes into the set, with `ddi_regs_map_setup(9F)`.
    ///
    /// A `len` of zero maps everything from `offset` to the end of the set.
    /// The size of the set comes from `ddi_dev_regsize(9F)`, and a request
    /// that does not fit inside the set fails here without reaching the
    /// mapping function. For a PCI device, register set 0 is configuration
    /// space, which [`crate::pci::PciConfig`] is for, and the base address
    /// registers follow from set 1.
    ///
    /// The mapping is removed when the returned value is dropped. Mapping
    /// may block, which is why the [`Sleepable`] token is required.
    pub fn map(
        dev: &DevInfo,
        rnumber: u32,
        offset: usize,
        len: usize,
        attr: &AccessAttr,
        _ctx: &Sleepable,
    ) -> Result<RegMap, DdiError> {
        let size = dev.regsize(rnumber)?;
        if offset >= size {
            return Err(DdiError);
        }
        let len = if len == 0 { size - offset } else { len };
        match offset.checked_add(len) {
            Some(end) if end <= size => {}
            _ => return Err(DdiError),
        }
        let off = sys::offset_t::try_from(offset).map_err(|_| DdiError)?;
        let olen = sys::offset_t::try_from(len).map_err(|_| DdiError)?;
        let raw_attr = attr.to_raw();
        let mut base: sys::caddr_t = core::ptr::null_mut();
        let mut handle: sys::ddi_acc_handle_t = core::ptr::null_mut();
        let r = unsafe { sys::ddi_regs_map_setup(dev.as_raw(), rnumber, &mut base, off, olen, &raw_attr, &mut handle) };
        DdiError::check(r)?;
        Ok(RegMap { handle, base: base.cast::<u8>(), len })
    }

    /// Returns the number of bytes that were mapped. Every valid offset is
    /// below this value.
    pub fn size(&self) -> usize {
        self.len
    }

    /// Returns the raw access handle, for `9F` functions that this crate
    /// does not wrap yet, such as `ddi_fm_acc_err_get(9F)`. The handle stays
    /// valid for as long as `self` exists.
    pub fn handle(&self) -> sys::ddi_acc_handle_t {
        self.handle
    }

    /// Checks that `count` elements of `T` starting at `off` lie inside the
    /// mapping and that the first one is aligned, and returns its device
    /// address.
    fn addr<T: RegValue>(&self, off: usize, count: usize) -> Result<*mut T, RegError> {
        let bytes = size_of::<T>().checked_mul(count).ok_or(RegError::OutOfRange)?;
        match off.checked_add(bytes) {
            Some(end) if end <= self.len => {}
            _ => return Err(RegError::OutOfRange),
        }
        let p = self.base.wrapping_add(off);
        if !(p as usize).is_multiple_of(size_of::<T>()) {
            return Err(RegError::Misaligned);
        }
        Ok(p.cast::<T>())
    }

    /// Reads one register of width `T` at byte offset `off`, using
    /// `ddi_get8(9F)`, `ddi_get16(9F)`, `ddi_get32(9F)` or `ddi_get64(9F)`.
    pub fn get<T: RegValue>(&self, off: usize) -> Result<T, RegError> {
        let p = self.addr::<T>(off, 1)?;
        // SAFETY: `addr` checked that the register lies inside the mapping
        // behind the handle and is aligned.
        Ok(unsafe { T::get(self.handle, p) })
    }

    /// Writes one register of width `T` at byte offset `off`, using
    /// `ddi_put8(9F)`, `ddi_put16(9F)`, `ddi_put32(9F)` or `ddi_put64(9F)`.
    pub fn put<T: RegValue>(&self, off: usize, value: T) -> Result<(), RegError> {
        let p = self.addr::<T>(off, 1)?;
        // SAFETY: As in `get`.
        unsafe { T::put(self.handle, p, value) };
        Ok(())
    }

    /// Reads `buf.len()` elements starting at byte offset `off` into `buf`,
    /// using `ddi_rep_get8(9F)` or one of its wider relatives. With
    /// [`RepMode::Increment`] the whole region must lie inside the mapping,
    /// with [`RepMode::Fixed`] only the one register.
    pub fn rep_get<T: RegValue>(&self, off: usize, buf: &mut [T], mode: RepMode) -> Result<(), RegError> {
        let span = match mode {
            RepMode::Increment => buf.len(),
            RepMode::Fixed => 1,
        };
        let p = self.addr::<T>(off, span)?;
        if buf.is_empty() {
            return Ok(());
        }
        // SAFETY: `addr` checked every device address the call touches, and
        // `buf` is valid for `buf.len()` writes.
        unsafe { T::rep_get(self.handle, buf.as_mut_ptr(), p, buf.len(), mode.flags()) };
        Ok(())
    }

    /// Writes the elements of `buf` starting at byte offset `off`, using
    /// `ddi_rep_put8(9F)` or one of its wider relatives. The range rules are
    /// those of [`RegMap::rep_get`].
    pub fn rep_put<T: RegValue>(&self, off: usize, buf: &[T], mode: RepMode) -> Result<(), RegError> {
        let span = match mode {
            RepMode::Increment => buf.len(),
            RepMode::Fixed => 1,
        };
        let p = self.addr::<T>(off, span)?;
        if buf.is_empty() {
            return Ok(());
        }
        // SAFETY: As in `rep_get`, with `buf` valid for reads.
        unsafe { T::rep_put(self.handle, buf.as_ptr(), p, buf.len(), mode.flags()) };
        Ok(())
    }

    /// Reports whether the framework has recorded a fault on this mapping,
    /// using `ddi_check_acc_handle(9F)`. `Err` means that values read
    /// through the handle can no longer be trusted and that writes may have
    /// been lost. `Ok` is not proof that the device is healthy, because not
    /// every platform detects every fault.
    pub fn check(&self) -> Result<(), DdiError> {
        DdiError::check(unsafe { sys::ddi_check_acc_handle(self.handle) })
    }
}

macro_rules! fixed_width {
    ($t:ty, $get:ident, $put:ident, $rget:ident, $rput:ident, $gname:literal, $pname:literal, $rgname:literal, $rpname:literal) => {
        impl RegMap {
            #[doc = concat!("Reads a `", stringify!($t), "` register at byte offset `off` with `", $gname, "(9F)`.")]
            pub fn $get(&self, off: usize) -> Result<$t, RegError> {
                self.get::<$t>(off)
            }
            #[doc = concat!("Writes a `", stringify!($t), "` register at byte offset `off` with `", $pname, "(9F)`.")]
            pub fn $put(&self, off: usize, value: $t) -> Result<(), RegError> {
                self.put::<$t>(off, value)
            }
            #[doc = concat!("Reads `", stringify!($t), "` elements into `buf` with `", $rgname, "(9F)`, as [`RegMap::rep_get`] describes.")]
            pub fn $rget(&self, off: usize, buf: &mut [$t], mode: RepMode) -> Result<(), RegError> {
                self.rep_get::<$t>(off, buf, mode)
            }
            #[doc = concat!("Writes the `", stringify!($t), "` elements of `buf` with `", $rpname, "(9F)`, as [`RegMap::rep_put`] describes.")]
            pub fn $rput(&self, off: usize, buf: &[$t], mode: RepMode) -> Result<(), RegError> {
                self.rep_put::<$t>(off, buf, mode)
            }
        }
    };
}
fixed_width!(u8, get8, put8, rep_get8, rep_put8, "ddi_get8", "ddi_put8", "ddi_rep_get8", "ddi_rep_put8");
fixed_width!(u16, get16, put16, rep_get16, rep_put16, "ddi_get16", "ddi_put16", "ddi_rep_get16", "ddi_rep_put16");
fixed_width!(u32, get32, put32, rep_get32, rep_put32, "ddi_get32", "ddi_put32", "ddi_rep_get32", "ddi_rep_put32");
fixed_width!(u64, get64, put64, rep_get64, rep_put64, "ddi_get64", "ddi_put64", "ddi_rep_get64", "ddi_rep_put64");

impl Drop for RegMap {
    fn drop(&mut self) {
        // SAFETY: The handle came from a successful `ddi_regs_map_setup(9F)`
        // and is freed exactly once. No accessor can run concurrently,
        // because `drop` has exclusive access.
        unsafe { sys::ddi_regs_map_free(&mut self.handle) }
    }
}

impl fmt::Debug for RegMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegMap").field("base", &self.base).field("len", &self.len).finish()
    }
}

/// A register of width `T` at byte offset `OFF`, as a constant a driver
/// writes down once from its datasheet.
///
/// ```ignore
/// const CTRL: Reg<u32, 0x00> = Reg::new();
/// const STATUS: Reg<u32, 0x04> = Reg::new();
/// const FIFO: Reg<u8, 0x10> = Reg::new();
/// ```
///
/// A `Reg` carries no data. [`RegBlock::read`] and [`RegBlock::write`] use
/// its type to check at compile time that the register lies inside the
/// block and is aligned to its width. Creating a `Reg` in a constant, as
/// above, reports a misaligned offset at the definition.
pub struct Reg<T: RegValue, const OFF: usize>(PhantomData<fn() -> T>);

impl<T: RegValue, const OFF: usize> Reg<T, OFF> {
    /// Names the register. The offset must be a multiple of the width.
    pub const fn new() -> Reg<T, OFF> {
        const { assert!(OFF.is_multiple_of(size_of::<T>()), "register offset is not a multiple of its width") };
        Reg(PhantomData)
    }

    /// The byte offset of the register.
    pub const OFFSET: usize = OFF;
}

impl<T: RegValue, const OFF: usize> Clone for Reg<T, OFF> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: RegValue, const OFF: usize> Copy for Reg<T, OFF> {}

impl<T: RegValue, const OFF: usize> Default for Reg<T, OFF> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RegValue, const OFF: usize> fmt::Debug for Reg<T, OFF> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reg<{}, {:#x}>", core::any::type_name::<T>(), OFF)
    }
}

/// A mapping known to be at least `LEN` bytes long, whose registers are
/// accessed through [`Reg`] constants without run-time checks or `Result`.
///
/// The block checks two things once, when it is created. The mapping must
/// be at least `LEN` bytes long, and its base address must be aligned to the
/// largest power of two that is at most 8 and at most `LEN`. The accessors
/// check at compile time that the register lies below `LEN` and that its
/// offset is a multiple of its width. Together these guarantee that every
/// access is inside the mapping and naturally aligned, because a register
/// that fits in `LEN` bytes is no wider than the alignment of the base.
///
/// The alignment rule is met by every register set mapped from its start,
/// since bus address decoders align a region to its size. It can only fail
/// when the driver maps from an unusual offset into the set.
///
/// A register outside the block is a compile error when the driver is
/// built. Because the check is evaluated when the generic accessor is
/// instantiated, `cargo check` may not report it, but `cargo build` does.
pub struct RegBlock<const LEN: usize> {
    map: RegMap,
}

impl<const LEN: usize> RegBlock<LEN> {
    /// The alignment the base address must have.
    const BASE_ALIGN: usize = {
        assert!(LEN > 0, "a register block cannot be empty");
        let p = 1usize << LEN.ilog2();
        if p < 8 { p } else { 8 }
    };

    /// Maps exactly `LEN` bytes of register set `rnumber`, starting at
    /// `offset`, with [`RegMap::map`], and checks the result as
    /// [`RegBlock::from_map`] does.
    pub fn map(
        dev: &DevInfo,
        rnumber: u32,
        offset: usize,
        attr: &AccessAttr,
        ctx: &Sleepable,
    ) -> Result<RegBlock<LEN>, DdiError> {
        let map = RegMap::map(dev, rnumber, offset, LEN, attr, ctx)?;
        RegBlock::from_map(map).map_err(|_| DdiError)
    }

    /// Turns an existing mapping into a block. The mapping is handed back
    /// unchanged if it is shorter than `LEN` or its base is not aligned as
    /// the type documentation describes.
    pub fn from_map(map: RegMap) -> Result<RegBlock<LEN>, RegMap> {
        if map.len >= LEN && (map.base as usize).is_multiple_of(Self::BASE_ALIGN) {
            Ok(RegBlock { map })
        } else {
            Err(map)
        }
    }

    /// Reads a register with the `ddi_get*(9F)` function of its width.
    #[inline]
    pub fn read<T: RegValue, const OFF: usize>(&self, _reg: Reg<T, OFF>) -> T {
        const { Self::assert_fits::<T, OFF>() };
        // SAFETY: `assert_fits` and `from_map` together guarantee that the
        // register lies inside the mapping and is aligned.
        unsafe { T::get(self.map.handle, self.map.base.wrapping_add(OFF).cast::<T>()) }
    }

    /// Writes a register with the `ddi_put*(9F)` function of its width.
    #[inline]
    pub fn write<T: RegValue, const OFF: usize>(&self, _reg: Reg<T, OFF>, value: T) {
        const { Self::assert_fits::<T, OFF>() };
        // SAFETY: As in `read`.
        unsafe { T::put(self.map.handle, self.map.base.wrapping_add(OFF).cast::<T>(), value) }
    }

    /// Reads a register, passes the value to `f`, and writes back what `f`
    /// returns. The two accesses are separate; nothing makes the sequence
    /// atomic with respect to the device or to other threads.
    #[inline]
    pub fn modify<T: RegValue, const OFF: usize>(&self, reg: Reg<T, OFF>, f: impl FnOnce(T) -> T) {
        let v = self.read(reg);
        self.write(reg, f(v));
    }

    const fn assert_fits<T: RegValue, const OFF: usize>() {
        assert!(OFF.is_multiple_of(size_of::<T>()), "register offset is not a multiple of its width");
        assert!(OFF < LEN && size_of::<T>() <= LEN - OFF, "register lies outside the register block");
    }

    /// Returns the underlying mapping, for the byte-offset and repeated
    /// accessors and for [`RegMap::check`].
    pub fn as_map(&self) -> &RegMap {
        &self.map
    }

    /// Gives up the compile-time view and returns the mapping.
    pub fn into_map(self) -> RegMap {
        self.map
    }
}

impl<const LEN: usize> fmt::Debug for RegBlock<LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegBlock").field("len", &LEN).field("map", &self.map).finish()
    }
}
