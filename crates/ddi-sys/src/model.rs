//! `sys/model.h`, `sys/sunddi.h`: data model of an ioctl caller.
//! `ddi_model_convert_from(9F)`.

use crate::types::c_int;

/// `model_t`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct model_t(pub c_int);
impl model_t {
    /// `DDI_MODEL_ILP32`
    pub const DDI_MODEL_ILP32: Self = Self(0x00100000);
    /// `DDI_MODEL_LP64`
    pub const DDI_MODEL_LP64: Self = Self(0x00200000);
    /// `DDI_MODEL_NATIVE` (LP64 kernel)
    pub const DDI_MODEL_NATIVE: Self = Self::DDI_MODEL_LP64;
    /// `DDI_MODEL_NONE`
    pub const DDI_MODEL_NONE: Self = Self(0);
}
/// `DDI_MODEL_MASK` / `FMODELS`: mask to apply to the ioctl `mode` argument.
pub const DDI_MODEL_MASK: c_int = 0x0FF00000;

unsafe extern "C" {
    /// `ddi_model_convert_from(9F)` — illumos DDI specific (illumos DDI).
    pub fn ddi_model_convert_from(model: model_t) -> model_t;
}
