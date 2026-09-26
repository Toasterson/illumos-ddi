//! The GLD service routines of `gld(9F)`, and the driver entry points that
//! GLD implements on the driver's behalf according to `gld(4D)`.
//!
//! The first group is what a GLDv2 driver calls. It allocates, registers,
//! unregisters and frees its `gld_mac_info(9S)` structure, hands received
//! packets to GLD, tells GLD to retry stalled transmissions, reports link
//! state changes, and may use GLD's interrupt handler.
//!
//! The second group is what a GLDv2 driver installs but does not call.
//! `gld(4D)` states that GLD implements `open(9E)`, `close(9E)`, the
//! STREAMS `put(9E)` and `srv(9E)` routines and `getinfo(9E)` for the
//! driver, and says which slots of the driver's `qinit(9S)` and
//! `dev_ops(9S)` structures must name them. These functions are declared in
//! `sys/gld.h` and exported by `misc/gld` for exactly that purpose. No
//! section 9 page documents them, so their doc comments cite `gld(4D)`,
//! whose interface level is the same as that of the section 9 pages. Their
//! signatures match the `qi_putp_t`, `qi_srvp_t`, `qi_qopen_t`,
//! `qi_qclose_t` and `devo_getinfo_t` types of `ddi-sys`, so they can be
//! stored in those slots directly.

use ddi_sys::{c_char, c_int, c_void, caddr_t, cred_t, ddi_info_cmd_t, dev_info_t, dev_t, mblk_t, queue_t, uint_t};

use crate::mac_info::gld_mac_info_t;

unsafe extern "C" {
    // --- gld(9F) ---------------------------------------------------------

    /// `gld_mac_alloc(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Allocates a new `gld_mac_info(9S)` structure for the device `dip` and
    /// returns a pointer to it. Some of the members private to GLD may be
    /// initialised before the function returns, and all other members are
    /// set to zero. The driver must then initialise the members that
    /// `gld_mac_info(9S)` describes before it passes the structure to
    /// [`gld_register`]. It is called from `attach(9E)`.
    pub fn gld_mac_alloc(dip: dev_info_t) -> *mut gld_mac_info_t;

    /// `gld_mac_free(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Frees the `gld_mac_info(9S)` structure `macinfo` that
    /// [`gld_mac_alloc`] allocated. A driver calls it from `detach(9E)`
    /// after [`gld_unregister`] has succeeded, or from `attach(9E)` when
    /// [`gld_register`] has failed.
    pub fn gld_mac_free(macinfo: *mut gld_mac_info_t);

    /// `gld_register(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Links the device-specific driver instance `dip` with the GLD
    /// framework. It is called from `attach(9E)` once the driver has
    /// allocated `macinfo` with [`gld_mac_alloc`] and initialised it as
    /// `gld_mac_info(9S)` describes, and once the device has been reset,
    /// its registers mapped and its interrupts added but not enabled. `name`
    /// is the device interface name and must be exactly the name of the
    /// driver module as it exists in the file system.
    ///
    /// On success the function links the driver with GLD, sets the driver's
    /// private data pointer with `ddi_set_driver_private(9F)` to point to
    /// `macinfo`, and creates the minor device node. From then on GLD may
    /// call the driver's `gld(9E)` entry points at any time. It returns
    /// `DDI_SUCCESS`, in which case `attach(9E)` should return `DDI_SUCCESS`
    /// as well, or `DDI_FAILURE`, in which case `attach(9E)` should release
    /// what it allocated and return `DDI_FAILURE`.
    pub fn gld_register(dip: dev_info_t, name: *mut c_char, macinfo: *mut gld_mac_info_t) -> c_int;

    /// `gld_unregister(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Unlinks the driver instance described by `macinfo` from GLD. It is
    /// called from `detach(9E)`. On success it makes sure the device's
    /// interrupts are stopped, calling the driver's `gldm_stop` routine if
    /// necessary, removes the minor device node, and unlinks the driver
    /// from GLD. It returns `DDI_SUCCESS`, after which `detach(9E)` should
    /// release the data structures allocated in `attach(9E)`, free
    /// `macinfo` with [`gld_mac_free`] and return `DDI_SUCCESS`, or
    /// `DDI_FAILURE`, after which `detach(9E)` must leave the device
    /// operational and return `DDI_FAILURE`.
    pub fn gld_unregister(macinfo: *mut gld_mac_info_t) -> c_int;

    /// `gld_recv(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Passes a received packet upstream. The driver's interrupt handler
    /// calls it with its `gld_mac_info(9S)` structure `macinfo` and a STREAMS
    /// `M_DATA` message `mp` that holds the raw packet, with `b_rptr` and
    /// `b_wptr` set correctly. GLD decides which streams, if any, should
    /// receive a copy of the packet, duplicates it where necessary, formats
    /// a `DL_UNITDATA_IND` message if required, and passes the data up.
    ///
    /// The driver should not hold a mutex or other lock during the call. In
    /// particular it must not hold a lock that a transmit thread could take,
    /// because the interrupt thread may in some cases go on to send an
    /// outgoing packet and so call the driver's `gldm_send` routine, and a
    /// `gldm_send` that tried to take a mutex held by `gldm_intr` would panic
    /// the system with a recursive mutex entry.
    pub fn gld_recv(macinfo: *mut gld_mac_info_t, mp: *mut mblk_t);

    /// `gld_sched(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Tells GLD to retry the outbound packets that stalled. Whenever the
    /// driver's `gldm_send` routine has returned `GLD_NORESOURCES`, the
    /// driver must later call this function for its `gld_mac_info(9S)`
    /// structure `macinfo`, as soon as possible after resources are
    /// available again, so that GLD resumes passing packets to `gldm_send`
    /// in a timely way. A call of the driver's `gldm_stop` routine absolves
    /// the driver from this obligation until `gldm_send` next returns
    /// `GLD_NORESOURCES`, and extra calls do no harm.
    pub fn gld_sched(macinfo: *mut gld_mac_info_t);

    /// `gld_intr(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// GLD's main interrupt handler. A driver normally installs it as its
    /// interrupt routine with `ddi_add_intr(9F)`, and its argument must then
    /// be a pointer to the driver's `gld_mac_info(9S)` structure. When
    /// appropriate, it calls the driver's `gldm_intr` routine with that
    /// pointer. A driver that uses a high-level interrupt must provide its
    /// own high-level handler that triggers a soft interrupt, and may then
    /// install this function as the soft interrupt handler with
    /// `ddi_add_softintr`. It returns a value appropriate for an interrupt
    /// handler, which is whatever `gldm_intr` returns.
    ///
    /// The manual page gives the argument as `caddr_t`, the header declares
    /// the function without a prototype, and `io/gld.c` defines it with a
    /// `gld_mac_info_t` pointer. The declaration follows the manual. The
    /// function takes one argument, so it does not have the type of the
    /// two-argument `ddi_intr_handler_t` that `ddi_intr_add_handler(9F)`
    /// expects, and a driver using the newer interrupt functions calls it
    /// from its own handler instead.
    pub fn gld_intr(macinfo: caddr_t) -> uint_t;

    /// `gld_linkstate(9F)`. illumos architecture specific (illumos DDI).
    ///
    /// Tells GLD that the media link state of the device described by
    /// `macinfo` changed. `newstate` is `GLD_LINKSTATE_DOWN` if the link is
    /// unavailable, `GLD_LINKSTATE_UP` if it is up, or
    /// `GLD_LINKSTATE_UNKNOWN` if its state is unknown. A driver that calls
    /// this function must also set `GLD_CAP_LINKSTATE` in the
    /// `gldm_capabilities` member of its `gld_mac_info(9S)` structure.
    pub fn gld_linkstate(macinfo: *mut gld_mac_info_t, newstate: i32);

    // --- entry points GLD implements for the driver, gld(4D) --------------

    /// `gld(4D)`. illumos architecture specific (illumos DDI).
    ///
    /// GLD's implementation of `getinfo(9E)`. A GLDv2 driver names it in the
    /// `devo_getinfo` slot of its `dev_ops(9S)` structure and never calls it
    /// itself. It translates the device number in `arg` to the device's
    /// `dev_info` node or instance number, as `cmd` asks, and stores the
    /// answer in `resultp`. It returns `DDI_SUCCESS` or `DDI_FAILURE`.
    pub fn gld_getinfo(dip: dev_info_t, cmd: ddi_info_cmd_t, arg: *mut c_void, resultp: *mut *mut c_void) -> c_int;

    /// `gld(4D)`. illumos architecture specific (illumos DDI).
    ///
    /// GLD's implementation of the STREAMS `open(9E)` routine. A GLDv2
    /// driver names it in the `qi_qopen` slot of its read side `qinit(9S)`
    /// structure and never calls it itself. `q` is the read queue of the new
    /// stream, `dev` the device number, which GLD changes for a clone open,
    /// `flag` the open flags, `sflag` the STREAMS open flag and `cred` the
    /// caller's credentials. It returns 0 or an error number.
    pub fn gld_open(q: *mut queue_t, dev: *mut dev_t, flag: c_int, sflag: c_int, cred: *mut cred_t) -> c_int;

    /// `gld(4D)`. illumos architecture specific (illumos DDI).
    ///
    /// GLD's implementation of the STREAMS `close(9E)` routine. A GLDv2
    /// driver names it in the `qi_qclose` slot of its read side `qinit(9S)`
    /// structure and never calls it itself. `q` is the read queue of the
    /// stream being closed, `flag` the open flags and `cred` the caller's
    /// credentials. It returns 0.
    pub fn gld_close(q: *mut queue_t, flag: c_int, cred: *mut cred_t) -> c_int;

    /// `gld(4D)`. illumos architecture specific (illumos DDI).
    ///
    /// GLD's implementation of the write side STREAMS `put(9E)` routine. A
    /// GLDv2 driver names it in the `qi_putp` slot of its write side
    /// `qinit(9S)` structure and never calls it itself. It handles the
    /// message `mp` arriving on the write queue `q`, passing data to the
    /// driver's `gldm_send` routine or queueing it for [`gld_wsrv`].
    pub fn gld_wput(q: *mut queue_t, mp: *mut mblk_t) -> c_int;

    /// `gld(4D)`. illumos architecture specific (illumos DDI).
    ///
    /// GLD's implementation of the write side STREAMS `srv(9E)` routine. A
    /// GLDv2 driver names it in the `qi_srvp` slot of its write side
    /// `qinit(9S)` structure and never calls it itself. It processes the
    /// messages queued on the write queue `q`, including the DLPI requests
    /// and the packets that `gldm_send` could not take at once.
    pub fn gld_wsrv(q: *mut queue_t) -> c_int;

    /// `gld(4D)`. illumos architecture specific (illumos DDI).
    ///
    /// GLD's implementation of the read side STREAMS `srv(9E)` routine. A
    /// GLDv2 driver names it in the `qi_srvp` slot of its read side
    /// `qinit(9S)` structure, whose `qi_putp` slot stays NULL, and never
    /// calls it itself. It passes the messages queued on the read queue `q`
    /// upstream as flow control allows.
    pub fn gld_rsrv(q: *mut queue_t) -> c_int;
}
