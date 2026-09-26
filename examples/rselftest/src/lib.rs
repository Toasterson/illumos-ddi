//! A pseudo driver that exercises the safe DDI layer inside a running kernel.
//!
//! When it attaches, the driver runs one small test for each family of the
//! `ddi` crate that a pseudo device can use without hardware: properties,
//! mutexes, reader-writer locks, semaphores, condition variables, the clock
//! functions, one-shot and periodic timers, task queues, named kernel
//! statistics and name-value lists. The layered driver interface may not be
//! used from `attach(9E)`, so its tests run on request through a second
//! ioctl, against `/dev/zero` and `/dev/null`. Each result is logged with
//! `dev_err(9F)`, counted in a kstat, and reported through an ioctl, so the
//! outcome can be read from `dmesg`, from `kstat -m rselftest` and from a
//! test program. The attach succeeds even when a test fails, because the
//! point is to report failures, not to hide them behind a missing device.
//!
//! Register access, PCI configuration space, interrupts and DMA need a real
//! device to bind to, so they are not exercised here.
#![no_std]

use core::ffi::CStr;
use core::num::NonZeroU32;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::time::Duration;

use ddi::prelude::*;
use ddi::time::{busy_wait, delay};
use zerocopy::{Immutable, IntoBytes};

ddi::module! {
    driver: RSelfTest,
    ops: [char],
}

/// Returns the self-test results as a [`Results`]. Encoded as
/// `_IOR('S', 1, Results)`.
pub const RSELFTEST_IOC_RESULTS: IoctlCmd = IoctlCmd::ior::<Results>(b'S', 1);

/// Runs the layered driver tests and returns an [`LdiResults`]. Encoded as
/// `_IOR('S', 2, LdiResults)`.
pub const RSELFTEST_IOC_LDI: IoctlCmd = IoctlCmd::ior::<LdiResults>(b'S', 2);

/// The results of the layered driver tests.
#[derive(Clone, Copy, Debug, Default, IntoBytes, Immutable)]
#[repr(C)]
pub struct LdiResults {
    /// How many checks passed.
    pub passed: u32,
    /// How many checks failed.
    pub failed: u32,
    /// Bit `n` is set when check number `n` failed.
    pub failed_mask: u64,
}

/// The self-test results, as copied out to userland.
#[derive(Clone, Copy, Debug, Default, IntoBytes, Immutable)]
#[repr(C)]
pub struct Results {
    /// How many tests passed.
    pub passed: u32,
    /// How many tests failed.
    pub failed: u32,
    /// Bit `n` is set when test number `n` failed.
    pub failed_mask: u64,
    /// How often the periodic timer has run since attach.
    pub periodic_ticks: u64,
}

ddi::named_kstat! {
    /// The statistics published under `rselftest:0:selftest`.
    pub struct SelfTestStats {
        /// Tests that passed at attach.
        pub passed: u32,
        /// Tests that failed at attach.
        pub failed: u32,
        /// Runs of the periodic timer, updated on every ioctl.
        pub periodic_ticks: u64 = "periodic-ticks",
        /// How often the device has been opened.
        pub opens: u64,
    }
}

/// The driver type that [`ddi::module!`] is invoked on.
pub struct RSelfTest;

/// The state of the attached instance.
pub struct Instance {
    dev: DevInfo,
    _node: MinorNode,
    stats: TypedKstat<SelfTestStats>,
    _periodic: Periodic<fn(&Sleepable)>,
    results: Results,
}

static PERIODIC_TICKS: AtomicU64 = AtomicU64::new(0);
static TASKS_RUN: AtomicU64 = AtomicU64::new(0);
static TIMEOUT_FIRED: AtomicBool = AtomicBool::new(false);

fn count_task(_ctx: &Sleepable) {
    TASKS_RUN.fetch_add(1, Ordering::SeqCst);
}

fn periodic_tick(_ctx: &Sleepable) {
    PERIODIC_TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Collects test results and logs each one.
struct Report {
    dev: DevInfo,
    results: Results,
    index: u32,
}

impl Report {
    fn check(&mut self, name: &str, ok: bool) {
        if ok {
            self.results.passed += 1;
            self.dev.note(format_args!("self-test {} {name}: ok", self.index));
        } else {
            self.results.failed += 1;
            self.results.failed_mask |= 1 << self.index;
            self.dev.warn(format_args!("self-test {} {name}: FAILED", self.index));
        }
        self.index += 1;
    }
}

fn cstr_eq(a: &CStr, b: &CStr) -> bool {
    a.to_bytes() == b.to_bytes()
}

fn test_props(r: &mut Report, ctx: &Sleepable) {
    let dev = r.dev;
    let p = dev.props();
    r.check("prop int from conf", p.int(c"answer", ctx) == Ok(Some(42)));
    r.check("prop int_or default", p.int_or(c"no-such-property", 7, ctx) == 7);
    r.check("prop missing is None", matches!(p.string(c"no-such-property", ctx), Ok(None)));
    r.check(
        "prop string from conf",
        matches!(p.string(c"greeting", ctx), Ok(Some(s)) if cstr_eq(&s, c"hello from rselftest.conf")),
    );
    r.check("prop int array from conf", matches!(p.int_array(c"primes", ctx), Ok(Some(a)) if *a == [2, 3, 5, 7, 11]));
    r.check(
        "prop string array from conf",
        matches!(p.strings(c"flavours", ctx), Ok(Some(s))
            if s.len() == 2 && s.get(0).is_some_and(|v| cstr_eq(v, c"vanilla"))
                && s.get(1).is_some_and(|v| cstr_eq(v, c"chocolate"))),
    );
    r.check("prop exists", p.exists(c"answer", ctx) && !p.exists(c"no-such-property", ctx));

    let published = p.set_int(c"rselftest-int", -5, ctx).is_ok()
        && p.set_int64(c"rselftest-int64", 1 << 40, ctx).is_ok()
        && p.set_string(c"rselftest-string", c"published by Rust", ctx).is_ok()
        && p.set_strings(c"rselftest-strings", &[c"one", c"two", c"three"], ctx).is_ok()
        && p.set_bytes(c"rselftest-bytes", &[0xde, 0xad, 0xbe, 0xef], ctx).is_ok()
        && p.set_int_array(c"rselftest-ints", &[1, 2, 3], ctx).is_ok();
    r.check("prop update", published);
    r.check(
        "prop read back",
        p.int(c"rselftest-int", ctx) == Ok(Some(-5))
            && p.int64(c"rselftest-int64", ctx) == Ok(Some(1 << 40))
            && matches!(p.string(c"rselftest-string", ctx), Ok(Some(s)) if cstr_eq(&s, c"published by Rust"))
            && matches!(p.strings(c"rselftest-strings", ctx), Ok(Some(s)) if s.len() == 3)
            && matches!(p.bytes(c"rselftest-bytes", ctx), Ok(Some(b)) if *b == [0xde, 0xad, 0xbe, 0xef])
            && matches!(p.int_array(c"rselftest-ints", ctx), Ok(Some(a)) if *a == [1, 2, 3]),
    );
    r.check("prop wrong type", matches!(p.int64(c"greeting", ctx), Ok(None) | Err(_)));
    r.check("prop remove", p.remove(c"rselftest-int", ctx) && !p.exists(c"rselftest-int", ctx));
}

fn test_sync(r: &mut Report, ctx: &Sleepable) {
    let m = Mutex::new(1u32);
    {
        let mut g = m.lock();
        *g += 1;
        r.check("mutex held", m.is_held_by_current_thread() && m.try_lock().is_none());
    }
    r.check("mutex released", *m.lock() == 2 && !m.is_held_by_current_thread());

    let rw = RwLock::new(10u32);
    {
        let a = rw.read();
        let b = rw.try_read();
        r.check("rwlock shared readers", *a == 10 && b.is_some() && rw.try_write().is_none());
    }
    {
        let mut w = rw.write();
        *w = 11;
        let rd = w.downgrade();
        r.check("rwlock downgrade", *rd == 11 && rw.try_write().is_none());
        let up = rd.try_upgrade();
        r.check("rwlock upgrade", up.is_ok());
    }

    let sem = Semaphore::new(1);
    let first = sem.try_acquire();
    let second = sem.try_acquire();
    sem.release();
    r.check("semaphore count", first && !second && sem.try_acquire());
    sem.release();

    let cv = CondVar::new();
    let g = m.lock();
    let start = HrTime::now();
    let (_g, res) = cv.wait_timeout(g, Duration::from_millis(20), ctx);
    r.check("condvar timeout", res.timed_out() && start.elapsed() >= Duration::from_millis(15));
}

fn test_time(r: &mut Report, ctx: &Sleepable) {
    let t = Ticks::from_duration(Duration::from_millis(10));
    r.check("ticks round trip", t.get() > 0 && t.as_duration() >= Duration::from_millis(10));

    let start = HrTime::now();
    delay(t, ctx);
    r.check("delay", start.elapsed() >= Duration::from_millis(9));

    let start = HrTime::now();
    busy_wait(Duration::from_micros(500));
    r.check("busy wait", start.elapsed() >= Duration::from_micros(500));

    let before = ddi::time::lbolt();
    delay(Ticks::new(2), ctx);
    r.check("lbolt advances", ddi::time::lbolt().get() >= before.get() + 2);

    TIMEOUT_FIRED.store(false, Ordering::SeqCst);
    let to = Timeout::new(Ticks::new(1), |_ctx: &Atomic| TIMEOUT_FIRED.store(true, Ordering::SeqCst), ctx);
    delay(Ticks::from_duration(Duration::from_millis(50)), ctx);
    r.check("timeout fires", to.has_fired() && TIMEOUT_FIRED.load(Ordering::SeqCst));
    drop(to);

    let never = Timeout::new(Ticks::from_duration(Duration::from_secs(3600)), |_ctx: &Atomic| {}, ctx);
    r.check("timeout cancel", never.cancel().is_some());
}

fn test_taskq(r: &mut Report, ctx: &Sleepable) {
    let Some(threads) = NonZeroU32::new(2) else { return };
    match TaskQueue::new(r.dev, c"rselftest_tq", threads, ctx) {
        Ok(tq) => {
            TASKS_RUN.store(0, Ordering::SeqCst);
            let mut ok = true;
            for _ in 0..8 {
                ok &= tq.dispatch(count_task, ctx).is_ok();
            }
            tq.wait(ctx);
            r.check("taskq dispatch and wait", ok && TASKS_RUN.load(Ordering::SeqCst) == 8);
            let nosleep = tq.try_dispatch(count_task);
            drop(tq);
            r.check("taskq destroy runs pending", nosleep.is_ok() && TASKS_RUN.load(Ordering::SeqCst) == 9);
        }
        Err(_) => r.check("taskq create", false),
    }
}

impl Driver for RSelfTest {
    type Instance = Instance;
    const LINKINFO: &'static CStr = c"Rust DDI self-test v1";

    fn attach(dev: DevInfo, ctx: &Sleepable) -> Result<Instance, DdiError> {
        let inst = dev.instance();
        let minor = Minor(u32::try_from(inst.0).map_err(|_| DdiError)?);
        let node = dev.create_minor_node(c"rselftest", MinorKind::Char, minor, NodeType::Pseudo)?;

        let mut r = Report { dev, results: Results::default(), index: 0 };
        test_props(&mut r, ctx);
        test_sync(&mut r, ctx);
        test_time(&mut r, ctx);
        test_taskq(&mut r, ctx);
        test_nvlist(&mut r, ctx);

        let stats = TypedKstat::<SelfTestStats>::new(
            KstatId { module: c"rselftest", instance: inst, name: c"selftest", class: c"misc" },
            ctx,
        )?;
        {
            let mut s = stats.lock();
            s.passed.set(r.results.passed);
            s.failed.set(r.results.failed);
        }
        let periodic = Periodic::new(Duration::from_millis(10), periodic_tick as fn(&Sleepable), ctx);

        dev.note(format_args!("self-test finished: {} passed, {} failed", r.results.passed, r.results.failed));
        dev.report();
        Ok(Instance { dev, _node: node, stats, _periodic: periodic, results: r.results })
    }

    fn detach(inst: Instance, dev: DevInfo, ctx: &Sleepable) -> Result<(), (Instance, DdiError)> {
        dev.props().remove_all(ctx);
        drop(inst);
        Ok(())
    }
}

fn test_nvlist(r: &mut Report, ctx: &Sleepable) {
    let mut nested = NvList::new(ctx);
    let built = nested.add_string(c"inner", c"deep", ctx).is_ok();
    let mut l = NvList::new(ctx);
    let built = built
        && l.add_u32(c"answer", 42, ctx).is_ok()
        && l.add_i64(c"negative", -7, ctx).is_ok()
        && l.add_string(c"name", c"rselftest", ctx).is_ok()
        && l.add_bool(c"enabled", true, ctx).is_ok()
        && l.add_u64_array(c"sizes", &[1, 2, 3], ctx).is_ok()
        && l.add_strings(c"tags", &[c"a", c"b"], ctx).is_ok()
        && l.add_nvlist(c"child", nested.as_list(), ctx).is_ok();
    r.check("nvlist add", built);
    r.check(
        "nvlist lookup",
        l.lookup_u32(c"answer") == Ok(Some(42))
            && l.lookup_i64(c"negative") == Ok(Some(-7))
            && matches!(l.lookup_string(c"name"), Ok(Some(s)) if cstr_eq(s, c"rselftest"))
            && l.lookup_bool(c"enabled") == Ok(Some(true))
            && matches!(l.lookup_u64_array(c"sizes"), Ok(Some(a)) if *a == [1, 2, 3])
            && matches!(l.lookup_strings(c"tags"), Ok(Some(t)) if t.len() == 2)
            && matches!(l.lookup_nvlist(c"child"), Ok(Some(c))
                if matches!(c.lookup_string(c"inner"), Ok(Some(s)) if cstr_eq(s, c"deep"))),
    );
    r.check("nvlist missing is None", l.lookup_u32(c"no-such-name") == Ok(None));
    r.check(
        "nvlist type mismatch reads as absent",
        l.lookup_u64(c"answer") == Ok(None)
            && matches!(l.lookup_pair(c"answer"), Ok(Some(p)) if p.data_type() == ddi::sys::data_type_t::DATA_TYPE_UINT32),
    );
    r.check("nvlist iterate", l.pairs().count() == 7);

    let mut round_trip = true;
    for enc in [NvEncoding::Native, NvEncoding::Xdr] {
        round_trip &= match l.pack(enc, ctx) {
            Ok(packed) => match NvList::unpack(&packed, ctx) {
                Ok(u) => u.lookup_u32(c"answer") == Ok(Some(42)) && u.pairs().count() == 7,
                Err(_) => false,
            },
            Err(_) => false,
        };
    }
    r.check("nvlist pack and unpack", round_trip);
    r.check("nvlist dup", matches!(l.dup(ctx), Ok(d) if d.lookup_i64(c"negative") == Ok(Some(-7))));
    r.check("nvlist remove", l.remove(c"answer") == Ok(true) && l.lookup_u32(c"answer") == Ok(None));
    let nosleep = match NvList::try_new() {
        Ok(mut n) => n.add_u8(c"b", 1).is_ok() && n.lookup_u8(c"b") == Ok(Some(1)),
        Err(_) => false,
    };
    r.check("nvlist without sleeping", nosleep);
}

/// The byte the device returns at offset `off`.
fn pattern(off: u64) -> u8 {
    (off % 251) as u8
}

/// Runs the layered driver tests. The manual forbids LDI opens in
/// `attach(9E)`, so these run from the ioctl path instead.
fn ldi_tests(dev: DevInfo, cred: &Cred<'_>, ctx: &Sleepable) -> LdiResults {
    let mut res = LdiResults::default();
    let mut check = |ok: bool| {
        if ok {
            res.passed += 1;
        } else {
            res.failed_mask |= 1 << (res.passed + res.failed);
            res.failed += 1;
        }
    };
    let Ok(ident) = LdiIdent::from_dip(&dev, ctx) else {
        check(false);
        return res;
    };
    check(true);
    // `/dev/zero` cannot serve a kernel caller: `mmrw` zeroes the buffer
    // with `uzero`, which faults on a kernel address. The driver's own node
    // serves a byte pattern through `uiomove(9F)` instead, which handles
    // kernel-space requests, so the test reads from itself.
    match LdiHandle::open_by_name(&ident, c"/devices/pseudo/rselftest@0:rselftest", LdiFlags::READ, cred, ctx) {
        Ok(me) => {
            let mut buf = [0u8; 64];
            let n = me.read(&mut buf, 3, cred, ctx);
            check(n == Ok(64) && buf.iter().enumerate().all(|(i, b)| *b == pattern(3 + i as u64)));
            check(me.get_dev(ctx).is_ok());
            check(me.close(ctx).is_ok());
        }
        Err(_) => check(false),
    }
    let kernel = Cred::kernel();
    match LdiHandle::open_by_name(&ident, c"/dev/null", LdiFlags::WRITE, &kernel, ctx) {
        Ok(null) => {
            check(matches!(null.write(&[7u8; 100], 0, &kernel, ctx), Ok(100)));
            drop(null);
            check(true);
        }
        Err(_) => check(false),
    }
    check(LdiHandle::open_by_name(&ident, c"/dev/rselftest-no-such-device", LdiFlags::READ, cred, ctx).is_err());
    res
}

impl CharDevice for Instance {
    fn open(
        &self,
        _minor: Minor,
        _flags: OpenFlags,
        _otyp: OpenType,
        _cred: &Cred<'_>,
        _ctx: &Sleepable,
    ) -> Result<()> {
        self.stats.lock().opens.inc();
        Ok(())
    }

    fn read(&self, _minor: Minor, uio: &mut Uio<'_>, _cred: &Cred<'_>, _ctx: &Sleepable) -> Result<()> {
        let mut chunk = [0u8; 256];
        while uio.resid() > 0 {
            let off = u64::try_from(uio.offset()).map_err(|_| Errno::EINVAL)?;
            let n = uio.resid().min(chunk.len());
            for (i, b) in chunk[..n].iter_mut().enumerate() {
                *b = pattern(off + i as u64);
            }
            uio.copy_out(&chunk[..n])?;
        }
        Ok(())
    }

    fn ioctl(&self, _minor: Minor, cmd: IoctlCmd, arg: IoctlArg<'_>, cred: &Cred<'_>, ctx: &Sleepable) -> Result<i32> {
        match cmd {
            RSELFTEST_IOC_RESULTS => {
                let ticks = PERIODIC_TICKS.load(Ordering::Relaxed);
                self.stats.lock().periodic_ticks.set(ticks);
                let out = Results { periodic_ticks: ticks, ..self.results };
                arg.copy_out(&out)?;
                Ok(0)
            }
            RSELFTEST_IOC_LDI => {
                let res = ldi_tests(self.dev, cred, ctx);
                self.dev.note(format_args!("LDI tests: {} passed, {} failed", res.passed, res.failed));
                arg.copy_out(&res)?;
                Ok(0)
            }
            _ => Err(Errno::ENOTTY),
        }
    }
}
