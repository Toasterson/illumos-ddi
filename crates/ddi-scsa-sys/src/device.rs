//! SCSI addresses and devices from `sys/scsi/scsi_address.h` and
//! `sys/scsi/conf/device.h`.
//!
//! A `scsi_address(9S)` names the destination of a command, and a
//! `scsi_device(9S)` describes one logical unit. The HBA framework creates a
//! `scsi_device` for every logical unit before the target driver's
//! `probe(9E)` runs, and the target driver finds it with
//! `ddi_get_driver_private(9F)`. The address inside it is what the target
//! driver passes to `scsi_init_pkt(9F)` and the other transport functions.
//!
//! An HBA driver that passes [`SCSI_HBA_ADDR_COMPLEX`](crate::hba::SCSI_HBA_ADDR_COMPLEX)
//! to `scsi_hba_attach_setup(9F)` uses complex addressing. Its devices are
//! then named by a unit address string rather than by a target and a
//! logical unit number, the address no longer contains a target and a
//! logical unit, and the HBA driver works with the functions of
//! `scsi_address_device(9F)` instead. The world wide name conversions of
//! `scsi_wwnstr_to_wwn(9F)` help such drivers build unit addresses.
//!
//! The header also declares `scsi_lun_t`, `scsi_lun_to_lun64`,
//! `scsi_lun64_to_lun`, `scsi_wwnstr_hexcase`, `scsi_wwnstr_skip_ua_prefix`,
//! the `scsi_device_prop_*` family, `scsi_device_size` and the
//! `SCSI_ADDR_PROP_*` property names. None of them has a manual page, so
//! they are not bound.

use ddi_sys::{c_char, c_int, c_uchar, c_uint, c_void, dev_info_t, kmutex_t, size_t, ushort_t};

use crate::hba::scsi_hba_tran;
use crate::inquiry::scsi_inquiry;
use crate::sense::scsi_extended_sense;

// --- scsi_address(9S) ----------------------------------------------------

/// `scsi_address(9S)`. illumos architecture specific (illumos DDI).
///
/// The parallel SCSI view of a unit address, the anonymous `spi` structure
/// inside [`scsi_address_a`]. C code reaches its members through the `a_target`, `a_lun` and
/// `a_sublun` macros. The members are not accessible when the HBA driver
/// uses complex addressing.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct scsi_address_spi {
    /// The target component of the address, the SCSI target identifier.
    pub a_target: ushort_t,
    /// The logical unit component of the address. It is 0 for a device that
    /// is addressed by its target alone.
    pub a_lun: c_uchar,
    /// Private to the framework.
    pub _a_sublun: c_uchar,
}

/// `scsi_address(9S)`. illumos architecture specific (illumos DDI).
///
/// The unit address information of a [`scsi_address`], the anonymous union
/// that C names `a`. Which member is valid depends on how the HBA driver addresses its
/// devices. With parallel SCSI style addressing it is `spi`, and with
/// complex addressing it is `a_sd`.
#[repr(C)]
#[derive(Clone, Copy)]
pub union scsi_address_a {
    /// The target and logical unit, for HBA drivers that do not use complex
    /// addressing.
    pub spi: scsi_address_spi,
    /// The `scsi_device(9S)` the address belongs to, for HBA drivers that
    /// use complex addressing. It is private to the framework; drivers call
    /// `scsi_address_device(9F)` instead of reading it.
    pub a_sd: *mut scsi_device,
}

/// `scsi_address(9S)`. illumos architecture specific (illumos DDI).
///
/// The address of a SCSI target device. It holds a pointer to the transport
/// vector of the controlling HBA, through which the framework passes
/// transport requests from the target driver to the HBA driver, and the
/// unit address of the device. Traditionally the unit address is a target
/// number and a logical unit number; a device addressed by its target alone
/// uses logical unit 0, and an HBA whose devices support several logical
/// units must interpret the logical unit field.
///
/// When the HBA driver uses complex addressing, only
/// [`a_hba_tran`](Self::a_hba_tran) remains accessible. The unit address is
/// then a string, and the driver maps the address to its `scsi_device(9S)`
/// with `scsi_address_device(9F)`. The `pkt_address` member of every
/// `scsi_pkt(9S)` is a copy of this structure made by `scsi_init_pkt(9F)`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct scsi_address {
    /// The transport vector of the HBA that controls the device. It is valid
    /// with either kind of addressing.
    pub a_hba_tran: *mut scsi_hba_tran,
    /// The unit address. See [`scsi_address_a`].
    pub a: scsi_address_a,
}

// Layout on amd64 with LP64. scsi_address_spi is a ushort_t at 0 and two
// uchar_t at 2 and 3, 4 bytes aligned to 2. The union holds that and a
// pointer, so it is 8 bytes aligned to 8. scsi_address is a_hba_tran 8 at 0
// and a 8 at 8; total 16.
const _: () = assert!(core::mem::size_of::<scsi_address_spi>() == 4);
const _: () = assert!(core::mem::offset_of!(scsi_address_spi, a_target) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_address_spi, a_lun) == 2);
const _: () = assert!(core::mem::offset_of!(scsi_address_spi, _a_sublun) == 3);
const _: () = assert!(core::mem::size_of::<scsi_address_a>() == 8);
const _: () = assert!(core::mem::size_of::<scsi_address>() == 16);
const _: () = assert!(core::mem::offset_of!(scsi_address, a_hba_tran) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_address, a) == 8);

// --- scsi_device(9S) -----------------------------------------------------

/// `scsi_device(9S)`. illumos DDI specific (illumos DDI).
///
/// Common information about one SCSI logical unit, with pointers to generic
/// and device specific data. There is one such structure for each logical
/// unit attached to the system. The HBA framework initialises part of it
/// before the target driver's `probe(9E)` runs and destroys it after a
/// failed probe or a successful `detach(9E)`. A target driver receives it
/// through `ddi_get_driver_private(9F)` on its device node.
///
/// Drivers use the members the manual page documents, which are
/// [`sd_address`](Self::sd_address), [`sd_dev`](Self::sd_dev),
/// [`sd_mutex`](Self::sd_mutex), [`sd_inq`](Self::sd_inq),
/// [`sd_sense`](Self::sd_sense) and [`sd_private`](Self::sd_private). The
/// others belong to the framework and are declared only for an exact
/// layout. Like `scsi_pkt(9S)`, this structure is allocated by the system
/// and drivers must not depend on its size.
#[repr(C)]
pub struct scsi_device {
    /// The routing information of the device, which the target driver
    /// passes to `scsi_init_pkt(9F)` and which ends up in the `pkt_address`
    /// of every packet. The SCSA library uses it to decide which HBA, bus,
    /// target and logical unit a command is meant for. The HBA framework
    /// initialises it.
    pub sd_address: scsi_address,
    /// The device node of the target device, set by the HBA framework.
    pub sd_dev: dev_info_t,
    /// A mutex the target driver may use to serialise access to the device.
    /// The framework initialises it before `probe(9E)` or `attach(9E)` is
    /// called.
    pub sd_mutex: kmutex_t,
    /// The HBA driver's private data for this device under complex
    /// addressing. Private to the framework; HBA drivers use
    /// `scsi_device_hba_private_get(9F)` and
    /// `scsi_device_hba_private_set(9F)`.
    pub sd_hba_private: *mut c_void,
    /// The inquiry data of the device. It is NULL until `scsi_probe(9F)`
    /// succeeds, which allocates and fills it in. The framework frees it.
    pub sd_inq: *mut scsi_inquiry,
    /// A buffer for request sense data that the target driver may provide.
    /// It is initially NULL; a target driver that wants to use it allocates
    /// a `scsi_extended_sense(9S)` buffer and stores its address here.
    pub sd_sense: *mut scsi_extended_sense,
    /// Reserved for the target driver, which typically points it to its
    /// soft state for the device in `attach(9E)` and clears it in
    /// `detach(9E)`.
    pub sd_private: *mut c_void,
    /// The fault management capabilities of the device. Private to the
    /// framework.
    pub sd_fm_capable: c_int,
    /// The multipathing path node for a device that the `scsi_vhci` driver
    /// allocated. Private to the framework.
    pub sd_pathinfo: *mut c_void,
    /// Three bit fields in one `unsigned int`, private to the framework.
    /// Bits 0 to 15 are `sd_uninit_prevent`, a count that keeps the node
    /// from being uninitialised, bit 16 is `sd_tran_tgt_free_done`, which
    /// records that `tran_tgt_free(9E)` has already been called, and bits 17
    /// to 31 are the padding `sd_flags_pad`.
    pub sd_bits: c_uint,
    /// A copy of the HBA transport vector kept by the framework in case a
    /// driver overwrites [`scsi_address::a_hba_tran`]. Private to the
    /// framework.
    pub sd_tran_safe: *mut scsi_hba_tran,
}

// Layout on amd64 with LP64, as `sys/scsi/conf/device.h` declares it
// without `SCSI_SIZE_CLEAN_VERIFY`. sd_address 16 at 0, sd_dev 8 at 16,
// sd_mutex 8 at 24 (a kmutex_t is one pointer), sd_hba_private 8 at 32,
// sd_inq 8 at 40, sd_sense 8 at 48, sd_private 8 at 56, sd_fm_capable 4 at
// 64 and 4 bytes of padding, sd_pathinfo 8 at 72, the bit field word 4 at 80
// and 4 bytes of padding, sd_tran_safe 8 at 88; total 96.
const _: () = assert!(core::mem::size_of::<scsi_device>() == 96);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_address) == 0);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_dev) == 16);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_mutex) == 24);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_hba_private) == 32);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_inq) == 40);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_sense) == 48);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_private) == 56);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_fm_capable) == 64);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_pathinfo) == 72);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_bits) == 80);
const _: () = assert!(core::mem::offset_of!(scsi_device, sd_tran_safe) == 88);

// --- world wide names ----------------------------------------------------

/// The number of characters in the string form of a world wide name,
/// without the unit address prefix and without the terminating NUL
/// (`SCSI_WWN_STRLEN`).
pub const SCSI_WWN_STRLEN: size_t = 16;
/// The number of characters in the unit address form of a world wide name,
/// which has a leading `w`, without the terminating NUL
/// (`SCSI_WWN_UA_STRLEN`).
pub const SCSI_WWN_UA_STRLEN: size_t = 17;
/// A buffer size that is always large enough for any string form of a world
/// wide name and its terminating NUL. It is `SCSI_MAXNAMELEN`, which is
/// `MAXNAMELEN` (`SCSI_WWN_BUFLEN`).
pub const SCSI_WWN_BUFLEN: size_t = 256;

unsafe extern "C" {
    /// `scsi_address_device(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Maps the `scsi_address(9S)` at `sa` back to the `scsi_device(9S)` it
    /// belongs to. It is meant for HBA drivers that enabled complex
    /// addressing by passing `SCSI_HBA_ADDR_COMPLEX` to
    /// `scsi_hba_attach_setup(9F)`. Returns the device on success. It
    /// returns NULL if an error occurred or if the HBA did not enable complex
    /// addressing, so a driver may also use it to find out whether the flag
    /// is set. It may be called from user, kernel or interrupt context.
    pub fn scsi_address_device(sa: *mut scsi_address) -> *mut scsi_device;
    /// `scsi_device_hba_private_get(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns the private value that the HBA driver stored for the device
    /// `sd` with [`scsi_device_hba_private_set`], or NULL if none was stored.
    /// The HBA driver must have enabled complex addressing with
    /// `SCSI_HBA_ADDR_COMPLEX`; otherwise it must not call this function. A
    /// typical driver stores a value in `tran_tgt_init(9E)` and retrieves it
    /// in its other entry points. It may be called from user, kernel or
    /// interrupt context.
    pub fn scsi_device_hba_private_get(sd: *mut scsi_device) -> *mut c_void;
    /// `scsi_device_hba_private_set(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Stores the private value `data` on the device `sd`, so that the HBA
    /// driver can retrieve it later with [`scsi_device_hba_private_get`].
    /// This simplifies the mapping between the driver's own data structures
    /// and the system's objects. The HBA driver must have enabled complex
    /// addressing with `SCSI_HBA_ADDR_COMPLEX`, and usually calls this
    /// function from `tran_tgt_init(9E)`. It may be called from user, kernel
    /// or interrupt context.
    pub fn scsi_device_hba_private_set(sd: *mut scsi_device, data: *mut c_void);
    /// `scsi_device_unit_address(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns the unit address string of the device `sd`, which comes from
    /// the values given when the device was enumerated, usually through an
    /// iport. The driver must not modify the string. Returns NULL on error
    /// or if the HBA driver did not enable complex addressing with
    /// `SCSI_HBA_ADDR_COMPLEX`, in which case it must not call this function
    /// at all. It may be called from user, kernel or interrupt context.
    pub fn scsi_device_unit_address(sd: *mut scsi_device) -> *mut c_char;
    /// `scsi_wwnstr_to_wwn(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Parses the NUL terminated string `wwnstr` as an 8 byte world wide
    /// name and stores the result in `wwnp`. The string must consist of 16
    /// hexadecimal digits in either case, optionally preceded by a `w` for
    /// the unit address form; any other length makes it invalid. Returns
    /// `DDI_SUCCESS` on success and `DDI_FAILURE` for an invalid argument or
    /// a malformed string. It may be called from user, kernel or interrupt
    /// context.
    pub fn scsi_wwnstr_to_wwn(wwnstr: *const c_char, wwnp: *mut u64) -> c_int;
    /// `scsi_wwn_to_wwnstr(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Converts the world wide name `wwn` into a string of 16 hexadecimal
    /// digits. If `unit_address_form` is non-zero, the string is given a
    /// leading `w` as used in unit addresses. If `wwnstr` is not NULL, it
    /// must point to a buffer large enough for the string and its NUL, for
    /// which [`SCSI_WWN_BUFLEN`] is recommended. If it is NULL, the system
    /// allocates the string, blocking until memory is available, and the
    /// caller releases it with [`scsi_free_wwnstr`]. Returns a pointer to
    /// the start of the string, or NULL if the conversion failed. It may be
    /// called from user, kernel or interrupt context, although the page's
    /// note that allocation blocks rules out a NULL `wwnstr` in interrupt
    /// context.
    pub fn scsi_wwn_to_wwnstr(wwn: u64, unit_address_form: c_int, wwnstr: *mut c_char) -> *mut c_char;
    /// `scsi_free_wwnstr(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Releases a string that [`scsi_wwn_to_wwnstr`] allocated because it was
    /// called with a NULL buffer. It may be called from user, kernel or
    /// interrupt context.
    pub fn scsi_free_wwnstr(wwnstr: *mut c_char);
}
