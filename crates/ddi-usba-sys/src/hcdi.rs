//! The USBA host controller driver interface from `sys/usb/usba/hcdi.h`,
//! `sys/usb/usba/hubdi.h` and the parts of `sys/usb/usba/usba_types.h` that
//! these headers expose.
//!
//! A host controller driver, HCD for short, is an ordinary character driver
//! for USB host controller hardware that also acts as the root hub of its
//! bus. The `usba_hcdi(9E)` page describes the whole life cycle. In
//! `attach(9E)` the driver sets up its hardware and interrupts, allocates an
//! operations vector with [`usba_alloc_hcdi_ops`], fills it in, and
//! registers with [`usba_hcdi_register`]. It then binds its root hub with
//! [`usba_hubdi_bind_root_hub`], passing packed descriptors that make the
//! controller look like a hub of the matching USB generation. From then on
//! the framework calls the entry points of the operations vector to open
//! pipes and to start transfers. The transfer entry points are
//! asynchronous, and the driver reports every completed transfer with
//! [`usba_hcdi_cb`]. For periodic interrupt and isochronous IN transfers it
//! duplicates the original request with [`usba_hcdi_dup_intr_req`] or
//! [`usba_hcdi_dup_isoc_req`] for each batch of data and returns the
//! original only when polling ends. In `detach(9E)` it unbinds the root hub
//! with [`usba_hubdi_unbind_root_hub`] and unregisters with
//! [`usba_hcdi_unregister`].
//!
//! The driver has to route its `open(9E)`, `ioctl(9E)` and `close(9E)`
//! entry points to [`usba_hubdi_open`], [`usba_hubdi_ioctl`] and
//! [`usba_hubdi_close`], as the `usba_hcdi_cb_ops(9E)` page explains. Those
//! entry points have the ordinary signatures that `ddi-sys` already declares
//! as `cb_open_t`, `cb_ioctl_t` and `cb_close_t`, so no new types are
//! declared for them. The `dev_ops(9S)` of the driver uses
//! [`usba_hubdi_busops`] as its bus operations and
//! [`usba_hubdi_root_hub_power`] as its `power(9E)` entry point. The driver
//! must not use `ddi_set_driver_private(9F)`, which belongs to the
//! framework, and must mask [`HUBD_IS_ROOT_HUB`] out of its minor numbers.
//!
//! Every page in this family marks its interface as Volatile and private to
//! illumos USB host controller drivers, with the warning that it may be
//! removed or changed at any time. The bindings quote that level verbatim.
//!
//! [`usba_device`] and [`usba_pipe_handle_data`] are declared with their
//! complete layout, because the `usba_device(9S)` and
//! `usba_pipe_handle_data(9S)` pages document fields that a host controller
//! driver has to read and, in the case of `p_hcd_private` and
//! `p_req_count`, write. Fields the pages do not list are declared only so
//! that the offsets match and are documented as private to the framework.
//! Their types come from `usba_types.h`, which `hcdi.h` includes, except for
//! `usba_list_entry_t` from the private `usba_private.h`, which is declared
//! as a sized opaque type in the manner of `kmutex_t`, and for the structures
//! behind the private pointers, which are opaque.
//!
//! The header declares several functions that no manual page documents and
//! that are therefore not bound: `usba_hcdi_get_req_private`,
//! `usba_hcdi_set_req_private`, `usba_hcdi_get_ph_data`,
//! `usba_hcdi_get_data_toggle` and `usba_hcdi_set_data_toggle` in `hcdi.h`,
//! and `usba_hubdi_initialization`, `usba_hubdi_destruction`,
//! `usba_hubdi_info`, `usba_hubdi_attach`, `usba_hubdi_probe`,
//! `usba_hubdi_detach`, `usba_hubdi_quiesce`, `usba_hubdi_reset_device`,
//! `usba_hubdi_incr_power_budget`, `usba_hubdi_decr_power_budget` and
//! `usba_hubdi_check_power_budget` in `hubdi.h`, which serve the hub driver
//! and the framework. The kstat structures `hcdi_hotplug_stats_t` and
//! `hcdi_error_stats_t`, the `HCDI_HOTPLUG_STATS` and `HCDI_ERROR_STATS`
//! macros, `USB_N_COUNT_KSTATS` and `HUBDI_OPS_VERSION_0` have no page
//! either. `ROOT_HUB_ADDR`, the address of the root hub that
//! `usba_hcdi(9E)` tells drivers to compare with, is defined in the private
//! `sys/usb/hubd/hub.h` rather than in these headers; its value is 1.

use crate::client::{
    usb_bulk_req_t, usb_client_dev_data_t, usb_cr_t, usb_ctrl_req_t, usb_dev_descr_t, usb_ep_descr_t, usb_ep_xdescr_t,
    usb_flags_t, usb_frame_number_t, usb_intr_req_t, usb_isoc_req_t, usb_opaque_t, usb_pipe_policy_t, usb_pipe_state_t,
};
use ddi_sys::{
    bus_ops, c_char, c_int, c_uchar, c_void, cred_t, ddi_dma_attr_t, ddi_eventcookie_t, ddi_iblock_cookie_t,
    dev_info_t, dev_t, intptr_t, kmutex_t, mblk_t, minor_t, size_t, uint_t,
};

opaque! {
    /// The state a console driver keeps for polled input and output through
    /// a USB device, `usb_console_info_impl_t` from `genconsole.h`. The host
    /// controller driver receives it in the console entry points but its
    /// layout belongs to the framework.
    usb_console_info_impl;
    /// The hub driver interface of a device that is a hub, `usba_hubdi_t`
    /// from the private `usba_impl.h`. Only the NULL test on `usb_hubdi`
    /// matters to a host controller driver.
    usba_hubdi;
    /// The soft state of the hub driver, `struct hubd`, from the private hub
    /// driver header.
    hubd;
    /// A per device node record of registered NDI event callbacks,
    /// `usba_evdata_t`. Private to the framework.
    usba_evdata;
    /// The parsed Binary Object Store of a device, `usb_bos_t` from
    /// `bos.h`. Private to the framework.
    usb_bos;
    /// A kernel task queue, `taskq_t`. Private to the framework here.
    taskq;
    /// A kernel thread, `kthread_t`. Private to the framework here.
    kthread;
}

/// The typedef name of [`usb_console_info_impl`].
pub type usb_console_info_impl_t = usb_console_info_impl;
/// The typedef name of [`usba_hubdi`].
pub type usba_hubdi_t = usba_hubdi;

/// The address of a device on a USB bus, `usb_addr_t`.
pub type usb_addr_t = c_uchar;
/// The negotiated speed of a device, `usb_port_status_t`, one of the
/// `USBA_*_SPEED_DEV` values.
pub type usb_port_status_t = u8;
/// The number of a port on a hub, `usb_port_t`. Ports are numbered from 1.
pub type usb_port_t = u16;
/// A mask of hub ports, `usb_port_mask_t`.
pub type usb_port_mask_t = u32;

/// The device runs at low speed, as a USB 1.x or USB 2.0 device. Faster
/// speeds always have larger values (`USBA_LOW_SPEED_DEV`).
pub const USBA_LOW_SPEED_DEV: usb_port_status_t = 0x1;
/// The device runs at full speed, as a USB 1.x or USB 2.0 device
/// (`USBA_FULL_SPEED_DEV`).
pub const USBA_FULL_SPEED_DEV: usb_port_status_t = 0x2;
/// The device runs at high speed, as a USB 2.x device
/// (`USBA_HIGH_SPEED_DEV`).
pub const USBA_HIGH_SPEED_DEV: usb_port_status_t = 0x3;
/// The device runs at super speed, as a USB 3.0 device
/// (`USBA_SUPER_SPEED_DEV`).
pub const USBA_SUPER_SPEED_DEV: usb_port_status_t = 0x4;
/// The number of endpoints per device, 16 IN and 16 OUT, which sizes the
/// per endpoint arrays of [`usba_device`] (`USBA_N_ENDPOINTS`).
pub const USBA_N_ENDPOINTS: usize = 32;

/// The first version of the operations vector (`HCDI_OPS_VERSION_0`).
pub const HCDI_OPS_VERSION_0: c_int = 0;
/// The version of the operations vector that added polled console output
/// (`HCDI_OPS_VERSION_1`).
pub const HCDI_OPS_VERSION_1: c_int = 1;
/// The version of the operations vector that added the device
/// initialisation entry points (`HCDI_OPS_VERSION_2`).
pub const HCDI_OPS_VERSION_2: c_int = 2;
/// The current version of the operations vector, which drivers should
/// store in `usba_hcdi_ops_version` (`HCDI_OPS_VERSION`).
pub const HCDI_OPS_VERSION: c_int = HCDI_OPS_VERSION_2;
/// The first version of the registration arguments
/// (`HCDI_REGISTER_VERS_0`).
pub const HCDI_REGISTER_VERS_0: uint_t = 0;
/// The current version of the registration arguments, which drivers store
/// in `usba_hcdi_register_version` (`HCDI_REGISTER_VERSION`).
pub const HCDI_REGISTER_VERSION: uint_t = HCDI_REGISTER_VERS_0;
/// The timeout in seconds that a host controller driver uses for a
/// non-periodic transfer whose request gives a timeout of 0
/// (`HCDI_DEFAULT_TIMEOUT`).
pub const HCDI_DEFAULT_TIMEOUT: uint_t = 5;
/// The bit of the minor number that the framework uses for the root hub.
/// A host controller driver masks it out with `getminor(dev) &
/// !HUBD_IS_ROOT_HUB` before it uses the minor number itself
/// (`HUBD_IS_ROOT_HUB`).
pub const HUBD_IS_ROOT_HUB: minor_t = 0x1000;

// --- Private building blocks of the framework structures -----------------------

/// `usba_device(9S)`. Volatile - illumos USB HCD private.
///
/// A list entry of the framework, `usba_list_entry_t` from the private
/// `usba_private.h`. It consists of two list pointers, a mutex, a private
/// pointer and a count, but its contents are private to the framework. Only
/// its size and alignment matter, because it is embedded in
/// [`usba_device`] and [`usba_pipe_handle_data`].
#[repr(C)]
pub struct usba_list_entry {
    _opaque: [*mut c_void; 5],
}
/// The typedef name of [`usba_list_entry`].
pub type usba_list_entry_t = usba_list_entry;

// usba_list_entry: next at 0, prev at 8, list_mutex at 16, private at 24,
// count at 32, padded to 40 bytes.
const _: () = assert!(core::mem::size_of::<usba_list_entry>() == 40);
const _: () = assert!(core::mem::align_of::<usba_list_entry>() == 8);

/// `usba_device(9S)`. Volatile - illumos USB HCD private.
///
/// The backing structure of an opaque client pipe handle,
/// `usba_ph_impl_t`. The framework keeps one per endpoint in each
/// [`usba_device`], and it points to the [`usba_pipe_handle_data`] that host
/// controller drivers see. All of its fields are private to the framework.
#[repr(C)]
pub struct usba_ph_impl {
    /// Private to the framework. The mutex that protects this structure.
    pub usba_ph_mutex: kmutex_t,
    /// Private to the framework. The actual pipe handle.
    pub usba_ph_data: *mut usba_pipe_handle_data,
    /// Private to the framework. The device node that owns the pipe.
    pub usba_ph_dip: dev_info_t,
    /// Private to the framework. A saved copy of the endpoint descriptor.
    pub usba_ph_ep: usb_ep_descr_t,
    /// Private to the framework. A saved copy of the pipe policy.
    pub usba_ph_policy: usb_pipe_policy_t,
    /// Private to the framework. Flags for the data toggle and for
    /// persistent pipes.
    pub usba_ph_flags: uint_t,
    /// Private to the framework. The number of concurrent activities on the
    /// pipe.
    pub usba_ph_ref_count: c_int,
    /// Private to the framework. The state of the pipe.
    pub usba_ph_state: usb_pipe_state_t,
    /// Private to the framework. Non-zero while the state is changing.
    pub usba_ph_state_changing: c_int,
}
/// The typedef name of [`usba_ph_impl`].
pub type usba_ph_impl_t = usba_ph_impl;

// usba_ph_impl: usba_ph_mutex at 0, usba_ph_data at 8, usba_ph_dip at 16,
// usba_ph_ep (8 bytes, alignment 2) at 24, usba_ph_policy at 32,
// usba_ph_flags at 36, usba_ph_ref_count at 40, usba_ph_state at 44,
// usba_ph_state_changing at 48, padded to the pointer alignment, 56 bytes.
const _: () = assert!(core::mem::size_of::<usba_ph_impl>() == 56);
const _: () = assert!(core::mem::offset_of!(usba_ph_impl, usba_ph_mutex) == 0);
const _: () = assert!(core::mem::offset_of!(usba_ph_impl, usba_ph_data) == 8);
const _: () = assert!(core::mem::offset_of!(usba_ph_impl, usba_ph_ep) == 24);
const _: () = assert!(core::mem::offset_of!(usba_ph_impl, usba_ph_flags) == 36);
const _: () = assert!(core::mem::offset_of!(usba_ph_impl, usba_ph_state_changing) == 48);

/// `usba_device(9S)`. Volatile - illumos USB HCD private.
///
/// An element of the list of client data that the framework keeps for each
/// device, `usb_client_dev_data_list_t`. A client may request its device
/// data several times, so the framework records every copy. It is embedded
/// in [`usba_device`] and all of its fields are private to the framework.
#[repr(C)]
pub struct usb_client_dev_data_list {
    /// Private to the framework. The next element.
    pub cddl_next: *mut usb_client_dev_data_list,
    /// Private to the framework. The previous element.
    pub cddl_prev: *mut usb_client_dev_data_list,
    /// Private to the framework. The device node of the client.
    pub cddl_dip: dev_info_t,
    /// Private to the framework. The data returned to the client.
    pub cddl_dev_data: *mut usb_client_dev_data_t,
    /// Private to the framework. The interface number of the client.
    pub cddl_ifno: uint_t,
}
/// The typedef name of [`usb_client_dev_data_list`].
pub type usb_client_dev_data_list_t = usb_client_dev_data_list;

// usb_client_dev_data_list: four pointers from 0 to 24, cddl_ifno at 32,
// padded to 40 bytes.
const _: () = assert!(core::mem::size_of::<usb_client_dev_data_list>() == 40);
const _: () = assert!(core::mem::offset_of!(usb_client_dev_data_list, cddl_next) == 0);
const _: () = assert!(core::mem::offset_of!(usb_client_dev_data_list, cddl_dev_data) == 24);
const _: () = assert!(core::mem::offset_of!(usb_client_dev_data_list, cddl_ifno) == 32);

// --- usba_device(9S) ------------------------------------------------------------

/// `usba_device(9S)`. Volatile - illumos USB HCD private.
///
/// The structure by which the framework represents a physical USB device,
/// `usba_device_t`. A composite device that implements several classes
/// still has a single one. Every device except the root hub is plugged into
/// a port of some hub and has an address on the bus. Many entry points of
/// the operations vector receive this structure, and a host controller
/// driver may only read it. The page lists `usb_dip`, `usb_hubdi`,
/// `usb_addr`, `usb_dev_descr`, `usb_mfg_str`, `usb_product_str`,
/// `usb_serialno_str`, `usb_port_status`, `usb_port`, `usb_hs_hub_usba_dev`
/// and `usb_parent_hub` as the fields useful to such a driver. The other
/// fields are private to the framework and are declared only so that the
/// layout matches. Per device private data of the driver is kept by the
/// framework and read with [`usba_hcdi_get_device_private`].
#[repr(C)]
pub struct usba_device {
    /// Private to the framework. Links all devices on this bus.
    pub usb_device_list: usba_list_entry_t,
    /// Private to the framework. The pipe handles of the device, one per
    /// endpoint.
    pub usb_ph_list: [usba_ph_impl; USBA_N_ENDPOINTS],
    /// Private to the framework. The mutex that protects this structure.
    pub usb_mutex: kmutex_t,
    /// The device node of the device. A host controller driver generally
    /// uses it for naming in diagnostics. When duplicating requests it
    /// should use the device node of the pipe handle instead.
    pub usb_dip: dev_info_t,
    /// Private to the framework. The operations vector of the host
    /// controller the device is attached to.
    pub usb_hcdi_ops: *mut usba_hcdi_ops,
    /// The hub driver interface of the device. It is not NULL exactly when
    /// the device is a hub.
    pub usb_hubdi: *mut usba_hubdi,
    /// The address of the device on the bus as assigned by the framework.
    /// The address actually used on the bus may differ when the driver
    /// implements `usba_hcdi_device_address(9E)`. The root hub always has
    /// the address `ROOT_HUB_ADDR`, which is 1.
    pub usb_addr: usb_addr_t,
    /// Private to the framework. Whether checkpoint and resume is disabled.
    pub usb_no_cpr: c_uchar,
    /// Private to the framework. The device node of the root hub.
    pub usb_root_hub_dip: dev_info_t,
    /// Private to the framework. The soft state of the root hub driver.
    pub usb_root_hubd: *mut hubd,
    /// The device descriptor of the device. It may be NULL, because it may
    /// not have been read yet when the device is attached, so a driver must
    /// check before dereferencing it.
    pub usb_dev_descr: *mut usb_dev_descr_t,
    /// Private to the framework. The raw configuration descriptor.
    pub usb_cfg: *mut c_uchar,
    /// Private to the framework. The length of the raw configuration
    /// descriptor.
    pub usb_cfg_length: size_t,
    /// The name of the manufacturer as read from the device, or NULL.
    pub usb_mfg_str: *mut c_char,
    /// The name of the product as read from the device, or NULL.
    pub usb_product_str: *mut c_char,
    /// The serial number as read from the device, or NULL.
    pub usb_serialno_str: *mut c_char,
    /// Private to the framework. The driver the user prefers for the
    /// device.
    pub usb_preferred_driver: *mut c_char,
    /// The current negotiated speed of the device, one of the
    /// `USBA_*_SPEED_DEV` values.
    pub usb_port_status: usb_port_status_t,
    /// The port of the hub the device is plugged into, counting from 1.
    pub usb_port: usb_port_t,
    /// The closest parent hub that runs at high speed, which low and full
    /// speed devices need for split transactions. It is always NULL for
    /// SuperSpeed devices, which can never be plugged into a high speed hub.
    pub usb_hs_hub_usba_dev: *mut usba_device,
    /// Private to the framework. The address of the high speed hub.
    pub usb_hs_hub_addr: usb_addr_t,
    /// Private to the framework. The port of the high speed hub.
    pub usb_hs_hub_port: usb_port_t,
    /// Private to the framework. The minimum bandwidth of the high speed
    /// hub.
    pub usb_hs_hub_min_bandwidth: uint_t,
    /// Private to the framework. The bandwidth allocated per microframe on
    /// the high speed hub.
    pub usb_hs_hub_bandwidth: [uint_t; 32],
    /// Private to the framework. The raw descriptors of all configurations.
    pub usb_cfg_array: *mut *mut c_uchar,
    /// Private to the framework. The number of entries in `usb_cfg_array`.
    pub usb_cfg_array_length: uint_t,
    /// Private to the framework. The lengths of the raw configurations.
    pub usb_cfg_array_len: *mut u16,
    /// Private to the framework. The number of entries in
    /// `usb_cfg_array_len`.
    pub usb_cfg_array_len_length: uint_t,
    /// Private to the framework. The current configuration value.
    pub usb_cfg_value: uint_t,
    /// Private to the framework. The index of the active configuration.
    pub usb_active_cfg_ndx: uint_t,
    /// Private to the framework. The configuration strings.
    pub usb_cfg_str_descr: *mut *mut c_char,
    /// Private to the framework. The number of configurations.
    pub usb_n_cfgs: c_uchar,
    /// Private to the framework. The number of interfaces.
    pub usb_n_ifs: c_uchar,
    /// Private to the framework. The power drawn from the parent hub.
    pub usb_pwr_from_hub: u16,
    /// Private to the framework. The reference count of this structure.
    pub usb_ref_count: c_int,
    /// Private to the framework. The requests allocated for this device,
    /// kept to detect leaks.
    pub usb_allocated: usba_list_entry_t,
    /// Private to the framework. The NDI event cookie for removal.
    pub rm_cookie: ddi_eventcookie_t,
    /// Private to the framework. The NDI event cookie for insertion.
    pub ins_cookie: ddi_eventcookie_t,
    /// Private to the framework. The NDI event cookie for suspend.
    pub suspend_cookie: ddi_eventcookie_t,
    /// Private to the framework. The NDI event cookie for resume.
    pub resume_cookie: ddi_eventcookie_t,
    /// Private to the framework. The event callbacks per device node.
    pub usb_evdata: *mut usba_evdata,
    /// Private to the framework. Flags for the cleanup checks of each
    /// client.
    pub usb_client_flags: *mut c_uchar,
    /// Private to the framework. A pointer to an anonymous structure that
    /// records the device node of each attached client.
    pub usb_client_attach_list: *mut c_void,
    /// Private to the framework. The device data handed out to clients.
    pub usb_client_dev_data_list: usb_client_dev_data_list_t,
    /// Private to the framework. A pointer to an anonymous structure that
    /// records the event callbacks of each client.
    pub usb_client_ev_cb_list: *mut c_void,
    /// Private to the framework. The shared task queues per endpoint.
    pub usb_shared_taskq: [*mut taskq; USBA_N_ENDPOINTS],
    /// Private to the framework. The reference counts of the shared task
    /// queues.
    pub usb_shared_taskq_ref_count: [c_uchar; USBA_N_ENDPOINTS],
    /// The hub the device is plugged into, or NULL for the root hub. The
    /// header adds that the structure may be shared, as with the `usb_mid`
    /// driver, so it is not strictly the parent device, and that the field
    /// is always set regardless of speed.
    pub usb_parent_hub: *mut usba_device,
    /// Private to the framework. The private data of the host controller
    /// driver, read with [`usba_hcdi_get_device_private`].
    pub usb_hcd_private: *mut c_void,
    /// Private to the framework. The raw Binary Object Store.
    pub usb_bos_mp: *mut mblk_t,
    /// Private to the framework. The number of Binary Object Store entries
    /// allocated.
    pub usb_bos_nalloc: uint_t,
    /// Private to the framework. The number of Binary Object Store entries
    /// in use.
    pub usb_bos_nents: uint_t,
    /// Private to the framework. The parsed Binary Object Store.
    pub usb_bos: *mut usb_bos,
}
/// The typedef name of [`usba_device`].
pub type usba_device_t = usba_device;

// usba_device: usb_device_list 0 to 40; usb_ph_list, 32 entries of 56 bytes,
// 40 to 1832; usb_mutex 1832; usb_dip 1840; usb_hcdi_ops 1848; usb_hubdi
// 1856; usb_addr 1864; usb_no_cpr 1865; usb_root_hub_dip 1872;
// usb_root_hubd 1880; usb_dev_descr 1888; usb_cfg 1896; usb_cfg_length
// 1904; the four strings 1912 to 1936; usb_port_status 1944; usb_port 1946;
// usb_hs_hub_usba_dev 1952; usb_hs_hub_addr 1960; usb_hs_hub_port 1962;
// usb_hs_hub_min_bandwidth 1964; usb_hs_hub_bandwidth 1968 to 2096;
// usb_cfg_array 2096; usb_cfg_array_length 2104; usb_cfg_array_len 2112;
// usb_cfg_array_len_length 2120; usb_cfg_value 2124; usb_active_cfg_ndx
// 2128; usb_cfg_str_descr 2136; usb_n_cfgs 2144; usb_n_ifs 2145;
// usb_pwr_from_hub 2146; usb_ref_count 2148; usb_allocated 2152 to 2192;
// the four cookies 2192 to 2216; usb_evdata 2224; usb_client_flags 2232;
// usb_client_attach_list 2240; usb_client_dev_data_list 2248 to 2288;
// usb_client_ev_cb_list 2288; usb_shared_taskq 2296 to 2552;
// usb_shared_taskq_ref_count 2552 to 2584; usb_parent_hub 2584;
// usb_hcd_private 2592; usb_bos_mp 2600; usb_bos_nalloc 2608; usb_bos_nents
// 2612; usb_bos 2616; total 2624.
const _: () = assert!(core::mem::size_of::<usba_device>() == 2624);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_device_list) == 0);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_ph_list) == 40);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_mutex) == 1832);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_dip) == 1840);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_hubdi) == 1856);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_addr) == 1864);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_root_hub_dip) == 1872);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_dev_descr) == 1888);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_mfg_str) == 1912);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_serialno_str) == 1928);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_port_status) == 1944);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_port) == 1946);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_hs_hub_usba_dev) == 1952);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_hs_hub_bandwidth) == 1968);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_cfg_array) == 2096);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_cfg_array_len) == 2112);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_cfg_str_descr) == 2136);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_ref_count) == 2148);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_allocated) == 2152);
const _: () = assert!(core::mem::offset_of!(usba_device, rm_cookie) == 2192);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_client_dev_data_list) == 2248);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_shared_taskq) == 2296);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_shared_taskq_ref_count) == 2552);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_parent_hub) == 2584);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_hcd_private) == 2592);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_bos_nalloc) == 2608);
const _: () = assert!(core::mem::offset_of!(usba_device, usb_bos) == 2616);

// --- usba_pipe_handle_data(9S) --------------------------------------------------

/// `usba_pipe_handle_data(9S)`. Volatile - illumos USB HCD private.
///
/// The way the framework represents a pipe, `usba_pipe_handle_data_t`.
/// Every pipe belongs to a USB device, and a pipe may be shared between
/// client drivers or exclusive to one. The structure is handed to the host
/// controller driver with every request. A driver first sees a pipe handle
/// in its `usba_hcdi_pipe_open(9E)` entry point, may store its own data in
/// it then, and uses it until `usba_hcdi_pipe_close(9E)` is called. The page
/// lists `p_usba_device`, `p_ep`, `p_xep`, `p_dip`, `p_hcd_private`,
/// `p_mutex` and `p_req_count`. Most of them are read only; the driver may
/// set `p_hcd_private` and must update `p_req_count` under `p_mutex`. The
/// other fields are private to the framework and are declared only so that
/// the layout matches.
#[repr(C)]
pub struct usba_pipe_handle_data {
    /// Private to the framework. The backing structure of the client handle.
    pub p_ph_impl: *mut usba_ph_impl,
    /// Private to the framework. Links the requests queued on the pipe.
    pub p_queue: usba_list_entry_t,
    /// The device this pipe belongs to. It is always set for a pipe handed
    /// to a host controller driver.
    pub p_usba_device: *mut usba_device,
    /// Private to the framework. The pipe policy.
    pub p_policy: usb_pipe_policy_t,
    /// The endpoint descriptor of the pipe, from which the driver learns the
    /// type, address and direction of the endpoint. It is identical to the
    /// endpoint descriptor inside `p_xep`.
    pub p_ep: usb_ep_descr_t,
    /// The endpoint descriptor together with any companion descriptors the
    /// endpoint has. The flags inside tell which companions are present.
    pub p_xep: usb_ep_xdescr_t,
    /// The device node passed when the pipe was opened. Host controller
    /// drivers use it when they duplicate interrupt and isochronous
    /// requests.
    pub p_dip: dev_info_t,
    /// The mutex that protects `p_req_count`. It should be entered after any
    /// locks of the host controller driver and must not be held across
    /// calls into the framework.
    pub p_mutex: kmutex_t,
    /// Reserved for the host controller driver, which may set it in
    /// `usba_hcdi_pipe_open(9E)`, use it for the lifetime of the pipe, and
    /// must clean it up in `usba_hcdi_pipe_close(9E)`.
    pub p_hcd_private: usb_opaque_t,
    /// Private to the framework. The private data of the client.
    pub p_client_private: usb_opaque_t,
    /// The number of outstanding requests on the pipe. A driver that
    /// duplicates a periodic request with [`usba_hcdi_dup_intr_req`] or
    /// [`usba_hcdi_dup_isoc_req`] increments it, and decrements it again if
    /// it drops a duplicate without passing it to the framework. The page
    /// asks drivers to keep it greater than one and to change it only while
    /// holding `p_mutex`.
    pub p_req_count: c_int,
    /// Private to the framework. Its own private data.
    pub p_usba_private: usb_opaque_t,
    /// Private to the framework. The task queue for callbacks and
    /// asynchronous requests on this pipe.
    pub p_taskq: *mut taskq,
    /// Private to the framework. The thread currently serving the queue.
    pub p_thread_id: *mut kthread,
    /// Private to the framework. The queue of callbacks served by the task
    /// queue.
    pub p_cb_queue: usba_list_entry_t,
    /// Private to the framework. The count of soft interrupts.
    pub p_soft_intr: uint_t,
    /// Private to the framework. Special flags of the pipe.
    pub p_spec_flag: uint_t,
}
/// The typedef name of [`usba_pipe_handle_data`].
pub type usba_pipe_handle_data_t = usba_pipe_handle_data;

// usba_pipe_handle_data: p_ph_impl at 0, p_queue 8 to 48, p_usba_device at
// 48, p_policy at 56, p_ep (alignment 2) at 58 to 66, p_xep (alignment 4) at
// 68 to 92, p_dip at 96, p_mutex at 104, p_hcd_private at 112,
// p_client_private at 120, p_req_count at 128, p_usba_private at 136,
// p_taskq at 144, p_thread_id at 152, p_cb_queue 160 to 200, p_soft_intr at
// 200, p_spec_flag at 204, 208 bytes.
const _: () = assert!(core::mem::size_of::<usba_pipe_handle_data>() == 208);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_ph_impl) == 0);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_queue) == 8);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_usba_device) == 48);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_policy) == 56);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_ep) == 58);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_xep) == 68);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_dip) == 96);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_mutex) == 104);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_hcd_private) == 112);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_req_count) == 128);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_usba_private) == 136);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_cb_queue) == 160);
const _: () = assert!(core::mem::offset_of!(usba_pipe_handle_data, p_spec_flag) == 204);

// --- Entry points of the operations vector ---------------------------------------

/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_pm_support` entry point. The page calls
/// the member vestigial; it may be NULL or a function that only returns
/// `USB_FAILURE`.
pub type usba_hcdi_pm_support_t = unsafe extern "C" fn(dip: dev_info_t) -> c_int;

/// `usba_hcdi_pipe_open(9E)`. Volatile - illumos USB HCD private function.
///
/// Opens the pipe `ph`. The framework calls it whenever a client or the
/// framework itself opens a pipe. The driver prepares the endpoint that
/// `p_ep` and `p_xep` describe for transfers, which may involve allocating
/// bandwidth and programming the controller, and may store its own data in
/// `p_hcd_private`. It should check the address of the device first,
/// because the pipe may belong to the root hub, which is usually synthetic.
/// A given endpoint can only be opened once, so a second open must fail.
/// Once the call returns, any transfer or reset entry point may be called
/// on the pipe. `usb_flags` tells whether allocations may block, and
/// `USB_FLAGS_SLEEP` does not guarantee success. Returns `USB_SUCCESS` on
/// success or a USB error, `USB_FAILURE` when in doubt.
pub type usba_hcdi_pipe_open_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, usb_flags: usb_flags_t) -> c_int;
/// `usba_hcdi_pipe_close(9E)`. Volatile - illumos USB HCD private function.
///
/// Closes the pipe `ph`, releasing every resource allocated when it was
/// opened, including the data in `p_hcd_private`. Non-periodic pipes have
/// no outstanding transfers at this point, but periodic ones may, and the
/// driver must stop them as though its stop polling entry point had been
/// called. Afterwards the pipe must be in a state in which it can be opened
/// again. As with open, the pipe may belong to the synthetic root hub.
/// `usb_flags` tells whether allocations may block. Returns `USB_SUCCESS`
/// on success or a USB error, `USB_FAILURE` when in doubt.
pub type usba_hcdi_pipe_close_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, usb_flags: usb_flags_t) -> c_int;
/// `usba_hcdi_pipe_reset(9E)`. Volatile - illumos USB HCD private function.
///
/// Returns the pipe `ph`, whatever its state, to the state it was in right
/// after it was opened. The call is synchronous. The driver quiesces and
/// stops the endpoint, clears any error on it, removes all scheduled or
/// queued transfers and completes each with [`usba_hcdi_cb`] and
/// `USB_CR_PIPE_RESET`, and prepares the endpoint for further I/O. The
/// framework calls it when closing a pipe and in response to a stall.
/// `usb_flags` tells whether allocations may block. Returns `USB_SUCCESS`
/// on success or a USB error, `USB_FAILURE` when in doubt.
pub type usba_hcdi_pipe_reset_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, usb_flags: usb_flags_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_pipe_reset_data_toggle` entry point,
/// which resets the USB data toggle of the pipe `ph` if the controller
/// exposes that to the host.
pub type usba_hcdi_pipe_reset_data_toggle_t = unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t);
/// `usba_hcdi_pipe_ctrl_xfer(9E)`. Volatile - illumos USB HCD private function.
///
/// Starts the asynchronous control transfer `ucrp` on the pipe `ph`.
/// Requests to the root hub have to be emulated from the state of the
/// hardware. Control transfers consist of a setup, an optional data and a
/// status stage. When `ctrl_wLength` is not 0 the data is in the message
/// block `ctrl_data`, and the driver copies it to or from its own buffer,
/// usually DMA memory, allocated according to `usb_flags`. If the driver
/// returns `USB_SUCCESS` it must eventually call [`usba_hcdi_cb`] with the
/// request, before returning only for root hub requests it handles inline.
/// It also times the request out, using `HCDI_DEFAULT_TIMEOUT` when
/// `ctrl_timeout` is 0. On completion it reports `USB_CR_TIMEOUT` for a
/// timeout, the matching error for a failure, `USB_CR_DATA_UNDERRUN` for a
/// short transfer without `USB_ATTRS_SHORT_XFER_OK`, and `USB_CR_OK`
/// otherwise, after copying IN data into the message block and advancing
/// its write pointer. Returns `USB_SUCCESS` when the I/O was scheduled, or
/// a USB error, `USB_FAILURE` when in doubt.
pub type usba_hcdi_pipe_ctrl_xfer_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, ucrp: *mut usb_ctrl_req_t, usb_flags: usb_flags_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_bulk_transfer_size` entry point, which
/// stores in `*size` the largest amount of data a single bulk transfer on
/// the device `ud` can carry. It backs
/// `usb_pipe_get_max_bulk_transfer_size(9F)`.
pub type usba_hcdi_bulk_transfer_size_t = unsafe extern "C" fn(ud: *mut usba_device_t, size: *mut size_t) -> c_int;
/// `usba_hcdi_pipe_bulk_xfer(9E)`. Volatile - illumos USB HCD private function.
///
/// Starts the asynchronous bulk transfer `ubrp` on the pipe `ph`. The root
/// hub has no bulk endpoints, so a request to it returns
/// `USB_NOT_SUPPORTED`. The direction follows from the endpoint address in
/// `p_ep`. The driver allocates its own memory according to `usb_flags` and
/// copies the `bulk_len` bytes of `bulk_data` to or from it, instead of
/// using the message block directly. If it returns `USB_SUCCESS` it must
/// call [`usba_hcdi_cb`] when the I/O completes, with an error if it
/// failed. It also times the request out, using `HCDI_DEFAULT_TIMEOUT` when
/// `bulk_timeout` is 0. The completion codes follow the same rules as for
/// control transfers. Returns `USB_SUCCESS` when the I/O was scheduled, or
/// a USB error, `USB_FAILURE` when in doubt.
pub type usba_hcdi_pipe_bulk_xfer_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, ubrp: *mut usb_bulk_req_t, usb_flags: usb_flags_t) -> c_int;
/// `usba_hcdi_pipe_intr_xfer(9E)`. Volatile - illumos USB HCD private function.
///
/// Starts the asynchronous interrupt transfer `uirp` on the pipe `ph`.
/// Requests to the root hub have to be emulated, and the root hub only
/// supports periodic interrupt IN polling of its status endpoint. With
/// `USB_ATTRS_ONE_XFER` in `intr_attributes` the transfer is a one-shot,
/// which the driver times out, using `HCDI_DEFAULT_TIMEOUT` when
/// `intr_timeout` is 0. Without it the transfer starts periodic polling.
/// The driver then reports each batch of data in a duplicate of the
/// original request made with [`usba_hcdi_dup_intr_req`], often keeping
/// several transfers scheduled to avoid losing data, and returns the
/// original request only on a pipe reset, a request to stop polling, a
/// pipe close, lack of memory, reported as `USB_CR_NO_RESOURCES`, or
/// another error. For interrupt IN transfers `intr_data` may not be
/// allocated, and the driver allocates the message block itself. Memory is
/// allocated according to `usb_flags`. The completion codes follow the
/// rules for control transfers, and periodic transfers are rescheduled
/// after each completion. Returns `USB_SUCCESS` when the I/O was
/// scheduled, or a USB error, `USB_FAILURE` when in doubt.
pub type usba_hcdi_pipe_intr_xfer_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, uirp: *mut usb_intr_req_t, usb_flags: usb_flags_t) -> c_int;
/// `usba_hcdi_pipe_stop_intr_polling(9E)`. Volatile - illumos USB HCD private function.
///
/// Stops periodic interrupt polling on the pipe `ph`, which may belong to
/// the root hub, whose synthetic polling then stops while its changes keep
/// being tracked. The call is synchronous. The driver quiesces and stops
/// the endpoint, removes the remaining scheduled transfers, calls
/// [`usba_hcdi_cb`] on the original request with `USB_CR_STOPPED_POLLING`,
/// frees the associated resources now or at the latest when the pipe is
/// closed, and leaves the endpoint ready for new transfers. As part of
/// hardening it should verify that transfers are outstanding, and it has to
/// synchronise with a concurrent pipe reset. `usb_flags` tells whether
/// allocations may block. Returns `USB_SUCCESS` on success or a USB error,
/// `USB_FAILURE` when in doubt.
pub type usba_hcdi_pipe_stop_intr_polling_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, usb_flags: usb_flags_t) -> c_int;
/// `usba_hcdi_pipe_isoc_xfer(9E)`. Volatile - illumos USB HCD private function.
///
/// Starts the asynchronous isochronous transfer `usrp` on the pipe `ph`.
/// The root hub has no isochronous endpoints, so a request to it returns
/// `USB_NOT_SUPPORTED`. A new driver may require
/// `USB_ATTRS_ISOC_XFER_ASAP` and return `USB_NOT_SUPPORTED` for a fixed
/// start frame it cannot honour. The data in the single message block is
/// split into the packets described by `isoc_pkt_descr`. For one-shot OUT
/// transfers the sum of the packet lengths must match the length of the
/// message block, or the driver returns `USB_INVALID_ARGS`, and the driver
/// times the transfer out with `HCDI_DEFAULT_TIMEOUT`. IN transfers are
/// always periodic, and the driver reports each batch of data in a
/// duplicate made with [`usba_hcdi_dup_isoc_req`], returning the original
/// under the same conditions as for interrupt polling. Memory is allocated
/// according to `usb_flags`. On completion the driver also fills in
/// `isoc_pkt_actual_length` of each packet. Returns `USB_SUCCESS` when the
/// I/O was scheduled, or a USB error, `USB_FAILURE` when in doubt.
pub type usba_hcdi_pipe_isoc_xfer_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, usrp: *mut usb_isoc_req_t, usb_flags: usb_flags_t) -> c_int;
/// `usba_hcdi_pipe_stop_isoc_polling(9E)`. Volatile - illumos USB HCD private function.
///
/// Stops periodic isochronous polling on the pipe `ph`, following the same
/// steps as the interrupt variant and returning the original request with
/// `USB_CR_STOPPED_POLLING`. Isochronous transfers are forbidden on the
/// root hub, so the pipe always belongs to a real device. `usb_flags` tells
/// whether allocations may block. Returns `USB_SUCCESS` on success or a USB
/// error, `USB_FAILURE` when in doubt.
pub type usba_hcdi_pipe_stop_isoc_polling_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, usb_flags: usb_flags_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_get_current_frame_number` entry point,
/// which stores the current virtual frame number of the bus of `ud` in
/// `*frame_number`. The entry point is required, but nothing in illumos
/// uses it, so a function that only returns `USB_FAILURE` is acceptable.
pub type usba_hcdi_get_current_frame_number_t =
    unsafe extern "C" fn(ud: *mut usba_device_t, frame_number: *mut usb_frame_number_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_get_max_isoc_pkts` entry point, which
/// stores in `*max_isoc_pkts_per_request` the largest number of
/// isochronous packets that can be queued in one transfer.
pub type usba_hcdi_get_max_isoc_pkts_t =
    unsafe extern "C" fn(ud: *mut usba_device_t, max_isoc_pkts_per_request: *mut uint_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_input_init` entry point, which
/// prepares polled console input from the pipe `ph`, returning the buffer
/// in `*obp_buf`. Console support is optional, and a driver without it
/// installs a function that returns `USB_FAILURE`.
pub type usba_hcdi_console_input_init_t = unsafe extern "C" fn(
    ph: *mut usba_pipe_handle_data_t,
    obp_buf: *mut *mut c_uchar,
    console_input_info: *mut usb_console_info_impl_t,
) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_input_fini` entry point, which
/// frees the resources that the input initialisation allocated.
pub type usba_hcdi_console_input_fini_t =
    unsafe extern "C" fn(console_input_info: *mut usb_console_info_impl_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_input_enter` entry point, which
/// saves the state of the controller before polled input begins.
pub type usba_hcdi_console_input_enter_t =
    unsafe extern "C" fn(console_input_info: *mut usb_console_info_impl_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_read` entry point, which reads
/// characters from the controller and stores their number in
/// `*num_characters`. The page shows the second argument without the
/// pointer; the header declares a `uint_t` pointer.
pub type usba_hcdi_console_read_t =
    unsafe extern "C" fn(console_input_info: *mut usb_console_info_impl_t, num_characters: *mut uint_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_input_exit` entry point, which
/// restores the state of the controller after polled input.
pub type usba_hcdi_console_input_exit_t =
    unsafe extern "C" fn(console_input_info: *mut usb_console_info_impl_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_output_init` entry point, which
/// prepares polled console output to the pipe `ph`. It was added in version
/// 1 of the operations vector.
pub type usba_hcdi_console_output_init_t =
    unsafe extern "C" fn(ph: *mut usba_pipe_handle_data_t, console_output_info: *mut usb_console_info_impl_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_output_fini` entry point, which
/// frees the resources that the output initialisation allocated.
pub type usba_hcdi_console_output_fini_t =
    unsafe extern "C" fn(console_output_info: *mut usb_console_info_impl_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_output_enter` entry point, which
/// saves the state of the controller before polled output begins.
pub type usba_hcdi_console_output_enter_t =
    unsafe extern "C" fn(console_output_info: *mut usb_console_info_impl_t) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_write` entry point, which writes
/// `num_characters` bytes from `buf` and stores the number actually written
/// in `*num_characters_written`.
pub type usba_hcdi_console_write_t = unsafe extern "C" fn(
    console_output_info: *mut usb_console_info_impl_t,
    buf: *mut c_uchar,
    num_characters: uint_t,
    num_characters_written: *mut uint_t,
) -> c_int;
/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The signature of the `usba_hcdi_console_output_exit` entry point, which
/// restores the state of the controller after polled output.
pub type usba_hcdi_console_output_exit_t =
    unsafe extern "C" fn(console_output_info: *mut usb_console_info_impl_t) -> c_int;
/// `usba_hcdi_device_init(9E)`. Volatile - illumos USB HCD private function.
///
/// An optional entry point, added in version 2 of the operations vector,
/// that the framework calls when a child device of the root hub is being
/// initialised, before any pipe to it is opened. `usb_device` is the
/// device and `port` the port it was detected on. The driver performs the
/// initialisation the controller requires and may store private data for
/// the device in `*hcd_privatep`, which [`usba_hcdi_get_device_private`]
/// returns later. No other entry point is called for the same device while
/// this one runs, although it may run in parallel for different devices.
/// It is called from kernel context only. Returns `USB_SUCCESS` on success
/// or the appropriate error. Drivers that do not need it leave the member
/// NULL.
pub type usba_hcdi_device_init_t =
    unsafe extern "C" fn(usb_device: *mut usba_device_t, port: usb_port_t, hcd_privatep: *mut *mut c_void) -> c_int;
/// `usba_hcdi_device_fini(9E)`. Volatile - illumos USB HCD private function.
///
/// An optional entry point that the framework calls when a child device of
/// the root hub is removed, by unplugging or through `cfgadm(8)`. All pipes
/// of the device have been closed by then. The driver finishes tearing the
/// device down in the controller and must release the private data
/// `hcd_private` it stored during initialisation, or it leaks. The manual
/// page shows a port argument between the two, but the header, which this
/// binding follows, has none. It is called from kernel context only, and
/// drivers that do not need it leave the member NULL.
pub type usba_hcdi_device_fini_t = unsafe extern "C" fn(usb_device: *mut usba_device_t, hcd_private: *mut c_void);
/// `usba_hcdi_device_address(9E)`. Volatile - illumos USB HCD private function.
///
/// An optional entry point for controllers that do not allow the USB
/// `SET_ADDRESS` request and assign addresses in their own way. The
/// framework calls it after `usba_hcdi_device_init(9E)`, when the device
/// `ud` needs an address. The framework still assigns its own address in
/// `usb_addr`, so a driver that needs the real one keeps track of it
/// separately. On return the device must be addressed and ready for its
/// endpoints to be opened, and a failure halts its enumeration. It is
/// called from kernel context only. Returns `USB_SUCCESS` on success or a
/// USB error, `USB_FAILURE` when in doubt. Drivers that do not need it
/// leave the member NULL.
pub type usba_hcdi_device_address_t = unsafe extern "C" fn(ud: *mut usba_device_t) -> c_int;
/// `usba_hcdi_hub_update(9E)`. Volatile - illumos USB HCD private function.
///
/// An optional entry point that the framework calls when enumeration finds
/// that the device `ud` is a hub, after `usba_hcdi_device_init(9E)`.
/// `nports` is the number of ports of the hub and `think_time` the Think
/// Time value of its hub descriptor. The driver programs the controller
/// synchronously before returning. A failure fails the enumeration of the
/// hub and hides every device plugged into it. It is called from kernel
/// context only. Returns `USB_SUCCESS` on success or a USB error,
/// `USB_FAILURE` when in doubt. Drivers that do not need it leave the
/// member NULL.
pub type usba_hcdi_hub_update_t = unsafe extern "C" fn(ud: *mut usba_device_t, nports: u8, think_time: u8) -> c_int;

// --- usba_hcdi_ops(9S) ----------------------------------------------------------

/// `usba_hcdi_ops(9S)`. Volatile - illumos USB HCD private.
///
/// The operations vector of a host controller driver, `usba_hcdi_ops_t`.
/// All client functions that need the host controller go through it. The
/// framework allocates it with [`usba_alloc_hcdi_ops`] so that it can
/// handle versioning, the driver fills it in and passes it to
/// [`usba_hcdi_register`], and when registration fails or the driver
/// unregisters it releases it with [`usba_free_hcdi_ops`]. See the
/// `usba_hcdi(9E)` page for the rules every entry point follows.
#[repr(C)]
pub struct usba_hcdi_ops {
    /// The revision of the vector, generally [`HCDI_OPS_VERSION`].
    pub usba_hcdi_ops_version: c_int,
    /// The device node the driver received in `attach(9E)`.
    pub usba_hcdi_dip: dev_info_t,
    /// A vestigial entry point that may be NULL or return `USB_FAILURE`.
    pub usba_hcdi_pm_support: Option<usba_hcdi_pm_support_t>,
    /// Opens a pipe. See `usba_hcdi_pipe_open(9E)`.
    pub usba_hcdi_pipe_open: Option<usba_hcdi_pipe_open_t>,
    /// Closes a pipe and cleans up any outstanding polling. See
    /// `usba_hcdi_pipe_close(9E)`.
    pub usba_hcdi_pipe_close: Option<usba_hcdi_pipe_close_t>,
    /// Resets a pipe, cleaning up outstanding requests and stopping
    /// polling. See `usba_hcdi_pipe_reset(9E)`.
    pub usba_hcdi_pipe_reset: Option<usba_hcdi_pipe_reset_t>,
    /// Resets the data toggle of a pipe if the controller exposes it.
    pub usba_hcdi_pipe_reset_data_toggle: Option<usba_hcdi_pipe_reset_data_toggle_t>,
    /// Performs control transfers for any device, including the root hub.
    /// See `usba_hcdi_pipe_ctrl_xfer(9E)`.
    pub usba_hcdi_pipe_ctrl_xfer: Option<usba_hcdi_pipe_ctrl_xfer_t>,
    /// Returns the largest amount of data of one bulk transfer.
    pub usba_hcdi_bulk_transfer_size: Option<usba_hcdi_bulk_transfer_size_t>,
    /// Performs bulk transfers. See `usba_hcdi_pipe_bulk_xfer(9E)`.
    pub usba_hcdi_pipe_bulk_xfer: Option<usba_hcdi_pipe_bulk_xfer_t>,
    /// Performs periodic and one-shot interrupt transfers, including those
    /// for the root hub. See `usba_hcdi_pipe_intr_xfer(9E)`.
    pub usba_hcdi_pipe_intr_xfer: Option<usba_hcdi_pipe_intr_xfer_t>,
    /// Stops interrupt polling and returns the original request. See
    /// `usba_hcdi_pipe_stop_intr_polling(9E)`.
    pub usba_hcdi_pipe_stop_intr_polling: Option<usba_hcdi_pipe_stop_intr_polling_t>,
    /// Performs periodic and one-shot isochronous transfers. See
    /// `usba_hcdi_pipe_isoc_xfer(9E)`.
    pub usba_hcdi_pipe_isoc_xfer: Option<usba_hcdi_pipe_isoc_xfer_t>,
    /// Stops isochronous polling and returns the original request. See
    /// `usba_hcdi_pipe_stop_isoc_polling(9E)`.
    pub usba_hcdi_pipe_stop_isoc_polling: Option<usba_hcdi_pipe_stop_isoc_polling_t>,
    /// Returns the current virtual frame number. It is required, but a
    /// function that returns `USB_FAILURE` is acceptable.
    pub usba_hcdi_get_current_frame_number: Option<usba_hcdi_get_current_frame_number_t>,
    /// Returns the largest number of isochronous packets per transfer.
    pub usba_hcdi_get_max_isoc_pkts: Option<usba_hcdi_get_max_isoc_pkts_t>,
    /// Prepares polled console input. Optional support; without it the
    /// function returns `USB_FAILURE`, as do the other console members.
    pub usba_hcdi_console_input_init: Option<usba_hcdi_console_input_init_t>,
    /// Frees the resources of polled console input.
    pub usba_hcdi_console_input_fini: Option<usba_hcdi_console_input_fini_t>,
    /// Saves the controller state before polled input.
    pub usba_hcdi_console_input_enter: Option<usba_hcdi_console_input_enter_t>,
    /// Reads characters for the console.
    pub usba_hcdi_console_read: Option<usba_hcdi_console_read_t>,
    /// Restores the controller state after polled input.
    pub usba_hcdi_console_input_exit: Option<usba_hcdi_console_input_exit_t>,
    /// Prepares polled console output, a version 1 addition.
    pub usba_hcdi_console_output_init: Option<usba_hcdi_console_output_init_t>,
    /// Frees the resources of polled console output.
    pub usba_hcdi_console_output_fini: Option<usba_hcdi_console_output_fini_t>,
    /// Saves the controller state before polled output.
    pub usba_hcdi_console_output_enter: Option<usba_hcdi_console_output_enter_t>,
    /// Writes characters for the console.
    pub usba_hcdi_console_write: Option<usba_hcdi_console_write_t>,
    /// Restores the controller state after polled output.
    pub usba_hcdi_console_output_exit: Option<usba_hcdi_console_output_exit_t>,
    /// Initialises a new device, a version 2 addition. Optional, NULL when
    /// not needed. See `usba_hcdi_device_init(9E)`.
    pub usba_hcdi_device_init: Option<usba_hcdi_device_init_t>,
    /// Tears down a removed device. Optional, NULL when not needed. See
    /// `usba_hcdi_device_fini(9E)`.
    pub usba_hcdi_device_fini: Option<usba_hcdi_device_fini_t>,
    /// Addresses a device. Optional, NULL when not needed. See
    /// `usba_hcdi_device_address(9E)`.
    pub usba_hcdi_device_address: Option<usba_hcdi_device_address_t>,
    /// Updates the controller after a hub was found. Optional, NULL when not
    /// needed. See `usba_hcdi_hub_update(9E)`.
    pub usba_hcdi_hub_update: Option<usba_hcdi_hub_update_t>,
}
/// The typedef name of [`usba_hcdi_ops`].
pub type usba_hcdi_ops_t = usba_hcdi_ops;

// usba_hcdi_ops: usba_hcdi_ops_version at 0 padded to 8, usba_hcdi_dip at
// 8, then 28 function pointers from 16 to 232, 240 bytes.
const _: () = assert!(core::mem::size_of::<usba_hcdi_ops>() == 16 + 28 * 8);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_ops, usba_hcdi_ops_version) == 0);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_ops, usba_hcdi_dip) == 8);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_ops, usba_hcdi_pm_support) == 16);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_ops, usba_hcdi_pipe_ctrl_xfer) == 56);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_ops, usba_hcdi_console_input_init) == 128);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_ops, usba_hcdi_console_output_init) == 168);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_ops, usba_hcdi_device_init) == 208);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_ops, usba_hcdi_hub_update) == 232);

// --- usba_hcdi_register_args(9S) -------------------------------------------------

/// `usba_hcdi_register_args(9S)`. Volatile - illumos USB HCD private.
///
/// The arguments with which a host controller driver registers with the
/// framework through [`usba_hcdi_register`], `usba_hcdi_register_args_t`.
/// The structure may live on the stack, because it is not used after the
/// call returns.
#[repr(C)]
pub struct usba_hcdi_register_args {
    /// The version of the structure, [`HCDI_REGISTER_VERSION`]. The page
    /// warns that because the structure is Volatile the version does not
    /// guarantee that older versions stay supported.
    pub usba_hcdi_register_version: uint_t,
    /// The device node of the driver instance that registers.
    pub usba_hcdi_register_dip: dev_info_t,
    /// The operations vector, allocated with [`usba_alloc_hcdi_ops`] and
    /// released with [`usba_free_hcdi_ops`] after registration.
    pub usba_hcdi_register_ops: *mut usba_hcdi_ops_t,
    /// DMA attributes that some client drivers, in particular `scsa2usb`,
    /// use for their allocations, mostly for bulk transfers. They should be
    /// valid for as many transfers as possible.
    pub usba_hcdi_register_dma_attr: *mut ddi_dma_attr_t,
    /// The general interrupt priority of the driver after it has allocated
    /// its interrupts, obtained with `ddi_intr_get_pri(9F)` and cast to an
    /// interrupt block cookie.
    pub usba_hcdi_register_iblock_cookie: ddi_iblock_cookie_t,
}
/// The typedef name of [`usba_hcdi_register_args`].
pub type usba_hcdi_register_args_t = usba_hcdi_register_args;

// usba_hcdi_register_args: the version at 0 padded to 8, then four pointers
// at 8, 16, 24 and 32, 40 bytes.
const _: () = assert!(core::mem::size_of::<usba_hcdi_register_args>() == 40);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_register_args, usba_hcdi_register_version) == 0);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_register_args, usba_hcdi_register_dip) == 8);
const _: () = assert!(core::mem::offset_of!(usba_hcdi_register_args, usba_hcdi_register_iblock_cookie) == 32);

// --- Functions ------------------------------------------------------------------

unsafe extern "C" {
    /// `usba_hubdi_dev_ops(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// The bus operations that every host controller driver must place in
    /// the `devo_bus_ops` member of its `dev_ops(9S)`, `usba_hubdi_busops`.
    /// The manual page lists it as `usba_hubdi_bus_ops`, but the header and
    /// the kernel spell it without the second underscore.
    pub static usba_hubdi_busops: bus_ops;

    /// `usba_alloc_hcdi_ops(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Allocates an operations vector for a host controller driver, which
    /// then fills it in. The function always succeeds and returns a pointer
    /// to the structure. It is generally called from kernel context in
    /// `attach(9E)`; calling it from user context is safe but unusual.
    pub fn usba_alloc_hcdi_ops() -> *mut usba_hcdi_ops_t;
    /// `usba_free_hcdi_ops(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Releases the operations vector `ops`, which must have come from
    /// [`usba_alloc_hcdi_ops`]. It is generally called from kernel context
    /// in `attach(9E)` or `detach(9E)`.
    pub fn usba_free_hcdi_ops(ops: *mut usba_hcdi_ops_t);
    /// `usba_hcdi_register(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Registers a host controller driver instance with the framework, using
    /// the filled in arguments `args`. `flags` should be 0. The driver calls
    /// it in `attach(9E)` after it has finished initialising the device, and
    /// must assume that any entry point of its operations vector may be
    /// called as soon as it returns successfully. Returns `USB_SUCCESS` on
    /// success and `USB_FAILURE` when the device was not registered, in
    /// which case `attach(9E)` should fail. It should only be called from
    /// `attach(9E)`.
    pub fn usba_hcdi_register(args: *mut usba_hcdi_register_args_t, flags: uint_t) -> c_int;
    /// `usba_hcdi_unregister(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Unregisters the host controller driver instance of `dip`. The driver
    /// calls it in `detach(9E)` after unbinding its root hub and before
    /// tearing down the rest of its state. Afterwards no entry point of its
    /// operations vector is called any more. The function always succeeds
    /// and should only be called from `detach(9E)`.
    pub fn usba_hcdi_unregister(dip: dev_info_t);
    /// `usba_hcdi_cb(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Hands a completed transfer request back to the framework, whether it
    /// succeeded or not. `ph` is the pipe handle the driver received when
    /// the pipe was opened, `req` is the control, bulk, interrupt or
    /// isochronous request cast to `usb_opaque_t`, with all relevant members
    /// such as the data and the packet descriptors filled in, and
    /// `completion_reason` is `USB_CR_OK` for success, including a permitted
    /// short transfer, or the reason for the failure. A periodic transfer
    /// may lead to several calls. The function must not be used for a
    /// transfer whose start entry point did not return `USB_SUCCESS`, the
    /// request must be treated as freed afterwards, and the driver must not
    /// hold its own locks across the call, since it may lead to another
    /// entry point being called. It may be called from user, kernel or
    /// interrupt context.
    pub fn usba_hcdi_cb(ph: *mut usba_pipe_handle_data_t, req: usb_opaque_t, completion_reason: usb_cr_t);
    /// `usba_hcdi_dup_intr_req(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Duplicates the original interrupt request `irqp` of a periodic
    /// transfer, so that the driver can return a batch of data through
    /// [`usba_hcdi_cb`] while keeping the original. `dip` is the device node
    /// of the host controller driver and `size` the number of bytes of the
    /// data block to allocate, which depends on the endpoint. `flags` is
    /// `USB_FLAGS_NOSLEEP` or `USB_FLAGS_SLEEP`, and even a sleeping
    /// allocation may fail. The caller may also have to increment
    /// `p_req_count` of the pipe. Returns the duplicate or NULL. The
    /// function may be called from user, kernel or interrupt context, but
    /// only with `USB_FLAGS_NOSLEEP` in interrupt context.
    pub fn usba_hcdi_dup_intr_req(
        dip: dev_info_t,
        irqp: *mut usb_intr_req_t,
        size: size_t,
        flags: usb_flags_t,
    ) -> *mut usb_intr_req_t;
    /// `usba_hcdi_dup_isoc_req(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Duplicates the original isochronous request `usrp` of a periodic IN
    /// transfer, so that the driver can copy available data into the
    /// `isoc_data` of the duplicate and return it through [`usba_hcdi_cb`].
    /// `dip` is the device node of the host controller driver. `flags` is
    /// `USB_FLAGS_NOSLEEP` or `USB_FLAGS_SLEEP` as the context allows, and
    /// even a sleeping allocation may fail. The caller has to increment
    /// `p_req_count` of the pipe. Returns the duplicate or NULL. The
    /// function may be called from user, kernel or interrupt context, but
    /// only with `USB_FLAGS_NOSLEEP` in interrupt context.
    pub fn usba_hcdi_dup_isoc_req(
        dip: dev_info_t,
        usrp: *mut usb_isoc_req_t,
        flags: usb_flags_t,
    ) -> *mut usb_isoc_req_t;
    /// `usba_hcdi_get_device_private(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Returns the private data that the driver stored for `usb_device` in
    /// its optional `usba_hcdi_device_init(9E)` entry point, or NULL if it
    /// stored none or does not implement that entry point. The framework
    /// does not lock the data. The function is generally called from an
    /// entry point of the driver but may be called from user, kernel or
    /// interrupt context.
    pub fn usba_hcdi_get_device_private(usb_device: *mut usba_device_t) -> *mut c_void;

    /// `usba_hubdi_open(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// The part of the `open(9E)` entry point of a host controller driver
    /// that the framework implements. The driver finds the device node for
    /// the device number in `devp` and passes it together with its own
    /// unmodified arguments, returning the result. The driver must not clone
    /// or renumber minors, because the framework uses part of the minor
    /// space. It should only be called from the `open(9E)` entry point.
    pub fn usba_hubdi_open(dip: dev_info_t, devp: *mut dev_t, flag: c_int, otyp: c_int, credp: *mut cred_t) -> c_int;
    /// `usba_hubdi_close(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// The part of the `close(9E)` entry point of a host controller driver
    /// that the framework implements. The driver passes the device node for
    /// `dev` and its own unmodified arguments and returns the result. It
    /// should only be called from the `close(9E)` entry point.
    pub fn usba_hubdi_close(dip: dev_info_t, dev: dev_t, flag: c_int, otyp: c_int, credp: *mut cred_t) -> c_int;
    /// `usba_hubdi_ioctl(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// The part of the `ioctl(9E)` entry point of a host controller driver
    /// that the framework implements. The driver passes the device node for
    /// `dev` and its own unmodified arguments and returns the result. A
    /// driver with private ioctls may check the command first, but must then
    /// verify the privileges of the caller itself, which this function
    /// normally does. It should only be called from the `ioctl(9E)` entry
    /// point.
    pub fn usba_hubdi_ioctl(
        dip: dev_info_t,
        dev: dev_t,
        cmd: c_int,
        arg: intptr_t,
        mode: c_int,
        credp: *mut cred_t,
        rvalp: *mut c_int,
    ) -> c_int;
    /// `usba_hubdi_root_hub_power(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// The `power(9E)` implementation for host controller drivers. A driver
    /// that supports power management places it in the `devo_power` member
    /// of its `dev_ops(9S)` instead of implementing its own. The arguments
    /// are those of `power(9E)`. It should not be called directly.
    pub fn usba_hubdi_root_hub_power(dip: dev_info_t, component: c_int, level: c_int) -> c_int;
    /// `usba_hubdi_bind_root_hub(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Binds the root hub of the host controller driver of `dip`, which
    /// attaches an instance of the hub driver to it. The driver calls it in
    /// `attach(9E)` after [`usba_hcdi_register`], once all initialisation is
    /// done, and must expect calls to its operations vector before it
    /// returns. `hub_descr` points to `descr_len` bytes of packed
    /// descriptors as a hub of the matching USB generation would return
    /// them, a configuration descriptor, an interface descriptor, the
    /// interrupt IN endpoint descriptor and any companion descriptors, and
    /// not the padded structures of this crate. `dev_descr` is a filled in
    /// device descriptor for such a hub. Returns `USB_SUCCESS` on success
    /// and `USB_FAILURE` otherwise, in which case the driver unregisters and
    /// fails attach. It should only be called from `attach(9E)`.
    pub fn usba_hubdi_bind_root_hub(
        dip: dev_info_t,
        hub_descr: *mut c_uchar,
        descr_len: size_t,
        dev_descr: *mut usb_dev_descr_t,
    ) -> c_int;
    /// `usba_hubdi_unbind_root_hub(9F)`. Volatile - illumos USB HCD private function.
    ///
    /// Detaches the root hub of the host controller driver of `dip`. The
    /// driver calls it first thing in `detach(9E)`, before
    /// [`usba_hcdi_unregister`], and fails the detach if it does not return
    /// `USB_SUCCESS`. Returns `USB_SUCCESS` on success and `USB_FAILURE`
    /// otherwise. It should only be called from `detach(9E)`.
    pub fn usba_hubdi_unbind_root_hub(dip: dev_info_t) -> c_int;
}
