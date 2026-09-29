//! File descriptors for a driver's own minor numbers, created from an
//! ioctl.
//!
//! Some kernel objects are handed to user space as file descriptors that an
//! ioctl returns: a DRM driver's dma-bufs, sync objects and sync files. The
//! descriptor must behave like any other: `close(2)` drops it once, it
//! survives `fork(2)` and `SCM_RIGHTS`, and `poll(2)` and `mmap(2)` work on
//! it. This module gives such a descriptor a specfs vnode for a device
//! number of the calling driver, so that every later operation on the file
//! goes to the driver's own `cb_ops` entry points with that minor number:
//! `close(9E)` on the last close, `chpoll(9E)`, `devmap(9E)` and
//! `ioctl(9E)`. The steps are the ones of `pipe(2)`
//! (`uts/common/syscall/pipe.c`) and of the layered driver interface's
//! `ldi_vp_from_dev` (`uts/common/os/driver_lyr.c`): `makespecvp`,
//! `VOP_OPEN` (which calls the driver's `open(9E)` for the minor),
//! `falloc`, `setf`, and `f_setfd_or` for close-on-exec. `docs/anon-fd.md`
//! in illumos-drm records why this design was chosen.
//!
//! The rules and their guards:
//!
//! - Descriptors belong to the calling process, so they can only be
//!   created and looked up in the system call of a user process. Both
//!   functions take the [`IoctlArg`] of the ioctl being served, refuse
//!   callers through LDI (`FKIOCTL`) with `EINVAL`, and refuse kernel
//!   threads (`ddi_get_pid(9F)` 0, the process `sched`), which a forged
//!   `IoctlArg` could not hide.
//! - Only minor numbers of the calling driver can be opened: the device
//!   number is built from `ddi_driver_major(9F)` of the driver's own node.
//! - The new vnode gets the path of the caller's own file, the one the
//!   ioctl was issued on, found with `specfind` for the device number the
//!   glue records in the [`IoctlArg`]. `makespecvp` gives a vnode no path,
//!   and the devmap framework's `segdev_create` cannot map a vnode whose
//!   path is not visible in the caller's zone: it dereferences NULL (the
//!   panic of 29 September 2026, `docs/DESIGN.md` section 5.12). An
//!   argument without a device number (made with [`IoctlArg::new`]) or a
//!   caller whose file has no visible vnode is refused with `EINVAL`, and
//!   the `devmap(9E)` glue refuses to map a device that `specfind` cannot
//!   find ([`mappable`]), which also covers a descriptor passed into
//!   another zone.
//! - [`NewFd::open`] calls the driver's own `open(9E)` for the minor, and
//!   an abandoned [`NewFd`] calls its `close(9E)`, so the caller must not
//!   hold a lock those entry points take.
//! - A new descriptor is reserved but invisible to the process until
//!   [`NewFd::install`]; dropping the [`NewFd`] instead undoes everything,
//!   so a caller can copy the number out to user space first and give up
//!   if that fails, as `pipe(2)` does.
//! - [`FileRef`] holds the descriptor (a concurrent `close(2)` waits) until
//!   it is dropped, on the same thread, and is therefore neither `Send`
//!   nor `Sync`.

use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use illumos_kernel_private_sys::file::{self as pfile, FD_CLOEXEC, file};
use illumos_kernel_private_sys::specfs::{makespecvp, specfind};
use illumos_kernel_private_sys::vnode::{VCHR, fop_close, fop_open, vn_copypath, vn_rele, vnode};

use crate::context::Sleepable;
use crate::device::{DevInfo, Minor};
use crate::error::Errno;
use crate::ioctl::IoctlArg;
use crate::sys;

/// How a new file is opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FdFlags {
    /// Open for writing as well as reading (`O_RDWR`); read-only otherwise.
    pub write: bool,
    /// Set close-on-exec on the descriptor (`O_CLOEXEC`).
    pub cloexec: bool,
}

/// Refuses callers that are not a user process in a system call.
fn user_call(arg: &IoctlArg<'_>) -> Result<(), Errno> {
    if arg.is_kernel_caller() {
        return Err(Errno::EINVAL);
    }
    // SAFETY: ddi_get_pid(9F) may be called from any context.
    if unsafe { sys::ddi_get_pid() } <= 0 {
        return Err(Errno::EINVAL);
    }
    Ok(())
}

/// A file descriptor of the calling process for a minor number of the
/// calling driver, opened but not yet visible to the process. See the
/// [module documentation](self).
#[must_use = "a NewFd is closed again unless it is installed"]
pub struct NewFd<'a> {
    fd: sys::c_int,
    fp: NonNull<file>,
    vp: NonNull<vnode>,
    flag: sys::c_int,
    cloexec: bool,
    // Bound to the ioctl being served, and to its thread.
    _call: PhantomData<(&'a (), *mut ())>,
}

impl<'a> NewFd<'a> {
    /// Opens minor `minor` of the driver of `dev` (the calling driver's own
    /// node) as a new file with `flags`, and reserves a descriptor for it
    /// in the calling process. The driver's `open(9E)` runs for the minor
    /// with `FREAD` (and `FWRITE`) and `OTYP_CHR`; its error is returned.
    /// `EMFILE` if the process has no free descriptor, `EINVAL` if the
    /// caller is not a user process in the ioctl `arg` belongs to.
    pub fn open(
        dev: &DevInfo,
        minor: Minor,
        flags: FdFlags,
        arg: &'a IoctlArg<'_>,
        _ctx: &Sleepable,
    ) -> Result<NewFd<'a>, Errno> {
        user_call(arg)?;
        let caller = arg.dev().ok_or(Errno::EINVAL)?;
        let flag = sys::FREAD | if flags.write { sys::FWRITE } else { 0 };
        // SAFETY: the node is live (the caller got it from its own entry
        // point); the major number is the calling driver's.
        let devt = unsafe { sys::makedevice(sys::ddi_driver_major(dev.as_raw()), minor.0) };
        // The caller's own file: a held vnode with a path visible in the
        // caller's zone, or none.
        // SAFETY: specfind may be called from user context and returns a
        // held vnode or NULL.
        let src = unsafe { specfind(caller, VCHR) };
        if src.is_null() {
            return Err(Errno::EINVAL);
        }
        // SAFETY: makespecvp allocates with KM_SLEEP, which the token allows,
        // and returns a vnode with one reference. vn_copypath copies the
        // path of the held `src`, which is released afterwards.
        let mut vp = unsafe {
            let vp = makespecvp(devt, VCHR);
            vn_copypath(src, vp);
            vn_rele(src);
            vp
        };
        // SAFETY: vp is a fresh specfs vnode; the credential is the calling
        // thread's, the one falloc records as well. On success vp may have
        // been replaced (a cloning open), and the reference moved with it.
        let r = unsafe { fop_open(&mut vp, flag, sys::ddi_get_cred(), ptr::null_mut()) };
        let Some(vpn) = NonNull::new(vp) else { return Err(Errno::ENXIO) };
        if r != 0 {
            // SAFETY: the reference makespecvp returned.
            unsafe { vn_rele(vp) };
            return Err(Errno::new(r).unwrap_or(Errno::EIO));
        }
        let mut fp: *mut file = ptr::null_mut();
        let mut fd: sys::c_int = -1;
        // SAFETY: user context of the calling process (checked above);
        // falloc takes over the vnode reference on success.
        let r = unsafe { pfile::falloc(vp, flag, &mut fp, &mut fd) };
        let Some(fpn) = NonNull::new(fp).filter(|_| r == 0) else {
            // SAFETY: undo the open and drop the reference.
            unsafe {
                fop_close(vp, flag, 1, 0, sys::ddi_get_cred(), ptr::null_mut());
                vn_rele(vp);
            }
            return Err(Errno::new(r).unwrap_or(Errno::EMFILE));
        };
        Ok(NewFd { fd, fp: fpn, vp: vpn, flag, cloexec: flags.cloexec, _call: PhantomData })
    }

    /// Returns the descriptor number, to copy out to the caller before
    /// [`NewFd::install`].
    pub fn fd(&self) -> i32 {
        self.fd
    }

    /// Makes the descriptor visible to the process (`setf`), with
    /// close-on-exec if it was asked for. From here on the process owns the
    /// file; its last close calls the driver's `close(9E)` for the minor.
    pub fn install(self) {
        let me = core::mem::ManuallyDrop::new(self);
        // SAFETY: falloc returned the file locked and the descriptor
        // reserved in this process; the lock is released before setf, as in
        // pipe(2). f_setfd_or only sets flags on the installed descriptor.
        unsafe {
            sys::mutex_exit(&raw mut (*me.fp.as_ptr()).f_tlock);
            pfile::setf(me.fd, me.fp.as_ptr());
            if me.cloexec {
                pfile::f_setfd_or(me.fd, FD_CLOEXEC);
            }
        }
    }
}

impl Drop for NewFd<'_> {
    /// Gives up the descriptor: closes the vnode (the driver's `close(9E)`
    /// runs if this was the last open of the minor), releases the
    /// descriptor, frees the file and drops the vnode reference, the undo
    /// path of `pipe(2)`.
    fn drop(&mut self) {
        // SAFETY: the file is still locked and private to us, the descriptor
        // reserved, the vnode open once and referenced once by the file.
        unsafe {
            let cred = (*self.fp.as_ptr()).f_cred;
            fop_close(self.vp.as_ptr(), self.flag, 1, 0, cred, ptr::null_mut());
            pfile::setf(self.fd, ptr::null_mut());
            // unfalloc drops the credential and frees the file, releasing the
            // lock falloc returned it with.
            pfile::unfalloc(self.fp.as_ptr());
            vn_rele(self.vp.as_ptr());
        }
    }
}

/// Reports whether the devmap framework can map device number `dev` for
/// the calling process: whether `specfind` finds a vnode for it whose path
/// is visible in the caller's zone. The `devmap(9E)` glue refuses the
/// mapping otherwise (see the module documentation).
pub(crate) fn mappable(dev: sys::dev_t) -> bool {
    // SAFETY: specfind may be called from user context (devmap runs in
    // mmap(2)); a vnode it returns is held and released at once.
    unsafe {
        let vp = specfind(dev, VCHR);
        if vp.is_null() {
            return false;
        }
        vn_rele(vp);
    }
    true
}

/// A descriptor of the calling process held for the duration of an ioctl
/// (`getf`/`releasef`). See the [module documentation](self).
pub struct FileRef<'a> {
    fd: sys::c_int,
    fp: NonNull<file>,
    _call: PhantomData<(&'a (), *mut ())>,
}

impl<'a> FileRef<'a> {
    /// Looks up descriptor `fd` of the calling process and holds it.
    /// `EBADF` if it is not open, `EINVAL` if the caller is not a user
    /// process in the ioctl `arg` belongs to.
    pub fn get(fd: i32, arg: &'a IoctlArg<'_>) -> Result<FileRef<'a>, Errno> {
        user_call(arg)?;
        // SAFETY: user context of the calling process; any fd value is
        // accepted and range-checked by getf.
        let fp = unsafe { pfile::getf(fd) };
        let fp = NonNull::new(fp).ok_or(Errno::EBADF)?;
        Ok(FileRef { fd, fp, _call: PhantomData })
    }

    /// Returns the minor number if the file is a character device of the
    /// driver of `dev` (the calling driver's own node), `None` for any
    /// other file. The driver's own tables decide what the minor means.
    pub fn own_minor(&self, dev: &DevInfo) -> Option<Minor> {
        // SAFETY: the held file keeps its vnode; v_type and v_rdev of an
        // open file never change.
        let (vtype, rdev) = unsafe {
            let vp = (*self.fp.as_ptr()).f_vnode;
            if vp.is_null() {
                return None;
            }
            ((*vp).v_type, (*vp).v_rdev)
        };
        if vtype != VCHR {
            return None;
        }
        // SAFETY: plain arithmetic on device numbers; the node is live.
        let (maj, own) = unsafe { (sys::getmajor(rdev), sys::ddi_driver_major(dev.as_raw())) };
        // SAFETY: as above.
        (maj == own).then(|| Minor(unsafe { sys::getminor(rdev) }))
    }

    /// Reports whether the file is open for writing.
    pub fn is_writable(&self) -> bool {
        // SAFETY: the held file; f_flag's access bits never change after
        // open.
        (i32::from(unsafe { (*self.fp.as_ptr()).f_flag }) & sys::FWRITE) != 0
    }
}

impl Drop for FileRef<'_> {
    fn drop(&mut self) {
        // SAFETY: getf succeeded for this descriptor on this thread.
        unsafe { pfile::releasef(self.fd) }
    }
}
