// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::str::FromStr;

// From this library
use crate::core::device::Tag;
use crate::core::errors::FsTabEntryError;
use crate::declare_tab_entry;
use crate::fs_tab_entry_shared_methods;

declare_tab_entry!(FsTabEntry, "A configuration line in `/etc/fstab`.");

fs_tab_entry_shared_methods!(FsTabEntry, FsTabEntryError);

impl FsTabEntry {
    //---- BEGIN mutators

    /// Appends a comment line to the mount entry.
    pub fn append_comment<T>(&mut self, comment: T) -> Result<(), FsTabEntryError>
    where
        T: AsRef<str>,
    {
        let comment = comment.as_ref();
        log::debug!("FsTabEntry::append_comment appending comment {:?}", comment);

        let attrs = ffi_utils::as_ref_str_to_c_string(comment)?;

        let result = unsafe { libmount::mnt_fs_append_comment(self.inner, attrs.as_ptr()) };

        match result {
            0 => {
                log::debug!("FsTabEntry::append_comment appended comment {:?}", comment);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to append comment {:?}", comment);
                log::debug!("FsTabEntry::append_comment {}. libmount::mnt_fs_append_comment returned error code: {:?}", err_msg, code);

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    //---- END mutators

    //---- BEGIN getters

    /// Returns the initial and additional comment lines appended to this instance.
    pub fn comment(&self) -> Option<&str> {
        log::debug!("FsTabEntry::comment getting comment line ");

        let mut comment_ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            comment_ptr.write(libmount::mnt_fs_get_comment(self.inner));
        }

        match unsafe { comment_ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("FsTabEntry::comment no comment line set. libmount::mnt_fs_get_comment returned a NULL pointer");

                None
            }
            comment_ptr => {
                let comment = ffi_utils::const_char_array_to_str_ref(comment_ptr);
                log::debug!("FsTabEntry::comment value: {:?}", comment);

                comment.ok()
            }
        }
    }

    /// Returns the [`Tag`] identifying the `FsTabEntry`'s mount source.
    pub fn tag(&self) -> Option<Tag> {
        log::debug!("FsTabEntry::tag getting tag");

        let mut name_ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        let mut value_ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        let result = unsafe {
            libmount::mnt_fs_get_tag(self.inner, name_ptr.as_mut_ptr(), value_ptr.as_mut_ptr())
        };

        match result {
            0 => match unsafe { (name_ptr.assume_init(), value_ptr.assume_init()) } {
                (name_ptr, value_ptr) if name_ptr.is_null() || value_ptr.is_null() => {
                    log::debug!("FsTabEntry::tag found a tag with an empty name or value");

                    None
                }
                (name_ptr, value_ptr) => {
                    let name = ffi_utils::const_char_array_to_str_ref(name_ptr).ok()?;
                    let value = ffi_utils::const_char_array_to_str_ref(value_ptr).ok()?;

                    let tag_str = format!(r#"{}="{}""#, name, value);

                    match Tag::from_str(&tag_str) {
                        Ok(tag) => {
                            log::debug!("FsTabEntry::tag value: {:?}", tag);

                            Some(tag)
                        }
                        Err(e) => {
                            log::debug!("FsTabEntry::tag {:?}", e);

                            None
                        }
                    }
                }
            },
            code => {
                log::debug!("FsTabEntry::tag failed to get tag. libmount::mnt_fs_get_tag returned error code: {:?}", code);

                None
            }
        }
    }

    /// Returns the interval in days between file system backups by the `dump` command on ext2/3/4 filesystems.
    pub fn backup_frequency(&self) -> i32 {
        let freq = unsafe { libmount::mnt_fs_get_freq(self.inner) };
        log::debug!("FsTabEntry::backup_frequency value: {:?} (days)", freq);

        freq
    }

    /// Returns the specified order in which file systems are checked by the `fsck` command.
    pub fn fsck_checking_order(&self) -> Option<usize> {
        log::debug!("FsTabEntry::fsck_checking_order getting fsck checking order");

        let mut order = MaybeUninit::<libc::c_int>::zeroed();
        order.write(unsafe { libmount::mnt_fs_get_passno(self.inner) });

        match unsafe { order.assume_init() } {
            code if code < 0 => {
                log::debug!("FsTabEntry::fsck_checking_order failed to get fsck checking order. libmount::mnt_fs_get_passno returned error code: {:?}", code);

                None
            }
            order => {
                log::debug!("FsTabEntry::fsck_checking_order value: {:?}", order);

                Some(order as usize)
            }
        }
    }

    //---- END getters
}
