// SPDX-License-Identifier: Apache-2.0

//! Test predicates against the test dataset
//!
//! Given:
//!     * The implementation under test uses the test dataset.
//! When:
//!     Evaluate each predicate in Predicates and expected results, if the
//!     conditional dependency is met.
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the expected result is returned;
//! * store the valid predicates for each data source.
//!

#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
use std::error::Error;

#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
#[rustfmt::skip]
const PREDICATES: [(&str, u32); 13] = [
    ("pop_other=1038280+8",           1),
    ("pop_other>=1038290-2*2^0",    123),
    ("pop_other>1038290-20/10",     122),
    ("pop_other>1038290-21 div 10", 122),
    ("pop_other>1038290-5%2",       122),
    ("pop_other<=1038200+8*11",     121),
    ("pop_other<1038280+2^3",       120),
    ("pop_other<>1038290-2^1",      242),
    ("pop_other between 4000000/4 and (3*(900000+100000))",      75),
    ("pop_other not between 4000000/4 and (3*(900000+100000))", 168),
    (r#"pop_other in (1000000+38288,1000000+600000+11692,
        3*1000000+13258,3*1000000+13257,30*100000+13259)"#,       3),
    (r#"pop_other not in (1000000+38288,1000000+600000+11692,
        3*1000000+13258,3*1000000+13257,30*100000+13259)"#,     240),
    ("1038280+8=pop_other",           1)
];

#[cfg(feature = "csv_ds")]
#[test]
fn test() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlaceCSV, harness};

    let ds = PlaceCSV::new();
    harness(ds, &PREDICATES)
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlaceGPkg, harness_gpkg};

    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &PREDICATES).await
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlaceGPkg, harness_sql};

    let gpkg = PlaceGPkg::new().await?;
    harness_sql(gpkg, &PREDICATES).await
}

#[cfg(feature = "pg_ds")]
#[tokio::test]
async fn test_pg_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{PlacePG, harness_sql};

    let gpkg = PlacePG::new().await?;
    harness_sql(gpkg, &PREDICATES).await
}
