// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_child_iter {
    ($table_type:ident, $table_entry_type:ident) => {
        use paste::paste;

        paste! {
                    // From dependency library

                    // From standard library
                    use std::mem::MaybeUninit;

                    // From this library
                    use $crate::owning_ref_from_ptr;
                    use $crate::core::iter::{Direction, GenIterator};
                    use $crate::core::errors::[<$table_type ChildIterError>];
                    use $crate::core::entries::$table_entry_type;
                    use $crate::tables::$table_type;

                    #[doc = concat!("Iterator over the children of [`", stringify!($table_type), "`] entries.")]
                    #[derive(Debug)]
                    pub struct [<$table_type ChildIter>]<'table> {
                        table: &'table $table_type,
                        parent: &'table $table_entry_type,
                        /// Forward iterator.
                        fwd_iter: GenIterator,
                        /// Backward iterator.
                        bwd_iter: GenIterator,
                        /// Current item in forward iteration.
                        fwd_cursor: *mut libmount::libmnt_fs,
                        /// Current item in backward iteration.
                        bwd_cursor: *mut libmount::libmnt_fs,
                        /// Indicator of forward and backward iterators meeting in the middle.
                        have_iterators_met: bool,
                    }

                    impl<'table> [<$table_type ChildIter>]<'table> {
                        #[doc = concat!("Creates a new `", stringify!($table_type), "ChildIter`.")]
                        #[allow(dead_code)]
                        pub(crate) fn new(
                            table: &'table $table_type,
                            parent: &'table $table_entry_type,
                        ) -> Result<[<$table_type ChildIter>]<'table>, [<$table_type ChildIterError>]> {
                            let fwd_iter = GenIterator::new(Direction::Forward)?;
                            let bwd_iter = GenIterator::new(Direction::Backward)?;
                            let fwd_cursor = std::ptr::null_mut();
                            let bwd_cursor = std::ptr::null_mut();
                            let have_iterators_met = false;

                            let iterator = Self {
                                table,
                                parent,
                                fwd_iter,
                                bwd_iter,
                                fwd_cursor,
                                bwd_cursor,
                                have_iterators_met,
                            };

                            Ok(iterator)
                        }
                    }


                    impl<'table> Iterator for [<$table_type ChildIter>]<'table> {
                        type Item = &'table $table_entry_type;

                        fn next(&mut self) -> Option<Self::Item> {
                            log::debug!(concat!(stringify!($table_type), "ChildIter::next getting next child of parent `", stringify!($table_entry_type), "`"));

                            let mut child_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                            let result = unsafe {  libmount::mnt_table_next_child_fs(
                                self.table.inner,
                                self.fwd_iter.inner,
                                self.parent.inner,
                                child_ptr.as_mut_ptr(),
                            )};

                            match result {
                                0 => {
                                    let ptr = unsafe { child_ptr.assume_init() };

                                    // Per the documentation of `DoubleEndedIterator`
                                    // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                                    if self.have_iterators_met
                                        || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                                    {
                                        log::debug!(
                                            concat!(stringify!($table_type), "ChildIter::next forward and backward iterators have met in the middle")
                                        );

                                        self.have_iterators_met = true;

                                        None
                                    } else {
                                        log::debug!(concat!(stringify!($table_type), "ChildIter::next got next child of parent `", stringify!($table_entry_type), "`"));

                                        self.fwd_cursor = ptr;
                                        let child = owning_ref_from_ptr!(self.table, $table_entry_type, ptr);

                                        Some(child)
                                    }
                                }
                                1 => {
                                    log::debug!(concat!(stringify!($table_type), "ChildIter::next reached the end of `", stringify!($table_type), "ChildIter`"));

                                    None
                                }
                                code => {
                                    log::debug!(concat!(stringify!($table_type), "ChildIter::next failed to get next child. libmount::mnt_table_next_child_fs returned error code: {:?}"), code);

                                    None
                                }
                            }
                        }

                        fn nth(&mut self, n: usize) -> Option<Self::Item> {
                            log::debug!(concat!(stringify!($table_type), "ChildIter::nth getting {}th child of `", stringify!($table_entry_type), "`"), n);

                            let mut result;

                            for i in 0..n {
                                let mut child_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                                result = unsafe {  libmount::mnt_table_next_child_fs(
                                    self.table.inner,
                                    self.fwd_iter.inner,
                                    self.parent.inner,
                                    child_ptr.as_mut_ptr(),
                                )};

                                match result {
                                    0 => {
                                        let ptr = unsafe { child_ptr.assume_init() };

                                        // Per the documentation of `DoubleEndedIterator`
                                        // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                                        if self.have_iterators_met
                                            || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                                        {
                                            log::debug!(
                                                concat!(stringify!($table_type), "ChildIter::nth forward and backward iterators have met in the middle")
                                            );

                                            self.have_iterators_met = true;

                                            return None;
                                        } else {
                                            log::debug!(concat!(stringify!($table_type), "ChildIter::nth got {}th child of `", stringify!($table_entry_type), "`"), i);

                                            self.fwd_cursor = ptr;
                                        }
                                    }
                                    1 => {
                                        log::debug!(concat!(stringify!($table_type), "ChildIter::nth reached the end of `", stringify!($table_type), "ChildIter`"));

                                        return None;
                                    }
                                    code => {
                                        log::debug!(concat!(stringify!($table_type), "ChildIter::nth failed to get {:?}th child. libmount::mnt_table_next_child_fs returned error code: {:?}"), i, code);

                                        return None;
                                    }
                                }
                            }

                            self.next()
                        }
                    }

                    impl<'table> DoubleEndedIterator for [<$table_type ChildIter>]<'table> {
                        fn next_back(&mut self) -> Option<Self::Item> {
                            log::debug!(concat!(stringify!($table_type), "ChildIter::next_back getting next table entry from the back"));

                            let mut child_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                            let result = unsafe {  libmount::mnt_table_next_child_fs(
                                self.table.inner,
                                self.bwd_iter.inner,
                                self.parent.inner,
                                child_ptr.as_mut_ptr(),
                            )};

                            match result {
                                0 => {
                                    let ptr = unsafe { child_ptr.assume_init() };

                                    // Per the documentation of `DoubleEndedIterator`
                                    // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                                    if self.have_iterators_met
                                        || (self.fwd_cursor != self.bwd_cursor && ptr == self.fwd_cursor)
                                    {
                                        log::debug!(
                                            concat!(stringify!($table_type), "ChildIter::next_back forward and backward iterators have met in the middle")
                                        );

                                        self.have_iterators_met = true;

                                        None
                                    } else {
                                        log::debug!(concat!(stringify!($table_type), "ChildIter::next_back got next child of parent `", stringify!($table_entry_type), "`"));

                                        self.bwd_cursor = ptr;
                                        let child = owning_ref_from_ptr!(self.table, $table_entry_type, ptr);

                                        Some(child)
                                    }
                                }
                                1 => {
                                    log::debug!(concat!(stringify!($table_type), "ChildIter::next_back reached the beginning of `", stringify!($table_type ), "ChildIter`"));

                                    None
                                }
                                code => {
                                    log::debug!(concat!(stringify!($table_type), "ChildIter::next_back failed to get next child from the back. libmount::mnt_table_next_child_fs returned error code: {:?}"), code);

                                    None
                                }
                            }
                        }

                        fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                            log::debug!(concat!(stringify!($table_type), "ChildIter::nth_back getting {}th child of `", stringify!($table_entry_type), "` from the back"), n);

                            let mut result;

                            // Skips n-1 children, and updates cursor.
                            for i in 0..n {
                                let mut child_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                                result = unsafe {  libmount::mnt_table_next_child_fs(
                                    self.table.inner,
                                    self.bwd_iter.inner,
                                    self.parent.inner,
                                    child_ptr.as_mut_ptr(),
                                )};

                                match result {
                                    0 => {
                                        let ptr = unsafe { child_ptr.assume_init() };

                                        // Per the documentation of `DoubleEndedIterator`
                                        // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                                        if self.have_iterators_met
                                            || (self.fwd_cursor != self.bwd_cursor && ptr == self.fwd_cursor)
                                        {
                                            log::debug!(
                                                concat!(stringify!($table_type), "ChildIter::nth_back forward and backward iterators have met in the middle")
                                            );

                                            self.have_iterators_met = true;

                                            return None;
                                        } else {
                                            log::debug!(concat!(stringify!($table_type), "ChildIter::nth_back got {}th child of `", stringify!($table_entry_type), "` from the back"), i);

                                            self.bwd_cursor = ptr;
                                        }
                                    }
                                    1 => {
                                        log::debug!(concat!(stringify!($table_type), "ChildIter::nth_back reached the beginning of `", stringify!($table_type ), "ChildIter`"));

                                        return None;
                                    }
                                    code => {
                                        log::debug!(concat!(stringify!($table_type), "ChildIter::nth_back failed to get {:?}th child from the back. libmount::mnt_table_next_child_fs returned error code: {:?}"), i, code);

                                        return None;
                                    }
                                }
                            }

                            self.next_back()
                        }
                    }
                } //---- END paste
    };
}
