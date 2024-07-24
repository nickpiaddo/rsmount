// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_table_iter {
    ($table_type:ident, $table_entry_type:ident) => {
        paste::paste! {
            // From dependency library

            // From standard library
            use std::mem::MaybeUninit;
            use std::num::NonZeroUsize;

            // From this library
            use $crate::owning_ref_from_ptr;
            use $crate::core::iter::{Direction, GenIterator};
            use $crate::core::errors::[<$table_type IterError>];
            use $crate::core::entries::$table_entry_type;
            use $crate::tables::$table_type;

            #[doc = concat!("Iterator over [`", stringify!($table_type), "`] entries.")]
            #[derive(Debug)]
            pub struct [<$table_type Iter>]<'table> {
                table: &'table $table_type,
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

            impl<'table> [<$table_type Iter>]<'table> {
                #[doc = concat!("Creates a new `", stringify!($table_type), "Iter`.")]
                #[allow(dead_code)]
                pub(crate) fn new(
                    table: &'table $table_type,
                ) -> Result<[<$table_type Iter>]<'table>, [<$table_type IterError>]> {
                    let fwd_iter = GenIterator::new(Direction::Forward)?;
                    let bwd_iter = GenIterator::new(Direction::Backward)?;
                    let fwd_cursor = std::ptr::null_mut();
                    let bwd_cursor = std::ptr::null_mut();
                    let have_iterators_met = false;

                    let iterator = Self {
                        table,
                        fwd_iter,
                        bwd_iter,
                        fwd_cursor,
                        bwd_cursor,
                        have_iterators_met,
                    };

                    Ok(iterator)
                }

                /// Advances the iterator by `n` elements.
                ///
                /// This method will eagerly skip `n` elements. The following call to `next()` will
                /// yield the element at index `n+1`.
                ///
                /// `advance_to(n)` will return `Ok(())` if the iterator successfully advances by
                /// `n` elements, or a `Err(NonZeroUsize)` with value `k`, if an eror occurs or if
                /// it reaches the end of the iterator, where `k` is the remaining number of steps that
                /// could not be advanced because the iterator ran out.
                ///
                /// If `self` is empty and `n` is non-zero, then this returns `Err(n)`.  Otherwise,
                /// `k` is always less than `n`.
                pub fn advance_to(&mut self, n: usize) -> Result<(), NonZeroUsize> {
                    // Reset iterator
                    self.fwd_iter.reset();
                    self.bwd_iter.reset();
                    self.fwd_cursor = std::ptr::null_mut();
                    self.bwd_cursor = std::ptr::null_mut();
                    self.have_iterators_met = false;

                    let mut result;

                    for i in 0..=n {

                        let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                        result = unsafe {  libmount::mnt_table_next_fs(
                            self.table.inner,
                            self.fwd_iter.inner,
                            entry_ptr.as_mut_ptr(),
                        )};

                        match result {
                            0 => {
                                let ptr = unsafe { entry_ptr.assume_init() };

                                // Per the documentation of `DoubleEndedIterator`
                                // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                                if self.have_iterators_met
                                    || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                                {
                                    log::debug!(
                                        concat!(stringify!($table_type), "Iter::advance_to forward and backward iterators have met in the middle")
                                    );

                                    self.have_iterators_met = true;

                                    return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                                } else {
                                    log::debug!(concat!(stringify!($table_type), "Iter::advance_to advanced to the {}th table entry"), i);

                                    self.fwd_cursor = ptr;
                                }
                            }
                            1 => {
                                log::debug!(concat!(stringify!($table_type), "Iter::advance_to reached the end of `", stringify!($table_type), "Iter`"));

                                return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                            }
                            code => {
                                log::debug!(concat!(stringify!($table_type), "Iter::advance_to failed to advance to {:?}th table entry. libmount::mnt_table_next_fs returned error code: {:?}"), i, code);

                                return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                            }
                        }
                    }

                    Ok(())
                }
            }


            impl<'table> Iterator for [<$table_type Iter>]<'table> {
                type Item = &'table $table_entry_type;

                fn next(&mut self) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "Iter::next getting next table entry"));

                    let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                    let result = unsafe {  libmount::mnt_table_next_fs(
                        self.table.inner,
                        self.fwd_iter.inner,
                        entry_ptr.as_mut_ptr(),
                    )};

                    match result {
                        0 => {
                            let ptr = unsafe { entry_ptr.assume_init() };

                            // Per the documentation of `DoubleEndedIterator`
                            // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                            if self.have_iterators_met
                                || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                            {
                                log::debug!(
                                    concat!(stringify!($table_type), "Iter::next forward and backward iterators have met in the middle")
                                );

                                self.have_iterators_met = true;

                                None
                            } else {
                                log::debug!(concat!(stringify!($table_type), "Iter::next got next table entry"));

                                self.fwd_cursor = ptr;
                                let entry = owning_ref_from_ptr!(self.table, $table_entry_type, ptr);

                                Some(entry)
                            }
                        }
                        1 => {
                            log::debug!(concat!(stringify!($table_type), "Iter::next reached end of `", stringify!($table_type), "Iter`"));

                            None
                        }
                        code => {
                            log::debug!(concat!(stringify!($table_type), "Iter::next failed to get next table entry. libmount::mnt_table_next_fs returned error code: {:?}"), code);

                            None
                        }
                    }
                }

                // Skips n-1 entries, and updates cursor.
                fn nth(&mut self, n: usize) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "Iter::nth getting {}th table entry"), n);

                    let mut result;

                    for i in 0..n {
                        let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                        result = unsafe {  libmount::mnt_table_next_fs(
                            self.table.inner,
                            self.fwd_iter.inner,
                            entry_ptr.as_mut_ptr(),
                        )};

                        match result {
                            0 => {
                                let ptr = unsafe { entry_ptr.assume_init() };

                                // Per the documentation of `DoubleEndedIterator`
                                // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                                if self.have_iterators_met
                                    || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                                {
                                    log::debug!(
                                        concat!(stringify!($table_type), "Iter::nth forward and backward iterators have met in the middle")
                                    );

                                    self.have_iterators_met = true;

                                    return None;
                                } else {
                                    log::debug!(concat!(stringify!($table_type), "Iter::nth got {}th table entry"), i);

                                    self.fwd_cursor = ptr;
                                }
                            }
                            1 => {
                                log::debug!(concat!(stringify!($table_type), "Iter::nth reached the end of `", stringify!($table_type), "Iter`"));

                                return None;
                            }
                            code => {
                                log::debug!(concat!(stringify!($table_type), "Iter::nth failed to get {:?}th table entry. libmount::mnt_table_next_fs returned error code: {:?}"), i, code);

                                return None;
                            }
                        }
                    }

                    self.next()
                }
            }

            impl<'table> DoubleEndedIterator for [<$table_type Iter>]<'table> {
                fn next_back(&mut self) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "Iter::next_back getting next table entry from the back"));

                    let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                    let result = unsafe { libmount::mnt_table_next_fs(
                        self.table.inner,
                        self.bwd_iter.inner,
                        entry_ptr.as_mut_ptr(),
                    ) };

                    match result {
                        0 => {
                            let ptr = unsafe { entry_ptr.assume_init() };

                            // Per the documentation of `DoubleEndedIterator`
                            // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                            if self.have_iterators_met
                                || (self.fwd_cursor != self.bwd_cursor && ptr == self.fwd_cursor)
                            {
                                log::debug!(
                                    concat!(stringify!($table_type), "Iter::next_back forward and backward iterators have met in the middle")
                                );

                                self.have_iterators_met = true;

                                None
                            } else {
                                log::debug!(concat!(stringify!($table_type), "Iter::next_back got next table entry"));

                                self.bwd_cursor = ptr;
                                let entry = owning_ref_from_ptr!(self.table, $table_entry_type, ptr);

                                Some(entry)
                            }
                        }
                        1 => {
                            log::debug!(concat!(stringify!($table_type), "Iter::next_back reached the beginning of `", stringify!($table_type ), "Iter`"));

                            None
                        }
                        code => {
                            log::debug!(concat!(stringify!($table_type), "Iter::next_back failed to get next table entry from the back. libmount::mnt_table_next_fs returned error code: {:?}"), code);

                            None
                        }
                    }
                }

                fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "Iter::nth_back getting {}th table entry from the back"), n);

                    let mut result;

                    for i in 0..n {
                        let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                        result = unsafe { libmount::mnt_table_next_fs(
                            self.table.inner,
                            self.bwd_iter.inner,
                            entry_ptr.as_mut_ptr(),
                        )};

                        match result {
                            0 => {
                                let ptr = unsafe { entry_ptr.assume_init() };

                                // Per the documentation of `DoubleEndedIterator`
                                // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                                if self.have_iterators_met
                                    || (self.fwd_cursor != self.bwd_cursor && ptr == self.fwd_cursor)
                                {
                                    log::debug!(
                                        concat!(stringify!($table_type), "Iter::nth_back forward and backward iterators have met in the middle")
                                    );

                                    self.have_iterators_met = true;

                                    return None;
                                } else {
                                        log::debug!(concat!(stringify!($table_type), "Iter::nth_back got {}th table entry from the back"), i);

                                    self.bwd_cursor = ptr;
                                }
                            }
                            1 => {
                                log::debug!(concat!(stringify!($table_type), "Iter::nth_back reached the beginning of `", stringify!($table_type ), "Iter`"));

                                return None;
                            }
                            code => {
                                log::debug!(concat!(stringify!($table_type), "Iter::nth_back failed to get {:?}th table entry from the back. libmount::mnt_table_next_fs returned error code: {:?}"), i, code);

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
