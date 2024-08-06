// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Module for working with file system description files.
//!
//! # Table of Contents
//! 1. [Description](#description)
//! 2. [Examples](#examples)
//!     1. [Import `/etc/fstab` to RAM](#import-etcfstab-to-ram)
//!     2. [Manually create an `fstab` file](#manually-create-an-fstab-file)
//!     3. [Print `/proc/self/mountinfo` to the terminal](#print-procselfmountinfo-to-the-terminal)
//!
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
//! ## Examples
//!
//! ### Import `/etc/fstab` to RAM
//!
//! ```
//! use rsmount::tables::FsTab;
//!
//! fn main() -> rsmount::Result<()> {
//!     let mut fstab = FsTab::new()?;
//!
//!     // Configure the file importer.
//!     fstab.import_without_comments();
//!
//!     // Import `/etc/fstab` without comments lines in the file.
//!     fstab.import_etc_fstab()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Manually create an `fstab` file
//!
//! ```
//! # use tempfile::NamedTempFile;
//! use std::str::FromStr;
//! use rsmount::tables::FsTab;
//! use rsmount::core::entries::FsTabEntry;
//! use rsmount::core::device::BlockDevice;
//! use rsmount::core::device::Pseudo;
//! use rsmount::core::device::Source;
//! use rsmount::core::device::Tag;
//! use rsmount::core::fs::FileSystem;
//!
//! fn main() -> rsmount::Result<()> {
//!     let mut fstab = FsTab::new()?;
//!
//!     fstab.set_intro_comments("# /etc/fstab\n")?;
//!     fstab.append_to_intro_comments("# Example from scratch\n")?;
//!
//!     let uuid = Tag::from_str("UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f").map(Source::from)?;
//!     let entry1 = FsTabEntry::builder()
//!         .source(uuid)
//!         .target("/")
//!         .file_system_type(FileSystem::Ext4)
//!         // Comma-separated list of mount options.
//!         .mount_options("rw,relatime")
//!         // Interval, in days, between file system backups by the dump command on ext2/3/4
//!         // file systems.
//!         .backup_frequency(0)
//!         // Order in which file systems are checked by the `fsck` command.
//!         .fsck_checking_order(1)
//!         .build()?;
//!
//!     let block_device = BlockDevice::from_str("/dev/usbdisk").map(Source::from)?;
//!     let entry2 = FsTabEntry::builder()
//!         .source(block_device)
//!         .target("/media/usb")
//!         .file_system_type(FileSystem::VFAT)
//!         .mount_options("noauto")
//!         .backup_frequency(0)
//!         .fsck_checking_order(0)
//!         .build()?;
//!
//!     let entry3 = FsTabEntry::builder()
//!         .source(Pseudo::None.into())
//!         .target("/tmp")
//!         .file_system_type(FileSystem::Tmpfs)
//!         .mount_options("nosuid,nodev")
//!         .backup_frequency(0)
//!         .fsck_checking_order(0)
//!         .build()?;
//!
//!     fstab.push(entry1)?;
//!     fstab.push(entry2)?;
//!     fstab.push(entry3)?;
//!
//!     # let temp_file = NamedTempFile::new().unwrap();
//!     # let file_path = temp_file.path();
//!     fstab.write_file(file_path)?;
//!
//!     // Example output
//!     //
//!     // # /etc/fstab
//!     // # Example from scratch
//!     //
//!     // UUID=dd476616-1ce4-415e-9dbd-8c2fa8f42f0f / ext4 rw,relatime 0 1
//!     // /dev/usbdisk /media/usb vfat noauto 0 0
//!     // none /tmp tmpfs nosuid,nodev 0 0
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Print `/proc/self/mountinfo` to the terminal
//!
//! ```
//! use rsmount::tables::MountInfo;
//!
//! fn main() -> rsmount::Result<()> {
//!     let mut mountinfo = MountInfo::new()?;
//!     // Import `/proc/self/mountinfo`.
//!     mountinfo.import_mountinfo()?;
//!
//!     println!("{}", mountinfo);
//!
//!     // Example output
//!     //
//!     // 21 26 0:20 / /sys rw,nosuid,nodev,noexec,relatime - sysfs sysfs rw
//!     // 23 26 0:21 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw
//!     // 25 22 0:23 / /dev/shm rw,nosuid,nodev,noexec,relatime - tmpfs shm rw,inode64
//!
//!     Ok(())
//! }
//! ```
//!

// From dependency library

// From standard library

// From this library
pub use comparison_enum::Comparison;
pub use fs_tab_diff_struct::FsTabDiff;
pub use fs_tab_struct::FsTab;
pub(crate) use gc_item_enum::GcItem;
pub use mount_info_diff_struct::MountInfoDiff;
pub use mount_info_struct::MountInfo;
pub use mount_option_struct::MountOption;
pub use parser_flow_enum::ParserFlow;
pub use swaps_struct::Swaps;
pub use utab_struct::UTab;

mod comparison_enum;
mod fs_tab_diff_struct;
mod fs_tab_struct;
mod gc_item_enum;
mod mount_info_diff_struct;
mod mount_info_struct;
mod mount_option_struct;
mod parser_flow_enum;
mod swaps_struct;
mod utab_struct;
