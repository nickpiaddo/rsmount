// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library
use crate::core::errors::GenIteratorError;

/// [`MountInfoIter`](crate::core::iter::MountInfoIter) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MountInfoIterError {
    #[error(transparent)]
    GenIterator(#[from] GenIteratorError),
}
