// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::entries::MountInfoEntry;
use crate::owning_ref_from_ptr;
use crate::tables::MountInfo;

/// Iterator over all [`MountInfo`] entries sharing the same mount point.
#[derive(Debug)]
pub struct MountInfoOvermountIter<'table> {
    table: &'table MountInfo,
    parent: &'table MountInfoEntry,
}

impl<'table> MountInfoOvermountIter<'table> {
    /// Creates a new `MountInfoOvermountIter`.
    #[allow(dead_code)]
    pub(crate) fn new(
        table: &'table MountInfo,
        parent: &'table MountInfoEntry,
    ) -> MountInfoOvermountIter<'table> {
        Self { table, parent }
    }
}

impl<'table> Iterator for MountInfoOvermountIter<'table> {
    type Item = &'table MountInfoEntry;

    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("MountInfoOvermountIter::next getting next over mounted entry");

        let mut overmount_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe {
            libmount::mnt_table_over_fs(
                self.table.inner,
                self.parent.inner,
                overmount_ptr.as_mut_ptr(),
            )
        };

        match result {
            0 => {
                let ptr = unsafe { overmount_ptr.assume_init() };
                log::debug!("MountInfoOvermountIter::next got next over mounted entry");

                let overmount = owning_ref_from_ptr!(self.table, MountInfoEntry, ptr);

                Some(overmount)
            }
            1 => {
                log::debug!("MountInfoOvermountIter::next reached end of `MountInfoOvermountIter`");

                None
            }
            code => {
                log::debug!( "MountInfoOvermountIter::next failed to get next child. libmount::mnt_table_over_fs returned error code: {code:?}");

                None
            }
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        log::debug!("MountInfoOvermountIter::nth getting {n}th child of `MountInfoEntry`");

        let mut overmount_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        let mut result;

        // Skip n-1 table entries
        for i in 0..n {
            result = unsafe {
                libmount::mnt_table_over_fs(
                    self.table.inner,
                    self.parent.inner,
                    overmount_ptr.as_mut_ptr(),
                )
            };

            match result {
                0 => {
                    log::debug!("MountInfoOvermountIter::nth got {i:?}th over mounted entry");
                }
                1 => {
                    log::debug!(
                        "MountInfoOvermountIter::nth reached end of `MountInfoOvermountIter`"
                    );

                    return None;
                }
                code => {
                    log::debug!( "MountInfoOvermountIter::nth failed to get {i:?}th child. libmount::mnt_table_over_fs returned error code: {code:?}");

                    return None;
                }
            }
        }

        self.next()
    }
}
