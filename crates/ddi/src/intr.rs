//! Hardware and soft interrupts.
//!
//! A C driver sets up its interrupts in `attach(9E)` in a fixed order. It
//! asks which interrupt types the device and the platform support with
//! `ddi_intr_get_supported_types(9F)`, allocates handles with
//! `ddi_intr_alloc(9F)`, reads their priority with `ddi_intr_get_pri(9F)`,
//! initialises every lock the handler will take with that priority, adds the
//! handler with `ddi_intr_add_handler(9F)`, and finally enables the
//! interrupts. Teardown in `detach(9E)` runs the same steps backwards. The
//! order matters, and most of it is only written down in prose. This module
//! turns it into ownership.
//!
//! [`Interrupts`] owns a set of allocated handles and the state their
//! handler works on. [`Interrupts::alloc`] performs the whole setup up to,
//! but not including, enabling. Dropping the value disables the interrupts,
//! removes the handlers, frees the handles and only then drops the state,
//! so the state outlives every call the kernel can still make into it.
//! [`SoftInterrupt`] does the same for a soft interrupt registered with
//! `ddi_intr_add_softint(9F)`.
//!
//! # Handlers and context
//!
//! A handler is a type that implements [`InterruptHandler`] or
//! [`SoftInterruptHandler`]. The state is the handler; the kernel calls its
//! `handle` method through a trampoline with a shared reference to the state
//! and an [`Atomic`] token. The token is what tells the rest of the crate
//! that this code may not block, so a handler cannot call
//! [`KBox::new`], `CondVar::wait` or anything else that demands a
//! [`Sleepable`]. Because handlers of the same interrupt can run on several
//! processors at once, and because the rest of the driver reaches the same
//! state through [`Interrupts::state`], the state must be `Send + Sync`, and
//! everything it changes must sit behind a lock or an atomic.
//!
//! # Locks and priorities
//!
//! A mutex that an interrupt handler takes must be initialised with the
//! priority of that interrupt, otherwise the handler can deadlock against
//! the thread it interrupted. In this crate that means creating the
//! [`Mutex`](crate::sync::Mutex) with `Mutex::with_priority(value,
//! priority)`, where the priority is the [`IntrPriority`] of the interrupt.
//! The priority is only known once the handles have been allocated, while
//! the handler state has to exist before the handler is added. The two
//! steps are therefore joined by a closure. [`Interrupts::alloc`] allocates
//! the handles, reads their priority, and then calls the closure with the
//! priority and the number of vectors to build the state, before any
//! handler is added.
//!
//! ```ignore
//! struct Hw {
//!     regs: RegBlock<0x100>,
//!     stats: Mutex<Stats>,
//! }
//!
//! impl InterruptHandler for Hw {
//!     fn handle(&self, _vector: usize, _ctx: &Atomic) -> Claimed {
//!         let cause = self.regs.read(INTR_CAUSE);
//!         if cause == 0 {
//!             return Claimed::No;
//!         }
//!         self.regs.write(INTR_CAUSE, cause);
//!         self.stats.lock().interrupts += 1;
//!         Claimed::Yes
//!     }
//! }
//!
//! // `node` is the `OwnNode` that `Driver::attach` received; the queries
//! // reach its `DevInfo` through `Deref`.
//! let types = node.intr_types(ctx)?;
//! let kind = types.best().ok_or(DdiError)?;
//! let intrs = Interrupts::alloc(&node, kind, 1, false, |pri, _n| Hw {
//!     regs,
//!     stats: Mutex::with_priority(Stats::default(), pri),
//! }, ctx)?;
//! intrs.enable(ctx)?;
//! ```
//!
//! High-level interrupts, those at or above the priority that
//! `ddi_intr_get_hilevel_pri(9F)` returns, run above the scheduler and may
//! not take adaptive mutexes or do anything else that involves thread state.
//! A Rust handler has no way to promise that, so [`Interrupts::alloc`]
//! refuses such interrupts with [`IntrError::HighLevel`].
//!
//! A soft interrupt is the usual way to move work out of a hardware
//! handler. Create the [`SoftInterrupt`] inside the closure passed to
//! [`Interrupts::try_alloc`], so that it can use the hardware priority for
//! the locks it shares with the hardware handler, and keep it in the
//! hardware handler's state. The hardware handler then calls
//! [`SoftInterrupt::trigger`], and dropping the [`Interrupts`] removes the
//! hardware handler before the soft interrupt it owns.
//!
//! # Dropping
//!
//! Removing a handler waits until no processor is running it any more. An
//! [`Interrupts`] or [`SoftInterrupt`] must therefore be dropped in kernel
//! context, normally in `detach(9E)`, and never from inside its own handler,
//! which would wait for itself forever. If the kernel refuses to remove a
//! handler, the state is leaked rather than freed under a handler that may
//! still run, and a warning is logged.

use core::fmt;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use crate::context::{Atomic, Sleepable};
use crate::device::{DevInfo, OwnNode};
use crate::error::DdiError;
use crate::kmem::KBox;
use crate::sys;

/// An interrupt priority, as `ddi_intr_get_pri(9F)` and
/// `ddi_intr_get_softint_pri(9F)` report it. A lock that an interrupt
/// handler takes must be initialised with the handler's priority, which is
/// what [`IntrPriority::cookie`] provides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntrPriority(pub(crate) u32);

impl IntrPriority {
    /// The raw priority value.
    pub fn get(self) -> u32 {
        self.0
    }

    /// The interrupt block cookie for this priority, as `DDI_INTR_PRI`
    /// computes it, for `mutex_init(9F)` and `rw_init(9F)`.
    pub fn cookie(self) -> sys::ddi_iblock_cookie_t {
        sys::DDI_INTR_PRI(self.0)
    }
}

/// What went wrong in an interrupt call, as the extended return codes of
/// the `ddi_intr_*(9F)` functions describe it, plus the one refusal this
/// crate adds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrError {
    /// Not enough interrupt resources were available (`DDI_EAGAIN`).
    Again,
    /// The request or one of its arguments was invalid (`DDI_EINVAL`). This
    /// includes calling a function in the wrong state, such as enabling an
    /// interrupt that is already enabled.
    Invalid,
    /// The device or the platform does not support the operation
    /// (`DDI_ENOTSUP`).
    NotSupported,
    /// A soft interrupt was triggered while an earlier trigger was still
    /// pending (`DDI_EPENDING`). The handler will run for the earlier
    /// trigger, so this is rarely a real failure.
    Pending,
    /// The operation has already been done (`DDI_EALREADY`).
    Already,
    /// The device has no interrupt of the requested type
    /// (`DDI_INTR_NOTFOUND`).
    NotFound,
    /// The call failed for an implementation-specific reason
    /// (`DDI_FAILURE`).
    Failure,
    /// The interrupts are high-level, at or above the priority that
    /// `ddi_intr_get_hilevel_pri(9F)` returns. A high-level handler may not
    /// take ordinary locks, so [`Interrupts::alloc`] refuses them. The value
    /// is the priority the interrupts have.
    HighLevel(IntrPriority),
    /// A code this crate does not know.
    Other(i32),
}

impl IntrError {
    /// Interprets an extended DDI return code.
    fn check(raw: sys::c_int) -> Result<(), IntrError> {
        match raw {
            sys::DDI_SUCCESS => Ok(()),
            sys::DDI_FAILURE => Err(IntrError::Failure),
            sys::DDI_EAGAIN => Err(IntrError::Again),
            sys::DDI_EINVAL => Err(IntrError::Invalid),
            sys::DDI_ENOTSUP => Err(IntrError::NotSupported),
            sys::DDI_EPENDING => Err(IntrError::Pending),
            sys::DDI_EALREADY => Err(IntrError::Already),
            sys::DDI_INTR_NOTFOUND => Err(IntrError::NotFound),
            other => Err(IntrError::Other(other)),
        }
    }
}

impl fmt::Display for IntrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntrError::Again => f.write_str("DDI_EAGAIN"),
            IntrError::Invalid => f.write_str("DDI_EINVAL"),
            IntrError::NotSupported => f.write_str("DDI_ENOTSUP"),
            IntrError::Pending => f.write_str("DDI_EPENDING"),
            IntrError::Already => f.write_str("DDI_EALREADY"),
            IntrError::NotFound => f.write_str("DDI_INTR_NOTFOUND"),
            IntrError::Failure => f.write_str("DDI_FAILURE"),
            IntrError::HighLevel(p) => write!(f, "high-level interrupt at priority {}", p.get()),
            IntrError::Other(c) => write!(f, "DDI interrupt code {c}"),
        }
    }
}

/// Every interrupt error is a failure in the `DDI_SUCCESS` and
/// `DDI_FAILURE` convention, which lets `attach(9E)` use `?` on the
/// functions of this module.
impl From<IntrError> for DdiError {
    fn from(_: IntrError) -> DdiError {
        DdiError
    }
}

/// A kind of hardware interrupt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntrKind {
    /// A line-based interrupt, or the INTx pin of a PCI function
    /// (`DDI_INTR_TYPE_FIXED`). A fixed interrupt may be shared with other
    /// devices.
    Fixed,
    /// A PCI message signalled interrupt (`DDI_INTR_TYPE_MSI`). A function
    /// has at most 32 vectors, allocated in powers of two.
    Msi,
    /// An extended PCI message signalled interrupt (`DDI_INTR_TYPE_MSIX`),
    /// with up to 2048 independently maskable vectors.
    MsiX,
}

impl IntrKind {
    /// Returns the `DDI_INTR_TYPE_*` value.
    pub fn raw(self) -> sys::c_int {
        match self {
            IntrKind::Fixed => sys::DDI_INTR_TYPE_FIXED,
            IntrKind::Msi => sys::DDI_INTR_TYPE_MSI,
            IntrKind::MsiX => sys::DDI_INTR_TYPE_MSIX,
        }
    }
}

/// A set of interrupt kinds, as `ddi_intr_get_supported_types(9F)` reports
/// it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IntrTypes(sys::c_int);

impl IntrTypes {
    /// The empty set.
    pub const EMPTY: IntrTypes = IntrTypes(0);

    /// Reports whether `kind` is in the set.
    pub fn contains(self, kind: IntrKind) -> bool {
        self.0 & kind.raw() != 0
    }

    /// Reports whether the set is empty, which is the case for a device
    /// without interrupts.
    pub fn is_empty(self) -> bool {
        self.0 & (sys::DDI_INTR_TYPE_FIXED | sys::DDI_INTR_TYPE_MSI | sys::DDI_INTR_TYPE_MSIX) == 0
    }

    /// Returns the kind most drivers should use, which is MSI-X if
    /// available, then MSI, then fixed.
    pub fn best(self) -> Option<IntrKind> {
        self.iter().next()
    }

    /// Returns the kinds in the set, in the order of preference that
    /// [`IntrTypes::best`] uses.
    pub fn iter(self) -> impl Iterator<Item = IntrKind> {
        [IntrKind::MsiX, IntrKind::Msi, IntrKind::Fixed].into_iter().filter(move |&k| self.contains(k))
    }

    /// Returns the raw `DDI_INTR_TYPE_*` bit mask.
    pub fn bits(self) -> sys::c_int {
        self.0
    }
}

impl fmt::Debug for IntrTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

/// The capability flags of an interrupt, as `ddi_intr_get_cap(9F)` reports
/// them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IntrCaps(sys::c_int);

impl IntrCaps {
    /// The interrupt is level triggered (`DDI_INTR_FLAG_LEVEL`).
    pub fn is_level(self) -> bool {
        self.0 & sys::DDI_INTR_FLAG_LEVEL != 0
    }

    /// The interrupt is edge triggered (`DDI_INTR_FLAG_EDGE`).
    pub fn is_edge(self) -> bool {
        self.0 & sys::DDI_INTR_FLAG_EDGE != 0
    }

    /// The interrupt can be masked (`DDI_INTR_FLAG_MASKABLE`).
    pub fn is_maskable(self) -> bool {
        self.0 & sys::DDI_INTR_FLAG_MASKABLE != 0
    }

    /// The interrupt has a readable pending bit (`DDI_INTR_FLAG_PENDING`).
    pub fn has_pending(self) -> bool {
        self.0 & sys::DDI_INTR_FLAG_PENDING != 0
    }

    /// All interrupts of the set must be enabled and disabled together with
    /// `ddi_intr_block_enable(9F)` (`DDI_INTR_FLAG_BLOCK`).
    /// [`Interrupts::enable`] and [`Interrupts::disable`] take care of this.
    pub fn needs_block_enable(self) -> bool {
        self.0 & sys::DDI_INTR_FLAG_BLOCK != 0
    }

    /// Returns the raw `DDI_INTR_FLAG_*` bit mask.
    pub fn bits(self) -> sys::c_int {
        self.0
    }
}

impl fmt::Debug for IntrCaps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IntrCaps({:#x})", self.0)
    }
}

/// Whether a handler serviced the interrupt, which is what it returns to
/// the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Claimed {
    /// The interrupt came from this device and was serviced
    /// (`DDI_INTR_CLAIMED`).
    Yes,
    /// The interrupt was not for this device (`DDI_INTR_UNCLAIMED`). On a
    /// shared line the kernel then offers it to the other handlers.
    No,
}

impl Claimed {
    fn raw(self) -> sys::uint_t {
        match self {
            Claimed::Yes => sys::DDI_INTR_CLAIMED,
            Claimed::No => sys::DDI_INTR_UNCLAIMED,
        }
    }
}

impl From<bool> for Claimed {
    fn from(claimed: bool) -> Claimed {
        if claimed { Claimed::Yes } else { Claimed::No }
    }
}

impl DevInfo {
    /// Returns the interrupt kinds that both the device and the platform
    /// support, using `ddi_intr_get_supported_types(9F)`. A device without
    /// interrupts yields the empty set. The manual forbids calling this from
    /// interrupt context.
    pub fn intr_types(&self, _ctx: &Sleepable) -> Result<IntrTypes, IntrError> {
        let mut types: sys::c_int = 0;
        match IntrError::check(unsafe { sys::ddi_intr_get_supported_types(self.as_raw(), &mut types) }) {
            Ok(()) => Ok(IntrTypes(types)),
            Err(IntrError::NotFound) => Ok(IntrTypes::EMPTY),
            Err(e) => Err(e),
        }
    }

    /// Returns how many interrupts of `kind` the device supports, using
    /// `ddi_intr_get_nintrs(9F)`. A device without such interrupts yields 0.
    pub fn intr_count(&self, kind: IntrKind, _ctx: &Sleepable) -> Result<usize, IntrError> {
        let mut n: sys::c_int = 0;
        match IntrError::check(unsafe { sys::ddi_intr_get_nintrs(self.as_raw(), kind.raw(), &mut n) }) {
            Ok(()) => Ok(usize::try_from(n).unwrap_or(0)),
            Err(IntrError::NotFound) => Ok(0),
            Err(e) => Err(e),
        }
    }

    /// Returns how many interrupts of `kind` the system is currently willing
    /// to give the device, using `ddi_intr_get_navail(9F)`. This is the
    /// number to ask [`Interrupts::alloc`] for. A device without such
    /// interrupts yields 0.
    pub fn intr_available(&self, kind: IntrKind, _ctx: &Sleepable) -> Result<usize, IntrError> {
        let mut n: sys::c_int = 0;
        match IntrError::check(unsafe { sys::ddi_intr_get_navail(self.as_raw(), kind.raw(), &mut n) }) {
            Ok(()) => Ok(usize::try_from(n).unwrap_or(0)),
            Err(IntrError::NotFound) => Ok(0),
            Err(e) => Err(e),
        }
    }
}

/// The handler of a set of hardware interrupts, implemented on the state
/// that [`Interrupts`] owns.
pub trait InterruptHandler: Send + Sync + 'static {
    /// Services an interrupt. `vector` is the index of the interrupt within
    /// its [`Interrupts`] set, from 0 to [`Interrupts::count`] minus one.
    ///
    /// The kernel may call this on several processors at once, and on a
    /// shared fixed interrupt it also calls it when another device
    /// interrupted, so the handler must check its device and return
    /// [`Claimed::No`] when there is nothing to do. The [`Atomic`] token
    /// marks the context as one that may not block.
    fn handle(&self, vector: usize, ctx: &Atomic) -> Claimed;
}

/// The kernel entry point for every hardware interrupt, monomorphised per
/// handler type.
///
/// # Safety
/// `arg1` must be the `KBox<T>` contents registered by [`Interrupts`], which
/// keeps them alive until the handler has been removed, and `arg2` the
/// vector index cast to a pointer.
unsafe extern "C" fn intr_trampoline<T: InterruptHandler>(arg1: sys::caddr_t, arg2: sys::caddr_t) -> sys::uint_t {
    // SAFETY: `Interrupts` passed a pointer to its state and frees the state
    // only after `ddi_intr_remove_handler(9F)` has returned, which waits for
    // running handlers.
    let state = unsafe { &*(arg1 as *const T) };
    let vector = arg2 as usize;
    // SAFETY: This is a hardware interrupt handler, which may not block.
    let ctx = unsafe { Atomic::assume() };
    state.handle(vector, &ctx).raw()
}

/// The handle array of an [`Interrupts`] set and how far its setup got.
///
/// The array itself comes from `kmem_zalloc(9F)` because its length is only
/// known at run time. Tearing down undoes exactly the steps recorded in
/// `allocated` and `added`.
struct Handles {
    array: NonNull<sys::ddi_intr_handle_t>,
    capacity: usize,
    allocated: usize,
    added: usize,
    block: bool,
}

impl Handles {
    fn new(capacity: usize) -> Handles {
        let bytes = capacity * size_of_handle();
        // SAFETY: A plain KM_SLEEP allocation, which cannot fail and which
        // the caller's Sleepable token permits.
        let raw = unsafe { sys::kmem_zalloc(bytes, sys::KM_SLEEP) } as *mut sys::ddi_intr_handle_t;
        let array = match NonNull::new(raw) {
            Some(p) => p,
            // kmem_zalloc(9F) with KM_SLEEP never returns NULL.
            None => unsafe { sys::panic(c"ddi::intr: kmem_zalloc(KM_SLEEP) returned NULL".as_ptr()) },
        };
        Handles { array, capacity, allocated: 0, added: 0, block: false }
    }

    fn get(&self, i: usize) -> sys::ddi_intr_handle_t {
        debug_assert!(i < self.allocated);
        // SAFETY: `i` is below `allocated`, which is at most `capacity`.
        unsafe { *self.array.as_ptr().add(i) }
    }

    /// Disables every interrupt, removes every handler and frees every
    /// handle. Returns `false` if a handler could not be removed, in which
    /// case the kernel may still call it and its state must be leaked.
    fn teardown(&mut self) -> bool {
        if self.added > 0 {
            // Disabling an interrupt that is not enabled fails harmlessly
            // with DDI_EINVAL, so the results are ignored.
            if self.block {
                // The count fits in a c_int, because allocation succeeded
                // with it.
                unsafe { sys::ddi_intr_block_disable(self.array.as_ptr(), self.added as sys::c_int) };
            } else {
                for i in 0..self.added {
                    unsafe { sys::ddi_intr_disable(self.get(i)) };
                }
            }
        }
        let mut removed = true;
        for i in 0..self.added {
            if unsafe { sys::ddi_intr_remove_handler(self.get(i)) } != sys::DDI_SUCCESS {
                removed = false;
            }
        }
        self.added = 0;
        for i in 0..self.allocated {
            // A handle whose handler is still attached cannot be freed; the
            // kernel refuses with DDI_EINVAL and the handle is leaked.
            unsafe { sys::ddi_intr_free(self.get(i)) };
        }
        self.allocated = 0;
        removed
    }
}

impl Drop for Handles {
    fn drop(&mut self) {
        self.teardown();
        // SAFETY: The array came from `kmem_zalloc` with this size, and the
        // kernel keeps no pointer to it.
        unsafe { sys::kmem_free(self.array.as_ptr().cast(), self.capacity * size_of_handle()) }
    }
}

const fn size_of_handle() -> usize {
    core::mem::size_of::<sys::ddi_intr_handle_t>()
}

/// A set of hardware interrupts of one kind with their handler state.
///
/// [`Interrupts::alloc`] allocates the handles with `ddi_intr_alloc(9F)`,
/// builds the state, stores it in a [`KBox`] so that its address stays
/// fixed, and adds the handler to every handle with
/// `ddi_intr_add_handler(9F)`, passing the state and the vector index. The
/// interrupts start out disabled; [`Interrupts::enable`] turns them on.
///
/// Dropping the value disables the interrupts, removes the handlers with
/// `ddi_intr_remove_handler(9F)`, frees the handles with `ddi_intr_free(9F)`
/// and then drops the state. The module documentation explains the context
/// rules for dropping.
pub struct Interrupts<T: InterruptHandler> {
    handles: Handles,
    kind: IntrKind,
    caps: IntrCaps,
    priority: IntrPriority,
    state: ManuallyDrop<KBox<T>>,
}

// SAFETY: The handles are plain values that the `ddi_intr_*` functions
// accept from any thread, and they serialise on each handle's own lock. The
// state is `Send` because `InterruptHandler` requires it, and it is freed on
// whichever thread drops the set.
unsafe impl<T: InterruptHandler> Send for Interrupts<T> {}
// SAFETY: Shared access hands out `&T`, which is sound because
// `InterruptHandler` requires `Sync`, and calls `enable` and `disable`,
// which the kernel serialises.
unsafe impl<T: InterruptHandler> Sync for Interrupts<T> {}

impl<T: InterruptHandler> Interrupts<T> {
    /// Allocates up to `count` interrupts of `kind`, builds the handler
    /// state and adds the handler to each of them.
    ///
    /// When `strict` is true the call fails unless all `count` interrupts
    /// could be allocated (`DDI_INTR_ALLOC_STRICT`); otherwise it succeeds
    /// with whatever number was available, as long as it is at least one
    /// (`DDI_INTR_ALLOC_NORMAL`). For MSI, `count` must be a power of two.
    /// Vectors are allocated from index 0.
    ///
    /// After allocation the priority is read with `ddi_intr_get_pri(9F)`.
    /// If it is at or above `ddi_intr_get_hilevel_pri(9F)`, the handles are
    /// freed and [`IntrError::HighLevel`] is returned. Otherwise `init` is
    /// called with the priority and the number of vectors actually
    /// allocated, and its result becomes the handler state. Locks in the
    /// state that the handler takes must be created with that priority.
    ///
    /// The interrupts are allocated for `node`, the driver's own attached
    /// node, as `ddi_intr_alloc(9F)` requires. The request goes to the
    /// node's parent nexus, and the root node has none, so the kernel would
    /// dereference NULL; a node that the guard of [`OwnNode`] refuses (the
    /// root node, or a node bound to another driver) fails with
    /// [`IntrError::Invalid`] before the kernel is called.
    pub fn alloc<F>(
        node: &OwnNode,
        kind: IntrKind,
        count: usize,
        strict: bool,
        init: F,
        ctx: &Sleepable,
    ) -> Result<Interrupts<T>, IntrError>
    where
        F: FnOnce(IntrPriority, usize) -> T,
    {
        Self::try_alloc(node, kind, count, strict, |pri, n| Ok(init(pri, n)), ctx)
    }

    /// Does what [`Interrupts::alloc`] does, with a state constructor that
    /// may fail. If `init` returns an error, the handles are freed and the
    /// error is returned. This is the variant to use when building the state
    /// involves other fallible setup, such as creating a [`SoftInterrupt`].
    pub fn try_alloc<E, F>(
        node: &OwnNode,
        kind: IntrKind,
        count: usize,
        strict: bool,
        init: F,
        ctx: &Sleepable,
    ) -> Result<Interrupts<T>, E>
    where
        E: From<IntrError>,
        F: FnOnce(IntrPriority, usize) -> Result<T, E>,
    {
        let want = match sys::c_int::try_from(count) {
            Ok(n) if n > 0 => n,
            _ => return Err(IntrError::Invalid.into()),
        };
        if !node.is_plausibly_own() {
            return Err(IntrError::Invalid.into());
        }
        let mut handles = Handles::new(count);
        let behavior = if strict { sys::DDI_INTR_ALLOC_STRICT } else { sys::DDI_INTR_ALLOC_NORMAL };
        let mut actual: sys::c_int = 0;
        IntrError::check(unsafe {
            sys::ddi_intr_alloc(node.as_raw(), handles.array.as_ptr(), kind.raw(), 0, want, &mut actual, behavior)
        })?;
        // The kernel stores at most `count` handles; clamp anyway so that a
        // misbehaving nexus cannot make teardown read past the array.
        handles.allocated = usize::try_from(actual).unwrap_or(0).min(count);
        if handles.allocated == 0 {
            return Err(IntrError::NotFound.into());
        }

        let mut flags: sys::c_int = 0;
        IntrError::check(unsafe { sys::ddi_intr_get_cap(handles.get(0), &mut flags) })?;
        let caps = IntrCaps(flags);
        handles.block = caps.needs_block_enable();

        // All handles from one allocation share a priority. Taking the
        // highest one anyway means a lock built from it is right for every
        // vector even if a platform ever differs.
        let mut pri: sys::uint_t = 0;
        for i in 0..handles.allocated {
            let mut p: sys::uint_t = 0;
            IntrError::check(unsafe { sys::ddi_intr_get_pri(handles.get(i), &mut p) })?;
            pri = pri.max(p);
        }
        let priority = IntrPriority(pri);
        if pri >= unsafe { sys::ddi_intr_get_hilevel_pri() } {
            return Err(IntrError::HighLevel(priority).into());
        }

        let n = handles.allocated;
        let state = KBox::new(init(priority, n)?, ctx);
        // From here on, `Drop for Interrupts` undoes whatever part of the
        // setup has been done, in the right order.
        let mut this = Interrupts { handles, kind, caps, priority, state: ManuallyDrop::new(state) };
        let arg1 = &**this.state as *const T as *mut sys::c_void;
        for i in 0..n {
            let arg2 = core::ptr::without_provenance_mut::<sys::c_void>(i);
            IntrError::check(unsafe {
                sys::ddi_intr_add_handler(this.handles.get(i), intr_trampoline::<T>, arg1, arg2)
            })?;
            this.handles.added = i + 1;
        }
        Ok(this)
    }

    /// Enables every interrupt of the set. When the capabilities include
    /// `DDI_INTR_FLAG_BLOCK` this is one call to `ddi_intr_block_enable(9F)`,
    /// otherwise one call to `ddi_intr_enable(9F)` per vector. If a vector
    /// fails to enable, the ones enabled before it are disabled again.
    /// Enabling interrupts that are already enabled fails with
    /// [`IntrError::Invalid`].
    pub fn enable(&self, _ctx: &Sleepable) -> Result<(), IntrError> {
        let h = &self.handles;
        if h.block {
            return IntrError::check(unsafe { sys::ddi_intr_block_enable(h.array.as_ptr(), h.added as sys::c_int) });
        }
        for i in 0..h.added {
            if let Err(e) = IntrError::check(unsafe { sys::ddi_intr_enable(h.get(i)) }) {
                for j in 0..i {
                    unsafe { sys::ddi_intr_disable(h.get(j)) };
                }
                return Err(e);
            }
        }
        Ok(())
    }

    /// Disables every interrupt of the set, with `ddi_intr_block_disable(9F)`
    /// or with `ddi_intr_disable(9F)` per vector, matching
    /// [`Interrupts::enable`]. After this returns, the handler is no longer
    /// running and will not be called until the set is enabled again. Every
    /// vector is attempted even if one fails, and the first failure is
    /// returned.
    pub fn disable(&self, _ctx: &Sleepable) -> Result<(), IntrError> {
        let h = &self.handles;
        if h.block {
            return IntrError::check(unsafe { sys::ddi_intr_block_disable(h.array.as_ptr(), h.added as sys::c_int) });
        }
        let mut result = Ok(());
        for i in 0..h.added {
            let r = IntrError::check(unsafe { sys::ddi_intr_disable(h.get(i)) });
            if result.is_ok() {
                result = r;
            }
        }
        result
    }

    /// Returns the priority of the interrupts. Every lock the handler takes
    /// must have been created with it.
    pub fn priority(&self) -> IntrPriority {
        self.priority
    }

    /// Returns the handler state, which the rest of the driver shares with
    /// the handler.
    pub fn state(&self) -> &T {
        &self.state
    }

    /// Returns the number of vectors in the set.
    pub fn count(&self) -> usize {
        self.handles.added
    }

    /// Returns the kind of the interrupts.
    pub fn kind(&self) -> IntrKind {
        self.kind
    }

    /// Returns the capability flags that `ddi_intr_get_cap(9F)` reported.
    pub fn caps(&self) -> IntrCaps {
        self.caps
    }
}

impl<T: InterruptHandler> Drop for Interrupts<T> {
    fn drop(&mut self) {
        if self.handles.teardown() {
            // SAFETY: Every handler has been removed, and removal waits for
            // running handlers, so nothing refers to the state any more. The
            // state is dropped exactly once, here.
            unsafe { ManuallyDrop::drop(&mut self.state) }
        } else {
            crate::kwarn!("ddi::intr: an interrupt handler could not be removed; leaking its state");
        }
    }
}

impl<T: InterruptHandler> fmt::Debug for Interrupts<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interrupts")
            .field("kind", &self.kind)
            .field("count", &self.handles.added)
            .field("priority", &self.priority)
            .field("caps", &self.caps)
            .finish()
    }
}

/// The handler of a soft interrupt, implemented on the state that
/// [`SoftInterrupt`] owns.
pub trait SoftInterruptHandler: Send + Sync + 'static {
    /// Services the soft interrupt.
    ///
    /// The manual warns that another driver may trigger a soft interrupt at
    /// the same level, so the handler must look at its own state to decide
    /// whether there is work, and return [`Claimed::No`] when there is none.
    /// The code that calls [`SoftInterrupt::trigger`] should record the work
    /// in the state first. The [`Atomic`] token marks the context as one
    /// that may not block.
    fn handle(&self, ctx: &Atomic) -> Claimed;
}

/// The kernel entry point for every soft interrupt, monomorphised per
/// handler type.
///
/// # Safety
/// `arg1` must be the `KBox<T>` contents registered by [`SoftInterrupt`],
/// which keeps them alive until the soft interrupt has been removed.
unsafe extern "C" fn softint_trampoline<T: SoftInterruptHandler>(
    arg1: sys::caddr_t,
    _arg2: sys::caddr_t,
) -> sys::uint_t {
    // SAFETY: `SoftInterrupt` passed a pointer to its state and frees the
    // state only after `ddi_intr_remove_softint(9F)` has returned, which
    // waits for running handlers.
    let state = unsafe { &*(arg1 as *const T) };
    // SAFETY: This is a soft interrupt handler, which may not block.
    let ctx = unsafe { Atomic::assume() };
    state.handle(&ctx).raw()
}

/// A soft interrupt with its handler state, registered with
/// `ddi_intr_add_softint(9F)` and removed with `ddi_intr_remove_softint(9F)`
/// when dropped, after which the state is dropped.
///
/// Soft interrupts run below the hardware interrupt levels, at one of the
/// priorities from 1 to 9. A hardware handler triggers one to hand work to
/// code that may take locks a hardware handler should not hold for long.
pub struct SoftInterrupt<T: SoftInterruptHandler> {
    handle: sys::ddi_softint_handle_t,
    priority: IntrPriority,
    state: ManuallyDrop<KBox<T>>,
}

// SAFETY: The handle is a plain value that the `ddi_intr_*softint`
// functions accept from any thread. The state is `Send` because
// `SoftInterruptHandler` requires it.
unsafe impl<T: SoftInterruptHandler> Send for SoftInterrupt<T> {}
// SAFETY: Shared access hands out `&T`, which is sound because
// `SoftInterruptHandler` requires `Sync`, and calls `trigger`, which the
// manual allows from any context and on any processor.
unsafe impl<T: SoftInterruptHandler> Sync for SoftInterrupt<T> {}

impl<T: SoftInterruptHandler> SoftInterrupt<T> {
    /// The priority to use when there is no reason to prefer another
    /// (`DDI_INTR_SOFTPRI_DEFAULT`).
    pub const DEFAULT_PRIORITY: u32 = sys::DDI_INTR_SOFTPRI_DEFAULT as u32;
    /// The lowest soft interrupt priority (`DDI_INTR_SOFTPRI_MIN`).
    pub const MIN_PRIORITY: u32 = sys::DDI_INTR_SOFTPRI_MIN as u32;
    /// The highest soft interrupt priority (`DDI_INTR_SOFTPRI_MAX`).
    pub const MAX_PRIORITY: u32 = sys::DDI_INTR_SOFTPRI_MAX as u32;

    /// Builds the handler state by calling `init` with the soft interrupt's
    /// priority, stores it in a [`KBox`], and registers the handler at that
    /// priority with `ddi_intr_add_softint(9F)`.
    ///
    /// `priority` must lie between [`SoftInterrupt::MIN_PRIORITY`] and
    /// [`SoftInterrupt::MAX_PRIORITY`], or the call fails with
    /// [`IntrError::Invalid`]. Locks the handler takes must be created with
    /// the priority `init` receives, or with a higher hardware priority when
    /// a hardware handler takes them as well.
    pub fn add<F>(dev: &DevInfo, priority: u32, init: F, ctx: &Sleepable) -> Result<SoftInterrupt<T>, IntrError>
    where
        F: FnOnce(IntrPriority) -> T,
    {
        if !(Self::MIN_PRIORITY..=Self::MAX_PRIORITY).contains(&priority) {
            return Err(IntrError::Invalid);
        }
        let pri = IntrPriority(priority);
        let state = KBox::new(init(pri), ctx);
        let arg1 = &*state as *const T as *mut sys::c_void;
        let mut handle: sys::ddi_softint_handle_t = core::ptr::null_mut();
        // The range check above guarantees the value fits in a c_int.
        IntrError::check(unsafe {
            sys::ddi_intr_add_softint(dev.as_raw(), &mut handle, priority as sys::c_int, softint_trampoline::<T>, arg1)
        })?;
        Ok(SoftInterrupt { handle, priority: pri, state: ManuallyDrop::new(state) })
    }

    /// Schedules the handler to run, using `ddi_intr_trigger_softint(9F)`.
    /// This may be called from any context, including a hardware interrupt
    /// handler.
    ///
    /// Only one trigger can be outstanding. While one is pending, a further
    /// trigger returns [`IntrError::Pending`] and changes nothing, which is
    /// harmless because the pending run will see the work recorded in the
    /// state.
    pub fn trigger(&self) -> Result<(), IntrError> {
        IntrError::check(unsafe { sys::ddi_intr_trigger_softint(self.handle, core::ptr::null_mut()) })
    }

    /// Returns the priority of the soft interrupt.
    pub fn priority(&self) -> IntrPriority {
        self.priority
    }

    /// Returns the handler state.
    pub fn state(&self) -> &T {
        &self.state
    }
}

impl<T: SoftInterruptHandler> Drop for SoftInterrupt<T> {
    fn drop(&mut self) {
        if unsafe { sys::ddi_intr_remove_softint(self.handle) } == sys::DDI_SUCCESS {
            // SAFETY: The handler has been removed, and removal waits for a
            // running handler, so nothing refers to the state any more. The
            // state is dropped exactly once, here.
            unsafe { ManuallyDrop::drop(&mut self.state) }
        } else {
            crate::kwarn!("ddi::intr: a soft interrupt could not be removed; leaking its state");
        }
    }
}

impl<T: SoftInterruptHandler> fmt::Debug for SoftInterrupt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoftInterrupt").field("priority", &self.priority).finish()
    }
}
