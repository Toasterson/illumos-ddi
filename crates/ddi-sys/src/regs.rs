//! Programmed I/O to device registers, from the register subset of
//! `sys/sunddi.h`.
//!
//! A driver discovers its register sets with `ddi_dev_nregs(9F)` and
//! `ddi_dev_regsize(9F)`, maps one with `ddi_regs_map_setup(9F)`, and then
//! reads and writes it through the returned handle with the `ddi_get*`,
//! `ddi_put*`, `ddi_rep_get*` and `ddi_rep_put*` functions. Going through
//! the handle rather than dereferencing the address directly is what lets
//! the framework apply the byte order and ordering rules recorded in the
//! access attributes, and what makes fault management possible, because the
//! handle is where an access fault is recorded for `ddi_check_acc_handle(9F)`
//! and `ddi_fm_acc_err_get(9F)` to find later.
//!
//! The `ddi_peek(9F)` and `ddi_poke(9F)` families take a different route.
//! They go through the parent nexus to touch an arbitrary virtual address
//! cautiously, and report a bus fault as a return value instead of taking
//! the machine down. They exist for the probing phase of `probe(9E)` and
//! `attach(9E)`, before a handle exists.
//!
//! On amd64 every function in this module is a real entry point, either in
//! `uts/intel/ml/ddi_i86_asm.S` for the handle-based accessors or in
//! `uts/common/os/sunddi.c` for the rest. None of them is a macro in
//! `sys/sunddi.h`, so a Rust caller links to exactly the symbol the C
//! prototype names.
//!
//! Three families from the same header are left out on purpose. The
//! `ddi_io_get8(9F)`, `ddi_io_put8(9F)`, `ddi_io_rep_get8(9F)` and
//! `ddi_io_rep_put8(9F)` pages, and the `ddi_mem_get8(9F)`,
//! `ddi_mem_put8(9F)`, `ddi_mem_rep_get8(9F)` and `ddi_mem_rep_put8(9F)`
//! pages, state in their interface level section that the functions
//! themselves are obsolete and replaced by the plain `ddi_get8` and
//! `ddi_rep_get8` families, and none of them appears in the `ddi` mapfile.
//! `ddi_map_regs(9F)` is obsolete for the same reason. The access handle
//! type and the `ddi_device_acc_attr(9S)` structure with its `DDI_*_ACC`
//! constants live in [`crate::types`].

use crate::types::*;

/// A `ddi_regs_map_setup(9F)` return value meaning the mapping cannot be
/// enabled because it conflicts with another mapping that is already enabled
/// (`DDI_REGS_ACC_CONFLICT`). The manual page advises checking for anything
/// other than `DDI_SUCCESS` rather than for specific failure codes, because
/// `DDI_ME_RNUMBER_RANGE` from `sys/ddimapreq.h` is not reported on every
/// platform and may overlap with `DDI_FAILURE`.
pub const DDI_REGS_ACC_CONFLICT: c_int = -10;

/// The `flags` value for the `ddi_rep_get8(9F)` and `ddi_rep_put8(9F)`
/// families that keeps the device address fixed for every datum, which is
/// what a driver wants when draining or filling a single data register
/// (`DDI_DEV_NO_AUTOINCR`).
pub const DDI_DEV_NO_AUTOINCR: uint_t = 0x0000;
/// The `flags` value for the `ddi_rep_get8(9F)` and `ddi_rep_put8(9F)`
/// families that advances the device address by one datum after each
/// access, so that the device side is treated as a memory buffer
/// (`DDI_DEV_AUTOINCR`).
pub const DDI_DEV_AUTOINCR: uint_t = 0x0001;

unsafe extern "C" {
    // --- register set discovery -------------------------------------------
    /// `ddi_dev_nregs(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports how many register sets the device node `dip` has, which is
    /// the number of tuples in its `reg` property. The count is stored in
    /// the integer that `resultp` points to. The function returns
    /// `DDI_SUCCESS` when the count was stored and `DDI_FAILURE` when the
    /// device has no registers at all.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_dev_nregs(dip: dev_info_t, resultp: *mut c_int) -> c_int;
    /// `ddi_dev_regsize(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports the size in bytes of register set `rnumber` of the device
    /// node `dip`. Register sets are numbered from zero in the order of the
    /// `reg` property; `ddi_dev_nregs` tells how many there are. The size is
    /// stored in the `off_t` that `resultp` points to. This is the way to
    /// learn the size of a register set whose extent is only known to the
    /// firmware, such as a frame buffer.
    ///
    /// The function returns `DDI_SUCCESS` when the size was stored and
    /// `DDI_FAILURE` when `rnumber` does not name an existing register set.
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_dev_regsize(dip: dev_info_t, rnumber: uint_t, resultp: *mut off_t) -> c_int;

    // --- mapping ------------------------------------------------------------
    /// `ddi_regs_map_setup(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Maps register set `rnumber` of the device node `dip` so that the
    /// driver can access it with the handle-based accessors in this module.
    /// `offset` is the starting byte within the register set and `len` is
    /// the number of bytes to map. A non-zero `len` overrides the length
    /// recorded in the `reg` property, and when both `offset` and `len` are
    /// zero the whole set is mapped. `accattrp` points to the
    /// `ddi_device_acc_attr(9S)` structure describing the byte order,
    /// ordering and fault handling the driver wants for this mapping.
    ///
    /// On success the base address of the mapping is stored in `addrp` and
    /// the new access handle in `handlep`. The address is only meaningful
    /// as the `dev_addr` argument of the accessor functions, with an offset
    /// of at most `len` added; the handle is opaque and must not be
    /// interpreted. For PCI and PCI Express devices the first base address
    /// register with a non-zero value is register set 1, because set 0 is
    /// the configuration space tuple.
    ///
    /// The function returns `DDI_SUCCESS` when the mapping was set up,
    /// `DDI_FAILURE` for an invalid register number, offset or length,
    /// `DDI_ME_RNUMBER_RANGE` on platforms that report an invalid register
    /// number or a missing `reg` property that way, and
    /// [`DDI_REGS_ACC_CONFLICT`] when the mapping conflicts with one that is
    /// already enabled. Drivers should test for anything other than
    /// `DDI_SUCCESS`. It must be called from user or kernel context.
    pub fn ddi_regs_map_setup(
        dip: dev_info_t,
        rnumber: uint_t,
        addrp: *mut caddr_t,
        offset: offset_t,
        len: offset_t,
        accattrp: *const ddi_device_acc_attr_t,
        handlep: *mut ddi_acc_handle_t,
    ) -> c_int;
    /// `ddi_regs_map_free(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Frees the mapping represented by the access handle that `handlep`
    /// points to and releases the system resources behind it. The handle is
    /// invalid afterwards, and so is the address returned by
    /// `ddi_regs_map_setup`. Drivers call it while detaching. It must be
    /// called from user or kernel context.
    pub fn ddi_regs_map_free(handlep: *mut ddi_acc_handle_t);

    // --- single datum accessors --------------------------------------------
    /// `ddi_get8(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads one byte from the device address `dev_addr`, which must lie
    /// within the mapping that `handle` describes. The handle may come from
    /// `ddi_regs_map_setup`, from `ddi_dma_mem_alloc(9F)`, or from any other
    /// function that returns an access handle. The byte order and ordering
    /// rules recorded in the handle are applied to the access, which for a
    /// single byte means only the ordering rules matter. The value read is
    /// returned.
    ///
    /// The page names `ddi_getb`, `ddi_getw`, `ddi_getl` and `ddi_getll` as
    /// the obsolete spellings that `ddi_get8`, `ddi_get16`, `ddi_get32` and
    /// `ddi_get64` replace. The function may be called from user, kernel or
    /// interrupt context, including high-level interrupt context on the
    /// ISA bus and, under the conditions described in `pci(5)`, on PCI.
    pub fn ddi_get8(handle: ddi_acc_handle_t, dev_addr: *const u8) -> u8;
    /// `ddi_get16(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads a 16-bit value from the device address `dev_addr`, which must
    /// be aligned to two bytes and lie within the mapping that `handle`
    /// describes. The value is translated to host byte order according to
    /// the endianness recorded in the handle, so a driver sees the same
    /// number regardless of the byte order the device uses. The value read
    /// is returned. This is the replacement for the obsolete `ddi_getw`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_get16(handle: ddi_acc_handle_t, dev_addr: *const u16) -> u16;
    /// `ddi_get32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads a 32-bit value from the device address `dev_addr`, which must
    /// be aligned to four bytes and lie within the mapping that `handle`
    /// describes. The value is translated to host byte order according to
    /// the endianness recorded in the handle. The value read is returned.
    /// This is the replacement for the obsolete `ddi_getl`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_get32(handle: ddi_acc_handle_t, dev_addr: *const u32) -> u32;
    /// `ddi_get64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reads a 64-bit value from the device address `dev_addr`, which must
    /// be aligned to eight bytes and lie within the mapping that `handle`
    /// describes. The value is translated to host byte order according to
    /// the endianness recorded in the handle. The value read is returned.
    /// This is the replacement for the obsolete `ddi_getll`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_get64(handle: ddi_acc_handle_t, dev_addr: *const u64) -> u64;
    /// `ddi_put8(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Writes the byte `value` to the device address `dev_addr`, which must
    /// lie within the mapping that `handle` describes. The ordering rules
    /// recorded in the handle are applied, so with `DDI_STRICTORDER_ACC`
    /// the write is not reordered with respect to other accesses through
    /// the same handle. Nothing is returned; if the handle was set up with
    /// `DDI_FLAGERR_ACC` a fault is recorded in it for
    /// [`ddi_check_acc_handle`] to report.
    ///
    /// The page names `ddi_putb`, `ddi_putw`, `ddi_putl` and `ddi_putll` as
    /// the obsolete spellings that `ddi_put8`, `ddi_put16`, `ddi_put32` and
    /// `ddi_put64` replace. The function may be called from user, kernel or
    /// interrupt context.
    pub fn ddi_put8(handle: ddi_acc_handle_t, dev_addr: *mut u8, value: u8);
    /// `ddi_put16(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Writes the 16-bit `value` to the device address `dev_addr`, which
    /// must be aligned to two bytes and lie within the mapping that `handle`
    /// describes. The value is translated from host byte order to the byte
    /// order recorded in the handle before it is written. This is the
    /// replacement for the obsolete `ddi_putw`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_put16(handle: ddi_acc_handle_t, dev_addr: *mut u16, value: u16);
    /// `ddi_put32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Writes the 32-bit `value` to the device address `dev_addr`, which
    /// must be aligned to four bytes and lie within the mapping that
    /// `handle` describes. The value is translated from host byte order to
    /// the byte order recorded in the handle before it is written. This is
    /// the replacement for the obsolete `ddi_putl`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_put32(handle: ddi_acc_handle_t, dev_addr: *mut u32, value: u32);
    /// `ddi_put64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Writes the 64-bit `value` to the device address `dev_addr`, which
    /// must be aligned to eight bytes and lie within the mapping that
    /// `handle` describes. The value is translated from host byte order to
    /// the byte order recorded in the handle before it is written. This is
    /// the replacement for the obsolete `ddi_putll`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_put64(handle: ddi_acc_handle_t, dev_addr: *mut u64, value: u64);

    // --- repeated accessors -------------------------------------------------
    /// `ddi_rep_get8(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs `repcount` byte reads from the device address `dev_addr`
    /// through `handle` and stores the results one after another in the
    /// host buffer `host_addr`. When `flags` is [`DDI_DEV_AUTOINCR`] the
    /// device address is advanced by one byte after each read, so that a
    /// region of device memory is copied to the host. When `flags` is
    /// [`DDI_DEV_NO_AUTOINCR`] every read is made from the same device
    /// address, which is the right choice for draining a FIFO data register.
    /// The ordering rules recorded in the handle apply to every access.
    ///
    /// The page names `ddi_rep_getb`, `ddi_rep_getw`, `ddi_rep_getl` and
    /// `ddi_rep_getll` as the obsolete spellings that `ddi_rep_get8`,
    /// `ddi_rep_get16`, `ddi_rep_get32` and `ddi_rep_get64` replace. The
    /// function returns nothing and may be called from user, kernel or
    /// interrupt context.
    pub fn ddi_rep_get8(
        handle: ddi_acc_handle_t,
        host_addr: *mut u8,
        dev_addr: *const u8,
        repcount: size_t,
        flags: uint_t,
    );
    /// `ddi_rep_get16(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs `repcount` 16-bit reads from the device address `dev_addr`
    /// through `handle` and stores the results in the host buffer
    /// `host_addr`. Both addresses must be aligned to two bytes. Each value
    /// is translated to host byte order according to the endianness recorded
    /// in the handle. With [`DDI_DEV_AUTOINCR`] in `flags` the device
    /// address advances by two bytes after each read; with
    /// [`DDI_DEV_NO_AUTOINCR`] it stays fixed. This is the replacement for
    /// the obsolete `ddi_rep_getw`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_rep_get16(
        handle: ddi_acc_handle_t,
        host_addr: *mut u16,
        dev_addr: *const u16,
        repcount: size_t,
        flags: uint_t,
    );
    /// `ddi_rep_get32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs `repcount` 32-bit reads from the device address `dev_addr`
    /// through `handle` and stores the results in the host buffer
    /// `host_addr`. Both addresses must be aligned to four bytes. Each value
    /// is translated to host byte order according to the endianness recorded
    /// in the handle. With [`DDI_DEV_AUTOINCR`] in `flags` the device
    /// address advances by four bytes after each read; with
    /// [`DDI_DEV_NO_AUTOINCR`] it stays fixed. This is the replacement for
    /// the obsolete `ddi_rep_getl`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_rep_get32(
        handle: ddi_acc_handle_t,
        host_addr: *mut u32,
        dev_addr: *const u32,
        repcount: size_t,
        flags: uint_t,
    );
    /// `ddi_rep_get64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs `repcount` 64-bit reads from the device address `dev_addr`
    /// through `handle` and stores the results in the host buffer
    /// `host_addr`. Both addresses must be aligned to eight bytes. Each
    /// value is translated to host byte order according to the endianness
    /// recorded in the handle. With [`DDI_DEV_AUTOINCR`] in `flags` the
    /// device address advances by eight bytes after each read; with
    /// [`DDI_DEV_NO_AUTOINCR`] it stays fixed. This is the replacement for
    /// the obsolete `ddi_rep_getll`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_rep_get64(
        handle: ddi_acc_handle_t,
        host_addr: *mut u64,
        dev_addr: *const u64,
        repcount: size_t,
        flags: uint_t,
    );
    /// `ddi_rep_put8(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs `repcount` byte writes to the device address `dev_addr`
    /// through `handle`, taking the values one after another from the host
    /// buffer `host_addr`. When `flags` is [`DDI_DEV_AUTOINCR`] the device
    /// address is advanced by one byte after each write, so that a host
    /// buffer is copied into device memory. When `flags` is
    /// [`DDI_DEV_NO_AUTOINCR`] every write goes to the same device address,
    /// which is the right choice for filling a FIFO data register. The
    /// ordering rules recorded in the handle apply to every access.
    ///
    /// The page names `ddi_rep_putb`, `ddi_rep_putw`, `ddi_rep_putl` and
    /// `ddi_rep_putll` as the obsolete spellings that `ddi_rep_put8`,
    /// `ddi_rep_put16`, `ddi_rep_put32` and `ddi_rep_put64` replace. The
    /// function returns nothing and may be called from user, kernel or
    /// interrupt context.
    pub fn ddi_rep_put8(
        handle: ddi_acc_handle_t,
        host_addr: *const u8,
        dev_addr: *mut u8,
        repcount: size_t,
        flags: uint_t,
    );
    /// `ddi_rep_put16(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs `repcount` 16-bit writes to the device address `dev_addr`
    /// through `handle`, taking the values from the host buffer
    /// `host_addr`. Both addresses must be aligned to two bytes. Each value
    /// is translated from host byte order to the byte order recorded in the
    /// handle before it is written. With [`DDI_DEV_AUTOINCR`] in `flags`
    /// the device address advances by two bytes after each write; with
    /// [`DDI_DEV_NO_AUTOINCR`] it stays fixed. This is the replacement for
    /// the obsolete `ddi_rep_putw`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_rep_put16(
        handle: ddi_acc_handle_t,
        host_addr: *const u16,
        dev_addr: *mut u16,
        repcount: size_t,
        flags: uint_t,
    );
    /// `ddi_rep_put32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs `repcount` 32-bit writes to the device address `dev_addr`
    /// through `handle`, taking the values from the host buffer
    /// `host_addr`. Both addresses must be aligned to four bytes. Each value
    /// is translated from host byte order to the byte order recorded in the
    /// handle before it is written. With [`DDI_DEV_AUTOINCR`] in `flags`
    /// the device address advances by four bytes after each write; with
    /// [`DDI_DEV_NO_AUTOINCR`] it stays fixed. This is the replacement for
    /// the obsolete `ddi_rep_putl`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_rep_put32(
        handle: ddi_acc_handle_t,
        host_addr: *const u32,
        dev_addr: *mut u32,
        repcount: size_t,
        flags: uint_t,
    );
    /// `ddi_rep_put64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Performs `repcount` 64-bit writes to the device address `dev_addr`
    /// through `handle`, taking the values from the host buffer
    /// `host_addr`. Both addresses must be aligned to eight bytes. Each
    /// value is translated from host byte order to the byte order recorded
    /// in the handle before it is written. With [`DDI_DEV_AUTOINCR`] in
    /// `flags` the device address advances by eight bytes after each write;
    /// with [`DDI_DEV_NO_AUTOINCR`] it stays fixed. This is the replacement
    /// for the obsolete `ddi_rep_putll`.
    ///
    /// It may be called from user, kernel or interrupt context.
    pub fn ddi_rep_put64(
        handle: ddi_acc_handle_t,
        host_addr: *const u64,
        dev_addr: *mut u64,
        repcount: size_t,
        flags: uint_t,
    );

    // --- fault checking -----------------------------------------------------
    /// `ddi_check_acc_handle(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Reports whether a fault has been detected that affects the register
    /// or memory mapping behind `handle`. A fault means the driver can no
    /// longer rely on programmed I/O through that handle. It typically
    /// follows a device that stopped responding, a bus error or a timeout.
    /// After such a fault reads through the handle may return arbitrary
    /// values and writes may or may not take effect. A fault of this kind is
    /// normally fatal to the device and should be reported through
    /// `ddi_dev_report_fault(9F)` with an impact of `DDI_SERVICE_LOST` and
    /// a location of `DDI_DATAPATH_FAULT`, or through the fault management
    /// functions in [`crate::fm`] when the driver uses them.
    ///
    /// Not every implementation can detect every kind of failure, so a
    /// clean result is not a guarantee that the device is reachable, while
    /// a fault indication is a positive sign that something is wrong. The
    /// function returns `DDI_SUCCESS` when no fault affecting the handle is
    /// known and `DDI_FAILURE` when one is. It may be called from user,
    /// kernel or interrupt context. Its companion `ddi_check_dma_handle`
    /// on the same page belongs with the DMA bindings.
    pub fn ddi_check_acc_handle(handle: ddi_acc_handle_t) -> c_int;

    // --- cautious probing ---------------------------------------------------
    /// `ddi_peek8(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cautiously reads one byte from the virtual address `addr` on behalf
    /// of the device node `dip`, using the parent nexus to arrange for any
    /// bus fault to be caught rather than to panic the system. The value is
    /// stored in the location `valuep` points to, unless `valuep` is NULL,
    /// in which case the value is read and discarded. The function exists
    /// for establishing whether a device is present in `probe(9E)` or
    /// `attach(9E)`, before any access handle has been set up.
    ///
    /// The page names `ddi_peekc`, `ddi_peeks`, `ddi_peekl` and `ddi_peekd`
    /// as the obsolete spellings that `ddi_peek8`, `ddi_peek16`,
    /// `ddi_peek32` and `ddi_peek64` replace. The function returns
    /// `DDI_SUCCESS` when the location was read, with `valuep` updated if it
    /// was given, and `DDI_FAILURE` when the address is invalid or the read
    /// faulted, in which case `valuep` is left alone. It may be called from
    /// user, kernel or interrupt context.
    pub fn ddi_peek8(dip: dev_info_t, addr: *const i8, valuep: *mut i8) -> c_int;
    /// `ddi_peek16(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cautiously reads a 16-bit value from the virtual address `addr` on
    /// behalf of the device node `dip`, going through the parent nexus so
    /// that a bus fault is reported instead of taking the system down. The
    /// value is stored through `valuep` unless `valuep` is NULL. No byte
    /// order translation takes place, because there is no access handle to
    /// describe the device. This is the replacement for the obsolete
    /// `ddi_peeks`.
    ///
    /// The function returns `DDI_SUCCESS` when the location was read and
    /// `DDI_FAILURE` when the address is invalid or the read faulted, in
    /// which case `valuep` is left alone. It may be called from user, kernel
    /// or interrupt context.
    pub fn ddi_peek16(dip: dev_info_t, addr: *const i16, valuep: *mut i16) -> c_int;
    /// `ddi_peek32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cautiously reads a 32-bit value from the virtual address `addr` on
    /// behalf of the device node `dip`, going through the parent nexus so
    /// that a bus fault is reported instead of taking the system down. The
    /// value is stored through `valuep` unless `valuep` is NULL. No byte
    /// order translation takes place. This is the replacement for the
    /// obsolete `ddi_peekl`.
    ///
    /// The function returns `DDI_SUCCESS` when the location was read and
    /// `DDI_FAILURE` when the address is invalid or the read faulted, in
    /// which case `valuep` is left alone. It may be called from user, kernel
    /// or interrupt context.
    pub fn ddi_peek32(dip: dev_info_t, addr: *const i32, valuep: *mut i32) -> c_int;
    /// `ddi_peek64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cautiously reads a 64-bit value from the virtual address `addr` on
    /// behalf of the device node `dip`, going through the parent nexus so
    /// that a bus fault is reported instead of taking the system down. The
    /// value is stored through `valuep` unless `valuep` is NULL. No byte
    /// order translation takes place. This is the replacement for the
    /// obsolete `ddi_peekd`.
    ///
    /// The function returns `DDI_SUCCESS` when the location was read and
    /// `DDI_FAILURE` when the address is invalid or the read faulted, in
    /// which case `valuep` is left alone. It may be called from user, kernel
    /// or interrupt context.
    pub fn ddi_peek64(dip: dev_info_t, addr: *const i64, valuep: *mut i64) -> c_int;
    /// `ddi_poke8(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cautiously writes the byte `value` to the virtual address `addr` on
    /// behalf of the device node `dip`, using the parent nexus to arrange
    /// for any bus fault to be caught rather than to panic the system. Like
    /// the peek family it exists for establishing whether a device is
    /// present in `probe(9E)` or `attach(9E)`. On multiprocessor machines a
    /// poke can be very expensive, because the nexus may have to quiesce the
    /// other processors to attribute a fault to this write, so the page
    /// recommends `ddi_peek8` where a read will do.
    ///
    /// The page names `ddi_pokec`, `ddi_pokes`, `ddi_pokel` and `ddi_poked`
    /// as the obsolete spellings that `ddi_poke8`, `ddi_poke16`,
    /// `ddi_poke32` and `ddi_poke64` replace. The function returns
    /// `DDI_SUCCESS` when the value was written and `DDI_FAILURE` when the
    /// address is invalid or the write faulted. It may be called from user,
    /// kernel or interrupt context.
    pub fn ddi_poke8(dip: dev_info_t, addr: *mut i8, value: i8) -> c_int;
    /// `ddi_poke16(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cautiously writes the 16-bit `value` to the virtual address `addr`
    /// on behalf of the device node `dip`, going through the parent nexus so
    /// that a bus fault is reported instead of taking the system down. No
    /// byte order translation takes place. The write can be very expensive
    /// on multiprocessor machines. This is the replacement for the obsolete
    /// `ddi_pokes`.
    ///
    /// The function returns `DDI_SUCCESS` when the value was written and
    /// `DDI_FAILURE` when the address is invalid or the write faulted. It
    /// may be called from user, kernel or interrupt context.
    pub fn ddi_poke16(dip: dev_info_t, addr: *mut i16, value: i16) -> c_int;
    /// `ddi_poke32(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cautiously writes the 32-bit `value` to the virtual address `addr`
    /// on behalf of the device node `dip`, going through the parent nexus so
    /// that a bus fault is reported instead of taking the system down. No
    /// byte order translation takes place. The write can be very expensive
    /// on multiprocessor machines. This is the replacement for the obsolete
    /// `ddi_pokel`.
    ///
    /// The function returns `DDI_SUCCESS` when the value was written and
    /// `DDI_FAILURE` when the address is invalid or the write faulted. It
    /// may be called from user, kernel or interrupt context.
    pub fn ddi_poke32(dip: dev_info_t, addr: *mut i32, value: i32) -> c_int;
    /// `ddi_poke64(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Cautiously writes the 64-bit `value` to the virtual address `addr`
    /// on behalf of the device node `dip`, going through the parent nexus so
    /// that a bus fault is reported instead of taking the system down. No
    /// byte order translation takes place. The write can be very expensive
    /// on multiprocessor machines. This is the replacement for the obsolete
    /// `ddi_poked`.
    ///
    /// The function returns `DDI_SUCCESS` when the value was written and
    /// `DDI_FAILURE` when the address is invalid or the write faulted. It
    /// may be called from user, kernel or interrupt context.
    pub fn ddi_poke64(dip: dev_info_t, addr: *mut i64, value: i64) -> c_int;
}
