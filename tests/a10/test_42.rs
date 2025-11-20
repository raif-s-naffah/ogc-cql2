// SPDX-License-Identifier: Apache-2.0

//! Test predicates against the test dataset
//!
//! Given:
//!     * The implementation under test uses the test dataset.
//! When:
//!     Evaluate each predicate in Predicates and expected results.
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the expected result is returned;
//! * store the valid predicates for each data source.
//!

use crate::utils::{PlaceCSV, PlaceGPkg, harness, harness_gpkg, harness_sql};
use std::error::Error;

#[rustfmt::skip]
const PREDICATES: [(&str, u32); 36] = [
    (r#"t_after("date",date('2022-04-16'))"#,                            1),
    (r#"t_before("date",date('2022-04-16'))"#,                           1),
    (r#"t_disjoint("date",date('2022-04-16'))"#,                         2),
    (r#"t_equals("date",date('2022-04-16'))"#,                           1),
    (r#"t_intersects("date",date('2022-04-16'))"#,                       1),
    (r#"t_after("date",interval('2022-01-01','2022-12-31'))"#,           1),
    (r#"t_before("date",interval('2022-01-01','2022-12-31'))"#,          1),
    (r#"t_disjoint("date",interval('2022-01-01','2022-12-31'))"#,        2),
    (r#"t_equals("date",interval('2022-01-01','2022-12-31'))"#,          0),
    (r#"t_equals("date",interval('2022-04-16','2022-04-16'))"#,          1),
    (r#"t_intersects("date",interval('2022-01-01','2022-12-31'))"#,      1),
    (r#"t_after(start,timestamp('2022-04-16T10:13:19Z'))"#,              1),
    (r#"t_before(start,timestamp('2022-04-16T10:13:19Z'))"#,             1),
    (r#"t_disjoint(start,timestamp('2022-04-16T10:13:19Z'))"#,           2),
    (r#"t_equals(start,timestamp('2022-04-16T10:13:19Z'))"#,             1),
    (r#"t_intersects(start,timestamp('2022-04-16T10:13:19Z'))"#,         1),
    (r#"t_after(start,interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'))"#,      0),
    (r#"t_before(start,interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'))"#,     1),
    (r#"t_disjoint(start,interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'))"#,   1),
    (r#"t_equals(start,interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'))"#,     0),
    (r#"t_intersects(start,interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'))"#, 2),
    (r#"t_after(interval(start,end),interval('..','2022-04-16T10:13:19Z'))"#,          1),
    (r#"t_before(interval(start,end),interval('2023-01-01T00:00:00Z','..'))"#,         2),
    (r#"t_disjoint(interval(start,end),interval('2022-04-16T10:13:19Z','2022-04-16T10:15:09Z'))"#,     1),
    (r#"t_equals(interval(start,end),interval('2021-04-16T10:15:59Z','2022-04-16T10:16:06Z'))"#,       1),
    (r#"t_intersects(interval(start,end),interval('2022-04-16T10:13:19Z','2022-04-16T10:15:09Z'))"#,   2),
    (r#"T_CONTAINS(interval(start,end),interval('2022-04-16T10:13:19Z','2022-04-16T10:15:10Z'))"#,     1),
    (r#"T_DURING(interval(start,end),interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'))"#,       1),
    (r#"T_FINISHES(interval(start,end),interval('2020-04-16T10:13:19Z','2022-04-16T10:16:06Z'))"#,     1),
    (r#"T_FINISHEDBY(interval(start,end),interval('2022-04-16T10:13:19Z','2022-04-16T10:16:06Z'))"#,   1),
    (r#"T_MEETS(interval(start,end),interval('2022-04-16T10:13:19Z','2022-04-16T10:15:10Z'))"#,        0),
    (r#"T_METBY(interval(start,end),interval('2022-04-16T10:13:19Z','2022-04-16T10:15:10Z'))"#,        1),
    (r#"T_OVERLAPPEDBY(interval(start,end),interval('2020-04-16T10:13:19Z','2022-04-16T10:15:10Z'))"#, 2),
    (r#"T_OVERLAPS(interval(start,end),interval('2022-04-16T10:13:19Z','2023-04-16T10:15:10Z'))"#,     1),
    (r#"T_STARTEDBY(interval(start,end),interval('2022-04-16T10:13:19Z','2022-04-16T10:15:10Z'))"#,    1),
    (r#"T_STARTS(interval(start,end),interval('2022-04-16T10:13:19Z','2022-04-16T10:15:10Z'))"#,       0),
];

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    let ds = PlaceCSV::new();
    harness(ds, &PREDICATES)
}

#[tokio::test]
async fn test_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &PREDICATES).await
}

#[tokio::test]
async fn test_sql() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_sql(ds, &PREDICATES).await
}
