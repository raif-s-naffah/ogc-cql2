// SPDX-License-Identifier: Apache-2.0

//! Test IS NULL predicate.
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * Test '/conf/basic-cql2/basic-test' passes.
//! When:
//!     For each queryable {queryable}, evaluate the following filter
//!     expressions
//!         * {queryable} IS NULL
//!         * {queryable} is not null
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets for each queryable have no item in common;
//! * store the valid predicates for each data source.
//!

use ogc_cql2::{Context, Evaluator, ExEvaluator, Expression, Outcome, Q, Resource};
use rand::Rng;
use std::error::Error;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    let mut rng = rand::rng();
    const E1: &str = r#"this is NOT null"#;
    const E2: &str = r#"this IS Null"#;

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    let expr1 = Expression::try_from_text(E1)?;
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    let expr2 = Expression::try_from_text(E2)?;
    evaluator2.setup(expr2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1) = (0, 0);
    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2) = (0, 0);
    for n in 0..1000 {
        let b = rng.random_bool(0.5);
        let feat = if b {
            expect_true1 += 1;
            expect_false2 += 1;
            Resource::from([
                ("fid".into(), Q::try_from(n + 1)?),
                ("this".into(), Q::from(3.14)),
            ])
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
            Resource::from([
                ("fid".into(), Q::try_from(n + 1)?),
                ("that".into(), Q::from(3.14)),
            ])
        };

        let res1 = evaluator1.evaluate(&feat)?;
        match res1 {
            Outcome::T => actual_true1 += 1,
            _ => actual_false1 += 1,
        }
        let res2 = evaluator2.evaluate(&feat)?;
        match res2 {
            Outcome::T => actual_true2 += 1,
            _ => actual_false2 += 1,
        }
    }

    tracing::debug!("#1 expect(T/F) = {expect_true1}, {expect_false1}");
    tracing::debug!("#2 expect(T/F) = {expect_true2}, {expect_false2}");

    assert_eq!(actual_true1, expect_true1);
    assert_eq!(actual_false1, expect_false1);
    assert_eq!(actual_true1 + actual_false1, expect_true1 + expect_false1);

    assert_eq!(actual_true2, expect_true2);
    assert_eq!(actual_false2, expect_false2);
    assert_eq!(actual_true2 + actual_false2, expect_true2 + expect_false2);

    Ok(())
}
