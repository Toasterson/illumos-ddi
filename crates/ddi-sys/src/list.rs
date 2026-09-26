//! Generic doubly linked lists from `sys/list.h`, as documented in
//! `list_create(9F)`.
//!
//! A list links structures of one type through a [`list_node_t`] that the
//! caller embeds in each of them. The list head, a [`list_t`], records the
//! size of the element type and the offset of the embedded node within it,
//! so that the functions can convert between element pointers and node
//! pointers. Every function except [`list_link_init`], [`list_link_active`]
//! and [`list_link_replace`] therefore takes and returns pointers to the
//! caller's elements rather than to nodes. A structure that must be on
//! several lists at once embeds one node per list.
//!
//! Both structures are defined in `sys/list_impl.h`, which the crate
//! otherwise avoids because such headers are private to the kernel. They are
//! declared here with their real layouts all the same, because a driver
//! embeds a `list_node_t` in each element and a `list_t` in its own soft
//! state, so their sizes and alignment are part of the ABI whether the
//! header calls them private or not. Drivers must still treat the fields as
//! read-only and change them only through the functions below. The
//! first node of a list points back into the `list_t`, so a list head must
//! not be moved or copied once `list_create` has initialised it.
//!
//! The manual page has no INTERFACE LEVEL section and gives the interface
//! stability as Committed, which is what the first line of each function's
//! documentation quotes. Every function may be called from user, kernel or
//! interrupt context. The lists do no locking of their own, so the caller
//! serialises access to each list.
//!
//! The `ddi` mapfile lists `list_create`, `list_destroy`, `list_head`,
//! `list_insert_head`, `list_insert_tail`, `list_next`, `list_remove` and
//! `list_remove_head`. The other functions are bound because the manual page
//! documents them.

use crate::types::*;

/// The link that a list element embeds, `struct list_node`, known in C as
/// `list_node_t`.
///
/// While the element is on a list the two pointers link it to its
/// neighbours, and the ends of the list point at the node inside the
/// [`list_t`]. An element that is on no list has both pointers NULL, which
/// is the state [`list_link_init`] produces and [`list_link_active`] tests.
#[repr(C)]
pub struct list_node {
    /// The node of the next element, or the list head's own node for the
    /// last element.
    pub list_next: *mut list_node,
    /// The node of the previous element, or the list head's own node for
    /// the first element.
    pub list_prev: *mut list_node,
}
/// The typedef name of [`list_node`].
pub type list_node_t = list_node;

// Layout under LP64: list_next 8 at 0, list_prev 8 at 8; total 16.
const _: () = assert!(core::mem::size_of::<list_node>() == 16);
const _: () = assert!(core::mem::offset_of!(list_node, list_next) == 0);
const _: () = assert!(core::mem::offset_of!(list_node, list_prev) == 8);

/// The head of a list, `struct list`, known in C as `list_t`.
///
/// The caller provides the storage, usually inside its own soft state, and
/// initialises it with [`list_create`]. The head embeds a node of its own
/// that closes the circle of links, which is why the structure must stay
/// where it was initialised until [`list_destroy`] has been called on it.
/// There is no manual page for the structure itself; `list_create(9F)`
/// describes how it is used.
#[repr(C)]
pub struct list {
    /// The size in bytes of one element, as passed to [`list_create`].
    pub list_size: size_t,
    /// The offset in bytes of the [`list_node_t`] within each element, as
    /// passed to [`list_create`].
    pub list_offset: size_t,
    /// The node that anchors the circular chain of element nodes. Its
    /// `list_next` is the first element's node and its `list_prev` the last
    /// element's; both point back at this node when the list is empty.
    pub list_head: list_node,
}
/// The typedef name of [`list`].
pub type list_t = list;

// Layout under LP64: list_size 8 at 0, list_offset 8 at 8, list_head 16 at
// 16; total 32.
const _: () = assert!(core::mem::size_of::<list>() == 32);
const _: () = assert!(core::mem::offset_of!(list, list_size) == 0);
const _: () = assert!(core::mem::offset_of!(list, list_offset) == 8);
const _: () = assert!(core::mem::offset_of!(list, list_head) == 16);

unsafe extern "C" {
    /// `list_create(9F)`. Interface stability: Committed.
    ///
    /// Initialises the list head at `list`, whose storage the caller
    /// provides. `size` is the size in bytes of one element and `offset` is
    /// the offset of the [`list_node_t`] within the element that this list
    /// uses for its links. Every element later put on the list must be of
    /// that layout. Once initialised, the list head must not be copied or
    /// moved to another address, because the list's nodes point back into
    /// it; the contents of a list are moved with [`list_move_tail`] instead.
    pub fn list_create(list: *mut list_t, size: size_t, offset: size_t);
    /// `list_destroy(9F)`. Interface stability: Committed.
    ///
    /// Destroys the list head at `list` and releases any resources that
    /// were allocated for it internally. The list must be empty when this
    /// function is called. The storage of the head itself belongs to the
    /// caller and is not freed.
    pub fn list_destroy(list: *mut list_t);
    /// `list_insert_after(9F)`. Interface stability: Committed.
    ///
    /// Inserts the element `new_item` into `list` directly after
    /// `reference_item`, which must already be on the list.
    pub fn list_insert_after(list: *mut list_t, reference_item: *mut c_void, new_item: *mut c_void);
    /// `list_insert_before(9F)`. Interface stability: Committed.
    ///
    /// Inserts the element `new_item` into `list` directly before
    /// `reference_item`, which must already be on the list.
    pub fn list_insert_before(list: *mut list_t, reference_item: *mut c_void, new_item: *mut c_void);
    /// `list_insert_head(9F)`. Interface stability: Committed.
    ///
    /// Inserts the element `new_item` at the head of `list`, making it the
    /// first element.
    pub fn list_insert_head(list: *mut list_t, new_item: *mut c_void);
    /// `list_insert_tail(9F)`. Interface stability: Committed.
    ///
    /// Inserts the element `new_item` at the tail of `list`, making it the
    /// last element.
    pub fn list_insert_tail(list: *mut list_t, new_item: *mut c_void);
    /// `list_remove(9F)`. Interface stability: Committed.
    ///
    /// Removes the element `item` from `list`. The element must be on that
    /// list. A caller that removes elements while walking the list reads
    /// the next element with [`list_next`] before removing the current one,
    /// as the manual page's first example shows.
    pub fn list_remove(list: *mut list_t, item: *mut c_void);
    /// `list_remove_head(9F)`. Interface stability: Committed.
    ///
    /// Removes the first element of `list` and returns it. If the list is
    /// empty, nothing changes and NULL is returned. Calling it until it
    /// returns NULL is the usual way to empty a list before
    /// [`list_destroy`].
    pub fn list_remove_head(list: *mut list_t) -> *mut c_void;
    /// `list_remove_tail(9F)`. Interface stability: Committed.
    ///
    /// Removes the last element of `list` and returns it. If the list is
    /// empty, nothing changes and NULL is returned.
    pub fn list_remove_tail(list: *mut list_t) -> *mut c_void;
    /// `list_move_tail(9F)`. Interface stability: Committed.
    ///
    /// Appends all the elements of the list `src` to the end of the list
    /// `dst`, leaving `src` empty. The two lists must have been initialised
    /// with identical size and offset arguments to [`list_create`].
    pub fn list_move_tail(dst: *mut list_t, src: *mut list_t);
    /// `list_head(9F)`. Interface stability: Committed.
    ///
    /// Returns the first element of `list` without removing it, or NULL if
    /// the list is empty.
    pub fn list_head(list: *mut list_t) -> *mut c_void;
    /// `list_tail(9F)`. Interface stability: Committed.
    ///
    /// Returns the last element of `list` without removing it, or NULL if
    /// the list is empty.
    pub fn list_tail(list: *mut list_t) -> *mut c_void;
    /// `list_next(9F)`. Interface stability: Committed.
    ///
    /// Returns the element that follows `reference_item` in `list`, or NULL
    /// if `reference_item` is the last element. The reference element must
    /// be linked on the list. Together with [`list_head`] this walks a list
    /// from front to back.
    pub fn list_next(list: *mut list_t, reference_item: *mut c_void) -> *mut c_void;
    /// `list_prev(9F)`. Interface stability: Committed.
    ///
    /// Returns the element that precedes `reference_item` in `list`, or
    /// NULL if `reference_item` is the first element. The reference element
    /// must be linked on the list. Together with [`list_tail`] this walks a
    /// list from back to front.
    pub fn list_prev(list: *mut list_t, reference_item: *mut c_void) -> *mut c_void;
    /// `list_is_empty(9F)`. Interface stability: Committed.
    ///
    /// Returns 0 if `list` has elements on it and a non-zero value if it is
    /// empty.
    pub fn list_is_empty(list: *mut list_t) -> c_int;
    /// `list_link_init(9F)`. Interface stability: Committed.
    ///
    /// Initialises the node at `node` to the unlinked state, which is
    /// equivalent to zeroing it. [`list_link_active`] reports 0 for a node
    /// in this state, so a driver that tests its elements with that function
    /// initialises each node this way before the element is first inserted.
    pub fn list_link_init(node: *mut list_node_t);
    /// `list_link_active(9F)`. Interface stability: Committed.
    ///
    /// Returns a non-zero value if the node at `node` is linked on a list
    /// and 0 otherwise.
    pub fn list_link_active(node: *mut list_node_t) -> c_int;
    /// `list_link_replace(9F)`. Interface stability: Committed.
    ///
    /// Puts the node `lnew` in the place that the node `lold` occupies on
    /// its list. `lnew` must not be linked on any list when the function is
    /// called, and `lold` is left unlinked from every list when it returns.
    pub fn list_link_replace(lold: *mut list_node_t, lnew: *mut list_node_t);
}
