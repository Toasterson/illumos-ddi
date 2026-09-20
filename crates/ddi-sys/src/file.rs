//! `sys/file.h`, `sys/open.h`: open flags and open types passed to
//! `open(9E)`, `close(9E)`, `ioctl(9E)`.

use crate::types::c_int;

/// `FREAD`
pub const FREAD: c_int = 0x01;
/// `FWRITE`
pub const FWRITE: c_int = 0x02;
/// `FNDELAY`
pub const FNDELAY: c_int = 0x04;
/// `FAPPEND`
pub const FAPPEND: c_int = 0x08;
/// `FSYNC`
pub const FSYNC: c_int = 0x10;
/// `FNONBLOCK`
pub const FNONBLOCK: c_int = 0x80;
/// `FEXCL`
pub const FEXCL: c_int = 0x400;
/// `FKIOCTL`: ioctl argument addresses are in kernel space (layered callers).
pub const FKIOCTL: c_int = 0x80000000u32 as c_int;

/// `OTYP_BLK`: block device open.
pub const OTYP_BLK: c_int = 0;
/// `OTYP_MNT`: mounted (block) open.
pub const OTYP_MNT: c_int = 1;
/// `OTYP_CHR`: character device open.
pub const OTYP_CHR: c_int = 2;
/// `OTYP_SWP`: swap.
pub const OTYP_SWP: c_int = 3;
/// `OTYP_LYR`: layered (LDI) open.
pub const OTYP_LYR: c_int = 4;
