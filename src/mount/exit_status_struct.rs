// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library
use crate::mount::ExitCode;

/// Data about the [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html)'s exit status.
#[derive(Debug)]
pub struct ExitStatus {
    exit_code: ExitCode,
    error_message: String,
}

impl ExitStatus {
    #[doc(hidden)]
    /// Creates a  new `ExitStatus`.
    #[allow(dead_code)]
    pub(crate) fn new(exit_code: ExitCode, error_message: String) -> ExitStatus {
        Self {
            exit_code,
            error_message,
        }
    }

    /// Returns the [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html)'s
    /// exit code.
    pub fn exit_code(&self) -> &ExitCode {
        &self.exit_code
    }

    /// Returns a mount's error message.
    pub fn error_message(&self) -> &str {
        &self.error_message
    }
}
