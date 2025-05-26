// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use num_enum::{IntoPrimitive, TryFromPrimitive};

// From standard library

// From this library

/// `libmount`'s private error codes.
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum ErrorCode {
    /// `libblkid` detected multiple file systems for the same device.
    FsCollision = libmount::MNT_ERR_AMBIFS,

    /// Failed to set the propagation type of mount and unmount events between namespaces.
    ApplyFlags = libmount::MNT_ERR_APPLYFLAGS,

    #[cfg(mount = "v2_39")]
    /// File system mounted, but subsequent `X-mount.mode=` [chmod(2)](https://www.man7.org/linux/man-pages/man2/chmod.2.html) failed
    ChangeMode = libmount::MNT_ERR_CHMOD,

    #[cfg(mount = "v2_39")]
    /// File system mounted, but subsequent `X-mount.owner=/X-mount.group=` [lchown(2)](https://www.man7.org/linux/man-pages/man2/lchown.2.html) failed
    ChangeOwner = libmount::MNT_ERR_CHOWN,

    #[cfg(mount = "v2_39")]
    /// File system mounted, but subsequent `X-mount.idmap=` failed.
    IdMap = libmount::MNT_ERR_IDMAP,

    /// Failed to lock `utab`.
    Lock = libmount::MNT_ERR_LOCK,

    /// Failed to setup a loop device.
    LoopDevice = libmount::MNT_ERR_LOOPDEV,

    /// Detected overlapping loop devices that can not be reused.
    LoopDeviceOverlap = libmount::MNT_ERR_LOOPOVERLAP,

    /// Failed to parse/use userspace mount options.
    UserspaceMountOptions = libmount::MNT_ERR_MOUNTOPT,

    /// Failed to switch namespace.
    NamespaceSwitch = libmount::MNT_ERR_NAMESPACE,

    /// Missing required entry in `/etc/fstab`.
    FsTabMissingEntry = libmount::MNT_ERR_NOFSTAB,

    /// Failed to detect file system type.
    NoFsType = libmount::MNT_ERR_NOFSTYPE,

    /// Required mount source was undefined.
    UndefinedMountSource = libmount::MNT_ERR_NOSOURCE,

    #[cfg(mount = "v2_39")]
    /// File system mounted, but `--onlyonce` specified
    OnlyOnce = libmount::MNT_ERR_ONLYONCE,
}
