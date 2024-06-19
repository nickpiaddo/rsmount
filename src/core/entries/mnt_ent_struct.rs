// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::path::{Path, PathBuf};

// From this library
use crate::ffi_utils;

/// A wrapper around libc's `struct mntent`.
///
/// (see this [`documentation page`](https://www.gnu.org/software/libc/manual/html_node/mtab.html) for more information).
#[derive(Debug)]
pub struct MntEnt {
    inner: *mut libmount::mntent,
    fs_name: PathBuf,
    mount_point: PathBuf,
    fs_type: String,
    options: String,
    backup_frequency: i32,
    pass_number: i32,
}

impl MntEnt {
    #[doc(hidden)]
    /// Wraps a raw `libmount::mntent` pointer with a safe `MntEnt`.
    #[allow(dead_code)]
    pub(crate) fn from_raw_parts(ptr: *mut libmount::mntent) -> MntEnt {
        let fs_name = unsafe { ffi_utils::const_c_char_array_to_path_buf((*ptr).mnt_fsname) };
        let mount_point = unsafe { ffi_utils::const_c_char_array_to_path_buf((*ptr).mnt_dir) };
        let fs_type = unsafe { ffi_utils::c_char_array_to_string((*ptr).mnt_type) };

        let options = unsafe { ffi_utils::c_char_array_to_string((*ptr).mnt_opts) };
        let backup_frequency = unsafe { (*ptr).mnt_freq };
        let pass_number = unsafe { (*ptr).mnt_passno };

        Self {
            inner: ptr,
            fs_name,
            mount_point,
            fs_type,
            options,
            backup_frequency,
            pass_number,
        }
    }

    /// Returns the name of the special device from which the file system is mounted.
    pub fn device_name(&self) -> &Path {
        &self.fs_name
    }

    /// Returns the file system type, one of:
    /// - `"ignore"`: the value is sometimes used in `fstab` files to make sure entries are not used without removing them,
    /// - `"nfs"`: default NFS implementation,
    /// - `"swap"`: one of the possibly multiple swap partitions.
    pub fn file_system_type(&self) -> &str {
        &self.fs_type
    }

    /// Returns the mount point of the file system.
    pub fn mount_point(&self) -> &Path {
        &self.mount_point
    }

    /// Returns the options used while mounting the file system.
    pub fn mount_options(&self) -> &str {
        &self.options
    }

    /// Returns the frequency in days in which dumps are made.
    pub fn backup_frequency(&self) -> i32 {
        self.backup_frequency
    }

    /// Returns the pass number on parallel `fsck`.
    pub fn pass_number(&self) -> i32 {
        self.pass_number
    }
}

impl Drop for MntEnt {
    fn drop(&mut self) {
        log::debug!("MntEnt::drop deallocating `MntEnt` instance");

        unsafe { libmount::mnt_free_mntent(self.inner) }
    }
}
