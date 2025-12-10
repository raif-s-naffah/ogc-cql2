// SPDX-License-Identifier: Apache-2.0

//! Test the S_WITHIN spatial function
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     for each queryable {queryable} with a geometry data type, evaluate the
//!     following filter expressions
//!     * S_WITHIN({queryable},BBOX(-180,-90,180,90))
//!     * S_WITHIN({queryable},POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))
//!     * S_WITHIN({queryable},LINESTRING(7 50,10 51))
//!     * S_WITHIN({queryable},MULTIPOINT(7 50,10 51))
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets of the first two filter expressions for
//!   each queryable are identical;
//! * store the valid predicates for each data source.
//!

use crate::utils::{CountryCSV, CountryGPkg, CountryPG, harness, harness_gpkg, harness_sql};
use std::error::Error;

// Countries data set contains 177 records...
#[rustfmt::skip]
const PREDICATES: [(&str, u32); 4] = [
    ("S_WITHIN(geom,BBOX(-180,-90,180,90))",                               177),
    ("S_WITHIN(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))", 177),
    ("S_WITHIN(geom,LINESTRING(7 50,10 51))",                                0),
    ("S_WITHIN(geom,MULTIPOINT((7 50),(10 51)))",                            0),
];

#[test]
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

#[tokio::test]
async fn test_pg_sql() -> Result<(), Box<dyn Error>> {
    let ds = CountryPG::new().await?;
    harness_sql(ds, &PREDICATES).await
}
