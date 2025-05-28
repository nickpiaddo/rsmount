// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library

/// [`FsTabDiff`](crate::tables::FsTabDiff) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FsTabDiffError {
    /// Error while creating a new [`FsTabDiff`](crate::tables::FsTabDiff) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while comparing [`FsTabDiff`](crate::tables::FsTabDiff) instances.
    #[error("{0}")]
    Diff(String),
}
