// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::env;

// From this library

fn main() {
    if let Ok(version_hex) = env::var("DEP_MOUNT_VERSION_NUMBER") {
        let version = u64::from_str_radix(&version_hex, 16).unwrap();
        // Add to the list of expected config names and values that is used when checking the
        // reachable cfg expressions with the unexpected_cfgs lint.
        // see: https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-check-cfg
        println!("cargo::rustc-check-cfg=cfg(mount, values(\"v2_39\"))");

        if version >= 2390 {
            println!("cargo:rustc-cfg=mount=\"v2_39\"");
        }
    }
}
