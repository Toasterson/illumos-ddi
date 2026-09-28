//! Device nodes, instance and minor numbers and minor nodes. Properties
//! are in [`crate::prop`].

use core::ffi::CStr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU32, Ordering};

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

/// The device node of an attached instance, for queries.
///
/// Only the framework glue creates a `DevInfo`. It is valid from `attach(9E)`
/// until the instance is detached. Because the instance state is destroyed
/// at detach, a `DevInfo` stored inside the instance cannot outlive the node
/// it refers to, which is why the type is `Copy` and may be kept.
///
/// A `DevInfo` answers questions about the node (its properties, name and
/// instance number) and logs on its behalf. It cannot create resources
/// that the kernel ties to the requesting device, such as DMA handles,
/// interrupts, register mappings, minor nodes, driver properties or LDI
/// identifiers; those take the driver's [`OwnNode`], because a `DevInfo` is
/// `Copy` and names no owner.
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

/// The major number the glue recorded at the last `attach(9E)` of this
/// module, or `DDI_MAJOR_T_NONE` before the first. Every Rust driver links its
/// own copy of this crate, so the value is per module.
static ATTACHED_MAJOR: AtomicU32 = AtomicU32::new(DDI_MAJOR_T_NONE);

/// `DDI_MAJOR_T_NONE` from `sys/ddipropdefs.h`, `(major_t)-1`.
const DDI_MAJOR_T_NONE: sys::major_t = sys::major_t::MAX;

/// The driver's own attached device node: the capability to allocate
/// resources that the kernel ties to the requesting device.
///
/// `ddi_dma_alloc_handle(9F)` takes "a pointer to the device's dev_info
/// structure" and says that "a DMA handle must only be used by the device
/// for which it was allocated". Interrupts, register mappings and PCI
/// configuration handles follow the same rule: the kernel asks the node's
/// parent nexus to provide them for that node. Minor nodes, driver
/// properties and LDI identifiers belong to the node and its driver in
/// another way: the kernel removes the minor nodes and driver properties of
/// a node when that node's driver detaches, and an LDI identifier records
/// the node without holding it. Passing another node is outside the DDI. On 28 September 2026 a driver that allocated its DMA
/// handles against `ddi_root_node(9F)` panicked the system at the first
/// bind, through a function pointer the root node never has
/// (`notes/illumos-drm-dma-panic-20260928.md` in the workspace; the
/// "Panics prevented" table in `docs/DESIGN.md`).
///
/// So those constructors take an `OwnNode`, not a [`DevInfo`]. Only the
/// [`module!`](crate::module!) glue creates one, in `attach(9E)` with
/// `DDI_ATTACH`, from the node the kernel passed, and hands it to
/// [`Driver::attach`](crate::Driver::attach). The driver moves it into its
/// instance state, which the framework destroys at detach, so the node
/// cannot be used after it is gone. It is neither `Copy` nor `Clone`, so
/// one instance cannot hand its node to another, and a node from anywhere
/// else needs the `unsafe` [`OwnNode::from_raw`]. For queries it
/// dereferences to the node's [`DevInfo`], which stays `Copy`.
///
/// The type rule has a runtime guard for code that bypasses it with
/// `unsafe`: each constructor that takes an `OwnNode` first refuses a node
/// without a parent (the root node) and a node bound to another driver than
/// the one this module attached, without calling the kernel. The same
/// holds for [`OwnNode::create_minor_node`], for the updates of the
/// property view from [`OwnNode::props`] and for
/// [`LdiIdent::from_dip`](crate::ldi::LdiIdent::from_dip).
///
/// A `DevInfo` does not allocate DMA handles:
///
/// ```compile_fail,E0308
/// fn alloc(dev: ddi::DevInfo, ctx: &ddi::Sleepable) {
///     let _ = ddi::DmaHandle::new(&dev, &ddi::DmaAttr::bits64(), ctx);
/// }
/// ```
///
/// An `OwnNode` cannot be copied:
///
/// ```compile_fail,E0507
/// fn second(node: &ddi::OwnNode) -> ddi::OwnNode {
///     *node
/// }
/// ```
///
/// nor cloned:
///
/// ```compile_fail,E0277
/// fn second(node: &ddi::OwnNode) -> ddi::OwnNode {
///     ddi::OwnNode::clone(node)
/// }
/// ```
///
/// and a node the kernel did not pass to `attach(9E)` needs `unsafe`:
///
/// ```compile_fail,E0133
/// fn root() -> ddi::OwnNode {
///     ddi::OwnNode::from_raw(unsafe { ddi::sys::ddi_root_node() })
/// }
/// ```
///
/// Each of those differs from code that compiles only in the rule it tests:
/// the driver's own node allocates, the capability moves, and the bypass
/// compiles inside `unsafe`.
///
/// ```no_run
/// fn alloc(node: &ddi::OwnNode, ctx: &ddi::Sleepable) {
///     let _ = ddi::DmaHandle::new(node, &ddi::DmaAttr::bits64(), ctx);
/// }
/// fn keep(node: ddi::OwnNode) -> ddi::OwnNode {
///     node
/// }
/// fn root() -> ddi::OwnNode {
///     unsafe { ddi::OwnNode::from_raw(ddi::sys::ddi_root_node()) }
/// }
/// ```
pub struct OwnNode {
    dev: DevInfo,
}

impl OwnNode {
    /// Creates the capability for the node the kernel passed to `attach(9E)`
    /// and records the node's driver major number for the guard.
    ///
    /// # Safety
    /// `dip` must be the node of the current `attach(9E)` call with
    /// `DDI_ATTACH`.
    pub(crate) unsafe fn attached(dip: sys::dev_info_t) -> OwnNode {
        // SAFETY: the caller passes the node of the current attach, which is
        // bound to this driver.
        let major = unsafe { sys::ddi_driver_major(dip) };
        ATTACHED_MAJOR.store(major, Ordering::Relaxed);
        // SAFETY: as above; the kernel never passes NULL to attach(9E).
        OwnNode { dev: unsafe { DevInfo::from_raw(dip) } }
    }

    /// Wraps a node that the framework did not hand to this driver.
    ///
    /// This bypasses the type rule described on [`OwnNode`], and the only
    /// legitimate use is a test of the runtime guard, such as the one in
    /// `rselftest` that passes `ddi_root_node(9F)` and expects
    /// [`DmaError::NotOwnNode`](crate::DmaError::NotOwnNode). A driver gets
    /// its node from [`Driver::attach`](crate::Driver::attach).
    ///
    /// # Safety
    /// `dip` must point to a device node that stays in the device tree for
    /// as long as the returned value exists. It must also be this driver's
    /// own attached node, as `ddi_dma_alloc_handle(9F)` requires ("the
    /// device's dev_info structure"), unless the value is only used to test
    /// the guard. The guard refuses the root node and nodes bound to another
    /// driver, but not a node of another instance of this driver or of a
    /// detached instance, and the kernel may panic for those.
    pub unsafe fn from_raw(dip: sys::dev_info_t) -> OwnNode {
        // SAFETY: the caller promises a live node, which is never NULL.
        OwnNode { dev: unsafe { DevInfo::from_raw(dip) } }
    }

    /// Returns the node for queries, as a `Copy` [`DevInfo`].
    pub fn dev(&self) -> DevInfo {
        self.dev
    }

    /// Creates a minor node with `ddi_create_minor_node(9F)`. The node is
    /// removed again when the returned [`MinorNode`] is dropped, so a driver
    /// keeps it in its instance state for as long as the node should exist.
    ///
    /// Minor nodes are created on the driver's own node: their device number
    /// carries the major number of the driver bound to the node, so opens of
    /// a minor node created on another driver's node reach that driver, and
    /// the kernel removes it when that driver detaches (`detach_node` in
    /// `os/devcfg.c`), after which dropping the [`MinorNode`] would touch a
    /// node that may be gone. A node the [`OwnNode`] guard refuses fails with
    /// [`DdiError`] before the kernel is called.
    ///
    /// A [`DevInfo`] cannot create minor nodes:
    ///
    /// ```compile_fail,E0599
    /// use ddi::{DevInfo, Minor, MinorKind, NodeType};
    /// fn create(dev: DevInfo) {
    ///     let _ = dev.create_minor_node(c"x", MinorKind::Char, Minor(0), NodeType::Pseudo);
    /// }
    /// ```
    ///
    /// The driver's own node can:
    ///
    /// ```no_run
    /// use ddi::{Minor, MinorKind, NodeType, OwnNode};
    /// fn create(node: &OwnNode) {
    ///     let _ = node.create_minor_node(c"x", MinorKind::Char, Minor(0), NodeType::Pseudo);
    /// }
    /// ```
    pub fn create_minor_node(
        &self,
        name: &'static CStr,
        kind: MinorKind,
        minor: Minor,
        node_type: NodeType,
    ) -> Result<MinorNode, DdiError> {
        if !self.is_plausibly_own() {
            return Err(DdiError);
        }
        // SAFETY: the node is live for as long as `self` exists, the name and
        // the node type are NUL-terminated strings the kernel copies or, for
        // the node type, keeps (they are `'static`).
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

    /// The runtime guard: reports whether the node can be this module's own
    /// attached node. The root node has no parent (`ddi_get_parent(9F)`
    /// returns NULL), and a node bound to another driver has another major
    /// number than the one recorded at attach (`ddi_driver_major(9F)`).
    /// Before the first attach there is no recorded major, and only the
    /// parent is checked.
    pub(crate) fn is_plausibly_own(&self) -> bool {
        let dip = self.dev.as_raw();
        // SAFETY: the node is live for as long as `self` exists, by the
        // contract of both constructors. Neither function changes anything.
        let has_parent = !unsafe { sys::ddi_get_parent(dip) }.is_null();
        own_node_verdict(has_parent, ATTACHED_MAJOR.load(Ordering::Relaxed), || unsafe { sys::ddi_driver_major(dip) })
    }
}

/// The decision of [`OwnNode::is_plausibly_own`], apart from the kernel
/// calls: a node needs a parent, and, once a major number was recorded at
/// attach, must be bound to that driver. `major` is only called when needed.
fn own_node_verdict(has_parent: bool, recorded: sys::major_t, major: impl FnOnce() -> sys::major_t) -> bool {
    has_parent && (recorded == DDI_MAJOR_T_NONE || major() == recorded)
}

impl core::ops::Deref for OwnNode {
    type Target = DevInfo;
    fn deref(&self) -> &DevInfo {
        &self.dev
    }
}

impl core::fmt::Debug for OwnNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OwnNode({:?})", self.dev)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_node_is_never_own() {
        assert!(!own_node_verdict(false, DDI_MAJOR_T_NONE, || 7));
        assert!(!own_node_verdict(false, 7, || 7));
    }

    #[test]
    fn major_must_match_once_recorded() {
        assert!(own_node_verdict(true, 7, || 7));
        assert!(!own_node_verdict(true, 7, || 8));
        assert!(!own_node_verdict(true, 7, || DDI_MAJOR_T_NONE));
    }

    #[test]
    fn before_attach_only_the_parent_counts() {
        assert!(own_node_verdict(true, DDI_MAJOR_T_NONE, || panic!("major not needed")));
    }
}
