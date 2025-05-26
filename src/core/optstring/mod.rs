// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level functions to manipulate mount option strings.

// From dependency library

// From standard library
use std::collections::HashSet;
use std::mem::MaybeUninit;

// From this library
use crate::core::errors::OptionIterError;
use crate::core::flags::MountFlag;
use crate::core::flags::UserspaceMountFlag;
use crate::ffi_utils;
pub use option_filter_enum::OptionFilter;
pub use option_iter_struct::OptionIter;

mod option_filter_enum;
mod option_iter_struct;

/// Returns a new list of mount options with `option_name=options_value,` prepended to it, or `None` if
/// an error occurred.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "";
///     let option_name = "ro";
///     let option_value = "recursive";
///
///     let actual = optstring::prepend_option(options_list, option_name, option_value);
///     let options = "ro=recursive,".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     let options_list = "user=";
///     let option_name = "ro";
///     let option_value = "recursive";
///
///     let actual = optstring::prepend_option(options_list, option_name, option_value);
///     let options = "ro=recursive,user=".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     let options_list = "user=";
///     let option_name = "context";
///     let option_value = "\"system_u:object_r:tmp_t:s0:c127,c456\"";
///
///     let actual = optstring::prepend_option(options_list, option_name, option_value);
///     let options = "context=\"system_u:object_r:tmp_t:s0:c127,c456\",user=".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn prepend_option(options_list: &str, option_name: &str, option_value: &str) -> Option<String> {
    log::debug!(
        "core::optstring::prepend_option prepending option {:?} with value {:?} to list {:?}.",
        option_name,
        option_value,
        options_list
    );

    let mut options_list_c_copy = ffi_utils::as_ref_str_to_owned_c_char_array(options_list).ok()?;
    let option_name_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;
    let option_value_cstr = ffi_utils::as_ref_str_to_c_string(option_value).ok()?;

    let result = unsafe {
        libmount::mnt_optstr_prepend_option(
            options_list_c_copy.as_mut_ptr(),
            option_name_cstr.as_ptr(),
            option_value_cstr.as_ptr(),
        )
    };

    let ptr = unsafe { options_list_c_copy.assume_init() };

    match result {
        0 => {
            log::debug!(
                "core::optstring::prepend_option prepended option {:?} with value {:?} to list {:?}.",
                option_name,
                option_value,
                options_list
            );

            let new_list = ffi_utils::c_char_array_to_string(ptr);
            // mnt_optstr_prepend_option reallocates the options list. We need to free it to avoid a
            // memory leak.
            unsafe {
                libc::free(ptr as *mut _);
            }

            Some(new_list)
        }
        code => {
            let err_msg = format!(
                "failed to prepend option {:?} with value {:?} to list {:?}",
                option_name, option_value, options_list
            );
            log::debug!("core::optstring::prepend_option {}. mnt_optstr_prepend_option returned error code {:?}", err_msg, code);

            // Free the memory allocated for the C copy of the options list.
            unsafe {
                libc::free(ptr as *mut _);
            }

            None
        }
    }
}

/// Returns a new list of mount options with `option_name=options_value` appended to it, or `None` if
/// an error occurred.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "";
///     let option_name = "ro";
///     let option_value = "recursive";
///
///     let actual = optstring::append_option(options_list, option_name, option_value);
///     let options = "ro=recursive".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     let options_list = "user=";
///     let option_name = "ro";
///     let option_value = "recursive";
///
///     let actual = optstring::append_option(options_list, option_name, option_value);
///     let options = "user=,ro=recursive".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     let options_list = "user=";
///     let option_name = "context";
///     let option_value = "\"system_u:object_r:tmp_t:s0:c127,c456\"";
///
///     let actual = optstring::append_option(options_list, option_name, option_value);
///     let options = "user=,context=\"system_u:object_r:tmp_t:s0:c127,c456\"".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn append_option(options_list: &str, option_name: &str, option_value: &str) -> Option<String> {
    log::debug!(
        "core::optstring::append_option appending option {:?} with value {:?} to list {:?}.",
        option_name,
        option_value,
        options_list
    );

    let mut options_list_c_copy = ffi_utils::as_ref_str_to_owned_c_char_array(options_list).ok()?;
    let option_name_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;
    let option_value_cstr = ffi_utils::as_ref_str_to_c_string(option_value).ok()?;

    let result = unsafe {
        libmount::mnt_optstr_append_option(
            options_list_c_copy.as_mut_ptr(),
            option_name_cstr.as_ptr(),
            option_value_cstr.as_ptr(),
        )
    };

    let ptr = unsafe { options_list_c_copy.assume_init() };

    match result {
        0 => {
            log::debug!(
                "core::optstring::append_option appended option {:?} with value {:?} to list {:?}.",
                option_name,
                option_value,
                options_list
            );

            let new_list = ffi_utils::c_char_array_to_string(ptr);
            // mnt_optstr_append_option reallocates the options list. We need to free it to avoid a
            // memory leak.
            unsafe {
                libc::free(ptr as *mut _);
            }

            Some(new_list)
        }
        code => {
            let err_msg = format!(
                "failed to append option {:?} with value {:?} to list {:?}",
                option_name, option_value, options_list
            );
            log::debug!("core::optstring::append_option {}. mnt_optstr_append_option returned error code {:?}", err_msg, code);

            // Free the memory allocated for the C copy of the options list.
            unsafe {
                libc::free(ptr as *mut _);
            }

            None
        }
    }
}

/// Returns a new list of options with all but the last instance of `option_name` removed.
/// `None` if there is no option matching `option_name`, or an error occurred.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "async,remount,async,ro=,async";
///     let option_name = "async";
///
///     let actual = optstring::deduplicate_option(options_list, option_name);
///     let options = "remount,ro=,async".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn deduplicate_option(options_list: &str, option_name: &str) -> Option<String> {
    log::debug!(
        "core::optstring::deduplicate_option removing duplicates of option {:?} from list {:?}.",
        option_name,
        options_list
    );

    let mut options_list_c_copy = ffi_utils::as_ref_str_to_owned_c_char_array(options_list).ok()?;
    let option_name_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;

    let result = unsafe {
        libmount::mnt_optstr_deduplicate_option(
            options_list_c_copy.as_mut_ptr(),
            option_name_cstr.as_ptr(),
        )
    };

    let ptr = unsafe { options_list_c_copy.assume_init() };

    match result {
        0 => {
            log::debug!(
                "core::optstring::deduplicate_option removed duplicates of option {:?} from list {:?}.",
                option_name,
                options_list
            );

            let new_list = ffi_utils::c_char_array_to_string(ptr);
            // mnt_optstr_deduplicate_option reallocates the options list. We need to free it to avoid a
            // memory leak.
            unsafe {
                libc::free(ptr as *mut _);
            }

            Some(new_list)
        }
        1 => {
            log::debug!(
                "core::optstring::deduplicate_option found no option {:?} in list: {:?}",
                option_name,
                options_list
            );

            // Free the memory allocated for the C copy of the options list.
            unsafe {
                libc::free(ptr as *mut _);
            }

            None
        }
        code => {
            let err_msg = format!(
                "failed to remove duplicates of option {:?} from list {:?}",
                option_name, options_list
            );
            log::debug!("core::optstring::deduplicate_option {}. mnt_optstr_deduplicate_option returned error code {:?}", err_msg, code);

            // Free the memory allocated for the C copy of the options list.
            unsafe {
                libc::free(ptr as *mut _);
            }

            None
        }
    }
}

/// Returns a new list of options with the first instance of `option_name` removed.
/// `None` if there is no option matching `option_name`, or an error occurred.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "async,remount,async,ro=,async";
///     let option_name = "async";
///
///     let actual = optstring::remove_option(options_list, option_name);
///     let options = "remount,async,ro=,async".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn remove_option(options_list: &str, option_name: &str) -> Option<String> {
    log::debug!(
        "core::optstring::remove_option removing option {:?} from list {:?}.",
        option_name,
        options_list
    );

    let mut options_list_c_copy = ffi_utils::as_ref_str_to_owned_c_char_array(options_list).ok()?;
    let option_name_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;

    let result = unsafe {
        libmount::mnt_optstr_remove_option(
            options_list_c_copy.as_mut_ptr(),
            option_name_cstr.as_ptr(),
        )
    };

    let ptr = unsafe { options_list_c_copy.assume_init() };

    match result {
        0 => {
            log::debug!(
                "core::optstring::remove_option removed option {:?} from list {:?}.",
                option_name,
                options_list
            );

            let new_list = ffi_utils::c_char_array_to_string(ptr);
            // mnt_optstr_remove_option reallocates the options list. We need to free it to avoid a
            // memory leak.
            unsafe {
                libc::free(ptr as *mut _);
            }

            Some(new_list)
        }
        1 => {
            log::debug!(
                "core::optstring::remove_option found no option {:?} in list: {:?}",
                option_name,
                options_list
            );

            // Free the memory allocated for the C copy of the options list.
            unsafe {
                libc::free(ptr as *mut _);
            }

            None
        }
        code => {
            let err_msg = format!(
                "failed to remove option {:?} from list {:?}",
                option_name, options_list
            );
            log::debug!("core::optstring::remove_option {}. mnt_optstr_remove_option returned error code {:?}", err_msg, code);

            // Free the memory allocated for the C copy of the options list.
            unsafe {
                libc::free(ptr as *mut _);
            }

            None
        }
    }
}

/// Returns the substring containing the value of an option of the form `option_name=`,
/// `option_name=option_value`, or `option_name="option value"` from a comma-separated list of
/// mount options.
///
/// This function returns `None` if there is no option matching `option_name` in the list, or an
/// error occurred.
///
/// **Warning:** some option values might contain commas, in which case the value has to be
/// properly quoted. Otherwise, the function will interpret the commas as a separator between mount
/// options.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list =
///       "user=,ro=recursive,context=\"system_u:object_r:tmp_t:s0:c127,c456\"";
///     let option_name = "user";
///
///     let actual = optstring::option_value(options_list, option_name);
///     let value = "";
///     let expected = Some(value);
///     assert_eq!(actual, expected);
///
///     let options_list =
///       "user=,ro=recursive,context=\"system_u:object_r:tmp_t:s0:c127,c456\"";
///     let option_name = "ro";
///
///     let actual = optstring::option_value(options_list, option_name);
///     let value = "recursive";
///     let expected = Some(value);
///     assert_eq!(actual, expected);
///
///     let options_list =
///       "user=,ro=recursive,context=\"system_u:object_r:tmp_t:s0:c127,c456\"";
///     let option_name = "context";
///
///     let actual = optstring::option_value(options_list, option_name);
///     let value = "\"system_u:object_r:tmp_t:s0:c127,c456\"";
///     let expected = Some(value);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
///
/// ```
pub fn option_value<'a>(options_list: &'a str, option_name: &str) -> Option<&'a str> {
    log::debug!(
        "core::optstring::option_value getting value of option {:?} in list {:?}.",
        option_name,
        options_list
    );

    let options_list_cstr = ffi_utils::as_ref_str_to_c_string(options_list).ok()?;
    let option_name_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;

    let mut start_ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
    let mut size_ptr = MaybeUninit::<usize>::zeroed();

    let result = unsafe {
        libmount::mnt_optstr_get_option(
            options_list_cstr.as_ptr(),
            option_name_cstr.as_ptr(),
            start_ptr.as_mut_ptr(),
            size_ptr.as_mut_ptr(),
        )
    };

    match result {
        0 => match unsafe { start_ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("core::optstring::option_value option {:?} does not have a value. libmount::mnt_optstr_get_option returned a NULL pointer", option_name);

                None
            }
            ptr => {
                let size = unsafe { size_ptr.assume_init() };
                let start = unsafe { ptr.offset_from(options_list_cstr.as_ptr()) as usize };
                let end = start + size;

                let value = &options_list[start..end];
                log::debug!(
                    "core::optstring::option_value option {:?} has value: {:?}",
                    option_name,
                    value
                );

                Some(value)
            }
        },
        1 => {
            let err_msg = format!(
                "did not find the value of option {:?} in the options list {:?}",
                option_name, options_list
            );
            log::debug!("core::optstring::option_value {}. libmount::mnt_optstr_get_option returned error code 1", err_msg);

            None
        }
        code => {
            let err_msg = format!(
                "failed to find option {:?} in options list {:?}",
                option_name, options_list
            );
            log::debug!("core::optstring::option_value {}. libmount::mnt_optstr_get_option returned error code {:?}", err_msg, code);

            None
        }
    }
}

#[doc(hidden)]
/// Sets an option's value, or unsets it if `option_value` is the NULL pointer.
fn set_option(
    mut options_list: MaybeUninit<*mut libc::c_char>,
    option_name: *const libc::c_char,
    option_value: *const libc::c_char,
) -> Option<String> {
    let result = unsafe {
        libmount::mnt_optstr_set_option(options_list.as_mut_ptr(), option_name, option_value)
    };

    let ptr = unsafe { options_list.assume_init() };

    match result {
        0 => {
            log::debug!("core::optstring::set_option set option value");

            let new_list = ffi_utils::c_char_array_to_string(ptr);
            // mnt_optstr_set_option reallocates the options list. We need to free it to avoid a
            // memory leak.
            unsafe {
                libc::free(ptr as *mut _);
            }

            Some(new_list)
        }
        1 => {
            log::debug!("core::optstring::set_option found no option to set");

            // Free the memory allocated for the C copy of the options list.
            unsafe {
                libc::free(ptr as *mut _);
            }

            None
        }
        code => {
            let err_msg = "failed to set option value".to_owned();
            log::debug!(
                "core::optstring::set_option {}. mnt_optstr_set_option returned error code {:?}",
                err_msg,
                code
            );

            // Free the memory allocated for the C copy of the options list.
            unsafe {
                libc::free(ptr as *mut _);
            }

            None
        }
    }
}

/// Returns a new list of options with the value of the option matching `option_name` set to
/// `option_value`. This function return `None` if it finds no matching option, or an error
/// occurred.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     // Set an option's value.
///     let options_list = "rbind,ro";
///     let option_name = "ro";
///     let option_value = "recursive";
///
///     let actual = optstring::set_option_value(options_list, option_name, option_value);
///     let options = "rbind,ro=recursive".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     // Update an option's value.
///     let options_list = "ro,user=root,noauto,unhide";
///     let option_name = "user";
///     let option_value = "tux";
///
///     let actual = optstring::set_option_value(options_list, option_name, option_value);
///     let options = "ro,user=tux,noauto,unhide".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn set_option_value(
    options_list: &str,
    option_name: &str,
    option_value: &str,
) -> Option<String> {
    log::debug!(
        "core::optstring::set_option_value setting value of option {:?} in list {:?} to {:?}.",
        option_name,
        options_list,
        option_value,
    );

    let options_list_c_copy = ffi_utils::as_ref_str_to_owned_c_char_array(options_list).ok()?;
    let option_name_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;
    let option_value_cstr = ffi_utils::as_ref_str_to_c_string(option_value).ok()?;

    set_option(
        options_list_c_copy,
        option_name_cstr.as_ptr(),
        option_value_cstr.as_ptr(),
    )
}

/// Returns a new list of options with the value of the option of the form
/// `option_name=option_value` removed. This function return `None` if it finds no matching
/// option, or an error occurred.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "rbind,ro=recursive";
///     let option_name = "ro";
///
///     let actual = optstring::unset_option_value(options_list, option_name);
///     let options = "rbind,ro".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn unset_option_value(options_list: &str, option_name: &str) -> Option<String> {
    log::debug!(
        "core::optstring::unset_option_value unsetting value of option {:?} in list {:?}.",
        option_name,
        options_list,
    );

    let options_list_c_copy = ffi_utils::as_ref_str_to_owned_c_char_array(options_list).ok()?;
    let option_name_cstr = ffi_utils::as_ref_str_to_c_string(option_name).ok()?;

    set_option(
        options_list_c_copy,
        option_name_cstr.as_ptr(),
        std::ptr::null(),
    )
}

#[doc(hidden)]
/// Parses an options list, and returns the mount flags matching options in the list.
fn get_flags(
    options_list: &str,
    option_map: *const libmount::libmnt_optmap,
) -> Option<libc::c_ulong> {
    let options_list_cstr = ffi_utils::as_ref_str_to_c_string(options_list).ok()?;

    let mut flags_ptr = MaybeUninit::<libc::c_ulong>::zeroed();
    let result = unsafe {
        libmount::mnt_optstr_get_flags(
            options_list_cstr.as_ptr(),
            flags_ptr.as_mut_ptr(),
            option_map,
        )
    };

    match result {
        0 => {
            let flags = unsafe { flags_ptr.assume_init() };
            log::debug!("optstring::get_flags value: {:x}", flags);

            Some(flags)
        }
        code => {
            let err_msg = "failed to get flags for options in option list".to_owned();
            log::debug!(
                "optstring::get_flags {}. mnt_optstr_get_flags returned error code {:?}",
                err_msg,
                code
            );

            None
        }
    }
}

#[doc(hidden)]
/// Converts bit flags to [`MountFlag`]s.
fn flags_to_mount_flags(flags: libc::c_ulong) -> Option<HashSet<MountFlag>> {
    let mount_flags: HashSet<_> = enum_iterator::all::<MountFlag>()
        .filter(|&mf| mf as u64 & flags != 0)
        .collect();
    if mount_flags.is_empty() {
        None
    } else {
        Some(mount_flags)
    }
}

#[doc(hidden)]
/// Converts bit flags to [`UserspaceMountFlag`]s.
fn flags_to_userspace_mount_flags(flags: libc::c_ulong) -> Option<HashSet<UserspaceMountFlag>> {
    let mount_flags: HashSet<_> = enum_iterator::all::<UserspaceMountFlag>()
        .filter(|&mf| mf as u64 & flags != 0)
        .collect();
    if mount_flags.is_empty() {
        None
    } else {
        Some(mount_flags)
    }
}

/// Returns the [`MountFlag`]s matching options in the `options_list`. This function return `None`
/// if it finds no matching option, or an error occurred.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use std::collections::HashSet;
/// use rsmount::core::flags::MountFlag;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "noauto,noatime,bind";
///
///     let actual = optstring::find_mount_flags(options_list);
///     let expected_flags = HashSet::from([
///         MountFlag::NoUpdateAccessTime,
///         MountFlag::Bind,
///         ]);
///     let expected = Some(expected_flags);
///     assert_eq!(actual, expected);
///
///     // One option can map to multiple flags.
///     let options_list = "noexec";
///
///     let actual = optstring::find_mount_flags(options_list);
///     let expected_flags = HashSet::from([
///         MountFlag::Secure,
///         MountFlag::NoExecute,
///         ]);
///     let expected = Some(expected_flags);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn find_mount_flags(options_list: &str) -> Option<HashSet<MountFlag>> {
    let option_map = unsafe { libmount::mnt_get_builtin_optmap(libmount::MNT_LINUX_MAP as i32) };

    get_flags(options_list, option_map).and_then(flags_to_mount_flags)
}

/// Returns the [`UserspaceMountFlag`]s matching options in the `options_list`. This function
/// return `None` if it finds no matching option, or an error occurred.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use std::collections::HashSet;
/// use rsmount::core::flags::UserspaceMountFlag;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "noauto,noatime,bind";
///
///     let actual = optstring::find_userspace_mount_flags(options_list);
///     let expected_flags = HashSet::from([UserspaceMountFlag::NoAuto]);
///     let expected = Some(expected_flags);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn find_userspace_mount_flags(options_list: &str) -> Option<HashSet<UserspaceMountFlag>> {
    let option_map =
        unsafe { libmount::mnt_get_builtin_optmap(libmount::MNT_USERSPACE_MAP as i32) };

    get_flags(options_list, option_map).and_then(flags_to_userspace_mount_flags)
}

macro_rules! unwrap_or_return {
    ($e:expr, $ret:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => {
                log::debug!(
                    concat!("optstring::unwrap_or_return ", stringify!($e), " {:?}"),
                    e
                );

                return $ret;
            }
        }
    };
}

/// Returns `true` if the list of mount options does (or does not) contain an element of the `pattern`
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
/// | List of mount options          | Search pattern   | Result |
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
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "atime,nodiratime,noatime,bind,discard";
///     let pattern = "atime";
///
///     let actual = optstring::matches_any_option(options_list, pattern);
///     let expected = true;
///     assert_eq!(actual, expected);
///
///     let options_list = "atime,nodiratime,noatime,bind,discard";
///     let pattern = "noatime";
///
///     let actual = optstring::matches_any_option(options_list, pattern);
///     let expected = false;
///     assert_eq!(actual, expected);
///
///     let options_list = "atime,nodiratime,noatime,bind,discard";
///     let pattern = "+noatime";
///
///     let actual = optstring::matches_any_option(options_list, pattern);
///     let expected = true;
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn matches_any_option(options_list: &str, pattern: &str) -> bool {
    let pattern_cstr = unwrap_or_return!(ffi_utils::as_ref_str_to_c_string(pattern), false);
    let options_list_cstr =
        unwrap_or_return!(ffi_utils::as_ref_str_to_c_string(options_list), false);

    let state = unsafe {
        libmount::mnt_match_options(options_list_cstr.as_ptr(), pattern_cstr.as_ptr()) == 1
    };
    log::debug!(
        "opstring::matches_any_option does any option in pattern: {:?} match option in list: {:?}? {:?}",
        pattern,
        options_list,
        state
    );

    state
}

/// Returns an iterator over the options in the given `options_list`.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::tables::MountOption;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "noatime,ro=recursive";
///     let mut iterator = optstring::iter_options(options_list)?;
///
///     // First option.
///     let actual = iterator.next();
///     let option: MountOption = "noatime".parse()?;
///     let expected = Some(option);
///     assert_eq!(actual, expected);
///
///     // Second option.
///     let actual = iterator.next();
///     let option: MountOption = "ro=recursive".parse()?;
///     let expected = Some(option);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn iter_options(options_list: &str) -> Result<OptionIter, OptionIterError> {
    OptionIter::new(options_list)
}

#[doc(hidden)]
/// Extracts options from the options list that match mount flags/userspace mount flags.
fn get_options(
    options_list: &str,
    option_map: *const libmount::libmnt_optmap,
    skip: &[OptionFilter],
) -> Option<String> {
    log::debug!(
        "optstring::get_options extracting options from list {:?} with filters {:?}",
        options_list,
        skip
    );

    let options_list_cstr = ffi_utils::as_ref_str_to_c_string(options_list).ok()?;
    let mut option_ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
    let ignore = skip.iter().fold(0i32, |acc, &x| acc | x as i32);

    let result = unsafe {
        libmount::mnt_optstr_get_options(
            options_list_cstr.as_ptr(),
            option_ptr.as_mut_ptr(),
            option_map,
            ignore,
        )
    };

    match result {
        0 => {
            match unsafe { option_ptr.assume_init() } {
                ptr if ptr.is_null() => {
                    log::debug!("optstring::get_options found no match");

                    None
                }
                ptr => {
                    let options = ffi_utils::c_char_array_to_string(ptr);

                    // option_ptr points to memory allocated by `mnt_optstr_get_options`, we free it here
                    // to avoid a leak.
                    unsafe {
                        libc::free(ptr as *mut _);
                    }

                    log::debug!("optstring::get_options extracted options {:?}", options);

                    Some(options)
                }
            }
        }
        code => {
            let err_msg = format!(
                "failed to extract options from list {:?} with filters {:?}",
                options_list, skip
            );
            log::debug!(
                "optstring::get_options {}. mnt_optstr_get_options returned error code: {:?}",
                err_msg,
                code
            );

            None
        }
    }
}

/// Returns all file system independent options from the list of mount options, skipping the ones
/// matching any of the given [`OptionFilter`]s.
///
/// For more information about file system specific mount options see the [`mount` command's
/// manpage](https://www.man7.org/linux/man-pages/man8/mount.8.html#FILESYSTEM-INDEPENDENT_MOUNT_OPTIONS).
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring::OptionFilter;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "noowner,protect,sync,noauto,verbose,rw,lazytime";
///     let skip = [];
///
///     let actual = optstring::take_fs_independent_options(options_list, skip);
///     let options = "sync,rw,lazytime".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     let options_list = "noowner,protect,sync,noauto,verbose,rw,lazytime";
///     let skip = [OptionFilter::FsIo];
///
///     let actual = optstring::take_fs_independent_options(options_list, skip);
///     let options = "rw".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn take_fs_independent_options<T>(options_list: &str, skip: T) -> Option<String>
where
    T: AsRef<[OptionFilter]>,
{
    let skip = skip.as_ref();
    let option_map = unsafe { libmount::mnt_get_builtin_optmap(libmount::MNT_LINUX_MAP as i32) };

    get_options(options_list, option_map, skip)
}

/// Returns all userspace options from the list of mount options, skipping the ones matching any of
/// the given [`OptionFilter`]s.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring::OptionFilter;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "noowner,protect,sync,noauto,verbose,rw,lazytime";
///     let skip = [];
///
///     let actual = optstring::take_userspace_options(options_list, skip);
///     let options = "noowner,noauto".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     let options_list = "noowner,protect,sync,noauto,verbose,rw,lazytime";
///     let skip = [OptionFilter::Negated];
///
///     let actual = optstring::take_userspace_options(options_list, skip);
///     let options = "noauto".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn take_userspace_options<T>(options_list: &str, skip: T) -> Option<String>
where
    T: AsRef<[OptionFilter]>,
{
    let skip = skip.as_ref();
    let option_map =
        unsafe { libmount::mnt_get_builtin_optmap(libmount::MNT_USERSPACE_MAP as i32) };

    get_options(options_list, option_map, skip)
}

/// Returns all file system specific options in the list of mount options.
///
/// For more information about file system specific mount options see the [`mount` command's
/// manpage](https://www.man7.org/linux/man-pages/man8/mount.8.html#FILESYSTEM-SPECIFIC_MOUNT_OPTIONS).
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::optstring::OptionFilter;
/// use rsmount::core::optstring;
///
/// fn main() -> rsmount::Result<()> {
///     let options_list = "noowner,protect,sync,noauto,verbose,rw,lazytime";
///
///     let actual = optstring::take_fs_specific_options(options_list);
///     let options = "protect,verbose".to_owned();
///     let expected = Some(options);
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
pub fn take_fs_specific_options(options_list: &str) -> Option<String> {
    log::debug!(
        "optstring::take_fs_specific_options getting file system specific options from list: {:?}",
        options_list
    );

    let options_list_cstr = ffi_utils::as_ref_str_to_c_string(options_list).ok()?;
    let mut options_ptr = MaybeUninit::<*mut libc::c_char>::zeroed();

    let result = unsafe {
        libmount::mnt_split_optstr(
            options_list_cstr.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            options_ptr.as_mut_ptr(),
            0,
            0,
        )
    };

    match result {
        0 => {
            match unsafe { options_ptr.assume_init() } {
                ptr if ptr.is_null() => {
                    log::debug!(
                        "optstring::take_fs_specific_options found no file system specific options"
                    );

                    None
                }
                ptr => {
                    let options = ffi_utils::c_char_array_to_string(ptr);

                    // option_ptr points to memory allocated by `mnt_optstr_split_optstr`, we free it here
                    // to avoid a leak.
                    unsafe {
                        libc::free(ptr as *mut _);
                    }

                    log::debug!(
                        "optstring::take_fs_specific_options extracted options {:?}",
                        options
                    );

                    Some(options)
                }
            }
        }
        code => {
            let err_msg = format!(
                "failed to extract file system specific options from list {:?}",
                options_list
            );
            log::debug!(
                "optstring::get_options {}. mnt_split_optstr returned error code: {:?}",
                err_msg,
                code
            );

            None
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn prepend_option_does_not_prepend_an_option_with_both_an_empty_name_and_value() {
        let option_name = "";
        let option_value = "";

        let options_list = "";

        let actual = prepend_option(options_list, option_name, option_value);
        let options = String::new();
        let expected = Some(options);
        assert_eq!(actual, expected);

        let options_list = "noatime";

        let actual = prepend_option(options_list, option_name, option_value);
        let options = "noatime".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);
    }

    #[test]
    fn prepend_option_does_not_prepend_an_option_with_an_empty_name() {
        let option_name = "";
        let option_value = "recursive";

        let options_list = "";

        let actual = prepend_option(options_list, option_name, option_value);
        let options = String::new();
        let expected = Some(options);
        assert_eq!(actual, expected);

        let options_list = "noatime";

        let actual = prepend_option(options_list, option_name, option_value);
        let options = "noatime".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);
    }

    #[test]
    fn prepend_option_prepends_an_option_to_a_list() {
        let option_name = "ro";
        let option_value = "recursive";
        let options_list = "";

        let actual = prepend_option(options_list, option_name, option_value);
        let options = "ro=recursive,".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);

        let options_list = "noatime";

        let actual = prepend_option(options_list, option_name, option_value);
        let options = "ro=recursive,noatime".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);
    }

    #[test]
    fn append_option_does_not_append_an_option_with_both_an_empty_name_and_value() {
        let option_name = "";
        let option_value = "";

        let options_list = "";

        let actual = append_option(options_list, option_name, option_value);
        let options = String::new();
        let expected = Some(options);
        assert_eq!(actual, expected);

        let options_list = "noatime";

        let actual = append_option(options_list, option_name, option_value);
        let options = "noatime".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);
    }

    #[test]
    fn append_option_does_not_append_an_option_with_an_empty_name() {
        let option_name = "";
        let option_value = "recursive";

        let options_list = "";

        let actual = append_option(options_list, option_name, option_value);
        let options = String::new();
        let expected = Some(options);
        assert_eq!(actual, expected);

        let options_list = "noatime";

        let actual = append_option(options_list, option_name, option_value);
        let options = "noatime".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);
    }

    #[test]
    fn append_option_appends_an_option_to_a_list() {
        let option_name = "ro";
        let option_value = "recursive";
        let options_list = "";

        let actual = append_option(options_list, option_name, option_value);
        let options = "ro=recursive".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);

        let options_list = "noatime";

        let actual = append_option(options_list, option_name, option_value);
        let options = "noatime,ro=recursive".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);
    }

    #[test]
    fn deduplicate_option_does_not_remove_duplicates_from_an_empty_list() {
        let option_name = "async";
        let options_list = "";

        let actual = deduplicate_option(options_list, option_name);
        let expected = None;

        assert_eq!(actual, expected);
    }

    #[test]
    fn deduplicate_option_does_not_remove_duplicates_from_a_list_with_no_instance_of_option() {
        let option_name = "async";
        let options_list = "ro=,noatime";

        let actual = deduplicate_option(options_list, option_name);
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn deduplicate_option_does_not_remove_duplicates_from_a_list_of_unique_options() {
        let option_name = "async";
        let options_list = "ro=,async,noatime";

        let actual = deduplicate_option(options_list, option_name);
        let options = "ro=,async,noatime".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);
    }

    #[test]
    fn deduplicate_option_removes_duplicates_from_a_list_with_duplicates() {
        let option_name = "async";
        let options_list = "async,async,async";

        let actual = deduplicate_option(options_list, option_name);
        let options = "async".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);

        let option_name = "async";
        let options_list = "async,remount,async,ro=,async,noatime";

        let actual = deduplicate_option(options_list, option_name);
        let options = "remount,ro=,async,noatime".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_option_does_not_remove_an_option_from_an_empty_list() {
        let option_name = "async";
        let options_list = "";

        let actual = remove_option(options_list, option_name);
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_option_does_not_remove_from_a_list_with_no_instance_of_option() {
        let option_name = "async";
        let options_list = "ro=,noatime";

        let actual = remove_option(options_list, option_name);
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_option_removes_the_first_instance_of_option_in_a_list() {
        let option_name = "async";
        let options_list = "async,async,async";

        let actual = remove_option(options_list, option_name);
        let options = "async,async".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);

        let option_name = "async";
        let options_list = "ro=,async,noatime";

        let actual = remove_option(options_list, option_name);
        let options = "ro=,noatime".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);

        let option_name = "async";
        let options_list = "async,remount,async,ro=,async,noatime";

        let actual = remove_option(options_list, option_name);
        let options = "remount,async,ro=,async,noatime".to_owned();
        let expected = Some(options);
        assert_eq!(actual, expected);
    }

    #[test]
    fn option_value_can_not_get_an_option_value_from_an_empty_list() {
        let option_name = "context";
        let options_list = "";

        let actual = option_value(options_list, option_name);
        assert!(actual.is_none());
    }

    #[test]
    fn option_value_can_not_get_a_value_from_an_option_not_in_the_list() {
        let option_name = "context";
        let options_list = "ro=recursive,async,noatime";

        let actual = option_value(options_list, option_name);
        assert!(actual.is_none());
    }

    #[test]
    fn option_value_can_not_get_a_value_from_an_option_without_value() {
        let option_name = "async";
        let options_list = "ro=,async,noatime";

        let actual = option_value(options_list, option_name);
        assert!(actual.is_none());
    }

    #[test]
    fn option_value_gets_an_empty_string_from_an_option_with_an_empty_value() {
        let option_name = "ro";
        let options_list = "ro=,async,noatime";

        let actual = option_value(options_list, option_name);
        let value = "";
        let expected = Some(value);
        assert_eq!(actual, expected);
    }

    #[test]
    fn option_value_can_get_a_value_from_an_option_with_value() {
        let option_name = "ro";
        let options_list = "ro=recursive,async,noatime";

        let actual = option_value(options_list, option_name);
        let value = "recursive";
        let expected = Some(value);
        assert_eq!(actual, expected);
    }

    #[test]
    #[ignore]
    fn set_option_value_can_not_set_the_value_of_an_option_not_in_the_list() {
        let options_list = "";
        let option_name = "ro";
        let option_value = "recursive";

        let actual = set_option_value(options_list, option_name, option_value);
        let expected = None;
        assert_eq!(actual, expected);

        let options_list = "async,noatime";
        let option_name = "ro";
        let option_value = "recursive";

        let actual = set_option_value(options_list, option_name, option_value);
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    #[ignore]
    fn unset_option_value_can_not_remove_the_value_of_an_option_not_in_the_list() {
        let options_list = "";
        let option_name = "ro";

        let actual = unset_option_value(options_list, option_name);
        let expected = None;
        assert_eq!(actual, expected);

        let options_list = "async,noatime";
        let option_name = "ro";

        let actual = unset_option_value(options_list, option_name);
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn find_mount_flags_returns_none_given_an_empty_options_list() {
        let options_list = "";

        let actual = find_mount_flags(options_list);
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn find_mount_flags_returns_none_given_an_options_list_without_matching_options() {
        let options_list = "ro=relative,async";

        let actual = find_mount_flags(options_list);
        let expected = None;
        assert_eq!(actual, expected);
    }

    #[test]
    fn find_mount_flags_returns_flags_matching_the_ones_in_the_list() {
        let options_list = "bind,noatime";

        let actual = find_mount_flags(options_list);
        let flags = HashSet::from([MountFlag::NoUpdateAccessTime, MountFlag::Bind]);
        let expected = Some(flags);

        assert_eq!(actual, expected);
    }

    // see
    // https://github.com/util-linux/util-linux/blob/stable/v2.39/libmount/src/optmap.c#L71
    // for a full list of option-mount flag mapping
    #[test]
    fn take_fs_independent_options_can_not_extract_options_from_an_empty_list() {
        let options_list = "";
        let skip = [];

        let actual = take_fs_independent_options(options_list, skip);
        let expected = None;

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_independent_options_can_not_extract_options_from_a_list_of_non_matching_options() {
        let options_list = "protect,usemp,verbose";
        let skip = [];

        let actual = take_fs_independent_options(options_list, skip);
        let expected = None;

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_independent_options_can_extract_options_from_list_with_one_matching_option_no_skip()
    {
        let options_list = "bind";
        let skip = [];

        let actual = take_fs_independent_options(options_list, skip);
        let options = "bind".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);

        let options_list = "protect,bind,usemp,verbose";
        let skip = [];

        let actual = take_fs_independent_options(options_list, skip);
        let options = "bind".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_independent_options_can_extract_options_from_list_with_multiple_matching_option_no_skip(
    ) {
        let options_list = "protect,verbose,rw,bind,noexec,sync,lazytime";
        let skip = [];

        let actual = take_fs_independent_options(options_list, skip);
        let options = "rw,bind,noexec,sync,lazytime".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_independent_options_can_extract_options_skipping_negated() {
        let options_list = "protect,verbose,rw,bind,noexec,sync,sub,lazytime";
        let skip = [OptionFilter::Negated];

        let actual = take_fs_independent_options(options_list, skip);
        let options = "bind,noexec,sync,lazytime".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_independent_options_can_extract_options_skipping_fs_io() {
        let options_list = "protect,verbose,rw,bind,noexec,sync,sub,lazytime";
        let skip = [OptionFilter::FsIo];

        let actual = take_fs_independent_options(options_list, skip);
        let options = "rw,bind,noexec".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_independent_options_can_extract_options_skipping_negated_and_fs_io() {
        let options_list = "protect,verbose,rw,bind,noexec,sync,sub,lazytime";
        let skip = [OptionFilter::Negated, OptionFilter::FsIo];

        let actual = take_fs_independent_options(options_list, skip);
        let options = "bind,noexec".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    // see
    // https://github.com/util-linux/util-linux/blob/stable/v2.39/libmount/src/optmap.c#L148
    // for a full list of option-userspace mount flag mapping
    #[test]
    fn take_userspace_options_can_not_extract_options_from_an_empty_list() {
        let options_list = "";
        let skip = [];

        let actual = take_userspace_options(options_list, skip);
        let expected = None;

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_userspace_options_can_not_extract_options_from_a_list_of_non_matching_options() {
        let options_list = "protect,usemp,verbose";
        let skip = [];

        let actual = take_userspace_options(options_list, skip);
        let expected = None;

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_userspace_options_can_extract_options_from_list_with_one_matching_option_no_skip() {
        let options_list = "auto";
        let skip = [];

        let actual = take_userspace_options(options_list, skip);
        let options = "auto".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);

        let options_list = "protect,auto,usemp,verbose";

        let actual = take_userspace_options(options_list, skip);
        let options = "auto".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_userspace_options_can_extract_options_skipping_negated() {
        let options_list = "protect,verbose,_netdev,auto,users,nogroup,nofail";
        let skip = [OptionFilter::Negated];

        let actual = take_userspace_options(options_list, skip);
        let options = "_netdev,users,nofail".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_userspace_options_can_extract_options_skipping_not_in_mountinfo() {
        let options_list = "protect,verbose,_netdev,auto,users,nogroup,nofail";
        let skip = [OptionFilter::NotInMountInfo];

        let actual = take_userspace_options(options_list, skip);
        let options = "_netdev".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_userspace_options_can_extract_options_skipping_negated_and_not_in_mountinfo() {
        let options_list = "protect,verbose,_netdev,auto,users,nogroup,nofail";
        let skip = [OptionFilter::Negated, OptionFilter::NotInMountInfo];

        let actual = take_userspace_options(options_list, skip);
        let options = "_netdev".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_specific_options_can_not_extract_options_from_an_empty_list() {
        let options_list = "";

        let actual = take_fs_specific_options(options_list);
        let expected = None;

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_specific_options_can_not_extract_options_from_a_list_of_non_matching_options() {
        let options_list = "noowner,sync,noauto,rw,lazytime";

        let actual = take_fs_specific_options(options_list);
        let expected = None;

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_specific_options_can_extract_options_from_list_with_one_matching_option() {
        let options_list = "protect";

        let actual = take_fs_specific_options(options_list);
        let options = "protect".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);

        let options_list = "noowner,protect,sync,noauto,rw,lazytime";

        let actual = take_fs_specific_options(options_list);
        let options = "protect".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }

    #[test]
    fn take_fs_specific_options_can_extract_options_from_list_with_multiple_matching_options() {
        let options_list = "protect,verbose";

        let actual = take_fs_specific_options(options_list);
        let options = "protect,verbose".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);

        let options_list = "noowner,protect,sync,noauto,verbose,rw,lazytime";

        let actual = take_fs_specific_options(options_list);
        let options = "protect,verbose".to_owned();
        let expected = Some(options);

        assert_eq!(actual, expected);
    }
}
