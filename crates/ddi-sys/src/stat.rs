//! Node types from `sys/stat.h`.
//!
//! `ddi_create_minor_node(9F)` takes one of these as its `spec_type`
//! argument to say whether the node is a character or a block device.

use crate::types::c_int;

/// The minor node is a character special file (`S_IFCHR`).
pub const S_IFCHR: c_int = 0x2000;
/// The minor node is a block special file (`S_IFBLK`).
pub const S_IFBLK: c_int = 0x6000;
