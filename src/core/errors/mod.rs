// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Runtime errors.

// From dependency library

// From standard library

// From this library
pub use cache_error_enum::CacheError;
pub use fs_tab_diff_error_enum::FsTabDiffError;
pub use fs_tab_diff_iter_error_enum::FsTabDiffIterError;
pub use fs_tab_entry_builder_error_enum::FsTabEntryBuilderError;
pub use fs_tab_entry_error_enum::FsTabEntryError;
pub use fs_tab_error_enum::FsTabError;
pub use fs_tab_iter_error_enum::FsTabIterError;
pub use gen_iterator_error_enum::GenIteratorError;
pub use mount_info_child_iter_error_enum::MountInfoChildIterError;
pub use mount_info_diff_error_enum::MountInfoDiffError;
pub use mount_info_diff_iter_error_enum::MountInfoDiffIterError;
pub use mount_info_entry_error_enum::MountInfoEntryError;
pub use mount_info_error_enum::MountInfoError;
pub use mount_info_iter_error_enum::MountInfoIterError;
pub use parser_error_enum::ParserError;
pub use swaps_diff_error_enum::SwapsDiffError;
pub use swaps_diff_iter_error_enum::SwapsDiffIterError;
pub use swaps_entry_error_enum::SwapsEntryError;
pub use swaps_error_enum::SwapsError;
pub use swaps_iter_error_enum::SwapsIterError;
pub use utab_entry_builder_error_enum::UTabEntryBuilderError;
pub use utab_entry_error_enum::UTabEntryError;
pub use utab_error_enum::UTabError;
pub use utab_iter_error_enum::UTabIterError;

mod cache_error_enum;
mod fs_tab_diff_error_enum;
mod fs_tab_diff_iter_error_enum;
mod fs_tab_entry_builder_error_enum;
mod fs_tab_entry_error_enum;
mod fs_tab_error_enum;
mod fs_tab_iter_error_enum;
mod gen_iterator_error_enum;
mod mount_info_child_iter_error_enum;
mod mount_info_diff_error_enum;
mod mount_info_diff_iter_error_enum;
mod mount_info_entry_error_enum;
mod mount_info_error_enum;
mod mount_info_iter_error_enum;
mod parser_error_enum;
mod swaps_diff_error_enum;
mod swaps_diff_iter_error_enum;
mod swaps_entry_error_enum;
mod swaps_error_enum;
mod swaps_iter_error_enum;
mod utab_entry_builder_error_enum;
mod utab_entry_error_enum;
mod utab_error_enum;
mod utab_iter_error_enum;
