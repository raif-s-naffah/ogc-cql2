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

use crate::utils::{
    CountryCSV, CountryGPkg, PlaceCSV, PlaceGPkg, RiverCSV, RiverGPkg, harness, harness_gpkg,
    harness_sql,
};
use std::error::Error;
use tracing_test::traced_test;

#[rustfmt::skip]
const COUNTRIES_PREDICATES: [(&str, u32); 15] = [
    ("S_INTERSECTS(geom,POLYGON((0 40,10 40,10 50,0 50,0 40)))",    8),
    ("S_INTERSECTS(geom,LINESTRING(0 40,10 50))",                   4),
    ("S_DISJOINT(geom,BBOX(0,40,10,50))",                         169),
    ("S_DISJOINT(geom,POLYGON((0 40,10 40,10 50,0 50,0 40)))",    169),
    ("S_DISJOINT(geom,LINESTRING(0 40,10 50))",                   173),
    ("S_DISJOINT(geom,POINT(7.02 49.92))",                        176),
    (r#"S_TOUCHES(geom,POLYGON((
        6.043073357781111 50.128051662794235,6.242751092156993 49.90222565367873,
        6.186320428094177 49.463802802114515,5.897759230176348 49.44266714130711,
        5.674051954784829 49.529483547557504,5.782417433300907 50.09032786722122,
        6.043073357781111 50.128051662794235)))"#,                  3),  // 6
    ("S_TOUCHES(geom,POINT(6.043073357781111 50.128051662794235))", 3),  // 7
    ("S_TOUCHES(geom,POINT(6.242751092156993 49.90222565367873))",  2),  // 8
    (r#"S_TOUCHES(geom,LINESTRING(
        6.043073357781111 50.128051662794235,
        6.242751092156993 49.90222565367873))"#,                    3),  // 9
    ("S_WITHIN(geom,BBOX(-180,-90,0,90))",                         44),
    ("S_CONTAINS(geom,BBOX(7,50,8,51))",                            1),
    ("S_CONTAINS(geom,LINESTRING(7 50,8 51))",                      1),
    ("S_CONTAINS(geom,POINT(7.02 49.92))",                          1),
    ("S_OVERLAPS(geom,BBOX(-180,-90,0,90))",                       11)
];

#[rustfmt::skip]
const PLACES_PREDICATES: [(&str, u32); 5] = [
    ("S_INTERSECTS(geom,POLYGON((0 40,10 40,10 50,0 50,0 40)))",  7),
    ("S_DISJOINT(geom,BBOX(0,40,10,50))",                       236),
    ("S_DISJOINT(geom,POLYGON((0 40,10 40,10 50,0 50,0 40)))",  236),
    ("S_EQUALS(geom,POINT(6.1300028 49.6116604))",                1),
    ("S_WITHIN(geom,BBOX(-180,-90,0,90))",                       74)
];

#[rustfmt::skip]
const RIVERS_PREDICATES: [(&str, u32); 6] = [
    ("S_INTERSECTS(geom,LINESTRING(-60 -90,-60 90))", 2),
    ("S_DISJOINT(geom,BBOX(-180,-90,0,90))",          9),
    ("S_DISJOINT(geom,LINESTRING(-60 -90,-60 90))",  11),
    ("S_CROSSES(geom,BBOX(0,40,10,50))",              1),
    ("S_CROSSES(geom,LINESTRING(-60 -90,-60 90))",    2),
    ("S_WITHIN(geom,BBOX(-180,-90,0,90))",            4)
];

#[test]
#[traced_test]
fn test_countries() -> Result<(), Box<dyn Error>> {
    let ds = CountryCSV::new();
    harness(ds, &COUNTRIES_PREDICATES)
}

#[tokio::test]
async fn test_countries_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    harness_gpkg(ds, &COUNTRIES_PREDICATES).await
}

#[tokio::test]
async fn test_countries_sql() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    harness_sql(ds, &COUNTRIES_PREDICATES).await
}

#[test]
#[traced_test]
fn test_places() -> Result<(), Box<dyn Error>> {
    let ds = PlaceCSV::new();
    harness(ds, &PLACES_PREDICATES)
}

#[tokio::test]
async fn test_places_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &PLACES_PREDICATES).await
}

#[tokio::test]
async fn test_places_sql() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_sql(ds, &PLACES_PREDICATES).await
}

#[test]
#[traced_test]
fn test_rivers() -> Result<(), Box<dyn Error>> {
    let ds = RiverCSV::new();
    harness(ds, &RIVERS_PREDICATES)
}

#[tokio::test]
async fn test_rivers_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = RiverGPkg::new().await?;
    harness_gpkg(ds, &RIVERS_PREDICATES).await
}

#[tokio::test]
async fn test_rivers_sql() -> Result<(), Box<dyn Error>> {
    let ds = RiverGPkg::new().await?;
    harness_sql(ds, &RIVERS_PREDICATES).await
}
