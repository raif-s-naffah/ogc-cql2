// SPDX-License-Identifier: Apache-2.0

//! Test the CASEI function in LIKE predicates
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * The conformance class Advanced Comparison Operators passes.
//! When:
//!     For each queryable {queryable} of type String, evaluate the following
//!     filter expressions
//!     * CASEI({queryable}) LIKE casei('foo%')
//!     * CASEI({queryable}) LIKE casei('FOO%')
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets for each queryable are identical;
//! * store the valid predicates for each data source.
//!

use crate::utils::random_ascii_word;
use ogc_cql2::{Context, Evaluator, ExEvaluator, Expression, Outcome, Q, Resource};
use rand::{Rng, rng};
use std::error::Error;

const PREFIX: &str = "Foo";

// Return a string that starts w/ PREFIX 25% of the times.
fn starts_with_prefix() -> String {
    let hit = rng().random_bool(0.25);
    if hit {
        format!("{PREFIX}{}", random_ascii_word())
    } else {
        random_ascii_word()
    }
}

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    const E1: &str = r#"CASEI(z_string) LIKE casei('foo%')"#;
    const E2: &str = r#"CASEI(z_string) LIKE casei('FOO%')"#;
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 1..=1_000 {
        let s = starts_with_prefix();
        let hit = s[0..3].to_lowercase().eq("foo");

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n)?),
            ("z_string".into(), Q::new_plain_str(&s)),
        ]);

        let res1 = evaluator1.evaluate(&feat)?;
        let pass1 = matches!(res1, Outcome::T);
        if pass1 {
            if !hit {
                panic!("Unexpected #1 success :( {feat:?}")
            }
        } else {
            if hit {
                panic!("Unexpected #1 failure :( {feat:?}")
            }
        }

        let res2 = evaluator2.evaluate(&feat)?;
        let pass2 = matches!(res2, Outcome::T);
        if pass2 {
            if !hit {
                panic!("Unexpected #2 success :( {feat:?}")
            }
        } else {
            if hit {
                panic!("Unexpected #2 failure :( {feat:?}")
            }
        }
    }

    Ok(())
}
