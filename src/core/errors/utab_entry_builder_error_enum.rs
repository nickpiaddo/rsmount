// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library
use crate::core::errors::UTabEntryError;

/// [`UTabEntryBuilder`](crate::core::entries::UTabEntryBuilder) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UTabEntryBuilderError {
    #[error("{0}")]
    MutuallyExclusive(String),

    #[error("{0}")]
    Required(String),

    #[error(transparent)]
    UTabEntry(#[from] UTabEntryError),
}
