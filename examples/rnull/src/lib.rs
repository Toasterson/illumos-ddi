//! `rnull`: a pseudo character device.
//!
//! - `read(2)` returns EOF.
//! - `write(2)` discards everything and counts it.
//! - `RNULL_IOC_VERSION` returns the driver version.
//! - `RNULL_IOC_STATS` returns the counters.
//!
//! Load with `add_drv rnull`; the node is `/devices/pseudo/rnull@0:rnull`.
#![no_std]

use ddi::prelude::*;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

ddi::module! {
    driver: RNull,
    ops: [char],
}

/// ioctl group letter.
const RNULL_IOC: u8 = b'N';
/// `_IOR('N', 1, u32)`: driver version.
pub const RNULL_IOC_VERSION: IoctlCmd = IoctlCmd::ior::<u32>(RNULL_IOC, 1);
/// `_IOR('N', 2, Stats)`: counters.
pub const RNULL_IOC_STATS: IoctlCmd = IoctlCmd::ior::<Stats>(RNULL_IOC, 2);
/// `_IO('N', 3)`: reset counters. Requires `PRIV_SYS_DEVICES`.
pub const RNULL_IOC_RESET: IoctlCmd = IoctlCmd::io(RNULL_IOC, 3);

/// Driver version reported by `RNULL_IOC_VERSION`.
pub const VERSION: u32 = 1;

/// Counters shared with userland; layout is the ABI.
#[derive(Clone, Copy, Debug, Default, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Stats {
    /// Number of `open(9E)` calls.
    pub opens: u64,
    /// Number of `read(9E)` calls.
    pub reads: u64,
    /// Number of `write(9E)` calls.
    pub writes: u64,
    /// Bytes discarded by `write(9E)`.
    pub bytes_discarded: u64,
}

/// The module.
pub struct RNull;

/// Per-instance state.
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
