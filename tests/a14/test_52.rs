// SPDX-License-Identifier: Apache-2.0

//! Test predicates with arithmetic expressions
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * At least one queryable has a numeric data type.
//! When:
//!     For each queryable construct multiple valid filter expressions
//!     involving arithmetic expressions.
//! Then:
//! * assert successful execution of the evaluation.
//!

use crate::utils::{CountryCSV, CountryGPkg, CountryPG, harness, harness_gpkg, harness_sql};
use std::error::Error;

#[rustfmt::skip]
const PREDICATES: [(&str, u32); 13] = [
    ("POP_EST=25364300+7",          1),
    ("25364300+7=POP_EST",          1),
    ("POP_EST>=1038290-2*2^0",    157),
    ("POP_EST>1038290-20/10",     157),
    ("POP_EST>1038290-21 div 10", 157),
    ("POP_EST>1038290-5%2",       157),
    ("POP_EST<=1038200+8*11",      20),
    ("POP_EST<1038280+2^3",        20),
    ("POP_EST<>25364300+3^2-2",   176),
    ("POP_EST between 4000000/4 and (3*(900000+100000))",      22),
    ("POP_EST not between 4000000/4 and (3*(900000+100000))", 155),
    (r#"POP_EST in (25364300+7,1000000+600000+11692,
        3*1000000+13258,3*1000000+13257,30*100000+13259)"#,     1),
    (r#"POP_EST not in (25364300+7,1000000+600000+11692,
        3*1000000+13258,3*1000000+13257,30*100000+13259)"#,   176),
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
    let gpkg = CountryGPkg::new().await?;
    harness_sql(gpkg, &PREDICATES).await
}

#[tokio::test]
async fn test_pg_sql() -> Result<(), Box<dyn Error>> {
    let gpkg = CountryPG::new().await?;
    harness_sql(gpkg, &PREDICATES).await
}
