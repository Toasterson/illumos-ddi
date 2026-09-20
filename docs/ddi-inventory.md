# illumos DDI/DKI surface inventory

Source: `illumos/illumos-gate` at `0fb1d5d8d3` (local clone
`repos/illumos/illumos-gate`), surveyed 2026-09-20. Paths are relative to
the gate root. This backs the crate table in `DESIGN.md` §4.

## Stability model

Three independent signals; none is sufficient alone.

1. `.SH INTERFACE LEVEL` in `usr/src/man/man9{e,f,s}`:
   `Architecture independent level 1 (DDI/DKI).` (≈90 man9f pages, the SVR4
   core) or `illumos DDI specific (illumos DDI).` (≈315 pages). Obsolete
   interfaces say so inline in the same section. Coverage: man9f 515/561,
   man9e 86/87, man9s 77/80, man9p 0/7.
2. `Interface Stability` row in `ATTRIBUTES`: man9f 91 Committed, 15
   Obsolete, 3 Evolving, 2 Private, 1 Volatile; man9e 4 Committed; man9s 8
   Committed + 1 Obsolete.
3. `usr/src/uts/common/mapfiles/*.mapfile` (`ddi`, `mac`, `kernel`, `i2c`,
   `gpio`, `ksensor`, `random`, `fnvlist`, `eedev`, `tlimod`). The README
   there says they are not a formal public/private statement, but `ddi` is
   the machine-readable DDI symbol set and is what `i40e`, `e1000g` link
   against with `-z defs`.

Header installation is *not* a signal: `usr/src/uts/common/sys/Makefile`
installs `ddi_impldefs.h` and `mac_impl.h` next to `sunddi.h`. Naming is
reliable for the negative: `*_impl.h`, `*_impldefs.h`, `*_private.h`,
`esunddi.h` ("drivers calling these functions are not portable").
`ddi_obsolete.h` is under `#ifndef _DDI_STRICT` and the kernel build
defines `-D_DDI_STRICT` (`usr/src/uts/Makefile.uts:141-142`), so it is dead
for in-gate modules.

## Core headers (`usr/src/uts/common/sys/`)

| Header | Family | Public? |
| --- | --- | --- |
| `ddi.h` | SVR4 DDI/DKI umbrella (types/map/buf/uio/stream) | yes |
| `sunddi.h` (65K) | all `ddi_*`/`ndi_*` prototypes: soft state, props, minor nodes, regs, DMA, copyin/out, `ddi_report_dev` | yes, Committed |
| `dditypes.h` | opaque handles `ddi_dma_handle_t`, `ddi_acc_handle_t`, `dev_info_t`, `ddi_device_acc_attr_t` | yes |
| `ddi_intr.h` | `ddi_intr_*` (14 man9f) | yes, Committed |
| `ddi_impldefs.h` (43K), `ddi_implfuncs.h`, `esunddi.h` | `struct dev_info` internals, `i_ddi_*` | **private** |
| `devops.h` | `dev_ops`, `bus_ops`, `cb_ops`, `power_req`, nexus ctlops | yes (`dev_ops(9S)`, `cb_ops(9S)` Committed) |
| `conf.h` | `fmodsw`, `nodev`/`nulldev`/`nochpoll`, `D_MP`/`D_64BIT` | yes |
| `modctl.h` | `modlinkage`, `modldrv`, `modlmisc`, `modlstrmod`, `modlfs`, … (16 kinds), `mod_ops`, `mod_install/remove/info` | yes, Committed |
| `cred.h` | `cred_t`, `crget*`, `drv_priv` (`cred_impl.h` private) | yes |
| `uio.h` | `uio`, `iovec`, `uiomove`, `ureadc/uwritec` | yes, Committed |
| `buf.h` | `buf`, `B_*`, `bio*`, `bp_mapin/out` | yes, level 1 |
| `open.h`, `file.h`, `stat.h`, `errno.h`, `types.h`, `param.h` | otyp, `F*` flags, `S_IF*`, errno, base types, `PAGESIZE`/`DEV_BSIZE` | yes (`file_t` private) |
| `cmn_err.h` | `cmn_err`, `vcmn_err`, `dev_err`, `CE_*` | yes, Committed |
| `kmem.h` | `kmem_alloc/zalloc/free`, `KM_*`; `kmem_cache_*` | yes, Committed |
| `ksynch.h` → `mutex.h`, `condvar.h`, `rwlock.h`, `semaphore.h` | `kmutex_t`, `kcondvar_t`, `krwlock_t`, `ksema_t` | yes, level 1 |
| `ddidmareq.h` | `ddi_dma_attr_t`, `ddi_dma_cookie_t`, `DDI_DMA_*` | yes |
| `ddimapreq.h`, `nexusdefs.h`, `sunndi.h`, `avintr.h` | nexus-side | nexus authors only |
| `ddipropdefs.h` | `ddi_prop_op_t`, `DDI_PROP_*` flags (structs private by its own comment) | enums yes |
| `ddidevmap.h` | `devmap(9E)` framework | yes |
| `ddifm.h` / `ddifm_impl.h` / `ndifm.h` | FMA DDI | yes / private / nexus |
| `ddi_periodic.h` | `ddi_periodic_add/delete` (header also exposes impl) | API yes |
| `ddi_ufm.h` / `ddi_ufm_impl.h` | UFM framework, versioned | yes / private |
| `ddi_hp.h` / `ddi_hp_impl.h` | hotplug | yes / private |
| `ddi_obsolete.h` | obsolete DDI | **exclude** |
| `kstat.h` | `kstat_create/install/delete`, `kstat_named_t`, `kstat_io_t` | yes, Committed |
| `sysmacros.h`, `atomic.h`, `taskq.h`, `debug.h`, `sdt.h`, `list.h`, `id_space.h` | helpers | mixed |
| `usr/src/uts/intel/sys/ddi_isa.h` | ISA-specific access plumbing | per-arch |

## Subsystem frameworks

| Subsystem | Public headers | Private headers | man 9 | kmod / mapfile |
| --- | --- | --- | --- | --- |
| GLDv3 / MAC | `mac_provider.h` (24K, versioned `MAC_VERSION_V1`), `mac.h`, `mac_ether.h`, `mac_client.h` (consumer side) | `mac_impl.h`, `mac_client_impl.h`, `mac_soft_ring.h`, `mac_flow_impl.h` | `mac(9E)`, 17 `mc_*`/`mr_*`/`mi_*`/`mgi_*`(9E), `mac_*(9F)` (14), `mac_register(9S)`, `mac_callbacks(9S)` | `misc/mac`, `mac.mapfile` |
| GLDv2 | `gld.h` | | `gld(9E)` | legacy, do not bind |
| STREAMS | `stream.h` (29K), `strsun.h`, `stropts.h` | `strsubr.h`, `strft.h` | `queue/qinit/msgb/datab/streamtab(9S)`, `put/srv(9E)`, `q*(9F)`, `mi*(9F)` | genunix |
| DLPI / DLD / DLS | `dlpi.h`, `dld.h`, `dld_ioc.h`, `dls.h` | `dld_impl.h`, `dls_impl.h` | | `drv/dld`, `misc/dls` |
| SCSA | `scsi/scsi.h`, `scsi/scsi_{address,pkt,device,ctl,resource,types,params,names,watch,fm}.h` | `scsi/impl/*` (except de-facto public `uscsi.h`) | `scsi_*(9F)` 41, `tran_*(9E)` 14, `scsi_hba_tran/scsi_pkt/scsi_device(9S)` | `misc/scsi` |
| blkdev | `blkdev.h` (`bd_ops_t`, `bd_handle_t`, `bd_xfer_t`, `bd_alloc_handle`, `bd_attach_handle`, `bd_xfer_done`, `bd_state_change`) | | **none** | `drv/blkdev` |
| USBA client | `usb/usba.h`, `usb/usbai.h` | `usba/usba_impl.h`, `usba_private.h`, `usbai_private.h` | `usb_*(9F)` 29, `usb_*(9S)` ~20 | `misc/usba` |
| USBA HCDI | `usb/usba/hcdi.h`, `hubdi.h` | `hcdi_impl.h` | `usba_hcdi_*(9E)` 12, `usba_*(9F)` 9 | `misc/usba` |
| SD card | `sdcard/sda.h`, `sda_ioctl.h` | `sda_impl.h` | | `misc/sda` |
| PCI / PCIe | `pci.h` (51K), `pcie.h` (44K), `pci_cap.h` | `pci_impl.h`, `pcie_impl.h` | `pci_config_*(9F)` 5 | genunix |
| LDI | `sunldi.h` | `sunldi_impl.h` | `ldi_*(9F)` 24 | genunix |
| FMA | `ddifm.h`, `fm/protocol.h`, `fm/util.h` | `ddifm_impl.h` | `ddi_fm_*(9F)` 6, `ddi_fm_error(9S)` | genunix |
| ksensor | `sensors.h` ("evolving and will change") | `ksensor_impl.h` | `ksensor(9E)`, `ksensor_ops(9E)`, 2 `ksensor_*(9F)` | `misc/ksensor`, `ksensor.mapfile` |
| KGPIO | | `gpio/kgpio_provider.h` ("private interface") | | `gpio.mapfile` |
| cmlb | `cmlb.h` | `cmlb_impl.h` | | `misc/cmlb` |
| ktest | `ktest.h` | `ktest_impl.h` | | |
| dcopy | `dcopy.h`, `dcopy_device.h` | | | |
| block ioctls | `dkio.h`, `dklabel.h`, `dkioc_free_util.h`, `vtoc.h`, `efi_partition.h` | | | |
| others | `i2c/`, `1394/`, `sata/`, `fibre-channel/`, `ib/`, `nvme/`, `audio/`, `crypto/` | | | mapfiles for `i2c`, `eedev`, `random`, `fnvlist`, `tlimod` |

## Build and link facts

- Link: `usr/src/uts/Makefile.targ:40` `$(LD) -ztype=kmod $(LDFLAGS) -o $@ $(OBJECTS)`, then `CTFMERGE_UNIQUIFY_AGAINST_GENUNIX`, then `POST_PROCESS`.
- `-ztype=kmod` (`usr/src/cmd/sgs/libld/common/args.c`): sets `FLG_OF_RELOBJ` and `FLG_OF_KMOD` (implies `-r`), forces `-dy` (explicit `-d` is fatal), warns on `-z combreloc`, emits `DT_SUNW_KMOD` (`update.c`, `sections.c`).
- `-N <path>` records `DT_NEEDED`; `krtld` (`usr/src/uts/common/krtld/kobj.c:1735`, `:769-820`) collapses them into `depends_on`. `_depends_on` symbol is a deprecated fallback (`kobj.c:1492-1533`).
- Examples: `intel/nvme/Makefile` `-N drv/blkdev -N misc/sata`; `intel/vioblk/Makefile` `-N misc/virtio -N drv/blkdev`; `intel/lofi/Makefile` `-Nmisc/cmlb`.
- Mapfiles: `usr/src/uts/Makefile.mapfile` adds `-z defs -M $(MAPBASE)/%.mapfile -M dtrace.mapfile` (the latter generated from `nm -u` for SDT probes). `i40e`: `MAPFILES += ddi mac random`.
- Defines: `-D_KERNEL -D_SYSCALL32 -D_SYSCALL32_IMPL -D_ELF64 -D_DDI_STRICT -Dsun -D__sun -D__SVR4`; `CPPFLAGS` is set with `=` so no userland include path leaks.
- amd64 flags after `cw`: `-m64 -ffreestanding -mcmodel=kernel -mno-red-zone -mno-mmx -mno-sse -msave-args -fno-asynchronous-unwind-tables -fno-shrink-wrap -mindirect-branch=thunk-extern -mindirect-branch-register -fstack-protector-strong` (`__stack_chk_fail`/`__stack_chk_guard` are `EXTERN` in `ddi.mapfile`). No `-fno-builtin`.
- Every object is `ctfconvert`ed; intel kmods are 64-bit only (`DEF_BUILDS = $(DEF_BUILDS64)`).

## lofi as a footprint sample (`usr/src/uts/common/io/lofi.c`, 3,897 lines)

Classic `cb_ops`/`dev_ops` (not blkdev), `cmlb` for labels, `-Nmisc/cmlb`.
Uses: soft state, minor nodes, `ddi_prop_*`, `ddi_copyin/out`,
`ddi_model_convert_from`, kmem, mutex+cv, raw `taskq_*`, `buf(9S)`,
`cmlb_*`, **raw VOPs** (`vn_open`, `vn_rdwr`, `VOP_*`, not LDI), KCF
crypto, `id_space`, zone keys, `dkioc_free_util`, `vm/seg_map.h`, LZMA.
Conclusion: a lofi port needs the private tier; `vioblk`
(`blkdev` + `virtio`) or a `ramdisk` over `blkdev` is the cleaner first
block-device target.

## man 9 sections

| Section | Entries | Content |
| --- | --- | --- |
| 9E | 87 | entry points implemented by the driver, plus MAC (`mc_*`), SCSA (`tran_*`), HCDI (`usba_hcdi_*`), `ksensor`, `ddi_ufm`, `ks_update`/`ks_snapshot` |
| 9F | 561 | callable functions; prefixes: `ddi_*` 73, `scsi_*` 41, `net_*` 31, `usb_*` 29, `ldi_*` 24, `mac_*` 14, `ddi_intr_*` 14, `ddi_dma_*` 14, `q*` 11, `bio*` 9, `atomic_*` 9, `kstat_*` 5, `pci_config_*` 5 |
| 9S | 80 | structures: `dev_ops`, `cb_ops`, `modl*`, `buf`, `uio`, `iovec`, `kstat*`, `ddi_dma_attr`, `ddi_dma_cookie`, `ddi_device_acc_attr`, STREAMS, `mac_*`, `scsi_*`, `usb*` |
| 9P | 7 | properties; no INTERFACE LEVEL sections |
