// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::fmt;
use std::str::FromStr;

// From this library
use crate::core::errors::ParserError;

/// Address of an SSHFS share.
///
/// # Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use rsmount::core::device::SshFs;
///
/// fn main() -> rsmount::Result<()> {
///    let user_name = "tux";
///    let host = "sshfs.server.internal";
///    let share = "/shared";
///
///    // tux@sshfs.server.internal:/shared
///    let address = format!("{user_name}@{host}:{share}");
///    let sshfs: SshFs = address.parse()?;
///
///    assert_eq!(sshfs.user_name(), Some(user_name));
///    assert_eq!(sshfs.host(), host);
///    assert_eq!(sshfs.share(), Some(share));
///
///    // sshfs.server.internal:/shared
///    let address = format!("{host}:{share}");
///    let sshfs: SshFs = address.parse()?;
///
///    assert_eq!(sshfs.user_name(), None);
///    assert_eq!(sshfs.host(), host);
///    assert_eq!(sshfs.share(), Some(share));
///
///    // sshfs.server.internal:
///    let address = format!("{host}:");
///    let sshfs: SshFs = address.parse()?;
///
///    assert_eq!(sshfs.user_name(), None);
///    assert_eq!(sshfs.host(), host);
///    assert_eq!(sshfs.share(), None);
///
///    Ok(())
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct SshFs {
    host: String,
    share: Option<String>,
    user_name: Option<String>,
}

impl SshFs {
    #[doc(hidden)]
    /// Creates a new `SshFs`.
    pub(crate) fn new<T>(user_name: T, host: T, share: T) -> SshFs
    where
        T: AsRef<str>,
    {
        let host = host.as_ref().to_owned();
        let share = share.as_ref();
        let user_name = user_name.as_ref();

        let share = if share.is_empty() {
            None
        } else {
            Some(share.to_owned())
        };

        let user_name = if user_name.is_empty() {
            None
        } else {
            Some(user_name.to_owned())
        };

        Self {
            host,
            share,
            user_name,
        }
    }

    /// Address of the host server.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Name of the shared file/directory.
    pub fn share(&self) -> Option<&str> {
        self.share.as_deref()
    }

    /// User name.
    pub fn user_name(&self) -> Option<&str> {
        self.user_name.as_deref()
    }
}

impl AsRef<SshFs> for SshFs {
    #[inline]
    fn as_ref(&self) -> &SshFs {
        self
    }
}

impl fmt::Display for SshFs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.user_name(), self.share()) {
            (None, None) => write!(f, "{}:", self.host),
            (None, Some(share)) => write!(f, "{}:{}", self.host, share),
            (Some(user_name), None) => write!(f, "{}@{}:", user_name, self.host),
            (Some(user_name), Some(share)) => write!(f, "{}@{}:{}", user_name, self.host, share),
        }
    }
}

impl FromStr for SshFs {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err_missing_column = format!("invalid SshFs address: {}. Missing ':' delimiter", s);

        // Split [user@]host:[dir]
        let parsed = s
            .trim()
            .rsplit_once(':')
            .ok_or(ParserError::SshFs(err_missing_column))?;

        match parsed {
            ("", _) => {
                let err_msg = format!("invalid SshFs address: {}. Missing host name", s);
                Err(ParserError::SshFs(err_msg))
            }
            // Split [user@]host
            (host, share) => match host.split_once('@') {
                None => Ok(SshFs::new("", host, share)),
                Some(("", _)) => {
                    let err_msg = format!("invalid SshFs address: {}. Missing user name", s);

                    Err(ParserError::SshFs(err_msg))
                }
                Some((user_name, host)) => Ok(SshFs::new(user_name, host, share)),
            },
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    #[should_panic(expected = "Missing ':'")]
    fn sshfs_share_can_not_parse_an_empty_string() {
        let _: SshFs = "".parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing host name")]
    fn sshfs_share_can_not_parse_an_address_with_an_empty_hostname_and_path() {
        let _: SshFs = ":".parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing host name")]
    fn sshfs_share_can_not_parse_an_address_with_an_empty_hostname() {
        let _: SshFs = ":/share".parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing user name")]
    fn sshfs_share_can_not_parse_an_address_with_an_empty_username_before_the_at_separator() {
        let _: SshFs = "@localhost:/share".parse().unwrap();
    }

    #[test]
    fn sshfs_share_can_parse_an_adress_with_the_hostname_only() -> crate::Result<()> {
        let address = "localhost:";
        let actual: SshFs = address.parse()?;
        let expected_host = "localhost";
        let expected_share = None;
        let expected_user_name = None;

        assert_eq!(actual.host(), expected_host);
        assert_eq!(actual.share(), expected_share);
        assert_eq!(actual.user_name(), expected_user_name);
        assert_eq!(&actual.to_string(), address);

        Ok(())
    }

    #[test]
    fn sshfs_share_can_parse_an_adress_sharing_a_directory() -> crate::Result<()> {
        let address = "localhost:/share";
        let actual: SshFs = address.parse()?;
        let expected_host = "localhost";
        let expected_share = Some("/share");
        let expected_user_name = None;

        assert_eq!(actual.host(), expected_host);
        assert_eq!(actual.share(), expected_share);
        assert_eq!(actual.user_name(), expected_user_name);
        assert_eq!(&actual.to_string(), address);

        Ok(())
    }

    #[test]
    fn sshfs_share_can_parse_an_adress_sharing_a_directory_with_a_username() -> crate::Result<()> {
        let address = "user@localhost:/share";
        let actual: SshFs = address.parse()?;
        let expected_host = "localhost";
        let expected_share = Some("/share");
        let expected_user_name = Some("user");

        assert_eq!(actual.host(), expected_host);
        assert_eq!(actual.share(), expected_share);
        assert_eq!(actual.user_name(), expected_user_name);
        assert_eq!(&actual.to_string(), address);

        Ok(())
    }
}
