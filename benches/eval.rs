// SPDX-License-Identifier: Apache-2.0

//! Benchamrks evaluating CQL2 Expressions w/ CSV data sources. Implicitly
//! exercise the WKT and EWKB parsing machinery used when converting
//! GeoPackage (WKT) and PostGIS (EWKB) columns to Rust types.
//!

mod common;

use crate::common::{CountryCSV, CountryGPkg, CountryPG, async_runtime};
use criterion::{Criterion, criterion_group, criterion_main};
use futures::TryStreamExt;
use ogc_cql2::prelude::*;
use std::error::Error;

#[rustfmt::skip]
const COUNTRIES_PREDICATES: [(&str, u32); 12] = [
    (r#"NAME='Luxembourg'"#,    1),
    (r#"NAME>='Luxembourg'"#,  84),
    (r#"NAME>'Luxembourg'"#,   83),
    (r#"NAME<='Luxembourg'"#,  94),
    (r#"NAME<'Luxembourg'"#,   93),
    (r#"NAME<>'Luxembourg'"#, 176),
    // -----
    (r#"POP_EST=37589262"#,    1),
    (r#"POP_EST>=37589262"#,  39),
    (r#"POP_EST>37589262"#,   38),
    (r#"POP_EST<=37589262"#, 139),
    (r#"POP_EST<37589262"#,  138),
    (r#"POP_EST<>37589262"#, 176),
];

fn do_iterable<T: IterableDS<Err = MyError>>(
    ds: T,
    predicates: &[(&str, u32)],
) -> Result<(), Box<dyn Error>>
where
    Resource: TryFrom<<T as IterableDS>::Item>,
{
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
    for x in ds.iter()? {
        let feature = x?;
        let resource = Resource::try_from(feature)
            .map_err(|_| MyError::Runtime("Failed into Resource".into()))?;
        for (p_ndx, evaluator) in evaluators.iter().enumerate() {
            let res = evaluator.evaluate(&resource)?;
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

async fn do_streamable<T: StreamableDS<Err = MyError>>(
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
            let res = evaluator.evaluate(&resource)?;
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

async fn do_streamable_sql<T: StreamableDS<Err = MyError> + std::fmt::Display>(
    gpkg: T,
    predicates: &[(&str, u32)],
) -> Result<(), Box<dyn Error>> {
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

// ----------------------------------------------------------------------------

// use local evaluators w/ iterable CSV data
fn csv_exe() -> Result<(), Box<dyn Error>> {
    let ds = CountryCSV::new();
    do_iterable(ds, &COUNTRIES_PREDICATES)
}

// use local evaluators w/ streamable GeoPackage data
async fn gpkg_exe() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    do_streamable(ds, &COUNTRIES_PREDICATES).await
}

// use native SQL evaluators w/ streamable GeoPackage data
async fn gpkg_sql() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    do_streamable_sql(ds, &COUNTRIES_PREDICATES).await
}

// use local evaluators w/ streamable PostGIS data
async fn pg_exe() -> Result<(), Box<dyn Error>> {
    let ds = CountryPG::new().await?;
    do_streamable(ds, &COUNTRIES_PREDICATES).await
}

// use native SQL evaluators w/ streamable PostGIS data
async fn pg_sql() -> Result<(), Box<dyn Error>> {
    let ds = CountryPG::new().await?;
    do_streamable_sql(ds, &COUNTRIES_PREDICATES).await
}

// ----------------------------------------------------------------------------

fn eval_csv(c: &mut Criterion) {
    c.bench_function("CSV + ExEvaluator", |b| b.iter(|| csv_exe()));
}

fn eval_gpkg(c: &mut Criterion) {
    c.bench_function("GeoPackage + ExEvaluator", |b| {
        let rt = async_runtime();
        b.to_async(rt).iter(|| gpkg_exe())
    });
}

fn eval_gpkg_sql(c: &mut Criterion) {
    c.bench_function("GeoPackage + SQL", |b| {
        let rt = async_runtime();
        b.to_async(rt).iter(|| gpkg_sql())
    });
}

fn eval_pg(c: &mut Criterion) {
    c.bench_function("PostGIS + ExEvaluator", |b| {
        let rt = async_runtime();
        b.to_async(rt).iter(|| pg_exe())
    });
}

fn eval_pg_sql(c: &mut Criterion) {
    c.bench_function("PostGIS + SQL", |b| {
        let rt = async_runtime();
        b.to_async(rt).iter(|| pg_sql())
    });
}

criterion_group! {
    name = benchmarks;
    config = Criterion::default().sample_size(10);
    targets = eval_csv, eval_gpkg, eval_gpkg_sql, eval_pg, eval_pg_sql
}
criterion_main!(benchmarks);
