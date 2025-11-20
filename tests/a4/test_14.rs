// SPDX-License-Identifier: Apache-2.0

//! Test filter expressions with AND, OR and NOT including sub-expressions
//!
//! Given:
//!     * The stored predicates for each data source, including from the
//!       dependencies.
//! When:
//!     For each data source, select at least 10 random combinations of four
//!     predicates ({p1} to {p4}) from the stored predicates and evaluate the
//!     filter expression ((NOT {p1} AND {p2}) OR ({p3} and NOT {p4}) or not
//!     ({p1} AND {p4})).
//! Then:
//! * assert successful execution of the evaluation.
//!

use crate::utils::{PlaceCSV, PlaceGPkg, harness, harness_gpkg, harness_sql};
use std::error::Error;
use tracing_test::traced_test;

#[rustfmt::skip]
const PREDICATES: [(&str, &str, &str, &str, u32); 10] = [
    (r#"pop_other<>1038288"#,                       r#"name<>'København'"#,
     r#"pop_other IS NULL"#,                        r#"name<'København'"#,    137),
    (r#"start IS NULL"#,                            r#"pop_other>1038288"#,
     r#"start IS NOT NULL"#,                        r#"name>'København'"#,    107),
    (r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"pop_other=1038288"#,                        r#"start IS NULL"#,         3),
    (r#"name<>'København'"#,                        r#"boolean=true"#,
     r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"start IS NULL"#,         3),
    (r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"name<='København'"#,                        r#"name<>'København'"#,     1),
    (r#"pop_other<=1038288"#,                       r#"name<'København'"#,
     r#"pop_other<1038288"#,                        r#"pop_other<1038288"#,   123),
    (r#"pop_other<=1038288"#,                       r#"pop_other<1038288"#,
     r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"pop_other>1038288"#,   243),
    (r#"name<='København'"#,                        r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"name<'København'"#,                         r#"boolean IS NULL"#,     139),
    (r#"name>='København'"#,                        r#"pop_other<>1038288"#,
     r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"name<>'København'"#,   107),
    (r#"pop_other>=1038288"#,                       r#"pop_other>1038288"#,
     r#"boolean IS NULL"#,                          r#"pop_other=1038288"#,   242),
];

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
    // compose the predicates into a single Expression...
    let mut expressions = vec![];
    for (p1, p2, p3, p4, count) in PREDICATES {
        let ex = format!("((NOT {p1} AND {p2}) OR ({p3} and NOT {p4}) or not ({p1} AND {p4}))");
        expressions.push((ex, count));
    }
    let predicates: Vec<(&str, u32)> = expressions.iter().map(|(s, c)| (s.as_str(), *c)).collect();

    let ds = PlaceCSV::new();
    harness(ds, &predicates)
}

#[tokio::test]
async fn test_gpkg() -> Result<(), Box<dyn Error>> {
    let mut expressions = vec![];
    for (p1, p2, p3, p4, count) in PREDICATES {
        let ex = format!("((NOT {p1} AND {p2}) OR ({p3} and NOT {p4}) or not ({p1} AND {p4}))");
        expressions.push((ex, count));
    }
    let predicates: Vec<(&str, u32)> = expressions.iter().map(|(s, c)| (s.as_str(), *c)).collect();

    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &predicates).await
}

#[tokio::test]
async fn test_sql() -> Result<(), Box<dyn Error>> {
    let mut expressions = vec![];
    for (p1, p2, p3, p4, count) in PREDICATES {
        let ex = format!("((NOT {p1} AND {p2}) OR ({p3} and NOT {p4}) or not ({p1} AND {p4}))");
        expressions.push((ex, count));
    }
    let predicates: Vec<(&str, u32)> = expressions.iter().map(|(s, c)| (s.as_str(), *c)).collect();

    let ds = PlaceGPkg::new().await?;
    harness_sql(ds, &predicates).await
}
