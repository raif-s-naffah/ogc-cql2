// SPDX-License-Identifier: Apache-2.0

//! Test boolean filter expression
//!
//! Given:
//! * One or more data sources.
//! * Test '/conf/basic-cql2/basic-test' passes.
//!
//! When:
//! For each data source, evaluate the following filter expressions
//!
//! * true
//! * false
//!
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the result sets for false are empty;
//! * store the valid predicates for each data source.
//!

use ogc_cql2::{Context, Evaluator, ExEvaluator, Expression, Outcome, Q, Resource};
use std::error::Error;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    const E1: &str = r#"true"#;
    const E2: &str = r#"FALSE"#;

    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("z_queryable".into(), Q::from(3.14)),
    ]);

    let res1 = evaluator1.evaluate(&feat)?;
    match res1 {
        Outcome::T => (),
        Outcome::F => panic!("Failed F"),
        Outcome::N => panic!("Failed N"),
    }

    let res2 = evaluator2.evaluate(&feat)?;
    match res2 {
        Outcome::T => panic!("Failed T"),
        Outcome::F => (),
        Outcome::N => panic!("Failed N"),
    }

    Ok(())
}
