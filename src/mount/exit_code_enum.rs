// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use num_enum::{IntoPrimitive, TryFromPrimitive};

// From standard library

// From this library

/// `libmount`'s general return codes (based on the return codes of
/// [mount(8)](https://www.man7.org/linux/man-pages/man8/mount.8.html) and
/// [umount(8)](https://www.man7.org/linux/man-pages/man8/umount.8.html)).
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum ExitCode {
    Failure = libmount::MNT_EX_FAIL,
    /// Error while writing/locking `utab`.
    IoError = libmount::MNT_EX_FILEIO,
    /// Internal mount bug or version error.
    InternalError = libmount::MNT_EX_SOFTWARE,
    /// Some mount operations succeeded, but not all, usually after invoking `mount --all`. Never
    /// returned by `libmount`.
    PartialSuccess = libmount::MNT_EX_SOMEOK,
    Success = libmount::MNT_EX_SUCCESS,
    /// Out of memory error, failed to fork a new process, etc.
    SystemError = libmount::MNT_EX_SYSERR,
    /// Incorrect invocation or lacks mandatory permissions.
    InvalidUsage = libmount::MNT_EX_USAGE,
    UserInterrupt = libmount::MNT_EX_USER,
}
