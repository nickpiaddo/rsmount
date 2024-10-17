// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::marker::PhantomData;

// From this library
use crate::mount::Unmount;

/// A mount namespace.
#[derive(Debug)]
#[repr(transparent)]
pub struct UMountNamespace<'mount> {
    #[allow(dead_code)]
    pub(crate) ptr: *mut libmount::libmnt_ns,
    _marker: PhantomData<&'mount Unmount>,
}

impl<'mount> UMountNamespace<'mount> {
    #[doc(hidden)]
    /// Wraps a raw `libmount::mnt_ns` pointer with a safe `UMountNamespace`.
    #[allow(dead_code)]
    pub(crate) fn from_raw_parts(
        ptr: *mut libmount::libmnt_ns,
        _: &Unmount,
    ) -> UMountNamespace<'mount> {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }
}
