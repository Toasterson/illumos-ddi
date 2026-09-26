//! The signatures of the `gld(9E)` entry points.
//!
//! A GLDv2 driver implements these routines and stores pointers to them in
//! the `gldm_*` members of its `gld_mac_info(9S)` structure in `attach(9E)`,
//! before it calls `gld_register(9F)`. The header declares the members as
//! unprototyped `int (*)()` pointers, so the prototypes here are taken from
//! the SYNOPSIS of `gld(9E)`, where each routine is written with a driver
//! chosen `prefix_` in place of `gldm_`. Once registration has succeeded,
//! GLD may call any of them at any time.
//!
//! The header has two more members that no page documents, `gldm_mctl` and
//! `gldm_send_tagged`. Their prototypes are taken from the calls in
//! `io/gld.c`, and they are declared so that the structure can be filled in
//! completely, but a driver has no documented contract for them and should
//! leave them NULL.

use ddi_sys::{c_int, c_uchar, mblk_t, queue_t, uint_t};

use crate::mac_info::gld_mac_info_t;
use crate::stats::gld_stats;

/// `gldm_reset(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that resets the hardware to its initial
/// state. `macinfo` is the driver's `gld_mac_info(9S)` structure. GLD calls
/// it before it programs the station address and starts the device. The
/// routine returns [`GLD_SUCCESS`](crate::GLD_SUCCESS) on success or
/// [`GLD_FAILURE`](crate::GLD_FAILURE) if the hardware failed.
pub type gldm_reset_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t) -> c_int;

/// `gldm_start(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that enables the device to generate
/// interrupts and prepares the driver to call `gld_recv(9F)` to deliver
/// received packets to GLD. `macinfo` is the driver's `gld_mac_info(9S)`
/// structure. The routine returns [`GLD_SUCCESS`](crate::GLD_SUCCESS) on
/// success or [`GLD_FAILURE`](crate::GLD_FAILURE) if the hardware failed.
pub type gldm_start_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t) -> c_int;

/// `gldm_stop(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that stops the device from generating
/// any interrupts and stops the driver from calling `gld_recv(9F)`.
/// `macinfo` is the driver's `gld_mac_info(9S)` structure. GLD depends on
/// this routine to make sure that the device will no longer interrupt, and
/// the routine must do so without fail. It should always return
/// [`GLD_SUCCESS`](crate::GLD_SUCCESS). Once it has been called, a driver
/// that earlier returned [`GLD_NORESOURCES`](crate::GLD_NORESOURCES) from
/// `gldm_send` no longer owes GLD a call to `gld_sched(9F)`.
pub type gldm_stop_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t) -> c_int;

/// `gldm_set_mac_addr(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that sets the physical address the
/// hardware uses for receiving data. `macinfo` is the driver's
/// `gld_mac_info(9S)` structure and `macaddr` points to a valid MAC address
/// of the length given in `gldm_addrlen`, which the routine should program
/// into the device. It returns [`GLD_SUCCESS`](crate::GLD_SUCCESS),
/// [`GLD_NORESOURCES`](crate::GLD_NORESOURCES),
/// [`GLD_NOTSUPPORTED`](crate::GLD_NOTSUPPORTED),
/// [`GLD_BADARG`](crate::GLD_BADARG) for an unsuitable address, or
/// [`GLD_FAILURE`](crate::GLD_FAILURE) if the hardware failed.
pub type gldm_set_mac_addr_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t, macaddr: *mut c_uchar) -> c_int;

/// `gldm_set_multicast(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that enables and disables reception of
/// a multicast, group or functional address at the device level. `macinfo`
/// is the driver's `gld_mac_info(9S)` structure and `multicastaddr` points
/// to an address of the length given in `gldm_addrlen`. If `multiflag` is
/// [`GLD_MULTI_ENABLE`](crate::GLD_MULTI_ENABLE), the device must receive
/// packets sent to that address, and if it is
/// [`GLD_MULTI_DISABLE`](crate::GLD_MULTI_DISABLE), the driver may stop
/// receiving them.
///
/// GLD keeps count of repeated requests for the same address and only calls
/// the routine for the first request to enable and the last request to
/// disable it, so the routine is never asked to enable an address that is
/// already enabled or to disable one that is not. A device may implement
/// the filter with a hash and a bit mask that lets through more addresses
/// than were asked for, because GLD discards the packets nobody wants. If
/// disabling one address could disable others at the device level, the
/// driver must keep enough state to avoid disabling an address that GLD
/// still has enabled.
///
/// The routine returns [`GLD_SUCCESS`](crate::GLD_SUCCESS),
/// [`GLD_NORESOURCES`](crate::GLD_NORESOURCES),
/// [`GLD_NOTSUPPORTED`](crate::GLD_NOTSUPPORTED),
/// [`GLD_BADARG`](crate::GLD_BADARG) for an unsuitable address, or
/// [`GLD_FAILURE`](crate::GLD_FAILURE) if the hardware failed.
pub type gldm_set_multicast_t =
    unsafe extern "C" fn(macinfo: *mut gld_mac_info_t, multicastaddr: *mut c_uchar, multiflag: c_int) -> c_int;

/// `gldm_set_promiscuous(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that enables and disables promiscuous
/// mode. `macinfo` is the driver's `gld_mac_info(9S)` structure.
/// If `promiscflag` is [`GLD_MAC_PROMISC_PHYS`](crate::GLD_MAC_PROMISC_PHYS),
/// the device must receive every packet on the medium. If it is
/// [`GLD_MAC_PROMISC_MULTI`](crate::GLD_MAC_PROMISC_MULTI), the device must
/// receive every multicast packet, and a device that has no multicast-only
/// promiscuous mode enters physical promiscuous mode instead and returns
/// success, leaving GLD to filter out the extra packets. If it is
/// [`GLD_MAC_PROMISC_NONE`](crate::GLD_MAC_PROMISC_NONE), promiscuous mode is
/// turned off. For forward compatibility the routine should treat any value
/// it does not recognise as `GLD_MAC_PROMISC_PHYS`.
///
/// The routine returns [`GLD_SUCCESS`](crate::GLD_SUCCESS),
/// [`GLD_NORESOURCES`](crate::GLD_NORESOURCES),
/// [`GLD_NOTSUPPORTED`](crate::GLD_NOTSUPPORTED),
/// [`GLD_BADARG`](crate::GLD_BADARG), or
/// [`GLD_FAILURE`](crate::GLD_FAILURE) if the hardware failed.
pub type gldm_set_promiscuous_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t, promiscflag: c_int) -> c_int;

/// `gldm_send(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that queues a packet to the device for
/// transmission. `macinfo` is the driver's `gld_mac_info(9S)` structure and
/// `mp` is a STREAMS message holding the packet. The message may consist of
/// several message blocks, including blocks of zero length, and the routine
/// must follow the whole chain. It should check that the packet does not
/// exceed the maximum packet size and must pad it to the minimum size if
/// necessary.
///
/// If the routine transmits or queues the packet it returns
/// [`GLD_SUCCESS`](crate::GLD_SUCCESS), and the driver then owns the message
/// and frees it once neither the driver nor the hardware needs it any more,
/// which for a device that reads the data blocks by DMA is only after the
/// transfer has completed. If the device cannot accept the packet at the
/// moment the routine returns [`GLD_NORESOURCES`](crate::GLD_NORESOURCES),
/// GLD retries later, and the driver must call `gld_sched(9F)` once
/// resources are available again. It returns
/// [`GLD_NOLINK`](crate::GLD_NOLINK) if there is no link partner,
/// [`GLD_BADARG`](crate::GLD_BADARG) for a bad packet or packet length, and
/// [`GLD_FAILURE`](crate::GLD_FAILURE) if the hardware failed. With any
/// return value other than `GLD_SUCCESS` the driver must not free the
/// message.
///
/// GLD may call this routine from the interrupt thread in which the driver
/// called `gld_recv(9F)`, so the driver must not hold a lock across
/// `gld_recv` that this routine also takes.
pub type gldm_send_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t, mp: *mut mblk_t) -> c_int;

/// `gldm_intr(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point called when the device might have
/// interrupted, normally from `gld_intr(9F)`. `macinfo` is the driver's
/// `gld_mac_info(9S)` structure. Because interrupts may be shared, the
/// routine must check the device status to find out whether its device
/// caused the interrupt. If not, it returns `DDI_INTR_UNCLAIMED`. Otherwise
/// it services the interrupt and returns `DDI_INTR_CLAIMED`. Each packet
/// received is put into an `M_DATA` message with correct `b_rptr` and
/// `b_wptr` and passed to `gld_recv(9F)`, and every error, including a
/// failure to allocate a receive buffer, should be counted in the
/// statistics. The driver should avoid holding locks while it calls
/// `gld_recv`.
pub type gldm_intr_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t) -> uint_t;

/// `gldm_get_stats(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the entry point that gathers statistics from the
/// hardware and the driver's private counters. `macinfo` is the driver's
/// `gld_mac_info(9S)` structure and `stats` points to the `gld_stats(9S)`
/// structure the routine updates. GLD calls it when it receives a request
/// for statistics, before it composes its reply. The routine returns
/// [`GLD_SUCCESS`](crate::GLD_SUCCESS) on success or
/// [`GLD_FAILURE`](crate::GLD_FAILURE) if the hardware failed.
pub type gldm_get_stats_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t, stats: *mut gld_stats) -> c_int;

/// `gldm_ioctl(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the optional entry point that implements device
/// specific ioctl commands. `macinfo` is the driver's `gld_mac_info(9S)`
/// structure, `q` is the `queue(9S)` to reply on and `mp` is the ioctl
/// message. The driver turns the message into an ioctl reply and passes it
/// to `qreply(9F)` before it returns. The routine should always return
/// [`GLD_SUCCESS`](crate::GLD_SUCCESS) and report any error in the reply
/// message instead. If the member is NULL, GLD answers every ioctl it does
/// not handle itself with an `M_IOCNAK` message carrying `EINVAL`.
pub type gldm_ioctl_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t, q: *mut queue_t, mp: *mut mblk_t) -> c_int;

/// `gld(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the undocumented `gldm_mctl` member. No manual page
/// describes it. When the write service routine of GLD finds an `M_CTL`
/// message on a stream, it calls this routine with the `gld_mac_info(9S)`
/// structure, the write queue and the message, while it holds its own lock
/// on the structure as a writer, and it ignores the return value. The
/// routine takes over the message. If the member is NULL, GLD frees such
/// messages instead. The prototype is taken from that call in `io/gld.c`.
pub type gldm_mctl_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t, q: *mut queue_t, mp: *mut mblk_t) -> c_int;

/// `gld(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of the undocumented `gldm_send_tagged` member. No manual
/// page describes it. If a driver sets it, GLD calls it instead of
/// `gldm_send` for every packet and passes the IEEE 802.1Q tag of the packet
/// in `vtag`, or 0 for an untagged packet, and the return values are those
/// of `gldm_send(9E)`. GLD in fact calls `gldm_send` with the same three
/// arguments and relies on the calling convention to drop the third. The
/// prototype is taken from that call in `io/gld.c`.
pub type gldm_send_tagged_t = unsafe extern "C" fn(macinfo: *mut gld_mac_info_t, mp: *mut mblk_t, vtag: u32) -> c_int;
