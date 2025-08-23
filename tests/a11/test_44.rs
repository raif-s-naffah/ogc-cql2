// SPDX-License-Identifier: Apache-2.0

//! Test the array comparison functions
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * At least one queryable has an array data type.
//! When:
//!     For each queryable {queryable} with an array data type, evaluate the
//!     following filter expressions
//!     * A_CONTAINS({queryable},("foo","bar"))
//!     * A_CONTAINEDBY({queryable},("foo","bar"))
//!     * A_EQUALS({queryable},("foo","bar"))
//!     * A_OVERLAPS({queryable},("foo","bar"))
//! Then:
//! * assert successful execution of the evaluation;
//! * store the valid predicates for each data source.
//!

use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome, Q, Resource};
use std::error::Error;

#[test]
fn test_contains() -> Result<(), Box<dyn Error>> {
    // IMPORTANT: "foo" and "bar" here are property names; not raw strings...
    const E: &str = r#"A_CONTAINS(list,("foo","bar"))"#;

    let shared_ctx = Context::new_shared();
    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let exp = Expression::try_from_text(E)?;
    evaluator.setup(exp)?;

    let z_foo = &Q::new_plain_str("foo-value");
    let z_bar = &Q::new_plain_str("bar-value");
    let z_coffee = &Q::new_plain_str("coffee-value");
    let z_tea = &Q::new_plain_str("tea-value");

    // the good...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("foo".into(), z_foo.clone()),
        ("bar".into(), z_bar.clone()),
        (
            "list".into(),
            Q::List(vec![z_foo.clone(), z_coffee.clone(), z_bar.clone()]),
        ),
    ]);
    let res = evaluator.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // the bad...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("foo".into(), z_foo.clone()),
        ("bar".into(), z_bar.clone()),
        (
            "list".into(),
            Q::List(vec![z_foo.clone(), z_coffee.clone(), z_tea.clone()]),
        ),
    ]);

    let res = evaluator.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator.teardown()?;
    Ok(())
}

#[test]
fn test_contained_by() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"A_CONTAINEDBY(list,("foo","bar"))"#;

    let shared_ctx = Context::new_shared();
    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let exp = Expression::try_from_text(E)?;
    evaluator.setup(exp)?;

    let z_foo = &Q::new_plain_str("foo-value");
    let z_bar = &Q::new_plain_str("bar-value");
    let z_coffee = &Q::new_plain_str("coffee-value");

    // the good...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("foo".into(), z_foo.clone()),
        ("bar".into(), z_bar.clone()),
        ("list".into(), Q::List(vec![z_bar.clone()])),
    ]);
    let res = evaluator.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // the bad...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("foo".into(), z_foo.clone()),
        ("bar".into(), z_bar.clone()),
        (
            "list".into(),
            Q::List(vec![z_foo.clone(), z_coffee.clone()]),
        ),
    ]);

    let res = evaluator.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator.teardown()?;
    Ok(())
}

#[test]
fn test_equals() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"A_EQUALS(list,("foo","bar"))"#;

    let shared_ctx = Context::new_shared();
    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let exp = Expression::try_from_text(E)?;
    evaluator.setup(exp)?;

    let z_foo = &Q::new_plain_str("foo-value");
    let z_bar = &Q::new_plain_str("bar-value");
    let z_coffee = &Q::new_plain_str("coffee-value");
    let z_tea = &Q::new_plain_str("tea-value");

    // the good...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("foo".into(), z_foo.clone()),
        ("bar".into(), z_bar.clone()),
        ("list".into(), Q::List(vec![z_foo.clone(), z_bar.clone()])),
    ]);
    let res = evaluator.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // the bad...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("foo".into(), z_foo.clone()),
        ("bar".into(), z_bar.clone()),
        (
            "list".into(),
            Q::List(vec![z_coffee.clone(), z_tea.clone()]),
        ),
    ]);

    let res = evaluator.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator.teardown()?;
    Ok(())
}

#[test]
fn test_overlaps() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"A_OVERLAPS(list,("foo","bar"))"#;

    let shared_ctx = Context::new_shared();
    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let exp = Expression::try_from_text(E)?;
    evaluator.setup(exp)?;

    let z_foo = &Q::new_plain_str("foo-value");
    let z_bar = &Q::new_plain_str("bar-value");
    let z_coffee = &Q::new_plain_str("coffee-value");
    let z_tea = &Q::new_plain_str("tea-value");

    // the good...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(1)?),
        ("foo".into(), z_foo.clone()),
        ("bar".into(), z_bar.clone()),
        ("coffee".into(), z_coffee.clone()),
        ("tea".into(), z_tea.clone()),
        (
            "list".into(),
            Q::List(vec![z_foo.clone(), z_coffee.clone(), z_tea.clone()]),
        ),
    ]);
    let res = evaluator.evaluate(&feat)?;
    assert!(matches!(res, Outcome::T));

    // the bad...
    let feat = Resource::from([
        ("fid".into(), Q::try_from(2)?),
        ("foo".into(), z_foo.clone()),
        ("bar".into(), z_bar.clone()),
        ("coffee".into(), z_coffee.clone()),
        ("tea".into(), z_tea.clone()),
        (
            "list".into(),
            Q::List(vec![z_coffee.clone(), z_tea.clone()]),
        ),
    ]);

    let res = evaluator.evaluate(&feat)?;
    assert!(matches!(res, Outcome::F));

    evaluator.teardown()?;
    Ok(())
}
