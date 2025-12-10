// SPDX-License-Identifier: Apache-2.0

//! Test the S_CROSSES spatial function
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     for each queryable {queryable} of type Point, MultiPoint, LineString or
//!     MultiLineString, evaluate the following filter expressions
//!     * S_CROSSES({queryable},BBOX(-180,-90,180,90))
//!     * S_CROSSES({queryable},POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))
//!     * S_CROSSES({queryable},LINESTRING(7 50,10 51))
//! Then:
//! * assert successful execution of the evaluation;
//! * store the valid predicates for each data source.
//!

use ogc_cql2::{Context, Evaluator, ExEvaluator, Expression, Outcome, Q, Resource};
use std::error::Error;

#[test]
fn test_bbox() -> Result<(), Box<dyn Error>> {
    const E: &str = "S_CROSSES(geom,BBOX(-180,-90,180,90))";
    // no geometry crosses the WGS'84 bounding box...
    // const PASS: [&str; 2] = [
    //     "MULTILINESTRING((160 0, 180 0), (-180 0, -160 0))",
    //     "GEOMETRYCOLLECTION(POINT(180 -90), MULTILINESTRING((160 0, 180 0), (-180 0, -160 0))",
    // ];
    const FAIL: [&str; 7] = [
        "POINT(180 90)",
        "LINESTRING(160 0, -160 0)",
        "POLYGON((0 0, 0 90, 90 90, 90 0, 0 0))",
        "MULTIPOINT((0 0), (90 90))",
        "MULTILINESTRING((0 0, 0 90), (90 90, 90 0))",
        "MULTIPOLYGON(((0 0, 0 90, 90 90, 90 0, 0 0)), ((90 0, 90 90, 180 90, 180 0, 90 0)))",
        "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(90 90, 90 0))",
    ];

    let expr = Expression::try_from_text(E)?;
    let shared_ctx = Context::try_with_crs("EPSG:4326")?.freeze();
    let mut evaluator = ExEvaluator::new(shared_ctx);
    evaluator.setup(expr)?;

    for (n, wkt) in FAIL.iter().enumerate() {
        let feat = Resource::from([
            ("fid".into(), Q::try_from(n)?),
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
fn test_polygon() -> Result<(), Box<dyn Error>> {
    const E: &str = "S_CROSSES(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))";
    // no geometry crosses the WGS'84 extent...
    // const PASS: [&str; 2] = [
    //     "MULTILINESTRING((160 0, 180 0), (-180 0, -160 0))",
    //     "GEOMETRYCOLLECTION(POINT(180 -90), MULTILINESTRING((160 0, 180 0), (-180 0, -160 0))",
    // ];
    const FAIL: [&str; 7] = [
        "POINT(180 90)",
        "LINESTRING(160 0, -160 0)",
        "POLYGON((0 0, 0 90, 90 90, 90 0, 0 0))",
        "MULTIPOINT((0 0), (90 90))",
        "MULTILINESTRING((0 0, 0 90), (90 90, 90 0))",
        "MULTIPOLYGON(((0 0, 0 90, 90 90, 90 0, 0 0)), ((90 0, 90 90, 180 90, 180 0, 90 0)))",
        "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(90 90, 90 0))",
    ];

    let expr = Expression::try_from_text(E)?;
    let shared_ctx = Context::try_with_crs("EPSG:4326")?.freeze();
    let mut evaluator = ExEvaluator::new(shared_ctx);
    evaluator.setup(expr)?;

    for (n, wkt) in FAIL.iter().enumerate() {
        let feat = Resource::from([
            ("fid".into(), Q::try_from(n)?),
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
fn test_line() -> Result<(), Box<dyn Error>> {
    const E: &str = "S_CROSSES(geom,LINESTRING(7 50,10 51))";
    const PASS: [&str; 6] = [
        "LINESTRING(6 49, 11 52)",
        "POLYGON((6 49, 6 51, 9 50, 9 49, 6 49))",
        "MULTIPOINT((0 0), (8.5 50.5))",
        "MULTILINESTRING((0 0, 0 90), (6 49, 11 52))",
        "MULTIPOLYGON(((0 0, 0 90, 90 90, 90 0, 0 0)), ((6 49, 6 51, 9 50, 9 49, 6 49)))",
        "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(6 49, 11 52))",
    ];
    const FAIL: [&str; 7] = [
        "POINT(7 50)",
        "LINESTRING(160 0, -160 0)",
        "POLYGON((0 0, 0 90, 90 90, 90 0, 0 0))",
        "MULTIPOINT((0 0), (90 90))",
        "MULTILINESTRING((0 0, 0 90), (90 90, 90 0))",
        "MULTIPOLYGON(((0 0, 0 90, 90 90, 90 0, 0 0)), ((90 0, 90 90, 180 90, 180 0, 90 0)))",
        "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(90 90, 90 0))",
    ];

    let expr = Expression::try_from_text(E)?;
    let shared_ctx = Context::try_with_crs("EPSG:4326")?.freeze();
    let mut evaluator = ExEvaluator::new(shared_ctx);
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
