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
use crate::owning_mut_from_ptr;
use crate::tables::UTab;

/// Iterator over mutable [`UTab`] entries.
#[derive(Debug)]
pub struct UTabIterMut<'table> {
    table: &'table mut UTab,
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

impl<'table> UTabIterMut<'table> {
    /// Creates a new `UTabIterMut`.
    #[allow(dead_code)]
    pub(crate) fn new(table: &'table mut UTab) -> Result<UTabIterMut<'table>, UTabIterError> {
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
 "UTabIterMut::advance_to forward and backward iterators have met in the middle"
                                    );

                        self.have_iterators_met = true;

                        return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                    } else {
                        log::debug!("UTabIterMut::advance_to advanced to the {i}th table entry");

                        self.fwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!("UTabIterMut::advance_to reached the end of `UTabIterMut");

                    return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                }
                code => {
                    log::debug!( "UTabIterMut::advance_to failed to advance to {i:?}th table entry. libmount::mnt_table_next_fs returned error code: {code:?}");

                    return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
                }
            }
        }

        Ok(())
    }
}

impl<'table> Iterator for UTabIterMut<'table> {
    type Item = &'table mut UTabEntry;

    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("UTabIterMut::next getting next table entry");

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
                        "UTabIterMut::next forward and backward iterators have met in the middle"
                    );

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!("UTabIterMut::next got next table entry");

                    self.fwd_cursor = ptr;
                    let entry = owning_mut_from_ptr!(self.table, UTabEntry, ptr);

                    Some(entry)
                }
            }
            1 => {
                log::debug!("UTabIterMut::next reached the end of `UTabIterMut");

                None
            }
            code => {
                log::debug!( "UTabIterMut::next failed to get next table entry. libmount::mnt_table_next_fs returned error code: {code:?}");

                None
            }
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!(
            concat!(
                stringify!(UTab),
                "UTabIterMut::nth getting {}th table entry"
            ),
            n
        );

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
 "UTabIterMut::nth forward and backward iterators have met in the middle"
                                    );

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!("UTabIterMut::nth got {i}th table entry");

                        self.fwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!("UTabIterMut::next reached the end of `UTabIterMut");

                    return None;
                }
                code => {
                    log::debug!( "UTabIterMut::next failed to get the {i:?}th table entry. libmount::mnt_table_next_fs returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next()
    }
}

impl<'table> DoubleEndedIterator for UTabIterMut<'table> {
    fn next_back(&mut self) -> Option<Self::Item> {
        log::debug!("UTabIterMut::next_back getting next table entry from the back");

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
 "UTabIterMut::next_back forward and backward iterators have met in the middle"
                                );

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!("UTabIterMut::next_back got next table entry from the back");

                    self.bwd_cursor = ptr;
                    let entry = owning_mut_from_ptr!(self.table, UTabEntry, ptr);

                    Some(entry)
                }
            }
            1 => {
                log::debug!("UTabIterMut::next_back reached the beginning of `UTabIterMut");

                None
            }
            code => {
                log::debug!( "UTabIterMut::next_back failed to get next table entry from the back. libmount::mnt_table_next_fs returned error code: {code:?}");

                None
            }
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!(
            concat!(
                stringify!(UTab),
                "UTabIterMut::nth getting {}th table entry"
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
                        log::debug!(
 "UTabIterMut::nth_back forward and backward iterators have met in the middle"
                                    );

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!("UTabIterMut::nth_back got {i}th table entry from the back");

                        self.bwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!("UTabIterMut::nth_back reached the beginning of `UTabIterMut");

                    return None;
                }
                code => {
                    log::debug!( "UTabIterMut::nth_back failed to get the {i:?}th table entry from the back. libmount::mnt_table_next_fs returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next_back()
    }
}
