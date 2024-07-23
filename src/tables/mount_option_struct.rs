// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::errors::ParserError;

/// A device mount option.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::tables::MountOption;
///
/// fn main() -> rsmount::Result<()> {
///     let option = "ro";
///     let actual: MountOption = option.parse()?;
///     assert_eq!(actual.name(), "ro");
///     assert_eq!(actual.value(), None);
///
///     let option = "ro=recursive";
///     let actual: MountOption = option.parse()?;
///     assert_eq!(actual.name(), "ro");
///     assert_eq!(actual.value(), Some("recursive"));
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MountOption {
    name: String,
    value: Option<String>,
}

impl MountOption {
    #[doc(hidden)]
    /// Creates a new `MountOption`.
    pub(crate) fn new(name: &str) -> MountOption {
        log::debug!(
            "MountOption::new creating a new `MountOption` instance with name: {:?} and no value",
            name,
        );

        let name = name.trim().to_owned();

        Self { name, value: None }
    }

    #[doc(hidden)]
    /// Creates a new `MountOption`.
    pub(crate) fn new_with_value(name: &str, value: &str) -> MountOption {
        log::debug!(
            "MountOption::new creating a new `MountOption` instance with name: {:?} and value: {:?}",
            name,
            value
        );

        let name = name.trim().to_owned();
        let value = value.trim().to_owned();

        Self {
            name,
            value: Some(value),
        }
    }

    /// Returns a `MountOption`'s name.
    pub fn name(&self) -> &str {
        log::debug!("MountOption::name value: {:?}", self.name);

        &self.name
    }

    /// Returns a `MountOption`'s value.
    pub fn value(&self) -> Option<&str> {
        log::debug!("MountOption::value value: {:?}", self.value);

        self.value.as_deref()
    }
}

impl AsRef<MountOption> for MountOption {
    #[inline]
    fn as_ref(&self) -> &MountOption {
        self
    }
}

impl FromStr for MountOption {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('=') {
            // Mount option with value
            Some((option_name, value)) => {
                let option_name = option_name.trim();

                let value = value.trim();

                // Remove opening opening/closing quotes/double-quotes if present
                let option_value = if value.starts_with('"') {
                    let err_msg =
                        format!("missing closing double-quote in option value: {:?}", value);
                    value
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'))
                        .ok_or(ParserError::MountOption(err_msg))
                } else if value.starts_with('\'') {
                    let err_msg = format!("missing closing quote in option value: {:?}", value);
                    value
                        .strip_prefix('\'')
                        .and_then(|s| s.strip_suffix('\''))
                        .ok_or(ParserError::MountOption(err_msg))
                } else {
                    Ok(value)
                }?;

                let option = Self::new_with_value(option_name, option_value);
                Ok(option)
            }
            // Mount option without value
            None => {
                let option = Self::new(s);

                Ok(option)
            }
        }
    }
}

impl fmt::Display for MountOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self.value() {
            Some(value) => {
                if value
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_ascii_punctuation())
                {
                    // Put value between quotes if it contains special characters.
                    format!("{}=\"{}\"", self.name(), value)
                } else {
                    format!("{}={}", self.name(), value)
                }
            }
            None => self.name().to_owned(),
        };

        write!(f, "{output}")
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    #[should_panic(expected = "missing closing double-quote")]
    fn mount_option_can_not_parse_a_mount_option_string_with_an_unclosed_double_quote() {
        let _: MountOption = r#"ro="resursive"#.parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "missing closing quote")]
    fn mount_option_can_not_parse_a_mount_option_string_with_an_unclosed_quote() {
        let _: MountOption = "ro='recursive".parse().unwrap();
    }

    #[test]
    fn mount_option_can_parse_a_mount_option() -> crate::Result<()> {
        let option = "ro";

        let actual: MountOption = option.parse()?;
        let expected_name = "ro";
        let expected_value = None;
        assert_eq!(actual.name(), expected_name);
        assert_eq!(actual.value(), expected_value);

        let option = "ro=recursive";

        let actual: MountOption = option.parse()?;
        let expected_name = "ro";
        let expected_value = Some("recursive");
        assert_eq!(actual.name(), expected_name);
        assert_eq!(actual.value(), expected_value);

        Ok(())
    }
}
