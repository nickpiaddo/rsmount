// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use num_enum::TryFromPrimitiveError;
use thiserror::Error;

// From standard library
use std::ffi::NulError;

// From this library
use crate::mount::ExitCode;

/// [`Mount`](crate::mount::Mount) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MountError {
    /// Error while performing an action on a [`Mount`](crate::mount::Mount) instance.
    #[error("{0}")]
    Action(String),

    /// Error while creating a new [`Mount`](crate::mount::Mount) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while configuring a new [`Mount`](crate::mount::Mount) instance.
    #[error("{0}")]
    Config(String),

    /// Error while converting a value to [`CString`](std::ffi::CString).
    #[error("failed to convert value to `CString`: {0}")]
    CStringConversion(#[from] NulError),

    /// Error if `libmount` was compiled without namespace support.
    #[error("{0}")]
    NoNamespaceSupport(String),

    /// Error while parsing a mount table from a file.
    #[error("{0}")]
    Parse(String),

    /// Error while converting a return code to a [`ExitCode`].
    #[error(transparent)]
    ExitCodeConversion(#[from] TryFromPrimitiveError<ExitCode>),
}
