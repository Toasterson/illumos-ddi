//! `sys/stat.h`: node types for `ddi_create_minor_node(9F)`.

use crate::types::c_int;

/// `S_IFCHR`: character special.
pub const S_IFCHR: c_int = 0x2000;
/// `S_IFBLK`: block special.
pub const S_IFBLK: c_int = 0x6000;
