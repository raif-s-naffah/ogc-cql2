// SPDX-License-Identifier: Apache-2.0

//! Test comparison predicates with values on both sides
//!
//! When:
//!     Evaluate the following filter expressions
//!     * {value} = {value}
//!     * {value} <> {value}
//!     * {value} > {value}
//!     * {value} < {value}
//!     * {value} >= {value}
//!     * {value} <= {value}
//!     for each {value} from the following list:
//!     * 'foo'
//!     * true
//!     * 3.14
//!     * 1
//!     * TIMESTAMP('2022-04-14T14:48:46Z')
//!     * DATE('2022-04-14')
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the result sets for each queryable for the operators <>, <
//!   and > is empty;
//! * assert that the result sets for each queryable for the operators =, >=
//!   and <= are identical;
//! * store the valid predicates for each data source.
//!

use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome, Q, Resource};
use std::error::Error;

#[test]
fn test_string() -> Result<(), Box<dyn Error>> {
    const E1: &str = "'foo' = 'foo'";
    const E2: &str = "'foo' >= 'foo'";
    const E3: &str = "'foo' <= 'foo'";
    const E4: &str = "'foo' <> 'foo'";
    const E5: &str = "'foo' > 'foo'";
    const E6: &str = "'foo' < 'foo'";

    let shared_ctx = Context::new_shared();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = EvaluatorImpl::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = EvaluatorImpl::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    let feat = Resource::from([("fid".into(), Q::try_from(1)?)]);

    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator1.teardown()?;
    evaluator2.teardown()?;
    evaluator3.teardown()?;
    evaluator4.teardown()?;
    evaluator5.teardown()?;
    evaluator6.teardown()?;

    Ok(())
}

#[test]
fn test_number() -> Result<(), Box<dyn Error>> {
    const E1: &str = "3.14 = 3.14";
    const E2: &str = "3.14 >= 3.14";
    const E3: &str = "3.14 <= 3.14";
    const E4: &str = "3.14 <> 3.14";
    const E5: &str = "3.14 > 3.14";
    const E6: &str = "3.14 < 3.14";

    let shared_ctx = Context::new_shared();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = EvaluatorImpl::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = EvaluatorImpl::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    let feat = Resource::from([("fid".into(), Q::try_from(1)?)]);

    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator1.teardown()?;
    evaluator2.teardown()?;
    evaluator3.teardown()?;
    evaluator4.teardown()?;
    evaluator5.teardown()?;
    evaluator6.teardown()?;

    Ok(())
}

#[test]
fn test_integer() -> Result<(), Box<dyn Error>> {
    const E1: &str = "1 = 1";
    const E2: &str = "1 >= 1";
    const E3: &str = "1 <= 1";
    const E4: &str = "1 <> 1";
    const E5: &str = "1 > 1";
    const E6: &str = "1 < 1";

    let shared_ctx = Context::new_shared();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = EvaluatorImpl::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = EvaluatorImpl::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    let feat = Resource::from([("fid".into(), Q::try_from(1)?)]);

    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator1.teardown()?;
    evaluator2.teardown()?;
    evaluator3.teardown()?;
    evaluator4.teardown()?;
    evaluator5.teardown()?;
    evaluator6.teardown()?;

    Ok(())
}

#[test]
fn test_boolean() -> Result<(), Box<dyn Error>> {
    const E1: &str = "true = true";
    const E2: &str = "true >= true";
    const E3: &str = "true <= true";
    const E4: &str = "true <> true";
    const E5: &str = "true > true";
    const E6: &str = "true < true";

    let shared_ctx = Context::new_shared();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = EvaluatorImpl::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = EvaluatorImpl::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    let feat = Resource::from([("fid".into(), Q::try_from(1)?)]);

    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator1.teardown()?;
    evaluator2.teardown()?;
    evaluator3.teardown()?;
    evaluator4.teardown()?;
    evaluator5.teardown()?;
    evaluator6.teardown()?;

    Ok(())
}

#[test]
fn test_timestamp() -> Result<(), Box<dyn Error>> {
    const E1: &str = "TIMESTAMP('2022-04-14T14:48:46Z') = TIMESTAMP('2022-04-14T14:48:46Z')";
    const E2: &str = "TIMESTAMP('2022-04-14T14:48:46Z') >= TIMESTAMP('2022-04-14T14:48:46Z')";
    const E3: &str = "TIMESTAMP('2022-04-14T14:48:46Z') <= TIMESTAMP('2022-04-14T14:48:46Z')";
    const E4: &str = "TIMESTAMP('2022-04-14T14:48:46Z') <> TIMESTAMP('2022-04-14T14:48:46Z')";
    const E5: &str = "TIMESTAMP('2022-04-14T14:48:46Z') > TIMESTAMP('2022-04-14T14:48:46Z')";
    const E6: &str = "TIMESTAMP('2022-04-14T14:48:46Z') < TIMESTAMP('2022-04-14T14:48:46Z')";

    let shared_ctx = Context::new_shared();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = EvaluatorImpl::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = EvaluatorImpl::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    let feat = Resource::from([("fid".into(), Q::try_from(1)?)]);

    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator1.teardown()?;
    evaluator2.teardown()?;
    evaluator3.teardown()?;
    evaluator4.teardown()?;
    evaluator5.teardown()?;
    evaluator6.teardown()?;

    Ok(())
}

#[test]
fn test_date() -> Result<(), Box<dyn Error>> {
    const E1: &str = "DATE('2022-04-14') = DATE('2022-04-14')";
    const E2: &str = "DATE('2022-04-14') >= DATE('2022-04-14')";
    const E3: &str = "DATE('2022-04-14') <= DATE('2022-04-14')";
    const E4: &str = "DATE('2022-04-14') <> DATE('2022-04-14')";
    const E5: &str = "DATE('2022-04-14') > DATE('2022-04-14')";
    const E6: &str = "DATE('2022-04-14') < DATE('2022-04-14')";

    let shared_ctx = Context::new_shared();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = EvaluatorImpl::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = EvaluatorImpl::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = EvaluatorImpl::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    let feat = Resource::from([("fid".into(), Q::try_from(1)?)]);

    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator1.teardown()?;
    evaluator2.teardown()?;
    evaluator3.teardown()?;
    evaluator4.teardown()?;
    evaluator5.teardown()?;
    evaluator6.teardown()?;

    Ok(())
}
