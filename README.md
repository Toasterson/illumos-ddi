# illumos-ddi

Hosted at <https://code.aopc.cloud/illumos/illumos-ddi>. State: the `rnull`
example driver has been built, linked with `ld -ztype=kmod`, loaded, exercised
and unloaded on an OpenIndiana Hipster 2026.04 kernel (2026-09-20).

Rust bindings and a safe driver-development layer for the illumos kernel
Driver Developer Interface (DDI/DKI, manual section 9).

- `crates/ddi-sys`: raw `#[repr(C)]` types, constants and `extern "C"`
  prototypes for the core DDI, one item per section 9 man page.
- `crates/ddi`: the safe layer drivers program against (`module!`,
  `Driver`, `CharDevice`, `KBox`, `Mutex`, `CondVar`, `Uio`, `IoctlArg`, …).
- `examples/rnull`: a pseudo character driver used to prove the toolchain.
- `xtask`: builds a driver for the kernel target and links it with
  `ld -ztype=kmod`.

Read `docs/DESIGN.md` first. `docs/ddi-inventory.md` maps the kernel's
headers, man pages and mapfiles to the planned crate boundaries.

```sh
cargo check --target x86_64-unknown-illumos      # any host
cargo xtask build rnull                          # nightly-2026-04-16 + -Zbuild-std
cargo xtask link rnull                           # on illumos: ld -ztype=kmod
```
