//! The x86 ISA DMA engine, `ddi_dmae(9F)`, from `uts/intel/sys/dma_engine.h`.
//!
//! Before bus-mastering devices were the norm, ISA devices moved data
//! through the two 8237 DMA controllers on the motherboard. A driver
//! reserves a channel, describes the transfer in a `ddi_dmae_req(9S)`,
//! hands the controller a cookie obtained through the ordinary DMA binding
//! functions, and starts the transfer. Only drivers for legacy devices such
//! as the floppy controller and some sound cards need this. The header
//! lives under `uts/intel`, so the module only exists on x86.

use crate::dma::{ddi_dma_attr_t, ddi_dma_cookie_t};
use crate::types::*;

/// `ddi_dmae_req(9S)`. illumos x86 DDI specific (illumos x86 DDI).
///
/// Describes a transfer to the ISA DMA engine. Most fields take one of the
/// `DMAE_*` constants below; zero is the default for each of them.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ddi_dmae_req {
    /// The device node of the original requester.
    pub der_rdip: dev_info_t,
    /// The transfer direction, one of the `DMAE_CMD_*` values.
    pub der_command: c_uchar,
    /// Whether the transfer is single, chained or auto-initialised, one of
    /// the `DMAE_BUF_*` values.
    pub der_bufprocess: c_uchar,
    /// How the address advances, one of the `DMAE_STEP_*` values.
    pub der_step: c_uchar,
    /// The transfer mode, one of the `DMAE_TRANS_*` values.
    pub der_trans: c_uchar,
    /// The transfer width, one of the `DMAE_PATH_*` values.
    pub der_path: c_uchar,
    /// The number of bus cycles per transfer on EISA, 1 or 2.
    pub der_cycles: c_uchar,
    /// Whether the other side of the transfer is memory or I/O space.
    pub der_dest: c_uchar,
    /// The MicroChannel arbitration register value.
    pub der_arbus: c_uchar,
    /// The MicroChannel I/O address register value.
    pub der_ioadr: c_ushort,
    /// A function that supplies the next cookie of a chained transfer, or
    /// NULL. The C declaration has no parameter list, so the pointer is
    /// declared without arguments here as well.
    pub proc_: Option<unsafe extern "C" fn() -> *mut ddi_dma_cookie_t>,
    /// The argument buffer for `proc_`.
    pub procparms: *mut c_void,
}

// A pointer at 0, eight bytes of uchar at 8..16, a ushort at 16, padding to
// the pointer at 24, and the last pointer at 32.
const _: () = assert!(core::mem::size_of::<ddi_dmae_req>() == 40);
const _: () = assert!(core::mem::offset_of!(ddi_dmae_req, der_command) == 8);
const _: () = assert!(core::mem::offset_of!(ddi_dmae_req, der_ioadr) == 16);
const _: () = assert!(core::mem::offset_of!(ddi_dmae_req, proc_) == 24);
const _: () = assert!(core::mem::offset_of!(ddi_dmae_req, procparms) == 32);

/// Verify the transfer without moving data (`DMAE_CMD_VRFY`).
pub const DMAE_CMD_VRFY: c_uchar = 0;
/// Transfer from memory to the device (`DMAE_CMD_WRITE`).
pub const DMAE_CMD_WRITE: c_uchar = 1;
/// Transfer from the device to memory (`DMAE_CMD_READ`).
pub const DMAE_CMD_READ: c_uchar = 2;
/// Translate only (`DMAE_CMD_TRAN`).
pub const DMAE_CMD_TRAN: c_uchar = 3;
/// A single transfer that is not reinitialised (`DMAE_BUF_NOAUTO`).
pub const DMAE_BUF_NOAUTO: c_uchar = 0;
/// A chained transfer that asks `proc_` for more cookies (`DMAE_BUF_CHAIN`).
pub const DMAE_BUF_CHAIN: c_uchar = 0x1;
/// An auto-initialising transfer that restarts at the end (`DMAE_BUF_AUTO`).
pub const DMAE_BUF_AUTO: c_uchar = 0x2;
/// The address increments after each transfer (`DMAE_STEP_INC`).
pub const DMAE_STEP_INC: c_uchar = 0;
/// The address decrements after each transfer (`DMAE_STEP_DEC`).
pub const DMAE_STEP_DEC: c_uchar = 1;
/// The address stays the same (`DMAE_STEP_HOLD`).
pub const DMAE_STEP_HOLD: c_uchar = 2;
/// Single transfer mode (`DMAE_TRANS_SNGL`).
pub const DMAE_TRANS_SNGL: c_uchar = 0;
/// Block transfer mode (`DMAE_TRANS_BLCK`).
pub const DMAE_TRANS_BLCK: c_uchar = 1;
/// Demand transfer mode (`DMAE_TRANS_DMND`).
pub const DMAE_TRANS_DMND: c_uchar = 2;
/// Cascade mode, for chaining a second controller (`DMAE_TRANS_CSCD`).
pub const DMAE_TRANS_CSCD: c_uchar = 3;
/// The default transfer width of the channel (`DMAE_PATH_DEF`).
pub const DMAE_PATH_DEF: c_uchar = 0;
/// An 8-bit transfer, the ISA default for channels 0 to 3 (`DMAE_PATH_8`).
pub const DMAE_PATH_8: c_uchar = 1;
/// A 16-bit transfer, the ISA default for channels 5 to 7 (`DMAE_PATH_16`).
pub const DMAE_PATH_16: c_uchar = 2;
/// A 32-bit transfer, EISA only (`DMAE_PATH_32`).
pub const DMAE_PATH_32: c_uchar = 3;

/// The callback that [`ddi_dmae_alloc`] calls from interrupt context when a
/// channel may have become free. It returns one of the
/// `DDI_DMA_CALLBACK_*` values.
pub type ddi_dmae_callback_t = unsafe extern "C" fn(arg: caddr_t) -> c_int;

unsafe extern "C" {
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reserves ISA DMA channel `chnl` for the device. `callback` is either
    /// a function, `DDI_DMA_SLEEP` or `DDI_DMA_DONTWAIT`, cast to this
    /// pointer type, with the same meaning as for the other DMA allocation
    /// functions. Returns `DDI_SUCCESS`, `DDI_DMA_NORESOURCES` or
    /// `DDI_FAILURE`.
    pub fn ddi_dmae_alloc(dip: dev_info_t, chnl: c_int, callback: Option<ddi_dmae_callback_t>, arg: caddr_t) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Releases a channel reserved with [`ddi_dmae_alloc`].
    pub fn ddi_dmae_release(dip: dev_info_t, chnl: c_int) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Programs the channel with the request and the first cookie and arms
    /// it. The transfer starts when the device asserts its DMA request line.
    pub fn ddi_dmae_prog(
        dip: dev_info_t,
        dmaereqp: *mut ddi_dmae_req,
        cookiep: *mut ddi_dma_cookie_t,
        chnl: c_int,
    ) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Programs the channel like [`ddi_dmae_prog`] but for a transfer that
    /// software starts with [`ddi_dmae_swstart`].
    pub fn ddi_dmae_swsetup(
        dip: dev_info_t,
        dmaereqp: *mut ddi_dmae_req,
        cookiep: *mut ddi_dma_cookie_t,
        chnl: c_int,
    ) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Starts a transfer prepared with [`ddi_dmae_swsetup`].
    pub fn ddi_dmae_swstart(dip: dev_info_t, chnl: c_int) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stops a transfer on the channel.
    pub fn ddi_dmae_stop(dip: dev_info_t, chnl: c_int) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Unmasks the channel so that device requests are honoured.
    pub fn ddi_dmae_enable(dip: dev_info_t, chnl: c_int) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Masks the channel so that device requests are ignored.
    pub fn ddi_dmae_disable(dip: dev_info_t, chnl: c_int) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Stores the number of bytes that remain to be transferred on the
    /// channel through `countp`.
    pub fn ddi_dmae_getcnt(dip: dev_info_t, chnl: c_int, countp: *mut c_int) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Puts the channel into cascade mode, for a device that is itself a
    /// bus master on the ISA bus.
    pub fn ddi_dmae_1stparty(dip: dev_info_t, chnl: c_int) -> c_int;
    /// `ddi_dmae(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Fills in the DMA attributes that describe what the ISA DMA engine can
    /// address, for use with `ddi_dma_alloc_handle(9F)`.
    pub fn ddi_dmae_getattr(dip: dev_info_t, attrp: *mut ddi_dma_attr_t) -> c_int;
}
