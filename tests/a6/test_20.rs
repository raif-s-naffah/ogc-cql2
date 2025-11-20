// SPDX-License-Identifier: Apache-2.0

//! Test the ACCENTI function in LIKE predicates
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * The conformance class Advanced Comparison Operators passes.
//! When:
//!     For each queryable {queryable} of type String, evaluate the following
//!     filter expressions
//!     * ACCENTI({queryable}) LIKE accenti('Ä%')
//!     * ACCENTI({queryable}) LIKE accenti('A%')
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets for each queryable are identical;
//! * store the valid predicates for each data source.
//!

use crate::utils::random_unicode_word;
use ogc_cql2::{Context, Evaluator, ExEvaluator, Expression, Outcome, Q, QString, Resource};
use rand::{Rng, rng};
use std::error::Error;
use tracing_test::traced_test;

const PREFIX: &str = "Ä";

fn starts_with_prefix() -> String {
    let hit = rng().random_bool(0.25);
    if hit {
        format!("{PREFIX}{}", random_unicode_word())
    } else {
        random_unicode_word()
    }
}

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    const E1: &str = r#"ACCENTI(z_string) LIKE accenti('Ä%')"#;
    const E2: &str = r#"ACCENTI(z_string) LIKE accenti('A%')"#;
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true, mut expect_false) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let s = starts_with_prefix();
        // a candidate that starts w/ 'A' is a hit...
        let hit = QString::unaccent(&s).starts_with('A');
        if hit {
            expect_true += 1;
        } else {
            expect_false += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_string".into(), Q::new_plain_str(&s)),
        ]);

        let res1 = evaluator1.evaluate(&feat)?;
        match res1 {
            Outcome::T => actual_true1 += 1,
            Outcome::F => actual_false1 += 1,
            Outcome::N => actual_null1 += 1,
        }

        let res2 = evaluator2.evaluate(&feat)?;
        match res2 {
            Outcome::T => actual_true2 += 1,
            Outcome::F => actual_false2 += 1,
            Outcome::N => actual_null2 += 1,
        }
    }

    tracing::debug!("    expect(T/F) = {expect_true}, {expect_false}");
    tracing::debug!("actual #1 (T/F) = {actual_true1}, {actual_false1}");
    tracing::debug!("actual #2 (T/F) = {actual_true2}, {actual_false2}");

    assert_eq!(actual_true1, expect_true);
    assert_eq!(actual_false1, expect_false);
    assert_eq!(actual_null1, 0);
    assert_eq!(
        actual_true1 + actual_false1 + actual_null1,
        expect_true + expect_false
    );

    assert_eq!(actual_true2, expect_true);
    assert_eq!(actual_false2, expect_false);
    assert_eq!(actual_null2, 0);
    assert_eq!(
        actual_true2 + actual_false2 + actual_null2,
        expect_true + expect_false
    );

    Ok(())
}
