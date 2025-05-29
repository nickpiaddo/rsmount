// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! declare_tab {
    ($table_type:ident, $doc:literal) => {
        // From dependency library

        // From standard library

        // From this library
        use $crate::tables::GcItem;

        #[doc = $doc]
        #[derive(Debug)]
        pub struct $table_type {
            pub(crate) inner: *mut libmount::libmnt_table,
            pub(crate) gc: Vec<GcItem>,
        }

        impl $table_type {
            fn collect_garbage(&mut self) {
                // Free item references created on the heap.
                while let Some(gc_item) = self.gc.pop() {
                    gc_item.destroy();
                }
            }
        }

        impl Drop for $table_type {
            fn drop(&mut self) {
                log::debug!(concat!(
                    stringify!($table_type),
                    "::drop deallocating `",
                    stringify!($table_type),
                    "` instance"
                ));

                unsafe { libmount::mnt_unref_table(self.inner) };
                self.collect_garbage();
            }
        }
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! owning_ref_from_ptr {
    ($object_ref:expr, $output_ref_ty:ident, $ptr: ident) => {{
        let mut obj_ptr = std::ptr::NonNull::from($object_ref.as_ref());
        let boxed = Box::new($ptr);
        let (boxed_ptr, item) = unsafe { <$output_ref_ty>::ref_from_boxed_ptr(boxed) };

        // Adds boxed pointer to garbage collector
        unsafe { obj_ptr.as_mut().gc.push(boxed_ptr.into()) };

        item
    }};
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! owning_mut_from_ptr {
    ($object_ref:expr, $output_ref_ty:ident, $ptr: ident) => {{
        let mut obj_ptr = std::ptr::NonNull::from($object_ref.as_ref());
        let boxed = Box::new($ptr);
        let (boxed_ptr, item) = unsafe { <$output_ref_ty>::mut_from_boxed_ptr(boxed) };

        // Adds boxed pointer to garbage collector
        unsafe { obj_ptr.as_mut().gc.push(boxed_ptr.into()) };

        item
    }};
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! swaps_shared_methods {
    ($table_type:tt, $table_entry_type: tt, $table_error_type:tt) => {
        $crate::table_shared_methods!($table_type, $table_entry_type, $table_error_type);
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! utab_shared_methods {
    ($table_type:tt, $table_entry_type: tt, $table_error_type:tt) => {
        $crate::table_shared_methods!($table_type, $table_entry_type, $table_error_type);
        $crate::table_shared_edit_methods!($table_type, $table_entry_type, $table_error_type);
        $crate::table_shared_target_methods!($table_type, $table_entry_type, $table_error_type);
        $crate::table_iter_mut!($table_type, $table_entry_type, $table_error_type);
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_shared_methods {
    ($table_type:ident, $table_entry_type:ident, $table_error_type:ident) => {
        paste::paste!{

            // From dependency library

            // From standard library
            use std::mem::MaybeUninit;
            use std::ops::Index;
            use std::path::Path;
            use std::cmp::Ordering;

            // From this library
            use $crate::owning_ref_from_ptr;
            use $crate::core::cache::Cache;
            use $crate::core::iter::Direction;
            use $crate::core::iter::GenIterator;
            use $crate::core::device::Source;
            use $crate::tables::ParserFlow;
            use $crate::core::iter::[<$table_type Iter>];

            use $crate::ffi_utils;

            #[allow(dead_code)]
            impl $table_type {
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
                pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_table) -> $table_type {
                    Self { inner: ptr, gc: vec![] }
                }

                #[doc(hidden)]
                /// Borrows an instance.
                #[allow(dead_code)]
                pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_table) -> $table_type {
                    let mut table = Self::from_ptr(ptr);
                    // We are virtually ceding ownership of this table which will be automatically
                    // deallocated once it is out of scope, incrementing its reference counter protects it from
                    // being freed prematurely.
                    table.incr_ref_counter();

                    table
                }

                #[doc = concat!("Creates a new empty `", stringify!($table_type),"`.")]
                pub fn new() -> Result<$table_type, $table_error_type> {
                    log::debug!(concat!(
                        stringify!($table_type),
                        "::new creating a new `",
                        stringify!($table_type),
                        "` instance"
                    ));

                    let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();

                    unsafe { ptr.write(libmount::mnt_new_table()) };

                    match unsafe { ptr.assume_init() } {
                        ptr if ptr.is_null() => {
                            let err_msg =
                                concat!("failed to create a new `", stringify!($table_type), "`")
                                    .to_owned();
                            log::debug!(
                                concat!(
                                    stringify!($table_type),
                                    "::new {}. libmount::mnt_new_table returned a NULL pointer"
                                ),
                                err_msg
                            );

                            Err(<$table_error_type>::Creation(err_msg))
                        }
                        ptr => {
                            log::debug!(concat!(
                                stringify!($table_type),
                                "::new created a new `",
                                stringify!($table_type),
                                "` instance"
                            ));
                            let table = Self::from_ptr(ptr);

                            Ok(table)
                        }
                    }
                }

                //---- BEGIN setters

                /// Sets an syntax error handler function for the file system description file parser.
                ///
                /// The error handler takes two parameters:
                /// - a `file_name`: the name of the file being parsed.
                /// - a `line_number`: the line number of the syntax error.
                pub fn set_parser_error_handler<F>(&mut self, err_handler: F) -> Result<(), $table_error_type>
                    where
                        F: Fn(&str, usize) -> ParserFlow,
                {
                    log::debug!(concat!(stringify!($table_type), "::set_parser_error_handler setting up parser error handler"));
                    #[doc(hidden)]
                    /// Callback function to handle syntax errors in file system description files
                    /// during parsing. Used by `libmount::mnt_table_parse_file`.
                    unsafe extern "C" fn parser_callback<F>(
                        table: *mut libmount::libmnt_table,
                        file_name: *const libc::c_char,
                        line: libc::c_int) -> libc::c_int
                        where
                            F: Fn(&str, usize) -> ParserFlow,
                    {
                       // Convert file name to string reference.
                       let file_name = ffi_utils::const_char_array_to_str_ref(file_name).ok().unwrap_or("");

                       // Rebuild the callback function.
                       let mut callback_ptr = MaybeUninit::<*mut libc::c_void>::zeroed();
                       unsafe { callback_ptr.write(libmount::mnt_table_get_userdata(table)); }

                       // Since we set the handler function ourselves, we can safely assume this pointer
                       // is never NULL.
                       let callback_ptr = unsafe { callback_ptr.assume_init() };
                       let handler =  &mut * (callback_ptr as *mut F);

                       handler(file_name, line as usize) as i32
                    }

                    // Moving the closure to the heap with `Box::new`, to live there for some unknown period of
                    // time.  Then, call `Box::into_raw` on it, to get a raw pointer to the closure, and
                    // prevent the memory it uses from being deallocated.
                    let user_data = Box::into_raw(Box::new(err_handler));

                    let result = unsafe { libmount::mnt_table_set_userdata(self.inner, user_data as *mut _) };

                    match result {
                        0 => {

                            let result = unsafe { libmount::mnt_table_set_parser_errcb(self.inner, Some(parser_callback::<F>)) };

                            match result {
                                0 => {
                                    log::debug!(concat!(stringify!($table_type), "::set_parser_error_handler set up parser error handler"));
                                    // FIXME the callback function is long lived. If the function is called
                                    // several times, we risk a substantial memory leak until the end of the program,
                                    // since `user_data` is never released between calls.

                                    Ok(())

                                }
                                code => {
                                    let err_msg = "failed to set parser syntax error handler".to_owned();
                                    log::debug!(concat!(stringify!($table_type), "::set_parser_error_handler {}. libmount::mnt_table_set_parser_errcb returned error code: {:?}"), err_msg, code);

                                    // Deallocate closure on the heap.
                                    let _ = unsafe { Box::from_raw(user_data) };

                                    Err(<$table_error_type>::Config(err_msg))
                                }
                            }
                        }
                        code => {
                            let err_msg = "failed to set error handler as userdata".to_owned();
                            log::debug!(concat!(stringify!($table_type), "::set_parser_error_handler {}. libmount::mnt_table_set_userdata returned error code: {:?}"), err_msg, code);

                            // Deallocate closure on the heap.
                            let _ = unsafe { Box::from_raw(user_data) };

                            Err(<$table_error_type>::Config(err_msg))
                        }
                    }

                }

                /// Sets up a [`Cache`] for canonicalized paths and evaluated tags (e.g. `LABEL`, `UUID`).
                ///
                #[doc = concat!("Assigning a cache to a `", stringify!($table_type), "` will help speed up all `find_*` methods, and perform")]
                /// more thorough searches.
                pub fn set_cache(&mut self, cache: Cache) -> Result<(), $table_error_type> {
                    log::debug!(concat!(stringify!($table_type), "::set_cache setting up a cache of paths and tags"));

                    let result = unsafe { libmount::mnt_table_set_cache(self.inner, cache.inner) } ;

                    match result {
                        0 => {
                            log::debug!(concat!(stringify!($table_type), "::set_cache set up a cache of paths and tags"));

                            Ok(())
                        }
                        code => {
                            let err_msg = "failed to set up a cache of paths and tags".to_owned();
                            log::debug!(concat!(stringify!($table_type), "::set_cache {}. libmount::mnt_table_set_cache returned error code: {:?}"), err_msg, code);

                            Err(<$table_error_type>::Config(err_msg))
                        }
                    }
                }

                //---- END setters

                //---- BEGIN mutators

                fn filter_by<F>(table: &mut Self, flags: u32, cmp_fn: F) -> Result<(), $table_error_type>
                    where
                        F: FnMut(&$table_entry_type, &$table_entry_type) -> Ordering,
                {

                    #[doc(hidden)]
                    /// Comparison function to identify duplicate entries.
                    unsafe extern "C" fn compare<F>(
                        table: *mut libmount::libmnt_table,
                        this: *mut libmount::libmnt_fs,
                        other: *mut libmount::libmnt_fs) -> libc::c_int
                        where
                            F: FnMut(&$table_entry_type, &$table_entry_type) -> Ordering,
                    {

                        // Temporarily increments the pointer's reference counter which will be decremented when
                        // entry goes out of scope. This prevents us from freeing the data structure while it is
                        // still in use.
                        let this = $table_entry_type::borrow_ptr(this);
                        let other = $table_entry_type::borrow_ptr(other);

                        // Rebuild the comparison function.
                        let mut user_data_ptr = MaybeUninit::<*mut libc::c_void>::zeroed();
                        unsafe { user_data_ptr.write(libmount::mnt_table_get_userdata(table)); }

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

                            let result = unsafe { libmount::mnt_table_uniq_fs(table.inner, flags as i32, Some(compare::<F>)) };

                            match result {
                                0 => {
                                    log::debug!(concat!(stringify!($table_type), "::filter_by removed duplicates"));
                                    // Deallocate closure on the heap.
                                    let _ = unsafe { Box::from_raw(user_data) };


                                    Ok(())

                                }
                                code => {
                                    let err_msg = "failed to remove duplicates from table".to_owned();
                                    log::debug!(concat!(stringify!($table_type), "::filter_by {}. libmount::mnt_table_uniq_fs returned error code: {:?}"), err_msg, code);

                                    // Deallocate closure on the heap.
                                    let _ = unsafe { Box::from_raw(user_data) };

                                    Err(<$table_error_type>::Deduplicate(err_msg))
                                }
                            }
                        }
                        code => {
                            let err_msg = "failed to set the comparison function as userdata".to_owned();
                            log::debug!(concat!(stringify!($table_type), "::filter_by {}. libmount::mnt_table_uniq_fs returned error code: {:?}"), err_msg, code);

                            // Deallocate closure on the heap.
                            let _ = unsafe { Box::from_raw(user_data) };

                            Err(<$table_error_type>::Deduplicate(err_msg))
                        }
                    }
                }

                /// Removes the duplicate entries in this table keeping the first occurrence of an
                /// entry for which the `cmp` function returns [`Ordering::Equal`].
                ///
                /// **Note:** this method preserves the index order of the entries in the table.
                pub fn dedup_first_by<F>(&mut self, cmp: F) -> Result<(), $table_error_type>
                    where
                        F: FnMut(&$table_entry_type, &$table_entry_type) -> Ordering,
                {
                    log::debug!(concat!(stringify!($table_type), "::dedup_first_by merging matching entries to the first occurrence"));

                    Self::filter_by(self, libmount::MNT_UNIQ_FORWARD, cmp)
                }

                /// Removes the duplicate entries in this table keeping the last occurrence of an
                /// entry for which the `cmp` function returns [`Ordering::Equal`].
                ///
                /// **Note:** this method preserves the index order of the entries in the table.
                pub fn dedup_last_by<F>(&mut self, cmp: F) -> Result<(), $table_error_type>
                    where
                        F: FnMut(&$table_entry_type, &$table_entry_type) -> Ordering,
                {
                    log::debug!(concat!(stringify!($table_type), "::dedup_last_by merging matching entries to the last occurrence"));
                    static MNT_UNIQ_BACKWARD: u32 = 0;

                    Self::filter_by(self, MNT_UNIQ_BACKWARD, cmp)
                }

                //---- END mutators

                //---- BEGIN getters

                #[doc = concat!("Returns a reference to the [`Cache`] instance associated with this `", stringify!($table_type), "`.")]
                pub fn cache(&self) -> Option<&Cache> {
                    log::debug!("MountTable::cache getting associated path and tag cache");

                    let mut ptr = MaybeUninit::<*mut libmount::libmnt_cache>::zeroed();

                    unsafe { ptr.write(libmount::mnt_table_get_cache(self.inner)) };

                    match unsafe { ptr.assume_init() } {
                        ptr if ptr.is_null() => {
                            log::debug!("MountTable::cache failed to get associated path and tag cache. libmount::mnt_table_get_cache returned a NULL pointer");

                        None
                        }
                        ptr => {
                            log::debug!("MountTable::cache got associated path and tag cache");
                            let cache = owning_ref_from_ptr!(self, Cache, ptr);

                            Some(cache)
                        }
                    }
                }

                #[doc = concat!("Returns a reference to the first element of the `", stringify!($table_type), "`, or `None` if it is empty.")]
                pub fn first(&self) -> Option<&$table_entry_type> {
                    log::debug!(concat!(stringify!($table_type), "::first getting reference to first table entry"));

                    let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                    let result = unsafe { libmount::mnt_table_first_fs(self.inner, ptr.as_mut_ptr()) };

                    match result {
                        0 => {
                            log::debug!(concat!(stringify!($table_type), "::first got reference to first table entry"));
                            let ptr = unsafe { ptr.assume_init() };
                            let entry = owning_ref_from_ptr!(self, $table_entry_type, ptr);

                            Some(entry)
                        }
                        code => {
                            log::debug!(concat!(stringify!($table_type), "::first failed to get reference to first table entry. libmount::mnt_table_first_fs returned error code: {:?}"), code);

                            None
                        }
                    }
                }

                #[doc = concat!("Returns a reference to the last element of the `", stringify!($table_type), "`, or `None` if it is empty.")]
                pub fn last(&self) -> Option<&$table_entry_type> {
                    log::debug!(concat!(stringify!($table_type), "::last getting reference to last table entry"));

                    let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                    let result = unsafe { libmount::mnt_table_last_fs(self.inner, ptr.as_mut_ptr()) };

                    match result {
                        0 => {
                            log::debug!(concat!(stringify!($table_type), "::last got reference to last table entry"));
                            let ptr = unsafe { ptr.assume_init() };
                            let entry = owning_ref_from_ptr!(self, $table_entry_type, ptr);

                            Some(entry)
                        }
                        code => {
                            log::debug!(concat!(stringify!($table_type), "::last failed to get reference to last table entry. libmount::mnt_table_last_fs returned error code: {:?}"), code);

                            None
                        }
                    }
                }

                /// Returns the index of a table entry.
                pub fn position(&self, entry: &$table_entry_type) -> Option<usize> {
                    log::debug!(concat!(stringify!($table_type), "::position searching for an entry in the table"));

                    let result = unsafe { libmount::mnt_table_find_fs(self.inner, entry.inner) };

                    match result {
                        index if index > 0 => {
                            log::debug!(
                                concat!(stringify!($table_type), "::position mount table contains entry at index: {:?}"),
                                index
                            );

                            Some(index as usize)
                        }
                        code => {
                            log::debug!(concat!(stringify!($table_type), "::position no matching entry in table: libmount::mnt_table_find_fs returned error code: {:?}"), code);

                            None
                        }
                    }
                }

                /// Returns the number of entries in the table.
                pub fn len(&self) -> usize {
                    let len = unsafe { libmount::mnt_table_get_nents(self.inner) };
                    log::debug!(concat!(stringify!($table_type), "::len value: {:?}"), len);

                    len as usize
                }

                /// Returns a reference to an element at `index`, or `None` if out of bounds.
                pub fn get(&self, index: usize) -> Option<&$table_entry_type> {
                    log::debug!(concat!(stringify!($table_type), "::get_mut getting reference of item at index: {:?}"), index);

                    let mut iter = [<$table_type Iter>]::new(self).unwrap();

                    iter.nth(index)
                }

                #[doc(hidden)]
                /// Searches forward/backward for the first entry in the `table` that satisfies the `predicate`
                /// depending on the [`Direction`] defined at the `iterator`'s creation.
                fn find_first_entry<'a, P>(
                    table: &mut Self,
                    iterator: *mut libmount::libmnt_iter,
                    predicate: P,
                ) -> Option<&'a $table_entry_type>
                where
                    P: FnMut(&$table_entry_type) -> bool,
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
                        P: FnMut(&$table_entry_type) -> bool,
                    {
                        // Temporarily increments the pointer's reference counter which will be decremented when
                        // `entry` goes out of scope. This prevents us from freeing the data structure while it is
                        // still in use.
                        let entry = $table_entry_type::borrow_ptr(entry_ptr);

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

                    let result = unsafe { libmount::mnt_table_find_next_fs(
                        table.inner,
                        iterator,
                        Some(callback::<P>),
                        data as *mut _,
                        ptr.as_mut_ptr(),
                    ) };

                    match result {
                        0 => {
                            // To ensure the closure is properly deallocated when this variable drops out
                            // of scope.
                            let _predicate = unsafe { Box::from_raw(data) };

                            log::debug!(concat!(stringify!($table_type), "::find_first_entry found first `$table_entry_type` matching predicate"));
                            let entry_ptr = unsafe { ptr.assume_init() };
                            let entry = owning_ref_from_ptr!(table, $table_entry_type, entry_ptr);

                            Some(entry)
                        }
                        code => {
                            // To ensure the closure is properly deallocated when this variable drops out
                            // of scope.
                            let _predicate = unsafe { Box::from_raw(data) };

                            let err_msg = "failed to find `$table_entry_type` matching predicate".to_owned();
                            log::debug!(concat!(stringify!($table_type), "::find_first_entry {}. libmount::mnt_table_find_next_fs returned error code: {:?}"), err_msg, code);

                            None
                        }
                    }
                }

                #[doc = concat!("Searches the table from **start** to **end**, and returns the first [`", stringify!($table_entry_type),"`] that")]
                /// satisfies the `predicate`.
                ///
                /// # Panics
                ///
                /// Panics if it can not create a forward iterator.
                pub fn find_first<P>(&mut self, predicate: P) -> Option<&$table_entry_type>
                where
                    P: FnMut(&$table_entry_type) -> bool,
                {
                    log::debug!(concat!(stringify!($table_type), "::find_first finding first table entry matching predicate while iterating Forward"));
                    let iterator = GenIterator::new(Direction::Forward).unwrap();

                    Self::find_first_entry(self, iterator.inner, predicate)
                }

                #[doc = concat!("Searches the table from **end** to **start**, and returns the first [`", stringify!($table_entry_type),"`] that")]
                /// satisfies the `predicate`.
                ///
                /// # Panics
                ///
                /// Panics if it can not create a backward iterator.
                pub fn find_back_first<P>(&mut self, predicate: P) -> Option<&$table_entry_type>
                where
                    P: FnMut(&$table_entry_type) -> bool,
                {
                    log::debug!(concat!(stringify!($table_type), "::find_back_first finding first table entry matching predicate while iterating Backward"));
                    let iterator = GenIterator::new(Direction::Backward).unwrap();

                    Self::find_first_entry(self, iterator.inner, predicate)
                }

                #[doc(hidden)]
                #[doc = concat!("Searches in [`Direction`] for a [`", stringify!($table_entry_type), "`] with fields matching the given")]
                /// source `path`.
                fn lookup_source<'a>(
                    table: &mut Self,
                    direction: Direction,
                    source: &Source,
                ) -> Option<&'a $table_entry_type>
                {
                    let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok()?;
                    let source_ptr = if source.is_pseudo_fs() {
                        // For pseudo file systems `libmount::mnt_table_find_source`
                        // expects a NULL pointer path.
                        std::ptr::null()
                    } else {
                        source_cstr.as_ptr()
                    };

                    log::debug!(
                        concat!(stringify!($table_type), "::lookup_source searching {:?} for entry matching source {:?}"),
                        direction,
                        source
                    );

                    let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                    unsafe { ptr.write(libmount::mnt_table_find_source(
                        table.inner,
                        source_ptr,
                        direction as i32,
                    ))};

                    match unsafe { ptr.assume_init() } {
                        ptr if ptr.is_null() => {
                            let err_msg = format!(
                                "failed to find entry matching source {:?} while searching {:?}",
                                source, direction
                            );
                            log::debug!(concat!(stringify!($table_type), "::lookup_source {}. libmount::mnt_table_find_source returned a NULL pointer"), err_msg);

                            None
                        }
                        ptr => {
                            log::debug!(
                                concat!(stringify!($table_type), "::lookup_source found entry matching source {:?} while searching {:?}"),
                                source,
                                direction
                            );

                            let entry = owning_ref_from_ptr!(table, $table_entry_type, ptr);

                            Some(entry)
                        }
                    }
                }

                #[doc = concat!("Searches the table from **start** to **end**, and returns the first [`", stringify!($table_entry_type), "`] with")]
                /// a field matching the given `source`.
                ///
                #[doc = concat!("By default, a `", stringify!($table_type), "` will perform a cursory search, looking for an entry with an exact")]
                /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
                #[doc = concat!("paths, etc., set up a [`Cache`] with [`", stringify!($table_type), "::set_cache`].")]
                ///
                /// **Note:** providing an **empty source** is equivalent to searching for a source matching `"none"`
                /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
                pub fn find_source(&mut self, source: &Source) -> Option<&$table_entry_type>
                {
                    let direction = Direction::Forward;
                    log::debug!(
                        concat!(stringify!($table_type), "::find_source searching {:?} for the first entry with a source matching {:?}"),
                        direction,
                        source
                    );

                    Self::lookup_source(self, direction, source)
                }

                #[doc = concat!("Searches the table from **end** to **start**, and returns the first [`", stringify!($table_entry_type), "`] with")]
                /// a field matching the given `source`.
                ///
                #[doc = concat!("By default, a `", stringify!($table_type), "` will perform a cursory search, looking for an entry with an exact")]
                /// `source` match. To perform a deep search, which implies following symlinks, canonicalizing
                #[doc = concat!("paths, etc., set up a [`Cache`] with [`", stringify!($table_type), "::set_cache`].")]
                ///
                /// **Note:** providing an **empty source** is equivalent to searching for a source matching `"none"`
                /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
                pub fn find_back_source(&mut self, source: &Source) -> Option<&$table_entry_type>
                {
                    let direction = Direction::Backward;
                    log::debug!(
                        concat!(stringify!($table_type), "::find_back_source searching {:?} for the first entry with a source matching {:?}"),
                        direction,
                        source
                    );

                    Self::lookup_source(self, direction, source)
                }

                #[doc(hidden)]
                #[doc = concat!("Searches in [`Direction`] for a [`", stringify!($table_entry_type), "`] with fields matching the given")]
                /// source `path`.
                fn lookup_source_path<'a>(
                    table: &mut Self,
                    direction: Direction,
                    path: &Path,
                ) -> Option<&'a $table_entry_type>
                {
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
                        concat!(stringify!($table_type), "::lookup_source_path searching {:?} for entry matching source path {:?}"),
                        direction,
                        path
                    );

                    let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                    unsafe { ptr.write(libmount::mnt_table_find_srcpath(
                        table.inner,
                        path_ptr,
                        direction as i32,
                    ))};

                    match unsafe { ptr.assume_init() } {
                        ptr if ptr.is_null() => {
                            let err_msg = format!(
                                "failed to find entry matching source path {:?} while searching {:?}",
                                path, direction
                            );
                            log::debug!(concat!(stringify!($table_type), "::lookup_source_path {}. libmount::mnt_table_find_srcpath returned a NULL pointer"), err_msg);

                            None
                        }
                        ptr => {
                            log::debug!(
                                concat!(stringify!($table_type), "::lookup_source_path found entry matching source path {:?} while searching {:?}"),
                                path,
                                direction
                            );

                            let entry = owning_ref_from_ptr!(table, $table_entry_type, ptr);

                            Some(entry)
                        }
                    }
                }

                #[doc = concat!("Searches the table from **start** to **end**, and returns the first [`", stringify!($table_entry_type), "`] with")]
                /// a `source` field matching the given `path`.
                ///
                #[doc = concat!("By default, a `", stringify!($table_type), "` will perform a cursory search, looking for an entry with an exact")]
                /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
                #[doc = concat!("paths, etc., set up a [`Cache`] with [`", stringify!($table_type), "::set_cache`].")]
                ///
                /// **Note:** providing an **empty path** is equivalent to searching for a source matching `"none"`
                /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
                pub fn find_source_path<T>(&mut self, path: T) -> Option<&$table_entry_type>
                where
                    T: AsRef<Path>,
                {
                    let path = path.as_ref();
                    let direction = Direction::Forward;
                    log::debug!(
                        concat!(stringify!($table_type), "::find_source_path searching {:?} for the first entry with a source matching {:?}"),
                        direction,
                        path
                    );

                    Self::lookup_source_path(self, direction, path)
                }

                #[doc = concat!("Searches the table from **end** to **start**, and returns the first [`", stringify!($table_entry_type), "`] with")]
                /// a `source` field matching the given `path`.
                ///
                #[doc = concat!("By default, a `", stringify!($table_type), "` will perform a cursory search, looking for an entry with an exact")]
                /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
                #[doc = concat!("paths, etc., set up a [`Cache`] with [`", stringify!($table_type), "::set_cache`].")]
                ///
                /// **Note:** providing an **empty path** is equivalent to searching for a source matching `"none"`
                /// (used in `/proc/mounts`, and `/proc/self/mountinfo` to designate pseudo-filesystems).
                pub fn find_back_source_path<T>(&mut self, path: T) -> Option<&$table_entry_type>
                where
                    T: AsRef<Path>,
                {
                    let path = path.as_ref();
                    let direction = Direction::Backward;
                    log::debug!(
                        concat!(stringify!($table_type), "::find_back_source_path searching {:?} for the first entry with a source matching {:?}"),
                        direction,
                        path
                    );

                    Self::lookup_source_path(self, direction, path)
                }
                //---- END getters

                //---- BEGIN iterators

                #[doc = concat!("Returns an iterator over immutable [`", stringify!($table_type), "`] entries")]
                ///
                /// # Panics
                ///
                #[doc = concat!("Panics if it fails to create a [`", stringify!($table_type), "Iter`].")]
                pub fn iter(&self) -> [<$table_type Iter>] {
                    log::debug!(concat!(stringify!($table_type), "::iter creating a new `", stringify!($table_type), "Iter`"));

                     [<$table_type Iter>]::new(self).unwrap()
                }

                //---- END iterators

                //---- BEGIN predicates

                /// Returns `true` if the table has length of 0.
                pub fn is_empty(&self) -> bool {
                    let state = unsafe { libmount::mnt_table_is_empty(self.inner) == 1 };
                    log::debug!(concat!(stringify!($table_type), "::is_empty value: {:?}"), state);

                    state
                }

                //---- END predicates
            } //---- END impl

            impl AsRef<$table_type> for $table_type {
                #[inline]
                fn as_ref(&self) -> &$table_type {
                    self
                }
            }

            impl Index<usize> for $table_type {
                type Output = $table_entry_type;

                /// Performs the indexing (`container\[index]`) operation.
                fn index(&self, index: usize) -> &Self::Output {
                    log::debug!(concat!(stringify!($table_type), "::index getting item at index: {:?}"), index);

                    #[cold]
                    #[inline(never)]
                    #[track_caller]
                    fn indexing_failed() -> ! {
                        panic!("Index out of bounds");
                    }

                    let mut iter = [<$table_type Iter>]::new(self).unwrap();

                    match iter.nth(index) {
                        Some(item) => item,
                        None => indexing_failed(),
                    }

                }
            }
        } //---- END paste
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_shared_target_methods {
    ($table_type:ident, $table_entry_type:ident, $table_error_type:ident) => {

        // From dependency library

        // From standard library

        // From this library
        use $crate::tables::MountOption;

        #[allow(dead_code)]
        impl $table_type {
            #[doc(hidden)]
            #[doc = concat!("Searches in [`Direction`] for a [`", stringify!($table_entry_type), "`] with fields matching the given")]
            /// target `path`.
            fn lookup_target<'a>(
                table: &mut Self,
                direction: Direction,
                path: &Path,
            ) -> Option<&'a $table_entry_type>
            {
                let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok()?;
                log::debug!(
                    concat!(stringify!($table_type), "::lookup_target searching {:?} for entry matching target {:?}"),
                    direction,
                    path
                );

                let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                unsafe { ptr.write(libmount::mnt_table_find_target(
                    table.inner,
                    path_cstr.as_ptr(),
                    direction as i32,
                ))};

                match unsafe { ptr.assume_init() } {
                    ptr if ptr.is_null() => {
                        let err_msg = format!(
                            "failed to find entry matching target {:?} while searching {:?}",
                            path, direction
                        );
                        log::debug!(concat!(stringify!($table_type), "::lookup_target {}. libmount::mnt_table_find_target returned a NULL pointer"), err_msg);

                        None
                    }
                    ptr => {
                        log::debug!(
                            concat!(stringify!($table_type), "::lookup_target found entry matching target {:?} while searching {:?}"),
                            path,
                            direction
                        );

                        let entry = owning_ref_from_ptr!(table, $table_entry_type, ptr);

                        Some(entry)
                    }
                }
            }

            #[doc = concat!("Searches the table from **start** to **end**, and returns the first [`", stringify!($table_entry_type), "`] with")]
            /// a `target` field matching the given `path`.
            ///
            #[doc = concat!("By default, a `", stringify!($table_type), "` will perform a cursory search, looking for an entry with an exact")]
            /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
            #[doc = concat!("paths, etc., set up a [`Cache`] with [`", stringify!($table_type), "::set_cache`].")]
            pub fn find_target<T>(&mut self, path: T) -> Option<&$table_entry_type>
            where
                T: AsRef<Path>,
            {
                let path = path.as_ref();
                let direction = Direction::Forward;
                log::debug!(
                    concat!(stringify!($table_type), "::find_target searching {:?} for the first entry with a target matching {:?}"),
                    direction,
                    path
                );

                Self::lookup_target(self, direction, path)
            }

            #[doc = concat!("Searches the table from **end** to **start**, and returns the first [`", stringify!($table_entry_type), "`] with")]
            /// a `target` field matching the given `path`.
            ///
            #[doc = concat!("By default, a `", stringify!($table_type), "` will perform a cursory search, looking for an entry with an exact")]
            /// `path` match. To perform a deep search, which implies following symlinks, canonicalizing
            #[doc = concat!("paths, etc., set up a [`Cache`] with [`", stringify!($table_type), "::set_cache`].")]
            pub fn find_back_target<T>(&mut self, path: T) -> Option<&$table_entry_type>
            where
                T: AsRef<Path>,
            {
                let path = path.as_ref();
                let direction = Direction::Backward;
                log::debug!(
                    concat!(stringify!($table_type), "::find_back_target searching {:?} for the first entry with a target matching {:?}"),
                    direction,
                    path
                );

                Self::lookup_target(self, direction, path)
            }

            #[doc(hidden)]
            #[doc = concat!("Searches in [`Direction`] for a [`", stringify!($table_entry_type), "`] with fields matching the given")]
            /// the combination of `path` and `option_name` with `option_value`.
            fn lookup_target_with_options<'a>(
                table: &mut Self,
                direction: Direction,
                path: &Path,
                option_name: &str,
                option_value: Option<&str>,
            ) -> Option<&'a $table_entry_type>
            {
                // Represent the missing value by an empty string.
                let option_value = option_value.map_or_else(|| String::new(), |value| value.to_owned());

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
                } else  {
                    format!(" with value {:?}", option_value)
                };

                log::debug!(
                    concat!(stringify!($table_type), "::lookup_target_with_options searching {:?} for entry matching the combination of path {:?} and option {:?}{}"),
                    direction,
                    path,
                    option_name,
                    opt_value
                );


                let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                unsafe { ptr.write(libmount::mnt_table_find_target_with_option(
                    table.inner,
                    path_cstr.as_ptr(),
                    opt_name_cstr.as_ptr(),
                    opt_value_ptr,
                    direction as i32,
                ))};

                match unsafe { ptr.assume_init() } {
                    ptr if ptr.is_null() => {
                        let err_msg = format!("found no entry matching the combination of path {:?} and option {:?}{} while searching {:?}", path, option_name, opt_value, direction );
                        log::debug!(concat!(stringify!($table_type), "::lookup_target_with_options {}. libmount::mnt_table_find_target_with_option  returned a NULL pointer"), err_msg);

                        None
                    }
                    ptr => {
                        log::debug!(
                            concat!(stringify!($table_type), "::lookup_target_with_options found entry matching the combination of path {:?} and option {:?}{}"),
                            path,
                            option_name,
                            opt_value
                        );

                        let entry = owning_ref_from_ptr!(table, $table_entry_type, ptr);

                        Some(entry)
                    }
                }
            }

            #[doc = concat!("Performs a cursory search in the table, from **start** to **end**, and returns the first [`", stringify!($table_entry_type), "`] with")]
            /// fields matching the given combination of `path` and `option_name`.
            pub fn find_target_with_option<P, T>(&mut self, path: P, option_name: T) -> Option<&$table_entry_type>
            where
                P: AsRef<Path>,
                T: AsRef<str>,
            {
                let path = path.as_ref();
                let option_name = option_name.as_ref();
                let direction = Direction::Forward;
                log::debug!(concat!(stringify!($table_type), "::find_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}"), direction, path, option_name);

                Self::lookup_target_with_options(self, direction, path, option_name, None)
            }

            #[doc = concat!("Performs a cursory search in the table, from **end** to **start**, and returns the first [`", stringify!($table_entry_type), "`] with")]
            /// fields matching the given combination of `path` and `option_name`.
            pub fn find_back_target_with_option<P, T>(
                &mut self,
                path: P,
                option_name: T,
            ) -> Option<&$table_entry_type>
            where
                P: AsRef<Path>,
                T: AsRef<str>,
            {
                let path = path.as_ref();
                let option_name = option_name.as_ref();
                let direction = Direction::Backward;
                log::debug!(concat!(stringify!($table_type), "::find_back_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}"), direction, path, option_name);

                Self::lookup_target_with_options(self, direction, path, option_name, None)
            }

            #[doc = concat!("Performs a cursory search in the table, from **start** to **end**, and returns the first [`", stringify!($table_entry_type), "`] with")]
            /// fields matching **exactly** the given combination of `path` and `option`.
            pub fn find_target_with_exact_option<P, T>(
                &mut self,
                path: P,
                option: &MountOption,
            ) -> Option<&$table_entry_type>
            where
                P: AsRef<Path>,
            {
                let path = path.as_ref();
                let direction = Direction::Forward;
                log::debug!(concat!(stringify!($table_type), "::find_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}"), direction, path, option);

                Self::lookup_target_with_options(self, direction, path, option.name(), option.value())
            }

            #[doc = concat!("Performs a cursory search in the table, from **end** to **start**, and returns the first [`", stringify!($table_entry_type), "`] with")]
            /// fields matching **exactly** the given combination of `path` and `option`.
            pub fn find_back_target_with_exact_option<P, T>(
                &mut self,
                path: P,
                option: &MountOption,
            ) -> Option<&$table_entry_type>
            where
                P: AsRef<Path>,
            {
                let path = path.as_ref();
                let direction = Direction::Backward;
                log::debug!(concat!(stringify!($table_type), "::find_back_target_with_option searching {:?} for entry matching the combination of path {:?} and option {:?}"), direction, path, option);

                Self::lookup_target_with_options(self, direction, path, option.name(), option.value())
            }

            #[doc(hidden)]
            #[doc = concat!("Searches in [`Direction`] for a [`", stringify!($table_entry_type), "`] with fields matching the given")]
            /// `source`/`target` pair.
            fn lookup_pair<'a>(
                table: &mut Self,
                direction: Direction,
                source: &Source,
                target: &Path,
            ) -> Option<&'a $table_entry_type>
            {
                let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok()?;
                let target_cstr = ffi_utils::as_ref_path_to_c_string(target).ok()?;

                log::debug!(concat!(stringify!($table_type), "::lookup_pair searching {:?} for entry matching source/target pair {:?} / {:?}"), direction, source, target);

                    let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                    unsafe { ptr.write(libmount::mnt_table_find_pair(
                        table.inner,
                        source_cstr.as_ptr(),
                        target_cstr.as_ptr(),
                        direction as i32,
                    ))};

                    match unsafe { ptr.assume_init() } {
                        ptr if ptr.is_null() => {
                            let err_msg = format!(
                                "found no entry with source/target pair {:?} / {:?} while searching {:?}",
                                source, target, direction,
                            );
                            log::debug!(concat!(stringify!($table_type), "::lookup_pair {}. libmount::mnt_table_find_pair returned a NULL pointer"), err_msg);

                            None
                        }
                        ptr => {
                            log::debug!(
                                concat!(stringify!($table_type), "::lookup_pair found entry matching source/target pair {:?} / {:?}"),
                                source,
                                target
                            );

                            let entry = owning_ref_from_ptr!(table, $table_entry_type, ptr);

                            Some(entry)
                        }
                    }
            }

            #[doc = concat!("Searches the table from **start** to **end**, and returns the first [`", stringify!($table_entry_type), "`] with")]
            /// fields matching the given `source`/`target` pair.
            ///
            #[doc = concat!("**Warning:** this method runs the same code as [`", stringify!($table_type), "::find_source_path`] and")]
            #[doc = concat!("[`", stringify!($table_type), "::find_target`] under the hood, evaluating every table entry, making it the")]
            /// slowest of the search methods.
            pub fn find_pair<T>(&mut self, source: &Source, target: T) -> Option<&$table_entry_type>
            where
                T: AsRef<Path>,
            {
                let target = target.as_ref();
                log::debug!(concat!(stringify!($table_type), "::find_pair searching table from top to bottom for entry with source/target pair {:?} / {:?}"), source, target);

                Self::lookup_pair(self, Direction::Forward, source, target)
            }

            #[doc = concat!("Searches the table from **end** to **start**, and returns the first [`", stringify!($table_entry_type), "`] with")]
            /// fields matching the given `source`/`target` pair.
            ///
            #[doc = concat!("**Warning:** this method runs the same code as [`", stringify!($table_type), "::find_back_source_path`] and")]
            #[doc = concat!("[`", stringify!($table_type), "::find_back_target`] under the hood, evaluating every table entry, making it the")]
            /// slowest of the search methods.
            pub fn find_back_pair<T>(&mut self, source: &Source, target: T) -> Option<&$table_entry_type>
            where
                T: AsRef<Path>,
            {
                let target = target.as_ref();
                log::debug!(concat!(stringify!($table_type), "::find_back_pair searching table from bottom to top for entry with source/target pair {:?} / {:?}"), source, target);

                Self::lookup_pair(self, Direction::Backward, source, target)
            }

        }
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_shared_edit_methods {
    ($table_type:ident, $table_entry_type:ident, $table_error_type:ident) => {
        paste::paste! {

            // From dependency library

            // From standard library
            use std::fs::File;

            // From this library

            #[allow(dead_code)]
            impl $table_type {

                //---- BEGIN mutators

                 #[doc = concat!("Appends a [`", stringify!($table_entry_type),"`] to this `", stringify!($table_type), "`.") ]
                pub fn push(&mut self, element: $table_entry_type) -> Result<(), $table_error_type> {
                    log::debug!(concat!(stringify!($table_type), "::push adding a new table entry"));

                    let result = unsafe { libmount::mnt_table_add_fs(self.inner, element.inner) };

                    match result {
                        0 => {
                            log::debug!(concat!(stringify!($table_type), "::push added a new table entry"));

                            Ok(())
                        }
                        code => {
                            let err_msg = "failed to add a new table entry".to_owned();
                            log::debug!(
                                concat!(stringify!($table_type), "::push {}. libmount::mnt_table_add_fs returned error code: {:?}"),
                                err_msg,
                                code
                            );

                            Err(<$table_error_type>::Action(err_msg))
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
                ) -> Result<(), $table_error_type> {
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
                                concat!(stringify!($table_type), "::insert_entry inserted new entry {} reference"),
                                op_str
                            );

                            Ok(())
                        }
                        code => {
                            let err_msg = format!("failed to insert new entry {} reference", op_str);
                            log::debug!(concat!(stringify!($table_type), "::insert_entry {}. libmount::mnt_table_insert_fs returned error code: {:?}"), err_msg, code);

                            Err(<$table_error_type>::Action(err_msg))
                        }
                    }
                }

                #[doc = concat!("Prepends a new element to the `", stringify!($table_type),"`.")]
                pub fn push_front(&mut self, element: $table_entry_type) -> Result<(), $table_error_type> {
                    log::debug!(concat!(stringify!($table_type), "::push_front prepending new entry"));

                    Self::insert_entry(self, true, std::ptr::null_mut(), element.inner)
                }

                /// Inserts an element at position `index` within the table, shifting all elements after it to
                /// the bottom.
                pub fn insert(&mut self, index: usize, element: $table_entry_type) -> Result<(), $table_error_type> {
                    log::debug!(concat!(stringify!($table_type), "::insert inserting new entry at index: {:?}"), index);

                    let mut iter = [<$table_type Iter>]::new(self)?;

                    match iter.nth(index) {
                        Some(position) => Self::insert_entry(self, false, position.inner, element.inner),
                        None => {
                            let err_msg = format!(
                                "failed to insert element at index: {:?}. Index out of bounds.",
                                index
                            );
                            log::debug!(concat!(stringify!($table_type), "::insert {}"), err_msg);

                            Err(<$table_error_type>::IndexOutOfBounds(err_msg))
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
                ) -> Result<(), $table_error_type> {
                    log::debug!(concat!(stringify!($table_type), "::move_entry transferring entry between tables"));

                    let op = if after { 1 } else { 0 };

                    let result =
                        unsafe { libmount::mnt_table_move_fs(source_table, dest_table, op, position, entry) };

                    match result {
                        0 => {
                            log::debug!(concat!(stringify!($table_type), "::move_entry transferred entry between tables"));

                            Ok(())
                        }
                        code => {
                            let err_msg = "failed to transfer entry between tables".to_owned();
                            log::debug!(
                                concat!(stringify!($table_type), "::move_entry {}. libmount::mnt_table_move_fs returned error code: {:?}"),
                                err_msg,
                                code
                            );

                            Err($table_error_type::Transfer(err_msg))
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
                #[doc = concat!("     let mut source_table = ", stringify!($table_type), "::new()?;")]
                ///     source_table.push(entry3)?;
                ///
                ///     // Initialize `dest_table`
                #[doc = concat!("     let mut dest_table = ", stringify!($table_type), "::new()?;")]
                ///     dest_table.push(entry1)?;
                ///     dest_table.push(entry2)?;
                ///
                ///     // Transfer `entry3` from `source_table` to the end of `dest_table`
                ///     source_table.transfer(0, &mut dest_table, 2)?;
                /// ```
                pub fn transfer(
                    &mut self,
                    index: usize,
                    destination: &mut $table_type,
                    dest_index: usize,
                ) -> Result<(), $table_error_type> {
                    let mut iter = [<$table_type Iter>]::new(self)?;

                    match iter.nth(index) {
                        Some(entry) if dest_index == 0 => {
                            log::debug!(
                                concat!(stringify!($table_type), "::transfer transferring element a index: {:?} to start of destination table"),
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
                                concat!(stringify!($table_type), "::transfer transferring element a index: {:?} to end of destination table"),
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
                            let mut iter_dest = [<$table_type Iter>]::new(destination)?;
                            match iter_dest.nth(dest_index) {
                                Some(position) => {
                                    log::debug!(concat!(stringify!($table_type), "::transfer transferring element at index {:?} to destination at index {:?}"), index, dest_index);

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
                                    log::debug!(concat!(stringify!($table_type), "::transfer {}"), err_msg);

                                    Err(<$table_error_type>::IndexOutOfBounds(err_msg))
                                }
                            }
                        }
                        None => {
                            let err_msg = format!(
                                "failed to access element at index {:?} in source table. Index out of bounds.",
                                index
                            );
                            log::debug!(concat!(stringify!($table_type), "::transfer {}"), err_msg);

                            Err(<$table_error_type>::IndexOutOfBounds(err_msg))
                        }
                    }
                }

                /// Removes the given `element` from the table.
                ///
                /// # Panics
                ///
                /// May panic if the index is out of bounds.
                pub fn remove(&mut self, index: usize) -> $table_entry_type {
                    log::debug!(concat!(stringify!($table_type), "::remove removing entry from table"));

                    let err_msg = format!("failed to find entry at index: {:?}", index);
                    let element: &$table_entry_type = self
                        .get(index)
                        .ok_or(Err::<&$table_entry_type, $table_error_type>(
                            <$table_error_type>::IndexOutOfBounds(err_msg),
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
                    let borrowed = <$table_entry_type>::borrow_ptr(element.inner);

                    let result = unsafe { libmount::mnt_table_remove_fs(self.inner, element.inner) };

                    match result {
                        0 => {
                            log::debug!(concat!(stringify!($table_type), "::remove removed entry from table"));

                            borrowed
                        }
                        code => {
                            let err_msg = "failed to remove entry from table".to_owned();
                            log::debug!(
                                concat!(stringify!($table_type), "::remove {}. libmount::mnt_table_remove_fs returned error code: {:?}"),
                                err_msg,
                                code
                            );

                            // the element is not in the table, so we decrement its reference counter by
                            // dropping it to cancel out the increment performed by $table_entry_type::borrow_ptr
                            drop(borrowed);
                            assert_failed()
                        }
                    }
                }

                /// Removes all table entries.
                pub fn clear(&mut self) -> Result<(), $table_error_type> {
                    log::debug!(concat!(stringify!($table_type), "::clear removing all table entries"));

                    unsafe {
                        match libmount::mnt_reset_table(self.inner) {
                            0 => {
                                log::debug!(concat!(stringify!($table_type), "::clear removed all table entries"));
                                self.collect_garbage();

                                Ok(())
                            }
                            code => {
                                let err_msg = "failed to remove all table entries".to_owned();
                                log::debug!(
                                    concat!(stringify!($table_type), "::clear {}. libmount::mnt_reset_table returned error code: {:?}"),
                                    err_msg,
                                    code
                                );

                                Err(<$table_error_type>::Action(err_msg))
                            }
                        }
                    }
                }

                /// Saves this table's entries to a file.
                pub fn write_file<T>(&mut self, file_path: T) -> Result<(), $table_error_type>
                where
                    T: AsRef<Path>,
                {
                    let file_path = file_path.as_ref();
                    let file_path_cstr = ffi_utils::as_ref_path_to_c_string(file_path)?;
                    log::debug!(concat!(stringify!($table_type), "::write_file saving table content to {:?}"), file_path);

                    let result =
                        unsafe { libmount::mnt_table_replace_file(self.inner, file_path_cstr.as_ptr()) };

                    match result {
                        0 => {
                            log::debug!(concat!(stringify!($table_type), "::write_file saved table content to {:?}"), file_path);

                            Ok(())
                        }
                        code => {
                            let err_msg = format!("failed to save table content to {:?}", file_path);
                            log::debug!(concat!(stringify!($table_type), "::write_file {}. libmount::mnt_table_replace_file returned error code: {:?}"), err_msg, code);

                            Err(<$table_error_type>::Export(err_msg))
                        }
                    }
                }

                /// Writes this table's entries to a file stream.
                pub fn write_stream(&mut self, file_stream: &mut File) -> Result<(), $table_error_type> {
                    log::debug!(concat!(stringify!($table_type), "::write_stream writing mount table content to file stream"));

                    if ffi_utils::is_open_write_only(file_stream)?
                        || ffi_utils::is_open_read_write(file_stream)?
                    {
                        let file = ffi_utils::write_only_c_file_stream_from(file_stream)?;

                        let result = unsafe { libmount::mnt_table_write_file(self.inner, file as *mut _) };

                        match result {
                            0 => {
                                log::debug!(concat!(stringify!($table_type), "::write_stream wrote mount table content to file stream"));

                                Ok(())
                            }
                            code => {
                                let err_msg = "failed to write mount table content to file stream".to_owned();
                                log::debug!(concat!(stringify!($table_type), "::write_stream {}. libmount::mnt_table_write_file  returned error code: {:?}"), err_msg, code);

                                Err($table_error_type::Export(err_msg))
                            }
                        }
                    } else {
                        let err_msg = "you do not have permission to write in this file stream".to_owned();
                        log::debug!(concat!(stringify!($table_type), "::write_stream {}"), err_msg);

                        Err(<$table_error_type>::Permission(err_msg))
                    }
                }

                //---- END mutators

                //---- BEGIN predicates

                #[doc = concat!("Returns `true` if this `", stringify!($table_type),"` contains a element matching **exactly** the given `element`.")]
                pub fn contains(&self, element: &$table_entry_type) -> bool {
                    let state = unsafe { libmount::mnt_table_find_fs(self.inner, element.inner) > 0 };
                    log::debug!(concat!(stringify!($table_type), "::contains value: {:?}"), state);

                    state
                }

                //---- END predicates

            }

        } //---- END paste
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_find_mount_point {
    ($table_type:ident, $table_entry_type:ident, $table_error_type:ident) => {

        // From dependency library

        // From standard library

        // From this library

        #[allow(dead_code)]
        impl $table_type {
            #[doc(hidden)]
            #[doc = concat!("Searches in [`Direction`] for a [`", stringify!($table_entry_type), "`] with fields matching the given `mount_point`.")]
            fn lookup_mount_point<'a>(
                table: &mut Self,
                direction: Direction,
                mount_point: &Path,
            ) -> Option<&'a $table_entry_type>
            {
                let mount_point_cstr = ffi_utils::as_ref_path_to_c_string(mount_point).ok()?;
                log::debug!(
                    concat!(stringify!($table_type), "::lookup_mount_point searching {:?} for entry with mount point {:?}"),
                    direction,
                    mount_point
                );

                let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                unsafe { ptr.write(libmount::mnt_table_find_mountpoint(
                    table.inner,
                    mount_point_cstr.as_ptr(),
                    direction as i32,
                ))};

                match unsafe { ptr.assume_init() } {
                    ptr if ptr.is_null() => {
                        log::debug!(concat!(stringify!($table_type), "::lookup_mount_point found no entry with mount point {:?} while searching {:?}. libmount::mnt_table_find_mountpoint returned a NULL pointer"), mount_point, direction);

                        None
                    }
                    ptr => {
                        log::debug!(
                            concat!(stringify!($table_type), "::lookup_mount_point found entry with  mount point {:?}"),
                            mount_point
                        );

                        let entry = owning_ref_from_ptr!(table, $table_entry_type, ptr);

                        Some(entry)
                    }
                }
            }

            #[doc = concat!("Searches the table from **start** to **end**, and returns the first [`", stringify!($table_entry_type), "`] matching")]
            /// the given `mount_point`.
            pub fn find_mount_point<T>(&mut self, mount_point: T) -> Option<&$table_entry_type>
            where
                T: AsRef<Path>,
            {
                let mount_point = mount_point.as_ref();
                log::debug!(concat!(stringify!($table_type), "::find_mount_point searching table from top to bottom for entry matching mount point {:?}"), mount_point);

                Self::lookup_mount_point(self, Direction::Forward, mount_point)
            }

            #[doc = concat!("Searches the table from **end** to **start**, and returns the first [`", stringify!($table_entry_type), "`] matching")]
            /// the given `mount_point`.
            pub fn find_back_mount_point<T>(&mut self, mount_point: T) -> Option<&$table_entry_type>
            where
                T: AsRef<Path>,
            {
                let mount_point = mount_point.as_ref();
                log::debug!(concat!(stringify!($table_type), "::find_back_mount_point searching table from bottom to top for entry matching mount point {:?}"), mount_point);

                Self::lookup_mount_point(self, Direction::Backward, mount_point)
            }

        }
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_iter_mut {
    ($table_type:ident, $table_entry_type:ident, $table_error_type:ident) => {
        paste::paste!{

            // From dependency library

            // From standard library
            use std::ops::IndexMut;

            // From this library
            use $crate::core::iter::[<$table_type IterMut>];

            #[allow(dead_code)]
            impl $table_type {
                //---- BEGIN iterators

                 #[doc = concat!("Returns an iterator over mutable [`", stringify!($table_type), "`] entries.")]
                ///
                /// # Panics
                ///
                #[doc = concat!("Panics if it fails to create a [`", stringify!($table_type), "IterMut`].")]
                pub fn iter_mut(&mut self) -> [<$table_type IterMut>] {
                    log::debug!(concat!(stringify!($table_type), "::iter_mut creating a new `", stringify!($table_type), "IterMut`"));

                    [<$table_type IterMut>]::new(self).unwrap()
                }

                //---- END iterators

                //---- BEGIN getters

                /// Returns a mutable reference to an element at `index`, or `None` if out of bounds.
                pub fn get_mut(&mut self, index: usize) -> Option<&mut $table_entry_type> {
                    log::debug!(concat!(stringify!($table_type), "::get_mut getting mutable reference of item at index: {:?}"), index);

                    let mut iter = [<$table_type IterMut>]::new(self).unwrap();

                    iter.nth(index)
                }

                //---- END getters

            } //---- END impl

            impl IndexMut<usize> for $table_type {
                /// Performs the mutable indexing (`container\[index]`) operation.
                fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                    log::debug!(concat!(stringify!($table_type), "::index getting item at index: {:?}"), index);

                    #[cold]
                    #[inline(never)]
                    #[track_caller]
                    fn indexing_failed() -> ! {
                        panic!("Index out of bounds");
                    }

                    let mut iter = [<$table_type IterMut>]::new(self).unwrap();

                    match iter.nth(index) {
                        Some(item) => item,
                        None => indexing_failed(),
                    }

                }
            }
        } //---- END paste
    };
}
