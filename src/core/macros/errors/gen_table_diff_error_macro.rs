// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_table_diff_error {
    ($table_type:ident) => {
        paste::paste! {

            // From dependency library
            use thiserror::Error;

            // From standard library

            // From this library

            #[doc = concat!("[`", stringify!($table_type), "Diff`](crate::tables::", stringify!($table_type), "Diff) runtime errors.")]
            #[derive(Debug, Error)]
            #[non_exhaustive]
            pub enum [<$table_type DiffError>] {
                #[doc = concat!("Error while creating a new [`", stringify!($table_type), "Diff`](crate::tables::", stringify!($table_type), "Diff) instance.")]
                #[error("{0}")]
                Creation(String),

                #[doc = concat!("Error while comparing [`", stringify!($table_type), "Diff`](crate::tables::", stringify!($table_type), "Diff) instances.")]
                #[error("{0}")]
                Diff(String),
            }
        } //---- END paste
    };
}
