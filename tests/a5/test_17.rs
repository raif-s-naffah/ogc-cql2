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
const PREDICATES: [(&str, u32); 10] = [
    (r#"CASEI(name)=casei('KIEV')"#,      1),
    (r#"CASEI(name)=casei('kiev')"#,      1),
    (r#"CASEI(name)=casei('Kiev')"#,      1),
    (r#"CASEI(name)=casei('København')"#, 1),
    (r#"CASEI(name)=casei('københavn')"#, 1),
    (r#"CASEI(name)=casei('KØBENHAVN')"#, 1),
    (r#"CASEI(name) LIKE casei('B_r%')"#, 3),
    (r#"CASEI(name) LIKE casei('b_r%')"#, 3),
    (r#"CASEI(name) LIKE casei('B_R%')"#, 3),
    (r#"CASEI(name) IN (casei('Kiev'),
        casei('kobenhavn'), casei('Berlin'), 
        casei('athens'), casei('foo'))"#, 3)
];

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    harness(PLACES, PREDICATES.to_vec())
}
