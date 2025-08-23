// SPDX-License-Identifier: Apache-2.0

//! Test LIKE predicate
//!
//! Given:
//!     *One or more data sources, each with a list of queryables.
//! When:
//! For each queryable {queryable} of type String, evaluate the following
//! filter expressions
//!     * {queryable} LIKE '%'
//!     * {queryable} like '_%'
//!     * {queryable} like ''
//!     * {queryable} like '%%'
//!     * {queryable} like '\\%\\_'
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the two result sets for each queryable for the pattern
//!   expression '_%' and '' have no item in common;
//! * assert that the two result sets for each queryable for the pattern
//!   expression '%' and '%%' are identical;
//! * store the valid predicates for each data source.
//!

use crate::utils::{ZCountry, countries, countries_reader};
use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome, Q, Resource};
use csv::Reader;
use std::{error::Error, fs::File};
use tracing::error;
use tracing_test::traced_test;

const LP1: &str = "NAME LIKE '%'";
const LP2: &str = "NAME like '_%'";
const LP3: &str = "NAME like ''";
const LP4: &str = "NAME like '%%'";
const LP5: &str = "NAME like '\\%\\_'";

// w/ LP1, any non-null input will match...
#[test]
#[traced_test]
fn test_outcome_1() -> Result<(), Box<dyn Error>> {
    let shared_ctx = Context::new_shared();
    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let input = format!("{LP1}");
    let exp = Expression::try_from_text(&input)?;
    // tracing::debug!("exp = {exp:?}");

    evaluator.setup(exp)?;

    for feat in countries()? {
        let res = evaluator.evaluate(&feat)?;

        assert!(matches!(res, Outcome::T));
    }
    evaluator.teardown()?;
    Ok(())
}

// w/ LP2 empty strings will NOT match, while w/ LP3, only they will...
#[test]
#[traced_test]
fn test_outcome_2() -> Result<(), Box<dyn Error>> {
    let shared_ctx = Context::new_shared();
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());

    let input1 = format!("{LP2}");
    let exp1 = Expression::try_from_text(&input1)?;
    // tracing::debug!("exp1 = {exp1:?}");
    let input2 = format!("{LP3}");
    let exp2 = Expression::try_from_text(&input2)?;
    // tracing::debug!("exp2 = {exp2:?}");

    evaluator1.setup(exp1)?;
    evaluator2.setup(exp2)?;

    // let file = File::open("./tests/samples/csv/ne_110m_admin_0_countries.csv")?;
    // let mut rdr = Reader::from_reader(file);
    let mut rdr = countries_reader::<Reader<File>>()?;
    for result in rdr.deserialize() {
        let row: ZCountry = result?;
        let is_empty = row.name().is_empty();
        let feat = Resource::try_from(row)?;

        let res1 = evaluator1.evaluate(&feat)?;
        let res2 = evaluator2.evaluate(&feat)?;

        if is_empty {
            assert!(matches!(res1, Outcome::F));
            assert!(matches!(res2, Outcome::T));
        } else {
            assert!(matches!(res1, Outcome::T));
            assert!(matches!(res2, Outcome::F));
        }
    }

    evaluator1.teardown()?;
    evaluator2.teardown()?;
    Ok(())
}

// LP1 and LP4 yield the same results...
#[test]
#[traced_test]
fn test_outcome_3() -> Result<(), Box<dyn Error>> {
    let shared_ctx = Context::new_shared();
    let mut evaluator1 = EvaluatorImpl::new(shared_ctx.clone());
    let mut evaluator2 = EvaluatorImpl::new(shared_ctx.clone());

    let input1 = format!("{LP1}");
    let exp1 = Expression::try_from_text(&input1)?;
    // tracing::debug!("exp1 = {exp1:?}");
    let input2 = format!("{LP4}");
    let exp2 = Expression::try_from_text(&input2)?;
    // tracing::debug!("exp2 = {exp2:?}");

    evaluator1.setup(exp1)?;
    evaluator2.setup(exp2)?;

    for feat in countries()? {
        let res1 = evaluator1.evaluate(&feat)?;
        let res2 = evaluator2.evaluate(&feat)?;

        assert_eq!(res1, res2);
    }

    evaluator1.teardown()?;
    evaluator2.teardown()?;
    Ok(())
}

#[test]
#[traced_test]
fn test_outcome_4() -> Result<(), Box<dyn Error>> {
    const TV: [(&str, bool); 5] = [
        (r#"abc\%def"#, false),
        (r#"abc%def"#, false),
        (r#"%d"#, true),
        (r#"_d"#, true),
        (r#"_"#, false),
    ];

    let shared_ctx = Context::new_shared();
    let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
    let input = format!("{LP5}");
    let exp = Expression::try_from_text(&input)?;
    // tracing::debug!("exp = {exp:?}");

    evaluator.setup(exp)?;

    let mut failures = 0;
    for (ndx, (name, flag)) in TV.iter().enumerate() {
        let expected = Outcome::new(Some(flag));
        let feat = Resource::from([
            ("fid".into(), Q::try_from(ndx)?),
            ("NAME".into(), Q::new_plain_str(&name)),
        ]);

        let actual = evaluator.evaluate(&feat)?;
        // tracing::debug!("actual = {actual:?}");
        if actual != expected {
            error!("Failed #{ndx}, name = \"{name}\"");
            failures += 1
        }
    }

    evaluator.teardown()?;

    assert_eq!(failures, 0);
    Ok(())
}
