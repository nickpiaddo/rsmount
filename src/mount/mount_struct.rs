// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

/// Object to mount/unmount a device.
#[derive(Debug)]
#[repr(transparent)]
pub struct Mount {
    pub(crate) inner: *mut libmount::libmnt_context,
}

impl AsRef<Mount> for Mount {
    fn as_ref(&self) -> &Mount {
        self
    }
}

impl Drop for Mount {
    fn drop(&mut self) {
        log::debug!("Mount::drop deallocating `Mount` instance");

        unsafe { libmount::mnt_free_context(self.inner) }
    }
}
