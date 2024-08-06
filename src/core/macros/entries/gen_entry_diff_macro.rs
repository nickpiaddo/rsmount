// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_entry_diff {
    ($table_type:ident, $table_entry_type:ident) => {
        paste::paste! {

            // From dependency library
            use enum_iterator::all;

            // From standard library
            use std::marker::PhantomData;

            // From this library
            use $crate::owning_ref_from_ptr;
            use $crate::core::entries::$table_entry_type;
            use $crate::tables::Comparison;
            use $crate::tables::[<$table_type Diff>];

            #[doc = concat!("Differences between entries in two [`", stringify!($table_type), "`](crate::tables::", stringify!($table_type), ")s.")]
            #[derive(Debug)]
            pub struct [<$table_entry_type Diff>]<'diff, 's, 'o> {
                source: Option<&'diff $table_entry_type>,
                other: Option<&'diff $table_entry_type>,
                comparisons: Vec<Comparison>,
                _marker: PhantomData<&'diff [<$table_type Diff>]<'s, 'o>>,
            }

            impl<'diff, 's, 'o> [<$table_entry_type Diff>]<'diff, 's, 'o> {
                #[doc(hidden)]
                /// Creates a new instance.
                #[allow(dead_code)]
                pub(crate) fn new(
                    table_diff: &'diff [<$table_type Diff>]<'s, 'o>,
                    source_entry_inner: *mut libmount::libmnt_fs,
                    other_entry_inner: *mut libmount::libmnt_fs,
                    flags: i32,
                ) -> [<$table_entry_type Diff>]<'diff, 's, 'o> {
                        let source = if source_entry_inner.is_null() {
                            None
                        } else {
                            let entry = owning_ref_from_ptr!(table_diff, $table_entry_type, source_entry_inner);

                            Some(entry)
                        };

                        let other = if other_entry_inner.is_null() {
                            None
                        } else {
                            let entry = owning_ref_from_ptr!(table_diff, $table_entry_type, other_entry_inner);

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

                #[doc = concat!("Returns the entry used as the reference for the comparison.")]
                pub fn source(&self) -> Option<&'diff $table_entry_type> {
                    self.source
                }

                #[doc = concat!("Returns the entry the reference is compared to.")]
                pub fn other(&self) -> Option<&'diff $table_entry_type> {
                    self.other
                }

                /// Returns a list of the [`Comparison`]s performed.
                pub fn comparisons(&self) -> &[Comparison] {
                    &self.comparisons
                }
            }
        } //---- END paste
    };
}
