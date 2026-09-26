//! Identifier spaces from `sys/id_space.h` and the 32-bit token table of
//! `id32_alloc(9F)` from `sys/id32.h`.
//!
//! An identifier space hands out integers from a set of ranges, keeps track
//! of which ones are in use, and takes them back. Drivers commonly use one
//! to manage minor numbers. The header defines `id_space_t` as a typedef of
//! `vmem_t`, because the implementation is a vmem arena created with
//! `VMC_IDENTIFIER`; this module preserves that alias.
//!
//! The `id32_*` functions solve a different problem. Hardware that can only
//! carry a 32-bit cookie through an I/O request cannot hold a 64-bit
//! kernel pointer, so the driver stores the pointer in a table and gives the
//! hardware a 32-bit token that maps back to it.

use crate::types::*;
use crate::vmem::vmem_t;

/// A generic identifier, `id_t`. Under LP64 it is an `int`.
pub type id_t = c_int;

/// An identifier space created by `id_space_create(9F)`, `id_space_t`. The
/// header defines it as an alias of `vmem_t`.
pub type id_space_t = vmem_t;

unsafe extern "C" {
    /// `id_space(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Creates an identifier space named `name` covering the range from
    /// `low` inclusive to `high` exclusive. The name should be unique; for
    /// drivers, combining the driver name and the instance number works.
    /// Returns the new space or NULL if it could not be created. May block,
    /// so it must only be called where a blocking allocation is acceptable.
    pub fn id_space_create(name: *const c_char, low: id_t, high: id_t) -> *mut id_space_t;
    /// `id_space(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Destroys an identifier space. Every identifier allocated from it must
    /// already have been freed. May block.
    pub fn id_space_destroy(idspace: *mut id_space_t);
    /// `id_space(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Adds the range from `low` inclusive to `high` exclusive to an
    /// existing identifier space, which allows disjoint ranges to be managed
    /// by a single space. The manual page omits the space argument from its
    /// synopsis; the header declares it as the first parameter. May block.
    pub fn id_space_extend(idspace: *mut id_space_t, low: id_t, high: id_t);
    /// `id_space(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Allocates the next identifier, preferring the one after the most
    /// recently allocated. Always returns an identifier in the range of the
    /// space, blocking if necessary. May be called from user or kernel
    /// context.
    pub fn id_alloc(idspace: *mut id_space_t) -> id_t;
    /// `id_space(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Like [`id_alloc`], but never blocks. Returns -1 if no identifier is
    /// available. May be called from interrupt context.
    pub fn id_alloc_nosleep(idspace: *mut id_space_t) -> id_t;
    /// `id_space(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Allocates the lowest available identifier, which keeps the set of
    /// allocated identifiers compact. Always returns an identifier in the
    /// range of the space, blocking if necessary. May be called from user
    /// or kernel context.
    pub fn id_allocff(idspace: *mut id_space_t) -> id_t;
    /// `id_space(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Like [`id_allocff`], but never blocks. Returns -1 if no identifier
    /// is available. May be called from interrupt context.
    pub fn id_allocff_nosleep(idspace: *mut id_space_t) -> id_t;
    /// `id_space(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Allocates the specific identifier `id` without blocking. Returns `id`
    /// on success and -1 if it is already allocated or outside the space.
    /// May be called from interrupt context.
    pub fn id_alloc_specific_nosleep(idspace: *mut id_space_t, id: id_t) -> id_t;
    /// `id_space(9F)`. Interface stability: illumos DDI specific.
    ///
    /// Returns an identifier to the space it was allocated from. Freeing an
    /// identifier that is not allocated is a programming error.
    pub fn id_free(idspace: *mut id_space_t, id: id_t);

    /// `id32_alloc(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Stores the pointer `ptr` in the kernel's token table and returns a
    /// 32-bit token for it. `flag` is `KM_SLEEP` or `KM_NOSLEEP` as for
    /// `kmem_alloc(9F)`. Returns 0 if `KM_NOSLEEP` was given and memory
    /// could not be allocated. Must not be called from interrupt context
    /// with `KM_SLEEP`.
    pub fn id32_alloc(ptr: *mut c_void, flag: c_int) -> u32;
    /// `id32_alloc(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Releases a token obtained from [`id32_alloc`]; the token is no longer
    /// valid afterwards. Documented on the `id32_alloc(9F)` page as
    /// `id32_free`. May be called from interrupt or kernel context.
    pub fn id32_free(token: u32);
    /// `id32_alloc(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Returns the pointer that was stored under `token`, or NULL if the
    /// token is not valid; callers must check for NULL before dereferencing
    /// the result. Documented on the `id32_alloc(9F)` page as
    /// `id32_lookup`. May be called from interrupt or kernel context.
    pub fn id32_lookup(token: u32) -> *mut c_void;
}
