//! Kernel statistics from `sys/kstat.h`.
//!
//! A kernel statistic, or kstat, is a named block of data that a driver
//! exports so that user programs such as `kstat(1M)` and `iostat(1M)` can
//! read it through `/dev/kstat`. Each kstat has a header, the `kstat(9S)`
//! structure declared here, and a data section whose shape is one of the
//! `KSTAT_TYPE_*` kinds. A driver obtains the header from `kstat_create(9F)`,
//! fills in the data section and any of the optional header fields, and
//! publishes it with `kstat_install(9F)`. `kstat_delete(9F)` removes it
//! again, usually in `detach(9E)`.
//!
//! The named kind, `kstat_named(9S)`, is an array of name and value pairs
//! and is what most drivers export. The I/O kind, `kstat_io(9S)`, holds the
//! byte and operation counters together with the wait and run queue
//! accounting that the `kstat_queue(9F)` family maintains. The interrupt
//! kind, `kstat_intr(9S)`, is an array of counters indexed by interrupt
//! class. The event timer kind is `kstat_timer_t`, whose helper functions
//! have no manual page and are therefore not bound here.
//!
//! This module also declares the `ks_update(9E)` and `ks_snapshot(9E)`
//! callback types that a driver can store in the header to compute
//! statistics on demand or to control how the data section is copied out.
//!
//! The header's `ks_lock` field is spelled `void *` in the C header so that
//! user programs need not see the kernel's lock types. It always holds a
//! `kmutex_t` pointer, and the manual page says so, so this module declares
//! it as `*mut kmutex_t` from [`crate::ksynch`]; the two have the same
//! representation.
//!
//! The functions in the header that have no manual page, namely
//! `kstat_create_zone`, `kstat_delete_byname`, `kstat_delete_byname_zone`,
//! `kstat_set_string`, `kstat_timer_init`, `kstat_timer_start`,
//! `kstat_timer_stop`, `kstat_zone_add`, `kstat_zone_remove`,
//! `kstat_zone_find`, `kstat_hold_bykid`, `kstat_hold_byname`,
//! `kstat_rele` and `kstat_init`, are not bound. The `net_kstat_create` and
//! `net_kstat_delete` functions belong to the `neti` family and are not part
//! of this module either.
//!
//! Every layout assertion below was derived by hand for amd64 under the
//! LP64 data model.

use crate::ksynch::kmutex_t;
use crate::types::*;

/// A unique kstat identifier, `kid_t`. The kernel assigns one to each kstat
/// in `kstat_create(9F)` and stores it in the `ks_kid` field. It is a plain
/// `int`.
pub type kid_t = c_int;

/// The size of the fixed name fields in a kstat header and in a named kstat,
/// including the terminating NUL (`KSTAT_STRLEN`). Only `KSTAT_STRLEN - 1`
/// characters of a name are significant. The C header defines it as the
/// integer 31; it is a `usize` here because it dimensions the arrays below.
pub const KSTAT_STRLEN: usize = 31;

/// `ks_update(9E)`. illumos DDI specific (illumos DDI).
///
/// The signature of the optional update callback stored in the `ks_update`
/// field of a `kstat(9S)`. The kernel calls it with `KSTAT_READ` before
/// copying the data section out to a reader, at which point the driver
/// refreshes the data section from its own counters, and with
/// `KSTAT_WRITE` after a writer has changed the data section, at which point
/// the driver may apply the new values to its own state or return `EACCES`
/// if it does not allow writes. The callback returns 0 on success, `EACCES`
/// if a write is refused and `EIO` for any other error.
///
/// The kernel already holds the kstat's `ks_lock`, if there is one, when it
/// calls this function, and it prevents the kstat from being deleted while
/// the call is in progress, so the callback must not take that lock itself.
/// The `ks_private` field is available for the driver to find its own data.
/// The field must be set before `kstat_install(9F)` is called.
pub type ks_update_t = unsafe extern "C" fn(ksp: *mut kstat, rw: c_int) -> c_int;

/// `ks_snapshot(9E)`. illumos DDI specific (illumos DDI).
///
/// The signature of the optional snapshot callback stored in the
/// `ks_snapshot` field of a `kstat(9S)`. The kernel calls it to marshal the
/// data section into `buf` on `KSTAT_READ`, and to copy `buf` back into the
/// data section on `KSTAT_WRITE`, in place of the default routine that
/// copies `ks_data_size` bytes. A driver supplies one when the data section
/// needs more than a plain copy, the usual case being named kstats with
/// `KSTAT_DATA_STRING` values whose string bytes must be appended to the
/// buffer and their pointers rewritten. The callback must set `ks_snaptime`
/// from `gethrtime(9F)` to timestamp the data. It returns 0 on success,
/// `EACCES` if a write is refused and `EIO` for any other error.
///
/// The kernel holds `ks_lock`, if there is one, across the call and keeps
/// the kstat alive, so the callback must not take the lock itself. It is
/// called from user context only. The field must be set before
/// `kstat_install(9F)` is called.
pub type ks_snapshot_t = unsafe extern "C" fn(ksp: *mut kstat, buf: *mut c_void, rw: c_int) -> c_int;

/// `kstat(9S)`. illumos DDI specific (illumos DDI).
///
/// The header of a kernel statistic. A driver only ever gets one from
/// `kstat_create(9F)` and must not allocate one any other way. Between
/// `kstat_create(9F)` and `kstat_install(9F)` the driver may set `ks_data`
/// for a virtual kstat, `ks_update`, `ks_snapshot`, `ks_private` and
/// `ks_lock`, and must initialise the data section, for example by naming
/// the entries of a named kstat. The remaining fields are maintained by the
/// kernel and are exposed here because the structure is public in the
/// header and user programs read the same layout through `/dev/kstat`.
#[repr(C)]
pub struct kstat {
    /// The time the kstat was created, from `gethrtime(9F)`. Set by the
    /// kernel.
    pub ks_crtime: hrtime_t,
    /// The next kstat on the system chain. Owned by the kernel.
    pub ks_next: *mut kstat,
    /// The unique identifier the kernel assigned to this kstat.
    pub ks_kid: kid_t,
    /// The name of the providing module, as passed to `kstat_create(9F)`.
    pub ks_module: [c_char; KSTAT_STRLEN],
    /// Reserved. Currently padding.
    pub ks_resv: c_uchar,
    /// The instance number of the provider, as passed to `kstat_create(9F)`.
    pub ks_instance: c_int,
    /// The name of the kstat, as passed to `kstat_create(9F)`.
    pub ks_name: [c_char; KSTAT_STRLEN],
    /// The kind of data section, one of the `KSTAT_TYPE_*` values.
    pub ks_type: c_uchar,
    /// The class of the kstat, as passed to `kstat_create(9F)`.
    pub ks_class: [c_char; KSTAT_STRLEN],
    /// The `KSTAT_FLAG_*` bits in effect for this kstat.
    pub ks_flags: c_uchar,
    /// The data section. `kstat_create(9F)` allocates it unless
    /// `KSTAT_FLAG_VIRTUAL` was given, in which case the driver points it at
    /// data of its own before `kstat_install(9F)`.
    pub ks_data: *mut c_void,
    /// The number of type specific records in the data section. For a
    /// variable size kstat the `ks_update(9E)` routine keeps it current.
    pub ks_ndata: uint_t,
    /// The total size of the data section in bytes. For a variable size
    /// kstat the `ks_update(9E)` routine keeps it current.
    pub ks_data_size: size_t,
    /// The time of the last snapshot of the data section. A custom
    /// `ks_snapshot(9E)` routine must set it.
    pub ks_snaptime: hrtime_t,
    /// The optional `ks_update(9E)` routine, or `None` for the default,
    /// which does nothing.
    pub ks_update: Option<ks_update_t>,
    /// Arbitrary provider private data, usually the driver's soft state,
    /// for use by the callbacks.
    pub ks_private: *mut c_void,
    /// The optional `ks_snapshot(9E)` routine, or `None` for the default
    /// copy.
    pub ks_snapshot: Option<ks_snapshot_t>,
    /// The mutex that protects the data section, or NULL. If it is set, the
    /// kernel takes it around every read and write of the kstat, so a driver
    /// that already updates its counters under one of its own mutexes can
    /// point this at that mutex and get consistent snapshots for free. The C
    /// header declares the field as `void *`.
    pub ks_lock: *mut kmutex_t,
}
/// The typedef name of [`kstat`].
pub type kstat_t = kstat;

// Layout under LP64: ks_crtime 8 at 0, ks_next 8 at 8, ks_kid 4 at 16,
// ks_module 31 at 20, ks_resv 1 at 51, ks_instance 4 at 52, ks_name 31 at 56,
// ks_type 1 at 87, ks_class 31 at 88, ks_flags 1 at 119, ks_data 8 at 120,
// ks_ndata 4 at 128, 4 bytes of padding, ks_data_size 8 at 136, ks_snaptime 8
// at 144, ks_update 8 at 152, ks_private 8 at 160, ks_snapshot 8 at 168,
// ks_lock 8 at 176; total 184.
const _: () = assert!(core::mem::size_of::<kstat>() == 184);
const _: () = assert!(core::mem::offset_of!(kstat, ks_kid) == 16);
const _: () = assert!(core::mem::offset_of!(kstat, ks_module) == 20);
const _: () = assert!(core::mem::offset_of!(kstat, ks_instance) == 52);
const _: () = assert!(core::mem::offset_of!(kstat, ks_name) == 56);
const _: () = assert!(core::mem::offset_of!(kstat, ks_type) == 87);
const _: () = assert!(core::mem::offset_of!(kstat, ks_class) == 88);
const _: () = assert!(core::mem::offset_of!(kstat, ks_flags) == 119);
const _: () = assert!(core::mem::offset_of!(kstat, ks_data) == 120);
const _: () = assert!(core::mem::offset_of!(kstat, ks_ndata) == 128);
const _: () = assert!(core::mem::offset_of!(kstat, ks_data_size) == 136);
const _: () = assert!(core::mem::offset_of!(kstat, ks_snaptime) == 144);
const _: () = assert!(core::mem::offset_of!(kstat, ks_update) == 152);
const _: () = assert!(core::mem::offset_of!(kstat, ks_lock) == 176);

/// A raw data section whose layout is private to the provider
/// (`KSTAT_TYPE_RAW`). `ks_ndata` is at least 1.
pub const KSTAT_TYPE_RAW: c_uchar = 0;
/// An array of `kstat_named(9S)` name and value pairs (`KSTAT_TYPE_NAMED`).
/// `ks_ndata` is at least 1.
pub const KSTAT_TYPE_NAMED: c_uchar = 1;
/// A single `kstat_intr(9S)` record of interrupt counters
/// (`KSTAT_TYPE_INTR`). `ks_ndata` is exactly 1.
pub const KSTAT_TYPE_INTR: c_uchar = 2;
/// A single `kstat_io(9S)` record of I/O statistics (`KSTAT_TYPE_IO`).
/// `ks_ndata` is exactly 1.
pub const KSTAT_TYPE_IO: c_uchar = 3;
/// An array of [`kstat_timer_t`] event timers (`KSTAT_TYPE_TIMER`).
/// `ks_ndata` is at least 1.
pub const KSTAT_TYPE_TIMER: c_uchar = 4;
/// The number of kstat types defined (`KSTAT_NUM_TYPES`).
pub const KSTAT_NUM_TYPES: c_uchar = 5;

/// The kernel does not allocate a data section; the driver points `ks_data`
/// at its own memory before `kstat_install(9F)` (`KSTAT_FLAG_VIRTUAL`). A
/// virtual kstat cannot also be persistent.
pub const KSTAT_FLAG_VIRTUAL: c_uchar = 0x01;
/// The size of the data section may change between reads, so the kernel
/// consults `ks_ndata` and `ks_data_size` after every `ks_update(9E)` call
/// (`KSTAT_FLAG_VAR_SIZE`).
pub const KSTAT_FLAG_VAR_SIZE: c_uchar = 0x02;
/// The data section may be written by a privileged user program
/// (`KSTAT_FLAG_WRITABLE`).
pub const KSTAT_FLAG_WRITABLE: c_uchar = 0x04;
/// The kstat survives `kstat_delete(9F)`, which merely marks it dormant, and
/// a later `kstat_create(9F)` with the same module, instance and name
/// reactivates it with its data intact (`KSTAT_FLAG_PERSISTENT`).
pub const KSTAT_FLAG_PERSISTENT: c_uchar = 0x08;
/// The kstat is a persistent kstat whose provider is currently gone
/// (`KSTAT_FLAG_DORMANT`). Set by the kernel.
pub const KSTAT_FLAG_DORMANT: c_uchar = 0x10;
/// The kstat has been created but not yet installed, so readers must skip it
/// (`KSTAT_FLAG_INVALID`). Set by the kernel.
pub const KSTAT_FLAG_INVALID: c_uchar = 0x20;
/// The named kstat contains `KSTAT_DATA_STRING` values whose bytes live
/// outside the data section (`KSTAT_FLAG_LONGSTRINGS`). Set by the kernel
/// when `kstat_named_setstr(9F)` is used.
pub const KSTAT_FLAG_LONGSTRINGS: c_uchar = 0x40;

/// The `rw` argument value telling `ks_update(9E)` or `ks_snapshot(9E)` that
/// the data section is about to be read (`KSTAT_READ`).
pub const KSTAT_READ: c_int = 0;
/// The `rw` argument value telling `ks_update(9E)` or `ks_snapshot(9E)` that
/// the data section has been written (`KSTAT_WRITE`).
pub const KSTAT_WRITE: c_int = 1;

/// The address half of the string view of a [`kstat_named_value`]. This is
/// the anonymous `addr` union inside the `str` member of the C structure. It
/// is 8 bytes wide in both data models thanks to the explicit padding.
#[repr(C)]
#[derive(Clone, Copy)]
pub union kstat_named_str_addr {
    /// The NUL-terminated string, as seen by the kernel and by 64-bit
    /// readers.
    pub ptr: *mut c_char,
    /// The same pointer as a 32-bit address, the view a 32-bit reader sees
    /// after the kernel has converted the snapshot.
    pub ptr32: u32,
    /// Padding that fixes the size of the union at 8 bytes.
    pub __pad: [c_char; 8],
}

/// The string view of a [`kstat_named_value`]. This is the anonymous `str`
/// structure inside the C union, used when `data_type` is
/// `KSTAT_DATA_STRING`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct kstat_named_str {
    /// Where the string lives.
    pub addr: kstat_named_str_addr,
    /// The length of the string in bytes including the terminating NUL.
    pub len: u32,
}

/// The `value` member of a `kstat_named(9S)`, which is an anonymous union in
/// C. The driver picks the member that matches the `data_type` it passed to
/// `kstat_named_init(9F)`. The `ll`, `ull`, `f` and `d` members are
/// obsolete; the manual page directs drivers to the fixed width integer
/// members instead.
#[repr(C)]
#[derive(Clone, Copy)]
pub union kstat_named_value {
    /// Sixteen bytes of characters, for `KSTAT_DATA_CHAR`. Large enough for
    /// a 128-bit integer.
    pub c: [c_char; 16],
    /// A signed 32-bit integer, for `KSTAT_DATA_INT32`.
    pub i32: i32,
    /// An unsigned 32-bit integer, for `KSTAT_DATA_UINT32`.
    pub ui32: u32,
    /// A pointer to a NUL-terminated string and its length, for
    /// `KSTAT_DATA_STRING`. Set it with `kstat_named_setstr(9F)`.
    pub str: kstat_named_str,
    /// A signed 64-bit integer, for `KSTAT_DATA_INT64`.
    pub i64: i64,
    /// An unsigned 64-bit integer, for `KSTAT_DATA_UINT64`.
    pub ui64: u64,
    /// A signed `long`, for `KSTAT_DATA_LONG`. 64 bits wide under LP64.
    pub l: c_long,
    /// An unsigned `long`, for `KSTAT_DATA_ULONG`. 64 bits wide under LP64.
    pub ul: ulong_t,
    /// A signed `long long`. Obsolete; use `i64`.
    pub ll: longlong_t,
    /// An unsigned `long long`. Obsolete; use `ui64`.
    pub ull: c_ulonglong,
    /// A single precision float. Obsolete and unused in the kernel.
    pub f: f32,
    /// A double precision float. Obsolete and unused in the kernel.
    pub d: f64,
}

/// `kstat_named(9S)`. illumos DDI specific (illumos DDI).
///
/// One name and value pair of a named kstat. `kstat_create(9F)` allocates
/// `ks_ndata` of these contiguously and points `ks_data` at the first. The
/// driver names each one and declares its type with `kstat_named_init(9F)`,
/// then updates the matching member of `value` as its statistics change.
#[repr(C)]
pub struct kstat_named {
    /// The name of the statistic, NUL-terminated. Set by
    /// `kstat_named_init(9F)`.
    pub name: [c_char; KSTAT_STRLEN],
    /// The `KSTAT_DATA_*` type of `value`. Set by `kstat_named_init(9F)`.
    pub data_type: c_uchar,
    /// The value of the statistic.
    pub value: kstat_named_value,
}
/// The typedef name of [`kstat_named`].
pub type kstat_named_t = kstat_named;

// Layout under LP64: name 31 at 0, data_type 1 at 31, value 16 at 32 with
// 8-byte alignment from its pointer and 64-bit members; total 48. Inside the
// value union the str view is addr 8 at 0 and len 4 at 8, padded to 16.
const _: () = assert!(core::mem::size_of::<kstat_named_str_addr>() == 8);
const _: () = assert!(core::mem::size_of::<kstat_named_str>() == 16);
const _: () = assert!(core::mem::size_of::<kstat_named_value>() == 16);
const _: () = assert!(core::mem::size_of::<kstat_named>() == 48);
const _: () = assert!(core::mem::offset_of!(kstat_named, data_type) == 31);
const _: () = assert!(core::mem::offset_of!(kstat_named, value) == 32);

/// The value is the `c` character array (`KSTAT_DATA_CHAR`).
pub const KSTAT_DATA_CHAR: c_uchar = 0;
/// The value is the signed 32-bit `i32` member (`KSTAT_DATA_INT32`).
pub const KSTAT_DATA_INT32: c_uchar = 1;
/// The value is the unsigned 32-bit `ui32` member (`KSTAT_DATA_UINT32`).
pub const KSTAT_DATA_UINT32: c_uchar = 2;
/// The value is the signed 64-bit `i64` member (`KSTAT_DATA_INT64`).
pub const KSTAT_DATA_INT64: c_uchar = 3;
/// The value is the unsigned 64-bit `ui64` member (`KSTAT_DATA_UINT64`).
pub const KSTAT_DATA_UINT64: c_uchar = 4;
/// The value is the obsolete single precision `f` member
/// (`KSTAT_DATA_FLOAT`).
pub const KSTAT_DATA_FLOAT: c_uchar = 5;
/// The value is the obsolete double precision `d` member
/// (`KSTAT_DATA_DOUBLE`).
pub const KSTAT_DATA_DOUBLE: c_uchar = 6;
/// The value is the `long` member `l` (`KSTAT_DATA_LONG`). This is the
/// kernel's private encoding under LP64; `kstat_install(9F)` rewrites it to
/// `KSTAT_DATA_INT64` so that user programs never see it.
pub const KSTAT_DATA_LONG: c_uchar = 7;
/// The value is the `unsigned long` member `ul` (`KSTAT_DATA_ULONG`). This
/// is the kernel's private encoding under LP64; `kstat_install(9F)` rewrites
/// it to `KSTAT_DATA_UINT64` so that user programs never see it.
pub const KSTAT_DATA_ULONG: c_uchar = 8;
/// The value is a string of arbitrary length described by the `str` member
/// (`KSTAT_DATA_STRING`). Such values are set with `kstat_named_setstr(9F)`
/// and need a custom `ks_snapshot(9E)` or the kernel's default handling of
/// long strings to reach user programs.
pub const KSTAT_DATA_STRING: c_uchar = 9;
/// The obsolete name for `KSTAT_DATA_INT64` (`KSTAT_DATA_LONGLONG`).
pub const KSTAT_DATA_LONGLONG: c_uchar = KSTAT_DATA_INT64;
/// The obsolete name for `KSTAT_DATA_UINT64` (`KSTAT_DATA_ULONGLONG`).
pub const KSTAT_DATA_ULONGLONG: c_uchar = KSTAT_DATA_UINT64;

/// The index of the hard interrupt counter in a `kstat_intr(9S)`, for
/// interrupts raised by the device itself (`KSTAT_INTR_HARD`).
pub const KSTAT_INTR_HARD: usize = 0;
/// The index of the soft interrupt counter, for interrupts the driver
/// triggers through the system (`KSTAT_INTR_SOFT`).
pub const KSTAT_INTR_SOFT: usize = 1;
/// The index of the watchdog interrupt counter, for interrupts raised by a
/// periodic timer (`KSTAT_INTR_WATCHDOG`).
pub const KSTAT_INTR_WATCHDOG: usize = 2;
/// The index of the spurious interrupt counter, for entries into the
/// handler that found nothing to service (`KSTAT_INTR_SPURIOUS`).
pub const KSTAT_INTR_SPURIOUS: usize = 3;
/// The index of the multiple service counter, for interrupts found and
/// serviced just before returning from another kind
/// (`KSTAT_INTR_MULTSVC`).
pub const KSTAT_INTR_MULTSVC: usize = 4;
/// The number of interrupt counters in a `kstat_intr(9S)`
/// (`KSTAT_NUM_INTRS`).
pub const KSTAT_NUM_INTRS: usize = 5;

/// `kstat_intr(9S)`. illumos DDI specific (illumos DDI).
///
/// The data section of an interrupt kstat. It is an array of counters
/// indexed by the `KSTAT_INTR_*` constants, one per class of interrupt. A
/// driver increments the appropriate counter from its handler; most report
/// only claimed hard interrupts and soft interrupts, but counting spurious
/// interrupts helps to pinpoint latency problems on auto-vectored buses. A
/// device with more than one interrupt of the same class uses one structure
/// per interrupt.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct kstat_intr {
    /// The counters, indexed by interrupt class.
    pub intrs: [uint_t; KSTAT_NUM_INTRS],
}
/// The typedef name of [`kstat_intr`].
pub type kstat_intr_t = kstat_intr;

// Layout: five 4-byte counters; total 20.
const _: () = assert!(core::mem::size_of::<kstat_intr>() == 20);

/// `kstat_io(9S)`. illumos DDI specific (illumos DDI).
///
/// The data section of an I/O kstat. The driver maintains the four basic
/// counters itself, updating them at the end of service, for example just
/// before calling `biodone(9F)`. The remaining fields are the accumulated
/// wait queue and run queue statistics: a running sum of the time each
/// queue was non-empty, a running sum of queue length multiplied by elapsed
/// time, the time of the last change, and the current length. They are
/// updated only through the `kstat_queue(9F)` family, which must be called
/// for every transition with `ks_lock` held so that the accounting stays
/// exact. All times are in nanoseconds from `gethrtime(9F)`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct kstat_io {
    /// The number of bytes read.
    pub nread: c_ulonglong,
    /// The number of bytes written.
    pub nwritten: c_ulonglong,
    /// The number of read operations.
    pub reads: uint_t,
    /// The number of write operations.
    pub writes: uint_t,
    /// The cumulative time the wait queue was non-empty.
    pub wtime: hrtime_t,
    /// The cumulative sum of wait queue length multiplied by time.
    pub wlentime: hrtime_t,
    /// The time the wait queue last changed.
    pub wlastupdate: hrtime_t,
    /// The cumulative time the run queue was non-empty.
    pub rtime: hrtime_t,
    /// The cumulative sum of run queue length multiplied by time.
    pub rlentime: hrtime_t,
    /// The time the run queue last changed.
    pub rlastupdate: hrtime_t,
    /// The number of requests currently on the wait queue.
    pub wcnt: uint_t,
    /// The number of requests currently on the run queue.
    pub rcnt: uint_t,
}
/// The typedef name of [`kstat_io`].
pub type kstat_io_t = kstat_io;

// Layout: nread 8 at 0, nwritten 8 at 8, reads 4 at 16, writes 4 at 20, six
// hrtime_t of 8 bytes each at 24 through 64, wcnt 4 at 72, rcnt 4 at 76;
// total 80.
const _: () = assert!(core::mem::size_of::<kstat_io>() == 80);
const _: () = assert!(core::mem::offset_of!(kstat_io, reads) == 16);
const _: () = assert!(core::mem::offset_of!(kstat_io, wtime) == 24);
const _: () = assert!(core::mem::offset_of!(kstat_io, wcnt) == 72);

/// One event timer of a `KSTAT_TYPE_TIMER` kstat, `kstat_timer_t`. There is
/// no manual page for this structure or for the `kstat_timer_*` functions
/// that maintain it; it is declared here because `KSTAT_TYPE_TIMER` is a
/// documented type of `kstat_create(9F)` and the layout is public in the
/// header.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct kstat_timer {
    /// The name of the event, NUL-terminated.
    pub name: [c_char; KSTAT_STRLEN],
    /// Reserved.
    pub resv: c_uchar,
    /// The number of events recorded.
    pub num_events: c_ulonglong,
    /// The cumulative duration of all events.
    pub elapsed_time: hrtime_t,
    /// The shortest event duration.
    pub min_time: hrtime_t,
    /// The longest event duration.
    pub max_time: hrtime_t,
    /// The start time of the previous event.
    pub start_time: hrtime_t,
    /// The stop time of the previous event.
    pub stop_time: hrtime_t,
}
/// The typedef name of [`kstat_timer`].
pub type kstat_timer_t = kstat_timer;

// Layout: name 31 at 0, resv 1 at 31, num_events 8 at 32, then five hrtime_t
// at 40 through 72; total 80.
const _: () = assert!(core::mem::size_of::<kstat_timer>() == 80);
const _: () = assert!(core::mem::offset_of!(kstat_timer, num_events) == 32);

unsafe extern "C" {
    /// `kstat_create(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Allocates and initialises a kstat header and, unless `ks_flag`
    /// contains `KSTAT_FLAG_VIRTUAL`, a zeroed data section large enough for
    /// `ks_ndata` records of the kind named by `ks_type`. The kstat is
    /// identified by the module name `ks_module`, the instance number
    /// `ks_instance`, usually from `ddi_get_instance(9F)`, and the name
    /// `ks_name`, of which only `KSTAT_STRLEN - 1` characters are
    /// significant. `ks_class` is a general category such as `disk`, `net`,
    /// `controller` or `misc`. `ks_flag` is a combination of the
    /// `KSTAT_FLAG_*` bits; a persistent kstat that was deleted earlier is
    /// reactivated with its data intact rather than created afresh.
    ///
    /// The kernel assigns a unique identifier, puts the kstat on the system
    /// chain and marks it invalid until `kstat_install(9F)` is called. In
    /// between, the driver initialises the data section and may set
    /// `ks_data` for a virtual kstat as well as `ks_update`, `ks_snapshot`,
    /// `ks_private` and `ks_lock`. Returns the new kstat, or NULL on
    /// failure. May be called from user or kernel context. The manual page
    /// gives `ks_ndata` as `ulong_t`; the header declares it `uint_t`, which
    /// is the prototype bound here.
    pub fn kstat_create(
        ks_module: *const c_char,
        ks_instance: c_int,
        ks_name: *const c_char,
        ks_class: *const c_char,
        ks_type: c_uchar,
        ks_ndata: uint_t,
        ks_flag: c_uchar,
    ) -> *mut kstat_t;
    /// `kstat_install(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Makes a fully initialised kstat visible to readers by clearing its
    /// invalid flag. The driver must have finished initialising the data
    /// section and any optional header fields before calling it. May be
    /// called from user or kernel context.
    pub fn kstat_install(ksp: *mut kstat_t);
    /// `kstat_delete(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Removes the kstat from the system chain and frees the header and any
    /// data section the kernel allocated. A persistent kstat is merely
    /// marked dormant. The caller must not hold the kstat's `ks_lock`, or it
    /// may deadlock against a reader. May be called from any context.
    pub fn kstat_delete(ksp: *mut kstat_t);
    /// `kstat_named_init(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Copies `name` into the named kstat entry and records `data_type`,
    /// which is one of the `KSTAT_DATA_*` values and selects the member of
    /// `value` the driver will use. May be called from user or kernel
    /// context.
    pub fn kstat_named_init(knp: *mut kstat_named_t, name: *const c_char, data_type: c_uchar);
    /// `kstat_named_setstr(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets the value of a named kstat entry of type `KSTAT_DATA_STRING` to
    /// point at the NUL-terminated string `src`, and records its length.
    /// This is the only supported way to change a long string value. The
    /// string is not copied, so it must remain valid for as long as the
    /// kstat does; for a persistent kstat that means even after the module
    /// that created it is unloaded. It is an error to call this on an entry
    /// of any other type. May be called from user or kernel context.
    pub fn kstat_named_setstr(knp: *mut kstat_named_t, src: *const c_char);
    /// `kstat_waitq_enter(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Records that a request has arrived and been placed in the pre-service
    /// wait queue, for example just before it is handed to `disksort(9F)`.
    /// Updates the wait queue time and length accounting of the
    /// `kstat_io(9S)`. Must be called with the kstat's `ks_lock` held, and
    /// every transition must be recorded or the statistics become
    /// meaningless. May be called from user or kernel context.
    pub fn kstat_waitq_enter(kiop: *mut kstat_io_t);
    /// `kstat_waitq_exit(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Records that a request has left the wait queue, for example just
    /// before the driver's start routine is called. Must be called with
    /// `ks_lock` held. May be called from user or kernel context.
    pub fn kstat_waitq_exit(kiop: *mut kstat_io_t);
    /// `kstat_runq_enter(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Records that a request has entered service and is on the run queue,
    /// just before the driver's start routine is called and after
    /// [`kstat_waitq_exit`]. Must be called with `ks_lock` held. May be
    /// called from user or kernel context.
    pub fn kstat_runq_enter(kiop: *mut kstat_io_t);
    /// `kstat_runq_exit(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Records that a request has finished service and left the run queue,
    /// just before `biodone(9F)` is called. Must be called with `ks_lock`
    /// held. May be called from user or kernel context.
    pub fn kstat_runq_exit(kiop: *mut kstat_io_t);
    /// `kstat_waitq_to_runq(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Moves a request from the wait queue to the run queue in one step,
    /// equivalent to [`kstat_waitq_exit`] followed by [`kstat_runq_enter`]
    /// but taking a single timestamp. Must be called with `ks_lock` held.
    /// May be called from user or kernel context.
    pub fn kstat_waitq_to_runq(kiop: *mut kstat_io_t);
    /// `kstat_runq_back_to_waitq(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Moves a request from the run queue back to the wait queue, which is
    /// occasionally needed, for example when throttling writes. Must be
    /// called with `ks_lock` held. May be called from user or kernel
    /// context.
    pub fn kstat_runq_back_to_waitq(kiop: *mut kstat_io_t);
}
