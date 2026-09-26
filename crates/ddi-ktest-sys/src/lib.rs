//! Raw bindings to the illumos kernel test framework, ktest.
//!
//! ktest runs tests inside the kernel, in the same context as the code they
//! exercise. A test is an ordinary kernel function that receives a context
//! handle, does its work, and reports a result through that handle. Tests
//! live in test modules, which are separate miscellaneous kernel modules,
//! and are run from user space with the `ktest(8)` command through the
//! character device of the `ktest` driver. The overview page `ktest(9)`
//! describes the concepts, and the section 9F pages `ktest_create_module`,
//! `ktest_get_fn`, `ktest_get_input` and `ktest_result_pass` describe the
//! functions this crate declares.
//!
//! Every test is named by a triple of module, suite and test name, written
//! as the three names separated by colons. A test module registers its
//! tests in `_init(9E)` in a fixed order. It creates a module object with
//! [`ktest_create_module`], adds a suite to it with [`ktest_add_suite`],
//! adds one or more tests to the suite with [`ktest_add_test`], repeats the
//! last two steps for further suites, and finally registers the module with
//! [`ktest_register_module`]. If registration fails, it frees the module
//! object with [`ktest_free_module`]. It then calls `mod_install(9F)` on a
//! `modlinkage` that holds a `modlmisc(9S)` pointing at `mod_miscops`, and
//! if that fails it calls [`ktest_unregister_module`]. Its `_fini(9E)`
//! calls `ktest_unregister_module` before `mod_remove(9F)`. The gate
//! installs its 64-bit test modules in `/usr/kernel/misc/ktest/amd64` on
//! x86.
//!
//! A test function has the type [`ktest_fn_t`]. It sets exactly one result
//! with [`ktest_result_pass`], [`ktest_result_fail`], [`ktest_result_error`]
//! or [`ktest_result_skip`], and ktest turns a second result into an error
//! that names both lines. It may add context to a failure message with
//! [`ktest_msg_prepend`] and [`ktest_msg_clear`], read an input stream
//! supplied by the user with [`ktest_get_input`] if it was registered with
//! [`ktest_test_flags_t::KTEST_FLAG_INPUT`], and reach static functions of
//! the module under test with [`ktest_hold_mod`], [`ktest_get_fn`] and
//! [`ktest_release_mod`].
//!
//! The functions live in the `ktest` driver, which the gate installs as
//! `drv/ktest`, not in `genunix`. A test module must therefore be linked
//! with `-N drv/ktest`, as the gate's own test modules such as
//! `stream_ktest` are, and the crate's `Cargo.toml` records that dependency
//! in its `[package.metadata.kmod]` table for the link tool.
//!
//! The `ktest_result_pass(9F)` page also documents the `KT_PASS`,
//! `KT_FAIL`, `KT_ERROR` and `KT_SKIP` macros and the `KT_ASSERT*`,
//! `KT_ASSERT*G`, `KT_EASSERT*` and `KT_EASSERT*G` families. They are C
//! preprocessor macros with no symbol behind them. The result macros pass
//! `__LINE__` to the result functions, and the assertion macros evaluate a
//! comparison, format a failure message from the text and values of both
//! operands, call `ktest_result_fail` or `ktest_result_error` and then
//! return from the test function or jump to a cleanup label. A safe Rust
//! layer provides the same behaviour with Rust macros built on the result
//! functions declared here, passing `line!()` for the line and returning
//! early instead of jumping to a label, so this crate does not declare them.
//!
//! The declarations follow the part of `sys/ktest.h` that is guarded by
//! `_KERNEL`. The rules of section 4 of `docs/DESIGN.md` apply. Every
//! function and type names its manual page and quotes its interface level,
//! which is Volatile for all of ktest. The handles are declared with
//! `ddi_sys::opaque!` because ktest keeps their layout private. There is no
//! code, only declarations.
//!
//! Several things in the header are left out. The handle type
//! `ktest_test_hdl_t` is declared by the header but used by no function and
//! documented on no page. The types `ktest_ioctl_t`, `ktest_result_type_t`,
//! `ktest_result_t`, `ktest_run_op_t` and `ktest_list_op_t` and the
//! `KTEST_IOCTL*`, `KTEST_*_KEY`, `KTEST_SER_FMT_*`, `KTEST_SEPARATOR` and
//! `KTEST_DEF_TRIPLE*` constants make up the ioctl interface between the
//! `ktest(8)` command and the driver, which test modules do not use and no
//! section 9 page documents.
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![deny(improper_ctypes, improper_ctypes_definitions)]
#![warn(missing_docs)]

use ddi_sys::{c_char, c_int, c_uchar, c_void, ddi_modhandle_t, opaque, size_t};

opaque! {
    /// `ktest_create_module(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// A ktest module object, `ktest_module_hdl_t`, which C declares as the
    /// private `struct __ktest_module_hdl`. It is created by
    /// [`ktest_create_module`], filled with suites by [`ktest_add_suite`],
    /// and then either registered with [`ktest_register_module`] or freed
    /// with [`ktest_free_module`]. It is only ever handled through a
    /// pointer.
    ktest_module_hdl_t;
    /// `ktest_create_module(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// A ktest suite object, `ktest_suite_hdl_t`, which C declares as the
    /// private `struct __ktest_suite_hdl`. It is created inside a module by
    /// [`ktest_add_suite`] and filled with tests by [`ktest_add_test`]. It
    /// belongs to its module and is freed with it. It is only ever handled
    /// through a pointer.
    ktest_suite_hdl_t;
    /// `ktest_result_pass(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// The context of one test run, `ktest_ctx_hdl_t`, which C declares as
    /// the private `struct __ktest_ctx_hdl`. The ktest facility passes a
    /// pointer to it as the only argument of the test function, and the test
    /// passes it on to the result functions and to [`ktest_get_input`]. It
    /// is only valid while the test function runs.
    ktest_ctx_hdl_t;
}

/// `ktest_create_module(9F)`. Volatile - This interface is still evolving in illumos.
///
/// The type of a test function, whose C typedef `ktest_fn_t` is a pointer to
/// such a function. The ktest facility calls it in kernel context with the
/// context handle `ctx` of the run, and the function reports its outcome by
/// calling exactly one of [`ktest_result_pass`], [`ktest_result_fail`],
/// [`ktest_result_error`] and [`ktest_result_skip`] with that handle before
/// it returns. By convention the name of a test function carries the name
/// of its test module as a prefix and `_test` as a suffix.
pub type ktest_fn_t = unsafe extern "C" fn(ctx: *mut ktest_ctx_hdl_t);

/// `ktest_create_module(9F)`. Volatile - This interface is still evolving in illumos.
///
/// The flags passed as the last argument of [`ktest_add_test`], the C
/// enumeration `ktest_test_flags_t`. They alter the behaviour of ktest or
/// tell it more about the test, and the argument is the bitwise OR of one or
/// more of the values below.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ktest_test_flags_t(pub c_int);
impl ktest_test_flags_t {
    /// No flags (`KTEST_FLAG_NONE`).
    pub const KTEST_FLAG_NONE: Self = Self(0);
    /// The test requires an input stream, which the user supplies when
    /// running it and the test reads with [`ktest_get_input`]
    /// (`KTEST_FLAG_INPUT`).
    pub const KTEST_FLAG_INPUT: Self = Self(1 << 0);
}

// The C enumeration has only small non-negative values, so on amd64 it is
// stored and passed as a 4 byte int. Checked with clang against the gate
// headers.
const _: () = assert!(core::mem::size_of::<ktest_test_flags_t>() == 4);

/// The size of the buffer for a module, suite or test name, including the
/// terminating NUL. A name must therefore be at most 63 characters long,
/// and a longer name makes the registration functions fail with `EOVERFLOW`.
/// A valid name consists only of the ASCII letters and digits, `.` and `_`
/// (`KTEST_MAX_NAME_LEN`).
pub const KTEST_MAX_NAME_LEN: usize = 64;

unsafe extern "C" {
    // --- ktest_create_module(9F) -----------------------------------------

    /// `ktest_create_module(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Creates a module object named by the NUL terminated string `name` and
    /// stores its handle in `km_hdl`. The module name is the first part of
    /// every test triple in the module. It is often the name of the module
    /// under test, but it only needs to be a string that groups the suites
    /// in a sensible way, and it must be unique among registered modules.
    /// The function is called from the test module's `_init(9E)` as the
    /// first step of registration.
    ///
    /// It returns 0 on success. It fails with `EINVAL` if the name contains
    /// characters that are not allowed, with `EOVERFLOW` if it is too long,
    /// and with `ENOMEM` if the object could not be allocated.
    pub fn ktest_create_module(name: *const c_char, km_hdl: *mut *mut ktest_module_hdl_t) -> c_int;

    /// `ktest_add_suite(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Creates a suite object named by the NUL terminated string `name`,
    /// adds it to the module object `km_hdl`, and stores its handle in
    /// `ks_hdl`. Suite names must be unique within their module. The
    /// function is called from `_init(9E)` after [`ktest_create_module`].
    ///
    /// It returns 0 on success. It fails with `EEXIST` if the name is
    /// already in use in the module, with `EINVAL` if it contains characters
    /// that are not allowed, with `EOVERFLOW` if it is too long, and with
    /// `ENOMEM` if the object could not be allocated.
    pub fn ktest_add_suite(
        km_hdl: *mut ktest_module_hdl_t,
        name: *const c_char,
        ks_hdl: *mut *mut ktest_suite_hdl_t,
    ) -> c_int;

    /// `ktest_add_test(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Creates a test object named by the NUL terminated string `name`, with
    /// the test function `fn_` and the flags `flags`, and adds it to the
    /// suite object `ks_hdl`. The test name is often the name of the test
    /// function and must be unique within its suite. `flags` is the bitwise
    /// OR of one or more [`ktest_test_flags_t`] values. The function is
    /// called from `_init(9E)`, once for each test of the suite.
    ///
    /// It returns 0 on success. It fails with `EEXIST` if the name is
    /// already in use in the suite, with `EINVAL` if it contains characters
    /// that are not allowed, with `EOVERFLOW` if it is too long, and with
    /// `ENOMEM` if the object could not be allocated.
    pub fn ktest_add_test(
        ks_hdl: *mut ktest_suite_hdl_t,
        name: *const c_char,
        fn_: ktest_fn_t,
        flags: ktest_test_flags_t,
    ) -> c_int;

    /// `ktest_register_module(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Registers the module object `km_hdl` with the ktest facility, which
    /// makes its tests visible to the user. Only registered tests can be
    /// run. The function is called from `_init(9E)` once all suites and
    /// tests have been added, before `mod_install(9F)`. It returns 0 on
    /// success, or `EEXIST` if a module of the same name is already
    /// registered. On failure the object stays with the caller, which frees
    /// it with [`ktest_free_module`] and returns the error from `_init(9E)`.
    pub fn ktest_register_module(km_hdl: *mut ktest_module_hdl_t) -> c_int;

    /// `ktest_unregister_module(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Unregisters the module object that is registered under the NUL
    /// terminated string `name`. The test module calls it from `_fini(9E)`
    /// before `mod_remove(9F)`, and from `_init(9E)` if `mod_install(9F)`
    /// fails after the module was registered.
    pub fn ktest_unregister_module(name: *const c_char);

    /// `ktest_free_module(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Frees the module object `km_hdl` together with the suites and tests it
    /// contains. It is used when [`ktest_register_module`] failed, so that
    /// `_init(9E)` can release the object before it returns the error.
    pub fn ktest_free_module(km_hdl: *mut ktest_module_hdl_t);

    // --- ktest_get_fn(9F) ------------------------------------------------

    /// `ktest_hold_mod(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Opens the kernel module named by the NUL terminated string `module`,
    /// places a hold on it and stores a handle to it in `hdl`. The hold makes
    /// sure the module and its symbols stay loaded while a test calls its
    /// static functions through pointers obtained with [`ktest_get_fn`]. The
    /// test releases the hold with [`ktest_release_mod`] as part of its
    /// cleanup. It returns 0 on success and a non-zero value on failure. The
    /// error values are not documented, because the underlying
    /// `ddi_modopen(9F)` does not document them.
    pub fn ktest_hold_mod(module: *const c_char, hdl: *mut ddi_modhandle_t) -> c_int;

    /// `ktest_get_fn(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Looks up the function named by the NUL terminated string `fn_name` in
    /// the module `hdl` that [`ktest_hold_mod`] opened and stores its address
    /// in `fnp`. This gives a test access to functions declared `static`,
    /// which it could not otherwise call because the test module is built
    /// as a separate unit. The caller casts the address to a function
    /// pointer type with the same signature as the private function. The
    /// compiler cannot check that signature, so the test has to be kept in
    /// step with the function by hand. The pointer is only valid until the
    /// hold is released with [`ktest_release_mod`].
    ///
    /// The function returns 0 on success and a non-zero value on failure. The
    /// error values are not documented, because the underlying
    /// `ddi_modopen(9F)` does not document them. The C parameter is called
    /// `fn`, which is a Rust keyword.
    pub fn ktest_get_fn(hdl: ddi_modhandle_t, fn_name: *const c_char, fnp: *mut *mut c_void) -> c_int;

    /// `ktest_release_mod(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Releases the hold on the module `hdl` that [`ktest_hold_mod`] placed.
    /// After the call every function pointer obtained through the handle
    /// with [`ktest_get_fn`] must be considered invalid.
    pub fn ktest_release_mod(hdl: ddi_modhandle_t);

    // --- ktest_get_input(9F) ---------------------------------------------

    /// `ktest_get_input(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Retrieves the input stream the user passed to the test whose context
    /// is `ctx`. It stores a pointer to the bytes of the stream in `bytes`
    /// and their number in `len`. Only a test registered with
    /// [`ktest_test_flags_t::KTEST_FLAG_INPUT`] has an input stream, and the
    /// function panics the system when it is called on a context without
    /// one.
    pub fn ktest_get_input(ctx: *const ktest_ctx_hdl_t, bytes: *mut *mut c_uchar, len: *mut size_t);

    // --- ktest_result_pass(9F) -------------------------------------------

    /// `ktest_result_pass(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Sets a passing result for the test whose context is `ctx`. `line` is
    /// the source line where the result is set, which in C is always
    /// `__LINE__` and in Rust `line!()`. If the test has already set a
    /// result, ktest instead records an error that names the line of the
    /// original result and the line of this one. A test that jumps to a
    /// cleanup label must therefore set its passing result before the label.
    pub fn ktest_result_pass(ctx: *mut ktest_ctx_hdl_t, line: c_int);

    /// `ktest_result_fail(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Sets a failure result for the test whose context is `ctx`, with the
    /// message built from the format string `msg` and the arguments that
    /// follow it, and with any text set by [`ktest_msg_prepend`] in front.
    /// `line` is the source line where the result is set. A failure means
    /// that the code under test did not behave as expected. A second result
    /// for the same run is turned into an error, as for
    /// [`ktest_result_pass`]. A Rust caller should pass `"%s"` as the format
    /// and the message as a NUL terminated argument, so that the message is
    /// never interpreted as a format.
    pub fn ktest_result_fail(ctx: *mut ktest_ctx_hdl_t, line: c_int, msg: *const c_char, ...);

    /// `ktest_result_error(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Sets an error result for the test whose context is `ctx`, with the
    /// message built from the format string `msg` and the arguments that
    /// follow it, and with any text set by [`ktest_msg_prepend`] in front.
    /// `line` is the source line where the result is set. An error means
    /// that the test itself could not run as intended, for example because
    /// an allocation it needed failed, rather than that the code under test
    /// misbehaved. A second result for the same run is turned into an error,
    /// as for [`ktest_result_pass`].
    pub fn ktest_result_error(ctx: *mut ktest_ctx_hdl_t, line: c_int, msg: *const c_char, ...);

    /// `ktest_result_skip(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Sets a skip result for the test whose context is `ctx`, with the
    /// message built from the format string `msg` and the arguments that
    /// follow it, which should say why the test was skipped. `line` is the
    /// source line where the result is set. Setting the result does not end
    /// the test, which has to return by itself. A second result for the same
    /// run is turned into an error, as for [`ktest_result_pass`].
    pub fn ktest_result_skip(ctx: *mut ktest_ctx_hdl_t, line: c_int, msg: *const c_char, ...);

    /// `ktest_msg_prepend(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Sets text, built from the format string `msg` and the arguments that
    /// follow it, that is put in front of the failure message of the test
    /// whose context is `ctx`. Each call replaces the text of the previous
    /// one, which makes it convenient inside a loop, for example to record
    /// the index of the element being checked. The manual page describes
    /// the text as appended to the failure message, but ktest keeps it in a
    /// buffer of its own, and `libktest` puts it in front of the message
    /// when it reports the result, as the name of the function says.
    pub fn ktest_msg_prepend(ctx: *mut ktest_ctx_hdl_t, msg: *const c_char, ...);

    /// `ktest_msg_clear(9F)`. Volatile - This interface is still evolving in illumos.
    ///
    /// Clears the text set by [`ktest_msg_prepend`] for the test whose
    /// context is `ctx`, which is the same as prepending an empty string. A
    /// test that sets the text inside a loop calls it after the loop, so
    /// that a later failure does not pick up the text of the last iteration.
    pub fn ktest_msg_clear(ctx: *mut ktest_ctx_hdl_t);
}
