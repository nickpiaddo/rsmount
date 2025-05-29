// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::cmp::Ordering;
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::Index;
use std::path::Path;

// From this library
use crate::core::cache::Cache;
use crate::core::device::Source;

use crate::core::entries::FsTabEntry;
use crate::core::entries::MountInfoEntry;

use crate::core::errors::MountInfoChildIterError;
use crate::core::errors::MountInfoError;
use crate::core::errors::MountInfoIterError;

use crate::core::iter::Direction;
use crate::core::iter::GenIterator;
use crate::core::iter::MountInfoChildIter;
use crate::core::iter::MountInfoIter;
use crate::core::iter::MountInfoOvermountIter;

use crate::owning_ref_from_ptr;

use crate::tables::GcItem;
use crate::tables::MountOption;
use crate::tables::ParserFlow;

use crate::ffi_utils;

/// An in-memory representation of `/proc/self/mountinfo`.
///
/// # `/proc/self/mountinfo`
///
/// The `/proc/self/mountinfo` file contains information about mount points in a process' mount
/// namespace; mount namespaces isolate the list of mount points seen by the processes in a
/// namespace. To put it another way, each mount namespace has its own list of mount points,
/// meaning that processes in different namespaces see, and are able to manipulate different views
/// of the system's directory tree.
///
/// # Load `/proc/self/mountinfo` to memory
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::tables::MountInfo;
/// fn main() -> rsmount::Result<()> {
///     let mut mount_info = MountInfo::new()?;
///
///     mount_info.import_mountinfo()?;
///     println!("{mount_info}");
///     // Example output
///     //
///     // 21 26 0:20 / /sys rw,nosuid,nodev,noexec,relatime - sysfs sysfs rw
///     // 22 26 0:5 / /dev rw,nosuid,noexec,relatime - devtmpfs devtmpfs rw,size=10240k,nr_inodes=26890,mode=755,inode64
///     // 23 26 0:21 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw
///     // 24 22 0:22 / /dev/pts rw,nosuid,noexec,relatime - devpts devpts rw,gid=5,mode=620,ptmxmode=000
///     // 25 22 0:23 / /dev/shm rw,nosuid,nodev,noexec,relatime - tmpfs shm rw,inode64
///     // 26 1 8:3 / / rw,relatime - ext4 /dev/sda3 rw
///     // ...snip...
///
///     Ok(())
/// }
/// ```
///
/// # `mountinfo` file format
///
/// The `/proc/self/mountinfo` file supplies various pieces of information that are missing from the (older)
/// `/proc/pid/mounts` file (e.g. mount points' propagation state, the root of bind mounts, an
/// identifier for each mount point and its parent), and fixes various other problems with that file
/// (e.g. non-extensibility, failure to distinguish *per-mount* from *per-filesystem* options).
///
/// You will find below a sample `/proc/self/mountinfo` file extracted from an Alpine Linux virtual
/// machine.
///
/// ```text
/// 21 26 0:20 / /sys rw,nosuid,nodev,noexec,relatime - sysfs sysfs rw
/// 22 26 0:5 / /dev rw,nosuid,noexec,relatime - devtmpfs devtmpfs rw,size=10240k,nr_inodes=26890,mode=755,inode64
/// 23 26 0:21 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw
/// 24 22 0:22 / /dev/pts rw,nosuid,noexec,relatime - devpts devpts rw,gid=5,mode=620,ptmxmode=000
/// 25 22 0:23 / /dev/shm rw,nosuid,nodev,noexec,relatime - tmpfs shm rw,inode64
/// 26 1 8:3 / / rw,relatime - ext4 /dev/sda3 rw
/// 27 26 0:24 / /run rw,nosuid,nodev - tmpfs tmpfs rw,size=45148k,nr_inodes=819200,mode=755,inode64
/// 28 22 0:19 / /dev/mqueue rw,nosuid,nodev,noexec,relatime - mqueue mqueue rw
/// 29 21 0:6 / /sys/kernel/security rw,nosuid,nodev,noexec,relatime - securityfs securityfs rw
/// 30 21 0:7 / /sys/kernel/debug rw,nosuid,nodev,noexec,relatime - debugfs debugfs rw
/// 31 21 0:25 / /sys/fs/pstore rw,nosuid,nodev,noexec,relatime - pstore pstore rw
/// 32 30 0:12 / /sys/kernel/debug/tracing rw,nosuid,nodev,noexec,relatime - tracefs tracefs rw
/// 34 26 8:1 / /boot rw,relatime - ext4 /dev/sda1 rw
/// 35 26 0:27 / /tmp rw,nosuid,nodev,relatime - tmpfs tmpfs rw,inode64
/// ```
///
/// The table shown above has an 11-column structure, where each column represents a specific parameter.
///
/// The following example will help explain each column's role:
///
/// ```text
/// 36 35 98:0 /mnt1 /mnt2 rw,noatime master:1 - ext3 /dev/root rw,errors=continue
/// (1)(2)(3)   (4)   (5)      (6)      (7)   (8) (9)   (10)         (11)
/// ```
/// - `(1)` **Mount ID**: a unique integer identifying the mount.
/// - `(2)` **Parent ID**: a unique integer identifying the parent of the mount point.<br> For
///   example, in the table above (line 5), the parent ID column in the description of `/dev/shm`
///   contains the mount ID of `/dev` (line 2).
/// - `(3)` **Device ID**: a device ID made of two integers separated by a `:`.<br> Those integers are
///   respectively:
///       - the ***major***: identifying a device class,
///       - the ***minor***: identifying a specific instance of a device in that class.
/// - `(4)` **Root**: the pathname of the directory a process sees as its root directory.
/// - `(5)` **Mount Point**: the directory on which the device is mounted, expressed relative to
///   the process's root directory.<br> From the example above, the device described has `/mnt1` as its root
///   directory, and `/mnt2` as its mount point. Thus, its absolute pathname is `/mnt1/mnt2`.
/// - `(6)` **Options**: a comma-separated list of [filesystem-independent mount
///   options](https://www.man7.org/linux/man-pages/man8/mount.8.html#FILESYSTEM-INDEPENDENT_MOUNT_OPTIONS).
/// - `(7)` **Optional Fields**: zero or more fields of the form ***tag\[:value]***, who describe a
///   mount point's ***propagation type***.<br> Under the shared subtrees feature, each mount point
///   is marked with a *propagation type*, which determines whether operations creating and/or
///   removing mount points below a given mount point, in a specific namespace, are propagated to
///   other mount points in other namespaces.<br><br> There are four tags representing different
///   propagation types:
///       - `shared:N`: a mount point with this tag shares mount, and unmount events with other
///         members of its ***peer group***. When a mount point is added (or removed) below a `shared`
///         mount point, changes will propagate to its peer group, so that a mount or unmount will also
///         take place below each of the peer mount points.<br> Event propagation works both ways, from
///         the mount point to its peers and vice-versa.<br>
///         The peer group is identified by a unique integer `N`, automatically generated by the kernel.<br>
///         All mount points in the same peer group will show the same group ID. These IDs
///         are assigned starting at `1`, and may be recycled when a peer group ceases to have any
///         member.
///       - `no tag`: this is the converse of a shared mount point. A mount point with no tag in its
///         optional field does neither propagate events to, nor receive propagation events
///         from peers.
///       - `master:N`: this propagation type sits midway between shared and private. A slave mount
///         has as master a shared peer group with ID `N`, whose members propagate mount and unmount
///         events to the slave mount. However, the slave mount does not propagate events to the
///         master peer group.
///       - `propagate_from:N`: a mount point with this tag is a slave which receives mount/unmount
///         propagation events from a shared peer group with ID `N`. This tag will always appear in
///         conjunction with the `master:N` tag.<br> Here, `N` is the closest dominant peer group under
///         the process's root directory. If `N` is the immediate master of the mount point, or if
///         there is no dominant peer group under the same root, then only the `master:N` field is
///         present and not the `propagate_from:N` field.
///       - `unbindable`: a mount point with this tag is *unbindable*. Like a private mount point, this mount
///         point does neither propagate to, nor receive events from peers. In addition, this mount point can't be
///       the source for a bind mount operation.
/// - `(8)` **Separator**: a single hyphen marking the end of the optional fields.
/// - `(9)` **File System Type**: the type of file system the device uses (e.g. `ext4`, `tmpfs`, etc.).
/// - `(10)` **Mount Source**: filesystem-specific information or `none`.
/// - `(11)` **Super Options**: list of comma-separated options specific to a particular file system type.
#[derive(Debug)]
pub struct MountInfo {
    pub(crate) inner: *mut libmount::libmnt_table,
    pub(crate) gc: Vec<GcItem>,
}

impl Drop for MountInfo {
    fn drop(&mut self) {
        log::debug!("MountInfo::drop deallocating `MountInfo` instance");

        unsafe { libmount::mnt_unref_table(self.inner) };
        self.collect_garbage();
    }
}

impl AsRef<MountInfo> for MountInfo {
    #[inline]
    fn as_ref(&self) -> &MountInfo {
        self
    }
}

impl Index<usize> for MountInfo {
    type Output = MountInfoEntry;

    /// Performs the indexing (`container\[index]`) operation.
    fn index(&self, index: usize) -> &Self::Output {
        log::debug!("MountInfo::index getting item at index: {:?}", index);

        #[cold]
        #[inline(never)]
        #[track_caller]
        fn indexing_failed() -> ! {
            panic!("Index out of bounds");
        }

        let mut iter = MountInfoIter::new(self).unwrap();

        match iter.nth(index) {
            Some(item) => item,
            None => indexing_failed(),
        }
    }
}

impl MountInfo {
    #[doc(hidden)]
    /// Increments the instance's reference counter.
    #[allow(dead_code)]
    pub(crate) fn incr_ref_counter(&mut self) {
        unsafe { libmount::mnt_ref_table(self.inner) }
    }

    #[doc(hidden)]
    /// Decrements the instance's reference counter.
    #[allow(dead_code)]
    pub(crate) fn decr_ref_counter(&mut self) {
        unsafe { libmount::mnt_unref_table(self.inner) }
    }

    #[doc(hidden)]
    /// Creates a new instance from a `libmount::libmnt_table` pointer.
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_table) -> MountInfo {
        Self {
            inner: ptr,
            gc: vec![],
        }
    }

    #[doc(hidden)]
    /// Borrows an instance.
    #[allow(dead_code)]
    pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_table) -> MountInfo {
        let mut table = Self::from_ptr(ptr);
        // We are virtually ceding ownership of this table which will be automatically
        // deallocated once it is out of scope, incrementing its reference counter protects it from
        // being freed prematurely.
        table.incr_ref_counter();

        table
    }

    //---- BEGIN constructors

    /// Creates a new empty `MountInfo`.
    pub fn new() -> Result<MountInfo, MountInfoError> {
        log::debug!("MountInfo::new creating a new `MountInfo` instance");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();

        unsafe { ptr.write(libmount::mnt_new_table()) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to create a new `MountInfo`".to_owned();
                log::debug!(
                    "MountInfo::new {err_msg}. libmount::mnt_new_table returned a NULL pointer"
                );

                Err(MountInfoError::Creation(err_msg))
            }
            ptr => {
                log::debug!("MountInfo::new created a new `MountInfo` instance");
                let table = Self::from_ptr(ptr);

                Ok(table)
            }
        }
    }

    //---- END constructors

    //---- BEGIN getters

    /// Returns a reference to the [`Cache`] instance associated with this `MountInfo`.
    pub fn cache(&self) -> Option<&Cache> {
        log::debug!("MountInfo::cache getting associated path and tag cache");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_cache>::zeroed();

        unsafe { ptr.write(libmount::mnt_table_get_cache(self.inner)) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("MountInfo::cache failed to get associated path and tag cache. libmount::mnt_table_get_cache returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!("MountInfo::cache got associated path and tag cache");
                let cache = owning_ref_from_ptr!(self, Cache, ptr);

                Some(cache)
            }
        }
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

    /// Returns a reference to the first element of the `MountInfo`, or `None` if it is empty.
    pub fn first(&self) -> Option<&MountInfoEntry> {
        log::debug!("MountInfo::first getting reference to first table entry");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe { libmount::mnt_table_first_fs(self.inner, ptr.as_mut_ptr()) };

        match result {
            0 => {
                log::debug!("MountInfo::first got reference to first table entry");
                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(self, MountInfoEntry, ptr);

                Some(entry)
            }
            code => {
                log::debug!( "MountInfo::first failed to get reference to first table entry. libmount::mnt_table_first_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns a reference to the last element of the `MountInfo`, or `None` if it is empty.
    pub fn last(&self) -> Option<&MountInfoEntry> {
        log::debug!("MountInfo::last getting reference to last table entry");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe { libmount::mnt_table_last_fs(self.inner, ptr.as_mut_ptr()) };

        match result {
            0 => {
                log::debug!("MountInfo::last got reference to last table entry");
                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(self, MountInfoEntry, ptr);

                Some(entry)
            }
            code => {
                log::debug!( "MountInfo::last failed to get reference to last table entry. libmount::mnt_table_last_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns the index of a table entry.
    pub fn position(&self, entry: &MountInfoEntry) -> Option<usize> {
        log::debug!("MountInfo::position searching for an entry in the table");

        let result = unsafe { libmount::mnt_table_find_fs(self.inner, entry.inner) };

        match result {
            index if index > 0 => {
                log::debug!(
                    "MountInfo::position mount table contains entry at index: {:?}",
                    index
                );

                Some(index as usize)
            }
            code => {
                log::debug!( "MountInfo::position no matching entry in table: libmount::mnt_table_find_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns the number of entries in the table.
    pub fn len(&self) -> usize {
        let len = unsafe { libmount::mnt_table_get_nents(self.inner) };
        log::debug!("MountInfo::len value: {:?}", len);

        len as usize
    }

    /// Returns a reference to an element at `index`, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&MountInfoEntry> {
        log::debug!(
            "MountInfo::get_mut getting reference of item at index: {:?}",
            index
        );

        MountInfoIter::new(self)
            .ok()
            .and_then(|mut iter| iter.nth(index))
    }

    #[doc(hidden)]
    /// Searches forward/backward for the first entry in the `table` that satisfies the `predicate`
    /// depending on the [`Direction`] defined at the `iterator`'s creation.
    fn find_first_entry<'a, P>(
        table: &mut Self,
        iterator: *mut libmount::libmnt_iter,
        predicate: P,
    ) -> Option<&'a MountInfoEntry>
    where
        P: FnMut(&MountInfoEntry) -> bool,
    {
        #[doc(hidden)]
        /// Callback function called by the `libmount::mnt_table_find_next_fs` C-binding. The
        /// function searches for the first element that satisfies the predicate using the callback
        /// as a C-compatible wrapper around the closure.
        unsafe extern "C" fn callback<P>(
            entry_ptr: *mut libmount::libmnt_fs,
            predicate_fn_ptr: *mut libc::c_void,
        ) -> libc::c_int
        where
            P: FnMut(&MountInfoEntry) -> bool,
        {
            // Temporarily increments the pointer's reference counter which will be decremented when
            // `entry` goes out of scope. This prevents us from freeing the data structure while it is
            // still in use.
            let entry = MountInfoEntry::borrow_ptr(entry_ptr);

            // Rebuild the predicate closure from the c_void pointer passed as user data.
            let predicate_fn = &mut *(predicate_fn_ptr as *mut P);

            match predicate_fn(&entry) {
                true => 1,
                false => 0,
            }
        }

        // Moving the closure to the heap with `Box::new`, to live for some unknown period of time.
        // Then, call `Box::into_raw` on it, to get a raw pointer to the closure, and prevent the
        // memory it uses from being deallocated.
        let data = Box::into_raw(Box::new(predicate));

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe {
            libmount::mnt_table_find_next_fs(
                table.inner,
                iterator,
                Some(callback::<P>),
                data as *mut _,
                ptr.as_mut_ptr(),
            )
        };

        match result {
            0 => {
                // To ensure the closure is properly deallocated when this variable drops out
                // of scope.
                let _predicate = unsafe { Box::from_raw(data) };

                log::debug!(
                    "MountInfo::find_first_entry found first `MountInfoEntry` matching predicate"
                );
                let entry_ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(table, MountInfoEntry, entry_ptr);

                Some(entry)
            }
            code => {
                // To ensure the closure is properly deallocated when this variable drops out
                // of scope.
                let _predicate = unsafe { Box::from_raw(data) };

                let err_msg = "failed to find `MountInfoEntry` matching predicate".to_owned();
                log::debug!( "MountInfo::find_first_entry {err_msg}. libmount::mnt_table_find_next_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`MountInfoEntry`] that
    /// satisfies the `predicate`.
    ///
    /// # Panics
    ///
    /// Panics if it can not create a forward iterator.
    pub fn find_first<P>(&mut self, predicate: P) -> Option<&MountInfoEntry>
    where
        P: FnMut(&MountInfoEntry) -> bool,
    {
        log::debug!( "MountInfo::find_first finding first table entry matching predicate while iterating Forward");
        GenIterator::new(Direction::Forward)
            .ok()
            .and_then(|iterator| MountInfo::find_first_entry(self, iterator.inner, predicate))
    }

    /// Searches the table from **end** to **start**, and returns the first [`MountInfoEntry`] that
    /// satisfies the `predicate`.
    ///
    /// # Panics
    ///
    /// Panics if it can not create a backward iterator.
    pub fn find_back_first<P>(&mut self, predicate: P) -> Option<&MountInfoEntry>
    where
        P: FnMut(&MountInfoEntry) -> bool,
    {
        log::debug!( "MountInfo::find_back_first finding first table entry matching predicate while iterating Backward");
        GenIterator::new(Direction::Backward)
            .ok()
            .and_then(|iterator| MountInfo::find_first_entry(self, iterator.inner, predicate))
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`MountInfoEntry`] with fields matching the given
    /// source `path`.
    fn lookup_source<'a>(
        table: &mut Self,
        direction: Direction,
        source: &Source,
    ) -> Option<&'a MountInfoEntry> {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok()?;
        let source_ptr = if source.is_pseudo_fs() {
            // For pseudo file systems `libmount::mnt_table_find_source`
            // expects a NULL pointer path.
            std::ptr::null()
        } else {
            source_cstr.as_ptr()
        };

        log::debug!(
            "MountInfo::lookup_source searching {:?} for entry matching source {:?}",
            direction,
            source
        );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_table_find_source(
                table.inner,
                source_ptr,
                direction as i32,
            ))
        };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!(
                    "failed to find entry matching source {:?} while searching {:?}",
                    source, direction
                );
                log::debug!( "MountInfo::lookup_source {err_msg}. libmount::mnt_table_find_source returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
              "MountInfo::lookup_source found entry matching source {:?} while searching {:?}",
                                source,
                                direction
                            );

                let entry = owning_ref_from_ptr!(table, MountInfoEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`MountInfoEntry`] with
    /// a field matching the given `source`.
    ///
    /// By default, a `MountInfo` will perform a cursory search, looking for an entry with an exact
    /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`MountInfo::set_cache`].
    ///
    /// **Note:** providing an **empty source** is equivalent to searching for a source matching
    /// `"none"` (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate
    /// pseudo-filesystems).
    pub fn find_source(&mut self, source: &Source) -> Option<&MountInfoEntry> {
        let direction = Direction::Forward;
        log::debug!(
            "MountInfo::find_source searching {:?} for the first entry with a source matching {:?}",
            direction,
            source
        );

        Self::lookup_source(self, direction, source)
    }

    /// Searches the table from **end** to **start**, and returns the first [`MountInfoEntry`] with
    /// a field matching the given `source`.
    ///
    /// By default, a `MountInfo` will perform a cursory search, looking for an entry with an exact
    /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`MountInfo::set_cache`].
    ///
    /// **Note:** providing an **empty source** is equivalent to searching for a source matching
    /// `"none"` (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate
    /// pseudo-filesystems).
    pub fn find_back_source(&mut self, source: &Source) -> Option<&MountInfoEntry> {
        let direction = Direction::Backward;
        log::debug!(
              "MountInfo::find_back_source searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        source
                    );

        Self::lookup_source(self, direction, source)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`MountInfoEntry`] with fields matching the given source
    /// `path`.
    fn lookup_source_path<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
    ) -> Option<&'a MountInfoEntry> {
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok()?;
        let path_ptr = if path_cstr.is_empty() {
            // An empty path is the same as setting the `none` pseudo
            // file system as source, for which `libmount::mnt_table_find_srcpath`
            // expects a NULL pointer path.
            std::ptr::null()
        } else {
            path_cstr.as_ptr()
        };

        log::debug!(
            "MountInfo::lookup_source_path searching {:?} for entry matching source path {:?}",
            direction,
            path
        );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_table_find_srcpath(
                table.inner,
                path_ptr,
                direction as i32,
            ))
        };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!(
                    "failed to find entry matching source path {:?} while searching {:?}",
                    path, direction
                );
                log::debug!( "MountInfo::lookup_source_path {err_msg}. libmount::mnt_table_find_srcpath returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
              "MountInfo::lookup_source_path found entry matching source path {:?} while searching {:?}",
                                path,
                                direction
                            );

                let entry = owning_ref_from_ptr!(table, MountInfoEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`MountInfoEntry`] with
    /// a `source` field matching the given `path`.
    ///
    /// By default, a `MountInfo` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`MountInfo::set_cache`].
    ///
    /// **Note:** providing an **empty path** is equivalent to searching for a source matching
    /// `"none"` (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate
    /// pseudo-filesystems).
    pub fn find_source_path<T>(&mut self, path: T) -> Option<&MountInfoEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!(
              "MountInfo::find_source_path searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        path
                    );

        Self::lookup_source_path(self, direction, path)
    }

    /// Searches the table from **end** to **start**, and returns the first [`MountInfoEntry`] with
    /// a `source` field matching the given `path`.
    ///
    /// By default, a `MountInfo` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`MountInfo::set_cache`].
    ///
    /// **Note:** providing an **empty path** is equivalent to searching for a source matching
    /// `"none"` (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate
    /// pseudo-filesystems).
    pub fn find_back_source_path<T>(&mut self, path: T) -> Option<&MountInfoEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!(
     "MountInfo::find_back_source_path searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        path
                    );

        Self::lookup_source_path(self, direction, path)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`MountInfoEntry`] with a device matching the given
    /// `device_number`.
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

    /// Searches the table from **top** to **bottom**, and returns the first [`MountInfoEntry`]
    /// with a device matching the given `device_number`.
    ///
    /// **Note:** `0` is a valid device number for root pseudo-filesystems (e.g `tmpfs`).
    pub fn find_device(&mut self, device_number: u64) -> Option<&MountInfoEntry> {
        log::debug!("MountInfo::find_device searching from top to bottom for entry matching device number {:?}", device_number);

        Self::lookup_device(self, Direction::Forward, device_number)
    }

    /// Searches the table from **bottom** to **top**, and returns the first [`MountInfoEntry`]
    /// with a device matching the given `device_number`.
    ///
    /// **Note:** `0` is a valid device number for root pseudo-filesystems (e.g `tmpfs`).
    pub fn find_back_device(&mut self, device_number: u64) -> Option<&MountInfoEntry> {
        log::debug!("MountInfo::find_back_device searching from bottom to top for entry matching device number {:?}", device_number);

        Self::lookup_device(self, Direction::Backward, device_number)
    }

    /// Removes the duplicate entries in this table keeping the first occurrence of an entry for
    /// which the `cmp` function returns [`Ordering::Equal`].
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

    /// Removes the duplicate entries in this table keeping the last occurrence of an entry for
    /// which the `cmp` function returns [`Ordering::Equal`].
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

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`MountInfoEntry`] with fields matching the given target
    /// `path`.
    fn lookup_target<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
    ) -> Option<&'a MountInfoEntry> {
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok()?;
        log::debug!(
            "MountInfo::lookup_target searching {:?} for entry matching target {:?}",
            direction,
            path
        );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_table_find_target(
                table.inner,
                path_cstr.as_ptr(),
                direction as i32,
            ))
        };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!(
                    "failed to find entry matching target {:?} while searching {:?}",
                    path, direction
                );
                log::debug!( "MountInfo::lookup_target {err_msg}. libmount::mnt_table_find_target returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
 "MountInfo::lookup_target found entry matching target {:?} while searching {:?}",
                            path,
                            direction
                        );

                let entry = owning_ref_from_ptr!(table, MountInfoEntry, ptr);

                Some(entry)
            }
        }
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`MountInfoEntry`] with fields matching the given
    /// `mount_point`.
    fn lookup_mount_point<'a>(
        table: &mut Self,
        direction: Direction,
        mount_point: &Path,
    ) -> Option<&'a MountInfoEntry> {
        let mount_point_cstr = ffi_utils::as_ref_path_to_c_string(mount_point).ok()?;
        log::debug!(
            "MountInfo::lookup_mount_point searching {:?} for entry with mount point {:?}",
            direction,
            mount_point
        );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_table_find_mountpoint(
                table.inner,
                mount_point_cstr.as_ptr(),
                direction as i32,
            ))
        };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "MountInfo::lookup_mount_point found no entry with mount point {:?} while searching {:?}. libmount::mnt_table_find_mountpoint returned a NULL pointer", mount_point, direction);

                None
            }
            ptr => {
                log::debug!(
                    "MountInfo::lookup_mount_point found entry with  mount point {:?}",
                    mount_point
                );

                let entry = owning_ref_from_ptr!(table, MountInfoEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`MountInfoEntry`]
    /// matching the given `mount_point`.
    pub fn find_mount_point<T>(&mut self, mount_point: T) -> Option<&MountInfoEntry>
    where
        T: AsRef<Path>,
    {
        let mount_point = mount_point.as_ref();
        log::debug!( "MountInfo::find_mount_point searching table from top to bottom for entry matching mount point {:?}", mount_point);

        Self::lookup_mount_point(self, Direction::Forward, mount_point)
    }

    /// Searches the table from **end** to **start**, and returns the first [`MountInfoEntry`]
    /// matching the given `mount_point`.
    pub fn find_back_mount_point<T>(&mut self, mount_point: T) -> Option<&MountInfoEntry>
    where
        T: AsRef<Path>,
    {
        let mount_point = mount_point.as_ref();
        log::debug!( "MountInfo::find_back_mount_point searching table from bottom to top for entry matching mount point {:?}", mount_point);

        Self::lookup_mount_point(self, Direction::Backward, mount_point)
    }

    /// Searches the table from **start** to **end**, and returns the first [`MountInfoEntry`] with
    /// a `target` field matching the given `path`.
    ///
    /// By default, a `MountInfo` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`MountInfo::set_cache`].
    pub fn find_target<T>(&mut self, path: T) -> Option<&MountInfoEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!(
            "MountInfo::find_target searching {:?} for the first entry with a target matching {:?}",
            direction,
            path
        );

        Self::lookup_target(self, direction, path)
    }

    /// Searches the table from **end** to **start**, and returns the first [`MountInfoEntry`] with
    /// a `target` field matching the given `path`.
    ///
    /// By default, a `MountInfo` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`MountInfo::set_cache`].
    pub fn find_back_target<T>(&mut self, path: T) -> Option<&MountInfoEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!(
 "MountInfo::find_back_target searching {:?} for the first entry with a target matching {:?}",
                    direction,
                    path
                );

        Self::lookup_target(self, direction, path)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`MountInfoEntry`] with fields matching the given the
    /// combination of `path` and `option_name` with `option_value`.
    fn lookup_target_with_options<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
        option_name: &str,
        option_value: Option<&str>,
    ) -> Option<&'a MountInfoEntry> {
        // Represent the missing value by an empty string.
        let option_value = option_value.map_or_else(String::new, |value| value.to_owned());

        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok()?;
        let opt_name_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;
        let opt_value_cstr = ffi_utils::as_ref_str_to_c_string(&option_value).ok()?;

        // For missing values `mnt_table_find_target_with_option` takes a NULL pointer.
        let opt_value_ptr = if opt_value_cstr.is_empty() {
            std::ptr::null()
        } else {
            opt_value_cstr.as_ptr()
        };

        // Format option value for log::debug!
        let opt_value = if option_value.is_empty() {
            option_value
        } else {
            format!(" with value {:?}", option_value)
        };

        log::debug!(
 "MountInfo::lookup_target_with_options searching {:?} for entry matching the combination of path {:?} and option {:?}{}",
                    direction,
                    path,
                    option_name,
                    opt_value
                );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_table_find_target_with_option(
                table.inner,
                path_cstr.as_ptr(),
                opt_name_cstr.as_ptr(),
                opt_value_ptr,
                direction as i32,
            ))
        };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!("found no entry matching the combination of path {:?} and option {:?}{} while searching {:?}", path, option_name, opt_value, direction );
                log::debug!( "MountInfo::lookup_target_with_options {err_msg}. libmount::mnt_table_find_target_with_option returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
             "MountInfo::lookup_target_with_options found entry matching the combination of path {:?} and option {:?}{}",
                            path,
                            option_name,
                            opt_value
                        );

                let entry = owning_ref_from_ptr!(table, MountInfoEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Performs a cursory search in the table, from **start** to **end**, and returns the first
    /// [`MountInfoEntry`] with fields matching the given combination of `path` and `option_name`.
    pub fn find_target_with_option<P, T>(
        &mut self,
        path: P,
        option_name: T,
    ) -> Option<&MountInfoEntry>
    where
        P: AsRef<Path>,
        T: AsRef<str>,
    {
        let path = path.as_ref();
        let option_name = option_name.as_ref();
        let direction = Direction::Forward;
        log::debug!( "MountInfo::find_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option_name);

        Self::lookup_target_with_options(self, direction, path, option_name, None)
    }

    /// Performs a cursory search in the table, from **end** to **start**, and returns the first
    /// [`MountInfoEntry`] with fields matching the given combination of `path` and `option_name`.
    pub fn find_back_target_with_option<P, T>(
        &mut self,
        path: P,
        option_name: T,
    ) -> Option<&MountInfoEntry>
    where
        P: AsRef<Path>,
        T: AsRef<str>,
    {
        let path = path.as_ref();
        let option_name = option_name.as_ref();
        let direction = Direction::Backward;
        log::debug!( "MountInfo::find_back_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option_name);

        Self::lookup_target_with_options(self, direction, path, option_name, None)
    }

    /// Performs a cursory search in the table, from **start** to **end**, and returns the first
    /// [`MountInfoEntry`] with fields matching **exactly** the given combination of `path` and
    /// `option`.
    pub fn find_target_with_exact_option<P, T>(
        &mut self,
        path: P,
        option: &MountOption,
    ) -> Option<&MountInfoEntry>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!( "MountInfo::find_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option);

        Self::lookup_target_with_options(self, direction, path, option.name(), option.value())
    }

    /// Performs a cursory search in the table, from **end** to **start**, and returns the first
    /// [`MountInfoEntry`] with fields matching **exactly** the given combination of `path` and
    /// `option`.
    pub fn find_back_target_with_exact_option<P, T>(
        &mut self,
        path: P,
        option: &MountOption,
    ) -> Option<&MountInfoEntry>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!( "MountInfo::find_back_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option);

        Self::lookup_target_with_options(self, direction, path, option.name(), option.value())
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`MountInfoEntry`] with fields matching the given
    /// `source`/`target` pair.
    fn lookup_pair<'a>(
        table: &mut Self,
        direction: Direction,
        source: &Source,
        target: &Path,
    ) -> Option<&'a MountInfoEntry> {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok()?;
        let target_cstr = ffi_utils::as_ref_path_to_c_string(target).ok()?;

        log::debug!( "MountInfo::lookup_pair searching {:?} for entry matching source/target pair {:?} / {:?}", direction, source, target);

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_table_find_pair(
                table.inner,
                source_cstr.as_ptr(),
                target_cstr.as_ptr(),
                direction as i32,
            ))
        };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!(
                    "found no entry with source/target pair {:?} / {:?} while searching {:?}",
                    source, target, direction,
                );
                log::debug!( "MountInfo::lookup_pair {err_msg}. libmount::mnt_table_find_pair returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
                    "MountInfo::lookup_pair found entry matching source/target pair {:?} / {:?}",
                    source,
                    target
                );

                let entry = owning_ref_from_ptr!(table, MountInfoEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`MountInfoEntry`] with
    /// fields matching the given `source`/`target` pair.
    ///
    /// **Warning:** this method runs the same code as [`MountInfo::find_source_path`] and
    /// [`MountInfo::find_target`] under the hood, evaluating every table entry, making it the
    /// slowest of the search methods.
    pub fn find_pair<T>(&mut self, source: &Source, target: T) -> Option<&MountInfoEntry>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        log::debug!( "MountInfo::find_pair searching table from top to bottom for entry with source/target pair {:?} / {:?}", source, target);

        Self::lookup_pair(self, Direction::Forward, source, target)
    }

    /// Searches the table from **end** to **start**, and returns the first [`MountInfoEntry`] with
    /// fields matching the given `source`/`target` pair.
    ///
    /// **Warning:** this method runs the same code as [`MountInfo::find_back_source_path`] and
    /// [`MountInfo::find_back_target`] under the hood, evaluating every table entry, making it the
    /// slowest of the search methods.
    pub fn find_back_pair<T>(&mut self, source: &Source, target: T) -> Option<&MountInfoEntry>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        log::debug!( "MountInfo::find_back_pair searching table from bottom to top for entry with source/target pair {:?} / {:?}", source, target);

        Self::lookup_pair(self, Direction::Backward, source, target)
    }

    //---- END getters

    //---- BEGIN iterators

    /// Returns an iterator over immutable [`MountInfo`] entries
    ///
    /// # Panics
    ///
    /// Panics if it fails to create a [`MountInfoIter`].
    pub fn iter(&self) -> MountInfoIter {
        log::debug!("MountInfo::iter creating a new `MountInfoIter`");

        MountInfoIter::new(self).unwrap()
    }

    /// Tries to instanciate an iterator over immutable [`MountInfo`] entries
    pub fn try_iter(&self) -> Result<MountInfoIter, MountInfoIterError> {
        log::debug!("MountInfo::try_iter creating a new `MountInfoIter`");

        MountInfoIter::new(self)
    }

    /// Returns an iterator over the children in the file system sub-tree of [`MountInfo`] entries.
    ///
    /// # Panics
    ///
    /// Panics if it fails to create a [`MountInfoChildIter`] iterator.
    pub fn iter_children<'table>(
        &'table self,
        parent: &'table MountInfoEntry,
    ) -> MountInfoChildIter {
        log::debug!("MountInfo::iter_children creating a new `MountInfoChildIter`");

        MountInfoChildIter::new(self, parent).unwrap()
    }

    /// Tries to instanciate an iterator over the children in the file system sub-tree of
    /// [`MountInfo`] entries.
    pub fn try_iter_children<'table>(
        &'table self,
        parent: &'table MountInfoEntry,
    ) -> Result<MountInfoChildIter, MountInfoChildIterError> {
        log::debug!("MountInfo::try_iter_children creating a new `MountInfoChildIter`");

        MountInfoChildIter::new(self, parent)
    }

    /// Returns an iterator over all [`MountInfo`] entries sharing the same mountpoint as `entry`
    /// (i.e. over mounted entries).
    ///
    /// # Panics
    ///
    /// Panics if it fails to create a [`MountInfoOvermountIter`] iterator.
    pub fn iter_overmounts<'table>(
        &'table self,
        entry: &'table MountInfoEntry,
    ) -> MountInfoOvermountIter {
        log::debug!("MountInfo::iter_overmounts creating a new `MountInfoOvermountIter`");

        MountInfoOvermountIter::new(self, entry)
    }

    //---- END iterators

    //---- BEGIN setters

    /// Sets an syntax error handler function for the file system description file parser.
    ///
    /// The error handler takes two parameters:
    /// - a `file_name`: the name of the file being parsed.
    /// - a `line_number`: the line number of the syntax error.
    pub fn set_parser_error_handler<F>(&mut self, err_handler: F) -> Result<(), MountInfoError>
    where
        F: Fn(&str, usize) -> ParserFlow,
    {
        log::debug!("MountInfo::set_parser_error_handler setting up parser error handler");
        #[doc(hidden)]
        /// Callback function to handle syntax errors in file system description files
        /// during parsing. Used by `libmount::mnt_table_parse_file`.
        unsafe extern "C" fn parser_callback<F>(
            table: *mut libmount::libmnt_table,
            file_name: *const libc::c_char,
            line: libc::c_int,
        ) -> libc::c_int
        where
            F: Fn(&str, usize) -> ParserFlow,
        {
            // Convert file name to string reference.
            let file_name = ffi_utils::const_char_array_to_str_ref(file_name)
                .ok()
                .unwrap_or("");

            // Rebuild the callback function.
            let mut callback_ptr = MaybeUninit::<*mut libc::c_void>::zeroed();
            unsafe {
                callback_ptr.write(libmount::mnt_table_get_userdata(table));
            }

            // Since we set the handler function ourselves, we can safely assume this pointer
            // is never NULL.
            let callback_ptr = unsafe { callback_ptr.assume_init() };
            let handler = &mut *(callback_ptr as *mut F);

            handler(file_name, line as usize) as i32
        }

        // Moving the closure to the heap with `Box::new`, to live there for some unknown period of
        // time.  Then, call `Box::into_raw` on it, to get a raw pointer to the closure, and
        // prevent the memory it uses from being deallocated.
        let user_data = Box::into_raw(Box::new(err_handler));

        let result = unsafe { libmount::mnt_table_set_userdata(self.inner, user_data as *mut _) };

        match result {
            0 => {
                let result = unsafe {
                    libmount::mnt_table_set_parser_errcb(self.inner, Some(parser_callback::<F>))
                };

                match result {
                    0 => {
                        log::debug!(
                            "MountInfo::set_parser_error_handler set up parser error handler"
                        );
                        // FIXME the callback function is long lived. If the function is called
                        // several times, we risk a substantial memory leak until the end of the program,
                        // since `user_data` is never released between calls.

                        Ok(())
                    }
                    code => {
                        let err_msg = "failed to set parser syntax error handler".to_owned();
                        log::debug!( "MountInfo::set_parser_error_handler {err_msg}. libmount::mnt_table_set_parser_errcb returned error code: {code:?}");

                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Err(MountInfoError::Config(err_msg))
                    }
                }
            }
            code => {
                let err_msg = "failed to set error handler as userdata".to_owned();
                log::debug!( "MountInfo::set_parser_error_handler {err_msg}. libmount::mnt_table_set_userdata returned error code: {code:?}");

                // Deallocate closure on the heap.
                let _ = unsafe { Box::from_raw(user_data) };

                Err(MountInfoError::Config(err_msg))
            }
        }
    }

    /// Sets up a [`Cache`] for canonicalized paths and evaluated tags (e.g. `LABEL`, `UUID`).
    ///
    /// Assigning a cache to a `MountInfo` will help speed up all `find_*` methods, and perform
    /// more thorough searches.
    pub fn set_cache(&mut self, cache: Cache) -> Result<(), MountInfoError> {
        log::debug!("MountInfo::set_cache setting up a cache of paths and tags");

        // Increment cache's reference counter to avoid a premature deallocation leading to a SIGSEV.
        unsafe {
            libmount::mnt_ref_cache(cache.inner);
        }

        let result = unsafe { libmount::mnt_table_set_cache(self.inner, cache.inner) };
        match result {
            0 => {
                log::debug!("MountInfo::set_cache set up a cache of paths and tags");

                Ok(())
            }
            code => {
                let err_msg = "failed to set up a cache of paths and tags".to_owned();
                log::debug!( "MountInfo::set_cache {err_msg}. libmount::mnt_table_set_cache returned error code: {code:?}");

                Err(MountInfoError::Config(err_msg))
            }
        }
    }

    //---- END setters

    //---- BEGIN mutators

    fn collect_garbage(&mut self) {
        // Free item references created on the heap.
        while let Some(gc_item) = self.gc.pop() {
            gc_item.destroy();
        }
    }

    fn filter_by<F>(table: &mut Self, flags: u32, cmp_fn: F) -> Result<(), MountInfoError>
    where
        F: FnMut(&MountInfoEntry, &MountInfoEntry) -> Ordering,
    {
        #[doc(hidden)]
        /// Comparison function to identify duplicate entries.
        unsafe extern "C" fn compare<F>(
            table: *mut libmount::libmnt_table,
            this: *mut libmount::libmnt_fs,
            other: *mut libmount::libmnt_fs,
        ) -> libc::c_int
        where
            F: FnMut(&MountInfoEntry, &MountInfoEntry) -> Ordering,
        {
            // Temporarily increments the pointer's reference counter which will be decremented when
            // entry goes out of scope. This prevents us from freeing the data structure while it is
            // still in use.
            let this = MountInfoEntry::borrow_ptr(this);
            let other = MountInfoEntry::borrow_ptr(other);

            // Rebuild the comparison function.
            let mut user_data_ptr = MaybeUninit::<*mut libc::c_void>::zeroed();
            unsafe {
                user_data_ptr.write(libmount::mnt_table_get_userdata(table));
            }

            // Since we set the handler function ourselves, we can safely assume this pointer
            // is never NULL.
            let user_data = unsafe { user_data_ptr.assume_init() };
            let fn_cmp = &mut *(user_data as *mut F);

            match fn_cmp(&this, &other) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            }
        }

        // Moving the closure to the heap with `Box::new`, to live there for some unknown period of
        // time.  Then, call `Box::into_raw` on it, to get a raw pointer to the closure, and
        // prevent the memory it uses from being deallocated.
        let user_data = Box::into_raw(Box::new(cmp_fn));

        let result = unsafe { libmount::mnt_table_set_userdata(table.inner, user_data as *mut _) };

        match result {
            0 => {
                let result = unsafe {
                    libmount::mnt_table_uniq_fs(table.inner, flags as i32, Some(compare::<F>))
                };

                match result {
                    0 => {
                        log::debug!("MountInfo::filter_by removed duplicates");
                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Ok(())
                    }
                    code => {
                        let err_msg = "failed to remove duplicates from table".to_owned();
                        log::debug!( "MountInfo::filter_by {err_msg}. libmount::mnt_table_uniq_fs returned error code: {code:?}");

                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Err(MountInfoError::Deduplicate(err_msg))
                    }
                }
            }
            code => {
                let err_msg = "failed to set the comparison function as userdata".to_owned();
                log::debug!( "MountInfo::filter_by {err_msg}. libmount::mnt_table_uniq_fs returned error code: {code:?}");

                // Deallocate closure on the heap.
                let _ = unsafe { Box::from_raw(user_data) };

                Err(MountInfoError::Deduplicate(err_msg))
            }
        }
    }

    /// Removes the duplicate entries in this table keeping the first occurrence of an entry for
    /// which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:** this method preserves the index order of the entries in the table.
    pub fn dedup_first_by<F>(&mut self, cmp: F) -> Result<(), MountInfoError>
    where
        F: FnMut(&MountInfoEntry, &MountInfoEntry) -> Ordering,
    {
        log::debug!("MountInfo::dedup_first_by merging matching entries to the first occurrence");

        Self::filter_by(self, libmount::MNT_UNIQ_FORWARD, cmp)
    }

    /// Removes the duplicate entries in this table keeping the last occurrence of an entry for
    /// which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:** this method preserves the index order of the entries in the table.
    pub fn dedup_last_by<F>(&mut self, cmp: F) -> Result<(), MountInfoError>
    where
        F: FnMut(&MountInfoEntry, &MountInfoEntry) -> Ordering,
    {
        log::debug!("MountInfo::dedup_last_by merging matching entries to the last occurrence");
        static MNT_UNIQ_BACKWARD: u32 = 0;

        Self::filter_by(self, MNT_UNIQ_BACKWARD, cmp)
    }

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

    //---- END mutators

    //---- BEGIN predicates

    /// Returns `true` if the table has length of 0.
    pub fn is_empty(&self) -> bool {
        let state = unsafe { libmount::mnt_table_is_empty(self.inner) == 1 };
        log::debug!("MountInfo::is_empty value: {:?}", state);

        state
    }

    /// Returns `true` if the provided  `entry` matches an element contained in this table.  The
    /// function compares the `source`, `target`, and `root` fields of the function parameter
    /// against those of each entry in this object.
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

impl fmt::Display for MountInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output: Vec<String> = vec![];

        for line in self.iter() {
            output.push(line.to_string());
        }

        write!(f, "{}", output.join("\n"))
    }
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
        println!("{mount_info}");

        Ok(())
    }
}
