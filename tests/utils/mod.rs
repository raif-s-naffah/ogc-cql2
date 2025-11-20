// SPDX-License-Identifier: Apache-2.0

//! Test harnesses and artifacts used in conformance tests w/ different
//! Expressions, Data Sources, and expected correct response(s).
//!

mod country;
mod place;
mod river;

pub(crate) use country::{CountryCSV, CountryGPkg, countries, countries_gpkg};
pub(crate) use place::{PlaceCSV, PlaceGPkg, places};
pub(crate) use river::{RiverCSV, RiverGPkg};

use futures::TryStreamExt;
use ogc_cql2::{
    Context, Evaluator, ExEvaluator, Expression, IterableDS, MyError, Outcome, Resource,
    StreamableDS,
};
use rand::{
    Rng,
    distr::{
        Alphanumeric,
        uniform::{UniformChar, UniformSampler},
    },
};
use std::error::Error;

const GPKG_URL: &str = "sqlite:tests/samples/data/ne110m4cql2.gpkg";

// Process the records of a named CSV data source, evaluating for each a
// collection of predicates and collecting for each one of those the tally of
// correct results.
//
// The test passes if the actual count of correct responses matches a given
// expected value and fails otherwise.
pub(crate) fn harness<T: IterableDS<Err = MyError> + std::fmt::Display>(
    ds: T,
    predicates: &[(&str, u32)],
) -> Result<(), Box<dyn Error>>
where
    Resource: TryFrom<<T as IterableDS>::Item>,
{
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
        let mut evaluator = ExEvaluator::new(shared_ctx.clone());
        let expr = Expression::try_from_text(input)?;
        evaluator.setup(expr)?;
        evaluators.push(evaluator);
        expected.push(success_count);
        actual.push(0);
    }

    for x in ds.iter()? {
        let feature = x?;
        let resource = Resource::try_from(feature)
            .map_err(|_| MyError::Runtime("Failed converting iterable item to Resource".into()))?;
        for (p_ndx, evaluator) in evaluators.iter().enumerate() {
            let res = evaluator
                .evaluate(&resource)
                .expect(&format!("Failed evaluating resource @{ds}"));
            if matches!(res, Outcome::T) {
                // tracing::debug!("-- match: {resource:?}");
                actual[p_ndx] += 1;
            }
        }
    }

    let mut failures = 0;
    for (ndx, count) in actual.iter().enumerate() {
        let n = expected[ndx];
        // tracing::debug!("Predicate #{ndx} - actual/expected: {count} / {n}");
        if count != n {
            tracing::error!("Failed predicate #{ndx} - actual/expected: {count} / {n}");
            failures += 1;
        }
    }

    assert_eq!(failures, 0);
    Ok(())
}

// similar to the iterable version but uses async streaming...
// remember though that this is painfully slow due to the conversions :(
pub(crate) async fn harness_gpkg<T: StreamableDS<Err = MyError> + std::fmt::Display>(
    ds: T,
    predicates: &[(&str, u32)],
) -> Result<(), Box<dyn Error>> {
    let mut evaluators = vec![];
    let mut expected = vec![];
    let mut actual = vec![];

    let shared_ctx = Context::try_with_crs("epsg:4326")?.freeze();
    for (input, success_count) in predicates {
        let mut evaluator = ExEvaluator::new(shared_ctx.clone());
        let expr = Expression::try_from_text(input)?;
        evaluator.setup(expr)?;
        evaluators.push(evaluator);
        expected.push(success_count);
        actual.push(0);
    }

    let mut stream = ds.stream().await?;
    while let Some(resource) = stream.try_next().await? {
        for (p_ndx, evaluator) in evaluators.iter().enumerate() {
            let res = evaluator
                .evaluate(&resource)
                .expect(&format!("Failed evaluating resource @{ds}"));
            if matches!(res, Outcome::T) {
                actual[p_ndx] += 1;
            }
        }
    }

    let mut failures = 0;
    for (ndx, count) in actual.iter().enumerate() {
        let n = expected[ndx];
        if count != n {
            tracing::error!("Failed predicate #{ndx} - actual/expected: {count} / {n}");
            failures += 1;
        }
    }

    assert_eq!(failures, 0);
    Ok(())
}

// the real McKoy! for GeoPackage (and future PostGIS) data sources.  delegates
// the filtering to the DB engine through a SELECT w/ a WHERE clause built from
// the CQL2 expression...
pub(crate) async fn harness_sql<T: StreamableDS<Err = MyError> + std::fmt::Display>(
    gpkg: T,
    predicates: &[(&str, u32)],
) -> Result<(), Box<dyn Error>> {
    // use the 'stream_where()' entry point -> TXxx...
    for (ndx, (filter, expected)) in predicates.iter().enumerate() {
        let exp = Expression::try_from_text(&filter)?;
        let mut actual = 0;
        let mut stream = gpkg.fetch_where(&exp).await?;
        while let Some(_) = stream.try_next().await? {
            actual += 1;
        }
        assert_eq!(actual, *expected, "Failed predicate #{ndx}");
    }
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

// Generate a random word between 5 and 15 unicode characters long.
pub(crate) fn random_unicode_word() -> String {
    let mut rng = rand::rng();
    let dist = UniformChar::new_inclusive('\u{0041}', '\u{10FFFF}')
        .expect("Failed setting up uniform distribution");
    let size: usize = rng.random_range(5..15);
    (0..size).map(|_| dist.sample(&mut rng)).collect()
}
