// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Module for working with file systems.

// From dependency library

// From standard library

// From this library
pub use file_lock_struct::FileLock;
pub use file_system_enum::FileSystem;
pub use fs_type_enum::FsType;
pub use monitor_kind_enum::MonitorKind;

mod file_lock_struct;
mod file_system_enum;
mod fs_type_enum;
mod monitor_kind_enum;
