// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library
use std::ffi::NulError;

// From this library

/// [`UtabManager`](crate::tables::UtabManager) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UtabManagerError {
    /// Error while performing an action on a [`UtabManager`](crate::tables::UtabManager) instance.
    #[error("{0}")]
    Action(String),

    /// Error while creating a new [`UtabManager`](crate::tables::UtabManager) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while configuring a [`UtabManager`](crate::tables::UtabManager) instance.
    #[error("{0}")]
    Config(String),

    /// Error while converting a value to [`CString`](std::ffi::CString).
    #[error("failed to convert value to `CString`: {0}")]
    CStringConversion(#[from] NulError),
}
