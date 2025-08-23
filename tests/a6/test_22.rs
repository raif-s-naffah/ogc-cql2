// SPDX-License-Identifier: Apache-2.0

//! Test the ACCENTI function with the CASEI function in LIKE predicates
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * The conformance class Case-insensitive Comparison passes.
//!     * The conformance class Advanced Comparison Operators passes.
//! When:
//!     For each queryable {queryable} of type String, evaluate the following
//!     filter expressions
//!     * ACCENTI(CASEI({queryable})) LIKE accenti(casei('Ä%'))
//!     * ACCENTI(CASEI({queryable})) LIKE accenti(casei('a%'))
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets for each queryable are identical;
//! * store the valid predicates for each data source.
//!

use crate::utils::random_unicode_word;
use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome, Q, Resource};
use rand::{Rng, rng};
use std::error::Error;
use tracing_test::traced_test;

const PREFIX: &str = "ä"; // small 'a' w/ umlaut

fn starts_with_prefix() -> String {
    let hit = rng().random_bool(0.5);
    if hit {
        format!("{PREFIX}{}", random_unicode_word())
    } else {
        random_unicode_word()
    }
}

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    const E1: &str = r#"ACCENTI(CASEI(z_string)) LIKE accenti(casei('Ä%'))"#;
    const E2: &str = r#"ACCENTI(CASEI(z_string)) LIKE accenti(casei('a%'))"#;
    const CANDIDATES: [char; 46] = [
        'a', 'à', 'á', 'â', 'ã', 'ä', 'å', 'ā', 'ă', 
        'ą', 'ǎ', 'ȁ', 'ȃ', 'ȧ', 'ḁ', 'ẚ', 'ạ', 'ả',
        'ặ', 'ằ', 'ầ', 'ẩ', 'ǟ',
        'A', 'À', 'Á', 'Â', 'Ã', 'Ä', 'Å', 'Ā', 'Ă',
        'Ą', 'Ǎ', 'Ȁ', 'Ȃ', 'Ȧ', 'Ḁ', 'Ạ', 'Ả', 'Ặ',
        'Ằ', 'Ậ', 'Ầ', 'Ẩ', 'Ǟ',
    ];
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let shared_ctx = Context::new_shared();
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let s = starts_with_prefix();
        let hit = CANDIDATES.contains(&s.chars().nth(0).unwrap());

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
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

    evaluator1.teardown()?;
    evaluator2.teardown()?;

    Ok(())
}
