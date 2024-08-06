// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use enum_iterator::Sequence;
use num_enum::{IntoPrimitive, TryFromPrimitive};

// From standard library

// From this library

/// Comparisons performed when diffing tables.
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive, Sequence)]
#[repr(i32)]
#[non_exhaustive]
pub enum Comparison {
    Mount = libmount::MNT_TABDIFF_MOUNT as i32,
    Move = libmount::MNT_TABDIFF_MOVE as i32,
    Propagation = libmount::MNT_TABDIFF_PROPAGATION as i32,
    ReMount = libmount::MNT_TABDIFF_REMOUNT as i32,
    UnMount = libmount::MNT_TABDIFF_UMOUNT as i32,
}
