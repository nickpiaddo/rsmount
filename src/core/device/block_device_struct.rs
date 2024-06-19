// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

// From this library
use crate::core::errors::ParserError;

/// A block device.
#[derive(Debug, Eq, PartialEq)]
pub struct BlockDevice {
    path: PathBuf,
}

impl BlockDevice {
    #[doc(hidden)]
    /// Creates a new `BlockDevice`.
    pub(crate) fn new<T>(path: T) -> BlockDevice
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();

        Self { path }
    }

    /// Path name of the block device.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl AsRef<BlockDevice> for BlockDevice {
    #[inline]
    fn as_ref(&self) -> &BlockDevice {
        self
    }
}

impl fmt::Display for BlockDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())
    }
}

impl FromStr for BlockDevice {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = Path::new(s);
        let device = BlockDevice::new(path);

        Ok(device)
    }
}

impl From<PathBuf> for BlockDevice {
    #[inline]
    fn from(path: PathBuf) -> BlockDevice {
        Self::new(path)
    }
}

impl From<&PathBuf> for BlockDevice {
    #[inline]
    fn from(path: &PathBuf) -> BlockDevice {
        Self::new(path)
    }
}

impl From<&Path> for BlockDevice {
    #[inline]
    fn from(path: &Path) -> BlockDevice {
        Self::new(path)
    }
}