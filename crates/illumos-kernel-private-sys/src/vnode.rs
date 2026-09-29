//! Vnodes, `sys/vnode.h` and `uts/common/fs/vnode.c`.
//!
//! Only the public prefix of `vnode_t` is declared (the header marks every
//! later field "PRIVATE FIELDS BELOW - DO NOT USE"); a Rust value of the
//! type never exists, only pointers the kernel hands out.

use ddi_sys::{c_int, c_void, cred_t, dev_t, kmutex_t, longlong_t, offset_t, uint_t, ulong_t};

ddi_sys::opaque! {
    /// A mounted file system, `struct vfs`.
    vfs;
    /// A stream head, `struct stdata`.
    stdata;
}

/// `vtype_t`. Private, `uts/common/sys/vnode.h:161`.
///
/// The type of a vnode, an `enum vtype` (4 bytes).
pub type vtype_t = c_int;
/// `VCHR`. Private, `uts/common/sys/vnode.h:166`.
///
/// A character special file.
pub const VCHR: vtype_t = 4;

/// `vnode_t`. Private, `uts/common/sys/vnode.h:284`.
///
/// The public prefix of the vnode, up to `v_rdev`; the private fields that
/// follow are left out, so the Rust type is smaller than the C one and is
/// only ever used behind a pointer.
#[repr(C)]
pub struct vnode {
    /// Protects the vnode fields.
    pub v_lock: kmutex_t,
    /// Vnode flags.
    pub v_flag: uint_t,
    /// Reference count.
    pub v_count: uint_t,
    /// File system private data.
    pub v_data: *mut c_void,
    /// The containing file system.
    pub v_vfsp: *mut vfs,
    /// The associated stream.
    pub v_stream: *mut stdata,
    /// The vnode type ([`VCHR`] for a character device).
    pub v_type: vtype_t,
    /// The device number, for `VCHR` and `VBLK`.
    pub v_rdev: dev_t,
    _private: [u8; 0],
}
/// The typedef name of [`vnode`].
pub type vnode_t = vnode;

const _: () = assert!(core::mem::offset_of!(vnode, v_lock) == 0);
const _: () = assert!(core::mem::offset_of!(vnode, v_flag) == 8);
const _: () = assert!(core::mem::offset_of!(vnode, v_count) == 12);
const _: () = assert!(core::mem::offset_of!(vnode, v_data) == 16);
const _: () = assert!(core::mem::offset_of!(vnode, v_vfsp) == 24);
const _: () = assert!(core::mem::offset_of!(vnode, v_stream) == 32);
const _: () = assert!(core::mem::offset_of!(vnode, v_type) == 40);
const _: () = assert!(core::mem::offset_of!(vnode, v_rdev) == 48);

/// `caller_context_t`. Private, `uts/common/sys/vnode.h:867`.
///
/// Caller context for vnode operations; always passed as NULL here.
#[repr(C)]
pub struct caller_context {
    /// Process ID of the caller.
    pub cc_pid: c_int,
    /// System ID, used for remote calls.
    pub cc_sysid: c_int,
    /// Identifier for a set of callers.
    pub cc_caller_id: longlong_t,
    /// Flags.
    pub cc_flags: ulong_t,
}
/// The typedef name of [`caller_context`].
pub type caller_context_t = caller_context;

const _: () = assert!(core::mem::size_of::<caller_context>() == 24);
const _: () = assert!(core::mem::offset_of!(caller_context, cc_pid) == 0);
const _: () = assert!(core::mem::offset_of!(caller_context, cc_sysid) == 4);
const _: () = assert!(core::mem::offset_of!(caller_context, cc_caller_id) == 8);
const _: () = assert!(core::mem::offset_of!(caller_context, cc_flags) == 16);

unsafe extern "C" {
    /// `fop_open`. Private, `uts/common/sys/vnode.h:1025`.
    ///
    /// `VOP_OPEN`: opens `*vpp` with the open flags `mode` (`FREAD`,
    /// `FWRITE`) and credential `cr`. For a specfs vnode this is
    /// `spec_open`, which calls the driver's `open(9E)`; a cloning driver
    /// may replace `*vpp` by the vnode of another device number, whose
    /// reference then replaces the caller's. On success the vnode needs a
    /// matching [`fop_close`] (a `closef` of the file does that).
    pub fn fop_open(vpp: *mut *mut vnode, mode: c_int, cr: *mut cred_t, ct: *mut caller_context) -> c_int;
    /// `fop_close`. Private, `uts/common/sys/vnode.h:1026`.
    ///
    /// `VOP_CLOSE`: undoes one [`fop_open`]; `count` is the number of file
    /// references being closed (1). For a specfs vnode the driver's
    /// `close(9E)` runs on the last close of the device number.
    pub fn fop_close(
        vp: *mut vnode,
        flag: c_int,
        count: c_int,
        offset: offset_t,
        cr: *mut cred_t,
        ct: *mut caller_context,
    ) -> c_int;
    /// `vn_rele`. Private, `uts/common/sys/vnode.h:1338`.
    ///
    /// `VN_RELE`: drops one reference to the vnode, which is freed at zero.
    pub fn vn_rele(vp: *mut vnode);
}
