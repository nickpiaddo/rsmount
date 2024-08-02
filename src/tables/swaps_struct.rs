// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;

// From this library
use crate::core::entries::SwapsEntry;
use crate::core::errors::SwapsError;
use crate::declare_tab;
use crate::swaps_shared_methods;

declare_tab!(
    Swaps,
    r##"
An in-memory representation of `/proc/swaps`.

# `/proc/swaps`

`/proc/swaps` records swap space, and its utilization. For systems with only one swap partition, the
output of `/proc/swaps` may be similar to the following:

```text
Filename                                Type            Size            Used            Priority
/dev/sda2                               partition       1048572         0               -2
```

`/proc/swaps` provides a snapshot of every swap `Filename`, the `Type` of swap space, the total
`Size`, and the amount of space `Used` (in kilobytes). The `Priority` column is useful when
multiple swap files are in use. The lower the priority, the more likely the swap file is used.
"##
);

swaps_shared_methods!(Swaps, SwapsEntry, SwapsError);

impl Swaps {
    /// Parses `/proc/swaps`, then appends the data it collected to the table.
    pub fn import_proc_swaps(&mut self) -> Result<(), SwapsError> {
        log::debug!("Swaps::import_proc_swaps importing entries from /proc/swaps");

        let result = unsafe { libmount::mnt_table_parse_swaps(self.inner, std::ptr::null()) };

        match result {
            0 => {
                log::debug!("Swaps::import_proc_swaps imported entries from /proc/swaps");

                Ok(())
            }
            code => {
                let err_msg = "failed to import entries from /proc/swaps".to_owned();
                log::debug!("Swaps::import_proc_swaps {}. libmount::mnt_table_parse_swaps returned error code: {:?}", err_msg, code);

                Err(SwapsError::Import(err_msg))
            }
        }
    }
}

impl fmt::Display for Swaps {
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
    fn swaps_can_import_proc_swaps() -> crate::Result<()> {
        let mut proc_swaps = Swaps::new()?;

        proc_swaps.import_proc_swaps()?;

        Ok(())
    }
}
