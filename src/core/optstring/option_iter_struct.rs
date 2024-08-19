// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::errors::OptionIterError;
use crate::ffi_utils;
use crate::tables::MountOption;

/// An iterator over options in a list of mount options.
#[derive(Debug)]
pub struct OptionIter<'a> {
    options_list: &'a str,
    cursor: MaybeUninit<*mut libc::c_char>,
    origin: *const libc::c_char,
}

impl<'a> OptionIter<'a> {
    #[doc(hidden)]
    /// Creates a new `OptionIter` instance.
    pub(crate) fn new(options_list: &'a str) -> Result<OptionIter<'a>, OptionIterError> {
        log::debug!("OptionIter::new creating a new `OptionIter` instance");

        let cursor = ffi_utils::as_ref_str_to_owned_c_char_array(options_list).map_err(|err| {
            let err_msg = format!(
                "failed to create new `OptionIter` for options list: {:?}. {:?}",
                options_list, err
            );

            OptionIterError::Creation(err_msg)
        })?;

        let origin = unsafe { *cursor.as_ptr() };

        let iterator = Self {
            options_list,
            cursor,
            origin,
        };

        Ok(iterator)
    }
}

impl<'a> Iterator for OptionIter<'a> {
    type Item = MountOption;

    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("OptionIter::next getting next option in option list");
        let mut name_ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
        let mut name_size = MaybeUninit::<usize>::zeroed();
        let mut value_ptr = MaybeUninit::<*mut libc::c_char>::zeroed();
        let mut value_size = MaybeUninit::<usize>::zeroed();

        let result = unsafe {
            libmount::mnt_optstr_next_option(
                self.cursor.as_mut_ptr(),
                name_ptr.as_mut_ptr(),
                name_size.as_mut_ptr(),
                value_ptr.as_mut_ptr(),
                value_size.as_mut_ptr(),
            )
        };

        match result {
            0 => {
                let name_ptr = unsafe { name_ptr.assume_init() };
                let name_size = unsafe { name_size.assume_init() };
                let value_ptr = unsafe { value_ptr.assume_init() };
                let value_size = unsafe { value_size.assume_init() };

                let name_start = unsafe { name_ptr.offset_from(self.origin) as usize };
                let name_end = name_start + name_size;

                let name = &self.options_list[name_start..name_end];

                let option = if value_ptr.is_null() {
                    MountOption::new(name)
                } else {
                    let value_start = unsafe { value_ptr.offset_from(self.origin) as usize };
                    let value_end = value_start + value_size;
                    let value = &self.options_list[value_start..value_end];

                    MountOption::new_with_value(name, value)
                };

                Some(option)
            }
            1 => {
                log::debug!("OptionIter::next reached the end of options list");

                None
            }
            code => {
                let err_msg = format!(
                    "failed to get next option from list: {:?}",
                    self.options_list
                );
                log::debug!(
                    "OptionIter::next {}. mnt_optstr_next_option returned error code: {:?}",
                    err_msg,
                    code
                );

                None
            }
        }
    }
}

impl<'a> AsRef<OptionIter<'a>> for OptionIter<'a> {
    #[inline]
    fn as_ref(&self) -> &OptionIter<'a> {
        self
    }
}

impl<'a> Drop for OptionIter<'a> {
    fn drop(&mut self) {
        log::debug!("OptionIter::drop deallocating `OptionIter` instance");

        unsafe { libc::free(self.origin as *mut _) }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::tables::MountOption;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn option_iter_can_iterate_over_an_empty_options_list() -> crate::Result<()> {
        let options_list = "";
        let mut iterator = OptionIter::new(options_list)?;

        let actual = iterator.next();
        let expected = None;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn option_iter_can_iterate_over_an_options_list_with_one_element() -> crate::Result<()> {
        let options_list = "noatime";
        let mut iterator = OptionIter::new(options_list).unwrap();

        let actual = iterator.next();
        let option: MountOption = "noatime".parse()?;
        let expected = Some(option);
        assert_eq!(actual, expected);

        // Reached the end of the list
        let actual = iterator.next();
        let expected = None;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn option_iter_can_iterate_over_an_options_list_with_more_than_one_element() -> crate::Result<()>
    {
        let options_list = "noatime,ro=recursive";
        let mut iterator = OptionIter::new(options_list).unwrap();

        // First option.
        let actual = iterator.next();
        let option: MountOption = "noatime".parse()?;
        let expected = Some(option);
        assert_eq!(actual, expected);

        // Second option.
        let actual = iterator.next();
        let option: MountOption = "ro=recursive".parse()?;
        let expected = Some(option);
        assert_eq!(actual, expected);

        // Reached the end of the list
        let actual = iterator.next();
        let expected = None;
        assert_eq!(actual, expected);

        Ok(())
    }
}
