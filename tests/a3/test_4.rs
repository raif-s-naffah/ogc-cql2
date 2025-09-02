// SPDX-License-Identifier: Apache-2.0

//! Implementation under test provides sufficient information to construct
//! filter expressions and supports comparison predicates.
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     The pattern used here is to assert outcome using a known CQL2 filter
//!     and a set of Features that reult in all 3 types of outcome. Each test
//!     checks a certain data-type.
//! Then:
//! * assert that there is at least one queryable for each data source;
//! * assert that the data type (String, Number, Integer, Boolean,
//!   Timestamp, Date, Interval, Point, MultiPoint, LineString,
//!   MultiLineString, Polygon, MultiPolygon, Geometry, GeometryCollection,
//!   or Array) is specified for each queryable;
//! * assert that at least one queryable for each data source is of data
//!   type String, Boolean, Number, Integer, Timestamp or Date.
//!

use ogc_cql2::{Bound, Context, Evaluator, EvaluatorImpl, Expression, Outcome, Q, Resource};
use std::error::Error;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test_boolean() -> Result<(), Box<dyn Error>> {
    const F: &str = r#"foo < 0.15 AND sat:row=13 AND sat:path=True"#;

    // start by parsing the test vector...
    let expr = Expression::try_from_text(F)?;
    // instantiate an evaluator + set it up...
    let shared_ctx = Context::new().freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    // make 3 features w/ the correct named property and data-type.
    // this one should pass...
    let f1 = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("foo".into(), Q::from(0.02)),
        ("sat:row".into(), Q::try_from(13)?),
        ("sat:path".into(), Q::from(true)),
    ]);
    let res = evaluator.evaluate(&f1)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::T));

    // this one should fail...
    let f2 = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("foo".into(), Q::from(0.02)),
        ("sat:row".into(), Q::try_from(13)?),
        ("sat:path".into(), Q::from(false)),
    ]);
    let res = evaluator.evaluate(&f2)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::F));

    // this one should fail w/ a N outcome, since the property-name used in the
    // filter is not defined ...
    let f3 = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("foo".into(), Q::from(0.02)),
        ("sat:row".into(), Q::try_from(13)?),
        ("sat:paht".into(), Q::from(true)),
    ]);
    let res = evaluator.evaluate(&f3)?;
    tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::N));

    evaluator.teardown()?;
    Ok(())
}

#[test]
#[traced_test]
fn test_num() -> Result<(), Box<dyn Error>> {
    const F: &str = r#"foo:x IN (0.1,0.2)"#;

    let expr = Expression::try_from_text(F)?;
    let shared_ctx = Context::new().freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    let f1 = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("foo:x".into(), Q::from(0.1)),
    ]);
    let res = evaluator.evaluate(&f1)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::T));

    let f2 = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("foo:x".into(), Q::from(0.3)),
    ]);
    let res = evaluator.evaluate(&f2)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::F));

    let f3 = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("foo:y".into(), Q::from(0.1)),
    ]);
    let res = evaluator.evaluate(&f3)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::N));

    evaluator.teardown()?;
    Ok(())
}

#[test]
#[traced_test]
fn test_string() -> Result<(), Box<dyn Error>> {
    const F: &str = r#"foo:id = 'bonza'"#;

    let expr = Expression::try_from_text(F)?;
    let shared_ctx = Context::new().freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    let f1 = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("foo:id".into(), Q::new_plain_str("bonza")),
    ]);
    let res = evaluator.evaluate(&f1)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::T));

    let f2 = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("foo:id".into(), Q::new_plain_str("wtf")),
    ]);
    let res = evaluator.evaluate(&f2)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::F));

    let f3 = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("bar:id".into(), Q::new_plain_str("bonza")),
    ]);
    let res = evaluator.evaluate(&f3)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::N));

    evaluator.teardown()?;
    Ok(())
}

#[test]
#[traced_test]
fn test_instant() -> Result<(), Box<dyn Error>> {
    const F: &str = r#"T_Before(built, date('2025-07-14'))"#;

    let expr = Expression::try_from_text(F)?;
    let shared_ctx = Context::new().freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    const D1: &str = "2015-01-01";
    const D2: &str = "2025-07-14";

    let f1 = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("built".into(), Q::from(Bound::try_new_date(D1)?)),
    ]);
    let res = evaluator.evaluate(&f1)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::T));

    let f2 = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("built".into(), Q::from(Bound::try_new_date(D2)?)),
    ]);
    let res = evaluator.evaluate(&f2)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::F));

    let f3 = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("build".into(), Q::from(Bound::try_new_date(D1)?)),
    ]);
    let res = evaluator.evaluate(&f3)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::N));

    evaluator.teardown()?;
    Ok(())
}

#[test]
#[traced_test]
fn test_interval() -> Result<(), Box<dyn Error>> {
    const F: &str =
        r#"T_During(interval(a, b), INTERVAL('2017-06-10T07:30:00Z','2017-06-11T10:30:00Z'))"#;

    let expr = Expression::try_from_text(F)?;
    let shared_ctx = Context::new().freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    const T1: &str = "2017-06-11T08:30:00Z";
    const T2: &str = "2017-06-11T09:30:00Z[UTC]";
    const T3: &str = "2017-06-09T08:30:00Z[UTC]";

    let f1 = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("a".into(), Q::from(Bound::try_new_timestamp(T1)?)),
        ("b".into(), Q::from(Bound::try_new_timestamp(T2)?)),
    ]);
    let res = evaluator.evaluate(&f1)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::T));

    let f2 = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("a".into(), Q::from(Bound::try_new_timestamp(T3)?)),
        ("b".into(), Q::from(Bound::try_new_timestamp(T2)?)),
    ]);
    let res = evaluator.evaluate(&f2)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::F));

    let f3 = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("a".into(), Q::from(Bound::try_new_timestamp(T2)?)),
        ("c".into(), Q::from(Bound::try_new_timestamp(T3)?)),
    ]);
    let res = evaluator.evaluate(&f3)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::N));

    evaluator.teardown()?;
    Ok(())
}

#[test]
#[traced_test]
fn test_point_and_polygon() -> Result<(), Box<dyn Error>> {
    const F: &str = r#"S_WITHIN("geom", POLYGON((-65.887123 2.00001, 0.333333 -53.017711, 180.0 0.0, -65.887123 2.00001), (-49.88024 0.5, -1.5 -0.99999, 0.0 0.5, -49.88024 0.5)))"#;

    let expr = Expression::try_from_text(F)?;
    let shared_ctx = Context::new().freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    const BALSAS: &str = "POINT(-46.03556 -7.5325)";
    let f1 = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("geom".into(), Q::try_from_wkt(BALSAS)?),
    ]);
    let res = evaluator.evaluate(&f1)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::T));

    const SYDNEY: &str = "POINT(151.22322000 -33.74821000)";
    let f2 = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("geom".into(), Q::try_from_wkt(SYDNEY)?),
    ]);
    let res = evaluator.evaluate(&f2)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::F));

    let f3 = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("geon".into(), Q::try_from_wkt("POINT(1.0 1.0 1.0)")?),
    ]);
    let res = evaluator.evaluate(&f3)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::N));

    evaluator.teardown()?;
    Ok(())
}

#[test]
#[traced_test]
fn test_line() -> Result<(), Box<dyn Error>> {
    const F: &str = r#"s_crosses(LineString(-65.0 2.0, 0.33 -53.017, 90.0 0.0), "geom")"#;

    let expr = Expression::try_from_text(F)?;
    let shared_ctx = Context::new().freeze();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
    evaluator.setup(expr)?;

    const L1: &str = "LINESTRING(-46.03556 -7.5325, 151.22322 -33.74821)";
    let f1 = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("geom".into(), Q::try_from_wkt(L1)?),
    ]);
    let res = evaluator.evaluate(&f1)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::T));

    const L2: &str = "LINESTRING(0.0 0.0, 10.0 -43.74821)";
    let f2 = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("geom".into(), Q::try_from_wkt(L2)?),
    ]);
    let res = evaluator.evaluate(&f2)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::F));

    let f3 = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("geon".into(), Q::try_from_wkt("POINT(1.0 1.0 1.0)")?),
    ]);
    let res = evaluator.evaluate(&f3)?;
    // tracing::debug!("res = {res}");
    assert!(matches!(res, Outcome::N));

    evaluator.teardown()?;
    Ok(())
}
