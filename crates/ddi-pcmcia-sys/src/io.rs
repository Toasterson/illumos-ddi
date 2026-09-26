//! I/O ranges and interrupts from `sys/cs.h` and `sys/cs_types.h`.
//!
//! A card that decodes I/O ports asks for up to two port ranges with
//! [`csx_RequestIO`], and a card that raises interrupts asks for an
//! interrupt and registers its handler with [`csx_RequestIRQ`]. Neither
//! resource is live until `csx_RequestConfiguration(9F)` applies a
//! configuration, and the interrupt handler is not called until a
//! configuration is requested with `CONF_ENABLE_IRQ_STEERING`. The
//! resources are given back with [`csx_ReleaseIO`] and [`csx_ReleaseIRQ`]
//! after the configuration has been released.
//!
//! The data width bits of an I/O range are the only I/O attribute bits
//! declared here; the byte order and ordering bits are the `WIN_ACC_*`
//! values in [`crate::window`]. The header also defines `IO_SHARED`,
//! `IO_FIRST_SHARED`, `IO_FORCE_ALIAS_ACCESS`, `IRQ_FORCED_PULSE`,
//! `IRQ_TYPE_TIME`, `IRQ_TYPE_DYNAMIC_SHARING`, `IRQ_FIRST_SHARED` and
//! `IRQ_PULSE_ALLOCATED`, which exist for compatibility only and are
//! rejected with `CS_BAD_ATTRIBUTE`, `IO_DEALLOCATE_WINDOW` and
//! `IO_DISABLE_WINDOW`, which are private to Card Services, and the
//! retired `IO_DATA_PATH_WIDTH*` spellings. None of them is declared.

use ddi_sys::{c_void, ddi_acc_handle_t, ddi_iblock_cookie_t};

use crate::client::{client_handle_t, csfunction_t, ddi_idevice_cookie_t};

// --- baseaddru_t and io_req_t ---------------------------------------------

/// `csx_RequestIO(9F)`. illumos DDI Specific (illumos DDI).
///
/// The base of one I/O range in an [`io_req_t`]. Despite the name it is a
/// structure, not a union. On the call [`base`](Self::base) holds the first
/// port wanted, and on successful return [`handle`](Self::handle) holds the
/// access handle for the range.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct baseaddru_t {
    /// The first port address requested. When it is 0, Card Services
    /// chooses a range from the available resources and aligns it on a
    /// multiple of the number of ports requested, rounded up to a power of
    /// two. When two ranges are requested, at least one of them must have a
    /// non-zero base.
    pub base: u32,
    /// On successful return, an access handle for the first byte of the
    /// allocated I/O range. The driver passes it to the common access
    /// functions such as `csx_Get8(9F)` and must not make any assumptions
    /// about its format. The manual page shows it as an `acc_handle_t`;
    /// `sys/cs_types.h` declares it as a `ddi_acc_handle_t`, and both are
    /// pointers.
    pub handle: ddi_acc_handle_t,
}

// Layout on amd64 with LP64. base 4 at 0, 4 bytes of padding, handle 8 at 8;
// total 16.
const _: () = assert!(core::mem::size_of::<baseaddru_t>() == 16);
const _: () = assert!(core::mem::offset_of!(baseaddru_t, base) == 0);
const _: () = assert!(core::mem::offset_of!(baseaddru_t, handle) == 8);

/// `csx_RequestIO(9F)`. illumos DDI Specific (illumos DDI).
///
/// Describes the one or two I/O ranges that [`csx_RequestIO`] allocates and
/// [`csx_ReleaseIO`] releases. Each range is given by a base, a number of
/// contiguous ports and attributes. When only one range is wanted,
/// [`NumPorts2`](Self::NumPorts2) must be 0. On some systems several calls
/// with different bases, port counts or address lines are needed before
/// Card Services finds a combination it can allocate.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct io_req_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// The base of the first range on the call and its access handle on
    /// return. See [`baseaddru_t`].
    pub BasePort1: baseaddru_t,
    /// The number of contiguous ports in the first range.
    pub NumPorts1: u32,
    /// The attributes of the first range, one of [`IO_DATA_WIDTH_8`] and
    /// [`IO_DATA_WIDTH_16`] combined with the `WIN_ACC_*` byte order and
    /// ordering values of [`crate::window`]. On some combinations of host
    /// bus and adapter the width cannot be set and the bus cycle decides it.
    /// All other bits must be 0.
    pub Attributes1: u32,
    /// The base of the second range on the call and its access handle on
    /// return. See [`baseaddru_t`].
    pub BasePort2: baseaddru_t,
    /// The number of contiguous ports in the second range, or 0 when only
    /// one range is requested.
    pub NumPorts2: u32,
    /// The attributes of the second range, as for
    /// [`Attributes1`](Self::Attributes1).
    pub Attributes2: u32,
    /// The number of I/O address lines the card decodes. The notes of the
    /// page advise trying 0 or the smallest number that decodes the card's
    /// I/O space first, to use as few I/O resources as possible.
    pub IOAddrLines: u32,
}

// Layout on amd64 with LP64. baseaddru_t is 16 bytes aligned to 8. Socket 4
// at 0 and 4 bytes of padding, BasePort1 16 at 8, NumPorts1 4 at 24,
// Attributes1 4 at 28, BasePort2 16 at 32, NumPorts2 4 at 48, Attributes2 4
// at 52, IOAddrLines 4 at 56, which ends at 60 and is padded to 64.
const _: () = assert!(core::mem::size_of::<io_req_t>() == 64);
const _: () = assert!(core::mem::offset_of!(io_req_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(io_req_t, BasePort1) == 8);
const _: () = assert!(core::mem::offset_of!(io_req_t, NumPorts1) == 24);
const _: () = assert!(core::mem::offset_of!(io_req_t, BasePort2) == 32);
const _: () = assert!(core::mem::offset_of!(io_req_t, NumPorts2) == 48);
const _: () = assert!(core::mem::offset_of!(io_req_t, IOAddrLines) == 56);

/// The I/O range uses an 8-bit data path (`IO_DATA_WIDTH_8`).
pub const IO_DATA_WIDTH_8: u32 = 0x0000_0000;
/// The I/O range uses a 16-bit data path (`IO_DATA_WIDTH_16`).
pub const IO_DATA_WIDTH_16: u32 = 0x0000_0001;

// --- irq_req_t -----------------------------------------------------------

/// `csx_RequestIRQ(9F)`. illumos DDI Specific (illumos DDI).
///
/// Describes the interrupt that [`csx_RequestIRQ`] allocates and the
/// handler it registers. For [`csx_ReleaseIRQ`] the structure must hold the
/// same values that the earlier [`csx_RequestIRQ`] returned, or the call
/// fails with `CS_BAD_ARGS` and changes nothing.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct irq_req_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// The kind of interrupt wanted. [`IRQ_TYPE_EXCLUSIVE`] must be set,
    /// dedicating the system interrupt to this card.
    pub Attributes: u32,
    /// The client's interrupt handler, declared as a `csfunction_t *` in C.
    /// The page gives no prototype for it. The `pcic` adapter driver calls
    /// it with [`irq_handler_arg`](Self::irq_handler_arg) as the first
    /// argument and a null pointer as the second, and treats a return value
    /// of `DDI_INTR_CLAIMED` as the handler claiming the interrupt. The
    /// handler always runs above lock level and must restrict itself to
    /// operations that are allowed there.
    pub irq_handler: Option<csfunction_t>,
    /// The argument passed to [`irq_handler`](Self::irq_handler), for the
    /// client's own data.
    pub irq_handler_arg: *mut c_void,
    /// Set by Card Services to point at the interrupt block cookie that the
    /// client must use to initialise the mutexes its interrupt handler
    /// takes.
    pub iblk_cookie: *mut ddi_iblock_cookie_t,
    /// Set by Card Services to point at the interrupt device cookie of the
    /// interrupt. See [`ddi_idevice_cookie_t`].
    pub idev_cookie: *mut ddi_idevice_cookie_t,
}

// Layout on amd64 with LP64. Socket 4 at 0, Attributes 4 at 4, then four
// pointers, irq_handler at 8, irq_handler_arg at 16, iblk_cookie at 24 and
// idev_cookie at 32; total 40.
const _: () = assert!(core::mem::size_of::<irq_req_t>() == 40);
const _: () = assert!(core::mem::offset_of!(irq_req_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(irq_req_t, irq_handler) == 8);
const _: () = assert!(core::mem::offset_of!(irq_req_t, irq_handler_arg) == 16);
const _: () = assert!(core::mem::offset_of!(irq_req_t, iblk_cookie) == 24);
const _: () = assert!(core::mem::offset_of!(irq_req_t, idev_cookie) == 32);

/// The interrupt is dedicated to this socket. [`csx_RequestIRQ`] requires
/// it (`IRQ_TYPE_EXCLUSIVE`).
pub const IRQ_TYPE_EXCLUSIVE: u32 = 0x0000_0002;

unsafe extern "C" {
    /// `csx_RequestIO(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Requests the I/O resources described by the [`io_req_t`] at `ir` for
    /// the client `ch`. On success the structure holds an access handle for
    /// each range. The configuration is then established with
    /// `csx_RequestConfiguration(9F)`, which uses the resources of the last
    /// successful call. The function may be called several times until a
    /// usable set of resources is found, but it fails if it has already
    /// succeeded without a matching [`csx_ReleaseIO`].
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_ATTRIBUTE`](crate::consts::CS_BAD_ATTRIBUTE) for invalid
    /// attributes, [`CS_BAD_BASE`](crate::consts::CS_BAD_BASE) for an
    /// invalid base port, [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE)
    /// for an invalid client handle,
    /// [`CS_CONFIGURATION_LOCKED`](crate::consts::CS_CONFIGURATION_LOCKED)
    /// when a configuration has already been requested,
    /// [`CS_IN_USE`](crate::consts::CS_IN_USE) when an earlier request has
    /// not been released, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the
    /// socket is empty, [`CS_BAD_WINDOW`](crate::consts::CS_BAD_WINDOW) or
    /// [`CS_OUT_OF_RESOURCE`](crate::consts::CS_OUT_OF_RESOURCE) when the
    /// resources could not be allocated, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_RequestIO(ch: client_handle_t, ir: *mut io_req_t) -> i32;
    /// `csx_RequestIO(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Releases the I/O resources that [`csx_RequestIO`] allocated for the
    /// client `ch`, as described by the [`io_req_t`] at `ir`. The window
    /// resource list of Card Services is adjusted, and depending on the
    /// adapter the I/O window may also be disabled.
    ///
    /// The return values are those of [`csx_RequestIO`]. It may be called
    /// from user or kernel context.
    pub fn csx_ReleaseIO(ch: client_handle_t, ir: *mut io_req_t) -> i32;
    /// `csx_RequestIRQ(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Requests an interrupt for the client `ch` and registers the handler
    /// given in the [`irq_req_t`] at `ir`. On success Card Services fills in
    /// the cookie pointers of the structure. The handler is not called until
    /// `csx_RequestConfiguration(9F)` or `csx_ModifyConfiguration(9F)` has
    /// succeeded with `CONF_ENABLE_IRQ_STEERING` set. The function fails if
    /// it has already succeeded without a matching [`csx_ReleaseIRQ`].
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_ARGS`](crate::consts::CS_BAD_ARGS) when the description does
    /// not match the allocation,
    /// [`CS_BAD_ATTRIBUTE`](crate::consts::CS_BAD_ATTRIBUTE) when
    /// [`IRQ_TYPE_EXCLUSIVE`] is missing or a reserved bit is set,
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid client
    /// handle, [`CS_BAD_IRQ`](crate::consts::CS_BAD_IRQ) when no interrupt
    /// could be allocated, [`CS_IN_USE`](crate::consts::CS_IN_USE) when an
    /// earlier request has not been released,
    /// [`CS_CONFIGURATION_LOCKED`](crate::consts::CS_CONFIGURATION_LOCKED)
    /// when a configuration is in place,
    /// [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the socket is empty, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_RequestIRQ(ch: client_handle_t, ir: *mut irq_req_t) -> i32;
    /// `csx_RequestIRQ(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Releases the interrupt that [`csx_RequestIRQ`] allocated for the
    /// client `ch`. The [`irq_req_t`] at `ir` must hold the values the
    /// request returned. The interrupt resource list of Card Services is
    /// adjusted, and depending on the adapter the connection to the host bus
    /// interrupt may also be disabled.
    ///
    /// The return values are those of [`csx_RequestIRQ`]. It may be called
    /// from user or kernel context.
    pub fn csx_ReleaseIRQ(ch: client_handle_t, ir: *mut irq_req_t) -> i32;
}
