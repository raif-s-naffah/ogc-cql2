// SPDX-License-Identifier: Apache-2.0

//! Annex A: Abstract Test Suite (Normative)
//!
//! https://docs.ogc.org/is/21-065r2/21-065r2.html#ats
//!

// Common test harnesses, utility functions and test data Resource structures.
mod utils;

/// Simple runners that ingest both test- and JSON-encoded 'sample' files
/// included in the specifications to check that they yield `Ok` Expression
/// results; i.e. successfully parse the input.
mod grammar;

/// A.1. Conformance Class "CQL2 Text"
mod a1;

/// A.2. Conformance Class "CQL2 JSON"
mod a2;

/// A.3. Conformance Class "Basic-CQL2"
mod a3;

/// A.4. Conformance Class "Advanced Comparison Operators"
mod a4;

/// A.5. Conformance Class "Case-insensitive Comparison"
mod a5;

/// A.6. Conformance Class "Accent-insensitive Comparison"
mod a6;

/// A.7. Conformance Class "Basic Spatial Functions"
mod a7;

/// A.8. Conformance Class "Basic Spatial Functions with additional Spatial Literals"
mod a8;

/// A.9. Conformance Class "Spatial Functions"
mod a9;

/// A.10. Conformance Class "Temporal Functions"
mod a10;

/// A.11. Conformance Class "Array Functions"
mod a11;

/// A.12. Conformance Class "Property-Property Comparisons"
mod a12;

/// A.13. Conformance Class "Functions"
mod a13;

/// A.14. Conformance Class "Arithmetic Expressions"
mod a14;
