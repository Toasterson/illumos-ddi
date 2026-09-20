# illumos DDI for Rust: design

This document is a draft. It was written on 20 September 2026, and the first
two milestones described in section 9 were reached the same day. The crate
names used throughout (`ddi-sys`, `ddi`, `ddi-<subsystem>`) are working
names. Naming is one of the open questions in section 10.

## 1. Goal

We want to write illumos kernel drivers in Rust against the documented
Driver Developer Interface, the DDI/DKI described in section 9 of the
manual, in a form that the illumos project itself could adopt. Rust drivers
for illumos exist today, but each one carries its own hand-written
`extern "C"` declarations. That works for a single driver. It does not
scale to a second one, and it gives the project nothing it can review or
maintain.

The design has two layers. This mirrors the convention the illumos GitHub
organisation already follows in userland, where `libdevinfo-sys` sits
beneath `devinfo-rs` and `libscf-sys` beneath `smf-rs`.

The raw layer consists of `ddi-sys` and one `ddi-<subsystem>-sys` crate per
kernel framework. These crates contain `#[repr(C)]` structures, opaque
handles, constants and `extern "C"` prototypes, one item per section 9
manual page, and nothing else. Everything in them is unsafe by nature.

The safe layer consists of `ddi` and one `ddi-<subsystem>` crate per
framework. Here the rules that the manual pages state in prose become the
shape of types: which context may allocate, who holds a lock, whether DMA
memory is bound, which return convention an entry point uses. These crates
use `unsafe` internally and never expose it at the driver boundary.

Drivers depend on the safe layer only. If a driver crate contains `unsafe`
anywhere other than hardware register access, something in the safe layer
is missing and should be added there.

Some things are deliberately out of scope for the first iterations. Nexus
drivers with their `bus_ops` and `ndi_*` interfaces are not covered. The
STREAMS framework is bound only as far as MAC needs it. Filesystem modules
(`modlfs`) are not covered. SPARC is not covered.

## 2. Prior art and what it teaches

Four earlier efforts shaped this design. Each one taught something
different.

Robert Mustacchi's `rust-illumos-kmod-poc` from 2021 ports the `pchtemp`
driver to a single `rmod.rs`, compiled with `rustc --emit=obj
-C code-model=kernel -C no-redzone -C soft-float -C panic=abort`. All
structures and constants are typed inline, `nodev` is transmuted into
whichever slot needs it, and at least one constant is wrong on inspection.
Its own README calls this "the wrong way, but faster". It proves that the
distance from Rust source to a loadable module is short. It also shows, as
a pleasant side effect, that a call to a `__dtrace_probe_*` symbol gives you
an SDT probe for free.

`apopiak/LoReFS` from 2018 is a lofs clone built with xargo for the old
`x86_64-sun-solaris` target, with a C shim in front of the Rust code. Its
main lesson is that a filesystem needs the VFS and vnode interfaces, which
are not part of the DDI at all. That is why section 4 has an explicitly
private tier.

Oxide's `xde` driver in the `opte` repository, together with its
`illumos-sys-hdrs` crate, is the only production Rust kernel module on
illumos we know of. Its build recipe is the one to copy: a `no_std`
static library, a custom target JSON with `code-model: kernel`,
`disable-redzone`, no SSE, soft float, `frame-pointer: always` and
`panic: abort`, then `-Zbuild-std`, and finally
`ld -ztype=kmod -Ndrv/dld -Nmisc/mac ... -z allextract xde.a`. The header
crate is less of a model. It is flat, a single `kernel.rs`, and it mixes
raw declarations with safe helpers such as `impl kmutex_t { fn new }`. Its
one bindgen-generated file, `ip.rs`, runs to 28,000 lines and the authors
say in a comment that they regret it. Two details are worth avoiding
outright. The `GlobalAlloc` implementation passes `KM_SLEEP`
unconditionally, and the panic handler allocates a `CString`, both of which
are unsound in interrupt context. The module also needs a stub for
`_Unwind_Resume`, because `alloc` had not been built with `panic=abort`.

Oxide's `propolis` also carries `bhyve-api` and `viona-api`, `no_std` FFI
crates for kernel ioctls. They show the same discipline (`repr(C)`,
versioned ABI) but from userland, so they are not a template for the DDI.

The Linux kernel's `rust/kernel` crate is the most developed comparison.
Its vocabulary for the safe layer is worth borrowing: a `module!` macro,
driver traits, typed errors, pinned locks and the `ForeignOwnable` idea.
Its approach to bindings is not. Linux runs bindgen over the kernel headers
with the kernel's own CFLAGS because it has no stable internal ABI. illumos
does have one. The DDI is a committed interface with manual pages, so a
curated, hand-written surface is both feasible and more honest about what
is actually supported.

## 3. Ground truth from the gate

Everything in this section was read from `illumos/illumos-gate` at commit
`0fb1d5d8d3`. These are the facts the design has to satisfy.

### 3.1 How a kernel module is linked

The rule that links every kernel module lives at
`usr/src/uts/Makefile.targ:40`:

```
$(LD) -ztype=kmod $(LDFLAGS) -o $@ $(OBJECTS)
```

It is followed by a CTF merge against `genunix` and the usual
post-processing. The important flag is `-ztype=kmod`. Reading
`usr/src/cmd/sgs/libld/common/args.c` shows what it does: it implies `-r`
and `-dy` together, it makes an explicit `-d` a fatal error, and it emits a
`DT_SUNW_KMOD` tag in the dynamic section. So nobody should pass `-r`
themselves; the Writing Device Drivers book predates this flag.

Dependencies on other modules are expressed as `-N drv/blkdev`,
`-N misc/mac` and so on. Each becomes a `DT_NEEDED` entry, and `krtld`
follows them recursively at load time. The older mechanism, a `_depends_on`
string in the module, is only consulted when no `DT_NEEDED` entries exist,
and the kernel warns about it. New code should not use it.

There is an optional and rather useful addition. Linking with
`-z defs -M usr/src/uts/common/mapfiles/ddi.mapfile` (plus `mac.mapfile`
and others as needed) makes the link fail if the module references any
symbol outside the listed set. The README next to those mapfiles says they
are not a formal statement of what is public. Even so, the `ddi` mapfile is
the closest thing the gate has to a machine-readable list of DDI symbols,
and drivers such as `i40e` and `e1000g` link against it.

### 3.2 Compile model for amd64 kernel code

After the `cw` compiler wrapper has translated the Studio-style options
(see `usr/src/Makefile.master`, `usr/src/uts/Makefile.uts` and
`usr/src/tools/cw/cw.c`), amd64 kernel code is compiled with these flags:

```
-m64 -ffreestanding -mcmodel=kernel -mno-red-zone -mno-mmx -mno-sse -msave-args
-fno-asynchronous-unwind-tables -fno-shrink-wrap -fstack-protector-strong
-mindirect-branch=thunk-extern -mindirect-branch-register
-D_KERNEL -D_SYSCALL32 -D_SYSCALL32_IMPL -D_ELF64 -D_DDI_STRICT -Dsun -D__sun -D__SVR4
```

Each of these has a consequence for Rust.

The custom target in `targets/x86_64-illumos-kmod.json` sets
`code-model: kernel`, `relocation-model: static`, `disable-redzone: true`,
the feature string `-mmx,-sse,-sse2,…,+soft-float`,
`panic-strategy: abort`, `frame-pointer: always` and
`eh-frame-header: false`. It is Oxide's target file with `os` left as
`illumos`, so that `cfg(target_os = "illumos")` holds in the crates.

`core` and `alloc` have to be rebuilt for that target, which means
`-Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem
-Zjson-target-spec` on a nightly toolchain. This is the only hard nightly
dependency in the whole design. The crates themselves are written for
stable Rust and use neither `extern_types` nor `alloc_error_handler`.

`-D_DDI_STRICT` compiles `ddi_obsolete.h` out of every module in the gate.
Nothing from that header is bound.

The kernel is built with the stack protector and exports `__stack_chk_fail`
and `__stack_chk_guard`, both listed in `ddi.mapfile`. Rust can opt in later
with `-Zstack-protector=strong`. For now it is off.

`-msave-args` has no Rust equivalent, so `mdb ::stack` will not display
argument values for Rust frames. Frame pointers are kept, so stack walking
itself works.

Kernel thread stacks are five pages (`DEFAULTSTKSZ` in
`i86pc/sys/machparam.h`). That rules out formatting into a `String` or
placing large arrays on the stack. The logging in the safe layer formats
into a fixed 256-byte buffer for this reason.

Retpolines are available in Rust only through nightly flags. We accept the
gap and document it. Indirect calls from Rust into `cb_ops` are rare in
practice.

### 3.3 What "public DDI" means, mechanically

There are three signals, and none of them is sufficient on its own. The
inventory in `docs/ddi-inventory.md` has the numbers.

The first is the `.SH INTERFACE LEVEL` section of the manual pages in
`usr/src/man/man9{e,f,s}`. It reads either "Architecture independent level 1
(DDI/DKI)." or "illumos DDI specific (illumos DDI).", sometimes followed by
a note that the interface is obsolete. Between 92 and 99 percent of the
pages have this section.

The second is the `Interface Stability` row in the `ATTRIBUTES` table, with
values Committed, Evolving, Obsolete, Private or Volatile. Only a minority
of pages carry it.

The third is membership in one of the `usr/src/uts/common/mapfiles/*.mapfile`
files.

The mapfile turns out to be incomplete relative to the manual pages.
`getminor(9F)`, `getmajor(9F)`, `makedevice(9F)`, `ddi_prop_exists(9F)`,
`ddi_model_convert_from(9F)`, `rw_read_locked(9F)`, `crgetuid(9F)`,
`uioskip(9F)` and `vpanic` are all documented DDI and all absent from
`ddi.mapfile` as of 20 September 2026. Linking `rnull` with
`-z defs -M ddi.mapfile` fails on exactly two of them, `getminor` and
`uioskip`. The conclusion is that the mapfile can gate a link but cannot
define the binding surface. The planned `xtask lint-man` should report
"documented but not in mapfile" so that the gaps can be sent upstream as
mapfile additions.

Header names, on the other hand, tell you nothing about what is public.
`ddi_impldefs.h` and `mac_impl.h` are installed into `/usr/include/sys`
right next to `sunddi.h`. The naming does work in the negative direction:
anything called `*_impl.h`, `*_impldefs.h` or `esunddi.h` (whose own
comment says that "drivers calling these functions are not portable") is
private.

### 3.4 Fixed ABI facts a binding must hard-code (amd64)

A few layouts are part of the ABI and have to be written down in the
bindings. The table lists them with their source in the gate.

| Type | Layout | Source |
| --- | --- | --- |
| `kmutex_t` | `void *_opaque[1]` (8 bytes) | `sys/mutex.h:65` |
| `kcondvar_t` | `ushort_t _opaque` (2 bytes) | `sys/condvar.h:60` |
| `krwlock_t` | `void *_opaque[1]` (8 bytes) | `sys/rwlock.h:57` |
| `ksema_t` | `void *_opaque[2]` | `sys/semaphore.h:50` |
| `struct modlinkage` | `int ml_rev; void *ml_linkage[7]` (64 bytes, LP64) | `sys/modctl.h:217-219` |
| `dev_ops` | `DEVO_REV == 4`; `cb_ops`: `CB_REV == 1` | `sys/devops.h:363-364` |
| `dev_t` | `ulong_t`; `major_t`/`minor_t` `uint_t` | `sys/types.h` |
| `ddi_device_acc_attr_t` | `u16 version; u8 endian; u8 dataorder; u8 access` | `sys/dditypes.h` |
| `struct uio` | documented fields in `uio(9S)`; layout in `sys/uio.h` | must be layout-tested |

These are exactly the things the CTF-driven layout test in section 8.1 has
to pin. They were spot-checked against the running kernel on `oibldofgate`
(build `illumos-cda5b9a49f`) with `mdb -k`. `struct dev_ops` is 0x58 bytes
with `devo_attach` at offset 0x20, `struct cb_ops` is 0x88, `struct uio` is
0x30 with `uio_resid` at 0x28, `kmutex_t` is 8, `kcondvar_t` is 2 and
`struct modlinkage` is 0x40. All of them match the constant assertions in
`ddi-sys`.

## 4. Crate boundaries

There is one `-sys` crate per documented interface family. The test for
drawing a boundary is a practical one. Does this family have its own set of
section 9 entry points or its own `9F` prefix, its own headers, and
usually its own kernel module that a driver would name with `-N`? If so,
it gets a crate. That crate then owns three things: the symbols it
declares, the `-N` dependency it implies, and the mapfile it can be
checked against.

| Crate | Headers (`usr/src/uts/common/sys/`) | man 9 family | kmod dep (`-N`) | mapfile | Tier |
| --- | --- | --- | --- | --- | --- |
| `ddi-sys` | `types.h`, `errno.h`, `param.h`, `modctl.h`, `devops.h`, `conf.h`, `dditypes.h`, `sunddi.h` (core subset), `ddipropdefs.h` (enums only), `cred.h`, `uio.h`, `buf.h`, `open.h`, `file.h` (flags), `stat.h` (`S_IF*`), `cmn_err.h`, `kmem.h`, `mutex.h`, `condvar.h`, `rwlock.h`, `semaphore.h`, `ddi_periodic.h` (API only), `kstat.h`, `sysmacros.h`/`atomic.h` as needed | `Intro(9E)`, `dev_ops(9S)`, `cb_ops(9S)`, `modl*(9S)`, `ddi_*`, `kmem_*`, `mutex_*`, `cv_*`, `rw_*`, `cmn_err`, `uio*`, `bio*`, `kstat_*`, `ddi_prop_*`, `ddi_soft_state_*`, `ddi_taskq_*`, `ddi_periodic_*` | none (genunix/unix) | `ddi` | Committed |
| `ddi-sys` feature `intr` | `ddi_intr.h` | `ddi_intr_*(9F)` (14 pages) | none | `ddi` | Committed |
| `ddi-sys` feature `dma` | `ddidmareq.h`, `sunddi.h` DMA subset | `ddi_dma_*(9F)`, `ddi_dma_attr(9S)`, `ddi_dma_cookie(9S)` | none | `ddi` | Committed |
| `ddi-sys` feature `regs` | `sunddi.h` regs subset, `dditypes.h` | `ddi_regs_map_setup(9F)`, `ddi_get8(9F)` and family | none | `ddi` | Committed |
| `ddi-sys` feature `fm` | `ddifm.h` | `ddi_fm_*(9F)`, `ddi_fm_error(9S)` | none | `ddi` | Committed |
| `ddi-ufm-sys` | `ddi_ufm.h` | `ddi_ufm(9E)`, `ddi_ufm_*(9F)` | none | none | Committed, versioned |
| `ddi-pci-sys` | `pci.h` (constants), `sunddi.h` `pci_config_*` | `pci_config_*(9F)` | none | `ddi` | Committed |
| `ddi-ldi-sys` | `sunldi.h` | `ldi_*(9F)` (24 pages) | none | none | Committed |
| `ddi-mac-sys` | `mac_provider.h`, `mac.h`, `mac_ether.h` | `mac(9E)`, `mc_*(9E)`, `mac_*(9F)`, `mac_register(9S)`, `mac_callbacks(9S)` | `misc/mac` | `mac` | Committed, versioned (`MAC_VERSION_V1`) |
| `ddi-streams-sys` | `stream.h`, `strsun.h`, `stropts.h` | `queue(9S)`, `msgb(9S)`, `qinit(9S)`, `put(9E)`, `srv(9E)`, `q*(9F)`, `mi*(9F)`, `allocb(9F)` and family | none | `ddi` | Committed |
| `ddi-blkdev-sys` | `blkdev.h` | `bd_*(9F)`; only the header and its comments exist, there are no manual pages | `drv/blkdev` | none | Committed by convention but undocumented; the first job is to write `blkdev(9E)` upstream |
| `ddi-scsa-sys` | `scsi/scsi.h` and the public `scsi/scsi_*.h` (not `scsi/impl/`) | `scsi_*(9F)` (41), `tran_*(9E)` (14), `scsi_hba_tran(9S)` and family | `misc/scsi` | none | Committed |
| `ddi-usba-sys` | `usb/usba.h`, `usb/usbai.h` | `usb_*(9F)` (29), `usb_*(9S)` | `misc/usba` | none | Committed |
| `ddi-usba-hcdi-sys` | `usb/usba/hcdi.h` | `usba_hcdi_*(9E)` (12), `usba_*(9F)` | `misc/usba` | none | Committed |
| `ddi-ksensor-sys` | `sensors.h` | `ksensor(9E)`, `ksensor_*(9F)` | `misc/ksensor` | `ksensor` | Evolving, according to the header itself |
| `ddi-cmlb-sys` | `cmlb.h` | none | `misc/cmlb` | none | Private by omission |
| `illumos-kernel-private-sys` | `virtio.h` (`io/virtio/`), `vnode.h`/`vfs.h`, `taskq.h` (raw), `id_space.h`, `zone.h`, items from `esunddi.h` | none | varies | none | Private, pinned to a gate commit, usable only with `--features unstable-gate-<hash>` |

A handful of rules keep these boundaries meaningful.

A `-sys` crate declares only what appears in its listed headers and that
either has a section 9 page or is present in its mapfile. Anything else
belongs in the private tier. It never goes into a Committed crate "just for
now", because "for now" has a way of becoming permanent.

Every declaration carries a doc comment that names its manual page and
interface level, in the form ``/// `kmem_alloc(9F)`. Architecture independent
level 1 (DDI/DKI).`` A CI script, `xtask lint-man`, checks that the named
page exists under `usr/src/man/man9*` and that its interface level is not
marked obsolete.

`-sys` crates contain no `impl` blocks apart from `Default` and `Debug` on
plain data. No allocation, no wrappers, no `unsafe fn` bodies. Oxide's
`impl kmutex_t { fn new }` inside a header crate is precisely the pattern to
avoid.

Function pointers in `9S` structures are `Option<unsafe extern "C" fn(..)>`
with their exact signatures. The safe crate provides correctly typed Rust
stubs for each slot instead of transmuting the kernel's `nodev`, `nulldev`
and `nochpoll`.

Opaque kernel types are zero-sized `#[repr(C)]` structures containing
`PhantomData<(*mut u8, PhantomPinned)>`. This is the stable pattern and
needs no `extern_types`.

Architecture differences, such as `ddi_isa.h` living under `uts/intel/sys/`
rather than `common/`, are handled with `cfg(target_arch)` modules inside
the same crate.

Each `-sys` crate's `Cargo.toml` carries `[package.metadata.kmod]` with a
`depends` list such as `["misc/mac"]`. The link tool assembles the `-N`
flags from the dependency graph, so a driver never writes them out.

## 5. The safe layer: types that express intent

The manual pages state their rules in prose. The safe crate turns each rule
into something the compiler checks. What follows is the contract for the
core `ddi` crate. The subsystem crates apply the same principles.

### 5.1 Two result conventions, two error types

`dev_ops` entry points return `DDI_SUCCESS` or `DDI_FAILURE`. `cb_ops`
entry points return an `errno`. These are unrelated integer spaces, and
mixing them up is a classic bug in C drivers. So they are two types:

```rust
pub struct Errno(NonZeroI32);           // cb_ops, ioctl, uiomove, ddi_copyin
pub struct DdiError;                    // dev_ops, and every ddi_* that returns DDI_*
pub type Result<T, E = Errno> = core::result::Result<T, E>;
```

The property functions have a third space (`DDI_PROP_NOT_FOUND` and
friends). They will get their own `PropError` when the property API grows
beyond the two helpers that exist today.

### 5.2 Module and driver definition

A driver implements the `Driver` trait, implements `CharDevice` on its
instance type if it is a character device, and invokes the `module!` macro:

```rust
ddi::module! {
    driver: RNull,
    ops: [char],                 // or [] for a module without cb_ops
}

pub struct RNull;

impl ddi::Driver for RNull {
    type Instance = RNullInstance;   // per-instance state, Send + Sync
    const LINKINFO: &'static CStr = c"Rust null pseudo device v1";

    fn attach(dev: DevInfo, ctx: &Sleepable) -> Result<Self::Instance, DdiError>;
    fn detach(inst: Self::Instance, dev: DevInfo, ctx: &Sleepable)
        -> Result<(), (Self::Instance, DdiError)>;
    // resume, suspend and quiesce have defaults; instance_of_minor defaults to identity
}

impl ddi::CharDevice for RNullInstance {
    // open, close, read, write, ioctl; each has a sensible default
}
```

The macro emits `_init`, `_fini` and `_info` with `#[unsafe(no_mangle)]`,
a static `modlinkage`, a `dev_ops` whose slots point at typed glue, and a
`cb_ops` filled from the traits the instance type implements. Slots without
an implementation get a correctly typed stub that behaves like `nodev`.

Instance state lives in a soft-state slot (`ddi_soft_state_*(9F)`) indexed
by instance number. The framework writes the instance into that slot on
attach and reads it back out on detach. This is what makes `getinfo(9E)`
and the mapping from `dev_t` to instance in the `cb_ops` glue work without
any driver code. On a failed detach the framework hands the instance back
to the slot, so a driver that refuses to detach cannot lose its state.

### 5.3 Allocation context is a type, not a flag

`kmem_alloc(9F)` says that `KM_SLEEP` may not be used from interrupt
context. In C this is an integer flag and a comment. Here it is a value:

```rust
pub struct Sleepable(..);       // proof that blocking is permitted
pub struct Atomic(..);          // interrupt or otherwise non-blocking context

impl<T> KBox<T> {
    pub fn new(v: T, _: &Sleepable) -> KBox<T>;          // KM_SLEEP: cannot fail
    pub fn try_new(v: T) -> Result<KBox<T>, Errno>;      // KM_NOSLEEP: allowed anywhere
}
```

Entry points receive whichever token matches how the kernel calls them.
`attach`, `detach`, `open`, `close`, `read`, `write` and `ioctl` receive a
`&Sleepable`. Interrupt handlers and `ks_update` will receive an `&Atomic`.
Neither token is `Copy`, neither is `Send`, and neither can be constructed
by a driver, so a `Sleepable` cannot be stashed away and reused inside an
interrupt handler.

There is no `#[global_allocator]` by default. `alloc::Box::new` would have
to pick a flag blindly, and whichever flag it picked would be wrong
somewhere. A driver may opt in to a `KM_SLEEP` global allocator with the
`global-alloc-sleep` feature while prototyping. The feature name states the
cost.

`Mutex<T>::lock` takes no token, because `mutex_enter` never sleeps for
memory. `CondVar::wait` requires a `&Sleepable`.

### 5.4 Locks own their data

`kmutex_t` must be initialised with `mutex_init(9F)` and destroyed with
`mutex_destroy(9F)`. `Mutex<T>` does both, in its constructor and in its
`Drop`. There is no `const fn new`; the constructor calls into the kernel.

An unlocked adaptive mutex with no waiters is a single zero word, so moving
a `Mutex` between its construction and its final home (normally inside the
driver's instance, which the framework stores in a soft-state slot) is
sound. Moving it while locked is prevented by the borrow that `MutexGuard`
holds. The guard calls `mutex_exit` on drop.

`CondVar::wait` consumes the guard and returns it. That is how the
precondition "the mutex must be held" becomes something the compiler
checks:

```rust
let mut g = inst.state.lock();
while !g.ready {
    g = inst.cv.wait(g, ctx);
}
```

`MUTEX_DRIVER` without an interrupt cookie is the only kind the safe layer
exposes for now. The variant that blocks interrupts arrives with the `intr`
feature and will take its cookie from `ddi_intr_get_pri`, so a spin mutex
cannot be created without an interrupt handle to justify it.

### 5.5 Handles carry the kernel's lifetime rules

`DevInfo` has no public constructor. The framework creates it in the glue
for `attach`, `detach` and `getinfo` and passes it to the driver. It is
`Copy`, and it may be stored in the instance, because the instance is
destroyed at detach and therefore cannot outlive the node.

`MinorNode` is an RAII type. Creating one calls `ddi_create_minor_node`;
dropping it calls `ddi_remove_minor_node`. Its constructor takes
`MinorKind::Char` or `MinorKind::Block` and a `NodeType` such as
`NodeType::Pseudo` rather than the `S_IFCHR` constant and the
`"ddi_pseudo"` string.

`Uio<'a>` wraps a `uio_t` and offers `copy_in` and `copy_out` over
`uiomove`, plus `skip` and `discard_all` over `uioskip`. `resid` and
`offset` expose the documented fields.

`Cred<'a>` wraps a `cred_t` for the duration of one entry point and offers
`has_sys_devices_priv` over `drv_priv`, plus `uid` and `zone_id`.

Two more handle types are designed but not yet written. For registers,
`Regs<'a>` over `ddi_regs_map_setup` will expose `read_u32(off)` and
`write_u32(off, v)`, with the access attributes (byte order, ordering)
built once into a `ddi_device_acc_attr_t` and offsets checked against the
mapped length at construction rather than on every access. For DMA, the
chain `DmaHandle` to `DmaMem` to `DmaBinding<'mem>` will make it impossible
to obtain a cookie from unbound memory; unbinding is simply the drop of the
binding. For interrupts, `Interrupts::alloc(kind)` will return a set of
slots, adding a handler will consume a slot and yield an armed interrupt
whose drop disables and removes it, and the handler will receive an
`&Atomic`.

### 5.6 ioctl arguments are typed and model-aware

```rust
fn ioctl(&self, minor: Minor, cmd: IoctlCmd, arg: IoctlArg<'_>,
         cred: &Cred<'_>, ctx: &Sleepable) -> Result<i32, Errno>;

impl IoctlArg<'_> {
    pub fn copy_in<T: FromBytes>(&self) -> Result<T, Errno>;              // ddi_copyin, honours FKIOCTL
    pub fn copy_out<T: IntoBytes + Immutable>(&self, v: &T) -> Result<(), Errno>;
    pub fn model(&self) -> DataModel;                                     // ddi_model_convert_from
}
```

`IoctlCmd` is a newtype around the command number. Drivers define their
commands with `IoctlCmd::io`, `IoctlCmd::ior::<T>`, `IoctlCmd::iow::<T>`
and `IoctlCmd::iowr::<T>`, which are `const fn` versions of the `_IO*`
macros from `sys/ioccom.h`. The `FromBytes` bound on `copy_in` comes from
`zerocopy` and guarantees that every byte pattern is a valid `T`, so no
validation is skipped by reading user memory straight into the type.

### 5.7 Logging without allocation

`cmn_err(9F)` takes a variable argument list. The safe layer formats the
message with `core::fmt::Write` into a 256-byte buffer on the stack and
calls `cmn_err(level, "%s", buf)` through a fixed-arity call. The `kwarn!`,
`knote!` and `kcont!` macros are therefore usable from any context and
never allocate. Rust's `panic!` reaches `panic(9F)` the same way, and
`DevInfo::warn` and `DevInfo::note` use `dev_err(9F)` so that the message
is prefixed with the driver name and instance.

### 5.8 What deliberately stays unsafe

A register read at a wrong offset is a hardware bug, not a memory-safety
bug, so `Regs::read_u32` will be a safe function. On the other side,
`ddi_get_soft_state` with an arbitrary instance number, raw `mblk_t`
manipulation and everything in the private tier remain `unsafe fn` with
documented preconditions. The safe crate should never claim more than the
manual page guarantees.

## 6. Build and link model

The two steps look like this:

```
cargo build -p rnull --target targets/x86_64-illumos-kmod.json \
    -Zjson-target-spec -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem \
    --release                              # produces target/.../rnull.a
ld -ztype=kmod -N misc/… -z allextract -z defs \
    -M $GATE/usr/src/uts/common/mapfiles/ddi.mapfile [-M mac.mapfile] \
    rnull.a -o rnull
```

The `xtask` binary (later perhaps a `cargo kmod` subcommand) wraps them. It
reads `[package.metadata.kmod]` from the driver and, transitively, from
every `ddi-*-sys` crate in the dependency graph to produce the `-N` list
and the list of mapfiles. It runs the link on illumos, since the link needs
the illumos linker; the compile step can happen on any host that has the
target JSON. It is meant to emit the driver's `.conf` file from
`[package.metadata.kmod.conf]` and, optionally, to produce an IPS package
following the `driver` action conventions of `oi-userland`, so that
`add_drv` runs at install time.

One thing is lost for now. The gate merges CTF data into every module so
that `mdb ::print` can display driver structures. Rust emits DWARF, and
`ctfconvert` does not understand Rust DWARF. We accept this for
driver-private types initially. The framework's `#[repr(C)]` types are
already in the CTF of `genunix`. Rust support in `ctfconvert` is a
follow-up to track.

The `compiler-builtins-mem` feature makes the module carry its own
`memcpy`, `memset`, `memcmp`, `memmove` and `bcmp` as weak symbols. The
kernel exports `memcpy` and `memset` (both are in `ddi.mapfile`) but not
`memcmp` or `memmove`, so a self-contained module is the only way to pass
`-z defs`. Inside the module the local definitions simply shadow the
kernel's.

With `-C symbol-mangling-version=v0` the Rust functions are visible to
DTrace's `fbt` provider and to `mdb` under their mangled names. The `xtask`
is meant to run `nm` over the result and reject any undefined symbol that is
not in the allowed mapfiles, any reference to `_Unwind_Resume`, and any
object that uses SSE.

## 7. Repository layout

```
illumos-ddi/
  Cargo.toml                 workspace
  rust-toolchain.toml        stable for check/test; the nightly is pinned in xtask
  targets/x86_64-illumos-kmod.json
  crates/ddi-sys/            core raw bindings; features intr, dma, regs, fm, kstat
  crates/ddi/                core safe layer, including the module! macro
  crates/ddi-<subsys>-sys/   one per row in section 4, added as needed
  crates/ddi-<subsys>/
  examples/rnull/            pseudo char driver: read gives EOF, write discards, three ioctls
  examples/…                 later: ramdisk over blkdev, pchtemp over ksensor
  xtask/                     build, link, deps; layout-check and lint-man are planned
  docs/DESIGN.md             this file
  docs/ddi-inventory.md      header, manual page and mapfile inventory from the gate
```

## 8. Verification

### 8.1 Layout tests from the running kernel's CTF

Every `#[repr(C)]` structure in a `-sys` crate is to be listed, with its
fields, in a `layout.toml` next to the crate. `xtask layout-check`, run on
an illumos host, asks the kernel itself for the truth:

```
echo '::sizeof struct dev_ops; ::offsetof struct dev_ops devo_attach' | mdb -k
```

(or `ctfdump -c /kernel/amd64/genunix` when a live kernel is not wanted)
and generates `tests/layout_generated.rs` containing `size_of` and
`offset_of!` assertions. The generated file is checked in per gate release,
so `cargo test --target x86_64-unknown-illumos` on any host re-verifies the
layouts. This replaces trusting hand-typed structures without shipping
bindgen output. The `mdb` queries it depends on were exercised during M1.

### 8.2 Manual page lint

`xtask lint-man --gate $GATE` parses every `/// name(9X)` doc comment in the
`-sys` crates, checks that the page exists and that its interface level is
not obsolete, and records the Committed, Evolving or Private status from the
`ATTRIBUTES` table in the generated rustdoc.

### 8.3 Host-side tests

The pure logic in `ddi` (errno mapping, ioctl number encoding, log
formatting, minor-number schemes) is tested with `cargo test` on the host.
A planned `host-mock` feature implements `kmem_*`, `mutex_*`, `cv_*` and
`cmn_err` over libc so that driver state machines can be unit-tested
outside the kernel.

### 8.4 In-kernel tests

`rnull` is loaded on `oibldofgate` (root, global zone) with `add_drv` and
exercised with `dd`, a small ioctl test program and `rem_drv`. Later, tests
can register with `ktest(9F)` once that framework's interface has settled.

### 8.5 Test infrastructure

Loading a module requires root on a kernel that is allowed to panic. Three
tiers exist or are needed, and only two of them exist.

The manual tier is `oibldofgate`, a Hipster 2026.04 VM with root access and
only the stable Rust from the package repository. The archive is compiled
elsewhere, copied over, linked there and installed with `add_drv`. This is
how M1 was done; the logs are in the workspace under `logs/oibldofgate/`.

The CI tier is the Solstice CI `illumos` label, which boots the
`openindiana-rust-ci` image from `refraction-forger` per job using
`vm-manager`. Jobs run as an unprivileged user and cannot `add_drv`, but
they can download a nightly, build with `-Zbuild-std` and link with
`ld -ztype=kmod`. That is what `.forgejo/workflows/ci.yml` does. The
workflow is pushed and its first run is queued.

The agent sandbox tier does not exist yet. For an agent to run the M1 loop
on its own it needs a disposable illumos VM with root, the nightly
toolchain and the gate mapfiles baked in, serial console capture for
panics, and snapshot and rollback. The building blocks are there: the
Solstice images, `vm-manager` with its qemu and bhyve backends and NoCloud
metadata, and Archibald. Anima's agent runners already run as pods in the
`anima-runners` namespace of the Archibald k0s cluster, managed by Flux from
`clusters/archibald/*` in `Toasterson/talos`, next to the
`solstice-executor`. What is missing is a sandbox provider in that cluster
that boots the illumos image for a session (through KubeVirt, or as a
`vm-manager` job the way Solstice does it), injects the checkout and a
Forgejo token, exposes SSH to the runner pod as its workspace, captures the
serial console and reaps the VM afterwards. This is a decision for the
Homelab and Anima side. Until it is made, verification beyond M1 stays
manual on `oibldofgate`.

For the image itself, the `openindiana-rust-ci` recipe would need `rustup`
with the pinned nightly and `rust-src`, the `developer/debug/mdb` and
`system/dtrace` packages, a checkout of `usr/src/uts/common/mapfiles`, and
either `sudo` or a root login. Panic capture means routing the bhyve serial
console to a file on the host.

## 9. Milestones

M0, the skeleton, was completed on 20 September 2026. It comprises the
workspace, the target JSON, the core subset of `ddi-sys`, the `ddi` crate
with `module!`, `Driver`, `CharDevice`, `KBox`, `Mutex`, `CondVar`, logging,
`Uio` and `IoctlArg`, and the `rnull` example. `cargo check` and `cargo
clippy` pass on `x86_64-unknown-illumos` with stable Rust.

M1, the first load, was completed the same day. `rnull.a` was cross-built
on macOS with nightly-2026-04-16 and linked on `oibldofgate` with
`ld -ztype=kmod -z allextract`. The result carries the `DT_SUNW_KMOD` tag
and its undefined symbols are DDI functions only. After `add_drv rnull`, a
test program from userland confirmed the version ioctl, three 4 KiB writes
counted correctly, a read returning end of file, the stats and reset
ioctls, and `ENOTTY` for an unknown command. A 1 MiB `dd` ran at 2.5 GiB/s.
The `dev_err` notices appeared in `dmesg`, and `rem_drv` unloaded the module
cleanly. While loaded, DTrace listed 456 `fbt` probes on the module and
`mdb -k` disassembled `rnull\`_init`. Two things from M1 remain open: the
`layout-check` tool, whose `mdb` queries are proven, and the strict
`-z defs -M ddi.mapfile` link, which is blocked on the two mapfile gaps
described in section 3.3.

M2 gives the framework a hardware shape: the `regs`, `intr` and `dma`
features, and a port of `pchtemp` (ksensor, Evolving tier) as the second
example, chosen because a reference C implementation exists to compare
against.

M3 is block devices: `ddi-blkdev-sys` and `ddi-blkdev`, a ramdisk example,
and the missing `blkdev(9E)` manual pages written and sent upstream.

M4 is networking: `ddi-mac-sys` and `ddi-mac`, compared against `xde`, with
an invitation to Oxide to converge `illumos-sys-hdrs` onto the shared
crates.

M5 is gate integration: a `Makefile.rust` for `usr/src/uts` that invokes
cargo with a pinned toolchain and reuses the link rule from
`Makefile.targ`, together with an IPD for illumos.

## 10. Open questions

The crate names are undecided. `ddi` and `ddi-sys` are short and accurate
but generic on crates.io. `illumos-ddi` is unambiguous but becomes long
once per-subsystem crates are added.

The eventual home is undecided. The repository currently lives at
`code.aopc.cloud/illumos/illumos-ddi`. Moving it to `github.com/illumos/`
next to the userland `-sys` crates would be the natural upstream step once
the design has settled.

The toolchain pin is undecided. We could follow Oxide's nightly cadence or
pin once per illumos release.

`blkdev` has no manual pages. Binding it as a Committed interface requires
writing them first.

MAC is versioned through `MAC_VERSION_V1`. Whether to expose that as a const
generic or as a Cargo feature is open.

CTF for Rust types is unresolved. We can wait for `ctfconvert` to learn Rust
DWARF, or generate CTF ourselves from the `layout.toml` files.
