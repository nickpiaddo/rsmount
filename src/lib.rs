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
//! ## API structure
//!
//! `rsmount`'s API is roughly divided into three main modules:
//! - `core`: a module for items in the library's low-level API.
//! - `tables`: a module for manipulating file system descriptions tables (`/etc/fstab`,
//! `/proc/self/mountinfo`, `/proc/swaps`, `/run/mount/utab`).
//! - `mount`: a module to mount devices on the system's file tree.
//!
//! Finally, look to the `debug` module if you need to consult debug messages during development.
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
//! | `libmount`                          | `rsmount` |
//! | ------------------                  | --------- |
//! | [`MNT_MS_COMMENT`][113]             |           |
//! | [`MNT_MS_GROUP`][114]               |           |
//! | [`MNT_MS_HELPER`][115]              |           |
//! | [`MNT_MS_LOOP`][116]                |           |
//! | [`MNT_MS_NETDEV`][117]              |           |
//! | [`MNT_MS_NOAUTO`][118]              |           |
//! | [`MNT_MS_NOFAIL`][119]              |           |
//! | [`MNT_MS_OFFSET`][120]              |           |
//! | [`MNT_MS_OWNER`][121]               |           |
//! | [`MNT_MS_SIZELIMIT`][122]           |           |
//! | [`MNT_MS_ENCRYPTION`][123]          |           |
//! | [`MNT_MS_UHELPER`][124]             |           |
//! | [`MNT_MS_USER`][125]                |           |
//! | [`MNT_MS_USERS`][126]               |           |
//! | [`MNT_MS_XCOMMENT`][127]            |           |
//! | [`MNT_MS_XFSTABCOMM`][128]          |           |
//! | [`MNT_MS_HASH_DEVICE`][129]         |           |
//! | [`MNT_MS_ROOT_HASH`][130]           |           |
//! | [`MNT_MS_HASH_OFFSET`][131]         |           |
//! | [`MNT_MS_ROOT_HASH_FILE`][132]      |           |
//! | [`MNT_MS_FEC_DEVICE`][133]          |           |
//! | [`MNT_MS_FEC_OFFSET`][134]          |           |
//! | [`MNT_MS_FEC_ROOTS`][135]           |           |
//! | [`MNT_MS_ROOT_HASH_SIG`][136]       |           |
//! | [`MS_BIND`][137]                    |           |
//! | [`MS_DIRSYNC`][138]                 |           |
//! | [`MS_I_VERSION`][139]               |           |
//! | [`MS_MANDLOCK`][140]                |           |
//! | [`MS_MGC_MSK`][141]                 |           |
//! | [`MS_MGC_VAL`][142]                 |           |
//! | [`MS_MOVE`][143]                    |           |
//! | [`MS_NOATIME`][144]                 |           |
//! | [`MS_NODEV`][145]                   |           |
//! | [`MS_NODIRATIME`][146]              |           |
//! | [`MS_NOEXEC`][147]                  |           |
//! | [`MS_NOSUID`][148]                  |           |
//! | [`MS_OWNERSECURE`][149]             |           |
//! | [`MS_PRIVATE`][150]                 |           |
//! | [`MS_PROPAGATION`][151]             |           |
//! | [`MS_RDONLY`][152]                  |           |
//! | [`MS_REC`][153]                     |           |
//! | [`MS_RELATIME`][154]                |           |
//! | [`MS_REMOUNT`][155]                 |           |
//! | [`MS_SECURE`][156]                  |           |
//! | [`MS_SHARED`][157]                  |           |
//! | [`MS_SILENT`][158]                  |           |
//! | [`MS_SLAVE`][159]                   |           |
//! | [`MS_STRICTATIME`][160]             |           |
//! | [`MS_SYNCHRONOUS`][161]             |           |
//! | [`MS_UNBINDABLE`][162]              |           |
//! | [`MS_LAZYTIME`][163]                |           |
//! | [`mnt_context_do_mount`][164]       |           |
//! | [`mnt_context_finalize_mount`][165] |           |
//! | [`mnt_context_mount`][166]          |           |
//! | [`mnt_context_next_mount`][167]     |           |
//! | [`mnt_context_next_remount`][168]   |           |
//! | [`mnt_context_prepare_mount`][169]  |           |
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
//! | `libmount`                                 | `rsmount` |
//! | ------------------                         | --------- |
//! | [`struct libmnt_table`][176]               |           |
//! | [`mnt_free_table`][177]                    |           |
//! | [`mnt_new_table`][178]                     |           |
//! | [`mnt_reset_table`][179]                   |           |
//! | [`mnt_ref_table`][180]                     |           |
//! | [`mnt_unref_table`][181]                   |           |
//! | [`mnt_new_table_from_dir`][182]            |           |
//! | [`mnt_new_table_from_file`][183]           |           |
//! | [`mnt_table_add_fs`][184]                  |           |
//! | [`mnt_table_append_intro_comment`][185]    |           |
//! | [`mnt_table_append_trailing_comment`][186] |           |
//! | [`mnt_table_enable_comments`][187]         |           |
//! | [`mnt_table_find_devno`][188]              |           |
//! | [`mnt_table_find_fs`][189]                 |           |
//! | [`mnt_table_find_mountpoint`][190]         |           |
//! | [`mnt_table_find_next_fs`][191]            |           |
//! | [`mnt_table_find_pair`][192]               |           |
//! | [`mnt_table_find_source`][193]             |           |
//! | [`mnt_table_find_srcpath`][194]            |           |
//! | [`mnt_table_find_tag`][195]                |           |
//! | [`mnt_table_find_target`][196]             |           |
//! | [`mnt_table_find_target_with_option`][197] |           |
//! | [`mnt_table_first_fs`][198]                |           |
//! | [`mnt_table_get_cache`][199]               |           |
//! | [`mnt_table_get_intro_comment`][200]       |           |
//! | [`mnt_table_get_nents`][201]               |           |
//! | [`mnt_table_get_root_fs`][202]             |           |
//! | [`mnt_table_get_trailing_comment`][203]    |           |
//! | [`mnt_table_get_userdata`][204]            |           |
//! | [`mnt_table_insert_fs`][205]               |           |
//! | [`mnt_table_is_empty`][206]                |           |
//! | [`mnt_table_is_fs_mounted`][207]           |           |
//! | [`mnt_table_last_fs`][208]                 |           |
//! | [`mnt_table_move_fs`][209]                 |           |
//! | [`mnt_table_next_child_fs`][210]           |           |
//! | [`mnt_table_next_fs`][211]                 |           |
//! | [`mnt_table_over_fs`][212]                 |           |
//! | [`mnt_table_parse_dir`][213]               |           |
//! | [`mnt_table_parse_file`][214]              |           |
//! | [`mnt_table_parse_fstab`][215]             |           |
//! | [`mnt_table_parse_mtab`][216]              |           |
//! | [`mnt_table_parse_stream`][217]            |           |
//! | [`mnt_table_parse_swaps`][218]             |           |
//! | [`mnt_table_remove_fs`][219]               |           |
//! | [`mnt_table_set_cache`][220]               |           |
//! | [`mnt_table_set_intro_comment`][221]       |           |
//! | [`mnt_table_set_iter`][222]                |           |
//! | [`mnt_table_set_parser_errcb`][223]        |           |
//! | [`mnt_table_set_trailing_comment`][224]    |           |
//! | [`mnt_table_set_userdata`][225]            |           |
//! | [`mnt_table_uniq_fs`][226]                 |           |
//! | [`mnt_table_with_comments`][227]           |           |
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
//! | `libmount`                          | `rsmount` |
//! | ------------------                  | --------- |
//! | [`struct libmnt_fs`][228]           |           |
//! | [`mnt_copy_fs`][229]                |           |
//! | [`mnt_free_fs`][230]                |           |
//! | [`mnt_free_mntent`][231]            |           |
//! | [`mnt_ref_fs`][232]                 |           |
//! | [`mnt_unref_fs`][233]               |           |
//! | [`mnt_fs_append_attributes`][234]   |           |
//! | [`mnt_fs_append_comment`][235]      |           |
//! | [`mnt_fs_append_options`][236]      |           |
//! | [`mnt_fs_get_attribute`][237]       |           |
//! | [`mnt_fs_get_attributes`][238]      |           |
//! | [`mnt_fs_get_bindsrc`][239]         |           |
//! | [`mnt_fs_get_comment`][240]         |           |
//! | [`mnt_fs_get_devno`][241]           |           |
//! | [`mnt_fs_get_freq`][242]            |           |
//! | [`mnt_fs_get_fs_options`][243]      |           |
//! | [`mnt_fs_get_fstype`][244]          |           |
//! | [`mnt_fs_get_id`][245]              |           |
//! | [`mnt_fs_get_option`][246]          |           |
//! | [`mnt_fs_get_optional_fields`][247] |           |
//! | [`mnt_fs_get_options`][248]         |           |
//! | [`mnt_fs_get_parent_id`][249]       |           |
//! | [`mnt_fs_get_passno`][250]          |           |
//! | [`mnt_fs_get_priority`][251]        |           |
//! | [`mnt_fs_get_propagation`][252]     |           |
//! | [`mnt_fs_get_root`][253]            |           |
//! | [`mnt_fs_get_size`][254]            |           |
//! | [`mnt_fs_get_source`][255]          |           |
//! | [`mnt_fs_get_srcpath`][256]         |           |
//! | [`mnt_fs_get_swaptype`][257]        |           |
//! | [`mnt_fs_get_tag`][258]             |           |
//! | [`mnt_fs_get_table`][259]           |           |
//! | [`mnt_fs_get_target`][260]          |           |
//! | [`mnt_fs_get_tid`][261]             |           |
//! | [`mnt_fs_get_usedsize`][262]        |           |
//! | [`mnt_fs_get_userdata`][263]        |           |
//! | [`mnt_fs_get_user_options`][264]    |           |
//! | [`mnt_fs_get_vfs_options`][265]     |           |
//! | [`mnt_fs_get_vfs_options_all`][266] |           |
//! | [`mnt_fs_is_kernel`][267]           |           |
//! | [`mnt_fs_is_netfs`][268]            |           |
//! | [`mnt_fs_is_pseudofs`][269]         |           |
//! | [`mnt_fs_is_regularfs`][270]        |           |
//! | [`mnt_fs_is_swaparea`][271]         |           |
//! | [`mnt_fs_match_fstype`][272]        |           |
//! | [`mnt_fs_match_options`][273]       |           |
//! | [`mnt_fs_match_source`][274]        |           |
//! | [`mnt_fs_match_target`][275]        |           |
//! | [`mnt_fs_prepend_attributes`][276]  |           |
//! | [`mnt_fs_prepend_options`][277]     |           |
//! | [`mnt_fs_print_debug`][278]         |           |
//! | [`mnt_fs_set_attributes`][279]      |           |
//! | [`mnt_fs_set_bindsrc`][280]         |           |
//! | [`mnt_fs_set_comment`][281]         |           |
//! | [`mnt_fs_set_freq`][282]            |           |
//! | [`mnt_fs_set_fstype`][283]          |           |
//! | [`mnt_fs_set_options`][284]         |           |
//! | [`mnt_fs_set_passno`][285]          |           |
//! | [`mnt_fs_set_priority`][286]        |           |
//! | [`mnt_fs_set_root`][287]            |           |
//! | [`mnt_fs_set_source`][288]          |           |
//! | [`mnt_fs_set_target`][289]          |           |
//! | [`mnt_fs_set_userdata`][290]        |           |
//! | [`mnt_fs_strdup_options`][291]      |           |
//! | [`mnt_fs_streq_srcpath`][292]       |           |
//! | [`mnt_fs_streq_target`][293]        |           |
//! | [`mnt_fs_to_mntent`][294]           |           |
//! | [`mnt_new_fs`][295]                 |           |
//! | [`mnt_reset_fs`][296]               |           |
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
//! | `libmount`                       | `rsmount` |
//! | ------------------               | --------- |
//! | [`struct libmnt_update`][303]    |           |
//! | [`mnt_free_update`][304]         |           |
//! | [`mnt_new_update`][305]          |           |
//! | [`mnt_table_replace_file`][306]  |           |
//! | [`mnt_table_write_file`][307]    |           |
//! | [`mnt_update_force_rdonly`][308] |           |
//! | [`mnt_update_get_filename`][309] |           |
//! | [`mnt_update_get_fs`][310]       |           |
//! | [`mnt_update_get_mflags`][311]   |           |
//! | [`mnt_update_is_ready`][312]     |           |
//! | [`mnt_update_set_fs`][313]       |           |
//! | [`mnt_update_table`][314]        |           |
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
//! | `libmount`              | `rsmount` |
//! | ------------------      | --------- |
//! | [`mnt_init_debug`][348] |           |
//!
//! [348]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Library-initialization.html#mnt-init-debug
//!
//! #### Cache
//!
//! | `libmount`                        | `rsmount` |
//! | ------------------                | --------- |
//! | [`struct libmnt_cache`][349]      |           |
//! | [`mnt_new_cache`][350]            |           |
//! | [`mnt_free_cache`][351]           |           |
//! | [`mnt_ref_cache`][352]            |           |
//! | [`mnt_unref_cache`][353]          |           |
//! | [`mnt_cache_device_has_tag`][354] |           |
//! | [`mnt_cache_find_tag_value`][355] |           |
//! | [`mnt_cache_read_tags`][356]      |           |
//! | [`mnt_cache_set_targets`][357]    |           |
//! | [`mnt_cache_set_sbprobe`][358]    |           |
//! | [`mnt_get_fstype`][359]           |           |
//! | [`mnt_pretty_path`][360]          |           |
//! | [`mnt_resolve_path`][361]         |           |
//! | [`mnt_resolve_spec`][362]         |           |
//! | [`mnt_resolve_tag`][363]          |           |
//! | [`mnt_resolve_target`][364]       |           |
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
//! | `libmount`                      | `rsmount` |
//! | ------------------              | --------- |
//! | [`struct libmnt_iter`][365]     |           |
//! | [`mnt_free_iter`][366]          |           |
//! | [`mnt_iter_get_direction`][367] |           |
//! | [`mnt_new_iter`][368]           |           |
//! | [`mnt_reset_iter`][369]         |           |
//!
//! [365]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#libmnt-iter
//! [366]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#mnt-free-iter
//! [367]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#mnt-iter-get-direction
//! [368]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#mnt-new-iter
//! [369]: https://mirrors.edge.kernel.org/pub/linux/utils/util-linux/v2.39/libmount-docs/libmount-Iterator.html#mnt-reset-iter
//!
//! #### Utils
//!
//! | `libmount`                      | `rsmount` |
//! | ------------------              | --------- |
//! | [`mnt_fstype_is_netfs`][370]    |           |
//! | [`mnt_fstype_is_pseudofs`][371] |           |
//! | [`mnt_get_fstab_path`][372]     |           |
//! | [`mnt_get_mountpoint`][373]     |           |
//! | [`mnt_get_mtab_path`][374]      |           |
//! | [`mnt_get_swaps_path`][375]     |           |
//! | [`mnt_guess_system_root`][376]  |           |
//! | [`mnt_has_regular_mtab`][377]   |           |
//! | [`mnt_mangle`][378]             |           |
//! | [`mnt_match_fstype`][379]       |           |
//! | [`mnt_tag_is_valid`][380]       |           |
//! | [`mnt_unmangle`][381]           |           |
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

mod error;
