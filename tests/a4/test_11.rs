// SPDX-License-Identifier: Apache-2.0

//! Test BETWEEN predicate
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//! for each queryable {queryable} of type Number or Integer, evaluate the
//! following filter expressions:
//!     * {queryable} BETWEEN 0 AND 100
//!     * {queryable} between 100.0 and 1.0
//! Then:
//! * assert successful execution of the evaluation;
//! *store the valid predicates for each data source.
//!

use ogc_cql2::{Context, Evaluator, ExEvaluator, Expression, Outcome, Q, Resource};
use rand::{Rng, distr::Uniform};
use std::error::Error;

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    const E1: &str = r#"foo BETWEEN 0 AND 100"#;
    const E2: &str = r#"foo between 100.0 and 1.0"#;

    let mut rng = rand::rng();
    let shared_ctx = Context::new().freeze();

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    let input1 = format!("{E1}");
    let exp1 = Expression::try_from_text(&input1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    let input2 = format!("{E2}");
    let exp2 = Expression::try_from_text(&input2)?;

    evaluator1.setup(exp1)?;
    evaluator2.setup(exp2)?;

    let (mut expected, mut actual1, mut actual2) = (0, 0, 0);
    // uniform distribution of random unsigned ints w/in 1..=200
    let dist = Uniform::new(1, 201)?;
    for ndx in 1..=1000 {
        let n = rng.sample(dist);
        if (0..=100).contains(&n) {
            expected += 1;
        }
        let feat = Resource::from([
            ("fid".into(), Q::try_from(ndx)?),
            ("foo".into(), Q::try_from(n)?),
        ]);

        let res1 = evaluator1.evaluate(&feat)?;
        match res1 {
            Outcome::T => actual1 += 1,
            _ => (),
        }
        let res2 = evaluator2.evaluate(&feat)?;
        match res2 {
            Outcome::T => actual2 += 1,
            _ => (),
        }
    }

    assert_eq!(actual1, expected);
    assert_eq!(actual2, expected);
    Ok(())
}
