# rsmount

----

[![Crates.io Version](https://img.shields.io/crates/v/rsmount?labelColor=%23222222&color=%23fdb42f)][1]
[![docs.rs](https://img.shields.io/docsrs/rsmount?labelColor=%23222222&color=%2322a884)][2]
![Crates.io MSRV](https://img.shields.io/crates/msrv/rsmount?labelColor=%23222222&color=%239c179e)
![Crates.io License](https://img.shields.io/crates/l/rsmount?labelColor=%23222222&color=%230d0887)

⚠️ WARNING: **This library is still in development, thus not yet suitable for
use in production.**

The `rsmount` library is a safe Rust wrapper around [`util-linux/libmount`][3].

`rsmount` allows users to, among other things:

- mount devices on an operating system’s file hierarchy,
- list/manage mount points in `/proc/<pid>/mountinfo`,
- consult the system’s swap usage from /proc/swaps,
- compose/edit /etc/fstab, the file describing all devices an OS should mount
  at boot.
- etc.

## Usage

This crate requires `libmount` version `2.39.2` or later.

Add the following to your `Cargo.toml`:

```toml
[dependencies]
rsmount = "0.2.0"
```

Then install the system packages below before running `cargo build`:

- `util-linux`: to generate Rust bindings from `libmount`'s header files.
- `libclang`: to satisfy the [dependency][4] of [`bindgen`][5] on `libclang`.
- `pkg-config`: to detect system libraries.

Read the [installation instructions](#install-required-dependencies) below to
install the required dependencies on your system.

[Documentation (docs.rs)][2]

## Example

In this example we mount a disk on `/mnt/backup`.

```rust
use rsmount::core::device::BlockDevice;
use rsmount::core::flags::MountFlag;
use rsmount::core::fs::FileSystem;
use rsmount::mount::Mount;

fn main() -> rsmount::Result<()> {
    // Configure the `Mount` struct.
    let block_device: BlockDevice = "/dev/vda".parse()?;
    let mut mount = Mount::builder()
        // Device to mount.
        .source(block_device.into())
        // Location of the mount point in the file tree.
        .target("/mnt/backup")
        // Do not allow writing to the file system while it is mounted.
        .mount_flags(vec![MountFlag::ReadOnly])
        // Gives a hint about the file system used by the device (optional).
        .file_system(FileSystem::Ext4)
        .build()?;

    // Mount `/dev/vda` at `/mnt/backup`.
    mount.mount_device()?;

    Ok(())
}

```

## Install required dependencies

### Alpine Linux

As `root`, issue the following command:

```console
apk add util-linux-dev clang-libclang pkgconfig
```

### NixOS

Install the packages in a temporary environment with:

```console
nix-shell -p util-linux.dev libclang.lib pkg-config
```

or permanently with:

```console
nix-env -iA nixos.util-linux.dev nixos.libclang.lib nixos.pkg-config
```

### Ubuntu

```console
sudo apt-get install libmount-dev libclang-dev pkg-config
```

## License

This project is licensed under either of the following:

- [Apache License, Version 2.0][6]
- [MIT License][7]

at your discretion.

Files in the [third-party/][8] and [web-snapshots/][9] directories are subject
to their own licenses and/or copyrights.

SPDX-License-Identifier: Apache-2.0 OR MIT

Copyright (c) 2023 Nick Piaddo

[1]: https://crates.io/crates/rsmount
[2]: https://docs.rs/rsmount
[3]: https://github.com/util-linux/util-linux/tree/master
[4]: https://rust-lang.github.io/rust-bindgen/requirements.html#clang
[5]: https://crates.io/crates/bindgen
[6]: https://www.apache.org/licenses/LICENSE-2.0
[7]: https://opensource.org/licenses/MIT
[8]: ./third-party/
[9]: ./web-snapshots/
