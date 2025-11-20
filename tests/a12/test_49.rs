// SPDX-License-Identifier: Apache-2.0

//! Test predicates against the test dataset
//!
//! Given:
//!     * The implementation under test uses the test dataset.
//! When:
//!     Evaluate each predicate in Predicates and expected results, if the
//!     conditional dependency is met.
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the expected result is returned;
//! * store the valid predicates for each data source.
//!

use crate::utils::{
    CountryCSV, CountryGPkg, PlaceCSV, PlaceGPkg, RiverCSV, RiverGPkg, harness, harness_gpkg,
    harness_sql,
};
use std::error::Error;

#[rustfmt::skip]
const PLACES_PREDICATES: [(&str, u32); 76] = [
    ("'København'=name",     1),
    ("'København'<=name",  137),
    ("'København'<name",   136),
    ("'København'>=name",  107),
    ("'København'>name",   106),
    ("'København'<>name",  242),
    ("name=nameascii",     230),
    ("name>=nameascii",    243),
    ("name>nameascii",      13),
    ("name<=nameascii",    230),
    ("name<nameascii",       0),
    ("name<>nameascii",     13),
    ("1038288=pop_other",    1),
    ("1038288<=pop_other", 123),
    ("1038288<pop_other",  122),
    ("1038288>=pop_other", 121),
    ("1038288>pop_other",  120),
    ("1038288<>pop_other", 242),
    ("pop_min=pop_max",     27),
    ("pop_min<=pop_max",   243),
    ("pop_min<pop_max",    216),
    ("pop_min>=pop_max",    27),
    ("pop_min>pop_max",      0),
    ("pop_min<>pop_max",   216),
    ("start=end",            0),
    ("start<=end",           3),
    ("start<end",            3),
    ("start>=end",           0),
    ("start>end",            0),
    ("start<>end",           3),
    ("'København' LIKE 'K_benhavn'",              243),
    ("'København' NOT LIKE 'K_benhavn'",            0),
    ("pop_other between pop_min and pop_max",      94),
    ("pop_other not between pop_min and pop_max", 149),

    ("S_INTERSECTS(BBOX(0,40,10,50),geom)",                         7),
    ("S_INTERSECTS(POLYGON((0 40,10 40,10 50,0 50,0 40)),geom)",    7),
    ("S_DISJOINT(BBOX(0,40,10,50),geom)",                         236),
    ("S_DISJOINT(POLYGON((0 40,10 40,10 50,0 50,0 40)),geom)",    236),
    ("S_EQUALS(POINT(6.1300028 49.6116604),geom)",                  1),
    ("S_CONTAINS(BBOX(-180,-90,0,90),geom)",                       74),

    (r#"t_after(date('2022-04-16'),"date")"#,                       1),
    (r#"t_before(date('2022-04-16'),"date")"#,                      1),
    (r#"t_disjoint(date('2022-04-16'),"date")"#,                    2),
    (r#"t_equals(date('2022-04-16'),"date")"#,                      1),
    (r#"t_intersects(date('2022-04-16'),"date")"#,                  1),
    (r#"t_after(interval('2022-01-01','2022-12-31'),"date")"#,      1),
    (r#"t_before(interval('2022-01-01','2022-12-31'),"date")"#,     1),
    (r#"t_disjoint(interval('2022-01-01','2022-12-31'),"date")"#,   2),
    (r#"t_equals(interval('2022-01-01','2022-12-31'),"date")"#,     0),
    (r#"t_equals(interval('2022-04-16','2022-04-16'),"date")"#,     1),
    (r#"t_intersects(interval('2022-01-01','2022-12-31'),"date")"#, 1),
    ("t_after(timestamp('2022-04-16T10:13:19Z'),start)",            1),
    ("t_before(timestamp('2022-04-16T10:13:19Z'),start)",           1),
    ("t_disjoint(timestamp('2022-04-16T10:13:19Z'),start)",         2),
    ("t_equals(timestamp('2022-04-16T10:13:19Z'),start)",           1),
    ("t_intersects(timestamp('2022-04-16T10:13:19Z'),start)",       1),
    ("t_after(interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'),start)",      1),
    ("t_before(interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'),start)",     0),
    ("t_disjoint(interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'),start)",   1),
    ("t_equals(interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'),start)",     0),
    ("t_intersects(interval('2022-01-01T00:00:00Z','2022-12-31T23:59:59Z'),start)", 2),
    ("t_after(interval('2023-01-01T00:00:00Z','..'),interval(start,end))",          2),
    ("t_before(interval('..','2022-04-16T10:13:19Z'),interval(start,end))",         1),
    ("t_disjoint(interval('2022-04-16T10:13:19Z','2022-04-16T10:15:09Z'),interval(start,end))",     1),
    ("t_equals(interval('2021-04-16T10:15:59Z','2022-04-16T10:16:06Z'),interval(start,end))",       1),
    ("t_intersects(interval('2022-04-16T10:13:19Z','2022-04-16T10:15:09Z'),interval(start,end))",   2),
    ("T_CONTAINS(interval('2021-04-16T10:13:19Z','2023-04-16T10:15:10Z'),interval(start,end))",     2),
    ("T_DURING(interval('2022-07-01T00:00:00Z','2022-12-31T23:59:59Z'),interval(start,end))",       1),
    ("T_FINISHES(interval('2022-04-16T10:13:19Z','2022-04-16T10:16:06Z'),interval(start,end))",     1),
    ("T_FINISHEDBY(interval('2022-04-16T10:13:19Z','2022-04-16T10:16:06Z'),interval(start,end))",   0),
    ("T_MEETS(interval('2022-04-16T10:13:19Z','2022-04-16T10:15:10Z'),interval(start,end))",        1),
    ("T_METBY(interval('2022-04-16T10:13:19Z','2022-04-16T10:15:10Z'),interval(start,end))",        0),
    ("T_OVERLAPPEDBY(interval('2020-04-16T10:13:19Z','2022-04-16T10:15:10Z'),interval(start,end))", 0),
    ("T_OVERLAPS(interval('2022-04-16T10:13:19Z','2023-04-16T10:15:10Z'),interval(start,end))",     0),
    ("T_STARTEDBY(interval('2022-04-16T10:13:19Z','2022-04-16T10:15:10Z'),interval(start,end))",    0),
    ("T_STARTS(interval('2022-04-16T10:13:19Z','2022-04-16T10:15:10Z'),interval(start,end))",       1)
];

#[rustfmt::skip]
const COUNTRIES_PREDICATES: [(&str, u32); 18] = [
    ("S_INTERSECTS(BBOX(0,40,10,50),geom)",                      8),
    ("S_INTERSECTS(BBOX(150,-90,-150,90),geom)",                10),
    ("S_INTERSECTS(POINT(7.02 49.92),geom)",                     1),
    ("S_INTERSECTS(POLYGON((0 40,10 40,10 50,0 50,0 40)),geom)", 8),
    ("S_INTERSECTS(LINESTRING(0 40,10 50),geom)",                4),
    ("S_DISJOINT(BBOX(0,40,10,50),geom)",                      169),
    ("S_DISJOINT(POLYGON((0 40,10 40,10 50,0 50,0 40)),geom)", 169),
    ("S_DISJOINT(LINESTRING(0 40,10 50),geom)",                173),
    ("S_DISJOINT(POINT(7.02 49.92),geom)",                     176),
    (r#"S_TOUCHES(POLYGON((
        6.043073357781111 50.128051662794235,
        6.242751092156993 49.90222565367873,
        6.186320428094177 49.463802802114515,
        5.897759230176348 49.44266714130711,
        5.674051954784829 49.529483547557504,
        5.782417433300907 50.09032786722122,
        6.043073357781111 50.128051662794235)),geom)"#,             3),
    ("S_TOUCHES(POINT(6.043073357781111 50.128051662794235),geom)", 3),
    ("S_TOUCHES(POINT(6.242751092156993 49.90222565367873),geom)",  2),
    (r#"S_TOUCHES(LINESTRING(
        6.043073357781111 50.128051662794235,
        6.242751092156993 49.90222565367873),geom)"#, 3),
    ("S_CONTAINS(BBOX(-180,-90,0,90),geom)",         44),
    ("S_WITHIN(BBOX(7,50,8,51),geom)",                1),
    ("S_WITHIN(LINESTRING(7 50,8 51),geom)",          1),
    ("S_WITHIN(POINT(7.02 49.92),geom)",              1),
    ("S_OVERLAPS(BBOX(-180,-90,0,90),geom)",         11),
];

#[rustfmt::skip]
const RIVERS_PREDICATES: [(&str, u32); 7] = [
    ("S_INTERSECTS(BBOX(-180,-90,0,90),geom)",        4),
    ("S_INTERSECTS(LINESTRING(-60 -90,-60 90),geom)", 2),
    ("S_DISJOINT(BBOX(-180,-90,0,90),geom)",          9),
    ("S_DISJOINT(LINESTRING(-60 -90,-60 90),geom)",  11),
    ("S_CROSSES(BBOX(0,40,10,50),geom)",              1),
    ("S_CROSSES(LINESTRING(-60 -90,-60 90),geom)",    2),
    ("S_CONTAINS(BBOX(-180,-90,0,90),geom)",          4),
];

#[test]
fn test_places() -> Result<(), Box<dyn Error>> {
    let ds = PlaceCSV::new();
    harness(ds, &PLACES_PREDICATES)
}

#[tokio::test]
async fn test_places_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = PlaceGPkg::new().await?;
    harness_gpkg(ds, &PLACES_PREDICATES).await
}

#[tokio::test]
async fn test_places_sql() -> Result<(), Box<dyn Error>> {
    let gpkg = PlaceGPkg::new().await?;
    harness_sql(gpkg, &PLACES_PREDICATES).await
}

#[test]
#[tracing_test::traced_test]
fn test_countries() -> Result<(), Box<dyn Error>> {
    let ds = CountryCSV::new();
    harness(ds, &COUNTRIES_PREDICATES)
}

#[tokio::test]
async fn test_countries_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = CountryGPkg::new().await?;
    harness_gpkg(ds, &COUNTRIES_PREDICATES).await
}

#[tokio::test]
async fn test_countries_sql() -> Result<(), Box<dyn Error>> {
    #[rustfmt::skip]
    const COUNTRIES_PREDICATES: [(&str, u32); 14] = [
        ("S_INTERSECTS(BBOX(0,40,10,50),geom)",                      8),
        ("S_INTERSECTS(BBOX(150,-90,-150,90),geom)",                10),
        ("S_INTERSECTS(POINT(7.02 49.92),geom)",                     1),
        ("S_INTERSECTS(POLYGON((0 40,10 40,10 50,0 50,0 40)),geom)", 8),
        ("S_INTERSECTS(LINESTRING(0 40,10 50),geom)",                4),
        ("S_DISJOINT(BBOX(0,40,10,50),geom)",                      169),
        ("S_DISJOINT(POLYGON((0 40,10 40,10 50,0 50,0 40)),geom)", 169),
        ("S_DISJOINT(LINESTRING(0 40,10 50),geom)",                173),
        ("S_DISJOINT(POINT(7.02 49.92),geom)",                     176),
        // (r#"S_TOUCHES(POLYGON((
        //     6.043073357781111 50.128051662794235,
        //     6.242751092156993 49.90222565367873,
        //     6.186320428094177 49.463802802114515,
        //     5.897759230176348 49.44266714130711,
        //     5.674051954784829 49.529483547557504,
        //     5.782417433300907 50.09032786722122,
        //     6.043073357781111 50.128051662794235)),geom)"#,             3),
        // ("S_TOUCHES(POINT(6.043073357781111 50.128051662794235),geom)", 3),
        // ("S_TOUCHES(POINT(6.242751092156993 49.90222565367873),geom)",  2),
        // (r#"S_TOUCHES(LINESTRING(
        //     6.043073357781111 50.128051662794235,
        //     6.242751092156993 49.90222565367873),geom)"#, 3),
        ("S_CONTAINS(BBOX(-180,-90,0,90),geom)",         44),
        ("S_WITHIN(BBOX(7,50,8,51),geom)",                1),
        ("S_WITHIN(LINESTRING(7 50,8 51),geom)",          1),
        ("S_WITHIN(POINT(7.02 49.92),geom)",              1),
        ("S_OVERLAPS(BBOX(-180,-90,0,90),geom)",         11),
    ];

    let gpkg = CountryGPkg::new().await?;
    harness_sql(gpkg, &COUNTRIES_PREDICATES).await
}

#[test]
fn test_rivers() -> Result<(), Box<dyn Error>> {
    let ds = RiverCSV::new();
    harness(ds, &RIVERS_PREDICATES)
}

#[tokio::test]
async fn test_rivers_gpkg() -> Result<(), Box<dyn Error>> {
    let ds = RiverGPkg::new().await?;
    harness_gpkg(ds, &RIVERS_PREDICATES).await
}

#[tokio::test]
async fn test_rivers_sql() -> Result<(), Box<dyn Error>> {
    let gpkg = RiverGPkg::new().await?;
    harness_sql(gpkg, &RIVERS_PREDICATES).await
}
