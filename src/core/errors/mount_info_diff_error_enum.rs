// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library

/// [`MountInfoDiff`](crate::tables::MountInfoDiff) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MountInfoDiffError {
    /// Error while creating a new [`MountInfoDiff`](crate::tables::MountInfoDiff) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while comparing [`MountInfoDiff`](crate::tables::MountInfoDiff) instances.
    #[error("{0}")]
    Diff(String),
}
