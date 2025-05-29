// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library
use crate::core::errors::GenIteratorError;

/// [`UMountIter`](crate::mount::UMountIter) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UMountIterError {
    #[error(transparent)]
    GenIterator(#[from] GenIteratorError),
}
