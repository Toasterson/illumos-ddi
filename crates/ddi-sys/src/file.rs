//! Open flags and open types from `sys/file.h` and `sys/open.h`.
//!
//! The kernel passes these values to `open(9E)`, `close(9E)` and, as part of
//! the `mode` argument, to `ioctl(9E)`.

use crate::types::c_int;

/// The file was opened for reading (`FREAD`).
pub const FREAD: c_int = 0x01;
/// The file was opened for writing (`FWRITE`).
pub const FWRITE: c_int = 0x02;
/// Non-blocking I/O was requested in the historical form (`FNDELAY`).
pub const FNDELAY: c_int = 0x04;
/// Writes append to the end of the file (`FAPPEND`).
pub const FAPPEND: c_int = 0x08;
/// Writes are synchronous (`FSYNC`).
pub const FSYNC: c_int = 0x10;
/// Non-blocking I/O was requested in the POSIX form (`FNONBLOCK`).
pub const FNONBLOCK: c_int = 0x80;
/// Exclusive access was requested (`FEXCL`).
pub const FEXCL: c_int = 0x400;
/// The ioctl argument addresses are in kernel space (`FKIOCTL`). This bit is
/// set when a layered driver calls through LDI rather than a user process
/// calling through the system call.
pub const FKIOCTL: c_int = 0x80000000u32 as c_int;

/// The device is being opened as a block device (`OTYP_BLK`).
pub const OTYP_BLK: c_int = 0;
/// The device is being opened as a block device in order to mount it (`OTYP_MNT`).
pub const OTYP_MNT: c_int = 1;
/// The device is being opened as a character device (`OTYP_CHR`).
pub const OTYP_CHR: c_int = 2;
/// The device is being opened for use as swap (`OTYP_SWP`).
pub const OTYP_SWP: c_int = 3;
/// The device is being opened by another kernel module through LDI (`OTYP_LYR`).
pub const OTYP_LYR: c_int = 4;
