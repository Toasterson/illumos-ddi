//! Module linkage and entry-point glue.
//!
//! A driver implements [`Driver`] (and [`CharDevice`] for a character
//! device) and invokes [`module!`]. The macro emits the C-ABI symbols the
//! kernel looks for and the `modlinkage`/`dev_ops`/`cb_ops` statics; the
//! generic functions in [`glue`] translate between the kernel's calling
//! conventions and the traits.
//!
//! Instance state lives in a soft-state slot (`ddi_soft_state_*(9F)`)
//! indexed by instance number, which is what makes `getinfo(9E)` and the
//! `dev_t → instance` mapping in `cb_ops` work without driver code.

use core::cell::UnsafeCell;
use core::ffi::CStr;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicPtr, Ordering};

use crate::context::Sleepable;
use crate::cred::Cred;
use crate::device::{DevInfo, Instance, Minor, OpenFlags, OpenType};
use crate::error::{DdiError, Errno};
use crate::ioctl::{IoctlArg, IoctlCmd};
use crate::sys;
use crate::uio::Uio;

/// What a kernel module must provide.
pub trait Driver: Sized + 'static {
    /// Per-instance state. Constructed by [`attach`](Driver::attach), owned
    /// by the framework, dropped by [`detach`](Driver::detach).
    type Instance: Send + Sync + 'static;

    /// Text shown by `modinfo(8)`.
    const LINKINFO: &'static CStr;

    /// Initial size hint for the soft-state table; it grows on demand.
    const INSTANCE_HINT: usize = 1;

    /// `attach(9E)` with `DDI_ATTACH`.
    fn attach(dev: DevInfo, ctx: &Sleepable) -> Result<Self::Instance, DdiError>;

    /// `detach(9E)` with `DDI_DETACH`. Return the instance in `Err` to
    /// refuse; the framework keeps it attached.
    fn detach(inst: Self::Instance, _dev: DevInfo, _ctx: &Sleepable) -> Result<(), (Self::Instance, DdiError)> {
        drop(inst);
        Ok(())
    }

    /// `attach(9E)` with `DDI_RESUME`. Default: unsupported.
    fn resume(_inst: &Self::Instance, _dev: DevInfo, _ctx: &Sleepable) -> Result<(), DdiError> {
        Err(DdiError)
    }

    /// `detach(9E)` with `DDI_SUSPEND`. Default: unsupported.
    fn suspend(_inst: &Self::Instance, _dev: DevInfo, _ctx: &Sleepable) -> Result<(), DdiError> {
        Err(DdiError)
    }

    /// `quiesce(9E)`: stop the hardware for fast reboot, with no locks or
    /// blocking. Default: nothing to do (`ddi_quiesce_not_needed`).
    fn quiesce(_inst: &Self::Instance, _dev: DevInfo) -> Result<(), DdiError> {
        Ok(())
    }

    /// Map a minor number to the instance that owns it. Default: identity,
    /// the common single-node-per-instance layout.
    fn instance_of_minor(minor: Minor) -> Option<Instance> {
        i32::try_from(minor.0).ok().map(Instance)
    }
}

/// Character-device entry points, on the instance type.
pub trait CharDevice: Sized + Send + Sync + 'static {
    /// `open(9E)`.
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
    /// `close(9E)`.
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
    /// `read(9E)`. Returning `Ok(())` with nothing copied is EOF.
    fn read(&self, _minor: Minor, _uio: &mut Uio<'_>, _cred: &Cred<'_>, _ctx: &Sleepable) -> Result<(), Errno> {
        Err(Errno::ENXIO)
    }
    /// `write(9E)`.
    fn write(&self, _minor: Minor, _uio: &mut Uio<'_>, _cred: &Cred<'_>, _ctx: &Sleepable) -> Result<(), Errno> {
        Err(Errno::ENXIO)
    }
    /// `ioctl(9E)`. `Ok(v)` becomes the syscall's return value.
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
}

/// Interior-mutable static storage for structures the kernel writes to
/// (`dev_ops.devo_refcnt`, `modlinkage`).
#[repr(transparent)]
pub struct StaticCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for StaticCell<T> {}

impl<T> StaticCell<T> {
    /// Wrap a value.
    pub const fn new(v: T) -> StaticCell<T> {
        StaticCell(UnsafeCell::new(v))
    }
    /// Raw pointer to the contents.
    pub const fn get(&self) -> *mut T {
        self.0.get()
    }
}

/// One soft-state item per instance.
#[repr(C)]
pub struct Slot<I> {
    dip: sys::dev_info_t,
    live: bool,
    inst: MaybeUninit<I>,
}

/// Per-module framework state; one static per `module!` invocation.
pub struct ModuleState<D: Driver> {
    soft: AtomicPtr<sys::c_void>,
    _d: core::marker::PhantomData<fn() -> D>,
}

impl<D: Driver> ModuleState<D> {
    /// Empty state.
    pub const fn new() -> ModuleState<D> {
        ModuleState { soft: AtomicPtr::new(core::ptr::null_mut()), _d: core::marker::PhantomData }
    }

    fn soft(&self) -> *mut sys::c_void {
        self.soft.load(Ordering::Acquire)
    }

    /// Locate the live slot for `instance`.
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

/// Implemented by [`module!`] to give the glue access to the statics.
pub trait ModuleDef: Driver {
    /// The module's framework state.
    fn state() -> &'static ModuleState<Self>;
    /// The module's `modlinkage`.
    fn linkage() -> *mut sys::modlinkage;
}

/// C-ABI functions the macro points the kernel at. All are generic over
/// the driver and monomorphised per module.
///
/// Every function here is an entry point the kernel calls with arguments
/// whose validity the DDI guarantees (`Intro(9E)`); they are `unsafe` only
/// because a Rust caller could not uphold that. Driver code never calls them.
#[allow(clippy::missing_safety_doc)]
pub mod glue {
    use super::*;

    const fn align_ok<I>() {
        assert!(core::mem::align_of::<Slot<I>>() <= 8, "instance state must not need more than 8-byte alignment");
    }

    /// `_init(9E)` body.
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

    /// `_fini(9E)` body.
    pub unsafe fn fini<D: ModuleDef>() -> sys::c_int {
        let r = unsafe { sys::mod_remove(D::linkage()) };
        if r == 0 {
            let st = D::state();
            let mut soft = st.soft.swap(core::ptr::null_mut(), Ordering::AcqRel);
            unsafe { sys::ddi_soft_state_fini(&mut soft) };
        }
        r
    }

    /// `_info(9E)` body.
    pub unsafe fn info<D: ModuleDef>(modinfop: *mut sys::modinfo) -> sys::c_int {
        unsafe { sys::mod_info(D::linkage(), modinfop) }
    }

    /// `getinfo(9E)`.
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

    /// `attach(9E)`.
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
                match D::attach(dev, &ctx) {
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

    /// `detach(9E)`.
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

    /// `quiesce(9E)`.
    pub unsafe extern "C" fn quiesce<D: ModuleDef>(dip: sys::dev_info_t) -> sys::c_int {
        let dev = unsafe { DevInfo::from_raw(dip) };
        match unsafe { D::state().slot(dev.instance()) } {
            Some(s) => DdiError::into_c(D::quiesce(unsafe { s.inst.assume_init_ref() }, dev)),
            None => sys::DDI_SUCCESS,
        }
    }

    /// `open(9E)` for [`CharDevice`].
    pub unsafe extern "C" fn open<D: ModuleDef>(
        devp: *mut sys::dev_t,
        flag: sys::c_int,
        otyp: sys::c_int,
        credp: *mut sys::cred_t,
    ) -> sys::c_int
    where
        D::Instance: CharDevice,
    {
        let Some((minor, inst)) = (unsafe { D::state().instance_for_dev(*devp) }) else { return sys::errno::ENXIO };
        let cred = unsafe { Cred::from_raw(credp) };
        let ctx = unsafe { Sleepable::assume() };
        Errno::into_c(inst.open(minor, OpenFlags(flag), OpenType::from_raw(otyp), &cred, &ctx))
    }

    /// `close(9E)` for [`CharDevice`].
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

    /// `read(9E)` for [`CharDevice`].
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

    /// `write(9E)` for [`CharDevice`].
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

    /// `ioctl(9E)` for [`CharDevice`].
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
        match inst.ioctl(minor, IoctlCmd(cmd), IoctlArg::new(arg, mode), &cred, &ctx) {
            Ok(rv) => {
                if !rvalp.is_null() {
                    unsafe { *rvalp = rv };
                }
                0
            }
            Err(e) => e.get(),
        }
    }

    /// Correctly typed `nulldev` for `identify(9E)`/`probe(9E)`.
    pub unsafe extern "C" fn nulldev_dip(_dip: sys::dev_info_t) -> sys::c_int {
        0
    }
    /// Correctly typed `nodev` for `devo_reset`.
    pub unsafe extern "C" fn nodev_reset(_dip: sys::dev_info_t, _cmd: sys::ddi_reset_cmd_t) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// Correctly typed `nodev` for `power(9E)`.
    pub unsafe extern "C" fn nodev_power(_dip: sys::dev_info_t, _c: sys::c_int, _l: sys::c_int) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// Correctly typed `nodev` for `strategy(9E)`.
    pub unsafe extern "C" fn nodev_strategy(_bp: *mut sys::buf) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// Correctly typed `nodev` for `print(9E)`.
    pub unsafe extern "C" fn nodev_print(_dev: sys::dev_t, _s: *mut sys::c_char) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// Correctly typed `nodev` for `dump(9E)`.
    pub unsafe extern "C" fn nodev_dump(
        _dev: sys::dev_t,
        _a: sys::caddr_t,
        _b: sys::daddr_t,
        _n: sys::c_int,
    ) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// Correctly typed `nodev` for `devmap(9E)`.
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
    /// Correctly typed `nodev` for `mmap(9E)`.
    pub unsafe extern "C" fn nodev_mmap(_dev: sys::dev_t, _off: sys::off_t, _prot: sys::c_int) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// Correctly typed `nodev` for `segmap(9E)`.
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
    /// Correctly typed `nodev` for `open(9E)`.
    pub unsafe extern "C" fn nodev_open(
        _devp: *mut sys::dev_t,
        _f: sys::c_int,
        _o: sys::c_int,
        _c: *mut sys::cred_t,
    ) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// Correctly typed `nodev` for `close(9E)`.
    pub unsafe extern "C" fn nodev_close(
        _dev: sys::dev_t,
        _f: sys::c_int,
        _o: sys::c_int,
        _c: *mut sys::cred_t,
    ) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// Correctly typed `nodev` for `read(9E)`/`write(9E)`.
    pub unsafe extern "C" fn nodev_rw(_dev: sys::dev_t, _u: *mut sys::uio_t, _c: *mut sys::cred_t) -> sys::c_int {
        sys::errno::ENXIO
    }
    /// Correctly typed `nodev` for `ioctl(9E)`.
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
    /// Correctly typed `nodev` for `aread(9E)`/`awrite(9E)`.
    pub unsafe extern "C" fn nodev_aio(_dev: sys::dev_t, _a: *mut sys::aio_req, _c: *mut sys::cred_t) -> sys::c_int {
        sys::errno::ENXIO
    }
}

/// Define a kernel module for a [`Driver`].
///
/// ```ignore
/// ddi::module! {
///     driver: RNull,
///     ops: [char],          // or [] for a driver with no cb_ops
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
            ioctl: Some($crate::module::glue::ioctl::<$drv>));
    };
    (driver: $drv:ty, ops: [] $(,)?) => {
        $crate::module!(@common $drv);
        $crate::module!(@cb_ops $drv,
            open: Some($crate::module::glue::nodev_open),
            close: Some($crate::module::glue::nodev_close),
            read: Some($crate::module::glue::nodev_rw),
            write: Some($crate::module::glue::nodev_rw),
            ioctl: Some($crate::module::glue::nodev_ioctl));
    };
    (@cb_ops $drv:ty, open: $open:expr, close: $close:expr, read: $read:expr, write: $write:expr, ioctl: $ioctl:expr) => {
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
                cb_devmap: Some($crate::module::glue::nodev_devmap),
                cb_mmap: Some($crate::module::glue::nodev_mmap),
                cb_segmap: Some($crate::module::glue::nodev_segmap),
                cb_chpoll: Some($crate::sys::nochpoll),
                cb_prop_op: Some($crate::sys::ddi_prop_op),
                cb_str: ::core::ptr::null_mut(),
                cb_flag: $crate::sys::D_MP | $crate::sys::D_64BIT | $crate::sys::D_NEW,
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
