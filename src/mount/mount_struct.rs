// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use enum_iterator::all;

// From standard library
use std::collections::HashSet;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::str::FromStr;

// From this library
use crate::core::cache::Cache;
use crate::core::device::Tag;
use crate::core::entries::{FsTabEntry, MountInfoEntry};
use crate::core::flags::MountFlag;
use crate::core::flags::UserspaceMountFlag;
use crate::core::fs::{FileLock, FileSystem};
use crate::tables::{FsTab, GcItem, MountInfo};
use crate::{owning_mut_from_ptr, owning_ref_from_ptr};

use crate::ffi_utils;
use crate::mount::ExitCode;
use crate::mount::ExitStatus;
use crate::mount::MntBuilder;
use crate::mount::MountBuilder;
use crate::mount::MountError;
use crate::mount::MountIter;
use crate::mount::MountNamespace;
use crate::mount::MountOptionsMode;
use crate::mount::MountSource;
use crate::mount::ProcessExitStatus;
use crate::mount::ReMountIter;

/// Object to mount/unmount a device.
#[derive(Debug)]
pub struct Mount {
    pub(crate) inner: *mut libmount::libmnt_context,
    pub(crate) gc: Vec<GcItem>,
}

impl Mount {
    #[doc(hidden)]
    /// Wraps a raw `libmount::mnt_context` pointer with a safe `Mount`.
    #[allow(dead_code)]
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_context) -> Mount {
        Self {
            inner: ptr,
            gc: vec![],
        }
    }

    #[doc(hidden)]
    /// Creates a new `Mount`.
    pub(crate) fn new() -> Result<Mount, MountError> {
        log::debug!("Mount::new creating a new `Mount` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_context>::zeroed();

        unsafe {
            inner.write(libmount::mnt_new_context());
        }

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `Mount` instance".to_owned();
                log::debug!(
                    "Mount::new {}. libmount::mnt_new_contex returned a NULL pointer",
                    err_msg
                );

                Err(MountError::Creation(err_msg))
            }
            inner => {
                log::debug!("Mount::new created a new `Mount` instance");
                let mount = Self::from_ptr(inner);

                Ok(mount)
            }
        }
    }

    #[doc(hidden)]
    /// Converts a function's return code to unified `libmount` exit code.
    fn return_code_to_exit_status(&self, return_code: i32) -> Result<ExitStatus, MountError> {
        log::debug!(
            "Mount::return_code_to_exit_status converting to exit status the return code: {:?}",
            return_code
        );

        const BUFFER_LENGTH: usize = 4097; // 4096 characters + `\0`
        let mut buffer: Vec<libc::c_char> = vec![0; BUFFER_LENGTH];

        let rc = unsafe {
            libmount::mnt_context_get_excode(
                self.inner,
                return_code,
                buffer.as_mut_ptr(),
                BUFFER_LENGTH,
            )
        };

        let exit_code = ExitCode::try_from(rc)?;
        let error_message = ffi_utils::c_char_array_to_string(buffer.as_ptr());
        let exit_status = ExitStatus::new(exit_code, error_message);

        log::debug!(
            "Mount::return_code_to_exit_status converted return code: {:?} to exit status {:?}",
            return_code,
            exit_status
        );

        Ok(exit_status)
    }

    //---- BEGIN setters

    #[doc(hidden)]
    /// Enables/disables path canonicalization.
    fn disable_canonicalize(
        mount: *mut libmount::libmnt_context,
        disable: bool,
    ) -> Result<(), MountError> {
        let op = if disable { 1 } else { 0 };
        let op_str = if disable {
            "disable".to_owned()
        } else {
            "enable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_disable_canonicalize(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Mount::disable_canonicalize {}d path canonicalization",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} path canonicalization", op_str);
                log::debug!("Mount::disable_canonicalize {}. libmount::mnt_context_disable_canonicalize returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Disables path canonicalization.
    pub(crate) fn disable_path_canonicalization(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::disable_path_canonicalization disabling path canonicalization");

        Self::disable_canonicalize(self.inner, true)
    }

    #[doc(hidden)]
    /// Enables path canonicalization.
    pub(crate) fn enable_path_canonicalization(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::enable_path_canonicalization enabling path canonicalization");

        Self::disable_canonicalize(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables mount helpers.
    fn disable_mnt_helpers(
        mount: *mut libmount::libmnt_context,
        disable: bool,
    ) -> Result<(), MountError> {
        let op = if disable { 1 } else { 0 };
        let op_str = if disable {
            "disable".to_owned()
        } else {
            "enable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_disable_helpers(mount, op) };

        match result {
            0 => {
                log::debug!("Mount::disable_mnt_helpers {}d mount helpers", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} mount helpers", op_str);
                log::debug!("Mount::disable_mnt_helpers {}. libmount::mnt_context_disable_helpers returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Prevents `Mount` from calling `/sbin/mount.suffix` helper functions, where *suffix* is a file system type.
    pub(crate) fn disable_helpers(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::disable_helpers disabling mount helpers");

        Self::disable_mnt_helpers(self.inner, true)
    }

    #[doc(hidden)]
    /// Allows `Mount` to call `/sbin/mount.suffix` helper functions, where *suffix* is a file system type.
    pub(crate) fn enable_helpers(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::enable_helpers enabling mount helpers");

        Self::disable_mnt_helpers(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables mount point lookup.
    fn disable_swap_match(
        mount: *mut libmount::libmnt_context,
        disable: bool,
    ) -> Result<(), MountError> {
        let op = if disable { 1 } else { 0 };
        let op_str = if disable {
            "disable".to_owned()
        } else {
            "enable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_disable_swapmatch(mount, op) };

        match result {
            0 => {
                log::debug!("Mount::disable_swap_match {}d mount point lookup", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} mount point lookup", op_str);
                log::debug!("Mount::disable_swap_match {}. libmount::mnt_context_disable_swapmatch returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Disables mount point lookup in `/etc/fstab` when either the mount `source` or `target` is
    /// not set.
    pub(crate) fn disable_mount_point_lookup(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::disable_mount_point_lookup disabling mount point lookup");

        Self::disable_swap_match(self.inner, true)
    }

    #[doc(hidden)]
    /// Enables mount point lookup in `/etc/fstab` when either the mount `source` or `target` is
    /// not set.
    pub(crate) fn enable_mount_point_lookup(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::enable_mount_point_lookup enabling mount point lookup");

        Self::disable_swap_match(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables userspace mount table updates.
    fn disable_mtab(mount: *mut libmount::libmnt_context, disable: bool) -> Result<(), MountError> {
        let op = if disable { 1 } else { 0 };
        let op_str = if disable {
            "disable".to_owned()
        } else {
            "enable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_disable_mtab(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Mount::disable_mtab {}d userspace mount table updates",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} userspace mount table updates", op_str);
                log::debug!("Mount::disable_mtab {}. libmount::mnt_context_disable_mtab returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Disables userspace mount table updates.
    pub(crate) fn do_not_update_utab(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::do_not_update_utab disabling userspace mount table updates");

        Self::disable_mtab(self.inner, true)
    }

    #[doc(hidden)]
    /// Enables userspace mount table updates.
    pub(crate) fn update_utab(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::update_utab enabling userspace mount table updates");

        Self::disable_mtab(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables skipping all mount source preparation, mount option analysis, and the actual mounting process.
    /// (see the [`mount` command](https://www.man7.org/linux/man-pages/man8/mount.8.html) man page, option `-f, --fake`)
    fn enable_fake(mount: *mut libmount::libmnt_context, enable: bool) -> Result<(), MountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_fake(mount, op) };

        match result {
            0 => {
                log::debug!("Mount::enable_fake {}d dry run", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} dry run", op_str);
                log::debug!("Mount::enable_fake {}. libmount::mnt_context_enable_fake returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Skips all mount source preparation, mount option analysis, and the actual mounting process.
    pub(crate) fn enable_dry_run(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::enable_dry_run enabling dry run");

        Self::enable_fake(self.inner, true)
    }

    #[doc(hidden)]
    pub(crate) fn disable_dry_run(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::disable_dry_run disabling dry run");

        Self::enable_fake(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables `Mount` functionality to force a device to be mounted in read-write mode.
    fn enable_force_mount_device_read_write(
        mount: *mut libmount::libmnt_context,
        enable: bool,
    ) -> Result<(), MountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_rwonly_mount(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Mount::enable_force_mount_device_read_write {}d force mount device read-write",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} force mount device read-write", op_str);
                log::debug!("Mount::enable_force_mount_device_read_write {}. libmount::mnt_context_enable_rwonly_mount returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Forces a device to be mounted in read-write mode.
    pub(crate) fn enable_force_mount_read_write(&mut self) -> Result<(), MountError> {
        log::debug!(
            "Mount::enable_force_mount_read_write enabling force mount device in read-write mode"
        );

        Self::enable_force_mount_device_read_write(self.inner, true)
    }

    #[doc(hidden)]
    /// Sets a device to be mounted in read-only mode.
    pub(crate) fn disable_force_mount_read_write(&mut self) -> Result<(), MountError> {
        log::debug!(
            "Mount::disable_force_mount_read_write disabling force mount device in read-write mode"
        );

        Self::enable_force_mount_device_read_write(self.inner, false)
    }

    #[cfg(mount = "v2_39")]
    #[doc(hidden)]
    /// Enables/disables ignore `autofs` mount table entries.
    fn ignore_autofs(mount: *mut libmount::libmnt_context, ignore: bool) -> Result<(), MountError> {
        let op = if ignore { 1 } else { 0 };
        let op_str = if ignore {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_noautofs(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Mount::ignore_autofs {}d ignore `autofs` mount table entries",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} ignore `autofs` mount table entries", op_str);
                log::debug!("Mount::ignore_autofs {}. libmount::mnt_context_enable_noautofs returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[cfg(mount = "v2_39")]
    #[doc(hidden)]
    /// Enables `Mount` to ignore `autofs` mount table entries.
    pub(crate) fn enable_ignore_autofs(&mut self) -> Result<(), MountError> {
        log::debug!(
            "Mount::enable_ignore_autofs enabling `Mount` functionality to ignore `autofs` mount table entries"
        );

        Self::ignore_autofs(self.inner, true)
    }

    #[cfg(mount = "v2_39")]
    #[doc(hidden)]
    /// Disables `Mount` to ignore `autofs` mount table entries.
    pub(crate) fn disable_ignore_autofs(&mut self) -> Result<(), MountError> {
        log::debug!(
            "Mount::enable_ignore_autofs disabling `Mount` functionality to ignore `autofs` mount table entries"
        );

        Self::ignore_autofs(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables `Mount` functionality to ignore mount options not supported by a file
    /// system.
    fn ignore_unsupported_mount_options(
        mount: *mut libmount::libmnt_context,
        ignore: bool,
    ) -> Result<(), MountError> {
        let op = if ignore { 1 } else { 0 };
        let op_str = if ignore {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_sloppy(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Mount::ignore_unsupported_mount_options {}d ignore unsupported mount options",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} ignore unsupported mount options", op_str);
                log::debug!("Mount::ignore_unsupported_mount_options {}. libmount::mnt_context_enable_sloppy returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Enables `Mount` to ignore `autofs` mount table entries.
    pub(crate) fn enable_ignore_unsupported_mount_options(&mut self) -> Result<(), MountError> {
        log::debug!(
            "Mount::enable_ignore_unsupported_mount_options enabling `Mount` functionality to ignore mount options not supported by a file system"
        );

        Self::ignore_unsupported_mount_options(self.inner, true)
    }

    #[doc(hidden)]
    /// Disables `Mount` to ignore `autofs` mount table entries.
    pub(crate) fn disable_ignore_unsupported_mount_options(&mut self) -> Result<(), MountError> {
        log::debug!(
            "Mount::disable_ignore_unsupported_mount_options disabling `Mount` functionality to ignore mount options not supported by a file system"
        );

        Self::ignore_unsupported_mount_options(self.inner, false)
    }

    #[doc(hidden)]
    /// Disables all of `libmount`'s security protocols using this `Mount` as if its user has root
    /// permissions.
    pub(crate) fn force_user_mount(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::force_user_mount removing all safety checks");

        let result = unsafe { libmount::mnt_context_force_unrestricted(self.inner) };

        match result {
            0 => {
                log::debug!("Mount::force_user_mount removed all safety checks");

                Ok(())
            }
            code => {
                let err_msg = "failed to remove all safety checks".to_owned();
                log::debug!("Mount::force_user_mount {}. libmount::mnt_context_force_unrestricted returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Enables/disables verbose output.
    fn enable_verbose(
        mount: *mut libmount::libmnt_context,
        enable: bool,
    ) -> Result<(), MountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_verbose(mount, op) };

        match result {
            0 => {
                log::debug!("Mount::enable_verbose {}d verbose output", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} verbose output", op_str);
                log::debug!("Mount::enable_verbose {}. libmount::mnt_context_enable_verbose returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Enables `Mount` to check that a device is not already mounted before mounting it.
    pub(crate) fn enable_verbose_output(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::enable_verbose_output enabling verbose output");

        Self::enable_verbose(self.inner, true)
    }

    #[doc(hidden)]
    /// Disables `Mount` to check that a device is not already mounted before mounting it.
    pub(crate) fn disable_verbose_output(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::disable_verbose_output disabling verbose output");

        Self::enable_verbose(self.inner, false)
    }

    #[cfg(mount = "v2_39")]
    #[doc(hidden)]
    /// Enables/disables `Mount` functionality to check that a device is not already mounted before mounting it.
    fn enable_only_once(
        mount: *mut libmount::libmnt_context,
        enable: bool,
    ) -> Result<(), MountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_onlyonce(mount, op) };

        match result {
            0 => {
                log::debug!("Mount::enable_only_once {}d mount only once", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} mount only once", op_str);
                log::debug!("Mount::enable_only_once {}. libmount::mnt_context_enable_onlyonce returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[cfg(mount = "v2_39")]
    #[doc(hidden)]
    /// Enables `Mount` to check that a device is not already mounted before mounting it.
    pub(crate) fn enable_mount_only_once(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::enable_mount_only_once enabling check if device not already mounted");

        Self::enable_only_once(self.inner, true)
    }

    #[cfg(mount = "v2_39")]
    #[doc(hidden)]
    /// Disables `Mount` to check that a device is not already mounted before mounting it.
    pub(crate) fn disable_mount_only_once(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::disable_mount_only_once disabling check if device not already mounted");

        Self::enable_only_once(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables parallel mounts.
    fn enable_fork(mount: *mut libmount::libmnt_context, enable: bool) -> Result<(), MountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_fork(mount, op) };

        match result {
            0 => {
                log::debug!("Mount::enable_fork {}d parallel mounts", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} parallel mounts", op_str);
                log::debug!("Mount::enable_fork {}. libmount::mnt_context_enable_fork returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Enables parallel mounts.
    pub(crate) fn enable_parallel_mount(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::enable_parallel_mount enabling parallel mounts");

        Self::enable_fork(self.inner, true)
    }

    #[doc(hidden)]
    /// Disables parallel mounts.
    pub(crate) fn disable_parallel_mount(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::disable_parallel_mount disabling parallel mounts");

        Self::enable_fork(self.inner, false)
    }

    #[doc(hidden)]
    pub(crate) fn set_cache(&mut self, cache: Cache) -> Result<(), MountError> {
        log::debug!("Mount::set_cache overriding internal cache with custom instance");

        let result = unsafe { libmount::mnt_context_set_cache(self.inner, cache.inner) };

        match result {
            0 => {
                log::debug!("Mount::set_cache overrode internal cache with custom table");

                Ok(())
            }
            code => {
                let err_msg = "failed to override internal cache with custom instance".to_owned();
                log::debug!("Mount::set_cache {}. libmount::mnt_context_set_cache returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Overrides this `Mount`'s internal `fstab` with a custom table.
    pub(crate) fn set_fstab(&mut self, table: FsTab) -> Result<(), MountError> {
        log::debug!("Mount::set_fstab overriding internal fstab with custom table");

        let result = unsafe { libmount::mnt_context_set_fstab(self.inner, table.inner) };

        match result {
            0 => {
                log::debug!("Mount::set_fstab overrode internal fstab with custom table");

                Ok(())
            }
            code => {
                let err_msg = "failed to override internal fstab with custom table".to_owned();
                log::debug!("Mount::set_fstab {}. libmount::mnt_context_set_fstab returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets a list of file systems.
    pub(crate) fn set_file_systems_filter<T>(&mut self, fs_list: T) -> Result<(), MountError>
    where
        T: AsRef<str>,
    {
        let fs_list = fs_list.as_ref();
        let fs_list_cstr = ffi_utils::as_ref_str_to_c_string(fs_list)?;
        log::debug!(
            "Mount::set_file_systems_filter setting the list of file systems: {:?}",
            fs_list
        );

        let result =
            unsafe { libmount::mnt_context_set_fstype_pattern(self.inner, fs_list_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "Mount::set_file_systems_filter set the list of file systems: {:?}",
                    fs_list
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set file systems list: {:?}", fs_list);
                log::debug!("Mount::set_file_systems_filter {}. libmount::mnt_context_set_fstype_pattern returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    // #[doc(hidden)]
    // /// Overrides this `Mount`'s internal mount table entry with a custom one.
    // pub(crate) fn set_table_entry(&mut self, entry: MountTableEntry) -> Result<(), MountError> {
    //     log::debug!("Mount::set_table_entry overriding internal table entry");

    //     let result = unsafe { libmount::mnt_context_set_fs(self.inner, entry.ptr) };

    //     match result {
    //         0 => {
    //             log::debug!("Mount::set_table_entry overrode internal table entry");

    //             Ok(())
    //         }
    //         code => {
    //             let err_msg = "failed to overrride internal table entry".to_owned();
    //             log::debug!("Mount::set_table_entry {}. libmount::mnt_context_set_fs returned error code: {:?}", err_msg, code);

    //             Err(MountError::Config(err_msg))
    //         }
    //     }
    // }

    #[doc(hidden)]
    /// Overrides the data argument of the [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html).
    pub(crate) fn set_mount_data(&mut self, data: NonNull<libc::c_void>) -> Result<(), MountError> {
        log::debug!("Mount::set_mount_data overriding data argument of mount syscall");

        let result = unsafe { libmount::mnt_context_set_mountdata(self.inner, data.as_ptr()) };

        match result {
            0 => {
                log::debug!("Mount::set_mount_data overrode data argument of mount syscall");

                Ok(())
            }
            code => {
                let err_msg = "failed to override data argument of mount syscall".to_owned();
                log::debug!("Mount::set_mount_data {}. libmount::mnt_context_set_mountdata returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets mount options.
    pub(crate) fn set_mount_options<T>(&mut self, options_list: T) -> Result<(), MountError>
    where
        T: AsRef<str>,
    {
        let options_list = options_list.as_ref();
        let options_list_cstr = ffi_utils::as_ref_str_to_c_string(options_list)?;
        log::debug!(
            "Mount::set_mount_options setting mount options: {:?}",
            options_list
        );

        let result =
            unsafe { libmount::mnt_context_set_options(self.inner, options_list_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "Mount::set_mount_options set mount options: {:?}",
                    options_list
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount options: {:?}", options_list);
                log::debug!("Mount::set_mount_options {}. libmount::mnt_context_set_options returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets mount options mode.
    pub(crate) fn set_mount_options_mode(
        &mut self,
        mode: Vec<MountOptionsMode>,
    ) -> Result<(), MountError> {
        log::debug!(
            "Mount::set_mount_options_mode setting mount options mode: {:?}",
            mode
        );

        let options_mode = mode.iter().fold(0, |acc, &m| acc | (m as i32));

        let result = unsafe { libmount::mnt_context_set_optsmode(self.inner, options_mode) };

        match result {
            0 => {
                log::debug!(
                    "Mount::set_mount_options_mode set mount options mode: {:?}",
                    mode
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount options mode: {:?}", mode);
                log::debug!("Mount::set_mount_options_mode {}. libmount::mnt_context_set_optsmode returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets the pattern of mount options to use as filter when mounting devices.
    pub(crate) fn set_mount_options_filter<T>(&mut self, options_list: T) -> Result<(), MountError>
    where
        T: AsRef<str>,
    {
        let options_list = options_list.as_ref();
        let options_list_cstr = ffi_utils::as_ref_str_to_c_string(options_list)?;
        log::debug!(
            "Mount::set_mount_options_filter setting mount options filter: {:?}",
            options_list
        );

        let result = unsafe {
            libmount::mnt_context_set_options_pattern(self.inner, options_list_cstr.as_ptr())
        };

        match result {
            0 => {
                log::debug!(
                    "Mount::set_mount_options_filter set mount options filter: {:?}",
                    options_list
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount options filter: {:?}", options_list);
                log::debug!("Mount::set_mount_options_filter {}. libmount::mnt_context_set_options_pattern returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Set the `Mount`'s source.
    fn set_source(mount: *mut libmount::libmnt_context, source: CString) -> Result<(), MountError> {
        let result = unsafe { libmount::mnt_context_set_source(mount, source.as_ptr()) };

        match result {
            0 => {
                log::debug!("Mount::set_source mount source set");

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount source: {:?}", source);
                log::debug!("Mount::set_source {}. libmount::mnt_context_set_source returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets the location/ID of the device to mount.
    ///
    /// A source can take any of the following forms:
    /// - block device path (e.g. `/dev/sda1`),
    /// - network ID:
    ///     - Samba: `smb://ip-address-or-hostname/shared-dir`,
    ///     - NFS: `hostname:/shared-dir`  (e.g. knuth.cwi.nl:/dir)
    ///     - SSHFS: `[user@]ip-address-or-hostname:[/shared-dir]` elements in brackets are optional (e.g.
    ///     tux@192.168.0.1:/share)
    ///
    /// - label:
    ///     - `UUID=uuid`,
    ///     - `LABEL=label`,
    ///     - `PARTLABEL=label`,
    ///     - `PARTUUID=uuid`,
    ///     - `ID=id`.
    pub(crate) fn set_mount_source(&mut self, source: MountSource) -> Result<(), MountError> {
        let source = source.to_string();
        let source_cstr = ffi_utils::as_ref_path_to_c_string(&source)?;
        log::debug!("Mount::set_mount_source setting mount source: {:?}", source);

        Self::set_source(self.inner, source_cstr)
    }

    #[doc(hidden)]
    /// Sets this `Mount`'s mount point.
    pub(crate) fn set_mount_target<T>(&mut self, target: T) -> Result<(), MountError>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        let target_cstr = ffi_utils::as_ref_path_to_c_string(target)?;
        log::debug!(
            "Mount::set_mount_target setting mount target to: {:?}",
            target
        );

        let result = unsafe { libmount::mnt_context_set_target(self.inner, target_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!("Mount::set_mount_target set mount target to: {:?}", target);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount target to: {:?}", target);
                log::debug!("Mount::set_mount_target {}. libmount::mnt_context_set_target returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets the namespace of this `Mount`'s mount point.
    pub(crate) fn set_mount_target_namespace<T>(&mut self, path: T) -> Result<(), MountError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;
        log::debug!(
            "Mount::set_mount_target_namespace setting mount target namespace: {:?}",
            path
        );

        let result = unsafe { libmount::mnt_context_set_target_ns(self.inner, path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "Mount::set_mount_target_namespace set mount target namespace: {:?}",
                    path
                );

                Ok(())
            }
            code if code == -libc::ENOSYS => {
                let err_msg = "`libmount` was compiled without namespace support".to_owned();
                log::debug!("Mount::set_mount_target_namespace {}. libmount::mnt_context_set_target returned error code: {:?}", err_msg, code);

                Err(MountError::NoNamespaceSupport(err_msg))
            }
            code => {
                let err_msg = format!("failed to set mount target namespace: {:?}", path);
                log::debug!("Mount::set_mount_target_namespace {}. libmount::mnt_context_set_target_ns returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets the prefix of this `Mount`'s mount point.
    pub(crate) fn set_mount_target_prefix<T>(&mut self, path: T) -> Result<(), MountError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;
        log::debug!(
            "Mount::set_mount_target_prefix setting mount target prefix: {:?}",
            path
        );

        let result =
            unsafe { libmount::mnt_context_set_target_prefix(self.inner, path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "Mount::set_mount_target_prefix set mount target prefix: {:?}",
                    path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount target prefix: {:?}", path);
                log::debug!("Mount::set_mount_target_prefix {}. libmount::mnt_context_set_target_prefix returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets userspace mount flags.
    pub(crate) fn set_userspace_mount_flags(
        &mut self,
        flags: Vec<UserspaceMountFlag>,
    ) -> Result<(), MountError> {
        log::debug!(
            "Mount::set_userspace_mount_flags setting userspace mount flags: {:?}",
            flags
        );

        let bits = flags.iter().fold(0, |acc, &flag| acc | (flag as u64));

        let result = unsafe { libmount::mnt_context_set_user_mflags(self.inner, bits) };

        match result {
            0 => {
                log::debug!(
                    "Mount::set_userspace_mount_flags set userspace mount flags: {:?}",
                    flags
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set userspace mount flags: {:?}", flags);
                log::debug!("Mount::set_userspace_mount_flags {}. libmount::mnt_context_set_user_mflags returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }
    //---- END setters

    //---- BEGIN mutators
    /// Creates a [`MountBuilder`] to configure and construct a new `Mount`
    /// instance.
    ///
    /// Call the `MountBuilder`'s [`build()`](MountBuilder::build) method to
    /// construct a new `Mount` instance.
    pub fn builder() -> MountBuilder {
        log::debug!("Mount::builder creating new `MountBuilder` instance");
        MntBuilder::builder()
    }

    /// Sets this `Mount`'s mount flags.
    pub fn set_mount_flags<T>(&mut self, flags: T) -> Result<(), MountError>
    where
        T: AsRef<[MountFlag]>,
    {
        let flags = flags.as_ref();
        log::debug!("Mount::set_mount_flags setting mount flags: {:?}", flags);

        let bits = flags.iter().fold(0, |acc, &flag| acc | (flag as u64));

        let result = unsafe { libmount::mnt_context_set_mflags(self.inner, bits) };

        match result {
            0 => {
                log::debug!("Mount::set_mount_flags set mount flags: {:?}", flags);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount flags: {:?}", flags);
                log::debug!("Mount::set_mount_flags {}. libmount::mnt_context_set_mflags returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    /// Sets this `Mount`'s file system type.
    pub fn set_file_system_type(&mut self, fs_type: FileSystem) -> Result<(), MountError> {
        log::debug!(
            "Mount::set_file_system_type setting file system type: {:?}",
            fs_type
        );

        let fs_type_cstr = ffi_utils::as_ref_str_to_c_string(&fs_type)?;

        let result = unsafe { libmount::mnt_context_set_fstype(self.inner, fs_type_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "Mount::set_file_system_type set file system type: {:?}",
                    fs_type
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set file system type: {:?}", fs_type);
                log::debug!("Mount::set_file_system_type {}. libmount::mnt_context_set_fstype returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    /// Appends a comma-delimited list of mount options.
    pub fn append_mount_options<T>(&mut self, option_list: T) -> Result<(), MountError>
    where
        T: AsRef<str>,
    {
        let option_list = option_list.as_ref();
        let opts_cstr = ffi_utils::as_ref_str_to_c_string(option_list)?;
        log::debug!(
            "Mount::append_mount_options appending mount options: {:?}",
            option_list
        );

        let result =
            unsafe { libmount::mnt_context_append_options(self.inner, opts_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "Mount::append_mount_options appended mount options: {:?}",
                    option_list
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to append mount options: {:?}", option_list);
                log::debug!("Mount::append_mount_options {}. libmount::mnt_context_append_options returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    /// Mounts a device using the [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html) and/or
    /// [`mount` helpers](https://www.man7.org/linux/man-pages/man8/mount.8.html#EXTERNAL_HELPERS).
    ///
    /// Equivalent to running the following functions in succession:
    /// - [`Mount::prepare_mount`]
    /// - [`Mount::call_mount_syscall`]
    /// - [`Mount::finalize_mount`]
    pub fn mount_device(&mut self) -> Result<ExitStatus, MountError> {
        log::debug!("Mount::mount_device mounting device");

        let return_code = unsafe { libmount::mnt_context_mount(self.inner) };
        self.return_code_to_exit_status(return_code)
    }

    /// Validates this `Mount`'s parameters before it tries to mount a device.
    ///
    /// **Note:** you do not need to call this method if you are using [`Mount::mount_device`], it
    /// will take care of parameter validation.
    pub fn prepare_mount(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::prepare_mount preparing for mount");

        let result = unsafe { libmount::mnt_context_prepare_mount(self.inner) };

        match result {
            0 => {
                log::debug!("Mount::prepare_mount preparation successful");

                Ok(())
            }
            code => {
                let err_msg = "failed to prepare for device mount".to_owned();
                log::debug!("Mount::prepare_mount {}. libmount::mnt_context_prepare_mount returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    /// Runs the [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html) and/or
    /// [mount helpers](https://www.man7.org/linux/man-pages/man8/mount.8.html#EXTERNAL_HELPERS).
    ///
    /// If you want to call this function again with different mount flags and/or file system type:
    /// - modify the corresponding parameters,
    /// - call [`Mount::reset_syscall_exit_status`],
    /// - then try again.
    ///
    /// **Note:** you do not need to call this method if you are using [`Mount::mount_device`], it
    /// will take care of everything for you.
    pub fn call_mount_syscall(&mut self) -> Result<ExitStatus, MountError> {
        log::debug!("Mount::call_mount_syscall mounting device");

        let return_code = unsafe { libmount::mnt_context_do_mount(self.inner) };
        self.return_code_to_exit_status(return_code)
    }

    /// Updates the system's mount tables to take the last modifications into account. You should
    /// call this function after invoking [`Mount::call_mount_syscall`].
    ///
    /// **Note:** you do not need to call this method if you are using [`Mount::mount_device`], it
    /// will take care of finalizing the mount.
    pub fn finalize_mount(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::finalize_mount finalizing mount");

        let result = unsafe { libmount::mnt_context_finalize_mount(self.inner) };

        match result {
            0 => {
                log::debug!("Mount::finalize_mount finalized mount");

                Ok(())
            }
            code => {
                let err_msg = "failed to finalize device mount".to_owned();
                log::debug!("Mount::finalize_mount {}. libmount::mnt_context_finalize_mount returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Searches `/proc/self/mountinfo` for an entry with a field matching the given `target`.
    fn find_mounted_entry<'a>(
        mount: &mut Self,
        target: CString,
    ) -> Result<&'a MountInfoEntry, MountError> {
        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        let result = unsafe {
            libmount::mnt_context_find_umount_fs(mount.inner, target.as_ptr(), ptr.as_mut_ptr())
        };

        match result {
            0 => {
                log::debug!(
                    "Mount::find_mounted_entry found mount table entry matching {:?}",
                    target
                );
                let ptr = unsafe { ptr.assume_init() };
                let entry = owning_ref_from_ptr!(mount, MountInfoEntry, ptr);

                Ok(entry)
            }
            code => {
                let err_msg = format!("failed to find mount table entry matching {:?}", target);
                log::debug!("Mount::find_mounted_entry {}. libmount::mnt_context_find_umount_fs returned error code: {:?}", err_msg, code);

                Err(MountError::Action(err_msg))
            }
        }
    }

    /// Searches `/proc/self/mountinfo` for an entry with a source field matching the given `source`.
    pub fn find_entry_matching_source<T>(
        &mut self,
        source: T,
    ) -> Result<&MountInfoEntry, MountError>
    where
        T: AsRef<Path>,
    {
        let source = source.as_ref();
        let source_cstr = ffi_utils::as_ref_path_to_c_string(source)?;
        log::debug!(
            "Mount::find_entry_matching_source finding mounted table entry matching source: {:?}",
            source
        );

        Self::find_mounted_entry(self, source_cstr)
    }

    /// Searches `/proc/self/mountinfo` for an entry with a target field matching the given `target`.
    pub fn find_entry_matching_target<T>(
        &mut self,
        target: T,
    ) -> Result<&MountInfoEntry, MountError>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        let target_cstr = ffi_utils::as_ref_path_to_c_string(target)?;
        log::debug!(
            "Mount::find_entry_matching_target finding mounted table entry matching target: {:?}",
            target
        );

        Self::find_mounted_entry(self, target_cstr)
    }

    /// Searches `/proc/self/mountinfo` for an entry with a tag field matching the given `tag`.
    pub fn find_entry_matching_tag(&mut self, tag: &Tag) -> Result<&MountInfoEntry, MountError> {
        let tag_cstr = ffi_utils::as_ref_path_to_c_string(tag.to_string())?;
        log::debug!(
            "Mount::find_entry_matching_tag finding mounted table entry matching tag: {:?}",
            tag
        );

        Self::find_mounted_entry(self, tag_cstr)
    }

    // /// Parses a file into a [`MountTable`]. This `Mount` will use its own cache, and parser error callback function during the conversion.
    // pub fn parse_mount_table<T>(&self, file: T) -> Result<MountTable, MountError>
    // where
    //     T: AsRef<Path>,
    // {
    //     let file = file.as_ref();
    //     let file_cstr = ffi_utils::as_ref_path_to_c_string(file)?;
    //     log::debug!(
    //         "Mount::parse_mount_table parsing mount table from {:?}",
    //         file
    //     );

    //     let mut table = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();

    //     let result = unsafe {
    //         libmount::mnt_context_get_table(self.inner, file_cstr.as_ptr(), table.as_mut_ptr())
    //     };

    //     match result {
    //         0 => {
    //             log::debug!(
    //                 "Mount::parse_mount_table parsed mount table from {:?}",
    //                 file
    //             );
    //             let ptr = unsafe { table.assume_init() };
    //             let table = MountTable::from(ptr);

    //             Ok(table)
    //         }
    //         code => {
    //             let err_msg = format!("failed to parse mount table from {:?}", file);
    //             log::debug!("Mount::parse_mount_table {}. libmount::mnt_context_get_table returned error code: {:?}", err_msg, code);

    //             Err(MountError::Parse(err_msg))
    //         }
    //     }
    // }

    /// Sets `mount`'s syscall exit status if the function was called outside of `libmount`.
    ///
    /// The `exit_status` should be `0` on success, and a negative number on error (e.g. `-errno`).
    pub fn set_syscall_exit_status(&mut self, exit_status: i32) -> Result<(), MountError> {
        log::debug!(
            "Mount::set_syscall_exit_status setting mount syscall exit status to {:?}",
            exit_status
        );

        let result = unsafe { libmount::mnt_context_set_syscall_status(self.inner, exit_status) };

        match result {
            0 => {
                log::debug!(
                    "Mount::set_syscall_exit_status set mount syscall exit status to {:?}",
                    exit_status
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "ailed to set mount syscall exit status to {:?}",
                    exit_status
                );
                log::debug!("Mount::set_syscall_exit_status {}. libmount::mnt_context_set_syscall_status returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    /// Resets `mount` exit status so that `mount` methods can be called again.
    pub fn reset_syscall_exit_status(&mut self) -> Result<(), MountError> {
        log::debug!("Mount::reset_syscall_exit_status resetting syscall exit status");

        let result = unsafe { libmount::mnt_context_reset_status(self.inner) };

        match result {
            0 => {
                log::debug!("Mount::reset_syscall_exit_status reset syscall exit status");

                Ok(())
            }
            code => {
                let err_msg = "failed to reset syscall exit status".to_owned();
                log::debug!("Mount::reset_syscall_exit_status {}. libmount::mnt_context_reset_status returned error code: {:?}", err_msg, code);

                Err(MountError::Config(err_msg))
            }
        }
    }

    /// Switches to the provided `namespace`, and returns the namespace used previously.
    pub fn switch_to_namespace(&mut self, namespace: MountNamespace) -> Option<MountNamespace> {
        log::debug!("Mount::switch_to_namespace switching namespace");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_ns>::zeroed();
        unsafe {
            ptr.write(libmount::mnt_context_switch_ns(self.inner, namespace.ptr));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no prior namespace";
                log::debug!("Mount::switch_to_namespace {}. libmount::mnt_context_switch_ns returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::switch_to_namespace switched namespace");
                let namespace = MountNamespace::from_raw_parts(ptr, self);

                Some(namespace)
            }
        }
    }

    /// Switches to the namespace at creation, and returns the replacement namespace used up to this point.
    pub fn switch_to_original_namespace(&mut self) -> Option<MountNamespace> {
        log::debug!("Mount::switch_to_original_namespace switching to original namespace");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_ns>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_switch_origin_ns(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no prior namespace";
                log::debug!("Mount::switch_to_original_namespace {}. libmount::mnt_context_switch_origin_ns returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::switch_to_original_namespace switched to original namespace");
                let namespace = MountNamespace::from_raw_parts(ptr, self);

                Some(namespace)
            }
        }
    }

    /// Switches to the target's namespace, and returns the namespace used previously.
    pub fn switch_to_target_namespace(&mut self) -> Option<MountNamespace> {
        log::debug!("Mount::switch_to_target_namespace switching to target namespace");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_ns>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_switch_target_ns(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no prior namespace";
                log::debug!("Mount::switch_to_target_namespace {}. libmount::mnt_context_switch_target_ns returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::switch_to_target_namespace switched to target namespace");
                let namespace = MountNamespace::from_raw_parts(ptr, self);

                Some(namespace)
            }
        }
    }

    /// Waits on parallel mount child processes.
    pub fn wait_on_children(&mut self) -> ProcessExitStatus {
        log::debug!("Mount::wait_on_children waiting on child processes");

        let mut children = 0i32;
        let mut errors = 0i32;

        unsafe {
            libmount::mnt_context_wait_for_children(
                self.inner,
                &mut children as *mut _,
                &mut errors as *mut _,
            );
        }

        ProcessExitStatus::new(children as usize, errors as usize)
    }
    //---- END mutators

    //---- BEGIN getters
    /// Returns the identifier of the device to mount, or `None` if it was not provided.
    pub fn source(&self) -> Option<String> {
        log::debug!("Mount::source getting identifier of device to mount");

        let mut identifier = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            identifier.write(libmount::mnt_context_get_source(self.inner));
        }

        match unsafe { identifier.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get identifier of device to mount";
                log::debug!(
                    "Mount::source {}. libmount::mnt_context_get_source returned a NULL pointer",
                    err_msg
                );

                None
            }
            ptr => {
                log::debug!("Mount::source got identifier of device to mount");
                let source = ffi_utils::c_char_array_to_string(ptr);

                Some(source)
            }
        }
    }

    /// Returns the configured device mount point, or `None` if it was not provided.
    pub fn target(&self) -> Option<PathBuf> {
        log::debug!("Mount::target getting mount point");

        let mut mount_point = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            mount_point.write(libmount::mnt_context_get_target(self.inner));
        }

        match unsafe { mount_point.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get mount point";
                log::debug!(
                    "Mount::target {}. libmount::mnt_context_get_target returned a NULL pointer",
                    err_msg
                );

                None
            }
            ptr => {
                log::debug!("Mount::target got mount point");
                let target = ffi_utils::const_c_char_array_to_path_buf(ptr);

                Some(target)
            }
        }
    }

    /// Returns the mount point's [`MountNamespace`], or `None` if it is
    /// not set.
    pub fn target_namespace(&self) -> Option<MountNamespace> {
        log::debug!("Mount::target_namespace getting mount point's namespace");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_ns>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_target_ns(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no mount point namespace";
                log::debug!("Mount::target_namespace {}. libmount::mnt_context_get_target_ns returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::target_namespace got mount point's namespace");
                let ns = MountNamespace::from_raw_parts(ptr, self);

                Some(ns)
            }
        }
    }

    /// Returns the prefix of the configured device's mount point.
    pub fn target_prefix(&self) -> Option<PathBuf> {
        log::debug!("Mount::target_prefix getting mount point prefix");

        let mut mount_point = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            mount_point.write(libmount::mnt_context_get_target_prefix(self.inner));
        }

        match unsafe { mount_point.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get mount point prefix ";
                log::debug!("Mount::target_prefix {}. libmount::mnt_context_get_target_prefix returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::target_prefix got mount point prefix ");
                let target_prefix = ffi_utils::const_c_char_array_to_path_buf(ptr);

                Some(target_prefix)
            }
        }
    }

    /// Returns a reference to the internal table entry describing this `Mount`, or `None` if it is
    /// not set.
    pub fn internal_table_entry(&self) -> Option<&FsTabEntry> {
        log::debug!("Mount::internal_table_entry getting reference to internal mount table entry");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_fs(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get reference to internal mount table entry";
                log::debug!(
                    "Mount::internal_table_entry {}. libmount::mnt_context_get_fs returned a NULL pointer",
                    err_msg
                );

                None
            }
            ptr => {
                log::debug!(
                    "Mount::internal_table_entry got reference to internal mount table entry"
                );
                let entry = owning_ref_from_ptr!(self, FsTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Returns the private user data associated with this `Mount`'s internal mount table entry.
    pub fn internal_table_entry_user_data(&self) -> Option<NonNull<libc::c_void>> {
        log::debug!("Mount::internal_table_entry_user_data getting private user data associated with internal table entry");

        let mut ptr = MaybeUninit::<*mut libc::c_void>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_fs_userdata(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg =
                    "failed to get private user data associated with internal table entry";
                log::debug!("Mount::internal_table_entry_user_data {}. libmount::mnt_context_get_fs_userdata returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::internal_table_entry_user_data got private user data associated with internal table entry");

                NonNull::new(ptr)
            }
        }
    }

    /// Returns a reference to a [`FileLock`] on the userspace mount table. In most cases,
    /// applications using `libmount`  do not need to worry about managing the lock on `utab`.
    /// However, in rare instances, they have to be able unlock `utab` when interrupted by a Linux
    /// signal or ignore them when the lock is active.
    ///
    /// **Note:** a locked [`FileLock`] ignores by default all signals, except `SIGALARM` and `SIGTRAP` for
    /// `utab` updates.
    pub fn utab_file_lock(&mut self) -> Option<&mut FileLock> {
        log::debug!("Mount::utab_file_lock getting `utab` file lock");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_lock>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_lock(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no `utab` file lock";
                log::debug!("Mount::utab_file_lock {}. libmount::mnt_context_get_lock returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::utab_file_lock got `utab` file lock");
                let lock = owning_mut_from_ptr!(self, FileLock, ptr);

                Some(lock)
            }
        }
    }

    /// Returns the mount options set by [`MountBuilder::mount_options`],
    /// [`MountBuilder::mount_flags`], and [`Mount::append_mount_options`].
    ///
    /// **Note:** before `v2.39` this function ignored options specified by
    /// [`MountBuilder::mount_flags`] before calling [`Mount::prepare_mount`]. Now it always
    /// returns all mount options.
    pub fn mount_options(&self) -> Option<String> {
        log::debug!("Mount::mount_options getting mount options");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_options(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get mount options";
                log::debug!("Mount::mount_options {}. libmount::mnt_context_get_options returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::mount_options got mount options");
                let options = ffi_utils::c_char_array_to_string(ptr);

                Some(options)
            }
        }
    }

    /// Returns the mode of `fstab` mount options.
    pub fn mount_options_mode(&self) -> Option<MountOptionsMode> {
        log::debug!("Mount::mount_options_mode getting mount options mode");

        let result = unsafe { libmount::mnt_context_get_optsmode(self.inner) };

        match result {
            0 => {
                let err_msg = "no mount options mode set";
                log::debug!("Mount::mount_options_mode {}. libmount::mnt_context_get_optsmode returned error code: 0", err_msg);

                None
            }
            mode => {
                log::debug!("Mount::mount_options_mode got mount options mode");

                MountOptionsMode::try_from(mode).ok()
            }
        }
    }

    /// Returns the set  of mount flags set during configuration, or `None` if they were
    /// not provided.
    pub fn mount_flags(&self) -> Option<HashSet<MountFlag>> {
        log::debug!("Mount::mount_flags getting mount flags");

        let mut bits = MaybeUninit::<libc::c_ulong>::zeroed();

        let result = unsafe { libmount::mnt_context_get_mflags(self.inner, bits.as_mut_ptr()) };

        match result {
            0 => {
                log::debug!("Mount::mount_flags got mount flags");
                let bits = unsafe { bits.assume_init() };
                let flags: HashSet<_> = all::<MountFlag>()
                    .filter(|&flag| (flag as u64) & bits != 0)
                    .collect();

                Some(flags)
            }
            code => {
                let err_msg = "failed to get mount flags";
                log::debug!("Mount::mount_flags {}. libmount::mnt_context_get_mflags returned error code: {:?}", err_msg, code);

                None
            }
        }
    }

    /// Returns the exit status of a mount helper (mount.*filesytem*) called by the user. The
    /// resulting value is pertinent only when the method [`Mount::has_run_mount_helper`] returns
    /// `true`.
    pub fn mount_helper_exit_status(&self) -> i32 {
        let status = unsafe { libmount::mnt_context_get_helper_status(self.inner) };
        log::debug!("Mount::mount_helper_exit_status value: {:?}", status);

        status
    }

    /// Returns the number of the last error,
    /// [errno](https://www.man7.org/linux/man-pages/man3/errno.3.html), if invoking the
    /// [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html) syscall resulted in a
    /// failure.
    pub fn mount_syscall_errno(&self) -> Option<i32> {
        log::debug!("Mount::mount_syscall_errno getting mount(2) syscall error number");

        let result = unsafe { libmount::mnt_context_get_syscall_errno(self.inner) };

        match result {
            0 => {
                let err_msg = "mount(2) syscall was never invoked, or ran successfully";
                log::debug!("Mount::mount_syscall_errno {}. libmount::mnt_context_get_syscall_errno returned error code: 0", err_msg);

                None
            }
            errno => {
                log::debug!("Mount::mount_syscall_errno got mount(2) syscall error number");

                Some(errno)
            }
        }
    }

    /// Returns the set of userspace mount flags set during configuration, or `None` if they were
    /// not provided.
    pub fn userspace_mount_flags(&self) -> Option<HashSet<UserspaceMountFlag>> {
        log::debug!("Mount::userspace_mount_flags getting user space mount flags");

        let mut bits = MaybeUninit::<libc::c_ulong>::zeroed();

        let result =
            unsafe { libmount::mnt_context_get_user_mflags(self.inner, bits.as_mut_ptr()) };

        match result {
            0 => {
                log::debug!("Mount::userspace_mount_flags got user space mount flags");
                let bits = unsafe { bits.assume_init() };
                let flags: HashSet<_> = all::<UserspaceMountFlag>()
                    .filter(|&flag| (flag as u64) & bits != 0)
                    .collect();

                Some(flags)
            }
            code => {
                let err_msg = "failed to get user space mount flags";
                log::debug!("Mount::userspace_mount_flags {}. libmount::mnt_context_get_user_mflags returned error code: {:?}", err_msg, code);

                None
            }
        }
    }

    /// Returns a reference the associated [`Cache`], or `None` if it was not set.
    pub fn cache(&self) -> Option<&Cache> {
        log::debug!("Mount::cache getting associated cache");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_cache>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_cache(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get associated cache";
                log::debug!(
                    "Mount::cache {}. libmount::mnt_context_get_cache returned a NULL pointer",
                    err_msg
                );

                None
            }
            ptr => {
                log::debug!("Mount::cache got associated cache");
                let cache = owning_ref_from_ptr!(self, Cache, ptr);

                Some(cache)
            }
        }
    }

    /// Returns the file system's description table, or `None` if it can't
    /// find one.
    pub fn fstab(&self) -> Option<FsTab> {
        log::debug!("Mount::fstab getting file system description file `fstab`");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();

        let result = unsafe { libmount::mnt_context_get_fstab(self.inner, ptr.as_mut_ptr()) };

        match result {
            0 => {
                log::debug!("Mount::fstab got file system description file `fstab`");
                let ptr = unsafe { ptr.assume_init() };
                let table = FsTab::from_ptr(ptr);

                Some(table)
            }
            code => {
                let err_msg = "failed to get file system description file `fstab`";
                log::debug!(
                    "Mount::fstab {}. libmount::mnt_context_get_fstab returned error code: {:?}",
                    err_msg,
                    code
                );

                None
            }
        }
    }

    /// Returns the user data associated to this `Mount`'s `fstab` table, or `None` if it does
    /// not exist.
    pub fn fstab_user_data(&self) -> Option<NonNull<libc::c_void>> {
        log::debug!("Mount::fstab_user_data getting `fstab` associated user data");

        let mut ptr = MaybeUninit::<*mut libc::c_void>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_fstab_userdata(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no associated user data";
                log::debug!("Mount::fstab_user_data {}. libmount::mnt_context_get_fstab_userdata returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::fstab_user_data got `fstab` associated user data");

                NonNull::new(ptr)
            }
        }
    }

    /// Returns this `Mount`'s file system type, or `None` if it was not set.
    pub fn file_system_type(&self) -> Option<FileSystem> {
        log::debug!("Mount::file_system_type getting file system type");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_fstype(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get file system type";
                log::debug!("Mount::file_system_type {}. libmount::mnt_context_get_fstype returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::file_system_type got file system type");
                let fs_type_cstr = ffi_utils::c_char_array_to_string(ptr);

                match FileSystem::from_str(fs_type_cstr.as_ref()) {
                    Ok(fs_type) => Some(fs_type),
                    Err(e) => {
                        log::debug!("Mount::file_system_type {:?}", e);

                        None
                    }
                }
            }
        }
    }

    /// Returns an instance of the `/proc/self/mountinfo`, or `None` if it can't find
    /// one.
    pub fn mountinfo(&self) -> Option<MountInfo> {
        log::debug!("Mount::mountinfo getting `mountinfo` table");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_table>::zeroed();

        let result = unsafe { libmount::mnt_context_get_mtab(self.inner, ptr.as_mut_ptr()) };

        match result {
            0 => {
                log::debug!("Mount::mountinfo got `mountinfo` table");
                let ptr = unsafe { ptr.assume_init() };
                let table = MountInfo::from_ptr(ptr);

                Some(table)
            }
            code => {
                let err_msg = "failed to get `mountinfo` table";
                log::debug!(
                    "Mount::mountinfo {}. libmount::mnt_context_get_mtab returned error code: {:?}",
                    err_msg,
                    code
                );

                None
            }
        }
    }

    /// Returns the user data associated to this `Mount`'s `mountinfo` table, or `None` if it does
    /// not exist.
    pub fn mountinfo_user_data(&self) -> Option<NonNull<libc::c_void>> {
        log::debug!("Mount::mountinfo_user_data getting `mountinfo` associated user data");

        let mut ptr = MaybeUninit::<*mut libc::c_void>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_mtab_userdata(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no associated user data";
                log::debug!("Mount::mountinfo_user_data {}. libmount::mnt_context_get_mtab_userdata returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::mountinfo_user_data got `mountinfo` associated user data");

                NonNull::new(ptr)
            }
        }
    }

    /// Returns this `Mount`'s original [`MountNamespace`], or `None` if it is
    /// not set.
    pub fn original_namespace(&self) -> Option<MountNamespace> {
        log::debug!("Mount::original_namespace getting mount namespace");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_ns>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_origin_ns(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no original namespace";
                log::debug!("Mount::original_namespace {}. libmount::mnt_context_get_origin_ns returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Mount::original_namespace got mount namespace");
                let ns = MountNamespace::from_raw_parts(ptr, self);

                Some(ns)
            }
        }
    }
    //---- END getters

    //---- BEGIN iterators

    /// Tries to sequentially mount entries in `/etc/fstab`.
    ///
    /// To filter devices to mount by file system type and/or mount options, use the
    /// methods [`MountBuilder::match_file_systems`] and/or [`MountBuilder::match_mount_options`]
    /// when instantiating a new `Mount` object.
    pub fn seq_mount(&mut self) -> MountIter {
        MountIter::new(self).unwrap()
    }

    /// Tries to sequentially remount entries in `/proc/self/mountinfo`.
    ///
    /// To filter devices to remount by file system type and/or mount options, use the
    /// methods [`MountBuilder::match_file_systems`] and/or [`MountBuilder::match_mount_options`]
    /// when instantiating a new `Mount` object.
    pub fn seq_remount(&mut self) -> ReMountIter {
        ReMountIter::new(self).unwrap()
    }

    //---- END iterators

    //---- BEGIN predicates

    /// Returns `true` if this `Mount`'s internal [`FsTab`] has a matching `entry`. This
    /// function compares the `source`, `target`, and `root` fields of the function parameter
    /// against those of each entry in the [`FsTab`].
    ///
    /// **Note:** the `source`, and `target` fields are canonicalized if a [`Cache`] is set for
    /// this `Mount`.
    ///
    /// **Note:** swap partitions are ignored.
    ///
    /// **Warning:** on `autofs` mount points, canonicalizing the `target` field may trigger
    /// an automount.
    pub fn is_entry_mounted(&self, entry: &FsTabEntry) -> bool {
        log::debug!("Mount::is_entry_mounted checking if mount table entry was mounted");

        let mut status = MaybeUninit::<libc::c_int>::zeroed();

        let result = unsafe {
            libmount::mnt_context_is_fs_mounted(self.inner, entry.inner, status.as_mut_ptr())
        };

        match result {
            0 => {
                log::debug!("Mount::is_entry_mounted checked if mount table entry was mounted");
                let status = unsafe { status.assume_init() };

                status == 1
            }
            code => {
                let err_msg = "failed to check if mount table entry was mounted";
                log::debug!("Mount::is_entry_mounted {}. libmount::mnt_context_is_fs_mounted returned error code: {:?}", err_msg, code);

                false
            }
        }
    }

    /// Returns `true` when this `Mount` is configured to mount devices in parallel and is the
    /// parent process.
    pub fn is_parent_process(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_parent(self.inner) == 1 };
        log::debug!("Mount::is_parent_process value: {:?}", state);

        state
    }

    /// Returns `true` when this `Mount` is configured to mount devices in parallel and is a
    /// child process.
    pub fn is_child_process(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_child(self.inner) == 1 };
        log::debug!("Mount::is_child_process value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is configured to perform a dry run.
    pub fn is_dry_run(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_fake(self.inner) == 1 };
        log::debug!("Mount::is_dry_run value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is configured to be verbose.
    pub fn is_verbose(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_verbose(self.inner) == 1 };
        log::debug!("Mount::is_verbose value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is set to perform unprivileged mounts (mon-root mounts).
    pub fn is_user_mount(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_restricted(self.inner) == 1 };
        log::debug!("Mount::is_user_mount value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is configured NOT to canonicalize paths.
    pub fn disabled_path_canonicalization(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_nocanonicalize(self.inner) == 1 };
        log::debug!("Mount::disabled_path_canonicalization value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is configured NOT to lookup a device or mount point in
    /// `/etc/fstab` if one is not provided when setting this `Mount`'s source or target.
    pub fn disabled_mount_point_lookup(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_swapmatch(self.inner) == 1 };
        log::debug!("Mount::disabled_mount_point_lookup value: {:?}", state);

        state
    }

    /// Returns `true` if the [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html)
    /// was invoked.
    pub fn has_called_mount_syscall(&self) -> bool {
        let state = unsafe { libmount::mnt_context_syscall_called(self.inner) == 1 };
        log::debug!("Mount::has_called_mount_syscall value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is configured NOT to use mount helpers.
    pub fn has_disabled_helpers(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_nohelpers(self.inner) == 1 };
        log::debug!("Mount::has_disabled_helpers value: {:?}", state);

        state
    }

    /// Returns `true` if a mount helper was run.
    pub fn has_run_mount_helper(&self) -> bool {
        let state = unsafe { libmount::mnt_context_helper_executed(self.inner) == 1 };
        log::debug!("Mount::has_run_mount_helper value: {:?}", state);

        state
    }

    /// Returns `true` if mount helpers, or the
    /// [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html) were run successfully.
    pub fn is_mount_successful(&self) -> bool {
        let state = unsafe { libmount::mnt_context_get_status(self.inner) == 1 };
        log::debug!("Mount::is_mount_successful value: {:?}", state);

        state
    }

    /// Returns `true` if a write protected device is mounted in read-only mode.
    pub fn is_mounted_read_only(&self) -> bool {
        let state = unsafe { libmount::mnt_context_forced_rdonly(self.inner) == 1 };
        log::debug!("Mount::is_mounted_read_only value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is configured to disable userpace mount table updates.
    pub fn does_not_update_utab(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_nohelpers(self.inner) == 1 };
        log::debug!("Mount::does_not_update_utab value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is configured to mount devices in parallel.
    pub fn does_parallel_mount(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_fork(self.inner) == 1 };
        log::debug!("Mount::does_parallel_mount value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is configured to force mount devices in read-write mode.
    pub fn forces_mount_read_write(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_rwonly_mount(self.inner) == 1 };
        log::debug!("Mount::forces_mount_read_write value: {:?}", state);

        state
    }

    /// Returns `true` if this `Mount` is configured to ignore mount options unsupported by a
    /// device's file system.
    pub fn ignores_unsupported_mount_options(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_sloppy(self.inner) == 1 };
        log::debug!(
            "Mount::ignores_unsupported_mount_options value: {:?}",
            state
        );

        state
    }

    #[cfg(mount = "v2_39")]
    /// Returns `true` if this `Mount` is configured to check that a device is not already mounted
    /// before mounting it.
    pub fn mounts_only_once(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_onlyonce(self.inner) == 1 };
        log::debug!("Mount::mounts_only_once value: {:?}", state);

        state
    }

    //---- END predicates
}

impl AsRef<Mount> for Mount {
    fn as_ref(&self) -> &Mount {
        self
    }
}

impl Drop for Mount {
    fn drop(&mut self) {
        log::debug!("Mount::drop deallocating `Mount` instance");

        unsafe {
            libmount::mnt_free_context(self.inner);
        }

        // Free objects allocated on the heap for returned references.
        while let Some(gc_item) = self.gc.pop() {
            gc_item.destroy();
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use tempfile::Builder;
    use tempfile::NamedTempFile;

    use std::fs::File;
    use std::io::Write;

    use super::*;
    use crate::core::device::BlockDevice;
    use crate::mount::ExitCode;

    //---- Helper functions

    static BASE_DIR_TEST_IMG_FILES: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/third-party/vendor/util-linux/blkid/images"
    );

    fn decode_into<P, W>(xz_file_path: P, writer: &mut W) -> std::io::Result<u64>
    where
        P: AsRef<Path>,
        W: Write + ?Sized,
    {
        let xz_file_path = xz_file_path.as_ref();

        // Copy decompressed image to temporary file
        let compressed_image_file = std::fs::File::open(xz_file_path)?;
        let mut decompressed = xz2::read::XzDecoder::new(compressed_image_file);

        std::io::copy(&mut decompressed, writer)
    }

    /// Creates a named temporary image file with one of the supported file systems from the
    /// compressed samples.
    fn disk_image(fs_type: &str) -> NamedTempFile {
        let img_path = format!("{BASE_DIR_TEST_IMG_FILES}/filesystems/{fs_type}.img.xz");
        let mut named_file = NamedTempFile::new().expect("failed to get new NamedTempFile");

        decode_into(img_path, named_file.as_file_mut()).expect("failed to create named disk image");
        named_file
    }

    //-------------------------------------------------------------------------

    // #[test]
    // #[should_panic(
    //     expected = "you MUST call at least one of the following functions: `MountBuilder::source`, `MountBuilder::target`"
    // )]
    // fn mount_must_have_a_source_or_target() {
    //     let _ = Mount::builder().build().unwrap();
    // }

    #[test]
    fn mount_can_mount_an_image_file() -> crate::Result<()> {
        if inside_vm::inside_vm() {
            let image_file = disk_image("ext3");
            let source = BlockDevice::from(image_file.path());
            let tmp_dir = Builder::new().prefix("rsmount-test-").tempdir().unwrap();
            let mut mount = Mount::builder()
                .source(source.into())
                .target(tmp_dir.path())
                .build()?;

            let status = mount.mount_device()?;

            let actual = mount.is_mount_successful();
            let expected = true;
            assert_eq!(actual, expected);

            let actual = status.exit_code();
            let expected = ExitCode::Success;
            assert_eq!(actual, &expected);
        }

        Ok(())
    }
}
