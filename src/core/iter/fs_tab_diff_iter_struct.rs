// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::entries::FsTabEntryDiff;
use crate::core::errors::FsTabDiffIterError;
use crate::core::iter::Direction;
use crate::core::iter::GenIterator;
use crate::tables::FsTabDiff;

/// Iterate over [`FsTabDiff`] entries.
#[derive(Debug)]
pub struct FsTabDiffIter<'diff, 's, 'o> {
    table_diff: &'diff FsTabDiff<'s, 'o>,
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

impl<'diff, 's, 'o> FsTabDiffIter<'diff, 's, 'o> {
    #[doc(hidden)]
    /// Creates a new instance.
    #[allow(dead_code)]
    pub(crate) fn new(
        table_diff: &'diff FsTabDiff<'s, 'o>,
    ) -> Result<FsTabDiffIter<'diff, 's, 'o>, FsTabDiffIterError> {
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

impl<'diff, 's, 'o> Iterator for FsTabDiffIter<'diff, 's, 'o> {
    type Item = FsTabEntryDiff<'diff, 's, 'o>;

    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("FsTabDiffIter::next getting next table changes");

        let mut source_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        let mut other_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        let mut flags = MaybeUninit::<libc::c_int>::zeroed();

        let result = unsafe {
            libmount::mnt_tabdiff_next_change(
                self.table_diff.inner,
                self.fwd_iter.inner,
                source_entry_inner.as_mut_ptr(),
                other_entry_inner.as_mut_ptr(),
                flags.as_mut_ptr(),
            )
        };

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
                    log::debug!(
                        "FsTabDiffIter::next forward and backward iterators have met in the middle"
                    );

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!("FsTabDiffIter::next got next table changes");
                    self.first_fwd_cursor = source_entry_inner;
                    self.second_fwd_cursor = other_entry_inner;
                    let diff_entry = FsTabEntryDiff::new(
                        self.table_diff,
                        source_entry_inner,
                        other_entry_inner,
                        flags,
                    );

                    Some(diff_entry)
                }
            }
            1 => {
                log::debug!("FsTabDiffIter::next reached the end of `FsTabDiffIter`");

                None
            }
            code => {
                let err_msg = "failed to get next changes".to_owned();
                log::debug!( "FsTabDiffIter::next {err_msg}. libmount::mnt_tabdiff_next_change returned error code: {code:?}");

                None
            }
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!("FsTabDiffIter::nth getting {n:?}th table diff item");

        let mut result;
        let mut flags = MaybeUninit::<libc::c_int>::zeroed();

        // Skip n-1 entries, and update cursors along the way.
        for i in 0..n {
            let mut source_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
            let mut other_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

            result = unsafe {
                libmount::mnt_tabdiff_next_change(
                    self.table_diff.inner,
                    self.fwd_iter.inner,
                    source_entry_inner.as_mut_ptr(),
                    other_entry_inner.as_mut_ptr(),
                    flags.as_mut_ptr(),
                )
            };

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
                        log::debug!( "FsTabDiffIter::next forward and backward iterators have met in the middle");

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!("FsTabDiffIter::nth got {i:?}th table diff item");
                        self.first_fwd_cursor = source_entry_inner;
                        self.second_fwd_cursor = other_entry_inner;
                    }
                }
                1 => {
                    log::debug!("FsTabDiffIter::nth reached the end of `FsTabDiffIter`");

                    return None;
                }
                code => {
                    let err_msg = format!("failed to get {:?}th table diff item", n);
                    log::debug!( "FsTabDiffIter::nth {err_msg}. libmount::mnt_tabdiff_next_change returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next()
    }
}

impl<'diff, 's, 'o> DoubleEndedIterator for FsTabDiffIter<'diff, 's, 'o> {
    fn next_back(&mut self) -> Option<Self::Item> {
        log::debug!("FsTabDiffIter::next_back getting next table diff item from the back");

        let mut source_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        let mut other_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        let mut flags = MaybeUninit::<libc::c_int>::zeroed();

        let result = unsafe {
            libmount::mnt_tabdiff_next_change(
                self.table_diff.inner,
                self.bwd_iter.inner,
                source_entry_inner.as_mut_ptr(),
                other_entry_inner.as_mut_ptr(),
                flags.as_mut_ptr(),
            )
        };

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
                    log::debug!( "FsTabDiffIter::next_back forward and backward iterators have met in the middle");

                    self.have_iterators_met = true;

                    None
                } else {
                    log::debug!(
                        "FsTabDiffIter::next_back got next table changes iterating backward"
                    );

                    self.first_bwd_cursor = source_entry_inner;
                    self.second_bwd_cursor = other_entry_inner;
                    let diff_entry = FsTabEntryDiff::new(
                        self.table_diff,
                        source_entry_inner,
                        other_entry_inner,
                        flags,
                    );

                    Some(diff_entry)
                }
            }
            1 => {
                log::debug!("FsTabDiffIter::next_back reached the end of `FsTabDiffIter`");

                None
            }
            code => {
                let err_msg = "failed to get next changes iterating backward".to_owned();
                log::debug!( "FsTabDiffIter::next_back {err_msg}. libmount::mnt_tabdiff_next_change returned error code: {code:?}");

                None
            }
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!("FsTabDiffIter::nth_back getting {n:?}th table diff item from the back");

        let mut result;
        let mut flags = MaybeUninit::<libc::c_int>::zeroed();

        // Skip n-1 entries, and update cursors along the way.
        for i in 0..n {
            let mut source_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
            let mut other_entry_inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

            result = unsafe {
                libmount::mnt_tabdiff_next_change(
                    self.table_diff.inner,
                    self.bwd_iter.inner,
                    source_entry_inner.as_mut_ptr(),
                    other_entry_inner.as_mut_ptr(),
                    flags.as_mut_ptr(),
                )
            };

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
                        log::debug!( "FsTabDiffIter::nth_back forward and backward iterators have met in the middle");

                        self.have_iterators_met = true;

                        return None;
                    } else {
                        log::debug!("FsTabDiffIter::nth got {i:?}th table diff item from the back");
                        self.first_bwd_cursor = source_entry_inner;
                        self.second_bwd_cursor = other_entry_inner;
                    }
                }
                1 => {
                    log::debug!("FsTabDiffIter::nth_back reached the beginning of `FsTabDiffIter`");

                    return None;
                }
                code => {
                    let err_msg = format!("failed to get {:?}th table diff item from the back", n);
                    log::debug!( "FsTabDiffIter::nth_back {err_msg}. libmount::mnt_tabdiff_next_change returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next_back()
    }
}
