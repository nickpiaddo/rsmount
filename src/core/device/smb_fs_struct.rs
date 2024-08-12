// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::errors::ParserError;

/// Address of a Samba share.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::device::SmbFs;
///
/// fn main() -> rsmount::Result<()> {
///    let host = "samba.server.internal";
///    let share = "/shared";
///
///    // smb://samba.server.internal/shared
///    let address = format!("smb://{host}{share}");
///    let smbfs: SmbFs = address.parse()?;
///
///    assert_eq!(smbfs.host(), host);
///    assert_eq!(smbfs.share(), share);
///
///    Ok(())
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct SmbFs {
    host: String,
    share: String,
}

impl SmbFs {
    #[doc(hidden)]
    /// Creates a new `SmbFs`.
    pub(crate) fn new<T>(host: T, share: T) -> SmbFs
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

impl AsRef<SmbFs> for SmbFs {
    #[inline]
    fn as_ref(&self) -> &SmbFs {
        self
    }
}

impl fmt::Display for SmbFs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "smb://{}{}", self.host, self.share)
    }
}

impl FromStr for SmbFs {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err_missing_prefix = format!(
            "invalid Samba share address: {}. Missing prefix `smb://`",
            s
        );
        let err_missing_dir = format!(
            "invalid Samba share address: {}. Missing host name and/or path of shared file/directory",
            s
        );

        let prefix = "smb://";
        let parsed = s
            .trim()
            .strip_prefix(prefix)
            .ok_or(ParserError::SmbFs(err_missing_prefix))
            .and_then(|stripped| {
                stripped
                    .split_once('/')
                    .ok_or(ParserError::SmbFs(err_missing_dir))
            })?;

        match parsed {
            ("", _) => {
                let err_msg = format!("invalid Samba share address: {}. Missing host name", s);

                Err(ParserError::SmbFs(err_msg))
            }
            (host, share) => {
                // Replace the `/` consumed during string splitting.
                let share = format!("/{}", share);

                let share = SmbFs::new(host, &share);

                Ok(share)
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
    #[should_panic(expected = "Missing prefix")]
    fn samba_share_can_not_parse_an_empty_string() {
        let _: SmbFs = "".parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing host name")]
    fn samba_share_can_not_parse_an_address_without_a_hostname() {
        let _: SmbFs = "smb:///".parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing host name and/or path")]
    fn samba_share_can_not_parse_an_address_without_a_hostname_and_share_dir() {
        let _: SmbFs = "smb://".parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing host name and/or path")]
    fn samba_share_can_not_parse_an_address_without_share_dir() {
        let _: SmbFs = "smb://localhost".parse().unwrap();
    }

    #[test]
    fn samba_share_can_parse_an_adress_sharing_the_root_directory() -> crate::Result<()> {
        let address = "smb://localhost/";
        let actual: SmbFs = address.parse()?;
        let expected_host = "localhost";
        let expected_share = "/";

        assert_eq!(actual.host(), expected_host);
        assert_eq!(actual.share(), expected_share);
        assert_eq!(&actual.to_string(), address);

        Ok(())
    }

    #[test]
    fn samba_share_can_parse_an_adress_sharing_a_directory() -> crate::Result<()> {
        let address = "smb://localhost/share";
        let actual: SmbFs = address.parse()?;
        let expected_host = "localhost";
        let expected_share = "/share";

        assert_eq!(actual.host(), expected_host);
        assert_eq!(actual.share(), expected_share);
        assert_eq!(&actual.to_string(), address);

        Ok(())
    }
}
