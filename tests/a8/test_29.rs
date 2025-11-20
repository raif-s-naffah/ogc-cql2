// SPDX-License-Identifier: Apache-2.0

//! Test predicates against the test dataset
//!
//! Given:
//!     * The implementation under test uses the test dataset.
//! When:
//!     Evaluate each predicate in Predicates and expected results.
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the expected result is returned.
//!

use crate::utils::{CountryCSV, CountryGPkg, harness, harness_gpkg, harness_sql};
use std::error::Error;
use tracing_test::traced_test;

const PREDICATES: [(&str, u32); 7] = [
    ("S_INTERSECTS(geom,LINESTRING(-180 -45, 0 -45))", 2),
    (
        "S_INTERSECTS(geom,MULTILINESTRING((-180 -45, 0 -45), (0 45, 180 45)))",
        14,
    ),
    (
        "S_INTERSECTS(geom,POLYGON(
            (-180 -90, -90 -90, -90 90, -180 90, -180 -90), 
            (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)))",
        8,
    ),
    (
        "S_INTERSECTS(geom,MULTIPOLYGON(
        ((-180 -90, -90 -90, -90 90, -180 90, -180 -90), 
            (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)),
        ((0 0, 10 0, 10 10, 0 10, 0 0))))",
        15,
    ),
    (
        "S_INTERSECTS(geom,GEOMETRYCOLLECTION(
        POINT(7.02 49.92), 
        POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))))",
        8,
    ),
    (
        "S_INTERSECTS(geom,POLYGON(
            (-180 -90, -90 -90, -90 90, -180 90, -180 -90), 
            (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50))) 
        or S_INTERSECTS(geom,POLYGON((0 0, 10 0, 10 10, 0 10, 0 0)))",
        15,
    ),
    (
        "S_INTERSECTS(geom,POLYGON(
            (-180 -90, -90 -90, -90 90, -180 90, -180 -90), 
            (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50))) 
        and not S_INTERSECTS(geom,POLYGON((-130 0, 0 0, 0 50, -130 50, -130 0)))",
        3,
    ),
];

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    let ds = CountryCSV::new();
    harness(ds, &PREDICATES)
}

#[tokio::test]
async fn test_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    harness_gpkg(ds, &PREDICATES).await
}

#[tokio::test]
async fn test_sql() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    harness_sql(ds, &PREDICATES).await
}
