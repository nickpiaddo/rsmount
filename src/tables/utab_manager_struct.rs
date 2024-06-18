// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use enum_iterator::all;

// From standard library
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};

// From this library
use crate::core::entries::UTabEntry;
use crate::core::errors::UtabManagerError;
use crate::core::flags::MountFlag;
use crate::core::fs::FileLock;
use crate::ffi_utils;
use crate::owning_ref_from_ptr;
use crate::tables::GcItem;

/// Userspace mount table manager.
#[derive(Debug)]
pub struct UtabManager {
    pub(crate) ptr: *mut libmount::libmnt_update,
    pub(crate) gc: Vec<GcItem>,
}

impl UtabManager {
    /// Creating a new `UtabManager`.
    pub fn new() -> Result<UtabManager, UtabManagerError> {
        log::debug!("UtabManager::new creating a new `UtabManager` instance");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_update>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_new_update());
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to create a new `UtabManager` instance".to_owned();
                log::debug!(
                    "UtabManager::new {}. libmount::mnt_new_update returned a NULL pointer",
                    err_msg
                );

                Err(UtabManagerError::Creation(err_msg))
            }
            ptr => {
                log::debug!("UtabManager::new created a new `UtabManager` instance");
                let manager = Self { ptr, gc: vec![] };

                Ok(manager)
            }
        }
    }

    #[doc(hidden)]
    /// Enables/disables read only mode.
    fn set_force_read_only(
        manager: *mut libmount::libmnt_update,
        enable: bool,
    ) -> Result<(), UtabManagerError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_update_force_rdonly(manager, op) };

        match result {
            0 => {
                log::debug!(
                    "UtabManager::set_force_read_only {}d force read only",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} force read only", op_str);
                log::debug!("UtabManager::set_force_read_only {}. libmount::mnt_update_force_rdonly returned error code: {:?}", err_msg, code);

                Err(UtabManagerError::Config(err_msg))
            }
        }
    }

    /// Forces this `UtabManager` to operate in read-only mode.
    pub fn enable_read_only(&mut self) -> Result<(), UtabManagerError> {
        log::debug!("UtabManager::enable_read_only enabling read only mode");

        Self::set_force_read_only(self.ptr, true)
    }

    /// Disables read-only mode.
    pub fn disable_read_only(&mut self) -> Result<(), UtabManagerError> {
        log::debug!("UtabManager::disable_read_only disabling read only mode");

        Self::set_force_read_only(self.ptr, false)
    }

    fn set_entry(
        manager: *mut libmount::libmnt_update,
        flags: u64,
        target: *const libc::c_char,
        entry: *mut libmount::libmnt_fs,
    ) -> Result<(), UtabManagerError> {
        let result = unsafe { libmount::mnt_update_set_fs(manager, flags, target, entry) };

        match result {
            0 => {
                log::debug!("UtabManager::set_entry set entry to update");

                Ok(())
            }
            1 => {
                log::debug!("UtabManager::set_entry unnecessary to update entry");

                Ok(())
            }
            code => {
                let err_msg = "failed to set entry to update".to_owned();
                log::debug!("UtabManager::set_entry {}. libmount::mnt_update_set_fs returned error code: {:?}", err_msg, code);

                Err(UtabManagerError::Config(err_msg))
            }
        }
    }

    /// Sets the mount table entry to insert/update.
    pub fn set_mount_table_entry(
        &mut self,
        entry: UTabEntry,
        mount_flags: Vec<MountFlag>,
    ) -> Result<(), UtabManagerError> {
        let flags = mount_flags.iter().fold(0, |acc, &flag| acc | (flag as u64));
        log::debug!(
            "UtabManager::set_mount_table_entry setting mount table entry with flags: {:?}",
            mount_flags
        );

        Self::set_entry(self.ptr, flags, std::ptr::null_mut(), entry.inner)
    }

    /// Set the umount target to insert/update.
    pub fn set_umount_target<T>(
        &mut self,
        target: T,
        mount_flags: Vec<MountFlag>,
    ) -> Result<(), UtabManagerError>
    where
        T: AsRef<Path>,
    {
        let target = target.as_ref();
        let target_cstr = ffi_utils::as_ref_path_to_c_string(target)?;
        let flags = mount_flags.iter().fold(0, |acc, &flag| acc | (flag as u64));
        log::debug!(
            "UtabManager::set_umount_target setting umount target: {:?} with flags: {:?}",
            target,
            mount_flags
        );

        Self::set_entry(self.ptr, flags, target_cstr.as_ptr(), std::ptr::null_mut())
    }

    #[doc(hidden)]
    /// Applies mount table updates.
    fn apply_update(
        manager: *mut libmount::libmnt_update,
        lock: *mut libmount::libmnt_lock,
    ) -> Result<(), UtabManagerError> {
        let result = unsafe { libmount::mnt_update_table(manager, lock) };

        match result {
            0 => {
                log::debug!("UtabManager::apply_update applied mount table update");

                Ok(())
            }
            code => {
                let err_msg = "failed to apply mount table update".to_owned();
                log::debug!("UtabManager::apply_update {}. libmount::mnt_update_table returned error code: {:?}", err_msg, code);

                Err(UtabManagerError::Action(err_msg))
            }
        }
    }

    /// Updates the mount table. This function will automatically create a default [`FileLock`]
    /// that blocks all signals over the userspace mount table.
    pub fn update_mount_table(&mut self) -> Result<(), UtabManagerError> {
        Self::apply_update(self.ptr, std::ptr::null_mut())
    }

    /// Updates the userspace mount table using a manually configured [`FileLock`].
    pub fn lock_and_update_mount_table(&mut self, lock: FileLock) -> Result<(), UtabManagerError> {
        Self::apply_update(self.ptr, lock.ptr)
    }

    /// Returns the file name of the userspace mount table, or `None` on error.
    pub fn mount_table_file_name(&self) -> Option<PathBuf> {
        log::debug!("UtabManager::mount_table_file getting mount table file name");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_update_get_filename(self.ptr));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to get mount table file name";
                log::debug!("UtabManager::mount_table_file_name {}. libmount::mnt_update_get_filename returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                let file_name = ffi_utils::const_c_char_array_to_path_buf(ptr);
                log::debug!(
                    "UtabManager::mount_table_file got mount table file name: {:?}",
                    file_name
                );

                Some(file_name)
            }
        }
    }

    /// Returns a reference to the table entry to update set by [`UtabManager::set_mount_table_entry`].
    pub fn table_entry(&self) -> Option<&UTabEntry> {
        log::debug!("UtabManager::table_entry getting mount table entry to update");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_update_get_fs(self.ptr));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "no table entry to update";
                log::debug!("UtabManager::table_entry {}. libmount::mnt_update_get_fs returned a NULL pointer", err_msg);

                None
            }
            ptr => {
                log::debug!("UtabManager::table_entry got mount table entry to update");
                let entry = owning_ref_from_ptr!(self, UTabEntry, ptr);

                Some(entry)
            }
        }
    }

    /// Returns the mount flags set by [`UtabManager::set_mount_table_entry`] or
    /// [`UtabManager::set_umount_target`].
    pub fn mount_flags(&self) -> Option<Vec<MountFlag>> {
        log::debug!("UtabManager::mount_flags getting mount flags");

        let mut bits = MaybeUninit::<u64>::zeroed();

        unsafe {
            bits.write(libmount::mnt_update_get_mflags(self.ptr));
        }

        match unsafe { bits.assume_init() } {
            0 => {
                log::debug!("UtabManager::mount_flags no mount flags set");

                None
            }
            bits => {
                log::debug!("UtabManager::mount_flags got mount flags");
                let flags: Vec<_> = all::<MountFlag>()
                    .filter(|&flag| bits & (flag as u64) != 0)
                    .collect();

                Some(flags)
            }
        }
    }

    /// Returns `true` if this `UtabManager` is correctly configured.
    pub fn is_ready_for_update(&self) -> bool {
        let state = unsafe { libmount::mnt_update_is_ready(self.ptr) == 1 };
        log::debug!("UtabManager::is_ready_for_update value: {:?}", state);

        state
    }
}

impl AsRef<UtabManager> for UtabManager {
    #[inline]
    fn as_ref(&self) -> &UtabManager {
        self
    }
}

impl Drop for UtabManager {
    fn drop(&mut self) {
        log::debug!("UtabManager::drop deallocating `UtabManager` instance");

        unsafe { libmount::mnt_free_update(self.ptr) }
    }
}
