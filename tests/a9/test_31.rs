// SPDX-License-Identifier: Apache-2.0

//! Test the S_DISJOINT spatial function
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     for each queryable {queryable} with a geometry data type, evaluate the
//!     following filter expressions
//!     * S_DISJOINT({queryable},BBOX(-180,-90,180,90))
//!     * S_DISJOINT({queryable},POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))
//!     * S_DISJOINT({queryable},LINESTRING(7 50,10 51))
//!     * S_DISJOINT({queryable},POINT(7.02 49.92))
//!     * S_DISJOINT({queryable},POINT(90 180))
//! Then:
//! * assert successful execution of the evaluation for the first four filter
//!   expressions;
//! * assert unsuccessful execution of the evaluation for the fifth filter
//!   expressions (invalid coordinate);
//! * assert that the two result sets of the first two filter expressions for
//!   each queryable are empty;
//! * assert that the results sets of the third and fourth filter expressions
//!   for each queryable do not have an item in common with the corresponding
//!   S_INTERSECTS expression;
//! * store the valid predicates for each data source.
//!

use ogc_cql2::prelude::*;
use std::error::Error;

// Countries data set contains 177 records...
#[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
#[rustfmt::skip]
const DISJOINT: [(&str, u32); 4] = [
    ("S_DISJOINT(geom,BBOX(-180,-90,180,90))",                               0),
    ("S_DISJOINT(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))", 0),
    ("S_DISJOINT(geom,LINESTRING(7 50, 10 51))",                           176),
    ("S_DISJOINT(geom,POINT(7.02 49.92))",                                 176),
];

// const _INTERSECTS: [(&str, u32); 4] = [
//     ("S_INTERSECTS(geom,BBOX(-180,-90,180,90))", 177),
//     ("S_INTERSECTS(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))", 177),
//     ("S_INTERSECTS(geom,LINESTRING(7 50, 10 51))", 1),
//     ("S_INTERSECTS(geom,POINT(7.02 49.92))", 1),
// ];

#[test]
fn test_invalid_coordinates() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"S_DISJOINT(geom,POINT(90 180))"#;

    let expr = Expression::try_from_text(E)?;
    let shared_ctx = Context::new().freeze();
    let mut evaluator = ExEvaluator::new(shared_ctx);
    evaluator.setup(expr)?;

    let f1 = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("geom".into(), Q::try_from_wkt("POINT(0 0)")?),
    ]);
    let res = evaluator.evaluate(&f1);
    assert!(res.is_err());
    assert!(matches!(res.err(), Some(MyError::Runtime(_))));

    Ok(())
}

#[cfg(feature = "csv_ds")]
#[test]
fn test() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryCSV, harness};

    let ds = CountryCSV::new();
    harness(ds, &DISJOINT)
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryGPkg, harness_gpkg};

    let ds = CountryGPkg::new().await?;
    harness_gpkg(ds, &DISJOINT).await
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryGPkg, harness_sql};

    let ds = CountryGPkg::new().await?;
    harness_sql(ds, &DISJOINT).await
}

#[cfg(feature = "pg_ds")]
#[tokio::test]
async fn test_pg_sql() -> Result<(), Box<dyn Error>> {
    use crate::utils::{CountryPG, harness_sql};

    let ds = CountryPG::new().await?;
    harness_sql(ds, &DISJOINT).await
}

#[cfg(feature = "csv_ds")]
#[test]
fn test_e3_intersect() -> Result<(), Box<dyn Error>> {
    use crate::utils::countries;

    const E1: &str = "S_DISJOINT(geom,LINESTRING(7 50, 10 51))";
    const E2: &str = "S_INTERSECTS(geom,LINESTRING(7 50, 10 51))";

    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let expr2 = Expression::try_from_text(E2)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for c in countries()? {
        // if a record passes in evaluator1 it should fail in evaluator 2
        // and v/v...
        let res1 = evaluator1.evaluate(&c)?;
        let res2 = evaluator2.evaluate(&c)?;
        match (res1, res2) {
            (Outcome::T, Outcome::F) | (Outcome::F, Outcome::T) => (),
            (x, y) => panic!("Unexpected results {x}, {y}. Abort"),
        }
    }

    Ok(())
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_e3_intersect_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::countries_gpkg;

    const E1: &str = "S_DISJOINT(geom,LINESTRING(7 50, 10 51))";
    const E2: &str = "S_INTERSECTS(geom,LINESTRING(7 50, 10 51))";

    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let expr2 = Expression::try_from_text(E2)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for c in countries_gpkg().await? {
        let res1 = evaluator1.evaluate(&c)?;
        let res2 = evaluator2.evaluate(&c)?;
        match (res1, res2) {
            (Outcome::T, Outcome::F) | (Outcome::F, Outcome::T) => (),
            (x, y) => panic!("Unexpected results {x}, {y}. Abort"),
        }
    }

    Ok(())
}

#[cfg(feature = "csv_ds")]
#[test]
fn test_e4_intersect() -> Result<(), Box<dyn Error>> {
    use crate::utils::countries;

    const E1: &str = "S_DISJOINT(geom,POINT(7.02 49.92))";
    const E2: &str = "S_INTERSECTS(geom,POINT(7.02 49.92))";

    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let expr2 = Expression::try_from_text(E2)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for c in countries()? {
        // if a record passes evaluator1 it should fail evaluator 2
        // and v/v...
        let res1 = evaluator1.evaluate(&c)?;
        let res2 = evaluator2.evaluate(&c)?;
        match (res1, res2) {
            (Outcome::T, Outcome::F) | (Outcome::F, Outcome::T) => (),
            (x, y) => panic!("Unexpected results {x}, {y}. Abort"),
        }
    }

    Ok(())
}

#[cfg(feature = "gpkg_ds")]
#[tokio::test]
async fn test_e4_intersect_gpkg() -> Result<(), Box<dyn Error>> {
    use crate::utils::countries_gpkg;

    const E1: &str = "S_DISJOINT(geom,POINT(7.02 49.92))";
    const E2: &str = "S_INTERSECTS(geom,POINT(7.02 49.92))";

    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let expr2 = Expression::try_from_text(E2)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for c in countries_gpkg().await? {
        let res1 = evaluator1.evaluate(&c)?;
        let res2 = evaluator2.evaluate(&c)?;
        match (res1, res2) {
            (Outcome::T, Outcome::F) | (Outcome::F, Outcome::T) => (),
            (x, y) => panic!("Unexpected results {x}, {y}. Abort"),
        }
    }

    Ok(())
}
