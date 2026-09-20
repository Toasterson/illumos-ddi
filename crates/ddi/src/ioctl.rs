//! `ioctl(9E)` commands and arguments.

use core::marker::PhantomData;
use core::mem::{size_of, MaybeUninit};

use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::error::Errno;
use crate::sys;

/// An ioctl command number, built with the `_IO*` encodings from
/// `sys/ioccom.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IoctlCmd(pub i32);

impl IoctlCmd {
    /// `_IO(group, num)`: no argument.
    pub const fn io(group: u8, num: u8) -> IoctlCmd {
        IoctlCmd(sys::_IO(group, num))
    }
    /// `_IOR(group, num, T)`: kernel writes a `T` to the user.
    pub const fn ior<T>(group: u8, num: u8) -> IoctlCmd {
        IoctlCmd(sys::_IOR(group, num, size_of::<T>()))
    }
    /// `_IOW(group, num, T)`: kernel reads a `T` from the user.
    pub const fn iow<T>(group: u8, num: u8) -> IoctlCmd {
        IoctlCmd(sys::_IOW(group, num, size_of::<T>()))
    }
    /// `_IOWR(group, num, T)`.
    pub const fn iowr<T>(group: u8, num: u8) -> IoctlCmd {
        IoctlCmd(sys::_IOWR(group, num, size_of::<T>()))
    }
}

/// Data model of the ioctl caller (`ddi_model_convert_from(9F)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataModel {
    /// 32-bit caller: `ILP32` structures.
    Ilp32,
    /// 64-bit caller.
    Lp64,
}

/// The `arg`/`mode` pair of `ioctl(9E)`. All user-memory access goes
/// through `ddi_copyin(9F)`/`ddi_copyout(9F)` with the original `mode`, so
/// `FKIOCTL` (kernel callers via LDI) is honoured automatically.
pub struct IoctlArg<'a> {
    arg: isize,
    mode: i32,
    _life: PhantomData<&'a ()>,
}

impl<'a> IoctlArg<'a> {
    /// Wrap the raw entry-point arguments.
    pub fn new(arg: isize, mode: i32) -> IoctlArg<'a> {
        IoctlArg { arg, mode, _life: PhantomData }
    }

    /// The raw `arg` value, for commands that pass an integer.
    pub fn raw(&self) -> isize {
        self.arg
    }

    /// The raw `mode` (`FREAD | FWRITE | FKIOCTL | model bits`).
    pub fn mode(&self) -> i32 {
        self.mode
    }

    /// Whether the caller is in the kernel (`FKIOCTL`).
    pub fn is_kernel_caller(&self) -> bool {
        self.mode & sys::FKIOCTL != 0
    }

    /// Caller's data model.
    pub fn model(&self) -> DataModel {
        let m = unsafe { sys::ddi_model_convert_from(sys::model_t(self.mode & sys::DDI_MODEL_MASK)) };
        if m == sys::model_t::DDI_MODEL_ILP32 { DataModel::Ilp32 } else { DataModel::Lp64 }
    }

    /// Copy a `T` in from the caller. `FromBytes` guarantees every byte
    /// pattern is a valid `T`, so no validation is skipped.
    pub fn copy_in<T: FromBytes>(&self) -> Result<T, Errno> {
        let mut v = MaybeUninit::<T>::uninit();
        let r = unsafe {
            sys::ddi_copyin(self.arg as *const sys::c_void, v.as_mut_ptr() as *mut sys::c_void, size_of::<T>(), self.mode)
        };
        if r != 0 {
            return Err(Errno::EFAULT);
        }
        Ok(unsafe { v.assume_init() })
    }

    /// Copy a `T` out to the caller.
    pub fn copy_out<T: IntoBytes + Immutable>(&self, v: &T) -> Result<(), Errno> {
        let bytes = v.as_bytes();
        let r = unsafe {
            sys::ddi_copyout(bytes.as_ptr() as *const sys::c_void, self.arg as *mut sys::c_void, bytes.len(), self.mode)
        };
        if r != 0 { Err(Errno::EFAULT) } else { Ok(()) }
    }
}
