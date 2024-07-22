// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::errors::MountInfoEntryError;
use crate::core::flags::MountFlag;
use crate::declare_tab_entry;
use crate::mount_info_entry_shared_methods;

declare_tab_entry!(
    MountInfoEntry,
    r"A line in `/proc/<pid>/mountinfo` (where `<pid>` is the ID of a process).

For example:
```text
26 1 8:3 / / rw,relatime - ext4 /dev/sda3 rw
```
    "
);

mount_info_entry_shared_methods!(MountInfoEntry, MountInfoEntryError);

impl MountInfoEntry {
    //---- BEGIN getters

    /// Returns the unique integer identifying a mount point. (The identifier may be reused to
    /// identify a new mount point after calling the `umount` syscall).
    pub fn mount_id(&self) -> Option<u32> {
        log::debug!("MountInfoEntry::mount_id getting mountinfo mount ID");

        let mut id = MaybeUninit::<libc::c_int>::zeroed();
        id.write(unsafe { libmount::mnt_fs_get_id(self.inner) });

        match unsafe { id.assume_init() } {
            code if code < 0 => {
                log::debug!("MountInfoEntry::mount_id failed to get mountinfo mount ID. libmount::mnt_fs_get_id returned error code: {:?}", code);

                None
            }
            id => {
                log::debug!("MountInfoEntry::mount_id value: {:?}", id);

                Some(id as u32)
            }
        }
    }

    /// Returns the unique integer identifying the parent of a mount point (or of self for the root node of the mount tree).
    pub fn parent_id(&self) -> Option<u32> {
        log::debug!("MountInfoEntry::parent_id getting mountinfo parent mount ID");

        let mut id = MaybeUninit::<libc::c_int>::zeroed();
        id.write(unsafe { libmount::mnt_fs_get_parent_id(self.inner) });

        match unsafe { id.assume_init() } {
            code if code < 0 => {
                log::debug!("MountInfoEntry::parent_id failed to get mountinfo parent mount ID. libmount::mnt_fs_get_parent_id returned error code: {:?}", code);

                None
            }
            id => {
                log::debug!("MountInfoEntry::parent_id value: {:?}", id);

                Some(id as u32)
            }
        }
    }

    /// Returns the ID of the device containing files on a file system.
    pub fn device_id(&self) -> Option<u64> {
        log::debug!(
            "MountInfoEntry::device_id getting ID of device containing files on a file system"
        );

        let mut dev_num = MaybeUninit::<libc::dev_t>::zeroed();
        dev_num.write(unsafe { libmount::mnt_fs_get_devno(self.inner) });

        match unsafe { dev_num.assume_init() } {
            0 => {
                log::debug!("MountInfoEntry::device_id failed to get ID of device containing files on a file system. libmount::mnt_fs_get_devno returned error code: 0");

                None
            }
            dev_num => {
                log::debug!("MountInfoEntry::device_id value: {:?}", dev_num);

                Some(dev_num)
            }
        }
    }

    /// Returns the ID of the device containing files on a file system as a major:minor pair.
    pub fn device_id_major_minor(&self) -> Option<(u64, u64)> {
        // From https://github.com/openbsd/src/blob/master/sys/sys/types.h#L211
        fn major(x: u64) -> u64 {
            (x >> 8) & 0xff
        }

        fn minor(x: u64) -> u64 {
            (x & 0xff) | ((x & 0xffff0000) >> 8)
        }

        self.device_id().map(|id| (major(id), minor(id)))
    }

    /// Returns the pathname of the directory a process sees as its root directory.
    pub fn root(&self) -> Option<&str> {
        log::debug!("MountInfoEntry::root getting the pathname of the directory a process sees as its root directory");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        ptr.write(unsafe { libmount::mnt_fs_get_root(self.inner) });

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("MountInfoEntry::root failed to get the pathname of the directory a process sees as its root directory. libmount::mnt_fs_get_root returned a NULL pointer");

                None
            }
            ptr => {
                let root = ffi_utils::const_char_array_to_str_ref(ptr);
                log::debug!("MountInfoEntry::root value: {:?}", root);

                root.ok()
            }
        }
    }

    /// Returns fs-independent mount options.
    pub fn fs_independent_options(&self) -> Option<&str> {
        log::debug!("MountInfoEntry::fs_independent_options getting vfs options");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        ptr.write(unsafe { libmount::mnt_fs_get_vfs_options(self.inner) });

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("MountInfoEntry::fs_independent_options failed to get vfs options. libmount::mnt_fs_get_vfs_options returned a NULL pointer");

                None
            }
            ptr => {
                let options = ffi_utils::const_char_array_to_str_ref(ptr);
                log::debug!(
                    "MountInfoEntry::fs_independent_options value: {:?}",
                    options
                );

                options.ok()
            }
        }
    }

    /// Returns **all** fs-independent mount options (including defaults).
    /// `"rw,exec,suid,dev,async,loud,nomand,atime,diratime,norelatime,nostrictatime,nolazytime,symfollow"`
    ///
    /// The last defined mount options take precedence over the defaults. For example setting
    /// `relatime` in `/etc/fstab` will cancel the effects of `norelatime`.
    pub fn fs_independent_options_full(&self) -> Option<String> {
        log::debug!("MountInfoEntry::fs_independent_options_full getting vfs options");

        let mut ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
        ptr.write(unsafe { libmount::mnt_fs_get_vfs_options_all(self.inner) });

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("MountInfoEntry::fs_independent_options_full failed to get vfs options. libmount::mnt_fs_get_vfs_options_all returned a NULL pointer");

                None
            }
            ptr => {
                let options = ffi_utils::c_char_array_to_string(ptr);
                unsafe { libc::free(ptr as *mut _) };

                log::debug!(
                    "MountInfoEntry::fs_independent_options_full value: {:?}",
                    options
                );

                Some(options)
            }
        }
    }

    /// Returns a list of comma-separated options, specific to a particular file system.
    pub fn fs_specific_options(&self) -> Option<&str> {
        log::debug!("MountInfoEntry::fs_specific_options getting mount options");

        let mut opts_ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        opts_ptr.write(unsafe { libmount::mnt_fs_get_fs_options(self.inner) });

        match unsafe { opts_ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("MountInfoEntry::fs_specific_options no options set. libmount::mnt_fs_get_fs_options returned a NULL pointer");

                None
            }
            opts_ptr => {
                let options = ffi_utils::const_char_array_to_str_ref(opts_ptr);
                log::debug!("MountInfoEntry::fs_specific_options value: {:?}", options);

                options.ok()
            }
        }
    }

    /// Returns the combination of the list of fs-independent options with the list of
    /// fs-specific options.
    pub fn fs_options(&self) -> Option<String> {
        log::debug!("MountInfoEntry::fs_options getting all file system options");

        let mut ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
        ptr.write(unsafe { libmount::mnt_fs_strdup_options(self.inner) });

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get all file system options".to_owned();
                log::debug!("MountInfoEntry::fs_options {}. libmount::mnt_fs_strdup_options returned a NULL pointer", err_msg);

                None
            }

            ptr => {
                let options = ffi_utils::c_char_array_to_string(ptr);
                unsafe { libc::free(ptr as *mut _) };
                log::debug!(
                    "MountInfoEntry::fs_options got all file system options {:?}",
                    options
                );

                Some(options)
            }
        }
    }

    /// Returns `mountinfo`'s optional fields (zero or more fields of the form *tag\[:value]*, which describe a mount point’s propagation type).
    pub fn optional_fields(&self) -> Option<&str> {
        log::debug!("MountInfoEntry::optional_fields getting mountinfo optional fields");

        let mut fields_ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        fields_ptr.write(unsafe { libmount::mnt_fs_get_optional_fields(self.inner) });

        match unsafe { fields_ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("MountInfoEntry::optional_fields failed to get mountinfo optional fields. libmount::mnt_fs_get_optional_fields returned a NULL pointer");

                None
            }
            fields_ptr => {
                let fields = ffi_utils::const_char_array_to_str_ref(fields_ptr);
                log::debug!("MountInfoEntry::optional_fields value: {:?}", fields);

                fields.ok()
            }
        }
    }

    /// Returns a mount point's propagation type set in the `mountinfo` file; might take one of the
    /// following values:
    /// - [`MountFlag::Shared`]
    /// - [`MountFlag::Private`]
    /// - [`MountFlag::Slave`]
    /// - [`MountFlag::Unbindable`]
    ///
    /// **Note:** The kernel default is [`MountFlag::Private`] which is not stored in the `mountinfo`
    /// file.
    pub fn propagation_flags(&self) -> Option<HashSet<MountFlag>> {
        log::debug!("MountInfoEntry::propagation_flags getting propagation flags");

        let mut bits = MaybeUninit::<libc::c_ulong>::zeroed();
        match unsafe { libmount::mnt_fs_get_propagation(self.inner, bits.as_mut_ptr()) } {
            0 => {
                let bits = unsafe { bits.assume_init() };
                let flags: HashSet<_> = [
                    MountFlag::Shared,
                    MountFlag::Private,
                    MountFlag::Slave,
                    MountFlag::Unbindable,
                ]
                .iter()
                .filter(|&&flag| bits & (flag as u64) != 0)
                .copied()
                .collect();

                log::debug!(
                    "MountInfoEntry::propagation_flags value: 0x{:x} -> {:?}",
                    bits,
                    flags
                );

                if flags.is_empty() {
                    None
                } else {
                    Some(flags)
                }
            }
            code => {
                log::debug!("MountInfoEntry::propagation_flags failed to get propagation flags. libmount::mnt_fs_get_propagation returned error code: {:?}", code);

                None
            }
        }
    }

    /// Returns the value of `<pid>` (the process ID) in `/proc/<pid>/mountinfo`.
    pub fn pid(&self) -> usize {
        let id = unsafe { libmount::mnt_fs_get_tid(self.inner) as usize };
        log::debug!("MountInfoEntry::pid value: {:?}", id);

        id
    }

    //---- END getters
}

impl fmt::Display for MountInfoEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output: Vec<String> = vec![];
        if let Some(mount_id) = self.mount_id() {
            output.push(mount_id.to_string());
        }

        if let Some(parent_id) = self.parent_id() {
            output.push(parent_id.to_string());
        }

        if let Some((major, minor)) = self.device_id_major_minor() {
            let dev_id = format!("{major}:{minor}");
            output.push(dev_id);
        }

        if let Some(root) = self.root() {
            output.push(root.to_string());
        }

        if let Some(path) = self.target() {
            let mount_point = format!("{}", path.display());
            output.push(mount_point);
        }

        if let Some(options) = self.fs_independent_options() {
            output.push(options.to_string());
        }

        if let Some(opt_fields) = self.optional_fields() {
            output.push(opt_fields.to_string())
        }

        output.push("-".to_string());

        if let Some(fs_type) = self.file_system_type() {
            output.push(fs_type.to_string());
        }

        if let Some(path) = self.source_path() {
            let source = format!("{}", path.display());
            output.push(source);
        }

        if let Some(fs_specific_options) = self.fs_specific_options() {
            output.push(fs_specific_options.to_string());
        }

        write!(f, "{}", output.join(" "))
    }
}
