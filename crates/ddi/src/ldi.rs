//! The layered driver interface, for opening and using other devices from
//! inside the kernel.
//!
//! The LDI lets a driver open a device that another driver controls and send
//! it the same requests a user process would send through a file descriptor.
//! A consumer first obtains an [`LdiIdent`], which tells the framework who is
//! doing the opening, with `ldi_ident_from_dip(9F)` or
//! `ldi_ident_from_dev(9F)`. It then opens the target by path or by device
//! number and receives an [`LdiHandle`]. The handle forwards reads, writes
//! and ioctls to the target's entry points, answers questions about the
//! target such as its size and device number, and looks up the target's
//! properties.
//!
//! # Ownership
//!
//! Both values release what they hold when dropped. An [`LdiIdent`] calls
//! `ldi_ident_release(9F)`, and an [`LdiHandle`] calls `ldi_close(9F)` with
//! the flags it was opened with. A handle holds its own reference to the
//! identifier it was opened with, so the identifier may be dropped as soon
//! as the open has returned.
//!
//! The identifier from [`LdiIdent::from_dip`] records the device node it was
//! created for without holding it. The identifier and every handle opened
//! with it must therefore be dropped before that node detaches. Keeping them
//! in the driver's instance state, or in values the instance owns, satisfies
//! this, because the instance is destroyed at detach.
//!
//! # Context
//!
//! Every LDI function may only be called from user or kernel context, so
//! every method here that reaches the kernel takes a [`Sleepable`] token.
//! The manual adds a rule the token cannot express. The open and close
//! functions must not be called from `attach(9E)`, `detach(9E)` or
//! `power(9E)`, where they can deadlock or crash the system. A layered
//! driver normally opens its targets in its own `open(9E)` or in a task
//! queue and closes them in `close(9E)`. A handle that is still alive when
//! the instance is destroyed is closed during detach, which is exactly what
//! the manual warns against, so close handles explicitly before that point.
//! The same applies to dropping a handle in an interrupt handler, which the
//! context rule forbids.
//!
//! # Credentials
//!
//! The open functions and every request function take the credential to act
//! with. A driver passes the [`Cred`] it received in its own entry point, or
//! [`Cred::kernel`] for work it does on its own behalf, for example in a
//! task queue. The handle takes a reference on the open's credential with
//! `crhold` and closes with that same credential, as the manual asks, whether
//! it is closed explicitly with [`LdiHandle::close`] or simply dropped. The
//! reference is released with `crfree` after the close.
//!
//! # What is missing
//!
//! `ldi_open_by_devid(9F)` needs a `ddi_devid_t`, and this crate has no
//! owned device identifier type yet, so it is not wrapped.
//! `ldi_strategy(9F)` hands a `buf(9S)` to the target, which completes it
//! asynchronously with `biodone(9F)`. The buffer and its data must stay
//! alive and untouched until then, and nothing in this crate can express
//! that yet, so it is not wrapped either. [`LdiHandle::as_raw`] gives access
//! to both through the raw bindings.

use core::ffi::CStr;
use core::fmt;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::BitOr;
use core::ptr::{self, NonNull};

use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::context::Sleepable;
use crate::cred::Cred;
use crate::device::DevInfo;
use crate::error::{DdiError, Errno};
use crate::ioctl::IoctlCmd;
use crate::prop::{PropBuf, PropError, PropString, PropStrings};
use crate::sys;

/// An LDI identifier, which tells the framework who is performing a layered
/// open. It is released with `ldi_ident_release(9F)` when dropped, which
/// must happen in user or kernel context.
pub struct LdiIdent {
    raw: NonNull<sys::__ldi_ident>,
}

// The identifier is a reference-counted kernel object that the framework
// manipulates under its own lock. Opening with it, or releasing it, from any
// thread is allowed.
unsafe impl Send for LdiIdent {}
unsafe impl Sync for LdiIdent {}

impl LdiIdent {
    /// Creates an identifier for the device node `dip`, normally the
    /// driver's own, with `ldi_ident_from_dip(9F)`. The node must stay
    /// attached for as long as the identifier and the handles opened with it
    /// exist, as the [module documentation](self) explains.
    pub fn from_dip(dip: &DevInfo, _ctx: &Sleepable) -> Result<LdiIdent, Errno> {
        let mut raw: sys::ldi_ident_t = ptr::null_mut();
        Errno::check(unsafe { sys::ldi_ident_from_dip(dip.as_raw(), &mut raw) })?;
        NonNull::new(raw).map(|raw| LdiIdent { raw }).ok_or(Errno::EINVAL)
    }

    /// Creates an identifier for the device number `dev` with
    /// `ldi_ident_from_dev(9F)`. A driver uses this when it acts on behalf
    /// of one of its minor devices rather than on behalf of a node.
    pub fn from_dev(dev: sys::dev_t, _ctx: &Sleepable) -> Result<LdiIdent, Errno> {
        let mut raw: sys::ldi_ident_t = ptr::null_mut();
        Errno::check(unsafe { sys::ldi_ident_from_dev(dev, &mut raw) })?;
        NonNull::new(raw).map(|raw| LdiIdent { raw }).ok_or(Errno::EINVAL)
    }

    /// Returns the raw identifier, for `9F` functions that are not wrapped
    /// yet. It remains owned by this value.
    pub fn as_raw(&self) -> sys::ldi_ident_t {
        self.raw.as_ptr()
    }
}

impl Drop for LdiIdent {
    fn drop(&mut self) {
        unsafe { sys::ldi_ident_release(self.raw.as_ptr()) }
    }
}

impl fmt::Debug for LdiIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LdiIdent").field(&self.raw).finish()
    }
}

/// The flags of a layered open, which say how the target is opened. They are
/// kept in the handle and passed again to `ldi_close(9F)`, to the
/// `uio_fmode` of reads and writes, and to the `mode` of ioctls.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LdiFlags(i32);

impl LdiFlags {
    /// Open for reading (`FREAD`).
    pub const READ: LdiFlags = LdiFlags(sys::FREAD);
    /// Open for writing (`FWRITE`).
    pub const WRITE: LdiFlags = LdiFlags(sys::FWRITE);
    /// Open for reading and writing (`FREAD` and `FWRITE`).
    pub const READ_WRITE: LdiFlags = LdiFlags(sys::FREAD | sys::FWRITE);
    /// Ask for exclusive access, so that every other open of the target
    /// fails (`FEXCL`).
    pub const EXCL: LdiFlags = LdiFlags(sys::FEXCL);
    /// Return at once instead of blocking in the open, even if the target is
    /// not ready (`FNDELAY`).
    pub const NDELAY: LdiFlags = LdiFlags(sys::FNDELAY);

    /// Builds flags from raw bits, for flags such as `FNOCTTY` that have no
    /// constant here.
    pub const fn from_bits(bits: i32) -> LdiFlags {
        LdiFlags(bits)
    }

    /// Returns the raw bits.
    pub const fn bits(self) -> i32 {
        self.0
    }

    /// Reports whether every flag in `other` is set in `self`.
    pub const fn contains(self, other: LdiFlags) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for LdiFlags {
    type Output = LdiFlags;
    fn bitor(self, rhs: LdiFlags) -> LdiFlags {
        LdiFlags(self.0 | rhs.0)
    }
}

impl fmt::Debug for LdiFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LdiFlags({:#x})", self.0)
    }
}

/// Which interface of the target a handle uses. `ldi_open_by_dev(9F)`
/// accepts only these two; the other open types are not valid for a layered
/// open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LdiOpenType {
    /// The block interface (`OTYP_BLK`).
    Block,
    /// The character interface, which is also the one to use for a STREAMS
    /// device (`OTYP_CHR`).
    Char,
}

impl LdiOpenType {
    fn raw(self) -> i32 {
        match self {
            LdiOpenType::Block => sys::OTYP_BLK,
            LdiOpenType::Char => sys::OTYP_CHR,
        }
    }
}

/// The flags of the `ldi_prop_*(9F)` lookups, which decide where the search
/// for a property of the target goes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LdiPropFlags(sys::uint_t);

impl LdiPropFlags {
    /// No flags. The search covers the target's own properties, the PROM and
    /// then the parent nodes, and matches only properties created for the
    /// handle's device number.
    pub const NONE: LdiPropFlags = LdiPropFlags(0);
    /// Do not pass the search on to the parent nodes (`DDI_PROP_DONTPASS`).
    pub const DONT_PASS: LdiPropFlags = LdiPropFlags(sys::DDI_PROP_DONTPASS);
    /// Do not search the PROM (`DDI_PROP_NOTPROM`).
    pub const NOT_PROM: LdiPropFlags = LdiPropFlags(sys::DDI_PROP_NOTPROM);
    /// Match properties regardless of the device number they were created
    /// with (`LDI_DEV_T_ANY`). This is the only way to find properties
    /// created with `DDI_DEV_T_NONE`, which includes every PROM property.
    pub const DEV_T_ANY: LdiPropFlags = LdiPropFlags(sys::LDI_DEV_T_ANY);

    /// Returns the raw bits.
    pub const fn bits(self) -> sys::uint_t {
        self.0
    }
}

impl BitOr for LdiPropFlags {
    type Output = LdiPropFlags;
    fn bitor(self, rhs: LdiPropFlags) -> LdiPropFlags {
        LdiPropFlags(self.0 | rhs.0)
    }
}

impl fmt::Debug for LdiPropFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LdiPropFlags({:#x})", self.0)
    }
}

/// A layered handle on another device, from the `ldi_open_by_dev(9F)`
/// family.
///
/// The handle remembers the flags and the credential it was opened with and
/// uses them for the close. Dropping the handle closes it with
/// `ldi_close(9F)`; use [`LdiHandle::close`] to learn whether the target's
/// close failed.
/// The handle is released even when the close fails.
pub struct LdiHandle {
    raw: NonNull<sys::__ldi_handle>,
    flags: LdiFlags,
    /// The open's credential, with a reference taken by `crhold`.
    cred: NonNull<sys::cred_t>,
}

// A layered handle refers to the target's vnode and may be used from any
// thread, like a file descriptor shared by the threads of a process. Its own
// state does not change after the open, and the held credential is
// reference counted by the kernel. Concurrent requests reach the
// target's entry points concurrently, which every driver must already cope
// with for user processes.
unsafe impl Send for LdiHandle {}
unsafe impl Sync for LdiHandle {}

impl LdiHandle {
    /// Opens the device at the absolute path `path` with
    /// `ldi_open_by_name(9F)`. A `/devices` path works at any time; any other
    /// path to a device node works only once the root file system is
    /// mounted. Before the target is reached the call fails with `EINVAL`
    /// for a path that does not start with `/`, `ENODEV` for a path that does
    /// not exist, `EACCES` when `cred` may not search the path or open the
    /// node, or `ENXIO` for an unsupported access mode; after that the
    /// target's own `open(9E)` decides.
    ///
    /// Must not be called from `attach(9E)`, `detach(9E)` or `power(9E)`.
    pub fn open_by_name(
        ident: &LdiIdent,
        path: &CStr,
        flags: LdiFlags,
        cred: &Cred<'_>,
        _ctx: &Sleepable,
    ) -> Result<LdiHandle, Errno> {
        let mut raw: sys::ldi_handle_t = ptr::null_mut();
        Errno::check(unsafe {
            sys::ldi_open_by_name(path.as_ptr(), flags.0, cred.as_raw(), &mut raw, ident.as_raw())
        })?;
        LdiHandle::adopt(raw, flags, cred)
    }

    /// Opens the device with the number `dev` through the interface `otyp`
    /// with `ldi_open_by_dev(9F)`. A cloning device changes the device
    /// number during the open; [`LdiHandle::get_dev`] reports the number the
    /// handle ended up with. Before the target is reached the call fails
    /// with `EINVAL`, `ENODEV` for a device that does not exist, or `ENXIO`
    /// for an unsupported operation or access mode; after that the target's
    /// own `open(9E)` decides.
    ///
    /// Must not be called from `attach(9E)`, `detach(9E)` or `power(9E)`.
    pub fn open_by_dev(
        ident: &LdiIdent,
        dev: sys::dev_t,
        otyp: LdiOpenType,
        flags: LdiFlags,
        cred: &Cred<'_>,
        _ctx: &Sleepable,
    ) -> Result<LdiHandle, Errno> {
        let mut dev = dev;
        let mut raw: sys::ldi_handle_t = ptr::null_mut();
        Errno::check(unsafe {
            sys::ldi_open_by_dev(&mut dev, otyp.raw(), flags.0, cred.as_raw(), &mut raw, ident.as_raw())
        })?;
        LdiHandle::adopt(raw, flags, cred)
    }

    /// Wraps a freshly opened handle and takes a reference on the credential
    /// it was opened with.
    fn adopt(raw: sys::ldi_handle_t, flags: LdiFlags, cred: &Cred<'_>) -> Result<LdiHandle, Errno> {
        let raw = NonNull::new(raw).ok_or(Errno::EINVAL)?;
        let cred_raw = NonNull::new(cred.as_raw()).ok_or(Errno::EINVAL)?;
        // SAFETY: the credential is valid for the duration of this call, and
        // the reference taken here keeps it alive for the handle's life.
        unsafe { sys::crhold(cred_raw.as_ptr()) };
        Ok(LdiHandle { raw, flags, cred: cred_raw })
    }

    /// Returns the flags the handle was opened with.
    pub fn flags(&self) -> LdiFlags {
        self.flags
    }

    /// Returns the raw handle, for `9F` functions that are not wrapped yet.
    /// It remains owned by this value.
    pub fn as_raw(&self) -> sys::ldi_handle_t {
        self.raw.as_ptr()
    }

    /// Closes the handle with `ldi_close(9F)`, using the flags and the
    /// credential it was opened with, and reports the target's result.
    /// Dropping the handle does the same but discards the result.
    ///
    /// Must not be called from `attach(9E)`, `detach(9E)` or `power(9E)`.
    pub fn close(self, _ctx: &Sleepable) -> Result<(), Errno> {
        let this = ManuallyDrop::new(self);
        // SAFETY: the handle is open and closed exactly once, here, because
        // `ManuallyDrop` keeps `Drop` from running; the credential reference
        // taken at open is released after the close.
        let r = unsafe { sys::ldi_close(this.raw.as_ptr(), this.flags.0, this.cred.as_ptr()) };
        unsafe { sys::crfree(this.cred.as_ptr()) };
        Errno::check(r)
    }

    /// Builds a request over one kernel buffer, with the handle's flags as
    /// the file mode.
    fn uio(&self, iov: &mut sys::iovec, offset: u64) -> Result<sys::uio_t, Errno> {
        let offset = i64::try_from(offset).map_err(|_| Errno::EINVAL)?;
        Ok(sys::uio_t {
            uio_resid: iov.iov_len as isize,
            uio_iov: iov,
            uio_iovcnt: 1,
            uio_loffset: offset,
            uio_segflg: sys::uio_seg_t::UIO_SYSSPACE,
            uio_fmode: self.flags.0 as u16,
            uio_extflg: 0,
            uio_llimit: i64::MAX,
        })
    }

    /// Reads from the target at byte offset `offset` into `buf`, with
    /// `ldi_read(9F)`, and returns the number of bytes read. The request is
    /// a `uio(9S)` with one vector over `buf` and `UIO_SYSSPACE`, built on
    /// the stack for the duration of the call. The target's `read(9E)`
    /// fills the buffer through `uiomove(9F)`, which cannot write past its
    /// end. An offset above `i64::MAX` is refused with `EINVAL`. When the
    /// target reports an error, the number of bytes it may have transferred
    /// before that is not reported.
    pub fn read(&self, buf: &mut [u8], offset: u64, cred: &Cred<'_>, _ctx: &Sleepable) -> Result<usize, Errno> {
        let len = buf.len();
        let mut iov = sys::iovec { iov_base: buf.as_mut_ptr() as sys::caddr_t, iov_len: len };
        let mut uio = self.uio(&mut iov, offset)?;
        Errno::check(unsafe { sys::ldi_read(self.raw.as_ptr(), &mut uio, cred.as_raw()) })?;
        Ok(len.saturating_sub(uio.uio_resid.max(0) as usize))
    }

    /// Writes `buf` to the target at byte offset `offset`, with
    /// `ldi_write(9F)`, and returns the number of bytes written. The request
    /// is built the same way as for [`LdiHandle::read`], and the target's
    /// `write(9E)` reads the buffer through `uiomove(9F)`. An offset above
    /// `i64::MAX` is refused with `EINVAL`.
    pub fn write(&self, buf: &[u8], offset: u64, cred: &Cred<'_>, _ctx: &Sleepable) -> Result<usize, Errno> {
        let len = buf.len();
        // The vector is declared mutable because the structure is shared with
        // reads; a write(9E) only copies out of it.
        let mut iov = sys::iovec { iov_base: buf.as_ptr() as sys::caddr_t, iov_len: len };
        let mut uio = self.uio(&mut iov, offset)?;
        Errno::check(unsafe { sys::ldi_write(self.raw.as_ptr(), &mut uio, cred.as_raw()) })?;
        Ok(len.saturating_sub(uio.uio_resid.max(0) as usize))
    }

    /// Sends the ioctl `cmd` with the raw argument `arg` to the target, with
    /// `ldi_ioctl(9F)`, and returns the value the target stored through
    /// `rvalp`. The `mode` is the handle's flags with `FKIOCTL` added, which
    /// tells the target that a pointer argument is a kernel address and needs
    /// no data model conversion.
    ///
    /// # Safety
    /// If the target interprets `arg` as a pointer for `cmd`, it must point
    /// to kernel memory that is valid for every access the target makes for
    /// that command, for the duration of the call. The target copies with
    /// `ddi_copyin(9F)` and `ddi_copyout(9F)`, which under `FKIOCTL` are
    /// plain memory copies of whatever length the target chooses.
    pub unsafe fn ioctl_raw(&self, cmd: IoctlCmd, arg: isize, cred: &Cred<'_>, _ctx: &Sleepable) -> Result<i32, Errno> {
        let mut rval: i32 = 0;
        let mode = self.flags.0 | sys::FKIOCTL;
        Errno::check(unsafe { sys::ldi_ioctl(self.raw.as_ptr(), cmd.0, arg, mode, cred.as_raw(), &mut rval) })?;
        Ok(rval)
    }

    /// Sends the ioctl `cmd` with a pointer to a copy of `arg`, for a command
    /// whose argument the target only reads, and returns the target's return
    /// value. The copy lives on the stack for the duration of the call, so a
    /// target that writes to its argument after all cannot change `arg`.
    ///
    /// # Safety
    /// The target must access at most `size_of::<T>()` bytes at the argument
    /// for `cmd`, and must not keep the pointer after it returns. This is the
    /// pairing of command and argument type that the C interface leaves to
    /// the caller; commands built with [`IoctlCmd::iow`] encode the size, but
    /// many illumos commands do not, and the target need not check it.
    pub unsafe fn ioctl_in<T: IntoBytes + Immutable>(
        &self,
        cmd: IoctlCmd,
        arg: &T,
        cred: &Cred<'_>,
        ctx: &Sleepable,
    ) -> Result<i32, Errno> {
        let mut copy = MaybeUninit::<T>::uninit();
        // A bitwise copy of an IntoBytes value is only ever used as bytes
        // here; it is never dropped or read back as a T.
        unsafe { ptr::copy_nonoverlapping(arg as *const T, copy.as_mut_ptr(), 1) };
        unsafe { self.ioctl_raw(cmd, copy.as_mut_ptr() as isize, cred, ctx) }
    }

    /// Sends the ioctl `cmd` with a pointer to a zeroed `T`, for a command
    /// whose argument the target fills in, and returns the filled-in value
    /// together with the target's return value. The `FromBytes` bound
    /// guarantees that whatever bytes the target writes form a valid `T`.
    ///
    /// # Safety
    /// The same as for [`LdiHandle::ioctl_in`].
    pub unsafe fn ioctl_out<T: FromBytes>(
        &self,
        cmd: IoctlCmd,
        cred: &Cred<'_>,
        ctx: &Sleepable,
    ) -> Result<(T, i32), Errno> {
        let mut v = T::new_zeroed();
        let rval = unsafe { self.ioctl_raw(cmd, &mut v as *mut T as isize, cred, ctx)? };
        Ok((v, rval))
    }

    /// Sends the ioctl `cmd` with a pointer to `arg`, for a command whose
    /// argument the target both reads and writes, and returns the target's
    /// return value. The `FromBytes` bound guarantees that whatever bytes the
    /// target writes leave `arg` a valid `T`.
    ///
    /// # Safety
    /// The same as for [`LdiHandle::ioctl_in`].
    pub unsafe fn ioctl_inout<T: FromBytes + IntoBytes>(
        &self,
        cmd: IoctlCmd,
        arg: &mut T,
        cred: &Cred<'_>,
        ctx: &Sleepable,
    ) -> Result<i32, Errno> {
        unsafe { self.ioctl_raw(cmd, arg as *mut T as isize, cred, ctx) }
    }

    /// Returns the size of the target in bytes, with `ldi_get_size(9F)`. The
    /// size comes from the target's `Nblocks`, `nblocks`, `Size` or `size`
    /// property. The function uses the `DDI_SUCCESS` and `DDI_FAILURE`
    /// convention, so a target that publishes none of them, or a STREAMS
    /// device, gives a [`DdiError`] without a reason.
    pub fn get_size(&self, _ctx: &Sleepable) -> Result<u64, DdiError> {
        let mut size: u64 = 0;
        DdiError::check(unsafe { sys::ldi_get_size(self.raw.as_ptr(), &mut size) })?;
        Ok(size)
    }

    /// Returns the device number of the target, with `ldi_get_dev(9F)`.
    /// After an open of a cloning device this is the number of the clone.
    pub fn get_dev(&self, _ctx: &Sleepable) -> Result<sys::dev_t, Errno> {
        let mut dev: sys::dev_t = 0;
        Errno::check(unsafe { sys::ldi_get_dev(self.raw.as_ptr(), &mut dev) })?;
        Ok(dev)
    }

    /// Returns the interface the target was opened through, with
    /// `ldi_get_otyp(9F)`. An open by name uses the character interface for
    /// a character special file and the block interface for a block special
    /// file. A value
    /// other than `OTYP_BLK` and `OTYP_CHR`, which the manual rules out, is
    /// reported as `EINVAL`.
    pub fn get_otyp(&self, _ctx: &Sleepable) -> Result<LdiOpenType, Errno> {
        let mut otyp: i32 = 0;
        Errno::check(unsafe { sys::ldi_get_otyp(self.raw.as_ptr(), &mut otyp) })?;
        match otyp {
            sys::OTYP_BLK => Ok(LdiOpenType::Block),
            sys::OTYP_CHR => Ok(LdiOpenType::Char),
            _ => Err(Errno::EINVAL),
        }
    }

    /// Reports whether the target has a property called `name`, of any
    /// type, with `ldi_prop_exists(9F)`. `flags` decides where the search
    /// goes.
    pub fn prop_exists(&self, name: &CStr, flags: LdiPropFlags, _ctx: &Sleepable) -> bool {
        unsafe { sys::ldi_prop_exists(self.raw.as_ptr(), flags.0, name.as_ptr() as *mut sys::c_char) != 0 }
    }

    /// Reads an integer property of the target with `ldi_prop_get_int(9F)`,
    /// returning `default` when it does not exist. When the property exists
    /// but cannot be decoded as an integer, the function returns the value
    /// of `DDI_PROP_NOT_FOUND`, which is 1, rather than `default`; use
    /// [`LdiHandle::prop_lookup_int_array`] to tell these cases apart.
    pub fn prop_get_int(&self, name: &CStr, default: i32, flags: LdiPropFlags, _ctx: &Sleepable) -> i32 {
        unsafe { sys::ldi_prop_get_int(self.raw.as_ptr(), flags.0, name.as_ptr() as *mut sys::c_char, default) }
    }

    /// Reads a 64-bit integer property of the target with
    /// `ldi_prop_get_int64(9F)`, returning `default` when it does not exist.
    /// The PROM is never searched for 64-bit values. The decoding quirk of
    /// [`LdiHandle::prop_get_int`] applies here too.
    pub fn prop_get_int64(&self, name: &CStr, default: i64, flags: LdiPropFlags, _ctx: &Sleepable) -> i64 {
        unsafe { sys::ldi_prop_get_int64(self.raw.as_ptr(), flags.0, name.as_ptr() as *mut sys::c_char, default) }
    }

    /// Reads an integer array property of the target with
    /// `ldi_prop_lookup_int_array(9F)`. Returns `Ok(None)` when the property
    /// does not exist or was explicitly undefined.
    pub fn prop_lookup_int_array(
        &self,
        name: &CStr,
        flags: LdiPropFlags,
        _ctx: &Sleepable,
    ) -> Result<Option<PropBuf<i32>>, PropError> {
        let mut data: *mut i32 = ptr::null_mut();
        let mut n: sys::uint_t = 0;
        let r = unsafe {
            sys::ldi_prop_lookup_int_array(
                self.raw.as_ptr(),
                flags.0,
                name.as_ptr() as *mut sys::c_char,
                &mut data,
                &mut n,
            )
        };
        Ok(PropError::check(r)?.then(|| unsafe { PropBuf::from_raw(data, n as usize) }))
    }

    /// Reads a 64-bit integer array property of the target with
    /// `ldi_prop_lookup_int64_array(9F)`. The PROM is never searched.
    /// Returns `Ok(None)` when the property does not exist or was explicitly
    /// undefined.
    pub fn prop_lookup_int64_array(
        &self,
        name: &CStr,
        flags: LdiPropFlags,
        _ctx: &Sleepable,
    ) -> Result<Option<PropBuf<i64>>, PropError> {
        let mut data: *mut i64 = ptr::null_mut();
        let mut n: sys::uint_t = 0;
        let r = unsafe {
            sys::ldi_prop_lookup_int64_array(
                self.raw.as_ptr(),
                flags.0,
                name.as_ptr() as *mut sys::c_char,
                &mut data,
                &mut n,
            )
        };
        Ok(PropError::check(r)?.then(|| unsafe { PropBuf::from_raw(data, n as usize) }))
    }

    /// Reads a byte array property of the target with
    /// `ldi_prop_lookup_byte_array(9F)`. Returns `Ok(None)` when the property
    /// does not exist or was explicitly undefined.
    pub fn prop_lookup_bytes(
        &self,
        name: &CStr,
        flags: LdiPropFlags,
        _ctx: &Sleepable,
    ) -> Result<Option<PropBuf<u8>>, PropError> {
        let mut data: *mut u8 = ptr::null_mut();
        let mut n: sys::uint_t = 0;
        let r = unsafe {
            sys::ldi_prop_lookup_byte_array(
                self.raw.as_ptr(),
                flags.0,
                name.as_ptr() as *mut sys::c_char,
                &mut data,
                &mut n,
            )
        };
        Ok(PropError::check(r)?.then(|| unsafe { PropBuf::from_raw(data, n as usize) }))
    }

    /// Reads a string property of the target with
    /// `ldi_prop_lookup_string(9F)`. Returns `Ok(None)` when the property
    /// does not exist or was explicitly undefined.
    pub fn prop_lookup_string(
        &self,
        name: &CStr,
        flags: LdiPropFlags,
        _ctx: &Sleepable,
    ) -> Result<Option<PropString>, PropError> {
        let mut data: *mut sys::c_char = ptr::null_mut();
        let r = unsafe {
            sys::ldi_prop_lookup_string(self.raw.as_ptr(), flags.0, name.as_ptr() as *mut sys::c_char, &mut data)
        };
        Ok(PropError::check(r)?
            .then(|| NonNull::new(data).map(|data| unsafe { PropString::from_raw(data.as_ptr()) }))
            .flatten())
    }

    /// Reads a string array property of the target with
    /// `ldi_prop_lookup_string_array(9F)`. Returns `Ok(None)` when the
    /// property does not exist or was explicitly undefined.
    pub fn prop_lookup_strings(
        &self,
        name: &CStr,
        flags: LdiPropFlags,
        _ctx: &Sleepable,
    ) -> Result<Option<PropStrings>, PropError> {
        let mut data: *mut *mut sys::c_char = ptr::null_mut();
        let mut n: sys::uint_t = 0;
        let r = unsafe {
            sys::ldi_prop_lookup_string_array(
                self.raw.as_ptr(),
                flags.0,
                name.as_ptr() as *mut sys::c_char,
                &mut data,
                &mut n,
            )
        };
        Ok(PropError::check(r)?.then(|| unsafe { PropStrings::from_raw(data, n as usize) }))
    }
}

impl Drop for LdiHandle {
    fn drop(&mut self) {
        // The handle is released even if the target's close fails, and
        // there is nobody to report the error to. The credential is the one
        // held since the open.
        unsafe {
            let _ = sys::ldi_close(self.raw.as_ptr(), self.flags.0, self.cred.as_ptr());
            sys::crfree(self.cred.as_ptr());
        }
    }
}

impl fmt::Debug for LdiHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LdiHandle").field("raw", &self.raw).field("flags", &self.flags).finish()
    }
}
