// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::device::Id;
use crate::core::device::Label;
use crate::core::device::TagName;
use crate::core::device::Uuid;
use crate::core::errors::ParserError;

/// A device tag.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Tag {
    /// Human readable filesystem identifier.
    Label(Label),
    /// Filesystem universally unique identifier.
    Uuid(Uuid),
    /// Human readable partition identifier.
    PartLabel(Label),
    /// Partition universally unique identifier.
    PartUuid(Uuid),
    /// Hardware block device ID as generated by `udevd`.
    Id(Id),
}

impl Tag {
    /// Returns a `Tag`'s name.
    pub fn name(&self) -> TagName {
        TagName::from(self)
    }

    /// Returns a `Tag`'s value.
    pub fn value(&self) -> &str {
        match self {
            Self::Label(value) => value.as_str(),
            Self::Uuid(value) => value.as_str(),
            Self::PartLabel(value) => value.as_str(),
            Self::PartUuid(value) => value.as_str(),
            Self::Id(value) => value.as_str(),
        }
    }

    /// Returns `true` if this `Tag` represents a `LABEL`.
    pub fn is_label(&self) -> bool {
        matches!(self, Self::Label(_))
    }

    /// Returns `true` if this `Tag` represents a `UUID`.
    pub fn is_uuid(&self) -> bool {
        matches!(self, Self::Uuid(_))
    }

    /// Returns `true` if this `Tag` represents a `PARTLABEL`.
    pub fn is_partition_label(&self) -> bool {
        matches!(self, Self::PartLabel(_))
    }

    /// Returns `true` if this `Tag` represents a `PARTUUID`.
    pub fn is_partition_uuid(&self) -> bool {
        matches!(self, Self::PartUuid(_))
    }

    /// Returns `true` if this `Tag` represents a udev device `ID`.
    pub fn is_id(&self) -> bool {
        matches!(self, Self::Id(_))
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Label(value) => value.to_string(),
            Self::Uuid(value) => value.to_string(),
            Self::PartLabel(value) => value.to_string(),
            Self::PartUuid(value) => value.to_string(),
            Self::Id(value) => value.to_string(),
        };

        write!(f, "{}={}", self.name(), value)
    }
}

impl AsRef<Tag> for Tag {
    #[inline]
    fn as_ref(&self) -> &Tag {
        self
    }
}

impl TryFrom<&str> for Tag {
    type Error = ParserError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let err_msg = format!("invalid tag: {:?}. Missing `=` sign", s);
        let (tag_name, value) = s.split_once('=').ok_or(ParserError::Tag(err_msg))?;

        let tag_name = TagName::from_str(tag_name)?;
        match tag_name {
            TagName::Label => Label::from_str(value).map(Self::Label),
            TagName::PartLabel => Label::from_str(value).map(Self::PartLabel),
            TagName::Uuid => Uuid::from_str(value).map(Self::Uuid),
            TagName::PartUuid => Uuid::from_str(value).map(Self::PartUuid),
            TagName::Id => Id::from_str(value).map(Self::Id),
        }
    }
}

impl TryFrom<String> for Tag {
    type Error = ParserError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl TryFrom<&String> for Tag {
    type Error = ParserError;

    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl FromStr for Tag {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}
