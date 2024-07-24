// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library

/// [`Swaps`](crate::tables::Swaps) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SwapsError {
    /// Error while creating a new [`Swaps`](crate::tables::Swaps) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while configuring a new [`Swaps`](crate::tables::Swaps) instance.
    #[error("{0}")]
    Config(String),

    /// Error while removing duplicate entries in a [`Swaps`](crate::tables::Swaps).
    #[error("{0}")]
    Deduplicate(String),

    /// Error while importing new entries into a [`Swaps`](crate::tables::Swaps).
    #[error("{0}")]
    Import(String),
}
