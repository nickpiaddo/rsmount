// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Library-level error module.

// From dependency library
use thiserror::Error;

// From standard library

// From this library
use crate::core::errors::CacheError;
use crate::core::errors::FsTabEntryBuilderError;
use crate::core::errors::FsTabEntryError;
use crate::core::errors::GenIteratorError;
use crate::core::errors::MountInfoEntryError;
use crate::core::errors::ParserError;
use crate::core::errors::SwapsEntryError;
use crate::core::errors::UTabEntryBuilderError;
use crate::core::errors::UTabEntryError;

/// A specialized [`Result`](std::result::Result) type for `rsmount`.
///
/// This typedef is generally used at the program-level to avoid writing out [`RsMountError`]
/// directly, and is, otherwise, a direct mapping to [`Result`](std::result::Result).
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, RsMountError>;

/// Library-level runtime errors.
///
/// This enum includes all variants of error types susceptible to occur in the library. Other, more
/// granular error types, are automatically converted to `RsMountError` when needed.
///
/// # Examples
/// ----
///
/// ```
/// fn main() -> rsmount::Result<()> {
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RsMountError {
    #[error(transparent)]
    Cache(#[from] CacheError),

    #[error(transparent)]
    FsTabEntry(#[from] FsTabEntryError),

    #[error(transparent)]
    FsTabEntryBuilder(#[from] FsTabEntryBuilderError),

    #[error(transparent)]
    GenIterator(#[from] GenIteratorError),

    #[error(transparent)]
    MountInfoEntry(#[from] MountInfoEntryError),

    #[error(transparent)]
    Parser(#[from] ParserError),

    #[error(transparent)]
    SwapsEntry(#[from] SwapsEntryError),

    #[error(transparent)]
    UTabEntry(#[from] UTabEntryError),

    #[error(transparent)]
    UTabEntryBuilder(#[from] UTabEntryBuilderError),
}
