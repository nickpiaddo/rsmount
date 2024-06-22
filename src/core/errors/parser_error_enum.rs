// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use thiserror::Error;

// From standard library

// From this library

/// String parser runtime errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParserError {
    /// Error while parsing a string into a [`Id`](crate::core::device::Id).
    #[error("{0}")]
    Id(String),

    /// Error while parsing a string into a [`Label`](crate::core::device::Label).
    #[error("{0}")]
    Label(String),

    /// Error while parsing a string into a [`TagName`](crate::core::device::TagName).
    #[error("{0}")]
    TagName(String),

    /// Error while parsing a string into a [`Uuid`](crate::core::device::Uuid).
    #[error("{0}")]
    Uuid(String),
}
