// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library
use std::ffi::NulError;

// From this library

/// [`FsTabEntry`](crate::core::entries::FsTabEntry) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MountInfoEntryError {
    /// Error while performing an action on a [`MountInfoEntry`](crate::core::entries::MountInfoEntry) instance.
    #[error("{0}")]
    Action(String),

    /// Error while creating a new [`MountInfoEntry`](crate::core::entries::MountInfoEntry) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while configuring a new [`MountInfoEntry`](crate::core::entries::MountInfoEntry) instance.
    #[error("{0}")]
    Config(String),

    /// Error while converting a value to [`CString`](std::ffi::CString).
    #[error("failed to convert value to `CString`: {0}")]
    CStringConversion(#[from] NulError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Error when accessing a file without having the proper permissions.
    #[error("{0}")]
    Permission(String),
}
