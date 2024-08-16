// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::errors::ParserError;

/// A device label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label(String);

impl Label {
    /// View this `Label` as a UTF-8 `str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Put label between quotes if it contains special characters.
        let label = if self
            .0
            .chars()
            .any(|c| c.is_whitespace() || c.is_ascii_punctuation())
        {
            format!("\"{}\"", self.0)
        } else {
            self.0.to_owned()
        };

        write!(f, "{label}")
    }
}

impl FromStr for Label {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Remove opening opening/closing quotes/double-quotes if present
        let err_missing_dquote = format!("missing closing double-quote in label: {:?}", s);
        let err_missing_quote = format!("missing closing quote in label: {:?}", s);

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

        let label = Self(parsed.to_owned());

        Ok(label)
    }
}
