//! SAS phy maps from `sys/scsi/impl/scsi_sas.h`.
//!
//! A SAS HBA has phys, and one or more phys that connect the same local and
//! remote world wide names form a logical port. A phy map follows the phys
//! as they come and go and tells the driver when a logical port appears,
//! because the first phy for a pair of names came up, and when it
//! disappears, because the last such phy went away. Each logical port has a
//! unit address built from the two names, which a driver that also uses an
//! iport map passes to `scsi_hba_iportmap_iport_add(9F)` as the iport's
//! address. Drivers for devices that are not SAS HBAs or similar fabrics do
//! not use this interface.
//!
//! The declarations live in `sys/scsi/impl/scsi_sas.h`, which
//! `sys/scsi/scsi.h` includes through `sys/scsi/impl/types.h`. The functions
//! bound here are documented by `sas_phymap_create(9F)` and
//! `sas_phymap_lookup_ua(9F)`, whose interface level is Evolving. The rest
//! of the header, the `sas_port_protocol_stats_t`, `sas_port_stats_t` and
//! `sas_phy_stats_t` kstat structures, the `SAS_PROTOCOL_*` and
//! `SAS_LINK_RATE_*` values and the property and event names, has no manual
//! page and is not bound.

use ddi_sys::{c_char, c_int, c_void, dev_info_t, opaque};

opaque! {
    /// `sas_phymap_create(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// A phy map, `sas_phymap_t`, which C declares as the private
    /// `struct __sas_phymap`. It is created by [`sas_phymap_create`] and only
    /// ever handled through a pointer.
    sas_phymap_t;
    /// `sas_phymap_ua2phys(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// A set of phys that belong to one logical port, `sas_phymap_phys_t`,
    /// which C declares as the private `struct __sas_phymap_phys`. It is
    /// created by [`sas_phymap_ua2phys`], walked with
    /// [`sas_phymap_phys_next`] and freed with [`sas_phymap_phys_free`].
    sas_phymap_phys_t;
}

/// `sas_phymap_create(9F)`. Evolving - This interface is still evolving in illumos.
///
/// The mode of operation of a phy map, the C enumeration
/// `sas_phymap_mode_t`. Its only value is [`PHYMAP_MODE_SIMPLE`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct sas_phymap_mode_t(pub c_int);
/// The simple mode of operation, which is the one drivers use
/// (`PHYMAP_MODE_SIMPLE`).
pub const PHYMAP_MODE_SIMPLE: sas_phymap_mode_t = sas_phymap_mode_t(0);

/// `sas_phymap_create(9F)`. Evolving - This interface is still evolving in illumos.
///
/// The signature of the optional activation callback of a phy map, whose C
/// typedef `sas_phymap_activate_cb_t` is a pointer to such a function. It is
/// called in kernel context when a new logical port is created because the
/// first phy with its pair of world wide names was added. `phymap_priv` is
/// the value given to [`sas_phymap_create`], and `ua` is the unit address
/// the system built from the two names. `ua_privp` lets the driver store a
/// private value for the port, which [`sas_phymap_lookup_uapriv`] returns
/// and the deactivation callback receives.
pub type sas_phymap_activate_cb_t =
    unsafe extern "C" fn(phymap_priv: *mut c_void, ua: *mut c_char, ua_privp: *mut *mut c_void);

/// `sas_phymap_create(9F)`. Evolving - This interface is still evolving in illumos.
///
/// The signature of the optional deactivation callback of a phy map, whose
/// C typedef `sas_phymap_deactivate_cb_t` is a pointer to such a function.
/// It is called in kernel context when a logical port goes away because its
/// last phy was removed. The arguments are those of the activation
/// callback, except that `ua_priv` is the value the activation callback
/// stored.
pub type sas_phymap_deactivate_cb_t =
    unsafe extern "C" fn(phymap_priv: *mut c_void, ua: *mut c_char, ua_priv: *mut c_void);

unsafe extern "C" {
    /// `sas_phymap_create(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Creates a phy map for the HBA node `hba_dip` and stores it in
    /// `phymapp`. `settle_usec` is how long in microseconds the map waits for
    /// changes to settle before reporting them. `mode` must be
    /// [`PHYMAP_MODE_SIMPLE`] and drivers pass NULL as `mode_argument`.
    /// `phymap_priv` is passed to the callbacks `activate_cb` and
    /// `deactivate_cb`, which are optional but which a driver generally needs
    /// for the map to be of use. Returns `DDI_SUCCESS` or `DDI_FAILURE`. It is
    /// generally called from `attach(9E)` and may be called from user or
    /// kernel context.
    pub fn sas_phymap_create(
        hba_dip: dev_info_t,
        settle_usec: c_int,
        mode: sas_phymap_mode_t,
        mode_argument: *mut c_void,
        phymap_priv: *mut c_void,
        activate_cb: Option<sas_phymap_activate_cb_t>,
        deactivate_cb: Option<sas_phymap_deactivate_cb_t>,
        phymapp: *mut *mut sas_phymap_t,
    ) -> c_int;
    /// `sas_phymap_destroy(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Destroys `phymap`, removing all its entries, for which the
    /// deactivation callbacks run. It is generally called from `detach(9E)`
    /// and may be called from user or kernel context.
    pub fn sas_phymap_destroy(phymap: *mut sas_phymap_t);
    /// `sas_phymap_phy_add(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Reports that the phy `phy`, a non-negative number that identifies it
    /// on the device, has come up with the local world wide name
    /// `local_sas_address`, usually the controller's SAS port, and the remote
    /// name `remote_sas_address` of the device on the other side. A phy maps
    /// to only one port at a time. If it is the first phy for this pair of
    /// names, a logical port is created and the activation callback runs.
    /// Returns `DDI_SUCCESS` or `DDI_FAILURE`. It may be called from user or
    /// kernel context.
    pub fn sas_phymap_phy_add(
        phymap: *mut sas_phymap_t,
        phy: c_int,
        local_sas_address: u64,
        remote_sas_address: u64,
    ) -> c_int;
    /// `sas_phymap_phy_rem(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Reports that the phy `phy` has gone offline. If it was the last phy of
    /// its logical port, the port is removed and the deactivation callback
    /// runs. Returns `DDI_SUCCESS` or `DDI_FAILURE`. It may be called from
    /// user or kernel context.
    pub fn sas_phymap_phy_rem(phymap: *mut sas_phymap_t, phy: c_int) -> c_int;
    /// `sas_phymap_lookup_ua(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns the unit address of the logical port of `phymap` that belongs
    /// to the pair of world wide names `local_sas_address` and
    /// `remote_sas_address`, or NULL if there is none. The string belongs to
    /// the system and must be neither modified nor freed. It may be called
    /// from user, kernel or interrupt context.
    pub fn sas_phymap_lookup_ua(
        phymap: *mut sas_phymap_t,
        local_sas_address: u64,
        remote_sas_address: u64,
    ) -> *mut c_char;
    /// `sas_phymap_lookup_uapriv(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns the private value that the activation callback stored for the
    /// logical port with unit address `ua` in `phymap`, or NULL if the port
    /// could not be found or another error occurred. It may be called from
    /// user, kernel or interrupt context.
    pub fn sas_phymap_lookup_uapriv(phymap: *mut sas_phymap_t, ua: *mut c_char) -> *mut c_void;
    /// `sas_phymap_phy2ua(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns the unit address of the logical port that the phy `phy`
    /// belongs to in `phymap`, or NULL if the phy could not be found or
    /// another error occurred. The string is allocated for the caller, who
    /// may modify it and must release it with [`sas_phymap_ua_free`]. It may
    /// be called from user, kernel or interrupt context.
    pub fn sas_phymap_phy2ua(phymap: *mut sas_phymap_t, phy: c_int) -> *mut c_char;
    /// `sas_phymap_ua_free(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Releases a unit address string that [`sas_phymap_phy2ua`] returned. It
    /// must not be used on the string from [`sas_phymap_lookup_ua`]. It may be
    /// called from user, kernel or interrupt context.
    pub fn sas_phymap_ua_free(ua: *mut c_char);
    /// `sas_phymap_uahasphys(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns 1 if the logical port with unit address `ua` in `phymap` has
    /// any phys, and 0 if it has none, does not exist, or an error occurred.
    /// It may be called from user, kernel or interrupt context.
    pub fn sas_phymap_uahasphys(phymap: *mut sas_phymap_t, ua: *mut c_char) -> c_int;
    /// `sas_phymap_ua2phys(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns a newly allocated set of the phys that belong to the logical
    /// port with unit address `ua` in `phymap`, or NULL on failure. The caller
    /// walks it with [`sas_phymap_phys_next`] and must release it with
    /// [`sas_phymap_phys_free`], whether or not it walked it. It may be
    /// called from user, kernel or interrupt context.
    pub fn sas_phymap_ua2phys(phymap: *mut sas_phymap_t, ua: *mut c_char) -> *mut sas_phymap_phys_t;
    /// `sas_phymap_phys_next(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Returns the next phy of the set `phys`, a non-negative number, or -1
    /// when no phys remain. It may be called from user, kernel or interrupt
    /// context.
    pub fn sas_phymap_phys_next(phys: *mut sas_phymap_phys_t) -> c_int;
    /// `sas_phymap_phys_free(9F)`. Evolving - This interface is still evolving in illumos.
    ///
    /// Releases a set of phys that [`sas_phymap_ua2phys`] returned. It may be
    /// called from user, kernel or interrupt context.
    pub fn sas_phymap_phys_free(phys: *mut sas_phymap_phys_t);
}
