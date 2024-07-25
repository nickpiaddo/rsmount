// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library
use std::ffi::NulError;

// From this library
use crate::core::errors::UTabIterError;

/// [`UTab`](crate::tables::UTab) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UTabError {
    /// Error while performing an action on a [`FsTab`](crate::tables::FsTab) instance.
    #[error("{0}")]
    Action(String),

    /// Error while creating a new [`UTab`](crate::tables::UTab) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while configuring a new [`UTab`](crate::tables::UTab) instance.
    #[error("{0}")]
    Config(String),

    /// Error while converting a value to [`CString`](std::ffi::CString).
    #[error("failed to convert value to `CString`: {0}")]
    CStringConversion(#[from] NulError),

    /// Error while removing duplicate entries in a [`UTab`](crate::tables::UTab).
    #[error("{0}")]
    Deduplicate(String),

    /// Error while exporting new entries into a [`UTab`](crate::tables::UTab).
    #[error("{0}")]
    Export(String),

    /// Error while importing new entries into a [`UTab`](crate::tables::UTab).
    #[error("{0}")]
    Import(String),

    /// Error while indexing entries in [`FsTab`](crate::tables::FsTab).
    #[error("{0}")]
    IndexOutOfBounds(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Error if a file lacks the required access permissions.
    #[error("{0}")]
    Permission(String),

    #[error(transparent)]
    UTabIter(#[from] UTabIterError),
}
