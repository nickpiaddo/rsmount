// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! High-level API to mount/unmount devices.

pub use error_code_enum::ErrorCode;
pub use exit_code_enum::ExitCode;
pub use exit_status_struct::ExitStatus;
pub use mount_builder_error_enum::MountBuilderError;
pub use mount_error_enum::MountError;
pub use mount_namespace_struct::MountNamespace;
pub use mount_options_mode_enum::MountOptionsMode;
pub use mount_source_enum::MountSource;
pub use mount_struct::Mount;
pub use process_exit_status_struct::ProcessExitStatus;

mod error_code_enum;
mod exit_code_enum;
mod exit_status_struct;
mod mount_builder_error_enum;
mod mount_error_enum;
mod mount_namespace_struct;
mod mount_options_mode_enum;
mod mount_source_enum;
mod mount_struct;
mod process_exit_status_struct;
