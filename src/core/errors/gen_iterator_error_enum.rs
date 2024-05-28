// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library

/// [`GenIterator`](crate::core::iter::GenIterator) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GenIteratorError {
    /// Error while creating a new [`GenIterator`](crate::core::iter::GenIterator) instance.
    #[error("{0}")]
    Creation(String),
}
