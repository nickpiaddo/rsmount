// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library

/// [`OptionIter`](crate::core::optstring::OptionIter) runtime error.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum OptionIterError {
    /// Error while creating a new [`OptionIter`](crate::core::optstring::OptionIter) instance.
    #[error("{0}")]
    Creation(String),
}
