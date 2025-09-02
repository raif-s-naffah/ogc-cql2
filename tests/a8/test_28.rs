// SPDX-License-Identifier: Apache-2.0

//! Test the S_INTERSECTS spatial comparison function with points, multi-points,
//! line strings, multi-line string, polygons, multi-polygons, geometry
//! collections and bounding boxes.
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * At least one queryable has a geometry data type.
//! When:
//!     For each queryable {queryable} with a geometry data type, evaluate the
//!     following filter expressions
//!     * S_INTERSECTS({queryable},MULTIPOINT(7.02 49.92, 90 180))
//!     * S_INTERSECTS({queryable},LINESTRING(-180 -45, 0 -45))
//!     * S_INTERSECTS({queryable},MULTILINESTRING((-180 -45, 0 -45), (0 45, 180 45)))
//!     * S_INTERSECTS({queryable},POLYGON(
//!         (-180 -90, -90 -90, -90 90, -180 90, -180 -90),
//!         (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)))
//!     * S_INTERSECTS({queryable},MULTIPOLYGON(
//!         ((-180 -90, -90 -90, -90 90, -180 90, -180 -90),
//!             (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)),
//!         ((0 0, 10 0, 10 10, 0 10, 0 0))))
//!     * S_INTERSECTS({queryable},GEOMETRYCOLLECTION(
//!         POINT(7.02 49.92),
//!         POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))))
//! Then:
//! * assert successful execution of the evaluation for all filter expressions
//!   except the first;
//! * assert unsuccessful execution of the evaluation for the first filter
//!   expressions (invalid coordinate);
//! * store the valid predicates for each data source.
//!
//! **IMPORTANT*: The 1st expression is malformed since a valid multi-point
//! WKT representation for the 2 points in question, following the normative
//! BNF, will look like so `MULTIPOINT((7.02 49.92), (90 180))`.
//!

use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, MyError, Outcome, Q, Resource};
use std::error::Error;
use tracing_test::traced_test;

#[test]
#[traced_test]
#[ignore = "we now allow MULTIPOINT coordinate pairs to be w/o parens"]
fn test_e1_invalid_wkt() {
    // malformed MULTIPOINT WKT according to BNF...
    const E: &str = r#"S_INTERSECTS(geom,MULTIPOINT(7.02 49.92, 90 180))"#;

    let expr = Expression::try_from_text(E);
    assert!(expr.is_err());
}

// the first filter expression that is supposed to fail b/c
#[test]
#[traced_test]
fn test_e1_invalid_coordinates() -> Result<(), Box<dyn Error>> {
    // wellformed MULTIPOINT WKT but using out-of-bounds WGS'84 coordinates.
    const E: &str = r#"S_INTERSECTS(geom,MULTIPOINT((7.02 49.92), (90 180)))"#;

    let expr = Expression::try_from_text(E)?;
    let shared_ctx = Context::try_with_crs("EPSG:4326")?.freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    // single point; one of the pair specified in the expresion.
    let f1 = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("geom".into(), Q::try_from_wkt("POINT(7.02 49.92)")?),
    ]);
    let res = evaluator.evaluate(&f1);
    assert!(res.is_err());
    assert!(matches!(res.err(), Some(MyError::Runtime(_))));

    Ok(())
}

#[test]
#[traced_test]
fn test_e2() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"S_INTERSECTS(geom,LINESTRING(-180 -45, 0 -45))"#;
    const PASS: [&str; 7] = [
        "POINT (-90 -45)",
        "MULTIPOINT ((-90 -45), (-180 -45))",
        "LINESTRING (-180 -45, 0 -45)",
        "MULTILINESTRING ((-180 -45, 0 -45), (-90 -45, -90 -40))",
        "POLYGON ((-180 -45, -90 -45, -90 -40, -180 -40, -180 -45))",
        "MULTIPOLYGON (((-180 -45, -90 -45, -90 -40, -180 -40, -180 -45)))",
        "GEOMETRYCOLLECTION (POINT (-90 -45), LINESTRING (-180 -45, 0 -45))",
    ];
    const FAIL: [&str; 7] = [
        "POINT (0 0)",
        "MULTIPOINT ((0 0), (10 10))",
        "LINESTRING (0 0, 10 10)",
        "MULTILINESTRING ((0 0, 10 10), (20 20, 30 30))",
        "POLYGON ((0 0, 10 0, 10 10, 0 10, 0 0))",
        "MULTIPOLYGON (((0 0, 10 0, 10 10, 0 10, 0 0)))",
        "GEOMETRYCOLLECTION (POINT (0 0), LINESTRING (0 0, 10 10))",
    ];

    let expr = Expression::try_from_text(E)?;
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
            _ => {
                tracing::error!("feat = {feat:?}");
                panic!("Unexpected failure :(")
            }
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
            _ => {
                tracing::error!("feat = {feat:?}");
                panic!("Unexpected success :(")
            }
        }
    }

    Ok(())
}

#[test]
#[traced_test]
fn test_e3() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"S_INTERSECTS(geom,MULTILINESTRING((-180 -45, 0 -45), (0 45, 180 45)))"#;
    const PASS: [&str; 7] = [
        "POINT (0 45)",
        "MULTIPOINT ((0 45), (0 -45))",
        "LINESTRING (0 45, 180 45)",
        "MULTILINESTRING ((0 45, 180 45), (-180 -45, 0 -45))",
        "POLYGON ((0 45, 180 45, 180 50, 0 50, 0 45))",
        "MULTIPOLYGON (((0 45, 180 45, 180 50, 0 50, 0 45)))",
        "GEOMETRYCOLLECTION (POINT (0 45), LINESTRING (0 45, 180 45))",
    ];
    const FAIL: [&str; 6] = [
        "POINT(90 90)",
        "LINESTRING(200 200, 300 300)",
        "POLYGON((60 60, 70 60, 70 70, 60 70, 60 60))",
        "MULTIPOINT((250 250), (260 260))",
        "MULTILINESTRING((80 80, 90 90), (100 100, 110 110))",
        "GEOMETRYCOLLECTION(POINT(300 300), POLYGON((250 250, 260 250, 260 260, 250 260, 250 250)))",
    ];

    let expr = Expression::try_from_text(E)?;
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
            _ => {
                tracing::error!("feat = {feat:?}");
                panic!("Unexpected failure :(")
            }
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
            _ => {
                tracing::error!("feat = {feat:?}");
                panic!("Unexpected success :(")
            }
        }
    }

    Ok(())
}

#[test]
#[traced_test]
fn test_e4() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"S_INTERSECTS(geom,POLYGON((-180 -90, -90 -90, -90 90, -180 90, -180 -90), (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)))"#;
    const PASS: [&str; 6] = [
        "POINT(-110 0)",
        "LINESTRING(-170 0, -100 0)",
        "POLYGON((-130 -40, -110 -40, -110 -30, -130 -30, -130 -40))",
        "MULTIPOINT((-110 -50), (-100 -45))",
        "MULTILINESTRING((-180 -90, -90 -90), (-120 -50, -100 -50))",
        "GEOMETRYCOLLECTION(POINT(-110 -45), POLYGON((-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)))",
    ];
    const FAIL: [&str; 6] = [
        "POINT(90 90)",
        "LINESTRING(100 100, 200 200)",
        "POLYGON((60 60, 70 60, 70 70, 60 70, 60 60))",
        "MULTIPOINT((200 200), (210 210))",
        "MULTILINESTRING((80 80, 90 90), (100 100, 110 110))",
        "GEOMETRYCOLLECTION(POINT(200 200), POLYGON((150 150, 160 150, 160 160, 150 160, 150 150)))",
    ];

    let expr = Expression::try_from_text(E)?;
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
            _ => {
                tracing::error!("feat = {feat:?}");
                panic!("Unexpected failure :(")
            }
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
            _ => {
                tracing::error!("feat = {feat:?}");
                panic!("Unexpected success :(")
            }
        }
    }

    Ok(())
}

#[test]
#[traced_test]
fn test_e5() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"S_INTERSECTS(geom,MULTIPOLYGON(
        ((-180 -90, -90 -90, -90 90, -180 90, -180 -90), 
         (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)),
        ((0 0, 10 0, 10 10, 0 10, 0 0)))
    )"#;
    const PASS: [&str; 6] = [
        "POINT(0 5)",
        "LINESTRING(-10 0, 10 10)",
        "POLYGON((0 0, 5 0, 5 5, 0 5, 0 0))",
        "MULTIPOINT((0 0), (5 5))",
        "MULTILINESTRING((0 0, 10 0), (0 10, 10 10))",
        "GEOMETRYCOLLECTION(POINT(5 5), POLYGON((0 0, 10 0, 10 10, 0 10, 0 0)))",
    ];
    const FAIL: [&str; 6] = [
        "POINT(90 90)",
        "LINESTRING(100 100, 200 200)",
        "POLYGON((60 60, 70 60, 70 70, 60 70, 60 60))",
        "MULTIPOINT((200 200), (210 210))",
        "MULTILINESTRING((80 80, 90 90), (100 100, 110 110))",
        "GEOMETRYCOLLECTION(POINT(200 200), POLYGON((150 150, 160 150, 160 160, 150 160, 150 150)))",
    ];

    let expr = Expression::try_from_text(E)?;
    let shared_ctx = Context::try_with_crs("EPSG:4326")?.freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    for (n, wkt) in PASS.iter().enumerate() {
        let feat = Resource::from([
            ("fid".into(), Q::try_from(n)?),
            ("geom".into(), Q::try_from_wkt(&wkt)?),
        ]);
        // tracing::debug!("feat = {feat:?}");
        let res = evaluator.evaluate(&feat)?;
        match res {
            Outcome::T => (),
            _ => {
                tracing::error!("feat = {feat:?}");
                panic!("Unexpected failure :(")
            }
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
            _ => {
                tracing::error!("feat = {feat:?}");
                panic!("Unexpected success :(")
            }
        }
    }

    Ok(())
}

#[test]
#[traced_test]
fn test_e6() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"S_INTERSECTS(geom,GEOMETRYCOLLECTION(
        POINT(7.02 49.92),
        POLYGON((0 0, 10 0, 10 10, 0 10, 0 0)))
    )"#;
    const PASS: [&str; 6] = [
        "POINT(7.02 49.92)",
        "LINESTRING(0 0, 10 10)",
        "POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))",
        "MULTIPOINT((7.02 49.92), (7.02 50))",
        "MULTILINESTRING((0 0, 10 0), (0 10, 10 10))",
        "GEOMETRYCOLLECTION(POINT(7.02 49.92), POLYGON((0 0, 20 0, 20 20, 0 20, 0 0)))",
    ];
    const FAIL: [&str; 6] = [
        "POINT(90 90)",
        "LINESTRING(100 100, 200 200)",
        "POLYGON((60 60, 70 60, 70 70, 60 70, 60 60))",
        "MULTIPOINT((200 200), (210 210))",
        "MULTILINESTRING((80 80, 90 90), (100 100, 110 110))",
        "GEOMETRYCOLLECTION(POINT(200 200), POLYGON((150 150, 160 150, 160 160, 150 160, 150 150)))",
    ];

    let expr = Expression::try_from_text(E)?;
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
