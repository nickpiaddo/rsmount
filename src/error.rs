// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Library-level error module.

// From dependency library
use thiserror::Error;

// From standard library

// From this library
use crate::core::errors::CacheError;
use crate::core::errors::FileLockError;
use crate::core::errors::FsTabDiffError;
use crate::core::errors::FsTabDiffIterError;
use crate::core::errors::FsTabEntryBuilderError;
use crate::core::errors::FsTabEntryError;
use crate::core::errors::FsTabError;
use crate::core::errors::FsTabIterError;
use crate::core::errors::GenIteratorError;
use crate::core::errors::MountInfoChildIterError;
use crate::core::errors::MountInfoDiffError;
use crate::core::errors::MountInfoDiffIterError;
use crate::core::errors::MountInfoEntryError;
use crate::core::errors::MountInfoError;
use crate::core::errors::MountInfoIterError;
use crate::core::errors::OptionIterError;
use crate::core::errors::ParserError;
use crate::core::errors::SwapsDiffError;
use crate::core::errors::SwapsDiffIterError;
use crate::core::errors::SwapsEntryError;
use crate::core::errors::SwapsError;
use crate::core::errors::SwapsIterError;
use crate::core::errors::TableMonitorError;
use crate::core::errors::UTabDiffError;
use crate::core::errors::UTabDiffIterError;
use crate::core::errors::UTabEntryBuilderError;
use crate::core::errors::UTabEntryError;
use crate::core::errors::UTabError;
use crate::core::errors::UTabIterError;
use crate::core::errors::UtabManagerError;

use crate::core::version::VersionError;

use crate::mount::MountBuilderError;
use crate::mount::MountError;
use crate::mount::MountIterError;
use crate::mount::ReMountIterError;
use crate::mount::UMountIterError;
use crate::mount::UnmountBuilderError;
use crate::mount::UnmountError;

/// A specialized [`Result`](std::result::Result) type for `rsmount`.
///
/// This typedef is generally used at the program-level to avoid writing out [`RsMountError`]
/// directly, and is, otherwise, a direct mapping to [`Result`](std::result::Result).
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, RsMountError>;

/// Library-level runtime errors.
///
/// This enum includes all variants of error types susceptible to occur in the library. Other, more
/// granular error types, are automatically converted to `RsMountError` when needed.
///
/// # Examples
/// ----
///
/// ```
/// fn main() -> rsmount::Result<()> {
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RsMountError {
    #[error(transparent)]
    Cache(#[from] CacheError),

    #[error(transparent)]
    FileLock(#[from] FileLockError),

    #[error(transparent)]
    FsTab(#[from] FsTabError),

    #[error(transparent)]
    FsTabDiff(#[from] FsTabDiffError),

    #[error(transparent)]
    FsTabDiffIter(#[from] FsTabDiffIterError),

    #[error(transparent)]
    FsTabEntry(#[from] FsTabEntryError),

    #[error(transparent)]
    FsTabEntryBuilder(#[from] FsTabEntryBuilderError),

    #[error(transparent)]
    FsTabIter(#[from] FsTabIterError),

    #[error(transparent)]
    GenIterator(#[from] GenIteratorError),

    #[error(transparent)]
    Mount(#[from] MountError),

    #[error(transparent)]
    MountBuilder(#[from] MountBuilderError),

    #[error(transparent)]
    MountInfo(#[from] MountInfoError),

    #[error(transparent)]
    MountInfoChildIter(#[from] MountInfoChildIterError),

    #[error(transparent)]
    MountInfoDiff(#[from] MountInfoDiffError),

    #[error(transparent)]
    MountInfoDiffIter(#[from] MountInfoDiffIterError),

    #[error(transparent)]
    MountInfoEntry(#[from] MountInfoEntryError),

    #[error(transparent)]
    MountInfoIter(#[from] MountInfoIterError),

    #[error(transparent)]
    MountIter(#[from] MountIterError),

    #[error(transparent)]
    ReMountIter(#[from] ReMountIterError),

    #[error(transparent)]
    UMountIter(#[from] UMountIterError),

    // #[error(transparent)]
    // MountTable(#[from] MountTableError),

    // #[error(transparent)]
    // MountTableEntry(#[from] MountTableEntryError),

    // #[error(transparent)]
    // MountTableEntryBuilder(#[from] MountTableEntryBuilderError),
    #[error(transparent)]
    OptionIter(#[from] OptionIterError),

    #[error(transparent)]
    Parser(#[from] ParserError),

    #[error(transparent)]
    Swaps(#[from] SwapsError),

    #[error(transparent)]
    SwapsDiff(#[from] SwapsDiffError),

    #[error(transparent)]
    SwapsDiffIter(#[from] SwapsDiffIterError),

    #[error(transparent)]
    SwapsEntry(#[from] SwapsEntryError),

    #[error(transparent)]
    SwapsIter(#[from] SwapsIterError),

    #[error(transparent)]
    TableMonitor(#[from] TableMonitorError),

    #[error(transparent)]
    Unmount(#[from] UnmountError),

    #[error(transparent)]
    UnmountBuilder(#[from] UnmountBuilderError),

    #[error(transparent)]
    UTab(#[from] UTabError),

    #[error(transparent)]
    UTabDiff(#[from] UTabDiffError),

    #[error(transparent)]
    UTabDiffIter(#[from] UTabDiffIterError),

    #[error(transparent)]
    UTabEntry(#[from] UTabEntryError),

    #[error(transparent)]
    UTabEntryBuilder(#[from] UTabEntryBuilderError),

    #[error(transparent)]
    UTabIter(#[from] UTabIterError),

    #[error(transparent)]
    UtabManager(#[from] UtabManagerError),

    #[error(transparent)]
    Version(#[from] VersionError),
}
