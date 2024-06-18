// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library
use std::ffi::NulError;

// From this library

/// [`TableMonitor`](crate::tables::TableMonitor) runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TableMonitorError {
    /// Error while performing an action on a [`TableMonitor`](crate::tables::TableMonitor) instance.
    #[error("{0}")]
    Action(String),

    /// Error while creating a new [`TableMonitor`](crate::tables::TableMonitor) instance.
    #[error("{0}")]
    Creation(String),

    /// Error while configuring a [`TableMonitor`](crate::tables::TableMonitor) instance.
    #[error("{0}")]
    Config(String),

    /// Error while converting a value to [`CString`](std::ffi::CString).
    #[error("failed to convert value to `CString`: {0}")]
    CStringConversion(#[from] NulError),

    /// Error while processing an I/O event.
    #[error("{0}")]
    Event(String),
}
