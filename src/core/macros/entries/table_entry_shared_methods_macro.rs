// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! declare_tab_entry {
    ($entry_type:ident, $doc: literal) => {
        #[doc = $doc]
        #[derive(Debug, PartialEq)]
        #[repr(transparent)]
        pub struct $entry_type {
            pub(crate) inner: *mut libmount::libmnt_fs,
        }

        impl Drop for $entry_type {
            fn drop(&mut self) {
                log::debug!(concat!(
                    stringify!($entry_type),
                    "::drop deallocating `",
                    stringify!($entry_type),
                    "` instance"
                ));

                unsafe { libmount::mnt_unref_fs(self.inner) }
            }
        }
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
/// Methods shared by FsTabEntry with other objects.
macro_rules! fs_tab_entry_shared_methods {
    ($entry_type:ident, $entry_error_type:ident) => {
        $crate::table_entry_set_source!($entry_type, $entry_error_type);
        $crate::table_entry_set_target!($entry_type, $entry_error_type);
        $crate::set_mount_options!($entry_type, $entry_error_type);
        $crate::fstab_entry_setters!($entry_type, $entry_error_type);
        $crate::print_debug_to!($entry_type, $entry_error_type);
        $crate::table_entry_complete!($entry_type, $entry_error_type);
        $crate::fstab_entry_getters!($entry_type, $entry_error_type);
        $crate::table_entry_shared_methods!($entry_type, $entry_error_type);
        $crate::table_entry_shared_fs_methods!($entry_type, $entry_error_type);
        $crate::table_entry_shared_target_methods!($entry_type, $entry_error_type);
        $crate::table_entry_shared_options_methods!($entry_type, $entry_error_type);
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
/// Methods shared by MountImfoEntry with other objects.
macro_rules! mount_info_entry_shared_methods {
    ($entry_type:ident, $entry_error_type:ident) => {
        $crate::print_debug_to!($entry_type, $entry_error_type);
        $crate::table_entry_shared_methods!($entry_type, $entry_error_type);
        $crate::table_entry_shared_fs_methods!($entry_type, $entry_error_type);
        $crate::table_entry_shared_target_methods!($entry_type, $entry_error_type);
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
/// Methods shared by SwapsEntry with other objects.
macro_rules! swaps_entry_shared_methods {
    ($entry_type:ident, $entry_error_type:ident) => {
        $crate::print_debug_to!($entry_type, $entry_error_type);
        $crate::table_entry_shared_methods!($entry_type, $entry_error_type);
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
/// Methods shared by UTabEntry with other objects.
macro_rules! utab_entry_shared_methods {
    ($entry_type:ident, $entry_error_type:ident) => {
        $crate::print_debug_to!($entry_type, $entry_error_type);
        $crate::table_entry_complete!($entry_type, $entry_error_type);
        $crate::set_mount_options!($entry_type, $entry_error_type);
        $crate::table_entry_shared_methods!($entry_type, $entry_error_type);
        $crate::table_entry_shared_target_methods!($entry_type, $entry_error_type);
        $crate::table_entry_shared_options_methods!($entry_type, $entry_error_type);
        $crate::table_entry_set_source_path!($entry_type, $entry_error_type);
        $crate::set_bind_source!($entry_type, $entry_error_type);
        $crate::table_entry_set_target!($entry_type, $entry_error_type);
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_entry_set_source {
    ($entry_type:ident, $entry_error_type:ident) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN setters

            /// Sets the source of the device to mount.
            pub fn set_source(&mut self, source: Source) -> Result<(), $entry_error_type> {
                log::debug!(
                    concat!(
                        stringify!($entry_type),
                        "::set_source setting the source of a device to mount: {:?}"
                    ),
                    source
                );

                self.set_mount_source(source.to_string())
            }
            //---- END setters
        } //---- END impl
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_entry_set_source_path {
    ($entry_type:ident, $entry_error_type:ident) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN setters

            /// Sets the source path of the device to mount.
            pub fn set_source_path<T>(&mut self, source: T) -> Result<(), $entry_error_type>
            where
                T: AsRef<Path>,
            {
                let source = source.as_ref().display().to_string();
                log::debug!(
                    concat!(
                        stringify!($entry_type),
                        "::set_source setting the source of a device to mount: {:?}"
                    ),
                    source
                );

                self.set_mount_source(source)
            }
            //---- END setters
        } //---- END impl
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_entry_set_target {
    ($entry_type:ident, $entry_error_type:ident) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN setters

            /// Sets a device's mount point.
            pub fn set_target<T>(&mut self, path: T) -> Result<(), $entry_error_type>
            where
                T: AsRef<Path>,
            {
                let path = path.as_ref();
                log::debug!(
                    concat!(
                        stringify!($entry_type),
                        "::set_target setting device mount point to: {:?}"
                    ),
                    path
                );

                self.set_mount_target(path)
            }
            //---- END setters
        } //---- END impl
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! fstab_entry_setters {
    ($entry_type:ident, $entry_error_type:ident) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN setters

            /// Sets a comment line.
            pub fn set_comment(&mut self, mut comment: String) -> Result<(), $entry_error_type> {
                log::debug!(
                    concat!(stringify!($entry_type), "::set_comment setting comment line: {:?}"),
                    comment
                );
                // To avoid commenting out the whole entry when writing this `Entry` to file.
                if !comment.ends_with('\n') {
                    comment.push('\n');
                }

                let comment_cstr = ffi_utils::as_ref_str_to_c_string(&comment)?;

                let result = unsafe { libmount::mnt_fs_set_comment(self.inner, comment_cstr.as_ptr()) };

                match result {
                    0 => {
                        log::debug!(concat!(stringify!($entry_type), "::set_comment set comment line: {:?}"), comment);

                        Ok(())
                    }
                    code => {
                        let err_msg = format!("failed to set comment line: {:?}", comment);
                        log::debug!(concat!(stringify!($entry_type), "::set_comment {}. libmount::mnt_fs_set_comment returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            /// Sets the file system associated with the device to mount.
            pub fn set_file_system_type(&mut self, fs_type: FileSystem) -> Result<(), $entry_error_type>
            {
                log::debug!(
                    concat!(stringify!($entry_type), "::set_file_system_type setting file system type to: {:?}"),
                    fs_type
                );

                let fs_type_cstr = ffi_utils::as_ref_str_to_c_string(&fs_type)?;

                let result = unsafe { libmount::mnt_fs_set_fstype(self.inner, fs_type_cstr.as_ptr()) };

                match result {
                    0 => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::set_file_system_type set file system type to: {:?}"),
                            fs_type
                        );

                        Ok(())
                    }
                    code => {
                        let err_msg = format!("failed to set file system type to: {:?}", fs_type);
                        log::debug!(concat!(stringify!($entry_type), "::set_file_system_type {}. libmount::mnt_fs_set_fstype returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            /// Sets the interval in days between file system backups by the `dump` command on ext2/3/4
            /// file systems. (see the `dump` command's [manpage](https://manpages.org/dump/8))
            pub fn set_backup_frequency(&mut self, interval: i32) -> Result<(), $entry_error_type> {
                log::debug!(
                    concat!(stringify!($entry_type), "::set_backup_frequency setting interval between backups to: {:?} days"),
                    interval
                );

                let result = unsafe { libmount::mnt_fs_set_freq(self.inner, interval) };

                match result {
                    0 => {
                        log::debug!(concat!(stringify!($entry_type), "::set_backup_frequency setting interval between backups to: {:?} days"), interval);

                        Ok(())
                    }
                    code => {
                        let err_msg = format!(
                            "failed to set interval between backups to: {:?} days",
                            interval
                        );
                        log::debug!(concat!(stringify!($entry_type), "::set_backup_frequency {}. libmount::mnt_fs_set_freq returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            /// Sets the order in which file systems are checked by the `fsck` command. Setting this value
            /// to `0` will direct `fsck` to skip and not check at all the device referenced in this
            /// data structure.
            pub fn set_fsck_checking_order(&mut self, order: i32) -> Result<(), $entry_error_type> {
                log::debug!(
                    concat!(stringify!($entry_type), "::set_fsck_checking_order setting file system checking order: {:?}"),
                    order
                );

                let result = unsafe { libmount::mnt_fs_set_passno(self.inner, order) };

                match result {
                    0 => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::set_fsck_checking_order set file system checking order: {:?}"),
                            order
                        );

                        Ok(())
                    }
                    code => {
                        let err_msg = format!("failed to set file system checking order: {:?}", order);
                        log::debug!(concat!(stringify!($entry_type), "::set_fsck_checking_order {}. libmount::mnt_fs_set_passno returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            //---- END setters
        } //---- END impl
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! fstab_entry_getters {
    ($entry_type:ident, $entry_error_type:ident) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN getters

            /// Returns the entry's source field.
            pub fn source(&self) -> Option<Source> {
                log::debug!(concat!(stringify!($entry_type), "::source getting the mount's source"));

                let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

                unsafe { ptr.write(libmount::mnt_fs_get_source(self.inner)); }

                match unsafe { ptr.assume_init() } {
                    ptr if ptr.is_null() => {
                        log::debug!(concat!(stringify!($entry_type), "::source failed to get the mount's source. libmount::mnt_fs_get_source returned a NULL pointer"));

                        None
                    }

                    ptr => {
                        let source = ffi_utils::const_char_array_to_str_ref(ptr);

                        match source {
                            Ok(source) => {
                                log::debug!(concat!(stringify!($entry_type), "::source value: {:?}"), source);

                                Source::from_str(source).ok()
                            }
                            Err(e) => {
                                log::debug!(concat!(stringify!($entry_type), "::source {:?}"), e);

                                None
                            }
                        }
                    }
                }
            }
            //---- END getters
        } //---- END impl
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! set_mount_options {
    ($entry_type:ident, $entry_error_type:ident) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN setters

            /// Sets mount options string.
            pub fn set_mount_options<T>(&mut self, options: T) -> Result<(), $entry_error_type>
            where
                T: AsRef<str>,
            {
                let options = options.as_ref();
                log::debug!(
                    concat!(stringify!($entry_type), "::set_mount_options setting mount options string to: {:?}"),
                    options
                );

                let options_cstr = ffi_utils::as_ref_str_to_c_string(options)?;

                let result = unsafe { libmount::mnt_fs_set_options(self.inner, options_cstr.as_ptr()) };

                match result {
                    0 => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::set_mount_options set mount options string to: {:?}"),
                            options
                        );

                        Ok(())
                    }
                    code => {
                        let err_msg = format!("failed to set mount options string to: {:?}", options);
                        log::debug!(concat!(stringify!($entry_type), "::set_mount_options {}. libmount::mnt_fs_set_options returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            //---- END setters
        } //---- END impl
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_entry_complete {
    ($entry_type:ident, $entry_error_type:ident) => {
        #[allow(dead_code)]
        impl $entry_type {
            /// Fills the empty fields in `destination` by copying data from the corresponding fields in
            /// this object.
            pub fn complete(
                &mut self,
                destination: &mut $entry_type,
            ) -> Result<(), $entry_error_type> {
                log::debug!(concat!(
                    stringify!($entry_type),
                    "::complete copying fields to destination `",
                    stringify!($entry_type),
                    "`"
                ));

                let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                unsafe {
                    ptr.write(libmount::mnt_copy_fs(destination.inner, self.inner));
                }

                match unsafe { ptr.assume_init() } {
                    ptr if ptr.is_null() => {
                        let err_msg =
                            "failed to copy fields to destination `FsTabEntry`".to_owned();
                        log::debug!(
                            concat!(
                                stringify!($entry_type),
                                "::complete {}. libmount::mnt_copy_fs returned a NULL pointer"
                            ),
                            err_msg
                        );

                        Err(<$entry_error_type>::Copy(err_msg))
                    }
                    _ptr => {
                        log::debug!(concat!(
                            stringify!($entry_type),
                            "::complete copied fields to destination `",
                            stringify!($entry_type),
                            "`"
                        ));

                        Ok(())
                    }
                }
            }
        } //---- END impl
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_entry_shared_methods {
    ($entry_type:ident, $entry_error_type:ident) => {
        // From dependency library

        // From standard library
        use std::mem::MaybeUninit;
        use std::path::Path;

        // From this library
        use $crate::ffi_utils;
        use $crate::core::cache::Cache;
        use $crate::core::device::Source;

        #[allow(dead_code)]
        impl $entry_type {
            #[doc(hidden)]
            /// Increments the inner value's reference counter.
            pub(crate) fn incr_ref_counter(&mut self) {
                unsafe { libmount::mnt_ref_fs(self.inner) }
            }

            #[doc(hidden)]
            /// Decrements the inner value's reference counter.
            pub(crate) fn decr_ref_counter(&mut self) {
                unsafe { libmount::mnt_unref_fs(self.inner) }
            }

            #[doc(hidden)]
            /// Borrows an instance.
            pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_fs) -> $entry_type {
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
            pub(crate) fn from_ptr(ptr: *mut libmount::libmnt_fs) -> $entry_type {
                Self { inner: ptr }
            }

            #[doc(hidden)]
            /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe reference.
            pub(crate) unsafe fn ref_from_boxed_ptr<'a>(ptr: Box<*mut libmount::libmnt_fs>) -> (*mut *mut libmount::libmnt_fs, &'a $entry_type) {
                let raw_ptr = Box::into_raw(ptr);
                let entry_ref = unsafe { &*(raw_ptr as *const _ as *const $entry_type) };

                (raw_ptr, entry_ref)
            }

            #[doc(hidden)]
            /// Wraps a boxed raw `libmount::mnt_fs` pointer in a safe mutable reference.
            pub(crate) unsafe fn mut_from_boxed_ptr<'a>(ptr: Box<*mut libmount::libmnt_fs>) -> (*mut *mut libmount::libmnt_fs, &'a mut $entry_type) {
                let raw_ptr = Box::into_raw(ptr);
                let entry_ref = unsafe { &mut *(raw_ptr as *mut $entry_type) };

                (raw_ptr, entry_ref)
            }

            #[doc(hidden)]
            /// Creates a new instance.
            pub(crate) fn new() -> Result<$entry_type, $entry_error_type> {
                log::debug!(concat!(stringify!($entry_type), "::new creating a new `", stringify!($entry_type), "` instance"));

                let mut inner = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
                unsafe { inner.write(libmount::mnt_new_fs()); }

                match unsafe { inner.assume_init() } {
                    inner if inner.is_null() => {
                        let err_msg = concat!("failed to create a new `", stringify!($entry_type), "` instance").to_owned();
                        log::debug!(
                            concat!(stringify!($entry_type), "::new {}. libmount::mnt_new_fs returned a NULL pointer"),
                            err_msg
                        );

                        Err(<$entry_error_type>::Creation(err_msg))
                    }
                    inner => {
                        log::debug!(concat!(stringify!($entry_type), "::new created a new `", stringify!($entry_type), "` instance"));
                        let entry = Self { inner };

                        Ok(entry)
                    }
                }
            }

            //---- BEGIN setters

            #[doc(hidden)]
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
            pub(crate) fn set_mount_source<T>(&mut self, source: T) -> Result<(), $entry_error_type>
            where
                T: AsRef<str>,
            {
                let source = source.as_ref();
                log::debug!(
                    concat!(stringify!($entry_type), "::set_mount_source setting the source of a device to mount: {:?}"),
                    source
                );

                let source_cstr = ffi_utils::as_ref_str_to_c_string(source)?;

                let result = unsafe { libmount::mnt_fs_set_source(self.inner, source_cstr.as_ptr()) };

                match result {
                    0 => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::set_mount_source set the source of a device to mount: {:?}"),
                            source
                        );

                        Ok(())
                    }
                    code => {
                        let err_msg = format!(
                            "failed to set the source of a device to mount: {:?}",
                            source
                        );
                        log::debug!(concat!(stringify!($entry_type), "::set_mount_source {}. libmount::mnt_fs_set_source returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            #[doc(hidden)]
            /// Sets a device's mount point.
            pub(crate) fn set_mount_target<T>(&mut self, path: T) -> Result<(), $entry_error_type>
            where
                T: AsRef<Path>,
            {
                let path = path.as_ref();
                log::debug!(
                    concat!(stringify!($entry_type), "::set_mount_target setting device mount point to: {:?}"),
                    path
                );

                let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;

                let result = unsafe { libmount::mnt_fs_set_target(self.inner, path_cstr.as_ptr()) };

                match result  {
                    0 => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::set_mount_target set device mount point to: {:?}"),
                            path
                        );

                        Ok(())
                    }
                    code => {
                        let err_msg = format!("failed to set a device's mount point to: {:?}", path);
                        log::debug!(concat!(stringify!($entry_type), "::set_mount_target {}. libmount::mnt_fs_set_target returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            //---- END setters

            //---- BEGIN mutators

            #[doc = concat!("Allocates a new `", stringify!($entry_type), "`, and a copies all the source's fields to the new")]
            /// instance except any private user data.
            pub fn copy(&self) -> Result<$entry_type, $entry_error_type> {
                log::debug!(concat!(stringify!($entry_type), "::copy copying `", stringify!($entry_type), "`"));

                let mut ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();

                unsafe { ptr.write(libmount::mnt_copy_fs(std::ptr::null_mut(), self.inner)); }

                match unsafe { ptr.assume_init() } {
                    ptr if ptr.is_null() => {
                        let err_msg = concat!("failed to copy `", stringify!($entry_type), "`").to_owned();
                        log::debug!(concat!(stringify!($entry_type), "::copy {}. libmount::mnt_copy_fs returned a NULL pointer"), err_msg);

                        Err(<$entry_error_type>::Action(err_msg))
                    }
                    ptr => {
                        log::debug!(concat!(stringify!($entry_type), "::copy copied `", stringify!($entry_type), "`"));
                        let entry = <$entry_type>::from_ptr(ptr);

                        Ok(entry)
                    }
                }
            }

            //---- END mutators

            //---- BEGIN getters

            /// Returns the entry's source path which can be
            /// - a directory for bind mounts (in `/etc/fstab` or `/etc/mtab` only)
            /// - a path to a block device for standard mounts.
            pub fn source_path(&self) -> Option<&Path> {
                log::debug!(concat!(stringify!($entry_type), "::source_path getting the mount's source path"));

                let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

                unsafe { ptr.write(libmount::mnt_fs_get_srcpath(self.inner)); }

                match unsafe { ptr.assume_init() } {
                    ptr if ptr.is_null() => {
                        log::debug!(concat!(stringify!($entry_type), "::source_path failed to get the mount's source path. libmount::mnt_fs_get_srcpath returned a NULL pointer"));

                        None
                    }

                    ptr => {
                        let path = ffi_utils::const_c_char_array_to_path(ptr);
                        log::debug!(concat!(stringify!($entry_type), "::source_path value: {:?}"), path);

                        Some(path)
                    }
                }
            }

            //---- END getters

            //---- BEGIN predicates

            /// Returns `true` if data is read directly from the kernel (e.g `/proc/mounts`).
            pub fn is_from_kernel(&self) -> bool {
                let state = unsafe { libmount::mnt_fs_is_kernel(self.inner) == 1 };
                log::debug!(concat!(stringify!($entry_type), "::is_from_kernel value: {:?}"), state);

                state
            }

            #[doc = concat!("Returns `true` if the file system of this `", stringify!($entry_type),"` is a network file system.")]
            pub fn is_net_fs(&self) -> bool {
                let state = unsafe { libmount::mnt_fs_is_netfs(self.inner) == 1 };
                log::debug!(concat!(stringify!($entry_type), "::is_net_fs value: {:?}"), state);

                state
            }

            #[doc = concat!("Returns `true` if the file system of this `", stringify!($entry_type),"` is a pseudo file system type (`proc`, `cgroups`).")]
            pub fn is_pseudo_fs(&self) -> bool {
                let state = unsafe { libmount::mnt_fs_is_pseudofs(self.inner) == 1 };
                log::debug!(concat!(stringify!($entry_type), "::is_pseudo_fs value: {:?}"), state);

                state
            }

            #[cfg(mount = "v2_39")]
            #[doc = concat!("Returns `true` if the file system of this `", stringify!($entry_type),"` is a regular file system (neither a network nor a pseudo file system).")]
            pub fn is_regular_fs(&self) -> bool {
                let state = unsafe { libmount::mnt_fs_is_regularfs(self.inner) == 1 };
                log::debug!(concat!(stringify!($entry_type), "::is_regular_fs value: {:?}"), state);

                state
            }

            #[doc = concat!("Returns `true` if this `", stringify!($entry_type),"` represents a swap partition.")]
            pub fn is_swap(&self) -> bool {
                let state = unsafe { libmount::mnt_fs_is_swaparea(self.inner) == 1 };
                log::debug!(concat!(stringify!($entry_type), "::is_swap value: {:?}"), state);

                state
            }

            #[doc = concat!("Returns `true` if the `source` parameter matches the `source` field in this `", stringify!($entry_type), "`.")]
            ///
            /// Using the provided `cache`, this method will perform the following comparisons in sequence:
            #[doc = concat!("- `source` vs the value of the `source` field in this `", stringify!($entry_type), "`")]
            ///
            /// - the resolved value of the `source` parameter vs the value of the `source` field in this
            #[doc = concat!("`", stringify!($entry_type), "`")]
            /// - the resolved value of the `source` parameter vs the resolved value of the `source` field
            #[doc = concat!("in this `", stringify!($entry_type), "`")]
            /// - the resolved value of the `source` parameter vs the evaluated tag of the `source` field
            #[doc = concat!("in this `", stringify!($entry_type), "`")]
            ///
            /// *Resolving* the `source` parameter means searching and returning the absolute path to
            /// the device it represents. The same for *evaluating* a tag.
            pub fn is_source(&self, source: &Source, cache: &Cache) -> bool
            {
                let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok();

                if let Some(source_cstr) = source_cstr {
                    let state = unsafe {
                        libmount::mnt_fs_match_source(self.inner, source_cstr.as_ptr(), cache.inner) == 1
                    };
                    log::debug!(
                        concat!(stringify!($entry_type), "::is_source is {:?} the source of this entry? {:?}"),
                        source,
                        state
                    );

                    state
                } else {
                    log::debug!(concat!(stringify!($entry_type), "::is_source failed to convert source to `CString`"));

                    false
                }
            }

            /// Returns `true` if the `source` parameter matches exactly the `source` field in this
            #[doc = concat!("`", stringify!($entry_type), "`")]
            ///
            /// **Note:** redundant forward slashes are ignored when comparing values.
            pub fn is_exact_source(&self, source: &Source) -> bool
            {
                let source_cstr = ffi_utils::as_ref_str_to_c_string(source.to_string()).ok();

                if let Some(source_cstr) = source_cstr {
                    let state =
                        unsafe { libmount::mnt_fs_streq_srcpath(self.inner, source_cstr.as_ptr()) == 1 };
                    log::debug!(
                        concat!(stringify!($entry_type), "::is_exact_source is {:?} the exact source of this entry? {:?}"),
                        source,
                        state
                    );

                    state
                } else {
                    log::debug!(concat!(stringify!($entry_type), "::is_exact_source failed to convert source to `CString`"));

                    false
                }
            }

            //---- END predicates
        }

        impl AsRef<$entry_type> for $entry_type {
            #[inline]
            fn as_ref(&self) -> &$entry_type {
                self
            }
        }
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_entry_shared_fs_methods {
    ($entry_type:tt, $entry_error_type:tt) => {
        use $crate::core::fs::FileSystem;

        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN getters

            /// Returns the file system type specified at creation.
            pub fn file_system_type(&self) -> Option<FileSystem> {
                log::debug!(concat!(stringify!($entry_type), "::file_system_type getting file system type"));

                let mut fs_ptr = MaybeUninit::<*const libc::c_char>::zeroed();

                unsafe { fs_ptr.write(libmount::mnt_fs_get_fstype(self.inner)); }

                match unsafe { fs_ptr.assume_init() } {
                    ptr if ptr.is_null() => {
                        log::debug!(concat!(stringify!($entry_type), "::file_system_type failed to get file system type. libmount::mnt_fs_get_fstype returned a NULL pointer"));

                        None
                    }
                    fs_ptr => {
                        let fs_type = ffi_utils::const_char_array_to_str_ref(fs_ptr);
                        log::debug!(concat!(stringify!($entry_type), "::file_system_type value: {:?}"), fs_type);

                        fs_type.ok().and_then(|s| FileSystem::from_str(s).ok())
                    }
                }
            }

            //---- END getters

            //---- BEGIN predicates

            #[doc = concat!("Returns `true` if the file system type of this `", stringify!($entry_type), "` matches any element of the")]
            /// of the comma-separated file system names in the `pattern` parameter (see the [`FileSystem`
            /// documentation page](crate::core::fs::FileSystem) for a list of supported file systems).
            ///
            /// **Note:**
            #[doc = concat!("- file system names prefixed with a `no` will match if this `", stringify!($entry_type), "` does **NOT**")]
            /// have the file system mentioned.
            /// - a test with a pattern list starting with `no` will apply the prefix to **all** file
            /// systems in the list (e.g. `"noapfs,ext4"` is equivalent to `"noapfs,noext4"`).
            ///
            #[doc = concat!("For example, if this `", stringify!($entry_type), "` represents an `ext4` device, a test with the")]
            /// following patterns:
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
                    log::debug!(concat!(stringify!($entry_type), "::has_any_fs_type does any element of the pattern list {:?} match? {:?}"), pattern, state);

                    state
                } else {
                    log::debug!(concat!(stringify!($entry_type), "::has_any_fs_type failed to convert pattern to `CString`"));

                    false
                }
            }

            //---- END predicates
        }
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_entry_shared_target_methods {
    ($entry_type:tt, $entry_error_type:tt) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN getters

            /// Returns the path to the mount point.
            pub fn target(&self) -> Option<&Path> {
                log::debug!(concat!(stringify!($entry_type), "::target getting path to mount point"));

                let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

                unsafe { ptr.write(libmount::mnt_fs_get_target(self.inner)); }

                match unsafe { ptr.assume_init() } {
                    ptr if ptr.is_null() => {
                        log::debug!(concat!(stringify!($entry_type), "::target failed to get path to mount point. libmount::mnt_fs_get_target returned a NULL pointer"));

                        None
                    }
                    ptr => {
                        let path = ffi_utils::const_c_char_array_to_path(ptr);
                        log::debug!(concat!(stringify!($entry_type), "::target value: {:?}"), path);

                        Some(path)
                    }
                }
            }

            //---- END getters

            //---- BEGIN predicates

            #[doc = concat!("Returns `true` if `path` matches the `target` field in this `", stringify!($entry_type), "`. Using")]
            /// the provided `cache`, this method will perform the following comparisons in sequence:
            ///
            #[doc = concat!("- `path` vs the value of the `target` field in this `", stringify!($entry_type), "`")]
            #[doc = concat!("- canonicalized `path` vs the value of the `target` field in this `", stringify!($entry_type), "`")]
            /// - canonicalized `path` vs the canonicalized value of the `target` field in this
            #[doc = concat!("`", stringify!($entry_type), "` if is not from `/proc/self/mountinfo`")]
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
                        concat!(stringify!($entry_type), "::is_target is {:?} the target of this entry? {:?}"),
                        path,
                        state
                    );

                    state
                } else {
                    log::debug!(concat!(stringify!($entry_type), "::is_target failed to convert path to `CString`"));

                    false
                }
            }

            #[doc = concat!("Returns `true` if `path` matches **exactly** the `target` field in this `", stringify!($entry_type), "`.")]
            ///
            /// **Note:** redundant forward slashes are ignored when comparing values.
            pub fn is_exact_target<T>(&self, path: T) -> bool
            where
                T: AsRef<Path>,
            {
                let path = path.as_ref();
                let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok();

                if let Some(path_cstr) = path_cstr {
                    let state = unsafe { libmount::mnt_fs_streq_target(self.inner, path_cstr.as_ptr()) == 1 };
                    log::debug!(
                        concat!(stringify!($entry_type), "::is_exact_target is {:?} the exact target of this entry? {:?}"),
                        path,
                        state
                    );

                    state
                } else {
                    log::debug!(concat!(stringify!($entry_type), "::is_exact_target failed to convert path to `CString`"));

                    false
                }
            }

            //---- END predicates
        }
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! table_entry_shared_options_methods {
    ($entry_type:tt, $entry_error_type:tt) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN mutators

            /// Prepends the `options` parameter to the corresponding VFS, `mountinfo` FS-specific, and userspace list of options.
            pub fn prepend_options<T>(&mut self, options: T) -> Result<(), $entry_error_type>
            where
                T: AsRef<str>,
            {
                let options = options.as_ref();
                let attrs_cstr = ffi_utils::as_ref_str_to_c_string(options)?;

                log::debug!(
                    concat!(stringify!($entry_type), "::prepend_options prepending options: {:?}"),
                    options
                );

                let result = unsafe { libmount::mnt_fs_prepend_options(self.inner, attrs_cstr.as_ptr()) };

                match result {
                    0 => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::prepend_options prepended options: {:?}"),
                            options
                        );

                        Ok(())
                    }
                    code => {
                        let err_msg = format!("failed to prepend options: {:?}", options);
                        log::debug!(concat!(stringify!($entry_type), "::prepend_options {}. libmount::mnt_fs_prepend_options returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            /// Appends the `options` parameter to the corresponding VFS, `mountinfo` FS-specific, and userspace list of options.
            pub fn append_options<T>(&mut self, options: T) -> Result<(), $entry_error_type>
            where
                T: AsRef<str>,
            {
                let options = options.as_ref();
                log::debug!(
                    concat!(stringify!($entry_type), "::append_options appending options {:?}"),
                    options
                );

                let opts = ffi_utils::as_ref_str_to_c_string(options)?;

                let result = unsafe { libmount::mnt_fs_append_options(self.inner, opts.as_ptr()) };

                match result {
                    0 => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::append_options appended options {:?}"),
                            options
                        );

                        Ok(())
                    }
                    code => {
                        let err_msg = format!("failed to append options {:?}", options);
                        log::debug!(concat!(stringify!($entry_type), "::append_options {}. libmount::mnt_fs_append_options returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            //---- END mutators

            //---- BEGIN getters

            /// Returns the value of the option matching `option_name`.
            pub fn option_value<T>(&self, option_name: T) -> Option<String>
            where
                T: AsRef<str>,
            {
                let option_name = option_name.as_ref();
                log::debug!(
                    concat!(stringify!($entry_type), "::option_value getting value of option: {:?}"),
                    option_name
                );

                let opt_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;


                let mut value_start = MaybeUninit::<*mut libc::c_char>::zeroed();
                let mut size = MaybeUninit::<libc::size_t>::zeroed();

                let result =  unsafe { libmount::mnt_fs_get_option(
                    self.inner,
                    opt_cstr.as_ptr(),
                    value_start.as_mut_ptr(),
                    size.as_mut_ptr(),
                    )};

                match result {
                    0 => {
                        let value_start = unsafe { value_start.assume_init() };
                        let size = unsafe { size.assume_init() };
                        let mut data = unsafe { std::slice::from_raw_parts(value_start, size).to_owned() };
                        // add NUL terminator
                        data.push(0);

                        let value = ffi_utils::c_char_array_to_string(data.as_ptr());

                        log::debug!(
                            concat!(stringify!($entry_type), "::option_value option {:?} has value {:?}"),
                            option_name,
                            value
                        );

                        Some(value)
                    }
                    1 => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::option_value found no option matching {:?}"),
                            option_name
                        );

                        None
                    }
                    code => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::option_value failed to get value of option: {:?}. libmount::mnt_fs_get_option returned error code: {:?}"),
                            option_name,
                            code
                        );

                        None
                    }
                }
            }

            //---- END getters

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
                    log::debug!(concat!(stringify!($entry_type), "::has_any_option does any element of the pattern list {:?} match? {:?}"), pattern, state);

                    state
                } else {
                    log::debug!(concat!(stringify!($entry_type), "::has_any_option failed to convert pattern to `CString`"));

                    false
                }
            }

            //---- END predicates
        }

    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! mount_args_methods {
    ($entry_type:ident, $entry_error_type:ident) => {
        $crate::set_bind_source!($entry_type, $entry_error_type);
        $crate::set_user_data!($entry_type, $entry_error_type);
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! set_bind_source {
    ($entry_type:ident, $entry_error_type:ident) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN setters

            /// Sets the source directory of a bind mount .
            pub fn set_bind_source<T>(&mut self, path: T) -> Result<(), $entry_error_type>
            where
                T: AsRef<Path>,
            {
                let path = path.as_ref();
                log::debug!(
                    concat!(stringify!($entry_type), "::set_bind_source setting bind mount source directory as {:?}"),
                    path
                );

                let path_cstr = ffi_utils::as_ref_path_to_c_string(path)?;

                let result = unsafe { libmount::mnt_fs_set_bindsrc(self.inner, path_cstr.as_ptr()) };

                match result {
                    0 => {
                        log::debug!(
                            concat!(stringify!($entry_type), "::set_bind_source set bind mount source directory as {:?}"),
                            path
                        );

                        Ok(())
                    }
                    code => {
                        let err_msg =
                            format!("failed to set bind mount source directory as {:?}", path);
                        log::debug!(concat!(stringify!($entry_type), "::set_bind_source {}. libmount::mnt_fs_set_bindsrc returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            //---- END setters
        } //---- END impl
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! set_user_data {
    ($entry_type:ident, $entry_error_type:ident) => {
        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN setters

            /// Sets library independent user data.
            pub(crate) fn set_user_data(
                &mut self,
                user_data: std::ptr::NonNull<libc::c_void>,
            ) -> Result<(), $entry_error_type> {
                log::debug!(concat!(stringify!($entry_type), "::set_user_data setting user data"));

                let result = unsafe { libmount::mnt_fs_set_userdata(self.inner, user_data.as_ptr()) };

                match result {
                    0 => {
                        log::debug!(concat!(stringify!($entry_type), "::set_user_data set user data"));

                        Ok(())
                    }
                    code => {
                        let err_msg = "failed to set user data".to_owned();
                        log::debug!(concat!(stringify!($entry_type), "::set_user_data {}. libmount::mnt_fs_set_userdata returned error code: {:?}"), err_msg, code);

                        Err(<$entry_error_type>::Config(err_msg))
                    }
                }
            }

            //---- END setters
        } //---- END impl
    };
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! print_debug_to {
    ($entry_type:ident, $entry_error_type:ident) => {
        // From dependency library

        // From standard library
        use std::fs::File;

        // From this library

        #[allow(dead_code)]
        impl $entry_type {
            //---- BEGIN setters

            /// Sets the file stream to print debug messages to.
            pub fn print_debug_to(&mut self, stream: &mut File) -> Result<(), $entry_error_type> {
                log::debug!(
                    concat!(stringify!($entry_type), "::print_debug_to setting file stream to print debug messages to")
                );

                if ffi_utils::is_open_write_only(stream)? || ffi_utils::is_open_read_write(stream)? {
                    let file_stream = ffi_utils::write_only_c_file_stream_from(stream)?;

                    let result = unsafe { libmount::mnt_fs_print_debug(self.inner, file_stream as *mut _) };
                    match result {
                        0 => {
                            log::debug!(concat!(stringify!($entry_type), "::print_debug_to set file stream to print debug messages to"));

                            Ok(())
                        }
                        code => {
                            let err_msg =
                                "failed to set file stream to print debug messages to".to_owned();
                            log::debug!(concat!(stringify!($entry_type), "::print_debug_to {}. libmount::mnt_fs_print_debug returned error code: {:?}"), err_msg, code);

                            Err(<$entry_error_type>::Action(err_msg))
                        }
                    }
                } else {
                    let err_msg = "missing write permission for given stream".to_owned();
                    log::debug!(concat!(stringify!($entry_type), "::print_debug_to {}"), err_msg);

                    Err(<$entry_error_type>::Permission(err_msg))
                }
            }

            //---- END setters
        } //---- END impl
    };
}
