// SPDX-License-Identifier: Apache-2.0

//! Test the T_AFTER, T_BEFORE, T_DISJOINT, T_EQUALS, T_INTERSECTS temporal
//! comparison functions.
//!
//! Given:
//!     * One or more data sources, each with a list of queryables with at
//!       least one queryable of type Timestamp or Date.
//! When:
//!     For each queryable {queryable} of data type Timestamp, evaluate the
//!     following filter expressions
//!     * T_AFTER({queryable},TIMESTAMP('2022-04-24T07:59:57Z'))
//!     * T_AFTER({queryable},INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_BEFORE({queryable},TIMESTAMP('2022-04-24T07:59:57Z'))
//!     * T_BEFORE({queryable},INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_DISJOINT({queryable},TIMESTAMP('2022-04-24T07:59:57Z'))
//!     * T_DISJOINT({queryable},INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_EQUALS({queryable},TIMESTAMP('2022-04-24T07:59:57Z'))
//!     * T_EQUALS({queryable},INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_INTERSECTS({queryable},TIMESTAMP('2022-04-24T07:59:57Z'))
//!     * T_INTERSECTS({queryable},INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!
//!     For each queryable {queryable} of data type Date, evaluate the following
//!     filter expressions
//!     * T_AFTER({queryable},DATE('2022-04-24'))
//!     * T_AFTER({queryable},INTERVAL('2021-01-01','2021-12-31'))
//!     * T_BEFORE({queryable},DATE('2022-04-24'))
//!     * T_BEFORE({queryable},INTERVAL('2021-01-01','2021-12-31'))
//!     * T_DISJOINT({queryable},DATE('2022-04-24'))
//!     * T_DISJOINT({queryable},INTERVAL('2021-01-01','2021-12-31'))
//!     * T_EQUALS({queryable},DATE('2022-04-24'))
//!     * T_EQUALS({queryable},INTERVAL('2021-01-01','2021-12-31'))
//!     * T_INTERSECTS({queryable},DATE('2022-04-24'))
//!     * T_INTERSECTS({queryable},INTERVAL('2021-01-01','2021-12-31'))
//! Then:
//! * assert successful execution of the evaluation;
//! * store the valid predicates for each data source.
//!

use crate::utils::{PlaceCSV, PlaceGPkg, PlacePG, harness, harness_gpkg, harness_sql};
use std::error::Error;

#[rustfmt::skip]
const TIMESTAMP_PREDICATES: [(&str, u32); 10] = [
    ("T_AFTER(start,   TIMESTAMP(  '2022-04-24T07:59:57Z'))",                        0),
    ("T_AFTER(start,   INTERVAL(   '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 2),
    ("T_BEFORE(start,  TIMESTAMP(  '2022-04-24T07:59:57Z'))",                        3),
    ("T_BEFORE(start,  INTERVAL(   '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_DISJOINT(start,TIMESTAMP(  '2022-04-24T07:59:57Z'))",                        3),
    ("T_DISJOINT(start,INTERVAL(   '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 2),
    ("T_EQUALS(start,TIMESTAMP(    '2022-04-24T07:59:57Z'))",                        0),
    ("T_EQUALS(start,INTERVAL(     '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_INTERSECTS(start,TIMESTAMP('2022-04-24T07:59:57Z'))",                        0),
    ("T_INTERSECTS(start,INTERVAL( '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 1),
];

#[rustfmt::skip]
const DATE_PREDICATES: [(&str, u32); 10] = [
    ("T_AFTER(date,DATE(         '2022-04-24'))",              1),
    ("T_AFTER(date,INTERVAL(     '2021-01-01','2021-12-31'))", 2),
    ("T_BEFORE(date,DATE(        '2022-04-24'))",              2),
    ("T_BEFORE(date,INTERVAL(    '2021-01-01','2021-12-31'))", 0),
    ("T_DISJOINT(date,DATE(      '2022-04-24'))",              3),
    ("T_DISJOINT(date,INTERVAL(  '2021-01-01','2021-12-31'))", 2),
    ("T_EQUALS(date,DATE(        '2022-04-24'))",              0),
    ("T_EQUALS(date,INTERVAL(    '2021-01-01','2021-12-31'))", 0),
    ("T_INTERSECTS(date,DATE(    '2022-04-24'))",              0),
    ("T_INTERSECTS(date,INTERVAL('2021-01-01','2021-12-31'))", 1)
];

#[test]
fn test_timestamps() -> Result<(), Box<dyn Error>> {
    let ds = PlaceCSV::new();
    harness(ds, &TIMESTAMP_PREDICATES)
}

#[tokio::test]
async fn test_timestamps_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &TIMESTAMP_PREDICATES).await
}

#[tokio::test]
async fn test_timestamps_sql() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_sql(ds, &TIMESTAMP_PREDICATES).await
}

#[tokio::test]
async fn test_timestamps_pg_sql() -> Result<(), Box<dyn Error>> {
    let ds = PlacePG::new().await?;
    harness_sql(ds, &TIMESTAMP_PREDICATES).await
}

#[test]
fn test_dates() -> Result<(), Box<dyn Error>> {
    let ds = PlaceCSV::new();
    harness(ds, &DATE_PREDICATES)
}

#[tokio::test]
async fn test_dates_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &DATE_PREDICATES).await
}

#[tokio::test]
async fn test_dates_sql() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_sql(ds, &DATE_PREDICATES).await
}

#[tokio::test]
async fn test_dates_pg_sql() -> Result<(), Box<dyn Error>> {
    let ds = PlacePG::new().await?;
    harness_sql(ds, &DATE_PREDICATES).await
}
