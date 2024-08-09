// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
#[cfg(v2_39)]
use rsblkid::probe::FsProperty;

// From standard library
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};
use std::str::FromStr;

// From this library
use crate::core::device::{Tag, TagName};
use crate::core::errors::CacheError;
use crate::core::fs::{FileSystem, FsType};
use crate::ffi_utils;
use crate::tables::MountInfo;

/// A cache of device paths, and tags.
#[derive(Debug)]
#[repr(transparent)]
pub struct Cache {
    pub(crate) inner: *mut libmount::libmnt_cache,
}

impl Cache {
    #[doc(hidden)]
    #[allow(dead_code)]
    /// Increments the `Cache`'s reference counter.
    pub(crate) fn incr_ref_counter(&mut self) {
        unsafe { libmount::mnt_ref_cache(self.inner) }
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Decrements the `Cache`'s reference counter.
    pub(crate) fn decr_ref_counter(&mut self) {
        unsafe { libmount::mnt_unref_cache(self.inner) }
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Borrows a `Cache` instance.
    pub(crate) fn borrow_ptr(ptr: *mut libmount::libmnt_cache) -> Cache {
        let mut cache = Self { inner: ptr };
        // We are virtually ceding ownership of this cache which will be automatically
        // deallocated once it is out of scope, incrementing its reference counter protects it from
        // being freed prematurely.
        cache.incr_ref_counter();

        cache
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Wraps a boxed raw `libmount::mnt_cache` pointer in a safe reference.
    pub(crate) unsafe fn ref_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_cache>,
    ) -> (*mut *mut libmount::libmnt_cache, &'a Cache) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &*(raw_ptr as *const _ as *const Cache) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    /// Wraps a boxed raw `libmount::mnt_cache` pointer in a safe mutable reference.
    pub(crate) unsafe fn mut_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_cache>,
    ) -> (*mut *mut libmount::libmnt_cache, &'a mut Cache) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &mut *(raw_ptr as *mut Cache) };

        (raw_ptr, entry_ref)
    }

    /// Creates a new `Cache`.
    pub fn new() -> Result<Cache, CacheError> {
        log::debug!("Cache::new creating a new `Cache` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_cache>::zeroed();

        unsafe { inner.write(libmount::mnt_new_cache()) };

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `Cache` instance".to_owned();
                log::debug!(
                    "Cache::new {}. libmount::mnt_new_cache returned a NULL pointer",
                    err_msg
                );

                Err(CacheError::Creation(err_msg))
            }
            inner => {
                log::debug!("Cache::new created a new `Cache` instance");
                let cache = Self { inner };

                Ok(cache)
            }
        }
    }

    /// Returns `true` if the device matching `device_name` has a corresponding `tag` in `Cache`.
    pub fn device_has_tag<P, T>(&self, device_name: P, tag: T) -> bool
    where
        P: AsRef<Path>,
        T: AsRef<Tag>,
    {
        let device_name = device_name.as_ref();
        let tag = tag.as_ref();
        log::debug!(
            "Cache::device_has_tag checking device {:?} for tag {:?}",
            device_name,
            tag
        );

        let path = ffi_utils::as_ref_path_to_c_string(device_name);
        let name = tag.name().to_c_string();
        let value = ffi_utils::as_ref_str_to_c_string(tag.value());

        if let (Ok(path_cstr), Ok(value_cstr)) = (&path, &value) {
            let state = unsafe {
                libmount::mnt_cache_device_has_tag(
                    self.inner,
                    path_cstr.as_ptr(),
                    name.as_ptr(),
                    value_cstr.as_ptr(),
                ) == 1
            };

            log::debug!(
                "Cache::device_has_tag checking device {:?} for tag {:?} -> result: {:?}",
                device_name,
                tag,
                state
            );

            state
        } else {
            log::debug!("Cache::device_has_tag failed to convert to `CString`: path: {:?}, tag name: {:?}, tag value: {:?}", path, name, value);

            false
        }
    }

    /// Returns the value associated with a `tag_name` for a device in `Cache`.
    pub fn find_tag_value<P>(&self, device_name: P, tag_name: TagName) -> Option<String>
    where
        P: AsRef<Path>,
    {
        let device_name = device_name.as_ref();

        log::debug!(
            "Cache::find_tag_value searching value of tag named {:?} for device {:?}",
            tag_name,
            device_name
        );

        let path_cstr = ffi_utils::as_ref_path_to_c_string(device_name).ok()?;
        let name_cstr = tag_name.to_c_string();

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_cache_find_tag_value(
                self.inner,
                path_cstr.as_ptr(),
                name_cstr.as_ptr(),
            ));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("Cache::find_tag_value found no value for tag name {:?} on device {:?}. libmount::mnt_cache_find_tag_value returned a NULL pointer", tag_name, device_name);

                None
            }
            value_ptr => {
                let value = ffi_utils::c_char_array_to_string(value_ptr);
                log::debug!(
                    "value of tag named {:?} for device {:?} -> {:?}",
                    tag_name,
                    device_name,
                    value
                );

                Some(value)
            }
        }
    }

    /// Imports canonicalized paths from a [`MountInfo`] table. This operation will provide already
    /// canonicalized paths to search functions using a `Cache`.
    pub fn import_paths(&mut self, table: &MountInfo) -> Result<(), CacheError> {
        log::debug!("Cache::import_paths import canonicalized paths");

        let result = unsafe { libmount::mnt_cache_set_targets(self.inner, table.inner) };

        match result {
            0 => {
                log::debug!("Cache::import_paths imported canonicalized paths");

                Ok(())
            }
            code => {
                let err_msg = "failed to import canonicalized paths".to_owned();
                log::debug!("Cache::import_paths {}. libmount::mnt_cache_set_targets returned error code: {:?}", err_msg, code);

                Err(CacheError::Import(err_msg))
            }
        }
    }

    /// Imports tags associated with a device.
    pub fn import_tags<P>(&mut self, device_name: P) -> Result<(), CacheError>
    where
        P: AsRef<Path>,
    {
        let device_name = device_name.as_ref();
        let name_cstr = ffi_utils::as_ref_path_to_c_string(device_name)?;

        log::debug!(
            "Cache::import_tags importing tags associated with device: {:?}",
            device_name
        );

        let result = unsafe { libmount::mnt_cache_read_tags(self.inner, name_cstr.as_ptr()) };

        match result {
            code if code == 0 || code == 1 => {
                let op = if code == 0 {
                    "imported tegs associated with".to_owned()
                } else {
                    "no tag imported from".to_owned()
                };
                log::debug!("Cache::import_tags {} device: {:?}", op, device_name);

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to import tags associated with device: {:?}",
                    device_name
                );
                log::debug!("Cache::import_tags {}. libmount::mnt_cache_read_tags returned error code: {:?}", err_msg, code);

                Err(CacheError::Import(err_msg))
            }
        }
    }

    #[cfg(v2_39)]
    /// Behind the scene, a `Cache` uses `rsblkid` to scan a device for tags. This method allows the
    /// caller to control which extra file system properties a `Cache` collects.
    ///
    /// **Warning:** do not use this function if you are unsure of its potential effects.
    pub fn collect_fs_properties<T>(&mut self, properties: T) -> Result<(), CacheError>
    where
        T: AsRef<[FsProperty]>,
    {
        let properties = properties.as_ref();
        log::debug!(
            "Cache::collect_fs_properties setting file system properties to collect: {:?}",
            properties
        );

        let flags = properties
            .iter()
            .fold(0 as libc::c_int, |acc, &prop| acc | prop as libc::c_int);

        let result = unsafe { libmount::mnt_cache_set_sbprobe(self.inner, flags) };

        match result {
            0 => {
                log::debug!(
                    "Cache::collect_fs_properties set file system properties to collect: {:?}",
                    properties
                );

                Ok(())
            }
            code => {
                let err_msg = format!(
                    "failed to set file system properties to collect: {:?}",
                    properties
                );
                log::debug!("Cache::collect_fs_properties {}. libmount::mnt_cache_set_sbprobe returned error code: {:?}", err_msg, code);

                Err(CacheError::Config(err_msg))
            }
        }
    }

    #[doc(hidden)]
    /// Returns the file system associated with the given device name, and save the result in a
    /// `Cache` if `cache_ptr` is not NULL.
    fn get_fstype<P>(cache_ptr: *mut libmount::libmnt_cache, device_name: P) -> Option<FsType>
    where
        P: AsRef<Path>,
    {
        let device_name = device_name.as_ref();
        let device_name_cstr = ffi_utils::as_ref_path_to_c_string(device_name).ok()?;
        log::debug!(
            "Cache::get_fstype identifying file system on device {:?}",
            device_name
        );

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();
        let mut collision = MaybeUninit::<libc::c_int>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_get_fstype(
                device_name_cstr.as_ptr(),
                collision.as_mut_ptr(),
                cache_ptr,
            ));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!(
                    "Cache::get_fstype failed to identify file systen on device {:?}",
                    device_name
                );

                None
            }
            ptr => {
                let collision = unsafe { collision.assume_init() };
                let fs = ffi_utils::c_char_array_to_string(ptr);

                if cache_ptr.is_null() {
                    unsafe {
                        libc::free(ptr as *mut _);
                    }
                }

                match FileSystem::from_str(fs.as_str()) {
                    Ok(file_system) => {
                        if collision == 0 {
                            Some(FsType::NoCollision(file_system))
                        } else {
                            Some(FsType::Collision(file_system))
                        }
                    }
                    Err(e) => {
                        log::debug!("Cache::get_fstype {:?}", e);
                        None
                    }
                }
            }
        }
    }

    /// Returns the file system associated with the given device.
    pub fn find_file_system_type<P>(device_name: P) -> Option<FsType>
    where
        P: AsRef<Path>,
    {
        let device_name = device_name.as_ref();
        log::debug!(
            "Cache::find_file_system_type identifying file system on device {:?}",
            device_name
        );

        Self::get_fstype(std::ptr::null_mut(), device_name)
    }

    /// Returns the file system associated with the given device, and saves the result in this
    /// `Cache`.
    pub fn find_and_cache_file_system_type<P>(&mut self, device_name: P) -> Option<FsType>
    where
        P: AsRef<Path>,
    {
        let device_name = device_name.as_ref();
        log::debug!(
            "Cache::find_and_cache_file_system_type identifying file system on device {:?}",
            device_name
        );

        Self::get_fstype(self.inner, device_name)
    }

    #[doc(hidden)]
    /// Canonicalizes a path; eventually saves the result in a `Cache`.
    fn canonicalize_path<P>(cache_ptr: *mut libmount::libmnt_cache, path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok()?;

        let mut path_ptr = MaybeUninit::<*mut libc::c_char>::zeroed();

        unsafe {
            path_ptr.write(libmount::mnt_pretty_path(path_cstr.as_ptr(), cache_ptr));
        }

        match unsafe { path_ptr.assume_init() } {
            path_ptr if path_ptr.is_null() => {
                log::debug!("Cache::canonicalize_path failed to canonicalize path: {:?}. libmount::mnt_pretty_path returned a NULL pointer", path);

                None
            }
            path_ptr => {
                let canonical_path = ffi_utils::const_c_char_array_to_path_buf(path_ptr);
                if cache_ptr.is_null() {
                    unsafe {
                        libc::free(path_ptr as *mut _);
                    }
                }
                log::debug!(
                    "Cache::canonical_path {:?} canonicalized to {:?}",
                    path,
                    canonical_path
                );

                Some(canonical_path)
            }
        }
    }

    /// Returns the canonical, absolute form of a path with all intermediate components normalized
    /// and symbolic links resolved.
    ///
    /// Returns the name of the mounted image file for loop back devices (e.g. `/dev/loopN`).
    ///
    /// Returns the value `none` if the parameter `path` is an empty value.
    pub fn canonicalize<P>(path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!("Cache::canonicalize canonicalizing path: {:?}", path);

        Self::canonicalize_path(std::ptr::null_mut(), path)
    }

    /// Returns the canonical, absolute form of a path with all intermediate components normalized
    /// and symbolic links resolved.
    ///
    /// Returns the name of the mounted image file for loop back devices (e.g. `/dev/loopN`).
    ///
    /// Returns the value `none` if the given path is empty.
    ///
    /// Saves the result in the `Cache`.
    pub fn canonicalize_and_cache<P>(&mut self, path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "Cache::canonicalize canonicalizing and caching path: {:?}",
            path
        );

        Self::canonicalize_path(self.inner, path)
    }

    #[doc(hidden)]
    /// Resolves a path and  saves the result in a `Cache` if `cache_ptr` is not NULL.
    fn resolve_path<P>(cache_ptr: *mut libmount::libmnt_cache, path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok()?;

        let mut path_ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
        unsafe {
            path_ptr.write(libmount::mnt_resolve_path(path_cstr.as_ptr(), cache_ptr));
        }

        match unsafe { path_ptr.assume_init() } {
            path_ptr if path_ptr.is_null() => {
                log::debug!("Cache::resolve_path failed to resolve path: {:?}. libmount::mnt_resolve_path returned a NULL pointer", path);

                None
            }
            path_ptr => {
                let resolved_path = ffi_utils::const_c_char_array_to_path_buf(path_ptr);
                if cache_ptr.is_null() {
                    unsafe {
                        libc::free(path_ptr as *mut _);
                    }
                }
                log::debug!(
                    "Cache::resolve_path {:?} resolved to {:?}",
                    path,
                    resolved_path
                );

                Some(resolved_path)
            }
        }
    }

    /// Returns the absolute form of a path, with all intermediate components normalized
    /// and symbolic links resolved.
    pub fn resolve<P>(path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!("Cache::resolve resolving path: {:?}", path);

        Self::resolve_path(std::ptr::null_mut(), path)
    }

    /// Returns the absolute form of a path, with all intermediate components normalized
    /// and symbolic links resolved.
    ///
    /// Saves the result in the `Cache`.
    pub fn resolve_and_cache<P>(&mut self, path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "Cache::resolve_and_cache resolving and caching path: {:?}",
            path
        );

        Self::resolve_path(self.inner, path)
    }

    #[doc(hidden)]
    /// Finds the first device matching the tag. Saves the result in a `Cache` if `cache_ptr` in
    /// not NULL.
    fn resolve_tag(cache_ptr: *mut libmount::libmnt_cache, tag: &Tag) -> Option<PathBuf> {
        let tag_name = ffi_utils::as_ref_str_to_c_string(tag.name().to_string()).ok()?;
        let tag_value = ffi_utils::as_ref_str_to_c_string(tag.value()).ok()?;

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_resolve_tag(
                tag_name.as_ptr(),
                tag_value.as_ptr(),
                cache_ptr,
            ));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("Cache::resolve_tag found no device with tag {:?}. libmount::mnt_resolve_tag returned a NULL pointer", tag);

                None
            }
            device_ptr => {
                let device_name = ffi_utils::const_c_char_array_to_path_buf(device_ptr);
                if cache_ptr.is_null() {
                    unsafe {
                        libc::free(device_ptr as *mut _);
                    }
                }
                log::debug!(
                    "Cache::resolve_tag found first device with tag {:?}: {:?}",
                    tag,
                    device_name
                );

                Some(device_name)
            }
        }
    }

    /// Finds the first device  with the given `tag`.
    pub fn find_first_device_with_tag(tag: &Tag) -> Option<PathBuf> {
        log::debug!(
            "Cache::find_first_device_with_tag searching for first device with tag: {:?}",
            tag
        );

        Self::resolve_tag(std::ptr::null_mut(), tag)
    }

    /// Finds the first device  with the given `tag`, and saves the result in this `Cache`.
    pub fn find_and_cache_first_device_with_tag(&mut self, tag: &Tag) -> Option<PathBuf> {
        log::debug!(
            "Cache::find_and_cache_first_device_with_tag searching for first device with tag: {:?}",
            tag
        );

        Self::resolve_tag(self.inner, tag)
    }

    #[doc(hidden)]
    /// Finds the name of the device associated with the given path, and saves the result in a
    /// `Cahce` if `cache_ptr` is not NULL.
    fn resolve_target<P>(cache_ptr: *mut libmount::libmnt_cache, path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_cstr = ffi_utils::as_ref_path_to_c_string(path).ok()?;

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_resolve_target(path_cstr.as_ptr(), cache_ptr));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("Cache::resolve_target found no device associated with path {:?}. libmount::mnt_resolve_target returned a NULL pointer", path);

                None
            }
            device_ptr => {
                let device_name = ffi_utils::const_c_char_array_to_path_buf(device_ptr);
                if cache_ptr.is_null() {
                    unsafe {
                        libc::free(device_ptr as *mut _);
                    }
                }
                log::debug!(
                    "Cache::resolve_target found device {:?} associated with path {:?}",
                    device_name,
                    path
                );

                Some(device_name)
            }
        }
    }

    /// Finds the name of the device mounted at the given path.
    pub fn find_device_mounted_at<P>(path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "Cache::find_device_mounted_at searching for device mounted at path {:?}",
            path
        );

        Self::resolve_target(std::ptr::null_mut(), path)
    }

    /// Finds the name of the device mounted at the given path, and saves the result in this `Cache`.
    pub fn find_and_cache_device_mounted_at<P>(&mut self, path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        log::debug!(
            "Cache::find_and_cache_device_mounted_at searching for device mounted at path {:?}",
            path
        );

        Self::resolve_target(self.inner, path)
    }
}

impl AsRef<Cache> for Cache {
    fn as_ref(&self) -> &Cache {
        self
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        log::debug!("Cache::drop deallocation `Cache` instance");

        unsafe { libmount::mnt_free_cache(self.inner) }
    }
}
