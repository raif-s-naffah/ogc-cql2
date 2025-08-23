// SPDX-License-Identifier: Apache-2.0

//! Test predicates against the test dataset
//!
//! Given:
//!     * The implementation under test uses the test dataset.
//! When:
//!     Evaluate each predicate in Predicates and expected results, if the
//!     conditional dependency is met.
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the expected result is returned;
//! * store the valid predicates for each data source.
//!

use crate::utils::{PLACES, harness};
use std::error::Error;
use tracing_test::traced_test;

#[rustfmt::skip]
const PREDICATES: [(&str, u32); 11] = [
    (r#"ACCENTI(name)=accenti('Chișinău')"#,                 1),  // 0
    (r#"ACCENTI(name)=accenti('Chisinau')"#,                 1),
    (r#"ACCENTI(name)=accenti('Kiev')"#,                     1),
    (r#"ACCENTI(CASEI(name))=accenti(casei('chișinău'))"#,   1),
    (r#"ACCENTI(CASEI(name))=accenti(casei('chisinau'))"#,   1),
    (r#"ACCENTI(CASEI(name))=accenti(casei('CHISINAU'))"#,   1),  // 5
    (r#"ACCENTI(CASEI(name))=accenti(casei('CHIȘINĂU'))"#,   1),

    // IMPORTANT: see https://github.com/opengeospatial/ogcapi-features/issues/1011
    (r#"ACCENTI(name) LIKE accenti('Ch%')"#,                 3),  // 7; was 2
    (r#"ACCENTI(CASEI(name)) LIKE accenti(casei('Chiș%'))"#, 1),  // 8; likewise
    (r#"ACCENTI(CASEI(name)) LIKE accenti(casei('cHis%'))"#, 1),  // 9; likewise

    (r#"ACCENTI(CASEI(name)) IN (accenti(casei('Kiev')),
     accenti(casei('chișinău')), accenti(casei('Berlin')),
     accenti(casei('athens')), accenti(casei('foo')))"#,     4)  // 10
];

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    harness(PLACES, PREDICATES.to_vec())
}
