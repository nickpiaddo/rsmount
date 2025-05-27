// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use enum_iterator::all;

// From standard library
use std::marker::PhantomData;

// From this library
use crate::core::entries::FsTabEntry;
use crate::owning_ref_from_ptr;
use crate::tables::Comparison;
use crate::tables::FsTabDiff;

/// Differences between entries in two [`FsTab`](crate::tables::FsTab)s.
#[derive(Debug)]
pub struct FsTabEntryDiff<'diff, 's, 'o> {
    source: Option<&'diff FsTabEntry>,
    other: Option<&'diff FsTabEntry>,
    comparisons: Vec<Comparison>,
    _marker: PhantomData<&'diff FsTabDiff<'s, 'o>>,
}

impl<'diff, 's, 'o> FsTabEntryDiff<'diff, 's, 'o> {
    #[doc(hidden)]
    /// Creates a new instance.
    #[allow(dead_code)]
    pub(crate) fn new(
        table_diff: &'diff FsTabDiff<'s, 'o>,
        source_entry_inner: *mut libmount::libmnt_fs,
        other_entry_inner: *mut libmount::libmnt_fs,
        flags: i32,
    ) -> FsTabEntryDiff<'diff, 's, 'o> {
        let source = if source_entry_inner.is_null() {
            None
        } else {
            let entry = owning_ref_from_ptr!(table_diff, FsTabEntry, source_entry_inner);

            Some(entry)
        };

        let other = if other_entry_inner.is_null() {
            None
        } else {
            let entry = owning_ref_from_ptr!(table_diff, FsTabEntry, other_entry_inner);

            Some(entry)
        };

        let comparisons: Vec<_> = all::<Comparison>()
            .filter(|&op| flags & (op as i32) != 0)
            .collect();

        Self {
            source,
            other,
            comparisons,
            _marker: PhantomData,
        }
    }

    /// Returns the entry used as the reference for the comparison.
    pub fn source(&self) -> Option<&'diff FsTabEntry> {
        self.source
    }

    /// Returns the entry the reference is compared to.
    pub fn other(&self) -> Option<&'diff FsTabEntry> {
        self.other
    }

    /// Returns a list of the [`Comparison`]s performed.
    pub fn comparisons(&self) -> &[Comparison] {
        &self.comparisons
    }
}
