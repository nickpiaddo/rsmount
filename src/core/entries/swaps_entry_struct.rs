// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;

// From this library
use crate::core::errors::SwapsEntryError;
use crate::declare_tab_entry;
use crate::swaps_entry_shared_methods;

declare_tab_entry!(
    SwapsEntry,
    r"A line in `/proc/swaps`.

For example:
```text
/dev/sda2                               partition       1048572         0               -2
```
"
);

swaps_entry_shared_methods!(SwapsEntry, SwapsEntryError);

impl SwapsEntry {
    //---- BEGIN setters

    /// Sets the priority of the swap device; swap priority takes a value between `-1` and `32767`.
    ///
    /// Higher numbers indicate higher priority (for more information see the [`swapon` command's
    /// manpage](https://manpages.org/swapon/8)).
    pub fn set_priority(&mut self, priority: i32) -> Result<(), SwapsEntryError> {
        log::debug!(
            "SwapsEntry::set_priority setting swap priority to: {:?}",
            priority
        );

        match unsafe { libmount::mnt_fs_set_priority(self.inner, priority) } {
            0 => {
                log::debug!(
                    "SwapsEntry::set_priority set swap priority to: {:?}",
                    priority
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to set swap priority to: {:?}", priority);
                log::debug!("SwapsEntry::set_priority {}. libmount::mnt_fs_set_priority returned error code: {:?}", err_msg, code);

                Err(SwapsEntryError::Config(err_msg))
            }
        }
    }

    //---- END setters

    //---- BEGIN getters

    /// Returns the type of swap partition.
    pub fn swap_type(&self) -> Option<&str> {
        log::debug!("SwapsEntry::swap_type getting swap type");

        let mut ptr = MaybeUninit::<*const libc::c_char>::zeroed();

        unsafe {
            ptr.write(libmount::mnt_fs_get_swaptype(self.inner));
        }

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                log::debug!("SwapsEntry::swap_type failed to get swap type. libmount::mnt_fs_get_swaptype returned a NULL pointer");

                None
            }

            ptr => {
                let swap_type = ffi_utils::const_char_array_to_str_ref(ptr);
                log::debug!("SwapsEntry::swap_type value: {:?}", swap_type);

                swap_type.ok()
            }
        }
    }

    /// Returns the total size of the swap partition (in kibibytes).
    pub fn size(&self) -> usize {
        let size = unsafe { libmount::mnt_fs_get_size(self.inner) as usize };
        log::debug!("SwapsEntry::size value: {:?}", size);

        size
    }

    /// Returns the size of the swap space used (in kibibytes).
    pub fn size_used(&self) -> usize {
        let size = unsafe { libmount::mnt_fs_get_usedsize(self.inner) as usize };
        log::debug!("SwapsEntry::size_used size: {:?}", size);

        size
    }

    /// Returns the priority number of the swap partition.
    pub fn priority(&self) -> i32 {
        let priority = unsafe { libmount::mnt_fs_get_priority(self.inner) };
        log::debug!("SwapsEntry::priority value: {:?}", priority);

        priority
    }

    //---- END getters
}

impl fmt::Display for SwapsEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formatting from Linux Kernel
        // linux/mm/swapfile.c
        // 2702:           seq_puts(swap, "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n");
        let mut output: Vec<String> = vec![];
        if let Some(path) = self.source_path() {
            let source_path = format!("{}\t\t", path.display());
            output.push(source_path.to_string());
        }

        if let Some(swap_type) = self.swap_type() {
            output.push(swap_type.to_string());
        }

        let size = self.size();
        output.push(size.to_string());

        let size_used = self.size_used();
        output.push(size_used.to_string());

        let priority = self.priority();
        output.push(priority.to_string());

        write!(f, "{}", output.join("\t\t"))
    }
}
