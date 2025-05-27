// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;
use std::num::NonZeroUsize;

// From this library
use crate::core::entries::FsTabEntry;
use crate::core::errors::FsTabIterError;
use crate::core::iter::Direction;
use crate::core::iter::GenIterator;
use crate::owning_ref_from_ptr;
use crate::tables::FsTab;

/// Iterator over [`FsTab`] entries.
#[derive(Debug)]
pub struct FsTabIter<'table> {
    table: &'table FsTab,
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

impl<'table> FsTabIter<'table> {
    /// Creates a new `FsTabIter`.
    #[allow(dead_code)]
    pub(crate) fn new(table: &'table FsTab) -> Result<FsTabIter<'table>, FsTabIterError> {
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

            result = unsafe {
                libmount::mnt_table_next_fs(
                    self.table.inner,
                    self.fwd_iter.inner,
                    entry_ptr.as_mut_ptr(),
                )
            };

            match result {
                0 => {
                    let ptr = unsafe { entry_ptr.assume_init() };

                    // Per the documentation of `DoubleEndedIterator`
                    // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                    if self.have_iterators_met
                        || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                    {
                        log::debug!(
 "FsTabIter::advance_to forward and backward iterators have met in the middle"
                                    );

                        self.have_iterators_met = true;

                        return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                    } else {
                        log::debug!("FsTabIter::advance_to advanced to the {i}th table entry");

                        self.fwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!("FsTabIter::advance_to reached the end of `FsTabIter`");

                    return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                }
                code => {
                    log::debug!( "FsTabIter::advance_to failed to advance to {i:?}th table entry. libmount::mnt_table_next_fs returned error code: {code:?}");

                    return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                }
            }
        }

        Ok(())
    }
}

impl<'table> Iterator for FsTabIter<'table> {
    type Item = &'table FsTabEntry;

    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("FsTabIter::next getting next table entry");

        let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe {
            libmount::mnt_table_next_fs(
                self.table.inner,
                self.fwd_iter.inner,
                entry_ptr.as_mut_ptr(),
            )
        };

        match result {
            0 => {
                let ptr = unsafe { entry_ptr.assume_init() };

                // Per the documentation of `DoubleEndedIterator`
                // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                if self.have_iterators_met
                    || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                {
                    log::debug!(
                        "FsTabIter::next forward and backward iterators have met in the middle"
                    );

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!("FsTabIter::next got next table entry");

                    self.fwd_cursor = ptr;
                    let entry = owning_ref_from_ptr!(self.table, FsTabEntry, ptr);

                    Some(entry)
                }
            }
            1 => {
                log::debug!("FsTabIter::next reached end of `FsTabIter`");

                None
            }
            code => {
                log::debug!( "FsTabIter::next failed to get next table entry. libmount::mnt_table_next_fs returned error code: {code:?}");

                None
            }
        }
    }

    // Skips n-1 entries, and updates cursor.
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!("FsTabIter::nth getting {n}th table entry");

        let mut result;

        for i in 0..n {
            let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

            result = unsafe {
                libmount::mnt_table_next_fs(
                    self.table.inner,
                    self.fwd_iter.inner,
                    entry_ptr.as_mut_ptr(),
                )
            };

            match result {
                0 => {
                    let ptr = unsafe { entry_ptr.assume_init() };

                    // Per the documentation of `DoubleEndedIterator`
                    // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                    if self.have_iterators_met
                        || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                    {
                        log::debug!(
                            "FsTabIter::nth forward and backward iterators have met in the middle"
                        );

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!("FsTabIter::nth got {i}th table entry");

                        self.fwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!("FsTabIter::nth reached the end of `FsTabIter`");

                    return None;
                }
                code => {
                    log::debug!( "FsTabIter::nth failed to get {i:?}th table entry. libmount::mnt_table_next_fs returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next()
    }
}

impl<'table> DoubleEndedIterator for FsTabIter<'table> {
    fn next_back(&mut self) -> Option<Self::Item> {
        log::debug!("FsTabIter::next_back getting next table entry from the back");

        let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe {
            libmount::mnt_table_next_fs(
                self.table.inner,
                self.bwd_iter.inner,
                entry_ptr.as_mut_ptr(),
            )
        };

        match result {
            0 => {
                let ptr = unsafe { entry_ptr.assume_init() };

                // Per the documentation of `DoubleEndedIterator`
                // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                if self.have_iterators_met
                    || (self.fwd_cursor != self.bwd_cursor && ptr == self.fwd_cursor)
                {
                    log::debug!(
 "FsTabIter::next_back forward and backward iterators have met in the middle"
                                );

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!("FsTabIter::next_back got next table entry");

                    self.bwd_cursor = ptr;
                    let entry = owning_ref_from_ptr!(self.table, FsTabEntry, ptr);

                    Some(entry)
                }
            }
            1 => {
                log::debug!("FsTabIter::next_back reached the beginning of `FsTabIter`");

                None
            }
            code => {
                log::debug!(concat!(stringify!(FsTab), "Iter::next_back failed to get next table entry from the back. libmount::mnt_table_next_fs returned error code: {:?}"), code);

                None
            }
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!(
            concat!(
                stringify!(FsTab),
                "Iter::nth_back getting {}th table entry from the back"
            ),
            n
        );

        let mut result;

        for i in 0..n {
            let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

            result = unsafe {
                libmount::mnt_table_next_fs(
                    self.table.inner,
                    self.bwd_iter.inner,
                    entry_ptr.as_mut_ptr(),
                )
            };

            match result {
                0 => {
                    let ptr = unsafe { entry_ptr.assume_init() };

                    // Per the documentation of `DoubleEndedIterator`
                    // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                    if self.have_iterators_met
                        || (self.fwd_cursor != self.bwd_cursor && ptr == self.fwd_cursor)
                    {
                        log::debug!(concat!(
                            stringify!(FsTab),
                            "Iter::nth_back forward and backward iterators have met in the middle"
                        ));

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!(
                            concat!(
                                stringify!(FsTab),
                                "Iter::nth_back got {}th table entry from the back"
                            ),
                            i
                        );

                        self.bwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!(concat!(
                        stringify!(FsTab),
                        "Iter::nth_back reached the beginning of `FsTabIter`"
                    ));

                    return None;
                }
                code => {
                    log::debug!( "FsTabIter::nth_back failed to get {i:?}th table entry from the back. libmount::mnt_table_next_fs returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next_back()
    }
}
