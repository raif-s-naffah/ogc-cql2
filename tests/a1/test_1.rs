// SPDX-License-Identifier: Apache-2.0

//! Validate that CQL2 Text is supported by the server
//!
//! When:
//!     Execute conformance tests for all supported conformance classes with
//!     the parameter "Filter Language". Use the value "CQL2 Text".
//! Then:
//! * assert that all conformance tests are successful.
//!

use crate::grammar;
use std::error::Error;

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    grammar::test_text_samples()
}
