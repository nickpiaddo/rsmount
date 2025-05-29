// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::entries::MountInfoEntry;
use crate::core::errors::MountInfoChildIterError;
use crate::core::iter::{Direction, GenIterator};
use crate::owning_ref_from_ptr;
use crate::tables::MountInfo;

/// Iterator over the children of [`MountInfo`] entries.
#[derive(Debug)]
pub struct MountInfoChildIter<'table> {
    table: &'table MountInfo,
    parent: &'table MountInfoEntry,
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

impl<'table> MountInfoChildIter<'table> {
    /// Creates a new `MountInfoChildIter`.
    #[allow(dead_code)]
    pub(crate) fn new(
        table: &'table MountInfo,
        parent: &'table MountInfoEntry,
    ) -> Result<MountInfoChildIter<'table>, MountInfoChildIterError> {
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

impl<'table> Iterator for MountInfoChildIter<'table> {
    type Item = &'table MountInfoEntry;

    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("MountInfoChildIter::next getting next child of parent `MountInfoEntry`");

        let mut child_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe {
            libmount::mnt_table_next_child_fs(
                self.table.inner,
                self.fwd_iter.inner,
                self.parent.inner,
                child_ptr.as_mut_ptr(),
            )
        };

        match result {
            0 => {
                let ptr = unsafe { child_ptr.assume_init() };

                // Per the documentation of `DoubleEndedIterator`
                // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                if self.have_iterators_met
                    || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                {
                    log::debug!(
 "MountInfoChildIter::next forward and backward iterators have met in the middle"
                                        );

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!(
                        "MountInfoChildIter::next got next child of parent `MountInfoEntry`"
                    );

                    self.fwd_cursor = ptr;
                    let child = owning_ref_from_ptr!(self.table, MountInfoEntry, ptr);

                    Some(child)
                }
            }
            1 => {
                log::debug!("MountInfoChildIter::next reached the end of `MountInfoChildIter`");

                None
            }
            code => {
                log::debug!( "MountInfoChildIter::next failed to get next child. libmount::mnt_table_next_child_fs returned error code: {code:?}");

                None
            }
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!("MountInfoChildIter::nth getting {n}th child of `MountInfoEntry`");

        let mut result;

        for i in 0..n {
            let mut child_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

            result = unsafe {
                libmount::mnt_table_next_child_fs(
                    self.table.inner,
                    self.fwd_iter.inner,
                    self.parent.inner,
                    child_ptr.as_mut_ptr(),
                )
            };

            match result {
                0 => {
                    let ptr = unsafe { child_ptr.assume_init() };

                    // Per the documentation of `DoubleEndedIterator`
                    // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                    if self.have_iterators_met
                        || (self.fwd_cursor != self.bwd_cursor && ptr == self.bwd_cursor)
                    {
                        log::debug!(
                                             "MountInfoChildIter::nth forward and backward iterators have met in the middle");

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!("MountInfoChildIter::nth got {i}th child of `MountInfoEntry`");

                        self.fwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!("MountInfoChildIter::nth reached the end of `MountInfoChildIter`");

                    return None;
                }
                code => {
                    log::debug!( "MountInfoChildIter::nth failed to get {i:?}th child. libmount::mnt_table_next_child_fs returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next()
    }
}

impl<'table> DoubleEndedIterator for MountInfoChildIter<'table> {
    fn next_back(&mut self) -> Option<Self::Item> {
        log::debug!("MountInfoChildIter::next_back getting next table entry from the back");

        let mut child_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe {
            libmount::mnt_table_next_child_fs(
                self.table.inner,
                self.bwd_iter.inner,
                self.parent.inner,
                child_ptr.as_mut_ptr(),
            )
        };

        match result {
            0 => {
                let ptr = unsafe { child_ptr.assume_init() };

                // Per the documentation of `DoubleEndedIterator`
                // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                if self.have_iterators_met
                    || (self.fwd_cursor != self.bwd_cursor && ptr == self.fwd_cursor)
                {
                    log::debug!(
 "MountInfoChildIter::next_back forward and backward iterators have met in the middle"
                                        );

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!(
                        "MountInfoChildIter::next_back got next child of parent `MountInfoEntry`"
                    );

                    self.bwd_cursor = ptr;
                    let child = owning_ref_from_ptr!(self.table, MountInfoEntry, ptr);

                    Some(child)
                }
            }
            1 => {
                log::debug!(
                    "MountInfoChildIter::next_back reached the beginning of `MountInfoChildIter`"
                );

                None
            }
            code => {
                log::debug!( "MountInfoChildIter::next_back failed to get next child from the back. libmount::mnt_table_next_child_fs returned error code: {code:?}");

                None
            }
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!(
            "MountInfoChildIter::nth_back getting {n}th child of `MountInfoEntry` from the back"
        );

        let mut result;

        // Skips n-1 children, and updates cursor.
        for i in 0..n {
            let mut child_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

            result = unsafe {
                libmount::mnt_table_next_child_fs(
                    self.table.inner,
                    self.bwd_iter.inner,
                    self.parent.inner,
                    child_ptr.as_mut_ptr(),
                )
            };

            match result {
                0 => {
                    let ptr = unsafe { child_ptr.assume_init() };

                    // Per the documentation of `DoubleEndedIterator`
                    // "It is important to note that both back and forth work on the same range, and do not cross: iteration is over when they meet in the middle."
                    if self.have_iterators_met
                        || (self.fwd_cursor != self.bwd_cursor && ptr == self.fwd_cursor)
                    {
                        log::debug!(
 "ChildIter::nth_back forward and backward iterators have met in the middle"
                                            );

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!( "MountInfoChildIter::nth_back got {i}th child of `MountInfoEntry` from the back");

                        self.bwd_cursor = ptr;
                    }
                }
                1 => {
                    log::debug!( "MountInfoChildIter::nth_back reached the beginning of `MountInfoChildIter`");

                    return None;
                }
                code => {
                    log::debug!( "MountInfoChildIter::nth_back failed to get {i:?}th child from the back. libmount::mnt_table_next_child_fs returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next_back()
    }
}
