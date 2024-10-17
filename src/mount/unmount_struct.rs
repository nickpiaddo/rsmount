// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

/// Object to unmount a device.
#[derive(Debug)]
#[repr(transparent)]
pub struct Unmount {
    pub(crate) inner: *mut libmount::libmnt_context,
}

impl AsRef<Unmount> for Unmount {
    fn as_ref(&self) -> &Unmount {
        self
    }
}

impl Drop for Unmount {
    fn drop(&mut self) {
        log::debug!("Unmount::drop deallocating `Unmount` instance");

        unsafe { libmount::mnt_free_context(self.inner) }
    }
}
