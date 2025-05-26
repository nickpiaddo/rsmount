// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::mem::MaybeUninit;
use std::path::Path;
use std::str::FromStr;

// From this library
use crate::core::cache::Cache;
use crate::core::device::Source;
use crate::core::errors::MountInfoEntryError;
use crate::core::flags::MountFlag;
use crate::core::fs::FileSystem;
use crate::ffi_utils;

/// A line in `/proc/<pid>/mountinfo` (where `<pid>` is the ID of a process).
///
/// For example:
///
/// ```text
/// 26 1 8:3 / / rw,relatime - ext4 /dev/sda3 rw
/// ```
#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct MountInfoEntry {
    pub(crate) inner: *mut libmount::libmnt_fs,
}

impl Drop for MountInfoEntry {
    fn drop(&mut self) {
        log::debug!("MountInfoEntry::drop deallocating `MountInfoEntry` instance");

        unsafe { libmount::mnt_unref_fs(self.inner) }
    }
}

impl MountInfoEntry {
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
    pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_fs) -> MountInfoEntry {
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
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_fs) -> MountInfoEntry {
        Self { inner: ptr }
    }

    #[doc(hidden)]
    /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe reference.
    pub(crate) unsafe fn ref_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_fs>,
    ) -> (*mut *mut libmount::libmnt_fs, &'a MountInfoEntry) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &*(raw_ptr as *const _ as *const MountInfoEntry) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe mutable reference.
    pub(crate) unsafe fn mut_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_fs>,
    ) -> (*mut *mut libmount::libmnt_fs, &'a mut MountInfoEntry) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &mut *(raw_ptr as *mut MountInfoEntry) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Creates a new instance.
    pub(crate) fn new() -> Result<MountInfoEntry, MountInfoEntryError> {
        log::debug!("MountInfoEntry::new creating a new `MountInfoEntry` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        unsafe {
            inner.write(libmount::mnt_new_fs());
        }

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `MountInfoEntry` instance".to_owned();
                log::debug!(
                    "MountInfoEntry::new {err_msg}. libmount::mnt_new_fs returned a NULL pointer"
                );

                Err(MountInfoEntryError::Creation(err_msg))
            }
            inner => {
                log::debug!("MountInfoEntry::new created a new `MountInfoEntry` instance");
                let entry = Self { inner };

                Ok(entry)
            }
        }
    }

    //---- BEGIN getters

    /// Returns the file system type specified at creation.
    pub fn file_system_type(&self) -> Option<FileSystem> {
        log::debug!("MountInfoEntry::file_system_type getting file system type");

        let mut fs_ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            fs_ptr.write(libmount::mnt_fs_get_fstype(self.inner));
        }

        match unsafe { fs_ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "MountInfoEntry::file_system_type failed to get file system type. libmount::mnt_fs_get_fstype returned a NULL pointer");

                None
            }
            fs_ptr => {
                let fs_type = ffi_utils::const_char_array_to_str_ref(fs_ptr);
                log::debug!("MountInfoEntry::file_system_type value: {:?}", fs_type);

                fs_type.ok().and_then(|s| FileSystem::from_str(s).ok())
            }
        }
    }

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

    /// Returns the entry's source path which can be
    /// - a directory for bind mounts (in `/etc/fstab` or `/etc/mtab` only)
    /// - a path to a block device for standard mounts.
    pub fn source_path(&self) -> Option<&Path> {
        log::debug!("MountInfoEntry::source_path getting the mount's source path");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_srcpath(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "MountInfoEntry::source_path failed to get the mount's source path. libmount::mnt_fs_get_srcpath returned a NULL pointer");

                None
            }

            ptr => {
                let path = ffi_utils::const_c_char_array_to_path(ptr);
                log::debug!("MountInfoEntry::source_path value: {:?}", path);

                Some(path)
            }
        }
    }

    /// Returns the path to the mount point.
    pub fn target(&self) -> Option<&Path> {
        log::debug!("MountInfoEntry::target getting path to mount point");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_target(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "MountInfoEntry::target failed to get path to mount point. libmount::mnt_fs_get_target returned a NULL pointer");

                None
            }
            ptr => {
                let path = ffi_utils::const_c_char_array_to_path(ptr);
                log::debug!("MountInfoEntry::target value: {:?}", path);

                Some(path)
            }
        }
    }

    //---- END getters

    //---- BEGIN setters

    /// Sets the file stream to print debug messages to.
    pub fn print_debug_to(&mut self, stream: &mut File) -> Result<(), MountInfoEntryError> {
        log::debug!(
            "MountInfoEntry::print_debug_to setting file stream to print debug messages to"
        );

        if ffi_utils::is_open_write_only(stream)? || ffi_utils::is_open_read_write(stream)? {
            let file_stream = ffi_utils::write_only_c_file_stream_from(stream)?;

            let result = unsafe { libmount::mnt_fs_print_debug(self.inner, file_stream as *mut _) };
            match result {
                0 => {
                    log::debug!(
                        "MountInfoEntry::print_debug_to set file stream to print debug messages to"
                    );

                    Ok(())
                }
                code => {
                    let err_msg = "failed to set file stream to print debug messages to".to_owned();
                    log::debug!( "MountInfoEntry::print_debug_to {err_msg}. libmount::mnt_fs_print_debug returned error code: {code:?}");

                    Err(MountInfoEntryError::Action(err_msg))
                }
            }
        } else {
            let err_msg = "missing write permission for given stream".to_owned();
            log::debug!("MountInfoEntry::print_debug_to {err_msg}");

            Err(MountInfoEntryError::Permission(err_msg))
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
    pub(crate) fn set_mount_source<T>(&mut self, source: T) -> Result<(), MountInfoEntryError>
    where
        T: AsRef<str>,
    {
        let source = source.as_ref();
        log::debug!(
            "MountInfoEntry::set_mount_source setting the source of a device to mount: {source:?}"
        );

        let source_cstr = ffi_utils::as_ref_str_to_c_string(source)?;

        let result = unsafe { libmount::mnt_fs_set_source(self.inner, source_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
 "MountInfoEntry::set_mount_source set the source of a device to mount: {source:?}"
                        );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to set the source of a device to mount: {:?}",
                    source
                );
                log::debug!( "MountInfoEntry::set_mount_source {err_msg}. libmount::mnt_fs_set_source returned error code: {code:?}");

                Err(MountInfoEntryError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Sets a device's mount point.
    pub(crate) fn set_mount_target<T>(&mut self, path: T) -> Result<(), MountInfoEntryError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "MountInfoEntry::set_mount_target setting device mount point to: {:?}",
            path
        );

        let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;

        let result = unsafe { libmount::mnt_fs_set_target(self.inner, path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "MountInfoEntry::set_mount_target set device mount point to: {:?}",
                    path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set a device's mount point to: {:?}", path);
                log::debug!( "MountInfoEntry::set_mount_target {err_msg}. libmount::mnt_fs_set_target returned error code: {code:?}");

                Err(MountInfoEntryError::Config(err_msg))
            }
        }
    }

    //---- END setters

    //---- BEGIN mutators

    /// Allocates a new `MountInfoEntry`, and a copies all the source's fields to the new
    /// instance except any private user data.
    pub fn copy(&self) -> Result<MountInfoEntry, MountInfoEntryError> {
        log::debug!("MountInfoEntry::copy copying `MountInfoEntry`");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_copy_fs(std::ptr::null_mut(), self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to copy `MountInfoEntry`".to_owned();
                log::debug!(
                    "MountInfoEntry::copy {err_msg}. libmount::mnt_copy_fs returned a NULL pointer"
                );

                Err(MountInfoEntryError::Action(err_msg))
            }
            ptr => {
                log::debug!("MountInfoEntry::copy copied `MountInfoEntry`");
                let entry = Self::from_ptr(ptr);

                Ok(entry)
            }
        }
    }

    //---- END mutators

    //---- BEGIN predicates

    /// Returns `true` if the file system type of this `MountInfoEntry` matches any element of the
    /// comma-separated file system names in the `pattern` parameter (see the [`FileSystem`
    /// documentation page](crate::core::fs::FileSystem) for a list of supported file systems).
    ///
    /// **Note:**
    /// - file system names prefixed with a `no` will match if this `MountInfoEntry` does **NOT** have
    ///   the file system mentioned.
    /// - a test with a pattern list starting with `no` will apply the prefix to **all** file
    ///   systems in the list (e.g. `"noapfs,ext4"` is equivalent to `"noapfs,noext4"`).
    ///
    /// For example, if this `MountInfoEntry` represents an `ext4` device, a test with the following
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
            log::debug!( "MountInfoEntry::has_any_fs_type does any element of the pattern list {:?} match? {:?}", pattern, state);

            state
        } else {
            log::debug!("MountInfoEntry::has_any_fs_type failed to convert pattern to `CString`");

            false
        }
    }

    /// Returns `true` if data is read directly from the kernel (e.g `/proc/mounts`).
    pub fn is_from_kernel(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_kernel(self.inner) == 1 };
        log::debug!("MountInfoEntry::is_from_kernel value: {:?}", state);

        state
    }

    /// Returns `true` if the file system of this `MountInfoEntry` is a network file system.
    pub fn is_net_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_netfs(self.inner) == 1 };
        log::debug!("MountInfoEntry::is_net_fs value: {:?}", state);

        state
    }

    /// Returns `true` if the file system of this `MountInfoEntry` is a pseudo file system type
    /// (`proc`, `cgroups`).
    pub fn is_pseudo_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_pseudofs(self.inner) == 1 };
        log::debug!("MountInfoEntry::is_pseudo_fs value: {:?}", state);

        state
    }

    #[cfg(mount = "v2_39")]
    /// Returns `true` if the file system of this `MountInfoEntry` is a regular file system (neither a
    /// network nor a pseudo file system).
    pub fn is_regular_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_regularfs(self.inner) == 1 };
        log::debug!("MountInfoEntry::is_regular_fs value: {:?}", state);

        state
    }

    /// Returns `true` if this `MountInfoEntry` represents a swap partition.
    pub fn is_swap(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_swaparea(self.inner) == 1 };
        log::debug!("MountInfoEntry::is_swap value: {:?}", state);

        state
    }

    /// Returns `true` if the `source` parameter matches the `source` field in this `MountInfoEntry`.
    ///
    /// Using the provided `cache`, this method will perform the following comparisons in sequence:
    /// - `source` vs the value of the `source` field in this `MountInfoEntry`
    ///
    /// - the resolved value of the `source` parameter vs the value of the `source` field in this
    ///   `MountInfoEntry`
    /// - the resolved value of the `source` parameter vs the resolved value of the `source` field
    ///   in this `MountInfoEntry`
    /// - the resolved value of the `source` parameter vs the evaluated tag of the `source` field
    ///   in this `MountInfoEntry`
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
                "MountInfoEntry::is_source is {:?} the source of this entry? {:?}",
                source,
                state
            );

            state
        } else {
            log::debug!("MountInfoEntry::is_source failed to convert source to `CString`");

            false
        }
    }

    /// Returns `true` if the `source` parameter matches exactly the `source` field in this
    ///   `MountInfoEntry`
    ///
    /// **Note:** redundant forward slashes are ignored when comparing values.
    pub fn is_exact_source(&self, source: &Source) -> bool {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok();

        if let Some(source_cstr) = source_cstr {
            let state =
                unsafe { libmount::mnt_fs_streq_srcpath(self.inner, source_cstr.as_ptr()) == 1 };
            log::debug!(
                "MountInfoEntry::is_exact_source is {:?} the exact source of this entry? {:?}",
                source,
                state
            );

            state
        } else {
            log::debug!("MountInfoEntry::is_exact_source failed to convert source to `CString`");

            false
        }
    }

    /// Returns `true` if `path` matches the `target` field in this `MountInfoEntry`. Using
    /// the provided `cache`, this method will perform the following comparisons in sequence:
    ///
    /// - `path` vs the value of the `target` field in this `MountInfoEntry`
    /// - canonicalized `path` vs the value of the `target` field in this `MountInfoEntry`
    /// - canonicalized `path` vs the canonicalized value of the `target` field in this
    ///   `MountInfoEntry` if is not from `/proc/self/mountinfo`
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
                "MountInfoEntry::is_target is {:?} the target of this entry? {:?}",
                path,
                state
            );

            state
        } else {
            log::debug!("MountInfoEntry::is_target failed to convert path to `CString`");

            false
        }
    }

    /// Returns `true` if `path` matches **exactly** the `target` field in this `MountInfoEntry`.
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
                "MountInfoEntry::is_exact_target is {:?} the exact target of this entry? {:?}",
                path,
                state
            );

            state
        } else {
            log::debug!("MountInfoEntry::is_exact_target failed to convert path to `CString`");

            false
        }
    }

    //---- END predicates
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
