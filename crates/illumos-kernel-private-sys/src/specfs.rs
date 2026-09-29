//! The special file system, `sys/fs/snode.h` and
//! `uts/common/fs/specfs/specsubr.c`.

use ddi_sys::dev_t;

use crate::vnode::{vnode, vtype_t};

unsafe extern "C" {
    /// `makespecvp`. Private, `uts/common/sys/fs/snode.h:153`.
    ///
    /// Returns a new specfs vnode (with one reference) for the device
    /// number `dev` of type `type_` ([`VCHR`](crate::vnode::VCHR)), not
    /// associated with any name in the file system, as the layered driver
    /// interface and the clone driver make them. `spec_open` finds and holds
    /// the device node on the first open. Allocates with `KM_SLEEP`.
    pub fn makespecvp(dev: dev_t, type_: vtype_t) -> *mut vnode;
    /// `specfind`. Private, `uts/common/sys/fs/snode.h:156`.
    ///
    /// Returns a held specfs vnode (not a common one) for device number
    /// `dev` of type `type_` whose cached path is visible in the caller's
    /// zone, or NULL. The devmap framework's `segdev_create` uses it for
    /// every mapping of a character device and assumes it finds one: a
    /// vnode without a visible path (`makespecvp` gives none) makes it
    /// dereference NULL.
    pub fn specfind(dev: dev_t, type_: vtype_t) -> *mut vnode;
}
