//! Generic AVL trees from `sys/avl.h`, as documented in `avl(9F)`.
//!
//! An AVL tree is a self-balancing binary search tree. It keeps structures
//! of one type in the order a caller-supplied comparison function defines,
//! and finds, inserts and removes them in logarithmic time. As with the
//! lists in [`crate::list`], the caller embeds a link, here an
//! [`avl_node_t`], in each structure, and the tree head, an
//! [`avl_tree_t`], records the element size and the offset of that link.
//! All functions take and return pointers to the caller's elements. A
//! structure that must be in several trees at once embeds one node per
//! tree.
//!
//! Both structures are defined in `sys/avl_impl.h`, which the crate
//! otherwise avoids because such headers are private to the kernel. They are
//! declared here with their real layouts all the same, because a driver
//! embeds an `avl_node_t` in each element and an `avl_tree_t` in its own
//! soft state, so their sizes and alignment are part of the ABI whether the
//! header calls them private or not. Drivers must still treat the fields as
//! read-only and change them only through the functions below. The layout
//! given is the LP64 one, in which the parent pointer, the child index and
//! the balance of a node share one word.
//!
//! The `avl(9F)` page has no INTERFACE LEVEL section and gives the interface
//! stability as Committed, which is what the first line of each function's
//! documentation quotes. It describes the family only in outline and refers
//! to the section 3AVL pages of the same names for the details, which the
//! descriptions here follow. Every function may be called from user, kernel
//! or interrupt context. The trees do no locking of their own, so the
//! caller serialises access to each tree.
//!
//! The page also documents `AVL_NEXT` and `AVL_PREV`. They are macros that
//! call [`avl_walk`] with [`AVL_AFTER`] and [`AVL_BEFORE`] respectively, so
//! the function is bound here and the macros belong to the safe crate, as do
//! the comparison helpers `AVL_CMP`, `AVL_ISIGN` and `AVL_PCMP`.

use crate::types::*;

/// A position in a tree where an element would be inserted, `avl_index_t`.
///
/// [`avl_find`] produces one when it does not find a match, and
/// [`avl_insert`] and [`avl_nearest`] consume it. The value encodes a node
/// pointer and a child direction and is only valid until the tree is next
/// modified. It is a `uintptr_t` in C.
pub type avl_index_t = uintptr_t;

/// The direction that selects the preceding element for [`avl_nearest`] and
/// [`avl_walk`], or insertion before the reference element for
/// [`avl_insert_here`] (`AVL_BEFORE`).
pub const AVL_BEFORE: c_int = 0;
/// The direction that selects the following element for [`avl_nearest`] and
/// [`avl_walk`], or insertion after the reference element for
/// [`avl_insert_here`] (`AVL_AFTER`).
pub const AVL_AFTER: c_int = 1;

/// The comparison function of a tree, as passed to [`avl_create`] and stored
/// in the `avl_compar` field of [`avl_tree_t`].
///
/// The tree calls it with pointers to two elements, which the function
/// casts to the element type. It must return exactly -1 if the first
/// element sorts before the second, 1 if it sorts after, and 0 if the two
/// compare equal. Any other return value results in undefined behaviour.
/// Two elements that compare equal are taken to represent the same data,
/// which is how [`avl_find`] recognises a match. The function is called
/// during every operation that traverses the tree, in the context of the
/// caller of that operation.
pub type avl_compar_t = unsafe extern "C" fn(first: *const c_void, second: *const c_void) -> c_int;

/// The link that a tree element embeds, `struct avl_node`, known in C as
/// `avl_node_t`.
///
/// There is no manual page for the structure itself; `avl(9F)` and
/// `avl_create(3AVL)` describe how it is used. Under LP64 the node holds its
/// two children and one packed word. That word keeps the parent pointer in
/// its high bits, the index of the node in its parent's `avl_child` array in
/// bit 2, and the balance of the node plus one in bits 0 and 1.
#[repr(C)]
pub struct avl_node {
    /// The left and right children of this node, those that sort before and
    /// after it, or NULL where there is no child.
    pub avl_child: [*mut avl_node; 2],
    /// The parent pointer, the child index and the balance packed into one
    /// word as described above. The C header's `AVL_XPARENT`, `AVL_XCHILD`
    /// and `AVL_XBALANCE` macros take it apart.
    pub avl_pcb: uintptr_t,
}
/// The typedef name of [`avl_node`].
pub type avl_node_t = avl_node;

// Layout under LP64: avl_child 16 at 0, avl_pcb 8 at 16; total 24.
const _: () = assert!(core::mem::size_of::<avl_node>() == 24);
const _: () = assert!(core::mem::offset_of!(avl_node, avl_child) == 0);
const _: () = assert!(core::mem::offset_of!(avl_node, avl_pcb) == 16);

/// The head of a tree, `struct avl_tree`, known in C as `avl_tree_t`.
///
/// The caller provides the storage, usually inside its own soft state, and
/// initialises it with [`avl_create`]. There is no manual page for the
/// structure itself; `avl(9F)` and `avl_create(3AVL)` describe how it is
/// used.
#[repr(C)]
pub struct avl_tree {
    /// The root node of the tree, or NULL if the tree is empty.
    pub avl_root: *mut avl_node,
    /// The comparison function passed to [`avl_create`].
    pub avl_compar: Option<avl_compar_t>,
    /// The offset in bytes of the [`avl_node_t`] within each element, as
    /// passed to [`avl_create`].
    pub avl_offset: size_t,
    /// The number of elements in the tree, as returned by
    /// [`avl_numnodes`].
    pub avl_numnodes: ulong_t,
    /// The size in bytes of one element, as passed to [`avl_create`].
    pub avl_size: size_t,
}
/// The typedef name of [`avl_tree`].
pub type avl_tree_t = avl_tree;

// Layout under LP64: avl_root 8 at 0, avl_compar 8 at 8, avl_offset 8 at 16,
// avl_numnodes 8 at 24, avl_size 8 at 32; total 40.
const _: () = assert!(core::mem::size_of::<avl_tree>() == 40);
const _: () = assert!(core::mem::offset_of!(avl_tree, avl_root) == 0);
const _: () = assert!(core::mem::offset_of!(avl_tree, avl_compar) == 8);
const _: () = assert!(core::mem::offset_of!(avl_tree, avl_offset) == 16);
const _: () = assert!(core::mem::offset_of!(avl_tree, avl_numnodes) == 24);
const _: () = assert!(core::mem::offset_of!(avl_tree, avl_size) == 32);

unsafe extern "C" {
    /// `avl_create(9F)`. Interface stability: Committed.
    ///
    /// Initialises the tree head at `tree`, whose storage the caller
    /// provides. `compar` is the function that orders the elements, `size`
    /// is the size in bytes of one element, and `offset` is the offset of
    /// the [`avl_node_t`] within the element that this tree uses. The
    /// function allocates nothing, so the lifetime and storage of the tree
    /// are entirely the caller's.
    pub fn avl_create(tree: *mut avl_tree_t, compar: Option<avl_compar_t>, size: size_t, offset: size_t);
    /// `avl_find(9F)`. Interface stability: Committed.
    ///
    /// Looks for an element of `tree` that compares equal to `node` and
    /// returns it, or returns NULL if there is none. `node` is an element
    /// built by the caller for the lookup, and only the fields that the
    /// comparison function reads need to be set. When no match is found and
    /// `where_` is not NULL, the function stores in it the position at which
    /// `node` would be inserted, for use with [`avl_insert`] or
    /// [`avl_nearest`]. That position stays valid only until the tree is
    /// next modified. When a match is found the value stored through
    /// `where_` is undefined. The C parameter is named `where`, which is a
    /// Rust keyword.
    pub fn avl_find(tree: *mut avl_tree_t, node: *const c_void, where_: *mut avl_index_t) -> *mut c_void;
    /// `avl_insert(9F)`. Interface stability: Committed.
    ///
    /// Inserts the element `node` into `tree` at the position `where_`,
    /// which must come from an [`avl_find`] call that failed to find the
    /// element, with no change to the tree in between. A caller that does
    /// not first check for a duplicate uses [`avl_add`] instead. The C
    /// parameter is named `where`, which is a Rust keyword.
    pub fn avl_insert(tree: *mut avl_tree_t, node: *mut c_void, where_: avl_index_t);
    /// `avl_insert_here(9F)`. Interface stability: Committed.
    ///
    /// Inserts the element `new_data` into `tree` directly before or after
    /// the element `here`, which must already be in the tree. `direction`
    /// is [`AVL_BEFORE`] or [`AVL_AFTER`]; any other value results in
    /// undefined behaviour. It lets a caller that already knows a
    /// neighbouring element avoid another [`avl_find`] call. The new element
    /// must belong at that place in the order the comparison function
    /// defines.
    pub fn avl_insert_here(tree: *mut avl_tree_t, new_data: *mut c_void, here: *mut c_void, direction: c_int);
    /// `avl_first(9F)`. Interface stability: Committed.
    ///
    /// Returns the element of `tree` that sorts first, or NULL if the tree
    /// is empty.
    pub fn avl_first(tree: *mut avl_tree_t) -> *mut c_void;
    /// `avl_last(9F)`. Interface stability: Committed.
    ///
    /// Returns the element of `tree` that sorts last, or NULL if the tree is
    /// empty.
    pub fn avl_last(tree: *mut avl_tree_t) -> *mut c_void;
    /// `avl_nearest(9F)`. Interface stability: Committed.
    ///
    /// Returns the element of `tree` next to the insertion position `where_`
    /// that an unsuccessful [`avl_find`] produced. With [`AVL_BEFORE`] as
    /// `direction` it returns the element that would precede an element
    /// inserted there, and with [`AVL_AFTER`] the element that would follow
    /// it. It returns NULL when there is no such element. If the tree has
    /// been modified since the [`avl_find`] call, the position is no longer
    /// valid and must be looked up again. The C parameter is named `where`,
    /// which is a Rust keyword.
    pub fn avl_nearest(tree: *mut avl_tree_t, where_: avl_index_t, direction: c_int) -> *mut c_void;
    /// `avl_add(9F)`. Interface stability: Committed.
    ///
    /// Inserts the element `node` into `tree`, finding its place with the
    /// comparison function. The element must not already be in the tree and
    /// must not compare equal to any element that is.
    pub fn avl_add(tree: *mut avl_tree_t, node: *mut c_void);
    /// `avl_remove(9F)`. Interface stability: Committed.
    ///
    /// Removes the element `node` from `tree`. The element must be in the
    /// tree; otherwise the behaviour is undefined.
    pub fn avl_remove(tree: *mut avl_tree_t, node: *mut c_void);
    /// `avl_update(9F)`. Interface stability: Committed.
    ///
    /// Moves the element `node` of `tree` to its correct place after the
    /// caller has changed fields that the comparison function reads. The
    /// element is only reinserted if its order relative to its nearest
    /// neighbours has changed. Returns `B_TRUE` if the element had to be
    /// moved and `B_FALSE` otherwise.
    pub fn avl_update(tree: *mut avl_tree_t, node: *mut c_void) -> boolean_t;
    /// `avl_update_lt(9F)`. Interface stability: Committed.
    ///
    /// Like [`avl_update`], but only compares the element with the element
    /// before it. It is for callers that know the element can only have
    /// moved towards the front of the order. Returns `B_TRUE` if the element
    /// had to be moved and `B_FALSE` otherwise.
    pub fn avl_update_lt(tree: *mut avl_tree_t, node: *mut c_void) -> boolean_t;
    /// `avl_update_gt(9F)`. Interface stability: Committed.
    ///
    /// Like [`avl_update`], but only compares the element with the element
    /// after it. It is for callers that know the element can only have moved
    /// towards the back of the order. Returns `B_TRUE` if the element had to
    /// be moved and `B_FALSE` otherwise.
    pub fn avl_update_gt(tree: *mut avl_tree_t, node: *mut c_void) -> boolean_t;
    /// `avl_swap(9F)`. Interface stability: Committed.
    ///
    /// Exchanges the contents of the trees `tree1` and `tree2`. The two trees
    /// must hold the same kind of element, which means they were created
    /// with identical arguments to [`avl_create`]; otherwise the behaviour
    /// is undefined.
    pub fn avl_swap(tree1: *mut avl_tree_t, tree2: *mut avl_tree_t);
    /// `avl_numnodes(9F)`. Interface stability: Committed.
    ///
    /// Returns the number of elements in `tree`.
    pub fn avl_numnodes(tree: *mut avl_tree_t) -> ulong_t;
    /// `avl_is_empty(9F)`. Interface stability: Committed.
    ///
    /// Returns `B_TRUE` if `tree` holds no elements and `B_FALSE` otherwise.
    pub fn avl_is_empty(tree: *mut avl_tree_t) -> boolean_t;
    /// `avl_destroy_nodes(9F)`. Interface stability: Committed.
    ///
    /// Removes one element from `tree` and returns it, so that the caller
    /// can release it, or returns NULL once the tree is empty. It empties a
    /// tree faster than repeated calls to [`avl_remove`]. Before the first
    /// call the caller sets a `void *` variable to NULL and passes its
    /// address as `cookie`; the function keeps its position there, and the
    /// caller must pass the same variable unchanged to every later call.
    /// After the first call the only functions that may be used on the tree
    /// are further calls to this one and finally [`avl_destroy`].
    pub fn avl_destroy_nodes(tree: *mut avl_tree_t, cookie: *mut *mut c_void) -> *mut c_void;
    /// `avl_destroy(9F)`. Interface stability: Committed.
    ///
    /// Destroys the tree head at `tree`, which must be empty. Calling it on
    /// a tree that still holds elements is a programming error. The tree
    /// must not be used again until it has been initialised anew with
    /// [`avl_create`]. The storage of the head itself belongs to the caller
    /// and is not freed.
    pub fn avl_destroy(tree: *mut avl_tree_t);
    /// Listed in the `ddi` mapfile; no manual page.
    ///
    /// Returns the element of `tree` that follows `node` when `direction`
    /// is [`AVL_AFTER`], or the one that precedes it when `direction` is
    /// [`AVL_BEFORE`]. It returns NULL when `node` is the last or the first
    /// element respectively. `node` must be in the tree. The function is
    /// declared in `sys/avl_impl.h` and is what the `AVL_NEXT` and
    /// `AVL_PREV` macros documented in `avl(9F)` expand to; together with
    /// [`avl_first`] and [`avl_last`] it walks a tree in order. It may be
    /// called from user, kernel or interrupt context.
    pub fn avl_walk(tree: *mut avl_tree_t, node: *mut c_void, direction: c_int) -> *mut c_void;
}
