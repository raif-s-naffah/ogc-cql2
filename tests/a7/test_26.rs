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

use crate::utils::{COUNTRIES, PLACES, RIVERS, harness};
use std::error::Error;
use tracing_test::traced_test;

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
const PLACES_PREDICATES: [(&str, u32); 1] = [(r#"S_INTERSECTS(geom,BBOX(0,40,10,50))"#, 7)];
const RIVERS_PREDICATES: [(&str, u32); 1] = [(r#"S_INTERSECTS(geom,BBOX(-180,-90,0,90))"#, 4)];

#[test]
#[traced_test]
fn test_countries() -> Result<(), Box<dyn Error>> {
    harness(COUNTRIES, COUNTRIES_PREDICATES.to_vec())
}

#[test]
#[traced_test]
fn test_places() -> Result<(), Box<dyn Error>> {
    harness(PLACES, PLACES_PREDICATES.to_vec())
}

#[test]
#[traced_test]
fn test_rivers() -> Result<(), Box<dyn Error>> {
    harness(RIVERS, RIVERS_PREDICATES.to_vec())
}
