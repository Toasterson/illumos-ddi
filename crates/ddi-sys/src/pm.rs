//! Device power management, the `pm_*(9F)` family from `sys/sunpm.h`.
//!
//! A power-manageable device is made of components, each of which the
//! framework can move between the power levels the driver declares. The
//! driver tells the framework about its components through the
//! `pm-components(9P)` property, a string array it creates on its own node
//! in `attach(9E)` and which the framework reads once the device has
//! attached. The array starts each component with a string of the form
//! `NAME=component name` and follows it with one string per power level of
//! the form `numeric level=level name`, for example `NAME=Spindle Motor`,
//! `0=Stopped`, `1=Full Speed`. Components are numbered from 0 in the order
//! listed, which must match the numbering the driver's `power(9E)` routine
//! uses; levels are listed in increasing order of power consumption; every
//! component needs at least two levels; and a level of 0, which means off,
//! must come first when it is present. Changes to the property after the
//! framework has read it are not recognised.
//!
//! At run time the driver marks a component busy with
//! `pm_busy_component(9F)` while it is in use and idle again with
//! `pm_idle_component(9F)`, raises a component's level before touching the
//! hardware with `pm_raise_power(9F)`, lowers every component on the way out
//! of `detach(9E)` with `pm_lower_power(9F)`, and reports level changes the
//! device made on its own with `pm_power_has_changed(9F)`. Drivers for
//! devices that wear with each power cycle consult `pm_trans_check(9F)`
//! from `power(9E)` before powering off.

use crate::sunddi::time_t;
use crate::types::*;

/// The power level of a component is not known (`PM_LEVEL_UNKNOWN`). This is
/// the level the framework assumes for every component until the driver
/// reports otherwise.
pub const PM_LEVEL_UNKNOWN: c_int = -1;
/// The `format` value of a [`pm_trans_data`] that carries SCSI log page data
/// in its `scsi_cycles` member (`DC_SCSI_FORMAT`).
pub const DC_SCSI_FORMAT: c_int = 0x1;
/// The `format` value of a [`pm_trans_data`] that carries SMART attribute
/// data in its `smart_count` member (`DC_SMART_FORMAT`).
pub const DC_SMART_FORMAT: c_int = 0x2;
/// The length of the `svc_date` field of [`pm_scsi_cycles`], which holds a
/// service date in the form `YYYYWW` (`DC_SCSI_MFR_LEN`).
pub const DC_SCSI_MFR_LEN: usize = 6;

/// Power cycle data taken from a SCSI log page, the `scsi_cycles` member of
/// [`pm_trans_data`]. The manual page for `pm_trans_check(9F)` documents it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct pm_scsi_cycles {
    /// The maximum number of power cycles over the lifetime of the device.
    pub lifemax: c_int,
    /// The number of power cycles so far.
    pub ncycles: c_int,
    /// The service date of the device as `YYYYWW`, not NUL terminated.
    pub svc_date: [c_char; DC_SCSI_MFR_LEN],
    /// Reserved for future use and must be 0.
    pub flag: c_int,
}

/// Power cycle data taken from a SMART attribute, the `smart_count` member
/// of [`pm_trans_data`]. The manual page for `pm_trans_check(9F)` documents
/// it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct pm_smart_count {
    /// The normalised number of cycles allowed before the borderline
    /// threshold is reached.
    pub allowed: c_int,
    /// The normalised number of cycles consumed so far. The manual page
    /// calls this field `usedup`; the header names it `consumed`.
    pub consumed: c_int,
    /// Reserved for future use and must be 0.
    pub flag: c_int,
}

/// The anonymous union inside [`pm_trans_data`], holding whichever kind of
/// power cycle data the `format` field announces.
#[repr(C)]
#[derive(Clone, Copy)]
pub union pm_trans_data_un {
    /// SCSI log page data, valid when `format` is [`DC_SCSI_FORMAT`].
    pub scsi_cycles: pm_scsi_cycles,
    /// SMART attribute data, valid when `format` is [`DC_SMART_FORMAT`].
    pub smart_count: pm_smart_count,
}

/// `pm_trans_data`. illumos DDI specific (illumos DDI).
///
/// The input to `pm_trans_check(9F)`. The `format` field selects which
/// member of the union carries data: [`DC_SCSI_FORMAT`] for values taken
/// from a SCSI log page, or [`DC_SMART_FORMAT`] for values taken from a SMART
/// attribute.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct pm_trans_data {
    /// The data format, [`DC_SCSI_FORMAT`] or [`DC_SMART_FORMAT`].
    pub format: c_int,
    /// The power cycle data in the announced format.
    pub un: pm_trans_data_un,
}

// pm_scsi_cycles is two ints, six chars padded to 8, and an int: 20 bytes.
// pm_smart_count is three ints: 12 bytes. The union takes the larger, 20
// bytes with 4-byte alignment, and pm_trans_data prefixes it with an int.
const _: () = assert!(core::mem::size_of::<pm_scsi_cycles>() == 20);
const _: () = assert!(core::mem::offset_of!(pm_scsi_cycles, lifemax) == 0);
const _: () = assert!(core::mem::offset_of!(pm_scsi_cycles, svc_date) == 8);
const _: () = assert!(core::mem::offset_of!(pm_scsi_cycles, flag) == 16);
const _: () = assert!(core::mem::size_of::<pm_smart_count>() == 12);
const _: () = assert!(core::mem::offset_of!(pm_smart_count, allowed) == 0);
const _: () = assert!(core::mem::offset_of!(pm_smart_count, flag) == 8);
const _: () = assert!(core::mem::size_of::<pm_trans_data_un>() == 20);
const _: () = assert!(core::mem::size_of::<pm_trans_data>() == 24);
const _: () = assert!(core::mem::offset_of!(pm_trans_data, format) == 0);
const _: () = assert!(core::mem::offset_of!(pm_trans_data, un) == 4);

unsafe extern "C" {
    /// `pm_busy_component(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Marks component `component_number` of the device `dip` busy, so that
    /// the framework will not power-manage it. Calls stack, and the same
    /// number of [`pm_idle_component`] calls is needed to make the component
    /// idle again. When called as a result of entry into `attach(9E)`,
    /// `detach(9E)` or `power(9E)`, it must be called from the thread that
    /// entered that routine. Returns `DDI_SUCCESS`, or `DDI_FAILURE` when the
    /// component number is invalid or the device has no components. May be
    /// called from user or kernel context, and from interrupt context
    /// provided it is not the first power management function the driver
    /// calls.
    pub fn pm_busy_component(dip: dev_info_t, component_number: c_int) -> c_int;
    /// `pm_idle_component(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Marks component `component_number` of the device `dip` idle and
    /// records the time it went idle, undoing one earlier
    /// [`pm_busy_component`] call. An idle component is available to be
    /// power-managed. When the component is already idle the call only
    /// updates the framework's notion of when it went idle. The same thread
    /// rule as for `pm_busy_component` applies. Returns `DDI_SUCCESS`, or
    /// `DDI_FAILURE` when the component number is invalid or the device has
    /// no components. May be called from user or kernel context, and from
    /// interrupt context provided it is not the first power management
    /// function the driver calls.
    pub fn pm_idle_component(dip: dev_info_t, component_number: c_int) -> c_int;
    /// `pm_raise_power(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the framework to raise component `comp` of the device `dip` to
    /// at least power level `level`. A driver calls it before an operation
    /// that needs the component at a higher level than its current one. On
    /// success the component is at least at the requested level and every
    /// device that depends on it is at full power, though the actual level
    /// may be higher than requested. The call may re-enter the driver's
    /// `power(9E)`, so holding driver locks across it can deadlock. When
    /// called as a result of entry into `attach(9E)`, `detach(9E)` or
    /// `power(9E)`, it must be called from the thread that entered that
    /// routine. Returns `DDI_SUCCESS`, or `DDI_FAILURE` when the component or
    /// level is out of range or the framework could not raise the level. May
    /// be called from user or kernel context.
    pub fn pm_raise_power(dip: dev_info_t, comp: c_int, level: c_int) -> c_int;
    /// `pm_lower_power(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Asks the framework to lower component `comp` of the device `dip` to
    /// at most power level `level`. Transitions to lower levels are normally
    /// initiated by the framework when a component is idle; this function is
    /// intended only for a detaching driver that wants every component at
    /// its lowest level, and it fails when the driver is not detaching. When
    /// automatic power management is disabled it succeeds without changing
    /// anything. On success the component is at most at the requested level,
    /// though the actual level may be lower. The call may re-enter the
    /// driver's `power(9E)`, so holding driver locks across it can deadlock,
    /// and the same thread rule as for [`pm_raise_power`] applies. Returns
    /// `DDI_SUCCESS`, or `DDI_FAILURE` when the component or level is out of
    /// range, the framework could not lower the level, or the device is not
    /// detaching. May be called from user or kernel context.
    pub fn pm_lower_power(dip: dev_info_t, comp: c_int, level: c_int) -> c_int;
    /// `pm_power_has_changed(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Tells the framework that component `component` of the device `dip`
    /// changed to power level `level` on its own, without a `power(9E)` call.
    /// Because the framework may have called `power(9E)` in the meantime,
    /// the driver should verify that the device really is at that level, or
    /// set it there when the hardware cannot be queried, before calling, and
    /// should prevent `power(9E)` from running in parallel with the call.
    /// When called as a result of entry into `attach(9E)`, `detach(9E)` or
    /// `power(9E)`, it must be called from the thread that entered that
    /// routine. Returns `DDI_SUCCESS`, or `DDI_FAILURE` when the component or
    /// level is invalid. May be called from user or kernel context, and from
    /// interrupt context provided it is not the first power management
    /// function the driver calls.
    pub fn pm_power_has_changed(dip: dev_info_t, component: c_int, level: c_int) -> c_int;
    /// `pm_trans_check(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether a power cycle is currently advisable for a device
    /// that is sensitive to the number of power cycles, based on the data in
    /// `datap`. Drivers for such devices call it from `power(9E)` before
    /// powering off, and fail the `power(9E)` call when a cycle is not
    /// advised. In that case the function also tries to work out when the
    /// next cycle will be advisable and stores the number of seconds from now
    /// through `intervalp`, or 0 when it cannot tell; the driver can mark the
    /// component busy for that interval and use a `timeout(9F)` to make it
    /// idle again, which avoids repeated `power(9E)` calls. Returns 1 when a
    /// power cycle is advised, 0 when it is not, and -1 when an argument is
    /// invalid.
    pub fn pm_trans_check(datap: *mut pm_trans_data, intervalp: *mut time_t) -> c_int;
}
