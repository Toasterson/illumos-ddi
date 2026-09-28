//! Direct memory access with the rules of the `ddi_dma_*(9F)` family in the
//! types.
//!
//! In C, setting up DMA is a sequence of calls whose order the manual pages
//! describe in prose. A driver describes its DMA engine in a
//! `ddi_dma_attr(9S)` structure, allocates a handle with
//! `ddi_dma_alloc_handle(9F)`, allocates memory for that handle with
//! `ddi_dma_mem_alloc(9F)`, binds the memory to the handle with
//! `ddi_dma_addr_bind_handle(9F)`, reads the device addresses out of the
//! handle as cookies, programs them into the device, and synchronises the
//! views of the CPU and the device with `ddi_dma_sync(9F)` around each
//! access. Teardown runs the same chain backwards with
//! `ddi_dma_unbind_handle(9F)`, `ddi_dma_mem_free(9F)` and
//! `ddi_dma_free_handle(9F)`. This module gives each link of the chain a
//! type, and the types only allow the calls in the order the manual demands.
//!
//! # Attributes
//!
//! [`DmaAttr`] wraps a `ddi_dma_attr_t`. It is built with `const fn`
//! methods, so a driver can keep its attributes in a `const` or a `static`
//! just as a C driver does. [`DmaAttr::bits64`] describes an engine that can
//! reach all of memory with a single, page aligned cookie, and
//! [`DmaAttr::bits32`] is the same for an engine limited to the first four
//! gigabytes. The other methods change one field each and are named after
//! what the field means rather than after the C member. The version field is
//! always `DMA_ATTR_V0`, the only version there is. A few rules that the
//! manual states for the fields, such as that the alignment and the minimum
//! transfer size are powers of two and that a scatter/gather length of zero
//! is reserved, are checked before the attributes reach the kernel and
//! reported as [`DmaError::BadAttr`], the same error the kernel reports for
//! attributes it cannot satisfy.
//!
//! # Handles
//!
//! [`DmaHandle`] owns a `ddi_dma_handle_t` from `ddi_dma_alloc_handle(9F)`
//! and frees it with `ddi_dma_free_handle(9F)` when dropped. The manual says
//! that a handle carries one transfer at a time and that "a DMA handle must
//! only be used by the device for which it was allocated". The handle is
//! created from the driver's [`OwnNode`], which only the attach glue hands
//! out, rather than from a [`DevInfo`](crate::DevInfo), and it is consumed
//! by the buffer that is allocated for it, so it cannot be shared between
//! two buffers or bound twice.
//!
//! Allocating against another node is the misuse that panicked a test
//! kernel on 28 September 2026: a handle allocated against
//! `ddi_root_node(9F)` succeeds, and the first bind calls through a NULL
//! function pointer. For code that obtains a foreign node with the `unsafe`
//! [`OwnNode::from_raw`], [`DmaHandle::new`] refuses the root node and nodes
//! bound to other drivers with [`DmaError::NotOwnNode`] before it calls the
//! kernel. A pseudo device, whose parent is the `pseudo` nexus, gets
//! [`DmaError::BadAttr`] from its own node, because that nexus does no DMA.
//!
//! # Memory
//!
//! [`DmaBuffer`] owns memory from `ddi_dma_mem_alloc(9F)` together with the
//! handle it was allocated for. The allocation honours the alignment,
//! padding and granularity in the attributes, and the kernel may round the
//! length up. The buffer always reports the real length. The transfer mode
//! is chosen with [`DmaMode`]. Use [`DmaMode::Consistent`] for control
//! structures the device and the driver both read and write at random, such
//! as descriptor rings, and [`DmaMode::Streaming`] for data that moves in
//! one direction in large blocks. The CPU cache attribute is chosen with
//! [`DmaCache`].
//!
//! The kernel does not initialise the memory, and Rust may not read
//! uninitialised bytes, so this module fills the memory with zeroes before
//! handing out the buffer.
//!
//! `ddi_dma_mem_alloc(9F)` also takes device access attributes. Only their
//! byte order matters, and the manual warns that when a swapping byte order
//! is requested the memory may only be accessed through `ddi_get8(9F)` and
//! its relatives. Because this module hands out the memory as ordinary byte
//! slices, it always passes `DDI_NEVERSWAP_ACC`. A driver for a device whose
//! structures have a fixed byte order converts the fields itself, for
//! example with the `zerocopy` byte order types.
//!
//! While the buffer is not bound, the device has no address for it and the
//! driver may use it freely through [`DmaBuffer::as_slice`],
//! [`DmaBuffer::as_mut_slice`], [`DmaBuffer::view`] and
//! [`DmaBuffer::view_mut`]. Dropping the buffer frees the memory with
//! `ddi_dma_mem_free(9F)` and then the handle with `ddi_dma_free_handle(9F)`,
//! in that order.
//!
//! # Binding
//!
//! [`DmaBuffer::bind`] calls `ddi_dma_addr_bind_handle(9F)` for the whole
//! buffer and returns a [`DmaBinding`]. The direction of the transfer is a
//! [`DmaDirection`], and the consistency flag passed with it is the mode the
//! buffer was allocated with, as the manual for `ddi_dma_mem_alloc(9F)`
//! requires. The binding borrows the buffer mutably for as long as it
//! exists. While it exists, the buffer's own accessors cannot be called, so
//! the only way for the CPU to reach the memory is through the binding's
//! synchronising methods. Dropping the binding calls
//! `ddi_dma_unbind_handle(9F)`, which performs an implicit synchronisation,
//! after which the buffer is available again. [`DmaBinding::unbind`] does
//! the same and reports the result.
//!
//! Partial mappings are not supported. The bind never sets
//! `DDI_DMA_PARTIAL`, so the kernel either maps the whole buffer or fails.
//! If a kernel nevertheless reports `DDI_DMA_PARTIAL_MAP`, the resources are
//! released again and the bind fails with [`DmaError::PartialMap`]. The
//! window functions `ddi_dma_numwin(9F)` and `ddi_dma_getwin(9F)` are
//! therefore not wrapped. Binding memory that the buffer does not own, such
//! as the data of a `buf(9S)`, is not wrapped either.
//!
//! # Cookies
//!
//! A bound buffer is described to the device by one or more cookies, each a
//! device visible address and a length. [`DmaBinding::cookies`] iterates
//! over them with `ddi_dma_cookie_iter(9F)`, [`DmaBinding::cookie`] fetches
//! one by index with `ddi_dma_cookie_get(9F)`, and [`DmaBinding::ncookies`]
//! reports how many there are. [`DmaBinding::single_cookie`] replaces
//! `ddi_dma_cookie_one(9F)`, which panics the system when the count is not
//! one, with a method that returns `None` instead. The deprecated
//! `ddi_dma_nextcookie(9F)` is not used. A cookie is plain data, but the
//! address in it is only meaningful while the binding exists, which is why
//! the cookie accessors are on the binding.
//!
//! # Synchronisation and direction
//!
//! The manual for `ddi_dma_sync(9F)` says that between binding and unbinding,
//! every change made by one side must be synchronised before the other side
//! looks at it, whatever the consistency mode of the memory. On a system that
//! uses bounce buffers, synchronisation is what copies the data, so a missing
//! call loses data rather than merely delaying it. The binding offers these
//! methods for it.
//!
//! - [`DmaBinding::sync_for_device`] calls `ddi_dma_sync(9F)` with
//!   `DDI_DMA_SYNC_FORDEV` after the CPU has written memory that the device
//!   will read.
//! - [`DmaBinding::sync_for_cpu`] calls it with `DDI_DMA_SYNC_FORCPU` after
//!   the device has written memory that the CPU will read, and returns a
//!   shared view of the buffer that lives until the binding is used again.
//! - [`DmaBinding::update`] synchronises for the CPU, lets a closure read and
//!   write the buffer, and synchronises for the device, which is the pattern
//!   for a descriptor ring that stays bound.
//! - The `_range` variants of the first two do the same for a part of the
//!   buffer.
//!
//! Binding synchronises implicitly for the device, so data that the device
//! is to read can simply be written into the buffer before it is bound.
//! Unbinding synchronises implicitly for the CPU, so data that the device
//! wrote can be read from the buffer after the binding is dropped.
//!
//! # Blocking
//!
//! Allocating a handle, allocating memory and binding may wait for resources
//! when the kernel is told `DDI_DMA_SLEEP`, which the manual forbids in
//! interrupt context. The methods that pass it, [`DmaHandle::new`],
//! [`DmaBuffer::new`] and [`DmaBuffer::bind`], take a [`Sleepable`] token.
//! Their counterparts [`DmaHandle::try_new`], [`DmaBuffer::try_new`] and
//! [`DmaBuffer::try_bind`] pass `DDI_DMA_DONTWAIT`, need no token, and fail
//! with [`DmaError::NoResources`] or [`DmaError::Failure`] when resources
//! are short. The callback form of the resource argument is not offered.
//! Synchronising, reading cookies, unbinding and freeing never block and
//! need no token.
//!
//! # What the types cannot promise
//!
//! A device that is told an address can write to it at any time, and a
//! device that is told a wrong address can write anywhere in memory. No type
//! in this crate can prevent that, and programming a device is a safe
//! operation here, as `docs/DESIGN.md` explains for registers. The types
//! guarantee that memory handed to the device stays allocated and mapped for
//! as long as the device may use it, that it is released in the documented
//! order, and that the CPU sees it only through a synchronisation call.
//! Whether the device has actually finished writing when the driver calls
//! [`DmaBinding::sync_for_cpu`] is part of the device's protocol, which the
//! driver knows and the crate does not.
//!
//! # Example
//!
//! ```ignore
//! const ATTR: DmaAttr = DmaAttr::bits64().max_cookies(1);
//!
//! // `node` is the `OwnNode` that `Driver::attach` received.
//! let handle = DmaHandle::new(&node, &ATTR, ctx)?;
//! let mut ring = DmaBuffer::new(handle, 4096, DmaMode::Consistent, DmaCache::Cached, ctx)?;
//! let mut binding = ring.bind(DmaDirection::Bidirectional, ctx)?;
//! let base = binding.single_cookie().ok_or(DmaError::TooBig)?.address;
//! // Program `base` into the device, then exchange descriptors with
//! // `binding.update(|bytes| ...)` and `binding.sync_for_cpu()`.
//! ```

use core::fmt;
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::context::Sleepable;
use crate::device::OwnNode;
use crate::error::Errno;
use crate::sys;

/// The MMU page size on amd64, `MMU_PAGESIZE` in `sys/machparam.h`, which is
/// the alignment of the default attributes.
const PAGE_SIZE: u64 = 0x1000;

// --- errors ----------------------------------------------------------------

/// What went wrong in a DMA call.
///
/// Most variants stand for one of the `DDI_DMA_*` return codes that
/// `ddi_dma_alloc_handle(9F)` and `ddi_dma_addr_bind_handle(9F)` document.
/// `ddi_dma_mem_alloc(9F)` and `ddi_dma_sync(9F)` only report `DDI_FAILURE`,
/// which becomes [`DmaError::Failure`]. A few variants describe problems this
/// module detects before calling the kernel.
///
/// The `DDI_DMA_*` codes share their numeric values with the general
/// `DDI_E*` codes of the interrupt functions, so `DDI_DMA_TOOSMALL` and
/// `DDI_EINVAL` are both -4. The DMA manual pages document only the
/// `DDI_DMA_*` meaning, and that is how values are interpreted here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaError {
    /// No resources are available at the moment. Waiting with a
    /// [`Sleepable`] token or trying again later may succeed
    /// (`DDI_DMA_NORESOURCES`).
    NoResources,
    /// The device cannot reach the memory with the attributes it was given
    /// (`DDI_DMA_NOMAPPING`).
    NoMapping,
    /// The request is larger than this system can ever map at once, or needs
    /// more cookies than the attributes allow (`DDI_DMA_TOOBIG`).
    TooBig,
    /// The request is too small to be mapped (`DDI_DMA_TOOSMALL`).
    TooSmall,
    /// Another DMA master has the memory locked against mapping
    /// (`DDI_DMA_LOCKED`).
    Locked,
    /// The limits given for the mapping are not sensible
    /// (`DDI_DMA_BADLIMITS`).
    BadLimits,
    /// A window or segment reference is stale (`DDI_DMA_STALE`).
    Stale,
    /// The attributes can never be satisfied on this system
    /// (`DDI_DMA_BADATTR`). This module also reports it for attributes that
    /// break a rule of `ddi_dma_attr(9S)`, such as an alignment that is not
    /// a power of two.
    BadAttr,
    /// The handle is already bound to another transfer (`DDI_DMA_INUSE`).
    InUse,
    /// The kernel mapped only part of the buffer (`DDI_DMA_PARTIAL_MAP`).
    /// Partial mappings are not supported, so the resources have already
    /// been released again when this error is returned.
    PartialMap,
    /// A function in the `DDI_SUCCESS` and `DDI_FAILURE` convention failed
    /// (`DDI_FAILURE`). `ddi_dma_mem_alloc(9F)` reports this when memory is
    /// short or the request is invalid, and `ddi_dma_sync(9F)` when the range
    /// lies outside the bound memory.
    Failure,
    /// A buffer of zero bytes was requested. The kernel cannot allocate or
    /// bind an empty object, so this module refuses before asking it.
    InvalidLength,
    /// A range given to a synchronisation method lies outside the buffer.
    /// This module checks the range before calling `ddi_dma_sync(9F)`.
    OutOfRange,
    /// The node a handle was requested for cannot be the driver's own
    /// attached node: it is the root node, or it is bound to another driver.
    /// Only an [`OwnNode`] made with the `unsafe` [`OwnNode::from_raw`] can
    /// get here. This module refuses before calling
    /// `ddi_dma_alloc_handle(9F)`, because a handle for the root node would
    /// panic the system in `ddi_dma_addr_bind_handle(9F)`.
    NotOwnNode,
    /// A return code this crate does not know.
    Other(i32),
}

impl DmaError {
    /// Interprets a return code in the `DDI_DMA_*` convention of
    /// `ddi_dma_alloc_handle(9F)` and `ddi_dma_addr_bind_handle(9F)`, where
    /// `DDI_SUCCESS` and `DDI_DMA_MAPPED` are both zero.
    fn check(raw: i32) -> Result<(), DmaError> {
        Err(match raw {
            sys::DDI_DMA_MAPPED => return Ok(()),
            sys::DDI_DMA_PARTIAL_MAP => DmaError::PartialMap,
            sys::DDI_DMA_NORESOURCES => DmaError::NoResources,
            sys::DDI_DMA_NOMAPPING => DmaError::NoMapping,
            sys::DDI_DMA_TOOBIG => DmaError::TooBig,
            sys::DDI_DMA_TOOSMALL => DmaError::TooSmall,
            sys::DDI_DMA_LOCKED => DmaError::Locked,
            sys::DDI_DMA_BADLIMITS => DmaError::BadLimits,
            sys::DDI_DMA_STALE => DmaError::Stale,
            sys::DDI_DMA_BADATTR => DmaError::BadAttr,
            sys::DDI_DMA_INUSE => DmaError::InUse,
            other => DmaError::Other(other),
        })
    }

    /// Interprets a return code in the `DDI_SUCCESS` and `DDI_FAILURE`
    /// convention of `ddi_dma_mem_alloc(9F)`, `ddi_dma_sync(9F)` and
    /// `ddi_dma_unbind_handle(9F)`.
    fn check_ddi(raw: i32) -> Result<(), DmaError> {
        if raw == sys::DDI_SUCCESS { Ok(()) } else { Err(DmaError::Failure) }
    }
}

impl fmt::Display for DmaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DmaError::NoResources => f.write_str("DDI_DMA_NORESOURCES"),
            DmaError::NoMapping => f.write_str("DDI_DMA_NOMAPPING"),
            DmaError::TooBig => f.write_str("DDI_DMA_TOOBIG"),
            DmaError::TooSmall => f.write_str("DDI_DMA_TOOSMALL"),
            DmaError::Locked => f.write_str("DDI_DMA_LOCKED"),
            DmaError::BadLimits => f.write_str("DDI_DMA_BADLIMITS"),
            DmaError::Stale => f.write_str("DDI_DMA_STALE"),
            DmaError::BadAttr => f.write_str("DDI_DMA_BADATTR"),
            DmaError::InUse => f.write_str("DDI_DMA_INUSE"),
            DmaError::PartialMap => f.write_str("DDI_DMA_PARTIAL_MAP (partial mappings are not supported)"),
            DmaError::Failure => f.write_str("DDI_FAILURE"),
            DmaError::InvalidLength => f.write_str("zero-length DMA buffer"),
            DmaError::OutOfRange => f.write_str("DMA range outside the buffer"),
            DmaError::NotOwnNode => f.write_str("DMA handle requested for a node that is not the driver's own"),
            DmaError::Other(c) => write!(f, "DDI_DMA_* code {c}"),
        }
    }
}

impl From<DmaError> for Errno {
    /// Maps a DMA error to the errno a `cb_ops(9S)` entry point returns for
    /// it: `ENOMEM` when resources are short or the request is too large, `EINVAL` for a request that can never succeed as made, `EFAULT`
    /// when the device cannot reach the memory, `EBUSY` for a handle or memory
    /// in use, `ENXIO` for [`DmaError::NotOwnNode`], and `EIO` otherwise.
    fn from(e: DmaError) -> Errno {
        match e {
            DmaError::NoResources | DmaError::TooBig => Errno::ENOMEM,
            DmaError::BadAttr
            | DmaError::BadLimits
            | DmaError::TooSmall
            | DmaError::InvalidLength
            | DmaError::OutOfRange => Errno::EINVAL,
            DmaError::NoMapping => Errno::EFAULT,
            DmaError::InUse | DmaError::Locked => Errno::EBUSY,
            DmaError::NotOwnNode => Errno::ENXIO,
            DmaError::PartialMap | DmaError::Stale | DmaError::Failure | DmaError::Other(_) => Errno::EIO,
        }
    }
}

// --- attributes ------------------------------------------------------------

/// The addressing capabilities of a DMA engine, a `ddi_dma_attr(9S)`.
///
/// The value is built from one of the two defaults with `const fn` methods
/// that each change one field, so it can be kept in a `const` or a `static`.
/// The kernel copies the attributes when a handle is allocated, so the value
/// need not outlive the call to [`DmaHandle::new`].
///
/// The defaults are these.
///
/// | Field | [`DmaAttr::bits64`] | [`DmaAttr::bits32`] |
/// | --- | --- | --- |
/// | address range | all 64 bits | the first 4 GiB |
/// | counter maximum | unlimited | `0xffff_ffff` |
/// | alignment | 4096, one page | 4096, one page |
/// | burst sizes | `0xfff`, 1 byte to 2 KiB | `0xfff` |
/// | minimum transfer | 1 byte | 1 byte |
/// | maximum transfer | unlimited | `0xffff_ffff` |
/// | segment boundary | none | 4 GiB |
/// | scatter/gather length | 1, a single cookie | 1 |
/// | granularity | 1 byte | 1 byte |
/// | flags | none | none |
///
/// A single cookie means that a bound buffer is physically contiguous as
/// the device sees it, which is what a device without scatter/gather needs
/// and what makes [`DmaBinding::single_cookie`] succeed. Drivers whose
/// engines take a list of addresses raise the limit with
/// [`DmaAttr::max_cookies`] or [`DmaAttr::unlimited_cookies`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct DmaAttr(sys::ddi_dma_attr_t);

impl DmaAttr {
    /// Attributes for an engine that can reach any 64-bit address and wants
    /// a single, page aligned cookie. The table in the type documentation
    /// lists every field.
    pub const fn bits64() -> DmaAttr {
        DmaAttr(sys::ddi_dma_attr_t {
            dma_attr_version: sys::DMA_ATTR_V0,
            dma_attr_addr_lo: 0,
            dma_attr_addr_hi: u64::MAX,
            dma_attr_count_max: u64::MAX,
            dma_attr_align: PAGE_SIZE,
            dma_attr_burstsizes: 0xfff,
            dma_attr_minxfer: 1,
            dma_attr_maxxfer: u64::MAX,
            dma_attr_seg: u64::MAX,
            dma_attr_sgllen: 1,
            dma_attr_granular: 1,
            dma_attr_flags: 0,
        })
    }

    /// Attributes for an engine whose address and counter registers are 32
    /// bits wide, so that every cookie lies below 4 GiB and none crosses a 4
    /// GiB boundary. Otherwise the same as [`DmaAttr::bits64`].
    pub const fn bits32() -> DmaAttr {
        let mut a = DmaAttr::bits64().0;
        a.dma_attr_addr_hi = 0xffff_ffff;
        a.dma_attr_count_max = 0xffff_ffff;
        a.dma_attr_maxxfer = 0xffff_ffff;
        a.dma_attr_seg = 0xffff_ffff;
        DmaAttr(a)
    }

    /// Wraps a raw attribute structure, for drivers that port a C table
    /// field by field. The structure is still checked when a handle is
    /// allocated.
    pub const fn from_raw(raw: sys::ddi_dma_attr_t) -> DmaAttr {
        DmaAttr(raw)
    }

    /// Returns the raw attribute structure.
    pub const fn as_raw(&self) -> &sys::ddi_dma_attr_t {
        &self.0
    }

    /// Sets the lowest and the highest address the engine can reach, both
    /// inclusive (`dma_attr_addr_lo` and `dma_attr_addr_hi`). The low bound
    /// must be below the high bound.
    pub const fn address_range(mut self, lo: u64, hi: u64) -> DmaAttr {
        self.0.dma_attr_addr_lo = lo;
        self.0.dma_attr_addr_hi = hi;
        self
    }

    /// Sets the largest value the engine's transfer counter can hold, as an
    /// inclusive bound, so that a 24-bit counter is `0xff_ffff`
    /// (`dma_attr_count_max`). Longer objects are split into several
    /// cookies.
    pub const fn count_max(mut self, max: u64) -> DmaAttr {
        self.0.dma_attr_count_max = max;
        self
    }

    /// Sets the alignment of the allocated memory and of every cookie, in
    /// bytes (`dma_attr_align`). It must be a power of two. Most engines
    /// need 1; the defaults use a page.
    pub const fn align(mut self, bytes: u64) -> DmaAttr {
        self.0.dma_attr_align = bytes;
        self
    }

    /// Sets the burst sizes the engine can produce, as a bit mask in which
    /// bit `n` stands for a burst of `2^n` bytes, so that bursts of 1, 2, 4
    /// and 16 bytes are `0x17` (`dma_attr_burstsizes`). The system may
    /// reduce the set, and the driver programs its engine with what
    /// [`DmaBinding::burst_sizes`] reports.
    pub const fn burst_sizes(mut self, mask: u32) -> DmaAttr {
        self.0.dma_attr_burstsizes = mask;
        self
    }

    /// Sets the smallest amount of memory a single access of the engine
    /// touches, in bytes (`dma_attr_minxfer`). It must be a power of two and
    /// affects the alignment and padding chosen by `ddi_dma_mem_alloc(9F)`.
    pub const fn min_transfer(mut self, bytes: u32) -> DmaAttr {
        self.0.dma_attr_minxfer = bytes;
        self
    }

    /// Sets the largest amount of memory the engine can move in one
    /// transfer, in bytes (`dma_attr_maxxfer`).
    pub const fn max_transfer(mut self, bytes: u64) -> DmaAttr {
        self.0.dma_attr_maxxfer = bytes;
        self
    }

    /// Sets the boundary that no cookie may cross, as an inclusive mask, so
    /// that `0xffff` keeps every cookie within one 64 KiB region
    /// (`dma_attr_seg`).
    pub const fn segment_boundary(mut self, mask: u64) -> DmaAttr {
        self.0.dma_attr_seg = mask;
        self
    }

    /// Sets the length of the engine's scatter/gather list, which is the
    /// largest number of cookies one binding may have (`dma_attr_sgllen`).
    /// One means the engine cannot scatter or gather. Zero is reserved by
    /// the manual and rejected when a handle is allocated. Values above
    /// `i32::MAX` are treated as `i32::MAX`.
    pub const fn max_cookies(mut self, n: u32) -> DmaAttr {
        self.0.dma_attr_sgllen = if n > i32::MAX as u32 { i32::MAX } else { n as i32 };
        self
    }

    /// Declares that the engine has no limit on the number of cookies, for
    /// example because it chains descriptors, by storing a negative
    /// scatter/gather length (`dma_attr_sgllen`).
    pub const fn unlimited_cookies(mut self) -> DmaAttr {
        self.0.dma_attr_sgllen = -1;
        self
    }

    /// Sets the granularity of the device's transfer size, in bytes,
    /// normally its sector size (`dma_attr_granular`). Every cookie is a
    /// multiple of it, and so must be the length of every buffer that is
    /// bound.
    pub const fn granularity(mut self, bytes: u32) -> DmaAttr {
        self.0.dma_attr_granular = bytes;
        self
    }

    /// Asks for physical rather than IOMMU addresses on platforms that offer
    /// both (`DDI_DMA_FORCE_PHYSICAL`). If the platform cannot provide
    /// them, allocating the handle fails with [`DmaError::BadAttr`], and the
    /// manual tells the driver to retry without the flag.
    pub const fn force_physical(mut self) -> DmaAttr {
        self.0.dma_attr_flags |= sys::DDI_DMA_FORCE_PHYSICAL;
        self
    }

    /// Declares that transfers through the handle need not be ordered with
    /// respect to each other or to other handles, which may let the host
    /// bridge work faster (`DDI_DMA_RELAXED_ORDERING`). The manual asks
    /// devices that can mark individual transactions as relaxed to do that
    /// instead.
    pub const fn relaxed_ordering(mut self) -> DmaAttr {
        self.0.dma_attr_flags |= sys::DDI_DMA_RELAXED_ORDERING;
        self
    }

    /// Declares that the driver copes with DMA faults and checks its handles
    /// with `ddi_fm_dma_err_get(9F)`, so that the system may mark a faulted
    /// handle instead of panicking (`DDI_DMA_FLAGERR`). This crate does not
    /// wrap fault management yet, so a driver that sets the flag must make
    /// those checks through the raw handle.
    pub const fn flag_errors(mut self) -> DmaAttr {
        self.0.dma_attr_flags |= sys::DDI_DMA_FLAGERR;
        self
    }

    /// Checks the rules of `ddi_dma_attr(9S)` that hold on every platform.
    /// The kernel checks the platform specific ones itself.
    fn check(&self) -> Result<(), DmaError> {
        let a = &self.0;
        let ok = a.dma_attr_version == sys::DMA_ATTR_V0
            && a.dma_attr_addr_lo < a.dma_attr_addr_hi
            && a.dma_attr_align.is_power_of_two()
            && a.dma_attr_minxfer.is_power_of_two()
            && a.dma_attr_sgllen != 0
            && a.dma_attr_granular != 0;
        if ok { Ok(()) } else { Err(DmaError::BadAttr) }
    }
}

impl Default for DmaAttr {
    /// The same as [`DmaAttr::bits64`].
    fn default() -> DmaAttr {
        DmaAttr::bits64()
    }
}

// --- handles ---------------------------------------------------------------

/// A DMA handle from `ddi_dma_alloc_handle(9F)`, freed with
/// `ddi_dma_free_handle(9F)` when dropped.
///
/// A handle stands for the resources of one transfer of one device. On its
/// own it only serves to allocate a [`DmaBuffer`], which takes ownership of
/// it. The handle is created from the driver's [`OwnNode`] and, like a
/// [`MinorNode`], belongs in the instance state, which is destroyed before
/// the node goes away.
///
/// [`MinorNode`]: crate::device::MinorNode
pub struct DmaHandle {
    raw: sys::ddi_dma_handle_t,
}

// SAFETY: A DMA handle has no affinity to the thread that allocated it. The
// manual allows every function that takes it to be called from any user or
// kernel thread, and the handle is freed by whichever thread drops it.
unsafe impl Send for DmaHandle {}
// SAFETY: A shared reference only yields the raw pointer. Every operation
// that changes the state of the handle goes through the owning value or a
// mutable reference, so no two threads can change it at the same time.
unsafe impl Sync for DmaHandle {}

impl DmaHandle {
    /// Allocates a handle for the driver's own node with
    /// `ddi_dma_alloc_handle(9F)` and `DDI_DMA_SLEEP`, waiting for resources
    /// if necessary, which is why a [`Sleepable`] token is required.
    ///
    /// The manual defines the node argument as "a pointer to the device's
    /// dev_info structure" and says that "a DMA handle must only be used by
    /// the device for which it was allocated", which is why this takes the
    /// [`OwnNode`] that [`Driver::attach`](crate::Driver::attach) received
    /// and not any [`DevInfo`](crate::DevInfo).
    ///
    /// Fails with [`DmaError::BadAttr`] when the attributes can never be
    /// satisfied. A pseudo device gets `BadAttr` from its own node as well,
    /// because the `pseudo` nexus refuses every DMA handle
    /// (`ddi_no_dma_allochdl`), as observed on illumos `cda5b9a49f`. Fails
    /// with [`DmaError::NotOwnNode`], without calling the kernel, when the
    /// node is the root node or is bound to another driver, which only an
    /// `unsafe` [`OwnNode::from_raw`] can produce.
    pub fn new(node: &OwnNode, attr: &DmaAttr, _ctx: &Sleepable) -> Result<DmaHandle, DmaError> {
        Self::alloc(node, attr, sys::DDI_DMA_SLEEP)
    }

    /// Allocates a handle for the driver's own node with
    /// `ddi_dma_alloc_handle(9F)` and `DDI_DMA_DONTWAIT`. This is allowed in
    /// any context and fails with [`DmaError::NoResources`] when resources
    /// are short. Otherwise the same as [`DmaHandle::new`], including the
    /// own-node rule.
    pub fn try_new(node: &OwnNode, attr: &DmaAttr) -> Result<DmaHandle, DmaError> {
        Self::alloc(node, attr, sys::DDI_DMA_DONTWAIT)
    }

    fn alloc(node: &OwnNode, attr: &DmaAttr, wait: sys::ddi_dma_callback_t) -> Result<DmaHandle, DmaError> {
        attr.check()?;
        // The guard for the unsafe bypass of the type rule: the root node
        // allocates through its own nexus operations and then panics in
        // ddi_dma_addr_bind_handle (NULL devi_bus_dma_bindfunc).
        if !node.is_plausibly_own() {
            return Err(DmaError::NotOwnNode);
        }
        let mut raw: sys::ddi_dma_handle_t = ptr::null_mut();
        // SAFETY: The node is valid for as long as the `OwnNode` exists, the
        // attributes are a valid structure that the kernel copies, the
        // callback argument is one of the two sentinels, and `raw` is
        // writable.
        let r = unsafe { sys::ddi_dma_alloc_handle(node.as_raw(), attr.as_raw(), wait, ptr::null_mut(), &mut raw) };
        DmaError::check(r)?;
        if raw.is_null() {
            return Err(DmaError::Failure);
        }
        Ok(DmaHandle { raw })
    }

    /// Returns the raw handle, for `9F` functions that are not wrapped yet.
    /// It must not be freed, bound or unbound through the pointer.
    pub fn as_raw(&self) -> sys::ddi_dma_handle_t {
        self.raw
    }
}

impl Drop for DmaHandle {
    fn drop(&mut self) {
        // SAFETY: The handle came from ddi_dma_alloc_handle and is freed only
        // here. The buffer that owns it unbinds it and frees its memory in
        // its own Drop, which runs before this one.
        unsafe { sys::ddi_dma_free_handle(&mut self.raw) }
    }
}

// --- memory ----------------------------------------------------------------

/// How the device will access a [`DmaBuffer`], the transfer mode that
/// `ddi_dma_mem_alloc(9F)` and `ddi_dma_addr_bind_handle(9F)` take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaMode {
    /// Small objects accessed at random by both sides, such as descriptor
    /// rings and other structures shared with the device. Synchronisation
    /// is made as cheap as the platform allows, but it is still required
    /// (`DDI_DMA_CONSISTENT`).
    Consistent,
    /// Data that moves sequentially in one direction in block sized, block
    /// aligned pieces. The system may pad and align the memory to use I/O
    /// caches, and the real length may be larger than requested
    /// (`DDI_DMA_STREAMING`).
    Streaming,
}

impl DmaMode {
    fn flag(self) -> sys::uint_t {
        match self {
            DmaMode::Consistent => sys::DDI_DMA_CONSISTENT,
            DmaMode::Streaming => sys::DDI_DMA_STREAMING,
        }
    }
}

/// How the CPU caches a [`DmaBuffer`], the cache attribute that
/// `ddi_dma_mem_alloc(9F)` takes. The manual allows exactly one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DmaCache {
    /// The CPU may cache the memory and write it back later. This is the
    /// kernel's default and the right choice for almost every driver
    /// (`IOMEM_DATA_CACHED`).
    #[default]
    Cached,
    /// The CPU does not cache the memory, but writes may be combined and
    /// reordered. Where the platform cannot do this, the memory is uncached
    /// instead. The length is rounded up to whole pages
    /// (`IOMEM_DATA_UC_WR_COMBINE`).
    WriteCombining,
    /// The CPU does not cache the memory and accesses it in program order.
    /// The length is rounded up to whole pages (`IOMEM_DATA_UNCACHED`).
    Uncached,
}

impl DmaCache {
    fn flag(self) -> sys::uint_t {
        match self {
            DmaCache::Cached => sys::IOMEM_DATA_CACHED,
            DmaCache::WriteCombining => sys::IOMEM_DATA_UC_WR_COMBINE,
            DmaCache::Uncached => sys::IOMEM_DATA_UNCACHED,
        }
    }
}

/// The device access attributes passed to `ddi_dma_mem_alloc(9F)`. Only the
/// byte order is meaningful there, and it must be `DDI_NEVERSWAP_ACC`
/// because the memory is handed out as ordinary slices; the manual allows
/// nothing but `ddi_get8(9F)` and its relatives for memory allocated with a
/// swapping byte order.
const DMA_MEM_ACC_ATTR: sys::ddi_device_acc_attr_t = sys::ddi_device_acc_attr_t {
    devacc_attr_version: sys::DDI_DEVICE_ATTR_V0,
    devacc_attr_endian_flags: sys::DDI_NEVERSWAP_ACC,
    devacc_attr_dataorder: sys::DDI_STRICTORDER_ACC,
    devacc_attr_access: 0,
};

/// Kernel memory for DMA from `ddi_dma_mem_alloc(9F)`, together with the
/// handle it was allocated for.
///
/// The memory is zeroed before the buffer is returned. While the buffer is
/// not bound it can be read and written like any slice. [`DmaBuffer::bind`]
/// hands it to the device and borrows it until the returned [`DmaBinding`]
/// is dropped.
///
/// Dropping the buffer frees the memory with `ddi_dma_mem_free(9F)` and then
/// the handle with `ddi_dma_free_handle(9F)`. If a binding was leaked with
/// `core::mem::forget`, the handle is unbound first, so the order the manual
/// demands holds in every case.
pub struct DmaBuffer {
    handle: DmaHandle,
    acc: sys::ddi_acc_handle_t,
    kaddr: NonNull<u8>,
    len: usize,
    mode: DmaMode,
    /// Whether the handle is bound. A [`DmaBinding`] clears it when dropped;
    /// it stays set only if a binding was leaked.
    bound: bool,
}

// SAFETY: The buffer owns kernel memory and two kernel handles, none of
// which has an affinity to a thread. The manual allows ddi_dma_mem_free,
// ddi_dma_unbind_handle and ddi_dma_free_handle from any context, so the
// buffer may be dropped on any thread.
unsafe impl Send for DmaBuffer {}
// SAFETY: Through a shared reference the buffer only reads its own fields and
// the memory as a `&[u8]`. The same holds for a shared reference to a
// `DmaBinding`, whose `&self` methods additionally read the cookie list and
// the burst sizes out of the handle, which the kernel does not change while
// the handle stays bound. Every method that calls into the kernel to change
// state takes `&mut self`.
unsafe impl Sync for DmaBuffer {}

impl DmaBuffer {
    /// Allocates `len` bytes for `handle` with `ddi_dma_mem_alloc(9F)` and
    /// `DDI_DMA_SLEEP`, waiting for memory if necessary, which is why a
    /// [`Sleepable`] token is required.
    ///
    /// The buffer takes ownership of the handle. If the allocation fails,
    /// the handle is freed. A length of zero fails with
    /// [`DmaError::InvalidLength`], and a failure of the kernel with
    /// [`DmaError::Failure`].
    pub fn new(
        handle: DmaHandle,
        len: usize,
        mode: DmaMode,
        cache: DmaCache,
        _ctx: &Sleepable,
    ) -> Result<DmaBuffer, DmaError> {
        Self::alloc(handle, len, mode, cache, sys::DDI_DMA_SLEEP)
    }

    /// Allocates `len` bytes for `handle` with `ddi_dma_mem_alloc(9F)` and
    /// `DDI_DMA_DONTWAIT`. This is allowed in any context and fails with
    /// [`DmaError::Failure`] when memory is short. Otherwise the same as
    /// [`DmaBuffer::new`].
    pub fn try_new(handle: DmaHandle, len: usize, mode: DmaMode, cache: DmaCache) -> Result<DmaBuffer, DmaError> {
        Self::alloc(handle, len, mode, cache, sys::DDI_DMA_DONTWAIT)
    }

    fn alloc(
        handle: DmaHandle,
        len: usize,
        mode: DmaMode,
        cache: DmaCache,
        wait: sys::ddi_dma_callback_t,
    ) -> Result<DmaBuffer, DmaError> {
        if len == 0 {
            return Err(DmaError::InvalidLength);
        }
        let mut kaddr: sys::caddr_t = ptr::null_mut();
        let mut real_len: usize = 0;
        let mut acc: sys::ddi_acc_handle_t = ptr::null_mut();
        // SAFETY: The handle is valid and has no memory yet, the access
        // attributes are a valid structure, the flags combine exactly one
        // transfer mode with exactly one cache attribute, the callback is one
        // of the two sentinels, and the three out pointers are writable.
        let r = unsafe {
            sys::ddi_dma_mem_alloc(
                handle.raw,
                len,
                &DMA_MEM_ACC_ATTR,
                mode.flag() | cache.flag(),
                wait,
                ptr::null_mut(),
                &mut kaddr,
                &mut real_len,
                &mut acc,
            )
        };
        DmaError::check_ddi(r)?;
        let Some(kaddr) = NonNull::new(kaddr as *mut u8) else {
            // SAFETY: The call succeeded, so `acc` is a valid access handle
            // that nothing else refers to.
            unsafe { sys::ddi_dma_mem_free(&mut acc) };
            return Err(DmaError::Failure);
        };
        // The kernel never reports less than was asked for; take the larger
        // value in case a platform leaves `real_len` alone.
        let len = real_len.max(len);
        // SAFETY: The kernel allocated at least `len` bytes at `kaddr`, and
        // nothing else refers to them yet. Zeroing initialises every byte so
        // that the slices handed out later never expose uninitialised memory.
        unsafe { ptr::write_bytes(kaddr.as_ptr(), 0, len) };
        Ok(DmaBuffer { handle, acc, kaddr, len, mode, bound: false })
    }

    /// Returns the length of the buffer in bytes. This is the real length
    /// reported by `ddi_dma_mem_alloc(9F)`, which may be larger than the
    /// length requested.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Always returns `false`, because an empty buffer cannot be allocated.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the transfer mode the buffer was allocated with, which is
    /// also the consistency flag used when it is bound.
    pub fn mode(&self) -> DmaMode {
        self.mode
    }

    /// Returns the handle the memory was allocated for.
    pub fn handle(&self) -> &DmaHandle {
        &self.handle
    }

    /// Returns the data access handle that `ddi_dma_mem_alloc(9F)` produced,
    /// for `ddi_get8(9F)` and the other access functions that are not
    /// wrapped yet. It is freed together with the buffer.
    pub fn acc_handle(&self) -> sys::ddi_acc_handle_t {
        self.acc
    }

    /// Returns the memory as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: The memory is `len` bytes long, was zeroed at allocation,
        // and stays allocated as long as `self`. A binding, the only way the
        // device learns the address, holds `&mut self`, so no binding exists
        // while this shared borrow does.
        unsafe { core::slice::from_raw_parts(self.kaddr.as_ptr(), self.len) }
    }

    /// Returns the memory as a mutable byte slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: As for `as_slice`, and the mutable borrow of `self` makes
        // this the only reference to the memory.
        unsafe { core::slice::from_raw_parts_mut(self.kaddr.as_ptr(), self.len) }
    }

    /// Interprets the start of the buffer as a `T`, which may be a slice
    /// type such as `[Descriptor]` that then covers as many elements as fit.
    /// Returns `None` if the buffer is too short or not aligned for `T`.
    /// The `zerocopy` traits guarantee that every byte pattern is a valid
    /// `T`, so no validation is skipped.
    pub fn view<T: FromBytes + KnownLayout + Immutable + ?Sized>(&self) -> Option<&T> {
        T::ref_from_prefix(self.as_slice()).ok().map(|(t, _)| t)
    }

    /// Interprets the start of the buffer as a mutable `T`, with the same
    /// rules as [`DmaBuffer::view`]. `IntoBytes` guarantees that writing a
    /// `T` leaves no uninitialised padding bytes in the buffer.
    pub fn view_mut<T: FromBytes + IntoBytes + KnownLayout + ?Sized>(&mut self) -> Option<&mut T> {
        T::mut_from_prefix(self.as_mut_slice()).ok().map(|(t, _)| t)
    }

    /// Binds the whole buffer to its handle with
    /// `ddi_dma_addr_bind_handle(9F)` and `DDI_DMA_SLEEP`, waiting for
    /// resources if necessary, which is why a [`Sleepable`] token is
    /// required.
    ///
    /// The flags are `direction` combined with the buffer's [`DmaMode`].
    /// The bind synchronises implicitly for the device, so whatever the CPU
    /// wrote into the buffer beforehand is what the device will see. The
    /// returned [`DmaBinding`] borrows the buffer until it is dropped, which
    /// unbinds it again.
    ///
    /// Fails with [`DmaError::NoMapping`] when the device cannot reach the
    /// memory, with [`DmaError::TooBig`] when the buffer needs more cookies
    /// than the attributes allow, and with [`DmaError::PartialMap`] in the
    /// unexpected case that the kernel mapped only part of it.
    pub fn bind(&mut self, direction: DmaDirection, _ctx: &Sleepable) -> Result<DmaBinding<'_>, DmaError> {
        self.bind_with(direction, sys::DDI_DMA_SLEEP)
    }

    /// Binds the whole buffer with `ddi_dma_addr_bind_handle(9F)` and
    /// `DDI_DMA_DONTWAIT`. This is allowed in any context and fails with
    /// [`DmaError::NoResources`] when resources are short. Otherwise the same
    /// as [`DmaBuffer::bind`].
    pub fn try_bind(&mut self, direction: DmaDirection) -> Result<DmaBinding<'_>, DmaError> {
        self.bind_with(direction, sys::DDI_DMA_DONTWAIT)
    }

    fn bind_with(
        &mut self,
        direction: DmaDirection,
        wait: sys::ddi_dma_callback_t,
    ) -> Result<DmaBinding<'_>, DmaError> {
        // A binding that was leaked with mem::forget left the handle bound.
        // Its borrow has ended, so nothing can use it any more; release it
        // so that the handle can be bound again.
        let _ = self.unbind_raw();
        let flags = direction.flag() | self.mode.flag();
        // SAFETY: The handle is valid and not bound. The address and length
        // describe memory the buffer owns, in the kernel address space, which
        // is what a NULL address space means. The callback is one of the two
        // sentinels. NULL for the cookie and count pointers is what the
        // manual asks of new callers.
        let r = unsafe {
            sys::ddi_dma_addr_bind_handle(
                self.handle.raw,
                ptr::null_mut(),
                self.kaddr.as_ptr() as sys::caddr_t,
                self.len,
                flags,
                wait,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        if r == sys::DDI_DMA_PARTIAL_MAP {
            // Resources were allocated for part of the buffer. Release them,
            // since this module does not walk windows.
            // SAFETY: The handle was bound by the call above.
            unsafe { sys::ddi_dma_unbind_handle(self.handle.raw) };
        }
        DmaError::check(r)?;
        self.bound = true;
        // SAFETY: The handle is valid and bound.
        let n = unsafe { sys::ddi_dma_ncookies(self.handle.raw) };
        Ok(DmaBinding { buf: self, direction, ncookies: usize::try_from(n).unwrap_or(0) })
    }

    /// Unbinds the handle with `ddi_dma_unbind_handle(9F)` if it is bound,
    /// and returns the kernel's answer. The rootnex returns `DDI_FAILURE`
    /// only when it finds the handle in an inconsistent state, and binding
    /// again is still the right thing to try afterwards, so the handle is
    /// considered unbound either way.
    fn unbind_raw(&mut self) -> Result<(), DmaError> {
        if !self.bound {
            return Ok(());
        }
        self.bound = false;
        // SAFETY: The handle is valid and bound, and no reference to the
        // memory exists, because the caller holds `&mut self`.
        DmaError::check_ddi(unsafe { sys::ddi_dma_unbind_handle(self.handle.raw) })
    }
}

impl Drop for DmaBuffer {
    fn drop(&mut self) {
        let _ = self.unbind_raw();
        // SAFETY: The access handle came from ddi_dma_mem_alloc and is freed
        // only here, after the handle has been unbound. No reference to the
        // memory can outlive `self`. The DMA handle is freed afterwards by
        // the Drop of the `handle` field.
        unsafe { sys::ddi_dma_mem_free(&mut self.acc) }
    }
}

impl fmt::Debug for DmaBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DmaBuffer").field("len", &self.len).field("mode", &self.mode).finish_non_exhaustive()
    }
}

// --- binding ---------------------------------------------------------------

/// The direction of a DMA transfer, as passed to
/// `ddi_dma_addr_bind_handle(9F)`.
///
/// The C names describe the transfer from the point of view of the device's
/// I/O operation, so that `DDI_DMA_READ` means the device writes memory. The
/// variants here name the direction in which the data moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaDirection {
    /// Data moves from memory to the device, which only reads the buffer
    /// (`DDI_DMA_WRITE`).
    ToDevice,
    /// Data moves from the device into memory, which the device writes
    /// (`DDI_DMA_READ`).
    FromDevice,
    /// Data moves in both directions (`DDI_DMA_RDWR`).
    Bidirectional,
}

impl DmaDirection {
    fn flag(self) -> sys::uint_t {
        match self {
            DmaDirection::ToDevice => sys::DDI_DMA_WRITE,
            DmaDirection::FromDevice => sys::DDI_DMA_READ,
            DmaDirection::Bidirectional => sys::DDI_DMA_RDWR,
        }
    }
}

/// A [`DmaBuffer`] bound for a transfer with
/// `ddi_dma_addr_bind_handle(9F)`.
///
/// The binding holds a mutable borrow of the buffer for its whole life. The
/// device addresses it exposes are therefore only available while the
/// memory is guaranteed to stay allocated and bound, and the CPU can reach
/// the memory only through the synchronising methods. Dropping the binding
/// unbinds the handle with `ddi_dma_unbind_handle(9F)`, which also
/// synchronises for the CPU, and ends the borrow.
pub struct DmaBinding<'a> {
    buf: &'a mut DmaBuffer,
    direction: DmaDirection,
    ncookies: usize,
}

impl<'a> DmaBinding<'a> {
    fn raw(&self) -> sys::ddi_dma_handle_t {
        self.buf.handle.raw
    }

    /// Returns the direction the buffer was bound for.
    pub fn direction(&self) -> DmaDirection {
        self.direction
    }

    /// Returns the length of the bound buffer in bytes.
    pub fn len(&self) -> usize {
        self.buf.len
    }

    /// Always returns `false`, because an empty buffer cannot be bound.
    pub fn is_empty(&self) -> bool {
        self.buf.len == 0
    }

    /// Returns the number of cookies, as `ddi_dma_ncookies(9F)` reported it
    /// right after binding. It never exceeds the scatter/gather length of
    /// the attributes.
    pub fn ncookies(&self) -> usize {
        self.ncookies
    }

    /// Iterates over the cookies in order with `ddi_dma_cookie_iter(9F)`.
    /// The iterator borrows the binding, so it cannot outlive the mapping.
    pub fn cookies(&self) -> DmaCookies<'_> {
        DmaCookies { handle: self.raw(), cur: ptr::null(), remaining: self.ncookies, _binding: PhantomData }
    }

    /// Returns the cookie at `index`, counting from zero, with
    /// `ddi_dma_cookie_get(9F)`, or `None` past the end.
    pub fn cookie(&self, index: usize) -> Option<DmaCookie> {
        if index >= self.ncookies {
            return None;
        }
        let index = sys::uint_t::try_from(index).ok()?;
        // SAFETY: The handle is valid and bound for the life of `self`, and
        // the kernel checks the index.
        let c = unsafe { sys::ddi_dma_cookie_get(self.raw(), index) };
        // SAFETY: A non-null result points to a cookie owned by the handle,
        // valid until the handle is unbound, which cannot happen while
        // `self` is borrowed.
        unsafe { c.as_ref() }.map(DmaCookie::from_raw)
    }

    /// Returns the only cookie if there is exactly one, and `None` otherwise.
    /// This replaces `ddi_dma_cookie_one(9F)`, which panics the system when
    /// the count is not one. With the default attributes, which allow a
    /// single cookie, a successful bind always has exactly one.
    pub fn single_cookie(&self) -> Option<DmaCookie> {
        if self.ncookies == 1 { self.cookie(0) } else { None }
    }

    /// Returns the burst sizes the device may use for this binding, with
    /// `ddi_dma_burstsizes(9F)`, in the bit mask encoding of
    /// [`DmaAttr::burst_sizes`]. The manual asks the driver to program its
    /// engine with this value rather than with the one it requested.
    pub fn burst_sizes(&self) -> u32 {
        // SAFETY: The handle is valid and bound.
        let b = unsafe { sys::ddi_dma_burstsizes(self.raw()) };
        b as u32
    }

    /// Calls `ddi_dma_sync(9F)` for the given range, after checking it.
    fn sync(&mut self, offset: usize, len: usize, whom: sys::uint_t) -> Result<(), DmaError> {
        let end = offset.checked_add(len).ok_or(DmaError::OutOfRange)?;
        if end > self.buf.len {
            return Err(DmaError::OutOfRange);
        }
        if len == 0 {
            // ddi_dma_sync treats a length of zero as "to the end"; an empty
            // range here means nothing to do.
            return Ok(());
        }
        let off = sys::off_t::try_from(offset).map_err(|_| DmaError::OutOfRange)?;
        // SAFETY: The handle is valid and bound, the range lies inside the
        // bound memory, and no reference to the memory exists while the
        // kernel may copy into it, because `self` is borrowed mutably.
        DmaError::check_ddi(unsafe { sys::ddi_dma_sync(self.raw(), off, len, whom) })
    }

    /// Makes what the CPU wrote into the buffer visible to the device, with
    /// `ddi_dma_sync(9F)` and `DDI_DMA_SYNC_FORDEV`.
    ///
    /// The binding itself only lets the CPU write the buffer inside
    /// [`DmaBinding::update`], which already ends with this call. The method
    /// exists for the cases the manual also covers, such as a device that
    /// is told to read the buffer a second time, and for drivers that write
    /// the memory through the raw access handle.
    pub fn sync_for_device(&mut self) -> Result<(), DmaError> {
        self.sync(0, self.buf.len, sys::DDI_DMA_SYNC_FORDEV)
    }

    /// Makes what the device wrote into the buffer visible to the CPU, with
    /// `ddi_dma_sync(9F)` and `DDI_DMA_SYNC_FORCPU`, and returns a view of
    /// the whole buffer.
    ///
    /// The view borrows the binding mutably, so it must be dropped before
    /// the binding can be synchronised again or dropped itself. The driver
    /// calls this when the device's protocol says the device has finished
    /// writing, typically after an interrupt or a status bit. A typed view
    /// is obtained by passing the slice to a `zerocopy` constructor such as
    /// `FromBytes::ref_from_prefix`.
    pub fn sync_for_cpu(&mut self) -> Result<&[u8], DmaError> {
        self.sync(0, self.buf.len, sys::DDI_DMA_SYNC_FORCPU)?;
        Ok(self.buf.as_slice())
    }

    /// Like [`DmaBinding::sync_for_device`], but only for the `len` bytes at
    /// `offset`. An empty range does nothing, and a range that does not lie
    /// inside the buffer fails with [`DmaError::OutOfRange`]. This is the
    /// cheaper choice when one descriptor of a ring has changed.
    pub fn sync_range_for_device(&mut self, offset: usize, len: usize) -> Result<(), DmaError> {
        self.sync(offset, len, sys::DDI_DMA_SYNC_FORDEV)
    }

    /// Like [`DmaBinding::sync_for_cpu`], but only for the `len` bytes at
    /// `offset`, and the returned view covers only that range. An empty
    /// range does nothing and returns an empty view, and a range that does
    /// not lie inside the buffer fails with [`DmaError::OutOfRange`].
    pub fn sync_range_for_cpu(&mut self, offset: usize, len: usize) -> Result<&[u8], DmaError> {
        self.sync(offset, len, sys::DDI_DMA_SYNC_FORCPU)?;
        Ok(&self.buf.as_slice()[offset..offset + len])
    }

    /// Gives the CPU read and write access to the bound buffer, with the
    /// synchronisation on both sides.
    ///
    /// The buffer is first synchronised for the CPU with
    /// `DDI_DMA_SYNC_FORCPU`, so that `f` sees what the device wrote. After
    /// `f` returns, the buffer is synchronised for the device with
    /// `DDI_DMA_SYNC_FORDEV`, so that the device sees what `f` wrote. This is
    /// how a driver exchanges descriptors with a device through a ring that
    /// stays bound. If the first synchronisation fails, `f` is not called.
    /// If the second fails, the value `f` returned is discarded and the
    /// error is returned instead.
    pub fn update<R>(&mut self, f: impl FnOnce(&mut [u8]) -> R) -> Result<R, DmaError> {
        self.sync(0, self.buf.len, sys::DDI_DMA_SYNC_FORCPU)?;
        let r = f(self.buf.as_mut_slice());
        self.sync(0, self.buf.len, sys::DDI_DMA_SYNC_FORDEV)?;
        Ok(r)
    }

    /// Unbinds the buffer with `ddi_dma_unbind_handle(9F)` and reports the
    /// result, which dropping the binding cannot. The unbind synchronises
    /// implicitly, so the device's writes are visible in the buffer
    /// afterwards. The handle counts as unbound even when the kernel reports
    /// [`DmaError::Failure`].
    pub fn unbind(self) -> Result<(), DmaError> {
        let mut this = core::mem::ManuallyDrop::new(self);
        this.buf.unbind_raw()
    }
}

impl Drop for DmaBinding<'_> {
    fn drop(&mut self) {
        let _ = self.buf.unbind_raw();
    }
}

impl fmt::Debug for DmaBinding<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DmaBinding")
            .field("len", &self.buf.len)
            .field("direction", &self.direction)
            .field("cookies", &DebugCookies(self))
            .finish()
    }
}

struct DebugCookies<'b, 'a>(&'b DmaBinding<'a>);

impl fmt::Debug for DebugCookies<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.cookies()).finish()
    }
}

// --- cookies ---------------------------------------------------------------

/// One contiguous piece of a bound buffer as the device sees it, read from a
/// `ddi_dma_cookie(9S)`.
///
/// The address is only meaningful while the [`DmaBinding`] it came from
/// exists. A device with 32-bit address registers is programmed with the low
/// half, which is what C calls `dmac_address`; the attributes guarantee that
/// the high half is zero in that case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DmaCookie {
    /// The device visible address, `dmac_laddress` in C.
    pub address: u64,
    /// The length of the piece in bytes, `dmac_size` in C.
    pub size: usize,
}

impl DmaCookie {
    fn from_raw(c: &sys::ddi_dma_cookie_t) -> DmaCookie {
        // SAFETY: Both members of the union are plain integers that cover the
        // same eight bytes, so every bit pattern is a valid `u64`.
        let address = unsafe { c._dmu._dmac_ll };
        DmaCookie { address, size: c.dmac_size }
    }
}

/// An iterator over the cookies of a [`DmaBinding`], built on
/// `ddi_dma_cookie_iter(9F)`.
///
/// It borrows the binding, so the cookie storage the kernel keeps in the
/// handle cannot be released while the iterator exists.
pub struct DmaCookies<'b> {
    handle: sys::ddi_dma_handle_t,
    cur: *const sys::ddi_dma_cookie_t,
    remaining: usize,
    _binding: PhantomData<&'b ()>,
}

impl Iterator for DmaCookies<'_> {
    type Item = DmaCookie;

    fn next(&mut self) -> Option<DmaCookie> {
        if self.remaining == 0 {
            return None;
        }
        // SAFETY: The handle is bound for the life of the borrow, and `cur`
        // is either NULL or a cookie the previous call returned, which is
        // what the manual requires.
        let next = unsafe { sys::ddi_dma_cookie_iter(self.handle, self.cur) };
        // SAFETY: A non-null result points to a cookie owned by the handle,
        // valid until the handle is unbound, which the borrow prevents.
        let Some(c) = (unsafe { next.as_ref() }) else {
            self.remaining = 0;
            return None;
        };
        self.cur = next;
        self.remaining -= 1;
        Some(DmaCookie::from_raw(c))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for DmaCookies<'_> {}

impl FusedIterator for DmaCookies<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_own_node_maps_to_enxio() {
        assert_eq!(Errno::from(DmaError::NotOwnNode), Errno::ENXIO);
    }

    #[test]
    fn errno_mapping() {
        assert_eq!(Errno::from(DmaError::BadAttr), Errno::EINVAL);
        assert_eq!(Errno::from(DmaError::NoResources), Errno::ENOMEM);
        assert_eq!(Errno::from(DmaError::TooBig), Errno::ENOMEM);
        assert_eq!(Errno::from(DmaError::NoMapping), Errno::EFAULT);
        assert_eq!(Errno::from(DmaError::InUse), Errno::EBUSY);
        assert_eq!(Errno::from(DmaError::Failure), Errno::EIO);
        assert_eq!(Errno::from(DmaError::Other(-99)), Errno::EIO);
    }

    #[test]
    fn default_attributes_pass_the_checks() {
        assert_eq!(DmaAttr::bits64().check(), Ok(()));
        assert_eq!(DmaAttr::bits32().max_cookies(4096).check(), Ok(()));
        assert_eq!(DmaAttr::bits64().align(3).check(), Err(DmaError::BadAttr));
    }
}
