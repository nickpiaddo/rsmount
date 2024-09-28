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

/// A mount point.
#[derive(Debug, Eq, PartialEq)]
pub struct MountPoint {
    path: PathBuf,
}

impl MountPoint {
    #[doc(hidden)]
    /// Creates a new `MountPoint`.
    pub(crate) fn new<T>(path: T) -> MountPoint
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();

        Self { path }
    }

    /// Path name of the mount point.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl AsRef<MountPoint> for MountPoint {
    #[inline]
    fn as_ref(&self) -> &MountPoint {
        self
    }
}

impl fmt::Display for MountPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())
    }
}

impl FromStr for MountPoint {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = Path::new(s);
        if path.is_dir() {
            let device = MountPoint::new(path);

            Ok(device)
        } else {
            let err_msg = format!("A mount point must be a directory. {:?} is not", s);
            Err(ParserError::MountPoint(err_msg))
        }
    }
}

impl From<PathBuf> for MountPoint {
    #[inline]
    fn from(path: PathBuf) -> MountPoint {
        Self::new(path)
    }
}

impl From<&Path> for MountPoint {
    #[inline]
    fn from(path: &Path) -> MountPoint {
        Self::new(path)
    }
}
