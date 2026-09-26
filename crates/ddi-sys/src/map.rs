//! Resource maps, `rmalloc(9F)` and family, from `sys/map.h`.
//!
//! A resource map manages an arbitrary resource that is addressed by index
//! and measured in whatever unit the driver chooses, such as blocks, pages
//! or table slots. The driver allocates a map with `rmallocmap(9F)`, hands
//! the whole resource to it with one initial `rmfree(9F)`, and then draws
//! ranges from it with `rmalloc(9F)` or `rmalloc_wait(9F)` and returns them
//! with `rmfree`. Index 0 is reserved to mean failure, so the initial
//! `rmfree` conventionally starts at index 1.
//!
//! The current `sys/map.h` declares only `struct map;` and spells every
//! function with a `void *` map argument. The structure's layout is private
//! to the kernel, so it is declared opaque here, and the prototypes use
//! `*mut map` where the header uses `void *`; the two are identical at the
//! ABI and the typed pointer matches the manual pages.

use crate::types::*;

opaque! {
    /// A resource map allocated by `rmallocmap(9F)`, `struct map`. The
    /// header keeps the layout private and passes it as `void *`.
    map;
}

unsafe extern "C" {
    /// `rmallocmap(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Allocates and initialises a resource map with room for `mapsize`
    /// entries, which is the number of allocations that can be outstanding
    /// at any one time. The map does not yet describe any resource; a
    /// following call to [`rmfree`] covering the whole resource makes it
    /// available. Returns the new map or NULL if memory could not be
    /// obtained without blocking. May be called from user, kernel or
    /// interrupt context.
    pub fn rmallocmap(mapsize: size_t) -> *mut map;
    /// `rmallocmap(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Like [`rmallocmap`], but waits for memory to become available if
    /// necessary and therefore always returns a map. Documented on the
    /// `rmallocmap(9F)` page as `rmallocmap_wait`. May be called from user
    /// or kernel context only.
    pub fn rmallocmap_wait(mapsize: size_t) -> *mut map;
    /// `rmallocmap(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Deallocates a map obtained from [`rmallocmap`] or
    /// [`rmallocmap_wait`]. Documented on the `rmallocmap(9F)` page as
    /// `rmfreemap`. May be called from user, kernel or interrupt context.
    pub fn rmfreemap(mp: *mut map);
    /// `rmalloc(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Allocates `size` units of the resource described by the map and
    /// returns the index of the first unit. Returns 0 if no free range of
    /// that size exists, or if every map entry is already in use. May be
    /// called from user, interrupt or kernel context.
    pub fn rmalloc(mp: *mut map, size: size_t) -> ulong_t;
    /// `rmalloc_wait(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Like [`rmalloc`], but waits for space to become available if
    /// necessary and so always returns a valid index. The manual says it
    /// can be called from any context but that in most cases it should not
    /// be called from interrupt context.
    pub fn rmalloc_wait(mp: *mut map, size: size_t) -> ulong_t;
    /// `rmfree(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns `size` units starting at `index` to the map. The pair must
    /// exactly match what an earlier [`rmalloc`] or [`rmalloc_wait`]
    /// returned; space cannot be returned piecemeal. The same call, with a
    /// range covering the whole resource, initialises a freshly allocated
    /// map. May be called from user, interrupt or kernel context.
    pub fn rmfree(mp: *mut map, size: size_t, index: ulong_t);
}
