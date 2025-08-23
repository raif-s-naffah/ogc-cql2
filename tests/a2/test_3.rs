// SPDX-License-Identifier: Apache-2.0

//! Validate that CQL2 JSON is supported by the server
//! 
//! Given:
//!     * A filter expression
//! When:
//!     Execute conformance tests for all supported conformance classes with
//!     the parameter "Filter Language". Use the value "CQL2 JSON". Note that
//!     the filter expressions in the test cases have to be converted to a CQL2
//!     JSON representation.
//! Then:
//! * assert the validation is successful.
//! 

use crate::grammar;
use std::error::Error;

/// Validate that CQL2 JSON is supported by the server.
#[test]
fn test() -> Result<(), Box<dyn Error>> {
    grammar::test_json_samples()
}
