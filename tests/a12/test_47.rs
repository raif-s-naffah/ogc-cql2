// SPDX-License-Identifier: Apache-2.0

//! Test comparison predicates with properties on both sides
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     For each queryable {queryable} of one of the data types String,
//!     Boolean, Number, Integer, Timestamp or Date, evaluate the following
//!     filter expressions
//!     * {queryable} = {queryable}
//!     * {queryable} <> {queryable}
//!     * {queryable} > {queryable}
//!     * {queryable} < {queryable}
//!     * {queryable} >= {queryable}
//!     * {queryable} <= {queryable}
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

const E1: &str = "x = var";
const E2: &str = "x <> var";
const E3: &str = "x > var";
const E4: &str = "x <= var";
const E5: &str = "x < var";
const E6: &str = "x >= var";

#[test]
fn test_string() -> Result<(), Box<dyn Error>> {
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

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("x".into(), Q::new_plain_str("foo")),
        ("var".into(), Q::new_plain_str("foo")),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("x".into(), Q::new_plain_str("foo")),
        ("var".into(), Q::new_plain_str("bar")),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("x".into(), Q::new_plain_str("foo")),
        ("var".into(), Q::new_plain_str("aaa")),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(4)?),
        ("x".into(), Q::new_plain_str("foo")),
        ("var".into(), Q::new_plain_str("zzz")),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
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

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("x".into(), Q::try_from(3.14)?),
        ("var".into(), Q::try_from(3.14)?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("x".into(), Q::try_from(3.14)?),
        ("var".into(), Q::try_from(42)?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("x".into(), Q::try_from(3.14)?),
        ("var".into(), Q::try_from(1.4142)?),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(4)?),
        ("x".into(), Q::try_from(3.14)?),
        ("var".into(), Q::try_from(99.9)?),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
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

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("x".into(), Q::try_from(1)?),
        ("var".into(), Q::try_from(1)?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("x".into(), Q::try_from(1)?),
        ("var".into(), Q::try_from(0)?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("x".into(), Q::try_from(1)?),
        ("var".into(), Q::try_from(-1)?),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(4)?),
        ("x".into(), Q::try_from(1)?),
        ("var".into(), Q::try_from(2)?),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
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

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("x".into(), Q::Bool(true)),
        ("var".into(), Q::Bool(true)),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("x".into(), Q::Bool(true)),
        ("var".into(), Q::Bool(false)),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("x".into(), Q::Bool(true)),
        ("var".into(), Q::Bool(true)),
    ]);
    // for booleans it's a bit more complicated...
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(4)?),
        ("x".into(), Q::Bool(true)),
        ("var".into(), Q::Bool(false)),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

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

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        (
            "x".into(),
            Q::try_from_timestamp_str("2022-04-14T14:48:46Z")?,
        ),
        (
            "var".into(),
            Q::try_from_timestamp_str("2022-04-14T14:48:46Z")?,
        ),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        (
            "x".into(),
            Q::try_from_timestamp_str("2022-04-14T14:48:46Z")?,
        ),
        (
            "var".into(),
            Q::try_from_timestamp_str("2022-04-14T14:48:47Z")?,
        ),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        (
            "x".into(),
            Q::try_from_timestamp_str("2022-04-14T14:48:46Z")?,
        ),
        (
            "var".into(),
            Q::try_from_timestamp_str("2012-04-14T14:48:46Z")?,
        ),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(4)?),
        (
            "x".into(),
            Q::try_from_timestamp_str("2022-04-14T14:48:46Z")?,
        ),
        (
            "var".into(),
            Q::try_from_timestamp_str("2025-08-17T14:48:46Z")?,
        ),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
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

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("x".into(), Q::try_from_date_str("2022-04-14")?),
        ("var".into(), Q::try_from_date_str("2022-04-14")?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("x".into(), Q::try_from_date_str("2022-04-14")?),
        ("var".into(), Q::try_from_date_str("2025-08-18")?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
        ("x".into(), Q::try_from_date_str("2022-04-14")?),
        ("var".into(), Q::try_from_date_str("2012-04-14")?),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator6.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(4)?),
        ("x".into(), Q::try_from_date_str("2022-04-14")?),
        ("var".into(), Q::try_from_date_str("2022-04-15")?),
    ]);
    let res = evaluator3.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator4.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator5.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
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
