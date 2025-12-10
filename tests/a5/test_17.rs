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

use crate::utils::{PlaceCSV, PlaceGPkg, PlacePG, harness, harness_gpkg, harness_sql};
use std::error::Error;

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
