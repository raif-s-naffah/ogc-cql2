// SPDX-License-Identifier: Apache-2.0

//! Test the ACCENTI function in comparisons
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     For each queryable {queryable} of type String, evaluate the following
//!     filter expressions
//!     * ACCENTI({queryable}) = accenti('äöüéáí')
//!     * ACCENTI({queryable}) <> accenti('aoueai')
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets for each queryable have no item in common;
//! * store the valid predicates for each data source.
//!

use crate::utils::random_unicode_word;
use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome, Q, Resource};
use rand::{Rng, rng};
use std::error::Error;

const WORD: &str = "äoüeái";

// Return WORD 25% of the times.
fn random_word() -> String {
    let hit = rng().random_bool(0.25);
    if hit {
        WORD.to_owned()
    } else {
        random_unicode_word()
    }
}

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    const E1: &str = r#"ACCENTI(z_string) = accenti('äöüéáí')"#;
    const E2: &str = r#"ACCENTI(z_string) <> accenti('aoueai')"#;
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let shared_ctx = Context::new_shared();
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);
    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    for n in 0..1000 {
        let s = random_word();
        if s == WORD {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
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

    evaluator1.teardown()?;
    evaluator2.teardown()?;

    // tracing::debug!("#1 expect(T/F) = {expect_true1}, {expect_false1}");
    // tracing::debug!("#2 expect(T/F) = {expect_true2}, {expect_false2}");

    assert_eq!(actual_true1, expect_true1);
    assert_eq!(actual_false1, expect_false1);
    assert_eq!(actual_null1, 0);
    assert_eq!(
        actual_true1 + actual_false1 + actual_null1,
        expect_true1 + expect_false1
    );

    assert_eq!(actual_true2, expect_true2);
    assert_eq!(actual_false2, expect_false2);
    assert_eq!(actual_null2, 0);
    assert_eq!(
        actual_true2 + actual_false2 + actual_null2,
        expect_true2 + expect_false2
    );

    Ok(())
}
