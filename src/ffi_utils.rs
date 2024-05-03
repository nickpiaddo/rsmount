// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Collection of helper functions.

// From dependency library

// From standard library
use std::ffi::{CStr, CString, NulError, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

// From this library

//---- Conversion functions

#[doc(hidden)]
/// Converts a [`Path`] reference to a [`CString`].
pub fn as_ref_path_to_c_string<T>(path: T) -> Result<CString, NulError>
where
    T: AsRef<Path>,
{
    log::debug!(
        "ffi_utils::as_ref_path_to_c_string converting `AsRef<Path>` to `CString`: {:?}",
        path.as_ref()
    );

    CString::new(path.as_ref().as_os_str().as_bytes())
}

#[doc(hidden)]
/// Converts a [`str`](std::str) reference to a [`CString`].
pub fn as_ref_str_to_c_string<T>(string: T) -> Result<CString, NulError>
where
    T: AsRef<str>,
{
    let string: &str = string.as_ref();
    log::debug!(
        "ffi_utils::as_ref_str_to_c_string converting `&str` to `CString`: {:?}",
        string
    );

    CString::new(string.as_bytes())
}

#[doc(hidden)]
/// Converts a `const` [`c_char`](libc::c_char) C string to a [`PathBuf`].
///
///  # Safety
///
///  - Assumes the  memory pointed to by `ptr` contains a valid nul terminator at the end of the string.
///
///  - `ptr` must be valid for reads of bytes up to and including the null terminator. This means in particular:
///      The entire memory range of the C string must be contained within a single allocated object!
///      `ptr` must be non-null even for a zero-length `cstr`.
///
///  - The memory referenced by the returned CStr must not be mutated for the duration of lifetime 'a.
///
pub fn const_c_char_array_to_path_buf(ptr: *const libc::c_char) -> PathBuf {
    unsafe {
        log::debug!(
            "ffi_utils::const_c_char_array_to_path_buf converting `*const libc::c_char` to `PathBuf`: {:?}",
            CStr::from_ptr(ptr)
        );

        let bytes = CStr::from_ptr(ptr).to_bytes();
        Path::new(OsStr::from_bytes(bytes)).to_path_buf()
    }
}

#[doc(hidden)]
/// Converts a [`c_char`](libc::c_char) array to a [`String`].
pub fn c_char_array_to_string(ptr: *const libc::c_char) -> String {
    let cstr = unsafe { CStr::from_ptr(ptr) };
    log::debug!(
        "ffi_utils::c_char_array_to_string converting `*libc::c_char` to `String`: {:?}",
        cstr
    );

    // Get copy-on-write Cow<'_, str>, then guarantee a freshly-owned String allocation
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}
