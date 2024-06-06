// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum GcItem {
    Cache(*mut *mut libmount::libmnt_cache),
    TabEntry(*mut *mut libmount::libmnt_fs),
    Lock(*mut *mut libmount::libmnt_lock),
}

impl GcItem {
    #[doc(hidden)]
    #[allow(dead_code)]
    /// Consumes the `GcItem` and frees the memory it points to.
    pub(crate) fn destroy(self) {
        match self {
            GcItem::Cache(boxed_ptr) => {
                let _ = unsafe { Box::from_raw(boxed_ptr) };
            }
            GcItem::TabEntry(boxed_ptr) => {
                let _ = unsafe { Box::from_raw(boxed_ptr) };
            }
            GcItem::Lock(boxed_ptr) => {
                let _ = unsafe { Box::from_raw(boxed_ptr) };
            }
        }
    }
}

impl From<*mut *mut libmount::libmnt_cache> for GcItem {
    fn from(ptr: *mut *mut libmount::libmnt_cache) -> GcItem {
        GcItem::Cache(ptr)
    }
}

impl From<*mut *mut libmount::libmnt_fs> for GcItem {
    fn from(ptr: *mut *mut libmount::libmnt_fs) -> GcItem {
        GcItem::TabEntry(ptr)
    }
}

impl From<*mut *mut libmount::libmnt_lock> for GcItem {
    fn from(ptr: *mut *mut libmount::libmnt_lock) -> GcItem {
        GcItem::Lock(ptr)
    }
}
