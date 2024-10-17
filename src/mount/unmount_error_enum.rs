// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use num_enum::TryFromPrimitiveError;
use thiserror::Error;

// From standard library
use std::ffi::NulError;

// From this library
use crate::mount::ExitCode;

/// [`Unmount`](crate::mount::Unmount) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UnmountError {
    /// Error while creating a new [`Unmount`](crate::mount::Unmount) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while configuring a new [`Unmount`](crate::mount::Unmount) instance.
    #[error("{0}")]
    Config(String),

    /// Error while converting a value to [`CString`](std::ffi::CString).
    #[error("failed to convert value to `CString`: {0}")]
    CStringConversion(#[from] NulError),

    /// Error while converting a return code to a [`ExitCode`].
    #[error(transparent)]
    ExitCodeConversion(#[from] TryFromPrimitiveError<ExitCode>),

    /// Error if `libmount` was compiled without namespace support.
    #[error("{0}")]
    NoNamespaceSupport(String),
}
