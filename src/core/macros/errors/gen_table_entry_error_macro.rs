// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library
#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_table_entry_error {
    ($table_entry_type:ident) => {
        paste::paste! {
            // From dependency library
            use thiserror::Error;

            // From standard library
            use std::ffi::NulError;

            // From this library

            /// [`FsTabEntry`](crate::core::entries::FsTabEntry) runtime errors.
            #[derive(Debug, Error)]
            #[non_exhaustive]
            pub enum [<$table_entry_type Error>] {
                #[doc = concat!("Error while performing an action on a [`", stringify!($table_entry_type), "`](crate::core::entries::", stringify!($table_entry_type), ") instance.")]
                #[error("{0}")]
                Action(String),

                #[doc = concat!("Error while creating a new [`", stringify!($table_entry_type), "`](crate::core::entries::", stringify!($table_entry_type), ") instance.")]
                #[error("{0}")]
                Creation(String),

                #[doc = concat!("Error while configuring a new [`", stringify!($table_entry_type), "`](crate::core::entries::", stringify!($table_entry_type), ") instance.")]
                #[error("{0}")]
                Config(String),

                #[doc = concat!("Error while copying data between [`", stringify!($table_entry_type), "`](crate::core::entries::", stringify!($table_entry_type), ") instances.")]
                #[error("{0}")]
                Copy(String),

                /// Error while converting a value to [`CString`](std::ffi::CString).
                #[error("failed to convert value to `CString`: {0}")]
                CStringConversion(#[from] NulError),

                #[error(transparent)]
                IoError(#[from] std::io::Error),

                /// Error when accessing a file without having the proper permissions.
                #[error("{0}")]
                Permission(String),
            }
        } //---- END paste

    };
}
