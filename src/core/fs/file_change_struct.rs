// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::path::{Path, PathBuf};

// From this library
use crate::core::fs::MonitorKind;

/// Detailed changes about monitored mount table files.
#[derive(Debug)]
pub struct FileChange {
    file_name: PathBuf,
    monitor_kind: Option<MonitorKind>,
}

impl FileChange {
    #[doc(hidden)]
    /// Creates a `FileChange`.
    #[allow(dead_code)]
    pub(crate) fn new(file_name: PathBuf, monitor_kind: Option<MonitorKind>) -> FileChange {
        Self {
            file_name,
            monitor_kind,
        }
    }

    /// Returns the name of the last modified file.
    pub fn file_name(&self) -> &Path {
        &self.file_name
    }

    /// Returns the kind of facility used to monitor the file.
    pub fn monitor_kind(&self) -> Option<&MonitorKind> {
        self.monitor_kind.as_ref()
    }
}
