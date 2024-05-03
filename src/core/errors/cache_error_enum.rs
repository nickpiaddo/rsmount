// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library
use std::ffi::NulError;

// From this library

/// [`Cache`](crate::core::cache::Cache) runtime error.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CacheError {
    /// Error while creating a new [`Cache`](crate::core::cache::Cache) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while configuring a [`Cache`](crate::core::cache::Cache) instance.
    #[error("{0}")]
    Config(String),

    /// Error while converting a value to [`CString`](std::ffi::CString).
    #[error("failed to convert value to `CString`: {0}")]
    CStringConversion(#[from] NulError),

    /// Error while importing data from device.
    #[error("{0}")]
    Import(String),
}
