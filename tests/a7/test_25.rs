// SPDX-License-Identifier: Apache-2.0

//! Test the S_INTERSECTS spatial comparison function with points and bounding
//! boxes.
//!
//! Given:
//!     * One or more data sources, each with a list of queryables.
//!     * At least one queryable has a geometry data type.
//! When:
//!     For each queryable {queryable} with a geometry data type, evaluate the
//!     following filter expressions
//!     * S_INTERSECTS({queryable},BBOX(-180,-90,180,90))
//!     * S_INTERSECTS({queryable},POINT(7.02 49.92))
//!     * S_INTERSECTS({queryable},POINT(90 180))
//!     * S_INTERSECTS({queryable},BBOX(-180,-90,-90,90)) 
//!       AND S_INTERSECTS({queryable},BBOX(90,-90,180,90))
//! Then:
//! * assert successful execution of the evaluation for the first two filter
//!   expressions;
//! * assert unsuccessful execution of the evaluation for the third filter
//!   expressions (invalid coordinate);
//! * store the valid predicates for each data source.
//!

use crate::utils::{harness, COUNTRIES, PLACES};
use std::error::Error;
use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Resource, Q};
use tracing_test::traced_test;

#[rustfmt::skip]
const COUNTRIES_PREDICATES: [(&str, u32); 3] = [
    // should match all records in the CSV test data set...
    (r#"S_INTERSECTS(geom,BBOX(-180,-90,180,90))"#, 177),
    // coordinates of a point in Germany...
    (r#"S_INTERSECTS(geom,POINT(7.02 49.92))"#,       1),
    (r#"S_INTERSECTS(geom,BBOX(-180,-90,-90,90)) 
      AND S_INTERSECTS(geom,BBOX(90,-90,180,90))"#,   3),
];

#[rustfmt::skip]
const PLACES_PREDICATES: [(&str, u32); 3] = [
    (r#"S_INTERSECTS(geom,BBOX(-180,-90,180,90))"#, 243),
    (r#"S_INTERSECTS(geom,POINT(7.02 49.92))"#,       0),
    (r#"S_INTERSECTS(geom,BBOX(-180,-90,-90,90)) 
      AND S_INTERSECTS(geom,BBOX(90,-90,180,90))"#,   0),
];

// OGC CQL2 references another standard (API) that expects a default CRS of
// EPSG:4326 (WGS'84). W/in that system latitudes must fall w/in -90.0 to +90.0
// only. Consequently the 3rd filter expression `S_INTERSECTS(geom,POINT(90 180))`
// should fail/panic...
#[test]
fn test_invalid_coordinate() -> Result<(), Box<dyn Error>> {
    const E: &str = "S_INTERSECTS(geom,POINT(90 180))";

    let shared_ctx = Context::new_shared();
        let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
        let expr = Expression::try_from_text(E)?;
        evaluator.setup(expr)?;

        let feat = Resource::from([
            ("fid".into(), Q::try_from(1)?),
            ("point".into(), Q::try_from_wkt("POINT (12.4533865 41.9032822)")?),
        ]);

        let res = evaluator.evaluate(&feat);
        assert!(res.is_err());

        Ok(())
}

#[test]
#[traced_test]
fn test_countries() -> Result<(), Box<dyn Error>> {
    harness(COUNTRIES, COUNTRIES_PREDICATES.to_vec())
}

#[test]
#[traced_test]
fn test_places() -> Result<(), Box<dyn Error>> {
    harness(PLACES, PLACES_PREDICATES.to_vec())
}
