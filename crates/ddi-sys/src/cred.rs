//! Credentials and privileges from `sys/cred.h`, `sys/priv.h`,
//! `sys/policy.h` and `sys/priv_names.h`.
//!
//! The kernel hands a `cred_t` to most entry points. The structure itself is
//! opaque to drivers (see [`crate::types::cred_t`]); the `crget*` functions
//! of `ddi_cred(9F)` are the documented way to read its fields and the
//! `crset*` functions the way to initialise a freshly allocated one. A
//! privilege check is done in two steps: `priv_getbyname(9F)` turns a
//! privilege name such as [`PRIV_SYS_DEVICES`] into a number, and the
//! `priv_policy(9F)` family tests that number against a credential. The
//! `drv_priv(9F)` shortcut performs the one check almost every driver
//! needs, namely whether the caller holds `PRIV_SYS_DEVICES`.

use crate::types::{boolean_t, c_char, c_int, cred_t, uint_t};

/// The C `uid_t`, a user ID. It is an `unsigned int`.
pub type uid_t = uint_t;
/// The C `gid_t`, a group ID. It is defined as `uid_t`.
pub type gid_t = uid_t;
/// The C `zoneid_t`, a zone ID. It is an `id_t`, which is an `int`.
pub type zoneid_t = c_int;
/// The C `projid_t`, a project ID. It is an `id_t`, which is an `int`.
pub type projid_t = c_int;

/// Asks `priv_getbyname(9F)` to allocate the privilege if it does not exist
/// yet (`PRIV_ALLOC`).
pub const PRIV_ALLOC: uint_t = 0x1;
/// The special privilege number that asks the `priv_policy(9F)` family to
/// test for all privileges (`PRIV_ALL`).
pub const PRIV_ALL: c_int = -1;
/// The maximum length of a privilege name (`PRIVNAME_MAX`).
pub const PRIVNAME_MAX: usize = 32;

/// Allows access to devices and their configuration (`PRIV_SYS_DEVICES`).
/// This is the privilege `drv_priv(9F)` checks and the one a driver requires
/// before an ioctl that changes device state.
pub const PRIV_SYS_DEVICES: &core::ffi::CStr = c"sys_devices";
/// Allows a process to perform system administration tasks such as
/// setting the host name (`PRIV_SYS_ADMIN`).
pub const PRIV_SYS_ADMIN: &core::ffi::CStr = c"sys_admin";
/// Allows a process to perform system configuration tasks such as adding
/// swap or configuring privileged sockets (`PRIV_SYS_CONFIG`).
pub const PRIV_SYS_CONFIG: &core::ffi::CStr = c"sys_config";
/// Allows a process to mount and unmount file systems (`PRIV_SYS_MOUNT`).
pub const PRIV_SYS_MOUNT: &core::ffi::CStr = c"sys_mount";
/// Allows a process to configure the network (`PRIV_SYS_NET_CONFIG`).
pub const PRIV_SYS_NET_CONFIG: &core::ffi::CStr = c"sys_net_config";
/// Allows a process to configure IP interfaces and routes (`PRIV_SYS_IP_CONFIG`).
pub const PRIV_SYS_IP_CONFIG: &core::ffi::CStr = c"sys_ip_config";
/// Allows a process to configure datalinks (`PRIV_SYS_DL_CONFIG`).
pub const PRIV_SYS_DL_CONFIG: &core::ffi::CStr = c"sys_dl_config";
/// Allows a process to configure IP tunnels (`PRIV_SYS_IPTUN_CONFIG`).
pub const PRIV_SYS_IPTUN_CONFIG: &core::ffi::CStr = c"sys_iptun_config";
/// Allows a process to configure PPP (`PRIV_SYS_PPP_CONFIG`).
pub const PRIV_SYS_PPP_CONFIG: &core::ffi::CStr = c"sys_ppp_config";
/// Allows a process to configure resource controls and pools
/// (`PRIV_SYS_RES_CONFIG`).
pub const PRIV_SYS_RES_CONFIG: &core::ffi::CStr = c"sys_res_config";
/// Allows a process to bind to resource pools (`PRIV_SYS_RES_BIND`).
pub const PRIV_SYS_RES_BIND: &core::ffi::CStr = c"sys_res_bind";
/// Allows a process to set the system time (`PRIV_SYS_TIME`).
pub const PRIV_SYS_TIME: &core::ffi::CStr = c"sys_time";
/// Allows a process to link and unlink directories (`PRIV_SYS_LINKDIR`).
pub const PRIV_SYS_LINKDIR: &core::ffi::CStr = c"sys_linkdir";
/// Allows a process to manage the SMB server (`PRIV_SYS_SMB`).
pub const PRIV_SYS_SMB: &core::ffi::CStr = c"sys_smb";
/// Allows a process to perform NFS operations (`PRIV_SYS_NFS`).
pub const PRIV_SYS_NFS: &core::ffi::CStr = c"sys_nfs";
/// Allows a process to configure auditing (`PRIV_SYS_AUDIT`).
pub const PRIV_SYS_AUDIT: &core::ffi::CStr = c"sys_audit";
/// Allows a process to configure accounting (`PRIV_SYS_ACCT`).
pub const PRIV_SYS_ACCT: &core::ffi::CStr = c"sys_acct";
/// Allows a process to change System V IPC parameters (`PRIV_SYS_IPC_CONFIG`).
pub const PRIV_SYS_IPC_CONFIG: &core::ffi::CStr = c"sys_ipc_config";
/// Marks operations that used to require the super-user and have no finer
/// privilege yet (`PRIV_SYS_SUSER_COMPAT`).
pub const PRIV_SYS_SUSER_COMPAT: &core::ffi::CStr = c"sys_suser_compat";
/// Allows a process to translate labels under Trusted Extensions
/// (`PRIV_SYS_TRANS_LABEL`).
pub const PRIV_SYS_TRANS_LABEL: &core::ffi::CStr = c"sys_trans_label";
/// Allows a process to read any file regardless of permission bits
/// (`PRIV_FILE_DAC_READ`).
pub const PRIV_FILE_DAC_READ: &core::ffi::CStr = c"file_dac_read";
/// Allows a process to write any file regardless of permission bits
/// (`PRIV_FILE_DAC_WRITE`).
pub const PRIV_FILE_DAC_WRITE: &core::ffi::CStr = c"file_dac_write";
/// Allows a process to execute any file regardless of permission bits
/// (`PRIV_FILE_DAC_EXECUTE`).
pub const PRIV_FILE_DAC_EXECUTE: &core::ffi::CStr = c"file_dac_execute";
/// Allows a process to search any directory regardless of permission bits
/// (`PRIV_FILE_DAC_SEARCH`).
pub const PRIV_FILE_DAC_SEARCH: &core::ffi::CStr = c"file_dac_search";
/// Allows a process to act as the owner of any file (`PRIV_FILE_OWNER`).
pub const PRIV_FILE_OWNER: &core::ffi::CStr = c"file_owner";
/// Allows a process to change the ownership of a file (`PRIV_FILE_CHOWN`).
pub const PRIV_FILE_CHOWN: &core::ffi::CStr = c"file_chown";
/// Allows a process to give away files it owns (`PRIV_FILE_CHOWN_SELF`).
pub const PRIV_FILE_CHOWN_SELF: &core::ffi::CStr = c"file_chown_self";
/// Allows a process to set the set-id bits on a file (`PRIV_FILE_SETID`).
pub const PRIV_FILE_SETID: &core::ffi::CStr = c"file_setid";
/// Allows a process to set the immutable and other system flags on a file
/// (`PRIV_FILE_FLAG_SET`).
pub const PRIV_FILE_FLAG_SET: &core::ffi::CStr = c"file_flag_set";
/// Allows a process to open raw network sockets (`PRIV_NET_RAWACCESS`).
pub const PRIV_NET_RAWACCESS: &core::ffi::CStr = c"net_rawaccess";
/// Allows a process to bind to privileged ports (`PRIV_NET_PRIVADDR`).
pub const PRIV_NET_PRIVADDR: &core::ffi::CStr = c"net_privaddr";
/// Allows a process to send and receive ICMP packets (`PRIV_NET_ICMPACCESS`).
pub const PRIV_NET_ICMPACCESS: &core::ffi::CStr = c"net_icmpaccess";
/// Allows a process to observe network traffic (`PRIV_NET_OBSERVABILITY`).
pub const PRIV_NET_OBSERVABILITY: &core::ffi::CStr = c"net_observability";
/// Allows a process to be aware of labels on network traffic
/// (`PRIV_NET_MAC_AWARE`).
pub const PRIV_NET_MAC_AWARE: &core::ffi::CStr = c"net_mac_aware";
/// Allows a process to bind to a multilevel port (`PRIV_NET_BINDMLP`).
pub const PRIV_NET_BINDMLP: &core::ffi::CStr = c"net_bindmlp";
/// Allows a process to set an implicit label on network traffic
/// (`PRIV_NET_MAC_IMPLICIT`).
pub const PRIV_NET_MAC_IMPLICIT: &core::ffi::CStr = c"net_mac_implicit";
/// Allows a process to act on other processes as if it owned them
/// (`PRIV_PROC_OWNER`).
pub const PRIV_PROC_OWNER: &core::ffi::CStr = c"proc_owner";
/// Allows a process to change scheduling class and priority
/// (`PRIV_PROC_PRIOCNTL`).
pub const PRIV_PROC_PRIOCNTL: &core::ffi::CStr = c"proc_priocntl";
/// Allows a process to raise its priority (`PRIV_PROC_PRIOUP`).
pub const PRIV_PROC_PRIOUP: &core::ffi::CStr = c"proc_prioup";
/// Allows a process to use high-resolution timers (`PRIV_PROC_CLOCK_HIGHRES`).
pub const PRIV_PROC_CLOCK_HIGHRES: &core::ffi::CStr = c"proc_clock_highres";
/// Allows a process to lock pages in memory (`PRIV_PROC_LOCK_MEMORY`).
pub const PRIV_PROC_LOCK_MEMORY: &core::ffi::CStr = c"proc_lock_memory";
/// Allows a process to read physical memory information (`PRIV_PROC_MEMINFO`).
pub const PRIV_PROC_MEMINFO: &core::ffi::CStr = c"proc_meminfo";
/// Allows a process to create task IDs (`PRIV_PROC_TASKID`).
pub const PRIV_PROC_TASKID: &core::ffi::CStr = c"proc_taskid";
/// Allows a process to enter a zone (`PRIV_PROC_ZONE`).
pub const PRIV_PROC_ZONE: &core::ffi::CStr = c"proc_zone";
/// Allows a process to change its root directory (`PRIV_PROC_CHROOT`).
pub const PRIV_PROC_CHROOT: &core::ffi::CStr = c"proc_chroot";
/// Allows a process to read System V IPC objects regardless of permission
/// (`PRIV_IPC_DAC_READ`).
pub const PRIV_IPC_DAC_READ: &core::ffi::CStr = c"ipc_dac_read";
/// Allows a process to write System V IPC objects regardless of permission
/// (`PRIV_IPC_DAC_WRITE`).
pub const PRIV_IPC_DAC_WRITE: &core::ffi::CStr = c"ipc_dac_write";
/// Allows a process to act as the owner of any System V IPC object
/// (`PRIV_IPC_OWNER`).
pub const PRIV_IPC_OWNER: &core::ffi::CStr = c"ipc_owner";
/// Allows a process to use kernel-level DTrace probes (`PRIV_DTRACE_KERNEL`).
pub const PRIV_DTRACE_KERNEL: &core::ffi::CStr = c"dtrace_kernel";
/// Allows a process to use process-level DTrace probes (`PRIV_DTRACE_PROC`).
pub const PRIV_DTRACE_PROC: &core::ffi::CStr = c"dtrace_proc";
/// Allows a process to use user-level DTrace probes (`PRIV_DTRACE_USER`).
pub const PRIV_DTRACE_USER: &core::ffi::CStr = c"dtrace_user";
/// Allows a process to access CPU performance counters (`PRIV_CPC_CPU`).
pub const PRIV_CPC_CPU: &core::ffi::CStr = c"cpc_cpu";
/// Allows a process to access graphics devices directly
/// (`PRIV_GRAPHICS_ACCESS`).
pub const PRIV_GRAPHICS_ACCESS: &core::ffi::CStr = c"graphics_access";
/// Allows a process to map graphics device memory (`PRIV_GRAPHICS_MAP`).
pub const PRIV_GRAPHICS_MAP: &core::ffi::CStr = c"graphics_map";
/// Allows a process to manage virtual machines (`PRIV_VIRT_MANAGE`).
pub const PRIV_VIRT_MANAGE: &core::ffi::CStr = c"virt_manage";
/// Allows a process to control the xVM hypervisor (`PRIV_XVM_CONTROL`).
pub const PRIV_XVM_CONTROL: &core::ffi::CStr = c"xvm_control";
/// Allows a process to request critical contract events without limitation
/// (`PRIV_CONTRACT_EVENT`).
pub const PRIV_CONTRACT_EVENT: &core::ffi::CStr = c"contract_event";
/// Allows a process to set the identity of a contract (`PRIV_CONTRACT_IDENTITY`).
pub const PRIV_CONTRACT_IDENTITY: &core::ffi::CStr = c"contract_identity";
/// Allows a process to observe contracts of other users
/// (`PRIV_CONTRACT_OBSERVER`).
pub const PRIV_CONTRACT_OBSERVER: &core::ffi::CStr = c"contract_observer";

unsafe extern "C" {
    /// `drv_priv(9F)`. Architecture independent level 1 (DDI/DKI).
    ///
    /// Returns 0 if the credential carries the `PRIV_SYS_DEVICES` privilege
    /// and `EPERM` otherwise. This is the check a driver performs before an
    /// ioctl that changes device state.
    pub fn drv_priv(cr: *mut cred_t) -> c_int;
    /// `ddi_get_cred(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the credential of the current thread.
    pub fn ddi_get_cred() -> *mut cred_t;
    /// `crgetuid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the effective user ID of the credential. May be called from
    /// user and kernel context.
    pub fn crgetuid(cr: *const cred_t) -> uid_t;
    /// `credentials(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Adds a reference to a credential. Each `crhold` must be balanced by a
    /// [`crfree`].
    pub fn crhold(cr: *mut cred_t);
    /// `credentials(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Drops a reference to a credential and frees it when the last one is
    /// gone.
    pub fn crfree(cr: *mut cred_t);
    /// `credentials(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Allocates a new credential with one reference and every field
    /// initialised to zero. It sleeps for memory, so it may only be called
    /// from user or kernel context.
    pub fn crget() -> *mut cred_t;
    /// `credentials(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Returns a copy of `cr` with one reference and drops one reference to
    /// `cr`. It sleeps for memory.
    pub fn crdup(cr: *mut cred_t) -> *mut cred_t;
    /// `credentials(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Returns the kernel credential of the current zone: all privileges,
    /// limited to what that zone may hold. The caller does not own a
    /// reference.
    pub fn zone_kcred() -> *mut cred_t;
    /// `credentials(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// The kernel's own credential, with every privilege. It is a global
    /// variable in `genunix`, which is why it is declared as a `static`.
    /// Using it bypasses every access check, so a driver should prefer the
    /// caller's credential and reach for `kcred` only for work it does on
    /// its own behalf.
    pub static kcred: *mut cred_t;
    /// `crgetruid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the real user ID of the credential. May be called from user
    /// and kernel context.
    pub fn crgetruid(cr: *const cred_t) -> uid_t;
    /// `crgetsuid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the saved user ID of the credential. May be called from user
    /// and kernel context.
    pub fn crgetsuid(cr: *const cred_t) -> uid_t;
    /// `crgetgid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the effective group ID of the credential. May be called from
    /// user and kernel context.
    pub fn crgetgid(cr: *const cred_t) -> gid_t;
    /// `crgetrgid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the real group ID of the credential. May be called from user
    /// and kernel context.
    pub fn crgetrgid(cr: *const cred_t) -> gid_t;
    /// `crgetsgid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the saved group ID of the credential. May be called from user
    /// and kernel context.
    pub fn crgetsgid(cr: *const cred_t) -> gid_t;
    /// `crgetzoneid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the ID of the zone the credential belongs to. May be called
    /// from user and kernel context.
    pub fn crgetzoneid(cr: *const cred_t) -> zoneid_t;
    /// `crgetgroups(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the supplementary group list of the credential. The number of
    /// entries is given by [`crgetngroups`]. May be called from user and
    /// kernel context.
    pub fn crgetgroups(cr: *const cred_t) -> *const gid_t;
    /// `crgetngroups(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Returns the number of supplementary groups in the credential. May be
    /// called from user and kernel context.
    pub fn crgetngroups(cr: *const cred_t) -> c_int;
    /// `crsetresuid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets the real, effective and saved user IDs of a newly allocated
    /// credential. All but one of the three may be given as -1, which leaves
    /// that ID unchanged. Returns 0 on success and -1 if any ID is invalid.
    /// Calling it on a credential that other parts of the system still
    /// reference may panic the system.
    pub fn crsetresuid(cr: *mut cred_t, ruid: uid_t, euid: uid_t, suid: uid_t) -> c_int;
    /// `crsetresgid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets the real, effective and saved group IDs of a newly allocated
    /// credential. All but one of the three may be given as -1, which leaves
    /// that ID unchanged. Returns 0 on success and -1 if any ID is invalid.
    /// Calling it on a credential that other parts of the system still
    /// reference may panic the system.
    pub fn crsetresgid(cr: *mut cred_t, rgid: gid_t, egid: gid_t, sgid: gid_t) -> c_int;
    /// `crsetugid(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Initialises the real, effective and saved user IDs of a newly
    /// allocated credential all to `uid`, and the real, effective and saved
    /// group IDs all to `gid`. Returns 0 on success and -1 if an ID is
    /// invalid. Calling it on a credential that other parts of the system
    /// still reference may panic the system.
    pub fn crsetugid(cr: *mut cred_t, uid: uid_t, gid: gid_t) -> c_int;
    /// `crsetgroups(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Sets the supplementary group list of a newly allocated credential to
    /// the `ngroups` entries at `gids`. When `ngroups` is 0 the `gids`
    /// pointer need not be valid. The manual page shows the last argument as
    /// a plain `gid_t`; the header declares a pointer, which is followed
    /// here. Returns 0 on success and -1 on failure.
    pub fn crsetgroups(cr: *mut cred_t, ngroups: c_int, gids: *mut gid_t) -> c_int;

    /// `priv_getbyname(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Maps a privilege name to the privilege number used by the `priv_*`
    /// kernel interfaces. The name may carry an optional `priv_` prefix,
    /// which is stripped, and is matched without regard to case. When
    /// `flags` contains [`PRIV_ALLOC`] and the privilege is not yet defined,
    /// it is allocated and its new number returned; allocated privileges keep
    /// their case, may be at most [`PRIVNAME_MAX`] characters long and may
    /// contain only letters, digits and the underscore. Returns the privilege
    /// number, which is zero or greater, or a negative errno value: `EINVAL`
    /// when the flags are invalid, the privilege does not exist or the name
    /// contains invalid characters, `ENOMEM` when no more privileges can be
    /// allocated, and `ENAMETOOLONG` when the name is too long. May be called
    /// from user and kernel context.
    pub fn priv_getbyname(priv_: *const c_char, flags: uint_t) -> c_int;
    /// `priv_policy(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Checks whether the privilege numbered `priv_` is asserted in the
    /// effective set of the credential, updates the ASU accounting flag, and
    /// records the privilege in the audit trail on success when it is not a
    /// basic privilege. The special value [`PRIV_ALL`] tests for all
    /// privileges. The `all` argument is `B_FALSE` for most checks and
    /// `B_TRUE` when the operation requires the caller to hold every
    /// privilege available in its zone. The `msg` argument is added to any
    /// privilege debugging message and is usually NULL, since the function
    /// name is already included. The manual page shows `err` before `all`;
    /// the header places `all` first, and the header is followed here.
    /// Returns 0 on success and `err` on failure. May be called from user,
    /// kernel or interrupt context.
    pub fn priv_policy(cr: *const cred_t, priv_: c_int, all: boolean_t, err: c_int, msg: *const c_char) -> c_int;
    /// `priv_policy_only(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Checks whether the privilege numbered `priv_` is asserted in the
    /// effective set of the credential, with no side effects. The `all`
    /// argument has the same meaning as for [`priv_policy`]. Returns 1 when
    /// the privilege is held and 0 otherwise. May be called from user,
    /// kernel or interrupt context.
    pub fn priv_policy_only(cr: *const cred_t, priv_: c_int, all: boolean_t) -> boolean_t;
    /// `priv_policy_choice(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Behaves like [`priv_policy_only`] but records successfully used
    /// non-basic privileges in the audit trail. Returns 1 when the privilege
    /// is held and 0 otherwise. May be called from user, kernel or interrupt
    /// context.
    pub fn priv_policy_choice(cr: *const cred_t, priv_: c_int, all: boolean_t) -> boolean_t;
}
