// SPDX-License-Identifier: Apache-2.0

//! Test the S_INTERSECTS spatial function
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * At least one queryable has a geometry data type.
//! When:
//!     For each queryable {queryable} with a geometry data type, evaluate the
//!     following filter expressions
//!     * S_INTERSECTS({queryable},BBOX(-180,-90,180,90))
//!     * S_INTERSECTS({queryable},POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))
//!     * S_INTERSECTS({queryable},LINESTRING(7 50, 10 51))
//!     * S_INTERSECTS({queryable},POINT(7.02 49.92))
//!     * S_INTERSECTS({queryable},POINT(90 180))
//! Then:
//! * assert successful execution of the evaluation for the first four filter
//!   expressions;
//! * assert unsuccessful execution of the evaluation for the fifth filter
//!   expressions (invalid coordinate);
//! * assert that the two result sets of the first two filter expressions for
//!   each queryable are identical;
//! * store the valid predicates for each data source.
//!

use crate::utils::{COUNTRIES, harness};
use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, MyError, Q, Resource};
use std::error::Error;
use tracing_test::traced_test;

// Countries CSV data set contains 177 records...
#[rustfmt::skip]
const PREDICATES: [(&str, u32); 4] = [
    ("S_INTERSECTS(geom,BBOX(-180,-90,180,90))",                               177),
    ("S_INTERSECTS(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))", 177),
    ("S_INTERSECTS(geom,LINESTRING(7 50, 10 51))",                               1),
    ("S_INTERSECTS(geom,POINT(7.02 49.92))",                                     1),
];

#[test]
#[traced_test]
fn test_invalid_coordinates() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"S_INTERSECTS(geom,POINT(90 180))"#;

    let expr = Expression::try_from_text(E)?;
    let shared_ctx = Context::new_shared();
    let mut evaluator = EvaluatorImpl::new(shared_ctx);
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

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    harness(COUNTRIES, PREDICATES.to_vec())
}
