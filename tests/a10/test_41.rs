// SPDX-License-Identifier: Apache-2.0

//! Test the temporal comparison functions with intervals
//!
//! Given:
//!     * One or more data sources, each with a list of queryables with at
//!       least two queryables of type Timestamp or Date.
//! When:
//!     For each pair of queryables {queryable1} and {queryable2} of data type
//!     Timestamp, evaluate the following filter expressions
//!     * T_AFTER(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_BEFORE(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_DISJOINT(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_EQUALS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_INTERSECTS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_CONTAINS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_DURING(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_FINISHEDBY(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_FINISHES(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_MEETS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_METBY(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_OVERLAPPEDBY(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_OVERLAPS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_STARTEDBY(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!     * T_STARTS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))
//!
//!     For each pair of queryables {queryable2} and {queryable2} of data type
//!     Date, evaluate the following filter expressions
//!     * T_AFTER(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_BEFORE(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_DISJOINT(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_EQUALS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_INTERSECTS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_CONTAINS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_DURING(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_FINISHEDBY(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_FINISHES(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_MEETS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_METBY(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_OVERLAPPEDBY(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_OVERLAPS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_STARTEDBY(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//!     * T_STARTS(INTERVAL({queryable1},{queryable2}),INTERVAL('2021-01-01','2021-12-31'))
//! Then:
//! * assert successful execution of the evaluation;
//! * store the valid predicates for each data source.
//!

use crate::utils::{PlaceCSV, PlaceGPkg, harness, harness_gpkg, harness_sql};
use std::error::Error;

#[rustfmt::skip]
const TIMESTAMP_PREDICATES: [(&str, u32); 15] = [
    ("T_AFTER(INTERVAL(start,end),INTERVAL(       '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 2),
    ("T_BEFORE(INTERVAL(start,end),INTERVAL(      '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_DISJOINT(INTERVAL(start,end),INTERVAL(    '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 2),
    ("T_EQUALS(INTERVAL(start,end),INTERVAL(      '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_INTERSECTS(INTERVAL(start,end),INTERVAL(  '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 1),
    ("T_CONTAINS(INTERVAL(start,end),INTERVAL(    '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_DURING(INTERVAL(start,end),INTERVAL(      '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_FINISHEDBY(INTERVAL(start,end),INTERVAL(  '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_FINISHES(INTERVAL(start,end),INTERVAL(    '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_MEETS(INTERVAL(start,end),INTERVAL(       '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_METBY(INTERVAL(start,end),INTERVAL(       '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_OVERLAPPEDBY(INTERVAL(start,end),INTERVAL('2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 1),
    ("T_OVERLAPS(INTERVAL(start,end),INTERVAL(    '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_STARTEDBY(INTERVAL(start,end),INTERVAL(   '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
    ("T_STARTS(INTERVAL(start,end),INTERVAL(      '2021-01-01T00:00:00Z','2021-12-31T23:59:59Z'))", 0),
];

#[rustfmt::skip]
const DATE_PREDICATES: [(&str, u32); 15] = [
    ("T_AFTER(       INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 2),
    ("T_BEFORE(      INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 3),
    ("T_DISJOINT(    INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 3),
    ("T_EQUALS(      INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_INTERSECTS(  INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_CONTAINS(    INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_DURING(      INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 3),
    ("T_FINISHEDBY(  INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_FINISHES(    INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_MEETS(       INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_METBY(       INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_OVERLAPPEDBY(INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_OVERLAPS(    INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_STARTEDBY(   INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
    ("T_STARTS(      INTERVAL(date,'..'),INTERVAL('2021-01-01','2021-12-31'))", 0),
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
