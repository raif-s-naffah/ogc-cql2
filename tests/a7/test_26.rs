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

#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
use std::error::Error;

#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
#[rustfmt::skip]
const COUNTRIES_PREDICATES: [(&str, u32); 6] = [
    (r#"S_INTERSECTS(geom,BBOX(0,40,10,50))"#,        8),
    (r#"S_INTERSECTS(geom,BBOX(150,-90,-150,90))"#,  10),
    (r#"S_INTERSECTS(geom,POINT(7.02 49.92))"#,       1),
    (r#"S_INTERSECTS(geom,BBOX(0,40,10,50)) 
       and S_INTERSECTS(geom,BBOX(5,50,10,60))"#,     3),
    (r#"S_INTERSECTS(geom,BBOX(0,40,10,50)) 
       and not S_INTERSECTS(geom,BBOX(5,50,10,60))"#, 5),
    (r#"S_INTERSECTS(geom,BBOX(0,40,10,50)) 
       or S_INTERSECTS(geom,BBOX(-90,40,-60,50))"#,  10)
];
#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
const PLACES_PREDICATES: [(&str, u32); 1] = [(r#"S_INTERSECTS(geom,BBOX(0,40,10,50))"#, 7)];
#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
const RIVERS_PREDICATES: [(&str, u32); 1] = [(r#"S_INTERSECTS(geom,BBOX(-180,-90,0,90))"#, 4)];

#[cfg(feature = "csv_ds")]
#[test]
fn test_countries() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryCSV, harness};

    let ds = CountryCSV::new();
    harness(ds, &COUNTRIES_PREDICATES)
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_countries_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryGPkg, harness_gpkg};

    let ds = CountryGPkg::new().await?;
    harness_gpkg(ds, &COUNTRIES_PREDICATES).await
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_countries_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryGPkg, harness_sql};

    let ds = CountryGPkg::new().await?;
    harness_sql(ds, &COUNTRIES_PREDICATES).await
}

#[cfg(feature = "pg_ds")]
#[tokio::test]
async fn test_countries_pg_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryPG, harness_sql};

    let ds = CountryPG::new().await?;
    harness_sql(ds, &COUNTRIES_PREDICATES).await
}

#[cfg(feature = "csv_ds")]
#[test]
fn test_places() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlaceCSV, harness};

    let ds = PlaceCSV::new();
    harness(ds, &PLACES_PREDICATES)
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_places_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlaceGPkg, harness_gpkg};

    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &PLACES_PREDICATES).await
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_places_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlaceGPkg, harness_sql};

    let ds = PlaceGPkg::new().await?;
    harness_sql(ds, &PLACES_PREDICATES).await
}

#[cfg(feature = "pg_ds")]
#[tokio::test]
async fn test_places_pg_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlacePG, harness_sql};

    let ds = PlacePG::new().await?;
    harness_sql(ds, &PLACES_PREDICATES).await
}

#[cfg(feature = "csv_ds")]
#[test]
fn test_rivers() -> Result<(), Box<dyn Error>> {
    use crate::utils::{RiverCSV, harness};

    let ds = RiverCSV::new();
    harness(ds, &RIVERS_PREDICATES)
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_rivers_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::{RiverGPkg, harness_gpkg};

    let ds = RiverGPkg::new().await?;
    harness_gpkg(ds, &RIVERS_PREDICATES).await
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_rivers_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{RiverGPkg, harness_sql};

    let ds = RiverGPkg::new().await?;
    harness_sql(ds, &RIVERS_PREDICATES).await
}

#[cfg(feature = "pg_ds")]
#[tokio::test]
async fn test_rivers_pg_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{RiverPG, harness_sql};

    let ds = RiverPG::new().await?;
    harness_sql(ds, &RIVERS_PREDICATES).await
}
