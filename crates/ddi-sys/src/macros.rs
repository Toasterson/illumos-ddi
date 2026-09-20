/// Declares kernel types whose layout is private to the kernel.
///
/// Each name becomes a zero-sized `#[repr(C)]` structure that cannot be
/// constructed, moved out of a pinned location, or sent between threads. A
/// value of such a type never exists on the Rust side; only raw pointers to
/// it do. This is the stable Rust replacement for the unstable `extern type`
/// feature and behaves the same way at the FFI boundary.
#[macro_export]
macro_rules! opaque {
    ($($(#[$m:meta])* $name:ident;)*) => {$(
        $(#[$m])*
        #[repr(C)]
        pub struct $name {
            _data: [u8; 0],
            _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
        }
    )*};
}
