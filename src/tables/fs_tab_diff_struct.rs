// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::errors::FsTabDiffError;
use crate::core::errors::FsTabDiffIterError;
use crate::core::iter::FsTabDiffIter;
use crate::tables::FsTab;
use crate::tables::GcItem;

/// [`FsTab`] comparator.
#[derive(Debug)]
pub struct FsTabDiff<'source, 'other> {
    pub(crate) inner: *mut libmount::libmnt_tabdiff,
    pub(crate) source: &'source FsTab,
    pub(crate) other: &'other FsTab,
    pub(crate) gc: Vec<GcItem>,
}

impl<'source, 'other> FsTabDiff<'source, 'other> {
    /// Creates a new `FsTabDiff` to compare the table `other` against the reference `source`.
    pub fn new(
        source: &'source FsTab,
        other: &'other FsTab,
    ) -> Result<FsTabDiff<'source, 'other>, FsTabDiffError> {
        log::debug!("TableDiff::new creating a new `TableDiff` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_tabdiff>::zeroed();
        unsafe { inner.write(libmount::mnt_new_tabdiff()) };

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `FsTabDiff` instance".to_owned();
                log::debug!(
                    "FsTabDiff::new {err_msg}. libmount::mnt_new_tabdiff returned a NULL pointer"
                );

                Err(FsTabDiffError::Creation(err_msg))
            }
            inner => {
                log::debug!("FsTabDiff::new created a new `FsTabDiff` instance");
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

    /// Compares two [`FsTab`]s, entry by entry.
    ///
    /// # Examples
    /// ----
    ///
    /// ```
    /// use rsmount::tables::FsTab;
    /// use rsmount::tables::FsTabDiff;
    /// # use pretty_assertions::assert_eq;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///      let empty_table = FsTab::new()?;
    ///
    ///      let table_diff = FsTabDiff::new(&empty_table, &empty_table)?;
    ///
    ///     let nb_changes = table_diff.diff()?;
    ///     assert_eq!(nb_changes, 0);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn diff(&self) -> Result<usize, FsTabDiffError> {
        log::debug!("FsTabDiff::diff comparing tables, entry by entry");

        let result =
            unsafe { libmount::mnt_diff_tables(self.inner, self.source.inner, self.other.inner) };

        match result {
            code if code < 0 => {
                let err_msg = "failed to compare tables, entry by entry".to_owned();
                log::debug!(
                                 "FsTabDiff::diff {err_msg}. libmount::mnt_diff_tables returned error code: {code:?}"
                            );

                Err(FsTabDiffError::Diff(err_msg))
            }

            nb_changes => {
                log::debug!(
                    "FsTabDiff::diff compared tables. Found {:?} changes",
                    nb_changes
                );

                Ok(nb_changes as usize)
            }
        }
    }

    /// Returns an iterator over `FsTabDiff` items comparing two [`FsTab`]s.
    ///
    /// # Panics
    ///
    /// Panics if it fails to create an [`FsTabDiffIter`] iterator.
    pub fn iter(&self) -> FsTabDiffIter {
        log::debug!("FsTabDiff::iter creating a new `FsTabDiffIter` instance");

        FsTabDiffIter::new(self).unwrap()
    }

    /// Tries to instantiate an iterator over `FsTabDiff` items comparing two [`FsTab`]s.
    pub fn try_iter(&self) -> Result<FsTabDiffIter, FsTabDiffIterError> {
        log::debug!("FsTabDiff::try_iter creating a new `FsTabDiffIter` instance");

        FsTabDiffIter::new(self)
    }
}

impl<'source, 'other> AsRef<FsTabDiff<'source, 'other>> for FsTabDiff<'source, 'other> {
    #[inline]
    fn as_ref(&self) -> &FsTabDiff<'source, 'other> {
        self
    }
}

impl<'source, 'other> Drop for FsTabDiff<'source, 'other> {
    fn drop(&mut self) {
        log::debug!("FsTabDiff::drop deallocating `FsTabDiff` instance");

        unsafe { libmount::mnt_free_tabdiff(self.inner) };

        // Free entries allocated on the heap for Iterator implementation.
        while let Some(gc_item) = self.gc.pop() {
            gc_item.destroy();
        }
    }
}
