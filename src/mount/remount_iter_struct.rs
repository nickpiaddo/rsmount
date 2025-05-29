// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::entries::MountInfoEntry;
use crate::core::iter::{Direction, GenIterator};
use crate::mount::Mount;
use crate::mount::ReMountIterError;
use crate::mount::StepResult;

/// Iterator to sequentially remount entries in `/etc/fstab`, or `/proc/self/mountinfo`.
#[derive(Debug)]
pub struct ReMountIter<'mount> {
    mount: &'mount mut Mount,
    iterator: GenIterator,
}

impl<'mount> ReMountIter<'mount> {
    #[doc(hidden)]
    /// Creates a new `ReMountIter`.
    #[allow(dead_code)]
    pub(crate) fn new(mount: &'mount mut Mount) -> Result<ReMountIter<'mount>, ReMountIterError> {
        GenIterator::new(Direction::Forward)
            .map(|iterator| ReMountIter { mount, iterator })
            .map_err(ReMountIterError::from)
    }
}

impl<'mount> Iterator for ReMountIter<'mount> {
    type Item = StepResult;

    /// Tries to remount an entry in `/etc/fstab`, or `/proc/self/mountinfo`.
    ///
    /// Returns the function's status after execution as a [`StepResult`], or
    /// `None` if there is no more entry to process or an error occurred.
    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("ReMountIter::next trying to remount next `MountInfoEntry`");

        let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        let mut mount_return_code = MaybeUninit::<libc::c_int>::zeroed();
        let mut ignored = MaybeUninit::<libc::c_int>::zeroed();

        let result = unsafe {
            libmount::mnt_context_next_remount(
                self.mount.inner,
                self.iterator.inner,
                entry_ptr.as_mut_ptr(),
                mount_return_code.as_mut_ptr(),
                ignored.as_mut_ptr(),
            )
        };

        match result {
            0 => {
                let ptr = unsafe { entry_ptr.assume_init() };
                let rc = unsafe { mount_return_code.assume_init() };
                let skipped = unsafe { ignored.assume_init() };
                let entry = <MountInfoEntry>::from_ptr(ptr);

                match (rc, skipped) {
                    (0, 0) => Some(StepResult::ReMountSuccess(entry)),
                    (_, 1) => Some(StepResult::ReMountSkipped(entry)),
                    (_, 2) => Some(StepResult::ReMountAlreadyDone(entry)),
                    (_, _) => Some(StepResult::ReMountFail(entry)),
                }
            }
            1 => {
                log::debug!("ReMountIter::next reached the end of `ReMountIter`");

                None
            }
            code => {
                log::debug!( "ReMountIter::next failed toremount next entry. mnt_context_next_remount returned error code: {code:?}");

                None
            }
        }
    }
}
