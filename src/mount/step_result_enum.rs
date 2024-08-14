// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library
use crate::core::entries::FsTabEntry;
use crate::core::entries::MountInfoEntry;

/// Result of a mount/remount/umount operation on an entry in `/etc/fstab` or
/// `/proc/self/mountinfo`.
#[derive(Debug)]
#[non_exhaustive]
pub enum StepResult {
    /// Already mounted an entry in `/etc/fstab`.
    MountAlreadyDone(FsTabEntry),

    /// Skipped an entry in `/etc/fstab` while sequentially mounting entries.
    MountSkipped(FsTabEntry),

    /// Successfully mounted an entry in `/etc/fstab`.
    MountSuccess(FsTabEntry),

    /// Failed to mount an entry in `/etc/fstab`.
    MountFail(FsTabEntry),

    /// Already remounted an entry in `/proc/self/mouninfo`.
    ReMountAlreadyDone(MountInfoEntry),

    /// Skipped an entry in `/proc/self/mountinfo` while sequentially remounting entries.
    ReMountSkipped(MountInfoEntry),

    /// Successfully remounted an entry in `/proc/self/mountinfo`.
    ReMountSuccess(MountInfoEntry),

    /// Failed to remount an entry in `/proc/self/mountinfo`.
    ReMountFail(MountInfoEntry),

    /// Already unmounted an entry in `/proc/self/mouninfo`.
    UMountAlreadyDone(MountInfoEntry),

    /// Skipped an entry in `/proc/self/mountinfo` while sequentially unmounting entries.
    UMountSkipped(MountInfoEntry),

    /// Successfully unmounted an entry in `/proc/self/mountinfo`.
    UMountSuccess(MountInfoEntry),

    /// Failed to unmount an entry in `/proc/self/mountinfo`.
    UMountFail(MountInfoEntry),
}
