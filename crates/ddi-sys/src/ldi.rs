//! The layered driver interface from `sys/sunldi.h`.
//!
//! The LDI lets one kernel module open and use a device driven by another,
//! in the same way a user process would through a file descriptor. A
//! consumer first obtains an `ldi_ident_t` that says who it is, from its
//! device node, device number or stream with the `ldi_ident_from_dev(9F)`
//! family, then opens the target by device number, path or device
//! identifier with the `ldi_open_by_dev(9F)` family and receives an
//! `ldi_handle_t`. The handle is passed to `ldi_read(9F)`,
//! `ldi_write(9F)`, `ldi_ioctl(9F)`, `ldi_strategy(9F)`, `ldi_poll(9F)`,
//! `ldi_putmsg(9F)` and the other request functions, which forward the
//! request to the target driver's entry point. `ldi_close(9F)` releases the
//! handle and `ldi_ident_release(9F)` the identifier.
//!
//! The `ldi_prop_*(9F)` functions look up properties of the opened device
//! the way the `ddi_prop_*(9F)` functions do for the caller's own node, and
//! the `ldi_get_*(9F)` functions report facts about the handle such as its
//! device number, open type, device identifier and minor name.
//!
//! The LDI event framework lets a consumer learn when the device it has
//! opened is about to go offline or become degraded. The consumer fetches a
//! cookie for the event name with `ldi_ev_get_cookie(9F)`, registers a
//! `ldi_ev_callback_t` with `ldi_ev_register_callbacks(9F)`, and removes it
//! again with `ldi_ev_remove_callbacks(9F)`. A layered driver that itself
//! exports minor nodes passes events further up the stack with
//! `ldi_ev_notify(9F)` and `ldi_ev_finalize(9F)`.
//!
//! Many of the older LDI manual pages have no INTERFACE LEVEL section at
//! all. The doc comment of each such function says so in place of the
//! level; the pages that do have the section all give illumos DDI specific
//! (illumos DDI), and none is marked obsolete.
//!
//! The pages `ldi_add_event_handler(9F)`, `ldi_get_eventcookie(9F)` and
//! `ldi_remove_event_handler(9F)` are marked Obsolete, having been replaced
//! by the `ldi_ev_*` functions, and are not bound. `ldi_ident_from_anon`,
//! `ldi_ident_from_mod`, `ldi_ident_from_major`, `ldi_vp_from_name`,
//! `ldi_vp_from_devid` and `ldi_prop_op` are prototyped in the header but
//! have no manual page and are not in the `ddi` mapfile, so they are not
//! bound either. `LDI_EV_NONE` is a return value private to the framework
//! that no manual page documents and is likewise omitted. Nothing is taken
//! from `sunldi_impl.h`.
//!
//! The `LDI_DEV_T_ANY` property flag lives in `sys/ddipropdefs.h` but is
//! used only by the LDI property functions, so it is declared here.

use crate::types::*;

opaque! {
    /// The structure behind an `ldi_ident_t`, `struct __ldi_ident`. It
    /// records which module or device is doing the layered open.
    __ldi_ident;
    /// The structure behind an `ldi_handle_t`, `struct __ldi_handle`. It
    /// represents one layered open of a device.
    __ldi_handle;
    /// The structure behind an `ldi_callback_id_t`, `struct
    /// __ldi_callback_id`. It identifies a registered event callback.
    __ldi_callback_id;
    /// The structure behind an `ldi_ev_cookie_t`, `struct __ldi_ev_cookie`.
    /// It identifies an event type.
    __ldi_ev_cookie;
}

/// An LDI identifier, `ldi_ident_t`, obtained from the
/// `ldi_ident_from_dev(9F)` family and passed to the open functions.
pub type ldi_ident_t = *mut __ldi_ident;
/// A layered handle, `ldi_handle_t`, returned by a successful open and
/// passed to every request function.
pub type ldi_handle_t = *mut __ldi_handle;
/// A callback registration identifier, `ldi_callback_id_t`, returned by
/// `ldi_ev_register_callbacks(9F)`.
pub type ldi_callback_id_t = *mut __ldi_callback_id;
/// An event cookie, `ldi_ev_cookie_t`, returned by `ldi_ev_get_cookie(9F)`.
pub type ldi_ev_cookie_t = *mut __ldi_ev_cookie;

/// The event or state change succeeded, or the callback permits it
/// (`LDI_EV_SUCCESS`).
pub const LDI_EV_SUCCESS: c_int = 0;
/// The event or state change failed, or the callback blocks it
/// (`LDI_EV_FAILURE`).
pub const LDI_EV_FAILURE: c_int = -1;

/// The name of the event raised when a device is about to go offline
/// (`LDI_EV_OFFLINE`). It can be blocked from a notify callback, and an
/// offline only succeeds if no one, layered consumers included, holds the
/// device open.
pub const LDI_EV_OFFLINE: &core::ffi::CStr = c"LDI:EVENT:OFFLINE";
/// The name of the event raised when a device becomes degraded
/// (`LDI_EV_DEGRADE`). It cannot be blocked, so only finalize callbacks are
/// invoked for it.
pub const LDI_EV_DEGRADE: &core::ffi::CStr = c"LDI:EVENT:DEGRADE";
/// The name of the event raised when a device has been physically removed
/// (`LDI_EV_DEVICE_REMOVE`). The header defines it alongside the other two;
/// the manual pages list only the offline and degrade events.
pub const LDI_EV_DEVICE_REMOVE: &core::ffi::CStr = c"LDI:EVENT:DEVICE_REMOVE";

/// The first version of the event callback structure (`LDI_EV_CB_VERS_1`).
pub const LDI_EV_CB_VERS_1: uint_t = 1;
/// The current version of the event callback structure (`LDI_EV_CB_VERS`).
/// A consumer sets `cb_vers` to this value.
pub const LDI_EV_CB_VERS: uint_t = LDI_EV_CB_VERS_1;

/// Match a property regardless of the device number it was created with,
/// including `DDI_DEV_T_NONE` (`LDI_DEV_T_ANY`). One of the `flags` bits of
/// the `ldi_prop_*(9F)` functions, alongside `DDI_PROP_DONTPASS` and
/// `DDI_PROP_NOTPROM`. It is the only way to find properties created with
/// `DDI_DEV_T_NONE`, which includes every PROM property.
pub const LDI_DEV_T_ANY: uint_t = 0x2000;

/// The signature of the `cb_notify` member of `ldi_ev_callback(9S)`, as
/// described in `ldi_ev_register_callbacks(9F)`.
///
/// The framework calls it just before an event that can be blocked, such
/// as `LDI_EV_OFFLINE`, occurs on the device behind `lh`. `cookie`
/// identifies the event, `arg` is the private value given at registration,
/// and `ev_data` is event specific. The callback returns `LDI_EV_SUCCESS` to
/// let the event proceed or `LDI_EV_FAILURE` to block it; any other value is
/// treated as an error and blocks the event. A layered driver that exports
/// minor nodes must call `ldi_ev_notify(9F)` first to ask its own consumers,
/// and if it then decides to block the event it must call
/// `ldi_ev_finalize(9F)` with `LDI_EV_FAILURE` for them before returning.
/// A consumer that permits an offline must close its handle here so the
/// offline can succeed.
pub type ldi_ev_cb_notify_t =
    unsafe extern "C" fn(lh: ldi_handle_t, cookie: ldi_ev_cookie_t, arg: *mut c_void, ev_data: *mut c_void) -> c_int;
/// The signature of the `cb_finalize` member of `ldi_ev_callback(9S)`, as
/// described in `ldi_ev_register_callbacks(9F)`.
///
/// The framework calls it after an event has run its course, for every
/// event including those that cannot be blocked. `ldi_result` is
/// `LDI_EV_SUCCESS` if the state change happened and `LDI_EV_FAILURE` if it
/// did not, so that a consumer can undo what it did in its notify callback,
/// for example by reopening the device. If the consumer closed its handle
/// in the notify callback, `lh` is NULL here and the consumer must have
/// kept enough state in `arg` to reopen the device. A layered driver
/// reconfigures itself before propagating the result with
/// `ldi_ev_finalize(9F)`.
pub type ldi_ev_cb_finalize_t = unsafe extern "C" fn(
    lh: ldi_handle_t,
    cookie: ldi_ev_cookie_t,
    ldi_result: c_int,
    arg: *mut c_void,
    ev_data: *mut c_void,
);

/// The callback vector passed to `ldi_ev_register_callbacks(9F)`,
/// `ldi_ev_callback_t`. It has no manual page of its own; its layout and
/// semantics are described in `ldi_ev_register_callbacks(9F)`.
///
/// A consumer may register a notify callback, a finalize callback or both,
/// setting the unused member to `None`; registering with both `None` is an
/// error. Notify callbacks are only invoked for events that can be blocked.
/// Callbacks are only invoked while the handle is open, with the single
/// exception of a finalize callback following a notify callback that closed
/// the handle. They are not removed automatically when the handle is
/// closed and must be removed with `ldi_ev_remove_callbacks(9F)`, or the
/// framework may panic once the registering entity is gone.
#[repr(C)]
pub struct ldi_ev_callback {
    /// Always [`LDI_EV_CB_VERS`].
    pub cb_vers: uint_t,
    /// The notify callback, or `None`.
    pub cb_notify: Option<ldi_ev_cb_notify_t>,
    /// The finalize callback, or `None`.
    pub cb_finalize: Option<ldi_ev_cb_finalize_t>,
}
/// The typedef name of [`ldi_ev_callback`].
pub type ldi_ev_callback_t = ldi_ev_callback;

// Layout under LP64: cb_vers 4 at 0, 4 bytes of padding, cb_notify 8 at 8,
// cb_finalize 8 at 16; total 24.
const _: () = assert!(core::mem::size_of::<ldi_ev_callback>() == 24);
const _: () = assert!(core::mem::offset_of!(ldi_ev_callback, cb_notify) == 8);
const _: () = assert!(core::mem::offset_of!(ldi_ev_callback, cb_finalize) == 16);

unsafe extern "C" {
    /// `ldi_ident_from_dev(9F)`. No interface level given.
    ///
    /// Allocates an LDI identifier associated with the device number `dev`
    /// and stores it through `lip`. The identifier tells the framework who
    /// is performing later layered opens. Returns 0 on success and `EINVAL`
    /// for invalid arguments. May be called from user or kernel context.
    pub fn ldi_ident_from_dev(dev: dev_t, lip: *mut ldi_ident_t) -> c_int;
    /// `ldi_ident_from_dip(9F)`. No interface level given.
    ///
    /// Allocates an LDI identifier associated with the device node `dip`,
    /// usually the caller's own, and stores it through `lip`. Returns 0 on
    /// success and `EINVAL` for invalid arguments. May be called from user
    /// or kernel context.
    pub fn ldi_ident_from_dip(dip: dev_info_t, lip: *mut ldi_ident_t) -> c_int;
    /// `ldi_ident_from_stream(9F)`. No interface level given.
    ///
    /// Allocates an LDI identifier associated with the stream that the
    /// queue `sq` belongs to and stores it through `lip`. Returns 0 on
    /// success and `EINVAL` for invalid arguments. May be called from user
    /// or kernel context.
    pub fn ldi_ident_from_stream(sq: *mut crate::streams::queue_t, lip: *mut ldi_ident_t) -> c_int;
    /// `ldi_ident_release(9F)`. No interface level given.
    ///
    /// Releases an identifier obtained from one of the `ldi_ident_from_*`
    /// functions. May be called from user or kernel context.
    pub fn ldi_ident_release(li: ldi_ident_t);

    /// `ldi_open_by_dev(9F)`. No interface level given.
    ///
    /// Opens the block, character or STREAMS device whose number is stored
    /// at `devp`. `otyp` is `OTYP_BLK` or `OTYP_CHR`, and must be `OTYP_CHR`
    /// for a STREAMS device; other open types, including `OTYP_LYR`, are
    /// deprecated for this function. `flag` is a combination of `FREAD`,
    /// `FWRITE`, `FEXCL`, `FNDELAY` and `FNOCTTY`, `cr` the credentials to
    /// open with, and `li` the caller's identifier. On success the function
    /// returns 0 and stores the layered handle through `lhp`; the caller
    /// should then re-read `*devp`, because a cloning device changes it.
    /// Before the device is reached the function may fail with `EINVAL`,
    /// `ENODEV` if the device does not exist, or `ENXIO` for an unsupported
    /// operation or access mode; otherwise the device's own error is
    /// returned. May be called from user or kernel context, but not from
    /// `attach(9E)`, `detach(9E)` or `power(9E)`, where it could deadlock or
    /// crash the system.
    pub fn ldi_open_by_dev(
        devp: *mut dev_t,
        otyp: c_int,
        flag: c_int,
        cr: *mut cred_t,
        lhp: *mut ldi_handle_t,
        li: ldi_ident_t,
    ) -> c_int;
    /// `ldi_open_by_name(9F)`. No interface level given.
    ///
    /// Opens the device at the absolute path `pathname`, a NUL-terminated
    /// string in kernel memory. A `/devices` path works before the root
    /// file system is mounted; any other path to a device node works only
    /// afterwards. `flag`, `cr`, `lhp` and `li` are as for
    /// [`ldi_open_by_dev`]. Before the device is reached the function may
    /// fail with `EINVAL`, `ENODEV` if the path does not exist, `EACCES` if
    /// search permission is denied on a path component or the node's
    /// permissions refuse `cr`, or `ENXIO`; otherwise the device's own error
    /// is returned. Same context rules as [`ldi_open_by_dev`].
    pub fn ldi_open_by_name(
        pathname: *const c_char,
        flag: c_int,
        cr: *mut cred_t,
        lhp: *mut ldi_handle_t,
        li: ldi_ident_t,
    ) -> c_int;
    /// `ldi_open_by_devid(9F)`. No interface level given.
    ///
    /// Opens the minor node named `minor_name` of the device with the
    /// device identifier `devid`. Where several paths lead to the device
    /// and multipathing is not virtualising them, the handle corresponds to
    /// one particular path, which may differ between calls; a caller that
    /// wants its own failover opens each path with [`ldi_open_by_name`]
    /// instead. `flag`, `cr`, `lhp` and `li` are as for [`ldi_open_by_dev`],
    /// and the return values and context rules are the same.
    pub fn ldi_open_by_devid(
        devid: crate::devid::ddi_devid_t,
        minor_name: *const c_char,
        flag: c_int,
        cr: *mut cred_t,
        lhp: *mut ldi_handle_t,
        li: ldi_ident_t,
    ) -> c_int;
    /// `ldi_close(9F)`. No interface level given.
    ///
    /// Closes a layered handle obtained from one of the open functions.
    /// `flag` is the open flags and `cr` the credentials. After the call
    /// the handle is invalid. Returns 0 on success, `EINVAL` for invalid
    /// arguments, or an error from the device. Same context rules as
    /// [`ldi_open_by_dev`].
    pub fn ldi_close(lh: ldi_handle_t, flag: c_int, cr: *mut cred_t) -> c_int;

    /// `ldi_read(9F)`. No interface level given.
    ///
    /// Passes a read request described by the `uio(9S)` at `uiop`, whose
    /// vectors may point into user or kernel memory, to the read entry point
    /// of the device behind `lh`. Supported for block, character and STREAMS
    /// devices. Returns 0 on success; before the device is reached it may
    /// fail with `EINVAL` or `ENOTSUP`, and otherwise returns the device's
    /// own error. May be called from user or kernel context.
    pub fn ldi_read(lh: ldi_handle_t, uiop: *mut crate::uio::uio, cr: *mut cred_t) -> c_int;
    /// `ldi_write(9F)`. No interface level given.
    ///
    /// Passes a write request described by the `uio(9S)` at `uiop` to the
    /// write entry point of the device behind `lh`. Supported for block,
    /// character and STREAMS devices. Returns 0 on success, `EINVAL` or
    /// `ENOTSUP` before the device is reached, or the device's own error.
    /// May be called from user or kernel context.
    pub fn ldi_write(lh: ldi_handle_t, uiop: *mut crate::uio::uio, cr: *mut cred_t) -> c_int;
    /// `ldi_aread(9F)`. No interface level given.
    ///
    /// Passes an asynchronous read request described by the `aio_req(9S)`
    /// at `aio_reqp` to the `aread(9E)` entry point of the device behind
    /// `lh`. Supported for block and character devices. Returns 0 on
    /// success, `EINVAL` or `ENOTSUP` before the device is reached, or the
    /// device's own error. May be called from user context.
    pub fn ldi_aread(lh: ldi_handle_t, aio_reqp: *mut crate::bio::aio_req, cr: *mut cred_t) -> c_int;
    /// `ldi_awrite(9F)`. No interface level given.
    ///
    /// Passes an asynchronous write request described by the `aio_req(9S)`
    /// at `aio_reqp` to the `awrite(9E)` entry point of the device behind
    /// `lh`. Supported for block and character devices. Returns 0 on
    /// success, `EINVAL` or `ENOTSUP` before the device is reached, or the
    /// device's own error. May be called from user context.
    pub fn ldi_awrite(lh: ldi_handle_t, aio_reqp: *mut crate::bio::aio_req, cr: *mut cred_t) -> c_int;
    /// `ldi_strategy(9F)`. No interface level given.
    ///
    /// Passes the block transfer described by the `buf(9S)` at `bp` to the
    /// `strategy(9E)` entry point of the block device behind `lh`. Returns 0
    /// once the request has been handed to the device, or `EINVAL` or
    /// `ENOTSUP` if it could not be; any error that occurs afterwards is
    /// reported through the buffer by `bioerror(9F)` and `biodone(9F)`. May
    /// be called from user or kernel context.
    pub fn ldi_strategy(lh: ldi_handle_t, bp: *mut crate::bio::buf) -> c_int;
    /// `ldi_dump(9F)`. No interface level given.
    ///
    /// Passes a crash dump request to the `dump(9E)` entry point of the
    /// block device behind `lh`, asking it to write `nblk` blocks from the
    /// memory at `addr` starting at block `blkno`. Returns 0 on success,
    /// `EINVAL` or `ENOTSUP` before the device is reached, or the device's
    /// own error. May be called from user or kernel context.
    pub fn ldi_dump(lh: ldi_handle_t, addr: caddr_t, blkno: daddr_t, nblk: c_int) -> c_int;
    /// `ldi_poll(9F)`. No interface level given.
    ///
    /// Passes a poll request to the `chpoll(9E)` entry point of the device
    /// behind `lh`. `events` is the mask of `POLLIN`, `POLLOUT`, `POLLPRI`
    /// and related bits of interest, `anyyet` is non-zero if another
    /// descriptor in the same `poll(2)` already has events pending, the
    /// satisfied events are stored through `reventsp`, and a pollhead may be
    /// stored through `phpp` for the caller to wait on. Supported for block,
    /// character and STREAMS devices. Returns 0 on success or `EINVAL` or
    /// `ENOTSUP` before the device is reached. May be called from user or
    /// kernel context.
    pub fn ldi_poll(
        lh: ldi_handle_t,
        events: c_short,
        anyyet: c_int,
        reventsp: *mut c_short,
        phpp: *mut *mut crate::poll::pollhead,
    ) -> c_int;
    /// `ldi_ioctl(9F)`. No interface level given.
    ///
    /// Passes the ioctl `cmd` with argument `arg` to the `ioctl(9E)` entry
    /// point of the device behind `lh`. `mode` carries the open flags plus
    /// `FKIOCTL` when `arg` points into kernel memory, which tells the
    /// target that no data model conversion is needed; a caller forwarding
    /// a user process's ioctl must instead pass the original `mode` so the
    /// `FMODELS` bits reach the target. The device may store a return value
    /// through `rvalp`, valid only if the call succeeds. Supported for
    /// block, character and STREAMS devices; the STREAMS `I_PLINK` and
    /// `I_UNPLINK` ioctls are supported with layered handles in place of
    /// file descriptors. Returns 0 on success, `EINVAL` or `ENOTSUP` before
    /// the device is reached, or the device's own error. May be called from
    /// user or kernel context.
    pub fn ldi_ioctl(
        lh: ldi_handle_t,
        cmd: c_int,
        arg: intptr_t,
        mode: c_int,
        cr: *mut cred_t,
        rvalp: *mut c_int,
    ) -> c_int;
    /// `ldi_devmap(9F)`. No interface level given.
    ///
    /// Passes a devmap request to the `devmap(9E)` entry point of the
    /// character device behind `lh`. `dhp` is the mapping handle, `off` the
    /// user offset and `len` the requested length; the device stores the
    /// length it validated, at most `len`, through `maplen`. `model` is the
    /// data model of the current thread. Returns 0 on success, `EINVAL` or
    /// `ENOTSUP` before the device is reached, or the device's own error.
    /// May be called from user or kernel context.
    pub fn ldi_devmap(
        lh: ldi_handle_t,
        dhp: devmap_cookie_t,
        off: offset_t,
        len: size_t,
        maplen: *mut size_t,
        model: uint_t,
    ) -> c_int;
    /// `ldi_putmsg(9F)`. No interface level given.
    ///
    /// Sends the message block `smp` down the STREAMS device behind `lh`.
    /// Once passed in, the message belongs to the stream and the caller
    /// must not free it, even if the call fails. Returns 0 on success,
    /// `EINVAL` or `ENOTSUP` before the device is reached, or the device's
    /// own error. May be called from user or kernel context.
    pub fn ldi_putmsg(lh: ldi_handle_t, smp: *mut crate::streams::mblk_t) -> c_int;
    /// `ldi_getmsg(9F)`. No interface level given.
    ///
    /// Receives a message block from the STREAMS device behind `lh` and
    /// stores it through `rmp`; the caller frees it with `freemsg(9F)`. If
    /// `timeo` is NULL the caller sleeps until a message arrives, otherwise
    /// it waits at most that long and gets `ETIME` if nothing arrived.
    /// Returns 0 on success, `EINVAL` or `ENOTSUP` before the device is
    /// reached, or the device's own error. May be called from user or
    /// kernel context.
    pub fn ldi_getmsg(
        lh: ldi_handle_t,
        rmp: *mut *mut crate::streams::mblk_t,
        timeo: *mut crate::time::timestruc_t,
    ) -> c_int;
    /// `ldi_get_size(9F)`. No interface level given.
    ///
    /// Determines the size in bytes of the device behind `lh`, using its
    /// properties, and stores it through `sizep`. Returns `DDI_SUCCESS` if
    /// the size was found and `DDI_FAILURE` if it could not be determined.
    /// May be called from user or kernel context.
    pub fn ldi_get_size(lh: ldi_handle_t, sizep: *mut u64) -> c_int;

    /// `ldi_get_dev(9F)`. No interface level given.
    ///
    /// Stores the device number of the device behind `lh` through `devp`.
    /// Returns 0 on success, or `EINVAL` or `ENOTSUP`. May be called from
    /// user or kernel context.
    pub fn ldi_get_dev(lh: ldi_handle_t, devp: *mut dev_t) -> c_int;
    /// `ldi_get_otyp(9F)`. No interface level given.
    ///
    /// Stores the open type, `OTYP_BLK` or `OTYP_CHR`, that the device
    /// behind `lh` was opened with through `otyp`. Returns 0 on success, or
    /// `EINVAL` or `ENOTSUP`. May be called from user or kernel context.
    pub fn ldi_get_otyp(lh: ldi_handle_t, otyp: *mut c_int) -> c_int;
    /// `ldi_get_devid(9F)`. No interface level given.
    ///
    /// Stores the device identifier of the device behind `lh` through
    /// `devid`. The caller frees it with `ddi_devid_free(9F)`. Returns 0 on
    /// success, or `EINVAL` or `ENOTSUP` if the device has no identifier.
    /// May be called from user or kernel context.
    pub fn ldi_get_devid(lh: ldi_handle_t, devid: *mut crate::devid::ddi_devid_t) -> c_int;
    /// `ldi_get_minor_name(9F)`. No interface level given.
    ///
    /// Allocates a buffer holding the name of the minor node that `lh`
    /// opened and stores it through `minor_name`. The caller frees the
    /// buffer with `kmem_free(9F)`, passing the string length plus one.
    /// Returns 0 on success, or `EINVAL` or `ENOTSUP`. May be called from
    /// user or kernel context.
    pub fn ldi_get_minor_name(lh: ldi_handle_t, minor_name: *mut *mut c_char) -> c_int;

    /// `ldi_prop_exists(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether a property called `name`, of any type, is associated
    /// with the device behind `lh`. The search covers the driver's own
    /// software properties, those created by the system, the driver's
    /// global properties, the PROM properties unless `flags` contains
    /// `DDI_PROP_NOTPROM`, and then the parent node unless `flags` contains
    /// `DDI_PROP_DONTPASS`. The device number of the handle is part of the
    /// match unless `flags` contains `LDI_DEV_T_ANY`. Returns 1 if the
    /// property exists and 0 otherwise. May be called from user or kernel
    /// context.
    pub fn ldi_prop_exists(lh: ldi_handle_t, flags: uint_t, name: *mut c_char) -> c_int;
    /// `ldi_prop_get_int(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Looks up the integer property `name` of the device behind `lh`,
    /// searching in the same order and under the same `flags` as
    /// [`ldi_prop_exists`], and returns its value. If the property is not
    /// found, `defvalue` is returned; if it is found but cannot be decoded
    /// as an `int`, `DDI_PROP_NOT_FOUND` is returned. May be called from
    /// user or kernel context.
    pub fn ldi_prop_get_int(lh: ldi_handle_t, flags: uint_t, name: *mut c_char, defvalue: c_int) -> c_int;
    /// `ldi_prop_get_int64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`ldi_prop_get_int`] for a 64-bit integer property, except that
    /// the PROM is never searched for 64-bit values. Returns the value,
    /// `defvalue` if the property is not found, or `DDI_PROP_NOT_FOUND` if
    /// it cannot be decoded. May be called from user or kernel context.
    pub fn ldi_prop_get_int64(lh: ldi_handle_t, flags: uint_t, name: *mut c_char, defvalue: i64) -> i64;
    /// `ldi_prop_lookup_int_array(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Looks up the property `name` of the device behind `lh` as an array
    /// of 4-byte integers, searching in the same order and under the same
    /// `flags` as [`ldi_prop_exists`]. On success it allocates memory for
    /// the value, stores its address through `data` and the element count
    /// through `nelements`, and returns `DDI_PROP_SUCCESS`; the caller frees
    /// the value with `ddi_prop_free(9F)`. Otherwise it returns
    /// `DDI_PROP_INVAL_ARG` for a NULL handle or an empty name,
    /// `DDI_PROP_NOT_FOUND`, `DDI_PROP_UNDEFINED` if the property was
    /// explicitly undefined, or `DDI_PROP_CANNOT_DECODE`. May block to
    /// allocate memory; may be called from user or kernel context.
    pub fn ldi_prop_lookup_int_array(
        lh: ldi_handle_t,
        flags: uint_t,
        name: *mut c_char,
        data: *mut *mut c_int,
        nelements: *mut uint_t,
    ) -> c_int;
    /// `ldi_prop_lookup_int64_array(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`ldi_prop_lookup_int_array`] for an array of 8-byte integers,
    /// except that the PROM is never searched. Same return values and
    /// freeing rule. May be called from user or kernel context.
    pub fn ldi_prop_lookup_int64_array(
        lh: ldi_handle_t,
        flags: uint_t,
        name: *mut c_char,
        data: *mut *mut i64,
        nelements: *mut uint_t,
    ) -> c_int;
    /// `ldi_prop_lookup_string_array(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`ldi_prop_lookup_int_array`] for an array of strings. The value
    /// stored through `data` is an array of pointers to NUL-terminated
    /// strings, laid out like the `argv` of `execve(2)`, and is freed as a
    /// whole with `ddi_prop_free(9F)`. Same return values. May be called
    /// from user or kernel context.
    pub fn ldi_prop_lookup_string_array(
        lh: ldi_handle_t,
        flags: uint_t,
        name: *mut c_char,
        data: *mut *mut *mut c_char,
        nelements: *mut uint_t,
    ) -> c_int;
    /// `ldi_prop_lookup_string(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`ldi_prop_lookup_int_array`] for a single NUL-terminated
    /// string, stored through `data` and freed with `ddi_prop_free(9F)`.
    /// Same return values. May be called from user or kernel context.
    pub fn ldi_prop_lookup_string(lh: ldi_handle_t, flags: uint_t, name: *mut c_char, data: *mut *mut c_char) -> c_int;
    /// `ldi_prop_lookup_byte_array(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Like [`ldi_prop_lookup_int_array`] for an array of bytes, stored
    /// through `data` with the byte count through `nelements` and freed
    /// with `ddi_prop_free(9F)`. Same return values. May be called from user
    /// or kernel context.
    pub fn ldi_prop_lookup_byte_array(
        lh: ldi_handle_t,
        flags: uint_t,
        name: *mut c_char,
        data: *mut *mut c_uchar,
        nelements: *mut uint_t,
    ) -> c_int;

    /// `ldi_ev_get_cookie(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Looks up the event named `evname`, such as `LDI_EV_OFFLINE` or
    /// `LDI_EV_DEGRADE`, for the device behind `lh` and stores an opaque
    /// cookie for it through `cookiep`. The name of any event defined for
    /// the NDI event service is also accepted, which is how this function
    /// replaces the obsolete `ldi_get_eventcookie(9F)`. The cookie is
    /// needed to register callbacks. Returns `LDI_EV_SUCCESS` or
    /// `LDI_EV_FAILURE`. May be called from user or kernel context only.
    pub fn ldi_ev_get_cookie(lh: ldi_handle_t, evname: *mut c_char, cookiep: *mut ldi_ev_cookie_t) -> c_int;
    /// `ldi_ev_get_type(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the event name string that `cookie` represents, or NULL on
    /// failure. The manual page's synopsis shows an `int` return; the
    /// header, followed here, returns `char *`, and the description agrees.
    /// May be called from user or kernel context only.
    pub fn ldi_ev_get_type(cookie: ldi_ev_cookie_t) -> *mut c_char;
    /// `ldi_ev_register_callbacks(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Registers the notify and finalize callbacks in `callb`, whose
    /// `cb_vers` must be `LDI_EV_CB_VERS`, for the event `cookie` on the
    /// device behind `lh`. `arg` is passed unchanged to both callbacks. On
    /// success the function returns `LDI_EV_SUCCESS` and stores a
    /// registration identifier through `id`, which is later given to
    /// [`ldi_ev_remove_callbacks`]; otherwise it returns `LDI_EV_FAILURE`.
    /// The callbacks stay registered until removed, even after the handle
    /// is closed. The manual page's synopsis shows `cookie` as a pointer;
    /// the header, followed here, takes the cookie by value, and the
    /// page's own example agrees. May be called from user or kernel context
    /// only.
    pub fn ldi_ev_register_callbacks(
        lh: ldi_handle_t,
        cookie: ldi_ev_cookie_t,
        callb: *mut ldi_ev_callback_t,
        arg: *mut c_void,
        id: *mut ldi_callback_id_t,
    ) -> c_int;
    /// `ldi_ev_remove_callbacks(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unregisters the callbacks registered under `id` and invalidates the
    /// identifier. A consumer must do this before the entity that
    /// registered them, whether a device node, a device number or a module,
    /// goes away, or the framework may panic. Returns `LDI_EV_SUCCESS`, or
    /// `LDI_EV_FAILURE` if `id` is NULL or the underlying
    /// `ddi_remove_event_handler(9F)` fails. May be called from user or
    /// kernel context only.
    pub fn ldi_ev_remove_callbacks(id: ldi_callback_id_t) -> c_int;
    /// `ldi_ev_notify(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Propagates an impending event up the software stack from a layered
    /// driver to the consumers of the minor node it exports, identified by
    /// the driver's node `dip`, the minor number `minor` and `spec_type` of
    /// `S_IFCHR` or `S_IFBLK`. It invokes the LDI notify callbacks
    /// registered against that minor and generates device contract events
    /// for user processes. `cookie` names the event being propagated, which
    /// may differ from the event the driver itself received, and `ev_data`
    /// is passed along. A layered driver calls it from its notify callback
    /// before deciding whether it can permit the event itself. Returns
    /// `LDI_EV_SUCCESS` if the consumers permit the change and
    /// `LDI_EV_FAILURE` if one of them blocks it, in which case the caller
    /// must return `LDI_EV_FAILURE` from its own callback. May be called
    /// from user or kernel context only.
    pub fn ldi_ev_notify(
        dip: dev_info_t,
        minor: minor_t,
        spec_type: c_int,
        cookie: ldi_ev_cookie_t,
        ev_data: *mut c_void,
    ) -> c_int;
    /// `ldi_ev_finalize(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Propagates the outcome of an event up the software stack from a
    /// layered driver to the consumers of the minor node it exports,
    /// identified as for [`ldi_ev_notify`]. `ldi_result` is `LDI_EV_SUCCESS`
    /// if the state change happened and `LDI_EV_FAILURE` if it did not, and
    /// `cookie` and `ev_data` describe the event. It invokes the finalize
    /// callbacks registered against the minor and generates device contract
    /// events. A layered driver calls it from its finalize callback once it
    /// has reconfigured itself, and also from its notify callback with
    /// `LDI_EV_FAILURE` when it is the first layer to block an event, since
    /// the framework only generates finalize calls for failures it detects
    /// itself. May be called from user or kernel context only.
    pub fn ldi_ev_finalize(
        dip: dev_info_t,
        minor: minor_t,
        spec_type: c_int,
        ldi_result: c_int,
        cookie: ldi_ev_cookie_t,
        ev_data: *mut c_void,
    );
}
