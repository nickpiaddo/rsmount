// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use typed_builder::TypedBuilder;

// From standard library
use std::path::{Path, PathBuf};

// From this library
use crate::core::entries::UTabEntry;
use crate::core::errors::UTabEntryBuilderError;

#[derive(Debug, TypedBuilder)]
#[builder(
    builder_type(
        name = UTabEntryBuilder,
        vis = "pub",
        doc ="Configure and instantiate a [`UTabEntry`].\n\nFor usage, see
[`UTabEntryBuilder::build`]."),
    build_method(vis = "", name = __make))]
pub(crate) struct UTbEntBuilder {
    #[builder(
        default,
        setter(
            transform = |source: impl AsRef<Path>| Some(source.as_ref().to_owned()),
            doc = "Sets the path to the device to mount."
        )
    )]
    source: Option<PathBuf>,

    #[builder(
        default,
        setter(
            transform = |source: impl AsRef<Path>| Some(source.as_ref().to_owned()),
            doc = "Sets the path of the bind mount source"
        )
    )]
    bind_source: Option<PathBuf>,

    #[builder(
        default,
        setter(transform = |target: impl AsRef<Path>| target.as_ref().to_path_buf(),
            doc = "Sets the location of the device mount point.")
    )]
    target: PathBuf,

    #[builder(default, setter(transform = |opts: impl AsRef<str>| Some(opts.as_ref().to_owned()),
    doc = "Sets a list of comma-separated options."))]
    mount_options: Option<String>,

    #[builder(
        default,
        setter(
            transform = |source: impl AsRef<Path>| Some(source.as_ref().to_owned()),
            doc = ""
        )
    )]
    //FIXME documentation after finding out what `ROOT=` is in /run/mount/utab
    root: Option<PathBuf>,

    #[builder(default, setter(transform = |opts: impl AsRef<str>| Some(opts.as_ref().to_owned()),
    doc = " Sets `utab` attributes, which are options independent from those used by the [`mount`
syscall](https://www.man7.org/linux/man-pages/man2/mount.2.html) and [`mount`
command](https://www.man7.org/linux/man-pages/man8/mount.8.html). They are neither sent to
the kernel, nor interpreted by `libmount`."))]
    attributes: Option<String>,
}

#[allow(non_camel_case_types)]
impl<
        __source: ::typed_builder::Optional<Option<PathBuf>>,
        __bind_source: ::typed_builder::Optional<Option<PathBuf>>,
        __target: ::typed_builder::Optional<PathBuf>,
        __mount_options: ::typed_builder::Optional<Option<String>>,
        __root: ::typed_builder::Optional<Option<PathBuf>>,
        __attributes: ::typed_builder::Optional<Option<String>>,
    >
    UTabEntryBuilder<(
        __source,
        __bind_source,
        __target,
        __mount_options,
        __root,
        __attributes,
    )>
{
    pub fn build(self) -> Result<UTabEntry, UTabEntryBuilderError> {
        log::debug!("UTabEntryBuilder::build building a new `UTabEntry` instance");

        let builder = self.__make();
        let mut entry = UTabEntry::new()?;

        match (builder.source, builder.bind_source) {
            (None, Some(path)) => {
                // Setting the file to bind mount.
                entry.set_bind_source(path)?;
            }
            (Some(path), None) => {
                // Setting the device/path to mount.
                entry.set_source_path(path)?;
            }
            (Some(_), Some(_)) => {
                let err_msg = "methods `source` and `bind_source` can not be called at the same time. You need to choose one or the other".to_owned();
                return Err(UTabEntryBuilderError::MutuallyExclusive(err_msg));
            }
            (None, None) => {
                let err_msg =
                    "you must set either the `source` or `bind_source` attribute".to_owned();

                return Err(UTabEntryBuilderError::Required(err_msg));
            }
        }

        // Setting a device's mount point.
        entry.set_target(builder.target)?;

        // Setting mount options string.
        if let Some(mount_options) = builder.mount_options {
            entry.set_mount_options(mount_options)?;
        }

        if let Some(root) = builder.root {
            entry.set_root(root)?;
        }

        if let Some(attributes) = builder.attributes {
            entry.set_attributes(attributes)?;
        }

        Ok(entry)
    }
}
