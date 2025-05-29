// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::entries::MountInfoEntry;
use crate::core::iter::Direction;
use crate::core::iter::GenIterator;
use crate::mount::StepResult;
use crate::mount::UMountIterError;
use crate::mount::Unmount;

/// Iterator to sequentially unmount entries in `/etc/fstab`, or `/proc/self/mountinfo`.
#[derive(Debug)]
pub struct UMountIter<'mount> {
    mount: &'mount mut Unmount,
    iterator: GenIterator,
}

impl<'mount> UMountIter<'mount> {
    #[doc(hidden)]
    /// Creates a new `UMountIter`.
    #[allow(dead_code)]
    pub(crate) fn new(mount: &'mount mut Unmount) -> Result<UMountIter<'mount>, UMountIterError> {
        GenIterator::new(Direction::Forward)
            .map(|iterator| UMountIter { mount, iterator })
            .map_err(UMountIterError::from)
    }
}

impl<'mount> Iterator for UMountIter<'mount> {
    type Item = StepResult;

    /// Tries to unmount an entry in `/etc/fstab`, or `/proc/self/mountinfo`.
    ///
    /// Returns the function's status after execution as a [`StepResult`], or
    /// `None` if there is no more entry to process or an error occurred.
    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("UMountIter::next trying to unmount next `MountInfoEntry`");

        let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        let mut mount_return_code = MaybeUninit::<libc::c_int>::zeroed();
        let mut ignored = MaybeUninit::<libc::c_int>::zeroed();

        let result = unsafe {
            libmount::mnt_context_next_umount(
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
                    (0, 0) => Some(StepResult::UMountSuccess(entry)),
                    (_, 1) => Some(StepResult::UMountSkipped(entry)),
                    (_, 2) => Some(StepResult::UMountAlreadyDone(entry)),
                    (_, _) => Some(StepResult::UMountFail(entry)),
                }
            }
            1 => {
                log::debug!("UMountIter::next reached the end of `UMountIter`");

                None
            }
            code => {
                log::debug!(
                    "UMountIter::next failed tounmount next entry. mnt_context_next_umount returned error code: {code:?}"
                );

                None
            }
        }
    }
}
