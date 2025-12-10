// SPDX-License-Identifier: Apache-2.0

//! Benchamrks parsing text- and JSON-encoded expressions.
//!

mod common;

use common::{JSON_SAMPLES, TEXT_SAMPLES};
use criterion::{Criterion, criterion_group, criterion_main};
use ogc_cql2::Expression;
use std::error::Error;

fn do_text() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for src in TEXT_SAMPLES {
        let _ = Expression::try_from_text(&src)?;
        count += 1;
    }
    assert_eq!(count, TEXT_SAMPLES.len());
    Ok(())
}

fn do_json() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for src in JSON_SAMPLES {
        let expr = Expression::try_from_json(&src)?;
        let text = expr.to_string();
        let _ = Expression::try_from_text(&text)?;
        count += 1;
    }
    assert_eq!(count, 109);
    Ok(())
}

fn parse_text(c: &mut Criterion) {
    c.bench_function("Parse Text", |b| b.iter(|| do_text()));
}

fn parse_json(c: &mut Criterion) {
    c.bench_function("Parse JSON", |b| b.iter(|| do_json()));
}

criterion_group!(benchmarks, parse_text, parse_json);
criterion_main!(benchmarks);
