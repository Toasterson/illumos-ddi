//! The common access functions and access handles from `sys/pctypes.h` and
//! `sys/cs.h`.
//!
//! A PC Card driver never dereferences card memory or I/O ports directly.
//! `csx_RequestWindow(9F)` and `csx_RequestIO(9F)` each return an
//! [`acc_handle_t`] for the start of the resource they allocated, and the
//! driver reads and writes through that handle with the `csx_Get8(9F)`,
//! `csx_Put8(9F)`, `csx_RepGet8(9F)` and `csx_RepPut8(9F)` families, giving
//! a byte offset from the base of the mapped resource. The handle records
//! the byte order and ordering rules the driver asked for with the
//! `WIN_ACC_*` attribute bits, and the access functions apply them.
//! `csx_DupHandle(9F)` is meant to make a second handle with different rules
//! for the same resource, and `csx_GetHandleOffset(9F)` and
//! `csx_SetHandleOffset(9F)` read and set the offset recorded in a handle.
//!
//! One prototype in `sys/cs.h` differs from its manual page.
//! `csx_Get64(9F)` shows a `uint64_t` offset on the page, while the header
//! and the definition in `cs_stubs.c` take a `uint32_t`, and the
//! declaration here follows the header. Unlike the rest of the interface,
//! the access and handle functions are implemented in `cs_stubs.c` itself
//! and do not depend on the Card Services module being loaded.

use ddi_sys::c_void;

/// `csx_Get8(9F)`. illumos DDI Specific (illumos DDI).
///
/// A data access handle for the common access functions, a `void *` in C.
/// It comes back from `csx_RequestIO(9F)` in the `handle` member of
/// [`io_req_t::BasePort1`](crate::io::io_req_t::BasePort1) and
/// [`io_req_t::BasePort2`](crate::io::io_req_t::BasePort2), from
/// `csx_RequestWindow(9F)` in
/// [`win_req_Base::handle`](crate::window::win_req_Base::handle), and from
/// [`csx_DupHandle`]. A driver must not make any assumptions about the
/// format of the value.
pub type acc_handle_t = *mut c_void;

/// The `flags` value for the `csx_RepGet8(9F)` and `csx_RepPut8(9F)`
/// families that advances the device offset by one datum after each
/// access. It has the value of `DDI_DEV_AUTOINCR` (`CS_DEV_AUTOINCR`).
pub const CS_DEV_AUTOINCR: u32 = ddi_sys::DDI_DEV_AUTOINCR;
/// The `flags` value for the `csx_RepGet8(9F)` and `csx_RepPut8(9F)`
/// families that uses the same device offset for every datum, which is what
/// a driver wants when draining or filling a data register. It has the
/// value of `DDI_DEV_NO_AUTOINCR` (`CS_DEV_NO_AUTOINCR`).
pub const CS_DEV_NO_AUTOINCR: u32 = ddi_sys::DDI_DEV_NO_AUTOINCR;

unsafe extern "C" {
    // --- single datum reads ------------------------------------------------
    /// `csx_Get8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Reads 8 bits from the mapped memory or device register that `handle`
    /// represents, at `offset` bytes from the base of the mapped resource.
    /// The handle is one returned by `csx_RequestIO(9F)`,
    /// `csx_RequestWindow(9F)` or [`csx_DupHandle`]. The value read is
    /// returned.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_Get8(handle: acc_handle_t, offset: u32) -> u8;
    /// `csx_Get8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Reads 16 bits from the mapped memory or device register that
    /// `handle` represents, at `offset` bytes from the base of the mapped
    /// resource. The value is translated to keep a consistent view between
    /// host and device according to the byte order recorded in the handle,
    /// which may mean swapping bytes when host and device differ. The value
    /// read is returned.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_Get16(handle: acc_handle_t, offset: u32) -> u16;
    /// `csx_Get8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Reads 32 bits from the mapped memory or device register that
    /// `handle` represents, at `offset` bytes from the base of the mapped
    /// resource, and translates the value according to the byte order
    /// recorded in the handle. The value read is returned.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_Get32(handle: acc_handle_t, offset: u32) -> u32;
    /// `csx_Get8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Reads 64 bits from the mapped memory or device register that
    /// `handle` represents, at `offset` bytes from the base of the mapped
    /// resource, and translates the value according to the byte order
    /// recorded in the handle. The value read is returned. The manual page
    /// shows `offset` as a `uint64_t`, but `sys/cs.h` and the definition
    /// take a `uint32_t`, as the other functions of the family do.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_Get64(handle: acc_handle_t, offset: u32) -> u64;

    // --- single datum writes -----------------------------------------------
    /// `csx_Put8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Writes the 8 bits in `value` to the mapped memory or device register
    /// that `handle` represents, at `offset` bytes from the base of the
    /// mapped resource. The handle is one returned by `csx_RequestIO(9F)`,
    /// `csx_RequestWindow(9F)` or [`csx_DupHandle`].
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_Put8(handle: acc_handle_t, offset: u32, value: u8);
    /// `csx_Put8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Writes the 16 bits in `value` to the mapped memory or device
    /// register that `handle` represents, at `offset` bytes from the base
    /// of the mapped resource. The value is translated from host byte order
    /// to the byte order recorded in the handle before it is written.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_Put16(handle: acc_handle_t, offset: u32, value: u16);
    /// `csx_Put8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Writes the 32 bits in `value` to the mapped memory or device
    /// register that `handle` represents, at `offset` bytes from the base
    /// of the mapped resource, translating the value to the byte order
    /// recorded in the handle.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_Put32(handle: acc_handle_t, offset: u32, value: u32);
    /// `csx_Put8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Writes the 64 bits in `value` to the mapped memory or device
    /// register that `handle` represents, at `offset` bytes from the base
    /// of the mapped resource, translating the value to the byte order
    /// recorded in the handle.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_Put64(handle: acc_handle_t, offset: u32, value: u64);

    // --- repeated reads ----------------------------------------------------
    /// `csx_RepGet8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Performs `repcount` reads of 8 bits each from the mapped memory or
    /// device register that `handle` represents, starting at `offset` bytes
    /// from the base of the mapped resource, and stores the data
    /// consecutively in the buffer at `hostaddr`. When `flags` is
    /// [`CS_DEV_AUTOINCR`] the device offset advances after each datum, and
    /// when it is [`CS_DEV_NO_AUTOINCR`] every datum is read from the same
    /// offset, which suits a data register.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_RepGet8(handle: acc_handle_t, hostaddr: *mut u8, offset: u32, repcount: u32, flags: u32);
    /// `csx_RepGet8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Performs `repcount` reads of 16 bits each from the resource that
    /// `handle` represents, starting at `offset` bytes from its base, and
    /// stores the data consecutively at `hostaddr`. Each datum is translated
    /// according to the byte order recorded in the handle. `flags` is
    /// [`CS_DEV_AUTOINCR`] or [`CS_DEV_NO_AUTOINCR`], as for
    /// [`csx_RepGet8`].
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_RepGet16(handle: acc_handle_t, hostaddr: *mut u16, offset: u32, repcount: u32, flags: u32);
    /// `csx_RepGet8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Performs `repcount` reads of 32 bits each from the resource that
    /// `handle` represents, starting at `offset` bytes from its base, and
    /// stores the data consecutively at `hostaddr`. Each datum is translated
    /// according to the byte order recorded in the handle. `flags` is
    /// [`CS_DEV_AUTOINCR`] or [`CS_DEV_NO_AUTOINCR`], as for
    /// [`csx_RepGet8`].
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_RepGet32(handle: acc_handle_t, hostaddr: *mut u32, offset: u32, repcount: u32, flags: u32);
    /// `csx_RepGet8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Performs `repcount` reads of 64 bits each from the resource that
    /// `handle` represents, starting at `offset` bytes from its base, and
    /// stores the data consecutively at `hostaddr`. Each datum is translated
    /// according to the byte order recorded in the handle. `flags` is
    /// [`CS_DEV_AUTOINCR`] or [`CS_DEV_NO_AUTOINCR`], as for
    /// [`csx_RepGet8`].
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_RepGet64(handle: acc_handle_t, hostaddr: *mut u64, offset: u32, repcount: u32, flags: u32);

    // --- repeated writes ---------------------------------------------------
    /// `csx_RepPut8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Performs `repcount` writes of 8 bits each to the mapped memory or
    /// device register that `handle` represents, starting at `offset` bytes
    /// from the base of the mapped resource, taking the data consecutively
    /// from the buffer at `hostaddr`. When `flags` is [`CS_DEV_AUTOINCR`]
    /// the device offset advances after each datum, and when it is
    /// [`CS_DEV_NO_AUTOINCR`] every datum goes to the same offset, which
    /// suits a data register. The C prototype takes a pointer to mutable
    /// data even though the buffer is only read.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_RepPut8(handle: acc_handle_t, hostaddr: *mut u8, offset: u32, repcount: u32, flags: u32);
    /// `csx_RepPut8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Performs `repcount` writes of 16 bits each to the resource that
    /// `handle` represents, starting at `offset` bytes from its base, taking
    /// the data consecutively from `hostaddr`. Each datum is translated to
    /// the byte order recorded in the handle. `flags` is
    /// [`CS_DEV_AUTOINCR`] or [`CS_DEV_NO_AUTOINCR`], as for
    /// [`csx_RepPut8`].
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_RepPut16(handle: acc_handle_t, hostaddr: *mut u16, offset: u32, repcount: u32, flags: u32);
    /// `csx_RepPut8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Performs `repcount` writes of 32 bits each to the resource that
    /// `handle` represents, starting at `offset` bytes from its base, taking
    /// the data consecutively from `hostaddr`. Each datum is translated to
    /// the byte order recorded in the handle. `flags` is
    /// [`CS_DEV_AUTOINCR`] or [`CS_DEV_NO_AUTOINCR`], as for
    /// [`csx_RepPut8`].
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_RepPut32(handle: acc_handle_t, hostaddr: *mut u32, offset: u32, repcount: u32, flags: u32);
    /// `csx_RepPut8(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Performs `repcount` writes of 64 bits each to the resource that
    /// `handle` represents, starting at `offset` bytes from its base, taking
    /// the data consecutively from `hostaddr`. Each datum is translated to
    /// the byte order recorded in the handle. `flags` is
    /// [`CS_DEV_AUTOINCR`] or [`CS_DEV_NO_AUTOINCR`], as for
    /// [`csx_RepPut8`].
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn csx_RepPut64(handle: acc_handle_t, hostaddr: *mut u64, offset: u32, repcount: u32, flags: u32);

    // --- handle functions --------------------------------------------------
    /// `csx_GetMappedAddr(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Stores in `addr` the mapped virtual address or the mapped I/O port
    /// number that the access handle `handle` represents. The handle is one
    /// returned by `csx_RequestIO(9F)`, `csx_RequestWindow(9F)` or
    /// [`csx_DupHandle`].
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS) when
    /// the caller may access the resulting address or port directly, and
    /// the implementation in `cs_stubs.c` always does. The
    /// page documents a `CS_FAILURE` result for the case where it may not
    /// and every access must go through the common access functions, but
    /// no header defines that name, so a driver should treat any other
    /// value except
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// that way. It may be called from user, kernel or interrupt context.
    pub fn csx_GetMappedAddr(handle: acc_handle_t, addr: *mut *mut c_void) -> i32;
    /// `csx_DupHandle(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Duplicates the access handle `handle1`, which came from
    /// `csx_RequestIO(9F)` or `csx_RequestWindow(9F)`, into a new handle
    /// stored in `handle2`. The new handle gets the access attributes in
    /// `flags`, a combination of one byte order value, which is
    /// [`WIN_ACC_NEVER_SWAP`](crate::window::WIN_ACC_NEVER_SWAP),
    /// [`WIN_ACC_BIG_ENDIAN`](crate::window::WIN_ACC_BIG_ENDIAN) or
    /// [`WIN_ACC_LITTLE_ENDIAN`](crate::window::WIN_ACC_LITTLE_ENDIAN), and
    /// one ordering value from
    /// [`WIN_ACC_STRICT_ORDER`](crate::window::WIN_ACC_STRICT_ORDER) to
    /// [`WIN_ACC_STORECACHING_OK`](crate::window::WIN_ACC_STORECACHING_OK).
    /// The ordering values are advisory. Both handles stay active and both
    /// must be freed with [`csx_FreeHandle`] when they are no longer
    /// needed.
    ///
    /// The page documents [`CS_SUCCESS`](crate::consts::CS_SUCCESS),
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// and a `CS_FAILURE` that no header defines, for a bad `flags` value or
    /// a handle that could not be duplicated. The implementation in
    /// `cs_stubs.c` does not duplicate anything and always returns
    /// [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE). It may be called
    /// from user or kernel context.
    pub fn csx_DupHandle(handle1: acc_handle_t, handle2: *mut acc_handle_t, flags: u32) -> i32;
    /// `csx_FreeHandle(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Frees the access handle that `handle` points to. For a handle made by
    /// [`csx_DupHandle`] only the storage of the handle is freed and the
    /// resources of the original handle are left alone. For a handle made by
    /// a request function the resources associated with it are released.
    ///
    /// The page documents [`CS_SUCCESS`](crate::consts::CS_SUCCESS) and
    /// [`CS_UNSUPPORTED_FUNCTION`](crate::consts::CS_UNSUPPORTED_FUNCTION)
    /// as results. The implementation in `cs_stubs.c` frees nothing and
    /// always returns [`CS_BAD_HANDLE`](crate::consts::CS_BAD_HANDLE). It
    /// may be called from user or kernel context.
    pub fn csx_FreeHandle(handle: *mut acc_handle_t) -> i32;
    /// `csx_GetHandleOffset(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Stores the current offset of the access handle `handle` in the
    /// integer that `offset` points to. The page names
    /// `csx_RequestIO(9F)` and `csx_RequestIRQ(9F)` as the sources of the
    /// handle, although only the former and `csx_RequestWindow(9F)` return
    /// one.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS). It
    /// may be called from user or kernel context.
    pub fn csx_GetHandleOffset(handle: acc_handle_t, offset: *mut u32) -> i32;
    /// `csx_SetHandleOffset(9F)`. illumos DDI Specific (illumos DDI).
    ///
    /// Sets the current offset of the access handle `handle` to `offset`.
    /// The page names `csx_RequestIO(9F)` and `csx_RequestIRQ(9F)` as the
    /// sources of the handle, as [`csx_GetHandleOffset`] does. The common
    /// access functions do not add this offset to the ones they are given.
    ///
    /// The function returns [`CS_SUCCESS`](crate::consts::CS_SUCCESS). It
    /// may be called from user or kernel context.
    pub fn csx_SetHandleOffset(handle: acc_handle_t, offset: u32) -> i32;
}
