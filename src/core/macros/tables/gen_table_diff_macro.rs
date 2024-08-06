// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_table_diff {
    ($table_type:ident, $table_entry_type:ident) => {
        paste::paste! {
            // From dependency library

            // From standard library
            use std::mem::MaybeUninit;

            // From this library
            use $crate::core::errors::[<$table_type DiffError>];
            use $crate::tables::GcItem;
            use $crate::tables::$table_type;

            #[doc = concat!("[`", stringify!($table_type), "`] comparator.")]
            #[derive(Debug)]
            pub struct [<$table_type Diff>]<'source, 'other> {
                pub(crate) inner: *mut libmount::libmnt_tabdiff,
                pub(crate) source: &'source $table_type,
                pub(crate) other: &'other $table_type,
                pub(crate) gc: Vec<GcItem>,
            }

            impl<'source, 'other> [<$table_type Diff>]<'source, 'other> {
                #[doc = concat!("Creates a new `", stringify!($table_type), "Diff` to compare the table `other` against the reference `source`.")]
                pub fn new(source: &'source $table_type, other: &'other $table_type) -> Result<[<$table_type Diff>]<'source, 'other>, [<$table_type DiffError>]> {
                    log::debug!("TableDiff::new creating a new `TableDiff` instance");

                    let mut inner = MaybeUninit::<*mut libmount::libmnt_tabdiff>::zeroed();
                    unsafe { inner.write(libmount::mnt_new_tabdiff()) };

                    match unsafe { inner.assume_init() } {
                        inner if inner.is_null() => {
                            let err_msg = concat!("failed to create a new `", stringify!($table_type), "Diff` instance").to_owned();
                            log::debug!(
                                concat!(stringify!($table_type), "Diff::new {}. libmount::mnt_new_tabdiff returned a NULL pointer"),
                                err_msg
                            );

                            Err([<$table_type DiffError>]::Creation(err_msg))
                        }
                        inner => {
                            log::debug!(concat!(stringify!($table_type), "Diff::new created a new `", stringify!($table_type), "Diff` instance"));
                            let table_diff = Self { inner, source, other, gc: vec![]};

                            Ok(table_diff)
                        }
                    }
                }

                #[doc = concat!("Compares two [`", stringify!($table_type), "`]s, entry by entry.")]
                ///
                /// # Examples
                /// ----
                ///
                /// ```
                #[doc = concat!("use rsmount::tables::", stringify!($table_type), ";")]
                #[doc = concat!("use rsmount::tables::", stringify!($table_type), "Diff;")]
                /// # use pretty_assertions::assert_eq;
                ///
                /// fn main() -> rsmount::Result<()> {
                #[doc = concat!("     let empty_table = ", stringify!($table_type), "::new()?;")]
                ///
                #[doc = concat!("     let table_diff = ", stringify!($table_type), "Diff::new(&empty_table, &empty_table)?;")]
                ///
                ///     let nb_changes = table_diff.diff()?;
                ///     assert_eq!(nb_changes, 0);
                ///
                ///     Ok(())
                /// }
                /// ```
                pub fn diff( &self) -> Result<usize, [<$table_type DiffError>]> {
                    log::debug!(concat!(stringify!($table_type), "Diff::diff comparing tables, entry by entry"));

                    let result = unsafe {  libmount::mnt_diff_tables(self.inner, self.source.inner, self.other.inner) };

                    match result {
                        code if code < 0 => {
                            let err_msg = "failed to compare tables, entry by entry".to_owned();
                            log::debug!(
                                concat!(stringify!($table_type), "Diff::diff {}. libmount::mnt_diff_tables returned error code: {:?}"),
                                err_msg,
                                code
                            );

                            Err([<$table_type DiffError>]::Diff(err_msg))
                        }

                        nb_changes => {
                            log::debug!(concat!(stringify!($table_type), "Diff::diff compared tables. Found {:?} changes"), nb_changes);

                            Ok(nb_changes as usize)
                        }
                    }
                }
            }

            impl<'source, 'other> AsRef<[<$table_type Diff>]<'source, 'other>> for [<$table_type Diff>]<'source, 'other> {
                #[inline]
                fn as_ref(&self) -> &[<$table_type Diff>]<'source, 'other> {
                    self
                }
            }

            impl<'source, 'other> Drop for [<$table_type Diff>]<'source, 'other> {
                fn drop(&mut self) {
                    log::debug!(concat!(stringify!($table_type), "Diff::drop deallocating `", stringify!($table_type), "Diff` instance"));

                    unsafe { libmount::mnt_free_tabdiff(self.inner) };

                    // Free entries allocated on the heap for Iterator implementation.
                    while let Some(gc_item) = self.gc.pop() {
                        gc_item.destroy();
                    }
                }
            }
        } //---- END paste
    };
}
