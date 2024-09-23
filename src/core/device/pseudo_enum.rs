// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::errors::ParserError;

/// Source of pseudo file systems.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Pseudo {
    None,
}

impl Pseudo {
    /// View this `Pseudo` as a UTF-8 `str`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
        }
    }
}

impl AsRef<Pseudo> for Pseudo {
    #[inline]
    fn as_ref(&self) -> &Pseudo {
        self
    }
}

impl fmt::Display for Pseudo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Pseudo {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "none" => Ok(Self::None),
            _ => {
                let err_msg = format!("unsupported pseudofs source: {:?}", s);
                Err(ParserError::Pseudo(err_msg))
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    #[should_panic(expected = "unsupported pseudofs source")]
    fn pseudo_can_not_parse_an_empty_string() {
        let _: Pseudo = "".parse().unwrap();
    }

    #[test]
    fn pseudo_can_parse_string_none() -> crate::Result<()> {
        let actual: Pseudo = "none".parse()?;
        let expected = Pseudo::None;
        assert_eq!(actual, expected);

        Ok(())
    }
}
