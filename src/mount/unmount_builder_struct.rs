// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use typed_builder::TypedBuilder;

// From standard library
use std::path::{Path, PathBuf};

// From this library
use crate::mount::MountSource;
use crate::mount::Unmount;
use crate::mount::UnmountBuilderError;

#[derive(Debug, TypedBuilder)]
#[builder(
    builder_type(
        name = UnmountBuilder,
        vis = "pub",
        doc ="Configure and instantiate an [`Unmount`]."),
    build_method(vis = "", name = __make))]
pub(super) struct UmntBuilder {
    #[builder(setter(
        strip_bool,
        doc = " Enables [`Unmount`]  to delete a loop device after unmounting it. This option is
unnecessary for devices initialized by the
[`mount` command](https://www.man7.org/linux/man-pages/man8/mount.8.html), in this case the
`autoclear` functionality is enabled by default."
    ))]
    detach_loop_device: bool,

    #[builder(setter(
        strip_bool,
        doc = "Prevents [`Unmount`] from calling /sbin/umount.suffix` helper functions, where
*suffix* is a file system type.  (see the [`umount`
command](https://www.man7.org/linux/man-pages/man8/umount.8.html#EXTERNAL_HELPERS) man
pages for more information)"
    ))]
    disable_helpers: bool,

    #[builder(setter(
        strip_bool,
        doc = " Prevents [`Unmount`] from looking-up a mountpoint or device in `/etc/fstab` if only
one of them is configured.

The standard form of the
[`umount` command](https://www.man7.org/linux/man-pages/man8/umount.8.html) is:
```text
umount -t type dir
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
[`umount` command](https://www.man7.org/linux/man-pages/man8/umount.8.html) man page, option `-n,
--no-mtab` repurposed from `/etc/mtab` to `/run/mount/utab`)"
    ))]
    do_not_update_utab: bool,

    #[builder(setter(
        strip_bool,
        doc = "Skips all unmount preparations, and the actual unmounting process."
    ))]
    dry_run: bool,

    #[builder(setter(
        strip_bool,
        doc = "
Force an unmount (in case of an unreachable NFS system).

Note that this option does not guarantee that the unmount command does not hang. It’s strongly
recommended to use absolute paths without symlinks to avoid unwanted
[readlink(2)](https://www.man7.org/linux/man-pages/man2/readlink.2.html) and
[stat(2)](https://www.man7.org/linux/man-pages/man2/stat.2.html) system
calls on unreachable NFS in umount.

 (see the
[`umount` command](https://www.man7.org/linux/man-pages/man8/umount.8.html) man page, option `-f,
--force` for more information)"
    ))]
    force_unmount: bool,

    #[builder(setter(
        strip_bool,
        doc = "Enables [`Unmount`] to detach the file system from the file hierarchy now, and clean
up all references to this file system as soon as it is not busy anymore.  (see the
[`umount` command](https://www.man7.org/linux/man-pages/man8/umount.8.html) man page,
option `-l, --lazy` for more information)"
    ))]
    lazy_unmount: bool,

    #[builder(setter(
        strip_bool,
        doc = "Tries to remount a file system in read-only mode when an unmount fails. (see the
[`umount` command](https://www.man7.org/linux/man-pages/man8/umount.8.html) man page, option `-r,
--read-only` for more information)"
    ))]
    on_fail_remount_read_only: bool,

    #[builder(setter(strip_bool, doc = "Enables verbose output."))]
    verbose: bool,

    #[builder(
        default,
        setter(
            transform = |fs_types: impl AsRef<str>| Some(fs_types.as_ref().to_string()),
            doc = "Sets a list of comma-separated file system types. The list of
file system types can be prefixed with `no`, to specify the file system types on which no
action should be taken.

For example, with the `umount` command, adding the following options
```text
umount -a -t nomsdos,smbfs
```
unmounts all file systems, except those of type `msdos` and `smbfs`."
        )
    )]
    match_file_systems: Option<String>,

    #[builder(
        default,
        setter(
            transform = |options: impl AsRef<str>| Some(options.as_ref().to_string()),
            doc = "Sets a comma-delimited list of mount options to use for
selecting, and then unmounting `/etc/fstab` entries matching any of the elements in the list.
(see the [`umount` command](https://www.man7.org/linux/man-pages/man8/umount.8.html#OPTIONS)
manpage, option `-O, --test-opts` for more information.)"
        )
    )]
    match_mount_options: Option<String>,

    //     #[builder(
    //         default,
    //         setter(
    //             strip_option,
    //             doc = "Overrides the [`Mount`]'s internal mount table entry with a custom one.
    //
    // Internally a [`Mount`] uses a [`MountTableEntry`] to represent a device to mount. You
    // can use the functions [`UnmountBuilder::source`], [`UnmountBuilder::target`], [`UnmountBuilder::file_system`], etc. to
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
}

#[allow(non_camel_case_types)]
impl<
        __detach_loop_device: ::typed_builder::Optional<bool>,
        __disable_helpers: ::typed_builder::Optional<bool>,
        __disable_mount_point_lookup: ::typed_builder::Optional<bool>,
        __disable_path_canonicalization: ::typed_builder::Optional<bool>,
        __do_not_update_utab: ::typed_builder::Optional<bool>,
        __dry_run: ::typed_builder::Optional<bool>,
        __force_unmount: ::typed_builder::Optional<bool>,
        __lazy_unmount: ::typed_builder::Optional<bool>,
        __on_fail_remount_read_only: ::typed_builder::Optional<bool>,
        __verbose: ::typed_builder::Optional<bool>,
        __match_file_systems: ::typed_builder::Optional<Option<String>>,
        __match_mount_options: ::typed_builder::Optional<Option<String>>,
        __source: ::typed_builder::Optional<Option<MountSource>>,
        __target: ::typed_builder::Optional<Option<PathBuf>>,
        __target_namespace: ::typed_builder::Optional<Option<PathBuf>>,
    >
    UnmountBuilder<(
        __detach_loop_device,
        __disable_helpers,
        __disable_mount_point_lookup,
        __disable_path_canonicalization,
        __do_not_update_utab,
        __dry_run,
        __force_unmount,
        __lazy_unmount,
        __on_fail_remount_read_only,
        __verbose,
        __match_file_systems,
        __match_mount_options,
        __source,
        __target,
        __target_namespace,
    )>
{
    pub fn build(self) -> Result<Unmount, UnmountBuilderError> {
        log::debug!("UnmountBuilder::build building a new `Mount` instance");

        let builder = self.__make();
        let mut unmount = Unmount::new()?;

        // Set source/target fields.
        match (builder.source, builder.target) {
            (None, Some(target)) => unmount.set_mount_target(target)?,
            (Some(source), None) => unmount.set_mount_source(source)?,
            (Some(source), Some(target)) => {
                unmount.set_mount_source(source)?;
                unmount.set_mount_target(target)?;
            }
            (None, None) => {
                // Assume `mount -a` if a source or target is not provided
                // let err_msg = "you MUST call at least one of the following functions: `UnmountBuilder::source`, `UnmountBuilder::target`".to_owned();
                // return Err(UnmountBuilderError::Required(err_msg));
            }
        }

        // Set file systems list.
        if let Some(fs_list) = builder.match_file_systems {
            unmount.set_file_systems_filter(fs_list)?;
        }

        // Set test options.
        if let Some(list) = builder.match_mount_options {
            unmount.set_mount_options_filter(list)?;
        }

        if let Some(namespace) = builder.target_namespace {
            unmount.set_mount_target_namespace(namespace)?;
        }

        if builder.detach_loop_device {
            unmount.enable_detach_loop_device()?;
        } else {
            unmount.disable_detach_loop_device()?;
        }

        if builder.disable_helpers {
            unmount.disable_helpers()?;
        } else {
            unmount.enable_helpers()?;
        }

        if builder.disable_mount_point_lookup {
            unmount.disable_mount_point_lookup()?;
        } else {
            unmount.enable_mount_point_lookup()?;
        }

        if builder.disable_path_canonicalization {
            unmount.disable_path_canonicalization()?;
        } else {
            unmount.enable_path_canonicalization()?;
        }

        if builder.do_not_update_utab {
            unmount.do_not_update_utab()?;
        } else {
            unmount.update_utab()?;
        }

        if builder.dry_run {
            unmount.enable_dry_run()?;
        } else {
            unmount.disable_dry_run()?;
        }

        if builder.force_unmount {
            unmount.enable_force_unmount()?;
        } else {
            unmount.disable_force_unmount()?;
        }

        if builder.lazy_unmount {
            unmount.enable_lazy_unmount()?;
        } else {
            unmount.disable_lazy_unmount()?;
        }

        if builder.verbose {
            unmount.enable_verbose_output()?;
        } else {
            unmount.disable_verbose_output()?;
        }

        if builder.on_fail_remount_read_only {
            unmount.enable_on_fail_remount_read_only()?;
        } else {
            unmount.disable_on_fail_remount_read_only()?;
        }

        Ok(unmount)
    }
}
