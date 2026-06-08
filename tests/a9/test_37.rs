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

#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
use std::error::Error;

#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
#[rustfmt::skip]
const POINT_PREDICATE: [(&str, u32); 1] = [
    ("S_OVERLAPS(geom,MULTIPOINT(7 50,10 51))", 0),
];

#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
#[rustfmt::skip]
const LINE_PREDICATE: [(&str, u32); 1] = [
    ("S_OVERLAPS(geom,LINESTRING(7 50,10 51))", 0),
];

#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
#[rustfmt::skip]
const POLYGON_PREDICATE: [(&str, u32); 1] = [
    ("S_OVERLAPS(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))", 0),
];

#[cfg(feature = "csv_ds")]
#[test]
fn test_points() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlaceCSV, harness};

    let ds = PlaceCSV::new();
    harness(ds, &POINT_PREDICATE)
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_points_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlaceGPkg, harness_gpkg};

    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &POINT_PREDICATE).await
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_points_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlaceGPkg, harness_sql};

    let ds = PlaceGPkg::new().await?;
    harness_sql(ds, &POINT_PREDICATE).await
}

#[cfg(feature = "pg_ds")]
#[tokio::test]
async fn test_points_pg_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlacePG, harness_sql};

    let ds = PlacePG::new().await?;
    harness_sql(ds, &POINT_PREDICATE).await
}

#[cfg(feature = "csv_ds")]
#[test]
fn test_lines() -> Result<(), Box<dyn Error>> {
    use crate::utils::{RiverCSV, harness};

    let ds = RiverCSV::new();
    harness(ds, &LINE_PREDICATE)
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_lines_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::{RiverGPkg, harness_gpkg};

    let ds = RiverGPkg::new().await?;
    harness_gpkg(ds, &LINE_PREDICATE).await
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_lines_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{RiverGPkg, harness_sql};
    let ds = RiverGPkg::new().await?;
    harness_sql(ds, &LINE_PREDICATE).await
}

#[cfg(feature = "pg_ds")]
#[tokio::test]
async fn test_lines_pg_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{RiverPG, harness_sql};

    let ds = RiverPG::new().await?;
    harness_sql(ds, &LINE_PREDICATE).await
}

#[cfg(feature = "csv_ds")]
#[test]
fn test_polygons() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryCSV, harness};

    let ds = CountryCSV::new();
    harness(ds, &POLYGON_PREDICATE)
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_polygons_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryGPkg, harness_gpkg};

    let ds = CountryGPkg::new().await?;
    harness_gpkg(ds, &POLYGON_PREDICATE).await
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_polygons_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryGPkg, harness_sql};

    let ds = CountryGPkg::new().await?;
    harness_sql(ds, &POLYGON_PREDICATE).await
}

#[cfg(feature = "pg_ds")]
#[tokio::test]
async fn test_polygons_pg_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryPG, harness_sql};

    let ds = CountryPG::new().await?;
    harness_sql(ds, &POLYGON_PREDICATE).await
}
