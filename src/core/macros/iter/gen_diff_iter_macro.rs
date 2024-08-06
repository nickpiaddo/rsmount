// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_diff_iter {
    ($table_type:ident, $table_entry_type:ident) => {
        paste::paste! {
            // From dependency library

            // From standard library
            use std::mem::MaybeUninit;

            // From this library
            use $crate::core::entries::[<$table_type EntryDiff>];
            use $crate::core::errors::[<$table_type DiffIterError>];
            use $crate::core::iter::{Direction, GenIterator};
            use $crate::tables::[<$table_type Diff>];

            #[doc = concat!("Iterate over [`", stringify!($table_type), "Diff`] entries.")]
            #[derive(Debug)]
            pub struct [<$table_type DiffIter>]<'diff, 's, 'o> {
                table_diff: &'diff [<$table_type Diff>]<'s, 'o>,
                /// Forward iterator.
                fwd_iter: GenIterator,
                /// Backward iterator.
                bwd_iter: GenIterator,
                /// Current item in forward iteration from the first table.
                first_fwd_cursor: *mut libmount::libmnt_fs,
                /// Current item in forward iteration from the second table.
                second_fwd_cursor: *mut libmount::libmnt_fs,
                /// Current item in backward iteration from the first table.
                first_bwd_cursor: *mut libmount::libmnt_fs,
                /// Current item in backward iteration from the second table.
                second_bwd_cursor: *mut libmount::libmnt_fs,
                /// Indicator of forward and backward iterators meeting in the middle.
                have_iterators_met: bool,
            }

            impl<'diff, 's, 'o> [<$table_type DiffIter>]<'diff, 's, 'o> {
                #[doc(hidden)]
                /// Creates a new instance.
                #[allow(dead_code)]
                pub(crate) fn new(
                    table_diff: &'diff [<$table_type Diff>]<'s, 'o>,
                ) -> Result<[<$table_type DiffIter>]<'diff, 's, 'o>, [<$table_type DiffIterError>]> {
                    let fwd_iter = GenIterator::new(Direction::Forward)?;
                    let bwd_iter = GenIterator::new(Direction::Backward)?;
                    let first_fwd_cursor = std::ptr::null_mut();
                    let second_fwd_cursor = std::ptr::null_mut();
                    let first_bwd_cursor = std::ptr::null_mut();
                    let second_bwd_cursor = std::ptr::null_mut();
                    let have_iterators_met = false;

                    let iterator = Self {
                        table_diff,
                        fwd_iter,
                        bwd_iter,
                        first_fwd_cursor,
                        second_fwd_cursor,
                        first_bwd_cursor,
                        second_bwd_cursor,
                        have_iterators_met,
             };

                    Ok(iterator)
                }
            }

            impl<'diff, 's, 'o> Iterator for [<$table_type DiffIter>]<'diff, 's, 'o> {
                type Item = [<$table_entry_type Diff>]<'diff, 's, 'o>;

                fn next(&mut self) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "DiffIter::next getting next table changes"));

                    let mut source_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
                    let mut other_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
                    let mut flags = MaybeUninit::<libc::c_int>::zeroed();

                    let result = unsafe { libmount::mnt_tabdiff_next_change(
                        self.table_diff.inner,
                        self.fwd_iter.inner,
                        source_entry_inner.as_mut_ptr(),
                        other_entry_inner.as_mut_ptr(),
                        flags.as_mut_ptr(),
                    ) };

                    match result {
                        0 => {
                            let source_entry_inner = unsafe { source_entry_inner.assume_init() };
                            let other_entry_inner = unsafe { other_entry_inner.assume_init() };
                            let flags = unsafe { flags.assume_init() };

                            // Per the documentation of `DoubleEndedIterator`
                            // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                            if self.have_iterators_met
                                || ((self.first_fwd_cursor != self.first_bwd_cursor)
                                    && (self.second_fwd_cursor != self.second_bwd_cursor)
                                    && (source_entry_inner == self.first_bwd_cursor
                                        && other_entry_inner == self.second_bwd_cursor))
                            {
                                log::debug!(concat!(stringify!($table_type), "DiffIter::next forward and backward iterators have met in the middle"));

                                self.have_iterators_met = true;

                                None
                            } else {
                                log::debug!(concat!(stringify!($table_type), "DiffIter::next got next table changes"));
                                self.first_fwd_cursor = source_entry_inner;
                                self.second_fwd_cursor = other_entry_inner;
                                let diff_entry =
                                    [<$table_entry_type Diff>]::new(self.table_diff, source_entry_inner, other_entry_inner, flags);

                                Some(diff_entry)
                            }
                        }
                        1 => {
                            log::debug!(concat!(stringify!($table_type), "DiffIter::next reached the end of `", stringify!($table_type), "DiffIter`"));

                            None
                        }
                        code => {
                            let err_msg = "failed to get next changes".to_owned();
                            log::debug!(concat!(stringify!($table_type), "DiffIter::next {}. libmount::mnt_tabdiff_next_change returned error code: {:?}"), err_msg, code);

                            None
                        }
                    }
                }

                fn nth(&mut self, n: usize) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "DiffIter::nth getting {:?}th table diff item"), n);

                    let mut result;
                    let mut flags = MaybeUninit::<libc::c_int>::zeroed();

                    // Skip n-1 entries, and update cursors along the way.
                    for i in 0..n {
                        let mut source_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
                        let mut other_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                        result = unsafe { libmount::mnt_tabdiff_next_change(
                            self.table_diff.inner,
                            self.fwd_iter.inner,
                            source_entry_inner.as_mut_ptr(),
                            other_entry_inner.as_mut_ptr(),
                            flags.as_mut_ptr(),
                        ) };

                        match result {
                            0 => {
                                let source_entry_inner = unsafe { source_entry_inner.assume_init() };
                                let other_entry_inner = unsafe { other_entry_inner.assume_init() };

                                // Per the documentation of `DoubleEndedIterator`
                                // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                                if self.have_iterators_met
                                    || ((self.first_fwd_cursor != self.first_bwd_cursor)
                                        && (self.second_fwd_cursor != self.second_bwd_cursor)
                                        && (source_entry_inner == self.first_bwd_cursor
                                            && other_entry_inner == self.second_bwd_cursor))
                                {
                                    log::debug!(concat!(stringify!($table_type), "DiffIter::next forward and backward iterators have met in the middle"));

                                    self.have_iterators_met = true;

                                    return None;
                                } else {
                                    log::debug!(concat!(stringify!($table_type), "DiffIter::nth got {:?}th table diff item"), i);
                                    self.first_fwd_cursor = source_entry_inner;
                                    self.second_fwd_cursor = other_entry_inner;
                                }
                            }
                            1 => {
                                log::debug!(concat!(stringify!($table_type), "DiffIter::nth reached the end of `", stringify!($table_type), "DiffIter`"));

                                return None;
                            }
                            code => {
                                let err_msg = format!("failed to get {:?}th table diff item", n);
                                log::debug!(concat!(stringify!($table_type), "DiffIter::nth {}. libmount::mnt_tabdiff_next_change returned error code: {:?}"), err_msg, code);

                                return None;
                            }
                        }
                    }

                    self.next()
                }

            }

            impl<'diff, 's, 'o> DoubleEndedIterator for [<$table_type DiffIter>]<'diff, 's, 'o> {
                fn next_back(&mut self) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "DiffIter::next_back getting next table diff item from the back"));

                    let mut source_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
                    let mut other_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
                    let mut flags = MaybeUninit::<libc::c_int>::zeroed();

                    let result = unsafe { libmount::mnt_tabdiff_next_change(
                        self.table_diff.inner,
                        self.bwd_iter.inner,
                        source_entry_inner.as_mut_ptr(),
                        other_entry_inner.as_mut_ptr(),
                        flags.as_mut_ptr(),
                    )};

                    match result {
                        0 => {
                            let source_entry_inner = unsafe { source_entry_inner.assume_init() };
                            let other_entry_inner = unsafe { other_entry_inner.assume_init() };
                            let flags = unsafe { flags.assume_init() };

                            // Per the documentation of `DoubleEndedIterator`
                            // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                            if self.have_iterators_met
                                || ((self.first_fwd_cursor != self.first_bwd_cursor)
                                    && (self.second_fwd_cursor != self.second_bwd_cursor)
                                    && (source_entry_inner == self.first_fwd_cursor
                                        && other_entry_inner == self.second_fwd_cursor))
                            {
                                log::debug!(concat!(stringify!($table_type), "DiffIter::next_back forward and backward iterators have met in the middle"));

                                self.have_iterators_met = true;

                                None
                            } else {
                                log::debug!(
                                    concat!(stringify!($table_type), "DiffIter::next_back got next table changes iterating backward")
                                );

                                self.first_bwd_cursor = source_entry_inner;
                                self.second_bwd_cursor = other_entry_inner;
                                let diff_entry =
                                    [<$table_entry_type Diff>]::new(self.table_diff, source_entry_inner, other_entry_inner, flags);

                                Some(diff_entry)
                            }
                        }
                        1 => {
                            log::debug!(concat!(stringify!($table_type), "DiffIter::next_back reached the end of `", stringify!($table_type), "DiffIter`"));

                            None
                        }
                        code => {
                            let err_msg = "failed to get next changes iterating backward".to_owned();
                            log::debug!(concat!(stringify!($table_type), "DiffIter::next_back {}. libmount::mnt_tabdiff_next_change returned error code: {:?}"), err_msg, code);

                            None
                        }
                    }
                }

                fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "DiffIter::nth_back getting {:?}th table diff item from the back"), n);

                    let mut result;
                    let mut flags = MaybeUninit::<libc::c_int>::zeroed();

                    // Skip n-1 entries, and update cursors along the way.
                    for i in 0..n {
                        let mut source_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
                        let mut other_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                        result = unsafe { libmount::mnt_tabdiff_next_change(
                            self.table_diff.inner,
                            self.bwd_iter.inner,
                            source_entry_inner.as_mut_ptr(),
                            other_entry_inner.as_mut_ptr(),
                            flags.as_mut_ptr(),
                        )};

                        match result {
                            0 => {
                                let source_entry_inner = unsafe { source_entry_inner.assume_init() };
                                let other_entry_inner = unsafe { other_entry_inner.assume_init() };

                                // Per the documentation of `DoubleEndedIterator`
                                // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                                if self.have_iterators_met
                                    || ((self.first_fwd_cursor != self.first_bwd_cursor)
                                        && (self.second_fwd_cursor != self.second_bwd_cursor)
                                        && (source_entry_inner == self.first_fwd_cursor
                                            && other_entry_inner == self.second_fwd_cursor))
                                {
                                    log::debug!(concat!(stringify!($table_type), "DiffIter::nth_back forward and backward iterators have met in the middle"));

                                    self.have_iterators_met = true;

                                    return None;
                                } else {
                                    log::debug!(concat!(stringify!($table_type), "DiffIter::nth got {:?}th table diff item from the back"), i);
                                    self.first_bwd_cursor = source_entry_inner;
                                    self.second_bwd_cursor = other_entry_inner;
                                }
                            }
                            1 => {
                                log::debug!(concat!(stringify!($table_type), "DiffIter::nth_back reached the beginning of `", stringify!($table_type), "DiffIter`"));

                                return None;
                            }
                            code => {
                                let err_msg = format!("failed to get {:?}th table diff item from the back", n);
                                log::debug!(concat!(stringify!($table_type), "DiffIter::nth_back {}. libmount::mnt_tabdiff_next_change returned error code: {:?}"), err_msg, code);

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
