// SPDX-License-Identifier: Apache-2.0

//! Test the S_TOUCHES spatial function
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     for each queryable {queryable} with a geometry data type, evaluate the
//!     following filter expressions
//!     * S_TOUCHES({queryable},BBOX(-180,-90,180,90))
//!     * S_TOUCHES({queryable},POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))
//!     * S_TOUCHES({queryable},LINESTRING(7 50,10 51))
//! Then:
//! * assert successful execution of the evaluation;
//! * store the valid predicates for each data source.
//!

use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome, Q, Resource};
use std::error::Error;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test_bbox() -> Result<(), Box<dyn Error>> {
    const E: &str = "S_TOUCHES(geom,BBOX(-180,-90,180,90))";
    const PASS: [&str; 2] = [
        "POINT(180 -90)",
        "LINESTRING(180 -90, 180 0)",
        // "POLYGON((180 -90, 180 0, 0 0, 0 -90, 180 -90))",
        // "MULTIPOINT((180 -90), (0 0))",
        // "MULTILINESTRING((180 -90, 180 0), (0 0, 0 -90))",
        // "MULTIPOLYGON(((180 -90, 180 0, 0 0, 0 -90, 180 -90)), ((0 0, 0 90, 90 90, 90 0, 0 0)))",
        // "GEOMETRYCOLLECTION(POINT(180 -90), LINESTRING(0 0, 0 -90))",
    ];
    const FAIL: [&str; 7] = [
        "POINT(0 0)",
        "LINESTRING(0 0, 0 90)",
        "POLYGON((0 0, 0 90, 90 90, 90 0, 0 0))",
        "MULTIPOINT((0 0), (90 90))",
        "MULTILINESTRING((0 0, 0 90), (90 90, 90 0))",
        "MULTIPOLYGON(((0 0, 0 90, 90 90, 90 0, 0 0)), ((90 0, 90 90, 180 90, 180 0, 90 0)))",
        "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(90 90, 90 0))",
    ];

    let expr = Expression::try_from_text(E)?;
    // tracing::debug!("expr = {expr:?}");
    let shared_ctx = Context::try_with_crs("EPSG:4326")?.freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    for (n, wkt) in PASS.iter().enumerate() {
        let feat = Resource::from([
            ("fid".into(), Q::try_from(n)?),
            ("geom".into(), Q::try_from_wkt(&wkt)?),
        ]);
        let res = evaluator.evaluate(&feat)?;
        match res {
            Outcome::T => (),
            _ => panic!("Unexpected failure :( {feat:?}"),
        }
    }

    for (n, wkt) in FAIL.iter().enumerate() {
        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + PASS.len())?),
            ("geom".into(), Q::try_from_wkt(&wkt)?),
        ]);
        let res = evaluator.evaluate(&feat)?;
        match res {
            Outcome::F => (),
            _ => panic!("Unexpected success :( {feat:?}"),
        }
    }

    Ok(())
}

#[test]
#[traced_test]
fn test_polygon() -> Result<(), Box<dyn Error>> {
    const E: &str = "S_TOUCHES(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))";
    const PASS: [&str; 2] = [
        "POINT(180 -90)",
        "LINESTRING(180 -90, 180 0)",
        // "POLYGON((180 -90, 180 0, 0 0, 0 -90, 180 -90))",
        // "MULTIPOINT((180 -90), (0 0))",
        // "MULTILINESTRING((180 -90, 180 0), (0 0, 0 -90))",
        // "MULTIPOLYGON(((180 -90, 180 0, 0 0, 0 -90, 180 -90)), ((0 0, 0 90, 90 90, 90 0, 0 0)))",
        // "GEOMETRYCOLLECTION(POINT(180 -90), LINESTRING(0 0, 0 -90))",
    ];
    const FAIL: [&str; 7] = [
        "POINT(0 0)",
        "LINESTRING(0 0, 0 90)",
        "POLYGON((0 0, 0 90, 90 90, 90 0, 0 0))",
        "MULTIPOINT((0 0), (90 90))",
        "MULTILINESTRING((0 0, 0 90), (90 90, 90 0))",
        "MULTIPOLYGON(((0 0, 0 90, 90 90, 90 0, 0 0)), ((90 0, 90 90, 180 90, 180 0, 90 0)))",
        "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(90 90, 90 0))",
    ];

    let expr = Expression::try_from_text(E)?;
    // tracing::debug!("expr = {expr:?}");
    let shared_ctx = Context::try_with_crs("EPSG:4326")?.freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    for (n, wkt) in PASS.iter().enumerate() {
        let feat = Resource::from([
            ("fid".into(), Q::try_from(n)?),
            ("geom".into(), Q::try_from_wkt(&wkt)?),
        ]);
        let res = evaluator.evaluate(&feat)?;
        match res {
            Outcome::T => (),
            _ => panic!("Unexpected failure :( {feat:?}"),
        }
    }

    for (n, wkt) in FAIL.iter().enumerate() {
        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + PASS.len())?),
            ("geom".into(), Q::try_from_wkt(&wkt)?),
        ]);
        let res = evaluator.evaluate(&feat)?;
        match res {
            Outcome::F => (),
            _ => panic!("Unexpected success :( {feat:?}"),
        }
    }

    Ok(())
}

#[test]
#[traced_test]
fn test_line() -> Result<(), Box<dyn Error>> {
    const E: &str = "S_TOUCHES(geom,LINESTRING(7 50,10 51))";
    const PASS: [&str; 7] = [
        "POINT(10 51)",
        "LINESTRING(5 49, 10 51)",
        "POLYGON((5 49, 10 51, 15 49, 10 47, 5 49))",
        "MULTIPOINT((10 51), (5 49))",
        "MULTILINESTRING((5 49, 10 51), (10 51, 15 53))",
        "MULTIPOLYGON(((5 49, 10 51, 15 49, 10 47, 5 49)), ((10 51, 15 53, 20 51, 15 49, 10 51)))",
        "GEOMETRYCOLLECTION(POINT(10 51), LINESTRING(5 49, 10 51))",
    ];
    const FAIL: [&str; 7] = [
        "POINT(0 0)",
        "LINESTRING(0 0, 5 45)",
        "POLYGON((0 0, 5 45, 10 40, 0 40, 0 0))",
        "MULTIPOINT((0 0), (5 45))",
        "MULTILINESTRING((0 0, 5 45), (15 55, 20 60))",
        "MULTIPOLYGON(((0 0, 5 45, 10 40, 0 40, 0 0)), ((15 55, 20 60, 25 55, 20 50, 15 55)))",
        "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(15 55, 20 60))",
    ];

    let expr = Expression::try_from_text(E)?;
    // tracing::debug!("expr = {expr:?}");
    let shared_ctx = Context::try_with_crs("EPSG:4326")?.freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    for (n, wkt) in PASS.iter().enumerate() {
        let feat = Resource::from([
            ("fid".into(), Q::try_from(n)?),
            ("geom".into(), Q::try_from_wkt(&wkt)?),
        ]);
        let res = evaluator.evaluate(&feat)?;
        match res {
            Outcome::T => (),
            _ => panic!("Unexpected failure :( {feat:?}"),
        }
    }

    for (n, wkt) in FAIL.iter().enumerate() {
        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + PASS.len())?),
            ("geom".into(), Q::try_from_wkt(&wkt)?),
        ]);
        let res = evaluator.evaluate(&feat)?;
        match res {
            Outcome::F => (),
            _ => panic!("Unexpected success :( {feat:?}"),
        }
    }

    Ok(())
}
