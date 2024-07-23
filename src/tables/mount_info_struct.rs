// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library
use crate::core::entries::FsTabEntry;
use crate::core::entries::MountInfoEntry;
use crate::core::errors::MountInfoError;
use crate::declare_tab;
use crate::mount_info_shared_methods;

declare_tab!(
    MountInfo,
    r##"
An in-memory representation of `/proc/self/mountinfo`.

# `/proc/self/mountinfo`

The `/proc/self/mountinfo` file contains information about mount points in a process' mount
namespace; mount namespaces isolate the list of mount points seen by the processes in a
namespace. To put it another way, each mount namespace has its own list of mount points,
meaning that processes in different namespaces see, and are able to manipulate different views
of the system's directory tree.

# `mountinfo` file format

The `/proc/self/mountinfo` file supplies various pieces of information that are missing from the (older)
`/proc/pid/mounts` file (e.g. mount points' propagation state, the root of bind mounts, an
identifier for each mount point and its parent), and fixes various other problems with that file
(e.g. non-extensibility, failure to distinguish *per-mount* from *per-filesystem* options).

You will find below a sample `/proc/self/mountinfo` file extracted from an Alpine Linux virtual
machine.

```text
21 26 0:20 / /sys rw,nosuid,nodev,noexec,relatime - sysfs sysfs rw
22 26 0:5 / /dev rw,nosuid,noexec,relatime - devtmpfs devtmpfs rw,size=10240k,nr_inodes=26890,mode=755,inode64
23 26 0:21 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw
24 22 0:22 / /dev/pts rw,nosuid,noexec,relatime - devpts devpts rw,gid=5,mode=620,ptmxmode=000
25 22 0:23 / /dev/shm rw,nosuid,nodev,noexec,relatime - tmpfs shm rw,inode64
26 1 8:3 / / rw,relatime - ext4 /dev/sda3 rw
27 26 0:24 / /run rw,nosuid,nodev - tmpfs tmpfs rw,size=45148k,nr_inodes=819200,mode=755,inode64
28 22 0:19 / /dev/mqueue rw,nosuid,nodev,noexec,relatime - mqueue mqueue rw
29 21 0:6 / /sys/kernel/security rw,nosuid,nodev,noexec,relatime - securityfs securityfs rw
30 21 0:7 / /sys/kernel/debug rw,nosuid,nodev,noexec,relatime - debugfs debugfs rw
31 21 0:25 / /sys/fs/pstore rw,nosuid,nodev,noexec,relatime - pstore pstore rw
32 30 0:12 / /sys/kernel/debug/tracing rw,nosuid,nodev,noexec,relatime - tracefs tracefs rw
34 26 8:1 / /boot rw,relatime - ext4 /dev/sda1 rw
35 26 0:27 / /tmp rw,nosuid,nodev,relatime - tmpfs tmpfs rw,inode64
```

The table shown above has an 11-column structure, where each column represents a specific parameter.

The following example will help explain each column's role:

```text
36 35 98:0 /mnt1 /mnt2 rw,noatime master:1 - ext3 /dev/root rw,errors=continue
(1)(2)(3)   (4)   (5)      (6)      (7)   (8) (9)   (10)         (11)
```
- `(1)` **Mount ID**: a unique integer identifying the mount.
- `(2)` **Parent ID**: a unique integer identifying the parent of the mount point.<br> For
example, in the table above (line 5), the parent ID column in the description of `/dev/shm`
contains the mount ID of `/dev` (line 2).
- `(3)` **Device ID**: a device ID made of two integers separated by a `:`.<br> Those integers are
respectively:
    - the ***major***: identifying a device class,
    - the ***minor***: identifying a specific instance of a device in that class.
- `(4)` **Root**: the pathname of the directory a process sees as its root directory.
- `(5)` **Mount Point**: the directory on which the device is mounted, expressed relative to
the process's root directory.<br> From the example above, the device described has `/mnt1` as its root
directory, and `/mnt2` as its mount point. Thus, its absolute pathname is `/mnt1/mnt2`.
- `(6)` **Options**: a comma-separated list of [filesystem-independent mount
options](https://www.man7.org/linux/man-pages/man8/mount.8.html#FILESYSTEM-INDEPENDENT_MOUNT_OPTIONS).
- `(7)` **Optional Fields**: zero or more fields of the form ***tag\[:value]***, who describe a
mount point's ***propagation type***.<br> Under the shared subtrees feature, each mount point
is marked with a *propagation type*, which determines whether operations creating and/or
removing mount points below a given mount point, in a specific namespace, are propagated to
other mount points in other namespaces.<br><br> There are four tags representing different
propagation types:
    - `shared:N`: a mount point with this tag shares mount, and unmount events with other
    members of its ***peer group***. When a mount point is added (or removed) below a `shared`
    mount point, changes will propagate to its peer group, so that a mount or unmount will also
    take place below each of the peer mount points.<br> Event propagation works both ways, from
    the mount point to its peers and vice-versa.<br>
    The peer group is identified by a unique integer `N`, automatically generated by the kernel.<br>
    All mount points in the same peer group will show the same group ID. These IDs
    are assigned starting at `1`, and may be recycled when a peer group ceases to have any
    member.
    - `no tag`: this is the converse of a shared mount point. A mount point with no tag in its
    optional field does neither propagate events to, nor receive propagation events
    from peers.
    - `master:N`: this propagation type sits midway between shared and private. A slave mount
    has as master a shared peer group with ID `N`, whose members propagate mount and unmount
    events to the slave mount. However, the slave mount does not propagate events to the
    master peer group.
    - `propagate_from:N`: a mount point with this tag is a slave which receives mount/unmount
    propagation events from a shared peer group with ID `N`. This tag will always appear in
    conjunction with the `master:N` tag.<br> Here, `N` is the closest dominant peer group under
    the process's root directory. If `N` is the immediate master of the mount point, or if
    there is no dominant peer group under the same root, then only the `master:N` field is
    present and not the `propagate_from:N` field.
    - `unbindable`: a mount point with this tag is *unbindable*. Like a private mount point, this mount
    point does neither propagate to, nor receive events from peers. In addition, this mount point can't be
    the source for a bind mount operation.
- `(8)` **Separator**: a single hyphen marking the end of the optional fields.
- `(9)` **File System Type**: the type of file system the device uses (e.g. `ext4`, `tmpfs`, etc.).
- `(10)` **Mount Source**: filesystem-specific information or `none`.
- `(11)` **Super Options**: list of comma-separated options specific to a particular file system type.
    "##
);

mount_info_shared_methods!(MountInfo, MountInfoEntry, MountInfoError);

impl MountInfo {
    /// Parses the `/proc/self/mountinfo` and `/run/mount/utab` files, then appends the entries it
    /// collects to this `MountInfo`.
    pub fn import_mountinfo(&mut self) -> Result<(), MountInfoError> {
        log::debug!("MountInfo::import_mountinfo import entries from /proc/self/mountinfo and /run/mount/utab");

        unsafe {
            match libmount::mnt_table_parse_mtab(self.inner, std::ptr::null()) {
                0 => {
                    log::debug!(
                        "MountInfo::import_mountinfo imported entries from /proc/self/mountinfo and /run/mount/utab"
                    );

                    Ok(())
                }
                code => {
                    let err_msg =
                        "failed to import entries from /proc/self/mountinfo and /run/mount/utab"
                            .to_owned();
                    log::debug!("MountInfo::import_mountinfo {}. libmount::mnt_table_parse_mtab returned error code: {:?}", err_msg, code);

                    Err(MountInfoError::Import(err_msg))
                }
            }
        }
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`MountInfoEntry`] with a device matching
    /// the given `device_number`.
    ///
    /// **Note:** `0` is a valid device number for root pseudo-filesystems (e.g `tmpfs`).
    fn lookup_device<'a>(
        table: &mut Self,
        direction: Direction,
        device_number: u64,
    ) -> Option<&'a MountInfoEntry> {
        log::debug!(
            "MountInfo::lookup_device searching {:?} for device numbered {:?}",
            direction,
            device_number
        );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        unsafe {
            ptr.write(libmount::mnt_table_find_devno(
                table.inner,
                device_number,
                direction as i32,
            ))
        };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("MountInfo::lookup_device found no device with number {:?} while searching {:?}. libmount::mnt_table_find_devno returned a NULL pointer", device_number, direction);

                None
            }
            ptr => {
                log::debug!(
                    "MountInfo::lookup_device found entry for device number {:?}",
                    device_number
                );

                let entry = owning_ref_from_ptr!(table, MountInfoEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **top** to **bottom**, and returns the first [`MountInfoEntry`] with a
    /// device matching the given `device_number`.
    ///
    /// **Note:** `0` is a valid device number for root pseudo-filesystems (e.g `tmpfs`).
    pub fn find_device(&mut self, device_number: u64) -> Option<&MountInfoEntry> {
        log::debug!("MountInfo::find_device searching from top to bottom for entry matching device number {:?}", device_number);

        Self::lookup_device(self, Direction::Forward, device_number)
    }

    /// Searches the table from **bottom** to **top**, and returns the first [`MountInfoEntry`] with a
    /// device matching the given `device_number`.
    ///
    /// **Note:** `0` is a valid device number for root pseudo-filesystems (e.g `tmpfs`).
    pub fn find_back_device(&mut self, device_number: u64) -> Option<&MountInfoEntry> {
        log::debug!("MountInfo::find_back_device searching from bottom to top for entry matching device number {:?}", device_number);

        Self::lookup_device(self, Direction::Backward, device_number)
    }

    /// Removes the duplicate entries in this table keeping the first occurrence of an
    /// entry for which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:**
    /// - this method preserves the index order of the entries in the table.
    /// - this method preserves the Parent ID -> Mount ID relationship between entries.
    pub fn distinct_first_by<F>(&mut self, cmp: F) -> Result<(), MountInfoError>
    where
        F: FnMut(&MountInfoEntry, &MountInfoEntry) -> Ordering,
    {
        log::debug!(
            "MountInfo::distinct_first_by merging matching entries to the first occurrence"
        );

        Self::filter_by(
            self,
            libmount::MNT_UNIQ_FORWARD | libmount::MNT_UNIQ_KEEPTREE,
            cmp,
        )
    }

    /// Removes the duplicate entries in this table keeping the last occurrence of an
    /// entry for which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:**
    /// - this method preserves the index order of the entries in the table.
    /// - this method preserves the Parent ID -> Mount ID relationship between entries.
    pub fn distinct_last_by<F>(&mut self, cmp: F) -> Result<(), MountInfoError>
    where
        F: FnMut(&MountInfoEntry, &MountInfoEntry) -> Ordering,
    {
        log::debug!("MountInfo::distinct_last_by merging matching entries to the last occurrence");

        Self::filter_by(self, libmount::MNT_UNIQ_KEEPTREE, cmp)
    }

    /// Returns the root file system table entry.
    ///
    /// This function uses the parent ID from the `mountinfo` file to determine the root file
    /// system (i.e. the file system with the smallest ID, missing a parent ID). The function
    /// is designed mostly for applications where it is necessary to sort mount points by IDs to
    /// get a tree of mount points (e.g. the default output of the
    /// [`findmnt`](https://www.man7.org/linux/man-pages/man8/findmnt.8.html) command).
    pub fn root(&self) -> Option<&MountInfoEntry> {
        log::debug!("MountInfo::root getting entry matching file system root");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        let result = unsafe { libmount::mnt_table_get_root_fs(self.inner, ptr.as_mut_ptr()) };
        match result {
            0 => {
                log::debug!("MountInfo::root got entry matching file system root");

                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(self, MountInfoEntry, ptr);

                Some(entry)
            }
            code => {
                log::debug!("MountInfo::root failed to get entry matching file system root. libmount::mnt_table_get_root_fs returned error code: {:?}", code);

                None
            }
        }
    }

    //---- BEGIN predicates

    /// Returns `true` if the provided  `entry` matches an element contained in this table.
    /// The function compares the `source`, `target`, and `root` fields of the function
    /// parameter against those of each entry in this object.
    ///
    ///
    /// **Note:** the `source`, and `target` fields are canonicalized if a
    /// [`Cache`](crate::core::cache::Cache) is set for this object.
    ///
    /// **Note:** swap partitions are ignored.
    ///
    /// **Warning:** on `autofs` mount points, canonicalizing the `target` field may trigger
    /// an automount.
    pub fn is_mounted(&self, entry: &FsTabEntry) -> bool {
        let state = unsafe { libmount::mnt_table_is_fs_mounted(self.inner, entry.inner) == 1 };
        log::debug!("FsTab::is_mounted value: {:?}", state);

        state
    }

    //---- END predicates
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn mount_info_can_import_mountinfo_file() -> crate::Result<()> {
        let mut mount_info = MountInfo::new()?;

        mount_info.import_mountinfo()?;

        Ok(())
    }
}
