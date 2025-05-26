// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::io;
use std::mem::MaybeUninit;
use std::ops::Index;
use std::ops::IndexMut;
use std::path::Path;

// From this library
use crate::core::cache::Cache;

use crate::core::device::Source;
use crate::core::device::Tag;

use crate::core::entries::FsTabEntry;
use crate::core::errors::FsTabError;
use crate::core::errors::FsTabIterError;

use crate::core::iter::Direction;
use crate::core::iter::FsTabIter;
use crate::core::iter::FsTabIterMut;
use crate::core::iter::GenIterator;

use crate::owning_ref_from_ptr;

use crate::tables::GcItem;
use crate::tables::MountOption;
use crate::tables::ParserFlow;

use crate::ffi_utils;

/// Composer/editor of `fstab` file system description files.
///
/// An `FsTab` allows you to programmatically
/// - compose,
/// - edit,
/// - import,
/// - export,
/// - and/or merge
///
/// file system description files used by the [`mount`
/// command](https://www.man7.org/linux/man-pages/man8/mount.8.html) to attach:
/// - a block device,
/// - a shared network file system,
/// - or a pseudo-filesystem, to a mount point in a file hierarchy.
///
/// It holds each line in a file system description file as a list of [`FsTabEntry`]
/// instances. You can create a description file from scratch, or import information from data
/// sources on your system.
///
/// # `/etc/fstab`
///
/// The `/etc/fstab` file contains descriptive information about devices the OS can mount.
///
/// It has the following layout:
/// - each file system is described on a separate line,
/// - fields on each line are separated by tabs or spaces,
/// - lines starting with `#` are comments,
/// - blank lines are ignored.
///
/// `/etc/fstab` is only read by programs, and not written; it is the duty of the system
/// administrator to properly create and maintain this file.
///
/// Below is a sample `/etc/fstab` file, with added comments, extracted from an Alpine
/// Linux virtual machine.
///
/// ```text
/// # /etc/fstab
/// # Alpine Linux 3.19 (installed from alpine-virt-3.19.1-x86_64.iso)
/// #
///
/// UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f  /      ext4  rw,relatime 0 1
/// UUID=07aae9ba-12a1-4325-8b16-2ee214a6d7fd  /boot  ext4  rw,relatime 0 2
/// UUID=b9d72af2-f231-4cf8-9d0a-ba19e94a5087  swap   swap  defaults    0 0
///
/// /dev/cdrom    /media/cdrom  iso9660 noauto,ro       0 0
/// /dev/usbdisk  /media/usb    vfat    noauto          0 0
/// none          /tmp          tmpfs   nosuid,nodev    0 0
/// ```
///
/// # `fstab` file format
///
/// The table shown above has a 6-column structure, where each column defines a specific
/// parameter, and must be setup in the same order as in the following excerpt:
///
/// ```text
/// UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f  /      ext4  rw,relatime  0   1
///                    (1)                    (2)      (3)     (4)      (5) (6)
/// ```
///
/// - `(1)` **Device**: the name or UUID of the device to mount, otherwise known as the
///   ***source***.
/// - `(2)` **Mount Point**: the directory on which the device will be mounted, called the
///   ***target***.
/// - `(3)` **File System Type**: the type of file system the device uses (e.g. `ext4`, `tmpfs`, etc.).
/// - `(4)` **Options**: a comma-separated list of [filesystem-independent mount
///   options](https://www.man7.org/linux/man-pages/man8/mount.8.html#FILESYSTEM-INDEPENDENT_MOUNT_OPTIONS).
/// - `(5)` **File System Backup**: a boolean value, `1` or `0`, to respectively allow or disallow
///   the `dump` utility to examine files on an `ext2`/`ext3` file system, and to determine which
///   files need to be backed up. This is a legacy method that should NOT be used! Always set it to
///   `0`.
/// - `(6)` **File System Check Order**: the order in which the `fsck` command is run on the
///   devices to mount, to check and repair Linux file systems. Entries with the **lowest** value
///   will be checked **first**. Two entries with the same check order value will be verified in
///   parallel.<br> The value for the root file system should be set to `1`, and the others should
///   have a value of at least `2`, keeping in mind that `0` means `fsck` will not check the file
///   system.
///
/// # Examples
///
/// ## Compose a file system description file
///
/// ```text
/// # /etc/fstab
/// # Example from scratch
///
/// UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
/// /dev/usbdisk /media/usb vfat noauto 0 0
/// none /tmp tmpfs nosuid,nodev 0 0
/// ```
///
/// The following code will create a file system description file identical to the example above.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use tempfile::tempfile;
/// use std::fs::File;
/// use rsmount::tables::FsTab;
/// use rsmount::entries::FsTabEntry;
/// use rsmount::errors::FsTabError;
/// use rsmount::device::BlockDevice;
/// use rsmount::device::Pseudo;
/// use rsmount::device::Tag;
/// use rsmount::fs::FileSystem;
///
/// fn main() -> rsmount::Result<()> {
///     let mut fstab = FsTab::new()?;
///     // # /etc/fstab
///     fstab.set_intro_comments("# /etc/fstab\n")?;
///     // # Example from scratch
///     fstab.append_to_intro_comments("# Example from scratch\n")?;
///     //
///     fstab.append_to_intro_comments("\n")?;
///
///     // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
///     let uuid = Tag::try_from("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
///     let entry1 = FsTabEntry::builder()
///         .source(uuid)
///         .target("/")
///         .file_system_type(FileSystem::Ext4)
///         // Comma-separated list of mount options.
///         .mount_options("rw,relatime")
///         // Interval, in days, between file system backups by the dump command on ext2/3/4
///         // file systems.
///         .backup_frequency(0)
///         // Order in which file systems are checked by the `fsck` command.
///         .fsck_checking_order(1)
///         .build()?;
///
///     // /dev/usbdisk /media/usb vfat noauto 0 0
///     let block_device = BlockDevice::try_from("/dev/usbdisk")?;
///     let entry2 = FsTabEntry::builder()
///         .source(block_device)
///         .target("/media/usb")
///         .file_system_type(FileSystem::VFAT)
///         .mount_options("noauto")
///         .backup_frequency(0)
///         .fsck_checking_order(0)
///         .build()?;
///
///     // none /tmp tmpfs nosuid,nodev 0 0
///     let entry3 = FsTabEntry::builder()
///         .source(Pseudo::None)
///         .target("/tmp")
///         .file_system_type(FileSystem::Tmpfs)
///         .mount_options("nosuid,nodev")
///         .backup_frequency(0)
///         .fsck_checking_order(0)
///         .build()?;
///
///     fstab.push(entry1);
///     fstab.push(entry2);
///     fstab.push(entry3);
///
///     // Open file
///     let mut file: File = tempfile().unwrap();
///
///     // Write to disk
///     fstab.export_with_comments();
///     fstab.write_stream(&mut file).map_err(FsTabError::from)?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct FsTab {
    pub(crate) inner: *mut libmount::libmnt_table,
    pub(crate) gc: Vec<GcItem>,
}

impl Drop for FsTab {
    fn drop(&mut self) {
        log::debug!("::drop deallocating `FsTab` instance");

        unsafe { libmount::mnt_unref_table(self.inner) };
        self.collect_garbage();
    }
}

impl AsRef<FsTab> for FsTab {
    #[inline]
    fn as_ref(&self) -> &FsTab {
        self
    }
}

impl Index<usize> for FsTab {
    type Output = FsTabEntry;

    /// Performs the indexing (`container\[index]`) operation.
    fn index(&self, index: usize) -> &Self::Output {
        log::debug!("FsTab::index getting item at index: {:?}", index);

        #[cold]
        #[inline(never)]
        #[track_caller]
        fn indexing_failed() -> ! {
            panic!("Index out of bounds");
        }

        let mut iter = FsTabIter::new(self).unwrap();

        match iter.nth(index) {
            Some(item) => item,
            None => indexing_failed(),
        }
    }
}

impl IndexMut<usize> for FsTab {
    /// Performs the mutable indexing (`container\[index]`) operation.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        log::debug!("FsTab::index getting item at index: {:?}", index);

        #[cold]
        #[inline(never)]
        #[track_caller]
        fn indexing_failed() -> ! {
            panic!("Index out of bounds");
        }

        let mut iter = FsTabIterMut::new(self).unwrap();

        match iter.nth(index) {
            Some(item) => item,
            None => indexing_failed(),
        }
    }
}

impl FsTab {
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
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_table) -> FsTab {
        Self {
            inner: ptr,
            gc: vec![],
        }
    }

    #[doc(hidden)]
    /// Borrows an instance.
    #[allow(dead_code)]
    pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_table) -> FsTab {
        let mut table = Self::from_ptr(ptr);
        // We are virtually ceding ownership of this table which will be automatically
        // deallocated once it is out of scope, incrementing its reference counter protects it from
        // being freed prematurely.
        table.incr_ref_counter();

        table
    }

    /// Creates a new empty `FsTab`.
    pub fn new() -> Result<FsTab, FsTabError> {
        log::debug!("FsTab::new creating a new `FsTab` instance");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();

        unsafe { ptr.write(libmount::mnt_new_table()) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to create a new `FsTab`".to_owned();
                log::debug!(
                    "FsTab::new {err_msg}. libmount::mnt_new_table returned a NULL pointer"
                );

                Err(FsTabError::Creation(err_msg))
            }
            ptr => {
                log::debug!("FsTab::new created a new `FsTab` instance");
                let table = Self::from_ptr(ptr);

                Ok(table)
            }
        }
    }

    /// Creates a new `FsTab`, and fills it with entries parsed from the given `file`.
    pub fn new_from_file<T>(file: T) -> Result<FsTab, FsTabError>
    where
        T: AsRef<Path>,
    {
        let file = file.as_ref();
        let file_cstr = ffi_utils::as_ref_path_to_c_string(file)?;
        log::debug!(
            "FsTab::new_from_file creating a new `FsTab` from file {:?}",
            file
        );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();
        unsafe { ptr.write(libmount::mnt_new_table_from_file(file_cstr.as_ptr())) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!("failed to create a new `FsTab` from file {:?}", file);
                log::debug!(
                        "FsTab::new_from_file {}. libmount::mnt_new_table_from_file returned a NULL pointer",
                        err_msg
                    );

                Err(FsTabError::Creation(err_msg))
            }
            ptr => {
                log::debug!("FsTab::new_from_file created a new `FsTab` instance");
                let table = Self::from_ptr(ptr);

                Ok(table)
            }
        }
    }

    /// Creates a new `FsTab`, and fills it with entries parsed from the files with extension
    /// `.fstab` in the given `directory`.
    pub fn new_from_directory<T>(directory: T) -> Result<FsTab, FsTabError>
    where
        T: AsRef<Path>,
    {
        let dir = directory.as_ref();
        let dir_cstr = ffi_utils::as_ref_path_to_c_string(dir)?;
        log::debug!(
            "FsTab::new_from_directory creating a new `FsTab` from files in {:?}",
            dir
        );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();
        unsafe { ptr.write(libmount::mnt_new_table_from_dir(dir_cstr.as_ptr())) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!("failed to create a new `FsTab` from files in {:?}", dir);
                log::debug!(
                        "FsTab::new_from_directory {}. libmount::mnt_new_table_from_dir returned a NULL pointer",
                        err_msg
                    );

                Err(FsTabError::Creation(err_msg))
            }
            ptr => {
                log::debug!("FsTab::new_from_directory created a new `FsTab` instance");
                let table = Self::from_ptr(ptr);

                Ok(table)
            }
        }
    }

    //---- BEGIN getters

    /// Returns a reference to the [`Cache`] instance associated with this `FsTab`.
    pub fn cache(&self) -> Option<&Cache> {
        log::debug!("FsTab::cache getting associated path and tag cache");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_cache>::zeroed();

        unsafe { ptr.write(libmount::mnt_table_get_cache(self.inner)) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("FsTab::cache failed to get associated path and tag cache. libmount::mnt_table_get_cache returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!("FsTab::cache got associated path and tag cache");
                let cache = owning_ref_from_ptr!(self, Cache, ptr);

                Some(cache)
            }
        }
    }

    /// Returns a reference to the first element of the `FsTab`, or `None` if it is empty.
    pub fn first(&self) -> Option<&FsTabEntry> {
        log::debug!("FsTab::first getting reference to first table entry");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe { libmount::mnt_table_first_fs(self.inner, ptr.as_mut_ptr()) };
        match result {
            0 => {
                log::debug!("FsTab::first got reference to first table entry");
                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(self, FsTabEntry, ptr);

                Some(entry)
            }
            code => {
                log::debug!( "FsTab::first failed to get reference to first table entry. libmount::mnt_table_first_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns a reference to the last element of the `FsTab`, or `None` if it is empty.
    pub fn last(&self) -> Option<&FsTabEntry> {
        log::debug!("FsTab::last getting reference to last table entry");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe { libmount::mnt_table_last_fs(self.inner, ptr.as_mut_ptr()) };
        match result {
            0 => {
                log::debug!("FsTab::last got reference to last table entry");
                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(self, FsTabEntry, ptr);

                Some(entry)
            }
            code => {
                log::debug!( "FsTab::last failed to get reference to last table entry. libmount::mnt_table_last_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns the index of a table entry.
    pub fn position(&self, entry: &FsTabEntry) -> Option<usize> {
        log::debug!("FsTab::position searching for an entry in the table");

        let result = unsafe { libmount::mnt_table_find_fs(self.inner, entry.inner) };
        match result {
            index if index > 0 => {
                log::debug!(
                    "FsTab::position mount table contains entry at index: {:?}",
                    index
                );

                Some(index as usize)
            }
            code => {
                log::debug!( "FsTab::position no matching entry in table: libmount::mnt_table_find_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns the number of entries in the table.
    pub fn len(&self) -> usize {
        let len = unsafe { libmount::mnt_table_get_nents(self.inner) };
        log::debug!("FsTab::len value: {:?}", len);

        len as usize
    }

    /// Returns a reference to an element at `index`, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&FsTabEntry> {
        log::debug!(
            "FsTab::get_mut getting reference of item at index: {:?}",
            index
        );

        FsTabIter::new(self)
            .ok()
            .and_then(|mut iter| iter.nth(index))
    }

    /// Returns a mutable reference to an element at `index`, or `None` if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut FsTabEntry> {
        log::debug!(
            "FsTab::get_mut getting mutable reference of item at index: {:?}",
            index
        );

        FsTabIterMut::new(self)
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
    ) -> Option<&'a FsTabEntry>
    where
        P: FnMut(&FsTabEntry) -> bool,
    {
        #[doc(hidden)]
        /// Callback function called by the `libmount::mnt_table_find_next_fs` C-binding. The function
        /// searches for the first element that satisfies the predicate using the callback as a
        /// C-compatible wrapper around the closure.
        unsafe extern "C" fn callback<P>(
            entry_ptr: *mut libmount::libmnt_fs,
            predicate_fn_ptr: *mut libc::c_void,
        ) -> libc::c_int
        where
            P: FnMut(&FsTabEntry) -> bool,
        {
            // Temporarily increments the pointer's reference counter which will be decremented when
            // `entry` goes out of scope. This prevents us from freeing the data structure while it is
            // still in use.
            let entry = FsTabEntry::borrow_ptr(entry_ptr);

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

                log::debug!("FsTab::find_first_entry found first `FsTabEntry` matching predicate");
                let entry_ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(table, FsTabEntry, entry_ptr);

                Some(entry)
            }
            code => {
                // To ensure the closure is properly deallocated when this variable drops out
                // of scope.
                let _predicate = unsafe { Box::from_raw(data) };

                let err_msg = "failed to find `FsTabEntry` matching predicate".to_owned();
                log::debug!( "FsTab::find_first_entry {err_msg}. libmount::mnt_table_find_next_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`FsTabEntry`] that satisfies
    /// the `predicate`.
    ///
    /// # Panics
    ///
    /// Panics if it can not create a forward iterator.
    pub fn find_first<P>(&mut self, predicate: P) -> Option<&FsTabEntry>
    where
        P: FnMut(&FsTabEntry) -> bool,
    {
        log::debug!( "FsTab::find_first finding first table entry matching predicate while iterating Forward");
        GenIterator::new(Direction::Forward)
            .ok()
            .and_then(|iterator| FsTab::find_first_entry(self, iterator.inner, predicate))
    }

    /// Searches the table from **end** to **start**, and returns the first [`FsTabEntry`] that
    /// satisfies the `predicate`.
    ///
    /// # Panics
    ///
    /// Panics if it can not create a backward iterator.
    pub fn find_back_first<P>(&mut self, predicate: P) -> Option<&FsTabEntry>
    where
        P: FnMut(&FsTabEntry) -> bool,
    {
        log::debug!( "FsTab::find_back_first finding first table entry matching predicate while iterating Backward");
        GenIterator::new(Direction::Backward)
            .ok()
            .and_then(|iterator| FsTab::find_first_entry(self, iterator.inner, predicate))
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`FsTabEntry`] with fields matching the given
    /// source `path`.
    fn lookup_source<'a>(
        table: &mut Self,
        direction: Direction,
        source: &Source,
    ) -> Option<&'a FsTabEntry> {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok()?;
        let source_ptr = if source.is_pseudo_fs() {
            // For pseudo file systems `libmount::mnt_table_find_source`
            // expects a NULL pointer path.
            std::ptr::null()
        } else {
            source_cstr.as_ptr()
        };

        log::debug!(
            "FsTab::lookup_source searching {:?} for entry matching source {:?}",
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
                log::debug!( "FsTab::lookup_source {err_msg}. libmount::mnt_table_find_source returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
                    "FsTab::lookup_source found entry matching source {:?} while searching {:?}",
                    source,
                    direction
                );

                let entry = owning_ref_from_ptr!(table, FsTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`FsTabEntry`] with a field
    /// matching the given `source`.
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an exact
    /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`FsTab::set_cache`].
    ///
    /// **Note:** providing an **empty source** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_source(&mut self, source: &Source) -> Option<&FsTabEntry> {
        let direction = Direction::Forward;
        log::debug!(
            "FsTab::find_source searching {:?} for the first entry with a source matching {:?}",
            direction,
            source
        );

        Self::lookup_source(self, direction, source)
    }

    /// Searches the table from **end** to **start**, and returns the first [`FsTabEntry`] with
    /// a field matching the given `source`.
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an exact
    /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`FsTab::set_cache`].
    ///
    /// **Note:** providing an **empty source** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_back_source(&mut self, source: &Source) -> Option<&FsTabEntry> {
        let direction = Direction::Backward;
        log::debug!(
 "FsTab::find_back_source searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        source
                    );

        Self::lookup_source(self, direction, source)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`FsTabEntry`] with fields matching the given source `path`.
    fn lookup_source_path<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
    ) -> Option<&'a FsTabEntry> {
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
            "FsTab::lookup_source_path searching {:?} for entry matching source path {:?}",
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
                log::debug!( "FsTab::lookup_source_path {err_msg}. libmount::mnt_table_find_srcpath returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
 "FsTab::lookup_source_path found entry matching source path {:?} while searching {:?}",
                                path,
                                direction
                            );

                let entry = owning_ref_from_ptr!(table, FsTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`FsTabEntry`] with
    /// a `source` field matching the given `path`.
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`FsTab::set_cache`].
    ///
    /// **Note:** providing an **empty path** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_source_path<T>(&mut self, path: T) -> Option<&FsTabEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!(
              "FsTab::find_source_path searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        path
                    );

        Self::lookup_source_path(self, direction, path)
    }

    /// Searches the table from **end** to **start**, and returns the first [`FsTabEntry`] with
    /// a `source` field matching the given `path`.
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`FsTab::set_cache`].
    ///
    /// **Note:** providing an **empty path** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_back_source_path<T>(&mut self, path: T) -> Option<&FsTabEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!(
              "FsTab::find_back_source_path searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        path
                    );

        Self::lookup_source_path(self, direction, path)
    }

    /// Returns the comments at the head of this `FsTab`.
    pub fn intro_comments(&self) -> Option<&str> {
        log::debug!("FsTab::intro_comments getting intro comment");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        unsafe { ptr.write(libmount::mnt_table_get_intro_comment(self.inner)) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("FsTab::intro_comments found no intro comment. libmount::mnt_table_get_intro_comment returned a NULL pointer");

                None
            }
            ptr => {
                let comment = ffi_utils::const_char_array_to_str_ref(ptr).ok();
                log::debug!("FsTab::intro_comments value: {:?}", comment);

                comment
            }
        }
    }

    /// Returns the comments at the tail of this `FsTab`.
    pub fn trailing_comments(&self) -> Option<&str> {
        log::debug!("FsTab::trailing_comments getting trailing comment");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        unsafe { ptr.write(libmount::mnt_table_get_trailing_comment(self.inner)) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("FsTab::trailing_comments found no trailing comment. libmount::mnt_table_get_trailing_comment returned a NULL pointer");

                None
            }
            ptr => {
                let comment = ffi_utils::const_char_array_to_str_ref(ptr).ok();
                log::debug!("FsTab::trailing_comments value: {:?}", comment);

                comment
            }
        }
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`FsTabEntry`] with fields matching the given [`Tag`].
    fn lookup_tag<'a>(table: &mut Self, direction: Direction, tag: &Tag) -> Option<&'a FsTabEntry> {
        log::debug!(
            "FsTab::lookup_tag searching {:?} for entry matching tag {:?}",
            direction,
            tag
        );
        let name_cstr = ffi_utils::as_ref_str_to_c_string(tag.name().to_string()).ok()?;
        let value_cstr = ffi_utils::as_ref_str_to_c_string(tag.value()).ok()?;

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        unsafe {
            ptr.write(libmount::mnt_table_find_tag(
                table.inner,
                name_cstr.as_ptr(),
                value_cstr.as_ptr(),
                direction as i32,
            ))
        };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!(
                    "failed to find entry matching tag {:?} while searching {:?}",
                    tag, direction
                );
                log::debug!(
                    "FsTab::lookup_tag {}. libmount::mnt_table_find_tag returned a NULL pointer",
                    err_msg
                );

                None
            }
            ptr => {
                log::debug!(
                    "FsTab::lookup_tag found entry matching tag {:?} while searching {:?}",
                    tag,
                    direction
                );
                let boxed = Box::new(ptr);
                let (boxed_ptr, entry) = unsafe { FsTabEntry::ref_from_boxed_ptr(boxed) };
                table.gc.push(boxed_ptr.into());

                Some(entry)
            }
        }
    }

    /// Searches the table from **top** to **bottom**, and returns the first [`FsTabEntry`] with
    /// a `source` field matching the given [`Tag`].
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an
    /// exact `tag` match. To perform a deep search, which implies converting the tag to its
    /// associated device's name, following symlinks, canonicalizing paths, etc., set up a
    /// [`Cache`] with [`FsTab::set_cache`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// use std::path::PathBuf;
    /// use std::str::FromStr;
    /// use rsmount::device::Source;
    /// use rsmount::device::Tag;
    /// use rsmount::entries::FsTabEntry;
    /// use rsmount::tables::FsTab;
    /// use rsmount::fs::FileSystem;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///     let mut fstab = FsTab::new()?;
    ///
    ///     // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f  /  ext4  rw,relatime 0 1
    ///     let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
    ///     let entry = FsTabEntry::builder()
    ///         .source(uuid)
    ///         .target("/")
    ///         .file_system_type(FileSystem::Ext4)
    ///         // Comma-separated list of mount options.
    ///         .mount_options("rw,relatime")
    ///         // Interval, in days, between file system backups by the dump command on ext2/3/4
    ///         // file systems.
    ///         .backup_frequency(0)
    ///         // Order in which file systems are checked by the `fsck` command.
    ///         .fsck_checking_order(1)
    ///         .build()?;
    ///
    ///     fstab.push(entry);
    ///
    ///     let tag: Tag = "UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f".parse()?;
    ///
    ///     let entry = fstab.find_source_tag(&tag);
    ///
    ///     let actual = entry.unwrap().tag();
    ///     let expected = Some(tag);
    ///     assert_eq!(actual, expected);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn find_source_tag(&mut self, tag: &Tag) -> Option<&FsTabEntry> {
        let direction = Direction::Forward;
        log::debug!(
            "FsTab::find_source_tag searching {:?} for the first entry a tag matching {:?}",
            direction,
            tag
        );

        Self::lookup_tag(self, direction, tag)
    }

    /// Searches the table from **bottom** to **top**, and returns the first [`FsTabEntry`] with
    /// a `source` field matching the given [`Tag`].
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an
    /// exact `tag` match. To perform a deep search, which implies converting the tag to its
    /// associated device's name, following symlinks, canonicalizing paths, etc., set up a
    /// [`Cache`] with [`FsTab::set_cache`].
    pub fn find_back_source_tag(&mut self, tag: &Tag) -> Option<&FsTabEntry> {
        let direction = Direction::Backward;
        log::debug!(
            "FsTab::find_back_source_tag searching {:?} for the first entry a tag matching {:?}",
            direction,
            tag
        );

        Self::lookup_tag(self, direction, tag)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`FsTabEntry`] with fields matching the given target `path`.
    fn lookup_target<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
    ) -> Option<&'a FsTabEntry> {
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok()?;
        log::debug!(
            "FsTab::lookup_target searching {:?} for entry matching target {:?}",
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
                log::debug!( "FsTab::lookup_target {err_msg}. libmount::mnt_table_find_target returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
                    "FsTab::lookup_target found entry matching target {:?} while searching {:?}",
                    path,
                    direction
                );

                let entry = owning_ref_from_ptr!(table, FsTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`FsTabEntry`] with
    /// a `target` field matching the given `path`.
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an exact `path`
    /// match. To perform a deep search, which implies following symlinks, canonicalizing paths, etc.,
    /// set up a [`Cache`] with [`FsTab::set_cache`].
    pub fn find_target<T>(&mut self, path: T) -> Option<&FsTabEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!(
            "FsTab::find_target searching {:?} for the first entry with a target matching {:?}",
            direction,
            path
        );

        Self::lookup_target(self, direction, path)
    }

    /// Searches the table from **end** to **start**, and returns the first [`FsTabEntry`] with a
    /// `target` field matching the given `path`.
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an exact `path`
    /// match. To perform a deep search, which implies following symlinks, canonicalizing paths, etc.,
    /// set up a [`Cache`] with [`FsTab::set_cache`].
    pub fn find_back_target<T>(&mut self, path: T) -> Option<&FsTabEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!(
 "FsTab::find_back_target searching {:?} for the first entry with a target matching {:?}",
                    direction,
                    path
                );

        Self::lookup_target(self, direction, path)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`FsTabEntry`] with fields matching the given the combination
    /// of `path` and `option_name` with `option_value`.
    fn lookup_target_with_options<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
        option_name: &str,
        option_value: Option<&str>,
    ) -> Option<&'a FsTabEntry> {
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
 "FsTab::lookup_target_with_options searching {:?} for entry matching the combination of path {:?} and option {:?}{}",
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
                log::debug!( "FsTab::lookup_target_with_options {err_msg}. libmount::mnt_table_find_target_with_option  returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
 "FsTab::lookup_target_with_options found entry matching the combination of path {:?} and option {:?}{}",
                            path,
                            option_name,
                            opt_value
                        );

                let entry = owning_ref_from_ptr!(table, FsTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Performs a cursory search in the table, from **start** to **end**, and returns the first
    /// [`FsTabEntry`] with fields matching the given combination of `path` and `option_name`.
    pub fn find_target_with_option<P, T>(&mut self, path: P, option_name: T) -> Option<&FsTabEntry>
    where
        P: AsRef<Path>,
        T: AsRef<str>,
    {
        let path = path.as_ref();
        let option_name = option_name.as_ref();
        let direction = Direction::Forward;
        log::debug!( "FsTab::find_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option_name);

        Self::lookup_target_with_options(self, direction, path, option_name, None)
    }

    /// Performs a cursory search in the table, from **end** to **start**, and returns the first
    /// [`FsTabEntry`] with fields matching the given combination of `path` and `option_name`.
    pub fn find_back_target_with_option<P, T>(
        &mut self,
        path: P,
        option_name: T,
    ) -> Option<&FsTabEntry>
    where
        P: AsRef<Path>,
        T: AsRef<str>,
    {
        let path = path.as_ref();
        let option_name = option_name.as_ref();
        let direction = Direction::Backward;
        log::debug!( "FsTab::find_back_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option_name);

        Self::lookup_target_with_options(self, direction, path, option_name, None)
    }

    /// Performs a cursory search in the table, from **start** to **end**, and returns the first
    /// [`FsTabEntry`] with fields matching **exactly** the given combination of `path` and `option`.
    pub fn find_target_with_exact_option<P, T>(
        &mut self,
        path: P,
        option: &MountOption,
    ) -> Option<&FsTabEntry>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!( "FsTab::find_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option);

        Self::lookup_target_with_options(self, direction, path, option.name(), option.value())
    }

    /// Performs a cursory search in the table, from **end** to **start**, and returns the first
    /// [`FsTabEntry`] with fields matching **exactly** the given combination of `path` and `option`.
    pub fn find_back_target_with_exact_option<P, T>(
        &mut self,
        path: P,
        option: &MountOption,
    ) -> Option<&FsTabEntry>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!( "FsTab::find_back_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option);

        Self::lookup_target_with_options(self, direction, path, option.name(), option.value())
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`FsTabEntry`] with fields matching the given `source`/`target`
    /// pair.
    fn lookup_pair<'a>(
        table: &mut Self,
        direction: Direction,
        source: &Source,
        target: &Path,
    ) -> Option<&'a FsTabEntry> {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok()?;
        let target_cstr = ffi_utils::as_ref_path_to_c_string(target).ok()?;

        log::debug!(
            "FsTab::lookup_pair searching {:?} for entry matching source/target pair {:?} / {:?}",
            direction,
            source,
            target
        );

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
                log::debug!(
                    "FsTab::lookup_pair {}. libmount::mnt_table_find_pair returned a NULL pointer",
                    err_msg
                );

                None
            }
            ptr => {
                log::debug!(
                    "FsTab::lookup_pair found entry matching source/target pair {:?} / {:?}",
                    source,
                    target
                );

                let entry = owning_ref_from_ptr!(table, FsTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`FsTabEntry`] with
    /// fields matching the given `source`/`target` pair.
    ///
    /// **Warning:** this method runs the same code as [`FsTab::find_source_path`] and
    /// [`FsTab::find_target`] under the hood, evaluating every table entry, making it the
    /// slowest of the search methods.
    pub fn find_pair<T>(&mut self, source: &Source, target: T) -> Option<&FsTabEntry>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        log::debug!( "FsTab::find_pair searching table from top to bottom for entry with source/target pair {:?} / {:?}", source, target);

        Self::lookup_pair(self, Direction::Forward, source, target)
    }

    /// Searches the table from **end** to **start**, and returns the first [`FsTabEntry`] with
    /// fields matching the given `source`/`target` pair.
    ///
    /// **Warning:** this method runs the same code as [`FsTab::find_back_source_path`] and
    /// [`FsTab::find_back_target`] under the hood, evaluating every table entry, making it the
    /// slowest of the search methods.
    pub fn find_back_pair<T>(&mut self, source: &Source, target: T) -> Option<&FsTabEntry>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        log::debug!( "FsTab::find_back_pair searching table from bottom to top for entry with source/target pair {:?} / {:?}", source, target);

        Self::lookup_pair(self, Direction::Backward, source, target)
    }

    //---- END getters

    //---- BEGIN iterators

    /// Returns an iterator over immutable [`FsTab`] entries
    ///
    /// # Panics
    ///
    /// Panics if it fails to create a [`FsTabIter`].
    pub fn iter(&self) -> FsTabIter {
        log::debug!("FsTab::iter creating a new `FsTabIter`");

        FsTabIter::new(self).unwrap()
    }
    /// Tries to instanciate an iterator over immutable [`FsTab`] entries
    pub fn try_iter(&self) -> Result<FsTabIter, FsTabIterError> {
        log::debug!("FsTab::try_iter creating a new `FsTabIter`");

        FsTabIter::new(self)
    }

    /// Returns an iterator over mutable [`FsTab`] entries.
    ///
    /// # Panics
    ///
    /// Panics if it fails to create a [`FsTabIterMut`].
    pub fn iter_mut(&mut self) -> FsTabIterMut {
        log::debug!("FsTab::iter_mut creating a new `FsTabIterMut`");

        FsTabIterMut::new(self).unwrap()
    }

    /// Tries to instanciate an iterator over mutable [`FsTab`] entries.
    pub fn try_iter_mut(&mut self) -> Result<FsTabIterMut, FsTabIterError> {
        log::debug!("FsTab::try_iter_mut creating a new `FsTabIterMut`");

        FsTabIterMut::new(self)
    }

    //---- END iterators

    //---- BEGIN setters

    /// Sets an syntax error handler function for the file system description file parser.
    ///
    /// The error handler takes two parameters:
    /// - a `file_name`: the name of the file being parsed.
    /// - a `line_number`: the line number of the syntax error.
    pub fn set_parser_error_handler<F>(&mut self, err_handler: F) -> Result<(), FsTabError>
    where
        F: Fn(&str, usize) -> ParserFlow,
    {
        log::debug!("FsTab::set_parser_error_handler setting up parser error handler");
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
                        log::debug!("FsTab::set_parser_error_handler set up parser error handler");
                        // FIXME the callback function is long lived. If the function is called
                        // several times, we risk a substantial memory leak until the end of the program,
                        // since `user_data` is never released between calls.

                        Ok(())
                    }
                    code => {
                        let err_msg = "failed to set parser syntax error handler".to_owned();
                        log::debug!( "FsTab::set_parser_error_handler {err_msg}. libmount::mnt_table_set_parser_errcb returned error code: {code:?}");

                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Err(FsTabError::Config(err_msg))
                    }
                }
            }
            code => {
                let err_msg = "failed to set error handler as userdata".to_owned();
                log::debug!( "FsTab::set_parser_error_handler {err_msg}. libmount::mnt_table_set_userdata returned error code: {code:?}");

                // Deallocate closure on the heap.
                let _ = unsafe { Box::from_raw(user_data) };

                Err(FsTabError::Config(err_msg))
            }
        }
    }

    /// Sets up a [`Cache`] for canonicalized paths and evaluated tags (e.g. `LABEL`, `UUID`).
    ///
    /// Assigning a cache to a `FsTab` will help speed up all `find_*` methods, and perform more
    /// thorough searches.
    pub fn set_cache(&mut self, cache: Cache) -> Result<(), FsTabError> {
        log::debug!("FsTab::set_cache setting up a cache of paths and tags");

        // Increment cache's reference counter to avoid a premature deallocation leading to a SIGSEV.
        unsafe {
            libmount::mnt_ref_cache(cache.inner);
        }

        let result = unsafe { libmount::mnt_table_set_cache(self.inner, cache.inner) };
        match result {
            0 => {
                log::debug!("FsTab::set_cache set up a cache of paths and tags");

                Ok(())
            }
            code => {
                let err_msg = "failed to set up a cache of paths and tags".to_owned();
                log::debug!( "FsTab::set_cache {err_msg}. libmount::mnt_table_set_cache returned error code: {code:?}");

                Err(FsTabError::Config(err_msg))
            }
        }
    }

    /// Sets the introductory comment in the table.
    ///
    /// In the example below, lines `1` through `3` included are the introduction comments.
    ///
    /// ```text
    /// 1 | # /etc/fstab
    /// 2 | # Example from scratch
    /// 3 |
    /// 4 | UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
    /// 5 | /dev/usbdisk /media/usb vfat noauto 0 0
    /// 6 | tmpfs /tmp tmpfs nosuid,nodev 0 0
    /// 7 |
    /// 8 | # Auto generated by Acme.
    /// 9 |
    /// ```
    pub fn set_intro_comments<T>(&mut self, comment: T) -> Result<(), FsTabError>
    where
        T: AsRef<str>,
    {
        let comment = comment.as_ref();
        let comment_cstr = ffi_utils::as_ref_str_to_c_string(comment)?;
        log::debug!(
            "FsTab::set_intro_comments setting intro comment to {:?}",
            comment
        );

        let result =
            unsafe { libmount::mnt_table_set_intro_comment(self.inner, comment_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::set_intro_comments set intro comment to {:?}",
                    comment
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set intro comment to {:?}", comment);
                log::debug!("FsTab::set_intro_comments {}. libmount::mnt_table_set_intro_comment returned error code: {:?}", err_msg, code);

                Err(FsTabError::Config(err_msg))
            }
        }
    }

    /// Sets the trailing comment in the table.
    ///
    /// In the example below, lines `7` through `9` included are the trailing comments.
    ///
    /// ```text
    /// 1 | # /etc/fstab
    /// 2 | # Example from scratch
    /// 3 |
    /// 4 | UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
    /// 5 | /dev/usbdisk /media/usb vfat noauto 0 0
    /// 6 | tmpfs /tmp tmpfs nosuid,nodev 0 0
    /// 7 |
    /// 8 | # Auto generated by Acme.
    /// 9 |
    /// ```
    pub fn set_trailing_comments<T>(&mut self, comment: T) -> Result<(), FsTabError>
    where
        T: AsRef<str>,
    {
        let comment = comment.as_ref();
        let comment_cstr = ffi_utils::as_ref_str_to_c_string(comment)?;
        log::debug!(
            "FsTab::set_trailing_comments setting trailing comment to {:?}",
            comment
        );

        let result =
            unsafe { libmount::mnt_table_set_trailing_comment(self.inner, comment_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::set_trailing_comments set trailing comment to {:?}",
                    comment
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set trailing comment to {:?}", comment);
                log::debug!("FsTab::set_trailing_comments {}. libmount::mnt_table_set_trailing_comment returned error code: {:?}", err_msg, code);

                Err(FsTabError::Config(err_msg))
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

    fn enable_comments(ptr: *mut libmount::libmnt_table, enable: bool) {
        let op = if enable { 1 } else { 0 };
        unsafe { libmount::mnt_table_enable_comments(ptr, op) }
    }

    /// Keeps comment lines when importing table entries from files.
    pub fn import_with_comments(&mut self) {
        log::debug!("FsTab::import_with_comments enabling comment parsing");

        Self::enable_comments(self.inner, true)
    }

    /// Skips comment lines when importing table entries from files.
    pub fn import_without_comments(&mut self) {
        log::debug!("FsTab::import_without_comments disabling comment parsing");

        Self::enable_comments(self.inner, false)
    }

    /// Imports entries from files with a `.fstab` extension in a given directory. File names are
    /// ordered by [strverscmp(3)](https://www.man7.org/linux/man-pages/man3/strverscmp.3.html)
    /// before their contents are added to the table.
    ///
    /// **Note:**
    /// - this method ignores any dotfile in the directory.
    /// - by default, comment lines are ignored during importation. If you want them included, call
    ///   [`FsTab::import_with_comments`] **before** invoking this method.
    pub fn import_directory<T>(&mut self, dir_path: T) -> Result<(), FsTabError>
    where
        T: AsRef<Path>,
    {
        let dir_path = dir_path.as_ref();
        let dir_path_cstr = ffi_utils::as_ref_path_to_c_string(dir_path)?;
        log::debug!(
            "FsTab::import_dir importing table entries from files in {:?}",
            dir_path
        );

        let result = unsafe { libmount::mnt_table_parse_dir(self.inner, dir_path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::import_directory imported table entries from files in {:?}",
                    dir_path
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to import table entries from files in {:?}",
                    dir_path
                );
                log::debug!("FsTab::import_directory {}. libmount::mnt_table_parse_dir returned error code: {:?}", err_msg, code);

                Err(FsTabError::Import(err_msg))
            }
        }
    }

    /// Parses `/etc/fstab` or the file specified by the environment variable `LIBMOUNT_FSTAB`,
    /// then appends the entries it collects to this `FsTab`.
    ///
    /// **Note:** by default, comment lines are ignored during importation. If you want them
    /// included, call [`FsTab::import_with_comments`] **before** invoking this method.
    pub fn import_etc_fstab(&mut self) -> Result<(), FsTabError> {
        log::debug!("FsTab::import_etc_fstab import entries from /etc/fstab");

        let result = unsafe { libmount::mnt_table_parse_fstab(self.inner, std::ptr::null()) };

        match result {
            0 => {
                log::debug!("FsTab::import_etc_fstab imported entries from /etc/fstab");

                Ok(())
            }
            code => {
                let err_msg = "failed to import entries from /etc/fstab".to_owned();
                log::debug!("FsTab::import_etc_fstab {}. libmount::mnt_table_parse_fstab returned error code: {:?}", err_msg, code);

                Err(FsTabError::Import(err_msg))
            }
        }
    }

    /// Parses the given file, then appends the entries it collected to the table.
    ///
    /// **Note:**
    /// - by default, comment lines are ignored during import. If you want them included, call
    ///   [`FsTab::import_with_comments`] **before** invoking this method.
    /// - the parser ignores lines with syntax errors. It will report defective lines to the caller
    ///   through an error callback function.
    ///
    // FIXME Defective lines are reported to the caller by the errcb() function (see mnt_table_set_parser_errcb()).
    // can not currently wrap the function `mnt_table_set_parser_errcb`
    pub fn import_file<T>(&mut self, file_path: T) -> Result<(), FsTabError>
    where
        T: AsRef<Path>,
    {
        let file_path = file_path.as_ref();
        let file_path_cstr = ffi_utils::as_ref_path_to_c_string(file_path)?;
        log::debug!(
            "FsTab::import_file importing table entries from file {:?}",
            file_path
        );

        let result = unsafe { libmount::mnt_table_parse_file(self.inner, file_path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::import_file imported table entries from file {:?}",
                    file_path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to import table entries from file {:?}", file_path);
                log::debug!("FsTab::import_file {}. libmount::mnt_table_parse_file returned error code: {:?}", err_msg, code);

                Err(FsTabError::Import(err_msg))
            }
        }
    }

    /// Parses the given [`File`] saving debug messages, and any parsing error to the
    /// `parsing_errors` file.
    ///
    /// **Note:** by default, comment lines are ignored during importation. If you want them
    /// included, call [`FsTab::import_with_comments`] **before** invoking this method.
    pub fn import_from_stream<T>(&mut self, file: &File, parsing_errors: T) -> io::Result<()>
    where
        T: AsRef<Path>,
    {
        if ffi_utils::is_open_read_only(file)? || ffi_utils::is_open_read_write(file)? {
            let parsing_errors = parsing_errors.as_ref();
            let path_cstr = ffi_utils::as_ref_path_to_c_string(parsing_errors)?;
            let file_stream = ffi_utils::read_only_c_file_stream_from(file)?;

            log::debug!(
            "FsTab::import_from_stream importing entries from file stream, saving parsing errors to {:?}",
            parsing_errors
        );

            let result = unsafe {
                libmount::mnt_table_parse_stream(
                    self.inner,
                    file_stream as *mut _,
                    path_cstr.as_ptr(),
                )
            };

            match result {
                0 => {
                    log::debug!("FsTab::import_from_stream imported entries from file stream, saving parsing errors to {:?}", parsing_errors);

                    Ok(())
                }
                code => {
                    let err_msg = format!(
                        "failed to import entries from file stream, saving parsing errors to {:?}",
                        parsing_errors
                    );
                    log::debug!("FsTab::import_from_stream {}. libmount::mnt_table_parse_stream returned error code: {:?}", err_msg, code);

                    Err(io::Error::from_raw_os_error(code))
                }
            }
        } else {
            let err_msg = "missing read permission for given file stream".to_owned();
            log::debug!("FsTab::import_from_stream {}", err_msg);

            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        }
    }

    /// Appends the content of the function parameter to the introduction comments.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// use rsmount::tables::FsTab;
    /// use rsmount::entries::FsTabEntry;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///     let mut fstab = FsTab::new()?;
    ///
    ///     // # /etc/fstab
    ///     fstab.set_intro_comments("# /etc/fstab\n")?;
    ///
    ///     let actual = fstab.intro_comments().unwrap();
    ///     let expected = "# /etc/fstab\n";
    ///     assert_eq!(actual, expected);
    ///
    ///     // Append a new comment line
    ///     fstab.append_to_intro_comments("# Example from scratch\n")?;
    ///
    ///     let actual = fstab.intro_comments().unwrap();
    ///     let expected = "# /etc/fstab\n# Example from scratch\n";
    ///     assert_eq!(actual, expected);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn append_to_intro_comments<T>(&mut self, comment: T) -> Result<(), FsTabError>
    where
        T: AsRef<str>,
    {
        let comment = comment.as_ref();
        let comment_cstr = ffi_utils::as_ref_str_to_c_string(comment)?;
        log::debug!(
            "FsTab::append_to_intro_comments appending to intro comment: {:?}",
            comment
        );

        let result =
            unsafe { libmount::mnt_table_append_intro_comment(self.inner, comment_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::append_to_intro_comments appended to intro comment: {:?}",
                    comment
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to append to intro comment: {:?}", comment);
                log::debug!("FsTab::append_to_intro_comments {}. libmount::mnt_table_append_intro_comment returned error code: {:?}", err_msg, code);

                Err(FsTabError::Action(err_msg))
            }
        }
    }

    /// Appends the content of the function parameter to the trailing comments.
    pub fn append_to_trailing_comments<T>(&mut self, comment: T) -> Result<(), FsTabError>
    where
        T: AsRef<str>,
    {
        let comment = comment.as_ref();
        let comment_cstr = ffi_utils::as_ref_str_to_c_string(comment)?;
        log::debug!(
            "FsTab::append_to_trailing_comments appending to trailing comment: {:?}",
            comment
        );

        let result = unsafe {
            libmount::mnt_table_append_trailing_comment(self.inner, comment_cstr.as_ptr())
        };

        match result {
            0 => {
                log::debug!(
                    "FsTab::append_to_trailing_comments appended to trailing comment: {:?}",
                    comment
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to append to trailing comment: {:?}", comment);
                log::debug!("FsTab::append_to_trailing_comments {}. libmount::mnt_table_append_trailing_comment returned error code: {:?}", err_msg, code);

                Err(FsTabError::Action(err_msg))
            }
        }
    }

    /// Keeps intro/trailing comment lines when exporting the table to disk.
    pub fn export_with_comments(&mut self) {
        log::debug!("FsTab::export_with_comments enabling comment parsing");

        Self::enable_comments(self.inner, true)
    }

    /// Skips intro/trailing comment lines when exporting the table to disk.
    pub fn export_without_comments(&mut self) {
        log::debug!("FsTab::export_without_comments disabling comment parsing");

        Self::enable_comments(self.inner, false)
    }

    fn filter_by<F>(table: &mut Self, flags: u32, cmp_fn: F) -> Result<(), FsTabError>
    where
        F: FnMut(&FsTabEntry, &FsTabEntry) -> Ordering,
    {
        #[doc(hidden)]
        /// Comparison function to identify duplicate entries.
        unsafe extern "C" fn compare<F>(
            table: *mut libmount::libmnt_table,
            this: *mut libmount::libmnt_fs,
            other: *mut libmount::libmnt_fs,
        ) -> libc::c_int
        where
            F: FnMut(&FsTabEntry, &FsTabEntry) -> Ordering,
        {
            // Temporarily increments the pointer's reference counter which will be decremented when
            // entry goes out of scope. This prevents us from freeing the data structure while it is
            // still in use.
            let this = FsTabEntry::borrow_ptr(this);
            let other = FsTabEntry::borrow_ptr(other);

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
                        log::debug!("FsTab::filter_by removed duplicates");
                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Ok(())
                    }
                    code => {
                        let err_msg = "failed to remove duplicates from table".to_owned();
                        log::debug!( "FsTab::filter_by {err_msg}. libmount::mnt_table_uniq_fs returned error code: {code:?}");

                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Err(FsTabError::Deduplicate(err_msg))
                    }
                }
            }
            code => {
                let err_msg = "failed to set the comparison function as userdata".to_owned();
                log::debug!( "FsTab::filter_by {err_msg}. libmount::mnt_table_uniq_fs returned error code: {code:?}");

                // Deallocate closure on the heap.
                let _ = unsafe { Box::from_raw(user_data) };

                Err(FsTabError::Deduplicate(err_msg))
            }
        }
    }

    /// Removes the duplicate entries in this table keeping the first occurrence of an
    /// entry for which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:** this method preserves the index order of the entries in the table.
    pub fn dedup_first_by<F>(&mut self, cmp: F) -> Result<(), FsTabError>
    where
        F: FnMut(&FsTabEntry, &FsTabEntry) -> Ordering,
    {
        log::debug!("FsTab::dedup_first_by merging matching entries to the first occurrence");

        Self::filter_by(self, libmount::MNT_UNIQ_FORWARD, cmp)
    }

    /// Removes the duplicate entries in this table keeping the last occurrence of an
    /// entry for which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:** this method preserves the index order of the entries in the table.
    pub fn dedup_last_by<F>(&mut self, cmp: F) -> Result<(), FsTabError>
    where
        F: FnMut(&FsTabEntry, &FsTabEntry) -> Ordering,
    {
        log::debug!("FsTab::dedup_last_by merging matching entries to the last occurrence");
        static MNT_UNIQ_BACKWARD: u32 = 0;

        Self::filter_by(self, MNT_UNIQ_BACKWARD, cmp)
    }

    /// Appends a [`FsTabEntry`] to this `FsTab`.
    ///
    /// # Panics
    ///
    /// Panics if memory allocation for extending capacity fails.
    pub fn push(&mut self, element: FsTabEntry) {
        self.try_push(element).unwrap()
    }

    /// Tries to append a [`FsTabEntry`] to this `FsTab`.
    pub fn try_push(&mut self, element: FsTabEntry) -> Result<(), FsTabError> {
        log::debug!("FsTab::try_push adding a new table entry");

        let result = unsafe { libmount::mnt_table_add_fs(self.inner, element.inner) };

        match result {
            0 => {
                log::debug!("FsTab::push added a new table entry");

                Ok(())
            }
            code => {
                let err_msg = "failed to add a new table entry".to_owned();
                log::debug!(
                             "FsTab::try_push {err_msg}. libmount::mnt_table_add_fs returned error code: {code:?}"
                            );

                Err(FsTabError::Action(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Adds a new entry to the table before or after a specific table entry `pos`.
    ///
    /// - If `pos` is NULL and `before` is set to `true`, adds the new entry to the beginning of the table
    /// - If `pos` is NULL and `before` is set to `false`, adds the new entry to the end of the table
    fn insert_entry(
        table: &mut Self,
        after: bool,
        pos: *mut libmount::libmnt_fs,
        entry: *mut libmount::libmnt_fs,
    ) -> Result<(), FsTabError> {
        let op = if after { 1 } else { 0 };
        let op_str = if after {
            "after".to_owned()
        } else {
            "before".to_owned()
        };

        let result = unsafe { libmount::mnt_table_insert_fs(table.inner, op, pos, entry) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::insert_entry inserted new entry {} reference",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to insert new entry {} reference", op_str);
                log::debug!( "FsTab::insert_entry {err_msg}. libmount::mnt_table_insert_fs returned error code: {code:?}");

                Err(FsTabError::Action(err_msg))
            }
        }
    }

    /// Prepends a new element to the `FsTab`.
    ///
    /// # Panics
    ///
    /// Panics if memory allocation fails.
    pub fn push_front(&mut self, element: FsTabEntry) {
        log::debug!("FsTab::push_front prepending new entry");

        Self::insert_entry(self, true, std::ptr::null_mut(), element.inner).unwrap()
    }

    /// Tries to prepend a new element to the `FsTab`.
    pub fn try_push_front(&mut self, element: FsTabEntry) -> Result<(), FsTabError> {
        log::debug!("FsTab::try_push_front prepending new entry");

        Self::insert_entry(self, true, std::ptr::null_mut(), element.inner)
    }

    /// Inserts an element at position `index` within the table, shifting all elements after it to
    /// the bottom.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn insert(&mut self, index: usize, element: FsTabEntry) {
        self.try_insert(index, element).unwrap()
    }

    /// Tries to insert an element at position `index` within the table, shifting all elements
    /// after it to the bottom.
    pub fn try_insert(&mut self, index: usize, element: FsTabEntry) -> Result<(), FsTabError> {
        log::debug!(
            "FsTab::try_insert inserting new entry at index: {:?}",
            index
        );

        let mut iter = FsTabIter::new(self)?;

        match iter.nth(index) {
            Some(position) => Self::insert_entry(self, false, position.inner, element.inner),
            None => {
                let err_msg = format!(
                    "failed to insert element at index: {:?}. Index out of bounds.",
                    index
                );
                log::debug!("FsTab::try_insert {err_msg}");

                Err(FsTabError::IndexOutOfBounds(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Moves an `entry` from `source_table` to `dest_table` before or after a given position.
    ///
    /// - If `position` is NULL and `before` is set to `true`, transfers the `entry` to the beginning of the `dest_table`.
    /// - If `position` is NULL and `before` is set to `false`, transfers the `entry` to the end of the `dest_table`.
    fn move_entry(
        after: bool,
        source_table: *mut libmount::libmnt_table,
        entry: *mut libmount::libmnt_fs,
        dest_table: *mut libmount::libmnt_table,
        position: *mut libmount::libmnt_fs,
    ) -> Result<(), FsTabError> {
        log::debug!("FsTab::move_entry transferring entry between tables");

        let op = if after { 1 } else { 0 };

        let result =
            unsafe { libmount::mnt_table_move_fs(source_table, dest_table, op, position, entry) };

        match result {
            0 => {
                log::debug!("FsTab::move_entry transferred entry between tables");

                Ok(())
            }
            code => {
                let err_msg = "failed to transfer entry between tables".to_owned();
                log::debug!(
 "FsTab::move_entry {err_msg}. libmount::mnt_table_move_fs returned error code: {code:?}"
                            );

                Err(FsTabError::Transfer(err_msg))
            }
        }
    }

    /// Transfers an element between two `FsTab`s, from `index` in the source table to
    /// `dest_index` in the destination table.
    ///
    /// # Examples
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// use std::path::Path;
    /// use rsmount::tables::FsTab;
    /// use rsmount::entries::FsTabEntry;
    /// use rsmount::device::BlockDevice;
    /// use rsmount::device::Pseudo;
    /// use rsmount::device::Source;
    /// use rsmount::device::Tag;
    /// use rsmount::fs::FileSystem;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///     let mut fstab = FsTab::new()?;
    ///
    ///     // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
    ///     let uuid = Tag::try_from("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
    ///     let entry1 = FsTabEntry::builder()
    ///         .source(uuid)
    ///         .target("/")
    ///         .file_system_type(FileSystem::Ext4)
    ///         // Comma-separated list of mount options.
    ///         .mount_options("rw,relatime")
    ///         // Interval, in days, between file system backups by the dump command on ext2/3/4
    ///         // file systems.
    ///         .backup_frequency(0)
    ///         // Order in which file systems are checked by the `fsck` command.
    ///         .fsck_checking_order(1)
    ///         .build()?;
    ///
    ///     // /dev/usbdisk /media/usb vfat noauto 0 0
    ///     let block_device = BlockDevice::try_from("/dev/usbdisk")?;
    ///     let entry2 = FsTabEntry::builder()
    ///         .source(block_device)
    ///         .target("/media/usb")
    ///         .file_system_type(FileSystem::VFAT)
    ///         .mount_options("noauto")
    ///         .backup_frequency(0)
    ///         .fsck_checking_order(0)
    ///         .build()?;
    ///
    ///     fstab.push(entry1);
    ///     fstab.push(entry2);
    ///
    ///     let mut other_fstab = FsTab::new()?;
    ///     // none /tmp tmpfs nosuid,nodev 0 0
    ///     let entry3 = FsTabEntry::builder()
    ///         .source(Pseudo::None)
    ///         .target("/tmp")
    ///         .file_system_type(FileSystem::Tmpfs)
    ///         .mount_options("nosuid,nodev")
    ///         .backup_frequency(0)
    ///         .fsck_checking_order(0)
    ///         .build()?;
    ///
    ///     other_fstab.push(entry3);
    ///
    ///     // Transfer `entry3` from `other_fstab` to the end of `fstab`
    ///     assert_eq!(fstab.len(), 2);
    ///     assert_eq!(other_fstab.len(), 1);
    ///
    ///     other_fstab.transfer(0, &mut fstab, 2)?;
    ///
    ///     assert_eq!(fstab.len(), 3);
    ///     assert!(other_fstab.is_empty());
    ///
    ///     assert_eq!(fstab[2].source(), Some(Source::from(Pseudo::None)));
    ///     assert_eq!(fstab[2].target(), Some(Path::new("/tmp")));
    ///     assert_eq!(fstab[2].mount_options(), Some("nosuid,nodev"));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn transfer(
        &mut self,
        index: usize,
        destination: &mut FsTab,
        dest_index: usize,
    ) -> Result<(), FsTabError> {
        let mut iter = FsTabIter::new(self)?;

        match iter.nth(index) {
            Some(entry) if dest_index == 0 => {
                log::debug!(
 "FsTab::transfer transferring element a index: {:?} to start of destination table",
                                index
                            );

                Self::move_entry(
                    true,
                    self.inner,
                    entry.inner,
                    destination.inner,
                    std::ptr::null_mut(),
                )
            }
            Some(entry) if dest_index == destination.len() => {
                log::debug!(
 "FsTab::transfer transferring element a index: {:?} to end of destination table",
                                index
                            );

                Self::move_entry(
                    false,
                    self.inner,
                    entry.inner,
                    destination.inner,
                    std::ptr::null_mut(),
                )
            }
            Some(element) => {
                let mut iter_dest = FsTabIter::new(destination)?;
                match iter_dest.nth(dest_index) {
                    Some(position) => {
                        log::debug!( "FsTab::transfer transferring element at index {:?} to destination at index {:?}", index, dest_index);

                        Self::move_entry(
                            false,
                            self.inner,
                            element.inner,
                            destination.inner,
                            position.inner,
                        )
                    }
                    None => {
                        let err_msg = format!(
                                "failed to transfer element at index {:?} to index {:?} in destination table. Index out of bounds.", index,
                                dest_index
                            );
                        log::debug!("FsTab::transfer {err_msg}");

                        Err(FsTabError::IndexOutOfBounds(err_msg))
                    }
                }
            }
            None => {
                let err_msg = format!(
                    "failed to access element at index {:?} in source table. Index out of bounds.",
                    index
                );
                log::debug!("FsTab::transfer {err_msg}");

                Err(FsTabError::IndexOutOfBounds(err_msg))
            }
        }
    }

    /// Removes the given `element` from the table.
    ///
    /// # Panics
    ///
    /// May panic if the index is out of bounds.
    pub fn remove(&mut self, index: usize) -> FsTabEntry {
        log::debug!("FsTab::remove removing entry from table");

        let err_msg = format!("failed to find entry at index: {:?}", index);
        let element: &FsTabEntry = self
            .get(index)
            .ok_or(Err::<&FsTabEntry, FsTabError>(
                FsTabError::IndexOutOfBounds(err_msg),
            ))
            .unwrap();

        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed() -> ! {
            panic!("cannot remove table entry. Not found");
        }

        // increment reference counter to prevent mnt_table_remove from deallocating the underlying
        // table entry
        let borrowed = FsTabEntry::borrow_ptr(element.inner);

        let result = unsafe { libmount::mnt_table_remove_fs(self.inner, element.inner) };

        match result {
            0 => {
                log::debug!("FsTab::remove removed entry from table");

                borrowed
            }
            code => {
                let err_msg = "failed to remove entry from table".to_owned();
                log::debug!(
                    "FsTab::remove {}. libmount::mnt_table_remove_fs returned error code: {:?}",
                    err_msg,
                    code
                );

                // the element is not in the table, so we decrement its reference counter by
                // dropping it to cancel out the increment performed by FsTabEntry::borrow_ptr
                drop(borrowed);
                assert_failed()
            }
        }
    }

    /// Removes all table entries.
    pub fn clear(&mut self) -> Result<(), FsTabError> {
        log::debug!("FsTab::clear removing all table entries");

        unsafe {
            match libmount::mnt_reset_table(self.inner) {
                0 => {
                    log::debug!("FsTab::clear removed all table entries");
                    self.collect_garbage();

                    Ok(())
                }
                code => {
                    let err_msg = "failed to remove all table entries".to_owned();
                    log::debug!(
                                 "FsTab::clear {err_msg}. libmount::mnt_reset_table returned error code: {code:?}"
                                );

                    Err(FsTabError::Action(err_msg))
                }
            }
        }
    }

    /// Saves this table's entries to a file.
    pub fn write_file<T>(&mut self, file_path: T) -> Result<(), FsTabError>
    where
        T: AsRef<Path>,
    {
        let file_path = file_path.as_ref();
        let file_path_cstr = ffi_utils::as_ref_path_to_c_string(file_path)?;
        log::debug!("FsTab::write_file saving table content to {:?}", file_path);

        let result =
            unsafe { libmount::mnt_table_replace_file(self.inner, file_path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!("FsTab::write_file saved table content to {:?}", file_path);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to save table content to {:?}", file_path);
                log::debug!( "FsTab::write_file {err_msg}. libmount::mnt_table_replace_file returned error code: {code:?}");

                Err(FsTabError::Export(err_msg))
            }
        }
    }

    /// Writes this table's entries to a file stream.
    pub fn write_stream(&mut self, file_stream: &mut File) -> io::Result<()> {
        log::debug!("FsTab::write_stream writing mount table content to file stream");

        if ffi_utils::is_open_write_only(file_stream)?
            || ffi_utils::is_open_read_write(file_stream)?
        {
            let file = ffi_utils::write_only_c_file_stream_from(file_stream)?;

            let result = unsafe { libmount::mnt_table_write_file(self.inner, file as *mut _) };
            match result {
                0 => {
                    log::debug!("FsTab::write_stream wrote mount table content to file stream");

                    Ok(())
                }
                code => {
                    let err_msg = "failed to write mount table content to file stream".to_owned();
                    log::debug!( "FsTab::write_stream {err_msg}. libmount::mnt_table_write_file  returned error code: {code:?}");

                    Err(io::Error::from_raw_os_error(code))
                }
            }
        } else {
            let err_msg = "you do not have permission to write in this file stream".to_owned();
            log::debug!("FsTab::write_stream {err_msg}");

            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        }
    }

    //---- END mutators

    //---- BEGIN predicates

    /// Returns `true` if this `FsTab` contains a element matching **exactly** the given `element`.
    pub fn contains(&self, element: &FsTabEntry) -> bool {
        let state = unsafe { libmount::mnt_table_find_fs(self.inner, element.inner) > 0 };
        log::debug!("FsTab::contains value: {:?}", state);

        state
    }

    /// Returns `true` if the table has length of 0.
    pub fn is_empty(&self) -> bool {
        let state = unsafe { libmount::mnt_table_is_empty(self.inner) == 1 };
        log::debug!("FsTab::is_empty value: {:?}", state);

        state
    }

    /// Returns `true` if this `FsTab` is set to import table entries with their comments.
    pub fn is_importing_comments(&self) -> bool {
        let state = unsafe { libmount::mnt_table_with_comments(self.inner) == 1 };
        log::debug!("FsTab::is_importing_comments value: {:?}", state);

        state
    }

    /// Returns `true` if this `FsTab` is set to export intro/trailing comments.
    pub fn is_exporting_comments(&self) -> bool {
        let state = unsafe { libmount::mnt_table_with_comments(self.inner) == 1 };
        log::debug!("FsTab::is_exporting_comments value: {:?}", state);

        state
    }

    //---- END predicates
}

impl fmt::Display for FsTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output: Vec<String> = vec![];
        if let Some(intro) = self.intro_comments() {
            output.push(intro.to_string());
        }

        for line in self.iter() {
            output.push(line.to_string());
        }

        if let Some(trailing) = self.trailing_comments() {
            output.push(trailing.to_string());
        }

        write!(f, "{}", output.join("\n"))
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::core::device::BlockDevice;
    use crate::core::device::Pseudo;
    use crate::core::device::Source;
    use crate::core::device::Tag;
    use crate::core::fs::FileSystem;
    use pretty_assertions::{assert_eq, assert_ne};
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Seek, SeekFrom};
    use std::str::FromStr;
    use tempfile::{tempdir, tempfile};

    #[test]
    fn fs_tab_a_new_table_is_empty() -> crate::Result<()> {
        let fs_tab = FsTab::new()?;

        assert!(fs_tab.is_empty());

        Ok(())
    }

    #[test]
    fn fs_tab_an_empty_table_has_no_first_element() -> crate::Result<()> {
        let fs_tab = FsTab::new()?;

        let actual = fs_tab.first();

        assert!(actual.is_none());

        Ok(())
    }

    #[test]
    fn fs_tab_an_empty_table_has_no_last_element() -> crate::Result<()> {
        let fs_tab = FsTab::new()?;

        let actual = fs_tab.last();

        assert!(actual.is_none());

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn fs_tab_indexing_an_empty_table_triggers_a_panic() {
        let fs_tab = FsTab::new().unwrap();

        let _ = fs_tab[0];
    }

    #[test]
    fn fs_tab_push_adds_an_element_to_a_table() -> crate::Result<()> {
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry);

        assert_eq!(fs_tab.len(), 1);

        Ok(())
    }

    #[test]
    fn fs_tab_push_front_adds_an_element_at_the_head_of_the_table() -> crate::Result<()> {
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push_front(entry1);
        fs_tab.push_front(entry2);

        let first = fs_tab.first().unwrap();
        let last = fs_tab.last().unwrap();

        assert_eq!(fs_tab.len(), 2);
        assert_eq!(first.inner, entry2_inner);
        assert_eq!(last.inner, entry1_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_a_table_of_size_1_has_the_same_first_and_last_element() -> crate::Result<()> {
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry);

        let first = fs_tab.first();
        let last = fs_tab.last();

        assert_eq!(first, last);

        Ok(())
    }

    #[test]
    fn fs_tab_finds_the_first_predicate_match_from_the_top() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b /media/disk xfs noauto 0 0
        let uuid = Tag::from_str("UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b")?;
        let entry3 = FsTabEntry::builder()
            .source(uuid)
            .target("/media/disk")
            .file_system_type(FileSystem::XFS)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry4 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        // /dev/usbdisk /media/usb vfat noauto 0 0
        // UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b /media/disk xfs noauto 0 0
        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.push(entry3);
        fs_tab.push(entry4);

        // Find the first entry with an `ext4` or `xfs` file system (going from last to first).
        let element = fs_tab.find_first(|element| element.has_any_fs_type("ext4,xfs"));

        // `entry1` is the first to match...
        assert!(element.is_some());
        let element = element.unwrap();

        // ...it has an `ext4` file system type,
        let actual = element.file_system_type();
        let fs = FileSystem::Ext4;
        let expected = Some(fs);
        assert_eq!(actual, expected);

        // ...and is mounted at `/`.
        let target = Path::new("/");
        let actual = element.target();
        let expected = Some(target);
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn fs_tab_finds_the_first_predicate_match_from_the_bottom() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b /media/disk xfs noauto 0 0
        let uuid = Tag::from_str("UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b")?;
        let entry3 = FsTabEntry::builder()
            .source(uuid)
            .target("/media/disk")
            .file_system_type(FileSystem::XFS)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry4 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        // /dev/usbdisk /media/usb vfat noauto 0 0
        // UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b /media/disk xfs noauto 0 0
        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.push(entry3);
        fs_tab.push(entry4);

        // Find the first entry with an `ext4` or `xfs` file system (going from last to first).
        let element = fs_tab.find_back_first(|element| element.has_any_fs_type("ext4,xfs"));

        // `entry3` is the first to match...
        assert!(element.is_some());
        let element = element.unwrap();

        // ...it has an `xfs` file system type,
        let actual = element.file_system_type();
        let fs = FileSystem::XFS;
        let expected = Some(fs);
        assert_eq!(actual, expected);

        // ...and is mounted at `/media/disk`.
        let target = Path::new("/media/disk");
        let actual = element.target();
        let expected = Some(target);
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn fs_tab_can_can_advance_its_iterator_to_a_given_position() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.push(entry3);

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();

        iter.advance_to(2).unwrap();
        let actual = iter.next();
        assert!(actual.is_none());

        iter.advance_to(0).unwrap();
        let actual = iter.next().unwrap().inner;
        assert_eq!(actual, entry2_inner);

        iter.advance_to(1).unwrap();
        let actual = iter.next().unwrap().inner;
        assert_eq!(actual, entry3_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_iterate_forwards_over_table_entries() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.push(entry3);

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();
        let first_inner = iter.next().unwrap().inner;
        let second_inner = iter.next().unwrap().inner;
        let third_inner = iter.next().unwrap().inner;

        assert_eq!(first_inner, entry1_inner);
        assert_eq!(second_inner, entry2_inner);
        assert_eq!(third_inner, entry3_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_iterate_backwards_over_table_entries() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.push(entry3);

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();
        let first_inner = iter.next_back().unwrap().inner;
        let second_inner = iter.next_back().unwrap().inner;
        let third_inner = iter.next_back().unwrap().inner;

        assert_eq!(first_inner, entry3_inner);
        assert_eq!(second_inner, entry2_inner);
        assert_eq!(third_inner, entry1_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_iterate_alternately_forwards_then_backwards_over_table_entries(
    ) -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.push(entry3);

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();
        let first_inner = iter.next().unwrap().inner;
        let second_inner = iter.next_back().unwrap().inner;
        let third_inner = iter.next().unwrap().inner;
        let fourth = iter.next_back();
        let fifth = iter.next();

        assert_eq!(first_inner, entry1_inner);
        assert_eq!(second_inner, entry3_inner);
        assert_eq!(third_inner, entry2_inner);
        assert!(fourth.is_none());
        assert!(fifth.is_none());

        Ok(())
    }

    #[test]
    fn fs_tab_can_iterate_alternately_backwards_then_forwards_over_table_entries(
    ) -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.push(entry3);

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();
        let first_inner = iter.next_back().unwrap().inner;
        let second_inner = iter.next().unwrap().inner;
        let third_inner = iter.next_back().unwrap().inner;
        let fourth = iter.next();
        let fifth = iter.next_back();

        assert_eq!(first_inner, entry3_inner);
        assert_eq!(second_inner, entry1_inner);
        assert_eq!(third_inner, entry2_inner);
        assert!(fourth.is_none());
        assert!(fifth.is_none());

        Ok(())
    }

    #[test]
    fn fs_tab_can_index_into_a_table() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        let expected = entry.inner;

        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry);

        let actual = fs_tab[0].inner;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn fs_tab_can_insert_an_element_at_a_predefined_position() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.insert(1, entry3);

        assert_eq!(fs_tab.len(), 3);
        let first_inner = fs_tab[0].inner;
        let second_inner = fs_tab[1].inner;
        let third_inner = fs_tab[2].inner;

        assert_eq!(first_inner, entry1_inner);
        assert_eq!(second_inner, entry3_inner);
        assert_eq!(third_inner, entry2_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_remove_an_element_from_a_table() -> crate::Result<()> {
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry);

        assert_eq!(fs_tab.len(), 1);

        let item = fs_tab.remove(0);

        let actual = item.tag().unwrap();
        let expected: Tag = "UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f".parse()?;
        assert_eq!(actual, expected);

        let actual = item.file_system_type().unwrap();
        let expected = FileSystem::Ext4;
        assert_eq!(actual, expected);

        let actual = item.mount_options().unwrap();
        let expected = "rw,relatime";
        assert_eq!(actual, expected);

        assert_eq!(fs_tab.is_empty(), true);

        Ok(())
    }

    #[test]
    fn fs_tab_can_transfer_an_element_between_tables_to_destination_start() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut source_table = FsTab::new()?;
        source_table.push(entry1);

        let mut dest_table = FsTab::new()?;
        dest_table.push(entry2);
        dest_table.push(entry3);

        // Before transfer
        assert_eq!(source_table.len(), 1);
        assert_eq!(dest_table.len(), 2);

        source_table.transfer(0, &mut dest_table, 0)?;

        // After transfer
        assert_eq!(source_table.is_empty(), true);

        assert_eq!(dest_table.len(), 3);

        let first_inner = dest_table[0].inner;
        let second_inner = dest_table[1].inner;
        let third_inner = dest_table[2].inner;

        assert_eq!(first_inner, entry1_inner);
        assert_eq!(second_inner, entry2_inner);
        assert_eq!(third_inner, entry3_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_transfer_an_element_between_tables_to_destination_middle() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut source_table = FsTab::new()?;
        source_table.push(entry1);

        let mut dest_table = FsTab::new()?;
        dest_table.push(entry2);
        dest_table.push(entry3);

        // Before transfer
        assert_eq!(source_table.len(), 1);
        assert_eq!(dest_table.len(), 2);

        source_table.transfer(0, &mut dest_table, 1)?;

        // After transfer
        assert_eq!(source_table.is_empty(), true);

        assert_eq!(dest_table.len(), 3);

        let first_inner = dest_table[0].inner;
        let second_inner = dest_table[1].inner;
        let third_inner = dest_table[2].inner;

        assert_eq!(first_inner, entry2_inner);
        assert_eq!(second_inner, entry1_inner);
        assert_eq!(third_inner, entry3_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_transfer_an_element_between_tables_to_destination_end() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut source_table = FsTab::new()?;
        source_table.push(entry1);

        let mut dest_table = FsTab::new()?;
        dest_table.push(entry2);
        dest_table.push(entry3);

        // Before transfer
        assert_eq!(source_table.len(), 1);
        assert_eq!(dest_table.len(), 2);

        source_table.transfer(0, &mut dest_table, 2)?;

        // After transfer
        assert_eq!(source_table.is_empty(), true);

        assert_eq!(dest_table.len(), 3);

        let first_inner = dest_table[0].inner;
        let second_inner = dest_table[1].inner;
        let third_inner = dest_table[2].inner;

        assert_eq!(first_inner, entry2_inner);
        assert_eq!(second_inner, entry3_inner);
        assert_eq!(third_inner, entry1_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_writes_to_a_file_stream() -> crate::Result<()> {
        let mut fs_tab = FsTab::new()?;

        // # /etc/fstab
        fs_tab.set_intro_comments("# /etc/fstab\n")?;
        // # Example from scratch
        fs_tab.append_to_intro_comments("# Example from scratch\n")?;
        //
        fs_tab.append_to_intro_comments("\n")?;

        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.push(entry3);

        assert_eq!(fs_tab.len(), 3);

        let mut tmpfile: File = tempfile().unwrap();

        // Write to disk
        fs_tab.export_with_comments();
        fs_tab.write_stream(&mut tmpfile).unwrap();

        // Seek to start
        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        // Read back
        let mut actual = String::new();
        tmpfile.read_to_string(&mut actual).unwrap();

        let expected = "# /etc/fstab\n# Example from scratch\n\nUUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1\n/dev/usbdisk /media/usb vfat noauto 0 0\nnone /tmp tmpfs nosuid,nodev 0 0\n";
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn fs_tab_does_not_write_to_read_only_file_stream() {
        let mut fs_tab = FsTab::new().unwrap();

        // # /etc/fstab
        fs_tab.set_intro_comments("# /etc/fstab\n").unwrap();
        // # Example from scratch
        fs_tab
            .append_to_intro_comments("# Example from scratch\n")
            .unwrap();
        //
        fs_tab.append_to_intro_comments("\n").unwrap();

        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").unwrap();
        let entry1 = FsTabEntry::builder()
            .source(uuid)
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()
            .unwrap();

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").unwrap();
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()
            .unwrap();

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None)
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()
            .unwrap();

        fs_tab.push(entry1);
        fs_tab.push(entry2);
        fs_tab.push(entry3);

        assert_eq!(fs_tab.len(), 3);

        let tmpdir = tempdir().unwrap();
        let tmpfile_path = tmpdir.path().join("read-only-file");
        let file = File::create(&tmpfile_path).unwrap();
        drop(file);
        let mut tmpfile: File = OpenOptions::new().read(true).open(&tmpfile_path).unwrap();

        // Write to disk
        fs_tab.export_with_comments();
        fs_tab.write_stream(&mut tmpfile).unwrap();
    }
}
