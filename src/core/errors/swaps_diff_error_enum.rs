// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library

/// [`SwapsDiff`](crate::tables::SwapsDiff) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SwapsDiffError {
    /// Error while creating a new [`SwapsDiff`](crate::tables::SwapsDiff) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while comparing [`SwapsDiff`](crate::tables::SwapsDiff) instances.
    #[error("{0}")]
    Diff(String),
}
