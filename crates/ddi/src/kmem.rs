//! Kernel heap allocation.
//!
//! The one decision that matters when allocating in the kernel is whether
//! the caller may block. This module puts that decision into the function
//! signatures rather than into a flag argument.

use core::alloc::Layout;
use core::mem::{align_of, size_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

use crate::context::Sleepable;
use crate::error::Errno;
use crate::sys;

/// A `T` that lives in memory from `kmem_alloc(9F)` and is returned to the
/// kernel with `kmem_free(9F)` when dropped.
///
/// `kmem_alloc` guarantees 8-byte alignment and nothing more, so `T` must
/// not require a larger alignment. This is checked at compile time.
pub struct KBox<T> {
    ptr: NonNull<T>,
}

unsafe impl<T: Send> Send for KBox<T> {}
unsafe impl<T: Sync> Sync for KBox<T> {}

impl<T> KBox<T> {
    const ALIGN_OK: () = assert!(align_of::<T>() <= 8, "kmem_alloc(9F) only guarantees 8-byte alignment");

    /// Allocates with `KM_SLEEP`. The allocation cannot fail, but it may
    /// block, which is why a [`Sleepable`] token is required.
    pub fn new(value: T, _ctx: &Sleepable) -> KBox<T> {
        match Self::alloc(value, sys::KM_SLEEP) {
            Ok(b) => b,
            // kmem_alloc(9F): "KM_SLEEP … success guaranteed".
            Err(_) => unsafe { sys::panic(c"KBox::new: kmem_alloc(KM_SLEEP) returned NULL".as_ptr()) },
        }
    }

    /// Allocates with `KM_NOSLEEP`. This is allowed from any context and
    /// fails with `ENOMEM` when memory is not available right away.
    pub fn try_new(value: T) -> Result<KBox<T>, Errno> {
        Self::alloc(value, sys::KM_NOSLEEP)
    }

    fn alloc(value: T, flag: i32) -> Result<KBox<T>, Errno> {
        let () = Self::ALIGN_OK;
        let size = size_of::<T>();
        if size == 0 {
            return Ok(KBox { ptr: NonNull::dangling() });
        }
        let raw = unsafe { sys::kmem_alloc(size, flag) } as *mut T;
        let Some(ptr) = NonNull::new(raw) else {
            return Err(Errno::ENOMEM);
        };
        unsafe { ptr::write(ptr.as_ptr(), value) };
        Ok(KBox { ptr })
    }

    /// Gives up ownership and returns the raw pointer. The memory is not
    /// freed until [`KBox::from_raw`] reconstitutes the box and drops it.
    pub fn into_raw(b: KBox<T>) -> *mut T {
        let p = b.ptr.as_ptr();
        core::mem::forget(b);
        p
    }

    /// Takes ownership back from a pointer returned by [`KBox::into_raw`].
    ///
    /// # Safety
    /// `p` must have come from `into_raw` and must not have been passed to
    /// `from_raw` before.
    pub unsafe fn from_raw(p: *mut T) -> KBox<T> {
        KBox { ptr: unsafe { NonNull::new_unchecked(p) } }
    }

    /// Returns the layout of the allocation, which is what `kmem_free(9F)`
    /// needs to be told when the box is dropped.
    pub fn layout() -> Layout {
        Layout::new::<T>()
    }
}

impl<T> Deref for KBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for KBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for KBox<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            let size = size_of::<T>();
            if size != 0 {
                sys::kmem_free(self.ptr.as_ptr() as *mut sys::c_void, size);
            }
        }
    }
}

/// An optional global allocator that always uses `KM_SLEEP`.
///
/// It exists so that prototypes can use `alloc::Box` and `alloc::Vec`, and
/// it is unsound to allocate through it from interrupt context. The feature
/// name, `global-alloc-sleep`, is meant to make that cost visible in the
/// driver's `Cargo.toml`.
#[cfg(feature = "global-alloc-sleep")]
mod global {
    use core::alloc::{GlobalAlloc, Layout};

    use crate::sys;

    struct KmemSleep;

    unsafe impl GlobalAlloc for KmemSleep {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            if layout.align() > 8 {
                return core::ptr::null_mut();
            }
            unsafe { sys::kmem_alloc(layout.size().max(1), sys::KM_SLEEP) as *mut u8 }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe { sys::kmem_free(ptr as *mut sys::c_void, layout.size().max(1)) }
        }
    }

    #[global_allocator]
    static GLOBAL: KmemSleep = KmemSleep;
}
