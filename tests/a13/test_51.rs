// SPDX-License-Identifier: Apache-2.0

//! Test predicates with functions
//!
//! Given:
//!     * The list of functions with arguments and return type supported by the
//!       implementation under test.
//! When:
//!     For each function construct multiple valid filter expressions involving
//!     different operators.
//! Then:
//! * assert successful execution of the evaluation.
//!

use ogc_cql2::{Context, Evaluator, ExEvaluator, Expression, Outcome, Q, Resource};
use std::error::Error;

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    let mut ctx = Context::new();
    ctx.register_builtins();
    let shared_ctx = ctx.freeze();

    let expr = Expression::try_from_text("min(a, b) + max(a, b) = 2 * avg(a, b)")?;
    let mut eval = ExEvaluator::new(shared_ctx);
    eval.setup(expr)?;

    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("a".into(), Q::try_from(10)?),
        ("b".into(), Q::try_from(20)?),
    ]);

    let res = eval.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    Ok(())
}
