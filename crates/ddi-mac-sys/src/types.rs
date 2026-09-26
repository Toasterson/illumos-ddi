//! Opaque handles from `sys/mac.h` and `sys/mac_provider.h`.
//!
//! The MAC framework and a provider refer to each other through handles
//! whose contents neither side may inspect. The framework hands out
//! [`mac_handle_t`] from `mac_register(9F)`, [`mac_ring_handle_t`] from
//! `mr_rget(9E)` and [`mac_group_handle_t`] from `mr_gget(9E)`. In the other
//! direction, the provider stores its own pointers in [`mac_ring_driver_t`],
//! [`mac_group_driver_t`] and [`mac_intr_handle_t`], and the framework
//! passes them back unchanged to the ring, group and interrupt entry points.
//!
//! Every handle is declared in C as a pointer to an incomplete structure.
//! This module declares each of those structures with [`ddi_sys::opaque!`],
//! so that they can only ever appear behind a raw pointer, and gives the
//! pointer typedefs their C names.

use ddi_sys::opaque;

opaque! {
    /// The structure behind [`mac_handle_t`], `struct __mac_handle`. The
    /// kernel never defines it; the handle is a disguised pointer to the
    /// framework's private per-instance state.
    __mac_handle;
    /// The structure behind [`mac_resource_handle_t`],
    /// `struct __mac_resource_handle`. The kernel never defines it.
    __mac_resource_handle;
    /// The structure behind [`mac_intr_handle_t`], `struct __mac_intr_handle`.
    /// The provider stores a pointer to its own data in this handle.
    __mac_intr_handle;
    /// The structure behind [`mac_ring_handle_t`], `struct __mac_ring_handle`.
    /// The kernel never defines it; the handle is a disguised pointer to the
    /// framework's private ring state.
    __mac_ring_handle;
    /// The structure behind [`mac_group_handle_t`],
    /// `struct __mac_group_handle`. The kernel never defines it; the handle
    /// is a disguised pointer to the framework's private group state.
    __mac_group_handle;
    /// The structure behind [`mac_ring_driver_t`], `struct __mac_ring_driver`.
    /// The provider stores a pointer to its own ring data in this handle.
    __mac_ring_driver;
    /// The structure behind [`mac_group_driver_t`],
    /// `struct __mac_group_driver`. The provider stores a pointer to its own
    /// group data in this handle.
    __mac_group_driver;
    /// The structure behind [`mac_prop_info_handle_t`],
    /// `struct __mac_prop_info_handle`. The kernel never defines it; the
    /// handle is a disguised pointer to private property information state.
    __mac_prop_info_handle;
    /// `mac_transceiver_info(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// The transceiver information structure, `mac_transceiver_info_t`, that
    /// the framework passes to the `mct_info(9E)` entry point. Its layout is
    /// private to the kernel, and the provider fills it in only through the
    /// `mac_transceiver_info_set_present(9F)` and
    /// `mac_transceiver_info_set_usable(9F)` functions.
    mac_transceiver_info_t;
}

/// The handle to a registered MAC provider instance, `mac_handle_t`.
/// `mac_register(9F)` stores it through its second argument, and the
/// provider passes it to `mac_rx(9F)`, `mac_tx_update(9F)`,
/// `mac_link_update(9F)`, `mac_maxsdu_update(9F)` and `mac_unregister(9F)`.
/// The manual recommends keeping it in the instance's soft state.
pub type mac_handle_t = *mut __mac_handle;

/// A receive resource handle, `mac_resource_handle_t`. `mac_rx(9F)` takes
/// one as its second argument, which the manual reserves and requires to be
/// NULL.
pub type mac_resource_handle_t = *mut __mac_resource_handle;

/// The provider's handle for one ring's interrupt, `mac_intr_handle_t`. The
/// provider stores it in the `mi_handle` member of `mac_intr(9S)`, and the
/// framework passes it back to `mi_enable(9E)` and `mi_disable(9E)`.
pub type mac_intr_handle_t = *mut __mac_intr_handle;

/// The framework's handle for one ring, `mac_ring_handle_t`. The provider
/// receives it in `mr_rget(9E)` and passes it to `mac_rx_ring(9F)`,
/// `mac_tx_ring_update(9F)` and `mac_ring_intr_set`.
pub type mac_ring_handle_t = *mut __mac_ring_handle;

/// The framework's handle for one ring group, `mac_group_handle_t`. The
/// provider receives it in `mr_gget(9E)`.
pub type mac_group_handle_t = *mut __mac_group_handle;

/// The provider's handle for one ring, `mac_ring_driver_t`. The provider
/// stores it in the `mri_driver` member of `mac_ring_info(9S)`, and the
/// framework passes it back to the ring entry points such as `mri_start(9E)`
/// and `mri_stat(9E)`.
pub type mac_ring_driver_t = *mut __mac_ring_driver;

/// The provider's handle for one ring group, `mac_group_driver_t`. The
/// provider stores it in the `mgi_driver` member of `mac_group_info(9S)`,
/// and the framework passes it back to the group entry points such as
/// `mgi_start(9E)`.
pub type mac_group_driver_t = *mut __mac_group_driver;

/// The property information handle passed to `mc_propinfo(9E)`,
/// `mac_prop_info_handle_t`. The provider only ever passes it on to the
/// `mac_prop_info(9F)` functions.
pub type mac_prop_info_handle_t = *mut __mac_prop_info_handle;
