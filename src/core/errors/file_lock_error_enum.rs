// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library
use std::ffi::NulError;

// From this library

/// [`FileLock`](crate::core::fs::FileLock) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FileLockError {
    /// Error while performing an action on a [`FileLock`](crate::core::fs::FileLock) instance.
    #[error("{0}")]
    Action(String),

    /// Error while configuring a new [`FileLock`](crate::core::fs::FileLock) instance.
    #[error("{0}")]
    Config(String),

    /// Error while creating a new [`FileLock`](crate::core::fs::FileLock) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while converting a value to [`CString`](std::ffi::CString).
    #[error("failed to convert value to `CString`: {0}")]
    CStringConversion(#[from] NulError),

    /// Error while locking a file.
    #[error("{0}")]
    Lock(String),
}
