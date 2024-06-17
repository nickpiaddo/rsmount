// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

/// Data about child process exit statuses for parallel mounts.
#[derive(Debug)]
pub struct ProcessExitStatus {
    children: usize,
    errors: usize,
}

impl ProcessExitStatus {
    #[doc(hidden)]
    /// Creates a  new `ProcessExitStatus`.
    #[allow(dead_code)]
    pub(crate) fn new(children: usize, errors: usize) -> ProcessExitStatus {
        Self { children, errors }
    }

    /// Returns the number of child processes that ran.
    pub fn children(&self) -> usize {
        self.children
    }

    /// Returns the number of child processes that exited with an error.
    pub fn errors(&self) -> usize {
        self.errors
    }
}
