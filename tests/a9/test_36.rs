// SPDX-License-Identifier: Apache-2.0

//! Test the S_CONTAINS spatial function
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     for each queryable {queryable} with a geometry data type, evaluate the
//!     following filter expressions
//!     * S_CONTAINS({queryable},BBOX(-180,-90,180,90))
//!     * S_CONTAINS({queryable},POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))
//!     * S_CONTAINS({queryable},LINESTRING(7 50,10 51))
//!     * S_CONTAINS({queryable},MULTIPOINT(7 50,10 51))
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets of the first two filter expressions for
//!   each queryable are identical;
//! * assert that the results sets for each queryable do not have an item in
//!   common with the corresponding S_WITHIN expression;
//! * store the valid predicates for each data source.
//!

use crate::utils::{COUNTRIES, countries, harness};
use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome};
use std::error::Error;
use tracing_test::traced_test;

// Countries CSV data set contains 177 records...
#[rustfmt::skip]
const CONTAINS: [(&str, u32); 4] = [
    ("S_CONTAINS(geom,BBOX(-180,-90,180,90))",                               0),
    ("S_CONTAINS(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))", 0),
    ("S_CONTAINS(geom,LINESTRING(7 50,10 51))",                              1),
    ("S_CONTAINS(geom,MULTIPOINT((7 50),(10 51)))",                          1),
];

#[rustfmt::skip]
const WITHIN: [(&str, u32); 2] = [
    ("S_WITHIN(geom,BBOX(-180,-90,180,90))",                               177),
    ("S_WITHIN(geom,POLYGON((-180 -90,180 -90,180 90,-180 90,-180 -90)))", 177),
];

#[test]
#[traced_test]
fn test_contains() -> Result<(), Box<dyn Error>> {
    harness(COUNTRIES, CONTAINS.to_vec())
}

#[test]
#[traced_test]
fn test_within() -> Result<(), Box<dyn Error>> {
    harness(COUNTRIES, WITHIN.to_vec())
}

#[test]
#[traced_test]
fn test_e3() -> Result<(), Box<dyn Error>> {
    const E1: &str = "S_CONTAINS(geom,LINESTRING(7 50,10 51))";
    const E2: &str = "S_WITHIN(geom,LINESTRING(7 50,10 51))";

    let shared_ctx = Context::new_shared();
    let expr1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let expr2 = Expression::try_from_text(E2)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for c in countries()? {
        // any geometry cannot satisfy both expressions simultaneously
        let res1 = evaluator1.evaluate(&c)?;
        let res2 = evaluator2.evaluate(&c)?;
        match (res1, res2) {
            (Outcome::T, Outcome::T) => {
                panic!("Unexpected result: {c:?}")
            }
            _ => (),
        }
    }

    Ok(())
}

#[test]
#[traced_test]
fn test_e4() -> Result<(), Box<dyn Error>> {
    const E1: &str = "S_CONTAINS(geom,MULTIPOINT((7 50),(10 51)))";
    const E2: &str = "S_WITHIN(geom,MULTIPOINT((7 50),(10 51)))";

    let shared_ctx = Context::new_shared();
    let expr1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let expr2 = Expression::try_from_text(E2)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for c in countries()? {
        // any geometry cannot satisfy both expressions simultaneously
        let res1 = evaluator1.evaluate(&c)?;
        let res2 = evaluator2.evaluate(&c)?;
        match (res1, res2) {
            (Outcome::T, Outcome::T) => {
                panic!("Unexpected result: {c:?}")
            }
            _ => (),
        }
    }

    Ok(())
}
