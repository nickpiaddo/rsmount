// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::entries::FsTabEntry;
use crate::core::iter::Direction;
use crate::core::iter::GenIterator;
use crate::mount::Mount;
use crate::mount::MountIterError;
use crate::mount::StepResult;

/// Iterator to sequentially mount entries in `/etc/fstab`, or `/proc/self/mountinfo`.
#[derive(Debug)]
pub struct MountIter<'mount> {
    mount: &'mount mut Mount,
    iterator: GenIterator,
}

impl<'mount> MountIter<'mount> {
    #[doc(hidden)]
    /// Creates a new `MountIter`.
    #[allow(dead_code)]
    pub(crate) fn new(mount: &'mount mut Mount) -> Result<MountIter<'mount>, MountIterError> {
        GenIterator::new(Direction::Forward)
            .map(|iterator| MountIter { mount, iterator })
            .map_err(MountIterError::from)
    }
}

impl<'mount> Iterator for MountIter<'mount> {
    type Item = StepResult;

    /// Tries to mount an entry in `/etc/fstab`, or `/proc/self/mountinfo`.
    ///
    /// Returns the function's status after execution as a [`StepResult`], or
    /// `None` if there is no more entry to process or an error occurred.
    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("MountIter::next trying to mount next `FsTabEntry`");

        let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        let mut mount_return_code = MaybeUninit::<libc::c_int>::zeroed();
        let mut ignored = MaybeUninit::<libc::c_int>::zeroed();

        let result = unsafe {
            libmount::mnt_context_next_mount(
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
                let entry = <FsTabEntry>::from_ptr(ptr);

                match (rc, skipped) {
                    (0, 0) => Some(StepResult::MountSuccess(entry)),
                    (_, 1) => Some(StepResult::MountSkipped(entry)),
                    (_, 2) => Some(StepResult::MountAlreadyDone(entry)),
                    (_, _) => Some(StepResult::MountFail(entry)),
                }
            }
            1 => {
                log::debug!("MountIter::next reached the end of `MountIter`");

                None
            }
            code => {
                log::debug!( "MountIter::next failed to mount next entry. mnt_context_next_mount returned error code: {code:?}");

                None
            }
        }
    }
}
