// SPDX-License-Identifier: Apache-2.0

//! Test IN predicate
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//! When:
//!     * for each queryable {queryable} of type Number or Integer, evaluate
//!       the following filter expression {queryable} IN (1, 2, 3);
//!     * for each queryable {queryable} of type String, evaluate the following
//!       filter expression {queryable} in ('foo', 'bar');
//!     * for each queryable {queryable} of type Boolean, evaluate the following
//!       filter expression {queryable} in (true);
//!     * for each queryable {queryable} of type Timestamp, evaluate the
//!       following filter expression {queryable} in ('2022-04-14T14:52:56Z',
//!       '2022-04-14T15:52:56Z');
//!     * for each queryable {queryable} of type Date, evaluate the following
//!       filter expression {queryable} in ('2022-04-14', '2022-04-15');
//! Then:
//! * assert successful execution of the evaluation;
//! * store the valid predicates for each data source.
//!

use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome, Q, Resource};
use jiff::{Timestamp, ToSpan, civil::Date};
use rand::{Rng, distr::Uniform};
use std::error::Error;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test_numbers() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"foo IN (1, 2, 3)"#;
    const LIST: [u32; 3] = [1, 2, 3];

    let mut rng = rand::rng();
    let shared_ctx = Context::new_shared();

    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let exp = Expression::try_from_text(&format!("{E}"))?;

    evaluator.setup(exp)?;

    let (mut expected, mut actual) = (0, 0);
    // uniform distribution of random unsigned ints w/in 1..=200
    let dist = Uniform::new(0, 10)?;
    for ndx in 1..=1000 {
        let n = rng.sample(dist);
        if LIST.contains(&n) {
            expected += 1;
        }
        let feat = Resource::from([
            ("fid".into(), Q::try_from(ndx)?),
            ("foo".into(), Q::try_from(n)?),
        ]);

        let res = evaluator.evaluate(&feat)?;
        if matches!(res, Outcome::T) {
            actual += 1
        }
    }

    evaluator.teardown()?;

    tracing::debug!("expected/actual = {expected}, {actual}");
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
#[traced_test]
fn test_strings() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"foo in ('foo', 'bar')"#;
    const LIST: [&str; 2] = ["foo", "bar"];
    #[rustfmt::skip]
    const DICT: [&str; 18] = [
        "bar",  "bazoo", "razoo", "foo",  "bozo", "zarf",
        "orzo", "barf",  "forb",  "afro", "boar", "roof",
        "boor", "bora",  "broo",  "bro",  "faro", "fora"
    ];

    let mut rng = rand::rng();
    let shared_ctx = Context::new_shared();

    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let exp = Expression::try_from_text(&format!("{E}"))?;

    evaluator.setup(exp)?;

    let (mut expected, mut actual) = (0, 0);
    // uniform distribution of random DICT indices...
    let dist = Uniform::new(0, DICT.len())?;
    for ndx in 1..=1000 {
        let w = DICT[rng.sample(dist)];
        if LIST.contains(&w) {
            expected += 1;
        }
        let feat = Resource::from([
            ("fid".into(), Q::try_from(ndx)?),
            ("foo".into(), Q::new_plain_str(w)),
        ]);

        let res = evaluator.evaluate(&feat)?;
        if matches!(res, Outcome::T) {
            actual += 1
        }
    }

    evaluator.teardown()?;

    tracing::debug!("expected/actual = {expected}, {actual}");
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
#[traced_test]
fn test_booleans() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"foo in (true)"#;
    const LIST: [bool; 1] = [true];

    let mut rng = rand::rng();
    let shared_ctx = Context::new_shared();

    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let exp = Expression::try_from_text(&format!("{E}"))?;

    evaluator.setup(exp)?;

    let (mut expected, mut actual) = (0, 0);
    for ndx in 1..=1000 {
        let b = rng.random_bool(0.5);
        if LIST.contains(&b) {
            expected += 1;
        }
        let feat = Resource::from([
            ("fid".into(), Q::try_from(ndx)?),
            ("foo".into(), Q::from(b)),
        ]);

        let res = evaluator.evaluate(&feat)?;
        if matches!(res, Outcome::T) {
            actual += 1
        }
    }

    evaluator.teardown()?;

    tracing::debug!("expected/actual = {expected}, {actual}");
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
#[traced_test]
fn test_timestamps() -> Result<(), Box<dyn Error>> {
    // const E: &str = r#"foo in ('2022-04-14T14:52:56Z', '2022-04-14T15:52:56Z')"#;
    const E: &str = r#"foo in (timestamp('2022-04-14T14:52:56Z'), TIMEStamp('2022-04-14T15:52:56Z'))"#;

    let t1: Timestamp = "2022-04-14T14:52:56Z".parse()?;
    let t2: Timestamp = "2022-04-14T15:52:56Z".parse()?;
    #[allow(non_snake_case)]
    let LIST: [Timestamp; 2] = [t1, t2];

    let mean = t1.checked_add(30.minutes())?; // interval is 1 hour
    let mut rng = rand::rng();
    let shared_ctx = Context::new_shared();

    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let exp = Expression::try_from_text(&format!("{E}"))?;
    // tracing::debug!("exp = {exp:?}");

    evaluator.setup(exp)?;

    let (mut expected, mut actual) = (0, 0);
    let dist = Uniform::new(-60, 61)?; // in minutes; 2-hour span
    for ndx in 1..=1000 {
        let delta = rng.sample(dist);
        let ts = mean + delta.minutes();
        if LIST.contains(&ts) {
            expected += 1;
        }
        let feat = Resource::from([
            ("fid".into(), Q::try_from(ndx)?),
            ("foo".into(), Q::from(ts)),
        ]);
        // tracing::debug!("feat = {feat:?}");

        let res = evaluator.evaluate(&feat)?;
        if matches!(res, Outcome::T) {
            actual += 1
        }
    }

    evaluator.teardown()?;

    tracing::debug!("expected/actual = {expected}, {actual}");
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
#[traced_test]
fn test_dates() -> Result<(), Box<dyn Error>> {
    const E: &str = r#"foo in (Date('2022-04-14'), date('2022-04-15'))"#;

    let d1: Date = "2022-04-14".parse()?;
    let d2: Date = "2022-04-15".parse()?;
    #[allow(non_snake_case)]
    let LIST: [Date; 2] = [d1, d2]; // interval is 1 day

    let mean = d1.checked_add(12.hours())?;

    let mut rng = rand::rng();
    let shared_ctx = Context::new_shared();

    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let exp = Expression::try_from_text(&format!("{E}"))?;

    evaluator.setup(exp)?;

    let (mut expected, mut actual) = (0, 0);
    let dist = Uniform::new(-2, 2)?; // in hours; 4 hours span
    for ndx in 1..=1000 {
        let delta = rng.sample(dist);
        let d = mean + delta.days();
        if LIST.contains(&d) {
            expected += 1;
        }
        let feat = Resource::from([
            ("fid".into(), Q::try_from(ndx)?),
            ("foo".into(), Q::try_from(d)?),
        ]);
        // tracing::debug!("feat = {feat:?}");

        let res = evaluator.evaluate(&feat)?;
        if matches!(res, Outcome::T) {
            actual += 1
        }
    }

    evaluator.teardown()?;

    tracing::debug!("expected/actual = {expected}, {actual}");
    assert_eq!(actual, expected);

    Ok(())
}
