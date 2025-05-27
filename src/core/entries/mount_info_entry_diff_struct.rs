// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use enum_iterator::all;

// From standard library
use std::marker::PhantomData;

// From this library
use crate::core::entries::MountInfoEntry;
use crate::owning_ref_from_ptr;
use crate::tables::Comparison;
use crate::tables::MountInfoDiff;

/// Differences between entries in two [`MountInfo`](crate::tables::MountInfo)s.
#[derive(Debug)]
pub struct MountInfoEntryDiff<'diff, 's, 'o> {
    source: Option<&'diff MountInfoEntry>,
    other: Option<&'diff MountInfoEntry>,
    comparisons: Vec<Comparison>,
    _marker: PhantomData<&'diff MountInfoDiff<'s, 'o>>,
}

impl<'diff, 's, 'o> MountInfoEntryDiff<'diff, 's, 'o> {
    #[doc(hidden)]
    /// Creates a new instance.
    #[allow(dead_code)]
    pub(crate) fn new(
        table_diff: &'diff MountInfoDiff<'s, 'o>,
        source_entry_inner: *mut libmount::libmnt_fs,
        other_entry_inner: *mut libmount::libmnt_fs,
        flags: i32,
    ) -> MountInfoEntryDiff<'diff, 's, 'o> {
        let source = if source_entry_inner.is_null() {
            None
        } else {
            let entry = owning_ref_from_ptr!(table_diff, MountInfoEntry, source_entry_inner);

            Some(entry)
        };

        let other = if other_entry_inner.is_null() {
            None
        } else {
            let entry = owning_ref_from_ptr!(table_diff, MountInfoEntry, other_entry_inner);

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
    pub fn source(&self) -> Option<&'diff MountInfoEntry> {
        self.source
    }

    /// Returns the entry the reference is compared to.
    pub fn other(&self) -> Option<&'diff MountInfoEntry> {
        self.other
    }

    /// Returns a list of the [`Comparison`]s performed.
    pub fn comparisons(&self) -> &[Comparison] {
        &self.comparisons
    }
}
