//! Module linkage and the glue between kernel entry points and Rust traits.
//!
//! A driver implements [`Driver`], implements [`CharDevice`] as well if it
//! is a character device, and invokes [`module!`](crate::module!). The macro emits the
//! `_init`, `_fini` and `_info` symbols the kernel looks for, together with
//! the `modlinkage`, `dev_ops` and `cb_ops` statics. The generic functions
//! in [`glue`] translate between the kernel's calling conventions and the
//! trait methods.
//!
//! Instance state lives in a soft-state slot, managed through the
//! `ddi_soft_state_*(9F)` functions and indexed by instance number. That is
//! what lets the framework answer `getinfo(9E)` and map a `dev_t` to its
//! instance in the `cb_ops` glue without any code in the driver.

use core::cell::UnsafeCell;
use core::ffi::CStr;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicPtr, Ordering};

use crate::context::Sleepable;
use crate::cred::Cred;
use crate::device::{DevInfo, Instance, Minor, OpenFlags, OpenType, OwnNode};
use crate::error::{DdiError, Errno};
use crate::ioctl::{DataModel, IoctlArg, IoctlCmd};
use crate::poll::{PollEvents, PollRegistration};
use crate::sys;
use crate::uio::Uio;

/// The behaviour a kernel module has to provide.
///
/// Only [`attach`](Driver::attach) is mandatory. The other methods have
/// defaults that are right for a driver without hardware state.
pub trait Driver: Sized + 'static {
    /// The state of one attached instance. [`attach`](Driver::attach)
    /// constructs it, the framework owns it, and [`detach`](Driver::detach)
    /// receives it back to drop.
    type Instance: Send + Sync + 'static;

    /// The description shown by `modinfo(8)`.
    const LINKINFO: &'static CStr;

    /// A hint for the initial size of the soft-state table. The table grows
    /// on demand, so this only affects the first allocation.
    const INSTANCE_HINT: usize = 1;

    /// Handles `attach(9E)` with `DDI_ATTACH`. Creates the instance state
    /// and, usually, the minor nodes.
    ///
    /// `node` is the capability for the node being attached, the only way to
    /// allocate the resources the kernel ties to the requesting device (DMA
    /// handles, interrupts, register mappings, PCI configuration handles).
    /// It dereferences to the node's [`DevInfo`] for queries. A driver that
    /// allocates such resources after attach keeps `node` in its instance
    /// state, which the framework destroys at detach; see [`OwnNode`].
    fn attach(node: OwnNode, ctx: &Sleepable) -> Result<Self::Instance, DdiError>;

    /// Handles `detach(9E)` with `DDI_DETACH`. Returning the instance inside
    /// `Err` refuses the detach, and the framework keeps the instance
    /// attached. The default drops the instance and succeeds.
    fn detach(inst: Self::Instance, _dev: DevInfo, _ctx: &Sleepable) -> Result<(), (Self::Instance, DdiError)> {
        drop(inst);
        Ok(())
    }

    /// Handles `attach(9E)` with `DDI_RESUME`. The default reports that
    /// resume is not supported.
    fn resume(_inst: &Self::Instance, _dev: DevInfo, _ctx: &Sleepable) -> Result<(), DdiError> {
        Err(DdiError)
    }

    /// Handles `detach(9E)` with `DDI_SUSPEND`. The default reports that
    /// suspend is not supported.
    fn suspend(_inst: &Self::Instance, _dev: DevInfo, _ctx: &Sleepable) -> Result<(), DdiError> {
        Err(DdiError)
    }

    /// Handles `quiesce(9E)`, which stops the hardware before a fast reboot.
    /// The kernel calls it with interrupts disabled, so the implementation
    /// must neither take locks nor block. The default reports that nothing
    /// needs to be done, like `ddi_quiesce_not_needed(9F)`.
    fn quiesce(_inst: &Self::Instance, _dev: DevInfo) -> Result<(), DdiError> {
        Ok(())
    }

    /// Maps a minor number to the instance that owns it. The default treats
    /// the minor number as the instance number, which is the common layout
    /// with one minor node per instance.
    fn instance_of_minor(minor: Minor) -> Option<Instance> {
        i32::try_from(minor.0).ok().map(Instance)
    }
}

/// The character device entry points, implemented on the instance type.
///
/// Every method has a default. `open` and `close` succeed, `read` and
/// `write` fail with `ENXIO`, and `ioctl` fails with `ENOTTY`.
pub trait CharDevice: Sized + Send + Sync + 'static {
    /// Handles `open(9E)`.
    fn open(
        &self,
        _minor: Minor,
        _flags: OpenFlags,
        _otyp: OpenType,
        _cred: &Cred<'_>,
        _ctx: &Sleepable,
    ) -> Result<(), Errno> {
        Ok(())
    }
    /// Handles `open(9E)` for a driver that uses clone opens.
    ///
    /// A character device normally sees one `open(9E)` per open(2) but only
    /// one `close(9E)`, on the last close of the device number, so it cannot
    /// keep state per open file. A clone open fixes that: the driver picks a
    /// fresh minor number for the new file and returns it here, the glue
    /// stores `makedevice(getmajor(dev), minor)` through the `devp` argument
    /// of `open(9E)`, and the kernel uses that device number for every later
    /// entry point of the file. The file therefore gets its own
    /// [`close`](CharDevice::close), called when its last reference goes
    /// away.
    ///
    /// Returning `Ok(None)` keeps the device number that was opened, which is
    /// what the default does after calling [`open`](CharDevice::open). A
    /// returned minor must map to the same instance through
    /// [`Driver::instance_of_minor`]. If it does not, the glue calls
    /// [`close`](CharDevice::close) with the returned minor, so that the
    /// driver can release what it set up, and fails the open with `ENXIO`.
    ///
    /// Keep clone minors at or below `L_MAXMIN32` (`0x3ffff`, 18 bits) if
    /// 32-bit processes may open the device: a larger minor number does not
    /// fit the 32-bit `dev_t`, and `fstat(2)` on the file fails with
    /// `EOVERFLOW` in such a process.
    fn clone_open(
        &self,
        minor: Minor,
        flags: OpenFlags,
        otyp: OpenType,
        cred: &Cred<'_>,
        ctx: &Sleepable,
    ) -> Result<Option<Minor>, Errno> {
        self.open(minor, flags, otyp, cred, ctx).map(|()| None)
    }
    /// Handles `close(9E)`.
    fn close(
        &self,
        _minor: Minor,
        _flags: OpenFlags,
        _otyp: OpenType,
        _cred: &Cred<'_>,
        _ctx: &Sleepable,
    ) -> Result<(), Errno> {
        Ok(())
    }
    /// Handles `read(9E)`. Returning `Ok(())` without copying anything
    /// signals end of file to the caller.
    fn read(&self, _minor: Minor, _uio: &mut Uio<'_>, _cred: &Cred<'_>, _ctx: &Sleepable) -> Result<(), Errno> {
        Err(Errno::ENXIO)
    }
    /// Handles `write(9E)`.
    fn write(&self, _minor: Minor, _uio: &mut Uio<'_>, _cred: &Cred<'_>, _ctx: &Sleepable) -> Result<(), Errno> {
        Err(Errno::ENXIO)
    }
    /// Handles `ioctl(9E)`. The value inside `Ok` becomes the return value
    /// of the system call.
    fn ioctl(
        &self,
        _minor: Minor,
        _cmd: IoctlCmd,
        _arg: IoctlArg<'_>,
        _cred: &Cred<'_>,
        _ctx: &Sleepable,
    ) -> Result<i32, Errno> {
        Err(Errno::ENOTTY)
    }
    /// Handles `devmap(9E)`, which validates a user mapping of the device
    /// and describes the memory behind it.
    ///
    /// The glue only calls this for a module declared with
    /// `ops: [char, devmap]`; that form installs a `segmap(9E)` entry point
    /// that runs [`CharDevice::segmap_check`] and then
    /// `ddi_devmap_segmap(9F)`, and sets `D_DEVMAP`, so `mmap(2)` on the
    /// device runs `devmap_setup(9F)`, which calls this method once for each
    /// piece of the requested range. `off` is the user offset at which the
    /// piece begins and `len` the length that is still to be mapped. The
    /// method must describe the memory by calling `devmap_devmem_setup(9F)`
    /// or `devmap_umem_setup(9F)` on `dhp` and return the number of bytes it
    /// validated, a non-zero multiple of the page size that is at most `len`
    /// rounded up to a whole page (`mmap(2)` passes the length unrounded);
    /// the framework calls it again for the rest. Any other length fails the
    /// mapping with `EINVAL`. `model` is the data model of
    /// the calling process.
    ///
    /// Any error fails the `mmap(2)`. `devmap_setup(9F)` reports every
    /// failure of this entry point to the caller as `ENXIO`, whatever the
    /// method returned. The default fails with `ENXIO`.
    ///
    /// No `devmap_map(9E)` or `devmap_unmap(9E)` callback is guaranteed to
    /// follow a successful call: if the framework cannot find an address for
    /// the mapping afterwards, it frees the handle without calling either.
    /// A driver that takes a reference here must therefore be able to
    /// release it without those callbacks.
    fn devmap(
        &self,
        _minor: Minor,
        _dhp: DevmapHandle<'_>,
        _off: u64,
        _len: usize,
        _model: DataModel,
        _ctx: &Sleepable,
    ) -> Result<usize, Errno> {
        Err(Errno::ENXIO)
    }
    /// Checks an `mmap(2)` of the minor device before the devmap framework
    /// creates the mapping, from the `segmap(9E)` entry point that
    /// `ops: [char, devmap]` installs (it then calls
    /// `ddi_devmap_segmap(9F)`). `req` holds the offset, length,
    /// protections and flags of the request (whether it is shared), which
    /// [`CharDevice::devmap`] does not see.
    ///
    /// Refuse here what must be refused: a `devmap_map(9E)` callback that
    /// fails leaves the new segment in the caller's address space until it
    /// exits (`devmap_setup` in `vm/seg_dev.c` frees the handles but does
    /// not unmap), which keeps the file mapped, and so open, until then.
    /// The default allows everything.
    fn segmap_check(&self, _minor: Minor, _req: &MapRequest, _ctx: &Sleepable) -> Result<(), Errno> {
        Ok(())
    }
    /// Handles `chpoll(9E)`: reports which of `events` are ready on the
    /// minor device.
    ///
    /// The glue only calls this for a module declared with
    /// `ops: [char, chpoll]` or `ops: [char, devmap, chpoll]`; with plain
    /// `[char]` the slot is `nochpoll`, and specfs then treats the device
    /// as always ready. Return the ready events (the glue masks them with
    /// `events`, but always passes `POLLERR`, `POLLHUP` and `POLLNVAL`
    /// through) and register a [`PollHead`](crate::PollHead) the instance
    /// owns (see [`PollRegistration`] for heads it does not own)
    /// in `reg` whenever nothing may be ready yet, that is always unless
    /// the returned set is known to be complete; the glue hands the head to
    /// the kernel exactly when `chpoll(9E)` requires it (nothing ready and
    /// `any_yet` false, or [`PollEvents::ET`] requested) and fails the call
    /// with `EINVAL` if it is required and none was registered, since the
    /// poller could otherwise never be woken. The call comes from `poll(2)`
    /// in the context of the polling process and must not block.
    fn chpoll<'s>(
        &'s self,
        _minor: Minor,
        _events: PollEvents,
        _any_yet: bool,
        _reg: &mut PollRegistration<'s>,
    ) -> Result<PollEvents, Errno> {
        Err(Errno::ENXIO)
    }
}

/// An `mmap(2)` request as `segmap(9E)` receives it; see
/// [`CharDevice::segmap_check`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapRequest {
    /// The offset into the device.
    pub off: u64,
    /// The length in bytes, as the caller gave it.
    pub len: usize,
    /// The requested protections (`PROT_*`).
    pub prot: u32,
    /// The protections the file's mode allows.
    pub maxprot: u32,
    /// The `mmap(2)` flags (`MAP_SHARED`, `MAP_PRIVATE`, `MAP_FIXED`, ...).
    pub flags: u32,
}

impl MapRequest {
    /// `MAP_TYPE`, the mask of the sharing type.
    const MAP_TYPE: u32 = 0xf;
    /// `MAP_SHARED`.
    const MAP_SHARED: u32 = 1;

    /// Reports whether the mapping is shared (`MAP_SHARED`); the devmap
    /// framework maps device memory itself, so a private (copy-on-write)
    /// mapping of it is rarely meaningful.
    pub fn is_shared(&self) -> bool {
        self.flags & Self::MAP_TYPE == Self::MAP_SHARED
    }
}

/// The mapping handle that `devmap(9E)` receives, a `devmap_cookie_t`.
///
/// It identifies the mapping being set up and is only valid for the
/// duration of the [`CharDevice::devmap`] call that receives it. The driver
/// passes [`DevmapHandle::as_raw`] to `devmap_umem_setup(9F)` or
/// `devmap_devmem_setup(9F)` together with its `devmap_callback_ctl(9S)`;
/// the callbacks registered there receive the same handle again, as a raw
/// `devmap_cookie_t`, for as long as the mapping exists.
///
/// Both setup functions take a device node, and it must be the driver's
/// own: pass `as_raw()` of the [`OwnNode`] the instance keeps. For
/// `devmap_devmem_setup(9F)` and `devmap_devmem_remap(9F)` this is required
/// for the same reason as for [`RegMap`](crate::RegMap): they map the
/// register set through `ddi_map`, which asks the node's parent nexus and
/// dereferences the missing parent of the root node. `devmap_umem_setup(9F)`
/// and `devmap_umem_remap(9F)` do not use the node at all in illumos
/// `cda5b9a49f` (`vm/seg_dev.c`), but the manual gives no other meaning to
/// it. The setup functions are not wrapped yet; a safe wrapper will take
/// `&OwnNode` and apply its guard, as `docs/DESIGN.md` section 5.12 records.
pub struct DevmapHandle<'a> {
    raw: sys::devmap_cookie_t,
    _call: core::marker::PhantomData<&'a ()>,
}

impl DevmapHandle<'_> {
    /// Returns the raw handle, for the `devmap_*_setup(9F)` functions.
    pub fn as_raw(&self) -> sys::devmap_cookie_t {
        self.raw
    }
}

/// Static storage with interior mutability, for structures the kernel writes
/// to after the module is loaded, such as `dev_ops` with its reference count
/// and `modlinkage`.
#[repr(transparent)]
pub struct StaticCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for StaticCell<T> {}

impl<T> StaticCell<T> {
    /// Wraps a value.
    pub const fn new(v: T) -> StaticCell<T> {
        StaticCell(UnsafeCell::new(v))
    }
    /// Returns a raw pointer to the contents.
    pub const fn get(&self) -> *mut T {
        self.0.get()
    }
}

/// The soft-state item of one instance. It records the device node, whether
/// the instance is live, and the instance state itself.
#[repr(C)]
pub struct Slot<I> {
    dip: sys::dev_info_t,
    live: bool,
    inst: MaybeUninit<I>,
}

/// The framework's state for one module. The [`module!`](crate::module!) macro creates one
/// static of this type per invocation.
pub struct ModuleState<D: Driver> {
    soft: AtomicPtr<sys::c_void>,
    _d: core::marker::PhantomData<fn() -> D>,
}

impl<D: Driver> ModuleState<D> {
    /// Creates the state for a module that has not been initialised yet.
    pub const fn new() -> ModuleState<D> {
        ModuleState { soft: AtomicPtr::new(core::ptr::null_mut()), _d: core::marker::PhantomData }
    }

    fn soft(&self) -> *mut sys::c_void {
        self.soft.load(Ordering::Acquire)
    }

    /// Finds the soft-state slot for `instance` if it exists and is live.
    unsafe fn slot(&self, instance: Instance) -> Option<&Slot<D::Instance>> {
        let p = unsafe { sys::ddi_get_soft_state(self.soft(), instance.0) } as *const Slot<D::Instance>;
        if p.is_null() {
            return None;
        }
        let s = unsafe { &*p };
        if s.live { Some(s) } else { None }
    }

    unsafe fn instance_for_dev(&self, dev: sys::dev_t) -> Option<(Minor, &D::Instance)> {
        let minor = Minor(unsafe { sys::getminor(dev) });
        let inst = D::instance_of_minor(minor)?;
        let slot = unsafe { self.slot(inst) }?;
        Some((minor, unsafe { slot.inst.assume_init_ref() }))
    }
}

impl<D: Driver> Default for ModuleState<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// Implemented by [`module!`](crate::module!) on the driver type so that the glue can reach
/// the module's statics.
pub trait ModuleDef: Driver {
    /// Returns the module's framework state.
    fn state() -> &'static ModuleState<Self>;
    /// Returns a pointer to the module's `modlinkage`.
    fn linkage() -> *mut sys::modlinkage;
}

/// The C-ABI functions the macro points the kernel at.
///
/// All of them are generic over the driver type and are monomorphised once
/// per module. Each is an entry point that the kernel calls with arguments
/// whose validity the DDI guarantees (see `Intro(9E)`). They are `unsafe`
/// only because a Rust caller could not uphold those guarantees. Driver
/// code never calls them directly.
#[allow(clippy::missing_safety_doc)]
pub mod glue {
    use super::*;

    const fn align_ok<I>() {
        assert!(core::mem::align_of::<Slot<I>>() <= 8, "instance state must not need more than 8-byte alignment");
    }

    /// The body of `_init(9E)`. Creates the soft-state table and installs
    /// the module.
    pub unsafe fn init<D: ModuleDef>() -> sys::c_int {
        const { align_ok::<D::Instance>() };
        let st = D::state();
        let mut soft: *mut sys::c_void = core::ptr::null_mut();
        let r =
            unsafe { sys::ddi_soft_state_init(&mut soft, core::mem::size_of::<Slot<D::Instance>>(), D::INSTANCE_HINT) };
        if r != 0 {
            return r;
        }
        st.soft.store(soft, Ordering::Release);
        let r = unsafe { sys::mod_install(D::linkage()) };
        if r != 0 {
            unsafe { sys::ddi_soft_state_fini(&mut soft) };
            st.soft.store(core::ptr::null_mut(), Ordering::Release);
        }
        r
    }

    /// The body of `_fini(9E)`. Removes the module and, if that succeeded,
    /// destroys the soft-state table.
    pub unsafe fn fini<D: ModuleDef>() -> sys::c_int {
        let r = unsafe { sys::mod_remove(D::linkage()) };
        if r == 0 {
            let st = D::state();
            let mut soft = st.soft.swap(core::ptr::null_mut(), Ordering::AcqRel);
            unsafe { sys::ddi_soft_state_fini(&mut soft) };
        }
        r
    }

    /// The body of `_info(9E)`.
    pub unsafe fn info<D: ModuleDef>(modinfop: *mut sys::modinfo) -> sys::c_int {
        unsafe { sys::mod_info(D::linkage(), modinfop) }
    }

    /// The `getinfo(9E)` entry point. Answers both the instance-number and
    /// the device-node query from the soft-state table.
    pub unsafe extern "C" fn getinfo<D: ModuleDef>(
        _dip: sys::dev_info_t,
        cmd: sys::ddi_info_cmd_t,
        arg: *mut sys::c_void,
        result: *mut *mut sys::c_void,
    ) -> sys::c_int {
        let dev = arg as usize as sys::dev_t;
        let minor = Minor(unsafe { sys::getminor(dev) });
        let Some(inst) = D::instance_of_minor(minor) else { return sys::DDI_FAILURE };
        match cmd {
            sys::ddi_info_cmd_t::DDI_INFO_DEVT2INSTANCE => {
                unsafe { *result = inst.0 as usize as *mut sys::c_void };
                sys::DDI_SUCCESS
            }
            sys::ddi_info_cmd_t::DDI_INFO_DEVT2DEVINFO => match unsafe { D::state().slot(inst) } {
                Some(s) => {
                    unsafe { *result = s.dip as *mut sys::c_void };
                    sys::DDI_SUCCESS
                }
                None => sys::DDI_FAILURE,
            },
            _ => sys::DDI_FAILURE,
        }
    }

    /// The `attach(9E)` entry point. Allocates the soft-state slot, calls
    /// [`Driver::attach`] and stores the result in the slot.
    pub unsafe extern "C" fn attach<D: ModuleDef>(dip: sys::dev_info_t, cmd: sys::ddi_attach_cmd_t) -> sys::c_int {
        let st = D::state();
        let dev = unsafe { DevInfo::from_raw(dip) };
        let inst = dev.instance();
        let ctx = unsafe { Sleepable::assume() };
        match cmd {
            sys::ddi_attach_cmd_t::DDI_ATTACH => {
                if unsafe { sys::ddi_soft_state_zalloc(st.soft(), inst.0) } != sys::DDI_SUCCESS {
                    return sys::DDI_FAILURE;
                }
                let slot = unsafe { sys::ddi_get_soft_state(st.soft(), inst.0) } as *mut Slot<D::Instance>;
                // The only place an OwnNode is made from a kernel-supplied
                // node: the node of this attach, which the instance state
                // (where the driver keeps it) cannot outlive.
                let node = unsafe { OwnNode::attached(dip) };
                match D::attach(node, &ctx) {
                    Ok(state) => unsafe {
                        (*slot).dip = dip;
                        (*slot).inst.write(state);
                        (*slot).live = true;
                        sys::DDI_SUCCESS
                    },
                    Err(DdiError) => {
                        unsafe { sys::ddi_soft_state_free(st.soft(), inst.0) };
                        sys::DDI_FAILURE
                    }
                }
            }
            sys::ddi_attach_cmd_t::DDI_RESUME => match unsafe { st.slot(inst) } {
                Some(s) => DdiError::into_c(D::resume(unsafe { s.inst.assume_init_ref() }, dev, &ctx)),
                None => sys::DDI_FAILURE,
            },
            _ => sys::DDI_FAILURE,
        }
    }

    /// The `detach(9E)` entry point. Takes the instance out of its slot,
    /// calls [`Driver::detach`], and either frees the slot or, if the driver
    /// refused, puts the instance back.
    pub unsafe extern "C" fn detach<D: ModuleDef>(dip: sys::dev_info_t, cmd: sys::ddi_detach_cmd_t) -> sys::c_int {
        let st = D::state();
        let dev = unsafe { DevInfo::from_raw(dip) };
        let inst = dev.instance();
        let ctx = unsafe { Sleepable::assume() };
        match cmd {
            sys::ddi_detach_cmd_t::DDI_DETACH => {
                let slot = unsafe { sys::ddi_get_soft_state(st.soft(), inst.0) } as *mut Slot<D::Instance>;
                if slot.is_null() || !unsafe { (*slot).live } {
                    return sys::DDI_FAILURE;
                }
                let state = unsafe {
                    (*slot).live = false;
                    (*slot).inst.assume_init_read()
                };
                match D::detach(state, dev, &ctx) {
                    Ok(()) => {
                        unsafe {
                            sys::ddi_remove_minor_node(dip, core::ptr::null());
                            sys::ddi_soft_state_free(st.soft(), inst.0);
                        }
                        sys::DDI_SUCCESS
                    }
                    Err((state, DdiError)) => {
                        unsafe {
                            (*slot).inst.write(state);
                            (*slot).live = true;
                        }
                        sys::DDI_FAILURE
                    }
                }
            }
            sys::ddi_detach_cmd_t::DDI_SUSPEND => match unsafe { st.slot(inst) } {
                Some(s) => DdiError::into_c(D::suspend(unsafe { s.inst.assume_init_ref() }, dev, &ctx)),
                None => sys::DDI_FAILURE,
            },
            _ => sys::DDI_FAILURE,
        }
    }

    /// The `quiesce(9E)` entry point.
    pub unsafe extern "C" fn quiesce<D: ModuleDef>(dip: sys::dev_info_t) -> sys::c_int {
        let dev = unsafe { DevInfo::from_raw(dip) };
        match unsafe { D::state().slot(dev.instance()) } {
            Some(s) => DdiError::into_c(D::quiesce(unsafe { s.inst.assume_init_ref() }, dev)),
            None => sys::DDI_SUCCESS,
        }
    }

    /// The `open(9E)` entry point for a [`CharDevice`].
    pub unsafe extern "C" fn open<D: ModuleDef>(
        devp: *mut sys::dev_t,
        flag: sys::c_int,
        otyp: sys::c_int,
        credp: *mut sys::cred_t,
    ) -> sys::c_int
    where
        D::Instance: CharDevice,
    {
        let dev = unsafe { *devp };
        let Some((minor, inst)) = (unsafe { D::state().instance_for_dev(dev) }) else { return sys::errno::ENXIO };
        let cred = unsafe { Cred::from_raw(credp) };
        let ctx = unsafe { Sleepable::assume() };
        let (flags, otyp) = (OpenFlags(flag), OpenType::from_raw(otyp));
        match inst.clone_open(minor, flags, otyp, &cred, &ctx) {
            Ok(None) => 0,
            Ok(Some(clone)) => {
                if D::instance_of_minor(clone) != D::instance_of_minor(minor) {
                    // The driver's open state is keyed by the clone minor, so
                    // let it undo that before the open fails.
                    let _ = inst.close(clone, flags, otyp, &cred, &ctx);
                    return sys::errno::ENXIO;
                }
                unsafe { *devp = sys::makedevice(sys::getmajor(dev), clone.0) };
                0
            }
            Err(e) => e.get(),
        }
    }

    /// The `close(9E)` entry point for a [`CharDevice`].
    pub unsafe extern "C" fn close<D: ModuleDef>(
        dev: sys::dev_t,
        flag: sys::c_int,
        otyp: sys::c_int,
        credp: *mut sys::cred_t,
    ) -> sys::c_int
    where
        D::Instance: CharDevice,
    {
        let Some((minor, inst)) = (unsafe { D::state().instance_for_dev(dev) }) else { return sys::errno::ENXIO };
        let cred = unsafe { Cred::from_raw(credp) };
        let ctx = unsafe { Sleepable::assume() };
        Errno::into_c(inst.close(minor, OpenFlags(flag), OpenType::from_raw(otyp), &cred, &ctx))
    }

    /// The `read(9E)` entry point for a [`CharDevice`].
    pub unsafe extern "C" fn read<D: ModuleDef>(
        dev: sys::dev_t,
        uiop: *mut sys::uio_t,
        credp: *mut sys::cred_t,
    ) -> sys::c_int
    where
        D::Instance: CharDevice,
    {
        let Some((minor, inst)) = (unsafe { D::state().instance_for_dev(dev) }) else { return sys::errno::ENXIO };
        let cred = unsafe { Cred::from_raw(credp) };
        let mut uio = unsafe { Uio::from_raw(uiop) };
        let ctx = unsafe { Sleepable::assume() };
        Errno::into_c(inst.read(minor, &mut uio, &cred, &ctx))
    }

    /// The `write(9E)` entry point for a [`CharDevice`].
    pub unsafe extern "C" fn write<D: ModuleDef>(
        dev: sys::dev_t,
        uiop: *mut sys::uio_t,
        credp: *mut sys::cred_t,
    ) -> sys::c_int
    where
        D::Instance: CharDevice,
    {
        let Some((minor, inst)) = (unsafe { D::state().instance_for_dev(dev) }) else { return sys::errno::ENXIO };
        let cred = unsafe { Cred::from_raw(credp) };
        let mut uio = unsafe { Uio::from_raw(uiop) };
        let ctx = unsafe { Sleepable::assume() };
        Errno::into_c(inst.write(minor, &mut uio, &cred, &ctx))
    }

    /// The `ioctl(9E)` entry point for a [`CharDevice`].
    pub unsafe extern "C" fn ioctl<D: ModuleDef>(
        dev: sys::dev_t,
        cmd: sys::c_int,
        arg: sys::intptr_t,
        mode: sys::c_int,
        credp: *mut sys::cred_t,
        rvalp: *mut sys::c_int,
    ) -> sys::c_int
    where
        D::Instance: CharDevice,
    {
        let Some((minor, inst)) = (unsafe { D::state().instance_for_dev(dev) }) else { return sys::errno::ENXIO };
        let cred = unsafe { Cred::from_raw(credp) };
        let ctx = unsafe { Sleepable::assume() };
        match inst.ioctl(minor, IoctlCmd(cmd), IoctlArg::from_entry(arg, mode, dev), &cred, &ctx) {
            Ok(rv) => {
                if !rvalp.is_null() {
                    unsafe { *rvalp = rv };
                }
                0
            }
            Err(e) => e.get(),
        }
    }

    /// The `devmap(9E)` entry point for a [`CharDevice`], installed by the
    /// `ops: [char, devmap]` form of [`module!`](crate::module!).
    pub unsafe extern "C" fn devmap<D: ModuleDef>(
        dev: sys::dev_t,
        dhp: sys::devmap_cookie_t,
        off: sys::offset_t,
        len: sys::size_t,
        maplen: *mut sys::size_t,
        model: sys::uint_t,
    ) -> sys::c_int
    where
        D::Instance: CharDevice,
    {
        let Some((minor, inst)) = (unsafe { D::state().instance_for_dev(dev) }) else { return sys::errno::ENXIO };
        // The devmap framework's segdev_create looks the device's vnode up
        // with specfind, which only finds vnodes whose path is visible in
        // the caller's zone, and dereferences NULL otherwise (a descriptor
        // passed into another zone, a vnode without a path). Refuse such a
        // mapping here, before the framework gets there
        // (docs/DESIGN.md section 5.12). The lookup is private, so only
        // modules built with the private tier have the guard; only they can
        // create pathless vnodes.
        #[cfg(feature = "unstable-gate-cda5b9a49f")]
        if !crate::private::fd::mappable(dev) {
            return sys::errno::ENXIO;
        }
        // A negative offset cannot name anything; offset_t is signed only
        // because it is shared with seek offsets.
        let Ok(off) = u64::try_from(off) else { return sys::errno::EINVAL };
        // devmap(9E) is called from mmap(2), in the context of the mapping
        // process, where blocking is allowed.
        let ctx = unsafe { Sleepable::assume() };
        // `model` is the caller's DATAMODEL_* value, which shares its bits
        // with DDI_MODEL_*.
        let ilp32 = (model as sys::c_int) & sys::DDI_MODEL_MASK == sys::model_t::DDI_MODEL_ILP32.0;
        let model = if ilp32 { DataModel::Ilp32 } else { DataModel::Lp64 };
        let handle = DevmapHandle { raw: dhp, _call: core::marker::PhantomData };
        // mmap(2) passes the length as the caller gave it, so the last piece
        // may be validated up to the end of its page, but not beyond.
        let page = unsafe { sys::ptob(1) } as usize;
        match inst.devmap(minor, handle, off, len, model, &ctx) {
            Ok(n) if n != 0 && n % page == 0 && (n <= len || n - len < page) && !maplen.is_null() => {
                unsafe { *maplen = n };
                0
            }
            // Nothing, a partial page or more than was asked for is a driver
            // bug; refuse the mapping rather than let the framework map beyond
            // the request, reject the length, or call again for ever (its
            // loop only advances by the validated length).
            Ok(_) => sys::errno::EINVAL,
            Err(e) => e.get(),
        }
    }

    /// The `chpoll(9E)` entry point for a [`CharDevice`], installed by the
    /// `ops: [char, chpoll]` and `ops: [char, devmap, chpoll]` forms of
    /// [`module!`](crate::module!). Implements the algorithm of the manual
    /// around [`CharDevice::chpoll`].
    pub unsafe extern "C" fn chpoll<D: ModuleDef>(
        dev: sys::dev_t,
        events: sys::c_short,
        anyyet: sys::c_int,
        reventsp: *mut sys::c_short,
        phpp: *mut *mut sys::pollhead,
    ) -> sys::c_int
    where
        D::Instance: CharDevice,
    {
        let Some((minor, inst)) = (unsafe { D::state().instance_for_dev(dev) }) else { return sys::errno::ENXIO };
        if reventsp.is_null() || phpp.is_null() {
            return sys::errno::EINVAL;
        }
        let events = PollEvents(events);
        let mut reg = PollRegistration::new();
        match inst.chpoll(minor, events, anyyet != 0, &mut reg) {
            Ok(ready) => {
                // POLLERR, POLLHUP and POLLNVAL are reported whether asked
                // for or not, as poll(2) specifies.
                let always = PollEvents::ERR.union(PollEvents::HUP).union(PollEvents::NVAL);
                let revents = ready.intersection(events.union(always));
                let need_head = (revents.is_empty() && anyyet == 0) || events.intersects(PollEvents::ET);
                if need_head {
                    if reg.raw().is_null() {
                        return sys::errno::EINVAL;
                    }
                    unsafe { *phpp = reg.raw() };
                }
                unsafe { *reventsp = revents.0 };
                0
            }
            Err(e) => e.get(),
        }
    }

    /// The `segmap(9E)` entry point of `ops: [char, devmap]`: runs
    /// [`CharDevice::segmap_check`], then `ddi_devmap_segmap(9F)`.
    #[allow(clippy::too_many_arguments, reason = "the shape of segmap(9E)")]
    pub unsafe extern "C" fn segmap<D: ModuleDef>(
        dev: sys::dev_t,
        off: sys::off_t,
        asp: *mut sys::addrspace,
        addrp: *mut sys::caddr_t,
        len: sys::off_t,
        prot: sys::c_uint,
        maxprot: sys::c_uint,
        flags: sys::c_uint,
        credp: *mut sys::cred_t,
    ) -> sys::c_int
    where
        D::Instance: CharDevice,
    {
        let Some((minor, inst)) = (unsafe { D::state().instance_for_dev(dev) }) else { return sys::errno::ENXIO };
        let (Ok(uoff), Ok(ulen)) = (u64::try_from(off), usize::try_from(len)) else { return sys::errno::EINVAL };
        // segmap(9E) is called from mmap(2), in the mapping process.
        let ctx = unsafe { Sleepable::assume() };
        let req = MapRequest { off: uoff, len: ulen, prot, maxprot, flags };
        if let Err(e) = inst.segmap_check(minor, &req, &ctx) {
            return e.get();
        }
        unsafe { sys::ddi_devmap_segmap(dev, off, asp, addrp, len, prot, maxprot, flags, credp) }
    }

    /// A stub with the signature of `identify(9E)` and `probe(9E)` that
    /// returns 0, standing in for the kernel's `nulldev`.
    pub unsafe extern "C" fn nulldev_dip(_dip: sys::dev_info_t) -> sys::c_int {
        0
    }
    /// A stub with the signature of `devo_reset` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_reset(_dip: sys::dev_info_t, _cmd: sys::ddi_reset_cmd_t) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `power(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_power(_dip: sys::dev_info_t, _c: sys::c_int, _l: sys::c_int) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `strategy(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_strategy(_bp: *mut sys::buf) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `print(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_print(_dev: sys::dev_t, _s: *mut sys::c_char) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `dump(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_dump(
        _dev: sys::dev_t,
        _a: sys::caddr_t,
        _b: sys::daddr_t,
        _n: sys::c_int,
    ) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `devmap(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_devmap(
        _dev: sys::dev_t,
        _dhp: sys::devmap_cookie_t,
        _off: sys::offset_t,
        _len: sys::size_t,
        _maplen: *mut sys::size_t,
        _model: sys::uint_t,
    ) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `mmap(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_mmap(_dev: sys::dev_t, _off: sys::off_t, _prot: sys::c_int) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `segmap(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_segmap(
        _dev: sys::dev_t,
        _off: sys::off_t,
        _asp: *mut sys::addrspace,
        _addrp: *mut sys::caddr_t,
        _len: sys::off_t,
        _prot: sys::c_uint,
        _maxprot: sys::c_uint,
        _flags: sys::c_uint,
        _credp: *mut sys::cred_t,
    ) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `open(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_open(
        _devp: *mut sys::dev_t,
        _f: sys::c_int,
        _o: sys::c_int,
        _c: *mut sys::cred_t,
    ) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `close(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_close(
        _dev: sys::dev_t,
        _f: sys::c_int,
        _o: sys::c_int,
        _c: *mut sys::cred_t,
    ) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the shared signature of `read(9E)` and `write(9E)` that
    /// returns `ENXIO`.
    pub unsafe extern "C" fn nodev_rw(_dev: sys::dev_t, _u: *mut sys::uio_t, _c: *mut sys::cred_t) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the signature of `ioctl(9E)` that returns `ENXIO`.
    pub unsafe extern "C" fn nodev_ioctl(
        _dev: sys::dev_t,
        _cmd: sys::c_int,
        _arg: sys::intptr_t,
        _mode: sys::c_int,
        _c: *mut sys::cred_t,
        _rv: *mut sys::c_int,
    ) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// A stub with the shared signature of `aread(9E)` and `awrite(9E)` that
    /// returns `ENXIO`.
    pub unsafe extern "C" fn nodev_aio(_dev: sys::dev_t, _a: *mut sys::aio_req, _c: *mut sys::cred_t) -> sys::c_int {
        sys::errno::ENXIO
    }
}

/// Defines a kernel module for a [`Driver`].
///
/// The `ops` list names the entry-point families the instance type
/// implements. `[char]` wires the [`CharDevice`] methods into `cb_ops`;
/// `[char, devmap]` does the same and also makes the device mappable
/// through the devmap framework: [`CharDevice::devmap`] becomes the
/// `devmap(9E)` entry point, glue that runs [`CharDevice::segmap_check`]
/// and then `ddi_devmap_segmap(9F)` the `segmap(9E)` entry point, and
/// `D_DEVMAP` is set in `cb_flag`. Adding `chpoll`
/// (`[char, chpoll]`, `[char, devmap, chpoll]`) installs
/// [`CharDevice::chpoll`] as the `chpoll(9E)` entry point instead of
/// `nochpoll`, so that `poll(2)` waits on the device. An empty list produces a
/// module whose `cb_ops` slots all return `ENXIO`.
///
/// ```ignore
/// ddi::module! {
///     driver: RNull,
///     ops: [char],
/// }
/// ```
#[macro_export]
macro_rules! module {
    (driver: $drv:ty, ops: [char] $(,)?) => {
        $crate::module!(@common $drv);
        $crate::module!(@cb_ops $drv,
            open: Some($crate::module::glue::open::<$drv>),
            close: Some($crate::module::glue::close::<$drv>),
            read: Some($crate::module::glue::read::<$drv>),
            write: Some($crate::module::glue::write::<$drv>),
            ioctl: Some($crate::module::glue::ioctl::<$drv>),
            devmap: Some($crate::module::glue::nodev_devmap),
            segmap: Some($crate::module::glue::nodev_segmap),
            chpoll: Some($crate::sys::nochpoll),
            flag: 0);
    };
    (driver: $drv:ty, ops: [char, chpoll] $(,)?) => {
        $crate::module!(@common $drv);
        $crate::module!(@cb_ops $drv,
            open: Some($crate::module::glue::open::<$drv>),
            close: Some($crate::module::glue::close::<$drv>),
            read: Some($crate::module::glue::read::<$drv>),
            write: Some($crate::module::glue::write::<$drv>),
            ioctl: Some($crate::module::glue::ioctl::<$drv>),
            devmap: Some($crate::module::glue::nodev_devmap),
            segmap: Some($crate::module::glue::nodev_segmap),
            chpoll: Some($crate::module::glue::chpoll::<$drv>),
            flag: 0);
    };
    (driver: $drv:ty, ops: [char, devmap, chpoll] $(,)?) => {
        $crate::module!(@common $drv);
        $crate::module!(@cb_ops $drv,
            open: Some($crate::module::glue::open::<$drv>),
            close: Some($crate::module::glue::close::<$drv>),
            read: Some($crate::module::glue::read::<$drv>),
            write: Some($crate::module::glue::write::<$drv>),
            ioctl: Some($crate::module::glue::ioctl::<$drv>),
            devmap: Some($crate::module::glue::devmap::<$drv>),
            segmap: Some($crate::module::glue::segmap::<$drv>),
            chpoll: Some($crate::module::glue::chpoll::<$drv>),
            flag: $crate::sys::D_DEVMAP);
    };
    (driver: $drv:ty, ops: [char, devmap] $(,)?) => {
        $crate::module!(@common $drv);
        $crate::module!(@cb_ops $drv,
            open: Some($crate::module::glue::open::<$drv>),
            close: Some($crate::module::glue::close::<$drv>),
            read: Some($crate::module::glue::read::<$drv>),
            write: Some($crate::module::glue::write::<$drv>),
            ioctl: Some($crate::module::glue::ioctl::<$drv>),
            devmap: Some($crate::module::glue::devmap::<$drv>),
            segmap: Some($crate::module::glue::segmap::<$drv>),
            chpoll: Some($crate::sys::nochpoll),
            flag: $crate::sys::D_DEVMAP);
    };
    (driver: $drv:ty, ops: [] $(,)?) => {
        $crate::module!(@common $drv);
        $crate::module!(@cb_ops $drv,
            open: Some($crate::module::glue::nodev_open),
            close: Some($crate::module::glue::nodev_close),
            read: Some($crate::module::glue::nodev_rw),
            write: Some($crate::module::glue::nodev_rw),
            ioctl: Some($crate::module::glue::nodev_ioctl),
            devmap: Some($crate::module::glue::nodev_devmap),
            segmap: Some($crate::module::glue::nodev_segmap),
            chpoll: Some($crate::sys::nochpoll),
            flag: 0);
    };
    (@cb_ops $drv:ty, open: $open:expr, close: $close:expr, read: $read:expr, write: $write:expr, ioctl: $ioctl:expr,
     devmap: $devmap:expr, segmap: $segmap:expr, chpoll: $chpoll:expr, flag: $flag:expr) => {
        static __DDI_CB_OPS: $crate::module::StaticCell<$crate::sys::cb_ops> =
            $crate::module::StaticCell::new($crate::sys::cb_ops {
                cb_open: $open,
                cb_close: $close,
                cb_strategy: Some($crate::module::glue::nodev_strategy),
                cb_print: Some($crate::module::glue::nodev_print),
                cb_dump: Some($crate::module::glue::nodev_dump),
                cb_read: $read,
                cb_write: $write,
                cb_ioctl: $ioctl,
                cb_devmap: $devmap,
                cb_mmap: Some($crate::module::glue::nodev_mmap),
                cb_segmap: $segmap,
                cb_chpoll: $chpoll,
                cb_prop_op: Some($crate::sys::ddi_prop_op),
                cb_str: ::core::ptr::null_mut(),
                cb_flag: $crate::sys::D_MP | $crate::sys::D_64BIT | $crate::sys::D_NEW | $flag,
                cb_rev: $crate::sys::CB_REV,
                cb_aread: Some($crate::module::glue::nodev_aio),
                cb_awrite: Some($crate::module::glue::nodev_aio),
            });
    };
    (@common $drv:ty) => {
        static __DDI_STATE: $crate::module::ModuleState<$drv> = $crate::module::ModuleState::new();

        static __DDI_DEV_OPS: $crate::module::StaticCell<$crate::sys::dev_ops> =
            $crate::module::StaticCell::new($crate::sys::dev_ops {
                devo_rev: $crate::sys::DEVO_REV,
                devo_refcnt: 0,
                devo_getinfo: Some($crate::module::glue::getinfo::<$drv>),
                devo_identify: Some($crate::module::glue::nulldev_dip),
                devo_probe: Some($crate::module::glue::nulldev_dip),
                devo_attach: Some($crate::module::glue::attach::<$drv>),
                devo_detach: Some($crate::module::glue::detach::<$drv>),
                devo_reset: Some($crate::module::glue::nodev_reset),
                devo_cb_ops: __DDI_CB_OPS.get(),
                devo_bus_ops: ::core::ptr::null_mut(),
                devo_power: Some($crate::module::glue::nodev_power),
                devo_quiesce: Some($crate::module::glue::quiesce::<$drv>),
            });

        static __DDI_MODLDRV: $crate::module::StaticCell<$crate::sys::modldrv> =
            $crate::module::StaticCell::new($crate::sys::modldrv {
                drv_modops: &raw const $crate::sys::mod_driverops as *mut $crate::sys::mod_ops,
                drv_linkinfo: <$drv as $crate::module::Driver>::LINKINFO.as_ptr(),
                drv_dev_ops: __DDI_DEV_OPS.get(),
            });

        static __DDI_MODLINKAGE: $crate::module::StaticCell<$crate::sys::modlinkage> =
            $crate::module::StaticCell::new($crate::sys::modlinkage {
                ml_rev: $crate::sys::MODREV_1,
                ml_linkage: [
                    __DDI_MODLDRV.get() as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut(), ::core::ptr::null_mut(), ::core::ptr::null_mut(),
                    ::core::ptr::null_mut(), ::core::ptr::null_mut(), ::core::ptr::null_mut(),
                ],
            });

        impl $crate::module::ModuleDef for $drv {
            fn state() -> &'static $crate::module::ModuleState<Self> { &__DDI_STATE }
            fn linkage() -> *mut $crate::sys::modlinkage { __DDI_MODLINKAGE.get() }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn _init() -> ::core::ffi::c_int {
            unsafe { $crate::module::glue::init::<$drv>() }
        }
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn _fini() -> ::core::ffi::c_int {
            unsafe { $crate::module::glue::fini::<$drv>() }
        }
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn _info(modinfop: *mut $crate::sys::modinfo) -> ::core::ffi::c_int {
            unsafe { $crate::module::glue::info::<$drv>(modinfop) }
        }
    };
}
