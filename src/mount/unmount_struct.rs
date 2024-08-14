// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};

// From this library

use crate::ffi_utils;
use crate::mount::ExitCode;
use crate::mount::ExitStatus;
use crate::mount::MountSource;
use crate::mount::UMountIter;
use crate::mount::UMountNamespace;
use crate::mount::UmntBuilder;
use crate::mount::UnmountBuilder;
use crate::mount::UnmountError;

/// Object to unmount a device.
#[derive(Debug)]
#[repr(transparent)]
pub struct Unmount {
    pub(crate) inner: *mut libmount::libmnt_context,
}

impl Unmount {
    #[doc(hidden)]
    /// Wraps a raw `libmount::mnt_context` pointer with a safe `Unmount`.
    #[allow(dead_code)]
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_context) -> Unmount {
        Self { inner: ptr }
    }

    #[doc(hidden)]
    /// Creates a new `Unmount`.
    pub(crate) fn new() -> Result<Unmount, UnmountError> {
        log::debug!("Unmount::new creating a new `Unmount` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_context>::zeroed();

        unsafe {
            inner.write(libmount::mnt_new_context());
        }

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `Unmount` instance".to_owned();
                log::debug!(
                    "Unmount::new {}. libmount::mnt_new_contex returned a NULL pointer",
                    err_msg
                );

                Err(UnmountError::Creation(err_msg))
            }
            inner => {
                log::debug!("Unmount::new created a new `Unmount` instance");
                let mount = Self::from_ptr(inner);

                Ok(mount)
            }
        }
    }

    #[doc(hidden)]
    /// Converts a function's return code to unified `libmount` exit code.
    fn return_code_to_exit_status(&self, return_code: i32) -> Result<ExitStatus, UnmountError> {
        log::debug!(
            "Unmount::return_code_to_exit_status converting to exit status the return code: {:?}",
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
            "Unmount::return_code_to_exit_status converted return code: {:?} to exit status {:?}",
            return_code,
            exit_status
        );

        Ok(exit_status)
    }

    //---- BEGIN setters

    #[doc(hidden)]
    /// Enables/disables deleting loop devices after unmounting.
    fn enable_delete_loop_device(
        mount: *mut libmount::libmnt_context,
        enable: bool,
    ) -> Result<(), UnmountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_loopdel(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Unmount::enable_delete_loop_device {}d loop device deletion after unmount",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} loop device deletion after unmount", op_str);
                log::debug!("Unmount::enable_delete_loop_device {}. libmount::mnt_context_enable_loopdel returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Enables `Unmount` to delete loop devices after unmount.
    pub(crate) fn enable_detach_loop_device(&mut self) -> Result<(), UnmountError> {
        log::debug!(
            "Unmount::enable_detach_loop_device enabling loop device deletion after unmount"
        );

        Self::enable_delete_loop_device(self.inner, true)
    }

    #[doc(hidden)]
    /// Disables `Unmount` to delete loop devices after unmount.
    pub(crate) fn disable_detach_loop_device(&mut self) -> Result<(), UnmountError> {
        log::debug!(
            "Unmount::disable_detach_loop_device disabling loop device deletion after unmount"
        );

        Self::enable_delete_loop_device(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables path canonicalization.
    fn disable_canonicalize(
        mount: *mut libmount::libmnt_context,
        disable: bool,
    ) -> Result<(), UnmountError> {
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
                    "Unmount::disable_canonicalize {}d path canonicalization",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} path canonicalization", op_str);
                log::debug!("Unmount::disable_canonicalize {}. libmount::mnt_context_disable_canonicalize returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Disables path canonicalization.
    pub(crate) fn disable_path_canonicalization(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::disable_path_canonicalization disabling path canonicalization");

        Self::disable_canonicalize(self.inner, true)
    }

    #[doc(hidden)]
    /// Enables path canonicalization.
    pub(crate) fn enable_path_canonicalization(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::enable_path_canonicalization enabling path canonicalization");

        Self::disable_canonicalize(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables mount helpers.
    fn disable_mnt_helpers(
        mount: *mut libmount::libmnt_context,
        disable: bool,
    ) -> Result<(), UnmountError> {
        let op = if disable { 1 } else { 0 };
        let op_str = if disable {
            "disable".to_owned()
        } else {
            "enable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_disable_helpers(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Unmount::disable_mnt_helpers {}d mount/umount helpers",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} mount/umount helpers", op_str);
                log::debug!("Unmount::disable_mnt_helpers {}. libmount::mnt_context_disable_helpers returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Prevents `Unmount` from calling `/sbin/umount.suffix` helper functions, where *suffix* is a file system type.
    pub(crate) fn disable_helpers(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::disable_helpers disabling mount/umount helpers");

        Self::disable_mnt_helpers(self.inner, true)
    }

    #[doc(hidden)]
    /// Allows `Unmount` to call `/sbin/umount.suffix` helper functions, where *suffix* is a file system type.
    pub(crate) fn enable_helpers(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::enable_helpers enabling mount/umount helpers");

        Self::disable_mnt_helpers(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables mount point lookup.
    fn disable_swap_match(
        mount: *mut libmount::libmnt_context,
        disable: bool,
    ) -> Result<(), UnmountError> {
        let op = if disable { 1 } else { 0 };
        let op_str = if disable {
            "disable".to_owned()
        } else {
            "enable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_disable_swapmatch(mount, op) };

        match result {
            0 => {
                log::debug!("Unmount::disable_swap_match {}d mount point lookup", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} mount point lookup", op_str);
                log::debug!("Unmount::disable_swap_match {}. libmount::mnt_context_disable_swapmatch returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Disables mount point lookup in `/etc/fstab` when either the mount `source` or `target` is
    /// not set.
    pub(crate) fn disable_mount_point_lookup(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::disable_mount_point_lookup disabling mount point lookup");

        Self::disable_swap_match(self.inner, true)
    }

    #[doc(hidden)]
    /// Enables mount point lookup in `/etc/fstab` when either the mount `source` or `target` is
    /// not set.
    pub(crate) fn enable_mount_point_lookup(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::enable_mount_point_lookup enabling mount point lookup");

        Self::disable_swap_match(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables userspace mount table updates.
    fn disable_mtab(
        mount: *mut libmount::libmnt_context,
        disable: bool,
    ) -> Result<(), UnmountError> {
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
                    "Unmount::disable_mtab {}d userspace mount table updates",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} userspace mount table updates", op_str);
                log::debug!("Unmount::disable_mtab {}. libmount::mnt_context_disable_mtab returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Disables userspace mount table updates.
    pub(crate) fn do_not_update_utab(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::do_not_update_utab disabling userspace mount table updates");

        Self::disable_mtab(self.inner, true)
    }

    #[doc(hidden)]
    /// Enables userspace mount table updates.
    pub(crate) fn update_utab(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::update_utab enabling userspace mount table updates");

        Self::disable_mtab(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables skipping all mount source preparation, mount option analysis, and the actual mounting process.
    /// (see the [`mount` command](https://www.man7.org/linux/man-pages/man8/mount.8.html) man page, option `-f, --fake`)
    fn enable_fake(mount: *mut libmount::libmnt_context, enable: bool) -> Result<(), UnmountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_fake(mount, op) };

        match result {
            0 => {
                log::debug!("Unmount::enable_fake {}d dry run", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} dry run", op_str);
                log::debug!("Unmount::enable_fake {}. libmount::mnt_context_enable_fake returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Skips all mount source preparation, mount option analysis, and the actual mounting process.
    pub(crate) fn enable_dry_run(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::enable_dry_run enabling dry run");

        Self::enable_fake(self.inner, true)
    }

    #[doc(hidden)]
    pub(crate) fn disable_dry_run(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::disable_dry_run disabling dry run");

        Self::enable_fake(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables force device unmount.
    fn enable_force_device_unmount(
        mount: *mut libmount::libmnt_context,
        enable: bool,
    ) -> Result<(), UnmountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_force(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Unmount::enable_force_device_unmount {}d force device unmount",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} force device unmount", op_str);
                log::debug!("Unmount::enable_force_device_unmount {}. libmount::mnt_context_enable_force returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Forces a device umount.
    pub(crate) fn enable_force_unmount(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::enable_force_unmount enabling force device unmount");

        Self::enable_force_device_unmount(self.inner, true)
    }

    #[doc(hidden)]
    /// Prevents `Unmount` from forcing a device umount.
    pub(crate) fn disable_force_unmount(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::disable_force_unmount disabling force device unmount");

        Self::enable_force_device_unmount(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables lazy device unmount.
    fn enable_lazy_device_unmount(
        mount: *mut libmount::libmnt_context,
        enable: bool,
    ) -> Result<(), UnmountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_lazy(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Unmount::enable_lazy_device_unmount {}d lazy device unmount",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} lazy device unmount", op_str);
                log::debug!("Unmount::enable_lazy_device_unmount {}. libmount::mnt_context_enable_lazy returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Enables `Unmount` to lazily unmount devices.
    pub(crate) fn enable_lazy_unmount(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::enable_lazy_umount enabling lazy device unmount");

        Self::enable_lazy_device_unmount(self.inner, true)
    }

    #[doc(hidden)]
    /// Disables `Unmount` to lazily unmount devices.
    pub(crate) fn disable_lazy_unmount(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::disable_lazy_umount disabling lazy device unmount");

        Self::enable_lazy_device_unmount(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables verbose output.
    fn enable_verbose(
        mount: *mut libmount::libmnt_context,
        enable: bool,
    ) -> Result<(), UnmountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_verbose(mount, op) };

        match result {
            0 => {
                log::debug!("Unmount::enable_verbose {}d verbose output", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} verbose output", op_str);
                log::debug!("Unmount::enable_verbose {}. libmount::mnt_context_enable_verbose returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Enables `Unmount` to check that a device is not already mounted before mounting it.
    pub(crate) fn enable_verbose_output(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::enable_verbose_output enabling verbose output");

        Self::enable_verbose(self.inner, true)
    }

    #[doc(hidden)]
    /// Disables `Unmount` to check that a device is not already mounted before mounting it.
    pub(crate) fn disable_verbose_output(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::disable_verbose_output disabling verbose output");

        Self::enable_verbose(self.inner, false)
    }

    #[doc(hidden)]
    /// Enables/disables `Unmount` functionality to remount a device in read-only mode after a failed unmount.
    fn enable_read_only_unmount(
        mount: *mut libmount::libmnt_context,
        enable: bool,
    ) -> Result<(), UnmountError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_context_enable_rdonly_umount(mount, op) };

        match result {
            0 => {
                log::debug!(
                    "Unmount::enable_read_only_unmount {}d remount read-only after failed unmount",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to {} remount read-only after failed unmount",
                    op_str
                );
                log::debug!("Unmount::enable_read_only_unmount {}. libmount::mnt_context_enable_rdonly_umount returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Enables `Unmount` functionality to remount a device in read-only mode after a failed unmount.
    pub(crate) fn enable_on_fail_remount_read_only(&mut self) -> Result<(), UnmountError> {
        log::debug!(
            "Unmount::enable_on_fail_remount_read_only enabling read only remount after failed unmount"
        );

        Self::enable_read_only_unmount(self.inner, true)
    }

    #[doc(hidden)]
    /// Disables `Unmount` functionality to remount a device in read-only mode after a failed unmount.
    pub(crate) fn disable_on_fail_remount_read_only(&mut self) -> Result<(), UnmountError> {
        log::debug!(
            "Unmount::disable_on_fail_remount_read_only disabling read only remount after failed unmount"
        );

        Self::enable_read_only_unmount(self.inner, false)
    }

    #[doc(hidden)]
    /// Sets a list of file systems.
    pub(crate) fn set_file_systems_filter<T>(&mut self, fs_list: T) -> Result<(), UnmountError>
    where
        T: AsRef<str>,
    {
        let fs_list = fs_list.as_ref();
        let fs_list_cstr = ffi_utils::as_ref_str_to_c_string(fs_list)?;
        log::debug!(
            "Unmount::set_file_systems_filter setting the list of file systems: {:?}",
            fs_list
        );

        let result =
            unsafe { libmount::mnt_context_set_fstype_pattern(self.inner, fs_list_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "Unmount::set_file_systems_filter set the list of file systems: {:?}",
                    fs_list
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set file systems list: {:?}", fs_list);
                log::debug!("Unmount::set_file_systems_filter {}. libmount::mnt_context_set_fstype_pattern returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets the pattern of mount options to use as filter when mounting devices.
    pub(crate) fn set_mount_options_filter<T>(
        &mut self,
        options_list: T,
    ) -> Result<(), UnmountError>
    where
        T: AsRef<str>,
    {
        let options_list = options_list.as_ref();
        let options_list_cstr = ffi_utils::as_ref_str_to_c_string(options_list)?;
        log::debug!(
            "Unmount::set_mount_options_filter setting mount options filter: {:?}",
            options_list
        );

        let result = unsafe {
            libmount::mnt_context_set_options_pattern(self.inner, options_list_cstr.as_ptr())
        };

        match result {
            0 => {
                log::debug!(
                    "Unmount::set_mount_options_filter set mount options filter: {:?}",
                    options_list
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount options filter: {:?}", options_list);
                log::debug!("Unmount::set_mount_options_filter {}. libmount::mnt_context_set_options_pattern returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Set the `Unmount`'s source.
    fn set_source(
        mount: *mut libmount::libmnt_context,
        source: CString,
    ) -> Result<(), UnmountError> {
        let result = unsafe { libmount::mnt_context_set_source(mount, source.as_ptr()) };

        match result {
            0 => {
                log::debug!("Unmount::set_source mount source set");

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount source: {:?}", source);
                log::debug!("Unmount::set_source {}. libmount::mnt_context_set_source returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
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
    pub(crate) fn set_mount_source(&mut self, source: MountSource) -> Result<(), UnmountError> {
        let source = source.to_string();
        let source_cstr = ffi_utils::as_ref_path_to_c_string(&source)?;
        log::debug!(
            "Unmount::set_mount_source setting mount source: {:?}",
            source
        );

        Self::set_source(self.inner, source_cstr)
    }

    #[doc(hidden)]
    /// Sets this `Unmount`'s mount point.
    pub(crate) fn set_mount_target<T>(&mut self, target: T) -> Result<(), UnmountError>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        let target_cstr = ffi_utils::as_ref_path_to_c_string(target)?;
        log::debug!(
            "Unmount::set_mount_target setting mount target to: {:?}",
            target
        );

        let result = unsafe { libmount::mnt_context_set_target(self.inner, target_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "Unmount::set_mount_target set mount target to: {:?}",
                    target
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount target to: {:?}", target);
                log::debug!("Unmount::set_mount_target {}. libmount::mnt_context_set_target returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Sets the namespace of this `Unmount`'s mount point.
    pub(crate) fn set_mount_target_namespace<T>(&mut self, path: T) -> Result<(), UnmountError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;
        log::debug!(
            "Unmount::set_mount_target_namespace setting mount target namespace: {:?}",
            path
        );

        let result = unsafe { libmount::mnt_context_set_target_ns(self.inner, path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "Unmount::set_mount_target_namespace set mount target namespace: {:?}",
                    path
                );

                Ok(())
            }
            code if code == -libc::ENOSYS => {
                let err_msg = "`libmount` was compiled without namespace support".to_owned();
                log::debug!("Unmount::set_mount_target_namespace {}. libmount::mnt_context_set_target returned error code: {:?}", err_msg, code);

                Err(UnmountError::NoNamespaceSupport(err_msg))
            }
            code => {
                let err_msg = format!("failed to set mount target namespace: {:?}", path);
                log::debug!("Unmount::set_mount_target_namespace {}. libmount::mnt_context_set_target_ns returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    //---- END setters

    /// Creates a [`UnmountBuilder`] to configure and construct a new `Unmount`
    /// instance.
    ///
    /// Call the `UnmountBuilder`'s [`build()`](UnmountBuilder::build) method to
    /// construct a new `Unmount` instance.
    pub fn builder() -> UnmountBuilder {
        log::debug!("Unmount::builder creating new `UnmountBuilder` instance");
        UmntBuilder::builder()
    }

    //---- BEGIN mutators

    /// Unmounts a device using the [`umount` syscall](https://www.man7.org/linux/man-pages/man2/umount.2.html) and/or
    /// [`umount` helpers](https://www.man7.org/linux/man-pages/man8/umount.8.html#EXTERNAL_HELPERS).
    ///
    /// Equivalent to running the following functions in succession:
    /// - [`Unmount::prepare_unmount`]
    /// - [`Unmount::call_umount_syscall`]
    /// - [`Unmount::finalize_umount`]
    pub fn unmount_device(&mut self) -> Result<ExitStatus, UnmountError> {
        log::debug!("Unmount::unmount_device unmounting device");

        let return_code = unsafe { libmount::mnt_context_umount(self.inner) };
        self.return_code_to_exit_status(return_code)
    }

    /// Validates this `Unmount`'s parameters before it tries to unmount a device.
    ///
    /// **Note:** you do not need to call this method if you are using [`Unmount::unmount_device`], it
    /// will take care of parameter validation.
    pub fn prepare_unmount(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::prepare_unmount preparing for unmount");

        let result = unsafe { libmount::mnt_context_prepare_umount(self.inner) };

        match result {
            0 => {
                log::debug!("Unmount::prepare_unmount preparation successful");

                Ok(())
            }
            code => {
                let err_msg = "failed to prepare for device unmount".to_owned();
                log::debug!("Unmount::prepare_unmount {}. libmount::mnt_context_prepare_umount returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    /// Runs the [`umount` syscall](https://www.man7.org/linux/man-pages/man2/umount.2.html) and/or
    /// [umount helpers](https://www.man7.org/linux/man-pages/man8/umount.8.html#EXTERNAL_HELPERS).
    ///
    /// **Note:** you do not need to call this method if you are using [`Unmount::unmount_device`], it
    /// will take care of everything for you.
    pub fn call_umount_syscall(&mut self) -> Result<ExitStatus, UnmountError> {
        log::debug!("Unmount::call_umount_syscall unmounting device");

        let return_code = unsafe { libmount::mnt_context_do_umount(self.inner) };
        self.return_code_to_exit_status(return_code)
    }

    /// Updates the system's mount tables to take the last modifications into account. You should
    /// call this function after invoking [`Unmount::call_umount_syscall`].
    ///
    /// **Note:** you do not need to call this method if you are using [`Unmount::unmount_device`], it
    /// will take care of finalizing the unmount.
    pub fn finalize_umount(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::finalize_umount finalizing unmount");

        let result = unsafe { libmount::mnt_context_finalize_umount(self.inner) };

        match result {
            0 => {
                log::debug!("Unmount::finalize_umount finalized unmount");

                Ok(())
            }
            code => {
                let err_msg = "failed to finalize device unmount".to_owned();
                log::debug!("Unmount::finalize_umount {}. libmount::mnt_context_finalize_umount returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    /// Sets `umount`'s syscall exit status if the function was called outside of `libmount`.
    ///
    /// The `exit_status` should be `0` on success, and a negative number on error (e.g. `-errno`).
    pub fn set_syscall_exit_status(&mut self, exit_status: i32) -> Result<(), UnmountError> {
        log::debug!(
            "Unmount::set_syscall_exit_status setting mount/umount syscall exit status to {:?}",
            exit_status
        );

        let result = unsafe { libmount::mnt_context_set_syscall_status(self.inner, exit_status) };

        match result {
            0 => {
                log::debug!(
                    "Unmount::set_syscall_exit_status set mount/umount syscall exit status to {:?}",
                    exit_status
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "ailed to set mount/umount syscall exit status to {:?}",
                    exit_status
                );
                log::debug!("Unmount::set_syscall_exit_status {}. libmount::mnt_context_set_syscall_status returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    /// Resets `umount` exit status so that `umount` methods can be called again.
    pub fn reset_syscall_exit_status(&mut self) -> Result<(), UnmountError> {
        log::debug!("Unmount::reset_syscall_exit_status resetting syscall exit status");

        let result = unsafe { libmount::mnt_context_reset_status(self.inner) };

        match result {
            0 => {
                log::debug!("Unmount::reset_syscall_exit_status reset syscall exit status");

                Ok(())
            }
            code => {
                let err_msg = "failed to reset syscall exit status".to_owned();
                log::debug!("Unmount::reset_syscall_exit_status {}. libmount::mnt_context_reset_status returned error code: {:?}", err_msg, code);

                Err(UnmountError::Config(err_msg))
            }
        }
    }

    /// Switches to the provided `namespace`, and returns the namespace used previously.
    pub fn switch_to_namespace(&mut self, namespace: UMountNamespace) -> Option<UMountNamespace> {
        log::debug!("Unmount::switch_to_namespace switching namespace");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_ns>::zeroed();
        unsafe {
            ptr.write(libmount::mnt_context_switch_ns(self.inner, namespace.ptr));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no prior namespace";
                log::debug!("Unmount::switch_to_namespace {}. libmount::mnt_context_switch_ns returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Unmount::switch_to_namespace switched namespace");
                let namespace = UMountNamespace::from_raw_parts(ptr, self);

                Some(namespace)
            }
        }
    }

    /// Switches to the namespace at creation, and returns the replacement namespace used up to this point.
    pub fn switch_to_original_namespace(&mut self) -> Option<UMountNamespace> {
        log::debug!("Unmount::switch_to_original_namespace switching to original namespace");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_ns>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_switch_origin_ns(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no prior namespace";
                log::debug!("Unmount::switch_to_original_namespace {}. libmount::mnt_context_switch_origin_ns returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Unmount::switch_to_original_namespace switched to original namespace");
                let namespace = UMountNamespace::from_raw_parts(ptr, self);

                Some(namespace)
            }
        }
    }

    /// Switches to the target's namespace, and returns the namespace used previously.
    pub fn switch_to_target_namespace(&mut self) -> Option<UMountNamespace> {
        log::debug!("Unmount::switch_to_target_namespace switching to target namespace");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_ns>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_switch_target_ns(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no prior namespace";
                log::debug!("Unmount::switch_to_target_namespace {}. libmount::mnt_context_switch_target_ns returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Unmount::switch_to_target_namespace switched to target namespace");
                let namespace = UMountNamespace::from_raw_parts(ptr, self);

                Some(namespace)
            }
        }
    }

    //---- END mutators

    //---- BEGIN getters

    /// Returns the identifier of the device to mount, or `None` if it was not provided.
    pub fn source(&self) -> Option<String> {
        log::debug!("Unmount::source getting identifier of device to mount");

        let mut identifier = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            identifier.write(libmount::mnt_context_get_source(self.inner));
        }

        match unsafe { identifier.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get identifier of device to mount";
                log::debug!(
                    "Unmount::source {}. libmount::mnt_context_get_source returned a NULL pointer",
                    err_msg
                );

                None
            }
            ptr => {
                log::debug!("Unmount::source got identifier of device to mount");
                let source = ffi_utils::c_char_array_to_string(ptr);

                Some(source)
            }
        }
    }

    /// Returns the configured device mount point, or `None` if it was not provided.
    pub fn target(&self) -> Option<PathBuf> {
        log::debug!("Unmount::target getting mount point");

        let mut mount_point = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            mount_point.write(libmount::mnt_context_get_target(self.inner));
        }

        match unsafe { mount_point.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get mount point";
                log::debug!(
                    "Unmount::target {}. libmount::mnt_context_get_target returned a NULL pointer",
                    err_msg
                );

                None
            }
            ptr => {
                log::debug!("Unmount::target got mount point");
                let target = ffi_utils::const_c_char_array_to_path_buf(ptr);

                Some(target)
            }
        }
    }

    /// Returns the mount point's [`UMountNamespace`], or `None` if it is
    /// not set.
    pub fn target_namespace(&self) -> Option<UMountNamespace> {
        log::debug!("Unmount::target_namespace getting mount point's namespace");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_ns>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_context_get_target_ns(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "found no mount point namespace";
                log::debug!("Unmount::target_namespace {}. libmount::mnt_context_get_target_ns returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("Unmount::target_namespace got mount point's namespace");
                let ns = UMountNamespace::from_raw_parts(ptr, self);

                Some(ns)
            }
        }
    }

    /// Returns the exit status of a mount helper (mount.*filesytem*) called by the user. The
    /// resulting value is pertinent only when the method [`Unmount::has_run_umount_helper`] returns
    /// `true`.
    pub fn umount_helper_exit_status(&self) -> i32 {
        let status = unsafe { libmount::mnt_context_get_helper_status(self.inner) };
        log::debug!("Unmount::umount_helper_exit_status value: {:?}", status);

        status
    }

    /// Returns the number of the last error,
    /// [errno](https://www.man7.org/linux/man-pages/man3/errno.3.html), if invoking the
    /// [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html) syscall resulted in a
    /// failure.
    pub fn umount_syscall_errno(&self) -> Option<i32> {
        log::debug!("Unmount::umount_syscall_errno getting mount(2) syscall error number");

        let result = unsafe { libmount::mnt_context_get_syscall_errno(self.inner) };

        match result {
            0 => {
                let err_msg = "mount(2) syscall was never invoked, or ran successfully";
                log::debug!("Unmount::mount_syscall_errno {}. libmount::mnt_context_get_syscall_errno returned error code: 0", err_msg);

                None
            }
            errno => {
                log::debug!("Unmount::mount_syscall_errno got mount(2) syscall error number");

                Some(errno)
            }
        }
    }

    //---- END getters

    //---- BEGIN iterators

    /// Tries to sequentially umount entries in `/proc/self/mountinfo`.
    ///
    /// To filter devices to umount by file system type and/or mount options, use the
    /// methods [`UnmountBuilder::match_file_systems`] and/or [`UnmountBuilder::match_mount_options`]
    /// when instantiating a new `Mount` object.
    pub fn seq_unmount(&mut self) -> UMountIter {
        UMountIter::new(self).unwrap()
    }

    //---- END iterators

    //---- BEGIN predicates

    /// Returns `true` if this `Unmount` is configured to perform a dry run.
    pub fn is_dry_run(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_fake(self.inner) == 1 };
        log::debug!("Unmount::is_dry_run value: {:?}", state);

        state
    }

    /// Returns `true` if this `Unmount` is configured to be verbose.
    pub fn is_verbose(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_verbose(self.inner) == 1 };
        log::debug!("Unmount::is_verbose value: {:?}", state);

        state
    }

    /// Returns `true` if this `Unmount` is configured to delete loop devices after unmounting them.
    pub fn detaches_loop_device(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_loopdel(self.inner) == 1 };
        log::debug!("Unmount::detaches_loop_device value: {:?}", state);

        state
    }

    /// Returns `true` if this `Unmount` is configured NOT to canonicalize paths.
    pub fn disabled_path_canonicalization(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_nocanonicalize(self.inner) == 1 };
        log::debug!("Unmount::disabled_path_canonicalization value: {:?}", state);

        state
    }

    /// Returns `true` if this `Unmount` is configured NOT to lookup a device or mount point in
    /// `/etc/fstab` if one is not provided when setting this `Unmount`'s source or target.
    pub fn disabled_mount_point_lookup(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_swapmatch(self.inner) == 1 };
        log::debug!("Unmount::disabled_mount_point_lookup value: {:?}", state);

        state
    }

    /// Returns `true` if the [`umount` syscall](https://www.man7.org/linux/man-pages/man2/umount.2.html)
    /// was invoked.
    pub fn has_called_umount_syscall(&self) -> bool {
        let state = unsafe { libmount::mnt_context_syscall_called(self.inner) == 1 };
        log::debug!("Unmount::has_called_umount_syscall value: {:?}", state);

        state
    }

    /// Returns `true` if this `Unmount` is configured NOT to use umount helpers.
    pub fn has_disabled_helpers(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_nohelpers(self.inner) == 1 };
        log::debug!("Unmount::has_disabled_helpers value: {:?}", state);

        state
    }

    /// Returns `true` if a umount helper was run.
    pub fn has_run_umount_helper(&self) -> bool {
        let state = unsafe { libmount::mnt_context_helper_executed(self.inner) == 1 };
        log::debug!("Unmount::has_run_umount_helper value: {:?}", state);

        state
    }

    /// Returns `true` if this `Unmount` is configured to disable userpace mount table updates.
    pub fn does_not_update_utab(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_nohelpers(self.inner) == 1 };
        log::debug!("Unmount::does_not_update_utab value: {:?}", state);

        state
    }

    /// Returns `true` if this `Unmount` is configured to unmount devices lazily.
    pub fn does_lazy_unmount(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_lazy(self.inner) == 1 };
        log::debug!("Unmount::does_lazy_unmount value: {:?}", state);

        state
    }

    /// Returns `true` if this `Unmount` is configured to force device to unmount.
    pub fn forces_unmount(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_force(self.inner) == 1 };
        log::debug!("Unmount::forces_unmount value: {:?}", state);

        state
    }

    /// Returns `true` if this `Unmount` is configured to remount a device in read-only mode after a
    /// failed unmount.
    pub fn on_fail_remounts_read_only(&self) -> bool {
        let state = unsafe { libmount::mnt_context_is_rdonly_umount(self.inner) == 1 };
        log::debug!("Unmount::on_fail_remounts_read_only value: {:?}", state);

        state
    }

    //---- END predicates
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
