// SPDX-License-Identifier: Apache-2.0

//! Benchamrks transforming CQL2 Expression intermediate form to SQL dialects
//! used for constructing SELECT... WHERE... used to filter data source input.
//!

mod common;

use crate::common::{CountryGPkg, CountryPG, TEXT_SAMPLES, async_runtime};
use criterion::{Criterion, criterion_group, criterion_main};
use ogc_cql2::Expression;
use std::error::Error;

async fn do_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    for (ndx, p) in TEXT_SAMPLES.iter().enumerate() {
        let exp = Expression::try_from_text(p)?;
        let _ = ds
            .inner()
            .to_sql(&exp)
            .expect(format!("Failed transforming sample #{ndx}").as_str());
    }
    Ok(())
}

async fn do_pg() -> Result<(), Box<dyn Error>> {
    let ds = CountryPG::new().await?;
    for (ndx, p) in TEXT_SAMPLES.iter().enumerate() {
        let exp = Expression::try_from_text(p)?;
        let _ = ds
            .inner()
            .to_sql(&exp)
            .expect(format!("Failed transforming sample #{ndx}").as_str());
    }
    Ok(())
}

fn tx_gpkg(c: &mut Criterion) {
    c.bench_function("SQL (GeoPackage)", |b| {
        let rt = async_runtime();
        b.to_async(rt).iter(|| do_gpkg())
    });
}

fn tx_pg(c: &mut Criterion) {
    c.bench_function("SQL (PostGIS)", |b| {
        let rt = async_runtime();
        b.to_async(rt).iter(|| do_pg())
    });
}

criterion_group! {
    name = benchmarks;
    config = Criterion::default().sample_size(60);
    targets = tx_gpkg, tx_pg
}
criterion_main!(benchmarks);
