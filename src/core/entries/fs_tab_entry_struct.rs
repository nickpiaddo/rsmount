// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::device::Tag;
use crate::core::entries::FsTabEntryBuilder;
use crate::core::entries::FsTbEntBuilder;
use crate::core::entries::MntEnt;
use crate::core::errors::FsTabEntryError;
use crate::declare_tab_entry;
use crate::fs_tab_entry_shared_methods;

declare_tab_entry!(FsTabEntry, "A configuration line in `/etc/fstab`.");

impl FsTabEntry {
    /// Creates a [`FsTabEntryBuilder`] to configure and construct a new `FsTabEntry`.
    ///
    /// Call the `FsTabEntryBuilder`'s [`build()`](FsTabEntryBuilder::build) method to
    /// construct a new `FsTabEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// use rsmount::core::entries::FsTabEntry;
    /// use rsmount::core::device::Tag;
    /// use rsmount::core::fs::FileSystem;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///     // Example entry in /etc/fstab
    ///     //
    ///     // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
    ///     let uuid: Tag = "UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f".parse()?;
    ///     let entry = FsTabEntry::builder()
    ///         .source(uuid.into())
    ///         .target("/")
    ///         .file_system_type(FileSystem::Ext4)
    ///         // Comma-separated list of mount options.
    ///         .mount_options("rw,relatime")
    ///         // Interval, in days, between file system backups by the dump command on ext2/3/4
    ///         // file systems.
    ///         .backup_frequency(0)
    ///         // Order in which file systems are checked by the `fsck` command.
    ///         .fsck_checking_order(1)
    ///         .build()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn builder() -> FsTabEntryBuilder {
        log::debug!("FsTabEntry::builder creating new `FsTabEntryBuilder` instance");
        FsTbEntBuilder::builder()
    }
}

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

    /// Converts this `FsTabEntry` to a [`MntEnt`].
    pub fn to_mnt_ent(&self) -> Result<MntEnt, FsTabEntryError> {
        log::debug!("FsTabEntry::to_mnt_ent converting `FsTabEntry` to `libmount::mntent`");

        let mut ptr = MaybeUninit::<*mut libmount::mntent>::zeroed();

        let result = unsafe { libmount::mnt_fs_to_mntent(self.inner, ptr.as_mut_ptr()) };

        match result {
            0 => {
                log::debug!("FsTabEntry::to_mnt_ent converted `FsTabEntry` to `libmount::mntent`");
                let ptr = unsafe { ptr.assume_init() };
                let entry = MntEnt::from_raw_parts(ptr);

                Ok(entry)
            }
            code => {
                let err_msg = "failed to convert `FsTabEntry` to `libmount::mntent`".to_owned();
                log::debug!("FsTabEntry::to_mnt_ent {}. libmount::mnt_fs_to_mntent returned error code: {:?}", err_msg, code);

                Err(FsTabEntryError::Action(err_msg))
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

    /// Returns mount options from `/etc/fstab`.
    pub fn mount_options(&self) -> Option<&str> {
        log::debug!("FsTabEntry::mount_options getting fstab mount options");

        let mut opts_ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            opts_ptr.write(libmount::mnt_fs_get_options(self.inner));
        }

        match unsafe { opts_ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("FsTabEntry::mount_options failed to get fstab mount options. libmount::mnt_fs_get_options returned a NULL pointer");

                None
            }
            opts_ptr => {
                let options = ffi_utils::const_char_array_to_str_ref(opts_ptr);
                log::debug!("FsTabEntry::mount_options value: {:?}", options);

                options.ok()
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

impl fmt::Display for FsTabEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output: Vec<String> = vec![];
        if let Some(source) = self.source() {
            output.push(source.to_string());
        }

        if let Some(path) = self.target() {
            let target = format!("{}", path.display());
            output.push(target);
        }

        if let Some(fs_type) = self.file_system_type() {
            output.push(fs_type.to_string());
        }

        if let Some(mount_options) = self.mount_options() {
            output.push(mount_options.to_string());
        }
        let backup_frequency = self.backup_frequency();
        output.push(backup_frequency.to_string());

        if let Some(fsck_checking_order) = self.fsck_checking_order() {
            output.push(fsck_checking_order.to_string());
        }

        write!(f, "{}", output.join(" "))
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::core::fs::FileSystem;
    use pretty_assertions::{assert_eq, assert_ne};
    use std::path::Path;

    #[test]
    fn fs_tab_entry_can_build_an_instance_with_a_uuid_source() -> crate::Result<()> {
        // Root mount
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid: Tag = "UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f".parse()?;
        let entry = FsTabEntry::builder()
            .comment_line("Root mount")
            .source(uuid.into())
            .target("/")
            .file_system_type(FileSystem::Ext4)
            // Comma-separated list of mount options.
            .mount_options("rw,relatime")
            // Interval, in days, between file system backups by the dump command on ext2/3/4
            // file systems.
            .backup_frequency(0)
            // Order in which file systems are checked by the `fsck` command.
            .fsck_checking_order(1)
            .build()?;

        let actual = entry.comment();
        let expected = Some("Root mount\n");
        assert_eq!(actual, expected);

        let actual = entry.source();
        let uuid: Tag = "UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f".parse()?;
        let expected = Some(Source::from(uuid));
        assert_eq!(actual, expected);

        let actual = entry.source_path();
        let expected = None;
        assert_eq!(actual, expected);

        let actual = entry.tag();
        let tag: Tag = "UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f".parse()?;
        let expected = Some(tag);
        assert_eq!(actual, expected);

        let actual = entry.target();
        let expected = Some(Path::new("/"));
        assert_eq!(actual, expected);

        let actual = entry.file_system_type();
        let expected = Some(FileSystem::Ext4);
        assert_eq!(actual, expected);

        let actual = entry.mount_options();
        let expected = Some("rw,relatime");
        assert_eq!(actual, expected);

        let actual = entry.backup_frequency();
        let expected = 0;
        assert_eq!(actual, expected);

        let actual = entry.fsck_checking_order();
        let expected = Some(1);
        assert_eq!(actual, expected);

        Ok(())
    }
}
