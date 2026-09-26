//! The configuration of card and socket from `sys/cs.h`.
//!
//! Once a client has the I/O ranges and the interrupt it needs, it asks Card
//! Services to configure the card and the socket with
//! [`csx_RequestConfiguration`]. That applies power, programs the socket
//! adapter, writes the configuration registers of the card, including the
//! configuration index into the Configuration Option Register, and, for a
//! multiple function card, programs its I/O base and limit registers. The
//! values usually come from the `CISTPL_CONFIG` and `CISTPL_CFTABLE_ENTRY`
//! tuples of the card. [`csx_ModifyConfiguration`] changes Vpp and the
//! interrupt steering of an established configuration,
//! [`csx_AccessConfigurationRegister`] reads or writes a single
//! configuration register, and [`csx_ReleaseConfiguration`] returns the card
//! to a memory only interface. The configuration must be released before
//! the resources it uses.
//!
//! Besides the constants the manual pages name, the offsets of the
//! configuration registers for [`access_config_reg_t::Offset`] are declared.
//! The pages also mention `CONF_VSOVERRIDE` and `CONFIG_ESR_REG_PRESENT`,
//! which no header defines; the header calls the latter
//! [`CONFIG_EXSTAT_REG_PRESENT`]. The Configuration Option Register and
//! Card Configuration Status Register bits `COR_*` and `CCSR_*` and the
//! `SCR_*` macros are defined in `sys/cs.h` but named by no page, and are
//! left out.

use crate::client::client_handle_t;

// --- config_req_t --------------------------------------------------------

/// `csx_RequestConfiguration(9F)`. illumos DDI Specific (illumos DDI).
///
/// The configuration that [`csx_RequestConfiguration`] establishes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct config_req_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// Configuration attributes. [`CONF_ENABLE_IRQ_STEERING`] connects the
    /// interrupt line of the card to the system interrupt chosen with
    /// `csx_RequestIRQ(9F)`, after which the interrupt handler may be
    /// called.
    pub Attributes: u32,
    /// The Vcc to apply, in tenths of a volt, from 0 to 25.5 volts. The
    /// exact voltage must be available from the system. After insertion the
    /// voltage is the one the key and voltage sense pins of the card
    /// indicate, but a card that declares several Vcc values in its
    /// `CISTPL_CFTABLE_ENTRY` tuples may be given any of them.
    pub Vcc: u32,
    /// The Vpp1 to apply, in tenths of a volt.
    pub Vpp1: u32,
    /// The Vpp2 to apply, in tenths of a volt.
    pub Vpp2: u32,
    /// The socket interface, [`SOCKET_INTERFACE_MEMORY`] or
    /// [`SOCKET_INTERFACE_MEMORY_AND_IO`].
    pub IntType: u32,
    /// The offset in bytes of the configuration registers from the start of
    /// attribute memory.
    pub ConfigBase: u32,
    /// The initial value for the Card Configuration Status Register, written
    /// if [`Present`](Self::Present) says it exists.
    pub Status: u32,
    /// The initial value for the Pin Replacement Register, written if it
    /// exists. It also tells Card Services which signals of the register
    /// are valid, which affects `csx_GetStatus(9F)`. For each valid signal
    /// both its `PRR_*_STATUS` and its `PRR_*_EVENT` bit must be set.
    pub Pin: u32,
    /// The initial value for the Socket and Copy Register, written if it
    /// exists.
    pub Copy: u32,
    /// The value written to the Configuration Option Register. Only the low
    /// six bits are significant, and Card Services always sets level mode
    /// interrupts.
    pub ConfigIndex: u32,
    /// Which configuration registers the card has, a combination of the
    /// `CONFIG_*_REG_PRESENT` bits.
    pub Present: u32,
    /// The initial value for the Extended Status Register, written if it
    /// exists.
    pub ExtendedStatus: u32,
}

// Layout on amd64 with LP64. Thirteen uint32_t at 0 to 48; total 52.
const _: () = assert!(core::mem::size_of::<config_req_t>() == 52);
const _: () = assert!(core::mem::offset_of!(config_req_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(config_req_t, ConfigIndex) == 40);
const _: () = assert!(core::mem::offset_of!(config_req_t, ExtendedStatus) == 48);

/// Connect the interrupt line of the card to the system interrupt chosen
/// with `csx_RequestIRQ(9F)`. It is valid in [`config_req_t::Attributes`]
/// and [`modify_config_t::Attributes`] (`CONF_ENABLE_IRQ_STEERING`).
pub const CONF_ENABLE_IRQ_STEERING: u32 = 0x0001_0000;
/// Asks [`csx_ModifyConfiguration`] to change Vpp1
/// (`CONF_VPP1_CHANGE_VALID`).
pub const CONF_VPP1_CHANGE_VALID: u32 = 0x0000_0002;
/// Asks [`csx_ModifyConfiguration`] to change Vpp2
/// (`CONF_VPP2_CHANGE_VALID`).
pub const CONF_VPP2_CHANGE_VALID: u32 = 0x0000_0004;
/// Asks [`csx_ModifyConfiguration`] to change the interrupt steering to
/// what [`CONF_ENABLE_IRQ_STEERING`] says (`CONF_IRQ_CHANGE_VALID`).
pub const CONF_IRQ_CHANGE_VALID: u32 = 0x0000_0008;

/// A memory only socket interface (`SOCKET_INTERFACE_MEMORY`).
pub const SOCKET_INTERFACE_MEMORY: u32 = 0x0000_0001;
/// A memory and I/O socket interface (`SOCKET_INTERFACE_MEMORY_AND_IO`).
pub const SOCKET_INTERFACE_MEMORY_AND_IO: u32 = 0x0000_0002;

/// The Configuration Option Register is present
/// (`CONFIG_OPTION_REG_PRESENT`).
pub const CONFIG_OPTION_REG_PRESENT: u32 = 0x0000_0001;
/// The Card Configuration Status Register is present
/// (`CONFIG_STATUS_REG_PRESENT`).
pub const CONFIG_STATUS_REG_PRESENT: u32 = 0x0000_0002;
/// The Pin Replacement Register is present (`CONFIG_PINREPL_REG_PRESENT`).
pub const CONFIG_PINREPL_REG_PRESENT: u32 = 0x0000_0004;
/// The Socket and Copy Register is present (`CONFIG_COPY_REG_PRESENT`).
pub const CONFIG_COPY_REG_PRESENT: u32 = 0x0000_0008;
/// The Extended Status Register is present. The page of
/// [`csx_RequestConfiguration`] calls it `CONFIG_ESR_REG_PRESENT`
/// (`CONFIG_EXSTAT_REG_PRESENT`).
pub const CONFIG_EXSTAT_REG_PRESENT: u32 = 0x0000_0010;
/// The first I/O base register is present (`CONFIG_IOBASE0_REG_PRESENT`).
pub const CONFIG_IOBASE0_REG_PRESENT: u32 = 0x0000_0020;
/// The second I/O base register is present
/// (`CONFIG_IOBASE1_REG_PRESENT`).
pub const CONFIG_IOBASE1_REG_PRESENT: u32 = 0x0000_0040;
/// The third I/O base register is present (`CONFIG_IOBASE2_REG_PRESENT`).
pub const CONFIG_IOBASE2_REG_PRESENT: u32 = 0x0000_0080;
/// The fourth I/O base register is present
/// (`CONFIG_IOBASE3_REG_PRESENT`).
pub const CONFIG_IOBASE3_REG_PRESENT: u32 = 0x0000_0100;
/// The I/O limit register is present (`CONFIG_IOLIMIT_REG_PRESENT`).
pub const CONFIG_IOLIMIT_REG_PRESENT: u32 = 0x0000_0200;

/// The write protect signal in the Pin Replacement Register. Reading gives
/// its state and writing sets the changed bit (`PRR_WP_STATUS`).
pub const PRR_WP_STATUS: u32 = 0x01;
/// The READY signal in the Pin Replacement Register. Reading gives its
/// state and writing sets the changed bit (`PRR_READY_STATUS`).
pub const PRR_READY_STATUS: u32 = 0x02;
/// The BVD2 signal in the Pin Replacement Register. Reading gives its state
/// and writing sets the changed bit (`PRR_BVD2_STATUS`).
pub const PRR_BVD2_STATUS: u32 = 0x04;
/// The BVD1 signal in the Pin Replacement Register. Reading gives its state
/// and writing sets the changed bit (`PRR_BVD1_STATUS`).
pub const PRR_BVD1_STATUS: u32 = 0x08;
/// The write protect signal has changed (`PRR_WP_EVENT`).
pub const PRR_WP_EVENT: u32 = 0x10;
/// The READY signal has changed (`PRR_READY_EVENT`).
pub const PRR_READY_EVENT: u32 = 0x20;
/// The BVD2 signal has changed (`PRR_BVD2_EVENT`).
pub const PRR_BVD2_EVENT: u32 = 0x40;
/// The BVD1 signal has changed (`PRR_BVD1_EVENT`).
pub const PRR_BVD1_EVENT: u32 = 0x80;

// --- modify_config_t -----------------------------------------------------

/// `csx_ModifyConfiguration(9F)`. illumos DDI Specific (illumos DDI).
///
/// The changes that [`csx_ModifyConfiguration`] applies to an established
/// configuration.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct modify_config_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// What to change, a combination of [`CONF_IRQ_CHANGE_VALID`] with
    /// [`CONF_ENABLE_IRQ_STEERING`] set or clear, [`CONF_VPP1_CHANGE_VALID`]
    /// and [`CONF_VPP2_CHANGE_VALID`].
    pub Attributes: u32,
    /// The new Vpp1, in tenths of a volt. Setting Vpp to 0 may lose the
    /// state of the card, which the client is then responsible for
    /// restoring.
    pub Vpp1: u32,
    /// The new Vpp2, in tenths of a volt.
    pub Vpp2: u32,
}

// Layout on amd64 with LP64. Four uint32_t at 0, 4, 8 and 12; total 16.
const _: () = assert!(core::mem::size_of::<modify_config_t>() == 16);
const _: () = assert!(core::mem::offset_of!(modify_config_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(modify_config_t, Vpp2) == 12);

// --- release_config_t ----------------------------------------------------

/// `csx_ReleaseConfiguration(9F)`. illumos DDI Specific (illumos DDI).
///
/// The argument of [`csx_ReleaseConfiguration`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct release_config_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
}

// Layout on amd64 with LP64. One uint32_t at 0; total 4.
const _: () = assert!(core::mem::size_of::<release_config_t>() == 4);
const _: () = assert!(core::mem::offset_of!(release_config_t, Socket) == 0);

// --- access_config_reg_t -------------------------------------------------

/// `csx_AccessConfigurationRegister(9F)`. illumos DDI Specific (illumos DDI).
///
/// The register access that [`csx_AccessConfigurationRegister`] performs.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct access_config_reg_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// The operation, [`CONFIG_REG_READ`] or [`CONFIG_REG_WRITE`]. All other
    /// values are reserved.
    pub Action: u32,
    /// The byte offset of the register from the configuration register base
    /// given to [`csx_RequestConfiguration`], such as
    /// [`CONFIG_OPTION_REG_OFFSET`].
    pub Offset: u32,
    /// The value read, or the value to write. After a write it still holds
    /// the value written, because Card Services does not read the register
    /// back.
    pub Value: u32,
}

// Layout on amd64 with LP64. Four uint32_t at 0, 4, 8 and 12; total 16.
const _: () = assert!(core::mem::size_of::<access_config_reg_t>() == 16);
const _: () = assert!(core::mem::offset_of!(access_config_reg_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(access_config_reg_t, Value) == 12);

/// Read the configuration register (`CONFIG_REG_READ`).
pub const CONFIG_REG_READ: u32 = 0x0000_0001;
/// Write the configuration register (`CONFIG_REG_WRITE`).
pub const CONFIG_REG_WRITE: u32 = 0x0000_0002;

/// The offset of the Configuration Option Register
/// (`CONFIG_OPTION_REG_OFFSET`).
pub const CONFIG_OPTION_REG_OFFSET: u32 = 0x00;
/// The offset of the Card Configuration Status Register
/// (`CONFIG_STATUS_REG_OFFSET`).
pub const CONFIG_STATUS_REG_OFFSET: u32 = 0x02;
/// The offset of the Pin Replacement Register
/// (`CONFIG_PINREPL_REG_OFFSET`).
pub const CONFIG_PINREPL_REG_OFFSET: u32 = 0x04;
/// The offset of the Socket and Copy Register (`CONFIG_COPY_REG_OFFSET`).
pub const CONFIG_COPY_REG_OFFSET: u32 = 0x06;
/// The offset of the Extended Status Register
/// (`CONFIG_EXSTAT_REG_OFFSET`).
pub const CONFIG_EXSTAT_REG_OFFSET: u32 = 0x08;
/// The offset of the first I/O base register
/// (`CONFIG_IOBASE0_REG_OFFSET`).
pub const CONFIG_IOBASE0_REG_OFFSET: u32 = 0x0a;
/// The offset of the second I/O base register
/// (`CONFIG_IOBASE1_REG_OFFSET`).
pub const CONFIG_IOBASE1_REG_OFFSET: u32 = 0x0c;
/// The offset of the third I/O base register
/// (`CONFIG_IOBASE2_REG_OFFSET`).
pub const CONFIG_IOBASE2_REG_OFFSET: u32 = 0x0e;
/// The offset of the fourth I/O base register
/// (`CONFIG_IOBASE3_REG_OFFSET`).
pub const CONFIG_IOBASE3_REG_OFFSET: u32 = 0x10;
/// The offset of the I/O limit register (`CONFIG_IOLIMIT_REG_OFFSET`).
pub const CONFIG_IOLIMIT_REG_OFFSET: u32 = 0x12;

// --- reset_function_t ----------------------------------------------------

/// `csx_ResetFunction(9F)`. illumos DDI Specific (illumos DDI).
///
/// The argument of [`csx_ResetFunction`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct reset_function_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// Must be 0.
    pub Attributes: u32,
}

// Layout on amd64 with LP64. Two uint32_t at 0 and 4; total 8.
const _: () = assert!(core::mem::size_of::<reset_function_t>() == 8);
const _: () = assert!(core::mem::offset_of!(reset_function_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(reset_function_t, Attributes) == 4);

unsafe extern "C" {
    /// `csx_RequestConfiguration(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Configures the card of the client `ch` and its socket as the
    /// [`config_req_t`] at `cr` describes. Clients that need I/O or
    /// interrupt resources must use it, after `csx_RequestIO(9F)` and
    /// `csx_RequestIRQ(9F)`. It establishes the configuration in the socket
    /// adapter and the card, and programs the base and limit registers of a
    /// multiple function card when they exist.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_BAD_SOCKET`](crate::consts::CS_BAD_SOCKET) when the
    /// socket hardware parameters could not be read or set,
    /// [`CS_BAD_VCC`](crate::consts::CS_BAD_VCC) or
    /// [`CS_BAD_VPP`](crate::consts::CS_BAD_VPP) when the requested voltage
    /// is not available, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the
    /// socket is empty, [`CS_BAD_TYPE`](crate::consts::CS_BAD_TYPE) when the
    /// socket does not support an I/O and memory interface,
    /// [`CS_CONFIGURATION_LOCKED`](crate::consts::CS_CONFIGURATION_LOCKED)
    /// when a configuration is already in place, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_RequestConfiguration(ch: client_handle_t, cr: *mut config_req_t) -> i32;
    /// `csx_ModifyConfiguration(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Modifies the configuration that [`csx_RequestConfiguration`]
    /// established for the client `ch`, as the [`modify_config_t`] at `mc`
    /// describes. Only Vpp and the interrupt steering can be changed. Vcc,
    /// the mapped I/O addresses and the interrupt priority can only be
    /// changed by releasing the configuration and the resources and
    /// requesting them again.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid handle
    /// or when no configuration was requested,
    /// [`CS_BAD_SOCKET`](crate::consts::CS_BAD_SOCKET) when the socket
    /// hardware parameters could not be read or set,
    /// [`CS_BAD_VPP`](crate::consts::CS_BAD_VPP) when the requested Vpp is
    /// not available, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the
    /// socket is empty, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_ModifyConfiguration(ch: client_handle_t, mc: *mut modify_config_t) -> i32;
    /// `csx_ReleaseConfiguration(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Returns the card of the client `ch` and its socket to a simple memory
    /// only interface and sets the card to configuration zero by writing 0
    /// to its Configuration Option Register. The argument `rc` points to a
    /// [`release_config_t`]. Card Services may remove power from the socket
    /// if no client still uses it through a configuration or a window, but
    /// it does not reset the card. The configuration must be released
    /// before the resources requested with `csx_RequestIO(9F)`,
    /// `csx_RequestIRQ(9F)` and `csx_RequestWindow(9F)`.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid handle
    /// or when no configuration was requested,
    /// [`CS_BAD_SOCKET`](crate::consts::CS_BAD_SOCKET) when the socket
    /// hardware parameters could not be read or set,
    /// [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the socket is empty, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_ReleaseConfiguration(ch: client_handle_t, rc: *mut release_config_t) -> i32;
    /// `csx_AccessConfigurationRegister(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Reads or writes one configuration register of the card of the client
    /// `ch`, as the [`access_config_reg_t`] at `acr` describes. A write to
    /// the Configuration Option Register at offset 0 can change the
    /// interrupt mode of the card or put it into reset, with undefined
    /// results, so the client should read the register first to preserve
    /// the interrupt mode in bit 6. To reset a card a client should use
    /// [`csx_ResetFunction`] instead, which notifies all clients of the card.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_ARGS`](crate::consts::CS_BAD_ARGS) when the offset is out of
    /// range or the action is neither read nor write,
    /// [`CS_UNSUPPORTED_MODE`](crate::consts::CS_UNSUPPORTED_MODE) when no
    /// configuration was requested,
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the socket is
    /// empty, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_AccessConfigurationRegister(ch: client_handle_t, acr: *mut access_config_reg_t) -> i32;
    /// `csx_ResetFunction(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Asks that the function of the card of the client `ch` start a reset,
    /// as the [`reset_function_t`] at `rf` describes. The page notes that
    /// the function is not implemented and always returns
    /// [`CS_IN_USE`](crate::consts::CS_IN_USE), the value that means this
    /// implementation does not allow configured cards to be reset.
    ///
    /// The other documented values are
    /// [`CS_SUCCESS`](crate::consts::CS_SUCCESS) when the request was
    /// noted, [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the socket is
    /// empty, [`CS_BAD_SOCKET`](crate::consts::CS_BAD_SOCKET) for an invalid
    /// socket or function, and
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_ResetFunction(ch: client_handle_t, rf: *mut reset_function_t) -> i32;
}
