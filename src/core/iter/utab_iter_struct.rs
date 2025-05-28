// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;
use std::num::NonZeroUsize;

// From this library
use crate::core::entries::UTabEntry;
use crate::core::errors::UTabIterError;
use crate::core::iter::{Direction, GenIterator};
use crate::owning_ref_from_ptr;
use crate::tables::UTab;

/// Iterator over [`UTab`] entries.
#[derive(Debug)]
pub struct UTabIter<'table> {
    table: &'table UTab,
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

impl<'table> UTabIter<'table> {
    /// Creates a new `UTabIter`.
    #[allow(dead_code)]
    pub(crate) fn new(table: &'table UTab) -> Result<UTabIter<'table>, UTabIterError> {
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
                         "UTabIter::advance_to forward and backward iterators have met in the middle"
                                    );

                        self.have_iterators_met = true;

                        return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                    } else {
                        log::debug!("UTabIter::advance_to advanced to the {i}th table entry");

                        self.fwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!("UTabIter::advance_to reached the end of `UTabIter`");

                    return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                }
                code => {
                    log::debug!( "UTabIter::advance_to failed to advance to {i:?}th table entry. libmount::mnt_table_next_fs returned error code: {code:?}");

                    return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                }
            }
        }

        Ok(())
    }
}

impl<'table> Iterator for UTabIter<'table> {
    type Item = &'table UTabEntry;

    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("UTabIter::next getting next table entry");

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
                        "UTabIter::next forward and backward iterators have met in the middle"
                    );

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!("UTabIter::next got next table entry");

                    self.fwd_cursor = ptr;
                    let entry = owning_ref_from_ptr!(self.table, UTabEntry, ptr);

                    Some(entry)
                }
            }
            1 => {
                log::debug!("UTabIter::next reached end of `UTabIter`");

                None
            }
            code => {
                log::debug!( "UTabIter::next failed to get next table entry. libmount::mnt_table_next_fs returned error code: {code:?}");

                None
            }
        }
    }

    // Skips n-1 entries, and updates cursor.
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!("UTabIter::nth getting {n}th table entry");

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
                            "UTabIter::nth forward and backward iterators have met in the middle"
                        );

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!("UTabIter::nth got {i}th table entry");

                        self.fwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!("UTabIter::nth reached the end of `UTabIter`");

                    return None;
                }
                code => {
                    log::debug!( "UTabIter::nth failed to get {i:?}th table entry. libmount::mnt_table_next_fs returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next()
    }
}

impl<'table> DoubleEndedIterator for UTabIter<'table> {
    fn next_back(&mut self) -> Option<Self::Item> {
        log::debug!("UTabIter::next_back getting next table entry from the back");

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
                        "UTabIter::next_back forward and backward iterators have met in the middle"
                    );

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!("UTabIter::next_back got next table entry");

                    self.bwd_cursor = ptr;
                    let entry = owning_ref_from_ptr!(self.table, UTabEntry, ptr);

                    Some(entry)
                }
            }
            1 => {
                log::debug!("UTabIter::next_back reached the beginning of `UTabIter`");

                None
            }
            code => {
                log::debug!( "UTabIter::next_back failed to get next table entry from the back. libmount::mnt_table_next_fs returned error code: {code:?}");

                None
            }
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!("UTabIter::nth_back getting {n}th table entry from the back");

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
                        log::debug!(
                            "UTabIter::nth_back forward and backward iterators have met in the middle"
                        );

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!("UTabIter::nth_back got {i}th table entry from the back");

                        self.bwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!("UTabIter::nth_back reached the beginning of `UTabIter`");

                    return None;
                }
                code => {
                    log::debug!( "UTabIter::nth_back failed to get {i:?}th table entry from the back. libmount::mnt_table_next_fs returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next_back()
    }
}
