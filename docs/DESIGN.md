# illumos DDI for Rust: design

This document is a draft. It was written on 20 September 2026, and the first
two milestones described in section 9 were reached the same day. The third,
the complete binding surface and a safe layer for every family a leaf
driver uses, was reached on 26 September 2026. The crate names used
throughout (`ddi-sys`, `ddi`, `ddi-<subsystem>`) are working names. Naming
is one of the open questions in section 10.

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

Some things are deliberately out of scope. Nexus drivers with their
`bus_ops` and `ndi_*` interfaces are not covered, because the manual does
not document them as DDI. Filesystem modules (`modlfs`) are not covered,
because the VFS and vnode interfaces are not DDI either. SPARC is not
covered. STREAMS is bound completely in the raw layer, but the safe layer
has no STREAMS support yet.

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
`uioskip`. The larger self-test driver fails on 24, among them the
semaphore functions, most of the property lookups and all of LDI. Measured
across the whole core crate with `xtask lint-man --mapfile-gaps`, 497
functions that have a manual page and that the running kernel exports are
not in the mapfile. The mapfile is a curated subset for the drivers that
opted into it, not a description of the DDI. It can gate a link, but it
cannot define the binding surface, and a strict link of a Rust driver needs
those additions upstream first.

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

These are exactly the things the CTF-driven layout test in section 8.1
pins. They were first spot-checked against the running kernel on
`oibldofgate` (build `illumos-cda5b9a49f`) with `mdb -k`. `struct dev_ops`
is 0x58 bytes with `devo_attach` at offset 0x20, `struct cb_ops` is 0x88,
`struct uio` is 0x30 with `uio_resid` at 0x28, `kmutex_t` is 8,
`kcondvar_t` is 2 and `struct modlinkage` is 0x40. Since then every
structure in every `-sys` crate has been checked the same way, with the
results in section 8.1.

## 4. Crate boundaries

There is one `-sys` crate per documented interface family. The test for
drawing a boundary is a practical one. Does this family have its own set of
section 9 entry points or its own `9F` prefix, its own headers, and
usually its own kernel module that a driver would name with `-N`? If so,
it gets a crate. That crate then owns three things: the symbols it
declares, the `-N` dependency it implies, and the mapfile it can be
checked against.

| Crate | What it covers | man 9 family | kmod dep (`-N`) | mapfile | Tier |
| --- | --- | --- | --- | --- | --- |
| `ddi-sys` | everything that resolves from `genunix` or `unix`: module linkage, `dev_ops` and `cb_ops`, device nodes and minor nodes, soft state, properties, credentials and privileges, memory and caches, locks, time and callouts, task queues, `uio`, `buf` and the `bio` family, registers, PCI configuration space, interrupts, DMA, the x86 ISA DMA engine, fault management, kstats, UFM, `devmap`, LDI, name-value lists, STREAMS, events and sysevents, device IDs, power management, `ddi_umem`, firmware loading, strings, atomics, `stdbit`, `bitext`, lists, AVL trees, ID spaces, `vmem`, resource maps and the character set conversions | every non-obsolete `9F` page of those families, with `9S` structures and `9E` callback types | none | `ddi` | Committed, with Volatile and Evolving families marked as such per item |
| `ddi-mac-sys` | `mac_provider.h`, `mac.h`, `mac_ether.h` | `mac(9E)`, `mc_*(9E)`, `mr_*`, `mri_*`, `mgi_*`, `mi_*(9E)`, `mac_*(9F)`, `mac_register(9S)` and friends | `misc/mac` | `mac` | Committed, versioned (`MAC_VERSION_V1`) |
| `ddi-scsa-sys` | `scsi/scsi.h` and the public `scsi/scsi_*.h`, `scsi/generic/*` | `scsi_*(9F)`, `sas_phymap_*(9F)`, `tran_*(9E)`, `scsi_*(9S)` | `misc/scsi` | none | Committed; the iport, target map and phymap families are Volatile |
| `ddi-usba-sys` | `usb/usba.h`, `usb/usbai.h`, `usb/usba/hcdi.h`, `usb/usba/hubdi.h` | `usb_*(9F)`, `usb_*(9S)`, `usba_hcdi_*(9E)`, `usba_*(9F)`, `usba_*(9S)` | `misc/usba` | none | Committed for clients, Volatile for host controllers |
| `ddi-neti-sys` | `neti.h`, `hook.h`, `hook_event.h` | `net_*(9F)`, `hook_*(9F)`, `net_*(9S)`, `hook_*(9S)` | `misc/neti`, `misc/hook` | none | Committed |
| `ddi-ksensor-sys` | `sensors.h` | `ksensor(9E)`, `ksensor_ops(9E)`, `ksensor_*(9F)` | none (in `genunix`) | `ksensor` | Volatile, according to the pages and the header |
| `ddi-pcmcia-sys` | `pccard.h` and the client parts of `pcmcia/` | `csx_*(9F)`, `csx_event_handler(9E)`, `tuple(9S)` | none (in `genunix`) | none | Committed, for legacy hardware |
| `ddi-gld-sys` | `gld.h` | `gld(9E)`, `gld(9F)`, `gld_mac_info(9S)`, `gld_stats(9S)` | `misc/gld` | none | Committed, superseded by MAC |
| `ddi-ktest-sys` | `ktest.h` | `ktest_*(9F)` | `drv/ktest` | none | Volatile |
| `ddi-blkdev-sys` | `blkdev.h` | none; the header and its comments are all there is | `drv/blkdev` | none | Planned. Committed by convention but undocumented, so the first job is to write `blkdev(9E)` upstream |
| `ddi-cmlb-sys` | `cmlb.h` | none | `misc/cmlb` | none | Planned, private by omission |
| `illumos-kernel-private-sys` | Since 29 September 2026: `file.h`/`fio.c` (`file_t`, `falloc`, `unfalloc`, `setf`, `getf`, `releasef`, `f_setfd_or`), `vnode.h` (the public prefix of `vnode_t`, `fop_open`, `fop_close`, `vn_copypath`, `vn_rele`), `snode.h` (`makespecvp`, `specfind`). Planned: `virtio.h` (`io/virtio/`), `taskq.h` (raw), `zone.h`, items from `esunddi.h` | none; items cite the gate source line | `fs/specfs` | none | Private, pinned to gate `cda5b9a49f`, empty unless built with `--features unstable-gate-cda5b9a49f` |

The private tier has its first committed consumer. On 27 September 2026
the owner decided to build a Rust DRM core and a Rust virtio-gpu driver on
these crates (decision record: `notes/drm-rust-core-assessment-20260927.md`
in the illumos workspace). The core lives in
[illumos-drm](https://code.aopc.cloud/illumos/illumos-drm). Its first two
milestones, R0 (device registration, per-file state and the core ioctls)
and R1 (GEM objects and dumb buffers), were done on 28 September 2026 with
the public tier alone, after clone opens and the `devmap(9E)` entry point
were added to the `ddi` glue. The later milestones need the private tier:
`falloc`/`setf` from `os/fio.c` for the anonymous file descriptors of PRIME,
syncobj export and `sync_file` (R2), the virtio framework for the
virtio-gpu driver (R4), `gfx_private` for the console framebuffer
hand-over, and `hat_*` for write-combining page pools, which only a later
TTM-like layer needs. So `illumos-kernel-private-sys` moves ahead of the
other planned crates.

The crate exists since 29 September 2026 with what R2 needed: the file
descriptor functions of `uts/common/os/fio.c`, a few vnode operations and
the two specfs functions that give a driver's own minor number a file
(illumos-drm `docs/anon-fd.md` records that design). Its rules:

- The crate is empty unless the feature naming the gate commit is enabled
  (`unstable-gate-cda5b9a49f`), so a consumer states which kernel it was
  built for. Moving to another gate commit means re-reading every cited
  line, running `xtask kernel-check` against that kernel, and renaming the
  feature.
- The first doc line of every item cites the gate file and line that
  define it, `` `name`. Private, `uts/common/os/fio.c:1362`. ``, instead of
  a manual page. `xtask lint-man --gate` checks each citation against the
  checkout: the file must exist and the cited line must name the item.
  The manual page checks skip the crate (`PRIVATE_CRATES` in
  `xtask/src/lint_man.rs`).
- `xtask kernel-check` covers it like every other `-sys` crate: the
  layout assertions (`file_t` in full, the public prefix of `vnode_t`
  field by field, `caller_context_t`) and every symbol, which must
  resolve from `genunix`, `unix` or `fs/specfs`.
- Safe wrappers live in the `ddi` crate under `ddi::private`, behind the
  `ddi` feature of the same name. A Committed crate never declares a
  private symbol.

The earlier plan had separate crates for UFM, PCI, LDI and STREAMS and
Cargo features for registers, interrupts, DMA and fault management inside
`ddi-sys`. Both were dropped. All of those families resolve from `genunix`
or `unix`, so they imply no `-N` dependency, and declarations cost nothing
to compile. A split would only have added names. The boundary that
matters is the kernel module a symbol comes from, and `xtask kernel-check`
verifies that boundary for every symbol (section 8.1).

A handful of rules keep these boundaries meaningful.

A `-sys` crate declares only what appears in its listed headers and that
either has a section 9 page or is present in its mapfile. Anything else
belongs in the private tier. It never goes into a Committed crate "just for
now", because "for now" has a way of becoming permanent. There are a few
documented exceptions, each explained at the item. `uioskip` and `uiocopy`
have neither a page nor a mapfile entry, but the safe `Uio` needs the first
and many gate drivers use both. `struct pollhead`, `list_t` and
`avl_tree_t` take their layouts from `_impl` headers, because `chpoll(9E)`
and the list and AVL pages require drivers to embed them. `esballoca` and
`desballoca` are documented on `esballoc(9F)` but prototyped in
`sys/strsubr.h`.

Every declaration carries a doc comment that names its manual page and
interface level, in the form ``/// `kmem_alloc(9F)`. Architecture independent
level 1 (DDI/DKI).`` Pages without an INTERFACE LEVEL section are quoted
as ``Interface stability: Committed.`` from their ATTRIBUTES table, and the
few pages with neither say ``No interface level given.`` `xtask lint-man`
checks that the named page exists, that the quoted text is exactly what the
page says, and that the page does not mark the item obsolete or private.

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
the same crate. The ISA DMA engine from `uts/intel/sys/dma_engine.h` is the
first such module.

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

Families with richer codes get a small enum of their own rather than a
number. `PropError` covers the `DDI_PROP_*` codes, `DmaError` the
`DDI_DMA_*` codes, `IntrError` the `DDI_E*` codes of the interrupt
functions and `RegError` the bounds and alignment checks of register
access. Where "not found" is an ordinary answer rather than a failure, as
for a property or a name-value pair, it becomes `Ok(None)`.

### 5.2 Module and driver definition

A driver implements the `Driver` trait, implements `CharDevice` on its
instance type if it is a character device, and invokes the `module!` macro:

```rust
ddi::module! {
    driver: RNull,
    ops: [char],                 // [char, devmap] to allow mmap(2); [] for no cb_ops
}

pub struct RNull;

impl ddi::Driver for RNull {
    type Instance = RNullInstance;   // per-instance state, Send + Sync
    const LINKINFO: &'static CStr = c"Rust null pseudo device v1";

    fn attach(node: OwnNode, ctx: &Sleepable) -> Result<Self::Instance, DdiError>;
    fn detach(inst: Self::Instance, dev: DevInfo, ctx: &Sleepable)
        -> Result<(), (Self::Instance, DdiError)>;
    // resume, suspend and quiesce have defaults; instance_of_minor defaults to identity
}

impl ddi::CharDevice for RNullInstance {
    // open, clone_open, close, read, write, ioctl, devmap; each has a sensible default
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

A driver that needs state per open file implements `clone_open` instead of
`open`. It returns a fresh minor number, and the glue stores
`makedevice(getmajor(dev), minor)` through the `devp` argument of
`open(9E)`, so the new file has its own device number and gets its own
`close(9E)` when its last reference goes away. The returned minor must map
to the same instance through `instance_of_minor`, and should stay within
18 bits (`L_MAXMIN32`) if 32-bit processes may open the device. The
default `clone_open` calls `open` and keeps the device number. The DRM core
in illumos-drm uses this for its per-file authentication and master state.

A driver whose memory user processes map with `mmap(2)` declares
`ops: [char, devmap]` and implements `devmap`. That form installs the
glue as `devmap(9E)`, glue that runs `CharDevice::segmap_check` and then
`ddi_devmap_segmap(9F)` as `segmap(9E)`, and sets `D_DEVMAP`, so every
`mmap(2)` goes through `devmap_setup(9F)`. `segmap_check` sees the
request's offset, length, protections and flags (a `MapRequest`), which
`devmap(9E)` does not, and is the place to refuse a mapping: when a
`devmap_map(9E)` callback refuses, `devmap_setup` frees the handles but
leaves the new segment in the address space until the process exits,
which keeps the device mapped and its file open (found by illumos-drm's
R2 run on 29 September 2026). The method
receives the minor number, the mapping handle as a `DevmapHandle` that
lives only for the call, the user offset, the remaining length and the
caller's data model; it describes the memory with `devmap_umem_setup(9F)`
or `devmap_devmem_setup(9F)` and returns the validated length. The
`devmap_callback_ctl(9S)` callbacks stay with the driver, which passes
them to the setup function, because their private data and lifetime rules
are the driver's own. Two facts from `seg_dev.c` shape the contract and
are documented on the method: `devmap_setup` reports any failure of the
entry point as `ENXIO`, and a handle can be freed after a successful
`devmap(9E)` without any `devmap_map(9E)` or `devmap_unmap(9E)` call, so a
reference taken in `devmap` needs another way back. The default fails
with `ENXIO`, and `[char]` keeps the `nodev` stubs, so existing drivers
are unchanged. The DRM core in illumos-drm uses this for GEM object
mappings.

A driver whose devices `poll(2)` waits on declares `chpoll` in the list
(`[char, chpoll]` or `[char, devmap, chpoll]`) and implements
`CharDevice::chpoll`, which returns the ready events and registers a
`PollHead` in the `PollRegistration` it receives; the glue hands the head
to the kernel exactly when `chpoll(9E)` requires it and fails with
`EINVAL` if a head is required and none was registered. `PollHead` keeps
its `pollhead` in a box of its own, so it never moves, wakes pollers with
`pollwakeup(9F)` from any context, and on drop wakes them with `POLLERR`
and calls `pollhead_clean(9F)`, so a head cannot be freed while pollers
are registered on it. Without `chpoll` the slot is `nochpoll`, and specfs
treats the device as always ready.

### 5.3 Allocation context is a type, not a flag

`kmem_alloc(9F)` says that `KM_SLEEP` may not be used from interrupt
context. In C this is an integer flag and a comment. Here it is a value:

```rust
pub struct Sleepable(..);       // proof that blocking is permitted
pub struct Atomic(..);          // interrupt or otherwise non-blocking context

impl<T> KBox<T> {
    pub fn new(v: T, _: &Sleepable) -> KBox<T>;          // KM_SLEEP: cannot fail
    pub fn try_new(v: T) -> Result<KBox<T>, Errno>;      // KM_NOSLEEP: allowed anywhere
    pub fn try_new_or_return(v: T) -> Result<KBox<T>, T>; // KM_NOSLEEP, value handed back
}
```

Entry points receive whichever token matches how the kernel calls them.
`attach`, `detach`, `open`, `close`, `read`, `write` and `ioctl` receive a
`&Sleepable`, and so do task queue functions and periodic callbacks, which
run on kernel threads. Hardware and soft interrupt handlers and `timeout(9F)`
callbacks receive an `&Atomic`.
Neither token is `Copy`, neither is `Send`, and neither can be constructed
by a driver, so a `Sleepable` cannot be stashed away and reused inside an
interrupt handler.

There is no `#[global_allocator]` by default. `alloc::Box::new` would have
to pick a flag blindly, and whichever flag it picked would be wrong
somewhere. A driver may opt in to a `KM_SLEEP` global allocator with the
`global-alloc-sleep` feature while prototyping. The feature name states the
cost.

`Mutex<T>::lock` takes no token, because `mutex_enter` never sleeps for
memory. `CondVar::wait` requires a `&Sleepable`. The same idea runs through
every module: whatever may block takes the token, and a non-blocking
variant exists wherever the manual allows one, such as
`TaskQueue::try_dispatch`, `DmaHandle::try_new` and `NvList::try_new`.

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

`Mutex::new` creates a `MUTEX_DRIVER` mutex without an interrupt cookie.
A mutex that an interrupt handler takes is created with
`Mutex::with_priority(value, pri)`, and the only way to obtain an
`IntrPriority` is from an allocated interrupt or soft interrupt. A lock at
interrupt priority therefore cannot exist without an interrupt to justify
it. `Interrupts::alloc` takes a closure that builds the handler state once
the priority is known, which is exactly when the locks inside it can be
created.

`RwLock<T>` and `Semaphore` follow the same pattern over `rw_*(9F)` and
`sema_*(9F)`. `RwLockReadGuard::try_upgrade` hands the read guard back on
failure, and `RwLockWriteGuard::downgrade` turns a write guard into a read
guard. The timed waits of `CondVar` return a `WaitResult` that says
whether the wait timed out, was interrupted by a signal or was woken. None
of the guards is `Send`, because illumos requires the thread that took a
lock to release it.

### 5.5 Handles carry the kernel's lifetime rules

`DevInfo` has no public constructor. The framework creates it in the glue
for `attach`, `detach` and `getinfo` and passes it to the driver. It is
`Copy`, and it may be stored in the instance, because the instance is
destroyed at detach and therefore cannot outlive the node. A `DevInfo` is
for queries: properties, names, the instance number and logging.

Resources that the kernel ties to the requesting device need the driver's
own node, and the manual pages say so: `ddi_dma_alloc_handle(9F)` takes
"the device's dev_info structure" and adds that "a DMA handle must only be
used by the device for which it was allocated". Those constructors take an
`OwnNode` instead: `DmaHandle::new` and `try_new`, `Interrupts::alloc` and
`try_alloc`, `RegMap::map`, `RegBlock::map` and `PciConfig::setup`. So do
the resources the kernel files under the node and removes when the node's
driver detaches, or records without a hold: `OwnNode::create_minor_node`,
the property updates of `OwnNode::props` (section 5.8) and
`LdiIdent::from_dip` (section 5.10). Only
the glue creates an `OwnNode`, in `attach(9E)` with `DDI_ATTACH`, from the
node the kernel passed, and hands it to `Driver::attach` by value. The
driver keeps it in its instance if it allocates such resources later. It
is neither `Copy` nor `Clone`, so one instance cannot lend its node to
another, and it dereferences to its `DevInfo` for queries. A node from
anywhere else needs the `unsafe` `OwnNode::from_raw`. Section 5.12 gives
the reason and the runtime guard behind it.

`MinorNode` is an RAII type. Creating one with
`OwnNode::create_minor_node` calls `ddi_create_minor_node`; dropping it
calls `ddi_remove_minor_node`. Its constructor takes
`MinorKind::Char` or `MinorKind::Block` and a `NodeType` such as
`NodeType::Pseudo` rather than the `S_IFCHR` constant and the
`"ddi_pseudo"` string.

`Uio<'a>` wraps a `uio_t` and offers `copy_in` and `copy_out` over
`uiomove`, plus `skip` and `discard_all` over `uioskip`. `resid` and
`offset` expose the documented fields.

`Cred<'a>` wraps a `cred_t` for the duration of one entry point and offers
`has_sys_devices_priv` over `drv_priv`, plus `uid` and `zone_id`.

`RegMap` owns a mapping from `ddi_regs_map_setup(9F)` with the access
attributes built once by `AccessAttr`. Access by byte offset is checked
against the mapped length and the alignment and returns `RegError` rather
than panicking, since a panic in the kernel is a system panic. For
registers known at compile time, `RegBlock<LEN>` and `Reg<T, OFF>` check
the bounds and alignment at compile time, so their accesses cannot fail.
`PciConfig` owns a configuration space handle from `pci_config_setup(9F)`
and walks the capability list with a guard against loops.

DMA is a chain of owners. `DmaHandle` owns the handle, `DmaBuffer` owns the
memory and the handle, and `DmaBinding<'a>` borrows the buffer mutably for
as long as the device may use it. A cookie can only be obtained from a
binding, and the CPU can only see the memory through the binding right
after `sync_for_cpu`. Unbinding is the drop of the binding, and the order
unbind, free memory, free handle holds even if a binding is leaked.

`Interrupts<T>` allocates a set of vectors, refuses high-level interrupts
whose handlers could not take ordinary locks, builds the handler state from
the priority, registers one trampoline per vector and enables the set. Its
drop disables, removes each handler, which waits for running handlers, frees
the vectors and only then drops the state. `SoftInterrupt<T>` does the
same for one soft interrupt, whose `trigger` may be called from any
context.

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

The `IoctlArg` the glue passes also records the device number the ioctl
was issued on. Only the glue can set it; `IoctlArg::new` makes an
argument without one. `ddi::private::fd` needs it to give a new file
descriptor the path of the caller's own file (section 5.12).

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

### 5.8 Properties

`DevInfo::props()` returns a `Props` view with a fixed scope: which minor
device the properties belong to and whether a lookup may continue to the
parent nodes. The default matches any minor and stays on the node, which is
what a driver reading its `.conf` file wants. Lookups return
`Result<Option<T>, PropError>`, so a missing property is not an error.
Values the kernel allocates come back as `PropBuf<T>`, `PropString` and
`PropStrings`, which free themselves with `ddi_prop_free(9F)`. Updates copy
the value, so the driver's data need not outlive the call. Every property
function in the manual is restricted to user and kernel context, so every
method takes the sleepable token.

Updates and removals change the driver property list of the node, which
belongs to the driver bound to it: the kernel deletes it when that driver
detaches (`detach_node` in `os/devcfg.c`), and the framework reads
settings such as `pm-components` from it (`os/sunpm.c`). So the access is
a type parameter of the view. `DevInfo::props()` gives a
`Props<'_, ReadOnly>` that only looks up; `OwnNode::props()` gives a
`Props<'_, ReadWrite>` that also has the `set_*`, `remove`, `remove_all`
and `undefine` methods. A view from an `OwnNode` that the guard of
section 5.12 refuses fails every update with `PropError::NotOwnNode`
before the kernel is called. Some gate drivers write properties of other
nodes: nexus drivers on their children (`io/lofi.c`, `io/fcoe/fcoe_fc.c`,
`pciex/npe_misc.c`, `mpt_sas`) and a few leaf drivers on their parent
(`sil3xxx.c`, `dnet.c`). That is out of scope for now; such a driver has to
use the raw `ddi::sys` functions until a capability for child nodes
exists.

### 5.9 Callbacks own their state

Many DDI functions take a function pointer and a `void *` argument and call
back later: `timeout(9F)`, `ddi_periodic_add(9F)`, `ddi_taskq_dispatch(9F)`,
`ddi_intr_add_handler(9F)` and `ddi_intr_add_softint(9F)`. The safe layer
handles all of them the same way. The Rust closure or state goes into a
`KBox`, which has a stable address. The kernel receives that address and a
trampoline monomorphised for the closure type. The Rust value that
represents the registration owns the box and frees it only after the
kernel guarantees that the callback can no longer run: after `untimeout`,
after `ddi_periodic_delete`, after `ddi_taskq_destroy` has drained the
queue, or after `ddi_intr_remove_handler` has waited for running handlers.

The waiting has a consequence that the types cannot express, and the
documentation states it prominently. Dropping a `Timeout`, a `Periodic`,
an `Interrupts` or a `SoftInterrupt` waits for a running callback, so it
must not happen while holding a lock that the callback takes, and a
`Periodic` must never be dropped from its own callback. A `Timeout` may be
dropped from its own callback, because `untimeout` detects that case.

`TaskQueue::suspend` returns a guard rather than setting a flag, because
`ddi_taskq_suspend` holds a reader-writer lock until the matching resume,
which must come from the same thread. The guard is neither `Send` nor
`Sync`, and it borrows the queue, so the queue cannot be destroyed while it
is suspended.

### 5.10 Statistics, name-value lists and layered devices

`NamedKstat` and `TypedKstat<S>` publish a named kstat. The kstat owns a
mutex whose address goes into `ks_lock`, and the data can only be reached
by locking it, so a snapshot by `kstat(8)` never sees a half-written
value. The `named_kstat!` macro declares a `#[repr(C)]` structure of
`KstatNamed<T>` fields and checks at compile time that each name fits.
`IoKstat` covers the I/O kstat type with the queue accounting functions.

`NvList<P>` owns a name-value list. The allocation flag given to
`nvlist_alloc(9F)` governs every later allocation of the list, so it is a
type parameter: a list created with the sleepable token is `KmSleep` and
its adds require the token, while a `KmNoSleep` list may be filled from
any context and its adds may fail with `ENOMEM`. Lookups borrow from the
list, and the methods that change it take `&mut self`, which is how the
rule that a pair must not be used after it was removed becomes a borrow
error. A typed lookup matches on name and type together, as the kernel
does, so a pair of another type reads as absent.

`LdiIdent::from_dip` takes the driver's `OwnNode`. The identifier records
the node without a hold, and the layered usage walker behind `prtconf`
takes a hold on it later (`ldi_usage_walker_helper` in
`os/driver_lyr.c`), so an identifier for another driver's node could
outlive that node. The guard refuses foreign nodes with `ENXIO`.

`LdiHandle` opens another device through the layered driver interface. It
takes a reference on the credential it was opened with and closes with the
same credential, as `ldi_close(9F)` asks, whether it is closed explicitly
or dropped. `Cred::kernel()` gives the kernel's own credential for work a
driver does on its own behalf. The manual forbids opening layered devices
from `attach(9E)`, `detach(9E)` and `power(9E)`; the token cannot express
that, so the documentation does.

### 5.11 What deliberately stays unsafe

A register read at a wrong offset is a hardware bug, not a memory-safety
bug, so the register accessors are safe functions. On the other side,
`ddi_get_soft_state` with an arbitrary instance number, raw `mblk_t`
manipulation, the typed LDI ioctls (the target decides how many bytes it
copies under `FKIOCTL`, so the caller must promise that the command matches
the type) and everything in the private tier remain `unsafe fn` with
documented preconditions. The safe crate should never claim more than the
manual page guarantees. `OwnNode::from_raw` is `unsafe` for a reason of
this kind: nothing but the attach glue can know that a node is the
driver's own.

### 5.12 Panics become types

A kernel panic that is traced to misuse of a DDI interface, where the
driver did something its manual page forbids, is a gap in this crate as
much as a bug in the driver, because the safe layer exists so that such
rules cannot be broken without `unsafe`. Every such panic therefore leads
to four changes to the safe layer, made together:

1. A type or capability that makes the misuse unrepresentable in safe
   code.
2. A runtime guard wherever the type can be bypassed with `unsafe`, which
   turns the misuse into an error before the kernel is called.
3. Regression tests: a `compile_fail` doctest for the type rule, with the
   expected error code and a compiling twin that differs only in the rule,
   and an in-kernel `rselftest` check for the guard. The check must not be
   able to panic even if the guard were missing.
4. A row in the table below that names the panic and its evidence.

Stable rustdoc does not check the error code of a `compile_fail` doctest;
the pinned nightly does, so the doctests are also run with it
(`cargo +nightly-2026-04-16 test -p ddi --no-default-features --doc
--target x86_64-unknown-illumos`), locally and as a step of the
`check-hosted` CI job.

Some rules resist types. The instance type is `'static`, so the compiler
cannot tie an `OwnNode` to the interval between attach and detach; it can
only make the node unique and impossible to construct without `unsafe`.
Leaking it past detach (a forgotten `Timeout` that captured it, a
`KBox::into_raw`) remains possible in safe Rust, as leaking always is, and
no guard can see it, because the node of a detached instance may be freed.

The capability also covers interfaces that did not panic but create
something the kernel ties to the node and its driver. Each decision was
made against the gate source of illumos `cda5b9a49f`:

| Interface | Decision | Reason in the gate | Guard |
| --- | --- | --- | --- |
| `ddi_create_minor_node(9F)`: `OwnNode::create_minor_node` | `&OwnNode` | The minor's device number takes the major of the driver bound to the node, `detach_node` (`os/devcfg.c`) removes the node's minor nodes when that driver detaches, and on a DEBUG kernel an unbound node trips `ASSERT(major != DDI_MAJOR_T_NONE)` (`os/sunddi.c`). | `DdiError` |
| `ddi_prop_update_*(9F)`, `ddi_prop_remove(9F)`, `ddi_prop_remove_all(9F)`, `ddi_prop_undefine(9F)`: `OwnNode::props` | `Props<'_, ReadWrite>` from `&OwnNode`; lookups stay on `DevInfo` | They change the node's driver property list (`ddi_prop_add`, `ddi_prop_remove_common`), which `detach_node` deletes when the node's driver detaches and which other code reads (`pm-components` in `os/sunpm.c`); `remove_all` on a foreign node deletes another driver's properties. Writes to a child or parent node, which gate drivers do (`ddi_prop_update_int` on a child in `io/lofi.c`, `io/fcoe/fcoe_fc.c`, `pciex/npe_misc.c` and `mpt_sas`; on the parent in `dktp/controller/ata/sil3xxx.c` and `io/dnet/dnet.c`), are out of scope: until a capability for child nodes exists they are possible only through raw `ddi::sys` calls. | `PropError::NotOwnNode` |
| `ldi_ident_from_dip(9F)`: `LdiIdent::from_dip` | `&OwnNode` | `ident_alloc` (`os/driver_lyr.c`) stores the node without a hold and `ldi_usage_walker_helper` later calls `e_ddi_hold_devi` on it. | `ENXIO` |
| `devmap_devmem_setup(9F)`, `devmap_devmem_remap(9F)` | Not wrapped; `DevmapHandle` documents that the node must be the driver's own, and a future wrapper takes `&OwnNode` | They call `ddi_map(dip)`, which asks the parent nexus, as `RegMap::map` does (`vm/seg_dev.c`). | For the wrapper |
| `devmap_umem_setup(9F)`, `devmap_umem_remap(9F)` | Not wrapped; the node passed should be the own node, but nothing is enforced | They never read the node (`vm/seg_dev.c`). | None needed |
| `ddi_prop_lookup*(9F)`, `ddi_prop_get_int*(9F)`, `ddi_prop_exists(9F)`, `ddi_driver_name(9F)`, `ddi_get_instance(9F)`, `ddi_report_dev(9F)`, `dev_err(9F)` | Stay on `DevInfo` | Queries and logging read the node and create nothing. | None |

Panics prevented:

| Date | Symptom | Root cause | Type rule | Guard | Tests | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-09-28 | `BAD TRAP: type=e (#pf Page fault)`, NULL pointer dereference at `pc=0` in `ddi_dma_addr_bind_handle`, at the first bind (illumos `cda5b9a49f`) | A pseudo driver allocated its DMA handles against `ddi_root_node(9F)`, not its own node. Rootnex accepts the root as its own requester, and the bind then calls the root node's `devi_bus_dma_bindfunc`, which is never filled because the root has no parent. | `DmaHandle::new` and `try_new` take `&OwnNode`, which only the attach glue creates and which is neither `Copy` nor `Clone`; a foreign node needs `unsafe` `OwnNode::from_raw`. `Interrupts::alloc`, `RegMap::map`, `RegBlock::map` and `PciConfig::setup`, which dereference the root node's missing parent, follow the same rule. | Before `ddi_dma_alloc_handle`, `DmaError::NotOwnNode` (`ENXIO`) for a node without a parent or bound to another driver than the one attached; the other constructors refuse the same nodes. | `compile_fail` doctests on `OwnNode` (a `DevInfo` for DMA, E0308; copy, E0507; clone, E0277; `from_raw` without `unsafe`, E0133); `rselftest` checks "dma root node refused by guard", "dma other driver's node refused by guard", "dma own pseudo node is BadAttr" | Workspace note `notes/illumos-drm-dma-panic-20260928.md`; illumos-drm pull request 3 |
| 2026-09-29 | `BAD TRAP: type=e (#pf Page fault)`, NULL pointer dereference at `addr=0x10` in `segdev_create+0xe4`, on the first `mmap(2)` of a dma-buf descriptor (illumos `cda5b9a49f`) | `ddi::private::fd::NewFd::open` made the descriptor's vnode with `makespecvp`, which gives it no path (`v_path` is `vn_vpath_empty`). `segdev_create` looks the mapped device's vnode up with `specfind`, which only returns vnodes whose path is visible in the caller's zone, and dereferences the NULL it got (`VTOCVP(sdp->vp)`). The same happens for any devmap driver when a descriptor is passed into another zone and mapped there. | `IoctlArg` records the device number of the ioctl, and only the glue can set it (`IoctlArg::from_entry` is crate-private); `NewFd::open` finds the caller's own file with `specfind` for that number and copies its path to the new vnode (`vn_copypath`). | `NewFd::open` refuses (`EINVAL`) an argument made with `IoctlArg::new` and a caller whose file has no visible vnode; with the private tier, the `devmap(9E)` glue refuses (`ENXIO`) a mapping of a device `specfind` cannot find, before the framework dereferences it. | `compile_fail` doctest on `IoctlArg` (`from_entry` outside the crate, E0624, with a compiling twin for `new`); `rselftest` built with `private-gate`, ioctl `RSELFTEST_IOC_ANONFD`: "a forged argument is refused" and the descriptor it makes is the driver's own minor, close-on-exec, read-only; illumos-drm's R2 test maps dma-bufs | Crash dump analysed with `mdb` on the test guest: the mapped vnode had `v_path` = `vn_vpath_empty`; illumos-drm pull request for R2 (WI-454), whose body links the dump and the logs |
| 2026-09-29 (review, no panic yet) | A use after free waiting to happen: `PollRegistration::register` accepted any `&PollHead` from safe code, and the poll framework links the poller into the head only after `chpoll(9E)` has returned, so a head dropped in between (a per-object head freed by another thread) would be written after free. `PollHead`'s drop also called `pollhead_clean(9F)`, which is for user and kernel context only, wherever it happened. | `CharDevice::chpoll` borrows the instance as `&'s self` and receives a `PollRegistration<'s>` whose safe `register` takes only a `&'s PollHead`, one the instance owns; other heads need the `unsafe` `register_unchecked` with its contract (dropped only after the polled file's `close(9E)`). `PollHead::release` takes the `Sleepable` token. | For `register_unchecked` none is possible: the window is inside the poll framework. An implicit drop of a `PollHead` in interrupt context leaks the head with a warning instead of cleaning it there (detected with the private tier's `servicing_interrupt`; without it the drop cleans unconditionally), counted by `ddi::poll::leaked_heads`. | `compile_fail` doctest on `ddi::poll` (a local head registered, E0597, with a compiling twin for an owned head); `rselftest` (`private-gate`): a head released with the token is not leaked, a head dropped in a soft interrupt is | Review of illumos-drm R2 (WI-454), round 1 |

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
  crates/ddi-sys/            core raw bindings, one module per family, no features
  crates/ddi-mac-sys/        GLDv3 MAC provider interface (misc/mac)
  crates/ddi-scsa-sys/       SCSI target and HBA interfaces (misc/scsi)
  crates/ddi-usba-sys/       USB client and host controller interfaces (misc/usba)
  crates/ddi-neti-sys/       network information and packet hooks (misc/neti, misc/hook)
  crates/ddi-ksensor-sys/    kernel sensors (genunix)
  crates/ddi-pcmcia-sys/     PC Card services (genunix)
  crates/ddi-gld-sys/        GLDv2 (misc/gld)
  crates/ddi-ktest-sys/      kernel test framework (drv/ktest)
  crates/illumos-kernel-private-sys/  private interfaces pinned to one gate commit (feature-gated)
  crates/ddi/                core safe layer, including the module! macro; ddi::private behind the same feature
  examples/rnull/            pseudo char driver: read gives EOF, write discards, three ioctls
  examples/rselftest/        pseudo driver that runs a self-test of the safe layer in the kernel
  xtask/                     build, link, deps, lint-man, kernel-check
  docs/DESIGN.md             this file
  docs/ddi-inventory.md      header, manual page and mapfile inventory from the gate
```

## 8. Verification

### 8.1 Layouts and symbols against the running kernel

`xtask kernel-check` compares the `-sys` crates with a running kernel and
needs nothing from the gate source. It collects every `size_of` and
`offset_of!` assertion and every function and static declared in an
`extern` block, and writes an `mdb(1)` script that asks the kernel's CTF
data for each size and offset and asks `::whatis` where each symbol lives.
The kernel host runs the script and the tool reads the output back:

```
cargo xtask kernel-check --emit | ssh root@host mdb -k | cargo xtask kernel-check --verify -
```

A symbol passes only if the kernel exports it and it resolves from `genunix`,
`unix` or a module the crate declares in its `[package.metadata.kmod]`
table. That turns the crate boundary of section 4 into something the tool
checks. Fields that store C bit-fields as whole bytes, and anonymous unions
that CTF cannot name, are reported separately and are covered by the size
of the containing structure.

The run on `oibldofgate` (illumos `cda5b9a49f`, OpenIndiana Hipster
2026.04) on 26 September 2026 gave these results.

| Crate | Sizes | Offsets | Symbols | Note |
| --- | --- | --- | --- | --- |
| `ddi-sys` | 53 of 60 | 185 of 185 | 983 of 984 | 7 anonymous unions; `ddi_ncpus_expected` postdates this kernel |
| `ddi-mac-sys` | 22 of 26 | 52 of 52 | 30 of 30 | 4 anonymous enumerations |
| `ddi-scsa-sys` | 16 of 20 | 87 of 103 | 80 of 80 | 4 anonymous members; 10 bytes hold C bit-fields |
| `ddi-usba-sys` | 26 of 29 | 126 of 134 | 65 of 65 | 3 descriptor types absent from `usba`'s CTF |
| `ddi-neti-sys` | 7 of 7 | 30 of 30 | 34 of 34 | |
| `ddi-ksensor-sys` | 3 of 3 | 7 of 7 | 6 of 6 | |
| `ddi-pcmcia-sys` | 68 of 84 | 175 of 201 | 83 of 83 | nested and anonymous structures without a CTF name |
| `ddi-gld-sys` | 7 of 7 | 41 of 41 | 14 of 14 | `misc/gld` loaded for the check |
| `ddi-ktest-sys` | not checked | not checked | 16 of 16 | `drv/ktest` is not installed on the image; its symbols were read from the packaged module |
| `illumos-kernel-private-sys` | 2 of 2 | 21 of 21 | 12 of 12 | checked on 29 September 2026 on a disposable guest with the same kernel (before `servicing_interrupt` was added) |

No size, no offset and no symbol disagreed with the kernel. Every count
below the total is a type that CTF cannot name; those types are covered by
the size of the structure that contains them. The PC Card, USB and GLDv2
assertions were also compiled as C `_Static_assert`s against the gate
headers with clang, which covers the types CTF does not name. The full
output is in the workspace under
`logs/oibldofgate/20260926-kernel-check.log`.

It found three real problems, all fixed. `min(9F)` and `max(9F)` are
defined in `subr.c` but not exported by the running kernel, so they were
removed; a module calling them would not load. `ddi_ncpus_expected(9F)` was
added to the gate in August 2026, after this kernel was built, and is kept
with a note. A binding for the interrupt block cookie existed twice with
different types and was merged.

### 8.2 Manual page lint and coverage

`xtask lint-man --gate $GATE` parses the first line of every doc comment
in the `-sys` crates that cites a section 9 page. It resolves the name
through the NAME and SYNOPSIS sections and the `Manlink` files, checks that
the quoted interface level is exactly what the page says, and rejects
anything the page marks obsolete or a "private DDI function". `--fix`
rewrites the first lines to the canonical text. `--coverage` compares every
prototype in section 9F with the declarations, and `--mapfile-gaps` lists
the documented functions that `ddi.mapfile` does not contain.

| Measure | Count |
| --- | --- |
| 9F prototypes on pages that are not obsolete or private | 1183 |
| bound in the `-sys` crates | 1183 |
| pages skipped as obsolete or "private DDI function" | 27 |
| C macros without a symbol, left to the safe layer | 47 |
| documented, but not exported by the running kernel | 9 |
| documented functions in `ddi-sys` absent from `ddi.mapfile` | 497 |

With `-sys` declarations counted as a whole, the crates declare 1304
functions. The difference to 1183 is functions that the mapfiles list
without a manual page and the few documented exceptions of section 4.

The lint found nine documented functions the kernel does not export.
`atomic_inc_ptr`, `atomic_dec_ptr` and their `_nv` forms,
`id_allocff_specific_nosleep`, `net_netidtozonid` and
`mac_prop_info_set_default_uint16` appear in the manual but in no header
and no kernel symbol table. `min` and `max` are declared and defined but
not exported. The bindings also record, at the item concerned, every place
where a page and its header disagree about a prototype, a structure member
or a constant; in each case the header wins. Both lists are candidates for
upstream manual fixes.

### 8.3 Host-side tests

The pure logic in `ddi` (errno mapping, ioctl number encoding, log
formatting, minor-number schemes) is tested with `cargo test` on the host.
A planned `host-mock` feature implements `kmem_*`, `mutex_*`, `cv_*` and
`cmn_err` over libc so that driver state machines can be unit-tested
outside the kernel.

### 8.4 In-kernel tests

`rnull` is loaded on `oibldofgate` (root, global zone) with `add_drv` and
exercised with `dd`, a small ioctl test program and `rem_drv`.

Built with its `private-gate` feature, `rselftest` also answers
`RSELFTEST_IOC_ANONFD`, eight checks of the private tier from the ioctl
path: an `IoctlArg` that does not come from the glue cannot make a
descriptor (the guard of section 5.12), a real one makes a close-on-exec,
read-only descriptor of the driver's own minor that `FileRef` recognises,
another device's descriptor is not recognised, a closed one is `EBADF`,
a `PollHead` released with the token is cleaned, and one dropped in a
soft interrupt is leaked rather than cleaned there. On 29 September 2026
the first six (the `PollHead` checks came later) and the 42 attach and 7
LDI checks passed on a disposable guest with illumos `cda5b9a49f`.
`rselftest_test` reports "not built" when the feature is off.

`rselftest` is a pseudo driver that tests the safe layer from inside the
kernel. When it attaches it runs 42 checks over properties from its `.conf`
file and published by itself, mutexes, reader-writer locks with upgrade and
downgrade, semaphores, condition variable timeouts, ticks, `delay`, busy
waiting, `lbolt`, one-shot timeouts and their cancellation, task queues
with blocking and non-blocking dispatch, name-value lists including
packing in both encodings, the own-node rule of section 5.12 for DMA
handles, and its guard for minor nodes, property updates and LDI
identifiers. It logs each result with `dev_err(9F)`, counts
them in a kstat and reports them through an ioctl. A second ioctl runs 7
layered-device checks, which the manual forbids in `attach`: it opens its
own node through LDI and reads a byte pattern through a kernel-space `uio`,
and writes to `/dev/null` with the kernel credential. A periodic timer runs
while the driver is attached.

On 26 September 2026 all 42 checks passed on `oibldofgate`. The periodic
timer at 10 ms ran 100 times per second, `prtconf -v` showed the six
published properties, `kstat -m rselftest` showed the counters, and
`rem_drv` unloaded the module and removed the kstat and the node. The
first run found two wrong expectations in the tests, not in the library. A
typed name-value lookup of the wrong type reads as absent, which the
module documentation now states, and `/dev/zero` cannot serve a kernel
caller at all, because its read routine uses `uzero` regardless of the
segment, which is why the LDI test reads from the driver itself. The logs
are in the workspace under `logs/oibldofgate/20260926-rselftest-*`.

The three DMA checks only allocate and never bind. The driver's own
pseudo node reaches the kernel and gets `DmaError::BadAttr`, because the
pseudo nexus refuses DMA handles. An `OwnNode` made with the `unsafe`
`OwnNode::from_raw` for `ddi_root_node(9F)`, and one for the pseudo nexus
node, which has a parent but another driver's major number, must both get
`DmaError::NotOwnNode` from the guard. A handle allocated against
expectation is freed at once and reported as a failure; freeing a
root-node handle works, only binding it panics.

On 28 September 2026 all 45 checks (38 at attach and 7 LDI checks) passed
on a disposable guest on Hephaistos with the same kernel as `oibldofgate`
(illumos `cda5b9a49f`). As a negative control, a build with the guard
patched out reported the two guard checks as failed and did not panic.
The logs are in the workspace under
`logs/hephaistos/illumos-drm/20260928-ddi-own-node-*`.

The same guard also protects minor nodes, property updates and LDI
identifiers. Three checks cover it, each with an `OwnNode` for the root
node: `create_minor_node` fails, `set_int` and `remove` fail with
`PropError::NotOwnNode`, and `LdiIdent::from_dip` fails with `ENXIO`. None
of them can panic without the guard: the kernel would create a minor node
(of a node type no `devfsadm` rule knows), a property or an identifier on
the root node, and the check would remove it again. On 28 September 2026
all 49 checks (42 at attach and 7 LDI checks) passed on a fresh guest on
Hephaistos (illumos `cda5b9a49f`), with no guard minor node under
`/devices` and a clean `rem_drv`. The log is
`logs/hephaistos/illumos-drm/20260928-loose-ends-ddi-rselftest.log` in the
workspace.

Apart from that DMA check, register access, PCI configuration space,
interrupts and DMA need a device to bind to and are not covered by
`rselftest`. They were checked against the gate source and the compiler
only. Tests can also register with
`ktest(9F)`, whose bindings now exist in `ddi-ktest-sys`.

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
`ld -ztype=kmod`. That is what `.forgejo/workflows/ci.yml` does for
`rnull` and `rselftest`. The workflow failed on every run until 26
September 2026, because it cloned the repository itself from a server URL
the runner does not set, and because the hosted image's shared rustup is
read-only; since those two fixes both jobs pass.

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
`mdb -k` disassembled `rnull\`_init`. Two things from M1 remained open. The
layout check became `xtask kernel-check` in M2. The strict
`-z defs -M ddi.mapfile` link is still blocked, now on the much larger set
of mapfile gaps described in section 3.3.

M2, the complete binding surface and the safe layer, was completed on 26
September 2026. Every function on a non-obsolete section 9F page is bound,
across `ddi-sys` and eight subsystem crates, apart from the C macros that
have no symbol and the documented functions the kernel does not export.
Every binding was checked against the running kernel's type data and symbol
table with `xtask kernel-check`, and every doc comment against its page with
`xtask lint-man`. The safe layer gained properties, reader-writer locks,
semaphores, timed condition variable waits, locks at interrupt priority,
timers, task queues, kstats, register and PCI configuration access,
interrupts and soft interrupts, DMA, name-value lists and LDI.
`rselftest` exercised everything that does not need hardware in the running
kernel, and all 42 checks passed.

M3 is hardware: a port of `pchtemp` over `ddi-ksensor-sys`, chosen because
a reference C implementation exists to compare against, which exercises
PCI configuration space and register access on a real device, followed by
a driver that uses interrupts and DMA.

M4 is block devices: `ddi-blkdev-sys` and `ddi-blkdev`, a ramdisk example,
and the missing `blkdev(9E)` manual pages written and sent upstream.

M5 is networking: a safe `ddi-mac` over `ddi-mac-sys`, compared against
`xde`, with an invitation to Oxide to converge `illumos-sys-hdrs` onto the
shared crates. Safe layers for SCSA, USBA and STREAMS follow the same
pattern when a driver needs them.

M6 is gate integration: a `Makefile.rust` for `usr/src/uts` that invokes
cargo with a pinned toolchain and reuses the link rule from
`Makefile.targ`, together with an IPD for illumos, and the mapfile and
manual page fixes that section 8.2 collected.

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
DWARF, or generate CTF ourselves from the layout assertions that
`xtask kernel-check` already collects.

The documentation of the kernel side of the safe layer relies on rules the
types cannot express: which drops wait for a callback, and which entry
points may not open layered devices. Whether a lint or a runtime assertion
over `servicing_interrupt` should back those up is open.
