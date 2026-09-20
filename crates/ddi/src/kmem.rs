//! Kernel heap allocation with the sleep/no-sleep distinction in the API.

use core::alloc::Layout;
use core::mem::{align_of, size_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

use crate::context::Sleepable;
use crate::error::Errno;
use crate::sys;

/// A `T` allocated with `kmem_alloc(9F)` and freed with `kmem_free(9F)`.
///
/// `kmem_alloc` guarantees only 8-byte alignment, so `T` must not require
/// more; this is checked at compile time.
pub struct KBox<T> {
    ptr: NonNull<T>,
}

unsafe impl<T: Send> Send for KBox<T> {}
unsafe impl<T: Sync> Sync for KBox<T> {}

impl<T> KBox<T> {
    const ALIGN_OK: () = assert!(align_of::<T>() <= 8, "kmem_alloc(9F) only guarantees 8-byte alignment");

    /// Allocate with `KM_SLEEP`. Cannot fail; may block, hence the token.
    pub fn new(value: T, _ctx: &Sleepable) -> KBox<T> {
        match Self::alloc(value, sys::KM_SLEEP) {
            Ok(b) => b,
            // kmem_alloc(9F): "KM_SLEEP … success guaranteed".
            Err(_) => unsafe { sys::panic(c"KBox::new: kmem_alloc(KM_SLEEP) returned NULL".as_ptr()) },
        }
    }

    /// Allocate with `KM_NOSLEEP`. Allowed from any context; may fail with
    /// `ENOMEM`.
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

    /// Leak the allocation, returning the raw pointer.
    pub fn into_raw(b: KBox<T>) -> *mut T {
        let p = b.ptr.as_ptr();
        core::mem::forget(b);
        p
    }

    /// Reconstitute from [`KBox::into_raw`].
    ///
    /// # Safety
    /// `p` must come from `into_raw` and not have been reconstituted before.
    pub unsafe fn from_raw(p: *mut T) -> KBox<T> {
        KBox { ptr: unsafe { NonNull::new_unchecked(p) } }
    }

    /// The layout `kmem_free(9F)` needs back.
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

/// Optional `KM_SLEEP` global allocator. Prototype use only; see the
/// `global-alloc-sleep` feature.
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
