//! Card Services events, the `csx_event_handler(9E)` callback and the text
//! conversions from `sys/cs.h` and `sys/cs_types.h`.
//!
//! Card Services tells a client about insertion, removal, readiness,
//! battery state, power management and the other things that happen to its
//! card by calling the event handler that the client registered with
//! `csx_RegisterClient(9F)`. The client chooses which events it wants with
//! the event masks of `csx_RegisterClient(9F)`, `csx_RequestSocketMask(9F)`
//! and `csx_SetEventMask(9F)`, using the `CS_EVENT_*` bits declared here.
//! No event is delivered before `csx_RequestSocketMask(9F)` succeeds.
//!
//! Each event comes with a priority. High priority events, marked with
//! [`CS_EVENT_PRI_HIGH`], are delivered above lock level, and the handler
//! must protect its state with a mutex initialised with the interrupt block
//! cookie that `csx_RegisterClient(9F)` returned. Low priority events,
//! marked with [`CS_EVENT_PRI_LOW`], are delivered below lock level and use
//! a mutex initialised without a cookie. Card removal is delivered twice,
//! first at high priority, when the handler should only note that the card
//! is gone, and then at low priority, when it should release the
//! configuration and free its resources.
//!
//! In illumos the event codes are bit flags, unlike the numbered events of
//! the PC Card standard, so that they can also be used in masks. Only the
//! events that `csx_event_handler(9E)` documents are declared. The header
//! also defines `CS_EVENT_MTD_REQUEST`, `CS_EVENT_TIMER_EXPIRED`,
//! `CS_EVENT_SS_UPDATED`, `CS_EVENT_STATUS_CHANGE` and
//! `CS_EVENT_CARD_REMOVAL_LOWP`, which the page does not describe, and
//! `CS_EVENT_ALL_CLIENTS` and `CS_EVENT_READY_TIMEOUT`, which it marks as
//! internal to Card Services and never seen by clients.

use ddi_sys::{c_char, c_void};

use crate::client::client_handle_t;

/// `csx_event_handler(9E)`. illumos architecture specific (illumos DDI).
///
/// A Card Services event code or event mask, a `uint32_t` in C. Each event
/// is one of the `CS_EVENT_*` bits, and a mask is a combination of them.
pub type event_t = u32;

/// The length of `sys/modctl.h`'s `MODMAXNAMELEN`, which sizes the driver
/// name arrays of Card Services structures.
pub(crate) const MODMAXNAMELEN: usize = 32;

// --- client_info_t -------------------------------------------------------

/// The length of the client and vendor name arrays in a [`client_info_t`]
/// (`CS_CLIENT_INFO_MAX_NAME_LEN`).
pub const CS_CLIENT_INFO_MAX_NAME_LEN: usize = 80;

/// `csx_event_handler(9E)`. illumos architecture specific (illumos DDI).
///
/// Information about a client, which the client fills in when it handles a
/// [`CS_EVENT_CLIENT_INFO`] event. It is embedded in the
/// [`event_callback_args_t`] passed to the event handler. The handler should
/// fill it in only when the sub-service field of
/// [`Attributes`](Self::Attributes), bits 8 to 15, equals
/// [`CS_CLIENT_INFO_SUBSVC_CS`], and return `CS_UNSUPPORTED_EVENT`
/// otherwise.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct client_info_t {
    /// Attribute flags. The low byte repeats the attributes the client
    /// registered with, bits 8 to 15 name the sub-service being asked about,
    /// and the handler must OR in [`CS_CLIENT_INFO_VALID`] when it fills in
    /// the structure.
    pub Attributes: u32,
    /// A driver private version number, in binary coded decimal.
    pub Revision: u32,
    /// The Card Services version the driver was written for, which must be
    /// [`CS_VERSION`](crate::consts::CS_VERSION).
    pub CSLevel: u32,
    /// The revision date of the driver. The C macro
    /// `CS_CLIENT_INFO_MAKE_DATE` builds it from the day of the month in
    /// bits 0 to 4, the month in bits 5 to 8, and the year less 1980 in bits
    /// 9 to 15.
    pub RevDate: u32,
    /// A string describing the driver.
    pub ClientName: [c_char; CS_CLIENT_INFO_MAX_NAME_LEN],
    /// A string naming the vendor of the driver.
    pub VendorName: [c_char; CS_CLIENT_INFO_MAX_NAME_LEN],
    /// The name of the driver. Card Services copies it in after the driver
    /// has handled the event, so the driver need not set it.
    pub DriverName: [c_char; MODMAXNAMELEN],
}

// Layout on amd64 with LP64. Four uint32_t at 0, 4, 8 and 12, ClientName 80
// at 16, VendorName 80 at 96 and DriverName 32 at 176, which ends at 208;
// the alignment is 4, so the total is 208.
const _: () = assert!(core::mem::size_of::<client_info_t>() == 208);
const _: () = assert!(core::mem::offset_of!(client_info_t, Attributes) == 0);
const _: () = assert!(core::mem::offset_of!(client_info_t, ClientName) == 16);
const _: () = assert!(core::mem::offset_of!(client_info_t, VendorName) == 96);
const _: () = assert!(core::mem::offset_of!(client_info_t, DriverName) == 176);

/// Set by an event handler in [`client_info_t::Attributes`] to say that it
/// filled in the client information (`CS_CLIENT_INFO_VALID`).
pub const CS_CLIENT_INFO_VALID: u32 = 0x0001_0000;
/// The mask of the sub-service field, bits 8 to 15, of
/// [`client_info_t::Attributes`]. The C macro `GET_CLIENT_INFO_SUBSVC`
/// extracts the field with it (`CS_CLIENT_INFO_SUBSVC_MASK`).
pub const CS_CLIENT_INFO_SUBSVC_MASK: u32 = 0x0000_ff00;
/// The sub-service value asking for Card Services client data, the only one
/// a driver answers (`CS_CLIENT_INFO_SUBSVC_CS`).
pub const CS_CLIENT_INFO_SUBSVC_CS: u32 = 0x0000_0000;

// --- event_callback_args_t -----------------------------------------------

/// `csx_event_handler(9E)`. illumos architecture specific (illumos DDI).
///
/// The arguments that accompany an event, passed by pointer as the third
/// argument of the event handler. A client supplies
/// [`client_data`](Self::client_data) in the copy inside its
/// `client_reg_t` when it registers. Most members correspond to the event
/// arguments of the PC Card standard. The manual page documents
/// [`info`](Self::info), [`client_data`](Self::client_data) and
/// [`client_info`](Self::client_info).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct event_callback_args_t {
    /// The handle of the client the event is for.
    pub client_handle: client_handle_t,
    /// Event specific information. For [`CS_EVENT_WRITE_PROTECT`] it holds
    /// [`CS_EVENT_WRITE_PROTECT_WPOFF`] or [`CS_EVENT_WRITE_PROTECT_WPON`]
    /// cast to a pointer.
    pub info: *mut c_void,
    /// The memory technology driver request of the event, from the PC Card
    /// standard. The manual page does not document it.
    pub mtdrequest: *mut c_void,
    /// The buffer argument of the event, from the PC Card standard. The
    /// manual page does not document it.
    pub buffer: *mut c_void,
    /// The miscellaneous argument of the event, from the PC Card standard.
    /// The manual page does not document it.
    pub misc: *mut c_void,
    /// The client's own data, given at registration. It is typically the
    /// soft state pointer of the driver instance.
    pub client_data: *mut c_void,
    /// The client information the handler fills in for
    /// [`CS_EVENT_CLIENT_INFO`].
    pub client_info: client_info_t,
}

// Layout on amd64 with LP64. client_handle 4 at 0 and 4 bytes of padding,
// five pointers info, mtdrequest, buffer, misc and client_data at 8, 16, 24,
// 32 and 40, and client_info 208 at 48, which ends at 256; total 256.
const _: () = assert!(core::mem::size_of::<event_callback_args_t>() == 256);
const _: () = assert!(core::mem::offset_of!(event_callback_args_t, client_handle) == 0);
const _: () = assert!(core::mem::offset_of!(event_callback_args_t, info) == 8);
const _: () = assert!(core::mem::offset_of!(event_callback_args_t, mtdrequest) == 16);
const _: () = assert!(core::mem::offset_of!(event_callback_args_t, buffer) == 24);
const _: () = assert!(core::mem::offset_of!(event_callback_args_t, misc) == 32);
const _: () = assert!(core::mem::offset_of!(event_callback_args_t, client_data) == 40);
const _: () = assert!(core::mem::offset_of!(event_callback_args_t, client_info) == 48);

// --- the event handler ---------------------------------------------------

/// `csx_event_handler(9E)`. illumos architecture specific (illumos DDI).
///
/// The signature of a PC Card driver's event handler. Card Services calls
/// it with the event code in `event`, the priority in `priority`, which has
/// [`CS_EVENT_PRI_HIGH`] or [`CS_EVENT_PRI_LOW`] set, and a pointer to the
/// [`event_callback_args_t`] of the event in `args`. Every instance of a
/// driver registers one, and on a multiple function card each instance
/// associated with a function receives every event.
///
/// The handler returns `CS_SUCCESS` when it handled the event,
/// `CS_UNSUPPORTED_EVENT` when it does not support the event, or, as the
/// page puts it, `CS_FAILURE` when an error occurred, a name that no header
/// defines. It is called from high-level interrupt context for high
/// priority events and from kernel context for low priority events.
///
/// C has no typedef for this signature. The `event_handler` member of
/// `client_reg_t` is declared as a pointer to the variadic
/// [`csfunction_t`](crate::client::csfunction_t), so a handler of this type
/// must be cast to that type before it is stored there, as C drivers do.
pub type csx_event_handler_t =
    unsafe extern "C" fn(event: event_t, priority: i32, args: *mut event_callback_args_t) -> i32;

/// The event is delivered at low priority, below lock level
/// (`CS_EVENT_PRI_LOW`).
pub const CS_EVENT_PRI_LOW: i32 = 0x0001;
/// The event is delivered at high priority, above lock level
/// (`CS_EVENT_PRI_HIGH`).
pub const CS_EVENT_PRI_HIGH: i32 = 0x0002;

/// The value of [`event_callback_args_t::info`] for
/// [`CS_EVENT_WRITE_PROTECT`] when the card is not write protected
/// (`CS_EVENT_WRITE_PROTECT_WPOFF`).
pub const CS_EVENT_WRITE_PROTECT_WPOFF: usize = 0x0000;
/// The value of [`event_callback_args_t::info`] for
/// [`CS_EVENT_WRITE_PROTECT`] when the card is write protected
/// (`CS_EVENT_WRITE_PROTECT_WPON`).
pub const CS_EVENT_WRITE_PROTECT_WPON: usize = 0xffff;

// --- event codes ---------------------------------------------------------

/// A registration request processed in the background has completed
/// (`CS_EVENT_REGISTRATION_COMPLETE`).
pub const CS_EVENT_REGISTRATION_COMPLETE: event_t = 0x0000_0001;
/// Card Services has received a resume notification from power management
/// (`CS_EVENT_PM_RESUME`).
pub const CS_EVENT_PM_RESUME: event_t = 0x0000_0002;
/// A card has been inserted in a socket (`CS_EVENT_CARD_INSERTION`).
pub const CS_EVENT_CARD_INSERTION: event_t = 0x0000_0004;
/// The READY line of a card has changed from busy to ready
/// (`CS_EVENT_CARD_READY`).
pub const CS_EVENT_CARD_READY: event_t = 0x0000_0008;
/// The battery of a card is weak and needs replacing
/// (`CS_EVENT_BATTERY_LOW`).
pub const CS_EVENT_BATTERY_LOW: event_t = 0x0000_0010;
/// The battery of a card no longer provides operational voltage
/// (`CS_EVENT_BATTERY_DEAD`).
pub const CS_EVENT_BATTERY_DEAD: event_t = 0x0000_0020;
/// A mechanical latch now prevents removal of the card
/// (`CS_EVENT_CARD_LOCK`).
pub const CS_EVENT_CARD_LOCK: event_t = 0x0000_0040;
/// Card Services has received a suspend notification from power management
/// (`CS_EVENT_PM_SUSPEND`).
pub const CS_EVENT_PM_SUSPEND: event_t = 0x0000_0080;
/// A hardware reset has occurred (`CS_EVENT_CARD_RESET`).
pub const CS_EVENT_CARD_RESET: event_t = 0x0000_0100;
/// A mechanical latch now allows removal of the card
/// (`CS_EVENT_CARD_UNLOCK`).
pub const CS_EVENT_CARD_UNLOCK: event_t = 0x0000_0200;
/// A motor has finished ejecting a card from a socket
/// (`CS_EVENT_EJECTION_COMPLETE`).
pub const CS_EVENT_EJECTION_COMPLETE: event_t = 0x0000_0400;
/// A request to eject the card with a motor driven mechanism
/// (`CS_EVENT_EJECTION_REQUEST`).
pub const CS_EVENT_EJECTION_REQUEST: event_t = 0x0000_0800;
/// A queued erase request processed in the background has completed
/// (`CS_EVENT_ERASE_COMPLETE`).
pub const CS_EVENT_ERASE_COMPLETE: event_t = 0x0000_1000;
/// A motor has finished inserting a card in a socket
/// (`CS_EVENT_INSERTION_COMPLETE`).
pub const CS_EVENT_INSERTION_COMPLETE: event_t = 0x0000_8000;
/// A request to insert a card with a motor driven mechanism
/// (`CS_EVENT_INSERTION_REQUEST`).
pub const CS_EVENT_INSERTION_REQUEST: event_t = 0x0001_0000;
/// A reset request processed in the background has completed
/// (`CS_EVENT_RESET_COMPLETE`).
pub const CS_EVENT_RESET_COMPLETE: event_t = 0x0002_0000;
/// A reset is about to occur (`CS_EVENT_RESET_PHYSICAL`).
pub const CS_EVENT_RESET_PHYSICAL: event_t = 0x0004_0000;
/// A client has requested a physical reset (`CS_EVENT_RESET_REQUEST`).
pub const CS_EVENT_RESET_REQUEST: event_t = 0x0008_0000;
/// A request that the client fill in [`event_callback_args_t::client_info`]
/// (`CS_EVENT_CLIENT_INFO`).
pub const CS_EVENT_CLIENT_INFO: event_t = 0x0020_0000;
/// The write protect state of the card has changed. The new state is in
/// [`event_callback_args_t::info`] (`CS_EVENT_WRITE_PROTECT`).
pub const CS_EVENT_WRITE_PROTECT: event_t = 0x0100_0000;
/// A card has been removed from a socket. The event is delivered first at
/// high priority and then at low priority. It has the highest value of the
/// real events so that it reaches the client after every other queued event
/// (`CS_EVENT_CARD_REMOVAL`).
pub const CS_EVENT_CARD_REMOVAL: event_t = 0x1000_0000;

// --- text conversion -----------------------------------------------------

/// The size of the text buffer of an [`event2text_t`], large enough for the
/// names of all events at once (`CS_EVENT_MAX_BUFSIZE`).
pub const CS_EVENT_MAX_BUFSIZE: usize = 512;
/// The size of the text buffer of an [`error2text_t`]
/// (`CS_ERROR_MAX_BUFSIZE`).
pub const CS_ERROR_MAX_BUFSIZE: usize = 64;

/// `csx_Event2Text(9F)`. illumos DDI Specific (illumos DDI).
///
/// The request and result of [`csx_Event2Text`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct event2text_t {
    /// The event code to describe.
    pub event: event_t,
    /// On return, a string naming the event.
    pub text: [c_char; CS_EVENT_MAX_BUFSIZE],
}

// Layout on amd64 with LP64. event 4 at 0 and text 512 at 4, which ends at
// 516; the alignment is 4, so the total is 516.
const _: () = assert!(core::mem::size_of::<event2text_t>() == 516);
const _: () = assert!(core::mem::offset_of!(event2text_t, event) == 0);
const _: () = assert!(core::mem::offset_of!(event2text_t, text) == 4);

/// `csx_Error2Text(9F)`. illumos DDI Specific (illumos DDI).
///
/// The request and result of [`csx_Error2Text`]. The manual page misspells
/// the second member as `test`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct error2text_t {
    /// The Card Services return code to describe.
    pub item: u32,
    /// On return, the text for the code, or a string of the form
    /// "{unknown Card Services return code}" when the code is not known.
    pub text: [c_char; CS_ERROR_MAX_BUFSIZE],
}

// Layout on amd64 with LP64. item 4 at 0 and text 64 at 4, which ends at
// 68; the alignment is 4, so the total is 68.
const _: () = assert!(core::mem::size_of::<error2text_t>() == 68);
const _: () = assert!(core::mem::offset_of!(error2text_t, item) == 0);
const _: () = assert!(core::mem::offset_of!(error2text_t, text) == 4);

unsafe extern "C" {
    /// `csx_Event2Text(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// An illumos extension that converts the event code in the
    /// [`event`](event2text_t::event) member of the structure at `ev` into a
    /// string naming the event, which it stores in the
    /// [`text`](event2text_t::text) member. It is useful for messages from
    /// an event handler.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS), or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_Event2Text(ev: *mut event2text_t) -> i32;
    /// `csx_Error2Text(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// An illumos extension that converts the Card Services return code in
    /// the [`item`](error2text_t::item) member of the structure at `er` into
    /// text, which it stores in the [`text`](error2text_t::text) member. The
    /// client need not allocate anything else to hold the text.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS), or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_Error2Text(er: *mut error2text_t) -> i32;
}
