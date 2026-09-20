//! The data model of an ioctl caller, from `sys/model.h` and `sys/sunddi.h`.
//!
//! A 64-bit kernel serves both 32-bit and 64-bit processes. When an ioctl
//! argument contains pointers or `long` values, the driver has to know which
//! layout the caller used. The model bits travel in the `mode` argument of
//! `ioctl(9E)`, and `ddi_model_convert_from(9F)` turns them into a
//! [`model_t`].

use crate::types::c_int;

/// The data model of a process, as used by `ddi_model_convert_from(9F)`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct model_t(pub c_int);
impl model_t {
    /// The caller is a 32-bit process (`DDI_MODEL_ILP32`).
    pub const DDI_MODEL_ILP32: Self = Self(0x00100000);
    /// The caller is a 64-bit process (`DDI_MODEL_LP64`).
    pub const DDI_MODEL_LP64: Self = Self(0x00200000);
    /// The model of the kernel itself (`DDI_MODEL_NATIVE`), which is LP64.
    pub const DDI_MODEL_NATIVE: Self = Self::DDI_MODEL_LP64;
    /// No model information is available (`DDI_MODEL_NONE`).
    pub const DDI_MODEL_NONE: Self = Self(0);
}
/// The bits of the ioctl `mode` argument that carry the data model
/// (`DDI_MODEL_MASK`, also known as `FMODELS`).
pub const DDI_MODEL_MASK: c_int = 0x0FF00000;

unsafe extern "C" {
    /// `ddi_model_convert_from(9F)`. illumos DDI specific (illumos DDI).
    ///
    /// Takes the model bits from the ioctl `mode` and returns the model of
    /// the caller relative to the kernel. On a 64-bit kernel the result is
    /// either `DDI_MODEL_ILP32` for a 32-bit caller or `DDI_MODEL_NONE` when
    /// no conversion is needed.
    pub fn ddi_model_convert_from(model: model_t) -> model_t;
}
