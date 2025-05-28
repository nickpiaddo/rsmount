// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::errors::SwapsDiffError;
use crate::core::errors::SwapsDiffIterError;
use crate::core::iter::SwapsDiffIter;
use crate::tables::GcItem;
use crate::tables::Swaps;

/// [`Swaps`] comparator.
#[derive(Debug)]
pub struct SwapsDiff<'source, 'other> {
    pub(crate) inner: *mut libmount::libmnt_tabdiff,
    pub(crate) source: &'source Swaps,
    pub(crate) other: &'other Swaps,
    pub(crate) gc: Vec<GcItem>,
}

impl<'source, 'other> SwapsDiff<'source, 'other> {
    /// Creates a new `SwapsDiff` to compare the table `other` against the reference `source`.
    pub fn new(
        source: &'source Swaps,
        other: &'other Swaps,
    ) -> Result<SwapsDiff<'source, 'other>, SwapsDiffError> {
        log::debug!("TableDiff::new creating a new `TableDiff` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_tabdiff>::zeroed();
        unsafe { inner.write(libmount::mnt_new_tabdiff()) };

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `SwapsDiff` instance".to_owned();
                log::debug!(
                    "SwapsDiff::new {err_msg}. libmount::mnt_new_tabdiff returned a NULL pointer"
                );

                Err(SwapsDiffError::Creation(err_msg))
            }
            inner => {
                log::debug!("SwapsDiff::new created a new `SwapsDiff` instance");
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

    /// Compares two [`Swaps`]s, entry by entry.
    ///
    /// # Examples
    /// ----
    ///
    /// ```
    /// use rsmount::tables::Swaps;
    /// use rsmount::tables::SwapsDiff;
    /// # use pretty_assertions::assert_eq;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///      let empty_table = Swaps::new()?;
    ///
    ///      let table_diff = SwapsDiff::new(&empty_table, &empty_table)?;
    ///
    ///     let nb_changes = table_diff.diff()?;
    ///     assert_eq!(nb_changes, 0);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn diff(&self) -> Result<usize, SwapsDiffError> {
        log::debug!("SwapsDiff::diff comparing tables, entry by entry");

        let result =
            unsafe { libmount::mnt_diff_tables(self.inner, self.source.inner, self.other.inner) };

        match result {
            code if code < 0 => {
                let err_msg = "failed to compare tables, entry by entry".to_owned();
                log::debug!(
                    "SwapsDiff::diff {}. libmount::mnt_diff_tables returned error code: {:?}",
                    err_msg,
                    code
                );

                Err(SwapsDiffError::Diff(err_msg))
            }

            nb_changes => {
                log::debug!(
                    "SwapsDiff::diff compared tables. Found {:?} changes",
                    nb_changes
                );

                Ok(nb_changes as usize)
            }
        }
    }

    /// Returns an iterator over `SwapsDiff` items comparing two [`Swaps`]s.
    ///
    /// # Panics
    ///
    /// Panics if it fails to create an [`SwapsDiffIter`] iterator.
    pub fn iter(&self) -> SwapsDiffIter {
        log::debug!("SwapsDiff::iter creating a new `SwapsDiffIter` instance");

        SwapsDiffIter::new(self).unwrap()
    }

    /// Returns an iterator over `SwapsDiff` items comparing two [`Swaps`]s.
    pub fn try_iter(&self) -> Result<SwapsDiffIter, SwapsDiffIterError> {
        log::debug!("SwapsDiff::try_iter creating a new `SwapsDiffIter` instance");

        SwapsDiffIter::new(self)
    }
}

impl<'source, 'other> AsRef<SwapsDiff<'source, 'other>> for SwapsDiff<'source, 'other> {
    #[inline]
    fn as_ref(&self) -> &SwapsDiff<'source, 'other> {
        self
    }
}

impl<'source, 'other> Drop for SwapsDiff<'source, 'other> {
    fn drop(&mut self) {
        log::debug!("SwapsDiff::drop deallocating `SwapsDiff` instance");

        unsafe { libmount::mnt_free_tabdiff(self.inner) };

        // Free entries allocated on the heap for Iterator implementation.
        while let Some(gc_item) = self.gc.pop() {
            gc_item.destroy();
        }
    }
}
