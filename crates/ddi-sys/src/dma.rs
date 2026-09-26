//! DMA resources from `sys/ddidmareq.h`, `sys/dditypes.h` and the DMA part
//! of `sys/sunddi.h`, the `ddi_dma_*(9F)` family.
//!
//! A driver describes what its DMA engine can address in a
//! [`ddi_dma_attr_t`] and hands it to [`ddi_dma_alloc_handle`], which
//! returns a handle that stands for one transfer at a time. Memory that the
//! device will read or write is either allocated for the purpose with
//! [`ddi_dma_mem_alloc`] or comes from elsewhere, such as the `buf(9S)` of
//! a block request. Binding the memory to the handle with
//! [`ddi_dma_addr_bind_handle`] or [`ddi_dma_buf_bind_handle`] allocates
//! the system's mapping resources and produces one or more
//! [`ddi_dma_cookie_t`] values, each a device visible address and length,
//! which the driver reads with [`ddi_dma_cookie_iter`],
//! [`ddi_dma_cookie_get`] or [`ddi_dma_cookie_one`] and programs into the
//! device. Between binding and unbinding, [`ddi_dma_sync`] makes the view
//! of one side visible to the other. [`ddi_dma_unbind_handle`] releases the
//! mapping, [`ddi_dma_mem_free`] the memory and [`ddi_dma_free_handle`] the
//! handle. Objects too large to map at once may be bound with
//! [`DDI_DMA_PARTIAL`] and walked window by window with [`ddi_dma_numwin`]
//! and [`ddi_dma_getwin`].
//!
//! The allocation and binding functions take a callback argument that is
//! either the address of a function, [`DDI_DMA_SLEEP`] or
//! [`DDI_DMA_DONTWAIT`]. In C all three have the function pointer type
//! `int (*)(caddr_t)`, with the two sentinels being the integers 1 and 0
//! cast to it. Rust cannot express an integer as a `fn` value, so the
//! argument is declared as the pointer sized [`ddi_dma_callback_t`] and the
//! real signature of a callback is [`ddi_dma_callback_fn_t`].
//!
//! The return codes shared with the interrupt module, `DDI_EINVAL` and its
//! relatives, live in [`crate::intr`]. The DMA specific codes such as
//! [`DDI_DMA_MAPPED`] and [`DDI_DMA_NORESOURCES`] are declared here.
//!
//! Several things in the headers are deliberately not bound. The
//! `ddi_dmae(9F)` family and `ddi_dmae_req(9S)` drive the ISA DMA engine
//! through `sys/dma_engine.h`, a header outside this module's scope, and
//! are not in the `ddi` mapfile. The `ddi_dma_lim_t`, `ddi_dma_req_t` and
//! `ddi_dma_obj_t` structures and the `ddi_dma_ctlops` enumeration belong
//! to the obsolete `ddi_dma_setup(9F)` interfaces and to nexus drivers.
//! `ddi_dma_attr_merge` has no manual page and is not in the mapfile.
//! `DDI_DMA_SBUS_64BIT`, `DDI_DMA_USE_PHYSICAL`, `DMA_ATTR_VERSION` and
//! `IOMEM_DATA_MASK` appear in the header only. The private
//! `_DDI_DMA_BOUNCE_ON_SEG` flag is consolidation private. There is no
//! `ddi_dma_get_attr` in the header or the manual. The
//! `ddi_check_dma_handle(9F)` function is part of fault management and is
//! left to that module.
//!
//! All layouts are for amd64 with the LP64 data model.

use crate::types::*;

// --- ddi_dma_attr(9S) ----------------------------------------------------

/// The only version of the DMA attribute structure, to be stored in
/// `dma_attr_version` (`DMA_ATTR_V0`).
pub const DMA_ATTR_V0: uint_t = 0;

/// On platforms with an IOMMU that normally hand out virtual I/O addresses,
/// ask for physical addresses instead. If the platform cannot,
/// [`ddi_dma_alloc_handle`] fails with [`DDI_DMA_BADATTR`] and the driver
/// should clear the flag and retry (`DDI_DMA_FORCE_PHYSICAL`).
pub const DDI_DMA_FORCE_PHYSICAL: uint_t = 0x0100;
/// The driver is hardened against DMA faults and checks its handles with
/// `ddi_fm_dma_err_get(9F)`, so the system should mark the handle as faulted
/// rather than panic when an I/O fault hits a transfer through it. The flag
/// is advisory; an implementation may still panic (`DDI_DMA_FLAGERR`).
pub const DDI_DMA_FLAGERR: uint_t = 0x200;
/// Writes through this handle need not be strongly ordered with respect to
/// each other or to writes through other handles, which lets the host
/// bridge move data more efficiently. Devices that can mark individual bus
/// transactions relaxed should do that instead (`DDI_DMA_RELAXED_ORDERING`).
pub const DDI_DMA_RELAXED_ORDERING: uint_t = 0x400;

/// `ddi_dma_attr(9S)`. illumos DDI specific (illumos DDI).
///
/// Describes the addressing capabilities of a device's DMA engine so that
/// the system can allocate resources the device can actually reach. A
/// driver fills one in, usually as a static, and passes it to
/// [`ddi_dma_alloc_handle`]. Nexus drivers between the device and memory may
/// tighten the constraints further but may never loosen them.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_dma_attr_t {
    /// The structure version, always [`DMA_ATTR_V0`].
    pub dma_attr_version: uint_t,
    /// The lowest address the DMA engine can access, inclusive.
    pub dma_attr_addr_lo: u64,
    /// The highest address the DMA engine can access, inclusive. Allocated
    /// resources always fall between the two bounds.
    pub dma_attr_addr_hi: u64,
    /// The largest value the DMA engine's transfer counter can hold, as an
    /// inclusive bound; a 24-bit counter is described by `0xffffff`. Objects
    /// longer than this are split into several cookies.
    pub dma_attr_count_max: u64,
    /// The alignment required of allocated DMA resources, in bytes. Most
    /// drivers use 1. It can force alignment stricter than the burst sizes
    /// or the minimum transfer imply, such as a page boundary. Memory that a
    /// driver binds from elsewhere must be at least as well aligned.
    pub dma_attr_align: u64,
    /// The burst sizes the DMA engine can produce, encoded as a bit mask of
    /// powers of two; bursts of 1, 2, 4 and 16 bytes give `0x17`. The system
    /// may reduce the set, and the driver must program its engine with what
    /// [`ddi_dma_burstsizes`] reports after binding.
    pub dma_attr_burstsizes: uint_t,
    /// The smallest amount of memory a single DMA access touches, in bytes.
    /// It affects alignment and padding in [`ddi_dma_mem_alloc`] and may be
    /// raised by I/O caches or write buffers between engine and memory.
    pub dma_attr_minxfer: u32,
    /// The largest amount of memory one DMA access can move, in bytes.
    pub dma_attr_maxxfer: u64,
    /// The boundary that a single cookie must not cross, as an inclusive
    /// mask; `0xffff` keeps every cookie within one 64 KiB region. The
    /// system splits objects into more cookies or more windows to honour it.
    pub dma_attr_seg: u64,
    /// The length of the device's scatter/gather list. A value greater than
    /// one is the maximum number of cookies per transfer, exactly one means
    /// the engine cannot scatter or gather, a negative value means there is
    /// no limit, for example with DMA chaining, and zero is reserved.
    pub dma_attr_sgllen: c_int,
    /// The granularity of the device's transfer size, in bytes, normally its
    /// sector size. Every segment the system allocates is a multiple of it,
    /// and every request a driver makes must be as well.
    pub dma_attr_granular: u32,
    /// A combination of [`DDI_DMA_FORCE_PHYSICAL`], [`DDI_DMA_FLAGERR`] and
    /// [`DDI_DMA_RELAXED_ORDERING`], or zero.
    pub dma_attr_flags: uint_t,
}

// Layout on amd64: uint_t at 0, then 4 bytes of padding so that the first
// uint64_t starts at 8. Four uint64_t fields occupy 8 to 40. uint_t at 40
// and uint32_t at 44 pack together, so the next uint64_t starts at 48 and
// the following one at 56. int at 64 and uint32_t at 68 pack, uint_t at 72,
// and 4 bytes of tail padding bring the size to a multiple of 8, giving 80.
const _: () = assert!(core::mem::size_of::<ddi_dma_attr_t>() == 80);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_version) == 0);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_addr_lo) == 8);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_addr_hi) == 16);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_count_max) == 24);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_align) == 32);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_burstsizes) == 40);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_minxfer) == 44);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_maxxfer) == 48);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_seg) == 56);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_sgllen) == 64);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_granular) == 68);
const _: () = assert!(core::mem::offset_of!(ddi_dma_attr_t, dma_attr_flags) == 72);

// --- ddi_dma_cookie(9S) --------------------------------------------------

/// The anonymous union at the start of [`ddi_dma_cookie_t`] that holds the
/// device visible address, which C code reaches through the `dmac_laddress`
/// and `dmac_address` macros. The C type has no name of its own.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ddi_dma_cookie_dmu_t {
    /// The full 64-bit I/O address, `dmac_laddress` in C. Devices with a
    /// 64-bit address register program this.
    pub _dmac_ll: u64,
    /// The same address as two 32-bit halves. On amd64, which is little
    /// endian, element 0 is the low half that C calls `dmac_address` and
    /// element 1 is `dmac_notused`. Devices with a 32-bit address register
    /// program element 0.
    pub _dmac_la: [u32; 2],
}

/// `ddi_dma_cookie(9S)`. illumos DDI specific (illumos DDI).
///
/// One contiguous piece of a bound DMA object as the device sees it: an
/// I/O address, a length and bus specific type bits. The system fills
/// cookies in when a handle is bound or a window is activated, and a driver
/// reads them through [`ddi_dma_cookie_iter`], [`ddi_dma_cookie_get`] or
/// [`ddi_dma_cookie_one`] to program its engine or its descriptor rings.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ddi_dma_cookie_t {
    /// The device visible address, in either width.
    pub _dmu: ddi_dma_cookie_dmu_t,
    /// The length of this piece in bytes.
    pub dmac_size: size_t,
    /// Bus specific bits, such as the address modifier bits of a PCI device.
    pub dmac_type: uint_t,
}

// Layout on amd64: the union is 8 bytes and 8 aligned at 0, size_t at 8,
// uint_t at 16, and 4 bytes of tail padding give 24.
const _: () = assert!(core::mem::size_of::<ddi_dma_cookie_dmu_t>() == 8);
const _: () = assert!(core::mem::size_of::<ddi_dma_cookie_t>() == 24);
const _: () = assert!(core::mem::offset_of!(ddi_dma_cookie_t, _dmu) == 0);
const _: () = assert!(core::mem::offset_of!(ddi_dma_cookie_t, dmac_size) == 8);
const _: () = assert!(core::mem::offset_of!(ddi_dma_cookie_t, dmac_type) == 16);

// --- resource callbacks --------------------------------------------------

/// The signature of a function passed as the callback argument of
/// [`ddi_dma_alloc_handle`], [`ddi_dma_mem_alloc`],
/// [`ddi_dma_addr_bind_handle`] or [`ddi_dma_buf_bind_handle`], which is
/// `int (*)(caddr_t)` in C.
///
/// When an allocation fails for lack of resources and the caller supplied
/// such a function, the system calls it with the caller's argument once
/// resources may have become available. The function retries the
/// allocation and returns [`DDI_DMA_CALLBACK_DONE`] if it succeeded or no
/// longer cares, or [`DDI_DMA_CALLBACK_RUNOUT`] if it failed again and wants
/// to be called once more. It runs in interrupt context and must take its
/// own locks.
pub type ddi_dma_callback_fn_t = unsafe extern "C" fn(arg: caddr_t) -> c_int;

/// The callback argument of the DMA allocation and binding functions.
///
/// In C this argument has type `int (*)(caddr_t)` and takes one of three
/// kinds of value: the address of a [`ddi_dma_callback_fn_t`], the sentinel
/// [`DDI_DMA_SLEEP`], which is the integer 1, or the sentinel
/// [`DDI_DMA_DONTWAIT`], which is 0. A Rust `fn` value cannot hold 1, so
/// the argument is declared as this pointer sized wrapper. To pass a real
/// function, cast it to a pointer, as in
/// `ddi_dma_callback_t(my_callback as *const c_void)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ddi_dma_callback_t(pub *const c_void);

/// Do not wait for resources and do not arrange a callback; the call simply
/// fails if resources are short (`DDI_DMA_DONTWAIT`).
pub const DDI_DMA_DONTWAIT: ddi_dma_callback_t = ddi_dma_callback_t(core::ptr::null());
/// Sleep until resources are available. A call made with this value must
/// not come from interrupt context (`DDI_DMA_SLEEP`).
pub const DDI_DMA_SLEEP: ddi_dma_callback_t = ddi_dma_callback_t(core::ptr::without_provenance(1));

/// The callback tried to allocate again and failed; the system keeps it on
/// the list and calls it again later (`DDI_DMA_CALLBACK_RUNOUT`).
pub const DDI_DMA_CALLBACK_RUNOUT: c_int = 0;
/// The callback succeeded or has given up; the system forgets it
/// (`DDI_DMA_CALLBACK_DONE`).
pub const DDI_DMA_CALLBACK_DONE: c_int = 1;

// --- binding flags -------------------------------------------------------

/// The device writes memory; data moves from the device into memory
/// (`DDI_DMA_READ`).
pub const DDI_DMA_READ: uint_t = 0x0002;
/// The device reads memory; data moves from memory to the device
/// (`DDI_DMA_WRITE`).
pub const DDI_DMA_WRITE: uint_t = 0x0001;
/// Data moves in both directions (`DDI_DMA_RDWR`).
pub const DDI_DMA_RDWR: uint_t = DDI_DMA_READ | DDI_DMA_WRITE;
/// Ask for an MMU red zone after the object to catch a runaway DMA engine.
/// Not every platform can grant it (`DDI_DMA_REDZONE`).
pub const DDI_DMA_REDZONE: uint_t = 0x0004;
/// The caller accepts resources for only part of the object, in which case
/// the bind returns [`DDI_DMA_PARTIAL_MAP`] and the caller moves through the
/// object with [`ddi_dma_getwin`] (`DDI_DMA_PARTIAL`).
pub const DDI_DMA_PARTIAL: uint_t = 0x0008;
/// The object is accessed randomly in small pieces, such as a block of
/// descriptors shared with the device, and synchronisation should be as
/// cheap as possible. Explicit [`ddi_dma_sync`] calls are still needed
/// (`DDI_DMA_CONSISTENT`).
pub const DDI_DMA_CONSISTENT: uint_t = 0x0010;
/// The mapping must not be shared with any other DMA master
/// (`DDI_DMA_EXCLUSIVE`).
pub const DDI_DMA_EXCLUSIVE: uint_t = 0x0020;
/// The object is transferred sequentially in one direction in block sized,
/// block aligned pieces, so the system may use I/O caches and pad or align
/// the memory for large transfers (`DDI_DMA_STREAMING`).
pub const DDI_DMA_STREAMING: uint_t = 0x0040;

// --- cache attributes for ddi_dma_mem_alloc(9F) --------------------------

/// The CPU may cache the memory and write it back later. This is the
/// default when no cache attribute is given (`IOMEM_DATA_CACHED`).
pub const IOMEM_DATA_CACHED: uint_t = 0x10000;
/// The CPU never caches the memory, but writes may be reordered or
/// combined. Falls back to uncached if the platform cannot do it
/// (`IOMEM_DATA_UC_WR_COMBINE`).
pub const IOMEM_DATA_UC_WR_COMBINE: uint_t = 0x20000;
/// The CPU never caches the memory and accesses it in strict order
/// (`IOMEM_DATA_UNCACHED`).
pub const IOMEM_DATA_UNCACHED: uint_t = 0x40000;

// --- synchronisation types -----------------------------------------------

/// Make the device's view of the memory current after the CPU has modified
/// it (`DDI_DMA_SYNC_FORDEV`).
pub const DDI_DMA_SYNC_FORDEV: uint_t = 0x0;
/// Make the CPU's view of the memory current after the device has written
/// it (`DDI_DMA_SYNC_FORCPU`).
pub const DDI_DMA_SYNC_FORCPU: uint_t = 0x1;
/// Like [`DDI_DMA_SYNC_FORCPU`], but only the kernel's mapping matters, so
/// the system may do less work if that is cheaper
/// (`DDI_DMA_SYNC_FORKERNEL`).
pub const DDI_DMA_SYNC_FORKERNEL: uint_t = 0x2;

// --- return values -------------------------------------------------------

/// Resources were allocated for the whole object (`DDI_DMA_MAPPED`).
pub const DDI_DMA_MAPPED: c_int = 0;
/// The mapping is legitimate, from advisory calls; the same value as
/// [`DDI_DMA_MAPPED`] (`DDI_DMA_MAPOK`).
pub const DDI_DMA_MAPOK: c_int = 0;
/// Resources were allocated for only the first window of the object, which
/// is acceptable when [`DDI_DMA_PARTIAL`] was set (`DDI_DMA_PARTIAL_MAP`).
pub const DDI_DMA_PARTIAL_MAP: c_int = 1;
/// The end of the list of windows or segments was reached (`DDI_DMA_DONE`).
pub const DDI_DMA_DONE: c_int = 2;
/// No resources are available at the moment (`DDI_DMA_NORESOURCES`).
pub const DDI_DMA_NORESOURCES: c_int = -1;
/// The device cannot reach the object at all (`DDI_DMA_NOMAPPING`).
pub const DDI_DMA_NOMAPPING: c_int = -2;
/// The object is larger than this system can ever map in one request
/// (`DDI_DMA_TOOBIG`).
pub const DDI_DMA_TOOBIG: c_int = -3;
/// The request is too small to be mapped (`DDI_DMA_TOOSMALL`).
pub const DDI_DMA_TOOSMALL: c_int = -4;
/// Another DMA master has the object locked against mapping
/// (`DDI_DMA_LOCKED`).
pub const DDI_DMA_LOCKED: c_int = -5;
/// The limits given for the mapping are not sensible (`DDI_DMA_BADLIMITS`).
pub const DDI_DMA_BADLIMITS: c_int = -6;
/// The window or segment reference is stale (`DDI_DMA_STALE`).
pub const DDI_DMA_STALE: c_int = -7;
/// The system can never allocate resources for the attributes given, for
/// example because [`DDI_DMA_FORCE_PHYSICAL`] is set on a platform without
/// physical DMA (`DDI_DMA_BADATTR`).
pub const DDI_DMA_BADATTR: c_int = -8;
/// The handle is already bound to another transfer (`DDI_DMA_INUSE`).
pub const DDI_DMA_INUSE: c_int = -9;

unsafe extern "C" {
    // --- handles -----------------------------------------------------------
    /// `ddi_dma_alloc_handle(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates a DMA handle for the device `dip` according to the
    /// attributes in `attr` and stores it through `handlep`. The handle is
    /// an opaque reference to resources allocated later by the bind
    /// functions, may only be used by the device it was allocated for, and
    /// carries one transfer at a time. `waitfp` is [`DDI_DMA_SLEEP`],
    /// [`DDI_DMA_DONTWAIT`] or a callback to run with `arg` when resources
    /// may have become available. Returns `DDI_SUCCESS`, [`DDI_DMA_BADATTR`]
    /// when the attributes can never be satisfied, or
    /// [`DDI_DMA_NORESOURCES`]. May be called from user, kernel or interrupt
    /// context, except that [`DDI_DMA_SLEEP`] is not allowed from interrupt
    /// context.
    pub fn ddi_dma_alloc_handle(
        dip: dev_info_t,
        attr: *const ddi_dma_attr_t,
        waitfp: ddi_dma_callback_t,
        arg: caddr_t,
        handlep: *mut ddi_dma_handle_t,
    ) -> c_int;
    /// `ddi_dma_free_handle(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Destroys the handle that `handlep` points to. Any resources bound to
    /// it must have been released with [`ddi_dma_unbind_handle`] first, and
    /// the handle must not be used afterwards. May be called from user,
    /// kernel or interrupt context.
    pub fn ddi_dma_free_handle(handlep: *mut ddi_dma_handle_t);

    // --- memory ------------------------------------------------------------
    /// `ddi_dma_mem_alloc(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates at least `length` bytes of memory suitable for DMA through
    /// `handle`, honouring the alignment, padding and granularity of the
    /// attributes the handle was created with and any stricter system
    /// requirements. `xfermodes` is [`DDI_DMA_STREAMING`] for sequential
    /// block transfers or [`DDI_DMA_CONSISTENT`] for shared control
    /// structures, optionally combined with one of [`IOMEM_DATA_CACHED`],
    /// [`IOMEM_DATA_UC_WR_COMBINE`] or [`IOMEM_DATA_UNCACHED`]. `accattrp`
    /// gives the access attributes; only the endian flags matter, and only
    /// on SPARC. The kernel address of the memory is stored through
    /// `kaddrp`, the possibly rounded up size through `real_length`, and an
    /// access handle through `handlep`; the driver reads and writes the
    /// memory through that handle with `ddi_get8(9F)` and its relatives.
    /// The memory must still be bound with [`ddi_dma_addr_bind_handle`]
    /// before the device can use it, and synchronised with [`ddi_dma_sync`]
    /// around each transfer. `waitfp` and `arg` follow the usual callback
    /// convention. Returns `DDI_SUCCESS` or `DDI_FAILURE`. May be called
    /// from user, kernel or interrupt context, except that
    /// [`DDI_DMA_SLEEP`] is not allowed from interrupt context.
    pub fn ddi_dma_mem_alloc(
        handle: ddi_dma_handle_t,
        length: size_t,
        accattrp: *const ddi_device_acc_attr_t,
        xfermodes: uint_t,
        waitfp: ddi_dma_callback_t,
        arg: caddr_t,
        kaddrp: *mut caddr_t,
        real_length: *mut size_t,
        handlep: *mut ddi_acc_handle_t,
    ) -> c_int;
    /// `ddi_dma_mem_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees memory allocated by [`ddi_dma_mem_alloc`] and destroys the
    /// access handle that `hp` points to. May be called from user, kernel
    /// or interrupt context.
    pub fn ddi_dma_mem_free(hp: *mut ddi_acc_handle_t);

    // --- binding -----------------------------------------------------------
    /// `ddi_dma_addr_bind_handle(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates DMA resources for the `len` bytes of memory at the virtual
    /// address `addr` so that the device behind `handle` can transfer to or
    /// from them. `r#as` is the address space the memory belongs to and is
    /// NULL for kernel memory, which is the only value drivers pass.
    /// `flags` combines a direction, [`DDI_DMA_READ`], [`DDI_DMA_WRITE`] or
    /// [`DDI_DMA_RDWR`], with [`DDI_DMA_STREAMING`] or
    /// [`DDI_DMA_CONSISTENT`] and optionally [`DDI_DMA_REDZONE`] or
    /// [`DDI_DMA_PARTIAL`]. The bind produces one or more cookies which new
    /// code reads with [`ddi_dma_cookie_iter`] or [`ddi_dma_cookie_get`]
    /// after passing NULL for both `cookiep` and `ccountp`; the two
    /// pointers are only needed by callers of the deprecated
    /// [`ddi_dma_nextcookie`], for which the first cookie and the cookie
    /// count are stored through them. `waitfp` and `arg` follow the usual
    /// callback convention. Returns [`DDI_DMA_MAPPED`],
    /// [`DDI_DMA_PARTIAL_MAP`] when only the first window was mapped,
    /// [`DDI_DMA_INUSE`], [`DDI_DMA_NORESOURCES`], [`DDI_DMA_NOMAPPING`] or
    /// [`DDI_DMA_TOOBIG`]. May be called from user, kernel or interrupt
    /// context, except that [`DDI_DMA_SLEEP`] is not allowed from interrupt
    /// context.
    pub fn ddi_dma_addr_bind_handle(
        handle: ddi_dma_handle_t,
        r#as: *mut addrspace,
        addr: caddr_t,
        len: size_t,
        flags: uint_t,
        waitfp: ddi_dma_callback_t,
        arg: caddr_t,
        cookiep: *mut ddi_dma_cookie_t,
        ccountp: *mut uint_t,
    ) -> c_int;
    /// `ddi_dma_buf_bind_handle(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The same as [`ddi_dma_addr_bind_handle`], but the memory is the data
    /// of the `buf(9S)` structure `bp`, as received by `strategy(9E)`. The
    /// meaning of `flags`, `waitfp`, `arg`, `cookiep` and `ccountp`, the
    /// return values and the calling rules are all as for that function.
    pub fn ddi_dma_buf_bind_handle(
        handle: ddi_dma_handle_t,
        bp: *mut crate::bio::buf,
        flags: uint_t,
        waitfp: ddi_dma_callback_t,
        arg: caddr_t,
        cookiep: *mut ddi_dma_cookie_t,
        ccountp: *mut uint_t,
    ) -> c_int;
    /// `ddi_dma_unbind_handle(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases every DMA resource bound to the handle by
    /// [`ddi_dma_addr_bind_handle`] or [`ddi_dma_buf_bind_handle`], once the
    /// transfer is complete. It performs an implicit [`ddi_dma_sync`], so
    /// no further synchronisation is needed. Returns `DDI_SUCCESS` or
    /// `DDI_FAILURE`. May be called from user, kernel or interrupt context.
    pub fn ddi_dma_unbind_handle(handle: ddi_dma_handle_t) -> c_int;

    // --- synchronisation ---------------------------------------------------
    /// `ddi_dma_sync(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Makes either the device's or the CPU's view of `len` bytes of the
    /// bound object starting at `offset` current, flushing or invalidating
    /// caches and draining write buffers as the platform requires. A `len`
    /// of zero means everything from `offset` to the end. `flags` is
    /// [`DDI_DMA_SYNC_FORDEV`] before the device reads memory the CPU
    /// wrote, [`DDI_DMA_SYNC_FORCPU`] before the CPU reads memory the device
    /// wrote, or [`DDI_DMA_SYNC_FORKERNEL`] as a cheaper variant of the
    /// latter when only the kernel mapping matters. Binding and unbinding
    /// synchronise implicitly; every access in between needs an explicit
    /// call, whatever the consistency flags of the memory. Returns
    /// `DDI_SUCCESS`, or `DDI_FAILURE` when the range lies outside the bound
    /// object. May be called from user, kernel or interrupt context.
    pub fn ddi_dma_sync(handle: ddi_dma_handle_t, offset: off_t, len: size_t, flags: uint_t) -> c_int;
    /// `ddi_check_acc_handle(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether the fault management framework has recorded an
    /// error on the DMA handle. Returns `DDI_SUCCESS` when the handle is
    /// clean and `DDI_FAILURE` when a fault was detected. Only meaningful
    /// for handles allocated with `DDI_DMA_FLAGERR` in their attributes.
    pub fn ddi_check_dma_handle(handle: ddi_dma_handle_t) -> c_int;

    // --- windows -----------------------------------------------------------
    /// `ddi_dma_numwin(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores through `nwinp` the number of windows into which a partially
    /// bound object was divided. Returns `DDI_SUCCESS`, or `DDI_FAILURE`
    /// when the handle was not bound with [`DDI_DMA_PARTIAL`]. May be called
    /// from user, kernel or interrupt context.
    pub fn ddi_dma_numwin(handle: ddi_dma_handle_t, nwinp: *mut uint_t) -> c_int;
    /// `ddi_dma_getwin(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Makes window `win`, numbered from zero up to one less than what
    /// [`ddi_dma_numwin`] reported, the current window of a partially bound
    /// object. The window's offset within the object is stored through
    /// `offp` and its length through `lenp`. Like the bind functions it
    /// produces a fresh set of cookies; new code passes NULL for `cookiep`
    /// and `ccountp` and uses [`ddi_dma_cookie_iter`] or
    /// [`ddi_dma_cookie_get`]. Moving the window handles the resource
    /// synchronisation of the shift itself, but data accesses before and
    /// after still need [`ddi_dma_sync`]. It is typically called from the
    /// interrupt handler that notices one window has completed and starts
    /// the next. Returns `DDI_SUCCESS`, or `DDI_FAILURE` for an invalid
    /// window number. May be called from user, kernel or interrupt context.
    pub fn ddi_dma_getwin(
        handle: ddi_dma_handle_t,
        win: uint_t,
        offp: *mut off_t,
        lenp: *mut size_t,
        cookiep: *mut ddi_dma_cookie_t,
        ccountp: *mut uint_t,
    ) -> c_int;

    // --- cookies -----------------------------------------------------------
    /// `ddi_dma_cookie_iter(9F)`. No interface level given.
    ///
    /// Walks the cookies of a bound handle. Passing NULL as `iter` returns
    /// the first cookie; passing a cookie previously returned by this
    /// function returns the next one; NULL comes back after the last. The
    /// cookie that the bind functions may have stored through `cookiep`
    /// must not be passed in. The returned pointer refers to storage owned
    /// by the handle and stays valid until the handle is unbound or its
    /// window changes. The `ddi_dma_cookie_iter(9F)` page carries no
    /// INTERFACE LEVEL section; the level given here is that of every other
    /// DMA page, and the function is in the `ddi` mapfile. May be called
    /// from user, kernel or interrupt context.
    pub fn ddi_dma_cookie_iter(handle: ddi_dma_handle_t, iter: *const ddi_dma_cookie_t) -> *const ddi_dma_cookie_t;
    /// `ddi_dma_cookie_get(9F)`. No interface level given.
    ///
    /// Returns the cookie at position `index`, counting from zero, or NULL
    /// if there is no such cookie. Together with [`ddi_dma_ncookies`] it
    /// lets a driver index cookies directly instead of iterating. The
    /// returned pointer has the same lifetime as one from
    /// [`ddi_dma_cookie_iter`]. The page it is documented on carries no
    /// INTERFACE LEVEL section. May be called from user, kernel or
    /// interrupt context.
    pub fn ddi_dma_cookie_get(handle: ddi_dma_handle_t, index: uint_t) -> *const ddi_dma_cookie_t;
    /// `ddi_dma_cookie_one(9F)`. No interface level given.
    ///
    /// Returns the single cookie of a handle that is known to have exactly
    /// one, for example because its attributes allow no scatter/gather. It
    /// never returns NULL; if the handle has any other number of cookies
    /// the system panics. The page it is documented on carries no INTERFACE
    /// LEVEL section, and the function is in the `ddi` mapfile. May be
    /// called from user, kernel or interrupt context.
    pub fn ddi_dma_cookie_one(handle: ddi_dma_handle_t) -> *const ddi_dma_cookie_t;
    /// `ddi_dma_ncookies(9F)`. No interface level given.
    ///
    /// Returns how many cookies are bound to the handle, or zero if nothing
    /// is bound. A driver need not keep the count from the bind call; it
    /// can ask again at any time. The header and the kernel declare the
    /// return type as `int` although the manual page shows `uint_t`. The
    /// page it is documented on carries no INTERFACE LEVEL section. May be
    /// called from user, kernel or interrupt context.
    pub fn ddi_dma_ncookies(handle: ddi_dma_handle_t) -> c_int;
    /// `ddi_dma_nextcookie(9F)`. No interface level given.
    ///
    /// The historical way to read cookies, retained because existing
    /// drivers and the `ddi` mapfile still refer to it. After a bind that
    /// stored the first cookie and the count through `cookiep` and
    /// `ccountp`, each call copies the next cookie into `cookiep`. It
    /// mutates the handle, so a cookie cannot be read twice without
    /// rebinding, and it cannot report that the cookies are exhausted;
    /// calling it more times than there are cookies has undefined results.
    /// The manual advises against using it for any purpose and points to
    /// [`ddi_dma_cookie_iter`], [`ddi_dma_cookie_get`] and
    /// [`ddi_dma_cookie_one`] instead. May be called from user, kernel or
    /// interrupt context.
    pub fn ddi_dma_nextcookie(handle: ddi_dma_handle_t, cookiep: *mut ddi_dma_cookie_t);

    // --- burst sizes -------------------------------------------------------
    /// `ddi_dma_burstsizes(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the burst sizes the device may actually use for the bound
    /// mapping, in the same bit mask encoding as `dma_attr_burstsizes` of
    /// [`ddi_dma_attr_t`] but after every intervening bus layer has imposed
    /// its own limits. A driver programs its engine with this value rather
    /// than with what it asked for. May be called from user or interrupt
    /// context.
    pub fn ddi_dma_burstsizes(handle: ddi_dma_handle_t) -> c_int;
    /// `ddi_dma_set_sbus64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks for 64-bit data transfers on an SBus, giving in `burstsizes`
    /// the burst sizes the engine can produce in 64-bit mode, which may
    /// differ from the 32-bit sizes in the attributes the handle was
    /// allocated with. If the bus supports it, [`ddi_dma_burstsizes`] will
    /// afterwards report the 64-bit sizes; otherwise the bus stays in
    /// 32-bit mode. This only matters on SPARC machines with an SBus; on
    /// amd64 there is no such bus and the call has nothing to do. The
    /// header declares `burstsizes` as `ulong_t` although the manual page
    /// shows `uint_t`. Returns `DDI_SUCCESS` or `DDI_FAILURE`. May be called
    /// from user, kernel or interrupt context.
    pub fn ddi_dma_set_sbus64(handle: ddi_dma_handle_t, burstsizes: ulong_t) -> c_int;
}
