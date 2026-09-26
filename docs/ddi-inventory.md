# illumos DDI/DKI surface inventory

This inventory was compiled on 20 September 2026 from `illumos/illumos-gate`
at commit `0fb1d5d8d3`, using the local clone in the workspace. All paths
are relative to the root of the gate. It is the evidence behind the crate
table in section 4 of `DESIGN.md`, and it is meant to be re-run against a
newer gate when the crate boundaries are revisited.

## Stability model

Deciding whether a kernel interface is public takes three independent
signals. None of them is sufficient on its own, so all three are recorded
here.

The first signal is the `.SH INTERFACE LEVEL` section in the manual pages
under `usr/src/man/man9{e,f,s}`. It contains one of two phrases. The older
SVR4 core is marked "Architecture independent level 1 (DDI/DKI)." and
accounts for roughly 90 pages in man9f. The Sun and illumos extensions are
marked "illumos DDI specific (illumos DDI)." and account for roughly 315
pages. Obsolete interfaces are marked as such in the same section. Coverage
is good: 515 of 561 pages in man9f, 86 of 87 in man9e, 77 of 80 in man9s,
and none of the 7 pages in man9p.

The second signal is the `Interface Stability` row in the `ATTRIBUTES`
table. It is present on far fewer pages. In man9f it appears on 111 pages,
of which 91 say Committed, 15 Obsolete, 3 Evolving, 2 Private and 1
Volatile. In man9e it appears on 4 pages, all Committed. In man9s it appears
on 9 pages, 8 Committed and 1 Obsolete.

The third signal is membership in one of the linker mapfiles under
`usr/src/uts/common/mapfiles/`. There are ten: `ddi`, `mac`, `kernel`,
`i2c`, `gpio`, `ksensor`, `random`, `fnvlist`, `eedev` and `tlimod`. The
README in that directory says explicitly that the mapfiles do not declare
what is or is not a public interface. Even so, the `ddi` mapfile is the
only machine-readable list of DDI symbols the gate has, and drivers such as
`i40e` and `e1000g` link against it with `-z defs`.

Header installation is not a signal. `usr/src/uts/common/sys/Makefile`
installs `ddi_impldefs.h` and `mac_impl.h` into `/usr/include/sys` right
next to `sunddi.h` and `mac_provider.h`. Header naming does work in the
negative direction. Anything called `*_impl.h`, `*_impldefs.h` or
`*_private.h` is private, and `esunddi.h` says about itself that "drivers
calling these functions are not portable".

`ddi_obsolete.h` deserves a separate note. Its contents are wrapped in
`#ifndef _DDI_STRICT`, and the kernel build defines `-D_DDI_STRICT` for
every module (`usr/src/uts/Makefile.uts:141-142`). The obsolete interfaces
are therefore compiled out of the entire gate, and nothing from that header
should be bound.

## Core headers

The following headers under `usr/src/uts/common/sys/` make up the core DDI
that a leaf driver touches. The table records what each one provides and
whether it is public.

| Header | Family | Public? |
| --- | --- | --- |
| `ddi.h` | SVR4 DDI/DKI umbrella (types/map/buf/uio/stream) | yes |
| `sunddi.h` (65K) | all `ddi_*`/`ndi_*` prototypes: soft state, props, minor nodes, regs, DMA, copyin/out, `ddi_report_dev` | yes, Committed |
| `dditypes.h` | opaque handles `ddi_dma_handle_t`, `ddi_acc_handle_t`, `dev_info_t`, `ddi_device_acc_attr_t` | yes |
| `ddi_intr.h` | `ddi_intr_*` (14 man9f) | yes, Committed |
| `ddi_impldefs.h` (43K), `ddi_implfuncs.h`, `esunddi.h` | `struct dev_info` internals, `i_ddi_*` | private |
| `devops.h` | `dev_ops`, `bus_ops`, `cb_ops`, `power_req`, nexus ctlops | yes (`dev_ops(9S)`, `cb_ops(9S)` Committed) |
| `conf.h` | `fmodsw`, `nodev`/`nulldev`/`nochpoll`, `D_MP`/`D_64BIT` | yes |
| `modctl.h` | `modlinkage`, `modldrv`, `modlmisc`, `modlstrmod`, `modlfs` and eleven more linkage kinds, `mod_ops`, `mod_install/remove/info` | yes, Committed |
| `cred.h` | `cred_t`, `crget*`, `drv_priv` (`cred_impl.h` private) | yes |
| `uio.h` | `uio`, `iovec`, `uiomove`, `ureadc/uwritec` | yes, Committed |
| `buf.h` | `buf`, `B_*`, `bio*`, `bp_mapin/out` | yes, level 1 |
| `open.h`, `file.h`, `stat.h`, `errno.h`, `types.h`, `param.h` | otyp, `F*` flags, `S_IF*`, errno, base types, `PAGESIZE`/`DEV_BSIZE` | yes (`file_t` private) |
| `cmn_err.h` | `cmn_err`, `vcmn_err`, `dev_err`, `CE_*` | yes, Committed |
| `kmem.h` | `kmem_alloc/zalloc/free`, `KM_*`; `kmem_cache_*` | yes, Committed |
| `ksynch.h`, which forwards to `mutex.h`, `condvar.h`, `rwlock.h`, `semaphore.h` | `kmutex_t`, `kcondvar_t`, `krwlock_t`, `ksema_t` | yes, level 1 |
| `ddidmareq.h` | `ddi_dma_attr_t`, `ddi_dma_cookie_t`, `DDI_DMA_*` | yes |
| `ddimapreq.h`, `nexusdefs.h`, `sunndi.h`, `avintr.h` | nexus-side | nexus authors only |
| `ddipropdefs.h` | `ddi_prop_op_t`, `DDI_PROP_*` flags (the structures are private by the header's own comment) | enums yes |
| `ddidevmap.h` | `devmap(9E)` framework | yes |
| `ddifm.h` / `ddifm_impl.h` / `ndifm.h` | FMA DDI | yes / private / nexus |
| `ddi_periodic.h` | `ddi_periodic_add/delete` (the header also exposes implementation details) | API yes |
| `ddi_ufm.h` / `ddi_ufm_impl.h` | UFM framework, versioned | yes / private |
| `ddi_hp.h` / `ddi_hp_impl.h` | hotplug | yes / private |
| `ddi_obsolete.h` | obsolete DDI | exclude |
| `kstat.h` | `kstat_create/install/delete`, `kstat_named_t`, `kstat_io_t` | yes, Committed |
| `sysmacros.h`, `atomic.h`, `taskq.h`, `debug.h`, `sdt.h`, `list.h`, `id_space.h` | helpers | mixed |
| `usr/src/uts/intel/sys/ddi_isa.h` | ISA-specific access plumbing | per architecture |

Two of these deserve a comment. `sunddi.h` at 65 kilobytes and
`ddi_impldefs.h` at 43 kilobytes looked like a reason to split the core
crate into Cargo features. That plan was dropped, because every one of
these families resolves from `genunix` or `unix` and declarations cost
nothing to compile; the core crate has one module per family instead. And
`ddi_isa.h` does not exist under `common/` at all. It lives in
`usr/src/uts/intel/sys/` and `usr/src/uts/sparc/sys/`, which is why the
design calls for `cfg(target_arch)` modules.

## Subsystem frameworks

Beyond the core, each kernel framework a driver might plug into has its own
headers, its own manual pages and usually its own module to depend on. The
table lists them with the private headers to stay away from.

| Subsystem | Public headers | Private headers | man 9 | kmod / mapfile |
| --- | --- | --- | --- | --- |
| GLDv3 / MAC | `mac_provider.h` (24K, versioned `MAC_VERSION_V1`), `mac.h`, `mac_ether.h`, `mac_client.h` (consumer side) | `mac_impl.h`, `mac_client_impl.h`, `mac_soft_ring.h`, `mac_flow_impl.h` | `mac(9E)`, 17 `mc_*`/`mr_*`/`mi_*`/`mgi_*`(9E), `mac_*(9F)` (14), `mac_register(9S)`, `mac_callbacks(9S)` | `misc/mac`, `mac.mapfile` |
| GLDv2 | `gld.h` | | `gld(9E)` | legacy, do not bind |
| STREAMS | `stream.h` (29K), `strsun.h`, `stropts.h` | `strsubr.h`, `strft.h` | `queue/qinit/msgb/datab/streamtab(9S)`, `put/srv(9E)`, `q*(9F)`, `mi*(9F)` | genunix |
| DLPI / DLD / DLS | `dlpi.h`, `dld.h`, `dld_ioc.h`, `dls.h` | `dld_impl.h`, `dls_impl.h` | | `drv/dld`, `misc/dls` |
| SCSA | `scsi/scsi.h`, `scsi/scsi_{address,pkt,device,ctl,resource,types,params,names,watch,fm}.h` | `scsi/impl/*` (except the de-facto public `uscsi.h`) | `scsi_*(9F)` 41, `tran_*(9E)` 14, `scsi_hba_tran/scsi_pkt/scsi_device(9S)` | `misc/scsi` |
| blkdev | `blkdev.h` (`bd_ops_t`, `bd_handle_t`, `bd_xfer_t`, `bd_alloc_handle`, `bd_attach_handle`, `bd_xfer_done`, `bd_state_change`) | | none | `drv/blkdev` |
| USBA client | `usb/usba.h`, `usb/usbai.h` | `usba/usba_impl.h`, `usba_private.h`, `usbai_private.h` | `usb_*(9F)` 29, `usb_*(9S)` about 20 | `misc/usba` |
| USBA HCDI | `usb/usba/hcdi.h`, `hubdi.h` | `hcdi_impl.h` | `usba_hcdi_*(9E)` 12, `usba_*(9F)` 9 | `misc/usba` |
| SD card | `sdcard/sda.h`, `sda_ioctl.h` | `sda_impl.h` | | `misc/sda` |
| PCI / PCIe | `pci.h` (51K), `pcie.h` (44K), `pci_cap.h` | `pci_impl.h`, `pcie_impl.h` | `pci_config_*(9F)` 5 | genunix |
| LDI | `sunldi.h` | `sunldi_impl.h` | `ldi_*(9F)` 24 | genunix |
| FMA | `ddifm.h`, `fm/protocol.h`, `fm/util.h` | `ddifm_impl.h` | `ddi_fm_*(9F)` 6, `ddi_fm_error(9S)` | genunix |
| ksensor | `sensors.h` (the header says the interfaces are "evolving and will change") | `ksensor_impl.h` | `ksensor(9E)`, `ksensor_ops(9E)`, 2 `ksensor_*(9F)` | in `genunix` (`ksensor.o` is in `GENUNIX_OBJS`), `ksensor.mapfile` |
| KGPIO | | `gpio/kgpio_provider.h` (describes itself as a private interface) | | `gpio.mapfile` |
| cmlb | `cmlb.h` | `cmlb_impl.h` | | `misc/cmlb` |
| ktest | `ktest.h` | `ktest_impl.h` | | |
| dcopy | `dcopy.h`, `dcopy_device.h` | | | |
| block ioctls | `dkio.h`, `dklabel.h`, `dkioc_free_util.h`, `vtoc.h`, `efi_partition.h` | | | |
| others | `i2c/`, `1394/`, `sata/`, `fibre-channel/`, `ib/`, `nvme/`, `audio/`, `crypto/` | | | mapfiles exist for `i2c`, `eedev`, `random`, `fnvlist`, `tlimod` |

The blkdev row is the one that matters most for planning. The framework is
used by `nvme`, `vioblk` and the SD card stack, its header describes its
purpose clearly ("a fairly simple block device" that wants the common
labelling support without all of SCSA), and yet it has no manual pages at
all. Binding it as a Committed interface means writing `blkdev(9E)` first.

## Build and link facts

The link rule for every kernel module is at `usr/src/uts/Makefile.targ:40`
and reads `$(LD) -ztype=kmod $(LDFLAGS) -o $@ $(OBJECTS)`. It is followed by
`CTFMERGE_UNIQUIFY_AGAINST_GENUNIX` and `POST_PROCESS`.

`-ztype=kmod` is implemented in `usr/src/cmd/sgs/libld/common/args.c`. It
sets both `FLG_OF_RELOBJ` and `FLG_OF_KMOD`, which is to say it implies
`-r`. It forces `-dy`, and an explicit `-d` on the command line is a fatal
error. It warns when combined with `-z combreloc`. And it emits a
`DT_SUNW_KMOD` dynamic tag (`update.c`, `sections.c`), which is how `krtld`
recognises a kernel module.

`-N <path>` records a `DT_NEEDED` entry. `krtld` reads these entries
(`usr/src/uts/common/krtld/kobj.c:1735`) and collapses them into the
module's `depends_on` string (`kobj.c:769-820`), which drives recursive
loading. The `_depends_on` symbol is a deprecated fallback that is only
consulted when no `DT_NEEDED` entries exist (`kobj.c:1492-1533`). Real
examples of `-N` usage are `-N drv/blkdev -N misc/sata` in
`intel/nvme/Makefile`, `-N misc/virtio -N drv/blkdev` in
`intel/vioblk/Makefile`, and `-Nmisc/cmlb` in `intel/lofi/Makefile`.

Mapfiles are opt-in per module through `usr/src/uts/Makefile.mapfile`,
which adds `-z defs -M $(MAPBASE)/%.mapfile -M dtrace.mapfile` to the link.
The DTrace mapfile is generated from `nm -u` over the objects so that SDT
probe names resolve. `i40e` uses `MAPFILES += ddi mac random`.

The preprocessor defines for all kernel code are `-D_KERNEL -D_SYSCALL32
-D_SYSCALL32_IMPL -D_ELF64 -D_DDI_STRICT -Dsun -D__sun -D__SVR4`. `CPPFLAGS`
is assigned with `=` rather than `+=`, so no userland include path from
`Makefile.master` leaks into kernel builds.

The amd64 compiler flags after the `cw` wrapper are `-m64 -ffreestanding
-mcmodel=kernel -mno-red-zone -mno-mmx -mno-sse -msave-args
-fno-asynchronous-unwind-tables -fno-shrink-wrap
-mindirect-branch=thunk-extern -mindirect-branch-register
-fstack-protector-strong`. The stack protector's `__stack_chk_fail` and
`__stack_chk_guard` are listed as `EXTERN` in `ddi.mapfile`. There is no
`-fno-builtin`. Every object is passed through `ctfconvert`, and intel
kernel modules are built for 64 bit only (`DEF_BUILDS = $(DEF_BUILDS64)`).

## lofi as a footprint sample

`usr/src/uts/common/io/lofi.c` is 3,897 lines and a reasonable example of a
mid-sized driver, so it is worth knowing which interfaces it actually uses.

It is a classic `cb_ops` and `dev_ops` driver, not a blkdev client, and it
uses `cmlb` for disk labels, which is its one `-N` dependency
(`-Nmisc/cmlb`). From the DDI it uses soft state, minor nodes, the
`ddi_prop_*` functions, `ddi_copyin` and `ddi_copyout`,
`ddi_model_convert_from` for 32-bit ioctl callers, `kmem`, mutexes and
condition variables, the raw `taskq_*` functions (not `ddi_taskq_*`),
`buf(9S)` with `biodone` and `bioerror`, and the `cmlb_*` calls.

It also reaches well beyond the DDI. It opens its backing file with
`vn_open` and reads it with `vn_rdwr` and raw `VOP_*` calls rather than
through LDI. It uses the kernel crypto framework, `id_space`, zone keys,
`dkioc_free_util`, `vm/seg_map.h` and the LZMA decoder.

The conclusion for this project is that a lofi port would need the private
tier from the start. A `ramdisk` over blkdev, or something shaped like
`vioblk` (blkdev plus virtio), is the cleaner first block device target.

## Manual sections

Section 9 of the manual has four parts. The counts exclude `Makefile`,
`Manlink` and the `Intro` pages.

| Section | Entries | Content |
| --- | --- | --- |
| 9E | 87 | entry points implemented by the driver, plus the provider sets for MAC (`mc_*`), SCSA (`tran_*`), USB host controllers (`usba_hcdi_*`), `ksensor`, `ddi_ufm`, and `ks_update`/`ks_snapshot` |
| 9F | 561 | functions the driver may call; by prefix `ddi_*` 73, `scsi_*` 41, `net_*` 31, `usb_*` 29, `ldi_*` 24, `mac_*` 14, `ddi_intr_*` 14, `ddi_dma_*` 14, `q*` 11, `bio*` 9, `atomic_*` 9, `kstat_*` 5, `pci_config_*` 5 |
| 9S | 80 | structures: `dev_ops`, `cb_ops`, `modl*`, `buf`, `uio`, `iovec`, `kstat*`, `ddi_dma_attr`, `ddi_dma_cookie`, `ddi_device_acc_attr`, the STREAMS structures, `mac_*`, `scsi_*`, `usb*` |
| 9P | 7 | device properties; none of these pages has an INTERFACE LEVEL section |

For automated classification of a binding surface, the practical rule that
falls out of these numbers is to parse `.SH INTERFACE LEVEL` first, treat
`Interface Stability` as a refinement where it exists, and treat the absence
of both as "unclassified, assume private".

## Coverage of the binding surface

This section was added on 26 September 2026, when the binding surface was
completed, and is produced by `cargo xtask lint-man --gate <gate>
--markdown` against the same gate commit. A name counts as covered when a
`-sys` crate declares a function of that name; the lint separately checks
that its doc comment quotes the right page and level.

| Measure | Count |
| --- | --- |
| 9F prototypes on pages that are not obsolete or private | 1183 |
| bound in the `-sys` crates | 1183 |
| pages skipped as obsolete or "private DDI function" | 27 |
| C macros without a symbol, left to the safe layer | 47 |
| documented, but not exported by the running kernel | 9 |

The declared functions and statics are spread over the crates as follows.
The core crate holds everything that resolves from `genunix` or `unix`;
each subsystem crate holds what its kernel module exports.

| Crate | Symbols declared | Module |
| --- | --- | --- |
| `ddi-sys` | 984 | `genunix`, `unix` |
| `ddi-mac-sys` | 30 | `misc/mac` |
| `ddi-scsa-sys` | 80 | `misc/scsi` |
| `ddi-usba-sys` | 65 | `misc/usba` |
| `ddi-neti-sys` | 34 | `misc/neti`, `misc/hook` |
| `ddi-ksensor-sys` | 6 | `genunix` |
| `ddi-pcmcia-sys` | 83 | `genunix` |
| `ddi-gld-sys` | 14 | `misc/gld` |
| `ddi-ktest-sys` | 16 | `drv/ktest` |

## Findings for upstream

Binding the whole surface turned up a number of things that would be worth
fixing in the gate. They are recorded here so that they can be turned into
issues.

The `ddi` mapfile is far from complete. 497 functions that have a section
9F page and that the kernel exports are not listed, among them
`getminor`, `uioskip`, the semaphore functions, most property lookups and
all of LDI. `cargo xtask lint-man --mapfile-gaps` prints the full list.

Nine documented functions do not exist for drivers. `atomic_inc_ptr`,
`atomic_dec_ptr` and their `_nv` forms, `id_allocff_specific_nosleep`,
`net_netidtozonid` and `mac_prop_info_set_default_uint16` are in no header
and no kernel symbol table. `min` and `max` are declared in `sys/systm.h`
and defined in `subr.c` but not exported by the running kernel.

Several pages disagree with their headers. Examples are the signatures of
`net_hook_unregister`, `net_getlifaddr` and `net_getmtu`, the argument
types of `scsi_reset_notify`, `scsi_ext_sense_fields`, `usb_get_cfg` and
`csx_Get64`, and constant names such as `DTYPE_BRIDGE`, `KEY_RESERVE`,
`ETHER_STAT_LP_CAP_REMFAULT` and `USB_DEV_PWRMASK_D*`. Each case is noted at
the binding concerned.

Some pages are wrong about behaviour. `gld(9F)` describes
`GLD_LINKSTATE_UP` with the text for "down", `ktest_result_pass(9F)` says
`ktest_msg_prepend` appends, and the `csx_DupHandle` and `csx_FreeHandle`
pages describe work that the stubs in `cs_stubs.c` never do.

`/dev/zero` cannot serve a read from inside the kernel. `mmrw` in
`io/mem.c` zeroes the caller's buffer with `uzero` regardless of the
segment flag of the `uio`, so a layered read from `/dev/zero` faults.

`blkdev` still has no manual pages at all, and `esballoca` and
`desballoca` are documented at the DDI level but prototyped only in the
private `sys/strsubr.h`.
