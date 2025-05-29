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
use crate::core::entries::UTabEntry;
use crate::core::errors::UTabError;
use crate::core::errors::UTabIterError;
use crate::core::iter::Direction;
use crate::core::iter::GenIterator;
use crate::core::iter::UTabIter;
use crate::core::iter::UTabIterMut;
use crate::ffi_utils;
use crate::owning_ref_from_ptr;
use crate::tables::GcItem;
use crate::tables::MountOption;
use crate::tables::ParserFlow;

/// An in-memory representation of `/run/mount/utab`.
///
///  # `/run/mount/utab`
///
///  ```text
///  SRC=/dev/vda TARGET=/mnt ROOT=/ OPTS=x-initrd.mount
///  ```
///
/// // FIXME get a definition for each item
/// // from source file <https://github.com/util-linux/util-linux/blob/stable/v2.39/libmount/src/tab_parse.c#L310>
///   - **ID**:
///   - **SRC**: the mounted device,
///   - **TARGET**: the device's mount point,
///   - **ROOT**:
///   - **BINDSRC**: the source of a bind mount,
///   - **OPTS**: mount options,
///   - **ATTRS**: options independent from those used by the [`mount`
///     syscall](https://manpages.org/mount/2) and [`mount` command](https://manpages.org/mount/8).
///     They are neither sent to the kernel, nor interpreted by `libmount`. They are stored in
///     `/run/mount/utab`, and managed by `libmount` in userspace only.
#[derive(Debug)]
pub struct UTab {
    pub(crate) inner: *mut libmount::libmnt_table,
    pub(crate) gc: Vec<GcItem>,
}

impl Drop for UTab {
    fn drop(&mut self) {
        log::debug!("UTab::drop deallocating `UTab` instance");

        unsafe { libmount::mnt_unref_table(self.inner) };
        self.collect_garbage();
    }
}

impl AsRef<UTab> for UTab {
    #[inline]
    fn as_ref(&self) -> &UTab {
        self
    }
}

impl Index<usize> for UTab {
    type Output = UTabEntry;

    /// Performs the indexing (`container\[index]`) operation.
    fn index(&self, index: usize) -> &Self::Output {
        log::debug!("UTab::index getting item at index: {:?}", index);

        #[cold]
        #[inline(never)]
        #[track_caller]
        fn indexing_failed() -> ! {
            panic!("Index out of bounds");
        }

        let mut iter = UTabIter::new(self).unwrap();

        match iter.nth(index) {
            Some(item) => item,
            None => indexing_failed(),
        }
    }
}

impl IndexMut<usize> for UTab {
    /// Performs the mutable indexing (`container\[index]`) operation.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        log::debug!("UTab::index getting item at index: {:?}", index);

        #[cold]
        #[inline(never)]
        #[track_caller]
        fn indexing_failed() -> ! {
            panic!("Index out of bounds");
        }

        let mut iter = UTabIterMut::new(self).unwrap();

        match iter.nth(index) {
            Some(item) => item,
            None => indexing_failed(),
        }
    }
}

impl UTab {
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
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_table) -> UTab {
        Self {
            inner: ptr,
            gc: vec![],
        }
    }

    #[doc(hidden)]
    /// Borrows an instance.
    #[allow(dead_code)]
    pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_table) -> UTab {
        let mut table = Self::from_ptr(ptr);
        // We are virtually ceding ownership of this table which will be automatically
        // deallocated once it is out of scope, incrementing its reference counter protects it from
        // being freed prematurely.
        table.incr_ref_counter();

        table
    }

    //---- BEGIN constructors

    /// Creates a new empty `UTab`.
    pub fn new() -> Result<UTab, UTabError> {
        log::debug!("UTab::new creating a new `UTab` instance");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();

        unsafe { ptr.write(libmount::mnt_new_table()) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to create a new `UTab`".to_owned();
                log::debug!("UTab::new {err_msg}. libmount::mnt_new_table returned a NULL pointer");

                Err(UTabError::Creation(err_msg))
            }
            ptr => {
                log::debug!("UTab::new created a new `UTab` instance");
                let table = Self::from_ptr(ptr);

                Ok(table)
            }
        }
    }

    //---- END constructors

    /// Parses the given file, then appends the entries it collected to the table.
    ///
    /// **Note:**
    /// - by default, comment lines are ignored during import. If you want them included, call
    ///   [`UTab::import_with_comments`] **before** invoking this method.
    /// - the parser ignores lines with syntax errors. It will report defective lines to the caller
    ///   through an error callback function.
    ///
    // FIXME Defective lines are reported to the caller by the errcb() function (see mnt_table_set_parser_errcb()).
    // can not currently wrap the function `mnt_table_set_parser_errcb`
    fn import_file<T>(&mut self, file_path: T) -> Result<(), UTabError>
    where
        T: AsRef<Path>,
    {
        let file_path = file_path.as_ref();
        let file_path_cstr = ffi_utils::as_ref_path_to_c_string(file_path)?;
        log::debug!(
            "UTab::import_file importing table entries from file {:?}",
            file_path
        );

        let result = unsafe { libmount::mnt_table_parse_file(self.inner, file_path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "UTab::import_file imported table entries from file {:?}",
                    file_path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to import table entries from file {:?}", file_path);
                log::debug!("UTab::import_file {}. libmount::mnt_table_parse_file returned error code: {:?}", err_msg, code);

                Err(UTabError::Import(err_msg))
            }
        }
    }

    // FIXME there is no function to import the content of /run/mount/utab from upstream
    // using a workaround
    /// Parses the `/run/mount/utab` file, then appends the entries it
    /// collects to this `UTab`.
    pub fn import_utab(&mut self) -> Result<(), UTabError> {
        self.import_file("/run/mount/utab")
    }

    //---- BEGIN getters

    /// Returns a reference to the [`Cache`] instance associated with this `UTab`.
    pub fn cache(&self) -> Option<&Cache> {
        log::debug!("UTab::cache getting associated path and tag cache");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_cache>::zeroed();

        unsafe { ptr.write(libmount::mnt_table_get_cache(self.inner)) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("UTab::cache failed to get associated path and tag cache. libmount::mnt_table_get_cache returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!("UTab::cache got associated path and tag cache");
                let cache = owning_ref_from_ptr!(self, Cache, ptr);

                Some(cache)
            }
        }
    }

    /// Returns a reference to the first element of the `UTab`, or `None` if it is empty.
    pub fn first(&self) -> Option<&UTabEntry> {
        log::debug!("UTab::first getting reference to first table entry");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe { libmount::mnt_table_first_fs(self.inner, ptr.as_mut_ptr()) };
        match result {
            0 => {
                log::debug!("UTab::first got reference to first table entry");
                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(self, UTabEntry, ptr);

                Some(entry)
            }
            code => {
                log::debug!( "UTab::first failed to get reference to first table entry. libmount::mnt_table_first_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns a reference to the last element of the `UTab`, or `None` if it is empty.
    pub fn last(&self) -> Option<&UTabEntry> {
        log::debug!("UTab::last getting reference to last table entry");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe { libmount::mnt_table_last_fs(self.inner, ptr.as_mut_ptr()) };
        match result {
            0 => {
                log::debug!("UTab::last got reference to last table entry");
                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(self, UTabEntry, ptr);

                Some(entry)
            }
            code => {
                log::debug!( "UTab::last failed to get reference to last table entry. libmount::mnt_table_last_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns the index of a table entry.
    pub fn position(&self, entry: &UTabEntry) -> Option<usize> {
        log::debug!("UTab::position searching for an entry in the table");

        let result = unsafe { libmount::mnt_table_find_fs(self.inner, entry.inner) };
        match result {
            index if index > 0 => {
                log::debug!(
                    "UTab::position mount table contains entry at index: {:?}",
                    index
                );

                Some(index as usize)
            }
            code => {
                log::debug!( "UTab::position no matching entry in table: libmount::mnt_table_find_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns the number of entries in the table.
    pub fn len(&self) -> usize {
        let len = unsafe { libmount::mnt_table_get_nents(self.inner) };
        log::debug!("UTab::len value: {:?}", len);

        len as usize
    }

    /// Returns a reference to an element at `index`, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&UTabEntry> {
        log::debug!(
            "UTab::get_mut getting reference of item at index: {:?}",
            index
        );

        UTabIter::new(self)
            .ok()
            .and_then(|mut iter| iter.nth(index))
    }

    /// Returns a mutable reference to an element at `index`, or `None` if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut UTabEntry> {
        log::debug!(
            "UTab::get_mut getting mutable reference of item at index: {:?}",
            index
        );

        UTabIterMut::new(self)
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
    ) -> Option<&'a UTabEntry>
    where
        P: FnMut(&UTabEntry) -> bool,
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
            P: FnMut(&UTabEntry) -> bool,
        {
            // Temporarily increments the pointer's reference counter which will be decremented when
            // `entry` goes out of scope. This prevents us from freeing the data structure while it is
            // still in use.
            let entry = UTabEntry::borrow_ptr(entry_ptr);

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

                log::debug!("UTab::find_first_entry found first `UTabEntry` matching predicate");
                let entry_ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(table, UTabEntry, entry_ptr);

                Some(entry)
            }
            code => {
                // To ensure the closure is properly deallocated when this variable drops out
                // of scope.
                let _predicate = unsafe { Box::from_raw(data) };

                let err_msg = "failed to find `UTabEntry` matching predicate".to_owned();
                log::debug!( "UTab::find_first_entry {err_msg}. libmount::mnt_table_find_next_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`UTabEntry`] that
    /// satisfies the `predicate`.
    ///
    /// # Panics
    ///
    /// Panics if it can not create a forward iterator.
    pub fn find_first<P>(&mut self, predicate: P) -> Option<&UTabEntry>
    where
        P: FnMut(&UTabEntry) -> bool,
    {
        log::debug!(
            "UTab::find_first finding first table entry matching predicate while iterating Forward"
        );
        GenIterator::new(Direction::Forward)
            .ok()
            .and_then(|iterator| UTab::find_first_entry(self, iterator.inner, predicate))
    }

    /// Searches the table from **end** to **start**, and returns the first [`UTabEntry`] that
    /// satisfies the `predicate`.
    ///
    /// # Panics
    ///
    /// Panics if it can not create a backward iterator.
    pub fn find_back_first<P>(&mut self, predicate: P) -> Option<&UTabEntry>
    where
        P: FnMut(&UTabEntry) -> bool,
    {
        log::debug!( "UTab::find_back_first finding first table entry matching predicate while iterating Backward");
        GenIterator::new(Direction::Backward)
            .ok()
            .and_then(|iterator| UTab::find_first_entry(self, iterator.inner, predicate))
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`UTabEntry`] with fields matching the given
    /// source `path`.
    fn lookup_source<'a>(
        table: &mut Self,
        direction: Direction,
        source: &Source,
    ) -> Option<&'a UTabEntry> {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok()?;
        let source_ptr = if source.is_pseudo_fs() {
            // For pseudo file systems `libmount::mnt_table_find_source`
            // expects a NULL pointer path.
            std::ptr::null()
        } else {
            source_cstr.as_ptr()
        };

        log::debug!(
            "UTab::lookup_source searching {:?} for entry matching source {:?}",
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
                log::debug!( "UTab::lookup_source {err_msg}. libmount::mnt_table_find_source returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
                    "UTab::lookup_source found entry matching source {:?} while searching {:?}",
                    source,
                    direction
                );

                let entry = owning_ref_from_ptr!(table, UTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`UTabEntry`] with
    /// a field matching the given `source`.
    ///
    /// By default, a `UTab` will perform a cursory search, looking for an entry with an exact
    /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`UTab::set_cache`].
    ///
    /// **Note:** providing an **empty source** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_source(&mut self, source: &Source) -> Option<&UTabEntry> {
        let direction = Direction::Forward;
        log::debug!(
            "UTab::find_source searching {:?} for the first entry with a source matching {:?}",
            direction,
            source
        );

        Self::lookup_source(self, direction, source)
    }

    /// Searches the table from **end** to **start**, and returns the first [`UTabEntry`] with
    /// a field matching the given `source`.
    ///
    /// By default, a `UTab` will perform a cursory search, looking for an entry with an exact
    /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`UTab::set_cache`].
    ///
    /// **Note:** providing an **empty source** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_back_source(&mut self, source: &Source) -> Option<&UTabEntry> {
        let direction = Direction::Backward;
        log::debug!(
            "UTab::find_back_source searching {:?} for the first entry with a source matching {:?}",
            direction,
            source
        );

        Self::lookup_source(self, direction, source)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`UTabEntry`] with fields matching the given
    /// source `path`.
    fn lookup_source_path<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
    ) -> Option<&'a UTabEntry> {
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
            "UTab::lookup_source_path searching {:?} for entry matching source path {:?}",
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
                log::debug!( "UTab::lookup_source_path {err_msg}. libmount::mnt_table_find_srcpath returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
 "UTab::lookup_source_path found entry matching source path {:?} while searching {:?}",
                                path,
                                direction
                            );

                let entry = owning_ref_from_ptr!(table, UTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`UTabEntry`] with
    /// a `source` field matching the given `path`.
    ///
    /// By default, a `UTab` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`UTab::set_cache`].
    ///
    /// **Note:** providing an **empty path** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_source_path<T>(&mut self, path: T) -> Option<&UTabEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!(
            "UTab::find_source_path searching {:?} for the first entry with a source matching {:?}",
            direction,
            path
        );

        Self::lookup_source_path(self, direction, path)
    }

    /// Searches the table from **end** to **start**, and returns the first [`UTabEntry`] with
    /// a `source` field matching the given `path`.
    ///
    /// By default, a `UTab` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`UTab::set_cache`].
    ///
    /// **Note:** providing an **empty path** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_back_source_path<T>(&mut self, path: T) -> Option<&UTabEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!(
 "UTab::find_back_source_path searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        path
                    );

        Self::lookup_source_path(self, direction, path)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`UTabEntry`] with fields matching the given
    /// target `path`.
    fn lookup_target<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
    ) -> Option<&'a UTabEntry> {
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok()?;
        log::debug!(
            "UTab::lookup_target searching {:?} for entry matching target {:?}",
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
                log::debug!( "UTab::lookup_target {err_msg}. libmount::mnt_table_find_target returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
                    "UTab::lookup_target found entry matching target {:?} while searching {:?}",
                    path,
                    direction
                );

                let entry = owning_ref_from_ptr!(table, UTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`UTabEntry`] with
    /// a `target` field matching the given `path`.
    ///
    /// By default, a `UTab` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`UTab::set_cache`].
    pub fn find_target<T>(&mut self, path: T) -> Option<&UTabEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!(
            "UTab::find_target searching {:?} for the first entry with a target matching {:?}",
            direction,
            path
        );

        Self::lookup_target(self, direction, path)
    }

    /// Searches the table from **end** to **start**, and returns the first [`UTabEntry`] with
    /// a `target` field matching the given `path`.
    ///
    /// By default, a `UTab` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`UTab::set_cache`].
    pub fn find_back_target<T>(&mut self, path: T) -> Option<&UTabEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!(
            "UTab::find_back_target searching {:?} for the first entry with a target matching {:?}",
            direction,
            path
        );

        Self::lookup_target(self, direction, path)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`UTabEntry`] with fields matching the given
    /// the combination of `path` and `option_name` with `option_value`.
    fn lookup_target_with_options<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
        option_name: &str,
        option_value: Option<&str>,
    ) -> Option<&'a UTabEntry> {
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
 "UTab::lookup_target_with_options searching {:?} for entry matching the combination of path {:?} and option {:?}{}",
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
                log::debug!( "UTab::lookup_target_with_options {err_msg}. libmount::mnt_table_find_target_with_option  returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
 "UTab::lookup_target_with_options found entry matching the combination of path {:?} and option {:?}{}",
                            path,
                            option_name,
                            opt_value
                        );

                let entry = owning_ref_from_ptr!(table, UTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Performs a cursory search in the table, from **start** to **end**, and returns the first [`UTabEntry`] with
    /// fields matching the given combination of `path` and `option_name`.
    pub fn find_target_with_option<P, T>(&mut self, path: P, option_name: T) -> Option<&UTabEntry>
    where
        P: AsRef<Path>,
        T: AsRef<str>,
    {
        let path = path.as_ref();
        let option_name = option_name.as_ref();
        let direction = Direction::Forward;
        log::debug!( "UTab::find_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option_name);

        Self::lookup_target_with_options(self, direction, path, option_name, None)
    }

    /// Performs a cursory search in the table, from **end** to **start**, and returns the first [`UTabEntry`] with
    /// fields matching the given combination of `path` and `option_name`.
    pub fn find_back_target_with_option<P, T>(
        &mut self,
        path: P,
        option_name: T,
    ) -> Option<&UTabEntry>
    where
        P: AsRef<Path>,
        T: AsRef<str>,
    {
        let path = path.as_ref();
        let option_name = option_name.as_ref();
        let direction = Direction::Backward;
        log::debug!( "UTab::find_back_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option_name);

        Self::lookup_target_with_options(self, direction, path, option_name, None)
    }

    /// Performs a cursory search in the table, from **start** to **end**, and returns the first [`UTabEntry`] with
    /// fields matching **exactly** the given combination of `path` and `option`.
    pub fn find_target_with_exact_option<P, T>(
        &mut self,
        path: P,
        option: &MountOption,
    ) -> Option<&UTabEntry>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!( "UTab::find_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option);

        Self::lookup_target_with_options(self, direction, path, option.name(), option.value())
    }

    /// Performs a cursory search in the table, from **end** to **start**, and returns the first [`UTabEntry`] with
    /// fields matching **exactly** the given combination of `path` and `option`.
    pub fn find_back_target_with_exact_option<P, T>(
        &mut self,
        path: P,
        option: &MountOption,
    ) -> Option<&UTabEntry>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!( "UTab::find_back_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}", direction, path, option);

        Self::lookup_target_with_options(self, direction, path, option.name(), option.value())
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`UTabEntry`] with fields matching the given
    /// `source`/`target` pair.
    fn lookup_pair<'a>(
        table: &mut Self,
        direction: Direction,
        source: &Source,
        target: &Path,
    ) -> Option<&'a UTabEntry> {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok()?;
        let target_cstr = ffi_utils::as_ref_path_to_c_string(target).ok()?;

        log::debug!(
            "UTab::lookup_pair searching {:?} for entry matching source/target pair {:?} / {:?}",
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
                log::debug!( "UTab::lookup_pair {err_msg}. libmount::mnt_table_find_pair returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
                    "UTab::lookup_pair found entry matching source/target pair {:?} / {:?}",
                    source,
                    target
                );

                let entry = owning_ref_from_ptr!(table, UTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`UTabEntry`] with
    /// fields matching the given `source`/`target` pair.
    ///
    /// **Warning:** this method runs the same code as [`UTab::find_source_path`] and
    /// [`UTab::find_target`] under the hood, evaluating every table entry, making it the
    /// slowest of the search methods.
    pub fn find_pair<T>(&mut self, source: &Source, target: T) -> Option<&UTabEntry>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        log::debug!( "UTab::find_pair searching table from top to bottom for entry with source/target pair {:?} / {:?}", source, target);

        Self::lookup_pair(self, Direction::Forward, source, target)
    }

    /// Searches the table from **end** to **start**, and returns the first [`UTabEntry`] with
    /// fields matching the given `source`/`target` pair.
    ///
    /// **Warning:** this method runs the same code as [`UTab::find_back_source_path`] and
    /// [`UTab::find_back_target`] under the hood, evaluating every table entry, making it the
    /// slowest of the search methods.
    pub fn find_back_pair<T>(&mut self, source: &Source, target: T) -> Option<&UTabEntry>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        log::debug!( "UTab::find_back_pair searching table from bottom to top for entry with source/target pair {:?} / {:?}", source, target);

        Self::lookup_pair(self, Direction::Backward, source, target)
    }
    //---- END getters

    //---- BEGIN iterators

    /// Returns an iterator over immutable [`UTab`] entries
    ///
    /// # Panics
    ///
    /// Panics if it fails to create a [`UTabIter`].
    pub fn iter(&self) -> UTabIter {
        log::debug!("UTab::iter creating a new `UTabIter`");

        UTabIter::new(self).unwrap()
    }

    /// Tries to instanciate an iterator over immutable [`UTab`] entries
    pub fn try_iter(&self) -> Result<UTabIter, UTabIterError> {
        log::debug!("UTab::try_iter creating a new `UTabIter`");

        UTabIter::new(self)
    }

    /// Returns an iterator over mutable [`UTab`] entries.
    ///
    /// # Panics
    ///
    /// Panics if it fails to create a [`UTabIterMut`].
    pub fn iter_mut(&mut self) -> UTabIterMut {
        log::debug!("UTab::iter_mut creating a new `UTabIterMut`");

        UTabIterMut::new(self).unwrap()
    }

    /// Tries to instanciate an iterator over mutable [`UTab`] entries.
    pub fn try_iter_mut(&mut self) -> Result<UTabIterMut, UTabIterError> {
        log::debug!("UTab::iter_mut creating a new `UTabIterMut`");

        UTabIterMut::new(self)
    }

    //---- END iterators

    //---- BEGIN setters

    /// Sets an syntax error handler function for the file system description file parser.
    ///
    /// The error handler takes two parameters:
    /// - a `file_name`: the name of the file being parsed.
    /// - a `line_number`: the line number of the syntax error.
    pub fn set_parser_error_handler<F>(&mut self, err_handler: F) -> Result<(), UTabError>
    where
        F: Fn(&str, usize) -> ParserFlow,
    {
        log::debug!("UTab::set_parser_error_handler setting up parser error handler");
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
                        log::debug!("UTab::set_parser_error_handler set up parser error handler");
                        // FIXME the callback function is long lived. If the function is called
                        // several times, we risk a substantial memory leak until the end of the program,
                        // since `user_data` is never released between calls.

                        Ok(())
                    }
                    code => {
                        let err_msg = "failed to set parser syntax error handler".to_owned();
                        log::debug!( "UTab::set_parser_error_handler {err_msg}. libmount::mnt_table_set_parser_errcb returned error code: {code:?}");

                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Err(UTabError::Config(err_msg))
                    }
                }
            }
            code => {
                let err_msg = "failed to set error handler as userdata".to_owned();
                log::debug!( "UTab::set_parser_error_handler {err_msg}. libmount::mnt_table_set_userdata returned error code: {code:?}");

                // Deallocate closure on the heap.
                let _ = unsafe { Box::from_raw(user_data) };

                Err(UTabError::Config(err_msg))
            }
        }
    }

    /// Sets up a [`Cache`] for canonicalized paths and evaluated tags (e.g. `LABEL`, `UUID`).
    ///
    /// Assigning a cache to a `UTab` will help speed up all `find_*` methods, and perform
    /// more thorough searches.
    pub fn set_cache(&mut self, cache: Cache) -> Result<(), UTabError> {
        log::debug!("UTab::set_cache setting up a cache of paths and tags");

        // Increment cache's reference counter to avoid a premature deallocation leading to a SIGSEV.
        unsafe {
            libmount::mnt_ref_cache(cache.inner);
        }

        let result = unsafe { libmount::mnt_table_set_cache(self.inner, cache.inner) };
        match result {
            0 => {
                log::debug!("UTab::set_cache set up a cache of paths and tags");

                Ok(())
            }
            code => {
                let err_msg = "failed to set up a cache of paths and tags".to_owned();
                log::debug!( "UTab::set_cache {err_msg}. libmount::mnt_table_set_cache returned error code: {code:?}");

                Err(UTabError::Config(err_msg))
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

    fn filter_by<F>(table: &mut Self, flags: u32, cmp_fn: F) -> Result<(), UTabError>
    where
        F: FnMut(&UTabEntry, &UTabEntry) -> Ordering,
    {
        #[doc(hidden)]
        /// Comparison function to identify duplicate entries.
        unsafe extern "C" fn compare<F>(
            table: *mut libmount::libmnt_table,
            this: *mut libmount::libmnt_fs,
            other: *mut libmount::libmnt_fs,
        ) -> libc::c_int
        where
            F: FnMut(&UTabEntry, &UTabEntry) -> Ordering,
        {
            // Temporarily increments the pointer's reference counter which will be decremented when
            // entry goes out of scope. This prevents us from freeing the data structure while it is
            // still in use.
            let this = UTabEntry::borrow_ptr(this);
            let other = UTabEntry::borrow_ptr(other);

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
                        log::debug!("UTab::filter_by removed duplicates");
                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Ok(())
                    }
                    code => {
                        let err_msg = "failed to remove duplicates from table".to_owned();
                        log::debug!( "UTab::filter_by {err_msg}. libmount::mnt_table_uniq_fs returned error code: {code:?}");

                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Err(UTabError::Deduplicate(err_msg))
                    }
                }
            }
            code => {
                let err_msg = "failed to set the comparison function as userdata".to_owned();
                log::debug!( "UTab::filter_by {err_msg}. libmount::mnt_table_uniq_fs returned error code: {code:?}");

                // Deallocate closure on the heap.
                let _ = unsafe { Box::from_raw(user_data) };

                Err(UTabError::Deduplicate(err_msg))
            }
        }
    }

    /// Removes the duplicate entries in this table keeping the first occurrence of an
    /// entry for which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:** this method preserves the index order of the entries in the table.
    pub fn dedup_first_by<F>(&mut self, cmp: F) -> Result<(), UTabError>
    where
        F: FnMut(&UTabEntry, &UTabEntry) -> Ordering,
    {
        log::debug!("UTab::dedup_first_by merging matching entries to the first occurrence");

        Self::filter_by(self, libmount::MNT_UNIQ_FORWARD, cmp)
    }

    /// Removes the duplicate entries in this table keeping the last occurrence of an
    /// entry for which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:** this method preserves the index order of the entries in the table.
    pub fn dedup_last_by<F>(&mut self, cmp: F) -> Result<(), UTabError>
    where
        F: FnMut(&UTabEntry, &UTabEntry) -> Ordering,
    {
        log::debug!("UTab::dedup_last_by merging matching entries to the last occurrence");
        static MNT_UNIQ_BACKWARD: u32 = 0;

        Self::filter_by(self, MNT_UNIQ_BACKWARD, cmp)
    }

    /// Appends a [`UTabEntry`] to this `UTab`.") ]
    ///
    /// # Panics
    ///
    /// Panics if memory allocation fails.
    pub fn push(&mut self, element: UTabEntry) {
        self.try_push(element).unwrap()
    }

    /// Tries to append a [`UTabEntry`] to this `UTab`.") ]
    pub fn try_push(&mut self, element: UTabEntry) -> Result<(), UTabError> {
        log::debug!("UTab::try_push adding a new table entry");

        let result = unsafe { libmount::mnt_table_add_fs(self.inner, element.inner) };
        match result {
            0 => {
                log::debug!("UTab::try_push added a new table entry");

                Ok(())
            }
            code => {
                let err_msg = "failed to add a new table entry".to_owned();
                log::debug!(
                     "UTab::try_push {err_msg}. libmount::mnt_table_add_fs returned error code: {code:?}"
                            );

                Err(UTabError::Action(err_msg))
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
    ) -> Result<(), UTabError> {
        let op = if after { 1 } else { 0 };
        let op_str = if after {
            "after".to_owned()
        } else {
            "before".to_owned()
        };

        let result = unsafe { libmount::mnt_table_insert_fs(table.inner, op, pos, entry) };
        match result {
            0 => {
                log::debug!("UTab::insert_entry inserted new entry {} reference", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to insert new entry {} reference", op_str);
                log::debug!( "UTab::insert_entry {err_msg}. libmount::mnt_table_insert_fs returned error code: {code:?}");

                Err(UTabError::Action(err_msg))
            }
        }
    }

    /// Prepends a new element to the `UTab`.
    pub fn push_front(&mut self, element: UTabEntry) -> Result<(), UTabError> {
        log::debug!("UTab::push_front prepending new entry");

        Self::insert_entry(self, true, std::ptr::null_mut(), element.inner)
    }

    /// Inserts an element at position `index` within the table, shifting all elements
    /// after it to the bottom.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn insert(&mut self, index: usize, element: UTabEntry) {
        self.try_insert(index, element).unwrap()
    }

    /// Tries to insert an element at position `index` within the table, shifting all elements
    /// after it to the bottom.
    pub fn try_insert(&mut self, index: usize, element: UTabEntry) -> Result<(), UTabError> {
        log::debug!("UTab::try_insert inserting new entry at index: {:?}", index);

        let mut iter = UTabIter::new(self)?;

        match iter.nth(index) {
            Some(position) => Self::insert_entry(self, false, position.inner, element.inner),
            None => {
                let err_msg = format!(
                    "failed to insert element at index: {:?}. Index out of bounds.",
                    index
                );
                log::debug!("UTab::try_insert {err_msg}");

                Err(UTabError::IndexOutOfBounds(err_msg))
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
    ) -> Result<(), UTabError> {
        log::debug!("UTab::move_entry transferring entry between tables");

        let op = if after { 1 } else { 0 };

        let result =
            unsafe { libmount::mnt_table_move_fs(source_table, dest_table, op, position, entry) };

        match result {
            0 => {
                log::debug!("UTab::move_entry transferred entry between tables");

                Ok(())
            }
            code => {
                let err_msg = "failed to transfer entry between tables".to_owned();
                log::debug!(
                                 "UTab::move_entry {err_msg}. libmount::mnt_table_move_fs returned error code: {code:?}"
                            );

                Err(UTabError::Transfer(err_msg))
            }
        }
    }

    /// Transfers an element between two `FsTab`s, from `index` in the source table to
    /// `dest_index` in the destination table.
    ///
    /// # Examples
    ///
    /// ```ignore
    ///     // Initialize `source_table`
    ///      let mut source_table = UTab::new()?;
    ///     source_table.push(entry3)?;
    ///
    ///     // Initialize `dest_table`
    ///      let mut dest_table = UTab::new()?;
    ///     dest_table.push(entry1)?;
    ///     dest_table.push(entry2)?;
    ///
    ///     // Transfer `entry3` from `source_table` to the end of `dest_table`
    ///     source_table.transfer(0, &mut dest_table, 2)?;
    /// ```
    pub fn transfer(
        &mut self,
        index: usize,
        destination: &mut UTab,
        dest_index: usize,
    ) -> Result<(), UTabError> {
        let mut iter = UTabIter::new(self)?;

        match iter.nth(index) {
            Some(entry) if dest_index == 0 => {
                log::debug!(
 "UTab::transfer transferring element a index: {:?} to start of destination table",
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
                    "UTab::transfer transferring element a index: {:?} to end of destination table",
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
                let mut iter_dest = UTabIter::new(destination)?;
                match iter_dest.nth(dest_index) {
                    Some(position) => {
                        log::debug!( "UTab::transfer transferring element at index {:?} to destination at index {:?}", index, dest_index);

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
                        log::debug!("UTab::transfer {err_msg}");

                        Err(UTabError::IndexOutOfBounds(err_msg))
                    }
                }
            }
            None => {
                let err_msg = format!(
                    "failed to access element at index {:?} in source table. Index out of bounds.",
                    index
                );
                log::debug!("UTab::transfer {err_msg}");

                Err(UTabError::IndexOutOfBounds(err_msg))
            }
        }
    }

    /// Removes the given `element` from the table.
    ///
    /// # Panics
    ///
    /// May panic if the index is out of bounds.
    pub fn remove(&mut self, index: usize) -> UTabEntry {
        log::debug!("UTab::remove removing entry from table");

        let err_msg = format!("failed to find entry at index: {:?}", index);
        let element: &UTabEntry = self
            .get(index)
            .ok_or(Err::<&UTabEntry, UTabError>(UTabError::IndexOutOfBounds(
                err_msg,
            )))
            .unwrap();

        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed() -> ! {
            panic!("cannot remove table entry. Not found");
        }

        // increment reference counter to prevent mnt_table_remove from deallocating the underlying
        // table entry
        let borrowed = <UTabEntry>::borrow_ptr(element.inner);

        let result = unsafe { libmount::mnt_table_remove_fs(self.inner, element.inner) };
        match result {
            0 => {
                log::debug!("UTab::remove removed entry from table");

                borrowed
            }
            code => {
                let err_msg = "failed to remove entry from table".to_owned();
                log::debug!(
 "UTab::remove {err_msg}. libmount::mnt_table_remove_fs returned error code: {code:?}"
                            );

                // the element is not in the table, so we decrement its reference counter by
                // dropping it to cancel out the increment performed by UTabEntry::borrow_ptr
                drop(borrowed);
                assert_failed()
            }
        }
    }

    /// Removes all table entries.
    pub fn clear(&mut self) -> Result<(), UTabError> {
        log::debug!("UTab::clear removing all table entries");

        unsafe {
            match libmount::mnt_reset_table(self.inner) {
                0 => {
                    log::debug!("UTab::clear removed all table entries");
                    self.collect_garbage();

                    Ok(())
                }
                code => {
                    let err_msg = "failed to remove all table entries".to_owned();
                    log::debug!(
 "UTab::clear {err_msg}. libmount::mnt_reset_table returned error code: {code:?}"
                                );

                    Err(UTabError::Action(err_msg))
                }
            }
        }
    }

    /// Saves this table's entries to a file.
    pub fn write_file<T>(&mut self, file_path: T) -> Result<(), UTabError>
    where
        T: AsRef<Path>,
    {
        let file_path = file_path.as_ref();
        let file_path_cstr = ffi_utils::as_ref_path_to_c_string(file_path)?;
        log::debug!("UTab::write_file saving table content to {:?}", file_path);

        let result =
            unsafe { libmount::mnt_table_replace_file(self.inner, file_path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!("UTab::write_file saved table content to {:?}", file_path);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to save table content to {:?}", file_path);
                log::debug!( "UTab::write_file {err_msg}. libmount::mnt_table_replace_file returned error code: {code:?}");

                Err(UTabError::Export(err_msg))
            }
        }
    }

    /// Writes this table's entries to a file stream.
    pub fn write_stream(&mut self, file_stream: &mut File) -> io::Result<()> {
        log::debug!("UTab::write_stream writing mount table content to file stream");

        if ffi_utils::is_open_write_only(file_stream)?
            || ffi_utils::is_open_read_write(file_stream)?
        {
            let file = ffi_utils::write_only_c_file_stream_from(file_stream)?;

            let result = unsafe { libmount::mnt_table_write_file(self.inner, file as *mut _) };
            match result {
                0 => {
                    log::debug!("UTab::write_stream wrote mount table content to file stream");

                    Ok(())
                }
                code => {
                    let err_msg = "failed to write mount table content to file stream".to_owned();
                    log::debug!( "UTab::write_stream {err_msg}. libmount::mnt_table_write_file  returned error code: {code:?}");

                    Err(io::Error::from_raw_os_error(code))
                }
            }
        } else {
            let err_msg = "you do not have permission to write in this file stream".to_owned();
            log::debug!("UTab::write_stream {err_msg}");

            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        }
    }

    //---- END mutators

    //---- BEGIN predicates

    /// Returns `true` if this `UTab` contains a element matching **exactly** the given `element`.
    pub fn contains(&self, element: &UTabEntry) -> bool {
        let state = unsafe { libmount::mnt_table_find_fs(self.inner, element.inner) > 0 };
        log::debug!("UTab::contains value: {:?}", state);

        state
    }

    /// Returns `true` if the table has length of 0.
    pub fn is_empty(&self) -> bool {
        let state = unsafe { libmount::mnt_table_is_empty(self.inner) == 1 };
        log::debug!("UTab::is_empty value: {:?}", state);

        state
    }

    //---- END predicates
}

impl fmt::Display for UTab {
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
    fn utab_can_import_run_mount_utab() -> crate::Result<()> {
        let mut utab = UTab::new()?;

        utab.import_utab()?;

        Ok(())
    }
}
