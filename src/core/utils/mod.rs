// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Miscellaneous utilities.

// From dependency library

// From standard library
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};

// From this library
use crate::core::cache::Cache;
use crate::core::fs::FileSystem;
use crate::ffi_utils;

#[doc(hidden)]
/// Converts a device number to its corresponding name.
fn guess_system_root(device_number: u64, cache: *mut libmount::libmnt_cache) -> Option<PathBuf> {
    let mut ptr = MaybeUninit::<*mut libc::c_char>::zeroed();

    let result = unsafe { libmount::mnt_guess_system_root(device_number, cache, ptr.as_mut_ptr()) };

    match result {
        0 => {
            match unsafe { ptr.assume_init() } {
                ptr if ptr.is_null() => {
                    let err_msg = format!(
                        "found device with no name for device number: {:?}",
                        device_number
                    );
                    log::debug!("guess_system_root {}. libmount::mnt_guess_system_root returned a NULL pointer", err_msg);

                    None
                }
                ptr => {
                    let device_name = ffi_utils::const_c_char_array_to_path_buf(ptr);
                    unsafe {
                        libc::free(ptr as *mut _);
                    }

                    Some(device_name)
                }
            }
        }
        1 => {
            let err_msg = format!(
                "found no device name for device number: {:?}",
                device_number
            );
            log::debug!(
                "guess_system_root {}. libmount::mnt_guess_system_root returned error code: 1",
                err_msg
            );

            None
        }
        code => {
            let err_msg = format!(
                "failed to find device name for device number: {:?}",
                device_number
            );
            log::debug!(
                "guess_system_root {}. libmount::mnt_guess_system_root returned error code: {:?}",
                err_msg,
                code
            );

            None
        }
    }
}

/// Converts a device number to its corresponding name.
///
/// Returns `None` if it finds no device matching the given `device_number`, or an error occurs.
pub fn device_number_to_device_name(device_number: u64) -> Option<PathBuf> {
    log::debug!(
        "device_number_to_device_name converting device number {:?} to device name",
        device_number
    );

    guess_system_root(device_number, std::ptr::null_mut())
}

/// Converts a device number to its corresponding name.
///
/// Returns `None` if it finds no device matching the given `device_number`, or an error occurs.
pub fn device_number_to_cached_device_name(device_number: u64, cache: &Cache) -> Option<PathBuf> {
    log::debug!(
        "device_number_to_cached_device_name converting device number {:?} to device name",
        device_number
    );

    guess_system_root(device_number, cache.inner)
}

/// Finds the mountpoint of a mounted device.
///
/// For better accuracy, the given `device_path` should be in canonical form.
///
/// **Warning:** due to the search method, this function might fail to find mountpoints for Linux
/// overlays.
pub fn find_device_mountpoint<T>(device_path: T) -> Option<PathBuf>
where
    T: AsRef<Path>,
{
    let device_path = device_path.as_ref();
    let device_path_cstr = ffi_utils::as_ref_path_to_c_string(device_path);

    match device_path_cstr {
        Ok(path) => {
            let mut ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
            unsafe {
                ptr.write(libmount::mnt_get_mountpoint(path.as_ptr()));
            }

            match unsafe { ptr.assume_init() } {
                ptr if ptr.is_null() => {
                    let err_msg =
                        format!("failed to find mountpoint for device: {:?}", device_path);
                    log::debug!(
                        "find_device_mountpoint {}. libmount::mnt_get_mountpoint returned a NULL pointer",
                        err_msg
                    );

                    None
                }
                ptr => {
                    let mountpoint = ffi_utils::const_c_char_array_to_path_buf(ptr);
                    unsafe {
                        libc::free(ptr as *mut _);
                    }
                    log::debug!("find_device_mountpoint device mountpoint: {:?}", mountpoint);

                    Some(mountpoint)
                }
            }
        }
        Err(e) => {
            log::debug!("find_device_mountpoint {:?}", e);

            None
        }
    }
}

/// Encodes a `string` to a format compatible with `fstab` by escaping space, tab, new line, and
/// backslash characters.
pub fn fstab_encode<T>(string: T) -> Option<String>
where
    T: AsRef<str>,
{
    let string = string.as_ref();
    let string_cstr = ffi_utils::as_ref_str_to_c_string(string);
    log::debug!("fstab_encode encoding string: {:?}", string);

    match string_cstr {
        Ok(string_cstr) => {
            let mut ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
            unsafe {
                ptr.write(libmount::mnt_mangle(string_cstr.as_ptr()));
            }

            match unsafe { ptr.assume_init() } {
                ptr if ptr.is_null() => {
                    let err_msg = format!("failed to encode string: {:?}", string);
                    log::debug!(
                        "fstab_encode {}. libmount::mnt_mangle returned a NULL pointer",
                        err_msg
                    );

                    None
                }
                ptr => {
                    let encoded = ffi_utils::c_char_array_to_string(ptr);
                    unsafe {
                        libc::free(ptr as *mut _);
                    }
                    log::debug!("fstab_encode encoded string: {:?}", encoded);

                    Some(encoded)
                }
            }
        }
        Err(e) => {
            log::debug!("fstab_encode {:?}", e);

            None
        }
    }
}

/// Decodes a `string` encoded to a format compatible with `fstab` into its original form.
pub fn fstab_decode<T>(string: T) -> Option<String>
where
    T: AsRef<str>,
{
    let string = string.as_ref();
    let string_cstr = ffi_utils::as_ref_str_to_c_string(string);
    log::debug!("fstab_decode decoding string: {:?}", string);

    match string_cstr {
        Ok(string_cstr) => {
            let mut ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
            unsafe {
                ptr.write(libmount::mnt_unmangle(string_cstr.as_ptr()));
            }

            match unsafe { ptr.assume_init() } {
                ptr if ptr.is_null() => {
                    let err_msg = format!("failed to decode string: {:?}", string);
                    log::debug!(
                        "fstab_decode {}. libmount::mnt_unmangle returned a NULL pointer",
                        err_msg
                    );

                    None
                }
                ptr => {
                    let decoded = ffi_utils::c_char_array_to_string(ptr);
                    unsafe {
                        libc::free(ptr as *mut _);
                    }
                    log::debug!("fstab_decode decoded string: {:?}", decoded);

                    Some(decoded)
                }
            }
        }
        Err(e) => {
            log::debug!("fstab_decode {:?}", e);

            None
        }
    }
}

/// Returns the path to either the file system description file `fstab`, or the one set in the
/// environment variable `LIBMOUNT_FSTAB`.
///
/// Returns `None` in case an error occurred while retrieving data.
pub fn path_to_fstab() -> Option<PathBuf> {
    log::debug!("path_to_fstab getting path to `fstab`");

    let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();
    unsafe {
        ptr.write(libmount::mnt_get_fstab_path());
    }

    match unsafe { ptr.assume_init() } {
        ptr if ptr.is_null() => {
            let err_msg = "failed to get path to `fstab`";
            log::debug!(
                "path_to_fstab {}. libmount::mnt_get_fstab_path returned a NULL pointer",
                err_msg
            );

            None
        }
        ptr => {
            let path = ffi_utils::const_c_char_array_to_path_buf(ptr);
            log::debug!("path_to_fstab path to `fstab`: {:?}", path);

            Some(path)
        }
    }
}

/// Returns the path to either the `swaps` description file, or the one set in the
/// environment variable `LIBMOUNT_SWAPS`.
///
/// Returns `None` in case an error occurred while retrieving data.
pub fn path_to_swaps() -> Option<PathBuf> {
    log::debug!("path_to_swaps getting path to `swaps`");

    let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();
    unsafe {
        ptr.write(libmount::mnt_get_swaps_path());
    }

    match unsafe { ptr.assume_init() } {
        ptr if ptr.is_null() => {
            let err_msg = "failed to get path to `swaps`";
            log::debug!(
                "path_to_swaps {}. libmount::mnt_get_swaps_path returned a NULL pointer",
                err_msg
            );

            None
        }
        ptr => {
            let path = ffi_utils::const_c_char_array_to_path_buf(ptr);
            log::debug!("path_to_swaps path to `swaps`: {:?}", path);

            Some(path)
        }
    }
}

/// Returns `true` if the given file system type is a network file system.
///
/// # Examples
///
/// ```
/// use rsmount::core::fs::FileSystem;
/// use rsmount::core::utils;
///
/// fn main() -> rsmount::Result<()> {
///
///     assert_eq!(utils::is_network_fs(FileSystem::Cifs), true);
///     assert_eq!(utils::is_network_fs(FileSystem::NFS), true);
///     assert_eq!(utils::is_network_fs(FileSystem::Ext2), false);
///
///     Ok(())
/// }
/// ```
pub fn is_network_fs(fs_type: FileSystem) -> bool {
    let fs_type_cstr = ffi_utils::as_ref_str_to_c_string(&fs_type);
    log::debug!(
        "is_network_fs checking if {:?} is a network file system",
        fs_type
    );

    match fs_type_cstr {
        Ok(fs_type_cstr) => {
            let state = unsafe { libmount::mnt_fstype_is_netfs(fs_type_cstr.as_ptr()) == 1 };
            log::debug!("is_network_fs {:?}: {:?}", fs_type, state);

            state
        }
        Err(e) => {
            log::debug!("is_network_fs {:?}", e);

            false
        }
    }
}

/// Returns `true` if the given file system type is a pseudo file system.
///
/// # Examples
///
/// ```
/// use rsmount::core::fs::FileSystem;
/// use rsmount::core::utils;
///
/// fn main() -> rsmount::Result<()> {
///
///     assert_eq!(utils::is_pseudo_fs(FileSystem::Proc), true);
///     assert_eq!(utils::is_pseudo_fs(FileSystem::Sysfs), true);
///     assert_eq!(utils::is_pseudo_fs(FileSystem::Ext2), false);
///
///     Ok(())
/// }
/// ```
pub fn is_pseudo_fs(fs_type: FileSystem) -> bool {
    let fs_type_cstr = ffi_utils::as_ref_str_to_c_string(&fs_type);
    log::debug!(
        "is_pseudo_fs checking if {:?} is a pseudo file system",
        fs_type
    );

    match fs_type_cstr {
        Ok(fs_type_cstr) => {
            let state = unsafe { libmount::mnt_fstype_is_pseudofs(fs_type_cstr.as_ptr()) == 1 };
            log::debug!("is_pseudo_fs {:?}: {:?}", fs_type, state);

            state
        }
        Err(e) => {
            log::debug!("is_pseudo_fs {:?}", e);

            false
        }
    }
}

/// Returns `true` if an item in the `pattern` (a comma-separated list of values) matches the file
/// system type `fs_type`.
///
/// **Note**:
/// - an item in the `pattern` prefixed with `no` will match if it is **NOT** the file system type
/// provided. For example, a `"noext4"` pattern means *return `true` if the `ext4` file system type
/// is not the parameter `fs_type`*.
/// - prefixing the `pattern` with a `no` will negate all the items in the list. For
/// example, the pattern `"noapfs,ntfs,ext4"` is equivalent to `"noapfs,nontfs,noext4"` which means
/// *return `true` if the file system type is neither of `apfs`, `ntfs` or `ext4`*.
///
/// # Examples
///
/// | File system | Search pattern         | Result |
/// | ----        | ----                   | ----   |
/// | ""          | ""                     | true   |
/// | ""          | "noext4"               | true   |
/// | ""          | "ext4"                 | false  |
/// | "ext4"      | ""                     | false  |
/// | "ext4"      | "ext4"                 | true   |
/// | "xfs"       | "ext4"                 | false  |
/// | "ext4"      | "noext4"               | false  |
/// | "xfs"       | "noext4"               | true   |
/// | "ext4"      | "apfs,ntfs,ext4"       | true   |
/// | "ext4"      | "apfs,ntfs,noext4"     | false  |
/// | "ext4"      | "noapfs,ntfs,ext4"     | false  |
/// | "ext4"      | "noapfs,nontfs,noext4" | false  |
/// | "xfs"       | "noapfs,ntfs,ext4"     | true   |
/// | "xfs"       | "noapfs,nontfs,noext4" | true   |
///
pub fn matches_fs_type(fs_type: &str, pattern: &str) -> bool {
    let fs_type_cstr = ffi_utils::as_ref_str_to_c_string(fs_type);
    let pattern_cstr = ffi_utils::as_ref_str_to_c_string(pattern);

    match (fs_type_cstr, pattern_cstr) {
        (Ok(fs_type_cstr), Ok(pattern_cstr)) => {
            let state = unsafe {
                libmount::mnt_match_fstype(fs_type_cstr.as_ptr(), pattern_cstr.as_ptr()) == 1
            };
            log::debug!("matches_fs_type {:?}", state);

            state
        }
        (Err(e), _) | (_, Err(e)) => {
            log::debug!("matches_fs_type {:?}", e);

            false
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn matches_fs_type_an_empty_pattern_matches_only_an_empty_fs_type() {
        let fs_type = "";
        let pattern = "";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = true;
        assert_eq!(actual, expected);

        let fs_type = "ext4";
        let pattern = "";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = false;
        assert_eq!(actual, expected);
    }

    #[test]
    fn matches_fs_type_pattern_matches_an_empty_fs_type() {
        let fs_type = "";

        let pattern = "ext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = false;
        assert_eq!(actual, expected);

        let pattern = "noext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = true;
        assert_eq!(actual, expected);

        let pattern = "noapfs,ext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = true;
        assert_eq!(actual, expected);
    }

    #[test]
    fn matches_fs_type_matches_a_pattern() {
        let fs_type = "ext4";

        let pattern = "ext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = true;
        assert_eq!(actual, expected);

        let pattern = "apfs,ntfs,ext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = true;
        assert_eq!(actual, expected);

        let pattern = "apfs,ntfs,noext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = false;
        assert_eq!(actual, expected);

        let pattern = "noapfs,ntfs,ext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = false;
        assert_eq!(actual, expected);

        let pattern = "noapfs,nontfs,noext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = false;
        assert_eq!(actual, expected);
    }

    #[test]
    fn matches_fs_type_negated_patterns_match() {
        let fs_type = "xfs";

        let pattern = "noext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = true;
        assert_eq!(actual, expected);

        let pattern = "noapfs,ntfs,ext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = true;
        assert_eq!(actual, expected);

        let pattern = "noapfs,nontfs,noext4";
        let actual = matches_fs_type(fs_type, pattern);
        let expected = true;
        assert_eq!(actual, expected);
    }
}
