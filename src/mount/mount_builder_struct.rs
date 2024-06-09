// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use typed_builder::TypedBuilder;

// From standard library
use std::path::{Path, PathBuf};
use std::ptr::NonNull;

// From this library
use crate::core::cache::Cache;
//use crate::core::entries::MountTableEntry;
use crate::core::flags::MountFlag;
use crate::core::flags::UserspaceMountFlag;
use crate::core::fs::FileSystem;
use crate::tables::FsTab;

use crate::mount::Mount;
use crate::mount::MountBuilderError;
use crate::mount::MountOptionsMode;
use crate::mount::MountSource;

#[derive(Debug, TypedBuilder)]
#[builder(
    builder_type(
        name = MountBuilder,
        vis = "pub",
        doc ="Configure and instantiate a [`Mount`]."),
    build_method(vis = "", name = __make))]
pub(super) struct MntBuilder {
    #[builder(setter(
        strip_bool,
        doc = "Prevents [`Mount`] from calling `/sbin/mount.suffix` or `/sbin/umount.suffix` helper functions, where *suffix* is a file system type.
(see the [`mount` command](https://www.man7.org/linux/man-pages/man8/mount.8.html#EXTERNAL_HELPERS) or
[`umount` command](https://www.man7.org/linux/man-pages/man8/umount.8.html#EXTERNAL_HELPERS) man
pages for more information)"
    ))]
    disable_helpers: bool,

    #[builder(setter(
        strip_bool,
        doc = " Prevents [`Mount`] from looking-up a mountpoint or device in `/etc/fstab` if only
one of them is configured.

The standard form of the
[`mount` command](https://www.man7.org/linux/man-pages/man8/mount.8.html) is:
```text
mount -t type device dir
```

This tells the kernel to attach the file system found on *device* (which is of type *type*) at the
directory *dir*.

If only the directory or the device is given, for example:
```text
mount /dir
````
then `mount` looks for a mountpoint (and if not found then for a device) in the `/etc/fstab` file."
    ))]
    disable_mount_point_lookup: bool,

    #[builder(setter(strip_bool, doc = "Disables path canonicalization."))]
    disable_path_canonicalization: bool,

    #[builder(setter(
        strip_bool,
        doc = "Disables userspace mount table updates. (see the
[`mount` command](https://www.man7.org/linux/man-pages/man8/mount.8.html) man page, option `-n,
--no-mtab` repurposed from `/etc/mtab` to `/run/mount/utab`)"
    ))]
    do_not_update_utab: bool,

    #[builder(setter(
        strip_bool,
        doc = "Skips all mount source preparation, mount option analysis,
and the actual mounting process."
    ))]
    dry_run: bool,

    #[builder(setter(
        strip_bool,
        doc = "Forces a device to be mounted in read-write mode. [`Mount`] will not try to remount
the device in read-only mode if the mount attempt fails. (see the
[`mount` command](https://www.man7.org/linux/man-pages/man8/mount.8.html) man page, option `-w,
-rw, --read-write` for more information)"
    ))]
    force_mount_read_write: bool,

    #[builder(setter(
        strip_bool,
        doc = "Ignores `autofs` mount table entries when reading a mount table."
    ))]
    ignore_autofs: bool,

    #[builder(setter(
        strip_bool,
        doc = "Ignores mount options not supported by a file system."
    ))]
    ignore_unsupported_mount_options: bool,

    #[builder(setter(
        strip_bool,
        doc = "Checks that a device is not already mounted before mounting it."
    ))]
    mount_only_once: bool,

    #[builder(setter(
        strip_bool,
        doc = "Enables [`Mount`] functionality to fork off a new incarnation of `mount` for each device. This will do the mounts on
different devices or different NFS servers in parallel. This has the advantage of resulting in
faster execution; NFS timeouts proceed also in parallel.

A disadvantage is the order of mount operations is undefined. Thus, you cannot use this
option if you want to mount both `/usr` and `/usr/spool` which have to be mounted in order."
    ))]
    parallel_mount: bool,

    #[builder(setter(
        strip_bool,
        doc = "Disables ALL of `libmount`'s safety checks using a [`Mount`] as if its user has
root permissions.

**Warning:** This function is designed for cases where you lack `suid` permissions, but still want to directly
manage a mount through the kernel. Please note that bypassing `libmount`'s safety checks is a very **DANGEROUS**
exercise. Careful what you wish for!"
    ))]
    force_user_mount: bool,

    #[builder(setter(strip_bool, doc = "Enables verbose output."))]
    verbose: bool,

    #[builder(
        default,
        setter(strip_option, doc = "Sets the device's file system type.")
    )]
    file_system: Option<FileSystem>,

    #[builder(
        default,
        setter(
            transform = |fs_types: impl AsRef<str>| Some(fs_types.as_ref().to_string()),
            doc = "Sets a list of comma-separated file system types. The list of
file system types can be prefixed with `no`, to specify the file system types on which no
action should be taken.

For example, with the `mount` command, adding the following options
```text
mount -a -t nomsdos,smbfs
```
mounts all file systems, except those of type `msdos` and `smbfs`."
        )
    )]
    match_file_systems: Option<String>,

    #[builder(default, setter(strip_option, doc = "Sets mount flags."))]
    mount_flags: Option<Vec<MountFlag>>,

    #[builder(
        default,
        setter(
            transform = |options_list: impl AsRef<str>| Some(options_list.as_ref().to_string()),
            doc = "Sets a comma-separated list of mount options. For example
`\"noatime,nodev,nosuid\"`."
        )
    )]
    mount_options: Option<String>,

    #[builder(
        default,
        setter(
            strip_option,
            doc = "Sets [`MountOptionsMode`]s that define how to combine options from the `fstab`
and `mountinfo` files with the ones set by [`MountBuilder::mount_options`].

**Note:**
- [`MountOptionsMode::NonRootUser`] is always active when [`MountBuilder::force_user_mount`] is set.
- [`MountOptionsMode::Auto`] is active if no `MountOptionsMode` variant is provided.
- [`MountOptionsMode`] variants are evaluated in the following order:
   - [`MountOptionsMode::NoReadFromFstab`],
   - [`MountOptionsMode::ForceFstabOptions`],
   - [`MountOptionsMode::ReadFromFstab`],
   - [`MountOptionsMode::ReadFromMountinfo`],
   - [`MountOptionsMode::IgnoreOptions`],
   - [`MountOptionsMode::AppendOptions`],
   - [`MountOptionsMode::PrependOptions`],
   - [`MountOptionsMode::ReplaceOptions`].
"
        )
    )]
    mount_options_mode: Option<Vec<MountOptionsMode>>,

    #[builder(
        default,
        setter(
            strip_option,
            doc = "Sets the [`mount` syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html)'s
*data* argument. The *data* argument is typically a string of comma-separated options interpreted
and understood by each file system.

**Warning:** [`Mount`] does not deallocate the data referenced by the pointer when it goes out of scope.
```text
#include <sys/mount.h>

int mount(const char *source, const char *target,
          const char *filesystemtype, unsigned long mountflags,
          const void *_Nullable data);
```
"
        )
    )]
    mount_data: Option<NonNull<libc::c_void>>,

    #[builder(
        default,
        setter(
            transform = |options: impl AsRef<str>| Some(options.as_ref().to_string()),
            doc = "Sets a comma-delimited list of mount options to use for
selecting, and then mounting `/etc/fstab` entries matching any of the elements in the list.
(see the [`mount` command](https://www.man7.org/linux/man-pages/man8/mount.8.html#COMMAND-LINE_OPTIONS)
manpage, option `-O, --test-opts` for more information.)"
        )
    )]
    match_mount_options: Option<String>,

    #[builder(default, setter(strip_option, doc = "Sets userspace mount flags."))]
    userspace_mount_flags: Option<Vec<UserspaceMountFlag>>,

    #[builder(
        default,
        setter(
            strip_option,
            doc = "Overrides the [`Mount`]'s internal [`Cache`] with a custom one. If they already
exist, this cache will be associated with `fstab` and `mountinfo` instances."
        )
    )]
    override_cache: Option<Cache>,

    #[builder(
        default,
        setter(
            strip_option,
            doc = "[`Mount`] keeps an internal copy of `/etc/fstab`. This function will override
that copy with the one provided."
        )
    )]
    override_fstab: Option<FsTab>,

    //     #[builder(
    //         default,
    //         setter(
    //             strip_option,
    //             doc = "Overrides the [`Mount`]'s internal mount table entry with a custom one.
    //
    // Internally a [`Mount`] uses a [`MountTableEntry`] to represent a device to mount. You
    // can use the functions [`MountBuilder::source`], [`MountBuilder::target`], [`MountBuilder::file_system`], etc. to
    // configure it manually, or override it with a custom [`MountTableEntry`] instance."
    //         )
    //     )]
    //     override_table_entry: Option<MountTableEntry>,
    #[builder(default, setter(strip_option, doc = "Sets the device to mount."))]
    source: Option<MountSource>,

    #[builder(
        default,
        setter(
            transform = |target: impl AsRef<Path>| Some(target.as_ref().to_path_buf()),
            doc = "Sets the device's mountpoint."
        )
    )]
    target: Option<PathBuf>,

    #[builder(
        default,
        setter(
            transform = |path: impl AsRef<Path>| Some(path.as_ref().to_path_buf()),
            doc = "Sets the target's Linux namespace to the one provided by `path`."
        )
    )]
    target_namespace: Option<PathBuf>,

    #[builder(
        default,
        setter(
            transform = |path: impl AsRef<Path>| Some(path.as_ref().to_path_buf()),
            doc = "Sets the target's prefix to the one provided by `path`."
        )
    )]
    target_prefix: Option<PathBuf>,
}

#[allow(non_camel_case_types)]
impl<
        __disable_helpers: ::typed_builder::Optional<bool>,
        __disable_mount_point_lookup: ::typed_builder::Optional<bool>,
        __disable_path_canonicalization: ::typed_builder::Optional<bool>,
        __do_not_update_utab: ::typed_builder::Optional<bool>,
        __dry_run: ::typed_builder::Optional<bool>,
        __force_mount_read_write: ::typed_builder::Optional<bool>,
        __ignore_autofs: ::typed_builder::Optional<bool>,
        __ignore_unsupported_mount_options: ::typed_builder::Optional<bool>,
        __mount_only_once: ::typed_builder::Optional<bool>,
        __parallel_mount: ::typed_builder::Optional<bool>,
        __force_user_mount: ::typed_builder::Optional<bool>,
        __verbose: ::typed_builder::Optional<bool>,
        __file_system: ::typed_builder::Optional<Option<FileSystem>>,
        __match_file_systems: ::typed_builder::Optional<Option<String>>,
        __mount_flags: ::typed_builder::Optional<Option<Vec<MountFlag>>>,
        __mount_options: ::typed_builder::Optional<Option<String>>,
        __mount_options_mode: ::typed_builder::Optional<Option<Vec<MountOptionsMode>>>,
        __mount_data: ::typed_builder::Optional<Option<NonNull<libc::c_void>>>,
        __match_mount_options: ::typed_builder::Optional<Option<String>>,
        __userspace_mount_flags: ::typed_builder::Optional<Option<Vec<UserspaceMountFlag>>>,
        __override_cache: ::typed_builder::Optional<Option<Cache>>,
        __override_fstab: ::typed_builder::Optional<Option<FsTab>>,
        //__override_table_entry: ::typed_builder::Optional<Option<MountTableEntry>>,
        __source: ::typed_builder::Optional<Option<MountSource>>,
        __target: ::typed_builder::Optional<Option<PathBuf>>,
        __target_namespace: ::typed_builder::Optional<Option<PathBuf>>,
        __target_prefix: ::typed_builder::Optional<Option<PathBuf>>,
    >
    MountBuilder<(
        __disable_helpers,
        __disable_mount_point_lookup,
        __disable_path_canonicalization,
        __do_not_update_utab,
        __dry_run,
        __force_mount_read_write,
        __ignore_autofs,
        __ignore_unsupported_mount_options,
        __mount_only_once,
        __parallel_mount,
        __force_user_mount,
        __verbose,
        __file_system,
        __match_file_systems,
        __mount_flags,
        __mount_options,
        __mount_options_mode,
        __mount_data,
        __match_mount_options,
        __userspace_mount_flags,
        __override_cache,
        __override_fstab,
        //__override_table_entry,
        __source,
        __target,
        __target_namespace,
        __target_prefix,
    )>
{
    pub fn build(self) -> Result<Mount, MountBuilderError> {
        log::debug!("MountBuilder::build building a new `Mount` instance");

        let builder = self.__make();
        let mut mount = Mount::new()?;

        // Set source/target fields.
        match (builder.source, builder.target) {
            (None, Some(target)) => mount.set_mount_target(target)?,
            (Some(source), None) => mount.set_mount_source(source)?,
            (Some(source), Some(target)) => {
                mount.set_mount_source(source)?;
                mount.set_mount_target(target)?;
            }
            (None, None) => {
                // Assume `mount -a` if a source or target is not provided
                // let err_msg = "you MUST call at least one of the following functions: `MountBuilder::source`, `MountBuilder::target`".to_owned();
                // return Err(MountBuilderError::Required(err_msg));
            }
        }

        // Set file system type.
        if let Some(fs_type) = builder.file_system {
            mount.set_file_system_type(fs_type)?;
        }

        // Set file systems list.
        if let Some(fs_list) = builder.match_file_systems {
            mount.set_file_systems_filter(fs_list)?;
        }

        // Override internal `fstab` with custom table.
        if let Some(fstab) = builder.override_fstab {
            mount.set_fstab(fstab)?;
        }

        if let Some(data) = builder.mount_data {
            mount.set_mount_data(data)?;
        }

        // if let Some(entry) = builder.override_table_entry {
        //     mount.set_table_entry(entry)?;
        // }

        // Override internal cache with custom instance.
        if let Some(cache) = builder.override_cache {
            mount.set_cache(cache)?;
        }

        // Set mount flags.
        if let Some(flags) = builder.mount_flags {
            mount.set_mount_flags(flags)?;
        }

        // Set mount options.
        if let Some(list) = builder.mount_options {
            mount.set_mount_options(list)?;
        }

        // Set mount options mode.
        if let Some(mode) = builder.mount_options_mode {
            mount.set_mount_options_mode(mode)?;
        }

        // Set test options.
        if let Some(list) = builder.match_mount_options {
            mount.set_mount_options_filter(list)?;
        }

        // Set mount flags.
        if let Some(flags) = builder.userspace_mount_flags {
            mount.set_userspace_mount_flags(flags)?;
        }

        if let Some(namespace) = builder.target_namespace {
            mount.set_mount_target_namespace(namespace)?;
        }

        if let Some(prefix) = builder.target_prefix {
            mount.set_mount_target_prefix(prefix)?;
        }

        if builder.disable_helpers {
            mount.disable_helpers()?;
        } else {
            mount.enable_helpers()?;
        }

        if builder.disable_mount_point_lookup {
            mount.disable_mount_point_lookup()?;
        } else {
            mount.enable_mount_point_lookup()?;
        }

        if builder.disable_path_canonicalization {
            mount.disable_path_canonicalization()?;
        } else {
            mount.enable_path_canonicalization()?;
        }

        if builder.do_not_update_utab {
            mount.do_not_update_utab()?;
        } else {
            mount.update_utab()?;
        }

        if builder.dry_run {
            mount.enable_dry_run()?;
        } else {
            mount.disable_dry_run()?;
        }

        if builder.force_mount_read_write {
            mount.enable_force_mount_read_write()?;
        } else {
            mount.disable_force_mount_read_write()?;
        }

        if builder.ignore_autofs {
            #[cfg(v2_39)]
            mount.enable_ignore_autofs()?;
        } else {
            #[cfg(v2_39)]
            mount.disable_ignore_autofs()?;
        }

        if builder.ignore_unsupported_mount_options {
            mount.enable_ignore_unsupported_mount_options()?;
        } else {
            mount.disable_ignore_unsupported_mount_options()?;
        }

        if builder.mount_only_once {
            #[cfg(v2_39)]
            mount.enable_mount_only_once()?;
        } else {
            #[cfg(v2_39)]
            mount.disable_mount_only_once()?;
        }

        if builder.parallel_mount {
            mount.enable_parallel_mount()?;
        } else {
            mount.disable_parallel_mount()?;
        }

        if builder.force_user_mount {
            mount.force_user_mount()?;
        }

        if builder.verbose {
            mount.enable_verbose_output()?;
        } else {
            mount.disable_verbose_output()?;
        }

        Ok(mount)
    }
}
