//! `dev_info_t`, instance and minor numbers, minor nodes, properties.

use core::ffi::CStr;
use core::ptr::NonNull;

use crate::error::DdiError;
use crate::sys;

/// A driver instance number (`ddi_get_instance(9F)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Instance(pub i32);

/// A minor number (`getminor(9F)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Minor(pub u32);

/// Flags passed to `open(9E)`/`close(9E)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenFlags(pub i32);

impl OpenFlags {
    /// `FREAD`
    pub fn read(self) -> bool {
        self.0 & sys::FREAD != 0
    }
    /// `FWRITE`
    pub fn write(self) -> bool {
        self.0 & sys::FWRITE != 0
    }
    /// `FEXCL`
    pub fn exclusive(self) -> bool {
        self.0 & sys::FEXCL != 0
    }
    /// `FNDELAY | FNONBLOCK`
    pub fn nonblocking(self) -> bool {
        self.0 & (sys::FNDELAY | sys::FNONBLOCK) != 0
    }
}

/// The `otyp` argument of `open(9E)`/`close(9E)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenType {
    /// `OTYP_BLK`
    Block,
    /// `OTYP_MNT`
    Mount,
    /// `OTYP_CHR`
    Char,
    /// `OTYP_SWP`
    Swap,
    /// `OTYP_LYR`
    Layered,
    /// Anything else the kernel might add.
    Other(i32),
}

impl OpenType {
    /// Decode the raw `otyp`.
    pub fn from_raw(otyp: i32) -> OpenType {
        match otyp {
            sys::OTYP_BLK => OpenType::Block,
            sys::OTYP_MNT => OpenType::Mount,
            sys::OTYP_CHR => OpenType::Char,
            sys::OTYP_SWP => OpenType::Swap,
            sys::OTYP_LYR => OpenType::Layered,
            o => OpenType::Other(o),
        }
    }
}

/// The device node of an attached instance.
///
/// Created only by the framework glue. Valid from `attach(9E)` until the
/// instance is detached; because the instance state is destroyed at detach,
/// a `DevInfo` stored inside the instance cannot outlive its node.
#[derive(Clone, Copy)]
pub struct DevInfo {
    raw: NonNull<sys::dev_info>,
}

unsafe impl Send for DevInfo {}
unsafe impl Sync for DevInfo {}

impl DevInfo {
    /// Wrap a `dev_info_t`.
    ///
    /// # Safety
    /// `dip` must be the node the kernel passed to the current entry point.
    pub unsafe fn from_raw(dip: sys::dev_info_t) -> DevInfo {
        DevInfo { raw: unsafe { NonNull::new_unchecked(dip) } }
    }

    /// The raw `dev_info_t`, for `9F` calls not yet wrapped.
    pub fn as_raw(&self) -> sys::dev_info_t {
        self.raw.as_ptr()
    }

    /// `ddi_get_instance(9F)`.
    pub fn instance(&self) -> Instance {
        Instance(unsafe { sys::ddi_get_instance(self.as_raw()) })
    }

    /// `ddi_driver_name(9F)`.
    pub fn driver_name(&self) -> &CStr {
        unsafe { CStr::from_ptr(sys::ddi_driver_name(self.as_raw())) }
    }

    /// `ddi_report_dev(9F)`: log the "instance N attached" line.
    pub fn report(&self) {
        unsafe { sys::ddi_report_dev(self.as_raw()) }
    }

    /// `ddi_prop_get_int(9F)` with `DDI_PROP_DONTPASS` on this node.
    pub fn prop_int(&self, name: &CStr, default: i32) -> i32 {
        unsafe {
            sys::ddi_prop_get_int(sys::DDI_DEV_T_ANY, self.as_raw(), sys::DDI_PROP_DONTPASS, name.as_ptr(), default)
        }
    }

    /// `ddi_prop_exists(9F)` with `DDI_PROP_DONTPASS`.
    pub fn prop_exists(&self, name: &CStr) -> bool {
        unsafe { sys::ddi_prop_exists(sys::DDI_DEV_T_ANY, self.as_raw(), sys::DDI_PROP_DONTPASS, name.as_ptr()) != 0 }
    }

    /// `ddi_create_minor_node(9F)`. The node is removed when the returned
    /// [`MinorNode`] is dropped, so keep it in the instance state.
    pub fn create_minor_node(
        &self, name: &'static CStr, kind: MinorKind, minor: Minor, node_type: NodeType,
    ) -> Result<MinorNode, DdiError> {
        let r = unsafe {
            sys::ddi_create_minor_node(self.as_raw(), name.as_ptr(), kind.spec_type(), minor.0, node_type.as_cstr().as_ptr(), 0)
        };
        DdiError::check(r).map(|()| MinorNode { dip: self.as_raw(), name })
    }

    /// `dev_err(9F)` at `CE_WARN`.
    pub fn warn(&self, args: core::fmt::Arguments<'_>) {
        unsafe { crate::log::dev_err(self.as_raw(), crate::log::Level::Warn, args) }
    }

    /// `dev_err(9F)` at `CE_NOTE`.
    pub fn note(&self, args: core::fmt::Arguments<'_>) {
        unsafe { crate::log::dev_err(self.as_raw(), crate::log::Level::Note, args) }
    }
}

/// Character or block minor node (`S_IFCHR`/`S_IFBLK`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinorKind {
    /// `S_IFCHR`
    Char,
    /// `S_IFBLK`
    Block,
}

impl MinorKind {
    fn spec_type(self) -> i32 {
        match self {
            MinorKind::Char => sys::S_IFCHR,
            MinorKind::Block => sys::S_IFBLK,
        }
    }
}

/// The `node_type` string of `ddi_create_minor_node(9F)`, which `devfsadm`
/// uses to decide `/dev` link policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// `DDI_PSEUDO`
    Pseudo,
    /// `DDI_NT_BLOCK`
    Block,
    /// `DDI_NT_BLOCK_CHAN`
    BlockChannel,
    /// `DDI_NT_CD`
    CdRom,
    /// `DDI_NT_SERIAL`
    Serial,
    /// `DDI_NT_NET`
    Net,
    /// Any other `DDI_NT_*` string.
    Custom(&'static CStr),
}

impl NodeType {
    fn as_cstr(self) -> &'static CStr {
        match self {
            NodeType::Pseudo => sys::DDI_PSEUDO,
            NodeType::Block => sys::DDI_NT_BLOCK,
            NodeType::BlockChannel => sys::DDI_NT_BLOCK_CHAN,
            NodeType::CdRom => sys::DDI_NT_CD,
            NodeType::Serial => sys::DDI_NT_SERIAL,
            NodeType::Net => sys::DDI_NT_NET,
            NodeType::Custom(s) => s,
        }
    }
}

/// A minor node that exists while this value does.
pub struct MinorNode {
    dip: sys::dev_info_t,
    name: &'static CStr,
}

unsafe impl Send for MinorNode {}
unsafe impl Sync for MinorNode {}

impl MinorNode {
    /// The node name.
    pub fn name(&self) -> &'static CStr {
        self.name
    }
}

impl Drop for MinorNode {
    fn drop(&mut self) {
        unsafe { sys::ddi_remove_minor_node(self.dip, self.name.as_ptr()) }
    }
}

impl core::fmt::Debug for DevInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}", self.driver_name().to_str().unwrap_or("?"), self.instance().0)
    }
}
