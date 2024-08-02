// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Table of Contents
//! 1. [Description](#description)
//! 2. [Examples](#examples)
//! 3. [API structure](#api-structure)
//! 4. [From `libmount` to `rsmount`](#from-libmount-to-rsmount)
//!     1. [Higher-level API](#higher-level-api)
//!         1. [Library high-level contaxt](#library-high-level-context)
//!         2. [Mount context](#mount-context)
//!         3. [Umount context](#umount-context)
//!     2. [Files parsing](#files-parsing)
//!         1. [Table of filesystems](#table-of-filesystems)
//!         2. [Filesystem](#filesystem)
//!     3. [Tables management](#tables-management)
//!         1. [Locking](#locking)
//!         2. [Tables update](#tables-update)
//!         3. [Monitor](#monitor)
//!         4. [Compare changes in mount tables](#compare-changes-in-mount-tables)
//!     4. [Mount options](#mount-options)
//!         1. [Options string](#options-string)
//!         2. [Option maps](#option-maps)
//!     5. [Misc](#misc)
//!         1. [Library initialization](#library-initialization)
//!         2. [Cache](#cache)
//!         3. [Iterator](#iterator)
//!         4. [Utils](#utils)
//!         5. [Version functions](#version-functions)
//!
//! ## Description
//!
//! The `rsmount` library is a safe Rust wrapper around `util-linux/libmount`.
//!
//! `rsmount` allows users to, among other things:
//! - mount devices on an operating system's file hierarchy,
//! - list/manage mount points in `/proc/<pid>/mountinfo`,
//! - consult the system's swap usage from `/proc/swaps`,
//! - compose/edit `/etc/fstab`, the file describing all devices an OS
//! should mount at boot.
//! - etc.
//!
//! ## Examples
//!
//! Create an `fstab`.
//! ```
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
//!     // Mount the device with the following UUID as the root file system.
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
//!     // Mount the removable device `/dev/usbdisk` on demand.
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
//!     // Mount a pseudo-filesystem (tmpfs) at `/tmp`
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
//!     println!("{}", fstab);
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
//! ## API structure
//!
//! `rsmount`'s API is roughly divided into three main modules:
//! - [`core`]: a module for items in the library's low-level API.
//! - [`tables`]: a module for manipulating file system descriptions tables (`/etc/fstab`,
//! `/proc/self/mountinfo`, `/proc/swaps`, `/run/mount/utab`).
//! - `mount`: a module to mount devices on the system's file tree.
//!
//! Finally, look to the [`debug`] module if you need to consult debug messages during development.
//!
//! ## From `libmount` to `rsmount` API
//!
//! This section maps `libmount` functions to `rsmount` methods. It follows the same layout as
//! `libmount`’s documentation. You can use it as a reference to ease the transition from one API
//! to the other.
//!
//! ### Higher-level API
//! #### Library high-level context
//!
//! | `libmount`                               | `rsmount` |
//! | ------------------                       | --------- |
//! | [`struct libmnt_context`][1]             |           |
//! | [`struct libmnt_ns`][2]                  |           |
//! | [`MNT_ERR_AMBIFS`][3]                    |           |
//! | [`MNT_ERR_APPLYFLAGS`][4]                |           |
//! | [`MNT_ERR_LOOPDEV`][5]                   |           |
//! | [`MNT_ERR_MOUNTOPT`][6]                  |           |
//! | [`MNT_ERR_NOFSTAB`][7]                   |           |
//! | [`MNT_ERR_NOFSTYPE`][8]                  |           |
//! | [`MNT_ERR_NOSOURCE`][9]                  |           |
//! | [`MNT_ERR_LOOPOVERLAP`][10]              |           |
//! | [`MNT_ERR_LOCK`][11]                     |           |
//! | [`MNT_ERR_NAMESPACE`][12]                |           |
//! | [`MNT_EX_SUCCESS`][13]                   |           |
//! | [`MNT_EX_USAGE`][14]                     |           |
//! | [`MNT_EX_SYSERR`][15]                    |           |
//! | [`MNT_EX_SOFTWARE`][16]                  |           |
//! | [`MNT_EX_USER`][17]                      |           |
//! | [`MNT_EX_FILEIO`][18]                    |           |
//! | [`MNT_EX_FAIL`][19]                      |           |
//! | [`MNT_EX_SOMEOK`][20]                    |           |
//! | [`mnt_free_context`][21]                 |           |
//! | [`mnt_new_context`][22]                  |           |
//! | [`mnt_reset_context`][23]                |           |
//! | [`mnt_context_append_options`][24]       |           |
//! | [`mnt_context_apply_fstab`][25]          |           |
//! | [`mnt_context_disable_canonicalize`][26] |           |
//! | [`mnt_context_disable_helpers`][27]      |           |
//! | [`mnt_context_disable_mtab`][28]         |           |
//! | [`mnt_context_disable_swapmatch`][29]    |           |
//! | [`mnt_context_enable_fake`][30]          |           |
//! | [`mnt_context_enable_force`][31]         |           |
//! | [`mnt_context_enable_fork`][32]          |           |
//! | [`mnt_context_enable_lazy`][33]          |           |
//! | [`mnt_context_enable_loopdel`][34]       |           |
//! | [`mnt_context_enable_noautofs`][35]      |           |
//! | [`mnt_context_enable_onlyonce`][36]      |           |
//! | [`mnt_context_enable_rdonly_umount`][37] |           |
//! | [`mnt_context_enable_rwonly_mount`][38]  |           |
//! | [`mnt_context_enable_sloppy`][39]        |           |
//! | [`mnt_context_enable_verbose`][40]       |           |
//! | [`mnt_context_forced_rdonly`][41]        |           |
//! | [`mnt_context_force_unrestricted`][42]   |           |
//! | [`mnt_context_get_cache`][43]            |           |
//! | [`mnt_context_get_excode`][44]           |           |
//! | [`mnt_context_get_fs`][45]               |           |
//! | [`mnt_context_get_fstab`][46]            |           |
//! | [`mnt_context_get_fstab_userdata`][47]   |           |
//! | [`mnt_context_get_fstype`][48]           |           |
//! | [`mnt_context_get_fs_userdata`][49]      |           |
//! | [`mnt_context_get_helper_status`][50]    |           |
//! | [`mnt_context_get_lock`][51]             |           |
//! | [`mnt_context_get_mflags`][52]           |           |
//! | [`mnt_context_get_mtab`][53]             |           |
//! | [`mnt_context_get_mtab_userdata`][54]    |           |
//! | [`mnt_context_get_options`][55]          |           |
//! | [`mnt_context_get_optsmode`][56]         |           |
//! | [`mnt_context_get_origin_ns`][57]        |           |
//! | [`mnt_context_get_source`][58]           |           |
//! | [`mnt_context_get_status`][59]           |           |
//! | [`mnt_context_get_syscall_errno`][60]    |           |
//! | [`mnt_context_get_table`][61]            |           |
//! | [`mnt_context_get_target`][62]           |           |
//! | [`mnt_context_get_target_ns`][63]        |           |
//! | [`mnt_context_get_target_prefix`][64]    |           |
//! | [`mnt_context_get_user_mflags`][65]      |           |
//! | [`mnt_context_helper_executed`][66]      |           |
//! | [`mnt_context_helper_setopt`][67]        |           |
//! | [`mnt_context_init_helper`][68]          |           |
//! | [`mnt_context_is_child`][69]             |           |
//! | [`mnt_context_is_fake`][70]              |           |
//! | [`mnt_context_is_force`][71]             |           |
//! | [`mnt_context_is_fork`][72]              |           |
//! | [`mnt_context_is_fs_mounted`][73]        |           |
//! | [`mnt_context_is_lazy`][74]              |           |
//! | [`mnt_context_is_loopdel`][75]           |           |
//! | [`mnt_context_is_nocanonicalize`][76]    |           |
//! | [`mnt_context_is_nohelpers`][77]         |           |
//! | [`mnt_context_is_nomtab`][78]            |           |
//! | [`mnt_context_is_onlyonce`][79]          |           |
//! | [`mnt_context_is_parent`][80]            |           |
//! | [`mnt_context_is_rdonly_umount`][81]     |           |
//! | [`mnt_context_is_restricted`][82]        |           |
//! | [`mnt_context_is_rwonly_mount`][83]      |           |
//! | [`mnt_context_is_sloppy`][84]            |           |
//! | [`mnt_context_is_swapmatch`][85]         |           |
//! | [`mnt_context_is_verbose`][86]           |           |
//! | [`mnt_context_reset_status`][87]         |           |
//! | [`mnt_context_set_cache`][88]            |           |
//! | [`mnt_context_set_fs`][89]               |           |
//! | [`mnt_context_set_fstab`][90]            |           |
//! | [`mnt_context_set_fstype`][91]           |           |
//! | [`mnt_context_set_fstype_pattern`][92]   |           |
//! | [`mnt_context_set_mflags`][93]           |           |
//! | [`mnt_context_set_mountdata`][94]        |           |
//! | [`mnt_context_set_options`][95]          |           |
//! | [`mnt_context_set_options_pattern`][96]  |           |
//! | [`mnt_context_set_optsmode`][97]         |           |
//! | [`mnt_context_set_passwd_cb`][98]        |           |
//! | [`mnt_context_set_source`][99]           |           |
//! | [`mnt_context_set_syscall_status`][100]  |           |
//! | [`mnt_context_set_tables_errcb`][101]    |           |
//! | [`mnt_context_set_target`][102]          |           |
//! | [`mnt_context_set_target_ns`][103]       |           |
//! | [`mnt_context_set_target_prefix`][104]   |           |
//! | [`mnt_context_set_user_mflags`][105]     |           |
//! | [`mnt_context_strerror`][106]            |           |
//! | [`mnt_context_switch_ns`][107]           |           |
//! | [`mnt_context_switch_origin_ns`][108]    |           |
//! | [`mnt_context_switch_target_ns`][109]    |           |
//! | [`mnt_context_syscall_called`][110]      |           |
//! | [`mnt_context_tab_applied`][111]         |           |
//! | [`mnt_context_wait_for_children`][112]   |           |
//!
//!
//! [1]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#libmnt-context
//! [2]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#libmnt-ns
//! [3]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-AMBIFS:CAPS
//! [4]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-APPLYFLAGS:CAPS
//! [5]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-LOOPDEV:CAPS
//! [6]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-MOUNTOPT:CAPS
//! [7]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-NOFSTAB:CAPS
//! [8]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-NOFSTYPE:CAPS
//! [9]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-NOSOURCE:CAPS
//! [10]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-LOOPOVERLAP:CAPS
//! [11]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-LOCK:CAPS
//! [12]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-ERR-NAMESPACE:CAPS
//! [13]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-EX-SUCCESS:CAPS
//! [14]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-EX-USAGE:CAPS
//! [15]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-EX-SYSERR:CAPS
//! [16]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-EX-SOFTWARE:CAPS
//! [17]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-EX-USER:CAPS
//! [18]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-EX-FILEIO:CAPS
//! [19]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-EX-FAIL:CAPS
//! [20]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#MNT-EX-SOMEOK:CAPS
//! [21]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-free-context
//! [22]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-new-context
//! [23]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-reset-context
//! [24]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-append-options
//! [25]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-apply-fstab
//! [26]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-disable-canonicalize
//! [27]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-disable-helpers
//! [28]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-disable-mtab
//! [29]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-disable-swapmatch
//! [30]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-fake
//! [31]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-force
//! [32]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-fork
//! [33]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-lazy
//! [34]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-loopdel
//! [35]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-noautofs
//! [36]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-onlyonce
//! [37]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-rdonly-umount
//! [38]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-rwonly-mount
//! [39]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-sloppy
//! [40]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-enable-verbose
//! [41]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-forced-rdonly
//! [42]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-force-unrestricted
//! [43]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-cache
//! [44]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-excode
//! [45]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-fs
//! [46]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-fstab
//! [47]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-fstab-userdata
//! [48]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-fstype
//! [49]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-fs-userdata
//! [50]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-helper-status
//! [51]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-lock
//! [52]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-mflags
//! [53]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-mtab
//! [54]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-mtab-userdata
//! [55]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-options
//! [56]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-optsmode
//! [57]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-origin-ns
//! [58]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-source
//! [59]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-status
//! [60]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-syscall-errno
//! [61]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-table
//! [62]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-target
//! [63]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-target-ns
//! [64]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-target-prefix
//! [65]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-get-user-mflags
//! [66]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-helper-executed
//! [67]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-helper-setopt
//! [68]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-init-helper
//! [69]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-child
//! [70]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-fake
//! [71]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-force
//! [72]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-fork
//! [73]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-fs-mounted
//! [74]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-lazy
//! [75]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-loopdel
//! [76]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-nocanonicalize
//! [77]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-nohelpers
//! [78]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-nomtab
//! [79]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-onlyonce
//! [80]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-parent
//! [81]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-rdonly-umount
//! [82]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-restricted
//! [83]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-rwonly-mount
//! [84]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-sloppy
//! [85]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-swapmatch
//! [86]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-is-verbose
//! [87]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-reset-status
//! [88]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-cache
//! [89]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-fs
//! [90]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-fstab
//! [91]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-fstype
//! [92]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-fstype-pattern
//! [93]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-mflags
//! [94]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-mountdata
//! [95]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-options
//! [96]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-options-pattern
//! [97]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-optsmode
//! [98]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-passwd-cb
//! [99]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-source
//! [100]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-syscall-status
//! [101]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-tables-errcb
//! [102]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-target
//! [103]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-target-ns
//! [104]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-target-prefix
//! [105]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-set-user-mflags
//! [106]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-strerror
//! [107]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-switch-ns
//! [108]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-switch-origin-ns
//! [109]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-switch-target-ns
//! [110]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-syscall-called
//! [111]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-tab-applied
//! [112]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-high-level-context.html#mnt-context-wait-for-children
//!
//! #### Mount context
//!
//! | `libmount`                          | `rsmount`                                                                                                                  |
//! | ------------------                  | ---------                                                                                                                  |
//! | [`MNT_MS_COMMENT`][113]             | [`UserspaceMountFlag::Comment`](crate::core::flags::UserspaceMountFlag::Comment)                                           |
//! | [`MNT_MS_GROUP`][114]               | [`UserspaceMountFlag::Group`](crate::core::flags::UserspaceMountFlag::Group)                                               |
//! | [`MNT_MS_HELPER`][115]              | [`UserspaceMountFlag::MountHelper`](crate::core::flags::UserspaceMountFlag::MountHelper)                                   |
//! | [`MNT_MS_LOOP`][116]                | [`UserspaceMountFlag::LoopDevice`](crate::core::flags::UserspaceMountFlag::LoopDevice)                                     |
//! | [`MNT_MS_NETDEV`][117]              | [`UserspaceMountFlag::DeviceRequiresNetwork`](crate::core::flags::UserspaceMountFlag::DeviceRequiresNetwork)               |
//! | [`MNT_MS_NOAUTO`][118]              | [`UserspaceMountFlag::NoAuto`](crate::core::flags::UserspaceMountFlag::NoAuto)                                             |
//! | [`MNT_MS_NOFAIL`][119]              | [`UserspaceMountFlag::NoFail`](crate::core::flags::UserspaceMountFlag::NoFail)                                             |
//! | [`MNT_MS_OFFSET`][120]              | [`UserspaceMountFlag::LoopDeviceOffset`](crate::core::flags::UserspaceMountFlag::LoopDeviceOffset)                         |
//! | [`MNT_MS_OWNER`][121]               | [`UserspaceMountFlag::Owner`](crate::core::flags::UserspaceMountFlag::Owner)                                               |
//! | [`MNT_MS_SIZELIMIT`][122]           | [`UserspaceMountFlag::LoopDeviceSizeLimit`](crate::core::flags::UserspaceMountFlag::LoopDeviceSizeLimit)                   |
//! | [`MNT_MS_ENCRYPTION`][123]          | [`UserspaceMountFlag::LoopDeviceEncryption`](crate::core::flags::UserspaceMountFlag::LoopDeviceEncryption)                 |
//! | [`MNT_MS_UHELPER`][124]             | [`UserspaceMountFlag::UmountHelper`](crate::core::flags::UserspaceMountFlag::UmountHelper)                                 |
//! | [`MNT_MS_USER`][125]                | [`UserspaceMountFlag::User`](crate::core::flags::UserspaceMountFlag::User)                                                 |
//! | [`MNT_MS_USERS`][126]               | [`UserspaceMountFlag::Users`](crate::core::flags::UserspaceMountFlag::Users)                                               |
//! | [`MNT_MS_XCOMMENT`][127]            | [`UserspaceMountFlag::XUTabComment`](crate::core::flags::UserspaceMountFlag::XUTabComment)                                 |
//! | [`MNT_MS_XFSTABCOMM`][128]          | [`UserspaceMountFlag::XFstabComment`](crate::core::flags::UserspaceMountFlag::XFstabComment)                               |
//! | [`MNT_MS_HASH_DEVICE`][129]         | [`UserspaceMountFlag::HashDevice`](crate::core::flags::UserspaceMountFlag::HashDevice)                                     |
//! | [`MNT_MS_ROOT_HASH`][130]           | [`UserspaceMountFlag::RootHash`](crate::core::flags::UserspaceMountFlag::RootHash)                                         |
//! | [`MNT_MS_HASH_OFFSET`][131]         | [`UserspaceMountFlag::HashOffset`](crate::core::flags::UserspaceMountFlag::HashOffset)                                     |
//! | [`MNT_MS_ROOT_HASH_FILE`][132]      | [`UserspaceMountFlag::RootHashFile`](crate::core::flags::UserspaceMountFlag::RootHashFile)                                 |
//! | [`MNT_MS_FEC_DEVICE`][133]          | [`UserspaceMountFlag::ForwardErrorCorrectionDevice`](crate::core::flags::UserspaceMountFlag::ForwardErrorCorrectionDevice) |
//! | [`MNT_MS_FEC_OFFSET`][134]          | [`UserspaceMountFlag::ForwardErrorCorrectionOffset`](crate::core::flags::UserspaceMountFlag::ForwardErrorCorrectionOffset) |
//! | [`MNT_MS_FEC_ROOTS`][135]           | [`UserspaceMountFlag::ForwardErrorCorrectionRoots`](crate::core::flags::UserspaceMountFlag::ForwardErrorCorrectionRoots)   |
//! | [`MNT_MS_ROOT_HASH_SIG`][136]       | [`UserspaceMountFlag::RootHashSignature`](crate::core::flags::UserspaceMountFlag::RootHashSignature)                       |
//! | [`MS_BIND`][137]                    | [`MountFlag::Bind`](crate::core::flags::MountFlag::Bind)                                                                   |
//! | [`MS_DIRSYNC`][138]                 | [`MountFlag::SynchronizeDirectories`](crate::core::flags::MountFlag::SynchronizeDirectories)                               |
//! | [`MS_I_VERSION`][139]               | [`MountFlag::IVersion`](crate::core::flags::MountFlag::IVersion)                                                           |
//! | [`MS_MANDLOCK`][140]                | [`MountFlag::MandatoryLocking`](crate::core::flags::MountFlag::MandatoryLocking)                                           |
//! | [`MS_MGC_MSK`][141]                 | [`MountFlag::MagicMask`](crate::core::flags::MountFlag::MagicMask)                                                         |
//! | [`MS_MGC_VAL`][142]                 | [`MountFlag::MagicValue`](crate::core::flags::MountFlag::MagicValue)                                                       |
//! | [`MS_MOVE`][143]                    | [`MountFlag::Move`](crate::core::flags::MountFlag::Move)                                                                   |
//! | [`MS_NOATIME`][144]                 | [`MountFlag::NoUpdateAccessTime`](crate::core::flags::MountFlag::NoUpdateAccessTime)                                       |
//! | [`MS_NODEV`][145]                   | [`MountFlag::NoDeviceAccess`](crate::core::flags::MountFlag::NoDeviceAccess)                                               |
//! | [`MS_NODIRATIME`][146]              | [`MountFlag::NoUpdateDirectoryAccessTime`](crate::core::flags::MountFlag::NoUpdateDirectoryAccessTime)                     |
//! | [`MS_NOEXEC`][147]                  | [`MountFlag::NoExecute`](crate::core::flags::MountFlag::NoExecute)                                                         |
//! | [`MS_NOSUID`][148]                  | [`MountFlag::NoSuid`](crate::core::flags::MountFlag::NoSuid)                                                               |
//! | [`MS_OWNERSECURE`][149]             | [`MountFlag::OwnerSecure`](crate::core::flags::MountFlag::OwnerSecure)                                                     |
//! | [`MS_PRIVATE`][150]                 | [`MountFlag::Private`](crate::core::flags::MountFlag::Private)                                                             |
//! | [`MS_PROPAGATION`][151]             | [`MountFlag::Propagation`](crate::core::flags::MountFlag::Propagation)                                                     |
//! | [`MS_RDONLY`][152]                  | [`MountFlag::ReadOnly`](crate::core::flags::MountFlag::ReadOnly)                                                           |
//! | [`MS_REC`][153]                     | [`MountFlag::Recursive`](crate::core::flags::MountFlag::Recursive)                                                         |
//! | [`MS_RELATIME`][154]                | [`MountFlag::RelativeAcessTime`](crate::core::flags::MountFlag::RelativeAcessTime)                                         |
//! | [`MS_REMOUNT`][155]                 | [`MountFlag::Remount`](crate::core::flags::MountFlag::Remount)                                                             |
//! | [`MS_SECURE`][156]                  | [`MountFlag::Secure`](crate::core::flags::MountFlag::Secure)                                                               |
//! | [`MS_SHARED`][157]                  | [`MountFlag::Shared`](crate::core::flags::MountFlag::Shared)                                                               |
//! | [`MS_SILENT`][158]                  | [`MountFlag::Silent`](crate::core::flags::MountFlag::Silent)                                                               |
//! | [`MS_SLAVE`][159]                   | [`MountFlag::Slave`](crate::core::flags::MountFlag::Slave)                                                                 |
//! | [`MS_STRICTATIME`][160]             | [`MountFlag::StrictUpdateAccessTime`](crate::core::flags::MountFlag::StrictUpdateAccessTime)                               |
//! | [`MS_SYNCHRONOUS`][161]             | [`MountFlag::Synchronous`](crate::core::flags::MountFlag::Synchronous)                                                     |
//! | [`MS_UNBINDABLE`][162]              | [`MountFlag::Unbindable`](crate::core::flags::MountFlag::Unbindable)                                                       |
//! | [`MS_LAZYTIME`][163]                | [`MountFlag::LazyTime`](crate::core::flags::MountFlag::LazyTime)                                                           |
//! | [`mnt_context_do_mount`][164]       |                                                                                                                            |
//! | [`mnt_context_finalize_mount`][165] |                                                                                                                            |
//! | [`mnt_context_mount`][166]          |                                                                                                                            |
//! | [`mnt_context_next_mount`][167]     |                                                                                                                            |
//! | [`mnt_context_next_remount`][168]   |                                                                                                                            |
//! | [`mnt_context_prepare_mount`][169]  |                                                                                                                            |
//!
//! [113]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-COMMENT:CAPS
//! [114]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-GROUP:CAPS
//! [115]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-HELPER:CAPS
//! [116]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-LOOP:CAPS
//! [117]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-NETDEV:CAPS
//! [118]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-NOAUTO:CAPS
//! [119]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-NOFAIL:CAPS
//! [120]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-OFFSET:CAPS
//! [121]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-OWNER:CAPS
//! [122]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-SIZELIMIT:CAPS
//! [123]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-ENCRYPTION:CAPS
//! [124]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-UHELPER:CAPS
//! [125]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-USER:CAPS
//! [126]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-USERS:CAPS
//! [127]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-XCOMMENT:CAPS
//! [128]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-XFSTABCOMM:CAPS
//! [129]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-HASH-DEVICE:CAPS
//! [130]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-ROOT-HASH:CAPS
//! [131]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-HASH-OFFSET:CAPS
//! [132]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-ROOT-HASH-FILE:CAPS
//! [133]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-FEC-DEVICE:CAPS
//! [134]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-FEC-OFFSET:CAPS
//! [135]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-FEC-ROOTS:CAPS
//! [136]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MNT-MS-ROOT-HASH-SIG:CAPS
//! [137]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-BIND:CAPS
//! [138]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-DIRSYNC:CAPS
//! [139]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-I-VERSION:CAPS
//! [140]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-MANDLOCK:CAPS
//! [141]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-MGC-MSK:CAPS
//! [142]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-MGC-VAL:CAPS
//! [143]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-MOVE:CAPS
//! [144]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-NOATIME:CAPS
//! [145]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-NODEV:CAPS
//! [146]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-NODIRATIME:CAPS
//! [147]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-NOEXEC:CAPS
//! [148]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-NOSUID:CAPS
//! [149]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-OWNERSECURE:CAPS
//! [150]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-PRIVATE:CAPS
//! [151]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-PROPAGATION:CAPS
//! [152]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-RDONLY:CAPS
//! [153]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-REC:CAPS
//! [154]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-RELATIME:CAPS
//! [155]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-REMOUNT:CAPS
//! [156]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-SECURE:CAPS
//! [157]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-SHARED:CAPS
//! [158]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-SILENT:CAPS
//! [159]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-SLAVE:CAPS
//! [160]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-STRICTATIME:CAPS
//! [161]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-SYNCHRONOUS:CAPS
//! [162]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-UNBINDABLE:CAPS
//! [163]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#MS-LAZYTIME:CAPS
//! [164]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#mnt-context-do-mount
//! [165]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#mnt-context-finalize-mount
//! [166]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#mnt-context-mount
//! [167]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#mnt-context-next-mount
//! [168]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#mnt-context-next-remount
//! [169]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Mount-context.html#mnt-context-prepare-mount
//!
//! #### Umount context
//!
//! | `libmount`                           | `rsmount` |
//! | ------------------                   | --------- |
//! | [`mnt_context_find_umount_fs`][170]  |           |
//! | [`mnt_context_do_umount`][171]       |           |
//! | [`mnt_context_finalize_umount`][172] |           |
//! | [`mnt_context_next_umount`][173]     |           |
//! | [`mnt_context_prepare_umount`][174]  |           |
//! | [`mnt_context_umount`][175]          |           |
//!
//! [170]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Umount-context.html#mnt-context-find-umount-fs
//! [171]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Umount-context.html#mnt-context-do-umount
//! [172]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Umount-context.html#mnt-context-finalize-umount
//! [173]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Umount-context.html#mnt-context-next-umount
//! [174]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Umount-context.html#mnt-context-prepare-umount
//! [175]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Umount-context.html#mnt-context-umount
//!
//! ### Files parsing
//! #### Table of filesystems
//!
//! | `libmount`                                 | `rsmount`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
//! | ------------------                         | ---------                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
//! | [`struct libmnt_table`][176]               | [`FsTab`](crate::tables::FsTab) <br> [`MountInfo`](crate::tables::MountInfo) <br> [`Swaps`](crate::tables::Swaps) <br> [`UTab`](crate::tables::UTab)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_free_table`][177]                    | [`FsTab`](crate::tables::FsTab), [`MountInfo`](crate::tables::MountInfo), [`Swaps`](crate::tables::Swaps), [`UTab`](crate::tables::UTab) are automatically deallocated when they go out of scope.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
//! | [`mnt_new_table`][178]                     | [`FsTab::new`](crate::tables::FsTab::new) <br> [`MountInfo::new`](crate::tables::MountInfo::new) <br> [`Swaps::new`](crate::tables::Swaps::new) <br> [`UTab::new`](crate::tables::UTab::new)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_reset_table`][179]                   | [`FsTab::clear`](crate::tables::FsTab::clear) <br> [`UTab::clear`](crate::tables::UTab::clear)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
//! | [`mnt_ref_table`][180]                     | Managed automatically.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_unref_table`][181]                   | Managed automatically.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_new_table_from_dir`][182]            | [`FsTab::new_from_directory`](crate::tables::FsTab::new_from_directory)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
//! | [`mnt_new_table_from_file`][183]           | [`FsTab::new_from_file`](crate::tables::FsTab::new_from_file)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
//! | [`mnt_table_add_fs`][184]                  | [`FsTab::push`](crate::tables::FsTab::push) <br> [`UTab::push`](crate::tables::UTab::push)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
//! | [`mnt_table_append_intro_comment`][185]    | [`FsTab::append_to_intro_comments`](crate::tables::FsTab::append_to_intro_comments)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
//! | [`mnt_table_append_trailing_comment`][186] | [`FsTab::append_to_trailing_comments`](crate::tables::FsTab::append_to_trailing_comments)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
//! | [`mnt_table_enable_comments`][187]         | [`FsTab::import_with_comments`](crate::tables::FsTab::import_with_comments) <br> [`FsTab::import_without_comments`](crate::tables::FsTab::import_without_comments) <br> [`FsTab::export_with_comments`](crate::tables::FsTab::export_with_comments) <br> [`FsTab::export_without_comments`](crate::tables::FsTab::export_without_comments)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
//! | [`mnt_table_find_devno`][188]              | [`MountInfo::find_device`](crate::tables::MountInfo::find_device) <br> [`MountInfo::find_back_device`](crate::tables::MountInfo::find_back_device)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_table_find_fs`][189]                 | [`FsTab::contains`](crate::tables::FsTab::contains) <br> [`FsTab::position`](crate::tables::FsTab::position) <br> [`MountInfo::position`](crate::tables::MountInfo::position) <br> [`Swaps::position`](crate::tables::Swaps::position) <br> [`UTab::contains`](crate::tables::UTab::contains) <br> [`UTab::position`](crate::tables::UTab::position)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_table_find_mountpoint`][190]         | [`MountInfo::find_mount_point`](crate::tables::MountInfo::find_mount_point) <br> [`MountInfo::find_back_mount_point`](crate::tables::MountInfo::find_back_mount_point)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_table_find_next_fs`][191]            | [`FsTab::find_first`](crate::tables::FsTab::find_first) <br> [`FsTab::find_back_first`](crate::tables::FsTab::find_back_first) <br> [`MountInfo::find_first`](crate::tables::MountInfo::find_first) <br> [`MountInfo::find_back_first`](crate::tables::MountInfo::find_back_first) <br> [`Swaps::find_first`](crate::tables::Swaps::find_first) <br> [`Swaps::find_back_first`](crate::tables::Swaps::find_back_first) <br> [`UTab::find_first`](crate::tables::UTab::find_first) <br> [`UTab::find_back_first`](crate::tables::UTab::find_back_first)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_table_find_pair`][192]               | [`FsTab::find_pair`](crate::tables::FsTab::find_pair) <br> [`FsTab::find_back_pair`](crate::tables::FsTab::find_back_pair) <br> [`MountInfo::find_pair`](crate::tables::MountInfo::find_pair) <br> [`MountInfo::find_back_pair`](crate::tables::MountInfo::find_back_pair) <br> [`UTab::find_pair`](crate::tables::UTab::find_pair) <br> [`UTab::find_back_pair`](crate::tables::UTab::find_back_pair)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_table_find_source`][193]             | [`FsTab::find_source`](crate::tables::FsTab::find_source) <br> [`FsTab::find_back_source`](crate::tables::FsTab::find_back_source) <br> [`MountInfo::find_source`](crate::tables::MountInfo::find_source) <br> [`MountInfo::find_back_source`](crate::tables::MountInfo::find_back_source) <br> [`Swaps::find_source`](crate::tables::Swaps::find_source) <br> [`Swaps::find_back_source`](crate::tables::Swaps::find_back_source) <br> [`UTab::find_source`](crate::tables::UTab::find_source) <br> [`UTab::find_back_source`](crate::tables::UTab::find_back_source)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_table_find_srcpath`][194]            | [`FsTab::find_source_path`](crate::tables::FsTab::find_source_path) <br> [`FsTab::find_back_source_path`](crate::tables::FsTab::find_back_source_path) <br> [`MountInfo::find_source_path`](crate::tables::MountInfo::find_source_path) <br> [`MountInfo::find_back_source_path`](crate::tables::MountInfo::find_back_source_path) <br> [`Swaps::find_source_path`](crate::tables::Swaps::find_source_path) <br> [`Swaps::find_back_source_path`](crate::tables::Swaps::find_back_source_path) <br> [`UTab::find_source_path`](crate::tables::UTab::find_source_path) <br> [`UTab::find_back_source_path`](crate::tables::UTab::find_back_source_path)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_table_find_tag`][195]                | [`FsTab::find_source_tag`](crate::tables::FsTab::find_source_tag) <br> [`FsTab::find_back_source_tag`](crate::tables::FsTab::find_back_source_tag)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_table_find_target`][196]             | [`FsTab::find_target`](crate::tables::FsTab::find_target) <br> [`FsTab::find_back_target`](crate::tables::FsTab::find_back_target) <br> [`MountInfo::find_target`](crate::tables::MountInfo::find_target) <br> [`MountInfo::find_back_target`](crate::tables::MountInfo::find_back_target) <br> [`UTab::find_target`](crate::tables::UTab::find_target) <br> [`UTab::find_back_target`](crate::tables::UTab::find_back_target)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
//! | [`mnt_table_find_target_with_option`][197] | [`FsTab::find_target_with_option`](crate::tables::FsTab::find_target_with_option) <br> [`FsTab::find_back_target_with_option`](crate::tables::FsTab::find_back_target_with_option) <br> [`FsTab::find_target_with_exact_option`](crate::tables::FsTab::find_target_with_exact_option) <br> [`FsTab::find_back_target_with_exact_option`](crate::tables::FsTab::find_back_target_with_exact_option) <br> [`MountInfo::find_target_with_option`](crate::tables::MountInfo::find_target_with_option) <br> [`MountInfo::find_back_target_with_option`](crate::tables::MountInfo::find_back_target_with_option) <br> [`MountInfo::find_target_with_exact_option`](crate::tables::MountInfo::find_target_with_exact_option) <br> [`MountInfo::find_back_target_with_exact_option`](crate::tables::MountInfo::find_back_target_with_exact_option) <br> [`UTab::find_target_with_option`](crate::tables::UTab::find_target_with_option) <br> [`UTab::find_back_target_with_option`](crate::tables::UTab::find_back_target_with_option) <br> [`UTab::find_target_with_exact_option`](crate::tables::UTab::find_target_with_exact_option) <br> [`UTab::find_back_target_with_exact_option`](crate::tables::UTab::find_back_target_with_exact_option) |
//! | [`mnt_table_first_fs`][198]                | [`FsTab::first`](crate::tables::FsTab::first) <br> [`MountInfo::first`](crate::tables::MountInfo::first) <br> [`Swaps::first`](crate::tables::Swaps::first)<br> [`UTab::first`](crate::tables::UTab::first)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
//! | [`mnt_table_get_cache`][199]               | [`FsTab::cache`](crate::tables::FsTab::cache) <br> [`MountInfo::cache`](crate::tables::MountInfo::cache) <br> [`Swaps::cache`](crate::tables::Swaps::cache) <br> [`UTab::cache`](crate::tables::UTab::cache)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_table_get_intro_comment`][200]       | [`FsTab::intro_comments`](crate::tables::FsTab::intro_comments)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
//! | [`mnt_table_get_nents`][201]               | [`FsTab::len`](crate::tables::FsTab::len) <br> [`MountInfo::len`](crate::tables::MountInfo::len) <br> [`Swaps::len`](crate::tables::Swaps::len) <br> [`UTab::len`](crate::tables::UTab::len)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_table_get_root_fs`][202]             | [`MountInfo::root`](crate::tables::MountInfo::root)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
//! | [`mnt_table_get_trailing_comment`][203]    | [`FsTab::trailing_comments`](crate::tables::FsTab::trailing_comments)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
//! | [`mnt_table_get_userdata`][204]            | Managed internally.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
//! | [`mnt_table_insert_fs`][205]               | [`FsTab::push_front`](crate::tables::FsTab::push_front) <br> [`UTab::push_front`](crate::tables::UTab::push_front)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_table_is_empty`][206]                | [`FsTab::is_empty`](crate::tables::FsTab::is_empty) <br> [`MountInfo::is_empty`](crate::tables::MountInfo::is_empty) <br> [`Swaps::is_empty`](crate::tables::Swaps::is_empty) <br> [`UTab::is_empty`](crate::tables::UTab::is_empty)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_table_is_fs_mounted`][207]           | [`MountInfo::is_mounted`](crate::tables::MountInfo::is_mounted)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
//! | [`mnt_table_last_fs`][208]                 | [`FsTab::last`](crate::tables::FsTab::last) <br> [`MountInfo::last`](crate::tables::MountInfo::last) <br> [`Swaps::last`](crate::tables::Swaps::last) <br> [`UTab::last`](crate::tables::UTab::last)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_table_move_fs`][209]                 |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
//! | [`mnt_table_next_child_fs`][210]           |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
//! | [`mnt_table_next_fs`][211]                 | [`FsTab::iter`](crate::tables::FsTab::iter) <br> [`MountInfo::iter`](crate::tables::MountInfo::iter) <br> [`Swaps::iter`](crate::tables::Swaps::iter) <br> [`UTab::iter`](crate::tables::UTab::iter)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_table_over_fs`][212]                 |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
//! | [`mnt_table_parse_dir`][213]               | [`FsTab::import_directory`](crate::tables::FsTab::import_directory)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
//! | [`mnt_table_parse_file`][214]              | [`FsTab::import_file`](crate::tables::FsTab::import_file)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
//! | [`mnt_table_parse_fstab`][215]             | [`FsTab::import_etc_fstab`](crate::tables::FsTab::import_etc_fstab)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
//! | [`mnt_table_parse_mtab`][216]              | [`MountInfo::import_mountinfo`](crate::tables::MountInfo::import_mountinfo)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
//! | [`mnt_table_parse_stream`][217]            | [`FsTab::import_from_stream`](crate::tables::FsTab::import_from_stream)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
//! | [`mnt_table_parse_swaps`][218]             | [`Swaps::import_proc_swaps`](crate::tables::Swaps::import_proc_swaps)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
//! | [`mnt_table_remove_fs`][219]               |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
//! | [`mnt_table_set_cache`][220]               | [`FsTab::set_cache`](crate::tables::FsTab::set_cache) <br> [`MountInfo::set_cache`](crate::tables::MountInfo::set_cache) <br> [`Swaps::set_cache`](crate::tables::Swaps::set_cache) <br> [`UTab::set_cache`](crate::tables::UTab::set_cache)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_table_set_intro_comment`][221]       | [`FsTab::set_intro_comments`](crate::tables::FsTab::set_intro_comments)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
//! | [`mnt_table_set_iter`][222]                | [`FsTabIter::advance_to`](crate::core::iter::FsTabIter::advance_to) <br> [`MountInfoIter::advance_to`](crate::core::iter::MountInfoIter::advance_to) <br> [`SwapsIter::advance_to`](crate::core::iter::SwapsIter::advance_to) <br> [`UTabIter::advance_to`](crate::core::iter::UTabIter::advance_to)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_table_set_parser_errcb`][223]        | [`FsTab::set_parser_error_handler`](crate::tables::FsTab::set_parser_error_handler) <br> [`MountInfo::set_parser_error_handler`](crate::tables::MountInfo::set_parser_error_handler) <br> [`Swaps::set_parser_error_handler`](crate::tables::Swaps::set_parser_error_handler) <br> [`UTab::set_parser_error_handler`](crate::tables::UTab::set_parser_error_handler)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_table_set_trailing_comment`][224]    | [`FsTab::set_trailing_comments`](crate::tables::FsTab::set_trailing_comments)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
//! | [`mnt_table_set_userdata`][225]            | Managed internally.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
//! | [`mnt_table_uniq_fs`][226]                 | [`FsTab::dedup_first_by`](crate::tables::FsTab::dedup_first_by) <br> [`FsTab::dedup_last_by`](crate::tables::FsTab::dedup_last_by) <br> [`MountInfo::dedup_first_by`](crate::tables::FsTab::dedup_first_by) <br> [`MountInfo::dedup_last_by`](crate::tables::MountInfo::dedup_last_by) <br> [`MountInfo::distinct_first_by`](crate::tables::MountInfo::distinct_first_by) <br> [`MountInfo::distinct_last_by`](crate::tables::MountInfo::distinct_last_by) <br> [`Swaps::dedup_first_by`](crate::tables::Swaps::dedup_first_by) <br> [`Swaps::dedup_last_by`](crate::tables::Swaps::dedup_last_by) <br> [`UTab::dedup_first_by`](crate::tables::UTab::dedup_first_by) <br> [`UTab::dedup_last_by`](crate::tables::UTab::dedup_last_by)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_table_with_comments`][227]           | [`FsTab::is_importing_comments`](crate::tables::FsTab::is_importing_comments) <br>  [`FsTab::is_exporting_comments`](crate::tables::FsTab::is_exporting_comments)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
//!
//! [176]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#libmnt-table
//! [177]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-free-table
//! [178]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-new-table
//! [179]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-reset-table
//! [180]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-ref-table
//! [181]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-unref-table
//! [182]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-new-table-from-dir
//! [183]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-new-table-from-file
//! [184]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-add-fs
//! [185]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-append-intro-comment
//! [186]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-append-trailing-comment
//! [187]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-enable-comments
//! [188]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-devno
//! [189]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-fs
//! [190]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-mountpoint
//! [191]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-next-fs
//! [192]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-pair
//! [193]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-source
//! [194]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-srcpath
//! [195]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-tag
//! [196]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-target
//! [197]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-find-target-with-option
//! [198]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-first-fs
//! [199]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-get-cache
//! [200]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-get-intro-comment
//! [201]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-get-nents
//! [202]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-get-root-fs
//! [203]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-get-trailing-comment
//! [204]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-get-userdata
//! [205]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-insert-fs
//! [206]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-is-empty
//! [207]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-is-fs-mounted
//! [208]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-last-fs
//! [209]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-move-fs
//! [210]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-next-child-fs
//! [211]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-next-fs
//! [212]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-over-fs
//! [213]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-parse-dir
//! [214]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-parse-file
//! [215]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-parse-fstab
//! [216]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-parse-mtab
//! [217]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-parse-stream
//! [218]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-parse-swaps
//! [219]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-remove-fs
//! [220]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-set-cache
//! [221]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-set-intro-comment
//! [222]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-set-iter
//! [223]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-set-parser-errcb
//! [224]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-set-trailing-comment
//! [225]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-set-userdata
//! [226]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-uniq-fs
//! [227]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Table-of-filesystems.html#mnt-table-with-comments
//!
//! #### Filesystem
//!
//! | `libmount`                          | `rsmount`                                                                                                                                                                                                                                                                                                                                                                                                                  |
//! | ------------------                  | ---------                                                                                                                                                                                                                                                                                                                                                                                                                  |
//! | [`struct libmnt_fs`][228]           | [`FsTabEntry`](crate::core::entries::FsTabEntry) <br> [`MountInfoEntry`](crate::core::entries::MountInfoEntry) <br> [`SwapsEntry`](crate::core::entries::SwapsEntry) <br> [`UTabEntry`](crate::core::entries::UTabEntry)                                                                                                                                                                                                   |
//! | [`mnt_copy_fs`][229]                | [`FsTabEntry::complete`](crate::core::entries::FsTabEntry::complete) <br> [`FsTabEntry::copy`](crate::core::entries::FsTabEntry::copy) <br> [`MountInfoEntry::copy`](crate::core::entries::MountInfoEntry::copy) <br> [`SwapsEntry::copy`](crate::core::entries::SwapsEntry::copy) <br> [`UTabEntry::complete`](crate::core::entries::UTabEntry::complete) <br> [`UTabEntry::copy`](crate::core::entries::UTabEntry::copy) |
//! | [`mnt_free_fs`][230]                | [`FsTabEntry`](crate::core::entries::FsTabEntry), [`MountInfoEntry`](crate::core::entries::MountInfoEntry), [`SwapsEntry`](crate::core::entries::SwapsEntry), [`UTabEntry`](crate::core::entries::UTabEntry) are automatically deallocated when they go out of scope.                                                                                                                                                      |
//! | [`mnt_free_mntent`][231]            | [`MntEnt`](crate::core::entries::MntEnt) is automatically deallocated when it goes out of scope.                                                                                                                                                                                                                                                                                                                           |
//! | [`mnt_ref_fs`][232]                 | Managed automatically.                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_unref_fs`][233]               | Managed automatically.                                                                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_fs_append_attributes`][234]   | [`UTabEntry::append_attributes`](crate::core::entries::UTabEntry::append_attributes)                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_fs_append_comment`][235]      | [`FsTabEntry::append_comment`](crate::core::entries::FsTabEntry::append_comment)                                                                                                                                                                                                                                                                                                                                           |
//! | [`mnt_fs_append_options`][236]      | [`FsTabEntry::append_options`](crate::core::entries::FsTabEntry::append_options)                                                                                                                                                                                                                                                                                                                                           |
//! | [`mnt_fs_get_attribute`][237]       | [`UTabEntry::attribute_value`](crate::core::entries::UTabEntry::attribute_value)                                                                                                                                                                                                                                                                                                                                           |
//! | [`mnt_fs_get_attributes`][238]      | [`UTabEntry::attributes`](crate::core::entries::UTabEntry::attributes)                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_fs_get_bindsrc`][239]         | [`UTabEntry::bind_source`](crate::core::entries::UTabEntry::bind_source)                                                                                                                                                                                                                                                                                                                                                   |
//! | [`mnt_fs_get_comment`][240]         | [`FsTabEntry::comment`](crate::core::entries::FsTabEntry::comment)                                                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_fs_get_devno`][241]           | [`MountInfoEntry::device_id`](crate::core::entries::MountInfoEntry::device_id)                                                                                                                                                                                                                                                                                                                                             |
//! | [`mnt_fs_get_freq`][242]            | [`FsTabEntry::backup_frequency`](crate::core::entries::FsTabEntry::backup_frequency)                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_fs_get_fs_options`][243]      | [`MountInfoEntry::fs_specific_options`](crate::core::entries::MountInfoEntry::fs_specific_options)                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_fs_get_fstype`][244]          | [`FsTabEntry::file_system_type`](crate::core::entries::FsTabEntry::file_system_type) <br> [`MountInfoEntry::file_system_type`](crate::core::entries::MountInfoEntry::file_system_type)                                                                                                                                                                                                                                     |
//! | [`mnt_fs_get_id`][245]              | [`MountInfoEntry::mount_id`](crate::core::entries::MountInfoEntry::mount_id) <br> [`UTabEntry::mount_id`](crate::core::entries::UTabEntry::mount_id)                                                                                                                                                                                                                                                                       |
//! | [`mnt_fs_get_option`][246]          | [`FsTabEntry::option_value`](crate::core::entries::FsTabEntry::option_value)                                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_fs_get_optional_fields`][247] | [`MountInfoEntry::optional_fields`](crate::core::entries::MountInfoEntry::optional_fields)                                                                                                                                                                                                                                                                                                                                 |
//! | [`mnt_fs_get_options`][248]         | [`FsTabEntry::mount_options`](crate::core::entries::FsTabEntry::mount_options)                                                                                                                                                                                                                                                                                                                                             |
//! | [`mnt_fs_get_parent_id`][249]       | [`MountInfoEntry::parent_id`](crate::core::entries::MountInfoEntry::parent_id)                                                                                                                                                                                                                                                                                                                                             |
//! | [`mnt_fs_get_passno`][250]          | [`FsTabEntry::fsck_checking_order`](crate::core::entries::FsTabEntry::fsck_checking_order)                                                                                                                                                                                                                                                                                                                                 |
//! | [`mnt_fs_get_priority`][251]        | [`SwapsEntry::priority`](crate::core::entries::SwapsEntry::priority)                                                                                                                                                                                                                                                                                                                                                       |
//! | [`mnt_fs_get_propagation`][252]     | [`MountInfoEntry::propagation_flags`](crate::core::entries::MountInfoEntry::propagation_flags)                                                                                                                                                                                                                                                                                                                             |
//! | [`mnt_fs_get_root`][253]            | [`MountInfoEntry::root`](crate::core::entries::MountInfoEntry::root) <br> [`UTabEntry::root`](crate::core::entries::UTabEntry::root)                                                                                                                                                                                                                                                                                       |
//! | [`mnt_fs_get_size`][254]            | [`SwapsEntry::size`](crate::core::entries::SwapsEntry::size)                                                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_fs_get_source`][255]          | [`FsTabEntry::source`](crate::core::entries::FsTabEntry::source)                                                                                                                                                                                                                                                                                                                                                           |
//! | [`mnt_fs_get_srcpath`][256]         | [`FsTabEntry::source_path`](crate::core::entries::FsTabEntry::source_path) <br> [`MountInfoEntry::source_path`](crate::core::entries::MountInfoEntry::source_path) <br> [`SwapsEntry::source_path`](crate::core::entries::SwapsEntry::source_path) <br> [`UTabEntry::source_path`](crate::core::entries::UTabEntry::source_path)                                                                                           |
//! | [`mnt_fs_get_swaptype`][257]        | [`SwapsEntry::swap_type`](crate::core::entries::SwapsEntry::swap_type)                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_fs_get_tag`][258]             | [`FsTabEntry::tag`](crate::core::entries::FsTabEntry::tag)                                                                                                                                                                                                                                                                                                                                                                 |
//! | [`mnt_fs_get_table`][259]           | Not implemented.                                                                                                                                                                                                                                                                                                                                                                                                                |
//! | [`mnt_fs_get_target`][260]          | [`FsTabEntry::target`](crate::core::entries::FsTabEntry::target) <br> [`MountInfoEntry::target`](crate::core::entries::MountInfoEntry::target)                                                                                                                                                                                                                                                                             |
//! | [`mnt_fs_get_tid`][261]             | [`MountInfoEntry::pid`](crate::core::entries::MountInfoEntry::pid)                                                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_fs_get_usedsize`][262]        | [`SwapsEntry::size_used`](crate::core::entries::SwapsEntry::size_used)                                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_fs_get_userdata`][263]        | Managed internally.                                                                                                                                                                                                                                                                                                                                                                                                        |
//! | [`mnt_fs_get_user_options`][264]    | [`UTabEntry::mount_options`](crate::core::entries::UTabEntry::mount_options)                                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_fs_get_vfs_options`][265]     | [`MountInfoEntry::fs_independent_options`](crate::core::entries::MountInfoEntry::fs_independent_options)                                                                                                                                                                                                                                                                                                                   |
//! | [`mnt_fs_get_vfs_options_all`][266] | [`MountInfoEntry::fs_independent_options_full`](crate::core::entries::MountInfoEntry::fs_independent_options_full)                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_fs_is_kernel`][267]           | [`FsTabEntry::is_from_kernel`](crate::core::entries::FsTabEntry::is_from_kernel) <br> [`MountInfoEntry::is_from_kernel`](crate::core::entries::MountInfoEntry::is_from_kernel) <br> [`SwapsEntry::is_from_kernel`](crate::core::entries::SwapsEntry::is_from_kernel) <br> [`UTabEntry::is_from_kernel`](crate::core::entries::UTabEntry::is_from_kernel)                                                                   |
//! | [`mnt_fs_is_netfs`][268]            | [`FsTabEntry::is_net_fs`](crate::core::entries::FsTabEntry::is_net_fs) <br> [`MountInfoEntry::is_net_fs`](crate::core::entries::MountInfoEntry::is_net_fs) <br> [`SwapsEntry::is_net_fs`](crate::core::entries::SwapsEntry::is_net_fs) <br> [`UTabEntry::is_net_fs`](crate::core::entries::UTabEntry::is_net_fs)                                                                                                           |
//! | [`mnt_fs_is_pseudofs`][269]         | [`FsTabEntry::is_pseudo_fs`](crate::core::entries::FsTabEntry::is_pseudo_fs) <br> [`MountInfoEntry::is_pseudo_fs`](crate::core::entries::MountInfoEntry::is_pseudo_fs) <br> [`SwapsEntry::is_pseudo_fs`](crate::core::entries::SwapsEntry::is_pseudo_fs) <br> [`UTabEntry::is_pseudo_fs`](crate::core::entries::UTabEntry::is_pseudo_fs)                                                                                   |
//! | [`mnt_fs_is_regularfs`][270]        | [`FsTabEntry::is_regular_fs`](crate::core::entries::FsTabEntry::is_regular_fs) <br> [`MountInfoEntry::is_regular_fs`](crate::core::entries::MountInfoEntry::is_regular_fs) <br> [`SwapsEntry::is_regular_fs`](crate::core::entries::SwapsEntry::is_regular_fs) <br> [`UTabEntry::is_regular_fs`](crate::core::entries::UTabEntry::is_regular_fs)                                                                           |
//! | [`mnt_fs_is_swaparea`][271]         | [`FsTabEntry::is_swap`](crate::core::entries::FsTabEntry::is_swap) <br> [`MountInfoEntry::is_swap`](crate::core::entries::MountInfoEntry::is_swap) <br> [`SwapsEntry::is_swap`](crate::core::entries::SwapsEntry::is_swap) <br> [`UTabEntry::is_swap`](crate::core::entries::UTabEntry::is_swap)                                                                                                                           |
//! | [`mnt_fs_match_fstype`][272]        | [`FsTabEntry::has_any_fs_type`](crate::core::entries::FsTabEntry::has_any_fs_type) <br> [`MountInfoEntry::has_any_fs_type`](crate::core::entries::MountInfoEntry::has_any_fs_type)                                                                                                                                                                                                                                         |
//! | [`mnt_fs_match_options`][273]       | [`FsTabEntry::has_any_option`](crate::core::entries::FsTabEntry::has_any_option) <br>  [`UTabEntry::has_any_option`](crate::core::entries::UTabEntry::has_any_option)                                                                                                                                                                                                                                                      |
//! | [`mnt_fs_match_source`][274]        | [`FsTabEntry::is_source`](crate::core::entries::FsTabEntry::is_source) <br> [`MountInfoEntry::is_source`](crate::core::entries::MountInfoEntry::is_source) <br> [`SwapsEntry::is_source`](crate::core::entries::SwapsEntry::is_source) <br> [`UTabEntry::is_source`](crate::core::entries::UTabEntry::is_source)                                                                                                           |
//! | [`mnt_fs_match_target`][275]        | [`FsTabEntry::is_target`](crate::core::entries::FsTabEntry::is_target) <br> [`MountInfoEntry::is_target`](crate::core::entries::MountInfoEntry::is_target) <br> [`UTabEntry::is_target`](crate::core::entries::UTabEntry::is_target)                                                                                                                                                                                       |
//! | [`mnt_fs_prepend_attributes`][276]  | [`UTabEntry::prepend_attributes`](crate::core::entries::UTabEntry::prepend_attributes)                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_fs_prepend_options`][277]     | [`FsTabEntry::prepend_options`](crate::core::entries::FsTabEntry::prepend_options)                                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_fs_print_debug`][278]         | [`FsTabEntry::print_debug_to`](crate::core::entries::FsTabEntry::print_debug_to) <br> [`MountInfoEntry::print_debug_to`](crate::core::entries::MountInfoEntry::print_debug_to)                                                                                                                                                                                                                                             |
//! | [`mnt_fs_set_attributes`][279]      | [`UTabEntry::set_attributes`](crate::core::entries::UTabEntry::set_attributes)                                                                                                                                                                                                                                                                                                                                             |
//! | [`mnt_fs_set_bindsrc`][280]         | [`UTabEntry::set_bind_source`](crate::core::entries::UTabEntry::set_bind_source)                                                                                                                                                                                                                                                                                                                                           |
//! | [`mnt_fs_set_comment`][281]         | [`FsTabEntry::set_comment`](crate::core::entries::FsTabEntry::set_comment)                                                                                                                                                                                                                                                                                                                                                 |
//! | [`mnt_fs_set_freq`][282]            | [`FsTabEntry::set_backup_frequency`](crate::core::entries::FsTabEntry::set_backup_frequency)                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_fs_set_fstype`][283]          | [`FsTabEntry::set_file_system_type`](crate::core::entries::FsTabEntry::set_file_system_type)                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_fs_set_options`][284]         | [`FsTabEntry::set_mount_options`](crate::core::entries::FsTabEntry::set_mount_options)                                                                                                                                                                                                                                                                                                                                     |
//! | [`mnt_fs_set_passno`][285]          | [`FsTabEntry::set_fsck_checking_order`](crate::core::entries::FsTabEntry::set_fsck_checking_order)                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_fs_set_priority`][286]        | [`SwapsEntry::set_priority`](crate::core::entries::SwapsEntry::set_priority)                                                                                                                                                                                                                                                                                                                                               |
//! | [`mnt_fs_set_root`][287]            | [`UTabEntry::set_root`](crate::core::entries::UTabEntry::set_root)                                                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_fs_set_source`][288]          | [`FsTabEntry::set_source`](crate::core::entries::FsTabEntry::set_source) <br> [`UTabEntry::set_source_path`](crate::core::entries::UTabEntry::set_source_path)                                                                                                                                                                                                                                                             |
//! | [`mnt_fs_set_target`][289]          | [`FsTabEntry::set_target`](crate::core::entries::FsTabEntry::set_target)                                                                                                                                                                                                                                                                                                                                                   |
//! | [`mnt_fs_set_userdata`][290]        | Managed internally.                                                                                                                                                                                                                                                                                                                                                                                                        |
//! | [`mnt_fs_strdup_options`][291]      | [`MountInfoEntry::fs_options`](crate::core::entries::MountInfoEntry::fs_options)                                                                                                                                                                                                                                                                                                                                           |
//! | [`mnt_fs_streq_srcpath`][292]       | [`FsTabEntry::is_exact_source`](crate::core::entries::FsTabEntry::is_exact_source) <br> [`MountInfoEntry::is_exact_source`](crate::core::entries::MountInfoEntry::is_exact_source) <br> [`SwapsEntry::is_exact_source`](crate::core::entries::SwapsEntry::is_exact_source) <br> [`UTabEntry::is_exact_source`](crate::core::entries::UTabEntry::is_exact_source)                                                           |
//! | [`mnt_fs_streq_target`][293]        | [`FsTabEntry::is_exact_target`](crate::core::entries::FsTabEntry::is_exact_target) <br> [`MountInfoEntry::is_exact_target`](crate::core::entries::MountInfoEntry::is_exact_target) <br> [`UTabEntry::is_exact_target`](crate::core::entries::UTabEntry::is_exact_target)                                                                                                                                                   |
//! | [`mnt_fs_to_mntent`][294]           | [`FsTabEntry::to_mnt_ent`](crate::core::entries::FsTabEntry::to_mnt_ent)                                                                                                                                                                                                                                                                                                                                                   |
//! | [`mnt_new_fs`][295]                 | [`FsTabEntry::builder`](crate::core::entries::FsTabEntry::builder)                                                                                                                                                                                                                                                                                                                                                         |
//! | [`mnt_reset_fs`][296]               | Not supported.                                                                                                                                                                                                                                                                                                                                                                                                             |
//!
//! [228]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#libmnt-fs
//! [229]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-copy-fs
//! [230]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-free-fs
//! [231]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-free-mntent
//! [232]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-ref-fs
//! [233]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-unref-fs
//! [234]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-append-attributes
//! [235]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-append-comment
//! [236]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-append-options
//! [237]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-attribute
//! [238]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-attributes
//! [239]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-bindsrc
//! [240]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-comment
//! [241]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-devno
//! [242]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-freq
//! [243]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-fs-options
//! [244]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-fstype
//! [245]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-id
//! [246]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-option
//! [247]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-optional-fields
//! [248]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-options
//! [249]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-parent-id
//! [250]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-passno
//! [251]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-priority
//! [252]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-propagation
//! [253]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-root
//! [254]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-size
//! [255]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-source
//! [256]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-srcpath
//! [257]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-swaptype
//! [258]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-tag
//! [259]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-table
//! [260]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-target
//! [261]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-tid
//! [262]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-usedsize
//! [263]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-userdata
//! [264]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-user-options
//! [265]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-vfs-options
//! [266]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-get-vfs-options-all
//! [267]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-is-kernel
//! [268]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-is-netfs
//! [269]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-is-pseudofs
//! [270]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-is-regularfs
//! [271]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-is-swaparea
//! [272]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-match-fstype
//! [273]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-match-options
//! [274]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-match-source
//! [275]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-match-target
//! [276]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-prepend-attributes
//! [277]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-prepend-options
//! [278]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-print-debug
//! [279]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-attributes
//! [280]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-bindsrc
//! [281]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-comment
//! [282]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-freq
//! [283]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-fstype
//! [284]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-options
//! [285]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-passno
//! [286]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-priority
//! [287]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-root
//! [288]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-source
//! [289]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-target
//! [290]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-set-userdata
//! [291]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-strdup-options
//! [292]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-streq-srcpath
//! [293]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-streq-target
//! [294]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-fs-to-mntent
//! [295]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-new-fs
//! [296]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Filesystem.html#mnt-reset-fs
//!
//! ### Tables management
//! #### Locking
//!
//! | `libmount`                      | `rsmount` |
//! | ------------------              | --------- |
//! | [`struct libmnt_lock`][297]     |           |
//! | [`mnt_free_lock`][298]          |           |
//! | [`mnt_lock_file`][299]          |           |
//! | [`mnt_new_lock`][300]           |           |
//! | [`mnt_unlock_file`][301]        |           |
//! | [`mnt_lock_block_signals`][302] |           |
//!
//! [297]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Locking.html#libmnt-lock
//! [298]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Locking.html#mnt-free-lock
//! [299]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Locking.html#mnt-lock-file
//! [300]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Locking.html#mnt-new-lock
//! [301]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Locking.html#mnt-unlock-file
//! [302]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Locking.html#mnt-lock-block-signals
//!
//! #### Tables update
//!
//! | `libmount`                       | `rsmount`                                                   |
//! | ------------------               | ---------                                                   |
//! | [`struct libmnt_update`][303]    |                                                             |
//! | [`mnt_free_update`][304]         |                                                             |
//! | [`mnt_new_update`][305]          |                                                             |
//! | [`mnt_table_replace_file`][306]  | [`FsTab::write_file`](crate::tables::FsTab::write_file)     |
//! | [`mnt_table_write_file`][307]    | [`FsTab::write_stream`](crate::tables::FsTab::write_stream) |
//! | [`mnt_update_force_rdonly`][308] |                                                             |
//! | [`mnt_update_get_filename`][309] |                                                             |
//! | [`mnt_update_get_fs`][310]       |                                                             |
//! | [`mnt_update_get_mflags`][311]   |                                                             |
//! | [`mnt_update_is_ready`][312]     |                                                             |
//! | [`mnt_update_set_fs`][313]       |                                                             |
//! | [`mnt_update_table`][314]        |                                                             |
//!
//! [303]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#libmnt-update
//! [304]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-free-update
//! [305]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-new-update
//! [306]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-table-replace-file
//! [307]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-table-write-file
//! [308]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-update-force-rdonly
//! [309]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-update-get-filename
//! [310]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-update-get-fs
//! [311]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-update-get-mflags
//! [312]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-update-is-ready
//! [313]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-update-set-fs
//! [314]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Tables-update.html#mnt-update-table
//!
//! #### Monitor
//!
//! | `libmount`                            | `rsmount` |
//! | ------------------                    | --------- |
//! | [`struct libmnt_monitor`][315]        |           |
//! | [`mnt_new_monitor`][316]              |           |
//! | [`mnt_ref_monitor`][317]              |           |
//! | [`mnt_unref_monitor`][318]            |           |
//! | [`mnt_monitor_enable_userspace`][319] |           |
//! | [`mnt_monitor_enable_kernel`][320]    |           |
//! | [`mnt_monitor_get_fd`][321]           |           |
//! | [`mnt_monitor_close_fd`][322]         |           |
//! | [`mnt_monitor_next_change`][323]      |           |
//! | [`mnt_monitor_event_cleanup`][324]    |           |
//! | [`mnt_monitor_wait`][325]             |           |
//!
//! [315]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#libmnt-monitor
//! [316]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-new-monitor
//! [317]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-ref-monitor
//! [318]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-unref-monitor
//! [319]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-monitor-enable-userspace
//! [320]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-monitor-enable-kernel
//! [321]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-monitor-get-fd
//! [322]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-monitor-close-fd
//! [323]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-monitor-next-change
//! [324]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-monitor-event-cleanup
//! [325]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Monitor.html#mnt-monitor-wait
//!
//! #### Compare changes in mount tables
//!
//! | `libmount`                       | `rsmount` |
//! | ------------------               | --------- |
//! | [`struct libmnt_tabdiff`][326]   |           |
//! | [`mnt_new_tabdiff`][327]         |           |
//! | [`mnt_free_tabdiff`][329]        |           |
//! | [`mnt_tabdiff_next_change`][331] |           |
//! | [`mnt_diff_tables`][329]         |           |
//!
//! [326]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Compare-changes-in-mount-tables.html#libmnt-tabdiff
//! [327]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Compare-changes-in-mount-tables.html#mnt-new-tabdiff
//! [328]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Compare-changes-in-mount-tables.html#mnt-free-tabdiff
//! [329]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Compare-changes-in-mount-tables.html#mnt-tabdiff-next-change
//! [330]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Compare-changes-in-mount-tables.html#mnt-diff-tables
//!
//! ### Mount options
//! #### Options string
//!
//! | `libmount`                             | `rsmount` |
//! | ------------------                     | --------- |
//! | [`mnt_optstr_append_option`][330]      |           |
//! | [`mnt_optstr_apply_flags`][331]        |           |
//! | [`mnt_optstr_deduplicate_option`][332] |           |
//! | [`mnt_optstr_get_flags`][333]          |           |
//! | [`mnt_optstr_get_option`][334]         |           |
//! | [`mnt_optstr_get_options`][335]        |           |
//! | [`mnt_optstr_next_option`][336]        |           |
//! | [`mnt_optstr_prepend_option`][337]     |           |
//! | [`mnt_optstr_remove_option`][338]      |           |
//! | [`mnt_optstr_set_option`][339]         |           |
//! | [`mnt_split_optstr`][340]              |           |
//! | [`mnt_match_options`][341]             |           |
//!
//! [330]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-append-option
//! [331]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-apply-flags
//! [332]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-deduplicate-option
//! [333]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-get-flags
//! [334]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-get-option
//! [335]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-get-options
//! [336]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-next-option
//! [337]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-prepend-option
//! [338]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-remove-option
//! [339]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-optstr-set-option
//! [340]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-split-optstr
//! [341]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Options-string.html#mnt-match-options
//!
//! #### Option maps
//!
//! | `libmount`                      | `rsmount` |
//! | ------------------              | --------- |
//! | [`struct libmnt_optmap`][342]   |           |
//! | [`MNT_INVERT`][343]             |           |
//! | [`MNT_NOMTAB`][344]             |           |
//! | [`MNT_PREFIX`][345]             |           |
//! | [`MNT_NOHLPS`][346]             |           |
//! | [`mnt_get_builtin_optmap`][347] |           |
//!
//! [342]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Option-maps.html#libmnt-optmap
//! [343]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Option-maps.html#MNT-INVERT:CAPS
//! [344]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Option-maps.html#MNT-NOMTAB:CAPS
//! [345]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Option-maps.html#MNT-PREFIX:CAPS
//! [346]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Option-maps.html#MNT-NOHLPS:CAPS
//! [347]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Option-maps.html#mnt-get-builtin-optmap
//!
//! ### Misc
//! #### Library initialization
//!
//! | `libmount`              | `rsmount`                                                   |
//! | ------------------      | ---------                                                   |
//! | [`mnt_init_debug`][348] | [`debug::init_default_debug`]<br>[`debug::init_full_debug`] |
//!
//! [348]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-initialization.html#mnt-init-debug
//!
//! #### Cache
//!
//! | `libmount`                        | `rsmount`                                                                                                                                                                                                        |
//! | ------------------                | ---------                                                                                                                                                                                                        |
//! | [`struct libmnt_cache`][349]      | [`Cache`](crate::core::cache::Cache)                                                                                                                                                                             |
//! | [`mnt_new_cache`][350]            | [`Cache::new`](crate::core::cache::Cache::new)                                                                                                                                                                   |
//! | [`mnt_free_cache`][351]           | [`Cache`](crate::core::cache::Cache) is automatically deallocated when it goes out of scope.                                                                                                                     |
//! | [`mnt_ref_cache`][352]            | Managed automatically.                                                                                                                                                                                           |
//! | [`mnt_unref_cache`][353]          | Managed automatically.                                                                                                                                                                                           |
//! | [`mnt_cache_device_has_tag`][354] | [`Cache::device_has_tag`](crate::core::cache::Cache::device_has_tag)                                                                                                                                             |
//! | [`mnt_cache_find_tag_value`][355] | [`Cache::find_tag_value`](crate::core::cache::Cache::find_tag_value)                                                                                                                                             |
//! | [`mnt_cache_read_tags`][356]      | [`Cache::import_tags`](crate::core::cache::Cache::import_tags)                                                                                                                                                   |
//! | [`mnt_cache_set_targets`][357]    | [`Cache::import_paths`](crate::core::cache::Cache::import_paths)                                                                                                                                                 |
//! | [`mnt_cache_set_sbprobe`][358]    | [`Cache::collect_fs_properties`](crate::core::cache::Cache::collect_fs_properties)                                                                                                                               |
//! | [`mnt_get_fstype`][359]           | [`Cache::find_file_system_type`](crate::core::cache::Cache::find_file_system_type)<br>[`Cache::find_and_cache_file_system_type`](crate::core::cache::Cache::find_and_cache_file_system_type)                     |
//! | [`mnt_pretty_path`][360]          | [`Cache::canonicalize`](crate::core::cache::Cache::canonicalize)<br>[`Cache::canonicalize_and_cache`](crate::core::cache::Cache::canonicalize_and_cache)                                                         |
//! | [`mnt_resolve_path`][361]         | [`Cache::resolve`](crate::core::cache::Cache::resolve)<br>[`Cache::resolve_and_cache`](crate::core::cache::Cache::resolve_and_cache)                                                                             |
//! | [`mnt_resolve_spec`][362]         | Not implemented. Use the specialized functions corresponding to `mnt_resolve_path` or `mnt_resolve_tag` as applicable.                                                                                           |
//! | [`mnt_resolve_tag`][363]          | [`Cache::find_first_device_with_tag`](crate::core::cache::Cache::find_first_device_with_tag)<br>[`Cache::find_and_cache_first_device_with_tag`](crate::core::cache::Cache::find_and_cache_first_device_with_tag) |
//! | [`mnt_resolve_target`][364]       | [`Cache::find_device_mounted_at`](crate::core::cache::Cache::find_device_mounted_at)<br>[`Cache::find_and_cache_device_mounted_at`](crate::core::cache::Cache::find_and_cache_device_mounted_at)                 |
//!
//! [349]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#libmnt-cache
//! [350]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-new-cache
//! [351]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-free-cache
//! [352]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-ref-cache
//! [353]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-unref-cache
//! [354]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-cache-device-has-tag
//! [355]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-cache-find-tag-value
//! [356]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-cache-read-tags
//! [357]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-cache-set-targets
//! [358]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-cache-set-sbprobe
//! [359]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-get-fstype
//! [360]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-pretty-path
//! [361]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-resolve-path
//! [362]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-resolve-spec
//! [363]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-resolve-tag
//! [364]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Cache.html#mnt-resolve-target
//!
//! #### Iterator
//!
//! | `libmount`                      | `rsmount`                                                                                                                                                                                                                         |
//! | ------------------              | ---------                                                                                                                                                                                                                         |
//! | [`struct libmnt_iter`][365]     | [`GenIterator`](crate::core::iter::GenIterator)                                                                                                                                                                                   |
//! | [`mnt_free_iter`][366]          | [`GenIterator`](crate::core::iter::GenIterator) is automatically deallocated when it goes out of scope.                                                                                                                           |
//! | [`mnt_iter_get_direction`][367] | [`GenIterator::direction`](crate::core::iter::GenIterator::direction)                                                                                                                                                             |
//! | [`mnt_new_iter`][368]           | [`GenIterator::new`](crate::core::iter::GenIterator::new)                                                                                                                                                                         |
//! | [`mnt_reset_iter`][369]         | [`GenIterator::reset`](crate::core::iter::GenIterator::reset)<br>[`GenIterator::reset_forward`](crate::core::iter::GenIterator::reset_forward)<br>[`GenIterator::reset_backward`](crate::core::iter::GenIterator::reset_backward) |
//!
//! [365]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#libmnt-iter
//! [366]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#mnt-free-iter
//! [367]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#mnt-iter-get-direction
//! [368]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#mnt-new-iter
//! [369]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#mnt-reset-iter
//!
//! #### Utils
//!
//! | `libmount`                      | `rsmount`                                                                                             |
//! | ------------------              | ---------                                                                                             |
//! | [`mnt_fstype_is_netfs`][370]    | [`core::utils::is_network_fs`]                                                                        |
//! | [`mnt_fstype_is_pseudofs`][371] | [`core::utils::is_pseudo_fs`]                                                                         |
//! | [`mnt_get_fstab_path`][372]     | [`core::utils::path_to_fstab`]                                                                        |
//! | [`mnt_get_mountpoint`][373]     | [`core::utils::find_device_mountpoint`]                                                               |
//! | [`mnt_get_mtab_path`][374]      | Deprecated.                                                                                           |
//! | [`mnt_get_swaps_path`][375]     | [`core::utils::path_to_swaps`]                                                                        |
//! | [`mnt_guess_system_root`][376]  | [`core::utils::device_number_to_device_name`]<br>[`core::utils::device_number_to_cached_device_name`] |
//! | [`mnt_has_regular_mtab`][377]   | Deprecated.                                                                                           |
//! | [`mnt_mangle`][378]             | [`core::utils::fstab_encode`]                                                                         |
//! | [`mnt_match_fstype`][379]       | [`core::utils::matches_fs_type`]                                                                      |
//! | [`mnt_tag_is_valid`][380]       | Not implemented. [`Tag`](crate::core::device::Tag)s are valid by definition.                          |
//! | [`mnt_unmangle`][381]           | [`core::utils::fstab_decode`]                                                                         |
//!
//! [370]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-fstype-is-netfs
//! [371]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-fstype-is-pseudofs
//! [372]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-get-fstab-path
//! [373]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-get-mountpoint
//! [374]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-get-mtab-path
//! [375]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-get-swaps-path
//! [376]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-guess-system-root
//! [377]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-has-regular-mtab
//! [378]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-mangle
//! [379]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-match-fstype
//! [380]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-tag-is-valid
//! [381]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Utils.html#mnt-unmangle
//!
//! #### Version functions
//!
//! | `libmount`                        | `rsmount` |
//! | ------------------                | --------- |
//! | [`LIBMOUNT_MAJOR_VERSION`][382]   |           |
//! | [`LIBMOUNT_MINOR_VERSION`][383]   |           |
//! | [`LIBMOUNT_PATCH_VERSION`][384]   |           |
//! | [`LIBMOUNT_VERSION`][385]         |           |
//! | [`mnt_parse_version_string`][386] |           |
//! | [`mnt_get_library_version`][387]  |           |
//! | [`mnt_get_library_features`][388] |           |
//!
//! [382]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Version-functions.html#LIBMOUNT-MAJOR-VERSION:CAPS
//! [383]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Version-functions.html#LIBMOUNT-MINOR-VERSION:CAPS
//! [384]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Version-functions.html#LIBMOUNT-PATCH-VERSION:CAPS
//! [385]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Version-functions.html#LIBMOUNT-VERSION:CAPS
//! [386]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Version-functions.html#mnt-parse-version-string
//! [387]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Version-functions.html#mnt-get-library-version
//! [388]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Version-functions.html#mnt-get-library-features

mod prelude {}

#[allow(unused_imports)]
use prelude::*;

pub use error::*;

pub mod core;
pub mod debug;
mod error;
pub(crate) mod ffi_utils;
pub mod tables;
