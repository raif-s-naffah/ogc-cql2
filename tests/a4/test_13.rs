// SPDX-License-Identifier: Apache-2.0

//! Test predicates against the test dataset
//!
//! Given:
//!     * The implementation under test uses the test dataset.
//! When:
//!     Evaluate each predicate in Predicates and expected results.
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the expected result is returned;
//! * store the valid predicates for each data source.
//!

use crate::utils::{PlaceCSV, PlaceGPkg, PlacePG, harness, harness_gpkg, harness_sql};
use std::error::Error;

#[rustfmt::skip]
const PREDICATES: [(&str, u32); 14] = [
    (r#"name LIKE 'B_r%'"#,                                                         3),
    (r#"name NOT LIKE 'B_r%'"#,                                                   240),
    (r#"pop_other between 1000000 and 3000000"#,                                   75),
    (r#"pop_other not between 1000000 and 3000000"#,                              168),
    (r#"name IN ('Kiev','kobenhavn','Berlin','athens','foo')"#,                     2),
    (r#"name NOT IN ('Kiev','kobenhavn','Berlin','athens','foo')"#,               241),
    (r#"pop_other in (1038288,1611692,3013258,3013257,3013259)"#,                   3),
    (r#"pop_other not in (1038288,1611692,3013258,3013257,3013259)"#,             240),
    (r#""date" in (DATE('2021-04-16'),DATE('2022-04-16'),DATE('2022-04-18'))"#,     2),
    (r#""date" not in (DATE('2021-04-16'),DATE('2022-04-16'),DATE('2022-04-18'))"#, 1),
    (r#"start in (TIMESTAMP('2022-04-16T10:13:19Z'))"#,                             1),
    (r#"start not in (TIMESTAMP('2022-04-16T10:13:19Z'))"#,                         2),
    (r#"boolean in (true)"#,                                                        2),
    (r#"boolean not in (false)"#,                                                   2),
];

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    let ds = PlaceCSV::new();
    harness(ds, &PREDICATES)
}

#[tokio::test]
async fn test_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &PREDICATES).await
}

#[tokio::test]
async fn test_sql() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_sql(ds, &PREDICATES).await
}

#[tokio::test]
async fn test_pg_sql() -> Result<(), Box<dyn Error>> {
    let ds = PlacePG::new().await?;
    harness_sql(ds, &PREDICATES).await
}
