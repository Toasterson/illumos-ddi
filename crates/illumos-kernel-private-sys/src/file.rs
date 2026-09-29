//! File structures and descriptors, `sys/file.h` and `uts/common/os/fio.c`.
//!
//! A file descriptor of a process names a `file_t`, which holds one
//! reference to a vnode and the open flags. `falloc` reserves a descriptor
//! in the calling process and allocates a `file_t` for a vnode whose
//! reference it takes over; the descriptor stays reserved (invisible to
//! the process) and the file locked until `setf` installs it. The undo path
//! before `setf` is `setf(fd, NULL)` and `unfalloc`. `getf` looks a
//! descriptor of the calling process up and holds it until `releasef`.
//! All of them act on `curproc`, so they are only meaningful in the context
//! of a system call of the process concerned.

use ddi_sys::{c_int, c_short, cred_t, kmutex_t, offset_t, ushort_t};

use crate::vnode::vnode;

ddi_sys::opaque! {
    /// Audit data of a file, `struct f_audit_data`; never touched here.
    f_audit_data;
    /// A file lock, `struct filock`; never touched here.
    filock;
}

/// `file_t`. Private, `uts/common/sys/file.h:62`.
///
/// The structure behind a file descriptor. Only `f_tlock` (released after
/// [`falloc`]), `f_flag` (the access mode) and `f_vnode` are used.
#[repr(C)]
pub struct file {
    /// Short-term lock; [`falloc`] returns with it held.
    pub f_tlock: kmutex_t,
    /// The open flags (`FREAD`, `FWRITE`, ...).
    pub f_flag: ushort_t,
    /// Extra flags (`FSEARCH`, `FEXEC`).
    pub f_flag2: ushort_t,
    /// The vnode; the file holds one reference to it.
    pub f_vnode: *mut vnode,
    /// The read/write offset.
    pub f_offset: offset_t,
    /// The credential of the opener.
    pub f_cred: *mut cred_t,
    /// Audit data.
    pub f_audit_data: *mut f_audit_data,
    /// The number of descriptors (in all processes) that refer to the file.
    pub f_count: c_int,
    /// The open file description lock.
    pub f_filock: *mut filock,
}
/// The typedef name of [`file`](struct@file).
pub type file_t = file;

const _: () = assert!(core::mem::size_of::<file>() == 64);
const _: () = assert!(core::mem::offset_of!(file, f_tlock) == 0);
const _: () = assert!(core::mem::offset_of!(file, f_flag) == 8);
const _: () = assert!(core::mem::offset_of!(file, f_flag2) == 10);
const _: () = assert!(core::mem::offset_of!(file, f_vnode) == 16);
const _: () = assert!(core::mem::offset_of!(file, f_offset) == 24);
const _: () = assert!(core::mem::offset_of!(file, f_cred) == 32);
const _: () = assert!(core::mem::offset_of!(file, f_audit_data) == 40);
const _: () = assert!(core::mem::offset_of!(file, f_count) == 48);
const _: () = assert!(core::mem::offset_of!(file, f_filock) == 56);

/// `FD_CLOEXEC`. Private, `uts/common/sys/fcntl.h:351`.
///
/// The per-descriptor close-on-exec flag for [`f_setfd_or`].
pub const FD_CLOEXEC: c_short = 1;

unsafe extern "C" {
    /// `falloc`. Private, `uts/common/os/fio.c:1362`.
    ///
    /// Reserves the lowest free descriptor of the calling process (unless
    /// `fdp` is NULL) and allocates a `file_t` with `f_count` 1, the flags
    /// `flag`, the vnode `vp` (whose reference the file takes over) and a
    /// hold on the caller's credential. Returns 0 with the file locked
    /// (`f_tlock` held) and the descriptor reserved but empty, or `EMFILE`.
    /// Allocates with `KM_SLEEP`; user context of the calling process only.
    pub fn falloc(vp: *mut vnode, flag: c_int, fpp: *mut *mut file, fdp: *mut c_int) -> c_int;
    /// `unfalloc`. Private, `uts/common/os/fio.c:1421`.
    ///
    /// Undoes [`falloc`] for a file that was never installed: drops the
    /// count and, at zero, the credential and the structure. The caller
    /// holds `f_tlock`, which the function releases. The vnode reference is
    /// not dropped; the caller does that.
    pub fn unfalloc(fp: *mut file);
    /// `setf`. Private, `uts/common/os/fio.c:1442`.
    ///
    /// Installs `fp` in the reserved descriptor `fd` of the calling process,
    /// making it visible, or with a NULL `fp` releases the reservation. The
    /// file must not be locked.
    pub fn setf(fd: c_int, fp: *mut file);
    /// `getf`. Private, `uts/common/os/fio.c:624`.
    ///
    /// Returns the file of descriptor `fd` of the calling process and holds
    /// the descriptor (a concurrent `close(2)` waits) until [`releasef`], or
    /// NULL if `fd` is not open. `fd` may be any value.
    pub fn getf(fd: c_int) -> *mut file;
    /// `releasef`. Private, `uts/common/os/fio.c:821`.
    ///
    /// Releases the hold [`getf`] took on `fd`; the same thread calls it
    /// once for each successful `getf`.
    pub fn releasef(fd: c_int);
    /// `f_setfd_or`. Private, `uts/common/os/fio.c:1587`.
    ///
    /// Adds descriptor flags (`FD_CLOEXEC`, `FD_CLOFORK`) to the installed
    /// descriptor `fd` of the calling process, as `pipe2(2)` does after
    /// `setf` for `O_CLOEXEC`.
    pub fn f_setfd_or(fd: c_int, flags: c_short);
}
