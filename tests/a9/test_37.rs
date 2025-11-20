// SPDX-License-Identifier: Apache-2.0

//! Test the S_OVERLAPS spatial function
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     * For each queryable {queryable} of type Point or MultiPoint, evaluate
//!       the filter expression S_OVERLAPS({queryable},MULTIPOINT(7 50,10 51))
//!     * For each queryable {queryable} of type LineString or MultiLineString,
//!       evaluate the filter expression
//!         S_OVERLAPS({queryable},LINESTRING(7 50,10 51))
//!     * For each queryable {queryable} of type Polygon or MultiPolygon,
//!       evaluate the filter expression
//!         S_OVERLAPS({queryable},POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))
//! Then:
//! * assert successful execution of the evaluation;
//! * store the valid predicates for each data source.
//!

use crate::utils::{
    CountryCSV, CountryGPkg, PlaceCSV, PlaceGPkg, RiverCSV, RiverGPkg, harness, harness_gpkg,
    harness_sql,
};
use std::error::Error;
use tracing_test::traced_test;

#[rustfmt::skip]
const POINT_PREDICATE: [(&str, u32); 1] = [
    ("S_OVERLAPS(geom,MULTIPOINT(7 50,10 51))", 0),
];

#[rustfmt::skip]
const LINE_PREDICATE: [(&str, u32); 1] = [
    ("S_OVERLAPS(geom,LINESTRING(7 50,10 51))", 0),
];

#[rustfmt::skip]
const POLYGON_PREDICATE: [(&str, u32); 1] = [
    ("S_OVERLAPS(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))", 0),
];

#[test]
fn test_points() -> Result<(), Box<dyn Error>> {
    let ds = PlaceCSV::new();
    harness(ds, &POINT_PREDICATE)
}

#[tokio::test]
async fn test_points_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &POINT_PREDICATE).await
}

#[tokio::test]
async fn test_points_sql() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_sql(ds, &POINT_PREDICATE).await
}

#[test]
#[traced_test]
fn test_lines() -> Result<(), Box<dyn Error>> {
    let ds = RiverCSV::new();
    harness(ds, &LINE_PREDICATE)
}

#[tokio::test]
async fn test_lines_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = RiverGPkg::new().await?;
    harness_gpkg(ds, &LINE_PREDICATE).await
}

#[tokio::test]
async fn test_lines_sql() -> Result<(), Box<dyn Error>> {
    let ds = RiverGPkg::new().await?;
    harness_sql(ds, &LINE_PREDICATE).await
}

#[test]
#[traced_test]
fn test_polygons() -> Result<(), Box<dyn Error>> {
    let ds = CountryCSV::new();
    harness(ds, &POLYGON_PREDICATE)
}

#[tokio::test]
async fn test_polygons_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    harness_gpkg(ds, &POLYGON_PREDICATE).await
}

#[tokio::test]
async fn test_polygons_sql() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    harness_sql(ds, &POLYGON_PREDICATE).await
}
