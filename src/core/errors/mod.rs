// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Runtime errors.

// From dependency library

// From standard library

// From this library
pub use cache_error_enum::CacheError;
pub use parser_error_enum::ParserError;

mod cache_error_enum;
mod parser_error_enum;
