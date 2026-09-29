//! Safe wrappers over the private tier, `illumos-kernel-private-sys`.
//!
//! Everything here depends on kernel interfaces without a manual page,
//! checked against illumos gate `cda5b9a49f` only, and exists only with the
//! `unstable-gate-cda5b9a49f` feature (`docs/DESIGN.md` section 4). The
//! wrappers follow the rules of the public layer: the context a function
//! needs is a type, and a runtime guard backs every rule a caller could
//! break with `unsafe` or with a forged token.

pub mod fd;
