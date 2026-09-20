//! Device nodes, instance and minor numbers, minor nodes and properties.

use core::ffi::CStr;
use core::ptr::NonNull;

use crate::error::DdiError;
use crate::sys;

/// The instance number of a device node, as returned by
/// `ddi_get_instance(9F)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Instance(pub i32);

/// The minor part of a device number, as returned by `getminor(9F)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Minor(pub u32);

/// The open flags passed to `open(9E)` and `close(9E)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenFlags(pub i32);

impl OpenFlags {
    /// Reports whether the file was opened for reading (`FREAD`).
    pub fn read(self) -> bool {
        self.0 & sys::FREAD != 0
    }
    /// Reports whether the file was opened for writing (`FWRITE`).
    pub fn write(self) -> bool {
        self.0 & sys::FWRITE != 0
    }
    /// Reports whether exclusive access was requested (`FEXCL`).
    pub fn exclusive(self) -> bool {
        self.0 & sys::FEXCL != 0
    }
    /// Reports whether non-blocking I/O was requested, in either the
    /// historical or the POSIX form (`FNDELAY` or `FNONBLOCK`).
    pub fn nonblocking(self) -> bool {
        self.0 & (sys::FNDELAY | sys::FNONBLOCK) != 0
    }
}

/// The `otyp` argument of `open(9E)` and `close(9E)`, which says how the
/// device is being opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenType {
    /// As a block device (`OTYP_BLK`).
    Block,
    /// As a block device that is about to be mounted (`OTYP_MNT`).
    Mount,
    /// As a character device (`OTYP_CHR`).
    Char,
    /// As a swap device (`OTYP_SWP`).
    Swap,
    /// By another kernel module through LDI (`OTYP_LYR`).
    Layered,
    /// A value this crate does not know, kept for forward compatibility.
    Other(i32),
}

impl OpenType {
    /// Decodes the raw `otyp` value.
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
/// Only the framework glue creates a `DevInfo`. It is valid from `attach(9E)`
/// until the instance is detached. Because the instance state is destroyed
/// at detach, a `DevInfo` stored inside the instance cannot outlive the node
/// it refers to, which is why the type is `Copy` and may be kept.
#[derive(Clone, Copy)]
pub struct DevInfo {
    raw: NonNull<sys::dev_info>,
}

unsafe impl Send for DevInfo {}
unsafe impl Sync for DevInfo {}

impl DevInfo {
    /// Wraps a `dev_info_t` pointer.
    ///
    /// # Safety
    /// `dip` must be the node the kernel passed to the current entry point.
    pub unsafe fn from_raw(dip: sys::dev_info_t) -> DevInfo {
        DevInfo { raw: unsafe { NonNull::new_unchecked(dip) } }
    }

    /// Returns the raw `dev_info_t`, for `9F` functions that are not wrapped
    /// yet.
    pub fn as_raw(&self) -> sys::dev_info_t {
        self.raw.as_ptr()
    }

    /// Returns the instance number, using `ddi_get_instance(9F)`.
    pub fn instance(&self) -> Instance {
        Instance(unsafe { sys::ddi_get_instance(self.as_raw()) })
    }

    /// Returns the name of the driver bound to this node, using
    /// `ddi_driver_name(9F)`.
    pub fn driver_name(&self) -> &CStr {
        unsafe { CStr::from_ptr(sys::ddi_driver_name(self.as_raw())) }
    }

    /// Logs the line announcing that the instance is attached, using
    /// `ddi_report_dev(9F)`. Conventionally called at the end of a successful
    /// attach.
    pub fn report(&self) {
        unsafe { sys::ddi_report_dev(self.as_raw()) }
    }

    /// Reads an integer property of this node, using `ddi_prop_get_int(9F)`
    /// with `DDI_PROP_DONTPASS`. Returns `default` if the property does not
    /// exist.
    pub fn prop_int(&self, name: &CStr, default: i32) -> i32 {
        unsafe {
            sys::ddi_prop_get_int(sys::DDI_DEV_T_ANY, self.as_raw(), sys::DDI_PROP_DONTPASS, name.as_ptr(), default)
        }
    }

    /// Reports whether this node has the named property, using
    /// `ddi_prop_exists(9F)` with `DDI_PROP_DONTPASS`.
    pub fn prop_exists(&self, name: &CStr) -> bool {
        unsafe { sys::ddi_prop_exists(sys::DDI_DEV_T_ANY, self.as_raw(), sys::DDI_PROP_DONTPASS, name.as_ptr()) != 0 }
    }

    /// Creates a minor node with `ddi_create_minor_node(9F)`. The node is
    /// removed again when the returned [`MinorNode`] is dropped, so a driver
    /// keeps it in its instance state for as long as the node should exist.
    pub fn create_minor_node(
        &self,
        name: &'static CStr,
        kind: MinorKind,
        minor: Minor,
        node_type: NodeType,
    ) -> Result<MinorNode, DdiError> {
        let r = unsafe {
            sys::ddi_create_minor_node(
                self.as_raw(),
                name.as_ptr(),
                kind.spec_type(),
                minor.0,
                node_type.as_cstr().as_ptr(),
                0,
            )
        };
        DdiError::check(r).map(|()| MinorNode { dip: self.as_raw(), name })
    }

    /// Logs a warning prefixed with the driver name and instance, using
    /// `dev_err(9F)` at `CE_WARN`.
    pub fn warn(&self, args: core::fmt::Arguments<'_>) {
        unsafe { crate::log::dev_err(self.as_raw(), crate::log::Level::Warn, args) }
    }

    /// Logs a notice prefixed with the driver name and instance, using
    /// `dev_err(9F)` at `CE_NOTE`.
    pub fn note(&self, args: core::fmt::Arguments<'_>) {
        unsafe { crate::log::dev_err(self.as_raw(), crate::log::Level::Note, args) }
    }
}

/// Whether a minor node is a character or a block device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinorKind {
    /// A character device (`S_IFCHR`).
    Char,
    /// A block device (`S_IFBLK`).
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

/// The node type of a minor node, which `devfsadm` uses to decide how to
/// create `/dev` links for it. Each variant stands for one of the
/// `DDI_NT_*` strings passed to `ddi_create_minor_node(9F)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// A pseudo device (`DDI_PSEUDO`).
    Pseudo,
    /// A block device (`DDI_NT_BLOCK`).
    Block,
    /// A block device addressed by channel (`DDI_NT_BLOCK_CHAN`).
    BlockChannel,
    /// A CD-ROM (`DDI_NT_CD`).
    CdRom,
    /// A serial port (`DDI_NT_SERIAL`).
    Serial,
    /// A network device (`DDI_NT_NET`).
    Net,
    /// Any other node type string the driver wants to use.
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

/// A minor node that exists for as long as this value does. Dropping it
/// calls `ddi_remove_minor_node(9F)`.
pub struct MinorNode {
    dip: sys::dev_info_t,
    name: &'static CStr,
}

unsafe impl Send for MinorNode {}
unsafe impl Sync for MinorNode {}

impl MinorNode {
    /// Returns the name the node was created with.
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
