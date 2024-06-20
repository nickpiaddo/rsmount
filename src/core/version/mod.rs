// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Functions to get library version data.

// From dependency library
use once_cell::sync::Lazy;
use std::mem::MaybeUninit;

// From standard library

// From this library
use crate::ffi_utils;

pub use version_error_enum::*;
mod version_error_enum;

/// Semantic version string e.g. `"2.39.4"`.
pub static VERSION_STRING: Lazy<&str> = Lazy::new(|| libmount::LIBMOUNT_VERSION.to_str().unwrap());

/// Semantic version major number e.g. `2`.
pub static VERSION_NUMBER_MAJOR: u32 = libmount::LIBMOUNT_MAJOR_VERSION as u32;

/// Semantic version minor number e.g. `39`.
pub static VERSION_NUMBER_MINOR: u32 = libmount::LIBMOUNT_MINOR_VERSION as u32;

/// Semantic version patch number e.g. `4`.
pub static VERSION_NUMBER_PATCH: u32 = libmount::LIBMOUNT_PATCH_VERSION as u32;

/// Converts a version string to the corresponding release code.
///
/// # Examples
///
/// ```
/// use rsmount::core::version;
///
/// fn main() -> rsmount::Result<()> {
///
///     let release_code = version::version_string_to_release_code("2.39.4")?;
///     assert_eq!(release_code, 2394);
///
///     Ok(())
/// }
/// ```
pub fn version_string_to_release_code<T>(version_string: T) -> Result<i32, VersionError>
where
    T: AsRef<str>,
{
    unsafe {
        let version_string = version_string.as_ref();
        let version_cstr = ffi_utils::as_ref_str_to_c_string(version_string)?;

        let version_code = libmount::mnt_parse_version_string(version_cstr.as_ptr());
        log::debug!(
            "version_string_to_release_code converted version string {:?} to release code {:?}",
            version_string,
            version_code
        );

        Ok(version_code)
    }
}

/// Returns a list of library features.
pub fn library_features() -> Result<Vec<String>, VersionError> {
    log::debug!("library_features getting list of library features");

    unsafe {
        let mut c_feature_array = MaybeUninit::<*mut *const libc::c_char>::zeroed();
        let mut array_len = MaybeUninit::<libc::c_int>::zeroed();

        array_len.write(libmount::mnt_get_library_features(
            c_feature_array.as_mut_ptr(),
        ));

        match array_len.assume_init() {
            len if len < 0 => {
                let err_msg = "failed to get library features".to_owned();
                log::debug!("library_features {}. libmount::mnt_get_library_features returned error code: {:?}", err_msg, len);

                Err(VersionError::FeaturesAccess(err_msg))
            }
            len => {
                let c_feature_array = c_feature_array.assume_init();
                let feature_array = std::slice::from_raw_parts(c_feature_array, len as usize);

                let features: Vec<_> = feature_array
                    .iter()
                    .map(|&feat| ffi_utils::c_char_array_to_string(feat))
                    .collect();

                log::debug!("library_features value: {:?}", features);

                Ok(features)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn version_string_to_release_code_fails_on_invalid_c_string() {
        let version_string = String::from_utf8(b"2.39\0.4".to_vec()).unwrap();
        let _result = version_string_to_release_code(version_string).unwrap();
    }

    #[test]
    fn version_string_to_release_code_converts_up_to_first_invalid_character() {
        let version_string = "v2.39.4";
        let expected: i32 = 0;
        let result = version_string_to_release_code(version_string).unwrap();

        assert_eq!(result, expected);

        let version_string = "2.39.x";
        let expected: i32 = 239;
        let result = version_string_to_release_code(version_string).unwrap();

        assert_eq!(result, expected);
    }
}
