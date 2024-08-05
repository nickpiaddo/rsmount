// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_overmount_iter {
    ($table_type:ident, $table_entry_type:ident) => {
        use paste::paste;

        paste! {

            // From dependency library

            // From standard library
            use std::mem::MaybeUninit;

            // From this library
            use $crate::owning_ref_from_ptr;
            use $crate::core::entries::$table_entry_type;
            use $crate::tables::$table_type;

            #[doc = concat!("Iterator over all [`", stringify!($table_type), "`] entries sharing the same mount point.")]
            #[derive(Debug)]
            pub struct [<$table_type OvermountIter>]<'table> {
                table: &'table $table_type,
                parent: &'table $table_entry_type,
            }

            impl<'table> [<$table_type OvermountIter>]<'table> {
                #[doc = concat!("Creates a new `", stringify!($table_type), "OvermountIter`.")]
                #[allow(dead_code)]
                pub(crate) fn new(
                    table: &'table $table_type,
                    parent: &'table $table_entry_type,
                ) -> [<$table_type OvermountIter>]<'table> {

                    let iterator = Self {
                        table,
                        parent,
                    };

                    iterator
                }
            }

            impl<'table> Iterator for [<$table_type OvermountIter>]<'table> {
                type Item = &'table $table_entry_type;

                fn next(&mut self) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "OvermountIter::next getting next over mounted entry"));

                    let mut overmount_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                    let result = unsafe {  libmount::mnt_table_over_fs(
                        self.table.inner,
                        self.parent.inner,
                        overmount_ptr.as_mut_ptr(),
                    )};

                    match result {
                        0 => {
                            let ptr = unsafe { overmount_ptr.assume_init() };
                            log::debug!(concat!(stringify!($table_type), "OvermountIter::next got next over mounted entry"));

                            let overmount = owning_ref_from_ptr!(self.table, $table_entry_type, ptr);

                            Some(overmount)
                        }
                        1 => {
                            log::debug!(concat!(stringify!($table_type), "OvermountIter::next reached end of `", stringify!($table_type), "OvermountIter`"));

                            None
                        }
                        code => {
                            log::debug!(concat!(stringify!($table_type), "OvermountIter::next failed to get next child. libmount::mnt_table_over_fs returned error code: {:?}"), code);

                            None
                        }
                    }
                }

                fn nth(&mut self, n: usize) -> Option<Self::Item> {
                    log::debug!(concat!(stringify!($table_type), "OvermountIter::nth getting {}th child of `", stringify!($table_entry_type), "`"), n);

                    let mut overmount_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
                    let mut result;

                    // Skip n-1 table entries
                    for i in 0..n {
                        result = unsafe {  libmount::mnt_table_over_fs(
                            self.table.inner,
                            self.parent.inner,
                            overmount_ptr.as_mut_ptr(),
                        )};

                        match result {
                            0 => {
                                log::debug!(concat!(stringify!($table_type), "OvermountIter::nth got {:?}th over mounted entry"), i);
                            }
                            1 => {
                                log::debug!(concat!(stringify!($table_type), "OvermountIter::nth reached end of `", stringify!($table_type), "OvermountIter`"));

                                return None;
                            }
                            code => {
                                log::debug!(concat!(stringify!($table_type), "OvermountIter::nth failed to get {:?}th child. libmount::mnt_table_over_fs returned error code: {:?}"), i, code);

                                return None;
                            }
                        }
                    }

                    self.next()
                }
            }
        } //---- END paste
    };
}
