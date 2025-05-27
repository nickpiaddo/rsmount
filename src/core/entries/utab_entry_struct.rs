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
use crate::core::entries::UTabEntryBuilder;
use crate::core::entries::UTbEntBuilder;
use crate::core::errors::UTabEntryError;
use crate::core::utils;
use crate::ffi_utils;

/// A line in `/run/mount/utab`.
///
/// For example:
/// ```text
/// SRC=/dev/vda TARGET=/mnt ROOT=/ OPTS=x-initrd.mount
/// ```
#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct UTabEntry {
    pub(crate) inner: *mut libmount::libmnt_fs,
}

impl Drop for UTabEntry {
    fn drop(&mut self) {
        log::debug!("UTabEntry::drop deallocating `UTabEntry` instance");

        unsafe { libmount::mnt_unref_fs(self.inner) }
    }
}

impl UTabEntry {
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
    pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_fs) -> UTabEntry {
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
    pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_fs) -> UTabEntry {
        Self { inner: ptr }
    }

    #[doc(hidden)]
    /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe reference.
    pub(crate) unsafe fn ref_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_fs>,
    ) -> (*mut *mut libmount::libmnt_fs, &'a UTabEntry) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &*(raw_ptr as *const _ as *const UTabEntry) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe mutable reference.
    pub(crate) unsafe fn mut_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_fs>,
    ) -> (*mut *mut libmount::libmnt_fs, &'a mut UTabEntry) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &mut *(raw_ptr as *mut UTabEntry) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    /// Creates a new instance.
    pub(crate) fn new() -> Result<UTabEntry, UTabEntryError> {
        log::debug!("UTabEntry::new creating a new `UTabEntry` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
        unsafe {
            inner.write(libmount::mnt_new_fs());
        }

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `UTabEntry` instance".to_owned();
                log::debug!(
                    "UTabEntry::new {err_msg}. libmount::mnt_new_fs returned a NULL pointer"
                );

                Err(UTabEntryError::Creation(err_msg))
            }
            inner => {
                log::debug!("UTabEntry::new created a new `UTabEntry` instance");
                let entry = Self { inner };

                Ok(entry)
            }
        }
    }

    /// Creates a [`UTabEntryBuilder`] to configure and construct a new `UTabEntry`.
    ///
    /// Call the `UTabEntryBuilder`'s [`build()`](UTabEntryBuilder::build) method to
    /// construct a new `UTabEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// use rsmount::core::entries::UTabEntry;
    ///
    /// fn main() -> rsmount::Result<()> {
    ///     // Example entry in /etc/fstab
    ///     //
    ///     // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
    ///     let entry = UTabEntry::builder()
    ///         .source("/dev/vda1")
    ///         .target("/")
    ///         // Comma-separated list of mount options.
    ///         .mount_options("rw,relatime")
    ///         .build()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn builder() -> UTabEntryBuilder {
        log::debug!("UTabEntry::builder creating new `UTabEntryBuilder` instance");
        UTbEntBuilder::builder()
    }

    //---- BEGIN getters

    /// Returns the pathname referring to a device, a directory or file, or a dummy string used to
    /// create a bind mount.
    pub fn bind_source(&self) -> Option<&str> {
        log::debug!("UTabEntry::bind_source getting bind mount source");

        let mut source_ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            source_ptr.write(libmount::mnt_fs_get_bindsrc(self.inner));
        }

        match unsafe { source_ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("UTabEntry::bind_source no bind mount source set. libmount::mnt_fs_get_bindsrc returned a NULL pointer");

                None
            }
            source_ptr => {
                let bind_source = ffi_utils::const_char_array_to_str_ref(source_ptr);
                log::debug!("UTabEntry::bind_source value: {:?}", bind_source);

                bind_source.ok()
            }
        }
    }

    /// Returns all mount attributes stored in `/run/mount/utab`
    pub fn attributes(&self) -> Option<&str> {
        log::debug!("UTabEntry::attributes getting mount attributes");

        let mut attrs_ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            attrs_ptr.write(libmount::mnt_fs_get_attributes(self.inner));
        }

        match unsafe { attrs_ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("UTabEntry::attributes no attributes set. libmount::mnt_fs_get_attributes returned a NULL pointer");

                None
            }
            attrs_ptr => {
                let attributes = ffi_utils::const_char_array_to_str_ref(attrs_ptr);
                log::debug!("UTabEntry::attributes value: {:?}", attributes);

                attributes.ok()
            }
        }
    }

    /// Returns the value of the `utab` attribute matching `attr_name`.
    pub fn attribute_value<T>(&self, attr_name: T) -> Option<String>
    where
        T: AsRef<str>,
    {
        let attr_name = attr_name.as_ref();
        log::debug!(
            "UTabEntry::attribute_value getting value of attribute: {:?}",
            attr_name
        );

        let attr_cstr = ffi_utils::as_ref_str_to_c_string(attr_name).ok()?;

        let mut value_start = MaybeUninit::<*mut libc::c_char>::zeroed();
        let mut size = MaybeUninit::<libc::size_t>::zeroed();

        let result = unsafe {
            libmount::mnt_fs_get_attribute(
                self.inner,
                attr_cstr.as_ptr(),
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
                    "UTabEntry::attribute_value attribute {:?} has value {:?}",
                    attr_name,
                    value
                );

                Some(value)
            }
            1 => {
                log::debug!(
                    "UTabEntry::attribute_value found no attribute matching {:?}",
                    attr_name
                );

                None
            }
            code => {
                log::debug!(
                        "UTabEntry::attribute_value failed to get value of attribute: {:?}. libmount::mnt_fs_get_attribute returned error code: {:?}",
                        attr_name,
                        code
                    );

                None
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

    /// Returns the value of the option matching `option_name`.
    pub fn option_value<T>(&self, option_name: T) -> Option<String>
    where
        T: AsRef<str>,
    {
        let option_name = option_name.as_ref();
        log::debug!(
            "UTabEntry::option_value getting value of option: {:?}",
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
                    "UTabEntry::option_value option {:?} has value {:?}",
                    option_name,
                    value
                );

                Some(value)
            }
            1 => {
                log::debug!(
                    "UTabEntry::option_value found no option matching {:?}",
                    option_name
                );

                None
            }
            code => {
                log::debug!(
 "UTabEntry::option_value failed to get value of option: {:?}. libmount::mnt_fs_get_option returned error code: {:?}",
                            option_name,
                            code
                        );

                None
            }
        }
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

    /// Returns userspace mount options.
    pub fn mount_options(&self) -> Option<&str> {
        log::debug!("UTabEntry::mount_options getting user options");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_user_options(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("UTabEntry::mount_options failed to get user options. libmount::mnt_fs_get_user_options returned a NULL pointer");

                None
            }
            ptr => {
                let options = ffi_utils::const_char_array_to_str_ref(ptr);
                log::debug!("UTabEntry::mount_options value: {:?}", options);

                options.ok()
            }
        }
    }

    /// Returns the entry's source path which can be
    /// - a directory for bind mounts (in `/etc/fstab` or `/etc/mtab` only)
    /// - a path to a block device for standard mounts.
    pub fn source_path(&self) -> Option<&Path> {
        log::debug!("UTabEntry::source_path getting the mount's source path");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_srcpath(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "UTabEntry::source_path failed to get the mount's source path. libmount::mnt_fs_get_srcpath returned a NULL pointer");

                None
            }

            ptr => {
                let path = ffi_utils::const_c_char_array_to_path(ptr);
                log::debug!("UTabEntry::source_path value: {:?}", path);

                Some(path)
            }
        }
    }

    /// Returns the path to the mount point.
    pub fn target(&self) -> Option<&Path> {
        log::debug!("UTabEntry::target getting path to mount point");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_target(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!( "UTabEntry::target failed to get path to mount point. libmount::mnt_fs_get_target returned a NULL pointer");

                None
            }
            ptr => {
                let path = ffi_utils::const_c_char_array_to_path(ptr);
                log::debug!("UTabEntry::target value: {:?}", path);

                Some(path)
            }
        }
    }

    //---- END getters

    //---- BEGIN setters

    /// Sets the file stream to print debug messages to.
    pub fn print_debug_to(&mut self, stream: &mut File) -> Result<(), UTabEntryError> {
        log::debug!("UTabEntry::print_debug_to setting file stream to print debug messages to");

        if ffi_utils::is_open_write_only(stream)? || ffi_utils::is_open_read_write(stream)? {
            let file_stream = ffi_utils::write_only_c_file_stream_from(stream)?;

            let result = unsafe { libmount::mnt_fs_print_debug(self.inner, file_stream as *mut _) };
            match result {
                0 => {
                    log::debug!(
                        "UTabEntry::print_debug_to set file stream to print debug messages to"
                    );

                    Ok(())
                }
                code => {
                    let err_msg = "failed to set file stream to print debug messages to".to_owned();
                    log::debug!( "UTabEntry::print_debug_to {err_msg}. libmount::mnt_fs_print_debug returned error code: {code:?}");

                    Err(UTabEntryError::Action(err_msg))
                }
            }
        } else {
            let err_msg = "missing write permission for given stream".to_owned();
            log::debug!("UTabEntry::print_debug_to {err_msg}");

            Err(UTabEntryError::Permission(err_msg))
        }
    }

    /// Sets `utab` attributes, which are options independent from those used by the [`mount`
    /// syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html) and [`mount`
    /// command](https://www.man7.org/linux/man-pages/man8/mount.8.html). They are neither sent to
    /// the kernel, nor interpreted by `libmount`.
    ///
    /// They are stored in `/run/mount/utab`, and managed by `libmount` in userspace only.
    ///
    /// **Warning:** it's possible that information stored in userspace will not be available to
    /// `libmount` after the [`unshare`
    /// syscall](https://www.man7.org/linux/man-pages/man2/unshare.2.html) is invoked with the
    /// `CLONE_FS` flag. Be careful, and don't use attributes if possible."))]
    pub fn set_attributes<T>(&mut self, attributes: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<str>,
    {
        let attributes = attributes.as_ref();
        log::debug!(
            "UTabEntry::set_attributes setting mount table entry attributes: {:?}",
            attributes
        );

        let attrs = ffi_utils::as_ref_str_to_c_string(attributes)?;

        let result = unsafe { libmount::mnt_fs_set_attributes(self.inner, attrs.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "UTabEntry::set_attributes set mount table entry attributes: {:?}",
                    attributes
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to set mount table entry attributes: {:?}",
                    attributes
                );
                log::debug!("UTabEntry::set_attributes {}. libmount::mnt_fs_set_attributes returned error code: {:?}", err_msg, code);

                Err(UTabEntryError::Config(err_msg))
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
    pub(crate) fn set_mount_source<T>(&mut self, source: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<str>,
    {
        let source = source.as_ref();
        log::debug!(
            "UTabEntry::set_mount_source setting the source of a device to mount: {:?}",
            source
        );

        let source_cstr = ffi_utils::as_ref_str_to_c_string(source)?;

        let result = unsafe { libmount::mnt_fs_set_source(self.inner, source_cstr.as_ptr()) };
        match result {
            0 => {
                log::debug!(
                    "UTabEntry::set_mount_source set the source of a device to mount: {:?}",
                    source
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to set the source of a device to mount: {:?}",
                    source
                );
                log::debug!( "UTabEntry::set_mount_source {err_msg}. libmount::mnt_fs_set_source returned error code: {code:?}");

                Err(UTabEntryError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Sets a device's mount point.
    pub(crate) fn set_mount_target<T>(&mut self, path: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "UTabEntry::set_mount_target setting device mount point to: {:?}",
            path
        );

        let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;

        let result = unsafe { libmount::mnt_fs_set_target(self.inner, path_cstr.as_ptr()) };
        match result {
            0 => {
                log::debug!(
                    "UTabEntry::set_mount_target set device mount point to: {:?}",
                    path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set a device's mount point to: {:?}", path);
                log::debug!( "UTabEntry::set_mount_target {err_msg}. libmount::mnt_fs_set_target returned error code: {code:?}");

                Err(UTabEntryError::Config(err_msg))
            }
        }
    }

    // FIXME is this the right documentation?
    //
    // Sets the pathname of the directory a process sees as its root directory (conforms to the
    // `/proc/<pid>/mountinfo` file format, more information on `Mountinfo`'s documentation
    // page).
    pub fn set_root<T>(&mut self, path: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!("UTabEntry::set_root setting root to: {:?}", path);

        let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;

        let result = unsafe { libmount::mnt_fs_set_root(self.inner, path_cstr.as_ptr()) };
        match result {
            0 => {
                log::debug!("UTabEntry::set_root set the root to: {:?}", path);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set the root to: {:?}", path);
                log::debug!(
                    "UTabEntry::set_root {}. libmount::mnt_fs_set_root returned error code: {:?}",
                    err_msg,
                    code
                );

                Err(UTabEntryError::Config(err_msg))
            }
        }
    }

    /// Prepends `attributes` to the list of `utab` attributes.
    pub fn prepend_attributes<T>(&mut self, attributes: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<str>,
    {
        let attributes = attributes.as_ref();
        let attrs_cstr = ffi_utils::as_ref_str_to_c_string(attributes)?;

        log::debug!(
            "UTabEntry::prepend_attributes prepending attributes: {:?}",
            attributes
        );

        let result =
            unsafe { libmount::mnt_fs_prepend_attributes(self.inner, attrs_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "UTabEntry::prepend_attributes prepended attributes: {:?}",
                    attributes
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to prepend attributes: {:?}", attributes);
                log::debug!("UTabEntry::prepend_attributes {}. libmount::mnt_fs_prepend_attributes returned error code: {:?}", err_msg, code);

                Err(UTabEntryError::Config(err_msg))
            }
        }
    }

    /// Appends `attributes` to the list of `utab` attributes.
    pub fn append_attributes<T>(&mut self, attributes: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<str>,
    {
        let attributes = attributes.as_ref();
        log::debug!(
            "UTabEntry::append_attributes appending attributes {:?}",
            attributes
        );

        let comment_cstr = ffi_utils::as_ref_str_to_c_string(attributes)?;

        let result =
            unsafe { libmount::mnt_fs_append_attributes(self.inner, comment_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "UTabEntry::append_attributes appended attributes {:?}",
                    attributes
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to append attributes {:?}", attributes);
                log::debug!("UTabEntry::append_attributes {}. libmount::mnt_fs_append_attributes returned error code: {:?}", err_msg, code);

                Err(UTabEntryError::Config(err_msg))
            }
        }
    }

    /// Sets mount options string.
    pub fn set_mount_options<T>(&mut self, options: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<str>,
    {
        let options = options.as_ref();
        log::debug!(
            "UTabEntry::set_mount_options setting mount options string to: {:?}",
            options
        );

        let options_cstr = ffi_utils::as_ref_str_to_c_string(options)?;

        let result = unsafe { libmount::mnt_fs_set_options(self.inner, options_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "UTabEntry::set_mount_options set mount options string to: {:?}",
                    options
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set mount options string to: {:?}", options);
                log::debug!( "UTabEntry::set_mount_options {err_msg}. libmount::mnt_fs_set_options returned error code: {code:?}");

                Err(UTabEntryError::Config(err_msg))
            }
        }
    }

    /// Sets the source path of the device to mount.
    pub fn set_source_path<T>(&mut self, source: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<Path>,
    {
        let source = source.as_ref().display().to_string();
        log::debug!(
            "UTabEntry::set_source setting the source of a device to mount: {:?}",
            source
        );

        self.set_mount_source(source)
    }

    /// Sets the source directory of a bind mount .
    pub fn set_bind_source<T>(&mut self, path: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "UTabEntry::set_bind_source setting bind mount source directory as {:?}",
            path
        );

        let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;

        let result = unsafe { libmount::mnt_fs_set_bindsrc(self.inner, path_cstr.as_ptr()) };
        match result {
            0 => {
                log::debug!(
                    "UTabEntry::set_bind_source set bind mount source directory as {:?}",
                    path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set bind mount source directory as {:?}", path);
                log::debug!( "UTabEntry::set_bind_source {err_msg}. libmount::mnt_fs_set_bindsrc returned error code: {code:?}");

                Err(UTabEntryError::Config(err_msg))
            }
        }
    }

    /// Sets a device's mount point.
    pub fn set_target<T>(&mut self, path: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "UTabEntry::set_target setting device mount point to: {:?}",
            path
        );

        self.set_mount_target(path)
    }

    //---- END setters

    /// Fills the empty fields in `destination` by copying data from the corresponding fields in
    /// this object.
    pub fn complete(&mut self, destination: &mut UTabEntry) -> Result<(), UTabEntryError> {
        log::debug!("UTabEntry::complete copying fields to destination `UTabEntry`");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_copy_fs(destination.inner, self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to copy fields to destination `FsTabEntry`".to_owned();
                log::debug!(
                    "UTabEntry::complete {err_msg}. libmount::mnt_copy_fs returned a NULL pointer"
                );

                Err(UTabEntryError::Copy(err_msg))
            }
            _ptr => {
                log::debug!("UTabEntry::complete copied fields to destination `UTabEntry`");

                Ok(())
            }
        }
    }

    //---- BEGIN mutators

    /// Allocates a new `UTabEntry`, and a copies all the source's fields to the new
    /// instance except any private user data.
    pub fn copy(&self) -> Result<UTabEntry, UTabEntryError> {
        log::debug!("UTabEntry::copy copying `UTabEntry`");

        let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_copy_fs(std::ptr::null_mut(), self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = "failed to copy `UTabEntry`".to_owned();
                log::debug!(
                    "UTabEntry::copy {err_msg}. libmount::mnt_copy_fs returned a NULL pointer"
                );

                Err(UTabEntryError::Action(err_msg))
            }
            ptr => {
                log::debug!("UTabEntry::copy copied `UTabEntry`");
                let entry = Self::from_ptr(ptr);

                Ok(entry)
            }
        }
    }

    /// Prepends the `options` parameter to the corresponding VFS, `mountinfo` FS-specific, and userspace list of options.
    pub fn prepend_options<T>(&mut self, options: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<str>,
    {
        let options = options.as_ref();
        let attrs_cstr = ffi_utils::as_ref_str_to_c_string(options)?;

        log::debug!(
            "UTabEntry::prepend_options prepending options: {:?}",
            options
        );

        let result = unsafe { libmount::mnt_fs_prepend_options(self.inner, attrs_cstr.as_ptr()) };
        match result {
            0 => {
                log::debug!(
                    "UTabEntry::prepend_options prepended options: {:?}",
                    options
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to prepend options: {:?}", options);
                log::debug!( "UTabEntry::prepend_options {err_msg}. libmount::mnt_fs_prepend_options returned error code: {code:?}");

                Err(UTabEntryError::Config(err_msg))
            }
        }
    }

    /// Appends the `options` parameter to the corresponding VFS, `mountinfo` FS-specific, and userspace list of options.
    pub fn append_options<T>(&mut self, options: T) -> Result<(), UTabEntryError>
    where
        T: AsRef<str>,
    {
        let options = options.as_ref();
        log::debug!("UTabEntry::append_options appending options {:?}", options);

        let opts = ffi_utils::as_ref_str_to_c_string(options)?;

        let result = unsafe { libmount::mnt_fs_append_options(self.inner, opts.as_ptr()) };
        match result {
            0 => {
                log::debug!("UTabEntry::append_options appended options {:?}", options);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to append options {:?}", options);
                log::debug!( "UTabEntry::append_options {err_msg}. libmount::mnt_fs_append_options returned error code: {code:?}");

                Err(UTabEntryError::Config(err_msg))
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
                "UTabEntry::has_any_option does any element of the pattern list {:?} match? {:?}",
                pattern,
                state
            );

            state
        } else {
            log::debug!("UTabEntry::has_any_option failed to convert pattern to `CString`");

            false
        }
    }

    /// Returns `true` if data is read directly from the kernel (e.g `/proc/mounts`).
    pub fn is_from_kernel(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_kernel(self.inner) == 1 };
        log::debug!("UTabEntry::is_from_kernel value: {:?}", state);

        state
    }

    /// Returns `true` if the file system of this `UTabEntry` is a network file system.
    pub fn is_net_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_netfs(self.inner) == 1 };
        log::debug!("UTabEntry::is_net_fs value: {:?}", state);

        state
    }

    /// Returns `true` if the file system of this `UTabEntry` is a pseudo file system type (`proc`, `cgroups`).
    pub fn is_pseudo_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_pseudofs(self.inner) == 1 };
        log::debug!("UTabEntry::is_pseudo_fs value: {:?}", state);

        state
    }

    #[cfg(mount = "v2_39")]
    /// Returns `true` if the file system of this `UTabEntry` is a regular file system (neither a network nor a pseudo file system).
    pub fn is_regular_fs(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_regularfs(self.inner) == 1 };
        log::debug!("UTabEntry::is_regular_fs value: {:?}", state);

        state
    }

    /// Returns `true` if this `UTabEntry` represents a swap partition.
    pub fn is_swap(&self) -> bool {
        let state = unsafe { libmount::mnt_fs_is_swaparea(self.inner) == 1 };
        log::debug!("UTabEntry::is_swap value: {:?}", state);

        state
    }

    /// Returns `true` if the `source` parameter matches the `source` field in this `UTabEntry`.
    ///
    /// Using the provided `cache`, this method will perform the following comparisons in sequence:
    /// - `source` vs the value of the `source` field in this `UTabEntry`
    ///
    /// - the resolved value of the `source` parameter vs the value of the `source` field in this
    ///   `UTabEntry`
    /// - the resolved value of the `source` parameter vs the resolved value of the `source` field
    ///   in this `UTabEntry`
    /// - the resolved value of the `source` parameter vs the evaluated tag of the `source` field
    ///   in this `UTabEntry`
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
                "UTabEntry::is_source is {:?} the source of this entry? {:?}",
                source,
                state
            );

            state
        } else {
            log::debug!("UTabEntry::is_source failed to convert source to `CString`");

            false
        }
    }

    /// Returns `true` if the `source` parameter matches exactly the `source` field in this
    /// `UTabEntry`
    ///
    /// **Note:** redundant forward slashes are ignored when comparing values.
    pub fn is_exact_source(&self, source: &Source) -> bool {
        let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok();

        if let Some(source_cstr) = source_cstr {
            let state =
                unsafe { libmount::mnt_fs_streq_srcpath(self.inner, source_cstr.as_ptr()) == 1 };
            log::debug!(
                "UTabEntry::is_exact_source is {:?} the exact source of this entry? {:?}",
                source,
                state
            );

            state
        } else {
            log::debug!("UTabEntry::is_exact_source failed to convert source to `CString`");

            false
        }
    }

    /// Returns `true` if `path` matches the `target` field in this `UTabEntry`. Using
    /// the provided `cache`, this method will perform the following comparisons in sequence:
    ///
    /// - `path` vs the value of the `target` field in this `UTabEntry`
    /// - canonicalized `path` vs the value of the `target` field in this `UTabEntry`
    /// - canonicalized `path` vs the canonicalized value of the `target` field in this
    ///   `UTabEntry` if is not from `/proc/self/mountinfo`
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
                "UTabEntry::is_target is {:?} the target of this entry? {:?}",
                path,
                state
            );

            state
        } else {
            log::debug!("UTabEntry::is_target failed to convert path to `CString`");

            false
        }
    }

    /// Returns `true` if `path` matches **exactly** the `target` field in this `UTabEntry`.
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
                "UTabEntry::is_exact_target is {:?} the exact target of this entry? {:?}",
                path,
                state
            );

            state
        } else {
            log::debug!("UTabEntry::is_exact_target failed to convert path to `CString`");

            false
        }
    }

    //---- END predicates
}

impl fmt::Display for UTabEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formatting `fprintf_utab_fs`
        // https://github.com/util-linux/util-linux/blob/8aa25617467a1249669cff7240ca31973bf9a127/libmount/src/tab_update.c#L430
        let mut output: Vec<String> = vec![];
        if let Some(mount_id) = self.mount_id() {
            let id = format!("ID={}", mount_id);
            output.push(id);
        }

        if let Some(path) = self.source_path() {
            let path = format!("{}", path.display());
            let source_path = format!("SRC={}", utils::fstab_encode(path).unwrap());
            output.push(source_path);
        }

        if let Some(path) = self.target() {
            let path = format!("{}", path.display());
            let target = format!("SRC={}", utils::fstab_encode(path).unwrap());
            output.push(target);
        }

        if let Some(root) = self.root() {
            let root = format!("ROOT={}", utils::fstab_encode(root).unwrap());
            output.push(root);
        }

        if let Some(bind_source) = self.bind_source() {
            let bind_src = format!("BINDSRC={}", utils::fstab_encode(bind_source).unwrap());
            output.push(bind_src);
        }

        if let Some(attributes) = self.attributes() {
            let attrs = format!("ATTRS={}", utils::fstab_encode(attributes).unwrap());
            output.push(attrs);
        }

        if let Some(mount_options) = self.mount_options() {
            let opts = format!("OPTS={}", utils::fstab_encode(mount_options).unwrap());
            output.push(opts);
        }

        write!(f, "{}", output.join(" "))
    }
}
