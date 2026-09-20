# illumos DDI for Rust: design

Status: draft, 2026-09-20. Working names throughout (`ddi-sys`, `ddi`,
`ddi-<subsystem>`); naming is an open question, see §10.

## 1. Goal

Make it possible to write illumos kernel drivers in Rust against the
*documented* Driver Developer Interface (DDI/DKI, section 9 of the manual),
in a way that could be adopted by the illumos project itself rather than
living as a per-driver pile of `extern "C"` declarations.

Two layers, mirroring the convention the illumos GitHub org already uses in
userland (`libdevinfo-sys` + `devinfo-rs`, `libscf-sys` + `smf-rs`):

| Layer | Crates | Contents | `unsafe`? |
| --- | --- | --- | --- |
| raw | `ddi-sys`, `ddi-<subsys>-sys` | `#[repr(C)]` structs, opaque handles, constants, `extern "C"` prototypes. One item per section 9 man page. No logic. | all |
| safe | `ddi`, `ddi-<subsys>` | Types whose shape encodes the rules the man pages state in prose: allocation context, lock ownership, DMA binding state, entry-point return conventions. | internally, never at the driver boundary |

Drivers depend on the safe layer only. A driver crate that contains
`unsafe` outside of hardware register access should be a code smell.

Non-goals for the first iterations: nexus drivers (`bus_ops`, `ndi_*`), the
STREAMS framework beyond what MAC needs, filesystem modules (`modlfs`),
SPARC.

## 2. Prior art and what it teaches

| Work | Year | Approach | Lesson |
| --- | --- | --- | --- |
| `rmustacc/rust-illumos-kmod-poc` (pchtemp port) | 2021 | Single `rmod.rs`, `rustc --emit=obj -C code-model=kernel -C no-redzone -C soft-float -C panic=abort`, structs and constants hand-typed inline; SDT probe via `__dtrace_probe_*` symbol | Proves the toolchain path is short. Everything else (`nodev` transmuted into typed slots, constants wrong by inspection) is what a real crate must fix. DTrace SDT from Rust works for free. |
| `apopiak/LoReFS` (lofs clone) | 2018 | xargo, `x86_64-sun-solaris`, C shim + Rust | Filesystems need VFS/vnode which is outside the DDI. Confirms the "escape hatch tier" in §4. |
| Oxide `opte/xde` + `crates/illumos-sys-hdrs` | 2022– | `no_std` staticlib, custom target JSON (`code-model: kernel`, `disable-redzone`, no SSE, soft-float, `frame-pointer: always`, `panic: abort`), `-Zbuild-std`, then `ld -ztype=kmod -Ndrv/dld -Nmisc/mac ... -z allextract xde.a`. Hand-written headers crate with a `kernel` feature; one 28k-line bindgen file (`ip.rs`) they regret. | This is the production-tested build recipe; copy it. Their header crate is *flat* (one `kernel.rs`) and mixes raw and safe (`impl kmutex_t { fn new }`), which is exactly what per-subsystem boundaries avoid. Their `GlobalAlloc` uses `KM_SLEEP` unconditionally and the panic handler allocates a `CString`, both unsound in interrupt context. `_Unwind_Resume` stub needed because `alloc` was not built with `panic=abort`. |
| Oxide `propolis` `bhyve-api`/`viona-api` | 2021– | userland `no_std` FFI to kernel ioctls | Same discipline (repr(C), versioned ABI) but userland; not a template for DDI. |
| Linux `rust/kernel` crate | 2022– | `bindgen` over kernel headers with the kernel's CFLAGS, `module!` macro, `Arc`/`Mutex`/`Pin` based safe layer, `ForeignOwnable`, typed error `Error`/`Result` | The safe-layer vocabulary (`module!`, driver traits, typed errors, pinned locks) is worth borrowing. bindgen-over-everything is not: illumos DDI is a *committed* ABI with man pages, so a curated hand-written surface is both feasible and more honest. |

## 3. Ground truth from the gate

Everything below was read from `illumos/illumos-gate` at commit `0fb1d5d8d3`
(local clone) and is what the design must satisfy.

### 3.1 How a kernel module is linked

`usr/src/uts/Makefile.targ:40`:

```
$(LD) -ztype=kmod $(LDFLAGS) -o $@ $(OBJECTS)
```

then CTF merge against `genunix` and post-processing. `-ztype=kmod` (see
`usr/src/cmd/sgs/libld/common/args.c`) implies `-r` **and** `-dy`, forbids an
explicit `-d`, and emits a `DT_SUNW_KMOD` dynamic tag. Do not pass `-r`
yourself. Dependencies on other modules are `-N drv/blkdev`, `-N misc/mac`
etc., which become `DT_NEEDED` entries that `krtld` loads recursively. The
old `_depends_on` string is a deprecated fallback and must not be used.

Optional but valuable: `-z defs -M usr/src/uts/common/mapfiles/ddi.mapfile`
(plus `mac.mapfile`, …) makes the link fail if the module references any
symbol outside the listed set. The gate's README for those mapfiles says
they are not a formal stability statement, but the `ddi` mapfile is the
closest thing to a machine-readable DDI symbol list, and it is what
`i40e`, `e1000g` and others link against.

### 3.2 Compile model for amd64 kernel code

Effective flags after the `cw` wrapper (`usr/src/Makefile.master`,
`usr/src/uts/Makefile.uts`, `usr/src/tools/cw/cw.c`):

```
-m64 -ffreestanding -mcmodel=kernel -mno-red-zone -mno-mmx -mno-sse -msave-args
-fno-asynchronous-unwind-tables -fno-shrink-wrap -fstack-protector-strong
-mindirect-branch=thunk-extern -mindirect-branch-register
-D_KERNEL -D_SYSCALL32 -D_SYSCALL32_IMPL -D_ELF64 -D_DDI_STRICT -Dsun -D__sun -D__SVR4
```

Consequences for Rust:

- Custom target (`targets/x86_64-illumos-kmod.json`): `code-model: kernel`,
  `relocation-model: static`, `disable-redzone: true`, features
  `-mmx,-sse,-sse2,…,+soft-float`, `panic-strategy: abort`,
  `frame-pointer: always`, `eh-frame-header: false`. This is Oxide's file
  with the `os` left as `illumos` so `cfg(target_os = "illumos")` holds.
- `core`/`alloc` must be rebuilt for that target: `-Zbuild-std=core,alloc
  -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec` (nightly). This is the only
  hard nightly dependency of the whole design; the crates themselves are
  written for stable Rust (no `extern_types`, no `alloc_error_handler`).
- `-D_DDI_STRICT` compiles `ddi_obsolete.h` out of every in-gate module.
  Nothing from that header is bound.
- Stack protector: the kernel exports `__stack_chk_fail`/`__stack_chk_guard`
  (both in `ddi.mapfile`). Rust can opt in with `-Zstack-protector=strong`
  later; off for now.
- `-msave-args` has no Rust equivalent; `mdb ::stack` argument display is
  lost for Rust frames. Frame pointers are kept so stack walking works.
- Kernel thread stacks are 5 pages (`DEFAULTSTKSZ`, `i86pc/sys/machparam.h`).
  No `format!` into `String`, no large arrays on the stack; the safe
  layer's logging formats into a fixed 256-byte buffer.
- Retpolines: Rust has `-Zretpoline`-class flags only on nightly; accept
  the gap and document it (indirect calls from Rust into `cb_ops` are rare).

### 3.3 What "public DDI" means, mechanically

Three signals, none sufficient alone (see the inventory in
`docs/ddi-inventory.md`):

1. `.SH INTERFACE LEVEL` in `usr/src/man/man9{e,f,s}`: either
   `Architecture independent level 1 (DDI/DKI).` or
   `illumos DDI specific (illumos DDI).`, optionally followed by
   "This interface is obsolete". Present on 92–99 % of pages.
2. `Interface Stability` row in `ATTRIBUTES` (Committed / Evolving /
   Obsolete / Private / Volatile). Present on a minority of pages.
3. Membership in `usr/src/uts/common/mapfiles/*.mapfile`.

The mapfile is *incomplete* relative to the man pages: `getminor(9F)`,
`getmajor(9F)`, `makedevice(9F)`, `ddi_prop_exists(9F)`,
`ddi_model_convert_from(9F)`, `rw_read_locked(9F)`, `crgetuid(9F)` and
`vpanic` are documented DDI but absent from `ddi.mapfile` (checked
2026-09-20). So the mapfile can gate a link (`-z defs`) but cannot define
the binding surface; `xtask lint-man` should report "documented but not in
mapfile" so the gaps can be sent upstream as mapfile additions.

Header names are not a signal: `ddi_impldefs.h` and `mac_impl.h` install
into `/usr/include/sys` next to `sunddi.h`. The `*_impl.h`, `*_impldefs.h`,
`esunddi.h` ("drivers calling these functions are not portable") naming is,
however, reliable for the negative case.

### 3.4 Fixed ABI facts a binding must hard-code (amd64)

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

These are the things a CTF-driven layout test (§8.1) must pin.

## 4. Crate boundaries

One `-sys` crate per documented interface family. The boundary test is:
"does this family have its own man 9 entry-point set or `9F` prefix, its
own header(s), and usually its own kernel module that a driver would name
with `-N`?" The crate then *owns* three things: the symbols it declares,
the `-N` dependency it implies, and the mapfile it can be checked against.

| Crate | Headers (`usr/src/uts/common/sys/`) | man 9 family | kmod dep (`-N`) | mapfile | Tier |
| --- | --- | --- | --- | --- | --- |
| `ddi-sys` | `types.h`, `errno.h`, `param.h`, `modctl.h`, `devops.h`, `conf.h`, `dditypes.h`, `sunddi.h` (core subset), `ddipropdefs.h` (enums only), `cred.h`, `uio.h`, `buf.h`, `open.h`, `file.h` (flags), `stat.h` (`S_IF*`), `cmn_err.h`, `kmem.h`, `mutex.h`, `condvar.h`, `rwlock.h`, `semaphore.h`, `ddi_periodic.h` (API only), `kstat.h`, `sysmacros.h`/`atomic.h` as needed | `Intro(9E)`, `dev_ops(9S)`, `cb_ops(9S)`, `modl*(9S)`, `ddi_*`, `kmem_*`, `mutex_*`, `cv_*`, `rw_*`, `cmn_err`, `uio*`, `bio*`, `kstat_*`, `ddi_prop_*`, `ddi_soft_state_*`, `ddi_taskq_*`, `ddi_periodic_*` | none (genunix/unix) | `ddi` | Committed |
| `ddi-sys` feature `intr` | `ddi_intr.h` | `ddi_intr_*(9F)` (14 pages) | none | `ddi` | Committed |
| `ddi-sys` feature `dma` | `ddidmareq.h`, `sunddi.h` DMA subset | `ddi_dma_*(9F)`, `ddi_dma_attr(9S)`, `ddi_dma_cookie(9S)` | none | `ddi` | Committed |
| `ddi-sys` feature `regs` | `sunddi.h` regs subset, `dditypes.h` | `ddi_regs_map_setup(9F)`, `ddi_get8(9F)`… | none | `ddi` | Committed |
| `ddi-sys` feature `fm` | `ddifm.h` | `ddi_fm_*(9F)`, `ddi_fm_error(9S)` | none | `ddi` | Committed |
| `ddi-ufm-sys` | `ddi_ufm.h` | `ddi_ufm(9E)`, `ddi_ufm_*(9F)` | none | – | Committed, versioned |
| `ddi-pci-sys` | `pci.h` (constants), `sunddi.h` `pci_config_*` | `pci_config_*(9F)` | none | `ddi` | Committed |
| `ddi-ldi-sys` | `sunldi.h` | `ldi_*(9F)` (24 pages) | none | – | Committed |
| `ddi-mac-sys` | `mac_provider.h`, `mac.h`, `mac_ether.h` | `mac(9E)`, `mc_*(9E)`, `mac_*(9F)`, `mac_register(9S)`, `mac_callbacks(9S)` | `misc/mac` | `mac` | Committed (versioned `MAC_VERSION_V1`) |
| `ddi-streams-sys` | `stream.h`, `strsun.h`, `stropts.h` | `queue(9S)`, `msgb(9S)`, `qinit(9S)`, `put(9E)`, `srv(9E)`, `q*(9F)`, `mi*(9F)`, `allocb(9F)`… | none | `ddi` | Committed |
| `ddi-blkdev-sys` | `blkdev.h` | `bd_*(9F)` (only header + comments today, no man pages) | `drv/blkdev` | – | Committed by convention, **undocumented**: first job is to write `blkdev(9E)` man pages upstream |
| `ddi-scsa-sys` | `scsi/scsi.h` and the public `scsi/scsi_*.h` (not `scsi/impl/`) | `scsi_*(9F)` (41), `tran_*(9E)` (14), `scsi_hba_tran(9S)`… | `misc/scsi` | – | Committed |
| `ddi-usba-sys` | `usb/usba.h`, `usb/usbai.h` | `usb_*(9F)` (29), `usb_*(9S)` | `misc/usba` | – | Committed |
| `ddi-usba-hcdi-sys` | `usb/usba/hcdi.h` | `usba_hcdi_*(9E)` (12), `usba_*(9F)` | `misc/usba` | – | Committed |
| `ddi-ksensor-sys` | `sensors.h` | `ksensor(9E)`, `ksensor_*(9F)` | `misc/ksensor` | `ksensor` | **Evolving** (header says so) |
| `ddi-cmlb-sys` | `cmlb.h` | none | `misc/cmlb` | – | Private-by-omission |
| `illumos-kernel-private-sys` | `virtio.h` (`io/virtio/`), `vnode.h`/`vfs.h`, `taskq.h` (raw), `id_space.h`, `zone.h`, `esunddi.h` items | none | varies | – | **Private**: pinned to a gate commit, only usable with `--features unstable-gate-<hash>` |

Rules that keep the boundaries meaningful:

- A `-sys` crate declares only what appears in its listed headers *and*
  either has a section 9 page or is in its mapfile. Anything else goes to
  the private tier, never into a Committed crate "just for now".
- Every declaration carries a doc comment of the form
  `/// kmem_alloc(9F) — Architecture independent level 1 (DDI/DKI).`
  A CI script (`xtask lint-man`) checks that the named page exists in
  `usr/src/man/man9*` and that its INTERFACE LEVEL is not "obsolete".
- `-sys` crates contain no `impl` blocks except `Default`/`Debug` on plain
  data. No allocation, no wrappers, no `unsafe fn` bodies. Oxide's
  `impl kmutex_t { fn new }` in a header crate is the anti-pattern.
- Function pointers in `9S` structs are `Option<unsafe extern "C" fn(..)>`
  with exact signatures. `nodev`/`nulldev`/`nochpoll` are provided by the
  safe crate as correctly-typed Rust stubs per slot rather than transmuted.
- Opaque kernel types are zero-sized `#[repr(C)]` structs with
  `PhantomData<(*mut u8, PhantomPinned)>` (stable pattern, no
  `extern_types`).
- Per-arch differences (`ddi_isa.h` lives under `uts/intel/sys/`, not
  `common/`) are `cfg(target_arch)` modules inside the same crate.
- Each `-sys` crate's `Cargo.toml` carries
  `[package.metadata.kmod] depends = ["misc/mac"]` so the link tool can
  assemble `-N` flags from the dependency graph. A driver never types `-N`.

## 5. The safe layer: types that express intent

The man pages state rules in prose. The safe crate turns each of them into
something the compiler checks. The list below is the contract for `ddi`
(core); subsystem crates follow the same principles.

### 5.1 Two result conventions, two error types

`dev_ops` entry points return `DDI_SUCCESS`/`DDI_FAILURE`; `cb_ops` entry
points return an `errno`. They are unrelated integer spaces and must not be
confused, so:

```rust
pub struct Errno(NonZeroI32);           // cb_ops, ioctl, uiomove, ddi_copyin
pub enum DdiError { Failure }           // dev_ops, ddi_* that return DDI_*
pub type Result<T, E = Errno> = core::result::Result<T, E>;
```

`ddi_prop_*` has a third space (`DDI_PROP_NOT_FOUND`, …); it gets `PropError`.

### 5.2 Module and driver definition

```rust
ddi::module! {
    type: RNull,                 // implements ddi::Driver
    name: "rnull",               // module name; also the minor node prefix
    linkinfo: "Rust null pseudo device",
}
pub struct RNull;
impl ddi::Driver for RNull {
    type Instance = RNullInstance;                     // per-dip state, Send + Sync
    fn attach(dev: &DevInfo, ctx: &Sleepable) -> Result<Self::Instance, DdiError>;
    fn detach(inst: Self::Instance, ctx: &Sleepable) -> Result<(), (Self::Instance, DdiError)>;
    // suspend/resume/quiesce have defaults that report "not supported"
}
impl ddi::CharDevice for RNullInstance { fn open / close / read / write / ioctl / chpoll }
```

The macro emits `_init`/`_fini`/`_info` with `#[unsafe(no_mangle)]`, a
`static modlinkage`, a `dev_ops` whose slots point at typed glue, and a
`cb_ops` filled from the traits the instance type implements (unimplemented
traits get the correct `nodev`-equivalent stub). Instance state is a
`Box<Instance>` stored with `ddi_set_driver_private`; `getinfo(9E)` is
implemented by the framework via a soft-state table indexed by instance.
Detach hands the instance *back* on failure so the driver cannot lose it.

### 5.3 Allocation context is a type, not a flag

`kmem_alloc(9F)`: "KM_SLEEP … may not be called from interrupt context".
Instead of an integer flag:

```rust
pub struct Sleepable(());       // proof that blocking is permitted
pub struct Atomic(());          // interrupt / lock-held context
pub trait Context { const KM: c_int; }
pub struct KBox<T>(…);
impl<T> KBox<T> {
    pub fn new(v: T, _: &Sleepable) -> KBox<T>;                 // KM_SLEEP: cannot fail
    pub fn try_new(v: T, _: &impl Context) -> Result<KBox<T>, Errno>; // KM_NOSLEEP
}
```

Entry points receive the context token appropriate to how the kernel calls
them: `attach`/`detach`/`open`/`close`/`read`/`write`/`ioctl` get
`&Sleepable`; interrupt handlers and `ks_update` get `&Atomic`. `Sleepable`
is not `Copy` and cannot be constructed by drivers, so it cannot leak into
an interrupt handler. There is **no** `#[global_allocator]` by default:
`alloc::Box::new` would have to pick a flag blindly. A driver may opt in
to a `KM_SLEEP` global allocator with `features = ["global-alloc-sleep"]`
for prototypes, and the feature name says what it costs.

`Mutex<T>::lock` takes no token (mutex_enter never sleeps for memory) but
`CondVar::wait` requires `&Sleepable`.

### 5.4 Locks own their data and have stable addresses

`kmutex_t` must be initialised by `mutex_init` and never moved while held
by the kernel. `Mutex<T>` therefore has no `const fn new`; it is created
by `Mutex::new(value, kind, ctx)` and lives inside the `KBox<Instance>`,
which is heap-pinned by construction. The guard exits on drop. `mutex_owned`
becomes `debug_assert!` inside `CondVar::wait`, and `CondVar::wait` consumes
and returns the guard so the "lock must be held" precondition is enforced:

```rust
let mut g = inst.state.lock();
while !g.ready { g = inst.cv.wait(g, ctx); }
```

`MUTEX_DRIVER` is the only kind the safe layer exposes for now; the
interrupt-block-cookie variant arrives with the `intr` feature and takes
the cookie type from `ddi_intr_get_pri`, so a spin mutex cannot be created
without an interrupt handle.

### 5.5 Handles are borrows with the kernel's lifetime rules

- `&DevInfo` is only reachable inside `attach`/`detach`/`getinfo` glue and
  through the instance's stored `DevInfoRef`; it has no public constructor.
- `MinorNode` is RAII: `ddi_create_minor_node` on creation,
  `ddi_remove_minor_node` on drop. Its constructor takes `MinorKind::Char |
  Block` and `NodeType::Pseudo | …` enums instead of `S_IFCHR` and the
  `"ddi_pseudo"` string.
- `Regs<'a>` (`ddi_regs_map_setup`) exposes `read_u32(off)`/`write_u32`;
  the access attribute (endianness, ordering) is a builder that produces a
  `ddi_device_acc_attr_t` once, and offsets are checked against the mapped
  length at construction, not per access.
- DMA (later): `DmaHandle` → `DmaMem` → `DmaBinding<'mem>` with cookies as
  an iterator; you cannot obtain a cookie from unbound memory, and
  `unbind` is the drop of the binding.
- Interrupts (later): `Interrupts::alloc(kind)` returns a set; adding a
  handler consumes an `IntrSlot` and yields an `ArmedIntr` whose drop
  disables and removes. The handler closure receives `&Atomic`.
- `Uio<'a>` wraps `uio_t` with `copy_in(&mut [u8])`/`copy_out(&[u8])` over
  `uiomove`; `resid()` and `offset()` are the documented fields.
- `Cred<'a>` with `has_priv_sys_devices()` over `drv_priv`.

### 5.6 ioctl arguments are typed and model-aware

```rust
fn ioctl(&self, cmd: IoctlCmd, arg: IoctlArg<'_>, cred: &Cred, ctx: &Sleepable) -> Result<i32, Errno>;
impl IoctlArg<'_> {
    pub fn copy_in<T: zerocopy::FromBytes>(&self) -> Result<T, Errno>;   // ddi_copyin, honours FKIOCTL
    pub fn copy_out<T: zerocopy::IntoBytes>(&self, v: &T) -> Result<(), Errno>;
    pub fn model(&self) -> DataModel;                                   // ddi_model_convert_from
}
```

`IoctlCmd` is a newtype; drivers define their commands with a
`const fn ioc(group: u8, n: u8) -> IoctlCmd` so the `_IO*` macros are typed.

### 5.7 Logging without allocation

`cmn_err(9F)` is varargs. The safe layer formats with `core::fmt::Write`
into a 256-byte stack buffer and calls `cmn_err(level, c"%s", buf)` via
a fixed-arity shim, so `kwarn!`/`knote!`/`kcont!` are usable from any
context and never allocate. `panic!` maps to `panic(9F)` the same way.
`dev_err(9F)` variants prefix the instance name.

### 5.8 What deliberately stays unsafe

Register access with a wrong offset is a hardware bug, not a memory-safety
bug, so `Regs::read_u32` is safe. Conversely, `ddi_get_soft_state` with an
arbitrary instance number, raw `mblk_t` surgery, and anything in the
private tier remain `unsafe fn` with documented preconditions. The safe
crate should never claim more than the man page guarantees.

## 6. Build and link model

```
cargo build -p rnull --target targets/x86_64-illumos-kmod.json \
    -Zjson-target-spec -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem \
    --release                              # → target/.../rnull.a
ld -ztype=kmod -N misc/… -z allextract -z defs \
    -M $GATE/usr/src/uts/common/mapfiles/ddi.mapfile [-M mac.mapfile] \
    rnull.a -o rnull
```

wrapped by an `xtask` (later `cargo kmod`) that:

1. reads `[package.metadata.kmod]` from the driver and, transitively, from
   every `ddi-*-sys` crate in its dependency graph to produce the `-N` list
   and the mapfile list;
2. runs the link on illumos (the link step needs illumos `ld`; the compile
   step can happen on any host with the target JSON);
3. emits the `.conf` file from `[package.metadata.kmod.conf]`
   (`name`, `parent`, `instance`, driver properties);
4. optionally produces an IPS package (`pkg/`), following the
   `oi-userland` `driver` action conventions, so `add_drv` runs on install.

CTF: the gate merges CTF for every module so `mdb ::print` works on driver
structs. Rust emits DWARF, and `ctfconvert` does not understand Rust DWARF.
Accept the loss for driver-private types initially; the framework's
`#[repr(C)]` types are already in `genunix` CTF. Track upstream
`ctfconvert` Rust support as a follow-up.

`compiler-builtins-mem` makes the module carry its own `memcpy`/`memset`/
`memcmp`/`memmove`/`bcmp`. The kernel exports `memcpy` and `memset` (both in
`ddi.mapfile`) but not `memcmp`/`memmove`, so a self-contained module is the
only way to pass `-z defs`; the local definitions simply shadow the kernel's
inside the module.

Symbols: with `-C symbol-mangling-version=v0` the Rust functions are
visible to DTrace `fbt` and `mdb` with their mangled names; the `xtask`
runs `nm` to reject any undefined symbol not in the allowed mapfiles and
to reject `_Unwind_Resume`, `memcpy` variants the kernel lacks, and any
SSE-using object.

## 7. Repository layout

```
illumos-ddi/
  Cargo.toml                 workspace
  rust-toolchain.toml        pinned nightly (for -Zbuild-std only)
  targets/x86_64-illumos-kmod.json
  crates/ddi-sys/            core raw bindings, features: intr dma regs fm
  crates/ddi/                core safe layer
  crates/ddi-<subsys>-sys/   one per row in §4, added as needed
  crates/ddi-<subsys>/
  crates/kmod-macros/        proc macro behind ddi::module!
  examples/rnull/            pseudo char driver: read=EOF, write=discard, ioctl=version
  examples/…                 later: ramdisk over blkdev, pchtemp over ksensor
  xtask/                     build, link, layout-check, lint-man
  docs/DESIGN.md             this file
  docs/ddi-inventory.md      header/man/mapfile inventory from the gate
```

## 8. Verification

### 8.1 Layout tests from the running kernel's CTF

Every `#[repr(C)]` struct in a `-sys` crate is listed in
`crates/<crate>/layout.toml` (`struct`, fields). `xtask layout-check`,
run on an illumos host (`oibldofgate`), asks the kernel itself:

```
echo '::sizeof struct dev_ops; ::offsetof struct dev_ops devo_attach' | mdb -k
```

(or `ctfdump -c /kernel/amd64/genunix` when no live kernel is wanted) and
generates `tests/layout_generated.rs` with `const _: () = assert!(size_of::<dev_ops>() == …)`
and `offset_of!` assertions. The generated file is checked in per gate
release so `cargo test --target x86_64-unknown-illumos` on any host
re-verifies it. This replaces trusting hand-typed structs, without
shipping bindgen output.

### 8.2 man page lint

`xtask lint-man --gate $GATE` parses every `/// name(9X)` doc comment in
`-sys` crates and checks the page exists and its `INTERFACE LEVEL` is not
obsolete; Committed/Evolving/Private from `ATTRIBUTES` is recorded in the
generated rustdoc.

### 8.3 Host-side tests

Pure logic in `ddi` (errno mapping, ioctl number encoding, log formatting,
minor-number schemes) is tested with `cargo test` on the host. A
`host-mock` feature implements `kmem_*`, `mutex_*`, `cv_*`, `cmn_err` over
libc so driver state machines can be unit-tested outside the kernel.

### 8.4 In-kernel tests

Load `rnull` on `oibldofgate` (root, global zone) with `add_drv`; exercise
with `dd`, an ioctl test binary, `modunload`. Later, register tests with
`ktest(9F)` once that framework's interface stabilises.

## 9. Milestones

1. **M0 skeleton** (this commit): workspace, target JSON, `ddi-sys` core
   subset, `ddi` with `module!`, `Driver`, `CharDevice`, `KBox`, `Mutex`,
   `CondVar`, logging, `Uio`, `IoctlArg`; `rnull` example;
   `cargo check --target x86_64-unknown-illumos` clean.
2. **M1 loads**: build with `-Zbuild-std` on the kmod target, link with
   `ld -ztype=kmod -z defs -M ddi.mapfile` on `oibldofgate`, `add_drv rnull`,
   read/write/ioctl from userland, `modunload`. Layout-check tool producing
   the generated assertions for `ddi-sys`.
3. **M2 hardware-shaped**: `regs`, `intr`, `dma` features; port
   `pchtemp` (ksensor, Evolving tier) as the second example since a
   reference C implementation exists.
4. **M3 block**: `ddi-blkdev-sys`/`ddi-blkdev`; a ramdisk example; write
   the missing `blkdev(9E)` man pages upstream.
5. **M4 network**: `ddi-mac-sys`/`ddi-mac`; compare against `xde` and
   invite Oxide to converge `illumos-sys-hdrs` onto it.
6. **M5 gate integration**: a `Makefile.rust` for `usr/src/uts` that
   invokes cargo with a pinned toolchain and reuses `Makefile.targ`'s link
   rule; IPD for illumos.

## 10. Open questions

- Naming: `ddi`/`ddi-sys` is short and accurate but generic on crates.io;
  `illumos-ddi` is unambiguous but long for per-subsystem crates.
- Where it lives: `github.com/illumos/` alongside the userland `-sys`
  crates, or `code.aopc.cloud/OpenIndiana/` first and upstream later.
- Toolchain pin: follow Oxide's nightly cadence or pin once per illumos
  release.
- `blkdev` has no man pages; binding it as Committed requires writing them.
- MAC is versioned (`MAC_VERSION_V1`); expose the version as a const
  generic or a feature.
- CTF for Rust types: wait for `ctfconvert` support or generate CTF from
  `layout.toml` ourselves.
