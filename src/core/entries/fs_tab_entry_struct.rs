// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::fs::File;
use std::mem::MaybeUninit;
use std::path::Path;
use std::str::FromStr;

// From this library
use crate::core::cache::Cache;

use crate::core::device::Source;
use crate::core::device::Tag;

use crate::core::entries::FsTabEntryBuilder;
use crate::core::entries::FsTbEntBuilder;
use crate::core::entries::MntEnt;

use crate::core::errors::FsTabEntryError;
use crate::core::fs::FileSystem;
use crate::ffi_utils;

/// A configuration line in `/etc/fstab`.
#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct FsTabEntry {
    pub(crate) inner: *mut libmount::libmnt_fs,
}

impl Drop for FsTabEntry {
    fn drop(&mut self) {
        log::debug!("FsTabEntry::drop deallocating `FsTabEntry` instance");

        unsafe { libmount::mnt_unref_fs(self.inner) }
    }
}

impl FsTabEntry {
    #[doc(hidden)]
    /// Increments the inner value's reference counter.
    pub(crate) fn incr_ref_counter(&mut self) {
        unsafe { libmount::mnt_ref_fs(self.inner) }
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Decrements the inner value's reference counter.
    pub(crate) fn decr_ref_counter(&mut self) {
        unsafe { libmount::mnt_unref_fs(self.inner) }
    }

    #[doc(hidden)]
    /// Borrows an instance.
    pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_fs) -> FsTabEntry {
        let mut entry = Self { inner: ptr };
        // We are virtually ceding ownership of this table entry which will be automatically
        // deallocated once it is out of scope, incrementing its reference counter protects it from
        // being freed prematurely.
        entry.incr_ref_counter();

        entry
    }

    #[doc(hidden)]
    /// Wraps a raw `libmount::mnt_fs` pointer in a safe instance.
    #[inline]
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_fs) -> FsTabEntry {
        Self { inner: ptr }
    }

    #[doc(hidden)]
    /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe reference.
    pub(crate) unsafe fn ref_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_fs>,
    ) -> (*mut *mut libmount::libmnt_fs, &'a FsTabEntry) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &*(raw_ptr as *const _ as *const FsTabEntry) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe mutable reference.
    pub(crate) unsafe fn mut_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_fs>,
    ) -> (*mut *mut libmount::libmnt_fs, &'a mut FsTabEntry) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &mut *(raw_ptr as *mut FsTabEntry) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    /// Creates a new instance.
    pub(crate) fn new() -> Result<FsTabEntry, FsTabEntryError> {
        log::debug!("FsTabEntry::new creating a new `FsTabEntry` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        unsafe {
            inner.write(libmount::mnt_new_fs());
        }

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `FsTabEntry` instance".to_owned();
                log::debug!(
                    "FsTabEntry::new {err_msg}. libmount::mnt_new_fs returned a NULL pointer"
                );

                Err(FsTabEntryError::Creation(err_msg))
            }
            inner => {
                log::debug!("FsTabEntry::new created a new `FsTabEntry` instance");
                let entry = Self { inner };

                Ok(entry)
            }
        }
    }

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
    ///     let uuid = Tag::try_from("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")?;
    ///     let entry = FsTabEntry::builder()
    ///         .source(uuid)
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

    //---- BEGIN getters

    /// Allocates a new `FsTabEntry`, and a copies all the source's fields to the new
    /// instance except any private user data.
    pub fn copy(&self) -> Result<FsTabEntry, FsTabEntryError> {
        log::debug!("FsTabEntry::copy copying `FsTabEntry`");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_copy_fs(std::ptr::null_mut(), self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to copy `FsTabEntry`".to_owned();
                log::debug!(
                    "FsTabEntry::copy {err_msg}. libmount::mnt_copy_fs returned a NULL pointer"
                );

                Err(FsTabEntryError::Action(err_msg))
            }
            ptr => {
                log::debug!("FsTabEntry::copy copied `FsTabEntry`");
                let entry = Self::from_ptr(ptr);

                Ok(entry)
            }
        }
    }

    /// Returns the file system type specified at creation.
    pub fn file_system_type(&self) -> Option<FileSystem> {
        log::debug!("FsTabEntry::file_system_type getting file system type");

        let mut fs_ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            fs_ptr.write(libmount::mnt_fs_get_fstype(self.inner));
        }

        match unsafe { fs_ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "FsTabEntry::file_system_type failed to get file system type. libmount::mnt_fs_get_fstype returned a NULL pointer");

                None
            }
            fs_ptr => {
                let fs_type = ffi_utils::const_char_array_to_str_ref(fs_ptr);
                log::debug!("FsTabEntry::file_system_type value: {:?}", fs_type);

                fs_type.ok().and_then(|s| FileSystem::from_str(s).ok())
            }
        }
    }

    /// Returns the value of the option matching `option_name`.
    pub fn option_value<T>(&self, option_name: T) -> Option<String>
    where
        T: AsRef<str>,
    {
        let option_name = option_name.as_ref();
        log::debug!(
            "FsTabEntry::option_value getting value of option: {:?}",
            option_name
        );

        let opt_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;

        let mut value_start = MaybeUninit::<*mut libc::c_char>::zeroed();
        let mut size = MaybeUninit::<libc::size_t>::zeroed();

        let result = unsafe {
            libmount::mnt_fs_get_option(
                self.inner,
                opt_cstr.as_ptr(),
                value_start.as_mut_ptr(),
                size.as_mut_ptr(),
            )
        };

        match result {
            0 => {
                let value_start = unsafe { value_start.assume_init() };
                let size = unsafe { size.assume_init() };
                let mut data = unsafe { std::slice::from_raw_parts(value_start, size).to_owned() };
                // add NUL terminator
                data.push(0);

                let value = ffi_utils::c_char_array_to_string(data.as_ptr());

                log::debug!(
                    "FsTabEntry::option_value option {:?} has value {:?}",
                    option_name,
                    value
                );

                Some(value)
            }
            1 => {
                log::debug!(
                    "FsTabEntry::option_value found no option matching {:?}",
                    option_name
                );

                None
            }
            code => {
                log::debug!(
 "FsTabEntry::option_value failed to get value of option: {:?}. libmount::mnt_fs_get_option returned error code: {:?}",
                            option_name,
                            code
                        );

                None
            }
        }
    }

    /// Returns the entry's source field.
    pub fn source(&self) -> Option<Source> {
        log::debug!("FsTabEntry::source getting the mount's source");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_source(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "FsTabEntry::source failed to get the mount's source. libmount::mnt_fs_get_source returned a NULL pointer");

                None
            }

            ptr => {
                let source = ffi_utils::const_char_array_to_str_ref(ptr);

                match source {
                    Ok(source) => {
                        log::debug!("FsTabEntry::source value: {:?}", source);

                        Source::from_str(source).ok()
                    }
                    Err(e) => {
                        log::debug!("FsTabEntry::source {e:?}");

                        None
                    }
                }
            }
        }
    }

    /// Returns the entry's source path which can be
    /// - a directory for bind mounts (in `/etc/fstab` or `/etc/mtab` only)
    /// - a path to a block device for standard mounts.
    pub fn source_path(&self) -> Option<&Path> {
        log::debug!("FsTabEntry::source_path getting the mount's source path");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_srcpath(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "FsTabEntry::source_path failed to get the mount's source path. libmount::mnt_fs_get_srcpath returned a NULL pointer");

                None
            }

            ptr => {
                let path = ffi_utils::const_c_char_array_to_path(ptr);
                log::debug!("FsTabEntry::source_path value: {:?}", path);

                Some(path)
            }
        }
    }

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
                            log::debug!("FsTabEntry::tag {e:?}");

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

    /// Returns the path to the mount point.
    pub fn target(&self) -> Option<&Path> {
        log::debug!("FsTabEntry::target getting path to mount point");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_target(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "FsTabEntry::target failed to get path to mount point. libmount::mnt_fs_get_target returned a NULL pointer");

                None
            }
            ptr => {
                let path = ffi_utils::const_c_char_array_to_path(ptr);
                log::debug!("FsTabEntry::target value: {:?}", path);

                Some(path)
            }
        }
    }

    //---- END getters

    //---- BEGIN setters

    //TODO change error type to io::Result
    /// Sets the file stream to print debug messages to.
    pub fn print_debug_to(&mut self, stream: &mut File) -> Result<(), FsTabEntryError> {
        log::debug!("FsTabEntry::print_debug_to setting file stream to print debug messages to");

        if ffi_utils::is_open_write_only(stream)? || ffi_utils::is_open_read_write(stream)? {
            let file_stream = ffi_utils::write_only_c_file_stream_from(stream)?;

            let result = unsafe { libmount::mnt_fs_print_debug(self.inner, file_stream as *mut _) };
            match result {
                0 => {
                    log::debug!(
                        "FsTabEntry::print_debug_to set file stream to print debug messages to"
                    );

                    Ok(())
                }
                code => {
                    let err_msg = "failed to set file stream to print debug messages to".to_owned();
                    log::debug!( "FsTabEntry::print_debug_to {err_msg}. libmount::mnt_fs_print_debug returned error code: {code:?}");

                    Err(FsTabEntryError::Action(err_msg))
                }
            }
        } else {
            let err_msg = "missing write permission for given stream".to_owned();
            log::debug!("FsTabEntry::print_debug_to {err_msg}");

            Err(FsTabEntryError::Permission(err_msg))
        }
    }

    /// Sets a comment line.
    pub fn set_comment(&mut self, mut comment: String) -> Result<(), FsTabEntryError> {
        log::debug!(
            "FsTabEntry::set_comment setting comment line: {:?}",
            comment
        );
        // To avoid commenting out the whole entry when writing this `Entry` to file.
        if !comment.ends_with('\n') {
            comment.push('\n');
        }

        let comment_cstr = ffi_utils::as_ref_str_to_c_string(&comment)?;

        let result = unsafe { libmount::mnt_fs_set_comment(self.inner, comment_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!("FsTabEntry::set_comment set comment line: {:?}", comment);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set comment line: {:?}", comment);
                log::debug!( "FsTabEntry::set_comment {}. libmount::mnt_fs_set_comment returned error code: {:?}", err_msg, code);

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    /// Sets the file system associated with the device to mount.
    pub fn set_file_system_type(&mut self, fs_type: FileSystem) -> Result<(), FsTabEntryError> {
        log::debug!(
            "FsTabEntry::set_file_system_type setting file system type to: {:?}",
            fs_type
        );

        let fs_type_cstr = ffi_utils::as_ref_str_to_c_string(&fs_type)?;

        let result = unsafe { libmount::mnt_fs_set_fstype(self.inner, fs_type_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTabEntry::set_file_system_type set file system type to: {:?}",
                    fs_type
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set file system type to: {:?}", fs_type);
                log::debug!( "FsTabEntry::set_file_system_type {err_msg}. libmount::mnt_fs_set_fstype returned error code: {code:?}");

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    /// Sets the interval in days between file system backups by the `dump` command on ext2/3/4
    /// file systems. (see the `dump` command's [manpage](https://manpages.org/dump/8))
    pub fn set_backup_frequency(&mut self, interval: i32) -> Result<(), FsTabEntryError> {
        log::debug!(
            "FsTabEntry::set_backup_frequency setting interval between backups to: {:?} days",
            interval
        );

        let result = unsafe { libmount::mnt_fs_set_freq(self.inner, interval) };

        match result {
            0 => {
                log::debug!( "FsTabEntry::set_backup_frequency setting interval between backups to: {:?} days", interval);

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to set interval between backups to: {:?} days",
                    interval
                );
                log::debug!( "FsTabEntry::set_backup_frequency {err_msg}. libmount::mnt_fs_set_freq returned error code: {code:?}");

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    /// Sets the order in which file systems are checked by the `fsck` command. Setting this value
    /// to `0` will direct `fsck` to skip and not check at all the device referenced in this
    /// data structure.
    pub fn set_fsck_checking_order(&mut self, order: i32) -> Result<(), FsTabEntryError> {
        log::debug!(
            "FsTabEntry::set_fsck_checking_order setting file system checking order: {:?}",
            order
        );

        let result = unsafe { libmount::mnt_fs_set_passno(self.inner, order) };

        match result {
            0 => {
                log::debug!(
                    "FsTabEntry::set_fsck_checking_order set file system checking order: {:?}",
                    order
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set file system checking order: {:?}", order);
                log::debug!( "FsTabEntry::set_fsck_checking_order {err_msg}. libmount::mnt_fs_set_passno returned error code: {code:?}");

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    /// Sets mount options string.
    pub fn set_mount_options<T>(&mut self, options: T) -> Result<(), FsTabEntryError>
    where
        T: AsRef<str>,
    {
        let options = options.as_ref();
        log::debug!(
            "FsTabEntry::set_mount_options setting mount options string to: {:?}",
            options
        );

        let options_cstr = ffi_utils::as_ref_str_to_c_string(options)?;

        let result = unsafe { libmount::mnt_fs_set_options(self.inner, options_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTabEntry::set_mount_options set mount options string to: {:?}",
                    options
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount options string to: {:?}", options);
                log::debug!( "FsTabEntry::set_mount_options {err_msg}. libmount::mnt_fs_set_options returned error code: {code:?}");

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets the source of the device to mount.
    ///
    /// A `source` can take any of the following forms:
    /// - block device path (e.g. `/dev/sda1`),
    /// - network ID:
    ///     - Samba: `smb://ip-address-or-hostname/shared-dir`,
    ///     - NFS: `hostname:/shared-dir`  (e.g. knuth.cwi.nl:/dir)
    ///     - SSHFS: `user@ip-address-or-hostname:/shared-dir`  (e.g. tux@192.168.0.1:/home/tux)
    /// - label:
    ///     - `UUID=uuid`,
    ///     - `LABEL=label`,
    ///     - `PARTLABEL=label`,
    ///     - `PARTUUID=uuid`,
    ///     - `ID=id`.
    pub(crate) fn set_mount_source<T>(&mut self, source: T) -> Result<(), FsTabEntryError>
    where
        T: AsRef<str>,
    {
        let source = source.as_ref();
        log::debug!(
            "FsTabEntry::set_mount_source setting the source of a device to mount: {:?}",
            source
        );

        let source_cstr = ffi_utils::as_ref_str_to_c_string(source)?;

        let result = unsafe { libmount::mnt_fs_set_source(self.inner, source_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTabEntry::set_mount_source set the source of a device to mount: {:?}",
                    source
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to set the source of a device to mount: {:?}",
                    source
                );
                log::debug!( "FsTabEntry::set_mount_source {err_msg}. libmount::mnt_fs_set_source returned error code: {code:?}");

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets a device's mount point.
    pub(crate) fn set_mount_target<T>(&mut self, path: T) -> Result<(), FsTabEntryError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "FsTabEntry::set_mount_target setting device mount point to: {:?}",
            path
        );

        let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;

        let result = unsafe { libmount::mnt_fs_set_target(self.inner, path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTabEntry::set_mount_target set device mount point to: {:?}",
                    path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set a device's mount point to: {:?}", path);
                log::debug!( "FsTabEntry::set_mount_target {err_msg}. libmount::mnt_fs_set_target returned error code: {code:?}");

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    /// Sets the source of the device to mount.
    pub fn set_source<S>(&mut self, source: S) -> Result<(), FsTabEntryError>
    where
        S: Into<Source>,
    {
        let source: Source = source.into();
        log::debug!(
            "FsTabEntry::set_source setting the source of a device to mount: {:?}",
            source
        );

        self.set_mount_source(source.to_string())
    }

    /// Sets a device's mount point.
    pub fn set_target<T>(&mut self, path: T) -> Result<(), FsTabEntryError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "FsTabEntry::set_target setting device mount point to: {:?}",
            path
        );

        self.set_mount_target(path)
    }

    //---- END setters

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
                log::debug!("FsTabEntry::append_comment {err_msg}. libmount::mnt_fs_append_comment returned error code: {code:?}");

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    /// Appends the `options` parameter to the corresponding VFS, `mountinfo` FS-specific, and userspace list of options.
    pub fn append_options<T>(&mut self, options: T) -> Result<(), FsTabEntryError>
    where
        T: AsRef<str>,
    {
        let options = options.as_ref();
        log::debug!("FsTabEntry::append_options appending options {:?}", options);

        let opts = ffi_utils::as_ref_str_to_c_string(options)?;

        let result = unsafe { libmount::mnt_fs_append_options(self.inner, opts.as_ptr()) };

        match result {
            0 => {
                log::debug!("FsTabEntry::append_options appended options {:?}", options);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to append options {:?}", options);
                log::debug!( "FsTabEntry::append_options {err_msg}. libmount::mnt_fs_append_options returned error code: {code:?}");

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    /// Prepends the `options` parameter to the corresponding VFS, `mountinfo` FS-specific, and userspace list of options.
    pub fn prepend_options<T>(&mut self, options: T) -> Result<(), FsTabEntryError>
    where
        T: AsRef<str>,
    {
        let options = options.as_ref();
        let attrs_cstr = ffi_utils::as_ref_str_to_c_string(options)?;

        log::debug!(
            "FsTabEntry::prepend_options prepending options: {:?}",
            options
        );

        let result = unsafe { libmount::mnt_fs_prepend_options(self.inner, attrs_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTabEntry::prepend_options prepended options: {:?}",
                    options
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to prepend options: {options:?}");
                log::debug!( "FsTabEntry::prepend_options {err_msg}. libmount::mnt_fs_prepend_options returned error code: {code:?}");

                Err(FsTabEntryError::Config(err_msg))
            }
        }
    }

    /// Fills the empty fields in `destination` by copying data from the corresponding fields in
    /// this object.
    pub fn complete(&mut self, destination: &mut FsTabEntry) -> Result<(), FsTabEntryError> {
        log::debug!("FsTabEntry::complete copying fields to destination `FsTabEntry`");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_copy_fs(destination.inner, self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to copy fields to destination `FsTabEntry`".to_owned();
                log::debug!(
                    "FsTabEntry::complete {}. libmount::mnt_copy_fs returned a NULL pointer",
                    err_msg
                );

                Err(FsTabEntryError::Copy(err_msg))
            }
            _ptr => {
                log::debug!("FsTabEntry::complete copied fields to destination `FsTabEntry`");

                Ok(())
            }
        }
    }

    /// Converts this `FsTabEntry` to an [`MntEnt`].
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
                log::debug!("FsTabEntry::to_mnt_ent {err_msg}. libmount::mnt_fs_to_mntent returned error code: {code:?}");

                Err(FsTabEntryError::Action(err_msg))
            }
        }
    }

    //---- END mutators

    //---- BEGIN predicates

    /// Returns `true` if mount options do (or do not) contain an element of the `pattern`
    /// parameter (a comma-separated list of values). See the [`mount` command's
    /// manpage](https://www.man7.org/linux/man-pages/man8/mount.8.html#FILESYSTEM-INDEPENDENT_MOUNT_OPTIONS)
    /// for examples of mount options.
    ///
    /// **Note**:
    /// - a value prefixed with `no` will match if it is **NOT** present in the options list. For
    ///   example, a `"noatime"` pattern means *return `true` if the `atime` option is absent from
    ///   the list of mount options*.
    /// - for values prefixed with a `no`, adding a `+` at the beginning will push the function to
    ///   search for an exact match. For example, a `"+noatime"` pattern means *return `true` if the
    ///   `noatime` option is present in the list of mount options*.
    ///
    /// # Examples
    ///
    /// | Mount options                  | Search pattern   | Result |
    /// | ----                           | ----             | ----   |
    /// | ""                             | ""               | true   |
    /// | ""                             | "noatime"        | true   |
    /// | ""                             | "atime"          | false  |
    /// | "nodiratime,atime,discard"     | ""               | true   |
    /// | "nodiratime,atime,discard"     | "+"              | true   |
    /// | "nodiratime,**atime**,discard" | "atime"          | true   |
    /// | "nodiratime,**atime**,discard" | "noatime"        | false  |
    /// | "nodiratime,atime,**discard**" | "discard,noauto" | true   |
    /// | "**diratime**,atime,discard"   | "nodiratime"     | false  |
    /// | "nodiratime,atime,discard"     | "nodiratime"     | true   |
    /// | "**nodiratime**,atime,discard" | "+nodiratime"    | true   |
    /// | "noexec,atime,discard"         | "+nodiratime"    | false  |
    ///
    pub fn has_any_option<T>(&self, pattern: T) -> bool
    where
        T: AsRef<str>,
    {
        let pattern = pattern.as_ref();
        let pattern_cstr = ffi_utils::as_ref_str_to_c_string(pattern).ok();

        if let Some(pattern_cstr) = pattern_cstr {
            let state =
                unsafe { libmount::mnt_fs_match_options(self.inner, pattern_cstr.as_ptr()) == 1 };
            log::debug!(
                "FsTabEntry::has_any_option does any element of the pattern list {:?} match? {:?}",
                pattern,
                state
            );

            state
        } else {
            log::debug!("FsTabEntry::has_any_option failed to convert pattern to `CString`");

            false
        }
    }

    /// Returns `true` if the file system type of this `FsTabEntry` matches any element of the
    /// of the comma-separated file system names in the `pattern` parameter (see the [`FileSystem`
    /// documentation page](crate::core::fs::FileSystem) for a list of supported file systems).
    ///
    /// **Note:**
    /// - file system names prefixed with a `no` will match if this `FsTabEntry` does **NOT** have the
    ///   file system mentioned.
    /// - a test with a pattern list starting with `no` will apply the prefix to **all** file
    ///   systems in the list (e.g. `"noapfs,ext4"` is equivalent to `"noapfs,noext4"`).
    ///
    /// For example, if this `FsTabEntry` represents an `ext4` device, a test with the following
    /// patterns:
    /// - `"apfs,ntfs"` would return `false`,
    /// - `"apfs,ext4"` would return `true`,
    /// - `"apfs,noext4"` would return `false`,
    /// - `"noapfs,ext4"` would return `false`.
    pub fn has_any_fs_type<T>(&self, pattern: T) -> bool
    where
        T: AsRef<str>,
    {
        let pattern = pattern.as_ref();
        let pattern_cstr = ffi_utils::as_ref_str_to_c_string(pattern).ok();

        if let Some(pattern_cstr) = pattern_cstr {
            let state =
                unsafe { libmount::mnt_fs_match_fstype(self.inner, pattern_cstr.as_ptr()) == 1 };
            log::debug!(
                "FsTabEntry::has_any_fs_type does any element of the pattern list {:?} match? {:?}",
                pattern,
                state
            );

            state
        } else {
            log::debug!("FsTabEntry::has_any_fs_type failed to convert pattern to `CString`");

            false
        }
    }

    /// Returns `true` if data is read directly from the kernel (e.g `/proc/mounts`).
    pub fn is_from_kernel(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_kernel(self.inner) == 1 };
        log::debug!(
            concat!(stringify!($entry_type), "::is_from_kernel value: {:?}"),
            state
        );

        state
    }

    /// Returns `true` if the file system of this `FsTabEntry` is a network file system.
    pub fn is_net_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_netfs(self.inner) == 1 };
        log::debug!("FsTabEntry::is_net_fs value: {:?}", state);

        state
    }

    /// Returns `true` if the file system of this `FsTabEntry` is a pseudo file system type (`proc`, `cgroups`).
    pub fn is_pseudo_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_pseudofs(self.inner) == 1 };
        log::debug!("FsTabEntry::is_pseudo_fs value: {:?}", state);

        state
    }

    #[cfg(mount = "v2_39")]
    /// Returns `true` if the file system of this `FsTabEntry` is a regular file system (neither a network nor a pseudo file system).
    pub fn is_regular_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_regularfs(self.inner) == 1 };
        log::debug!("FsTabEntry::is_regular_fs value: {:?}", state);

        state
    }

    /// Returns `true` if this `FsTabEntry` represents a swap partition.
    pub fn is_swap(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_swaparea(self.inner) == 1 };
        log::debug!("FsTabEntry::is_swap value: {:?}", state);

        state
    }

    /// Returns `true` if the `source` parameter matches the `source` field in this `FsTabEntry`.
    ///
    /// Using the provided `cache`, this method will perform the following comparisons in sequence:
    /// - `source` vs the value of the `source` field in this `FsTabEntry`
    ///
    /// - the resolved value of the `source` parameter vs the value of the `source` field in this
    ///   `FsTabEntry`
    /// - the resolved value of the `source` parameter vs the resolved value of the `source` field
    ///   in this `FsTabEntry`
    /// - the resolved value of the `source` parameter vs the evaluated tag of the `source` field
    ///   in this `FsTabEntry`
    ///
    /// *Resolving* the `source` parameter means searching and returning the absolute path to
    /// the device it represents. The same for *evaluating* a tag.
    pub fn is_source(&self, source: &Source, cache: &Cache) -> bool {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok();

        if let Some(source_cstr) = source_cstr {
            let state = unsafe {
                libmount::mnt_fs_match_source(self.inner, source_cstr.as_ptr(), cache.inner) == 1
            };
            log::debug!(
                "FsTabEntry::is_source is {:?} the source of this entry? {:?}",
                source,
                state
            );

            state
        } else {
            log::debug!("FsTabEntry::is_source failed to convert source to `CString`");

            false
        }
    }

    /// Returns `true` if the `source` parameter matches exactly the `source` field in this
    /// `FsTabEntry`.
    ///
    /// **Note:** redundant forward slashes are ignored when comparing values.
    pub fn is_exact_source(&self, source: &Source) -> bool {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok();

        if let Some(source_cstr) = source_cstr {
            let state =
                unsafe { libmount::mnt_fs_streq_srcpath(self.inner, source_cstr.as_ptr()) == 1 };
            log::debug!(
                "FsTabEntry::is_exact_source is {:?} the exact source of this entry? {:?}",
                source,
                state
            );

            state
        } else {
            log::debug!("FsTabEntry::is_exact_source failed to convert source to `CString`");

            false
        }
    }

    /// Returns `true` if `path` matches the `target` field in this `FsTabEntry`. Using
    /// the provided `cache`, this method will perform the following comparisons in sequence:
    ///
    /// - `path` vs the value of the `target` field in this `FsTabEntry`
    /// - canonicalized `path` vs the value of the `target` field in this `FsTabEntry`
    /// - canonicalized `path` vs the canonicalized value of the `target` field in this
    ///   `FsTabEntry` if is not from `/proc/self/mountinfo`
    pub fn is_target<T>(&self, path: T, cache: &Cache) -> bool
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok();

        if let Some(path_cstr) = path_cstr {
            let state = unsafe {
                libmount::mnt_fs_match_target(self.inner, path_cstr.as_ptr(), cache.inner) == 1
            };
            log::debug!(
                "FsTabEntry::is_target is {:?} the target of this entry? {:?}",
                path,
                state
            );

            state
        } else {
            log::debug!("FsTabEntry::is_target failed to convert path to `CString`");

            false
        }
    }

    /// Returns `true` if `path` matches **exactly** the `target` field in this `FsTabEntry`.
    ///
    /// **Note:** redundant forward slashes are ignored when comparing values.
    pub fn is_exact_target<T>(&self, path: T) -> bool
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok();

        if let Some(path_cstr) = path_cstr {
            let state =
                unsafe { libmount::mnt_fs_streq_target(self.inner, path_cstr.as_ptr()) == 1 };
            log::debug!(
                "FsTabEntry::is_exact_target is {:?} the exact target of this entry? {:?}",
                path,
                state
            );

            state
        } else {
            log::debug!("FsTabEntry::is_exact_target failed to convert path to `CString`");

            false
        }
    }

    //---- END predicates
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
            .source(uuid)
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
