//! The dispatcher, `sys/disp.h`.

use ddi_sys::c_int;

unsafe extern "C" {
    /// `servicing_interrupt`. Private, `uts/common/sys/disp.h:143`.
    ///
    /// Returns non-zero if the calling thread is an interrupt thread or runs
    /// on the high-level interrupt stack. Any context.
    pub fn servicing_interrupt() -> c_int;
}
