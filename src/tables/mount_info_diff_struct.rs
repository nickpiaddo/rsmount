// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::errors::MountInfoDiffError;
use crate::core::errors::MountInfoDiffIterError;
use crate::core::iter::MountInfoDiffIter;
use crate::tables::GcItem;
use crate::tables::MountInfo;

/// [`MountInfo`] comparator.
#[derive(Debug)]
pub struct MountInfoDiff<'source, 'other> {
    pub(crate) inner: *mut libmount::libmnt_tabdiff,
    pub(crate) source: &'source MountInfo,
    pub(crate) other: &'other MountInfo,
    pub(crate) gc: Vec<GcItem>,
}

impl<'source, 'other> MountInfoDiff<'source, 'other> {
    /// Creates a new `MountInfoDiff` to compare the table `other` against the reference `source`.
    pub fn new(
        source: &'source MountInfo,
        other: &'other MountInfo,
    ) -> Result<MountInfoDiff<'source, 'other>, MountInfoDiffError> {
        log::debug!("TableDiff::new creating a new `TableDiff` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_tabdiff>::zeroed();
        unsafe { inner.write(libmount::mnt_new_tabdiff()) };

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `MountInfoDiff` instance".to_owned();
                log::debug!(
                    "MountInfoDiff::new {}. libmount::mnt_new_tabdiff returned a NULL pointer",
                    err_msg
                );

                Err(MountInfoDiffError::Creation(err_msg))
            }
            inner => {
                log::debug!("MountInfo::Diff::new created a new `MountInfoDiff` instance");
                let table_diff = Self {
                    inner,
                    source,
                    other,
                    gc: vec![],
                };

                Ok(table_diff)
            }
        }
    }

    /// Compares two [`MountInfo`]s, entry by entry.
    ///
    /// # Examples
    /// ----
    ///
    /// ```
    /// use rsmount::tables::MountInfo;
    /// use rsmount::tables::MountInfoDiff;
    /// # use pretty_assertions::assert_eq;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///      let empty_table = MountInfo::new()?;
    ///
    ///      let table_diff = MountInfoDiff::new(&empty_table, &empty_table)?;
    ///
    ///     let nb_changes = table_diff.diff()?;
    ///     assert_eq!(nb_changes, 0);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn diff(&self) -> Result<usize, MountInfoDiffError> {
        log::debug!("MountInfoDiff::diff comparing tables, entry by entry");

        let result =
            unsafe { libmount::mnt_diff_tables(self.inner, self.source.inner, self.other.inner) };

        match result {
            code if code < 0 => {
                let err_msg = "failed to compare tables, entry by entry".to_owned();
                log::debug!(
                    "MountInfoDiff::diff {}. libmount::mnt_diff_tables returned error code: {:?}",
                    err_msg,
                    code
                );

                Err(MountInfoDiffError::Diff(err_msg))
            }

            nb_changes => {
                log::debug!(
                    "MountInfoDiff::diff compared tables. Found {:?} changes",
                    nb_changes
                );

                Ok(nb_changes as usize)
            }
        }
    }

    /// Returns an iterator over `MountInfoDiff` items comparing two [`MountInfo`]s.
    ///
    /// # Panics
    ///
    /// Panics if it fails to create an [`MountInfoDiffIter`] iterator.
    pub fn iter(&self) -> MountInfoDiffIter {
        log::debug!("MountInfoDiff::iter creating a new `MountInfoDiffIter` instance");

        MountInfoDiffIter::new(self).unwrap()
    }

    /// Tries to instantiate an iterator over `MountInfoDiff` items comparing two [`MountInfo`]s.
    pub fn try_iter(&self) -> Result<MountInfoDiffIter, MountInfoDiffIterError> {
        log::debug!("MountInfoDiff::try_iter creating a new `MountInfoDiffIter` instance");

        MountInfoDiffIter::new(self)
    }
}

impl<'source, 'other> AsRef<MountInfoDiff<'source, 'other>> for MountInfoDiff<'source, 'other> {
    #[inline]
    fn as_ref(&self) -> &MountInfoDiff<'source, 'other> {
        self
    }
}

impl<'source, 'other> Drop for MountInfoDiff<'source, 'other> {
    fn drop(&mut self) {
        log::debug!("MountInfoDiff::drop deallocating `MountInfoDiff` instance");

        unsafe { libmount::mnt_free_tabdiff(self.inner) };

        // Free entries allocated on the heap for Iterator implementation.
        while let Some(gc_item) = self.gc.pop() {
            gc_item.destroy();
        }
    }
}
