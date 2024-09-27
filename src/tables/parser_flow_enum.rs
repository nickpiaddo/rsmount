// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library
use num_enum::IntoPrimitive;

// From standard library

// From this library

/// Used to tell a parser whether it should exit early, ignore a parsing error or go on as usual.
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum ParserFlow {
    Abort = -1,
    Continue = 0,
    Ignore = 1,
}
