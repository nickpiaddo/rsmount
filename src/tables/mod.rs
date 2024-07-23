// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Module for working with file system description files.
//!
//! # Table of Contents
//! 1. [Description](#description)
//!
//! ## Description
//!
//! On Linux, descriptive information about the devices the system can mount, or devices already
//! mounted are kept in files, respectively `/etc/fstab` and `/proc/mounts` (or the per-process
//! `/proc/<pid>/mountinfo` file).
//!
//! This modules provides tools to load, search, edit, create, or compare file system description
//! files.
//!

// From dependency library

// From standard library

// From this library
pub use mount_option_struct::MountOption;

mod mount_option_struct;
