//! The ioctl command encoding from `sys/ioccom.h`.
//!
//! An ioctl command number packs a direction, the size of the argument
//! structure, a group letter and a sequence number into one 32-bit value.
//! The C header provides the `_IO`, `_IOR`, `_IOW` and `_IOWR` macros for
//! this; the `const fn`s below are their exact equivalents and are usable in
//! constant expressions.

use crate::types::c_int;

/// The mask applied to the argument size before it is packed into the
/// command (`IOCPARM_MASK`). Arguments larger than 255 bytes are truncated
/// in the encoding, which is why large ioctl structures are passed by
/// pointer.
pub const IOCPARM_MASK: u32 = 0xff;
/// The command carries no argument (`IOC_VOID`).
pub const IOC_VOID: u32 = 0x20000000;
/// The kernel copies the argument out to the caller (`IOC_OUT`).
pub const IOC_OUT: u32 = 0x40000000;
/// The kernel copies the argument in from the caller (`IOC_IN`).
pub const IOC_IN: u32 = 0x80000000;
/// The argument travels in both directions (`IOC_INOUT`).
pub const IOC_INOUT: u32 = IOC_IN | IOC_OUT;

/// Encodes a command without an argument, like the C macro `_IO(x, y)`.
pub const fn _IO(x: u8, y: u8) -> c_int {
    (IOC_VOID | ((x as u32) << 8) | y as u32) as c_int
}
/// Encodes a command whose argument the kernel writes to the caller, like
/// the C macro `_IOR(x, y, t)`. `size` stands for `sizeof(t)`.
pub const fn _IOR(x: u8, y: u8, size: usize) -> c_int {
    (IOC_OUT | (((size as u32) & IOCPARM_MASK) << 16) | ((x as u32) << 8) | y as u32) as c_int
}
/// Encodes a command whose argument the kernel reads from the caller, like
/// the C macro `_IOW(x, y, t)`.
pub const fn _IOW(x: u8, y: u8, size: usize) -> c_int {
    (IOC_IN | (((size as u32) & IOCPARM_MASK) << 16) | ((x as u32) << 8) | y as u32) as c_int
}
/// Encodes a command whose argument travels in both directions, like the C
/// macro `_IOWR(x, y, t)`.
pub const fn _IOWR(x: u8, y: u8, size: usize) -> c_int {
    (IOC_INOUT | (((size as u32) & IOCPARM_MASK) << 16) | ((x as u32) << 8) | y as u32) as c_int
}
