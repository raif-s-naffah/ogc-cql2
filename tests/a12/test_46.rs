// SPDX-License-Identifier: Apache-2.0

//! Test comparison predicates with properties on the right-hand side and
//! values on the left-hand side
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     For each queryable {queryable} of one of the data types String, Boolean,
//!     Number, Integer, Timestamp or Date, evaluate the following filter
//!     expressions
//!     * {value} = {queryable}
//!     * {value} <> {queryable}
//!     * {value} > {queryable}
//!     * {value} < {queryable}
//!     * {value} >= {queryable}
//!     * {value} <= {queryable}
//!     where {value} depends on the data type:
//!     * String: 'foo'
//!     * Boolean: true
//!     * Number: 3.14
//!     * Integer: 1
//!     * Timestamp: TIMESTAMP('2022-04-14T14:48:46Z')
//!     * Date: DATE('2022-04-14')
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets for each queryable for the operators =
//!   and <> have no item in common;
//! * assert that the two result sets for each queryable for the operators >
//!   and <= have no item in common;
//! * assert that the two result sets for each queryable for the operators <
//!   and >= have no item in common;
//! * store the valid predicates for each data source.
//!

use ogc_cql2::{Context, Evaluator, ExEvaluator, Expression, Outcome, Q, Resource};
use std::error::Error;

#[test]
fn test_string() -> Result<(), Box<dyn Error>> {
    const E1: &str = "'foo' = var";
    const E2: &str = "'foo' <> var";
    const E3: &str = "'foo' > var";
    const E4: &str = "'foo' <= var";
    const E5: &str = "'foo' < var";
    const E6: &str = "'foo' >= var";

    let shared_ctx = Context::new().freeze();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = ExEvaluator::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = ExEvaluator::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = ExEvaluator::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = ExEvaluator::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("var".into(), Q::new_plain_str("foo")),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("var".into(), Q::new_plain_str("bar")),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
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

    Ok(())
}

#[test]
fn test_number() -> Result<(), Box<dyn Error>> {
    const E1: &str = "3.14 = var";
    const E2: &str = "3.14 <> var";
    const E3: &str = "3.14 > var";
    const E4: &str = "3.14 <= var";
    const E5: &str = "3.14 < var";
    const E6: &str = "3.14 >= var";

    let shared_ctx = Context::new().freeze();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = ExEvaluator::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = ExEvaluator::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = ExEvaluator::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = ExEvaluator::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("var".into(), Q::try_from(3.14)?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("var".into(), Q::try_from(42)?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
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

    Ok(())
}

#[test]
fn test_integer() -> Result<(), Box<dyn Error>> {
    const E1: &str = "1 = var";
    const E2: &str = "1 <> var";
    const E3: &str = "1 > var";
    const E4: &str = "1 <= var";
    const E5: &str = "1 < var";
    const E6: &str = "1 >= var";

    let shared_ctx = Context::new().freeze();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = ExEvaluator::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = ExEvaluator::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = ExEvaluator::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = ExEvaluator::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("var".into(), Q::try_from(1)?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("var".into(), Q::try_from(0)?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
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

    Ok(())
}

// NOTE (rsn) 20250817 - it's not clear from the specs what to expect as
// the "right" answer when evaluating, for example, TRUE <= FALSE.
// my understanding is that this is programming language related and there
// is no known "standard" that one can follow.  in Rust, these relations
// hold as of release 1.89.0...
//
// [true | false] < false? false
// true > false? true
// true <= false? false
// [true | false] >= false? true
//
// true < true? false
// [true | false] > true? false
// [true | false] <= true? true
// true >= true? true
//
// false > false? false
// false <= false? true
//
// false < true? true
// false >= true? false
#[test]
fn test_boolean() -> Result<(), Box<dyn Error>> {
    const E1: &str = "true = var";
    const E2: &str = "true <> var";
    const E3: &str = "true > var";
    const E4: &str = "true <= var";
    const E5: &str = "true < var";
    const E6: &str = "true >= var";

    let shared_ctx = Context::new().freeze();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = ExEvaluator::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = ExEvaluator::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = ExEvaluator::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = ExEvaluator::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("var".into(), Q::Bool(true)),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("var".into(), Q::Bool(false)),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
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

    Ok(())
}

#[test]
fn test_timestamp() -> Result<(), Box<dyn Error>> {
    const E1: &str = "TIMESTAMP('2022-04-14T14:48:46Z') = var";
    const E2: &str = "TIMESTAMP('2022-04-14T14:48:46Z') <> var";
    const E3: &str = "TIMESTAMP('2022-04-14T14:48:46Z') > var";
    const E4: &str = "TIMESTAMP('2022-04-14T14:48:46Z') <= var";
    const E5: &str = "TIMESTAMP('2022-04-14T14:48:46Z') < var";
    const E6: &str = "TIMESTAMP('2022-04-14T14:48:46Z') >= var";

    let shared_ctx = Context::new().freeze();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = ExEvaluator::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = ExEvaluator::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = ExEvaluator::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = ExEvaluator::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
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

    Ok(())
}

#[test]
fn test_date() -> Result<(), Box<dyn Error>> {
    const E1: &str = "DATE('2022-04-14') = var";
    const E2: &str = "DATE('2022-04-14') <> var";
    const E3: &str = "DATE('2022-04-14') > var";
    const E4: &str = "DATE('2022-04-14') <= var";
    const E5: &str = "DATE('2022-04-14') < var";
    const E6: &str = "DATE('2022-04-14') >= var";

    let shared_ctx = Context::new().freeze();
    let exp1 = Expression::try_from_text(E1)?;
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(exp1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    let exp2 = Expression::try_from_text(E2)?;
    evaluator2.setup(exp2)?;
    let exp3 = Expression::try_from_text(E3)?;
    let mut evaluator3 = ExEvaluator::new(shared_ctx.clone());
    evaluator3.setup(exp3)?;
    let mut evaluator4 = ExEvaluator::new(shared_ctx.clone());
    let exp4 = Expression::try_from_text(E4)?;
    evaluator4.setup(exp4)?;
    let exp5 = Expression::try_from_text(E5)?;
    let mut evaluator5 = ExEvaluator::new(shared_ctx.clone());
    evaluator5.setup(exp5)?;
    let mut evaluator6 = ExEvaluator::new(shared_ctx.clone());
    let exp6 = Expression::try_from_text(E6)?;
    evaluator6.setup(exp6)?;

    // test equal and not equal...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("var".into(), Q::try_from_date_str("2022-04-14")?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("var".into(), Q::try_from_date_str("2025-08-18")?),
    ]);
    let res = evaluator1.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));
    let res = evaluator2.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // test the rest...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(3)?),
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

    Ok(())
}
