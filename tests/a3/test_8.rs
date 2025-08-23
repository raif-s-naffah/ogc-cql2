// SPDX-License-Identifier: Apache-2.0

//! Test predicates against the test dataset
//!
//! Given:
//! * The implementation under test uses the test dataset.
//!
//! When:
//! Evaluate each predicate in Predicates and expected results.
//!
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the expected result is returned;
//! * store the valid predicates for each data source.
//!

use crate::utils::{COUNTRIES, PLACES, harness};
use std::error::Error;
use tracing_test::traced_test;

#[rustfmt::skip]
const COUNTRIES_PREDICATES: [(&str, u32); 12] = [
    (r#"NAME='Luxembourg'"#,    1),
    (r#"NAME>='Luxembourg'"#,  84),
    (r#"NAME>'Luxembourg'"#,   83),
    (r#"NAME<='Luxembourg'"#,  94),
    (r#"NAME<'Luxembourg'"#,   93),
    (r#"NAME<>'Luxembourg'"#, 176),
    // -----
    (r#"POP_EST=37589262"#,    1),
    (r#"POP_EST>=37589262"#,  39),
    (r#"POP_EST>37589262"#,   38),
    (r#"POP_EST<=37589262"#, 139),
    (r#"POP_EST<37589262"#,  138),
    (r#"POP_EST<>37589262"#, 176),
];

#[rustfmt::skip]
const PLACES_PREDICATES: [(&str, u32); 36] = [
    (r#"name IS NOT NULL"#,  243),
    (r#"name IS NULL"#,        0),
    (r#"name='København'"#,    1),
    (r#"name>='København'"#, 137),
    (r#"name>'København'"#,  136),
    (r#"name<='København'"#, 107),
    (r#"name<'København'"#,  106),
    (r#"name<>'København'"#, 242),
    // -----
    (r#"pop_other IS NOT NULL"#, 243),
    (r#"pop_other IS NULL"#,       0),
    (r#"pop_other=1038288"#,       1),
    (r#"pop_other>=1038288"#,    123),
    (r#"pop_other>1038288"#,     122),
    (r#"pop_other<=1038288"#,    121),
    (r#"pop_other<1038288"#,     120),
    (r#"pop_other<>1038288"#,    242),
    // -----
    (r#""date" IS NOT NULL"#,         3),
    (r#""date" IS NULL"#,           240),
    (r#""date"=DATE('2022-04-16')"#,  1),
    (r#""date">=DATE('2022-04-16')"#, 2),
    (r#""date">DATE('2022-04-16')"#,  1),
    (r#""date"<=DATE('2022-04-16')"#, 2),
    (r#""date"<DATE('2022-04-16')"#,  1),
    (r#""date"<>DATE('2022-04-16')"#, 2),
    // -----
    (r#"start IS NOT NULL"#,                        3),
    (r#"start IS NULL"#,                          240),
    (r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  1),
    (r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
    (r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  1),
    (r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
    (r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  1),
    (r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
    // -----
    (r#"boolean IS NOT NULL"#, 3),
    (r#"boolean IS NULL"#,   240),
    (r#"boolean=true"#,        2),
    (r#"boolean=false"#,       1),
];

#[test]
#[traced_test]
fn test_countries() -> Result<(), Box<dyn Error>> {
    harness(COUNTRIES, COUNTRIES_PREDICATES.to_vec())
}

#[test]
#[traced_test]
fn test_places() -> Result<(), Box<dyn Error>> {
    harness(PLACES, PLACES_PREDICATES.to_vec())
}
