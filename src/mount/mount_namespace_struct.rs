// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::marker::PhantomData;

// From this library
use crate::mount::Mount;

/// A mount namespace.
#[derive(Debug)]
#[repr(transparent)]
pub struct MountNamespace<'mount> {
    #[allow(dead_code)]
    pub(crate) ptr: *mut libmount::libmnt_ns,
    _marker: PhantomData<&'mount Mount>,
}

impl<'mount> MountNamespace<'mount> {
    #[doc(hidden)]
    /// Wraps a raw `libmount::mnt_ns` pointer with a safe `MountNamespace`.
    #[allow(dead_code)]
    pub(crate) fn from_raw_parts(
        ptr: *mut libmount::libmnt_ns,
        _: &Mount,
    ) -> MountNamespace<'mount> {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }
}
