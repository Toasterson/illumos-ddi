//! A pseudo character device that behaves like `/dev/null` and keeps count.
//!
//! Reading from the device returns end of file. Writing to it discards the
//! data and counts the bytes. Three ioctls report the driver version, return
//! the counters, and reset them; the reset requires the `PRIV_SYS_DEVICES`
//! privilege. The driver exists to prove the toolchain from Rust source to a
//! loaded kernel module and to serve as a template for other drivers.
//!
//! Install it with `add_drv rnull`. The device node appears as
//! `/devices/pseudo/rnull@0:rnull`.
#![no_std]

use ddi::prelude::*;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

ddi::module! {
    driver: RNull,
    ops: [char],
}

/// The ioctl group letter that all `rnull` commands share.
const RNULL_IOC: u8 = b'N';
/// Returns the driver version as a `u32`. Encoded as `_IOR('N', 1, u32)`.
pub const RNULL_IOC_VERSION: IoctlCmd = IoctlCmd::ior::<u32>(RNULL_IOC, 1);
/// Returns the counters as a [`Stats`]. Encoded as `_IOR('N', 2, Stats)`.
pub const RNULL_IOC_STATS: IoctlCmd = IoctlCmd::ior::<Stats>(RNULL_IOC, 2);
/// Resets the counters. Requires the `PRIV_SYS_DEVICES` privilege. Encoded
/// as `_IO('N', 3)`.
pub const RNULL_IOC_RESET: IoctlCmd = IoctlCmd::io(RNULL_IOC, 3);

/// The version reported by [`RNULL_IOC_VERSION`].
pub const VERSION: u32 = 1;

/// The counters, as copied out to userland. The layout of this structure is
/// the ABI of [`RNULL_IOC_STATS`].
#[derive(Clone, Copy, Debug, Default, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Stats {
    /// How often the device has been opened.
    pub opens: u64,
    /// How often `read(9E)` has been called.
    pub reads: u64,
    /// How often `write(9E)` has been called.
    pub writes: u64,
    /// How many bytes `write(9E)` has discarded in total.
    pub bytes_discarded: u64,
}

/// The driver type that [`module!`] is invoked on.
pub struct RNull;

/// The state of one attached instance. Dropping it removes the minor node.
pub struct RNullInstance {
    dev: DevInfo,
    _node: MinorNode,
    stats: Mutex<Stats>,
}

impl Driver for RNull {
    type Instance = RNullInstance;
    const LINKINFO: &'static core::ffi::CStr = c"Rust null pseudo device v1";

    fn attach(dev: DevInfo, _ctx: &Sleepable) -> Result<RNullInstance, DdiError> {
        let inst = dev.instance();
        let minor = Minor(u32::try_from(inst.0).map_err(|_| DdiError)?);
        let node = dev.create_minor_node(c"rnull", MinorKind::Char, minor, NodeType::Pseudo)?;
        dev.report();
        Ok(RNullInstance { dev, _node: node, stats: Mutex::new(Stats::default()) })
    }

    fn detach(inst: RNullInstance, dev: DevInfo, _ctx: &Sleepable) -> Result<(), (RNullInstance, DdiError)> {
        let s = *inst.stats.lock();
        dev.note(format_args!("detaching after {} writes, {} bytes discarded", s.writes, s.bytes_discarded));
        Ok(())
    }
}

impl CharDevice for RNullInstance {
    fn open(&self, _minor: Minor, _flags: OpenFlags, otyp: OpenType, _cred: &Cred<'_>, _ctx: &Sleepable) -> Result<()> {
        if otyp != OpenType::Char {
            return Err(Errno::EINVAL);
        }
        self.stats.lock().opens += 1;
        Ok(())
    }

    fn read(&self, _minor: Minor, _uio: &mut Uio<'_>, _cred: &Cred<'_>, _ctx: &Sleepable) -> Result<()> {
        self.stats.lock().reads += 1;
        Ok(()) // nothing copied: EOF
    }

    fn write(&self, _minor: Minor, uio: &mut Uio<'_>, _cred: &Cred<'_>, _ctx: &Sleepable) -> Result<()> {
        let n = uio.discard_all();
        let mut s = self.stats.lock();
        s.writes += 1;
        s.bytes_discarded += n as u64;
        Ok(())
    }

    fn ioctl(&self, _minor: Minor, cmd: IoctlCmd, arg: IoctlArg<'_>, cred: &Cred<'_>, _ctx: &Sleepable) -> Result<i32> {
        match cmd {
            RNULL_IOC_VERSION => {
                arg.copy_out(&VERSION)?;
                Ok(0)
            }
            RNULL_IOC_STATS => {
                let s = *self.stats.lock();
                arg.copy_out(&s)?;
                Ok(0)
            }
            RNULL_IOC_RESET => {
                if !cred.has_sys_devices_priv() {
                    return Err(Errno::EPERM);
                }
                *self.stats.lock() = Stats::default();
                self.dev.note(format_args!("counters reset"));
                Ok(0)
            }
            _ => Err(Errno::ENOTTY),
        }
    }
}
