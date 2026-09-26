//! The Card Services return codes and version from `sys/cs.h`.
//!
//! Every `csx_*` function that reports an outcome returns an `int32_t`
//! holding one of these codes. The codes from [`CS_SUCCESS`] to
//! [`CS_BAD_HANDLE`] follow the numbering of the PC Card standard, with
//! gaps where the standard reserves a value. The codes from [`CS_NO_CIS`]
//! upwards are illumos extensions. `csx_Error2Text(9F)` turns any of them
//! into a string for a message.
//!
//! The header also defines `CS_ERRORLIST_END`, which only terminates the
//! table inside Card Services, and it is left out. Several manual pages
//! name a code `CS_FAILURE`, but no header defines it.

// --- return codes --------------------------------------------------------

/// The request succeeded (`CS_SUCCESS`).
pub const CS_SUCCESS: i32 = 0x00;
/// The specified adapter is invalid (`CS_BAD_ADAPTER`).
pub const CS_BAD_ADAPTER: i32 = 0x01;
/// An attribute value is invalid, for instance no client type or more than
/// one was given to `csx_RegisterClient(9F)` (`CS_BAD_ATTRIBUTE`).
pub const CS_BAD_ATTRIBUTE: i32 = 0x02;
/// The system base address, such as an I/O base port, is invalid
/// (`CS_BAD_BASE`).
pub const CS_BAD_BASE: i32 = 0x03;
/// The error detection code generator is invalid (`CS_BAD_EDC`).
pub const CS_BAD_EDC: i32 = 0x04;
/// The interrupt request is invalid or no interrupt could be allocated
/// (`CS_BAD_IRQ`).
pub const CS_BAD_IRQ: i32 = 0x06;
/// The card offset is invalid (`CS_BAD_OFFSET`).
pub const CS_BAD_OFFSET: i32 = 0x07;
/// The card page is invalid (`CS_BAD_PAGE`).
pub const CS_BAD_PAGE: i32 = 0x08;
/// The read request could not be completed (`CS_READ_FAILURE`).
pub const CS_READ_FAILURE: i32 = 0x09;
/// The size is invalid (`CS_BAD_SIZE`).
pub const CS_BAD_SIZE: i32 = 0x0a;
/// The specified socket is invalid, or the socket state could not be read
/// or set (`CS_BAD_SOCKET`).
pub const CS_BAD_SOCKET: i32 = 0x0b;
/// The window or interface type is invalid (`CS_BAD_TYPE`).
pub const CS_BAD_TYPE: i32 = 0x0d;
/// The requested Vcc is not available on the socket (`CS_BAD_VCC`).
pub const CS_BAD_VCC: i32 = 0x0e;
/// The requested Vpp is not available on the socket (`CS_BAD_VPP`).
pub const CS_BAD_VPP: i32 = 0x0f;
/// The specified window is invalid or its hardware parameters could not be
/// read or set (`CS_BAD_WINDOW`).
pub const CS_BAD_WINDOW: i32 = 0x11;
/// The write request could not be completed (`CS_WRITE_FAILURE`).
pub const CS_WRITE_FAILURE: i32 = 0x12;
/// There is no PC Card in the socket (`CS_NO_CARD`).
pub const CS_NO_CARD: i32 = 0x14;
/// The function is not supported. Every `csx_*` function returns this when
/// no Card Services module is loaded, which the manual pages describe as no
/// PCMCIA hardware being installed (`CS_UNSUPPORTED_FUNCTION`).
pub const CS_UNSUPPORTED_FUNCTION: i32 = 0x15;
/// The processor mode is not supported, or a configuration register was
/// accessed before a configuration was requested (`CS_UNSUPPORTED_MODE`).
pub const CS_UNSUPPORTED_MODE: i32 = 0x16;
/// The specified speed is not available (`CS_BAD_SPEED`).
pub const CS_BAD_SPEED: i32 = 0x17;
/// Card Services is busy and the request should be tried again later
/// (`CS_BUSY`).
pub const CS_BUSY: i32 = 0x18;
/// An error without a more specific code occurred (`CS_GENERAL_FAILURE`).
pub const CS_GENERAL_FAILURE: i32 = 0x19;
/// The media is write protected (`CS_WRITE_PROTECTED`).
pub const CS_WRITE_PROTECTED: i32 = 0x1a;
/// An argument length is invalid (`CS_BAD_ARG_LENGTH`).
pub const CS_BAD_ARG_LENGTH: i32 = 0x1b;
/// An argument value is invalid (`CS_BAD_ARGS`).
pub const CS_BAD_ARGS: i32 = 0x1c;
/// The configuration is locked because `csx_RequestConfiguration(9F)` has
/// already been done (`CS_CONFIGURATION_LOCKED`).
pub const CS_CONFIGURATION_LOCKED: i32 = 0x1d;
/// The requested resource is in use, or the client still holds resources
/// it must release first (`CS_IN_USE`).
pub const CS_IN_USE: i32 = 0x1e;
/// There are no more items of the kind requested, such as tuples or clients
/// (`CS_NO_MORE_ITEMS`).
pub const CS_NO_MORE_ITEMS: i32 = 0x1f;
/// The internal resources of Card Services are exhausted
/// (`CS_OUT_OF_RESOURCE`).
pub const CS_OUT_OF_RESOURCE: i32 = 0x20;
/// The client or window handle is invalid (`CS_BAD_HANDLE`).
pub const CS_BAD_HANDLE: i32 = 0x21;

/// An illumos extension meaning that the card has no Card Information
/// Structure, or that it is invalid (`CS_NO_CIS`).
pub const CS_NO_CIS: i32 = 0x80;
/// An illumos extension meaning that the Card Information Structure on the
/// card is bad, which the tuple parsers also report as a generic parser
/// error (`CS_BAD_CIS`).
pub const CS_BAD_CIS: i32 = 0x81;
/// An illumos extension meaning that the parser does not know how to parse
/// the tuple (`CS_UNKNOWN_TUPLE`).
pub const CS_UNKNOWN_TUPLE: i32 = 0x82;
/// An illumos extension meaning that the Card Services version the client
/// asked for is incompatible (`CS_BAD_VERSION`).
pub const CS_BAD_VERSION: i32 = 0x83;
/// An illumos extension that a client's event handler returns for an event
/// it does not support (`CS_UNSUPPORTED_EVENT`).
pub const CS_UNSUPPORTED_EVENT: i32 = 0x84;
/// An illumos extension reporting an error in the protocol of the Card
/// Services interface driver (`CS_CSI_ERROR`).
pub const CS_CSI_ERROR: i32 = 0x85;
/// An illumos extension meaning that the Card Services interface library or
/// driver is not initialised (`CS_CSI_NOT_INIT`).
pub const CS_CSI_NOT_INIT: i32 = 0x86;
/// An illumos extension meaning that there is no parser for the tuple
/// (`CS_NO_TUPLE_PARSER`).
pub const CS_NO_TUPLE_PARSER: i32 = 0x87;
/// An illumos extension meaning that the card is not ready
/// (`CS_CARD_NOT_READY`).
pub const CS_CARD_NOT_READY: i32 = 0x88;

// --- version -------------------------------------------------------------

/// The Card Services version this implementation provides, release 5.2 of
/// the PC Card standard encoded as the major number in the upper 16 bits
/// and the minor number in the lower 16. A client puts it in
/// [`client_reg_t::Version`](crate::client::client_reg_t::Version) and, when
/// answering a client information request, in
/// [`client_info_t::CSLevel`](crate::event::client_info_t::CSLevel)
/// (`CS_VERSION`).
pub const CS_VERSION: u32 = (5 << 16) | 2;
