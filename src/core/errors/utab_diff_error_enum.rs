// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library

/// [`UTabDiff`](crate::tables::UTabDiff) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UTabDiffError {
    /// Error while creating a new [`UTabDiff`](crate::tables::UTabDiff) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while comparing [`UTabDiff`](crate::tables::UTabDiff) instances.
    #[error("{0}")]
    Diff(String),
}
