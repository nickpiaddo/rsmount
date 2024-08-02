// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;

// From this library
use crate::core::entries::UTabEntry;
use crate::core::errors::UTabError;
use crate::declare_tab;
use crate::utab_shared_methods;

declare_tab!(
    UTab,
    r##"
An in-memory representation of `/run/mount/utab`.

 # `/run/mount/utab`

 ```text
 SRC=/dev/vda TARGET=/mnt ROOT=/ OPTS=x-initrd.mount
 ```

// FIXME get a definition for each item
// from source file <https://github.com/util-linux/util-linux/blob/stable/v2.39/libmount/src/tab_parse.c#L310>
  - **ID**:
  - **SRC**: the mounted device,
  - **TARGET**: the device's mount point,
  - **ROOT**:
  - **BINDSRC**: the source of a bind mount,
  - **OPTS**: mount options,
  - **ATTRS**: options independent from those used by the [`mount`
  syscall](https://manpages.org/mount/2) and [`mount` command](https://manpages.org/mount/8).
  They are neither sent to the kernel, nor interpreted by `libmount`. They are stored in
  `/run/mount/utab`, and managed by `libmount` in userspace only.
"##
);

utab_shared_methods!(UTab, UTabEntry, UTabError);

impl UTab {
    /// Parses the given file, then appends the entries it collected to the table.
    ///
    /// **Note:**
    /// - by default, comment lines are ignored during import. If you want them included, call
    /// [`UTab::import_with_comments`] **before** invoking this method.
    /// - the parser ignores lines with syntax errors. It will report defective lines to the caller
    /// through an error callback function.
    ///
    // FIXME Defective lines are reported to the caller by the errcb() function (see mnt_table_set_parser_errcb()).
    // can not currently wrap the function `mnt_table_set_parser_errcb`
    fn import_file<T>(&mut self, file_path: T) -> Result<(), UTabError>
    where
        T: AsRef<Path>,
    {
        let file_path = file_path.as_ref();
        let file_path_cstr = ffi_utils::as_ref_path_to_c_string(file_path)?;
        log::debug!(
            "UTab::import_file importing table entries from file {:?}",
            file_path
        );

        let result = unsafe { libmount::mnt_table_parse_file(self.inner, file_path_cstr.as_ptr()) };

        match result {
            0 => {
                log::debug!(
                    "UTab::import_file imported table entries from file {:?}",
                    file_path
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to import table entries from file {:?}", file_path);
                log::debug!("UTab::import_file {}. libmount::mnt_table_parse_file returned error code: {:?}", err_msg, code);

                Err(UTabError::Import(err_msg))
            }
        }
    }

    // FIXME there is no function to import the content of /run/mount/utab from upstream
    // using a workaround
    /// Parses the `/run/mount/utab` file, then appends the entries it
    /// collects to this `UTab`.
    pub fn import_utab(&mut self) -> Result<(), UTabError> {
        self.import_file("/run/mount/utab")
    }
}

impl fmt::Display for UTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output: Vec<String> = vec![];

        for line in self.iter() {
            output.push(line.to_string());
        }

        write!(f, "{}", output.join("\n"))
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn utab_can_import_run_mount_utab() -> crate::Result<()> {
        let mut utab = UTab::new()?;

        utab.import_utab()?;

        Ok(())
    }
}
