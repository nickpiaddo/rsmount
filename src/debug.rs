// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Activate debug message output.
//!
//! `rsmount` provides a facility to log debug messages through the
//! [log](https://crates.io/crates/log) lightweight logging *facade*.
//!
//! From the package's README, you need to provide a logger implementation compatible with the
//! *facade*:
//!
//! > In order to produce log output, executables have to use a logger implementation compatible with the facade.
//! > There are many available implementations to choose from, here are some options:
//! >
//! > * Simple minimal loggers:
//! >     * [`env_logger`](https://docs.rs/env_logger/*/env_logger/)
//! >     * [`colog`](https://docs.rs/colog/*/colog/)
//! >     * [`simple_logger`](https://docs.rs/simple_logger/*/simple_logger/)
//! >     * [`simplelog`](https://docs.rs/simplelog/*/simplelog/)
//! >     * [`pretty_env_logger`](https://docs.rs/pretty_env_logger/*/pretty_env_logger/)
//! >     * [`stderrlog`](https://docs.rs/stderrlog/*/stderrlog/)
//! >     * [`flexi_logger`](https://docs.rs/flexi_logger/*/flexi_logger/)
//! >     * [`call_logger`](https://docs.rs/call_logger/*/call_logger/)
//! >     * [`std-logger`](https://docs.rs/std-logger/*/std_logger/)
//! >     * [`structured-logger`](https://docs.rs/structured-logger/latest/structured_logger/)
//! > * Complex configurable frameworks:
//! >     * [`log4rs`](https://docs.rs/log4rs/*/log4rs/)
//! >     * [`logforth`](https://docs.rs/logforth/*/logforth/)
//! >     * [`fern`](https://docs.rs/fern/*/fern/)
//! > * Adaptors for other facilities:
//! >     * [`syslog`](https://docs.rs/syslog/*/syslog/)
//! >     * [`systemd-journal-logger`](https://docs.rs/systemd-journal-logger/*/systemd_journal_logger/)
//! >     * [`slog-stdlog`](https://docs.rs/slog-stdlog/*/slog_stdlog/)
//! >     * [`android_log`](https://docs.rs/android_log/*/android_log/)
//! >     * [`win_dbg_logger`](https://docs.rs/win_dbg_logger/*/win_dbg_logger/)
//! >     * [`db_logger`](https://docs.rs/db_logger/*/db_logger/)
//! >     * [`log-to-defmt`](https://docs.rs/log-to-defmt/*/log_to_defmt/)
//! >     * [`logcontrol-log`](https://docs.rs/logcontrol-log/*/logcontrol_log/)
//! > * For WebAssembly binaries:
//! >     * [`console_log`](https://docs.rs/console_log/*/console_log/)
//! > * For dynamic libraries:
//! >     * You may need to construct [an FFI-safe wrapper over `log`](https://github.com/rust-lang/log/issues/421) to initialize in your libraries.
//! > * Utilities:
//! >     * [`log_err`](https://docs.rs/log_err/*/log_err/)
//! >     * [`log-reload`](https://docs.rs/log-reload/*/log_reload/)
//! >     * [`alterable_logger`](https://docs.rs/alterable_logger/*/alterable_logger)
//! >
//! > Executables should choose a logger implementation and initialize it early in the
//! > runtime of the program. Logger implementations will typically include a
//! > function to do this. Any log messages generated before the logger is
//! > initialized will be ignored.
//! >
//! > The executable itself may use the `log` crate to log as well.
//!
//! Here is an example of debug message initialization using the
//! [`env_logger`](https://docs.rs/env_logger/*/env_logger/) crate, and `libmount`'s own debug
//! interface.
//!
//! ```ignore
//! static INIT: std::sync::Once = std::sync::Once::new();
//!
//! fn main() {
//!    // Initialize debug output
//!    INIT.call_once(|| {
//!        // rsmount debug messages
//!        env_logger::init();
//!        // libmount debug messages
//!        rsmount::debug::init_default_debug();
//!    });
//!
//!    // The rest of your program...
//!
//! }
//!
//! ```
//!
//! Assuming your executable is called `main` you can adjust the log-level of `libmount` and/or
//! `rsmount` by setting respectively the `LIBMOUNT_DEBUG` and/or `RUST_LOG` environment variables.
//!
//! ```text
//! # libmount debug messages only
//! # (look to the `init_default_debug` function's documentation for an exhaustive list of options)
//! $ LIBMOUNT_DEBUG="cxt,fs,loop" ./main
//! ```
//!
//! Example output:
//! ```text
//! ...snip...
//! ```
//!
//! ```text
//! # rsmount debug messages only
//! $ RUST_LOG=debug ./main
//! ```
//!
//! Example output:
//! ```text
//! ...snip...
//! ```
//!
//! Debugging modes can not be modified after calling [`init_default_debug`] or [`init_full_debug`]
//! once. The first function to get called sets the debug mode; a debug mode you can NOT change as
//! long as your program is running.

/// Activates library debugging messages. This function reads the `LIBMOUNT_DEBUG` environment
/// variable to set the level of debug output.
///
/// It accepts the following values:
/// - `all`:      info about all subsystems
/// - `cache`:    paths and tags cache
/// - `cxt`:      library context (handler)
/// - `diff`:     mountinfo changes tracking
/// - `fs`:       FS abstraction
/// - `help`:     this help
/// - `hook`:     hooks functionality
/// - `locks`:    mtab and utab locking
/// - `loop`:     loop devices routines
/// - `options`:  mount options parsing
/// - `optlist`:  mount options container
/// - `tab`:      fstab, mtab, mountinfo routines
/// - `update`:   mtab, utab updates
/// - `utils`:    misc library utils
/// - `monitor`:  mount tables monitor
/// - `btrfs`:    btrfs specific routines
/// - `verify`:   verity specific routines
///
/// # Examples
///
/// ```console
/// # You can set multiple values separated by commas
/// LIBMOUNT_DEBUG="cache,options,btrfs"
/// ```
pub fn init_default_debug() {
    unsafe { libmount::mnt_init_debug(0) }
}

/// Enables full debugging.
pub fn init_full_debug() {
    unsafe { libmount::mnt_init_debug(0xffff) }
}
