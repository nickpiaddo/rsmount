// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use num_enum::{IntoPrimitive, TryFromPrimitive};

// From standard library

// From this library

/// Defines how to combine options from `fstab` and `mountinfo` with options provided by a user.
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum MountOptionsMode {
    /// Combination of `ReplaceOptions`, `ForceFstabOptions`, and `ReadFromFstab`.
    NonRootUser = libmount::MNT_OMODE_USER as i32,

    /// Combination of `PrependOptions`, `ReadFromFstab`, and `ReadFromMountinfo`.
    Auto = libmount::MNT_OMODE_AUTO as i32,

    /// Do not read `fstab` or `mountinfo` at all.
    NoReadFromFstab = libmount::MNT_OMODE_NOTAB as i32,

    /// Always read `fstab` options.
    ForceFstabOptions = libmount::MNT_OMODE_FORCE as i32,

    /// Read from `fstab`.
    ReadFromFstab = libmount::MNT_OMODE_FSTAB as i32,

    /// Read from `mountinfo` if `fstab` is not enabled, or failed.
    ReadFromMountinfo = libmount::MNT_OMODE_MTAB as i32,

    /// Ignore `fstab` options.
    IgnoreOptions = libmount::MNT_OMODE_IGNORE as i32,

    /// Append options to existing `fstab` options.
    AppendOptions = libmount::MNT_OMODE_APPEND as i32,

    /// Prepend options to existing `fstab` options.
    PrependOptions = libmount::MNT_OMODE_PREPEND as i32,

    /// Replace existing options with those from `fstab`/`mountinfo`.
    ReplaceOptions = libmount::MNT_OMODE_REPLACE as i32,
}
