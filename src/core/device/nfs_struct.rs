// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::errors::ParserError;

/// Address of an NFS share.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::device::NFS;
///
/// fn main() -> rsmount::Result<()> {
///    let host = "nfs.server.internal";
///    let share = "/shared";
///
///    // nfs.server.internal:/shared
///    let address = format!("{host}:{share}");
///    let nfs = NFS::try_from(address)?;
///
///    assert_eq!(nfs.host(), host);
///    assert_eq!(nfs.share(), share);
///
///    Ok(())
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct NFS {
    host: String,
    share: String,
}

impl NFS {
    #[doc(hidden)]
    /// Creates a new `NFS`.
    pub(crate) fn new<T>(host: T, share: T) -> NFS
    where
        T: AsRef<str>,
    {
        let host = host.as_ref().to_owned();
        let share = share.as_ref().to_owned();

        Self { host, share }
    }

    /// Address of the host server.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Name of the shared file/directory.
    pub fn share(&self) -> &str {
        &self.share
    }
}

impl AsRef<NFS> for NFS {
    #[inline]
    fn as_ref(&self) -> &NFS {
        self
    }
}

impl fmt::Display for NFS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.host(), self.share())
    }
}

impl TryFrom<&str> for NFS {
    type Error = ParserError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let err_missing_host_and_path = format!(
            "invalid NFS share address: {}. Missing host name and path of shared file/directory",
            s
        );

        let parsed = s
            .trim()
            .rsplit_once(':')
            .ok_or(ParserError::NFS(err_missing_host_and_path.clone()))?;

        match parsed {
            ("", "") => Err(ParserError::NFS(err_missing_host_and_path)),
            ("", _) => {
                let err_msg = format!("invalid NFS share address: {}. Missing host name", s);

                Err(ParserError::NFS(err_msg))
            }
            (_, "") => {
                let err_msg = format!(
                    "invalid NFS share address: {}. Missing path of shared file/directory",
                    s
                );

                Err(ParserError::NFS(err_msg))
            }
            (host, share) => {
                let share = NFS::new(host.trim(), share.trim());

                Ok(share)
            }
        }
    }
}

impl TryFrom<String> for NFS {
    type Error = ParserError;

    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl TryFrom<&String> for NFS {
    type Error = ParserError;

    #[inline]
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl FromStr for NFS {
    type Err = ParserError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    #[should_panic(expected = "Missing host name and path")]
    fn nfs_share_can_not_parse_an_empty_string() {
        let _: NFS = "".parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing host name and path")]
    fn nfs_share_can_not_parse_an_address_without_a_hostname_and_a_path() {
        let _: NFS = ":".parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing host name")]
    fn nfs_share_can_not_parse_an_address_without_a_hostname() {
        let _: NFS = ":/share".parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing path")]
    fn nfs_share_can_not_parse_an_address_without_a_path() {
        let _: NFS = "localhost:".parse().unwrap();
    }

    #[test]
    fn nfs_share_can_parse_an_adress_sharing_a_directory() -> crate::Result<()> {
        let address = "localhost:/share";
        let actual: NFS = address.parse()?;
        let expected_host = "localhost";
        let expected_share = "/share";

        assert_eq!(actual.host(), expected_host);
        assert_eq!(actual.share(), expected_share);
        assert_eq!(&actual.to_string(), address);

        Ok(())
    }
}
