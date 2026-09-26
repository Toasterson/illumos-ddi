//! Raw bindings to the illumos kernel sensor framework, ksensor, from
//! `sys/sensors.h`.
//!
//! The kernel sensor framework lets a device driver publish sensors, such as
//! the temperature of a chip, the voltage on a rail or the current through a
//! supply, without writing a character device of its own. The framework
//! creates the minor nodes under `/dev/sensors`, answers the ioctls that user
//! programs send to them, and calls back into the driver whenever a reading
//! is wanted. A driver creates its sensors in `attach(9E)` with
//! `ksensor_create(9F)` or `ksensor_create_scalar_pcidev(9F)`, passing a
//! `ksensor_ops_t` vector of two entry points, `kso_kind(9E)` and
//! `kso_scalar(9E)`, and a private pointer that the framework hands back to
//! those entry points. It removes them again in `detach(9E)` with
//! `ksensor_remove(9F)`. The framework never calls the entry points during
//! `attach(9E)` or `detach(9E)`, but it may call them in parallel from any
//! number of threads at other times, so any locking is the driver's
//! responsibility. The general rules are described in `ksensor(9E)`.
//!
//! The framework itself is part of `genunix`, in `os/ksensor.c`, so a driver
//! that uses it needs no module dependency. The six functions it exports to
//! drivers are listed in the `ksensor` mapfile under
//! `uts/common/mapfiles/`, which is what the link step checks this crate
//! against. The `drv/ksensor` driver only implements the `/dev/sensors`
//! nodes and has nothing to offer a sensor provider.
//!
//! These interfaces are not stable. The comment at the top of
//! `sys/sensors.h` says that they should not be relied on at all, that they
//! are evolving and that they will change as more is added to the system.
//! Every manual page in the family gives the interface level "Volatile - This
//! interface is still evolving in illumos. API and ABI stability is not
//! guaranteed." The first sentence of that level is quoted on each item
//! below, as the other `-sys` crates do.
//!
//! This crate binds the three sensor functions on the `ksensor_create(9F)`
//! page, the three stock kind entry points on the `ksensor_kind(9F)` page,
//! the `ksensor_ops_t` vector and its two entry point types from
//! `ksensor_ops(9E)`, the two structures those entry points fill in, the
//! `SENSOR_KIND_*` and `SENSOR_UNIT_*` constants and `KSENSOR_ALL_IDS`.
//!
//! Three things are left out. The `SENSOR_IOCTL`, `SENSOR_IOCTL_KIND` and
//! `SENSOR_IOCTL_SCALAR` request codes belong to the user interface of
//! `ksensor(4D)`, and a driver never sees them. The sensor classes
//! `DDI_NT_SENSOR_TEMP_CPU` and `DDI_NT_SENSOR_TEMP_PCH`, which
//! `ksensor(9E)` recommends as the `class` argument of `ksensor_create(9F)`,
//! are defined in `sys/sunddi.h` together with the other minor node types
//! and not in `sys/sensors.h`, so they belong with the core bindings in
//! `ddi-sys`. Nothing is taken from `sys/ksensor_impl.h`, which is private
//! to the kernel.
//!
//! The layout assertions were derived by hand for amd64 under the LP64 data
//! model with `_KERNEL` defined.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

use ddi_sys::{c_char, c_int, c_void, dev_info_t, id_t};

/// The kind of the sensor is unknown. The framework uses this value to show
/// that a driver did not report a kind, and a driver must not use it itself
/// (`SENSOR_KIND_UNKNOWN`).
pub const SENSOR_KIND_UNKNOWN: u64 = 0x00;
/// The sensor measures temperature, in degrees Celsius, Fahrenheit or Kelvin.
/// Many chips embed such sensors for health monitoring, which makes this one
/// of the most common kinds (`SENSOR_KIND_TEMPERATURE`).
pub const SENSOR_KIND_TEMPERATURE: u64 = 0x01;
/// The sensor measures the voltage at a particular point in a circuit, which
/// is one part of working out how much power a device consumes
/// (`SENSOR_KIND_VOLTAGE`).
pub const SENSOR_KIND_VOLTAGE: u64 = 0x02;
/// The sensor measures the current in amperes that passes through a point in
/// a circuit, often as a proxy for the power a device draws at a fixed
/// voltage (`SENSOR_KIND_CURRENT`).
pub const SENSOR_KIND_CURRENT: u64 = 0x03;
/// The sensor does not measure a physical quantity directly but reports a
/// value on some fixed scale that is usually derived from one. The AMD Tctl
/// control temperature exposed by `smntemp(4D)` is an example. A synthetic
/// sensor reports the kind it is derived from in the `sik_derive` member of
/// [`sensor_ioctl_kind_t`] (`SENSOR_KIND_SYNTHETIC`).
pub const SENSOR_KIND_SYNTHETIC: u64 = 0x04;

/// The unit of the sensor is unknown. The framework uses this value to show
/// that a driver did not report a unit, and a driver must not use it itself
/// (`SENSOR_UNIT_UNKNOWN`).
pub const SENSOR_UNIT_UNKNOWN: u32 = 0x00;
/// The sensor measures degrees Celsius (`SENSOR_UNIT_CELSIUS`).
pub const SENSOR_UNIT_CELSIUS: u32 = 0x01;
/// The sensor measures degrees Fahrenheit (`SENSOR_UNIT_FAHRENHEIT`).
pub const SENSOR_UNIT_FAHRENHEIT: u32 = 0x02;
/// The sensor measures Kelvin (`SENSOR_UNIT_KELVIN`).
pub const SENSOR_UNIT_KELVIN: u32 = 0x03;
/// The sensor measures voltage in volts (`SENSOR_UNIT_VOLTS`).
pub const SENSOR_UNIT_VOLTS: u32 = 0x04;
/// The sensor measures current in amperes (`SENSOR_UNIT_AMPS`).
pub const SENSOR_UNIT_AMPS: u32 = 0x05;
/// The measurement has no unit because it is not tied to a physical
/// property. Only sensors of the kind [`SENSOR_KIND_SYNTHETIC`] should use it
/// (`SENSOR_UNIT_NONE`).
pub const SENSOR_UNIT_NONE: u32 = 0x06;

/// The special identifier that asks `ksensor_remove(9F)` to remove every
/// sensor the device instance has registered, which simplifies teardown in
/// `detach(9E)`. The C header defines it as `INT_MIN` (`KSENSOR_ALL_IDS`).
pub const KSENSOR_ALL_IDS: id_t = c_int::MIN;

/// `ksensor_ops(9E)`. Volatile - This interface is still evolving in illumos.
///
/// The answer to the question of what kind of sensor this is, filled in by
/// the `kso_kind(9E)` entry point. The page adds that API and ABI stability
/// is not guaranteed. There is no manual page for the structure itself; its
/// members are described on the `ksensor_ops(9E)` page. The same layout is
/// returned to user programs by the `SENSOR_IOCTL_KIND` ioctl of
/// `ksensor(4D)`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct sensor_ioctl_kind {
    /// The kind of the sensor, one of the `SENSOR_KIND_*` values other than
    /// [`SENSOR_KIND_UNKNOWN`]. A driver that sets
    /// [`SENSOR_KIND_SYNTHETIC`] should also fill in `sik_derive`.
    pub sik_kind: u64,
    /// For a synthetic sensor, the kind of physical quantity it is derived
    /// from, if that is known, which may be any kind other than
    /// [`SENSOR_KIND_UNKNOWN`] and [`SENSOR_KIND_SYNTHETIC`]. For any other
    /// kind of sensor the driver leaves the default value the framework put
    /// there.
    pub sik_derive: u64,
}
/// The typedef name of [`sensor_ioctl_kind`].
pub type sensor_ioctl_kind_t = sensor_ioctl_kind;

// sensor_ioctl_kind under LP64 is two uint64_t members, sik_kind 8 at 0 and
// sik_derive 8 at 8, with no padding; total 16.
const _: () = assert!(core::mem::size_of::<sensor_ioctl_kind>() == 16);
const _: () = assert!(core::mem::offset_of!(sensor_ioctl_kind, sik_kind) == 0);
const _: () = assert!(core::mem::offset_of!(sensor_ioctl_kind, sik_derive) == 8);

/// `ksensor_ops(9E)`. Volatile - This interface is still evolving in illumos.
///
/// A scalar reading of a sensor, filled in by the `kso_scalar(9E)` entry
/// point. The page adds that API and ABI stability is not guaranteed. There
/// is no manual page for the structure itself; its members are described on
/// the `ksensor_ops(9E)` page and in `ksensor(9E)`. A scalar measurement is
/// a signed value together with a granularity that says how many increments
/// of the value make up one unit, and a precision that says how accurate the
/// sensor is. The same layout is returned to user programs by the
/// `SENSOR_IOCTL_SCALAR` ioctl of `ksensor(4D)`.
///
/// As an example, a temperature sensor that counts in half degrees Celsius
/// and is accurate to one degree either way reports the unit
/// [`SENSOR_UNIT_CELSIUS`], a granularity of 2 and a precision of 2.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct sensor_ioctl_scalar {
    /// The unit of the reading, one of the `SENSOR_UNIT_*` values other than
    /// [`SENSOR_UNIT_UNKNOWN`]. [`SENSOR_UNIT_NONE`] is only appropriate for a
    /// sensor whose kind is [`SENSOR_KIND_SYNTHETIC`].
    pub sis_unit: u32,
    /// The granularity of the reading, the number of increments per unit. A
    /// value of 10 means that `sis_value` is in tenths of the unit, so a
    /// reader divides by 10. A negative value means that a reader multiplies
    /// instead, so that -2 means each increment is two units.
    pub sis_gran: i32,
    /// The accuracy of the sensor, as a plus or minus value in units of the
    /// granularity. A driver leaves it at zero if the precision is not known.
    pub sis_prec: u32,
    /// Padding that keeps `sis_value` aligned. It carries no information.
    pub sis_pad: u32,
    /// The reading itself, a signed quantity in units of the granularity.
    pub sis_value: i64,
}
/// The typedef name of [`sensor_ioctl_scalar`].
pub type sensor_ioctl_scalar_t = sensor_ioctl_scalar;

// sensor_ioctl_scalar under LP64: sis_unit 4 at 0, sis_gran 4 at 4, sis_prec
// 4 at 8, sis_pad 4 at 12, sis_value 8 at 16, which is naturally aligned;
// total 24.
const _: () = assert!(core::mem::size_of::<sensor_ioctl_scalar>() == 24);
const _: () = assert!(core::mem::offset_of!(sensor_ioctl_scalar, sis_unit) == 0);
const _: () = assert!(core::mem::offset_of!(sensor_ioctl_scalar, sis_pad) == 12);
const _: () = assert!(core::mem::offset_of!(sensor_ioctl_scalar, sis_value) == 16);

/// `kso_kind(9E)`. Volatile - This interface is still evolving in illumos.
///
/// The signature of the entry point that tells the framework what kind of
/// physical quantity a sensor measures. The C header names the type
/// `ksensor_kind_f`. The page adds that API and ABI stability is not
/// guaranteed. `driver` is the private pointer the driver passed to
/// `ksensor_create(9F)` or `ksensor_create_scalar_pcidev(9F)`, and `kind`
/// points to a [`sensor_ioctl_kind_t`] that the driver fills in. The entry
/// point is required.
///
/// Drivers whose sensor is an ordinary temperature, voltage or current
/// sensor need not write this entry point and can use one of the stock
/// implementations, [`ksensor_kind_temperature`], [`ksensor_kind_voltage`]
/// or [`ksensor_kind_current`], instead.
///
/// The entry point returns 0 once the structure is filled in, or a positive
/// error number otherwise. It is generally called from kernel context and
/// may be called from user context, but the driver must not assume either
/// and must not copy data into or out of a user process. The framework does
/// not serialise calls, so the entry point may run in several threads at the
/// same time.
pub type ksensor_kind_f = unsafe extern "C" fn(driver: *mut c_void, kind: *mut sensor_ioctl_kind_t) -> c_int;

/// `kso_scalar(9E)`. Volatile - This interface is still evolving in illumos.
///
/// The signature of the entry point that reads the current value of a
/// scalar sensor. The C header names the type `ksensor_scalar_f`. The page
/// adds that API and ABI stability is not guaranteed. `driver` is the
/// private pointer the driver passed when it created the sensor, and
/// `scalar` points to a [`sensor_ioctl_scalar_t`] that the driver fills in
/// with the unit, granularity, precision and value of the reading. The entry
/// point is required.
///
/// The entry point returns 0 once the structure is filled in, or a positive
/// error number otherwise. `EIO` is the error to use when the driver could
/// not communicate with the device to read the sensor, but any error that
/// describes the situation more accurately may be returned. It is generally
/// called from kernel context and may be called from user context, but the
/// driver must not assume either and must not copy data into or out of a
/// user process. The framework does not serialise calls, so the entry point
/// may run in several threads at the same time.
pub type ksensor_scalar_f = unsafe extern "C" fn(driver: *mut c_void, scalar: *mut sensor_ioctl_scalar_t) -> c_int;

/// `ksensor_ops(9E)`. Volatile - This interface is still evolving in illumos.
///
/// The operations vector of a sensor, passed to `ksensor_create(9F)` or
/// `ksensor_create_scalar_pcidev(9F)`. The page adds that API and ABI
/// stability is not guaranteed. Both entry points are required. The vector
/// must remain valid for as long as the sensor exists and is therefore
/// usually a constant in the driver's program text. The framework never
/// calls the entry points during the driver's `attach(9E)` or `detach(9E)`,
/// but it may call them in parallel at any other time, both for the same
/// sensor and for different sensors of the same driver.
///
/// In C the structure has no tag, only the typedef name, so the Rust
/// structure carries the typedef name.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ksensor_ops_t {
    /// The `kso_kind(9E)` entry point, which reports the kind of the sensor.
    /// It may be one of the stock implementations from `ksensor_kind(9F)`.
    pub kso_kind: Option<ksensor_kind_f>,
    /// The `kso_scalar(9E)` entry point, which reads the current value.
    pub kso_scalar: Option<ksensor_scalar_f>,
}

// ksensor_ops_t under LP64 is two function pointers, kso_kind 8 at 0 and
// kso_scalar 8 at 8; total 16.
const _: () = assert!(core::mem::size_of::<ksensor_ops_t>() == 16);
const _: () = assert!(core::mem::offset_of!(ksensor_ops_t, kso_kind) == 0);
const _: () = assert!(core::mem::offset_of!(ksensor_ops_t, kso_scalar) == 8);

unsafe extern "C" {
    /// `ksensor_create(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Creates a sensor for the device instance `dip` and registers it with
    /// the framework. The page adds that API and ABI stability is not
    /// guaranteed. `ops` is the operations vector, which must stay valid for
    /// the lifetime of the sensor, and `driver` is an opaque pointer that the
    /// framework passes back to every entry point. Because the entry points
    /// do not receive the `dev_info_t`, `driver` should generally not be
    /// NULL. `name` names the sensor and becomes the name of its node, and
    /// `class` is a colon separated string that begins with `ddi_sensor`,
    /// continues with the kind and then describes the hardware, and decides
    /// the directory under `/dev/sensors`. Drivers should use one of the
    /// classes the framework defines, such as `DDI_NT_SENSOR_TEMP_CPU`, so
    /// that the sensor is tied into system services. On success the new
    /// sensor's identifier is stored through `idp`, and the driver keeps it
    /// for `ksensor_remove(9F)`.
    ///
    /// The function returns 0 on success and a non-zero error number
    /// otherwise. It fails with `EAGAIN` if it is not called from the
    /// driver's `attach(9E)` entry point, with `EEXIST` if a sensor with the
    /// same class and name already exists, and with `EINVAL` if a pointer is
    /// invalid or `ops`, `name` or `class` is malformed. It may only be
    /// called from a driver's `attach(9E)` entry point.
    pub fn ksensor_create(
        dip: dev_info_t,
        ops: *const ksensor_ops_t,
        driver: *mut c_void,
        name: *const c_char,
        class: *const c_char,
        idp: *mut id_t,
    ) -> c_int;

    /// `ksensor_create_scalar_pcidev(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Creates a scalar sensor for a PCI or PCI Express device and registers
    /// it with the fault management architecture as well as with the
    /// framework. The page adds that API and ABI stability is not
    /// guaranteed. It is meant for sensors that cover the whole device and
    /// should not be used for a sensor that belongs to a single function.
    /// `dip` is the device instance, `kind` is one of the `SENSOR_KIND_*`
    /// values from which the framework derives the class, `ops` and
    /// `driver` are as for `ksensor_create(9F)`, and `name` should describe
    /// what the sensor measures or where, such as the supply rail of a
    /// voltage sensor. On success the new sensor's identifier is stored
    /// through `idp`.
    ///
    /// The function returns 0 on success and a non-zero error number
    /// otherwise. Besides the errors of `ksensor_create(9F)`, it fails with
    /// `EINVAL` if `dip` is not a PCI or PCI Express device node with a
    /// `regs` property, with `EIO` if that property is malformed, and with
    /// `ENOTSUP` if `kind` is not supported. It may only be called from a
    /// driver's `attach(9E)` entry point.
    pub fn ksensor_create_scalar_pcidev(
        dip: dev_info_t,
        kind: u64,
        ops: *const ksensor_ops_t,
        driver: *mut c_void,
        name: *const c_char,
        idp: *mut id_t,
    ) -> c_int;

    /// `ksensor_remove(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Removes the sensor `id` that the device instance `dip` created
    /// earlier. The page adds that API and ABI stability is not guaranteed.
    /// Passing [`KSENSOR_ALL_IDS`] removes every sensor the instance has
    /// registered. Because the framework guarantees that no entry point runs
    /// during `attach(9E)` or `detach(9E)`, the driver need not synchronise
    /// with its entry points before calling this function.
    ///
    /// The function returns 0 once the sensor is deleted and a non-zero
    /// error number otherwise. It fails with `EAGAIN` if it is not called
    /// from `attach(9E)` or `detach(9E)`, and with `ENOENT` if `id` does not
    /// exist for `dip` or, for [`KSENSOR_ALL_IDS`], if the driver has no
    /// sensors. It may only be called from a driver's `attach(9E)` or
    /// `detach(9E)` entry point.
    pub fn ksensor_remove(dip: dev_info_t, id: id_t) -> c_int;

    /// `ksensor_kind_current(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// A stock implementation of the `kso_kind(9E)` entry point for a
    /// current sensor. The page adds that API and ABI stability is not
    /// guaranteed. It sets the kind in `kind` to [`SENSOR_KIND_CURRENT`] and
    /// ignores `driver`. It is meant to be placed directly in the `kso_kind`
    /// member of a [`ksensor_ops_t`]. A driver that calls it from its own
    /// entry point must pass the return value back to the framework.
    ///
    /// The function returns 0 after updating `kind` and a non-zero error
    /// number otherwise. It may be called from user or kernel context.
    pub fn ksensor_kind_current(driver: *mut c_void, kind: *mut sensor_ioctl_kind_t) -> c_int;

    /// `ksensor_kind_temperature(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// A stock implementation of the `kso_kind(9E)` entry point for a
    /// temperature sensor. The page adds that API and ABI stability is not
    /// guaranteed. It sets the kind in `kind` to [`SENSOR_KIND_TEMPERATURE`]
    /// and ignores `driver`. It is meant to be placed directly in the
    /// `kso_kind` member of a [`ksensor_ops_t`]. A driver that calls it from
    /// its own entry point must pass the return value back to the
    /// framework.
    ///
    /// The function returns 0 after updating `kind` and a non-zero error
    /// number otherwise. It may be called from user or kernel context.
    pub fn ksensor_kind_temperature(driver: *mut c_void, kind: *mut sensor_ioctl_kind_t) -> c_int;

    /// `ksensor_kind_voltage(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// A stock implementation of the `kso_kind(9E)` entry point for a
    /// voltage sensor. The page adds that API and ABI stability is not
    /// guaranteed. It sets the kind in `kind` to [`SENSOR_KIND_VOLTAGE`] and
    /// ignores `driver`. It is meant to be placed directly in the `kso_kind`
    /// member of a [`ksensor_ops_t`]. A driver that calls it from its own
    /// entry point must pass the return value back to the framework.
    ///
    /// The function returns 0 after updating `kind` and a non-zero error
    /// number otherwise. It may be called from user or kernel context.
    pub fn ksensor_kind_voltage(driver: *mut c_void, kind: *mut sensor_ioctl_kind_t) -> c_int;
}
