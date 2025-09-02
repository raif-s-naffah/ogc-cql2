// SPDX-License-Identifier: Apache-2.0

//! Test harnesses re-used in conformance tests w/ different Expressions and
//! expected correct response.
//!

mod country;
mod place;
mod river;

pub(crate) use country::ZCountry;
use csv::{Reader, StringRecord};
use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, MyError, Outcome, Resource};
use place::ZPlace;
use rand::{
    Rng,
    distr::{Alphanumeric, Uniform},
};
use river::ZRiver;
use std::{error::Error, fs::File};

const DATASETS: [&str; 3] = [
    "./tests/samples/csv/ne_110m_admin_0_countries.csv",
    "./tests/samples/csv/ne_110m_populated_places_simple.csv",
    "./tests/samples/csv/ne_110m_rivers_lake_centerlines.csv",
];
pub(crate) const COUNTRIES: usize = 0;
pub(crate) const PLACES: usize = 1;
pub(crate) const RIVERS: usize = 2;

pub(crate) fn countries_reader<R>() -> Result<Reader<File>, MyError> {
    let file = File::open(DATASETS[COUNTRIES])?;
    Ok(Reader::from_reader(file))
}

/// Try converting one CSV record into a [Resource].
fn to_feature(data_set: usize, record: StringRecord) -> Result<Resource, Box<dyn Error>> {
    match data_set {
        COUNTRIES => ZCountry::new_from_record(record),
        PLACES => ZPlace::new_from_record(record),
        _ => ZRiver::new_from_record(record),
    }
}

/// Try reading all records of a CSV test data set, converting them into a
/// [Resource] collection.
fn features(data_set: usize) -> Result<Vec<Resource>, Box<dyn Error>> {
    let file = File::open(DATASETS[data_set])?;
    let mut rdr = Reader::from_reader(file);
    let mut result = vec![];
    for record in rdr.records() {
        let feat = to_feature(data_set, record?)?;
        result.push(feat);
    }

    Ok(result)
}

/// Read all _Countries_ CSV test data-set rows, convert each to a [Resource]
/// and return the lot.
pub(crate) fn countries() -> Result<Vec<Resource>, Box<dyn Error>> {
    features(COUNTRIES)
}

/// Read all _Simple Places_ CSV test data-set rows, convert each to a
/// [Resource] and return the lot.
pub(crate) fn places() -> Result<Vec<Resource>, Box<dyn Error>> {
    features(PLACES)
}

// Process the records of a named CSV test data set, evaluating for each a
// collection of predicates and collecting for each one of those the tally of
// correct results.
//
// The test passes if the actual count of correct responses matches a given
// expected value and fails otherwise.
pub(crate) fn harness(data_set: usize, predicates: Vec<(&str, u32)>) -> Result<(), Box<dyn Error>> {
    let mut evaluators = vec![];
    let mut expected = vec![];
    let mut actual = vec![];

    // IMPORTANT (rsn) 20250901 - as mentioned often in the comments + docs,
    // some conformance tests that expect a failure due to "invalid coordinate"
    // assume that the _implicit_ CRS against which coordinates are checked is
    // in fact EPSG:4326.  Our library allows configuring any valid CRS as the
    // implicit one to use when evaluating Expressions. W/o properly setting
    // a `Context` that takes this into account, the test(s) may fail.
    let shared_ctx = Context::try_with_crs("epsg:4326")?.freeze();

    for (input, success_count) in predicates {
        let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
        let expr = Expression::try_from_text(input)?;
        // tracing::debug!("expr = {expr:?}");
        evaluator.setup(expr)?;
        evaluators.push(evaluator);
        expected.push(success_count);
        actual.push(0);
    }

    let file = File::open(DATASETS[data_set])?;
    let mut rdr = Reader::from_reader(file);
    for (ndx, record) in rdr.records().enumerate() {
        let feat = to_feature(data_set, record?)?;

        for (p_ndx, evaluator) in evaluators.iter().enumerate() {
            let res = evaluator
                .evaluate(&feat)
                .expect(&format!("Failed evaluating resource @{ndx}"));
            if matches!(res, Outcome::T) {
                // tracing::debug!("match: {feat:?}");
                actual[p_ndx] += 1;
            }
        }
    }

    let mut failures = 0;
    for (ndx, count) in actual.iter().enumerate() {
        let n = expected[ndx];
        // tracing::debug!("Predicate #{ndx} - actual/expected: {count} / {n}");
        if *count != n {
            tracing::error!("Failed predicate #{ndx} - actual/expected: {count} / {n}");
            failures += 1;
        }
    }

    for evaluator in &mut evaluators {
        evaluator.teardown()?;
    }

    assert_eq!(failures, 0);
    Ok(())
}

// Generate a random alphanumeric string between 5 and 15 ASCII characters long.
pub(crate) fn random_ascii_word() -> String {
    let mut rng = rand::rng();
    let size: usize = rng.random_range(5..15);
    (0..size)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub(crate) fn random_unicode_word() -> String {
    let mut rng = rand::rng();
    let dist =
        Uniform::new_inclusive(0x0080, 0x10FFFF).expect("Failed setting up uniform distribution");
    let size: usize = rng.random_range(5..15);
    (0..size)
        .map(|_| {
            loop {
                if let Some(x) = char::from_u32(rng.sample(dist)) {
                    if x.is_alphanumeric() {
                        return x;
                    }
                }
            }
        })
        .collect()
}
