// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use enum_iterator::Sequence;

// From standard library

// From this library

/// Mount flags.
///
/// Used to configure the type of operation performed by [mount(2)](https://www.man7.org/linux/man-pages/man2/mount.2.html).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Sequence)]
#[repr(u64)]
#[non_exhaustive]
pub enum MountFlag {
    /// Remount the file system.
    Remount = libc::MS_REMOUNT,

    /// Create a bind mount.
    Bind = libc::MS_BIND,

    /// Move an existing mount to a new location.
    Move = libc::MS_MOVE,

    /// Do not allow writing to the file system while it is mounted. This cannot be overridden by
    /// [`ioctl`](https://www.man7.org/linux/man-pages/man2/ioctl.2.html).
    ReadOnly = libc::MS_RDONLY,

    /// Used in conjunction with `Bind` to create a recursive bind mount. In conjunction with the
    /// propagation type flags, it recursively changes the propagation type of all mount points in
    /// a subtree.
    Recursive = libc::MS_REC,

    /// Set the propagation type of this mount point to `shared`. Mount and unmount events on sub mount
    /// points will propagate to its peers.
    Shared = libc::MS_SHARED,

    /// Set the propagation type of this mount point to `private`. Mount and unmount events do not
    /// propagate into or out of this mount point.
    Private = libc::MS_PRIVATE,

    /// If this is a `shared` mount point, member of a peer group that contains other members,
    /// convert it to a `slave` mount point.
    ///
    /// If this is a `shared` mount point, member of a peer group that contains no other members,
    /// convert it to a `private` mount point.
    ///
    /// Otherwise, the propagation type of the mount point is left unchanged.
    Slave = libc::MS_SLAVE,

    /// Make this mount point `unbindable`, i.e. a `private` mount point that can't be bind
    /// mounted.
    Unbindable = libc::MS_UNBINDABLE,

    /// Combines the variants `Shared`, `Private` , `Slave`, and `Unbindable`.
    Propagation = (libc::MS_SHARED | libc::MS_SLAVE | libc::MS_UNBINDABLE | libc::MS_PRIVATE),

    /// Permit mandatory locking on files while the file system is mounted.
    MandatoryLocking = libc::MS_MANDLOCK,

    /// Do not allow access to device special files while the file system is mounted.
    NoDeviceAccess = libc::MS_NODEV,

    /// Do not allow files in the file system to be executed while it is mounted.
    NoExecute = libc::MS_NOEXEC,

    /// Ignore Set-User-ID and Set-Group-ID permissions on files in the file system while it is
    /// mounted.
    NoSuid = libc::MS_NOSUID,

    /// All writes to the file system, while it is mounted, will be synchronous  (i.e. synchronizes
    /// data to disk before each write completes, rather than holding it in the buffer cache).
    Synchronous = libc::MS_SYNCHRONOUS,

    /// Make directory changes synchronous while the file system is mounted (i.e. synchronize data
    /// to disk before each write completes, rather than holding it in the buffer cache).
    SynchronizeDirectories = libc::MS_DIRSYNC,

    /// Reduce on-disk updates of inode timestamps (atime, mtime, ctime) by maintaining these
    /// changes in memory only.
    LazyTime = libc::MS_LAZYTIME,

    /// Do not update access times for all files while the file system is mounted.
    NoUpdateAccessTime = libc::MS_NOATIME,

    /// Do not update the access times of directories when they are accessed while the file system
    /// is mounted. This flag provides a subset of the functionality offered by
    /// `NoUpdateAccessTime` (i.e. `NoUpdateAccessTime` implies `NoUpdateDirectoryAccessTime`).
    NoUpdateDirectoryAccessTime = libc::MS_NODIRATIME,

    /// Combines the variants: `NoExecute`, `NoSuid`, and `NoDeviceAccess`.
    Secure = (libc::MS_NOEXEC | libc::MS_NOSUID | libc::MS_NODEV),

    /// Combines the variants: `NoSuid`, and `NoDeviceAccess`.
    OwnerSecure = (libc::MS_NOSUID | libc::MS_NODEV),

    /// When a file on this file system is accessed, update the file's last access time (atime)
    /// only if the current value of atime is less than or equal to the file's last modification
    /// time (mtime), or last status change time (ctime).
    RelativeAcessTime = libc::MS_RELATIME,

    /// Suppress certain printk() warning messages in the kernel log.
    Silent = libc::MS_SILENT,

    ///Always update the last access time (atime) when files are accessed while the file system is
    ///mounted.
    StrictUpdateAccessTime = libc::MS_STRICTATIME,

    // Sync = libc::MS_SYNC,
    // Async = libc::MS_ASYNC,
    Active = libc::MS_ACTIVE,
    NoUser = libc::MS_NOUSER,

    // Taken from https://www.gnu.org/software/libc/manual/html_node/Mount_002dUnmount_002dRemount.html
    /// This multibit field contains a magic number. If it does not have the value `MagicValue`,
    /// `mount` assumes all the following bits are zero and the *data* argument is a null string,
    /// regardless of their actual values.
    MagicMask = libc::MS_MGC_MSK,

    /// Magic flag number mask.
    MagicValue = libc::MS_MGC_VAL,

    /// Require usage of [POSIX Access Control
    /// Lists](https://www.man7.org/linux/man-pages/man5/acl.5.html) while the file system is
    /// mounted.
    PosixAcl = libc::MS_POSIXACL,

    /// Flags that can be altered by `Remount`. Combines the variants `ReadOnly`, `Synchronous`,
    /// `MandatoryLocking`, `IVersion`, and `LazyTime`.
    RemountMask = libc::MS_RMT_MASK,

    /// Update inodes' `I_version` field.
    IVersion = libc::MS_I_VERSION,

    /// Perform a `kern_mount` call.
    KernelMount = libc::MS_KERNMOUNT,
    // Invalidate = libc::MS_INVALIDATE,
}
