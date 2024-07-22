// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use typed_builder::TypedBuilder;

// From standard library
use std::path::{Path, PathBuf};

// From this library
use crate::core::device::Source;
use crate::core::entries::FsTabEntry;
use crate::core::errors::FsTabEntryBuilderError;
use crate::core::fs::FileSystem;

#[derive(Debug, TypedBuilder)]
#[builder(
    builder_type(
        name = FsTabEntryBuilder,
        vis = "pub",
        doc ="Configure and instantiate a [`FsTabEntry`].\n\nFor usage, see
[`FsTabEntryBuilder::build`]."),
    build_method(vis = "", name = __make))]
pub(crate) struct FsTbEntBuilder {
    #[builder(default, setter(transform = |line: impl AsRef<str>| Some(line.as_ref().to_owned()),
    doc = "Sets a comment line.\n **Note:** this method adds a newline character at the end of the
parameter `line` if it is not present."))]
    comment_line: Option<String>,

    #[builder(setter(doc = "Sets the device to mount."))]
    source: Source,

    #[builder(
        default,
        setter(transform = |target: impl AsRef<Path>| target.as_ref().to_path_buf(),
            doc = "Sets the location of the device mount point.")
    )]
    target: PathBuf,

    #[builder(
        default,
        setter(
            strip_option,
            doc = "Sets the file system type of the device to mount."
        )
    )]
    file_system_type: Option<FileSystem>,

    #[builder(default, setter(transform = |opts: impl AsRef<str>| Some(opts.as_ref().to_owned()),
    doc = "Sets a list of comma-separated options."))]
    mount_options: Option<String>,
    #[builder(
        default,
        setter(
            strip_option,
            doc = "Sets the interval, in days, between file system backups by the `dump` command on
ext2/3/4 file systems. (see the [`dump` command's manpage](https://manpages.org/dump/8))"
        )
    )]
    backup_frequency: Option<i32>,

    #[builder(
        default,
        setter(
            strip_option,
            doc = "Sets the order in which file systems are checked by the `fsck` command. Setting this
value to `0` will direct `fsck` to skip the device referenced in this [`FsTabEntry`]."
        )
    )]
    fsck_checking_order: Option<i32>,
}

#[allow(non_camel_case_types)]
impl<
        __comment_line: ::typed_builder::Optional<Option<String>>,
        __target: ::typed_builder::Optional<PathBuf>,
        __file_system_type: ::typed_builder::Optional<Option<FileSystem>>,
        __mount_options: ::typed_builder::Optional<Option<String>>,
        __backup_frequency: ::typed_builder::Optional<Option<i32>>,
        __fsck_checking_order: ::typed_builder::Optional<Option<i32>>,
    >
    FsTabEntryBuilder<(
        __comment_line,
        (Source,),
        __target,
        __file_system_type,
        __mount_options,
        __backup_frequency,
        __fsck_checking_order,
    )>
{
    pub fn build(self) -> Result<FsTabEntry, FsTabEntryBuilderError> {
        log::debug!("FsTabEntryBuilder::build building a new `FsTabEntry` instance");

        let builder = self.__make();
        let mut entry = FsTabEntry::new()?;

        // Setting the device/path to mount.
        entry.set_mount_source(builder.source.to_string())?;

        // Setting a device's mount point.
        entry.set_mount_target(builder.target)?;

        // Setting comment line.
        if let Some(line) = builder.comment_line {
            entry.set_comment(line)?;
        }

        // Setting interval between file system backups.
        if let Some(backup_frequency) = builder.backup_frequency {
            entry.set_backup_frequency(backup_frequency)?;
        }

        // Setting file_system_type.
        if let Some(file_system_type) = builder.file_system_type {
            entry.set_file_system_type(file_system_type)?;
        }

        // Setting mount options string.
        if let Some(mount_options) = builder.mount_options {
            entry.set_mount_options(mount_options)?;
        }

        // Setting the order in which file systems are checked by the `fsck` command.
        if let Some(fsck_checking_order) = builder.fsck_checking_order {
            entry.set_fsck_checking_order(fsck_checking_order)?;
        }

        Ok(entry)
    }
}
