// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;

// From this library
use crate::core::device::Tag;
use crate::core::entries::FsTabEntry;
use crate::core::errors::FsTabError;
use crate::declare_tab;
use crate::fs_tab_shared_methods;

declare_tab!(
    FsTab,
    r##"

Composer/editor of `fstab` file system description files.

An `FsTab` allows you to programmatically
- compose,
- edit,
- import,
- export,
- and/or merge

file system description files used by the [`mount`
command](https://www.man7.org/linux/man-pages/man8/mount.8.html) to attach:
- a block device,
- a shared network file system,
- or a pseudo-filesystem, to a mount point in a file hierarchy.

It holds each line in a file system description file as a list of [`FsTabEntry`]
instances. You can create a description file from scratch, or import information from data
sources on your system.

# `/etc/fstab`

The `/etc/fstab` file contains descriptive information about devices the OS can mount.

It has the following layout:
- each file system is described on a separate line,
- fields on each line are separated by tabs or spaces,
- lines starting with `#` are comments,
- blank lines are ignored.

`/etc/fstab` is only read by programs, and not written; it is the duty of the system
administrator to properly create and maintain this file.

Below is a sample `/etc/fstab` file, with added comments, extracted from an Alpine
Linux virtual machine.

```text
# /etc/fstab
# Alpine Linux 3.19 (installed from alpine-virt-3.19.1-x86_64.iso)
#

UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f  /      ext4  rw,relatime 0 1
UUID=07aae9ba-12a1-4325-8b16-2ee214a6d7fd  /boot  ext4  rw,relatime 0 2
UUID=b9d72af2-f231-4cf8-9d0a-ba19e94a5087  swap   swap  defaults    0 0

/dev/cdrom    /media/cdrom  iso9660 noauto,ro       0 0
/dev/usbdisk  /media/usb    vfat    noauto          0 0
none          /tmp          tmpfs   nosuid,nodev    0 0
```

# `fstab` file format

The table shown above has a 6-column structure, where each column defines a specific
parameter, and must be setup in the same order as in the following excerpt:

```text
UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f  /      ext4  rw,relatime  0   1
                   (1)                    (2)      (3)     (4)      (5) (6)
```

- `(1)` **Device**: the name or UUID of the device to mount, otherwise known as the
***source***.
- `(2)` **Mount Point**: the directory on which the device will be mounted, called the
***target***.
- `(3)` **File System Type**: the type of file system the device uses (e.g. `ext4`, `tmpfs`, etc.).
- `(4)` **Options**: a comma-separated list of [filesystem-independent mount
options](https://www.man7.org/linux/man-pages/man8/mount.8.html#FILESYSTEM-INDEPENDENT_MOUNT_OPTIONS).
- `(5)` **File System Backup**: a boolean value, `1` or `0`, to respectively allow or disallow
the `dump` utility to examine files on an `ext2`/`ext3` file system, and to determine which
files need to be backed up. This is a legacy method that should NOT be used! Always set it to
`0`.
- `(6)` **File System Check Order**: the order in which the `fsck` command is run on the
devices to mount, to check and repair Linux file systems. Entries with the **lowest** value
will be checked **first**. Two entries with the same check order value will be verified in
parallel.<br> The value for the root file system should be set to `1`, and the others should
have a value of at least `2`, keeping in mind that `0` means `fsck` will not check the file
system.

# Examples

## Compose a file system description file

```text
# /etc/fstab
# Example from scratch

UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
/dev/usbdisk /media/usb vfat noauto 0 0
none /tmp tmpfs nosuid,nodev 0 0
```

The following code will create a file system description file identical to the example above.

```
# use pretty_assertions::assert_eq;
use tempfile::tempfile;
use std::fs::File;
use std::str::FromStr;
use rsmount::tables::FsTab;
use rsmount::core::entries::FsTabEntry;
use rsmount::core::device::BlockDevice;
use rsmount::core::device::Pseudo;
use rsmount::core::device::Source;
use rsmount::core::device::Tag;
use rsmount::core::fs::FileSystem;

fn main() -> rsmount::Result<()> {
    let mut fstab = FsTab::new()?;

    // # /etc/fstab
    fstab.set_intro_comments("# /etc/fstab\n")?;
    // # Example from scratch
    fstab.append_to_intro_comments("# Example from scratch\n")?;
    //
    fstab.append_to_intro_comments("\n")?;

    // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
    let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
    let entry1 = FsTabEntry::builder()
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

    // /dev/usbdisk /media/usb vfat noauto 0 0
    let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
    let entry2 = FsTabEntry::builder()
        .source(block_device)
        .target("/media/usb")
        .file_system_type(FileSystem::VFAT)
        .mount_options("noauto")
        .backup_frequency(0)
        .fsck_checking_order(0)
        .build()?;

    // none /tmp tmpfs nosuid,nodev 0 0
    let entry3 = FsTabEntry::builder()
        .source(Pseudo::None.into())
        .target("/tmp")
        .file_system_type(FileSystem::Tmpfs)
        .mount_options("nosuid,nodev")
        .backup_frequency(0)
        .fsck_checking_order(0)
        .build()?;

    fstab.push(entry1)?;
    fstab.push(entry2)?;
    fstab.push(entry3)?;

    // Open file
    let mut file: File = tempfile().unwrap();

    // Write to disk
    fstab.export_with_comments();
    fstab.write_stream(&mut file)?;

    Ok(())
}
```
"##
);

fs_tab_shared_methods!(FsTab, FsTabEntry, FsTabError);

impl FsTab {
    /// Creates a new `FsTab`, and fills it with entries parsed from the given `file`.
    pub fn new_from_file<T>(file: T) -> Result<FsTab, FsTabError>
    where
        T: AsRef<Path>,
    {
        let file = file.as_ref();
        let file_cstr = ffi_utils::as_ref_path_to_c_string(file)?;
        log::debug!(
            "FsTab::new_from_file creating a new `FsTab` from file {:?}",
            file
        );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();
        unsafe { ptr.write(libmount::mnt_new_table_from_file(file_cstr.as_ptr())) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!("failed to create a new `FsTab` from file {:?}", file);
                log::debug!(
                        "FsTab::new_from_file {}. libmount::mnt_new_table_from_file returned a NULL pointer",
                        err_msg
                    );

                Err(FsTabError::Creation(err_msg))
            }
            ptr => {
                log::debug!("FsTab::new_from_file created a new `FsTab` instance");
                let table = Self::from_ptr(ptr);

                Ok(table)
            }
        }
    }

    /// Creates a new `FsTab`, and fills it with entries parsed from the files with extension
    /// `.fstab` in the given `directory`.
    pub fn new_from_directory<T>(directory: T) -> Result<FsTab, FsTabError>
    where
        T: AsRef<Path>,
    {
        let dir = directory.as_ref();
        let dir_cstr = ffi_utils::as_ref_path_to_c_string(dir)?;
        log::debug!(
            "FsTab::new_from_directory creating a new `FsTab` from files in {:?}",
            dir
        );

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();
        unsafe { ptr.write(libmount::mnt_new_table_from_dir(dir_cstr.as_ptr())) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!("failed to create a new `FsTab` from files in {:?}", dir);
                log::debug!(
                        "FsTab::new_from_directory {}. libmount::mnt_new_table_from_dir returned a NULL pointer",
                        err_msg
                    );

                Err(FsTabError::Creation(err_msg))
            }
            ptr => {
                log::debug!("FsTab::new_from_directory created a new `FsTab` instance");
                let table = Self::from_ptr(ptr);

                Ok(table)
            }
        }
    }

    //---- BEGIN setters

    /// Sets the introductory comment in the table.
    ///
    /// In the example below, lines `1` through `3` included are the introduction comments.
    ///
    /// ```text
    /// 1 | # /etc/fstab
    /// 2 | # Example from scratch
    /// 3 |
    /// 4 | UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
    /// 5 | /dev/usbdisk /media/usb vfat noauto 0 0
    /// 6 | tmpfs /tmp tmpfs nosuid,nodev 0 0
    /// 7 |
    /// 8 | # Auto generated by Acme.
    /// 9 |
    /// ```
    pub fn set_intro_comments<T>(&mut self, comment: T) -> Result<(), FsTabError>
    where
        T: AsRef<str>,
    {
        let comment = comment.as_ref();
        let comment_cstr = ffi_utils::as_ref_str_to_c_string(comment)?;
        log::debug!(
            "FsTab::set_intro_comments setting intro comment to {:?}",
            comment
        );

        let result =
            unsafe { libmount::mnt_table_set_intro_comment(self.inner, comment_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::set_intro_comments set intro comment to {:?}",
                    comment
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set intro comment to {:?}", comment);
                log::debug!("FsTab::set_intro_comments {}. libmount::mnt_table_set_intro_comment returned error code: {:?}", err_msg, code);

                Err(FsTabError::Config(err_msg))
            }
        }
    }

    /// Sets the trailing comment in the table.
    ///
    /// In the example below, lines `7` through `9` included are the trailing comments.
    ///
    /// ```text
    /// 1 | # /etc/fstab
    /// 2 | # Example from scratch
    /// 3 |
    /// 4 | UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
    /// 5 | /dev/usbdisk /media/usb vfat noauto 0 0
    /// 6 | tmpfs /tmp tmpfs nosuid,nodev 0 0
    /// 7 |
    /// 8 | # Auto generated by Acme.
    /// 9 |
    /// ```
    pub fn set_trailing_comments<T>(&mut self, comment: T) -> Result<(), FsTabError>
    where
        T: AsRef<str>,
    {
        let comment = comment.as_ref();
        let comment_cstr = ffi_utils::as_ref_str_to_c_string(comment)?;
        log::debug!(
            "FsTab::set_trailing_comments setting trailing comment to {:?}",
            comment
        );

        let result =
            unsafe { libmount::mnt_table_set_trailing_comment(self.inner, comment_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::set_trailing_comments set trailing comment to {:?}",
                    comment
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set trailing comment to {:?}", comment);
                log::debug!("FsTab::set_trailing_comments {}. libmount::mnt_table_set_trailing_comment returned error code: {:?}", err_msg, code);

                Err(FsTabError::Config(err_msg))
            }
        }
    }

    //---- END setters

    //---- BEGIN mutators

    fn enable_comments(ptr: *mut libmount::libmnt_table, enable: bool) {
        let op = if enable { 1 } else { 0 };
        unsafe { libmount::mnt_table_enable_comments(ptr, op) }
    }

    /// Keeps comment lines when importing table entries from files.
    pub fn import_with_comments(&mut self) {
        log::debug!("FsTab::import_with_comments enabling comment parsing");

        Self::enable_comments(self.inner, true)
    }

    /// Skips comment lines when importing table entries from files.
    pub fn import_without_comments(&mut self) {
        log::debug!("FsTab::import_without_comments disabling comment parsing");

        Self::enable_comments(self.inner, false)
    }

    /// Imports entries from files with a `.fstab` extension in a given directory. File names are
    /// ordered by [strverscmp(3)](https://www.man7.org/linux/man-pages/man3/strverscmp.3.html)
    /// before their contents are added to the table.
    ///
    /// **Note:**
    /// - this method ignores any dotfile in the directory.
    /// - by default, comment lines are ignored during importation. If you want them included, call
    /// [`FsTab::import_with_comments`] **before** invoking this method.
    pub fn import_directory<T>(&mut self, dir_path: T) -> Result<(), FsTabError>
    where
        T: AsRef<Path>,
    {
        let dir_path = dir_path.as_ref();
        let dir_path_cstr = ffi_utils::as_ref_path_to_c_string(dir_path)?;
        log::debug!(
            "FsTab::import_dir importing table entries from files in {:?}",
            dir_path
        );

        let result = unsafe { libmount::mnt_table_parse_dir(self.inner, dir_path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::import_directory imported table entries from files in {:?}",
                    dir_path
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to import table entries from files in {:?}",
                    dir_path
                );
                log::debug!("FsTab::import_directory {}. libmount::mnt_table_parse_dir returned error code: {:?}", err_msg, code);

                Err(FsTabError::Import(err_msg))
            }
        }
    }

    /// Parses `/etc/fstab` or the file specified by the environment variable `LIBMOUNT_FSTAB`,
    /// then appends the entries it collects to this `FsTab`.
    ///
    /// **Note:** by default, comment lines are ignored during importation. If you want them
    /// included, call [`FsTab::import_with_comments`] **before** invoking this method.
    pub fn import_etc_fstab(&mut self) -> Result<(), FsTabError> {
        log::debug!("FsTab::import_etc_fstab import entries from /etc/fstab");

        let result = unsafe { libmount::mnt_table_parse_fstab(self.inner, std::ptr::null()) };

        match result {
            0 => {
                log::debug!("FsTab::import_etc_fstab imported entries from /etc/fstab");

                Ok(())
            }
            code => {
                let err_msg = "failed to import entries from /etc/fstab".to_owned();
                log::debug!("FsTab::import_etc_fstab {}. libmount::mnt_table_parse_fstab returned error code: {:?}", err_msg, code);

                Err(FsTabError::Import(err_msg))
            }
        }
    }

    /// Parses the given file, then appends the entries it collected to the table.
    ///
    /// **Note:**
    /// - by default, comment lines are ignored during import. If you want them included, call
    /// [`FsTab::import_with_comments`] **before** invoking this method.
    /// - the parser ignores lines with syntax errors. It will report defective lines to the caller
    /// through an error callback function.
    ///
    // FIXME Defective lines are reported to the caller by the errcb() function (see mnt_table_set_parser_errcb()).
    // can not currently wrap the function `mnt_table_set_parser_errcb`
    pub fn import_file<T>(&mut self, file_path: T) -> Result<(), FsTabError>
    where
        T: AsRef<Path>,
    {
        let file_path = file_path.as_ref();
        let file_path_cstr = ffi_utils::as_ref_path_to_c_string(file_path)?;
        log::debug!(
            "FsTab::import_file importing table entries from file {:?}",
            file_path
        );

        let result = unsafe { libmount::mnt_table_parse_file(self.inner, file_path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::import_file imported table entries from file {:?}",
                    file_path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to import table entries from file {:?}", file_path);
                log::debug!("FsTab::import_file {}. libmount::mnt_table_parse_file returned error code: {:?}", err_msg, code);

                Err(FsTabError::Import(err_msg))
            }
        }
    }

    /// Parses the given [`File`] saving debug messages, and any parsing error to the
    /// `parsing_errors` file.
    ///
    /// **Note:** by default, comment lines are ignored during importation. If you want them
    /// included, call [`FsTab::import_with_comments`] **before** invoking this method.
    pub fn import_from_stream<T>(
        &mut self,
        file: &File,
        parsing_errors: T,
    ) -> Result<(), FsTabError>
    where
        T: AsRef<Path>,
    {
        if ffi_utils::is_open_read_only(file)? || ffi_utils::is_open_read_write(file)? {
            let parsing_errors = parsing_errors.as_ref();
            let path_cstr = ffi_utils::as_ref_path_to_c_string(parsing_errors)?;
            let file_stream = ffi_utils::read_only_c_file_stream_from(file)?;

            log::debug!(
            "FsTab::import_from_stream importing entries from file stream, saving parsing errors to {:?}",
            parsing_errors
        );

            let result = unsafe {
                libmount::mnt_table_parse_stream(
                    self.inner,
                    file_stream as *mut _,
                    path_cstr.as_ptr(),
                )
            };

            match result {
                0 => {
                    log::debug!("FsTab::import_from_stream imported entries from file stream, saving parsing errors to {:?}", parsing_errors);

                    Ok(())
                }
                code => {
                    let err_msg = format!(
                        "failed to import entries from file stream, saving parsing errors to {:?}",
                        parsing_errors
                    );
                    log::debug!("FsTab::import_from_stream {}. libmount::mnt_table_parse_stream returned error code: {:?}", err_msg, code);

                    Err(FsTabError::Import(err_msg))
                }
            }
        } else {
            let err_msg = "missing read permission for given file stream".to_owned();
            log::debug!("FsTab::import_from_stream {}", err_msg);

            Err(FsTabError::Permission(err_msg))
        }
    }

    /// Appends the content of the function parameter to the introduction comments.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// use rsmount::tables::FsTab;
    /// use rsmount::core::entries::FsTabEntry;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///     let mut fstab = FsTab::new()?;
    ///
    ///     // # /etc/fstab
    ///     fstab.set_intro_comments("# /etc/fstab\n")?;
    ///
    ///     let actual = fstab.intro_comments().unwrap();
    ///     let expected = "# /etc/fstab\n";
    ///     assert_eq!(actual, expected);
    ///
    ///     // Append a new comment line
    ///     fstab.append_to_intro_comments("# Example from scratch\n")?;
    ///
    ///     let actual = fstab.intro_comments().unwrap();
    ///     let expected = "# /etc/fstab\n# Example from scratch\n";
    ///     assert_eq!(actual, expected);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn append_to_intro_comments<T>(&mut self, comment: T) -> Result<(), FsTabError>
    where
        T: AsRef<str>,
    {
        let comment = comment.as_ref();
        let comment_cstr = ffi_utils::as_ref_str_to_c_string(comment)?;
        log::debug!(
            "FsTab::append_to_intro_comments appending to intro comment: {:?}",
            comment
        );

        let result =
            unsafe { libmount::mnt_table_append_intro_comment(self.inner, comment_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "FsTab::append_to_intro_comments appended to intro comment: {:?}",
                    comment
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to append to intro comment: {:?}", comment);
                log::debug!("FsTab::append_to_intro_comments {}. libmount::mnt_table_append_intro_comment returned error code: {:?}", err_msg, code);

                Err(FsTabError::Action(err_msg))
            }
        }
    }

    /// Appends the content of the function parameter to the trailing comments.
    pub fn append_to_trailing_comments<T>(&mut self, comment: T) -> Result<(), FsTabError>
    where
        T: AsRef<str>,
    {
        let comment = comment.as_ref();
        let comment_cstr = ffi_utils::as_ref_str_to_c_string(comment)?;
        log::debug!(
            "FsTab::append_to_trailing_comments appending to trailing comment: {:?}",
            comment
        );

        let result = unsafe {
            libmount::mnt_table_append_trailing_comment(self.inner, comment_cstr.as_ptr())
        };

        match result {
            0 => {
                log::debug!(
                    "FsTab::append_to_trailing_comments appended to trailing comment: {:?}",
                    comment
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to append to trailing comment: {:?}", comment);
                log::debug!("FsTab::append_to_trailing_comments {}. libmount::mnt_table_append_trailing_comment returned error code: {:?}", err_msg, code);

                Err(FsTabError::Action(err_msg))
            }
        }
    }

    /// Keeps intro/trailing comment lines when exporting the table to disk.
    pub fn export_with_comments(&mut self) {
        log::debug!("FsTab::export_with_comments enabling comment parsing");

        Self::enable_comments(self.inner, true)
    }

    /// Skips intro/trailing comment lines when exporting the table to disk.
    pub fn export_without_comments(&mut self) {
        log::debug!("FsTab::export_without_comments disabling comment parsing");

        Self::enable_comments(self.inner, false)
    }

    //---- END mutators

    //---- BEGIN predicates

    /// Returns `true` if this `FsTab` is set to import table entries with their comments.
    pub fn is_importing_comments(&self) -> bool {
        let state = unsafe { libmount::mnt_table_with_comments(self.inner) == 1 };
        log::debug!("FsTab::is_importing_comments value: {:?}", state);

        state
    }

    /// Returns `true` if this `FsTab` is set to export intro/trailing comments.
    pub fn is_exporting_comments(&self) -> bool {
        let state = unsafe { libmount::mnt_table_with_comments(self.inner) == 1 };
        log::debug!("FsTab::is_exporting_comments value: {:?}", state);

        state
    }

    //---- END predicates

    //---- BEGIN getters

    /// Returns the comments at the head of this `FsTab`.
    pub fn intro_comments(&self) -> Option<&str> {
        log::debug!("FsTab::intro_comments getting intro comment");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        unsafe { ptr.write(libmount::mnt_table_get_intro_comment(self.inner)) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("FsTab::intro_comments found no intro comment. libmount::mnt_table_get_intro_comment returned a NULL pointer");

                None
            }
            ptr => {
                let comment = ffi_utils::const_char_array_to_str_ref(ptr).ok();
                log::debug!("FsTab::intro_comments value: {:?}", comment);

                comment
            }
        }
    }

    /// Returns the comments at the tail of this `FsTab`.
    pub fn trailing_comments(&self) -> Option<&str> {
        log::debug!("FsTab::trailing_comments getting trailing comment");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        unsafe { ptr.write(libmount::mnt_table_get_trailing_comment(self.inner)) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("FsTab::trailing_comments found no trailing comment. libmount::mnt_table_get_trailing_comment returned a NULL pointer");

                None
            }
            ptr => {
                let comment = ffi_utils::const_char_array_to_str_ref(ptr).ok();
                log::debug!("FsTab::trailing_comments value: {:?}", comment);

                comment
            }
        }
    }

    #[doc(hidden)]
    /// Searches in [`Direction`] for a [`FsTabEntry`] with fields matching the given [`Tag`].
    fn lookup_tag<'a>(table: &mut Self, direction: Direction, tag: &Tag) -> Option<&'a FsTabEntry> {
        log::debug!(
            "FsTab::lookup_tag searching {:?} for entry matching tag {:?}",
            direction,
            tag
        );
        let name_cstr = ffi_utils::as_ref_str_to_c_string(tag.name().to_string()).ok()?;
        let value_cstr = ffi_utils::as_ref_str_to_c_string(tag.value()).ok()?;

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        unsafe {
            ptr.write(libmount::mnt_table_find_tag(
                table.inner,
                name_cstr.as_ptr(),
                value_cstr.as_ptr(),
                direction as i32,
            ))
        };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!(
                    "failed to find entry matching tag {:?} while searching {:?}",
                    tag, direction
                );
                log::debug!(
                    "FsTab::lookup_tag {}. libmount::mnt_table_find_tag returned a NULL pointer",
                    err_msg
                );

                None
            }
            ptr => {
                log::debug!(
                    "FsTab::lookup_tag found entry matching tag {:?} while searching {:?}",
                    tag,
                    direction
                );
                let boxed = Box::new(ptr);
                let (boxed_ptr, entry) = unsafe { FsTabEntry::ref_from_boxed_ptr(boxed) };
                table.gc.push(boxed_ptr.into());

                Some(entry)
            }
        }
    }

    /// Searches the table from **top** to **bottom**, and returns the first [`FsTabEntry`] with
    /// a `source` field matching the given [`Tag`].
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an
    /// exact `tag` match. To perform a deep search, which implies converting the tag to its
    /// associated device's name, following symlinks, canonicalizing paths, etc., set up a
    /// [`Cache`] with [`FsTab::set_cache`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// use std::path::PathBuf;
    /// use std::str::FromStr;
    /// use rsmount::core::device::Source;
    /// use rsmount::core::device::Tag;
    /// use rsmount::core::entries::FsTabEntry;
    /// use rsmount::tables::FsTab;
    /// use rsmount::core::fs::FileSystem;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///     let mut fstab = FsTab::new()?;
    ///
    ///     // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f  /  ext4  rw,relatime 0 1
    ///     let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
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
    ///     fstab.push(entry)?;
    ///
    ///     let tag: Tag = "UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f".parse()?;
    ///
    ///     let entry = fstab.find_source_tag(&tag);
    ///
    ///     let actual = entry.unwrap().tag();
    ///     let expected = Some(tag);
    ///     assert_eq!(actual, expected);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn find_source_tag(&mut self, tag: &Tag) -> Option<&FsTabEntry> {
        let direction = Direction::Forward;
        log::debug!(
            "FsTab::find_source_tag searching {:?} for the first entry a tag matching {:?}",
            direction,
            tag
        );

        Self::lookup_tag(self, direction, tag)
    }

    /// Searches the table from **bottom** to **top**, and returns the first [`FsTabEntry`] with
    /// a `source` field matching the given [`Tag`].
    ///
    /// By default, a `FsTab` will perform a cursory search, looking for an entry with an
    /// exact `tag` match. To perform a deep search, which implies converting the tag to its
    /// associated device's name, following symlinks, canonicalizing paths, etc., set up a
    /// [`Cache`] with [`FsTab::set_cache`].
    pub fn find_back_source_tag(&mut self, tag: &Tag) -> Option<&FsTabEntry> {
        let direction = Direction::Backward;
        log::debug!(
            "FsTab::find_back_source_tag searching {:?} for the first entry a tag matching {:?}",
            direction,
            tag
        );

        Self::lookup_tag(self, direction, tag)
    }

    //---- END getters
}

impl fmt::Display for FsTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output: Vec<String> = vec![];
        if let Some(intro) = self.intro_comments() {
            output.push(intro.to_string());
        }

        for line in self.iter() {
            output.push(line.to_string());
        }

        if let Some(trailing) = self.trailing_comments() {
            output.push(trailing.to_string());
        }

        write!(f, "{}", output.join("\n"))
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::core::device::BlockDevice;
    use crate::core::device::Pseudo;
    use crate::core::device::Source;
    use crate::core::device::Tag;
    use crate::core::fs::FileSystem;
    use pretty_assertions::{assert_eq, assert_ne};
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Seek, SeekFrom};
    use std::str::FromStr;
    use tempfile::{tempdir, tempfile};

    #[test]
    fn fs_tab_a_new_table_is_empty() -> crate::Result<()> {
        let fs_tab = FsTab::new()?;

        assert!(fs_tab.is_empty());

        Ok(())
    }

    #[test]
    fn fs_tab_an_empty_table_has_no_first_element() -> crate::Result<()> {
        let fs_tab = FsTab::new()?;

        let actual = fs_tab.first();

        assert!(actual.is_none());

        Ok(())
    }

    #[test]
    fn fs_tab_an_empty_table_has_no_last_element() -> crate::Result<()> {
        let fs_tab = FsTab::new()?;

        let actual = fs_tab.last();

        assert!(actual.is_none());

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn fs_tab_indexing_an_empty_table_triggers_a_panic() {
        let fs_tab = FsTab::new().unwrap();

        let _ = fs_tab[0];
    }

    #[test]
    fn fs_tab_push_adds_an_element_to_a_table() -> crate::Result<()> {
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry = FsTabEntry::builder()
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

        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry)?;

        assert_eq!(fs_tab.len(), 1);

        Ok(())
    }

    #[test]
    fn fs_tab_push_front_adds_an_element_at_the_head_of_the_table() -> crate::Result<()> {
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push_front(entry1)?;
        fs_tab.push_front(entry2)?;

        let first = fs_tab.first().unwrap();
        let last = fs_tab.last().unwrap();

        assert_eq!(fs_tab.len(), 2);
        assert_eq!(first.inner, entry2_inner);
        assert_eq!(last.inner, entry1_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_a_table_of_size_1_has_the_same_first_and_last_element() -> crate::Result<()> {
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry = FsTabEntry::builder()
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

        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry)?;

        let first = fs_tab.first();
        let last = fs_tab.last();

        assert_eq!(first, last);

        Ok(())
    }

    #[test]
    fn fs_tab_finds_the_first_predicate_match_from_the_top() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b /media/disk xfs noauto 0 0
        let uuid = Tag::from_str("UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b").map(Source::from)?;
        let entry3 = FsTabEntry::builder()
            .source(uuid)
            .target("/media/disk")
            .file_system_type(FileSystem::XFS)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry4 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        // /dev/usbdisk /media/usb vfat noauto 0 0
        // UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b /media/disk xfs noauto 0 0
        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry1)?;
        fs_tab.push(entry2)?;
        fs_tab.push(entry3)?;
        fs_tab.push(entry4)?;

        // Find the first entry with an `ext4` or `xfs` file system (going from last to first).
        let element = fs_tab.find_first(|element| element.has_any_fs_type("ext4,xfs"));

        // `entry1` is the first to match...
        assert!(element.is_some());
        let element = element.unwrap();

        // ...it has an `ext4` file system type,
        let actual = element.file_system_type();
        let fs = FileSystem::Ext4;
        let expected = Some(fs);
        assert_eq!(actual, expected);

        // ...and is mounted at `/`.
        let target = Path::new("/");
        let actual = element.target();
        let expected = Some(target);
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn fs_tab_finds_the_first_predicate_match_from_the_bottom() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b /media/disk xfs noauto 0 0
        let uuid = Tag::from_str("UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b").map(Source::from)?;
        let entry3 = FsTabEntry::builder()
            .source(uuid)
            .target("/media/disk")
            .file_system_type(FileSystem::XFS)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry4 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        // /dev/usbdisk /media/usb vfat noauto 0 0
        // UUID=dd479919-1ce4-415e-9dbd-3c2ba3b42b0b /media/disk xfs noauto 0 0
        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1)?;
        fs_tab.push(entry2)?;
        fs_tab.push(entry3)?;
        fs_tab.push(entry4)?;

        // Find the first entry with an `ext4` or `xfs` file system (going from last to first).
        let element = fs_tab.find_back_first(|element| element.has_any_fs_type("ext4,xfs"));

        // `entry3` is the first to match...
        assert!(element.is_some());
        let element = element.unwrap();

        // ...it has an `xfs` file system type,
        let actual = element.file_system_type();
        let fs = FileSystem::XFS;
        let expected = Some(fs);
        assert_eq!(actual, expected);

        // ...and is mounted at `/media/disk`.
        let target = Path::new("/media/disk");
        let actual = element.target();
        let expected = Some(target);
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn fs_tab_can_can_advance_its_iterator_to_a_given_position() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1)?;
        fs_tab.push(entry2)?;
        fs_tab.push(entry3)?;

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();

        iter.advance_to(2).unwrap();
        let actual = iter.next();
        assert!(actual.is_none());

        iter.advance_to(0).unwrap();
        let actual = iter.next().unwrap().inner;
        assert_eq!(actual, entry2_inner);

        iter.advance_to(1).unwrap();
        let actual = iter.next().unwrap().inner;
        assert_eq!(actual, entry3_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_iterate_forwards_over_table_entries() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1)?;
        fs_tab.push(entry2)?;
        fs_tab.push(entry3)?;

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();
        let first_inner = iter.next().unwrap().inner;
        let second_inner = iter.next().unwrap().inner;
        let third_inner = iter.next().unwrap().inner;

        assert_eq!(first_inner, entry1_inner);
        assert_eq!(second_inner, entry2_inner);
        assert_eq!(third_inner, entry3_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_iterate_backwards_over_table_entries() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1)?;
        fs_tab.push(entry2)?;
        fs_tab.push(entry3)?;

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();
        let first_inner = iter.next_back().unwrap().inner;
        let second_inner = iter.next_back().unwrap().inner;
        let third_inner = iter.next_back().unwrap().inner;

        assert_eq!(first_inner, entry3_inner);
        assert_eq!(second_inner, entry2_inner);
        assert_eq!(third_inner, entry1_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_iterate_alternately_forwards_then_backwards_over_table_entries(
    ) -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1)?;
        fs_tab.push(entry2)?;
        fs_tab.push(entry3)?;

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();
        let first_inner = iter.next().unwrap().inner;
        let second_inner = iter.next_back().unwrap().inner;
        let third_inner = iter.next().unwrap().inner;
        let fourth = iter.next_back();
        let fifth = iter.next();

        assert_eq!(first_inner, entry1_inner);
        assert_eq!(second_inner, entry3_inner);
        assert_eq!(third_inner, entry2_inner);
        assert!(fourth.is_none());
        assert!(fifth.is_none());

        Ok(())
    }

    #[test]
    fn fs_tab_can_iterate_alternately_backwards_then_forwards_over_table_entries(
    ) -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1)?;
        fs_tab.push(entry2)?;
        fs_tab.push(entry3)?;

        assert_eq!(fs_tab.len(), 3);

        let mut iter = fs_tab.iter();
        let first_inner = iter.next_back().unwrap().inner;
        let second_inner = iter.next().unwrap().inner;
        let third_inner = iter.next_back().unwrap().inner;
        let fourth = iter.next();
        let fifth = iter.next_back();

        assert_eq!(first_inner, entry3_inner);
        assert_eq!(second_inner, entry1_inner);
        assert_eq!(third_inner, entry2_inner);
        assert!(fourth.is_none());
        assert!(fifth.is_none());

        Ok(())
    }

    #[test]
    fn fs_tab_can_index_into_a_table() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry = FsTabEntry::builder()
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

        let expected = entry.inner;

        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry)?;

        let actual = fs_tab[0].inner;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn fs_tab_can_insert_an_element_at_a_predefined_position() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut fs_tab = FsTab::new()?;

        fs_tab.push(entry1)?;
        fs_tab.push(entry2)?;
        fs_tab.insert(1, entry3)?;

        assert_eq!(fs_tab.len(), 3);
        let first_inner = fs_tab[0].inner;
        let second_inner = fs_tab[1].inner;
        let third_inner = fs_tab[2].inner;

        assert_eq!(first_inner, entry1_inner);
        assert_eq!(second_inner, entry3_inner);
        assert_eq!(third_inner, entry2_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_remove_an_element_from_a_table() -> crate::Result<()> {
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry = FsTabEntry::builder()
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

        let mut fs_tab = FsTab::new()?;
        fs_tab.push(entry)?;

        assert_eq!(fs_tab.len(), 1);

        let item = fs_tab.remove(0);

        let actual = item.tag().unwrap();
        let expected: Tag = "UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f".parse()?;
        assert_eq!(actual, expected);

        let actual = item.file_system_type().unwrap();
        let expected = FileSystem::Ext4;
        assert_eq!(actual, expected);

        let actual = item.mount_options().unwrap();
        let expected = "rw,relatime";
        assert_eq!(actual, expected);

        assert_eq!(fs_tab.is_empty(), true);

        Ok(())
    }

    #[test]
    fn fs_tab_can_transfer_an_element_between_tables_to_destination_start() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut source_table = FsTab::new()?;
        source_table.push(entry1)?;

        let mut dest_table = FsTab::new()?;
        dest_table.push(entry2)?;
        dest_table.push(entry3)?;

        // Before transfer
        assert_eq!(source_table.len(), 1);
        assert_eq!(dest_table.len(), 2);

        source_table.transfer(0, &mut dest_table, 0)?;

        // After transfer
        assert_eq!(source_table.is_empty(), true);

        assert_eq!(dest_table.len(), 3);

        let first_inner = dest_table[0].inner;
        let second_inner = dest_table[1].inner;
        let third_inner = dest_table[2].inner;

        assert_eq!(first_inner, entry1_inner);
        assert_eq!(second_inner, entry2_inner);
        assert_eq!(third_inner, entry3_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_transfer_an_element_between_tables_to_destination_middle() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut source_table = FsTab::new()?;
        source_table.push(entry1)?;

        let mut dest_table = FsTab::new()?;
        dest_table.push(entry2)?;
        dest_table.push(entry3)?;

        // Before transfer
        assert_eq!(source_table.len(), 1);
        assert_eq!(dest_table.len(), 2);

        source_table.transfer(0, &mut dest_table, 1)?;

        // After transfer
        assert_eq!(source_table.is_empty(), true);

        assert_eq!(dest_table.len(), 3);

        let first_inner = dest_table[0].inner;
        let second_inner = dest_table[1].inner;
        let third_inner = dest_table[2].inner;

        assert_eq!(first_inner, entry2_inner);
        assert_eq!(second_inner, entry1_inner);
        assert_eq!(third_inner, entry3_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_can_transfer_an_element_between_tables_to_destination_end() -> crate::Result<()> {
        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        let entry1_inner = entry1.inner;
        let entry2_inner = entry2.inner;
        let entry3_inner = entry3.inner;

        let mut source_table = FsTab::new()?;
        source_table.push(entry1)?;

        let mut dest_table = FsTab::new()?;
        dest_table.push(entry2)?;
        dest_table.push(entry3)?;

        // Before transfer
        assert_eq!(source_table.len(), 1);
        assert_eq!(dest_table.len(), 2);

        source_table.transfer(0, &mut dest_table, 2)?;

        // After transfer
        assert_eq!(source_table.is_empty(), true);

        assert_eq!(dest_table.len(), 3);

        let first_inner = dest_table[0].inner;
        let second_inner = dest_table[1].inner;
        let third_inner = dest_table[2].inner;

        assert_eq!(first_inner, entry2_inner);
        assert_eq!(second_inner, entry3_inner);
        assert_eq!(third_inner, entry1_inner);

        Ok(())
    }

    #[test]
    fn fs_tab_writes_to_a_file_stream() -> crate::Result<()> {
        let mut fs_tab = FsTab::new()?;

        // # /etc/fstab
        fs_tab.set_intro_comments("# /etc/fstab\n")?;
        // # Example from scratch
        fs_tab.append_to_intro_comments("# Example from scratch\n")?;
        //
        fs_tab.append_to_intro_comments("\n")?;

        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
        let entry1 = FsTabEntry::builder()
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

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()?;

        fs_tab.push(entry1)?;
        fs_tab.push(entry2)?;
        fs_tab.push(entry3)?;

        assert_eq!(fs_tab.len(), 3);

        let mut tmpfile: File = tempfile().unwrap();

        // Write to disk
        fs_tab.export_with_comments();
        fs_tab.write_stream(&mut tmpfile)?;

        // Seek to start
        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        // Read back
        let mut actual = String::new();
        tmpfile.read_to_string(&mut actual).unwrap();

        let expected = "# /etc/fstab\n# Example from scratch\n\nUUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1\n/dev/usbdisk /media/usb vfat noauto 0 0\nnone /tmp tmpfs nosuid,nodev 0 0\n";
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "you do not have permission to write in this file stream")]
    fn fs_tab_does_not_write_to_read_only_file_stream() {
        let mut fs_tab = FsTab::new().unwrap();

        // # /etc/fstab
        fs_tab.set_intro_comments("# /etc/fstab\n").unwrap();
        // # Example from scratch
        fs_tab
            .append_to_intro_comments("# Example from scratch\n")
            .unwrap();
        //
        fs_tab.append_to_intro_comments("\n").unwrap();

        // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
        let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f")
            .map(Source::from)
            .unwrap();
        let entry1 = FsTabEntry::builder()
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
            .build()
            .unwrap();

        // /dev/usbdisk /media/usb vfat noauto 0 0
        let block_device = BlockDevice::from_str("/dev/usbdisk")
            .map(Source::from)
            .unwrap();
        let entry2 = FsTabEntry::builder()
            .source(block_device)
            .target("/media/usb")
            .file_system_type(FileSystem::VFAT)
            .mount_options("noauto")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()
            .unwrap();

        // tmpfs /tmp tmpfs nosuid,nodev 0 0
        let entry3 = FsTabEntry::builder()
            .source(Pseudo::None.into())
            .target("/tmp")
            .file_system_type(FileSystem::Tmpfs)
            .mount_options("nosuid,nodev")
            .backup_frequency(0)
            .fsck_checking_order(0)
            .build()
            .unwrap();

        fs_tab.push(entry1).unwrap();
        fs_tab.push(entry2).unwrap();
        fs_tab.push(entry3).unwrap();

        assert_eq!(fs_tab.len(), 3);

        let tmpdir = tempdir().unwrap();
        let tmpfile_path = tmpdir.path().join("read-only-file");
        let file = File::create(&tmpfile_path).unwrap();
        drop(file);
        let mut tmpfile: File = OpenOptions::new().read(true).open(&tmpfile_path).unwrap();

        // Write to disk
        fs_tab.export_with_comments();
        fs_tab.write_stream(&mut tmpfile).unwrap();
    }
}
