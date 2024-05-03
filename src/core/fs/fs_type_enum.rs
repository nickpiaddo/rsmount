// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library
use crate::core::fs::FileSystem;

/// File system type.
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FsType {
    /// No collision detected during file system identification.
    NoCollision(FileSystem),
    /// Collisions were detected during file system identification. Collisions occur when
    /// conflicting values exist between the main and backup device metadata.
    Collision(FileSystem),
}
