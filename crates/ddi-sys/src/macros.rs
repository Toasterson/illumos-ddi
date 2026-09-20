/// Declare kernel types whose layout is private to the kernel.
///
/// This is the stable replacement for `extern type`: a zero-sized
/// `#[repr(C)]` struct that is `!Send`, `!Sync`, `!Unpin`, and can only be
/// used behind a raw pointer.
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
