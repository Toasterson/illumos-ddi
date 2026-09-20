//! Commands and arguments of `ioctl(9E)`.

use core::marker::PhantomData;
use core::mem::{MaybeUninit, size_of};

use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::error::Errno;
use crate::sys;

/// An ioctl command number.
///
/// Drivers build their commands with the associated functions, which are
/// the `const fn` equivalents of the `_IO`, `_IOR`, `_IOW` and `_IOWR`
/// macros from `sys/ioccom.h`, and match on them in `ioctl(9E)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IoctlCmd(pub i32);

impl IoctlCmd {
    /// Encodes a command without an argument, like `_IO(group, num)`.
    pub const fn io(group: u8, num: u8) -> IoctlCmd {
        IoctlCmd(sys::_IO(group, num))
    }
    /// Encodes a command whose `T` the kernel writes to the caller, like
    /// `_IOR(group, num, T)`.
    pub const fn ior<T>(group: u8, num: u8) -> IoctlCmd {
        IoctlCmd(sys::_IOR(group, num, size_of::<T>()))
    }
    /// Encodes a command whose `T` the kernel reads from the caller, like
    /// `_IOW(group, num, T)`.
    pub const fn iow<T>(group: u8, num: u8) -> IoctlCmd {
        IoctlCmd(sys::_IOW(group, num, size_of::<T>()))
    }
    /// Encodes a command whose `T` travels in both directions, like
    /// `_IOWR(group, num, T)`.
    pub const fn iowr<T>(group: u8, num: u8) -> IoctlCmd {
        IoctlCmd(sys::_IOWR(group, num, size_of::<T>()))
    }
}

/// The data model of the process that issued the ioctl, as reported by
/// `ddi_model_convert_from(9F)`. It matters when an argument structure
/// contains pointers or `long` fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataModel {
    /// The caller is a 32-bit process and uses ILP32 structure layouts.
    Ilp32,
    /// The caller is a 64-bit process, the same as the kernel.
    Lp64,
}

/// The `arg` and `mode` arguments of `ioctl(9E)`, kept together.
///
/// All access to the argument goes through `ddi_copyin(9F)` and
/// `ddi_copyout(9F)` with the original `mode`. That is what makes kernel
/// callers work: when another module calls through LDI, `mode` carries
/// `FKIOCTL` and the copy functions know that `arg` is already a kernel
/// address.
pub struct IoctlArg<'a> {
    arg: isize,
    mode: i32,
    _life: PhantomData<&'a ()>,
}

impl<'a> IoctlArg<'a> {
    /// Wraps the raw `arg` and `mode` values from the entry point.
    pub fn new(arg: isize, mode: i32) -> IoctlArg<'a> {
        IoctlArg { arg, mode, _life: PhantomData }
    }

    /// Returns the raw `arg` value, for commands that pass an integer rather
    /// than a pointer.
    pub fn raw(&self) -> isize {
        self.arg
    }

    /// Returns the raw `mode`, which combines the open flags, `FKIOCTL` and
    /// the data model bits.
    pub fn mode(&self) -> i32 {
        self.mode
    }

    /// Reports whether the caller is another kernel module rather than a
    /// user process, which is signalled by `FKIOCTL` in `mode`.
    pub fn is_kernel_caller(&self) -> bool {
        self.mode & sys::FKIOCTL != 0
    }

    /// Returns the caller's data model.
    pub fn model(&self) -> DataModel {
        let m = unsafe { sys::ddi_model_convert_from(sys::model_t(self.mode & sys::DDI_MODEL_MASK)) };
        if m == sys::model_t::DDI_MODEL_ILP32 { DataModel::Ilp32 } else { DataModel::Lp64 }
    }

    /// Copies a `T` in from the caller. The `FromBytes` bound guarantees that
    /// every byte pattern is a valid `T`, so reading user memory straight
    /// into the type skips no validation.
    pub fn copy_in<T: FromBytes>(&self) -> Result<T, Errno> {
        let mut v = MaybeUninit::<T>::uninit();
        let r = unsafe {
            sys::ddi_copyin(
                self.arg as *const sys::c_void,
                v.as_mut_ptr() as *mut sys::c_void,
                size_of::<T>(),
                self.mode,
            )
        };
        if r != 0 {
            return Err(Errno::EFAULT);
        }
        Ok(unsafe { v.assume_init() })
    }

    /// Copies a `T` out to the caller.
    pub fn copy_out<T: IntoBytes + Immutable>(&self, v: &T) -> Result<(), Errno> {
        let bytes = v.as_bytes();
        let r = unsafe {
            sys::ddi_copyout(bytes.as_ptr() as *const sys::c_void, self.arg as *mut sys::c_void, bytes.len(), self.mode)
        };
        if r != 0 { Err(Errno::EFAULT) } else { Ok(()) }
    }
}
