// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library

/// [`MountInfo`](crate::tables::MountInfo) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MountInfoError {
    /// Error while creating a new [`MountInfo`](crate::tables::MountInfo) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while configuring a new [`MountInfo`](crate::tables::MountInfo) instance.
    #[error("{0}")]
    Config(String),

    /// Error while removing duplicate entries in a [`MountInfo`](crate::tables::MountInfo).
    #[error("{0}")]
    Deduplicate(String),

    /// Error while importing new entries into a [`MountInfo`](crate::tables::MountInfo).
    #[error("{0}")]
    Import(String),
}
