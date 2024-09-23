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
    /// Error while parsing a string into a [`BlockDevice`](crate::core::device::BlockDevice).
    #[error("{0}")]
    BlockDevice(String),

    /// Error while parsing a string into a [`Id`](crate::core::device::Id).
    #[error("{0}")]
    Id(String),

    /// Error while parsing a string into a [`Label`](crate::core::device::Label).
    #[error("{0}")]
    Label(String),

    /// Error while parsing a string into a [`MountPoint`](crate::core::device::MountPoint).
    #[error("{0}")]
    MountPoint(String),

    /// Error while parsing a string into a [`NFS`](crate::core::device::NFS).
    #[error("{0}")]
    NFS(String),

    /// Error while parsing a string into a [`Pseudo`](crate::core::device::Pseudo).
    #[error("{0}")]
    Pseudo(String),

    /// Error while parsing a string into a [`SmbFs`](crate::core::device::SmbFs).
    #[error("{0}")]
    SmbFs(String),

    /// Error while parsing a string into a [`SshFs`](crate::core::device::SshFs).
    #[error("{0}")]
    SshFs(String),

    /// Error while parsing a string into a [`Tag`](crate::core::device::Tag).
    #[error("{0}")]
    Tag(String),

    /// Error while parsing a string into a [`TagName`](crate::core::device::TagName).
    #[error("{0}")]
    TagName(String),

    /// Error while parsing a string into a [`Uuid`](crate::core::device::Uuid).
    #[error("{0}")]
    Uuid(String),
}
