//! Memory windows and the speed and size conversions from `sys/cs.h` and
//! `sys/cs_types.h`.
//!
//! A window is a block of system address space that Card Services maps onto
//! the attribute memory, common memory or I/O space of a card. A driver asks
//! for one with [`csx_RequestWindow`], describing it in a [`win_req_t`],
//! and gets back a [`window_handle_t`] for the window and an access handle
//! for its first byte. [`csx_MapMemPage`] selects which offset of the card
//! the window shows, [`csx_ModifyWindow`] changes its type, enable state or
//! access speed, and [`csx_ReleaseWindow`] gives it back.
//!
//! Access speeds use the format of the extended speed byte of the Device
//! ID tuple, and card sizes use the device size format of the same tuple.
//! [`csx_ConvertSpeed`] and [`csx_ConvertSize`] translate between those
//! formats and nanoseconds or bytes, and the manual pages recommend them
//! over building the codes by hand.
//!
//! The `WIN_ACC_*` values describe byte order and access ordering for
//! windows, for the I/O ranges of `csx_RequestIO(9F)` and for
//! `csx_DupHandle(9F)`. In `sys/cs.h` the byte order values lie inside the
//! mask of the ordering values, so [`WIN_ACC_BIG_ENDIAN`] has the same value
//! as [`WIN_ACC_UNORDERED_OK`] and [`WIN_ACC_LITTLE_ENDIAN`] the same as
//! [`WIN_ACC_MERGING_OK`]. The constants are declared as the header defines
//! them.
//!
//! Only the attribute bits that the manual pages name are declared. The
//! header also defines the `WIN_DATA_WIDTH_*` and `WIN_PREFETCH*` bits, the
//! CardBus `WIN_BAR_*` values and the masks, which the pages do not
//! mention and which `csx_RequestWindow(9F)` says must be zero, the flag
//! `WIN_DATA_WIDTH_VALID`, which is internal to Card Services, and
//! `WIN_PAGED`, `WIN_SHARED`, `WIN_FIRST_SHARED` and
//! `WIN_BINDING_SPECIFIC`, which exist for compatibility only and are
//! rejected with `CS_BAD_ATTRIBUTE`.

use crate::access::acc_handle_t;
use crate::client::client_handle_t;

/// `csx_RequestWindow(9F)`. illumos DDI Specific (illumos DDI).
///
/// The handle of a window, a `uint32_t` in C. [`csx_RequestWindow`] returns
/// it, [`csx_ModifyWindow`] and [`csx_MapMemPage`] take it, and
/// [`csx_ReleaseWindow`] frees it.
pub type window_handle_t = u32;

// --- win_req_t -----------------------------------------------------------

/// `csx_RequestWindow(9F)`. illumos DDI Specific (illumos DDI).
///
/// The `Base` member of a [`win_req_t`], an anonymous union in C. On the
/// call it holds the requested base address, which must be zero, and on
/// successful return it holds the access handle for the first byte of the
/// window.
#[repr(C)]
#[derive(Clone, Copy)]
pub union win_req_Base {
    /// The requested base address of the window. It must be set to 0 when
    /// calling [`csx_RequestWindow`].
    pub base: u32,
    /// On successful return from [`csx_RequestWindow`], an access handle for
    /// the first byte of the allocated window. The driver passes it to the
    /// common access functions such as `csx_Get8(9F)` and must not make any
    /// assumptions about its format.
    pub handle: acc_handle_t,
}

/// `csx_RequestWindow(9F)`. illumos DDI Specific (illumos DDI).
///
/// The `win_params` member of a [`win_req_t`], an anonymous union in C.
/// Which member applies depends on whether a memory window or an I/O window
/// is requested.
#[repr(C)]
#[derive(Clone, Copy)]
pub union win_req_win_params {
    /// The access speed of a memory window, in the format of the extended
    /// speed byte of the Device ID tuple. When the mantissa is 0, which the
    /// PC Card 95 standard marks reserved, the low bits are a code for a
    /// speed, with 1 meaning 250 ns, 2 meaning 200 ns, 3 meaning 150 ns and
    /// 4 meaning 100 ns, while 0 and 5 to 7 are reserved. OR in
    /// [`WIN_USE_WAIT`] to ask for a window that supports the WAIT signal.
    /// The page recommends generating the value with [`csx_ConvertSpeed`].
    pub AccessSpeed: u32,
    /// For an I/O window, the number of I/O address lines the card decodes.
    /// Access to an I/O window is not enabled until
    /// `csx_RequestConfiguration(9F)` has succeeded.
    pub IOAddrLines: u32,
}

/// `csx_RequestWindow(9F)`. illumos DDI Specific (illumos DDI).
///
/// Describes a window that a driver asks [`csx_RequestWindow`] for, and on
/// return describes the window it got. The driver sets the type, byte order
/// and ordering in [`Attributes`](Self::Attributes), the size it wants in
/// [`Size`](Self::Size) and the speed or address lines in
/// [`win_params`](Self::win_params). Card Services returns the access handle
/// in [`Base`](Self::Base) and the granted size in [`Size`](Self::Size).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct win_req_t {
    /// Not used in illumos. For portability with other Card Services
    /// implementations it should be set to the logical socket number.
    pub Socket: u32,
    /// The window attributes. Exactly one of [`WIN_MEMORY_TYPE_CM`],
    /// [`WIN_MEMORY_TYPE_AM`] and [`WIN_MEMORY_TYPE_IO`] selects the kind of
    /// window, [`WIN_ENABLE`] must be set to enable it, and the `WIN_ACC_*`
    /// bits give the byte order and the access ordering. All other bits must
    /// be 0. On successful return [`WIN_OFFSET_SIZE`] is set if card offsets
    /// passed to [`csx_MapMemPage`] must be a multiple of the window size.
    pub Attributes: u32,
    /// The requested base address on the call and the access handle for the
    /// window on return. See [`win_req_Base`].
    pub Base: win_req_Base,
    /// The size in bytes of the window. On the call it is the size wanted,
    /// or 0 for the smallest window available, and on successful return it
    /// is the size of the window allocated.
    pub Size: u32,
    /// The access speed of a memory window or the number of address lines
    /// of an I/O window. See [`win_req_win_params`].
    pub win_params: win_req_win_params,
    /// An illumos extension that clients can use to compute the best window
    /// offsets to pass to [`csx_MapMemPage`].
    pub ReqOffset: u32,
}

// Layout on amd64 with LP64. The union win_req_Base holds a uint32_t and a
// pointer, so it is 8 bytes aligned to 8; win_req_win_params holds two
// uint32_t and is 4 bytes. win_req_t is Socket 4 at 0, Attributes 4 at 4,
// Base 8 at 8, Size 4 at 16, win_params 4 at 20 and ReqOffset 4 at 24,
// which ends at 28 and is padded to 32 for the alignment of Base.
const _: () = assert!(core::mem::size_of::<win_req_Base>() == 8);
const _: () = assert!(core::mem::size_of::<win_req_win_params>() == 4);
const _: () = assert!(core::mem::size_of::<win_req_t>() == 32);
const _: () = assert!(core::mem::offset_of!(win_req_t, Socket) == 0);
const _: () = assert!(core::mem::offset_of!(win_req_t, Base) == 8);
const _: () = assert!(core::mem::offset_of!(win_req_t, Size) == 16);
const _: () = assert!(core::mem::offset_of!(win_req_t, win_params) == 20);
const _: () = assert!(core::mem::offset_of!(win_req_t, ReqOffset) == 24);

// --- modify_win_t --------------------------------------------------------

/// `csx_ModifyWindow(9F)`. illumos DDI Specific (illumos DDI).
///
/// The changes that [`csx_ModifyWindow`] applies to a window. Only some of
/// the attributes and the access speed can be changed this way; the base
/// and the size can only be changed by releasing the window and requesting
/// a new one.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct modify_win_t {
    /// The new window attributes. [`WIN_MEMORY_TYPE_CM`] or
    /// [`WIN_MEMORY_TYPE_AM`] maps the window to common or attribute memory,
    /// [`WIN_ENABLE`] must be set to keep the window enabled, and
    /// [`WIN_ACCESS_SPEED_VALID`] must be set when
    /// [`AccessSpeed`](Self::AccessSpeed) holds a value to apply.
    pub Attributes: u32,
    /// The new access speed, in the same format as
    /// [`win_req_win_params::AccessSpeed`]. The page recommends generating
    /// the value with [`csx_ConvertSpeed`].
    pub AccessSpeed: u32,
}

// Layout on amd64 with LP64. Two uint32_t, Attributes at 0 and AccessSpeed
// at 4; total 8.
const _: () = assert!(core::mem::size_of::<modify_win_t>() == 8);
const _: () = assert!(core::mem::offset_of!(modify_win_t, Attributes) == 0);
const _: () = assert!(core::mem::offset_of!(modify_win_t, AccessSpeed) == 4);

// --- map_mem_page_t ------------------------------------------------------

/// `csx_MapMemPage(9F)`. illumos DDI Specific (illumos DDI).
///
/// Says which part of the card memory [`csx_MapMemPage`] maps into a
/// window.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct map_mem_page_t {
    /// The absolute offset in bytes from the beginning of the card of the
    /// memory to map into system memory.
    pub CardOffset: u32,
    /// Used internally by Card Services. Clients must set it to 0.
    pub Page: u32,
}

// Layout on amd64 with LP64. Two uint32_t, CardOffset at 0 and Page at 4;
// total 8.
const _: () = assert!(core::mem::size_of::<map_mem_page_t>() == 8);
const _: () = assert!(core::mem::offset_of!(map_mem_page_t, CardOffset) == 0);
const _: () = assert!(core::mem::offset_of!(map_mem_page_t, Page) == 4);

// --- convert_speed_t and convert_size_t ----------------------------------

/// `csx_ConvertSpeed(9F)`. illumos DDI Specific (illumos DDI).
///
/// The request and result of [`csx_ConvertSpeed`], which converts a device
/// speed between nanoseconds and the device speed format.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct convert_speed_t {
    /// The direction of the conversion, either [`CONVERT_NS_TO_DEVSPEED`] or
    /// [`CONVERT_DEVSPEED_TO_NS`].
    pub Attributes: u32,
    /// A speed in nanoseconds. It is the input with
    /// [`CONVERT_NS_TO_DEVSPEED`] and the result with
    /// [`CONVERT_DEVSPEED_TO_NS`].
    pub nS: u32,
    /// A speed in device speed format. It is the input with
    /// [`CONVERT_DEVSPEED_TO_NS`] and the result with
    /// [`CONVERT_NS_TO_DEVSPEED`].
    pub devspeed: u32,
}

// Layout on amd64 with LP64. Three uint32_t at 0, 4 and 8; total 12.
const _: () = assert!(core::mem::size_of::<convert_speed_t>() == 12);
const _: () = assert!(core::mem::offset_of!(convert_speed_t, Attributes) == 0);
const _: () = assert!(core::mem::offset_of!(convert_speed_t, devspeed) == 8);

/// `csx_ConvertSize(9F)`. illumos DDI Specific (illumos DDI).
///
/// The request and result of [`csx_ConvertSize`], which converts a device
/// size between bytes and the device size format.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct convert_size_t {
    /// The direction of the conversion, either [`CONVERT_BYTES_TO_DEVSIZE`]
    /// or [`CONVERT_DEVSIZE_TO_BYTES`].
    pub Attributes: u32,
    /// A size in bytes. It is the input with [`CONVERT_BYTES_TO_DEVSIZE`]
    /// and the result with [`CONVERT_DEVSIZE_TO_BYTES`].
    pub bytes: u32,
    /// A size in device size format. It is the input with
    /// [`CONVERT_DEVSIZE_TO_BYTES`] and the result with
    /// [`CONVERT_BYTES_TO_DEVSIZE`].
    pub devsize: u32,
}

// Layout on amd64 with LP64. Three uint32_t at 0, 4 and 8; total 12.
const _: () = assert!(core::mem::size_of::<convert_size_t>() == 12);
const _: () = assert!(core::mem::offset_of!(convert_size_t, Attributes) == 0);
const _: () = assert!(core::mem::offset_of!(convert_size_t, devsize) == 8);

// --- window attributes ---------------------------------------------------

/// The window maps common memory (`WIN_MEMORY_TYPE_CM`).
pub const WIN_MEMORY_TYPE_CM: u32 = 0x0000_0000;
/// The window maps attribute memory (`WIN_MEMORY_TYPE_AM`).
pub const WIN_MEMORY_TYPE_AM: u32 = 0x0000_0001;
/// The window maps I/O space (`WIN_MEMORY_TYPE_IO`).
pub const WIN_MEMORY_TYPE_IO: u32 = 0x0000_0020;
/// Enables the window. The client must set it on [`csx_RequestWindow`] and
/// [`csx_ModifyWindow`] for the window to be usable (`WIN_ENABLE`).
pub const WIN_ENABLE: u32 = 0x0000_0004;
/// Set by [`csx_RequestWindow`] on return when the card offsets passed to
/// [`csx_MapMemPage`] must be a multiple of the window size
/// (`WIN_OFFSET_SIZE`).
pub const WIN_OFFSET_SIZE: u32 = 0x0000_0008;
/// Tells [`csx_ModifyWindow`] that [`modify_win_t::AccessSpeed`] holds a
/// speed to apply (`WIN_ACCESS_SPEED_VALID`).
pub const WIN_ACCESS_SPEED_VALID: u32 = 0x0000_0010;
/// ORed into an access speed to ask for a window that supports the WAIT
/// signal. It is part of the speed value, not of the attributes
/// (`WIN_USE_WAIT`).
pub const WIN_USE_WAIT: u32 = 0x80;

/// Byte order attribute meaning that bytes are never swapped, so the data
/// access functions use the host byte order (`WIN_ACC_NEVER_SWAP`).
pub const WIN_ACC_NEVER_SWAP: u32 = 0x0000_0000;
/// Byte order attribute describing a big-endian device. The system swaps
/// bytes when host and device byte order differ. The header gives it the
/// value of [`WIN_ACC_UNORDERED_OK`] (`WIN_ACC_BIG_ENDIAN`).
pub const WIN_ACC_BIG_ENDIAN: u32 = 0x0010_0000;
/// Byte order attribute describing a little-endian device. The system swaps
/// bytes when host and device byte order differ. The header gives it the
/// value of [`WIN_ACC_MERGING_OK`] (`WIN_ACC_LITTLE_ENDIAN`).
pub const WIN_ACC_LITTLE_ENDIAN: u32 = 0x0020_0000;
/// Ordering attribute requiring the CPU to issue data references in program
/// order. This is the default (`WIN_ACC_STRICT_ORDER`).
pub const WIN_ACC_STRICT_ORDER: u32 = 0x0000_0000;
/// Ordering attribute allowing the CPU to reorder data references of any
/// kind (`WIN_ACC_UNORDERED_OK`).
pub const WIN_ACC_UNORDERED_OK: u32 = 0x0010_0000;
/// Ordering attribute allowing the CPU to merge individual stores or loads
/// to consecutive locations, which implies reordering
/// (`WIN_ACC_MERGING_OK`).
pub const WIN_ACC_MERGING_OK: u32 = 0x0020_0000;
/// Ordering attribute allowing the CPU to cache fetched data and reuse it
/// until another store occurs, which implies merging and reordering
/// (`WIN_ACC_LOADCACHING_OK`).
pub const WIN_ACC_LOADCACHING_OK: u32 = 0x0030_0000;
/// Ordering attribute allowing the CPU to keep stored data in the cache and
/// push it to the device later, which implies load caching, merging and
/// reordering (`WIN_ACC_STORECACHING_OK`).
pub const WIN_ACC_STORECACHING_OK: u32 = 0x0040_0000;

// --- conversion directions -----------------------------------------------

/// Converts [`convert_speed_t::nS`] to device speed format
/// (`CONVERT_NS_TO_DEVSPEED`).
pub const CONVERT_NS_TO_DEVSPEED: u32 = 0x0000_0001;
/// Converts [`convert_speed_t::devspeed`] to nanoseconds
/// (`CONVERT_DEVSPEED_TO_NS`).
pub const CONVERT_DEVSPEED_TO_NS: u32 = 0x0000_0002;
/// Converts [`convert_size_t::bytes`] to device size format
/// (`CONVERT_BYTES_TO_DEVSIZE`).
pub const CONVERT_BYTES_TO_DEVSIZE: u32 = 0x0000_0001;
/// Converts [`convert_size_t::devsize`] to bytes
/// (`CONVERT_DEVSIZE_TO_BYTES`).
pub const CONVERT_DEVSIZE_TO_BYTES: u32 = 0x0000_0002;

unsafe extern "C" {
    /// `csx_RequestWindow(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Asks Card Services to assign a block of system address space to the
    /// card of the client `ch`, as described by the [`win_req_t`] that `wr`
    /// points to. On success the window handle is stored in `wh`, and the
    /// structure holds the access handle for the first byte of the window
    /// and the size actually granted. The offset of the card that the window
    /// shows is set afterwards with [`csx_MapMemPage`], and the window
    /// handle must be freed with [`csx_ReleaseWindow`] when the client is
    /// done with it.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_ATTRIBUTE`](crate::consts::CS_BAD_ATTRIBUTE) for invalid
    /// attributes, [`CS_BAD_SPEED`](crate::consts::CS_BAD_SPEED) for an
    /// invalid speed, [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an
    /// invalid client handle, [`CS_BAD_SIZE`](crate::consts::CS_BAD_SIZE)
    /// for an invalid size, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when
    /// the socket is empty,
    /// [`CS_OUT_OF_RESOURCE`](crate::consts::CS_OUT_OF_RESOURCE) when no
    /// window could be allocated, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_RequestWindow(ch: client_handle_t, wh: *mut window_handle_t, wr: *mut win_req_t) -> i32;
    /// `csx_RequestWindow(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Releases the window `wh` that [`csx_RequestWindow`] allocated. No
    /// adapter or socket hardware is changed by this call.
    ///
    /// The return values are those of [`csx_RequestWindow`]. It may be
    /// called from user or kernel context.
    pub fn csx_ReleaseWindow(wh: window_handle_t) -> i32;
    /// `csx_ModifyWindow(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Changes the attributes or the access speed of the window `wh` as the
    /// [`modify_win_t`] that `mw` points to describes. The base and the size
    /// of a window cannot be changed this way; that needs
    /// [`csx_ReleaseWindow`] followed by a new [`csx_RequestWindow`].
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid window
    /// handle, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the socket is
    /// empty, [`CS_BAD_OFFSET`](crate::consts::CS_BAD_OFFSET) or
    /// [`CS_BAD_WINDOW`](crate::consts::CS_BAD_WINDOW) when the window
    /// hardware parameters could not be read or set,
    /// [`CS_BAD_SPEED`](crate::consts::CS_BAD_SPEED) for an invalid access
    /// speed, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_ModifyWindow(wh: window_handle_t, mw: *mut modify_win_t) -> i32;
    /// `csx_MapMemPage(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Maps the memory of the card at the offset given in the
    /// [`map_mem_page_t`] that `mp` points to into the window `wh`, which
    /// came from [`csx_RequestWindow`].
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE) for an invalid
    /// handle, [`CS_BAD_OFFSET`](crate::consts::CS_BAD_OFFSET) for an
    /// invalid offset, [`CS_BAD_PAGE`](crate::consts::CS_BAD_PAGE) when the
    /// page is not zero, [`CS_NO_CARD`](crate::consts::CS_NO_CARD) when the
    /// socket is empty, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_MapMemPage(wh: window_handle_t, mp: *mut map_mem_page_t) -> i32;
    /// `csx_ConvertSpeed(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// An illumos extension that converts a device speed between
    /// nanoseconds and the device speed format, in the direction that the
    /// [`Attributes`](convert_speed_t::Attributes) member of the structure
    /// at `cs` selects.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_SPEED`](crate::consts::CS_BAD_SPEED) for an invalid speed,
    /// [`CS_BAD_ATTRIBUTE`](crate::consts::CS_BAD_ATTRIBUTE) for a bad
    /// direction, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_ConvertSpeed(cs: *mut convert_speed_t) -> i32;
    /// `csx_ConvertSize(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// An illumos extension that converts a device size between bytes and
    /// the device size format, in the direction that the
    /// [`Attributes`](convert_size_t::Attributes) member of the structure at
    /// `cs` selects.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_BAD_SIZE`](crate::consts::CS_BAD_SIZE) for an invalid size, or
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// when no PCMCIA hardware is installed. It may be called from user or
    /// kernel context.
    pub fn csx_ConvertSize(cs: *mut convert_size_t) -> i32;
}
