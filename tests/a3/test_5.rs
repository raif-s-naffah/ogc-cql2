// SPDX-License-Identifier: Apache-2.0

//! Test comparison predicates
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * Test '/conf/basic-cql2/basic-test' passes.
//! When:
//!     For each queryable {queryable} of one of the data types String,
//!     Boolean, Number, Integer, Timestamp or Date, evaluate the following
//!     filter expressions:
//!         * {queryable} = {value}
//!         * {queryable} <> {value}
//!         * {queryable} > {value}
//!         * {queryable} < {value}
//!         * {queryable} >= {value}
//!         * {queryable} <= {value}
//!     where {value} depends on the data type:
//!         * String: 'foo'
//!         * Boolean: true
//!         * Number: 3.14
//!         * Integer: 1
//!         * Timestamp: TIMESTAMP('2022-04-14T14:48:46Z')
//!     * Date: DATE('2022-04-14')
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets for each queryable for the operators `=`
//!   and `<>` have no item in common;
//! * assert that the two result sets for each queryable for the operators `>`
//!   and `<=` have no item in common;
//! * assert that the two result sets for each queryable for the operators `<`
//!   and `>=` have no item in common;
//! * store the valid predicates for each data source.
//!

use crate::utils::random_ascii_word;
use jiff::{Timestamp, ToSpan, civil::Date};
use ogc_cql2::{Context, Evaluator, ExEvaluator, Expression, Outcome, Q, Resource};
use rand::{
    Rng,
    distr::{Distribution, Uniform},
};
use std::error::Error;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test_equal_bool() -> Result<(), Box<dyn Error>> {
    let mut rng = rand::rng();
    let mut random_bool = || rng.random_bool(0.5);
    let shared_ctx = Context::new().freeze();

    const E1: &str = r#"z_bool = TRUE"#;
    let expr1 = Expression::try_from_text(E1)?;

    const E2: &str = r#"z_bool <> TRUE"#;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let b = random_bool();
        if b {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_bool".into(), Q::from(b)),
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

#[test]
#[traced_test]
fn test_equal_int() -> Result<(), Box<dyn Error>> {
    const MEDIAN: i32 = 1;
    const E1: &str = r#"z_int = 1"#;
    const E2: &str = r#"z_int <> 1"#;

    let mut rng = rand::rng();
    let dist = Uniform::new_inclusive(MEDIAN - 2, MEDIAN + 2).expect("Failed uniform distribution");
    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let i = dist.sample(&mut rng);
        if i == MEDIAN {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_int".into(), Q::try_from(i)?),
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

#[test]
#[traced_test]
fn test_less_int() -> Result<(), Box<dyn Error>> {
    const MEDIAN: i32 = 1;
    const E1: &str = r#"z_int < 1"#;
    const E2: &str = r#"z_int >= 1"#;

    let mut rng = rand::rng();
    let dist =
        Uniform::new_inclusive(MEDIAN - 10, MEDIAN + 10).expect("Failed uniform distribution");
    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let i = dist.sample(&mut rng);
        if i < MEDIAN {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_int".into(), Q::try_from(i)?),
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

#[test]
#[traced_test]
fn test_greater_int() -> Result<(), Box<dyn Error>> {
    const MEDIAN: i32 = 1;
    const E1: &str = r#"z_int > 1"#;
    const E2: &str = r#"z_int <= 1"#;

    let mut rng = rand::rng();
    let dist =
        Uniform::new_inclusive(MEDIAN - 10, MEDIAN + 10).expect("Failed uniform distribution");
    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let i = dist.sample(&mut rng);
        if i > MEDIAN {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_int".into(), Q::try_from(i)?),
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

#[test]
#[traced_test]
fn test_equal_float() -> Result<(), Box<dyn Error>> {
    const MEDIAN: f64 = 3.14;
    const E1: &str = r#"z_float = 3.14"#;
    const E2: &str = r#"z_float <> 3.14"#;

    let mut rng = rand::rng();
    let dist =
        Uniform::new_inclusive(MEDIAN - 2.0, MEDIAN + 2.0).expect("Failed uniform distribution");
    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let f = dist.sample(&mut rng);
        if f == MEDIAN {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_float".into(), Q::from(f)),
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

#[test]
#[traced_test]
fn test_less_float() -> Result<(), Box<dyn Error>> {
    const MEDIAN: f64 = 3.14;
    const E1: &str = r#"z_float < 3.14"#;
    const E2: &str = r#"z_float >= 3.14"#;

    let mut rng = rand::rng();
    let dist =
        Uniform::new_inclusive(MEDIAN - 10.0, MEDIAN + 10.0).expect("Failed uniform distribution");
    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let f = dist.sample(&mut rng);
        if f < MEDIAN {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_float".into(), Q::from(f)),
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

#[test]
#[traced_test]
fn test_greater_float() -> Result<(), Box<dyn Error>> {
    const MEDIAN: f64 = 3.14;
    const E1: &str = r#"z_float > 3.14"#;
    const E2: &str = r#"z_float <= 3.14"#;

    let mut rng = rand::rng();
    let dist =
        Uniform::new_inclusive(MEDIAN - 10.0, MEDIAN + 10.0).expect("Failed uniform distribution");
    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let f = dist.sample(&mut rng);
        if f > MEDIAN {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_float".into(), Q::from(f)),
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

#[test]
#[traced_test]
fn test_equal_timestamp() -> Result<(), Box<dyn Error>> {
    let time: Timestamp = "2022-04-14T14:48:46Z".parse()?;
    let median: i128 = time.as_nanosecond();
    const E1: &str = r#"z_timestamp = timestamp('2022-04-14T14:48:46Z')"#;
    const E2: &str = r#"z_timestamp <> TimeStamp('2022-04-14T14:48:46Z')"#;

    let mut rng = rand::rng();
    let dist = Uniform::new_inclusive(median - 2, median + 2).expect("Failed uniform distribution");
    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let t = dist.sample(&mut rng);
        if t == median {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_timestamp".into(), Q::try_from_timestamp_ns(t)?),
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

#[test]
#[traced_test]
fn test_less_timestamp() -> Result<(), Box<dyn Error>> {
    let time: Timestamp = "2022-04-14T14:48:46Z".parse()?;
    let median: i128 = time.as_nanosecond();
    const E1: &str = r#"z_timestamp < timestamp('2022-04-14T14:48:46Z')"#;
    const E2: &str = r#"z_timestamp >= TimeStamp('2022-04-14T14:48:46Z')"#;

    let mut rng = rand::rng();
    let dist =
        Uniform::new_inclusive(median - 2000, median + 2000).expect("Failed uniform distribution");
    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let t = dist.sample(&mut rng);
        if t < median {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_timestamp".into(), Q::try_from_timestamp_ns(t)?),
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

#[test]
#[traced_test]
fn test_greater_timestamp() -> Result<(), Box<dyn Error>> {
    let time: Timestamp = "2022-04-14T14:48:46Z".parse()?;
    let median: i128 = time.as_nanosecond();
    const E1: &str = r#"z_timestamp > timestamp('2022-04-14T14:48:46Z')"#;
    const E2: &str = r#"z_timestamp <= TimeStamp('2022-04-14T14:48:46Z')"#;

    let mut rng = rand::rng();
    let dist =
        Uniform::new_inclusive(median - 2000, median + 2000).expect("Failed uniform distribution");
    let shared_ctx = Context::new().freeze();
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);

    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;

    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    for n in 0..1000 {
        let t = dist.sample(&mut rng);
        if t > median {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_timestamp".into(), Q::try_from_timestamp_ns(t)?),
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

#[test]
#[traced_test]
fn test_equal_date() -> Result<(), Box<dyn Error>> {
    let median: Date = "2022-04-14".parse()?;
    const E1: &str = r#"z_date = date('2022-04-14')"#;
    const E2: &str = r#"z_date <> DATE('2022-04-14')"#;
    let dist = Uniform::new_inclusive(-31, 31).expect("Failed uniform distribution");
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);
    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    let mut rng = rand::rng();
    for n in 0..1000 {
        let delta = dist.sample(&mut rng);
        let d = median.checked_add(delta.days())?;
        if d == median {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_date".into(), Q::try_from(d)?),
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

#[test]
#[traced_test]
fn test_less_date() -> Result<(), Box<dyn Error>> {
    let median: Date = "2022-04-14".parse()?;
    const E1: &str = r#"z_date < Date('2022-04-14')"#;
    const E2: &str = r#"z_date >= date('2022-04-14')"#;
    let dist = Uniform::new_inclusive(-31, 31).expect("Failed uniform distribution");
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);
    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    let mut rng = rand::rng();
    for n in 0..1000 {
        let delta = dist.sample(&mut rng);
        let d = median.checked_add(delta.days())?;
        if d < median {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_date".into(), Q::try_from(d)?),
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

#[test]
#[traced_test]
fn test_greater_date() -> Result<(), Box<dyn Error>> {
    let median: Date = "2022-04-14".parse()?;
    const E1: &str = r#"z_date > Date('2022-04-14')"#;
    const E2: &str = r#"z_date <= date('2022-04-14')"#;
    let dist = Uniform::new_inclusive(-31, 31).expect("Failed uniform distribution");
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);
    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    let mut rng = rand::rng();
    for n in 0..1000 {
        let delta = dist.sample(&mut rng);
        let d = median.checked_add(delta.days())?;
        if d > median {
            expect_true1 += 1;
            expect_false2 += 1;
        } else {
            expect_false1 += 1;
            expect_true2 += 1;
        }

        let feat = Resource::from([
            ("fid".into(), Q::try_from(n + 1)?),
            ("z_date".into(), Q::try_from(d)?),
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

#[test]
#[traced_test]
fn test_equal_str() -> Result<(), Box<dyn Error>> {
    let median: &str = "foo";
    const E1: &str = r#"z_string = 'foo'"#;
    const E2: &str = r#"z_string <> 'foo'"#;
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);
    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    let mut rng = rand::rng();
    for n in 0..1000 {
        // match 'foo' 25% of the time...
        let is_median = rng.random_bool(0.25);
        let s: String = if is_median {
            median.to_owned()
        } else {
            random_ascii_word()
        };
        if s == median {
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

#[test]
#[traced_test]
fn test_less_str() -> Result<(), Box<dyn Error>> {
    let median: &str = "foo";
    const E1: &str = r#"z_string < 'foo'"#;
    const E2: &str = r#"z_string >= 'foo'"#;
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);
    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    let mut rng = rand::rng();
    for n in 0..1000 {
        let is_median = rng.random_bool(0.25);
        let s: String = if is_median {
            median.to_owned()
        } else {
            random_ascii_word()
        };
        if s.as_str() < median {
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

#[test]
#[traced_test]
fn test_greater_str() -> Result<(), Box<dyn Error>> {
    let median: &str = "foo";
    const E1: &str = r#"z_string > 'foo'"#;
    const E2: &str = r#"z_string <= 'foo'"#;
    let expr1 = Expression::try_from_text(E1)?;
    let expr2 = Expression::try_from_text(E2)?;

    let (mut expect_true1, mut expect_false1) = (0, 0);
    let (mut actual_true1, mut actual_false1, mut actual_null1) = (0, 0, 0);
    let (mut expect_true2, mut expect_false2) = (0, 0);
    let (mut actual_true2, mut actual_false2, mut actual_null2) = (0, 0, 0);

    let shared_ctx = Context::new().freeze();
    let mut evaluator1 = ExEvaluator::new(shared_ctx.clone());
    evaluator1.setup(expr1)?;
    let mut evaluator2 = ExEvaluator::new(shared_ctx.clone());
    evaluator2.setup(expr2)?;

    let mut rng = rand::rng();
    for n in 0..1000 {
        let is_median = rng.random_bool(0.25);
        let s: String = if is_median {
            median.to_owned()
        } else {
            random_ascii_word()
        };
        if s.as_str() > median {
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
