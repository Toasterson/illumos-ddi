//! The device memory mapping framework from `sys/ddidevmap.h`.
//!
//! The devmap framework is how a driver lets a user process map device
//! registers or kernel memory into its address space with `mmap(2)`. The
//! driver's `segmap(9E)` entry point calls `devmap_setup(9F)`, or simply
//! installs `ddi_devmap_segmap(9F)` as the entry point, and the framework
//! then calls the driver's `devmap(9E)` entry point to validate the range
//! and translate the user offset. Inside `devmap(9E)` the driver calls
//! `devmap_devmem_setup(9F)` to map a register set, or
//! `devmap_umem_setup(9F)` to map memory obtained from
//! `ddi_umem_alloc(9F)`, passing a `devmap_callback_ctl(9S)` if it wants to
//! hear about events on the mapping.
//!
//! The callback structure names up to four driver routines. `devmap_map(9E)`
//! runs when the mapping is created and usually allocates private data;
//! `devmap_access(9E)` runs when the process touches an address that has no
//! translation, and must end by calling either `devmap_default_access(9F)`
//! or, for devices whose registers are switched between processes,
//! `devmap_do_ctxmgt(9F)` with a `devmap_contextmgt(9E)` routine that in
//! turn uses `devmap_unload(9F)` and `devmap_load(9F)`; `devmap_dup(9E)`
//! runs on `fork(2)` and `devmap_unmap(9E)` on `munmap(2)` or `exit(2)`.
//! `devmap_set_ctx_timeout(9F)` bounds how often the context switching
//! routine is invoked.
//!
//! The `devmap_cookie_t` handle that identifies a mapping is declared in
//! [`crate::types`] because `cb_ops(9S)` needs it. The `ddi_umem_*`
//! functions and the `ddi_umem_cookie_t` type belong to [`crate::umem`].
//!
//! `ddi_segmap(9F)` and `ddi_segmap_setup(9F)`, the pre-devmap way of
//! mapping device memory, are marked obsolete by their manual page and are
//! not bound. `devmap_devmem_remap` and `devmap_umem_remap` are prototyped
//! in `sunddi.h` but have no manual page and are not in the `ddi` mapfile,
//! so they are not bound either. The `devmap_handle_t`, `devmap_softlock`,
//! `devmap_ctx` and `devmap_info` structures in the header are private to
//! the segment driver and are not declared.
//!
//! The `devmap_fault_type` and `devmap_rw` enumerations are declared as
//! `uint_t` constants because the `type` and `rw` arguments that carry them
//! are `uint_t` in every prototype. `devmap_access(9E)` documents
//! `DEVMAP_ACCESS`, `DEVMAP_LOCK`, `DEVMAP_UNLOCK`, `DEVMAP_READ`,
//! `DEVMAP_WRITE` and `DEVMAP_EXEC`; the remaining enumerators
//! `DEVMAP_PROT`, `DEVMAP_OTHER` and `DEVMAP_CREATE` are values the kernel
//! can pass to the same entry point and are declared for completeness.

use crate::types::*;

/// An opaque handle to a process address space, `ddi_as_handle_t`, the
/// third argument of `devmap_setup(9F)`. The header defines it as a pointer
/// to `struct as`.
pub type ddi_as_handle_t = *mut addrspace;

/// The current revision of `devmap_callback_ctl(9S)` (`DEVMAP_OPS_REV`). A
/// driver sets `devmap_rev` to this value.
pub const DEVMAP_OPS_REV: c_int = 1;

/// No special mapping behaviour requested (`DEVMAP_DEFAULTS`). This is the
/// `flags` value most drivers pass to `devmap_devmem_setup(9F)`.
pub const DEVMAP_DEFAULTS: uint_t = 0x00;
/// The mapping starts out without valid translations, so the first access
/// to every page calls `devmap_access(9E)` (`DEVMAP_MAPPING_INVALID`).
/// Drivers that switch device contexts between processes use it.
pub const DEVMAP_MAPPING_INVALID: uint_t = 0x01;
/// The driver may later remap the range to different memory with the remap
/// functions (`DEVMAP_ALLOW_REMAP`).
pub const DEVMAP_ALLOW_REMAP: uint_t = 0x02;
/// Load translations one page at a time rather than in the largest size
/// the MMU supports (`DEVMAP_USE_PAGESIZE`).
pub const DEVMAP_USE_PAGESIZE: uint_t = 0x04;

/// The access is an ordinary memory access to a page without a translation
/// (`DEVMAP_ACCESS`). A value of the `type` argument of `devmap_access(9E)`.
pub const DEVMAP_ACCESS: uint_t = 0;
/// The access violated the protection currently loaded for the page
/// (`DEVMAP_PROT`). A value of the `type` argument of `devmap_access(9E)`
/// that the manual page does not list.
pub const DEVMAP_PROT: uint_t = 1;
/// The system asks the driver to lock the range in memory (`DEVMAP_LOCK`).
/// A value of the `type` argument of `devmap_access(9E)`; the length
/// passed on to `devmap_do_ctxmgt(9F)` or `devmap_default_access(9F)` must
/// then equal the length received.
pub const DEVMAP_LOCK: uint_t = 2;
/// The system asks the driver to unlock a range locked earlier
/// (`DEVMAP_UNLOCK`). A value of the `type` argument of
/// `devmap_access(9E)` with the same length rule as `DEVMAP_LOCK`.
pub const DEVMAP_UNLOCK: uint_t = 3;

/// The direction of the access is unknown or the page was not touched
/// (`DEVMAP_OTHER`). A value of the `rw` argument of `devmap_access(9E)`
/// that the manual page does not list.
pub const DEVMAP_OTHER: uint_t = 0;
/// The process attempted to read (`DEVMAP_READ`). A value of the `rw`
/// argument of `devmap_access(9E)`.
pub const DEVMAP_READ: uint_t = 1;
/// The process attempted to write (`DEVMAP_WRITE`). A value of the `rw`
/// argument of `devmap_access(9E)`.
pub const DEVMAP_WRITE: uint_t = 2;
/// The process attempted to execute (`DEVMAP_EXEC`). A value of the `rw`
/// argument of `devmap_access(9E)`.
pub const DEVMAP_EXEC: uint_t = 3;
/// The page should be created if it does not exist (`DEVMAP_CREATE`). A
/// value of the `rw` argument of `devmap_access(9E)` that the manual page
/// does not list.
pub const DEVMAP_CREATE: uint_t = 4;

/// `devmap_map(9E)`. illumos DDI specific (illumos DDI).
///
/// The signature of the optional callback the system invokes once the user
/// mapping to device memory has been established, after `devmap(9E)` has
/// run. `dhp` identifies the mapping, `dev` is the device, `flags` is
/// `MAP_PRIVATE` or `MAP_SHARED`, and `off` and `len` give the user offset
/// and length of the mapping. The driver allocates its private data for
/// the mapping, records `off`, `len` and, if it switches contexts, `dhp` in
/// it, and stores the pointer through `pvtp`; the same pointer is passed to
/// every later callback on this mapping. Returns 0 on success or a non-zero
/// value to fail the `mmap(2)`.
pub type devmap_map_t = unsafe extern "C" fn(
    dhp: devmap_cookie_t,
    dev: dev_t,
    flags: uint_t,
    off: offset_t,
    len: size_t,
    pvtp: *mut *mut c_void,
) -> c_int;
/// `devmap_access(9E)`. illumos DDI specific (illumos DDI).
///
/// The signature of the optional callback the system invokes when the
/// process touches part of the mapping that has no translation or
/// insufficient protection. `dhp` identifies the mapping, `pvtp` is the
/// private data from `devmap_map(9E)`, `off` and `len` give the range being
/// accessed, `type_` is one of `DEVMAP_ACCESS`, `DEVMAP_LOCK` or
/// `DEVMAP_UNLOCK`, and `rw` is `DEVMAP_READ`, `DEVMAP_WRITE` or
/// `DEVMAP_EXEC`. The callback must load the translations before it
/// returns, by calling `devmap_do_ctxmgt(9F)` for mappings that switch
/// device contexts and `devmap_default_access(9F)` otherwise, and it should
/// return whatever that call returned. A non-zero return may deliver
/// `SIGSEGV` or `SIGBUS` to the process.
pub type devmap_access_t = unsafe extern "C" fn(
    dhp: devmap_cookie_t,
    pvtp: *mut c_void,
    off: offset_t,
    len: size_t,
    type_: uint_t,
    rw: uint_t,
) -> c_int;
/// `devmap_contextmgt(9E)`. illumos DDI specific (illumos DDI).
///
/// The signature of the driver routine that switches device contexts,
/// passed as the last argument of `devmap_do_ctxmgt(9F)` from within
/// `devmap_access(9E)`. It receives the same `dhp`, `pvtp`, `off`, `len`,
/// `type_` and `rw` as the access callback. The routine must check whether
/// another mapping currently owns the device context and, if so, call
/// `devmap_unload(9F)` on that mapping so its next access faults; it then
/// switches the hardware context and calls `devmap_load(9F)` on the mapping
/// that faulted. For `DEVMAP_LOCK` and `DEVMAP_UNLOCK` the length passed to
/// those two functions must equal `len`. A non-zero return propagates to
/// `devmap_access(9E)` and fails the access.
pub type devmap_contextmgt_t = unsafe extern "C" fn(
    dhp: devmap_cookie_t,
    pvtp: *mut c_void,
    off: offset_t,
    len: size_t,
    type_: uint_t,
    rw: uint_t,
) -> c_int;
/// `devmap_dup(9E)`. illumos DDI specific (illumos DDI).
///
/// The signature of the optional callback the system invokes when a mapping
/// is duplicated, as happens in `fork(2)`. `dhp` and `pvtp` describe the
/// original mapping and `new_dhp` the copy. The driver allocates private
/// data for the copy and stores it through `new_pvtp`. A non-zero return
/// fails the operation that caused the duplication.
pub type devmap_dup_t = unsafe extern "C" fn(
    dhp: devmap_cookie_t,
    pvtp: *mut c_void,
    new_dhp: devmap_cookie_t,
    new_pvtp: *mut *mut c_void,
) -> c_int;
/// `devmap_unmap(9E)`. illumos DDI specific (illumos DDI).
///
/// The signature of the optional callback the system invokes when the range
/// from `off` to `off + len` of the mapping is removed by `munmap(2)` or
/// `exit(2)`. When the whole mapping goes, all four `new_` arguments are
/// NULL and the driver frees everything it allocated for it. When only a
/// part goes, the mapping splits: `new_dhp1` is the handle of the remaining
/// piece that ends just before `off`, or NULL if there is none, and
/// `new_dhp2` the handle of the piece that starts at `off + len`, or NULL.
/// For each non-NULL handle the driver allocates fresh private data and
/// stores it through the matching `new_pvtp` pointer. In every case the
/// driver frees the old private data `pvtp` before returning.
pub type devmap_unmap_t = unsafe extern "C" fn(
    dhp: devmap_cookie_t,
    pvtp: *mut c_void,
    off: offset_t,
    len: size_t,
    new_dhp1: devmap_cookie_t,
    new_pvtp1: *mut *mut c_void,
    new_dhp2: devmap_cookie_t,
    new_pvtp2: *mut *mut c_void,
);

/// `devmap_callback_ctl(9S)`. illumos DDI specific (illumos DDI).
///
/// The set of callbacks through which the system tells a driver about
/// events on a mapping. A driver fills one in and passes it to
/// `devmap_devmem_setup(9F)` or `devmap_umem_setup(9F)` inside its
/// `devmap(9E)` entry point; the system copies the structure, so it may be
/// freed or reused afterwards, and different mappings may use different
/// callback sets. Any slot the driver does not need is `None`. The context
/// switching routine is not a member of this structure; it is passed
/// directly to `devmap_do_ctxmgt(9F)`.
#[repr(C)]
pub struct devmap_callback_ctl {
    /// Always [`DEVMAP_OPS_REV`].
    pub devmap_rev: c_int,
    /// The `devmap_map(9E)` callback, run during `mmap(2)`.
    pub devmap_map: Option<devmap_map_t>,
    /// The `devmap_access(9E)` callback, run on an access without a
    /// translation.
    pub devmap_access: Option<devmap_access_t>,
    /// The `devmap_dup(9E)` callback, run during `fork(2)`.
    pub devmap_dup: Option<devmap_dup_t>,
    /// The `devmap_unmap(9E)` callback, run during `munmap(2)` and
    /// `exit(2)`.
    pub devmap_unmap: Option<devmap_unmap_t>,
}

// Layout under LP64: devmap_rev 4 at 0, 4 bytes of padding, then four
// function pointers of 8 bytes at 8, 16, 24 and 32; total 40.
const _: () = assert!(core::mem::size_of::<devmap_callback_ctl>() == 40);
const _: () = assert!(core::mem::offset_of!(devmap_callback_ctl, devmap_map) == 8);
const _: () = assert!(core::mem::offset_of!(devmap_callback_ctl, devmap_unmap) == 32);

unsafe extern "C" {
    /// `devmap_setup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets up a user mapping to device memory through the devmap
    /// framework. A driver calls it from its `segmap(9E)` entry point,
    /// passing the arguments straight through: the device `dev`, the user
    /// offset `off`, the address space `as_`, the address `addrp` the
    /// mapping should occupy or receive, the length `len`, the requested
    /// protection `prot` and the maximum protection `maxprot` as `PROT_*`
    /// bits, the `MAP_SHARED`, `MAP_PRIVATE` or `MAP_FIXED` flags, and the
    /// caller's credentials. The framework calls the driver's `devmap(9E)`
    /// entry point to validate the range and translate the offset, and
    /// returns `EINVAL` if there is no such entry point. Returns 0 on
    /// success or an error number that the `segmap(9E)` entry point should
    /// return unchanged. May be called from user or kernel context only.
    pub fn devmap_setup(
        dev: dev_t,
        off: offset_t,
        as_: ddi_as_handle_t,
        addrp: *mut caddr_t,
        len: size_t,
        prot: uint_t,
        maxprot: uint_t,
        flags: uint_t,
        cred: *mut cred_t,
    ) -> c_int;
    /// `ddi_devmap_segmap(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// The same as [`devmap_setup`] except that `off` and `len` are `off_t`
    /// rather than `offset_t` and `size_t`, which makes its signature match
    /// `segmap(9E)` exactly. A driver may therefore install it directly as
    /// the `cb_segmap` entry point rather than writing one that forwards to
    /// `devmap_setup`. Returns 0 on success or an error number. May be
    /// called from user or kernel context only.
    pub fn ddi_devmap_segmap(
        dev: dev_t,
        off: off_t,
        as_: ddi_as_handle_t,
        addrp: *mut caddr_t,
        len: off_t,
        prot: uint_t,
        maxprot: uint_t,
        flags: uint_t,
        cred: *mut cred_t,
    ) -> c_int;
    /// `devmap_devmem_setup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Called from `devmap(9E)` to map device registers. `dhp` is the
    /// mapping handle received by the entry point, `dip` the device node,
    /// and `callback_ops` the driver's `devmap_callback_ctl(9S)` or NULL if
    /// it wants no callbacks; the structure is copied. `rnumber` selects the
    /// register set and `roff` the offset into it; `len` bytes from `roff`
    /// must be physically contiguous and page aligned. `maxprot` is the
    /// most permissive `PROT_*` combination the mapping may ever have,
    /// `flags` selects the cache attribute with `IOMEM_DATA_CACHED`,
    /// `IOMEM_DATA_UC_WR_COMBINE` or `IOMEM_DATA_UNCACHED`, and `accattrp`
    /// gives the device access attributes, of which only the endianness is
    /// meaningful and only on SPARC. Returns 0 on success and -1 on
    /// failure. May be called from user, kernel or interrupt context.
    pub fn devmap_devmem_setup(
        dhp: devmap_cookie_t,
        dip: dev_info_t,
        callback_ops: *mut devmap_callback_ctl,
        rnumber: uint_t,
        roff: offset_t,
        len: size_t,
        maxprot: uint_t,
        flags: uint_t,
        accattrp: *const ddi_device_acc_attr_t,
    ) -> c_int;
    /// `devmap_umem_setup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Called from `devmap(9E)` to export kernel memory allocated with
    /// `ddi_umem_alloc(9F)` to the process. `dhp`, `dip`, `callback_ops` and
    /// `maxprot` are as for [`devmap_devmem_setup`]. `cookie` is the memory
    /// cookie from `ddi_umem_alloc(9F)`, and `off` and `len` select a page
    /// aligned range within it; a NULL cookie fails the call. `flags` must
    /// be 0 and `accattrp` is currently ignored. Unless the mapping is
    /// `MAP_FIXED`, the system picks a user address aligned with the kernel
    /// address to avoid cache aliasing. Returns 0 on success and -1 on
    /// failure. May be called from user, kernel or interrupt context.
    pub fn devmap_umem_setup(
        dhp: devmap_cookie_t,
        dip: dev_info_t,
        callback_ops: *mut devmap_callback_ctl,
        cookie: crate::umem::ddi_umem_cookie_t,
        off: offset_t,
        len: size_t,
        maxprot: uint_t,
        flags: uint_t,
        accattrp: *const ddi_device_acc_attr_t,
    ) -> c_int;
    /// `devmap_default_access(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Provides the default semantics of `devmap_access(9E)` for mappings
    /// that do not switch device contexts: it loads the translations for
    /// the range and returns. A driver either calls it from its own
    /// `devmap_access(9E)` callback or installs it as that callback
    /// directly. The arguments are exactly those the callback received and
    /// must not be modified. Returns 0 on success or a non-zero value on
    /// failure, which the callback should return as is. Must be called from
    /// the `devmap_access(9E)` entry point.
    pub fn devmap_default_access(
        dhp: devmap_cookie_t,
        pvtp: *mut c_void,
        off: offset_t,
        len: size_t,
        type_: uint_t,
        rw: uint_t,
    ) -> c_int;
    /// `devmap_do_ctxmgt(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs device context switching for the range `off` to `off + len`
    /// of the mapping `dhp` by arranging for the system to call the
    /// driver's `devmap_contextmgt(9E)` routine `ctxmgt` with `dhp`, `pvtp`,
    /// `off`, `len`, `type_` and `rw`. A driver calls it from its
    /// `devmap_access(9E)` callback for the parts of the mapping that
    /// support context switching; `dhp`, `pvtp`, `type_` and `rw` are those
    /// the callback received and must not be modified. The value returned
    /// by the context management routine is returned to the caller. If the
    /// routine is not valid, the access fails and the process may receive
    /// `SIGSEGV` or `SIGBUS`. Must be called from the `devmap_access(9E)`
    /// entry point.
    pub fn devmap_do_ctxmgt(
        dhp: devmap_cookie_t,
        pvtp: *mut c_void,
        off: offset_t,
        len: size_t,
        type_: uint_t,
        rw: uint_t,
        ctxmgt: devmap_contextmgt_t,
    ) -> c_int;
    /// `devmap_set_ctx_timeout(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets the minimum interval, in clock ticks, that the system waits
    /// between successive calls to the driver's `devmap_contextmgt(9E)`
    /// routine for the mapping `dhp`, which keeps two processes from
    /// thrashing the device context. Drivers typically call it from
    /// `devmap_map(9E)`. The default of 0 imposes no delay. May be called
    /// from user, kernel or interrupt context.
    pub fn devmap_set_ctx_timeout(dhp: devmap_cookie_t, ticks: clock_t);
    /// `devmap_load(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Validates the translations of the mapping `dhp` for the range from
    /// `offset` to `offset + len`, rounded out to whole pages. A length of
    /// 0 means everything from `offset` to the end of the mapping, and an
    /// offset and length of 0 mean the whole mapping. `type_` and `rw` are
    /// the values the calling routine, usually `devmap_contextmgt(9E)`,
    /// received from the system and must not be modified. Only the mappings
    /// of the calling process may be loaded; loading another process's
    /// mapping panics the system. Returns 0 on success or a non-zero value
    /// that fails the access. May be called from user or kernel context
    /// only.
    pub fn devmap_load(dhp: devmap_cookie_t, offset: offset_t, len: size_t, type_: uint_t, rw: uint_t) -> c_int;
    /// `devmap_unload(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Invalidates the translations of the mapping `dhp` for the range from
    /// `offset` to `offset + len`, rounded out to whole pages and with the
    /// same zero length conventions as [`devmap_load`], so that the next
    /// access to the range calls `devmap_access(9E)` again. Unlike
    /// [`devmap_load`] it may be applied to the mappings of other processes,
    /// which is how a context management routine takes the device away from
    /// the process that last used it. Returns 0 on success or a non-zero
    /// value on failure. May be called from user or kernel context only.
    pub fn devmap_unload(dhp: devmap_cookie_t, offset: offset_t, len: size_t) -> c_int;
}
