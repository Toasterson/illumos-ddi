# illumos-ddi

Rust for illumos kernel drivers, built on the documented Driver Developer
Interface (the DDI/DKI in section 9 of the manual) rather than on a private
pile of `extern "C"` declarations.

The project is hosted at <https://code.aopc.cloud/illumos/illumos-ddi>.

## Where things stand

On 20 September 2026 the example driver `rnull` was compiled for the kernel
target, linked with the illumos linker as a kernel module, loaded with
`add_drv` on an OpenIndiana Hipster 2026.04 kernel, exercised from userland
(reads, writes and three ioctls), and unloaded again with `rem_drv`. DTrace
saw 456 function boundary probes on the module, and `mdb -k` disassembled its
entry point in the live kernel. The design document records the details.

This is a starting point, not a finished framework. The core of the DDI is
bound, the safe layer covers what a pseudo character driver needs, and the
build path is proven. Registers, interrupts, DMA and the subsystem
frameworks (MAC, blkdev, SCSA, USBA) are still to come.

## What is in the repository

The repository is a Cargo workspace with two library crates, one example
and one build tool.

`crates/ddi-sys` holds the raw bindings: `#[repr(C)]` structures, opaque
handles, constants and `extern "C"` prototypes for the core DDI. Every item
corresponds to one section 9 manual page and says so in its doc comment.
The crate contains no logic.

`crates/ddi` is the layer a driver actually programs against. Its types
carry the rules that the manual pages state in prose. A `Sleepable` token
proves that the current context may block. `Errno` and `DdiError` keep the
two return conventions of the DDI apart. `Mutex` and `CondVar` own their
data and run `mutex_init` and `mutex_destroy` for you. A `MinorNode`
removes itself when dropped. `Uio` and `IoctlArg` are the only way to
touch user memory. The `module!` macro produces the module linkage and the
entry-point glue from a `Driver` implementation.

`examples/rnull` is a pseudo character device that discards writes, returns
end of file on reads, and answers three ioctls. It exists to prove the
toolchain and to serve as a template.

`xtask` builds a driver for the kernel target and links it with
`ld -ztype=kmod`, collecting module dependencies from the crate graph so a
driver never has to spell out `-N` flags itself.

`docs/DESIGN.md` explains the reasoning behind all of this. Read it before
changing the crate boundaries. `docs/ddi-inventory.md` maps the kernel's
headers, manual pages and linker mapfiles to the planned crates.

## Building

Type checking and linting work on any host with stable Rust and the hosted
illumos target installed:

```sh
rustup target add x86_64-unknown-illumos
cargo check --workspace --target x86_64-unknown-illumos
cargo clippy --workspace --target x86_64-unknown-illumos -- -D warnings
```

Building an actual kernel module needs a nightly toolchain, because `core`
and `alloc` have to be rebuilt without SSE for the kernel target. The
compile step runs on any host. The link step needs the illumos linker and
therefore runs on an illumos machine.

```sh
cargo xtask build rnull      # nightly-2026-04-16, -Zbuild-std, targets/x86_64-illumos-kmod.json
cargo xtask link rnull       # on illumos: ld -ztype=kmod, -N flags from crate metadata
cargo xtask deps rnull       # show the -N list and mapfiles the crate graph implies
```

The result of the link is a kernel module file. Installing it is the usual
illumos procedure: copy the module to `/kernel/drv/amd64/`, copy the `.conf`
file to `/kernel/drv/`, and run `add_drv`. The design document lists the
exact commands that were used for the first load.

## Continuous integration

`.forgejo/workflows/ci.yml` runs on Solstice CI. One job type-checks, lints
and checks formatting on a hosted Linux image against the illumos target.
A second job runs natively on an illumos image, installs the pinned
nightly, builds `rnull` for the kernel target and links it as a kernel
module. Neither job loads the module, since that needs root on a kernel
that is allowed to panic.

## Licence

MPL-2.0.
