# illumos-ddi

Rust for illumos kernel drivers, built on the documented Driver Developer
Interface (the DDI/DKI in section 9 of the manual) rather than on a private
pile of `extern "C"` declarations.

The project is hosted at <https://code.aopc.cloud/illumos/illumos-ddi>.

## Where things stand

On 20 September 2026 the example driver `rnull` was compiled for the kernel
target, linked with the illumos linker as a kernel module, loaded with
`add_drv` on an OpenIndiana Hipster 2026.04 kernel, exercised from userland
and unloaded again with `rem_drv`.

On 26 September 2026 the binding surface became complete. Every function
that section 9F of the manual documents and the kernel exports is declared,
in the core crate or in one of eight subsystem crates, and every structure
layout and every symbol was checked against the running kernel. The safe
layer now covers what a leaf driver needs: properties, locks of every
kind, timers and task queues, kernel statistics, register and PCI
configuration access, interrupts, DMA, name-value lists and layered
devices. A second example driver, `rselftest`, runs 42 checks of the safe
layer inside the running kernel, and all of them pass.

Two things are not there yet. The hardware parts of the safe layer have
only been checked against the gate source and the compiler, because
testing them needs a device to bind to; porting `pchtemp` is the next step.
And safe layers for the subsystem frameworks, such as MAC for network
drivers, are still to be written on top of their raw bindings.

## What is in the repository

The repository is a Cargo workspace.

`crates/ddi-sys` holds the raw bindings for everything that resolves from
the base kernel: `#[repr(C)]` structures, opaque handles, constants and
`extern "C"` prototypes. Every item corresponds to a section 9 manual page,
and its doc comment names the page and quotes its interface level. The
crate contains no logic.

The subsystem crates follow the same rules for frameworks that live in
their own kernel modules. `ddi-mac-sys` binds the GLDv3 network driver
interface, `ddi-scsa-sys` the SCSI target and host adapter interfaces,
`ddi-usba-sys` the USB client and host controller interfaces,
`ddi-neti-sys` the network information and packet hook interfaces, and
`ddi-ksensor-sys` the kernel sensors. `ddi-pcmcia-sys`, `ddi-gld-sys` and
`ddi-ktest-sys` cover PC Card services, the older GLDv2 network interface
and the kernel test framework. Each crate records the module it needs, so
the link step can add the `-N` flag.

`crates/ddi` is the layer a driver actually programs against. Its types
carry the rules that the manual pages state in prose. A `Sleepable` token
proves that the current context may block, and interrupt handlers receive
an `Atomic` token instead. `Errno`, `DdiError` and a few family-specific
error types keep the return conventions of the DDI apart. Locks own their
data. Handles release what they own when dropped, and callbacks keep their
state alive until the kernel can no longer call them. The `module!` macro
produces the module linkage and the entry-point glue from a `Driver`
implementation.

`examples/rnull` is a pseudo character device that discards writes, returns
end of file on reads, and answers three ioctls. It is the template for a
new driver. `examples/rselftest` is the in-kernel test of the safe layer.

`xtask` builds a driver for the kernel target and links it with
`ld -ztype=kmod`, collecting module dependencies from the crate graph. It
also holds the two checks that keep the bindings honest: `lint-man`
compares every doc comment with its manual page and measures coverage, and
`kernel-check` compares every layout and symbol with a running kernel.

`docs/DESIGN.md` explains the reasoning behind all of this. Read it before
changing the crate boundaries. `docs/ddi-inventory.md` maps the kernel's
headers, manual pages and linker mapfiles to the crates.

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

The two checks need an illumos gate checkout and a running illumos kernel
respectively:

```sh
cargo xtask lint-man --gate ~/illumos-gate --coverage
cargo xtask kernel-check --emit | ssh root@host mdb -k | cargo xtask kernel-check --verify -
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
