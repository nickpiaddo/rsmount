// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library
#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_table_iter_error {
    ($table_type:ident) => {
        paste::paste! {
            // From dependency library
            use thiserror::Error;

            // From standard library

            // From this library
            use $crate::core::errors::GenIteratorError;

            #[doc = concat!("[`", stringify!($table_type), "Iter`](crate::core::iter::", stringify!($table_type), "Iter) runtime errors.")]
            #[derive(Debug, Error)]
            #[non_exhaustive]
            pub enum [<$table_type IterError>] {
                #[error(transparent)]
                GenIterator(#[from] GenIteratorError),
            }
        } //---- END paste

    };
}
