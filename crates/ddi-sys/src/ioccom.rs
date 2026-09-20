//! `sys/ioccom.h`: ioctl command encoding (`_IO`, `_IOR`, `_IOW`, `_IOWR`).

use crate::types::c_int;

/// `IOCPARM_MASK`
pub const IOCPARM_MASK: u32 = 0xff;
/// `IOC_VOID`: no parameters.
pub const IOC_VOID: u32 = 0x20000000;
/// `IOC_OUT`: copy out parameters.
pub const IOC_OUT: u32 = 0x40000000;
/// `IOC_IN`: copy in parameters.
pub const IOC_IN: u32 = 0x80000000;
/// `IOC_INOUT`
pub const IOC_INOUT: u32 = IOC_IN | IOC_OUT;

/// `_IO(x, y)`
pub const fn _IO(x: u8, y: u8) -> c_int {
    (IOC_VOID | ((x as u32) << 8) | y as u32) as c_int
}
/// `_IOR(x, y, t)` with `size = sizeof(t)`.
pub const fn _IOR(x: u8, y: u8, size: usize) -> c_int {
    (IOC_OUT | (((size as u32) & IOCPARM_MASK) << 16) | ((x as u32) << 8) | y as u32) as c_int
}
/// `_IOW(x, y, t)`
pub const fn _IOW(x: u8, y: u8, size: usize) -> c_int {
    (IOC_IN | (((size as u32) & IOCPARM_MASK) << 16) | ((x as u32) << 8) | y as u32) as c_int
}
/// `_IOWR(x, y, t)`
pub const fn _IOWR(x: u8, y: u8, size: usize) -> c_int {
    (IOC_INOUT | (((size as u32) & IOCPARM_MASK) << 16) | ((x as u32) << 8) | y as u32) as c_int
}
