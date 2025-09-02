// SPDX-License-Identifier: Apache-2.0

//! Test the S_EQUALS spatial function
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     for each queryable {queryable} with a geometry data type, evaluate the
//!     following filter expressions
//!     * S_EQUALS({queryable},POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))
//!     * S_EQUALS({queryable},LINESTRING(7 50,10 51))
//!     * S_EQUALS({queryable},POINT(7.02 49.92))
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets of the first two filter expressions for
//!   each queryable are identical;
//! * store the valid predicates for each data source.
//!

use crate::utils::{COUNTRIES, harness};
use std::error::Error;
use tracing_test::traced_test;

// Countries CSV data set contains 177 records all being polygons...
#[rustfmt::skip]
const PREDICATES: [(&str, u32); 3] = [
    ("S_EQUALS(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))", 0),
    ("S_EQUALS(geom,LINESTRING(7 50,10 51))",                              0),
    ("S_EQUALS(geom,POINT(7.02 49.92))",                                   0),
];

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    harness(COUNTRIES, PREDICATES.to_vec())
}
