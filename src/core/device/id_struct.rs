// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::errors::ParserError;

/// A udev device ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id(String);

impl Id {
    /// View this `Id` as a UTF-8 `str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = &self.0;

        write!(f, "{id}")
    }
}

impl FromStr for Id {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Remove opening opening/closing quotes/double-quotes if present
        let err_missing_dquote = format!("missing closing double-quote in ID: {:?}", s);
        let err_missing_quote = format!("missing closing quote in ID: {:?}", s);

        let trimmed = s.trim();
        let parsed = if trimmed.starts_with('"') {
            trimmed
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .ok_or(ParserError::Label(err_missing_dquote))
        } else if trimmed.starts_with('\'') {
            trimmed
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
                .ok_or(ParserError::Label(err_missing_quote))
        } else {
            Ok(trimmed)
        }?;

        let id = Self(parsed.trim().to_owned());

        Ok(id)
    }
}
