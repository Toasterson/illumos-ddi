//! Block I/O from `sys/buf.h` and `sys/aio_req.h`, namely the `buf(9S)` and
//! `aio_req(9S)` structures and the `bio*(9F)` family that manages buffers.
//!
//! A `buf(9S)` describes one block transfer. The kernel hands one to a
//! driver's `strategy(9E)` entry point, `physio(9F)` builds them to carry out
//! the character I/O requests of `read(9E)` and `write(9E)`, and a driver can
//! allocate its own with `getrbuf(9F)` or with `kmem_alloc(9F)` of
//! `biosize(9F)` bytes followed by `bioinit(9F)`. The driver reports the end
//! of a transfer with `biodone(9F)` and, if it waits for its own buffers,
//! blocks in `biowait(9F)`.
//!
//! The structure is declared with its full layout because drivers read and
//! write its fields directly. The manual page lists the members that a
//! driver may use, which are `b_flags`, `av_forw`, `av_back`, `b_bcount`,
//! `b_un.b_addr`, `b_blkno`, `b_lblkno`, `b_resid`, `b_bufsize`, `b_iodone`,
//! `b_error`, `b_private` and `b_edev`, and warns that the other members
//! belong to the kernel. It also tells drivers not to depend on the size of
//! the structure, which is why buffers are allocated through `getrbuf(9F)`
//! or `biosize(9F)` rather than by declaring one. The layout assertions
//! below exist to catch a mistake in this declaration, not to invite drivers
//! to embed a `buf` in their own structures.
//!
//! The C header spells several members through macros. `b_blkno` and
//! `b_lblkno` both name the 64-bit value inside `_b_blkno` under LP64, and
//! `b_actf`, `b_actl`, `b_active` and `b_errcnt` are older names for
//! `av_forw`, `av_back`, `b_bcount` and `b_resid`. Those spellings, like
//! every other macro, belong to the safe `ddi` crate. `clrbuf(9F)` and
//! `geterror(9F)` are real functions in the kernel, not macros, and are
//! bound here.
//!
//! Only the seven `B_*` flags that the header marks as part of the DDI, and
//! that `buf(9S)` documents, are declared. The header's second group of
//! flags, from `B_WANTED` to `B_PAGE_NOWAIT`, is explicitly not part of the
//! DDI. The buffer cache structures `hbuf`, `dwbuf` and `biostats` and the
//! buffer cache functions such as `bread`, `brelse`, `bwrite`, `iodone`,
//! `iowait` and `pageio_setup` have no manual page and are not in the `ddi`
//! mapfile, so they are not bound either. `aphysio` and `anocancel` in
//! `sys/aio_req.h` are named by `aread(9E)`, but no `aphysio(9F)` page exists
//! in the gate and the mapfile does not list them, so they are left out
//! until the page is written. `bufcall(9F)` and `unbufcall(9F)` concern
//! STREAMS message buffers, not `buf(9S)`, and are declared in
//! [`crate::streams`].

use crate::devops::cb_strategy_t;
use crate::ksynch::ksema_t;
use crate::types::*;
use crate::uio::uio;

/// The buffer is in use (`B_BUSY`). A driver may only change this flag on a
/// buffer it allocated with `getrbuf(9F)` while no transfer is in progress.
pub const B_BUSY: c_int = 0x0001;
/// The transfer has completed (`B_DONE`). `biodone(9F)` sets it, and drivers
/// only read it.
pub const B_DONE: c_int = 0x0002;
/// The transfer failed (`B_ERROR`). It is set together with `b_error`;
/// drivers use `bioerror(9F)` rather than setting it themselves.
pub const B_ERROR: c_int = 0x0004;
/// The buffer describes a paged I/O request, so `b_un.b_addr` holds kernel
/// private data rather than an address (`B_PAGEIO`). Drivers only read it.
pub const B_PAGEIO: c_int = 0x0010;
/// The buffer describes physical I/O to a user data area, so `b_un.b_addr`
/// is a user virtual address (`B_PHYS`). Drivers only read it.
pub const B_PHYS: c_int = 0x0020;
/// The data moves from the device into memory (`B_READ`).
pub const B_READ: c_int = 0x0040;
/// The data moves from memory to the device (`B_WRITE`). This is a pseudo
/// flag that cannot be tested directly; a transfer is a write when
/// `B_READ` is clear. It is passed as the `rw` argument of `physio(9F)`.
pub const B_WRITE: c_int = 0x0100;

/// The `b_un` member of a `buf(9S)`, which is an anonymous union in C.
///
/// Drivers use `b_addr` only. The other members are the views that the UFS
/// file system takes of a buffer holding its own metadata. Their C types
/// point to UFS structures that are private to that file system, so they are
/// declared here as untyped pointers of the same size.
#[repr(C)]
#[derive(Clone, Copy)]
pub union buf_un {
    /// The virtual address of the data. It is a kernel address unless
    /// `B_PHYS` is set, in which case it is a user address, and it holds
    /// kernel private data when `B_PAGEIO` is set. `bp_mapin(9F)` stores
    /// the kernel mapping of a paged request here.
    pub b_addr: caddr_t,
    /// A UFS superblock, `struct fs *` in C.
    pub b_fs: *mut c_void,
    /// A UFS cylinder group block, `struct cg *` in C.
    pub b_cg: *mut c_void,
    /// A UFS inode list block, `struct dinode *` in C.
    pub b_dino: *mut c_void,
    /// A block of 32-bit disk addresses, `daddr32_t *` in C.
    pub b_daddr: *mut i32,
}

const _: () = assert!(core::mem::size_of::<buf_un>() == 8);

/// `buf(9S)`. Architecture independent level 1 (DDI/DKI).
///
/// The buffer header that describes a block I/O transfer. It is the only
/// argument of `strategy(9E)`. A buffer can be linked on several lists at
/// once, so a driver changes only the members the manual page names, and
/// never clears `b_flags`. The members that are not documented are
/// declared for the sake of an exact layout and must be left alone. Several
/// of them point to kernel structures, such as processes, pages and vnodes,
/// that this crate does not declare, and are therefore given as untyped
/// pointers.
#[repr(C)]
pub struct buf {
    /// The buffer status, a combination of the `B_*` flags. It tells the
    /// driver whether to read or write, among other things. The driver must
    /// never clear it, and all bits not documented are reserved for the
    /// kernel.
    pub b_flags: c_int,
    /// The forward link on the buffer cache hash chain. Owned by the kernel.
    pub b_forw: *mut buf,
    /// The backward link on the buffer cache hash chain. Owned by the
    /// kernel.
    pub b_back: *mut buf,
    /// A link the driver may use to put the buffer on its own work list. It
    /// is also the forward link that `disksort(9F)` maintains.
    pub av_forw: *mut buf,
    /// A second link the driver may use for its own work list.
    pub av_back: *mut buf,
    /// The old 16-bit device number, `o_dev_t` in C. Obsolete; drivers use
    /// `b_edev`.
    pub b_dev: c_short,
    /// The number of bytes to transfer, for both paged and non-paged
    /// requests.
    pub b_bcount: size_t,
    /// The address of the data. See [`buf_un`].
    pub b_un: buf_un,
    /// The block on the device at which the transfer starts. The C type is
    /// the `lldaddr_t` union, whose 64-bit member `_f` is what both the
    /// `b_blkno` and the `b_lblkno` macros name under LP64; its other member
    /// only matters to 32-bit kernels. A driver may have to convert the
    /// logical block into a physical location on the device.
    pub _b_blkno: longlong_t,
    /// Obsolete and unused.
    pub b_obs1: c_char,
    /// The number of bytes not transferred, which the driver sets when a
    /// transfer ends early. Before calling `disksort(9F)` the driver stores
    /// the sort key here instead.
    pub b_resid: size_t,
    /// The time the request started. Owned by the kernel.
    pub b_start: clock_t,
    /// The process doing physical or swap I/O, `struct proc *` in C. Owned by
    /// the kernel.
    pub b_proc: *mut c_void,
    /// The page list of a paged request, `struct page *` in C. Owned by the
    /// kernel.
    pub b_pages: *mut c_void,
    /// Obsolete and unused.
    pub b_obs2: clock_t,
    /// The size of the allocated buffer.
    pub b_bufsize: size_t,
    /// The routine that `biodone(9F)` calls instead of completing the buffer
    /// itself, or `None`. A driver that sets it saves the old value first
    /// and restores it before the buffer is released.
    pub b_iodone: Option<unsafe extern "C" fn(bp: *mut buf) -> c_int>,
    /// The vnode associated with the block, `struct vnode *` in C. Owned by
    /// the kernel.
    pub b_vp: *mut c_void,
    /// A chain of all buffers, used by the buffer cache. Owned by the
    /// kernel.
    pub b_chain: *mut buf,
    /// Obsolete and unused.
    pub b_obs3: c_int,
    /// The error number to return from the driver when `B_ERROR` is set.
    /// Drivers set it through `bioerror(9F)` and read it through
    /// `geterror(9F)`.
    pub b_error: c_int,
    /// A pointer for the private use of the driver.
    pub b_private: *mut c_void,
    /// The device number of the device accessed, with its major and minor
    /// parts.
    pub b_edev: dev_t,
    /// The semaphore that gives exclusive access to the buffer. Owned by the
    /// kernel.
    pub b_sem: ksema_t,
    /// The semaphore on which `biowait(9F)` sleeps until `biodone(9F)`
    /// signals it. Owned by the kernel.
    pub b_io: ksema_t,
    /// The list of buffers that may need a delayed write. Owned by the
    /// kernel.
    pub b_list: *mut buf,
    /// The shadow page list, `struct page **` in C. Owned by the kernel.
    pub b_shadow: *mut *mut c_void,
    /// The device node of the device, stored as a `void *`. Owned by the
    /// kernel.
    pub b_dip: *mut c_void,
    /// The file associated with the buffer, `struct vnode *` in C. Owned by
    /// the kernel.
    pub b_file: *mut c_void,
    /// The offset in `b_file` that the buffer corresponds to. Owned by the
    /// kernel.
    pub b_offset: offset_t,
}
/// The typedef name of [`buf`].
pub type buf_t = buf;

// Layout under LP64: b_flags 4 at 0, 4 bytes of padding, b_forw 8 at 8,
// b_back 8 at 16, av_forw 8 at 24, av_back 8 at 32, b_dev 2 at 40, 6 bytes
// of padding, b_bcount 8 at 48, b_un 8 at 56, _b_blkno 8 at 64, b_obs1 1 at
// 72, 7 bytes of padding, b_resid 8 at 80, b_start 8 at 88, b_proc 8 at 96,
// b_pages 8 at 104, b_obs2 8 at 112, b_bufsize 8 at 120, b_iodone 8 at 128,
// b_vp 8 at 136, b_chain 8 at 144, b_obs3 4 at 152, b_error 4 at 156,
// b_private 8 at 160, b_edev 8 at 168, b_sem 16 at 176, b_io 16 at 192,
// b_list 8 at 208, b_shadow 8 at 216, b_dip 8 at 224, b_file 8 at 232,
// b_offset 8 at 240; total 248.
const _: () = assert!(core::mem::size_of::<buf>() == 248);
const _: () = assert!(core::mem::offset_of!(buf, b_flags) == 0);
const _: () = assert!(core::mem::offset_of!(buf, b_forw) == 8);
const _: () = assert!(core::mem::offset_of!(buf, av_forw) == 24);
const _: () = assert!(core::mem::offset_of!(buf, av_back) == 32);
const _: () = assert!(core::mem::offset_of!(buf, b_dev) == 40);
const _: () = assert!(core::mem::offset_of!(buf, b_bcount) == 48);
const _: () = assert!(core::mem::offset_of!(buf, b_un) == 56);
const _: () = assert!(core::mem::offset_of!(buf, _b_blkno) == 64);
const _: () = assert!(core::mem::offset_of!(buf, b_obs1) == 72);
const _: () = assert!(core::mem::offset_of!(buf, b_resid) == 80);
const _: () = assert!(core::mem::offset_of!(buf, b_bufsize) == 120);
const _: () = assert!(core::mem::offset_of!(buf, b_iodone) == 128);
const _: () = assert!(core::mem::offset_of!(buf, b_obs3) == 152);
const _: () = assert!(core::mem::offset_of!(buf, b_error) == 156);
const _: () = assert!(core::mem::offset_of!(buf, b_private) == 160);
const _: () = assert!(core::mem::offset_of!(buf, b_edev) == 168);
const _: () = assert!(core::mem::offset_of!(buf, b_sem) == 176);
const _: () = assert!(core::mem::offset_of!(buf, b_io) == 192);
const _: () = assert!(core::mem::offset_of!(buf, b_list) == 208);
const _: () = assert!(core::mem::offset_of!(buf, b_offset) == 240);

/// The head of a driver's request queue as sorted by `disksort(9F)`,
/// `struct diskhd`.
///
/// Its leading members mirror those of a `buf(9S)`. Only `av_forw` and
/// `av_back` are of interest; they hold the first and the last buffer of
/// the queue, which is linked forward through the `av_forw` member of each
/// buffer. The driver provides the storage. An empty queue has a NULL
/// `av_forw`, so the driver zeroes the structure before first use.
#[repr(C)]
pub struct diskhd {
    /// Unused. Present so that the leading members line up with `buf`.
    pub b_flags: c_int,
    /// Unused by `disksort(9F)`.
    pub b_forw: *mut buf,
    /// Unused by `disksort(9F)`.
    pub b_back: *mut buf,
    /// The first buffer on the queue, or NULL if the queue is empty.
    pub av_forw: *mut buf,
    /// The last buffer on the queue.
    pub av_back: *mut buf,
    /// The old 16-bit device number, `o_dev_t` in C. Unused.
    pub b_dev: c_short,
    /// A transfer count. Unused by `disksort(9F)`.
    pub b_bcount: size_t,
}

// Layout under LP64: b_flags 4 at 0, 4 bytes of padding, b_forw 8 at 8,
// b_back 8 at 16, av_forw 8 at 24, av_back 8 at 32, b_dev 2 at 40, 6 bytes of
// padding, b_bcount 8 at 48; total 56.
const _: () = assert!(core::mem::size_of::<diskhd>() == 56);
const _: () = assert!(core::mem::offset_of!(diskhd, b_flags) == 0);
const _: () = assert!(core::mem::offset_of!(diskhd, b_forw) == 8);
const _: () = assert!(core::mem::offset_of!(diskhd, av_forw) == 24);
const _: () = assert!(core::mem::offset_of!(diskhd, av_back) == 32);
const _: () = assert!(core::mem::offset_of!(diskhd, b_dev) == 40);
const _: () = assert!(core::mem::offset_of!(diskhd, b_bcount) == 48);

/// `aio_req(9S)`. illumos DDI specific (illumos DDI).
///
/// Describes an asynchronous I/O request. The kernel passes one to the
/// `aread(9E)` and `awrite(9E)` entry points, which hand it on to the
/// kernel's asynchronous physical I/O routine. The header declares the
/// structure only for the kernel.
#[repr(C)]
pub struct aio_req {
    /// The `uio(9S)` structure that describes the transfer.
    pub aio_uio: *mut uio,
    /// Private to the kernel's asynchronous I/O framework; the manual page
    /// does not document it.
    pub aio_private: *mut c_void,
}

// Layout under LP64: aio_uio 8 at 0, aio_private 8 at 8; total 16.
const _: () = assert!(core::mem::size_of::<aio_req>() == 16);
const _: () = assert!(core::mem::offset_of!(aio_req, aio_uio) == 0);
const _: () = assert!(core::mem::offset_of!(aio_req, aio_private) == 8);

/// The signature of the `mincnt` routine that `physio(9F)` takes.
///
/// The routine receives each buffer before it goes to the strategy routine
/// and lowers `b_bcount` to the largest transfer the system and the device
/// can handle.
/// The kernel's own [`minphys`] has this signature, and a driver that
/// supplies its own routine calls `minphys` from it as well, so that it
/// never exceeds the system limit.
pub type minphys_t = unsafe extern "C" fn(bp: *mut buf);

unsafe extern "C" {
    /// `bioclone(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns a buffer set up to perform I/O on part of the transfer that
    /// `bp` describes. The new transfer starts `off` bytes into the original
    /// and is `len` bytes long, and `off` plus `len` must not exceed the
    /// original `b_bcount`. `dev` is the device to which the new buffer
    /// transfers, and `blkno` becomes its `b_blkno`. `iodone` is the routine
    /// that `biodone(9F)` calls when the new transfer completes, or `None`.
    /// If `bp_mem` is NULL the buffer is allocated with [`getrbuf`] and
    /// `sleepflag`, which is `KM_SLEEP` or `KM_NOSLEEP`, decides whether the
    /// allocation may wait for memory; such a buffer is released with
    /// [`freerbuf`]. Otherwise `bp_mem` is used as the buffer and must have
    /// been initialised with [`getrbuf`] or [`bioinit`].
    ///
    /// If the original buffer was mapped into kernel virtual memory with
    /// [`bp_mapin`], the clone shares that mapping. The original buffer must
    /// not be freed while any clone is still transferring, and `biodone`
    /// must be called on every clone before it is called on the original.
    /// Returns the new buffer, or NULL if no memory was available. The
    /// function may be called from user, interrupt or kernel context, but
    /// must not be allowed to sleep when called from an interrupt routine.
    pub fn bioclone(
        bp: *mut buf,
        off: off_t,
        len: size_t,
        dev: dev_t,
        blkno: daddr_t,
        iodone: Option<unsafe extern "C" fn(bp: *mut buf) -> c_int>,
        bp_mem: *mut buf,
        sleepflag: c_int,
    ) -> *mut buf;
    /// `biodone(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Marks the transfer described by `bp` as complete. It sets `B_DONE` in
    /// `b_flags`, wakes the threads waiting in [`biowait`] for it, and
    /// releases the buffer if the transfer was asynchronous. If `b_iodone`
    /// is set, the function calls that routine instead and does nothing
    /// else. A driver calls it from its interrupt routine or from
    /// `strategy(9E)` when a request ends, including requests it rejects.
    /// The driver must not refer to `bp` after the call. It may be called
    /// from user, interrupt or kernel context.
    pub fn biodone(bp: *mut buf);
    /// `bioerror(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Records that the transfer described by `bp` failed with the error
    /// number `error`, so that a later [`geterror`] returns it. If `error`
    /// is 0 the error indication is cleared instead. Drivers use it in
    /// preference to setting `B_ERROR` and `b_error` themselves. It may be
    /// called from any context.
    pub fn bioerror(bp: *mut buf, error: c_int);
    /// `biofini(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Uninitialises the buffer header at `bp`. A buffer that was allocated
    /// with `kmem_alloc(9F)` and initialised with [`bioinit`] must be passed
    /// to this function before its memory is released with `kmem_free(9F)`.
    /// A buffer from [`getrbuf`] does not need it, because [`freerbuf`] calls
    /// it. It may be called from any context.
    pub fn biofini(bp: *mut buf);
    /// `bioinit(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Initialises the buffer header at `bp`, whose memory the driver
    /// allocated with `kmem_alloc(9F)` using the size from [`biosize`]. A
    /// buffer from [`getrbuf`] is already initialised. It may be called from
    /// any context.
    pub fn bioinit(bp: *mut buf);
    /// `biomodified(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether any of the memory pages of the paged I/O request `bp`
    /// has been modified, as recorded by the virtual memory system's
    /// modification bit. The file system clears that bit when it writes the
    /// pages to their backing store, so a mirroring driver can use this
    /// function to detect a program changing a buffer while it is being
    /// written to several devices. `B_PAGEIO` must be set in `b_flags`.
    /// Returns 1 if the buffer is modified, 0 if it is not, and -1 if the
    /// buffer is not used for a paged I/O request. It may be called from
    /// any context.
    pub fn biomodified(bp: *mut buf) -> c_int;
    /// `bioreset(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the buffer header at `bp` to the state it had when
    /// [`getrbuf`] allocated it or [`bioinit`] initialised it, so that a
    /// driver can reuse a private buffer for another transfer before
    /// releasing it. `bp` must not describe a transfer in progress. It may
    /// be called from any context.
    pub fn bioreset(bp: *mut buf);
    /// `biosize(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the size in bytes of a `buf(9S)` structure. A driver that
    /// allocates buffer headers with `kmem_alloc(9F)`, for instance to embed
    /// them in its own data structures, uses this size together with
    /// [`bioinit`]. It may be called from any context.
    pub fn biosize() -> size_t;
    /// `biowait(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Suspends the calling thread until the transfer described by `bp`
    /// completes, which the driver signals by calling [`biodone`], usually
    /// from its interrupt routine. It is meant for drivers that allocated
    /// the buffer themselves with [`getrbuf`]. Returns 0 on success, or on
    /// failure the non-zero error number that [`geterror`] reports. It may
    /// be called from user context only.
    pub fn biowait(bp: *mut buf) -> c_int;
    /// `bp_copyin(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Copies `size` bytes from the memory associated with `bp`, starting
    /// `offset` bytes into it, to the driver buffer `driverbuf`. The offset
    /// applies to `bp` only. Returns 0 on success and -1 if `bp` refers to
    /// invalid pages. It may be called from user or kernel context only.
    pub fn bp_copyin(bp: *mut buf, driverbuf: *mut c_void, offset: offset_t, size: size_t) -> c_int;
    /// `bp_copyout(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Copies `size` bytes from the driver buffer `driverbuf` into the
    /// memory associated with `bp`, starting `offset` bytes into it. The
    /// offset applies to `bp` only. Returns 0 on success and -1 if `bp`
    /// refers to invalid pages. It may be called from user or kernel context
    /// only.
    pub fn bp_copyout(driverbuf: *mut c_void, bp: *mut buf, offset: offset_t, size: size_t) -> c_int;
    /// `bp_mapin(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Maps the page list of the paged I/O request `bp` into kernel virtual
    /// address space and stores the start of the mapping in `b_un.b_addr`.
    /// If the page list is empty, nothing is mapped. The mapping is released
    /// with [`bp_mapout`]. It may be called from user or kernel context.
    pub fn bp_mapin(bp: *mut buf);
    /// `bp_mapout(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Releases the kernel virtual address space that [`bp_mapin`] allocated
    /// for `bp`. It must only be called on buffers that the driver allocated
    /// and owns, never on buffers passed to `strategy(9E)`, because
    /// `bp_mapin` keeps no reference count and the call would remove a
    /// mapping that a layer above the driver may rely on. It may be called
    /// from user context only.
    pub fn bp_mapout(bp: *mut buf);
    /// `clrbuf(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Fills the `b_bcount` bytes starting at `b_un.b_addr` of the buffer
    /// `bp` with zeros and sets its `b_resid` to 0. It may be called from
    /// user, interrupt or kernel context.
    pub fn clrbuf(bp: *mut buf);
    /// `disksort(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Inserts the buffer `bp` into the request queue headed by the
    /// `av_forw` member of `dp`, using a single direction elevator
    /// algorithm. The queue is kept in ascending order of the key in each
    /// buffer's `b_resid` member, which the driver sets before the call to a
    /// cylinder number or an absolute block number, depending on how finely
    /// it wants requests sorted. The buffers are linked through their
    /// `av_forw` members, and `dp` records the tail of the queue in its
    /// `av_back` member. The buffer at the head of the queue is taken to be
    /// the one currently active on the device. It may be called from user,
    /// interrupt or kernel context.
    pub fn disksort(dp: *mut diskhd, bp: *mut buf);
    /// `freerbuf(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Releases a buffer header that [`getrbuf`] allocated. It does not
    /// sleep, so it may be called from user, interrupt or kernel context.
    pub fn freerbuf(bp: *mut buf);
    /// `geterror(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns the error number recorded in the buffer header `bp`, or 0 if
    /// the transfer completed successfully. It may be called from user,
    /// interrupt or kernel context.
    pub fn geterror(bp: *mut buf) -> c_int;
    /// `getrbuf(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Allocates and initialises a buffer header that is not associated with
    /// the buffer cache, as a driver needs for raw I/O. `sleepflag` is
    /// passed to `kmem_alloc(9F)`; with `KM_SLEEP` the call may wait for
    /// memory, and with `KM_NOSLEEP` it does not. Returns the buffer, or
    /// NULL if no memory was available. The buffer is released with
    /// [`freerbuf`]. It may be called from user, interrupt or kernel
    /// context, but must not be allowed to sleep when called from an
    /// interrupt routine.
    pub fn getrbuf(sleepflag: c_int) -> *mut buf;
    /// `physio(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs unbuffered I/O between the device `dev` and the address
    /// space that `uio` describes, and is how a block driver usually
    /// implements `read(9E)` and `write(9E)`. It checks that the caller may
    /// access the memory in the requested way, locks the pages involved,
    /// and calls the strategy routine `strat` one or more times, waiting
    /// with [`biowait`] for each transfer to complete. It then unlocks the
    /// pages. `rw` is `B_READ` to read from the device or `B_WRITE` to write
    /// to it. `mincnt` bounds the size of each transfer, and is [`minphys`]
    /// for drivers that have no limit of their own. If `bp` is NULL, the
    /// function allocates a buffer and releases it afterwards; otherwise it
    /// uses the buffer given. Returns 0 on success and a non-zero error
    /// number on failure. It may be called from user context only.
    pub fn physio(
        strat: Option<cb_strategy_t>,
        bp: *mut buf,
        dev: dev_t,
        rw: c_int,
        mincnt: Option<minphys_t>,
        uio: *mut uio,
    ) -> c_int;
    /// `minphys(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The system's `mincnt` routine for [`physio`]. It lowers `b_bcount` in
    /// the buffer `bp` to a default limit that suits the capabilities of the
    /// system. A driver that supplies its own routine to `physio` calls this
    /// one from it as well, so as not to exceed the system limit. The page
    /// gives no calling context of its own for this routine; `physio` calls
    /// it in the user context in which `physio` itself runs.
    pub fn minphys(bp: *mut buf);
}
