// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::errors::UTabDiffError;
use crate::core::errors::UTabDiffIterError;
use crate::core::iter::UTabDiffIter;
use crate::tables::GcItem;
use crate::tables::UTab;

/// [`UTab`] comparator.
#[derive(Debug)]
pub struct UTabDiff<'source, 'other> {
    pub(crate) inner: *mut libmount::libmnt_tabdiff,
    pub(crate) source: &'source UTab,
    pub(crate) other: &'other UTab,
    pub(crate) gc: Vec<GcItem>,
}

impl<'source, 'other> UTabDiff<'source, 'other> {
    /// Creates a new `UTabDiff` to compare the table `other` against the reference `source`.
    pub fn new(
        source: &'source UTab,
        other: &'other UTab,
    ) -> Result<UTabDiff<'source, 'other>, UTabDiffError> {
        log::debug!("TableDiff::new creating a new `TableDiff` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_tabdiff>::zeroed();
        unsafe { inner.write(libmount::mnt_new_tabdiff()) };

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `UTabDiff` instance".to_owned();
                log::debug!(
                    "UTabDiff::new {err_msg}. libmount::mnt_new_tabdiff returned a NULL pointer"
                );

                Err(UTabDiffError::Creation(err_msg))
            }
            inner => {
                log::debug!("UTabDiff::new created a new `UTabDiff` instance");
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

    /// Compares two [`UTab`]s, entry by entry.
    ///
    /// # Examples
    /// ----
    ///
    /// ```
    /// use rsmount::tables::UTab;
    /// use rsmount::tables::UTabDiff;
    /// # use pretty_assertions::assert_eq;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///      let empty_table = UTab::new()?;
    ///
    ///      let table_diff = UTabDiff::new(&empty_table, &empty_table)?;
    ///
    ///     let nb_changes = table_diff.diff()?;
    ///     assert_eq!(nb_changes, 0);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn diff(&self) -> Result<usize, UTabDiffError> {
        log::debug!("UTabDiff::diff comparing tables, entry by entry");

        let result =
            unsafe { libmount::mnt_diff_tables(self.inner, self.source.inner, self.other.inner) };
        match result {
            code if code < 0 => {
                let err_msg = "failed to compare tables, entry by entry".to_owned();
                log::debug!(
 "UTabDiff::diff {err_msg}. libmount::mnt_diff_tables returned error code: {code:?}"
                            );

                Err(UTabDiffError::Diff(err_msg))
            }

            nb_changes => {
                log::debug!(
                    "UTabDiff::diff compared tables. Found {:?} changes",
                    nb_changes
                );

                Ok(nb_changes as usize)
            }
        }
    }

    /// Returns an iterator over `UTabDiff` items comparing two [`UTab`]s.
    ///
    /// # Panics
    ///
    /// Panics if it fails to create an [`UTabDiffIter`] iterator.
    pub fn iter(&self) -> UTabDiffIter {
        log::debug!("UTabDiff::iter creating a new `UTabDiffIter` instance");

        UTabDiffIter::new(self).unwrap()
    }

    /// Tries to instantiate an iterator over `UTabDiff` items comparing two [`UTab`]s.
    pub fn try_iter(&self) -> Result<UTabDiffIter, UTabDiffIterError> {
        log::debug!("UTabDiff::try_iter creating a new `UTabDiffIter` instance");

        UTabDiffIter::new(self)
    }
}

impl<'source, 'other> AsRef<UTabDiff<'source, 'other>> for UTabDiff<'source, 'other> {
    #[inline]
    fn as_ref(&self) -> &UTabDiff<'source, 'other> {
        self
    }
}

impl<'source, 'other> Drop for UTabDiff<'source, 'other> {
    fn drop(&mut self) {
        log::debug!("UTabDiff::drop deallocating `UTabDiff` instance");

        unsafe { libmount::mnt_free_tabdiff(self.inner) };

        // Free entries allocated on the heap for Iterator implementation.
        while let Some(gc_item) = self.gc.pop() {
            gc_item.destroy();
        }
    }
}
