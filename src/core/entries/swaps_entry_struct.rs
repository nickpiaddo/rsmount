// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::fs::File;
use std::mem::MaybeUninit;
use std::path::Path;

// From this library
use crate::core::cache::Cache;
use crate::core::device::Source;
use crate::core::errors::SwapsEntryError;
use crate::ffi_utils;

/// A line in `/proc/swaps`.
///
/// For example:
///
/// ```text
/// /dev/sda2                               partition       1048572         0               -2
/// ```
#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct SwapsEntry {
    pub(crate) inner: *mut libmount::libmnt_fs,
}

impl Drop for SwapsEntry {
    fn drop(&mut self) {
        log::debug!("SwapsEntry::drop deallocating `SwapsEntry` instance");

        unsafe { libmount::mnt_unref_fs(self.inner) }
    }
}

impl SwapsEntry {
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
    pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_fs) -> SwapsEntry {
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
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_fs) -> SwapsEntry {
        Self { inner: ptr }
    }

    #[doc(hidden)]
    /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe reference.
    pub(crate) unsafe fn ref_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_fs>,
    ) -> (*mut *mut libmount::libmnt_fs, &'a SwapsEntry) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &*(raw_ptr as *const _ as *const SwapsEntry) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe mutable reference.
    pub(crate) unsafe fn mut_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_fs>,
    ) -> (*mut *mut libmount::libmnt_fs, &'a mut SwapsEntry) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &mut *(raw_ptr as *mut SwapsEntry) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Creates a new instance.
    pub(crate) fn new() -> Result<SwapsEntry, SwapsEntryError> {
        log::debug!("SwapsEntry::new creating a new `SwapsEntry` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        unsafe {
            inner.write(libmount::mnt_new_fs());
        }

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `SwapsEntry` instance".to_owned();
                log::debug!(
                    "SwapsEntry::new {err_msg}. libmount::mnt_new_fs returned a NULL pointer"
                );

                Err(SwapsEntryError::Creation(err_msg))
            }
            inner => {
                log::debug!("SwapsEntry::new created a new `SwapsEntry` instance");
                let entry = Self { inner };

                Ok(entry)
            }
        }
    }

    //---- BEGIN getters

    /// Returns the entry's source path which can be
    /// - a directory for bind mounts (in `/etc/fstab` or `/etc/mtab` only)
    /// - a path to a block device for standard mounts.
    pub fn source_path(&self) -> Option<&Path> {
        log::debug!(concat!(
            stringify!($entry_type),
            "::source_path getting the mount's source path"
        ));

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_srcpath(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!(concat!(stringify!($entry_type), "::source_path failed to get the mount's source path. libmount::mnt_fs_get_srcpath returned a NULL pointer"));

                None
            }

            ptr => {
                let path = ffi_utils::const_c_char_array_to_path(ptr);
                log::debug!(
                    concat!(stringify!($entry_type), "::source_path value: {:?}"),
                    path
                );

                Some(path)
            }
        }
    }

    /// Returns the type of swap partition.
    pub fn swap_type(&self) -> Option<&str> {
        log::debug!("SwapsEntry::swap_type getting swap type");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_swaptype(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("SwapsEntry::swap_type failed to get swap type. libmount::mnt_fs_get_swaptype returned a NULL pointer");

                None
            }

            ptr => {
                let swap_type = ffi_utils::const_char_array_to_str_ref(ptr);
                log::debug!("SwapsEntry::swap_type value: {:?}", swap_type);

                swap_type.ok()
            }
        }
    }

    /// Returns the total size of the swap partition (in kibibytes).
    pub fn size(&self) -> usize {
        let size = unsafe { libmount::mnt_fs_get_size(self.inner) as usize };
        log::debug!("SwapsEntry::size value: {:?}", size);

        size
    }

    /// Returns the size of the swap space used (in kibibytes).
    pub fn size_used(&self) -> usize {
        let size = unsafe { libmount::mnt_fs_get_usedsize(self.inner) as usize };
        log::debug!("SwapsEntry::size_used size: {:?}", size);

        size
    }

    /// Returns the priority number of the swap partition.
    pub fn priority(&self) -> i32 {
        let priority = unsafe { libmount::mnt_fs_get_priority(self.inner) };
        log::debug!("SwapsEntry::priority value: {:?}", priority);

        priority
    }

    //---- END getters

    //---- BEGIN setters

    /// Sets the file stream to print debug messages to.
    pub fn print_debug_to(&mut self, stream: &mut File) -> Result<(), SwapsEntryError> {
        log::debug!("SwapsEntry::print_debug_to setting file stream to print debug messages to");

        if ffi_utils::is_open_write_only(stream)? || ffi_utils::is_open_read_write(stream)? {
            let file_stream = ffi_utils::write_only_c_file_stream_from(stream)?;

            let result = unsafe { libmount::mnt_fs_print_debug(self.inner, file_stream as *mut _) };
            match result {
                0 => {
                    log::debug!(
                        "SwapsEntry::print_debug_to set file stream to print debug messages to"
                    );

                    Ok(())
                }
                code => {
                    let err_msg = "failed to set file stream to print debug messages to".to_owned();
                    log::debug!( "SwapsEntry::print_debug_to {err_msg}. libmount::mnt_fs_print_debug returned error code: {code:?}");

                    Err(SwapsEntryError::Action(err_msg))
                }
            }
        } else {
            let err_msg = "missing write permission for given stream".to_owned();
            log::debug!("SwapsEntry::print_debug_to {err_msg}");

            Err(SwapsEntryError::Permission(err_msg))
        }
    }

    /// Sets the priority of the swap device; swap priority takes a value between `-1` and `32767`.
    ///
    /// Higher numbers indicate higher priority (for more information see the [`swapon` command's
    /// manpage](https://manpages.org/swapon/8)).
    pub fn set_priority(&mut self, priority: i32) -> Result<(), SwapsEntryError> {
        log::debug!(
            "SwapsEntry::set_priority setting swap priority to: {:?}",
            priority
        );

        let result = unsafe { libmount::mnt_fs_set_priority(self.inner, priority) };
        match result {
            0 => {
                log::debug!(
                    "SwapsEntry::set_priority set swap priority to: {:?}",
                    priority
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set swap priority to: {:?}", priority);
                log::debug!("SwapsEntry::set_priority {}. libmount::mnt_fs_set_priority returned error code: {:?}", err_msg, code);

                Err(SwapsEntryError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    #[allow(dead_code)]
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
    pub(crate) fn set_mount_source<T>(&mut self, source: T) -> Result<(), SwapsEntryError>
    where
        T: AsRef<str>,
    {
        let source = source.as_ref();
        log::debug!(
            "SwapsEntry::set_mount_source setting the source of a device to mount: {:?}",
            source
        );

        let source_cstr = ffi_utils::as_ref_str_to_c_string(source)?;

        let result = unsafe { libmount::mnt_fs_set_source(self.inner, source_cstr.as_ptr()) };
        match result {
            0 => {
                log::debug!(
                    "SwapsEntry::set_mount_source set the source of a device to mount: {:?}",
                    source
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to set the source of a device to mount: {:?}",
                    source
                );
                log::debug!( "SwapsEntry::set_mount_source {err_msg}. libmount::mnt_fs_set_source returned error code: {code:?}");

                Err(SwapsEntryError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Sets a device's mount point.
    pub(crate) fn set_mount_target<T>(&mut self, path: T) -> Result<(), SwapsEntryError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "SwapsEntry::set_mount_target setting device mount point to: {:?}",
            path
        );

        let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;

        let result = unsafe { libmount::mnt_fs_set_target(self.inner, path_cstr.as_ptr()) };
        match result {
            0 => {
                log::debug!(
                    "SwapsEntry::set_mount_target set device mount point to: {:?}",
                    path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set a device's mount point to: {:?}", path);
                log::debug!( "SwapsEntry::set_mount_target {err_msg}. libmount::mnt_fs_set_target returned error code: {code:?}");

                Err(SwapsEntryError::Config(err_msg))
            }
        }
    }

    //---- END setters

    //---- BEGIN mutators

    /// Allocates a new `SwapsEntry`, and a copies all the source's fields to the new instance except
    /// any private user data.
    pub fn copy(&self) -> Result<SwapsEntry, SwapsEntryError> {
        log::debug!("SwapsEntry::copy copying `SwapsEntry`");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_copy_fs(std::ptr::null_mut(), self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to copy `SwapsEntry`".to_owned();
                log::debug!(
                    "SwapsEntry::copy {err_msg}. libmount::mnt_copy_fs returned a NULL pointer"
                );

                Err(SwapsEntryError::Action(err_msg))
            }
            ptr => {
                log::debug!("SwapsEntry::copy copied `SwapsEntry`");
                let entry = Self::from_ptr(ptr);

                Ok(entry)
            }
        }
    }

    //---- END mutators

    //---- BEGIN predicates

    /// Returns `true` if data is read directly from the kernel (e.g `/proc/mounts`).
    pub fn is_from_kernel(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_kernel(self.inner) == 1 };
        log::debug!("SwapsEntry::is_from_kernel value: {:?}", state);

        state
    }

    /// Returns `true` if the file system of this `SwapsEntry` is a network file system.
    pub fn is_net_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_netfs(self.inner) == 1 };
        log::debug!("SwapsEntry::is_net_fs value: {:?}", state);

        state
    }

    /// Returns `true` if the file system of this `SwapsEntry` is a pseudo file system type (`proc`, `cgroups`).
    pub fn is_pseudo_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_pseudofs(self.inner) == 1 };
        log::debug!("SwapsEntry::is_pseudo_fs value: {:?}", state);

        state
    }

    #[cfg(mount = "v2_39")]
    /// Returns `true` if the file system of this `SwapsEntry` is a regular file system (neither a network nor a pseudo file system).
    pub fn is_regular_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_regularfs(self.inner) == 1 };
        log::debug!("SwapsEntry::is_regular_fs value: {:?}", state);

        state
    }

    /// Returns `true` if this `SwapsEntry` represents a swap partition.
    pub fn is_swap(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_swaparea(self.inner) == 1 };
        log::debug!("SwapsEntry::is_swap value: {:?}", state);

        state
    }

    /// Returns `true` if the `source` parameter matches the `source` field in this `SwapsEntry`.
    ///
    /// Using the provided `cache`, this method will perform the following comparisons in sequence:
    /// - `source` vs the value of the `source` field in this `SwapsEntry`
    ///
    /// - the resolved value of the `source` parameter vs the value of the `source` field in this
    ///   `SwapsEntry`
    /// - the resolved value of the `source` parameter vs the resolved value of the `source` field
    ///   in this `SwapsEntry`
    /// - the resolved value of the `source` parameter vs the evaluated tag of the `source` field
    ///   in this `SwapsEntry`
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
                "SwapsEntry::is_source is {:?} the source of this entry? {:?}",
                source,
                state
            );

            state
        } else {
            log::debug!("SwapsEntry::is_source failed to convert source to `CString`");

            false
        }
    }

    /// Returns `true` if the `source` parameter matches exactly the `source` field in this
    /// `SwapsEntry`
    ///
    /// **Note:** redundant forward slashes are ignored when comparing values.
    pub fn is_exact_source(&self, source: &Source) -> bool {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok();

        if let Some(source_cstr) = source_cstr {
            let state =
                unsafe { libmount::mnt_fs_streq_srcpath(self.inner, source_cstr.as_ptr()) == 1 };
            log::debug!(
                "SwapsEntry::is_exact_source is {:?} the exact source of this entry? {:?}",
                source,
                state
            );

            state
        } else {
            log::debug!("SwapsEntry::is_exact_source failed to convert source to `CString`");

            false
        }
    }

    //---- END predicates
}

impl fmt::Display for SwapsEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formatting from Linux Kernel
        // linux/mm/swapfile.c
        // 2702:           seq_puts(swap, "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n");
        let mut output: Vec<String> = vec![];
        if let Some(path) = self.source_path() {
            let source_path = format!("{}\t\t", path.display());
            output.push(source_path.to_string());
        }

        if let Some(swap_type) = self.swap_type() {
            output.push(swap_type.to_string());
        }

        let size = self.size();
        output.push(size.to_string());

        let size_used = self.size_used();
        output.push(size_used.to_string());

        let priority = self.priority();
        output.push(priority.to_string());

        write!(f, "{}", output.join("\t\t"))
    }
}
