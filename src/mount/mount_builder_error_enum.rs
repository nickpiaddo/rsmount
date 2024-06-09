// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library
use crate::mount::MountError;

/// [`MountBuilder`](crate::mount::MountBuilder) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MountBuilderError {
    #[error(transparent)]
    Mount(#[from] MountError),

    /// Error if two mutually exclusive setter functions are called.
    #[error("{0}")]
    MutuallyExclusive(String),

    /// Error if required functions were NOT called.
    #[error("{0}")]
    Required(String),
}
