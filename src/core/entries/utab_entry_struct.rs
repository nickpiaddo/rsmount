// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;

// From this library
use crate::core::errors::UTabEntryError;
use crate::core::utils;
use crate::declare_tab_entry;
use crate::utab_entry_shared_methods;

declare_tab_entry!(
    UTabEntry,
    r"A line in `/run/mount/utab`.

For example:
```text
SRC=/dev/vda TARGET=/mnt ROOT=/ OPTS=x-initrd.mount
```"
);

utab_entry_shared_methods!(UTabEntry, UTabEntryError);

impl UTabEntry {
    //---- BEGIN setters

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

        match unsafe { libmount::mnt_fs_set_root(self.inner, path_cstr.as_ptr()) } {
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

    //---- END setters

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

    //---- END getters
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
