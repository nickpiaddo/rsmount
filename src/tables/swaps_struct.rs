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
use crate::core::entries::SwapsEntry;

use crate::core::errors::SwapsError;
use crate::core::errors::SwapsIterError;

use crate::core::iter::Direction;
use crate::core::iter::GenIterator;
use crate::core::iter::SwapsIter;

use crate::owning_ref_from_ptr;

use crate::tables::GcItem;
use crate::tables::ParserFlow;

use crate::ffi_utils;

/// An in-memory representation of `/proc/swaps`.
///
/// # `/proc/swaps`
///
/// `/proc/swaps` records swap space, and its utilization. For systems with only one swap partition, the
/// output of `/proc/swaps` may be similar to the following:
///
/// ```text
/// Filename                                Type            Size            Used            Priority
/// /dev/sda2                               partition       1048572         0               -2
/// ```
///
/// `/proc/swaps` provides a snapshot of every swap `Filename`, the `Type` of swap space, the total
/// `Size`, and the amount of space `Used` (in kilobytes). The `Priority` column is useful when
/// multiple swap files are in use. The lower the priority, the more likely the swap file is used.

#[derive(Debug)]
pub struct Swaps {
    pub(crate) inner: *mut libmount::libmnt_table,
    pub(crate) gc: Vec<GcItem>,
}

impl Swaps {
    fn collect_garbage(&mut self) {
        // Free item references created on the heap.
        while let Some(gc_item) = self.gc.pop() {
            gc_item.destroy();
        }
    }
}

impl Drop for Swaps {
    fn drop(&mut self) {
        log::debug!("::drop deallocating `Swaps` instance");

        unsafe { libmount::mnt_unref_table(self.inner) };
        self.collect_garbage();
    }
}

impl AsRef<Swaps> for Swaps {
    #[inline]
    fn as_ref(&self) -> &Swaps {
        self
    }
}

impl Index<usize> for Swaps {
    type Output = SwapsEntry;

    /// Performs the indexing (`container\[index]`) operation.
    fn index(&self, index: usize) -> &Self::Output {
        log::debug!("Swaps::index getting item at index: {:?}", index);

        #[cold]
        #[inline(never)]
        #[track_caller]
        fn indexing_failed() -> ! {
            panic!("Index out of bounds");
        }

        let mut iter = SwapsIter::new(self).unwrap();

        match iter.nth(index) {
            Some(item) => item,
            None => indexing_failed(),
        }
    }
}

impl Swaps {
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
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_table) -> Swaps {
        Self {
            inner: ptr,
            gc: vec![],
        }
    }

    #[doc(hidden)]
    /// Borrows an instance.
    #[allow(dead_code)]
    pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_table) -> Swaps {
        let mut table = Self::from_ptr(ptr);
        // We are virtually ceding ownership of this table which will be automatically
        // deallocated once it is out of scope, incrementing its reference counter protects it from
        // being freed prematurely.
        table.incr_ref_counter();

        table
    }

    //---- BEGIN constructors

    /// Creates a new empty `Swaps`.
    pub fn new() -> Result<Swaps, SwapsError> {
        log::debug!("Swaps::new creating a new `Swaps` instance");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();

        unsafe { ptr.write(libmount::mnt_new_table()) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to create a new `Swaps`".to_owned();
                log::debug!(
                    "Swaps::new {err_msg}. libmount::mnt_new_table returned a NULL pointer"
                );

                Err(SwapsError::Creation(err_msg))
            }
            ptr => {
                log::debug!("Swaps::new created a new `Swaps` instance");
                let table = Self::from_ptr(ptr);

                Ok(table)
            }
        }
    }

    //---- END constructors

    /// Parses `/proc/swaps`, then appends the data it collected to the table.
    pub fn import_proc_swaps(&mut self) -> Result<(), SwapsError> {
        log::debug!("Swaps::import_proc_swaps importing entries from /proc/swaps");

        let result = unsafe { libmount::mnt_table_parse_swaps(self.inner, std::ptr::null()) };

        match result {
            0 => {
                log::debug!("Swaps::import_proc_swaps imported entries from /proc/swaps");

                Ok(())
            }
            code => {
                let err_msg = "failed to import entries from /proc/swaps".to_owned();
                log::debug!("Swaps::import_proc_swaps {}. libmount::mnt_table_parse_swaps returned error code: {:?}", err_msg, code);

                Err(SwapsError::Import(err_msg))
            }
        }
    }

    //---- BEGIN getters

    /// Returns a reference to the [`Cache`] instance associated with this `Swaps`.
    pub fn cache(&self) -> Option<&Cache> {
        log::debug!("Swaps::cache getting associated path and tag cache");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_cache>::zeroed();

        unsafe { ptr.write(libmount::mnt_table_get_cache(self.inner)) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("Swaps::cache failed to get associated path and tag cache. libmount::mnt_table_get_cache returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!("Swaps::cache got associated path and tag cache");
                let cache = owning_ref_from_ptr!(self, Cache, ptr);

                Some(cache)
            }
        }
    }

    /// Returns a reference to the first element of the `Swaps`, or `None` if it is empty.
    pub fn first(&self) -> Option<&SwapsEntry> {
        log::debug!("Swaps::first getting reference to first table entry");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe { libmount::mnt_table_first_fs(self.inner, ptr.as_mut_ptr()) };

        match result {
            0 => {
                log::debug!("Swaps::first got reference to first table entry");
                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(self, SwapsEntry, ptr);

                Some(entry)
            }
            code => {
                log::debug!( "Swaps::first failed to get reference to first table entry. libmount::mnt_table_first_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns a reference to the last element of the `Swaps`, or `None` if it is empty.
    pub fn last(&self) -> Option<&SwapsEntry> {
        log::debug!("Swaps::last getting reference to last table entry");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe { libmount::mnt_table_last_fs(self.inner, ptr.as_mut_ptr()) };

        match result {
            0 => {
                log::debug!("Swaps::last got reference to last table entry");
                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(self, SwapsEntry, ptr);

                Some(entry)
            }
            code => {
                log::debug!( "Swaps::last failed to get reference to last table entry. libmount::mnt_table_last_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns the index of a table entry.
    pub fn position(&self, entry: &SwapsEntry) -> Option<usize> {
        log::debug!("Swaps::position searching for an entry in the table");

        let result = unsafe { libmount::mnt_table_find_fs(self.inner, entry.inner) };

        match result {
            index if index > 0 => {
                log::debug!(
                    "Swaps::position mount table contains entry at index: {:?}",
                    index
                );

                Some(index as usize)
            }
            code => {
                log::debug!( "Swaps::position no matching entry in table: libmount::mnt_table_find_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Returns the number of entries in the table.
    pub fn len(&self) -> usize {
        let len = unsafe { libmount::mnt_table_get_nents(self.inner) };
        log::debug!("Swaps::len value: {:?}", len);

        len as usize
    }

    /// Returns a reference to an element at `index`, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&SwapsEntry> {
        log::debug!(
            "Swaps::get_mut getting reference of item at index: {:?}",
            index
        );

        SwapsIter::new(self)
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
    ) -> Option<&'a SwapsEntry>
    where
        P: FnMut(&SwapsEntry) -> bool,
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
            P: FnMut(&SwapsEntry) -> bool,
        {
            // Temporarily increments the pointer's reference counter which will be decremented when
            // `entry` goes out of scope. This prevents us from freeing the data structure while it is
            // still in use.
            let entry = SwapsEntry::borrow_ptr(entry_ptr);

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

                log::debug!("Swaps::find_first_entry found first `SwapsEntry` matching predicate");
                let entry_ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(table, SwapsEntry, entry_ptr);

                Some(entry)
            }
            code => {
                // To ensure the closure is properly deallocated when this variable drops out
                // of scope.
                let _predicate = unsafe { Box::from_raw(data) };

                let err_msg = "failed to find `SwapsEntry` matching predicate".to_owned();
                log::debug!( "Swaps::find_first_entry {err_msg}. libmount::mnt_table_find_next_fs returned error code: {code:?}");

                None
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`SwapsEntry`] that
    /// satisfies the `predicate`.
    ///
    /// # Panics
    ///
    /// Panics if it can not create a forward iterator.
    pub fn find_first<P>(&mut self, predicate: P) -> Option<&SwapsEntry>
    where
        P: FnMut(&SwapsEntry) -> bool,
    {
        log::debug!( "Swaps::find_first finding first table entry matching predicate while iterating Forward");
        GenIterator::new(Direction::Forward)
            .ok()
            .and_then(|iterator| Swaps::find_first_entry(self, iterator.inner, predicate))
    }

    /// Searches the table from **end** to **start**, and returns the first [`SwapsEntry`] that
    /// satisfies the `predicate`.
    ///
    /// # Panics
    ///
    /// Panics if it can not create a backward iterator.
    pub fn find_back_first<P>(&mut self, predicate: P) -> Option<&SwapsEntry>
    where
        P: FnMut(&SwapsEntry) -> bool,
    {
        log::debug!( "Swaps::find_back_first finding first table entry matching predicate while iterating Backward");
        GenIterator::new(Direction::Backward)
            .ok()
            .and_then(|iterator| Swaps::find_first_entry(self, iterator.inner, predicate))
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`SwapsEntry`] with fields matching the given
    /// source `path`.
    fn lookup_source<'a>(
        table: &mut Self,
        direction: Direction,
        source: &Source,
    ) -> Option<&'a SwapsEntry> {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok()?;
        let source_ptr = if source.is_pseudo_fs() {
            // For pseudo file systems `libmount::mnt_table_find_source`
            // expects a NULL pointer path.
            std::ptr::null()
        } else {
            source_cstr.as_ptr()
        };

        log::debug!(
            "Swaps::lookup_source searching {:?} for entry matching source {:?}",
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
                log::debug!( "Swaps::lookup_source {err_msg}. libmount::mnt_table_find_source returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
                    "Swaps::lookup_source found entry matching source {:?} while searching {:?}",
                    source,
                    direction
                );

                let entry = owning_ref_from_ptr!(table, SwapsEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`SwapsEntry`] with
    /// a field matching the given `source`.
    ///
    /// By default, a `Swaps` will perform a cursory search, looking for an entry with an exact
    /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`Swaps::set_cache`].
    ///
    /// **Note:** providing an **empty source** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_source(&mut self, source: &Source) -> Option<&SwapsEntry> {
        let direction = Direction::Forward;
        log::debug!(
            "Swaps::find_source searching {:?} for the first entry with a source matching {:?}",
            direction,
            source
        );

        Self::lookup_source(self, direction, source)
    }

    /// Searches the table from **end** to **start**, and returns the first [`SwapsEntry`] with
    /// a field matching the given `source`.
    ///
    /// By default, a `Swaps` will perform a cursory search, looking for an entry with an exact
    /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`Swaps::set_cache`].
    ///
    /// **Note:** providing an **empty source** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_back_source(&mut self, source: &Source) -> Option<&SwapsEntry> {
        let direction = Direction::Backward;
        log::debug!(
 "Swaps::find_back_source searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        source
                    );

        Self::lookup_source(self, direction, source)
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`SwapsEntry`] with fields matching the given
    /// source `path`.
    fn lookup_source_path<'a>(
        table: &mut Self,
        direction: Direction,
        path: &Path,
    ) -> Option<&'a SwapsEntry> {
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
            "Swaps::lookup_source_path searching {:?} for entry matching source path {:?}",
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
                log::debug!( "Swaps::lookup_source_path {err_msg}. libmount::mnt_table_find_srcpath returned a NULL pointer");

                None
            }
            ptr => {
                log::debug!(
 "Swaps::lookup_source_path found entry matching source path {:?} while searching {:?}",
                                path,
                                direction
                            );

                let entry = owning_ref_from_ptr!(table, SwapsEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Searches the table from **start** to **end**, and returns the first [`SwapsEntry`] with
    /// a `source` field matching the given `path`.
    ///
    /// By default, a `Swaps` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`Swaps::set_cache`].
    ///
    /// **Note:** providing an **empty path** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_source_path<T>(&mut self, path: T) -> Option<&SwapsEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Forward;
        log::debug!(
 "Swaps::find_source_path searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        path
                    );

        Self::lookup_source_path(self, direction, path)
    }

    /// Searches the table from **end** to **start**, and returns the first [`SwapsEntry`] with
    /// a `source` field matching the given `path`.
    ///
    /// By default, a `Swaps` will perform a cursory search, looking for an entry with an exact
    /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
    /// paths, etc., set up a [`Cache`] with [`Swaps::set_cache`].
    ///
    /// **Note:** providing an **empty path** is equivalent to searching for a source matching `"none"`
    /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
    pub fn find_back_source_path<T>(&mut self, path: T) -> Option<&SwapsEntry>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let direction = Direction::Backward;
        log::debug!(
 "Swaps::find_back_source_path searching {:?} for the first entry with a source matching {:?}",
                        direction,
                        path
                    );

        Self::lookup_source_path(self, direction, path)
    }
    //---- END getters

    //---- BEGIN iterators

    /// Returns an iterator over immutable [`Swaps`] entries
    ///
    /// # Panics
    ///
    /// Panics if it fails to create a [`SwapsIter`].
    pub fn iter(&self) -> SwapsIter {
        log::debug!("Swaps::iter creating a new `SwapsIter`");

        SwapsIter::new(self).unwrap()
    }

    /// Tries to instanciate an iterator over immutable [`Swaps`] entries
    pub fn try_iter(&self) -> Result<SwapsIter, SwapsIterError> {
        log::debug!("Swaps::iter creating a new `SwapsIter`");

        SwapsIter::new(self)
    }

    //---- END iterators

    //---- BEGIN setters

    /// Sets an syntax error handler function for the file system description file parser.
    ///
    /// The error handler takes two parameters:
    /// - a `file_name`: the name of the file being parsed.
    /// - a `line_number`: the line number of the syntax error.
    pub fn set_parser_error_handler<F>(&mut self, err_handler: F) -> Result<(), SwapsError>
    where
        F: Fn(&str, usize) -> ParserFlow,
    {
        log::debug!("Swaps::set_parser_error_handler setting up parser error handler");
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
                        log::debug!("Swaps::set_parser_error_handler set up parser error handler");
                        // FIXME the callback function is long lived. If the function is called
                        // several times, we risk a substantial memory leak until the end of the program,
                        // since `user_data` is never released between calls.

                        Ok(())
                    }
                    code => {
                        let err_msg = "failed to set parser syntax error handler".to_owned();
                        log::debug!( "Swaps::set_parser_error_handler {err_msg}. libmount::mnt_table_set_parser_errcb returned error code: {code:?}");

                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Err(SwapsError::Config(err_msg))
                    }
                }
            }
            code => {
                let err_msg = "failed to set error handler as userdata".to_owned();
                log::debug!( "Swaps::set_parser_error_handler {err_msg}. libmount::mnt_table_set_userdata returned error code: {code:?}");

                // Deallocate closure on the heap.
                let _ = unsafe { Box::from_raw(user_data) };

                Err(SwapsError::Config(err_msg))
            }
        }
    }

    /// Sets up a [`Cache`] for canonicalized paths and evaluated tags (e.g. `LABEL`, `UUID`).
    ///
    /// Assigning a cache to a `Swaps` will help speed up all `find_*` methods, and perform
    /// more thorough searches.
    pub fn set_cache(&mut self, cache: Cache) -> Result<(), SwapsError> {
        log::debug!("Swaps::set_cache setting up a cache of paths and tags");

        // Increment cache's reference counter to avoid a premature deallocation leading to a SIGSEV.
        unsafe {
            libmount::mnt_ref_cache(cache.inner);
        }

        let result = unsafe { libmount::mnt_table_set_cache(self.inner, cache.inner) };
        match result {
            0 => {
                log::debug!("Swaps::set_cache set up a cache of paths and tags");

                Ok(())
            }
            code => {
                let err_msg = "failed to set up a cache of paths and tags".to_owned();
                log::debug!( "Swaps::set_cache {err_msg}. libmount::mnt_table_set_cache returned error code: {code:?}");

                Err(SwapsError::Config(err_msg))
            }
        }
    }

    //---- END setters

    //---- BEGIN mutators

    fn filter_by<F>(table: &mut Self, flags: u32, cmp_fn: F) -> Result<(), SwapsError>
    where
        F: FnMut(&SwapsEntry, &SwapsEntry) -> Ordering,
    {
        #[doc(hidden)]
        /// Comparison function to identify duplicate entries.
        unsafe extern "C" fn compare<F>(
            table: *mut libmount::libmnt_table,
            this: *mut libmount::libmnt_fs,
            other: *mut libmount::libmnt_fs,
        ) -> libc::c_int
        where
            F: FnMut(&SwapsEntry, &SwapsEntry) -> Ordering,
        {
            // Temporarily increments the pointer's reference counter which will be decremented when
            // entry goes out of scope. This prevents us from freeing the data structure while it is
            // still in use.
            let this = SwapsEntry::borrow_ptr(this);
            let other = SwapsEntry::borrow_ptr(other);

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
                        log::debug!("Swaps::filter_by removed duplicates");
                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Ok(())
                    }
                    code => {
                        let err_msg = "failed to remove duplicates from table".to_owned();
                        log::debug!( "Swaps::filter_by {err_msg}. libmount::mnt_table_uniq_fs returned error code: {code:?}");

                        // Deallocate closure on the heap.
                        let _ = unsafe { Box::from_raw(user_data) };

                        Err(SwapsError::Deduplicate(err_msg))
                    }
                }
            }
            code => {
                let err_msg = "failed to set the comparison function as userdata".to_owned();
                log::debug!( "Swaps::filter_by {err_msg}. libmount::mnt_table_uniq_fs returned error code: {code:?}");

                // Deallocate closure on the heap.
                let _ = unsafe { Box::from_raw(user_data) };

                Err(SwapsError::Deduplicate(err_msg))
            }
        }
    }

    /// Removes the duplicate entries in this table keeping the first occurrence of an
    /// entry for which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:** this method preserves the index order of the entries in the table.
    pub fn dedup_first_by<F>(&mut self, cmp: F) -> Result<(), SwapsError>
    where
        F: FnMut(&SwapsEntry, &SwapsEntry) -> Ordering,
    {
        log::debug!("Swaps::dedup_first_by merging matching entries to the first occurrence");

        Self::filter_by(self, libmount::MNT_UNIQ_FORWARD, cmp)
    }

    /// Removes the duplicate entries in this table keeping the last occurrence of an
    /// entry for which the `cmp` function returns [`Ordering::Equal`].
    ///
    /// **Note:** this method preserves the index order of the entries in the table.
    pub fn dedup_last_by<F>(&mut self, cmp: F) -> Result<(), SwapsError>
    where
        F: FnMut(&SwapsEntry, &SwapsEntry) -> Ordering,
    {
        log::debug!("Swaps::dedup_last_by merging matching entries to the last occurrence");
        static MNT_UNIQ_BACKWARD: u32 = 0;

        Self::filter_by(self, MNT_UNIQ_BACKWARD, cmp)
    }

    //---- END mutators

    //---- BEGIN predicates

    /// Returns `true` if the table has length of 0.
    pub fn is_empty(&self) -> bool {
        let state = unsafe { libmount::mnt_table_is_empty(self.inner) == 1 };
        log::debug!("Swaps::is_empty value: {:?}", state);

        state
    }

    //---- END predicates
}

impl fmt::Display for Swaps {
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
    fn swaps_can_import_proc_swaps() -> crate::Result<()> {
        let mut proc_swaps = Swaps::new()?;

        proc_swaps.import_proc_swaps()?;

        Ok(())
    }
}
