// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

/// Status of a [`TableMonitor`](crate::tables::TableMonitor).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MonitorStatus {
    ChangeDetected,
    TimeOut,
    Error,
}
