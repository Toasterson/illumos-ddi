//! Registration with the MAC framework, from `sys/mac_provider.h` and
//! `sys/mac_ether.h`.
//!
//! A provider takes part in the framework in four steps, which `mac(9E)`
//! describes in its sections on initializing, registering and unregistering.
//! In `_init(9E)` it calls [`mac_init_ops`] on its `dev_ops(9S)` before
//! `mod_install(9F)`, and in `_fini(9E)` it calls [`mac_fini_ops`] after a
//! successful `mod_remove(9F)`. In `attach(9E)` it obtains a
//! [`mac_register_t`] from [`mac_alloc`], fills it in, passes it to
//! [`mac_register`] and releases it with [`mac_free`]. In `detach(9E)` it
//! calls [`mac_unregister`] and fails the detach if that call fails.
//!
//! The functions for providers that create minor nodes of their own,
//! [`mac_private_minor`], [`mac_getinfo`] and [`mac_devt_to_instance`], are
//! declared here as well, because they belong to the same part of the
//! lifecycle. None of the three is listed in the `mac` mapfile, although each
//! has a manual page and the `misc/mac` module exports it.

use core::ffi::CStr;

use ddi_sys::{c_char, c_int, c_void, ddi_info_cmd_t, dev_info_t, dev_ops, dev_t, minor_t, size_t, uint_t};

use crate::callbacks::mac_callbacks_t;
use crate::types::mac_handle_t;

/// The first stable version of the provider interface. A provider built
/// against it passes this value to `mac_alloc(9F)`, which refuses versions
/// the kernel does not support (`MAC_VERSION_V1`).
pub const MAC_VERSION_V1: uint_t = 0x1;

/// The version of the provider interface these bindings describe, which is
/// the value the manual tells every provider to pass to `mac_alloc(9F)`. It
/// is currently equal to [`MAC_VERSION_V1`] (`MAC_VERSION`).
pub const MAC_VERSION: uint_t = MAC_VERSION_V1;

/// The type identifier of the Ethernet MAC plugin, which a provider for an
/// IEEE 802.3 device stores in the `m_type_ident` member of
/// `mac_register(9S)`. The C header defines it as the string literal
/// `"mac_ether"`; here it is a C string, so the member takes
/// `MAC_PLUGIN_IDENT_ETHER.as_ptr()` (`MAC_PLUGIN_IDENT_ETHER`).
pub const MAC_PLUGIN_IDENT_ETHER: &CStr = c"mac_ether";

/// `mac_register(9S)`. illumos DDI specific.
///
/// The registration structure through which a GLDv3 provider describes one
/// instance of its device to the MAC framework. The provider obtains it from
/// `mac_alloc(9F)`, sets the members, and passes it to `mac_register(9F)`.
/// The framework copies what it needs during registration, so the provider
/// may release the structure with `mac_free(9F)` as soon as
/// `mac_register(9F)` has returned, whether that call succeeded or not.
///
/// The structure may only be allocated by `mac_alloc(9F)`, because the
/// framework allocates a larger private structure around it. A value of this
/// type must therefore never be created on the Rust side.
///
/// The last two members, `m_v12n` and `m_multicast_sdu`, are not described
/// by the manual page. They are declared so that the layout matches the
/// header, and `mac_alloc(9F)` leaves them at zero.
#[repr(C)]
pub struct mac_register_t {
    /// The interface version, set by `mac_alloc(9F)`. The provider must not
    /// change it.
    pub m_version: uint_t,
    /// The kind of networking device this instance represents, which selects
    /// the MAC plugin. An Ethernet provider sets it to
    /// [`MAC_PLUGIN_IDENT_ETHER`].
    pub m_type_ident: *const c_char,
    /// A private value of the provider, usually the soft state of the
    /// instance. The framework passes it as the first argument to the
    /// `mac_callbacks(9S)` entry points.
    pub m_driver: *mut c_void,
    /// The device node of the instance, as received in `attach(9E)`.
    pub m_dip: dev_info_t,
    /// Must be set to zero. The framework determines the instance itself.
    pub m_instance: uint_t,
    /// The source MAC address of the device, usually the factory default
    /// address. It must be at least as long as the plugin's address length,
    /// six bytes for Ethernet.
    pub m_src_addr: *mut u8,
    /// An optional fixed destination address for outgoing frames. Most
    /// providers set it to NULL.
    pub m_dst_addr: *mut u8,
    /// The entry points of the provider, which are described in
    /// `mac_callbacks(9S)`. The structure they point to must outlive the
    /// registration, so it is normally a static.
    pub m_callbacks: *mut mac_callbacks_t,
    /// The minimum service data unit, the smallest payload the device can
    /// transmit without its own headers. Zero means that the protocol has no
    /// minimum or that the provider and hardware pad short frames themselves.
    pub m_min_sdu: uint_t,
    /// The maximum service data unit, the largest payload the device can
    /// transmit without its own headers and VLAN tags. This is the MTU.
    pub m_max_sdu: uint_t,
    /// Data specific to the plugin named in `m_type_ident`. For Ethernet it
    /// must be NULL.
    pub m_pdata: *mut c_void,
    /// The size of `m_pdata` in bytes. For Ethernet it must be 0.
    pub m_pdata_size: size_t,
    /// The names of the provider's private properties, as a NULL-terminated
    /// array of NUL-terminated strings, or NULL if there are none. Each name
    /// should begin with an underscore.
    pub m_priv_props: *mut *mut c_char,
    /// The number of bytes the device accepts beyond the basic MAC header and
    /// `m_max_sdu`. An Ethernet device that accepts a VLAN tag sets it to the
    /// size of the tag, four bytes.
    pub m_margin: u32,
    /// The virtualization level, one of the `MAC_VIRT_*` values of the
    /// header. Not described by the manual page; `mac_alloc(9F)` leaves it at
    /// zero.
    pub m_v12n: u32,
    /// The largest multicast payload, if it differs from `m_max_sdu`. Not
    /// described by the manual page; `mac_alloc(9F)` leaves it at zero, which
    /// makes the framework use `m_max_sdu`.
    pub m_multicast_sdu: uint_t,
}

// mac_register_t under LP64: m_version at 0 padded to 8, three pointers at
// 8, 16 and 24, m_instance at 32 padded to 40, three pointers at 40, 48 and
// 56, m_min_sdu at 64, m_max_sdu at 68, m_pdata at 72, m_pdata_size at 80,
// m_priv_props at 88, m_margin at 96, m_v12n at 100, m_multicast_sdu at 104,
// tail padding to the 8-byte alignment; total 112.
const _: () = assert!(core::mem::size_of::<mac_register_t>() == 112);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_version) == 0);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_type_ident) == 8);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_dip) == 24);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_instance) == 32);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_src_addr) == 40);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_callbacks) == 56);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_min_sdu) == 64);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_pdata) == 72);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_priv_props) == 88);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_margin) == 96);
const _: () = assert!(core::mem::offset_of!(mac_register_t, m_multicast_sdu) == 104);

unsafe extern "C" {
    /// `mac_alloc(9F)`. illumos DDI specific.
    ///
    /// Allocates a `mac_register(9S)` structure for a provider to fill in
    /// and pass to `mac_register(9F)`. The `mac_version` argument is the
    /// version of the interface the provider was built against, which should
    /// always be [`MAC_VERSION`]. The function returns a pointer to the new
    /// structure, or NULL if it could not be allocated, the most common cause
    /// being a version that the kernel does not support.
    ///
    /// It is generally called from `attach(9E)`, but may be called from user
    /// or kernel context.
    pub fn mac_alloc(mac_version: uint_t) -> *mut mac_register_t;

    /// `mac_alloc(9F)`. illumos DDI specific.
    ///
    /// Releases a structure obtained from [`mac_alloc`], together with any
    /// memory associated with it. The provider may call it as soon as
    /// `mac_register(9F)` has returned, whatever the outcome of that call.
    ///
    /// It is generally called from `attach(9E)`, but may be called from user
    /// or kernel context.
    pub fn mac_free(mregp: *mut mac_register_t);

    /// `mac_register(9F)`. illumos DDI specific.
    ///
    /// Registers one instance of the provider with the MAC framework, using
    /// the information in `mregp`, and stores the resulting handle through
    /// `mhp`. The provider keeps that handle, usually in its soft state, and
    /// passes it to every other MAC function it calls for this instance. As
    /// soon as the call succeeds the framework may begin calling the entry
    /// points in `mac_callbacks(9S)`, so the provider should have configured
    /// the device beforehand, while leaving its interrupts disabled until
    /// the call has returned.
    ///
    /// Returns 0 on success and an error number otherwise. `EEXIST` means
    /// that an instance with the same name and number is already
    /// registered, `EINVAL` that the registration information was invalid,
    /// for example because `mac_init_ops(9F)` was not called or a required
    /// entry point is missing, and `ENOSPC` that no minor number was left.
    /// On failure the provider unwinds its `attach(9E)` and fails it.
    ///
    /// It is generally called only from `attach(9E)`, but may be called from
    /// user or kernel context.
    pub fn mac_register(mregp: *mut mac_register_t, mhp: *mut mac_handle_t) -> c_int;

    /// `mac_register(9F)`. illumos DDI specific.
    ///
    /// Unregisters the instance identified by `mh` from the MAC framework. A
    /// provider calls it in `detach(9E)` after draining any transmit and
    /// receive buffers it has loaned to the system, and before tearing down
    /// anything else, and in `attach(9E)` when unwinding after a successful
    /// `mac_register(9F)`.
    ///
    /// Returns 0 on success and an error number otherwise. `EBUSY` means that
    /// the device is still in use and `ENOTEMPTY` that its flow table is not
    /// empty. When the call fails in `detach(9E)`, the provider must fail the
    /// detach with `DDI_FAILURE`.
    ///
    /// It is generally called only from `attach(9E)` and `detach(9E)`, but
    /// may be called from user or kernel context.
    pub fn mac_unregister(mh: mac_handle_t) -> c_int;

    /// `mac_init_ops(9F)`. illumos DDI specific.
    ///
    /// Prepares the provider's `dev_ops(9S)` structure `ops` for the MAC
    /// framework. The `name` argument is the NUL-terminated name of the
    /// driver. In the current implementation the function marks the driver's
    /// major number as a GLDv3 network driver, allocates a STREAMS table that
    /// it stores in the `cb_str` member of the `cb_ops(9S)` structure that
    /// `ops` points to, and installs the framework's `getinfo(9E)` if
    /// `devo_getinfo` is NULL. The `dev_ops(9S)` structure must therefore
    /// have a `cb_ops(9S)` structure and live in writable memory.
    ///
    /// The function must be called before `mod_install(9F)`, otherwise
    /// `mac_register(9F)` will fail later. If `mod_install(9F)` fails, the
    /// provider must undo it with [`mac_fini_ops`]. The function always
    /// succeeds and has no return value.
    ///
    /// It may only be called from `_init(9E)`.
    pub fn mac_init_ops(ops: *mut dev_ops, name: *const c_char);

    /// `mac_init_ops(9F)`. illumos DDI specific.
    ///
    /// Undoes [`mac_init_ops`] for the `dev_ops(9S)` structure `ops` and
    /// releases the STREAMS table it allocated. The provider calls it in
    /// `_fini(9E)` only after `mod_remove(9F)` has succeeded, and in
    /// `_init(9E)` if `mod_install(9F)` has failed. If `mod_remove(9F)` fails,
    /// the provider must not call it and must fail `_fini(9E)` instead. The
    /// function always succeeds and has no return value.
    ///
    /// It may only be called from `_init(9E)` or `_fini(9E)`.
    pub fn mac_fini_ops(ops: *mut dev_ops);

    /// `mac_devt_to_instance(9F)`. illumos DDI specific.
    ///
    /// Returns the instance number that the MAC framework associates with
    /// the device number `devt`, or -1 on failure. The framework normally
    /// supplies the provider's `getinfo(9E)`. A provider that creates minor
    /// nodes of its own, or that maps its instances differently, supplies its
    /// own `getinfo(9E)` and uses this function there to translate the minor
    /// numbers the framework reserves.
    ///
    /// It should be called from kernel context and is generally called from
    /// `getinfo(9E)`. The function has a manual page but is not listed in the
    /// `mac` mapfile.
    pub fn mac_devt_to_instance(devt: dev_t) -> c_int;

    /// `mac_getinfo(9F)`. illumos DDI specific.
    ///
    /// The framework's default `getinfo(9E)` entry point, with the same
    /// arguments. A provider that supplies its own `getinfo(9E)` because it
    /// creates minor nodes of its own calls this function for the minor
    /// numbers the framework reserves and returns its result. Returns
    /// `DDI_SUCCESS` or `DDI_FAILURE`.
    ///
    /// It should be used during the execution of a provider's `getinfo(9E)`.
    /// The function has a manual page but is not listed in the `mac` mapfile.
    pub fn mac_getinfo(dip: dev_info_t, cmd: ddi_info_cmd_t, arg: *mut c_void, resp: *mut *mut c_void) -> c_int;

    /// `mac_private_minor(9F)`. illumos DDI specific.
    ///
    /// Returns the first minor number that the provider may use for minor
    /// nodes of its own. Every minor number from this value upwards is free
    /// for the provider; every smaller one is reserved for the framework. A
    /// provider typically calls it in `attach(9E)` before
    /// `ddi_create_minor_node(9F)`, and must then also supply its own
    /// `getinfo(9E)`.
    ///
    /// It can be called from user or kernel context. The function has a
    /// manual page but is not listed in the `mac` mapfile.
    pub fn mac_private_minor() -> minor_t;
}
