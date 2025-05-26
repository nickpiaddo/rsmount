// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use num_enum::{IntoPrimitive, TryFromPrimitive};

// From standard library

// From this library

#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum OptionFilter {
    // FIXME find a better explanation Options opposite to mount option default values (e.g `noatime`, `suid`).
    Negated = libmount::MNT_INVERT,

    /// Options prefixed with `x-` or `X-`.
    /// - `X-mount.mkdir[=mode]`: Allow to make a target directory (mount point) if it does not
    ///   exist yet,
    /// - `X-mount.auto-fstypes=list`: Specifies allowed or forbidden file system types for
    ///   automatic filesystem detection,
    /// - etc.
    ///
    /// (for more information see the [`mount` command's list of file system independent mount
    /// options](https://www.man7.org/linux/man-pages/man8/mount.8.html#FILESYSTEM-INDEPENDENT_MOUNT_OPTIONS))
    Prefixed = libmount::MNT_PREFIX,

    /// Options not reflected in `/proc/self/mountinfo`.
    NotInMountInfo = libmount::MNT_NOMTAB,

    /// Options not used by mount helpers.
    NotForMountHelpers = libmount::MNT_NOHLPS,

    #[cfg(mount = "v2_39")]
    /// Options matching the *mountflags* parameter of the [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html):
    /// - `sync`: `MS_SYNCHRONOUS` make writes on this file system synchronous,
    /// - `lazytime`: `MS_LAZYTIME` reduce on-disk updates of inode timestamps (atime, mtime, ctime) by
    ///   maintaining these changes in memory only,
    /// - etc.
    ///
    /// (for an exhaustive list, see the [`mount`
    /// command](https://www.man7.org/linux/man-pages/man8/mount.8.html#FILESYSTEM-INDEPENDENT_MOUNT_OPTIONS)
    /// and [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html) manpages)
    FsIo = libmount::MNT_SUPERBLOCK,
}
