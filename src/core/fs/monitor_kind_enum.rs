// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use enum_iterator::Sequence;
use num_enum::TryFromPrimitive;

// From standard library

// From this library

/// Types of file monitors.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence, TryFromPrimitive)]
#[repr(u32)]
#[non_exhaustive]
pub enum MonitorKind {
    Kernel = libmount::MNT_MONITOR_TYPE_KERNEL,
    Userspace = libmount::MNT_MONITOR_TYPE_USERSPACE,
}
