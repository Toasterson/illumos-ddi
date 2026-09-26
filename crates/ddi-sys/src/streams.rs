//! STREAMS from `sys/stream.h`, `sys/stropts.h`, `sys/strsun.h`,
//! `sys/strlog.h` and the DLPI helper prototypes in `sys/dlpi.h`.
//!
//! STREAMS is the kernel's framework for character devices and protocol
//! modules that process data as a sequence of messages rather than as a
//! byte stream. A stream is a chain of queue pairs between the stream head,
//! which talks to the process, and the driver at the bottom; each pair has a
//! read queue for upstream traffic and a write queue for downstream
//! traffic, and messages travel through them as chains of message blocks.
//! Modules can be pushed between the head and the driver, and a multiplexing
//! driver can link whole lower streams underneath itself.
//!
//! This module declares the structures with a section 9S page, namely
//! `queue(9S)`, `qinit(9S)`, `module_info(9S)`, `streamtab(9S)`, `msgb(9S)`,
//! `datab(9S)`, `free_rtn(9S)`, `iocblk(9S)`, `copyreq(9S)`, `copyresp(9S)`,
//! `linkblk(9S)`, `qband(9S)` and `stroptions(9S)`, the constants those
//! pages and the 9F pages refer to, and every STREAMS function with a
//! section 9F page whose interface level is not obsolete, plus the three
//! `ddi.mapfile` entries without a page of their own, `freemsgchain`,
//! `msgpullup_pad` and `desballoc`. The `RD`, `WR`, `OTHERQ`, `SAMESTR` and
//! `datamsg` names are macros in `sys/stream.h`, but `sys/ddi.h` undefines
//! them and declares real functions that `os/ddi.c` exports from `genunix`,
//! so they are bound as functions here. The DLPI helpers on the
//! `dlbindack(9F)` page are implemented in `io/sundlpi.c`, which is part of
//! `genunix`, so they need no module dependency.
//!
//! The accessor macros in `sys/strsun.h`, namely `DB_BASE`, `DB_LIM`,
//! `DB_REF`, `DB_TYPE`, `DB_FLAGS`, `MBLKL`, `MBLKSIZE`, `MBLKHEAD`,
//! `MBLKTAIL` and `MBLKIN`, and the `canenable`, `queclass`, `straln` and
//! `bpsize` macros in `sys/stream.h`, have no function form in the kernel.
//! They are left to the safe `ddi` crate, together with the `STRLOG` macro
//! from `sys/strlog.h` and the `IOC_CONVERT_FROM` macro. Nothing is taken
//! from `sys/strsubr.h`; the private fields of `queue` and `datab` are
//! declared so that the layouts match, but they are documented as private.

use crate::ksynch::{kcondvar_t, kmutex_t};
use crate::time::timeout_id_t;
use crate::types::*;

opaque! {
    /// The per-stream state kept by the stream head, `struct stdata`. It is
    /// private to the kernel and only appears behind `q_stream`.
    stdata;
    /// A synchronisation queue, `struct syncq`, which implements the
    /// perimeters described in `mt-streams(9F)`. Private to the kernel.
    syncq;
    /// The kernel's record of a pushable module, `struct fmodsw_impl`.
    /// Private to the kernel.
    fmodsw_impl;
    /// The module statistics structure, `struct module_stat`. The `qinit(9S)`
    /// page notes that this release includes no support for module
    /// statistics, so the slot is always NULL.
    module_stat;
    /// The flow trace header, `struct fthdr`, from the private `sys/strft.h`.
    fthdr;
    /// The argument of the synchronous STREAMS read/write procedure,
    /// `struiod_t`. Not part of the DDI.
    struiod;
    /// The argument of the synchronous STREAMS information procedure,
    /// `infod_t`. Not part of the DDI.
    infod;
}

/// The opaque handle returned by `bufcall(9F)`, `esbbcall(9F)` and
/// `qbufcall(9F)`, `bufcall_id_t`. The header defines it as a `void *`; a
/// value of 0 means that no callback was scheduled.
pub type bufcall_id_t = *mut c_void;

// --- queue(9S) -------------------------------------------------------------

/// `queue(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// A STREAMS driver or module consists of two queue structures, a read
/// queue for upstream processing and a write queue for downstream
/// processing. The queue structure is the major building block of a
/// stream. It can be accessed at any time from inside a STREAMS entry point
/// associated with that queue.
///
/// Only the fields the manual page lists are public, namely `q_qinfo`,
/// `q_first`, `q_last`, `q_next`, `q_ptr`, `q_count`, `q_flag`, `q_minpsz`,
/// `q_maxpsz`, `q_hiwat` and `q_lowat`. Every other field is private to the
/// STREAMS framework and must not be accessed; they are declared here only
/// so that the layout and size match the kernel's. Aside from `q_ptr` and
/// `q_qinfo`, a module or driver must never assume that a field value will
/// remain unchanged across calls to STREAMS entry points.
///
/// The initial values of `q_minpsz`, `q_maxpsz`, `q_hiwat` and `q_lowat`
/// are copied from the `module_info(9S)` structure when the queue is
/// created, and the module or driver may change them afterwards. The
/// remaining visible fields, `q_qinfo`, `q_first`, `q_last`, `q_next`,
/// `q_count` and `q_flag`, must never be modified by a module or driver.
///
/// A module or driver must not access the structure pointed to by
/// `q_next`, must not rely on its value before `qprocson(9F)` or after
/// `qprocsoff(9F)`, and must not pass it to any framework function other
/// than `put(9F)`, `canput(9F)`, `bcanput(9F)`, `putctl(9F)` and
/// `putctl1(9F)`, preferring the `putnext(9F)` family in every case.
/// Checking `q_next` for NULL is the sanctioned way to tell a driver from a
/// module in shared code.
#[repr(C)]
pub struct queue {
    /// The processing procedures of this queue. May be changed before
    /// `qprocson(9F)` is called.
    pub q_qinfo: *mut qinit,
    /// The first message on the queue, or NULL if the queue is empty.
    pub q_first: *mut msgb,
    /// The last message on the queue, or NULL if the queue is empty.
    pub q_last: *mut msgb,
    /// The next queue in the stream in the direction of flow. NULL at the
    /// end of a stream, which is how a driver can be told from a module.
    pub q_next: *mut queue,
    /// Private to the framework. The next queue on the service list.
    pub q_link: *mut queue,
    /// Module-specific data. The framework never inspects it. It usually
    /// points to per-queue state allocated in `open(9E)` and freed in
    /// `close(9E)`.
    pub q_ptr: *mut c_void,
    /// The number of bytes of message data currently on the queue.
    pub q_count: size_t,
    /// The queue state, a combination of the `Q*` flags. Only `QFULL`,
    /// `QREADR`, `QUSE`, `QENAB`, `QNOENB`, `QWANTR` and `QWANTW` may be
    /// tested by a driver. The value can change at any time.
    pub q_flag: uint_t,
    /// The smallest packet the queue accepts, in bytes.
    pub q_minpsz: ssize_t,
    /// The largest packet the queue accepts, in bytes, or `INFPSZ`.
    pub q_maxpsz: ssize_t,
    /// The high water mark for flow control, in bytes.
    pub q_hiwat: size_t,
    /// The low water mark for flow control, in bytes.
    pub q_lowat: size_t,
    /// Private to the framework. The list of `qband` structures.
    pub q_bandp: *mut qband,
    /// Private to the framework. The lock protecting the queue.
    pub q_lock: kmutex_t,
    /// Private to the framework. The stream head this queue belongs to.
    pub q_stream: *mut stdata,
    /// Private to the framework. The associated synchronisation queue.
    pub q_syncq: *mut syncq,
    /// Private to the framework. The number of priority bands in use.
    pub q_nband: c_uchar,
    /// Private to the framework. The condition variable for sleeping
    /// readers and writers at the stream head.
    pub q_wait: kcondvar_t,
    /// Private to the framework. The next queue with a service routine.
    pub q_nfsrv: *mut queue,
    /// Private to the framework. Non-zero while the queue is being drained.
    pub q_draining: c_ushort,
    /// Private to the framework. The synchronous STREAMS uio mode.
    pub q_struiot: c_short,
    /// Private to the framework. The time the queue was enabled.
    pub q_qtstamp: clock_t,
    /// Private to the framework. The number of message blocks on the queue.
    pub q_mblkcnt: size_t,
    /// Private to the framework. The number of messages on the syncq.
    pub q_syncqmsgs: uint_t,
    /// Private to the framework. The number of threads in `rwnext`.
    pub q_rwcnt: size_t,
    /// Private to the framework. The scheduling priority of the queue.
    pub q_spri: pri_t,
    /// Private to the framework. The first message queued on the syncq.
    pub q_sqhead: *mut msgb,
    /// Private to the framework. The last message queued on the syncq.
    pub q_sqtail: *mut msgb,
    /// Private to the framework. The next queue on the syncq list.
    pub q_sqnext: *mut queue,
    /// Private to the framework. The previous queue on the syncq list.
    pub q_sqprev: *mut queue,
    /// Private to the framework. The syncq flags.
    pub q_sqflags: uint_t,
    /// Private to the framework. The time the queue was scheduled.
    pub q_sqtstamp: clock_t,
    /// Private to the framework. The module implementation record, NULL for
    /// a driver's queues.
    pub q_fp: *mut fmodsw_impl,
}
/// The typedef name of [`queue`].
pub type queue_t = queue;

// queue under LP64: 6 pointers (48), q_count (56), q_flag at 56 padded to
// 64, four size fields to 96, q_bandp 96, q_lock 104, q_stream 112,
// q_syncq 120, q_nband 128, q_wait at 130, q_nfsrv 136, q_draining 144,
// q_struiot 146, q_qtstamp 152, q_mblkcnt 160, q_syncqmsgs 168 padded to
// 176, q_rwcnt 176, q_spri 184 padded to 192, four pointers to 224,
// q_sqflags 224 padded to 232, q_sqtstamp 232, q_fp 240; total 248.
const _: () = assert!(core::mem::size_of::<queue>() == 248);
const _: () = assert!(core::mem::offset_of!(queue, q_qinfo) == 0);
const _: () = assert!(core::mem::offset_of!(queue, q_ptr) == 40);
const _: () = assert!(core::mem::offset_of!(queue, q_flag) == 56);
const _: () = assert!(core::mem::offset_of!(queue, q_minpsz) == 64);
const _: () = assert!(core::mem::offset_of!(queue, q_lowat) == 88);
const _: () = assert!(core::mem::offset_of!(queue, q_lock) == 104);
const _: () = assert!(core::mem::offset_of!(queue, q_nband) == 128);
const _: () = assert!(core::mem::offset_of!(queue, q_wait) == 130);
const _: () = assert!(core::mem::offset_of!(queue, q_nfsrv) == 136);
const _: () = assert!(core::mem::offset_of!(queue, q_qtstamp) == 152);
const _: () = assert!(core::mem::offset_of!(queue, q_rwcnt) == 176);
const _: () = assert!(core::mem::offset_of!(queue, q_sqhead) == 192);
const _: () = assert!(core::mem::offset_of!(queue, q_sqtstamp) == 232);
const _: () = assert!(core::mem::offset_of!(queue, q_fp) == 240);

/// The queue is already enabled to run (`QENAB`). Set by `qenable(9F)`.
pub const QENAB: uint_t = 0x00000001;
/// An upstream processing element wants to read from the queue (`QWANTR`).
pub const QWANTR: uint_t = 0x00000002;
/// A downstream processing element wants to write to the queue (`QWANTW`).
pub const QWANTW: uint_t = 0x00000004;
/// The queue is considered full for flow control purposes (`QFULL`).
pub const QFULL: uint_t = 0x00000008;
/// The queue is the read queue of its pair, used for upstream processing
/// (`QREADR`).
pub const QREADR: uint_t = 0x00000010;
/// The queue has been allocated and is in use (`QUSE`).
pub const QUSE: uint_t = 0x00000020;
/// The queue will not be scheduled for service by `putq(9F)`, `putbq(9F)` or
/// `insq(9F)`; set by `noenable(9F)` and cleared by `enableok(9F)`
/// (`QNOENB`).
pub const QNOENB: uint_t = 0x00000040;

/// The maximum number of priority bands a queue can have (`NBAND`).
pub const NBAND: c_int = 256;

/// `qband(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// Contains the flow control information for one priority band of a
/// queue. The framework allocates one of these per band in use and chains
/// them from the queue. Drivers read band information through
/// `strqget(9F)` rather than by walking this chain.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct qband {
    /// The information for the next band.
    pub qb_next: *mut qband,
    /// The number of bytes of message data in the band.
    pub qb_count: size_t,
    /// The first message of the band's data.
    pub qb_first: *mut msgb,
    /// The last message of the band's data.
    pub qb_last: *mut msgb,
    /// The band's high water mark.
    pub qb_hiwat: size_t,
    /// The band's low water mark.
    pub qb_lowat: size_t,
    /// The band state, a combination of `QB_FULL` and `QB_WANTW`.
    pub qb_flag: uint_t,
    /// Private to the framework. The number of message blocks in the band,
    /// used to detect runaway messages.
    pub qb_mblkcnt: size_t,
}
/// The typedef name of [`qband`].
pub type qband_t = qband;

// qband: pointer, size, two pointers, two sizes (48), qb_flag at 48 padded
// to 56, qb_mblkcnt 56; total 64.
const _: () = assert!(core::mem::size_of::<qband>() == 64);
const _: () = assert!(core::mem::offset_of!(qband, qb_next) == 0);
const _: () = assert!(core::mem::offset_of!(qband, qb_first) == 16);
const _: () = assert!(core::mem::offset_of!(qband, qb_flag) == 48);
const _: () = assert!(core::mem::offset_of!(qband, qb_mblkcnt) == 56);

/// The band is considered full (`QB_FULL`).
pub const QB_FULL: uint_t = 0x01;
/// Someone wants to write to the band (`QB_WANTW`).
pub const QB_WANTW: uint_t = 0x02;
/// The band has been back-enabled (`QB_BACK`).
pub const QB_BACK: uint_t = 0x04;

/// The field selector of `strqget(9F)` and `strqset(9F)`, `qfields_t`.
///
/// Both functions take a selector naming the queue field, or the field of
/// the priority band given by their `pri` argument, that is to be read or
/// written. `strqset(9F)` accepts only the first four values.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct qfields_t(pub c_int);
impl qfields_t {
    /// The high water mark, `q_hiwat` or `qb_hiwat` (`QHIWAT`).
    pub const QHIWAT: Self = Self(0);
    /// The low water mark, `q_lowat` or `qb_lowat` (`QLOWAT`).
    pub const QLOWAT: Self = Self(1);
    /// The largest packet accepted, `q_maxpsz` (`QMAXPSZ`).
    pub const QMAXPSZ: Self = Self(2);
    /// The smallest packet accepted, `q_minpsz` (`QMINPSZ`).
    pub const QMINPSZ: Self = Self(3);
    /// The approximate number of bytes of data, `q_count` or `qb_count`
    /// (`QCOUNT`).
    pub const QCOUNT: Self = Self(4);
    /// The first message, `q_first` or `qb_first` (`QFIRST`).
    pub const QFIRST: Self = Self(5);
    /// The last message, `q_last` or `qb_last` (`QLAST`).
    pub const QLAST: Self = Self(6);
    /// The status flags, `q_flag` or `qb_flag` (`QFLAG`).
    pub const QFLAG: Self = Self(7);
    /// The synchronous STREAMS uio mode, `q_struiot` (`QSTRUIOT`). Not
    /// documented on the manual page.
    pub const QSTRUIOT: Self = Self(8);
    /// The first invalid selector (`QBAD`).
    pub const QBAD: Self = Self(9);
}

// --- module_info(9S), qinit(9S), streamtab(9S) ------------------------------

/// `module_info(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// Holds the identification and limit values that are set when a module or
/// driver is declared. The structure is intended to be read-only, but the
/// flow control limits `mi_hiwat` and `mi_lowat` and the packet size
/// limits `mi_minpsz` and `mi_maxpsz` are copied into the queue structure
/// when a queue is created, where they can be modified. For a driver,
/// `mi_idname` must match the name of the driver binary; for a module it
/// must match the `fname` field of the `fmodsw` structure. The constant
/// `FMNAMESZ` limits the length of a module name to eight characters.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct module_info {
    /// The module identification number, also passed as `mid` to
    /// `strlog(9F)`.
    pub mi_idnum: c_ushort,
    /// The module name.
    pub mi_idname: *mut c_char,
    /// The minimum packet size accepted.
    pub mi_minpsz: ssize_t,
    /// The maximum packet size accepted, or `INFPSZ`.
    pub mi_maxpsz: ssize_t,
    /// The high water mark for flow control.
    pub mi_hiwat: size_t,
    /// The low water mark for flow control.
    pub mi_lowat: size_t,
}

// module_info: mi_idnum at 0 padded to 8, mi_idname 8, four size fields 16
// to 48; total 48.
const _: () = assert!(core::mem::size_of::<module_info>() == 48);
const _: () = assert!(core::mem::offset_of!(module_info, mi_idnum) == 0);
const _: () = assert!(core::mem::offset_of!(module_info, mi_idname) == 8);
const _: () = assert!(core::mem::offset_of!(module_info, mi_lowat) == 40);

/// The signature of the STREAMS `put(9E)` entry point, `qi_putp_t`. It
/// receives the queue and the message and its return value is ignored.
pub type qi_putp_t = unsafe extern "C" fn(q: *mut queue, mp: *mut msgb) -> c_int;
/// The signature of the STREAMS `srv(9E)` entry point, `qi_srvp_t`. Its
/// return value is ignored.
pub type qi_srvp_t = unsafe extern "C" fn(q: *mut queue) -> c_int;
/// The signature of the STREAMS `open(9E)` entry point, `qi_qopen_t`. The
/// arguments are the read queue, the device number, which a clone open
/// updates, the open flags, the stream flag which is 0, `MODOPEN` or
/// `CLONEOPEN`, and the caller's credentials. It returns 0 or an errno.
pub type qi_qopen_t =
    unsafe extern "C" fn(q: *mut queue, devp: *mut dev_t, oflag: c_int, sflag: c_int, credp: *mut cred_t) -> c_int;
/// The signature of the STREAMS `close(9E)` entry point, `qi_qclose_t`. The
/// arguments are the read queue, the open flags and the caller's
/// credentials. It returns 0 or an errno.
pub type qi_qclose_t = unsafe extern "C" fn(q: *mut queue, flag: c_int, credp: *mut cred_t) -> c_int;
/// The signature of the unused administrative entry point, `qi_qadmin_t`.
/// The header reserves the slot for future use.
pub type qi_qadmin_t = unsafe extern "C" fn() -> c_int;
/// The signature of the synchronous STREAMS read/write procedure,
/// `qi_rwp_t`. Not part of the DDI; drivers leave the slot NULL.
pub type qi_rwp_t = unsafe extern "C" fn(q: *mut queue, dp: *mut struiod) -> c_int;
/// The signature of the synchronous STREAMS information procedure,
/// `qi_infop_t`. Not part of the DDI; drivers leave the slot NULL.
pub type qi_infop_t = unsafe extern "C" fn(q: *mut queue, dp: *mut infod) -> c_int;

/// `qinit(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// Contains the pointers to the processing procedures of a queue. The
/// `streamtab(9S)` structure of a module or driver points to one `qinit`
/// for the read side and one for the write side. Normally the read side
/// carries the open and close routines, and both sides may carry put and
/// service procedures. The manual page notes that this release includes
/// no support for module statistics, so `qi_mstat` is always NULL.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct qinit {
    /// The `put(9E)` procedure.
    pub qi_putp: Option<qi_putp_t>,
    /// The `srv(9E)` procedure, or NULL if the queue has none.
    pub qi_srvp: Option<qi_srvp_t>,
    /// The `open(9E)` procedure, called on startup.
    pub qi_qopen: Option<qi_qopen_t>,
    /// The `close(9E)` procedure, called on finish.
    pub qi_qclose: Option<qi_qclose_t>,
    /// Reserved for future use. Always NULL.
    pub qi_qadmin: Option<qi_qadmin_t>,
    /// The `module_info(9S)` of the module or driver.
    pub qi_minfo: *mut module_info,
    /// The module statistics structure. Unsupported; always NULL.
    pub qi_mstat: *mut module_stat,
    /// The synchronous STREAMS read/write procedure. Drivers set NULL.
    pub qi_rwp: Option<qi_rwp_t>,
    /// The synchronous STREAMS information procedure. Drivers set NULL.
    pub qi_infop: Option<qi_infop_t>,
    /// The stream uio type for `struio`, one of the `STRUIOT_*` values.
    /// Drivers set 0, which is `STRUIOT_DONTCARE`.
    pub qi_struiot: c_int,
}

// qinit: nine pointers (72) followed by an int at 72 padded to 80.
const _: () = assert!(core::mem::size_of::<qinit>() == 80);
const _: () = assert!(core::mem::offset_of!(qinit, qi_putp) == 0);
const _: () = assert!(core::mem::offset_of!(qinit, qi_minfo) == 40);
const _: () = assert!(core::mem::offset_of!(qinit, qi_infop) == 64);
const _: () = assert!(core::mem::offset_of!(qinit, qi_struiot) == 72);

/// The queue does not support `struio` (`STRUIOT_NONE`).
pub const STRUIOT_NONE: c_int = -1;
/// The queue uses the current `uiomove` implementation, which is the
/// default (`STRUIOT_DONTCARE`).
pub const STRUIOT_DONTCARE: c_int = 0;
/// The queue uses the standard `uiomove` (`STRUIOT_STANDARD`).
pub const STRUIOT_STANDARD: c_int = 1;

/// `streamtab(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// Every STREAMS driver or module must have a `streamtab`. It is made up of
/// `qinit(9S)` pointers for the read and write queue of the module or
/// driver, and for multiplexing drivers also for the lower read and write
/// queues. A driver publishes it through the `cb_str` field of
/// `cb_ops(9S)`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct streamtab {
    /// The read queue procedures.
    pub st_rdinit: *mut qinit,
    /// The write queue procedures.
    pub st_wrinit: *mut qinit,
    /// The lower read queue procedures of a multiplexer, or NULL.
    pub st_muxrinit: *mut qinit,
    /// The lower write queue procedures of a multiplexer, or NULL.
    pub st_muxwinit: *mut qinit,
}

// streamtab: four pointers.
const _: () = assert!(core::mem::size_of::<streamtab>() == 32);
const _: () = assert!(core::mem::offset_of!(streamtab, st_rdinit) == 0);
const _: () = assert!(core::mem::offset_of!(streamtab, st_muxwinit) == 24);

/// `linkblk(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// Connects a lower stream to an upper multiplexing driver. The stream head
/// allocates and initialises it as a result of the `I_LINK`, `I_UNLINK`,
/// `I_PLINK` and `I_PUNLINK` ioctl commands and passes it to the driver in
/// the `M_DATA` portion of the `M_IOCTL` message.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct linkblk {
    /// The lowest level write queue of the upper stream. NULL for
    /// persistent links.
    pub l_qtop: *mut queue,
    /// The highest level write queue of the lower stream.
    pub l_qbot: *mut queue,
    /// The index identifying the lower stream, which the driver stores to
    /// match a later unlink.
    pub l_index: c_int,
}

// linkblk: two pointers and an int padded to 24.
const _: () = assert!(core::mem::size_of::<linkblk>() == 24);
const _: () = assert!(core::mem::offset_of!(linkblk, l_qtop) == 0);
const _: () = assert!(core::mem::offset_of!(linkblk, l_qbot) == 8);
const _: () = assert!(core::mem::offset_of!(linkblk, l_index) == 16);

// --- msgb(9S), datab(9S), free_rtn(9S) --------------------------------------

/// The signature of the freeing routine in [`free_rtn`]. The header
/// declares it without a prototype as `void (*free_func)()`; the kernel
/// calls it with the `free_arg` field as its only argument.
pub type free_func_t = unsafe extern "C" fn(arg: caddr_t);

/// `free_rtn(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// Specifies a driver's message-freeing routine. The `datab(9S)` of a
/// message allocated with `esballoc(9F)` or `desballoc(9F)` refers to it,
/// and when `freeb(9F)` releases the last reference to the data block it
/// calls `free_func` with `free_arg` so that the driver can release the
/// buffer it supplied. The structure is defined as type `frtn_t`. Its
/// address is stored in the data block, so it must remain valid until the
/// message has been freed.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct free_rtn {
    /// The driver's freeing routine.
    pub free_func: Option<free_func_t>,
    /// The argument passed to `free_func`. It is a `caddr_t` so that the
    /// driver can pass a pointer to a structure when more than one value is
    /// needed.
    pub free_arg: caddr_t,
}
/// The typedef name of [`free_rtn`].
pub type frtn_t = free_rtn;

// free_rtn: two pointers.
const _: () = assert!(core::mem::size_of::<free_rtn>() == 16);
const _: () = assert!(core::mem::offset_of!(free_rtn, free_func) == 0);
const _: () = assert!(core::mem::offset_of!(free_rtn, free_arg) == 8);

/// The maximum value of `db_ref` (`DBLK_REFMAX`). `dupb(9F)` fails once a
/// data block has this many references.
pub const DBLK_REFMAX: c_uchar = 255;

/// The checksum value member of the private `db_struioun` union of
/// `datab(9S)`. Anonymous in C.
#[repr(C)]
#[derive(Clone, Copy)]
pub union datab_cksum_val {
    /// A 32-bit checksum.
    pub u32: u32,
    /// A 16-bit checksum.
    pub u16: u16,
}

/// The checksum member of the private `db_struioun` union of `datab(9S)`.
/// Anonymous in C.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct datab_cksum {
    /// The stored checksum.
    pub cksum_val: datab_cksum_val,
    /// Checksum flags.
    pub flags: u16,
    /// Padding, which GLDv2 uses to carry VLAN tag information.
    pub pad: u16,
}

/// The private `db_struioun` union of `datab(9S)`, which the network stack
/// uses to carry checksum state. Anonymous in C. It is eight bytes and
/// aligned to eight because of the `double` member.
#[repr(C)]
#[derive(Clone, Copy)]
pub union datab_struioun {
    /// Forces eight-byte alignment.
    pub enforce_alignment: f64,
    /// The raw bytes.
    pub data: [u8; 8],
    /// The checksum view.
    pub cksum: datab_cksum,
}

const _: () = assert!(core::mem::size_of::<datab_struioun>() == 8);
const _: () = assert!(core::mem::align_of::<datab_struioun>() == 8);

/// The signature of the private data block freeing hooks in [`datab`].
pub type db_free_t = unsafe extern "C" fn(mp: *mut msgb, dbp: *mut datab);

/// `datab(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// Describes the data of a STREAMS message. The actual data is stored in a
/// buffer that this structure points to, and a `msgb(9S)` message block
/// points to the `datab`. Because more than one message block can point to
/// the same data block, `db_ref` counts the references and prevents the
/// block from being freed until every message block is finished with it.
///
/// The manual page documents only `db_base`, `db_lim`, `db_ref` and
/// `db_type`. Every other field is private to the framework and is declared
/// here only so that the layout matches. The structure is defined as type
/// `dblk_t`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct datab {
    /// Private to the framework. The `free_rtn(9S)` of an externally
    /// supplied buffer, or NULL.
    pub db_frtnp: *mut free_rtn,
    /// The first byte of the buffer.
    pub db_base: *mut c_uchar,
    /// One past the last byte of the buffer.
    pub db_lim: *mut c_uchar,
    /// The number of message blocks pointing to this data block.
    pub db_ref: c_uchar,
    /// The message type, one of the `M_*` values.
    pub db_type: c_uchar,
    /// Private to the framework. The `DBLK_*` state flags.
    pub db_flags: c_uchar,
    /// Private to the framework. The synchronous STREAMS flags.
    pub db_struioflag: c_uchar,
    /// Private to the framework. The cached process ID of the sender, a
    /// `pid_t`, which is an `int`.
    pub db_cpid: c_int,
    /// Private to the framework. The kmem cache the block came from.
    pub db_cache: *mut c_void,
    /// Private to the framework. The message block allocated together with
    /// this data block.
    pub db_mblk: *mut msgb,
    /// Private to the framework. The routine that frees a message block
    /// referring to this data block.
    pub db_free: Option<db_free_t>,
    /// Private to the framework. The routine that frees the last reference.
    pub db_lastfree: Option<db_free_t>,
    /// Private to the framework. Checksum start offset.
    pub db_cksumstart: intptr_t,
    /// Private to the framework. Checksum end offset.
    pub db_cksumend: intptr_t,
    /// Private to the framework. Checksum stuff offset.
    pub db_cksumstuff: intptr_t,
    /// Private to the framework. Checksum state.
    pub db_struioun: datab_struioun,
    /// Private to the framework. The flow trace header.
    pub db_fthdr: *mut fthdr,
    /// Private to the framework. The credentials attached to the message.
    pub db_credp: *mut cred_t,
}
/// The typedef name of [`datab`].
pub type dblk_t = datab;

// datab: three pointers (24), four bytes 24 to 28, db_cpid at 28, four
// pointers 32 to 64, three intptr_t 64 to 88, db_struioun 88, db_fthdr 96,
// db_credp 104; total 112.
const _: () = assert!(core::mem::size_of::<datab>() == 112);
const _: () = assert!(core::mem::offset_of!(datab, db_frtnp) == 0);
const _: () = assert!(core::mem::offset_of!(datab, db_base) == 8);
const _: () = assert!(core::mem::offset_of!(datab, db_ref) == 24);
const _: () = assert!(core::mem::offset_of!(datab, db_type) == 25);
const _: () = assert!(core::mem::offset_of!(datab, db_cpid) == 28);
const _: () = assert!(core::mem::offset_of!(datab, db_cache) == 32);
const _: () = assert!(core::mem::offset_of!(datab, db_free) == 48);
const _: () = assert!(core::mem::offset_of!(datab, db_cksumstart) == 64);
const _: () = assert!(core::mem::offset_of!(datab, db_struioun) == 88);
const _: () = assert!(core::mem::offset_of!(datab, db_fthdr) == 96);
const _: () = assert!(core::mem::offset_of!(datab, db_credp) == 104);

/// `msgb(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// A STREAMS message is made up of one or more message blocks, referenced
/// by a pointer to a `msgb`. The `b_next` and `b_prev` pointers link
/// messages together on a queue, and `b_cont` links the blocks of a
/// message that consists of more than one block. Each block points to a
/// `datab(9S)`, which holds the message type and the location of the data.
/// The structure is defined as type `mblk_t`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct msgb {
    /// The next message on the queue.
    pub b_next: *mut msgb,
    /// The previous message on the queue.
    pub b_prev: *mut msgb,
    /// The next block of this message.
    pub b_cont: *mut msgb,
    /// The first unread byte of the data buffer.
    pub b_rptr: *mut c_uchar,
    /// The first unwritten byte of the data buffer.
    pub b_wptr: *mut c_uchar,
    /// The data block.
    pub b_datap: *mut datab,
    /// The priority band of the message; 0 for ordinary messages.
    pub b_band: c_uchar,
    /// Private to the framework.
    pub b_tag: c_uchar,
    /// Flags used by the stream head, a combination of `MSGMARK`,
    /// `MSGNOLOOP` and `MSGDELIM`.
    pub b_flag: c_ushort,
    /// Private to the framework. The queue a message on a synchronisation
    /// queue is destined for.
    pub b_queue: *mut queue,
}
/// The typedef name of [`msgb`].
pub type mblk_t = msgb;

// msgb: six pointers (48), b_band 48, b_tag 49, b_flag 50 padded to 56,
// b_queue 56; total 64.
const _: () = assert!(core::mem::size_of::<msgb>() == 64);
const _: () = assert!(core::mem::offset_of!(msgb, b_next) == 0);
const _: () = assert!(core::mem::offset_of!(msgb, b_datap) == 40);
const _: () = assert!(core::mem::offset_of!(msgb, b_band) == 48);
const _: () = assert!(core::mem::offset_of!(msgb, b_flag) == 50);
const _: () = assert!(core::mem::offset_of!(msgb, b_queue) == 56);

/// The last byte of the message is marked (`MSGMARK`).
pub const MSGMARK: c_ushort = 0x01;
/// The message must not be looped around to the write side of the stream
/// (`MSGNOLOOP`).
pub const MSGNOLOOP: c_ushort = 0x02;
/// The message is delimited (`MSGDELIM`).
pub const MSGDELIM: c_ushort = 0x04;

// --- message types -----------------------------------------------------------

/// Regular data (`M_DATA`).
pub const M_DATA: c_uchar = 0x00;
/// Protocol control information (`M_PROTO`).
pub const M_PROTO: c_uchar = 0x01;
/// A line break (`M_BREAK`).
pub const M_BREAK: c_uchar = 0x08;
/// Passes a file pointer (`M_PASSFP`).
pub const M_PASSFP: c_uchar = 0x09;
/// Generates a process signal (`M_SIG`).
pub const M_SIG: c_uchar = 0x0b;
/// A real-time transmit delay with a one-byte parameter (`M_DELAY`).
pub const M_DELAY: c_uchar = 0x0c;
/// A device-specific control message (`M_CTL`).
pub const M_CTL: c_uchar = 0x0d;
/// An ioctl request carrying an `iocblk(9S)` (`M_IOCTL`).
pub const M_IOCTL: c_uchar = 0x0e;
/// Sets stream head options with a `stroptions(9S)` (`M_SETOPTS`).
pub const M_SETOPTS: c_uchar = 0x10;
/// Reserved for RSE use (`M_RSE`).
pub const M_RSE: c_uchar = 0x11;
/// The class of normal priority messages (`QNORM`).
pub const QNORM: c_uchar = 0x00;
/// The class of high priority control messages, which go to the head of a
/// queue; every type at or above this value is high priority (`QPCTL`).
pub const QPCTL: c_uchar = 0x80;
/// A positive ioctl acknowledgement (`M_IOCACK`).
pub const M_IOCACK: c_uchar = 0x81;
/// A negative ioctl acknowledgement (`M_IOCNAK`).
pub const M_IOCNAK: c_uchar = 0x82;
/// High priority protocol control information (`M_PCPROTO`).
pub const M_PCPROTO: c_uchar = 0x83;
/// Generates a process signal at high priority (`M_PCSIG`).
pub const M_PCSIG: c_uchar = 0x84;
/// A read notification (`M_READ`).
pub const M_READ: c_uchar = 0x85;
/// Flush the queues; the one-byte payload holds the `FLUSH*` flags
/// (`M_FLUSH`).
pub const M_FLUSH: c_uchar = 0x86;
/// Stop transmission immediately (`M_STOP`).
pub const M_STOP: c_uchar = 0x87;
/// Restart transmission after a stop (`M_START`).
pub const M_START: c_uchar = 0x88;
/// The line has been disconnected (`M_HANGUP`).
pub const M_HANGUP: c_uchar = 0x89;
/// Sends an error to the stream head (`M_ERROR`).
pub const M_ERROR: c_uchar = 0x8a;
/// A request to copy data in from the user, carrying a `copyreq(9S)`
/// (`M_COPYIN`).
pub const M_COPYIN: c_uchar = 0x8b;
/// A request to copy data out to the user, carrying a `copyreq(9S)`
/// (`M_COPYOUT`).
pub const M_COPYOUT: c_uchar = 0x8c;
/// The response to `M_COPYIN` or `M_COPYOUT`, carrying a `copyresp(9S)`
/// (`M_IOCDATA`).
pub const M_IOCDATA: c_uchar = 0x8d;
/// Reserved for RSE use at high priority (`M_PCRSE`).
pub const M_PCRSE: c_uchar = 0x8e;
/// Stop reception immediately (`M_STOPI`).
pub const M_STOPI: c_uchar = 0x8f;
/// Restart reception after a stop (`M_STARTI`).
pub const M_STARTI: c_uchar = 0x90;
/// The line has been reconnected (`M_UNHANGUP`).
pub const M_UNHANGUP: c_uchar = 0x92;
/// An out-of-band ioctl command (`M_CMD`).
pub const M_CMD: c_uchar = 0x93;

/// For two-byte `M_ERROR` messages, indicates that a side has no error
/// (`NOERROR`).
pub const NOERROR: c_uchar = 0xff;

// --- ioctl messages -------------------------------------------------------------

/// `iocblk(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// The payload of an `M_IOCTL` message. The stream head fills it in from
/// the `ioctl(2)` call and the driver answers by converting the message
/// into an `M_IOCACK` or `M_IOCNAK`, usually with `miocack(9F)` or
/// `miocnak(9F)`. The layout shown is the LP64 one. The manual page shows
/// `ioc_count` as a `uint_t`; the header declares it as a `size_t`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct iocblk {
    /// The ioctl command.
    pub ioc_cmd: c_int,
    /// The full credentials of the caller.
    pub ioc_cr: *mut cred_t,
    /// A unique identifier of this ioctl, echoed in the acknowledgement.
    pub ioc_id: uint_t,
    /// The data model of the caller, one of the `IOC_*` values.
    pub ioc_flag: uint_t,
    /// The number of bytes in the data field, or `TRANSPARENT` when the
    /// ioctl was issued without `I_STR`.
    pub ioc_count: size_t,
    /// The return value delivered to the caller by an `M_IOCACK`.
    pub ioc_rval: c_int,
    /// The error code delivered to the caller by an `M_IOCNAK`.
    pub ioc_error: c_int,
}
/// A pointer to an [`iocblk`], `IOCP`.
pub type IOCP = *mut iocblk;

// iocblk (LP64): ioc_cmd 0 padded to 8, ioc_cr 8, ioc_id 16, ioc_flag 20,
// ioc_count 24, ioc_rval 32, ioc_error 36; total 40.
const _: () = assert!(core::mem::size_of::<iocblk>() == 40);
const _: () = assert!(core::mem::offset_of!(iocblk, ioc_cmd) == 0);
const _: () = assert!(core::mem::offset_of!(iocblk, ioc_cr) == 8);
const _: () = assert!(core::mem::offset_of!(iocblk, ioc_flag) == 20);
const _: () = assert!(core::mem::offset_of!(iocblk, ioc_count) == 24);
const _: () = assert!(core::mem::offset_of!(iocblk, ioc_error) == 36);

/// The bits of `ioc_flag` and `cp_flag` that carry the data model of the
/// ioctl caller (`IOC_MODELS`). The same value as `DDI_MODEL_MASK` in
/// [`crate::model`].
pub const IOC_MODELS: uint_t = 0x0FF00000;
/// The ioctl originated from a 32-bit process (`IOC_ILP32`).
pub const IOC_ILP32: uint_t = 0x00100000;
/// The ioctl originated from a 64-bit process (`IOC_LP64`).
pub const IOC_LP64: uint_t = 0x00200000;
/// The ioctl uses the kernel's own data model, which is LP64 on amd64
/// (`IOC_NATIVE`).
pub const IOC_NATIVE: uint_t = IOC_LP64;
/// A dummy comparison value meaning no model information (`IOC_NONE`).
pub const IOC_NONE: uint_t = 0;

/// The value of `ioc_count` that marks a transparent ioctl (`TRANSPARENT`).
/// The header defines it as `(unsigned int)(-1)`, so it is 32 bits wide and
/// a `size_t` field compares equal to it only after the usual widening.
pub const TRANSPARENT: uint_t = u32::MAX;

/// `copyreq(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// The payload of the `M_COPYIN` and `M_COPYOUT` message types, which a
/// driver sends upstream to ask the stream head to move data between the
/// user's address space and the message. It is designed to overlay
/// `iocblk(9S)`, so `mcopyin(9F)` and `mcopyout(9F)` convert an `M_IOCTL`
/// in place. The layout shown is the LP64 one.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct copyreq {
    /// The ioctl command, copied from `ioc_cmd`.
    pub cq_cmd: c_int,
    /// The full credentials, copied from `ioc_cr`.
    pub cq_cr: *mut cred_t,
    /// The ioctl identifier, copied from `ioc_id`.
    pub cq_id: uint_t,
    /// Must be zero.
    pub cq_flag: uint_t,
    /// Private state information the driver stores to recognise the
    /// `M_IOCDATA` reply.
    pub cq_private: *mut msgb,
    /// The user address to copy data to or from.
    pub cq_addr: caddr_t,
    /// The number of bytes to copy.
    pub cq_size: size_t,
}

// copyreq (LP64): cq_cmd 0 padded to 8, cq_cr 8, cq_id 16, cq_flag 20,
// cq_private 24, cq_addr 32, cq_size 40; total 48.
const _: () = assert!(core::mem::size_of::<copyreq>() == 48);
const _: () = assert!(core::mem::offset_of!(copyreq, cq_cmd) == 0);
const _: () = assert!(core::mem::offset_of!(copyreq, cq_cr) == 8);
const _: () = assert!(core::mem::offset_of!(copyreq, cq_private) == 24);
const _: () = assert!(core::mem::offset_of!(copyreq, cq_size) == 40);

/// `copyresp(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// The payload of the `M_IOCDATA` message type, which the stream head sends
/// back to the driver after acting on an `M_COPYIN` or `M_COPYOUT`. The
/// `cp_rval` field is 0 when the copy succeeded and non-zero when it
/// failed. The layout shown is the LP64 one.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct copyresp {
    /// The ioctl command, copied from `ioc_cmd`.
    pub cp_cmd: c_int,
    /// The full credentials, copied from `ioc_cr`.
    pub cp_cr: *mut cred_t,
    /// The ioctl identifier, copied from `ioc_id`.
    pub cp_id: uint_t,
    /// The data model of the caller, one of the `IOC_*` values.
    pub cp_flag: uint_t,
    /// The private state information the driver stored in `cq_private`.
    pub cp_private: *mut msgb,
    /// The status of the request, 0 for success and non-zero for failure.
    pub cp_rval: caddr_t,
}

// copyresp (LP64): cp_cmd 0 padded to 8, cp_cr 8, cp_id 16, cp_flag 20,
// cp_private 24, cp_rval 32; total 40.
const _: () = assert!(core::mem::size_of::<copyresp>() == 40);
const _: () = assert!(core::mem::offset_of!(copyresp, cp_cmd) == 0);
const _: () = assert!(core::mem::offset_of!(copyresp, cp_cr) == 8);
const _: () = assert!(core::mem::offset_of!(copyresp, cp_private) == 24);
const _: () = assert!(core::mem::offset_of!(copyresp, cp_rval) == 32);

// --- stroptions(9S) --------------------------------------------------------------

/// `stroptions(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// The payload of an `M_SETOPTS` message, which a module or driver sends
/// upstream to change options in the stream head. `so_flags` names the
/// options being set, and only the fields the flags select are examined.
/// The manual page documents the fields up to `so_erropt`; the header adds
/// `so_maxblk`, `so_copyopt` and `so_tail`, which are selected by
/// `SO_MAXBLK`, `SO_COPYOPT` and `SO_TAIL`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct stroptions {
    /// The options to set, a combination of the `SO_*` flags.
    pub so_flags: uint_t,
    /// The read option, one of `RNORM`, `RMSGD` or `RMSGN`, when
    /// `SO_READOPT` is set.
    pub so_readopt: c_short,
    /// The write offset the stream head leaves at the front of each
    /// message, when `SO_WROFF` is set.
    pub so_wroff: c_ushort,
    /// The minimum read packet size, when `SO_MINPSZ` is set.
    pub so_minpsz: ssize_t,
    /// The maximum read packet size, when `SO_MAXPSZ` is set.
    pub so_maxpsz: ssize_t,
    /// The read queue high water mark, when `SO_HIWAT` is set.
    pub so_hiwat: size_t,
    /// The read queue low water mark, when `SO_LOWAT` is set.
    pub so_lowat: size_t,
    /// The band the water marks apply to, when `SO_BAND` is set.
    pub so_band: c_uchar,
    /// The error option, a combination of one `RERR*` and one `WERR*`
    /// value, when `SO_ERROPT` is set.
    pub so_erropt: c_ushort,
    /// The maximum message block size, when `SO_MAXBLK` is set.
    pub so_maxblk: ssize_t,
    /// The copy options from `sys/stropts.h`, when `SO_COPYOPT` is set.
    pub so_copyopt: c_ushort,
    /// The space to leave at the end of each message, when `SO_TAIL` is set.
    pub so_tail: c_ushort,
}

// stroptions: so_flags 0, so_readopt 4, so_wroff 6, four size fields 8 to
// 40, so_band 40, so_erropt 42, so_maxblk 48, so_copyopt 56, so_tail 58
// padded to 64.
const _: () = assert!(core::mem::size_of::<stroptions>() == 64);
const _: () = assert!(core::mem::offset_of!(stroptions, so_flags) == 0);
const _: () = assert!(core::mem::offset_of!(stroptions, so_readopt) == 4);
const _: () = assert!(core::mem::offset_of!(stroptions, so_minpsz) == 8);
const _: () = assert!(core::mem::offset_of!(stroptions, so_band) == 40);
const _: () = assert!(core::mem::offset_of!(stroptions, so_erropt) == 42);
const _: () = assert!(core::mem::offset_of!(stroptions, so_maxblk) == 48);
const _: () = assert!(core::mem::offset_of!(stroptions, so_tail) == 58);

/// Set every one of the six original options (`SO_ALL`).
pub const SO_ALL: uint_t = 0x003f;
/// Set the read option from `so_readopt` (`SO_READOPT`).
pub const SO_READOPT: uint_t = 0x0001;
/// Set the write offset from `so_wroff` (`SO_WROFF`).
pub const SO_WROFF: uint_t = 0x0002;
/// Set the minimum packet size from `so_minpsz` (`SO_MINPSZ`).
pub const SO_MINPSZ: uint_t = 0x0004;
/// Set the maximum packet size from `so_maxpsz` (`SO_MAXPSZ`).
pub const SO_MAXPSZ: uint_t = 0x0008;
/// Set the high water mark from `so_hiwat` (`SO_HIWAT`).
pub const SO_HIWAT: uint_t = 0x0010;
/// Set the low water mark from `so_lowat` (`SO_LOWAT`).
pub const SO_LOWAT: uint_t = 0x0020;
/// Turn read notification on, so that the stream head sends `M_READ`
/// messages downstream (`SO_MREADON`).
pub const SO_MREADON: uint_t = 0x0040;
/// Turn read notification off (`SO_MREADOFF`).
pub const SO_MREADOFF: uint_t = 0x0080;
/// Use the old TTY semantics for non-blocking reads and writes
/// (`SO_NDELON`).
pub const SO_NDELON: uint_t = 0x0100;
/// Use the STREAMS semantics for non-blocking reads and writes
/// (`SO_NDELOFF`).
pub const SO_NDELOFF: uint_t = 0x0200;
/// The stream is acting as a terminal (`SO_ISTTY`).
pub const SO_ISTTY: uint_t = 0x0400;
/// The stream is not acting as a terminal (`SO_ISNTTY`).
pub const SO_ISNTTY: uint_t = 0x0800;
/// Stop background writes to this stream (`SO_TOSTOP`).
pub const SO_TOSTOP: uint_t = 0x1000;
/// Do not stop background writes to this stream (`SO_TONSTOP`).
pub const SO_TONSTOP: uint_t = 0x2000;
/// The water marks apply to the band in `so_band` (`SO_BAND`).
pub const SO_BAND: uint_t = 0x4000;
/// Messages are delimited (`SO_DELIM`).
pub const SO_DELIM: uint_t = 0x8000;
/// Turn delimiters off (`SO_NODELIM`).
pub const SO_NODELIM: uint_t = 0x010000;
/// Set the error option from `so_erropt` (`SO_ERROPT`).
pub const SO_ERROPT: uint_t = 0x040000;
/// Copy options are present in `so_copyopt` (`SO_COPYOPT`).
pub const SO_COPYOPT: uint_t = 0x080000;
/// Set the maximum message block size from `so_maxblk` (`SO_MAXBLK`).
pub const SO_MAXBLK: uint_t = 0x100000;
/// Set the extra space allocated at the end of messages from `so_tail`
/// (`SO_TAIL`).
pub const SO_TAIL: uint_t = 0x200000;

/// Read messages normally, as a byte stream (`RNORM`).
pub const RNORM: c_short = 0x000;
/// Read one message at a time and discard what does not fit (`RMSGD`).
pub const RMSGD: c_short = 0x001;
/// Read one message at a time and keep what does not fit (`RMSGN`).
pub const RMSGN: c_short = 0x002;
/// The mask of the read mode bits (`RMODEMASK`).
pub const RMODEMASK: c_short = 0x003;
/// Deliver protocol messages as data (`RPROTDAT`).
pub const RPROTDAT: c_short = 0x004;
/// Discard protocol messages but deliver their data (`RPROTDIS`).
pub const RPROTDIS: c_short = 0x008;
/// Fail reads that would return a protocol message, which is the default
/// (`RPROTNORM`).
pub const RPROTNORM: c_short = 0x010;
/// The mask of the protocol handling bits (`RPROTMASK`).
pub const RPROTMASK: c_short = 0x01c;
/// Do not flush `M_PCPROTO` messages (`RFLUSHPCPROT`).
pub const RFLUSHPCPROT: c_short = 0x020;
/// The mask of the flush handling bits (`RFLUSHMASK`).
pub const RFLUSHMASK: c_short = 0x020;

/// Read errors are persistent, which is the default (`RERRNORM`).
pub const RERRNORM: c_ushort = 0x001;
/// Read errors are reported once and cleared (`RERRNONPERSIST`).
pub const RERRNONPERSIST: c_ushort = 0x002;
/// The mask of the read error bits (`RERRMASK`).
pub const RERRMASK: c_ushort = RERRNORM | RERRNONPERSIST;
/// Write errors are persistent, which is the default (`WERRNORM`).
pub const WERRNORM: c_ushort = 0x004;
/// Write errors are reported once and cleared (`WERRNONPERSIST`).
pub const WERRNONPERSIST: c_ushort = 0x008;
/// The mask of the write error bits (`WERRMASK`).
pub const WERRMASK: c_ushort = WERRNORM | WERRNONPERSIST;

/// It is safe to borrow mapped file pages for zero-copy writes
/// (`ZCVMSAFE`).
pub const ZCVMSAFE: c_ushort = 0x01;
/// It is unsafe to borrow mapped file pages (`ZCVMUNSAFE`).
pub const ZCVMUNSAFE: c_ushort = 0x02;
/// Copies should not bypass the cache (`COPYCACHED`).
pub const COPYCACHED: c_ushort = 0x04;

// --- flush, open and allocation constants ----------------------------------------

/// Flush the read queue (`FLUSHR`). Carried in the payload of an `M_FLUSH`
/// message.
pub const FLUSHR: c_int = 0x01;
/// Flush the write queue (`FLUSHW`).
pub const FLUSHW: c_int = 0x02;
/// Flush both queues (`FLUSHRW`).
pub const FLUSHRW: c_int = 0x03;
/// Flush only the band given in the second payload byte (`FLUSHBAND`).
pub const FLUSHBAND: c_int = 0x04;

/// The `flag` argument of `flushq(9F)` and `flushband(9F)` that flushes
/// every message (`FLUSHALL`).
pub const FLUSHALL: c_int = 1;
/// The `flag` argument of `flushq(9F)` and `flushband(9F)` that flushes
/// only data messages, leaving control messages on the queue
/// (`FLUSHDATA`).
pub const FLUSHDATA: c_int = 0;

/// The stream flag passed to a STREAMS `open(9E)` when the entity is
/// being opened as a module (`MODOPEN`).
pub const MODOPEN: c_int = 0x1;
/// The stream flag passed to a STREAMS `open(9E)` when the driver should
/// pick its own minor number (`CLONEOPEN`).
pub const CLONEOPEN: c_int = 0x2;
/// The historical failure return of a STREAMS open (`OPENFAIL`). Current
/// drivers return an errno instead.
pub const OPENFAIL: c_int = -1;

/// Low priority for a block allocation (`BPRI_LO`). The priority is no
/// longer used but is still passed for compatibility.
pub const BPRI_LO: uint_t = 1;
/// Medium priority for a block allocation (`BPRI_MED`).
pub const BPRI_MED: uint_t = 2;
/// High priority for a block allocation (`BPRI_HI`).
pub const BPRI_HI: uint_t = 3;

/// The packet size value that denotes infinity (`INFPSZ`), used in
/// `q_maxpsz` and `mi_maxpsz`.
pub const INFPSZ: ssize_t = -1;

/// The default high water mark of the stream head (`STRHIGH`).
pub const STRHIGH: size_t = 5120;
/// The default low water mark of the stream head (`STRLOW`).
pub const STRLOW: size_t = 1024;

/// The inner perimeter, for `qwriter(9F)` (`PERIM_INNER`).
pub const PERIM_INNER: c_int = 1;
/// The outer perimeter, for `qwriter(9F)` (`PERIM_OUTER`).
pub const PERIM_OUTER: c_int = 2;

// --- strlog(9F) flags (sys/strlog.h) -------------------------------------------

/// The message reports a fatal error (`SL_FATAL`).
pub const SL_FATAL: c_ushort = 0x01;
/// The logger must notify the administrator (`SL_NOTIFY`).
pub const SL_NOTIFY: c_ushort = 0x02;
/// Include the message in the error log (`SL_ERROR`).
pub const SL_ERROR: c_ushort = 0x04;
/// Include the message in the trace log (`SL_TRACE`).
pub const SL_TRACE: c_ushort = 0x08;
/// Include the message in the console log (`SL_CONSOLE`).
pub const SL_CONSOLE: c_ushort = 0x10;
/// The message is a warning (`SL_WARN`).
pub const SL_WARN: c_ushort = 0x20;
/// The message is a notice (`SL_NOTE`).
pub const SL_NOTE: c_ushort = 0x40;
/// Send the message only to the console (`SL_CONSONLY`).
pub const SL_CONSONLY: c_ushort = 0x1000;
/// Send the message only to the system log (`SL_LOGONLY`).
pub const SL_LOGONLY: c_ushort = 0x2000;
/// Send the message to the user's terminal (`SL_USER`).
pub const SL_USER: c_ushort = 0x4000;
/// The message was created while panicking (`SL_PANICMSG`).
pub const SL_PANICMSG: c_ushort = 0x8000;

// --- functions ------------------------------------------------------------------

/// The signature of a `bufcall(9F)`, `esbbcall(9F)` or `qbufcall(9F)`
/// callback, and of a `qtimeout(9F)` callback.
pub type bufcall_fn_t = unsafe extern "C" fn(arg: *mut c_void);
/// The signature of a `qwriter(9F)` callback. The header declares the
/// argument without a prototype; the manual page states that the function
/// is called with the queue and the message.
pub type qwriter_fn_t = unsafe extern "C" fn(qp: *mut queue, mp: *mut msgb);

unsafe extern "C" {
    // --- queue navigation (sys/ddi.h, os/ddi.c) -----------------------------
    /// `OTHERQ(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns a pointer to the other of the two queue structures that make
    /// up a STREAMS module or driver. If `q` points to the read queue the
    /// write queue is returned, and vice versa. It can be called from user,
    /// interrupt or kernel context.
    pub fn OTHERQ(q: *mut queue) -> *mut queue;
    /// `RD(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Accepts a write queue pointer and returns a pointer to the read
    /// queue of the same module. The function does not check the queue
    /// type, and a system panic can result if the argument is not a write
    /// queue. It can be called from user, interrupt or kernel context.
    pub fn RD(q: *mut queue) -> *mut queue;
    /// `WR(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Accepts a read queue pointer and returns a pointer to the write
    /// queue of the same module. The function does not check the queue
    /// type, and a system panic can result if the argument is not a read
    /// queue. It can be called from user, interrupt or kernel context.
    pub fn WR(q: *mut queue) -> *mut queue;
    /// `SAMESTR(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Tells whether the next queue in the stream, if it exists, is of the
    /// same type as `q`, that is both are read queues or both are write
    /// queues. It accounts for the twisted queue connections of a STREAMS
    /// pipe and should be used instead of examining `q_next` directly to
    /// see whether the stream continues beyond `q`. Returns 1 if the next
    /// queue is of the same type and 0 if it does not exist or is not. It
    /// can be called from user or interrupt context.
    pub fn SAMESTR(q: *mut queue) -> c_int;
    /// `datamsg(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Tests a message type, as found in the `db_type` field of the data
    /// block, to determine whether it is one of the data message types
    /// `M_DATA`, `M_DELAY`, `M_PROTO` and `M_PCPROTO`. Returns 1 if it is
    /// and 0 otherwise. It can be called from user, interrupt or kernel
    /// context.
    pub fn datamsg(db_type: c_uchar) -> c_int;
    /// `backq(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns a pointer to the queue preceding `cq`. If `cq` is a read
    /// queue, the queue downstream of it is returned unless `cq` is the
    /// stream end; if `cq` is a write queue, the queue upstream of it is
    /// returned unless `cq` is the stream head. Returns NULL when there is
    /// no such queue. It can be called from user, interrupt or kernel
    /// context.
    pub fn backq(cq: *mut queue) -> *mut queue;

    // --- message allocation and release -----------------------------------
    /// `allocb(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Tries to allocate a STREAMS message block of `size` bytes. The
    /// result is a message block whose `b_rptr` and `b_wptr` both point at
    /// the start of a data buffer of at least the requested size, whose
    /// `b_cont` is NULL, and whose data block has type `M_DATA`. The buffer
    /// is at least double-word aligned. The `pri` argument is no longer
    /// used. Allocation fails only when the system is out of memory; when
    /// it does, `bufcall(9F)` can help a module recover. Returns the new
    /// block, or NULL on failure. It can be called from user, interrupt or
    /// kernel context.
    pub fn allocb(size: size_t, pri: uint_t) -> *mut msgb;
    /// `esballoc(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Operates like `allocb(9F)`, except that the data buffer of `size`
    /// bytes at `base` is supplied by the caller. The message block and
    /// data block header are allocated as if by `allocb(9F)`, and `b_rptr`
    /// and `b_wptr` are set to `base`. When `freeb(9F)` releases the last
    /// reference, the `free_func` of the `free_rtn(9S)` at `fr_rtnp` is
    /// called asynchronously with `free_arg` to release the buffer. The
    /// `free_rtn` must remain valid until then, and `free_func` must not
    /// sleep. Returns the new block, or NULL on failure. It can be called
    /// from user, interrupt or kernel context.
    pub fn esballoc(base: *mut c_uchar, size: size_t, pri: uint_t, fr_rtnp: *mut free_rtn) -> *mut msgb;
    /// `desballoc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Documented on the `esballoc(9F)` page and listed in `ddi.mapfile`.
    /// It behaves like `esballoc(9F)` except that `free_func` is called
    /// synchronously by the thread that releases the final reference.
    /// Because of that, `free_func` must not call another module's put
    /// procedure, must not acquire a lock that might be held across a
    /// STREAMS call that could free a message, and may run in interrupt
    /// context, so it may only use synchronisation primitives initialised
    /// with an interrupt priority. Returns the new block, or NULL on
    /// failure. It can be called from user, interrupt or kernel context.
    pub fn desballoc(base: *mut c_uchar, size: size_t, pri: uint_t, fr_rtnp: *mut free_rtn) -> *mut msgb;
    /// `esballoc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like `esballoc(9F)`, but the caller's buffer is only freed when
    /// the whole message chain it belongs to is gone, which the manual calls
    /// an "a" variant. The prototype lives in the private `sys/strsubr.h`;
    /// the function is bound because the manual page documents it at the
    /// illumos DDI level and `genunix` exports it.
    pub fn esballoca(base: *mut c_uchar, size: size_t, pri: uint_t, fr_rtnp: *mut free_rtn) -> *mut msgb;
    /// `esballoc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The synchronous-free counterpart of [`esballoca`], in the same way
    /// that [`desballoc`] relates to [`esballoc`]. Like `esballoca`, its
    /// prototype lives in `sys/strsubr.h`.
    pub fn desballoca(base: *mut c_uchar, size: size_t, pri: uint_t, fr_rtnp: *mut free_rtn) -> *mut msgb;
    /// `mkiocb(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates an `M_IOCTL` message for a module or driver that needs to
    /// issue an ioctl to a lower module or driver. The block is large
    /// enough to hold an `iocblk(9S)`, `copyreq(9S)` or `copyresp(9S)`;
    /// its `b_wptr` is set past the `iocblk`, `b_cont` is NULL and the
    /// type is `M_IOCTL`. In the `iocblk`, `ioc_cmd` is set to `command`,
    /// `ioc_id` to a unique identifier, `ioc_cr` to a credential carrying
    /// maximum privilege that need not be freed, `ioc_flag` to
    /// `IOC_NATIVE`, and `ioc_count`, `ioc_rval` and `ioc_error` to 0.
    /// Returns the block, or NULL when memory is exhausted, in which case
    /// `qbufcall(9F)` can help the module recover. It can be called from
    /// user, interrupt or kernel context.
    pub fn mkiocb(command: uint_t) -> *mut msgb;
    /// `testb(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Checks whether an `allocb(9F)` call for `size` bytes is likely to
    /// succeed. Even when it returns 1 the allocation can still fail. The
    /// `pri` argument is no longer used. Returns 1 if a buffer of the
    /// requested size is available and 0 otherwise. It can be called from
    /// user, interrupt or kernel context.
    pub fn testb(size: size_t, pri: uint_t) -> c_int;
    /// `bufcall(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Serves as a `timeout(9F)` of indeterminate length. When a buffer
    /// allocation fails, `bufcall` schedules `func` to be called with `arg`
    /// once a buffer of `size` bytes becomes available. The `pri` argument
    /// is not used. Returns a bufcall identifier that can be passed to
    /// `unbufcall(9F)`, or 0 if the scheduling failed, in which case `func`
    /// is never called. It can be called from user, interrupt or kernel
    /// context.
    pub fn bufcall(size: size_t, pri: uint_t, func: bufcall_fn_t, arg: *mut c_void) -> bufcall_id_t;
    /// `esbbcall(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Like `bufcall(9F)`, but for `esballoc(9F)`. When `esballoc` cannot
    /// allocate the message and data block headers for an externally
    /// supplied buffer, `esbbcall` schedules `func` to be called with `arg`
    /// when a buffer becomes available. Returns a bufcall identifier, or 0
    /// on failure. The identifier should be saved so that the request can
    /// be cancelled with `unbufcall(9F)` at close time. It can be called
    /// from user, interrupt or kernel context.
    pub fn esbbcall(pri: uint_t, func: bufcall_fn_t, arg: *mut c_void) -> bufcall_id_t;
    /// `unbufcall(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Cancels a pending `bufcall(9F)` or `esbbcall(9F)` request identified
    /// by the non-zero `id`. It does not return until the callback has
    /// been cancelled or has run, so locks acquired by the callback must
    /// not be held across the call. It can be called from user, interrupt
    /// or kernel context.
    pub fn unbufcall(id: bufcall_id_t);
    /// `freeb(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Deallocates a single message block. If `db_ref` of the data block is
    /// greater than 1 it is decremented; if it equals 1 the message block,
    /// the data block and the buffer are freed. For a buffer supplied with
    /// `esballoc(9F)` the driver's `free_rtn(9S)` routine is run to release
    /// it. It can be called from user, interrupt or kernel context.
    pub fn freeb(bp: *mut msgb);
    /// `freemsg(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Calls `freeb(9F)` for every block of the message `mp`, following
    /// the `b_cont` chain. If `mp` is NULL it returns immediately. It can
    /// be called from user, interrupt or kernel context.
    pub fn freemsg(mp: *mut msgb);
    /// `freemsgchain`. Listed in `ddi.mapfile`; no manual page.
    ///
    /// Frees a chain of messages linked through `b_next`, calling
    /// `freemsg(9F)` on each. The prototype is in `sys/strsubr.h`, but the
    /// symbol is part of the DDI mapfile, which is why it is bound.
    pub fn freemsgchain(mp: *mut msgb);
    /// `dupb(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Creates a new `msgb(9S)` that references the same data block as
    /// `bp`, incrementing `db_ref`, and copies `b_rptr` and `b_wptr` from
    /// `bp`. The data itself is not copied. Returns the new block, or NULL
    /// if a message block cannot be allocated or `db_ref` has reached its
    /// maximum of 255. It can be called from user, kernel or interrupt
    /// context.
    pub fn dupb(bp: *mut msgb) -> *mut msgb;
    /// `dupmsg(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Forms a new message by calling `dupb(9F)` on every block of `mp`
    /// and linking the results. The data blocks are not duplicated.
    /// Returns the new message, or NULL on memory depletion or when a data
    /// block's reference count has reached its limit. It can be called
    /// from user, kernel or interrupt context.
    pub fn dupmsg(mp: *mut msgb) -> *mut msgb;
    /// `copyb(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Allocates a new message block and copies into it the data between
    /// `b_rptr` and `b_wptr` of `bp`. The new block is at least as large
    /// as the one copied. Returns the new block, or NULL on failure. It can
    /// be called from user, interrupt or kernel context.
    pub fn copyb(bp: *mut msgb) -> *mut msgb;
    /// `copymsg(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Forms a new message by allocating new blocks and copying the
    /// contents of every block of `mp` with `copyb(9F)`. Returns the new
    /// message, or NULL on failure. It can be called from user, interrupt
    /// or kernel context.
    pub fn copymsg(mp: *mut msgb) -> *mut msgb;

    // --- message manipulation ----------------------------------------------
    /// `linkb(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Adds the message `mp2` to the tail of `mp1` by setting the `b_cont`
    /// pointer of the last block of `mp1` to `mp2`. It can be called from
    /// user, interrupt or kernel context.
    pub fn linkb(mp1: *mut msgb, mp2: *mut msgb);
    /// `unlinkb(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Removes the first message block from the message `mp` and returns
    /// the remainder. Returns NULL if the message had only one block. The
    /// removed block is not freed. It can be called from user, interrupt
    /// or kernel context.
    pub fn unlinkb(mp: *mut msgb) -> *mut msgb;
    /// `rmvb(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Removes the block `bp` from the message `mp` and returns the
    /// altered message. The block is not freed; the caller must free it.
    /// Returns NULL if `bp` was the only block of the message, and the
    /// value -1 cast to a pointer if `bp` is not part of `mp`. It can be
    /// called from user, interrupt or kernel context.
    pub fn rmvb(mp: *mut msgb, bp: *mut msgb) -> *mut msgb;
    /// `pullupmsg(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Concatenates and aligns the first `len` data bytes of `mp` into a
    /// single block, in place. If `len` is -1 all the data is concatenated.
    /// It fails if `len` bytes of the same message type cannot be found.
    /// Returns 1 on success and 0 on failure. It can be called from user,
    /// interrupt or kernel context.
    pub fn pullupmsg(mp: *mut msgb, len: ssize_t) -> c_int;
    /// `msgpullup(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Concatenates and aligns the first `len` data bytes of `mp` into a
    /// new message, leaving the original intact. If `len` is -1 all the
    /// data is copied. The remainder of the original message, if any, is
    /// duplicated and linked after the new block. Returns the new message,
    /// or NULL if `len` bytes of the same type are not present or memory
    /// is exhausted. It can be called from user, interrupt or kernel
    /// context.
    pub fn msgpullup(mp: *mut msgb, len: ssize_t) -> *mut msgb;
    /// `msgpullup_pad(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Like `msgpullup(9F)`, but inserts exactly `pad` uninitialised bytes
    /// before the copied data so that the caller can set up the alignment
    /// of internal structures. Documented on the `msgpullup(9F)` page and
    /// listed in `ddi.mapfile`. Returns the new message, or NULL on failure.
    pub fn msgpullup_pad(mp: *mut msgb, len: ssize_t, pad: size_t) -> *mut msgb;
    /// `adjmsg(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Removes bytes from a message. The absolute value of `len` gives the
    /// number of bytes; a positive `len` removes them from the start of the
    /// maximal leading sequence of blocks of the same type as the first
    /// block, and a negative `len` removes them from the end of that
    /// sequence. It fails if that sequence holds fewer bytes than
    /// requested. Zero-length blocks created by the adjustment, other than
    /// the first, may be removed. Returns 1 on success and 0 on failure. It
    /// can be called from user, interrupt or kernel context.
    pub fn adjmsg(mp: *mut msgb, len: ssize_t) -> c_int;
    /// `msgdsize(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Counts the number of data bytes in a message, including only the
    /// bytes of blocks whose type is `M_DATA`. It can be called from user,
    /// interrupt or kernel context.
    pub fn msgdsize(mp: *mut msgb) -> size_t;
    /// `msgsize(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Counts the number of bytes in a message regardless of the type of
    /// the underlying data blocks. It can be called from user, kernel or
    /// interrupt context.
    pub fn msgsize(mp: *mut msgb) -> size_t;

    // --- queue operations ------------------------------------------------------
    /// `getq(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Retrieves the next message from the top of the queue `q`. It is
    /// used by a service routine, which must process every enqueued message
    /// unless flow control prevents it, so it is called in a loop that
    /// exits only when no message remains or flow control intervenes. If a
    /// writer was blocked by flow control, `getq` back-enables it once the
    /// queue falls below its low water mark. Returns the message, or NULL
    /// if the queue is empty. It can be called from user, interrupt or
    /// kernel context.
    pub fn getq(q: *mut queue) -> *mut msgb;
    /// `putq(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Puts the message `bp` on the queue `q` after any other messages of
    /// the same priority and updates the flow control parameters. Unless
    /// `QNOENB` is set, the service routine is enabled. It can serve as a
    /// module's put procedure when no other processing is needed. Returns
    /// 1 on success and 0 on failure, in which case the caller should free
    /// the message. It can be called from user, interrupt or kernel
    /// context.
    pub fn putq(q: *mut queue, bp: *mut msgb) -> c_int;
    /// `putbq(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Places the message `bp` at the beginning of the appropriate section
    /// of the queue, that is ahead of other messages of its priority band.
    /// It is usually called when `canput(9F)` or `bcanput(9F)` has found
    /// that the message cannot be passed on. Using it with a high priority
    /// message is likely to lead to an infinite loop unless precautions are
    /// taken. Flow control parameters are updated and, unless `QNOENB` is
    /// set, the service routine is enabled. Returns 1 on success and 0 on
    /// failure, in which case the caller should free the message. It can
    /// be called from user, interrupt or kernel context.
    pub fn putbq(q: *mut queue, bp: *mut msgb) -> c_int;
    /// `insq(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Inserts the message `nmp` into the queue `q` immediately before the
    /// enqueued message `emp`. If `emp` is NULL the new message is placed
    /// at the end of the queue. The queue class of the new message is
    /// ignored, flow control parameters are updated and the service
    /// procedure is enabled unless `QNOENB` is set. Returns 1 on success
    /// and 0 on failure. It can be called from user, interrupt or kernel
    /// context.
    pub fn insq(q: *mut queue, emp: *mut msgb, nmp: *mut msgb) -> c_int;
    /// `rmvq(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Removes the message `mp` from anywhere on the queue `q`. Together
    /// with `getq(9F)` it is the sanctioned way to take a message off a
    /// queue without touching the linkage directly. It can be called from
    /// user, interrupt or kernel context.
    pub fn rmvq(q: *mut queue, mp: *mut msgb);
    /// `flushq(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Frees messages on the queue with `freemsg(9F)`. With `FLUSHDATA`
    /// only data messages, of types `M_DATA`, `M_DELAY`, `M_PROTO` and
    /// `M_PCPROTO`, are flushed; with `FLUSHALL` every message is. If the
    /// queue count falls below the low water mark and the queue was
    /// blocking an upstream service procedure, that procedure is enabled.
    /// It can be called from user, interrupt or kernel context.
    pub fn flushq(q: *mut queue, flag: c_int);
    /// `flushband(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Flushes the messages of the priority band `pri` from the queue,
    /// according to `flag` as for `flushq(9F)`. If `pri` is 0 only normal
    /// and high priority messages are flushed. It can be called from user,
    /// interrupt or kernel context.
    pub fn flushband(q: *mut queue, pri: c_uchar, flag: c_int);
    /// `canput(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Searches the stream starting at `q` for a queue with a service
    /// routine where a message could be enqueued, or the end of the stream,
    /// and tests whether that queue has room for a message. Returns 1 if
    /// the queue is not full and 0 if it is. `canputnext(9F)` should
    /// always be used in preference to calling `canput` on `q_next`. It can
    /// be called from user or interrupt context.
    pub fn canput(q: *mut queue) -> c_int;
    /// `bcanput(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Like `canput(9F)`, but tests for room for a message of priority
    /// band `pri`. With `pri` equal to 0 it is equivalent to `canput`.
    /// Returns 1 if a message of that priority can be placed on the queue
    /// and 0 if the band is full. `bcanputnext(9F)` should always be used
    /// in preference to calling `bcanput` on `q_next`. It can be called
    /// from user, interrupt or kernel context.
    pub fn bcanput(q: *mut queue, pri: c_uchar) -> c_int;
    /// `canputnext(9F)`. Architecture independent (DDI/DDK).
    ///
    /// Atomically tests for available space in the next linked queue. It
    /// is preferable to and safer than calling `canput(9F)` on `q_next`,
    /// and drivers and modules should call it before `putnext(9F)`.
    /// Returns 1 if the next queue is not full and 0 if it is. It can be
    /// called from user, kernel or interrupt context.
    pub fn canputnext(q: *mut queue) -> c_int;
    /// `bcanputnext(9F)`. Architecture independent (DDI/DDK).
    ///
    /// Atomically tests for available space in the priority band `pri` of
    /// the next linked queue. Documented on the `canputnext(9F)` page.
    /// Returns 1 if the band is not full and 0 if it is. It can be called
    /// from user, kernel or interrupt context.
    pub fn bcanputnext(q: *mut queue, pri: c_uchar) -> c_int;
    /// `put(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Calls the `put(9E)` procedure of the queue `q` with the message
    /// `mp`. It is typically used by a driver or module to call its own put
    /// procedure, and is the DDI/DKI conforming replacement for a direct
    /// call. It can be called from a put or service routine, or from an
    /// associated interrupt handler, timeout, bufcall or esballoc
    /// callback, in which case the caller must guarantee that `q` is
    /// valid. Because it may re-enter the module, no mutex or other lock
    /// should be held across the call.
    pub fn put(q: *mut queue, mp: *mut msgb);
    /// `putnext(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Passes the message `mp` to the `put(9E)` routine of the queue that
    /// follows `q` in the stream. It can be called from user, interrupt or
    /// kernel context.
    pub fn putnext(q: *mut queue, mp: *mut msgb);
    /// `putctl(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Allocates a zero-length control message of the given type and
    /// passes it to the `put(9E)` routine of the queue `q`. It fails if
    /// `type` is one of the data types `M_DATA`, `M_PROTO` or `M_PCPROTO`,
    /// or if a message block cannot be allocated. Returns 1 on success and
    /// 0 on failure. It can be called from user, interrupt or kernel
    /// context.
    pub fn putctl(q: *mut queue, r#type: c_int) -> c_int;
    /// `putctl1(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Like `putctl(9F)`, but the message carries the one-byte parameter
    /// `p`, which can for example give the length of the delay in an
    /// `M_DELAY` message or the flush flags of an `M_FLUSH`. Returns 1 on
    /// success and 0 if `type` is a data type or a block cannot be
    /// allocated. It can be called from user, interrupt or kernel context.
    pub fn putctl1(q: *mut queue, r#type: c_int, p: c_int) -> c_int;
    /// `putnextctl(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// The atomic equivalent of calling `putctl(9F)` on `q_next`; the
    /// framework provides whatever mutual exclusion is necessary so that
    /// dereferencing `q_next` and invoking the put routine proceed without
    /// interference from other threads. It should always be used in
    /// preference to `putctl`. Returns 1 on success and 0 if `type` is a
    /// data type or a block cannot be allocated. It can be called from
    /// user, interrupt or kernel context.
    pub fn putnextctl(q: *mut queue, r#type: c_int) -> c_int;
    /// `putnextctl1(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// The atomic equivalent of calling `putctl1(9F)` on `q_next`, with
    /// the one-byte parameter `p`. It should always be used in preference
    /// to `putctl1`. Returns 1 on success and 0 if `type` is a data type or
    /// a block cannot be allocated. It can be called from user, interrupt
    /// or kernel context.
    pub fn putnextctl1(q: *mut queue, r#type: c_int, p: c_int) -> c_int;
    /// `qreply(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Sends the message `mp` in the reverse direction of normal flow; it
    /// is equivalent to `putnext(OTHERQ(q), mp)`. A driver uses it to
    /// answer an ioctl received on its write queue by sending the
    /// acknowledgement up its read queue. It can be called from user,
    /// interrupt or kernel context.
    pub fn qreply(q: *mut queue, mp: *mut msgb);
    /// `qenable(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Adds the queue `q` to the list of queues whose service routines are
    /// ready to be called by the STREAMS scheduler. It can be called from
    /// user, interrupt or kernel context.
    pub fn qenable(q: *mut queue);
    /// `qsize(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns the number of messages on the queue `q`, or 0 if it is
    /// empty. It can be called from user, interrupt or kernel context.
    pub fn qsize(q: *mut queue) -> c_int;
    /// `noenable(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Prevents the queue `q` from being scheduled for service by
    /// `insq(9F)`, `putq(9F)` or `putbq(9F)` when an ordinary priority
    /// message is enqueued, by setting `QNOENB`. The queue can be
    /// re-enabled with `enableok(9F)`. It can be called from user,
    /// interrupt or kernel context.
    pub fn noenable(q: *mut queue);
    /// `enableok(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Allows the queue `q` to be scheduled for service again, reversing
    /// the effect of `noenable(9F)` by clearing `QNOENB`. It can be called
    /// from user, interrupt or kernel context.
    pub fn enableok(q: *mut queue);
    /// `strqget(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Reads the field selected by `what` from the queue `q`, or from its
    /// priority band `pri` when the field is per band, and stores it at
    /// `valp`. It gives drivers a way to inspect a queue without touching
    /// the STREAMS structures directly, which insulates them from changes
    /// in the implementation. Returns 0 on success and an errno on failure.
    /// It can be called from user, interrupt or kernel context.
    pub fn strqget(q: *mut queue, what: qfields_t, pri: c_uchar, valp: *mut c_void) -> c_int;
    /// `strqset(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Sets the field selected by `what`, which must be one of `QHIWAT`,
    /// `QLOWAT`, `QMAXPSZ` or `QMINPSZ`, on the queue `q` or its priority
    /// band `pri` to `val`. Returns 0 on success and `EINVAL` if an
    /// undefined attribute is specified. It can be called from user,
    /// interrupt or kernel context.
    pub fn strqset(q: *mut queue, what: qfields_t, pri: c_uchar, val: intptr_t) -> c_int;
    /// `qprocson(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Enables the put and service routines of the driver or module whose
    /// read queue is `q`. It must be called by the open routine before
    /// returning, after any initialisation the put and service routines
    /// need, and before `put(9F)`, `putnext(9F)`, `qbufcall(9F)`,
    /// `qtimeout(9F)`, `qwait(9F)` or `qwait_sig(9F)` is called. The
    /// instance is guaranteed to be single-threaded before the call,
    /// except for asynchronous events such as interrupts and callbacks. It
    /// can be called from user, interrupt or kernel context.
    pub fn qprocson(q: *mut queue);
    /// `qprocsoff(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Disables the put and service routines of the driver or module whose
    /// read queue is `q`, removes its service routines from the service
    /// queue and blocks until any pending service processing completes.
    /// It must be called by the close routine before returning and before
    /// deallocating any resources the put and service routines need. The
    /// instance is guaranteed to be single-threaded after the call, except
    /// for asynchronous events. Documented on the `qprocson(9F)` page. It
    /// can be called from user, interrupt or kernel context.
    pub fn qprocsoff(q: *mut queue);
    /// `freezestr(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Freezes the state of the entire stream containing the queue pair
    /// `q`. A frozen stream blocks any thread trying to enter an open,
    /// close, put or service routine of any queue in the stream, and any
    /// thread already inside that tries to put a message on or take one
    /// off any queue in the stream, with the sole exception of the caller.
    /// Drivers and modules must freeze the stream before manipulating
    /// queues directly rather than through `getq(9F)`, `putq(9F)` and the
    /// other programmatic interfaces. It can be called from any open,
    /// close, put or service routine, as well as from interrupt handlers,
    /// callouts and callbacks.
    pub fn freezestr(q: *mut queue);
    /// `unfreezestr(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Thaws a stream frozen by `freezestr(9F)`, releasing the threads it
    /// blocked. Documented on the `freezestr(9F)` page, with the same
    /// calling context.
    pub fn unfreezestr(q: *mut queue);
    /// `qwait(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Waits for a message to arrive at the `put(9E)` or `srv(9E)`
    /// procedures, or for a `qbufcall(9F)` or `qtimeout(9F)` callback to
    /// run. It atomically exits the inner and outer perimeters associated
    /// with the queue, waits for a thread to leave the module's put,
    /// service or callback procedures, and re-enters the perimeters before
    /// returning. It is not interrupted by a signal. `qprocson(9F)` must
    /// have been called first. It can only be called from an `open(9E)` or
    /// `close(9E)` routine.
    pub fn qwait(q: *mut queue);
    /// `qwait_sig(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like `qwait(9F)`, but a pending signal ends the wait. Returns
    /// non-zero normally and 0 when the wait was interrupted by a signal.
    /// The thread calling close does not necessarily receive signals,
    /// particularly when called from `exit(2)`, in which case it behaves
    /// exactly as `qwait`; `ddi_can_receive_sig(9F)` tells when that is
    /// so. Documented on the `qwait(9F)` page. It can only be called from
    /// an `open(9E)` or `close(9E)` routine.
    pub fn qwait_sig(q: *mut queue) -> c_int;
    /// `qwriter(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Upgrades the access at the inner or outer perimeter, given by
    /// `perimeter` as `PERIM_INNER` or `PERIM_OUTER`, from shared to
    /// exclusive and calls `func` with `qp` and `mp` once the upgrade has
    /// succeeded. If exclusive access is available immediately the
    /// callback runs before `qwriter` returns; otherwise the upgrade is
    /// deferred and `qwriter` returns first, so modules must not assume
    /// the callback has run. While the upgrade is pending, the framework
    /// keeps other messages out of the inner perimeter. It can only be
    /// called from a `put(9E)` or `srv(9E)` routine or from a `qwriter`,
    /// `qtimeout(9F)` or `qbufcall(9F)` callback.
    pub fn qwriter(qp: *mut queue, mp: *mut msgb, func: qwriter_fn_t, perimeter: c_int);
    /// `qtimeout(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Schedules `func` to be called with `arg` after `ticks` clock ticks,
    /// after entering the perimeters associated with the queue `q`, and
    /// returns immediately. The interval is a close approximation rather
    /// than a guarantee. `qprocson(9F)` must have been called first, and
    /// all outstanding timeouts must be cancelled with `quntimeout(9F)`
    /// before the close routine blocks or calls `qprocsoff(9F)`. Returns an
    /// opaque non-zero identifier for `quntimeout`. It can be called from
    /// user, interrupt or kernel context.
    pub fn qtimeout(q: *mut queue, func: bufcall_fn_t, arg: *mut c_void, ticks: clock_t) -> timeout_id_t;
    /// `quntimeout(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cancels the pending `qtimeout(9F)` request `id` on the queue `q`.
    /// It returns when the timeout has been cancelled or has finished
    /// executing, and cancels it even if it is blocked at the perimeters.
    /// It should be called for every outstanding timeout before close
    /// returns. Returns -1 if the identifier is not found and 0 or a
    /// positive value otherwise. It can be called from user, interrupt or
    /// kernel context.
    pub fn quntimeout(q: *mut queue, id: timeout_id_t) -> clock_t;
    /// `qbufcall(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like `bufcall(9F)`, but `func` is called with `arg` after entering
    /// the perimeters associated with the queue `q`. It serves as a
    /// `qtimeout(9F)` of indeterminate length when an allocation of `size`
    /// bytes has failed. The `pri` argument is not used. `qprocson(9F)`
    /// must have been called first, and all outstanding bufcalls must be
    /// cancelled with `qunbufcall(9F)` before the close routine blocks or
    /// calls `qprocsoff(9F)`. Returns a bufcall identifier, or 0 if the
    /// scheduling failed, in which case `func` is never called. It can be
    /// called from user, interrupt or kernel context.
    pub fn qbufcall(q: *mut queue, size: size_t, pri: uint_t, func: bufcall_fn_t, arg: *mut c_void) -> bufcall_id_t;
    /// `qunbufcall(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cancels the pending `qbufcall(9F)` request identified by the
    /// non-zero `id` on the queue `q`. It returns when the bufcall has
    /// been cancelled or has finished executing, and cancels it even if it
    /// is blocked at the perimeters. It can be called from user, interrupt
    /// or kernel context.
    pub fn qunbufcall(q: *mut queue, id: bufcall_id_t);
    /// `qassociate(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Associates the stream containing the queue `q`, which may be either
    /// the read or the write queue, with the given `instance` of the
    /// bottom driver, dissolving any prior association on success. DLPI
    /// style 2 drivers must call it while processing `DL_ATTACH_REQ` and
    /// whenever they associate a stream by other means; `gld(4D)` does so
    /// on behalf of the drivers built on it. An `instance` of -1 leaves the
    /// stream unassociated and never fails. On failure, typically because
    /// the instance could not be located and attached, the prior
    /// association is left untouched. Returns 0 on success and -1 on
    /// failure. It can be called from the stream's `put(9E)` entry point.
    pub fn qassociate(q: *mut queue, instance: c_int) -> c_int;

    // --- sys/strsun.h helpers --------------------------------------------------
    /// `mcopyin(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the `M_IOCTL` or `M_IOCDATA` message `mp` into an
    /// `M_COPYIN` by changing its type and rewriting its payload from an
    /// `iocblk(9S)` into a `copyreq(9S)`, setting `cq_private` to
    /// `private`, `cq_size` to `size` and `cq_addr` to `useraddr`. If
    /// `useraddr` is NULL the message must be a transparent `M_IOCTL` and
    /// `cq_addr` is taken from the pointer-sized value at
    /// `mp->b_cont->b_rptr`. Any trailing message blocks are freed. It can
    /// be called from user, kernel or interrupt context.
    pub fn mcopyin(mp: *mut msgb, private: *mut c_void, size: size_t, useraddr: *mut c_void);
    /// `mcopyout(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the `M_IOCTL` or `M_IOCDATA` message `mp` into an
    /// `M_COPYOUT` in the same way as `mcopyin(9F)`. If `dp` is not NULL,
    /// any trailing blocks of `mp` are freed, `mp->b_cont` is set to `dp`
    /// and `dp->b_wptr` is set to `dp->b_rptr + size`; otherwise the
    /// trailing blocks are left alone. It can be called from user, kernel
    /// or interrupt context.
    pub fn mcopyout(mp: *mut msgb, private: *mut c_void, size: size_t, useraddr: *mut c_void, dp: *mut msgb);
    /// `mcopymsg(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies the contents of the message `mp`, block after block, into
    /// the buffer `buf` and then frees the message. The buffer must be
    /// large enough for the whole message, which `msgsize(9F)` can
    /// compute; otherwise the results are unspecified. It can be called
    /// from user, kernel or interrupt context.
    pub fn mcopymsg(mp: *mut msgb, buf: *mut c_void);
    /// `merror(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Constructs a one-byte `M_ERROR` message carrying `error` and sends
    /// it upstream on the read queue paired with the write queue `wq`. If
    /// `mp` is NULL a new message is allocated; otherwise `mp` is converted
    /// in place unless it has more than one reference or is of zero
    /// length, in which case it is freed and a new one allocated. If the
    /// allocation fails the function silently does nothing. Callers must
    /// not hold any lock that `put(9E)` processing might acquire. It can be
    /// called from user, kernel or interrupt context.
    pub fn merror(wq: *mut queue, mp: *mut msgb, error: c_int);
    /// `mexchange(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Exchanges the message `mp` for one of `size` bytes and message type
    /// `type`. If `mp` is not NULL, is at least `size` bytes and has only
    /// one reference it is converted in place; otherwise a new message is
    /// allocated and, if that fails and `wq` is not NULL, `merror(9F)`
    /// attempts to send an error to the stream head. If `primtype` is not
    /// -1 and `size` is at least 4, the first four bytes are set to
    /// `primtype`, which suits protocols such as DLPI and TPI that store
    /// the primitive there. Returns the message, or NULL on failure. It
    /// can be called from user, kernel or interrupt context.
    pub fn mexchange(wq: *mut queue, mp: *mut msgb, size: size_t, r#type: c_uchar, primtype: i32) -> *mut msgb;
    /// `mioc2ack(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the `M_IOCTL` message `mp` into an `M_IOCACK` without
    /// sending it. The type is changed, `ioc_count` and `ioc_rval` are set
    /// to `count` and `rval`, `ioc_error` is cleared, any blocks chained
    /// off `mp->b_cont` are freed and `mp->b_cont` is set to `dp`. If `dp`
    /// is not NULL its `b_wptr` is set to `b_rptr + count`. It can be called
    /// from user, kernel or interrupt context.
    pub fn mioc2ack(mp: *mut msgb, dp: *mut msgb, count: size_t, rval: c_int);
    /// `miocack(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the `M_IOCTL` message `mp` into an `M_IOCACK`, setting
    /// `ioc_count` and `ioc_rval` to `count` and `rval` and clearing
    /// `ioc_error`, and sends it upstream on the read queue paired with
    /// the write queue `wq`. If `count` is non-zero the caller must already
    /// have set `mp->b_cont` to a block of at least `count` bytes. Callers
    /// that need to do more between the conversion and the send should
    /// use `mioc2ack(9F)`. Callers must not hold any lock that `put(9E)`
    /// processing might acquire. It can be called from user, kernel or
    /// interrupt context.
    pub fn miocack(wq: *mut queue, mp: *mut msgb, count: c_int, rval: c_int);
    /// `miocnak(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Converts the `M_IOCTL` message `mp` into an `M_IOCNAK`, setting
    /// `ioc_count` and `ioc_error` to `count` and `error` and clearing
    /// `ioc_rval`, and sends it upstream on the read queue paired with the
    /// write queue `wq`. Payloads cannot be attached to an `M_IOCNAK`, so
    /// `count` must be zero. If `error` is 0, `EINVAL` is assumed. Callers
    /// must not hold any lock that `put(9E)` processing might acquire. It
    /// can be called from user, kernel or interrupt context.
    pub fn miocnak(wq: *mut queue, mp: *mut msgb, count: c_int, error: c_int);
    /// `miocpullup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Prepares the payload of the `M_IOCTL` message `mp` for access by
    /// ensuring that it consists of at least `size` contiguous bytes,
    /// concatenating blocks as necessary. If the ioctl is transparent or
    /// the payload is shorter than `size`, an error is returned. With a
    /// `size` of 0 it succeeds even when there is no payload. Returns 0 on
    /// success and an errno otherwise. It can be called from user, kernel
    /// or interrupt context.
    pub fn miocpullup(mp: *mut msgb, size: size_t) -> c_int;

    // --- sys/strlog.h ------------------------------------------------------------
    /// `strlog(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Submits a formatted message to the `log(4D)` driver. `mid` is the
    /// module identifier, the `mi_idnum` of `module_info(9S)`, `sid`
    /// identifies a minor device, `level` is a tracing level where larger
    /// values mean less important information, and `flags` is a
    /// combination of the `SL_*` values naming the logs the message goes
    /// to. The format string follows `printf(3C)`, but `%e`, `%g` and `%G`
    /// are not allowed; the arguments are recorded as 32-bit values, so a
    /// 64-bit argument is truncated in the stored copy though fully
    /// represented in the formatted text. Returns 1 on success and 0 if
    /// the message could not be submitted. It can be called from user,
    /// interrupt or kernel context.
    pub fn strlog(mid: c_short, sid: c_short, level: c_char, flags: c_ushort, fmt: *mut c_char, ...) -> c_int;

    // --- sys/dlpi.h helpers (io/sundlpi.c, genunix) ------------------------------
    /// `dlokack(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sends a `DL_OK_ACK` reply for the primitive `correct_primitive`, a
    /// `t_uscalar_t`, upstream with `qreply(9F)` on the write queue `wq`.
    /// The request message `mp` is reused for the reply if it is large
    /// enough and otherwise freed and replaced. Documented on the
    /// `dlbindack(9F)` page. It can be called from user, interrupt or
    /// kernel context.
    pub fn dlokack(wq: *mut queue, mp: *mut msgb, correct_primitive: u32);
    /// `dlerrorack(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sends a `DL_ERROR_ACK` reply upstream with `qreply(9F)` on the
    /// write queue `wq`, naming the failed primitive `error_primitive`,
    /// the DLPI error `error` and the corresponding UNIX errno
    /// `unix_errno`, all of type `t_uscalar_t`. The request message `mp`
    /// is reused if large enough. Documented on the `dlbindack(9F)` page.
    /// It can be called from user, interrupt or kernel context.
    pub fn dlerrorack(wq: *mut queue, mp: *mut msgb, error_primitive: u32, error: u32, unix_errno: u32);
    /// `dlbindack(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sends a `DL_BIND_ACK` reply upstream with `qreply(9F)` on the write
    /// queue `wq` to complete a `DL_BIND_REQ`. `sap` is the service access
    /// point bound, a `t_scalar_t`; `addrp` and `addrlen` give the DLPI
    /// layer source address; `maxconind` is the maximum number of
    /// outstanding `DL_CONNECT_IND` messages; and `xidtest` names the XID
    /// and TEST responses supported. The last three are `t_uscalar_t`. The
    /// request message `mp` is reused if large enough. It can be called
    /// from user, interrupt or kernel context.
    pub fn dlbindack(
        wq: *mut queue,
        mp: *mut msgb,
        sap: i32,
        addrp: *const c_void,
        addrlen: u32,
        maxconind: u32,
        xidtest: u32,
    );
    /// `dlphysaddrack(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sends a `DL_PHYS_ADDR_ACK` reply carrying the `addrlen` bytes of
    /// address at `addrp` upstream with `qreply(9F)` on the write queue
    /// `wq`. `addrlen` is a `t_uscalar_t`. The request message `mp` is
    /// reused if large enough. Documented on the `dlbindack(9F)` page. It
    /// can be called from user, interrupt or kernel context.
    pub fn dlphysaddrack(wq: *mut queue, mp: *mut msgb, addrp: *const c_void, addrlen: u32);
    /// `dluderrorind(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sends a `DL_UDERROR_IND` reply upstream with `qreply(9F)` on the
    /// write queue `wq` to report an unsuccessful `DL_UNITDATA_REQ`. The
    /// destination address is the `addrlen` bytes at `addrp`, and `error`
    /// and `unix_errno` give the DLPI error and the corresponding errno;
    /// the scalars are `t_uscalar_t`. The request message `mp` is reused if
    /// large enough. Documented on the `dlbindack(9F)` page. It can be
    /// called from user, interrupt or kernel context.
    pub fn dluderrorind(wq: *mut queue, mp: *mut msgb, addrp: *const c_void, addrlen: u32, error: u32, unix_errno: u32);
}
