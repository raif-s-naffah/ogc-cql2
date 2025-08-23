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

use crate::utils::{COUNTRIES, PLACES, RIVERS, harness};
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
#[traced_test]
fn test_points() -> Result<(), Box<dyn Error>> {
    harness(PLACES, POINT_PREDICATE.to_vec())
}

#[test]
#[traced_test]
fn test_lines() -> Result<(), Box<dyn Error>> {
    harness(RIVERS, LINE_PREDICATE.to_vec())
}

#[test]
#[traced_test]
fn test_polygons() -> Result<(), Box<dyn Error>> {
    harness(COUNTRIES, POLYGON_PREDICATE.to_vec())
}
