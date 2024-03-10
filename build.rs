// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::env;

// From this library

fn main() {
    if let Ok(version_hex) = env::var("DEP_MOUNT_VERSION_NUMBER") {
        let version = u64::from_str_radix(&version_hex, 16).unwrap();

        if version >= 2390 {
            println!("cargo:rustc-cfg=v2_39");
        }
    }
}
