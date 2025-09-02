// SPDX-License-Identifier: Apache-2.0

//! Test filter expressions with AND, OR and NOT including sub-expressions
//!
//! Given:
//! * One or more data sources.
//! * The stored predicates for each data source.
//!
//! When:
//! Evaluate each predicate in [Combinations of predicates and expected results][1].
//! For the data source 'ne_110m_populated_places_simple', evaluate the filter
//! expression (NOT ({p2}) AND {p1}) OR ({p3} and {p4}) or not ({p1} OR {p4})
//! for each combination of predicates {p1} to {p4} in Combinations of predicates
//! and expected results.
//!
//! Then:
//! * assert successful execution of the evaluation;
//! * assert that the expected result is returned.
//!
//! [1]: <https://docs.ogc.org/is/21-065r2/21-065r2.html#test-data-predicates-basic-cql2-combinations>
//!

use crate::utils::places;
use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome};
use std::error::Error;

#[rustfmt::skip]
const PREDICATES: [(&str, &str, &str, &str, u32); 83] = [
    (r#"pop_other<>1038288"#,                       r#"name<>'København'"#,
     r#"pop_other IS NULL"#,                        r#"name<'København'"#, 1),
    (r#"pop_other<>1038288"#,                       r#"name>'København'"#,
     r#"name<='København'"#,                        r#"boolean=true"#, 107),
    (r#"start IS NULL"#,                            r#"pop_other IS NOT NULL"#,
     r#"pop_other IS NOT NULL"#,                    r#"pop_other>1038288"#, 124),
    (r#"pop_other<1038288"#,                        r#"pop_other>1038288"#,
     r#"pop_other IS NULL"#,                        r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#, 121),
    (r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"pop_other<1038288"#,
     r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"name<>'København'"#, 2),
    (r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"name<>'København'"#,
     r#"boolean=true"#,                             r#"name<'København'"#, 2),
    (r#"pop_other=1038288"#,                        r#"start IS NULL"#,
     r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"boolean IS NOT NULL"#, 242),
    (r#"start IS NULL"#,                            r#"pop_other>1038288"#,
     r#"start IS NOT NULL"#,                        r#"name>'København'"#, 122),
    (r#"pop_other<1038288"#,                        r#"name<>'København'"#,
     r#"name='København'"#,                         r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
    (r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"name IS NOT NULL"#,
     r#"start IS NULL"#,                            r#"pop_other<1038288"#, 120),
    (r#"name>='København'"#,                        r#"start IS NOT NULL"#,
     r#"boolean=true"#,                             r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, 137),
    (r#"start IS NOT NULL"#,                        r#"name>='København'"#,
     r#"start IS NOT NULL"#,                        r#"name IS NOT NULL"#, 3),
    (r#"name IS NULL"#,                             r#"name<'København'"#,
     r#"pop_other IS NOT NULL"#,                    r#"boolean IS NOT NULL"#, 243),
    (r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"name>'København'"#,
     r#"pop_other=1038288"#,                        r#"name<'København'"#, 3),
    (r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"name<='København'"#,
     r#"boolean IS NULL"#,                          r#"name>'København'"#, 138),
    (r#"pop_other IS NOT NULL"#,                    r#"start IS NULL"#,
     r#"pop_other>=1038288"#,                       r#"name>'København'"#, 62),
    (r#"name='København'"#,                         r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"boolean=true"#,                             r#"pop_other IS NULL"#, 243),
    (r#"name>'København'"#,                         r#"pop_other<1038288"#,
     r#"pop_other>1038288"#,                        r#"name<='København'"#, 122),
    (r#"pop_other<>1038288"#,                       r#"name='København'"#,
     r#"name<='København'"#,                        r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#, 243),
    (r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"pop_other=1038288"#,                        r#"start IS NULL"#, 3),
    (r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"name<='København'"#,
     r#"boolean IS NULL"#,                          r#"name>'København'"#, 138),
    (r#"pop_other IS NOT NULL"#,                    r#"start IS NULL"#,
     r#"pop_other>=1038288"#,                       r#"name>'København'"#, 62),
    (r#"name='København'"#,                         r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"boolean=true"#,                             r#"pop_other IS NULL"#, 243),
    (r#"name>'København'"#,                         r#"pop_other<1038288"#,
     r#"pop_other>1038288"#,                        r#"name<='København'"#, 122),
    (r#"pop_other<>1038288"#,                       r#"name='København'"#,
     r#"name<='København'"#,                        r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#, 243),
    (r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"pop_other=1038288"#,                        r#"start IS NULL"#, 3),
    (r#"name<>'København'"#,                        r#"boolean=true"#,
     r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"start IS NULL"#, 2),
    (r#"name IS NULL"#,                             r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"name IS NULL"#, 243),
    (r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"name>'København'"#,
     r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"name IS NOT NULL"#, 3),
    (r#"name<>'København'"#,                        r#"pop_other<>1038288"#,
     r#"pop_other<1038288"#,                        r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
    (r#"boolean IS NULL"#,                          r#"pop_other>1038288"#,
     r#"boolean IS NOT NULL"#,                      r#"pop_other IS NULL"#, 122),
    (r#"pop_other=1038288"#,                        r#"start IS NULL"#,
     r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"pop_other IS NOT NULL"#, 2),
    (r#"pop_other<>1038288"#,                       r#"start IS NULL"#,
     r#"pop_other>1038288"#,                        r#"boolean=true"#, 2),
    (r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"pop_other<1038288"#,
     r#"name<='København'"#,                        r#"pop_other=1038288"#, 2),
    (r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"name<='København'"#,                        r#"name<>'København'"#, 107),
    (r#"boolean=true"#,                             r#"name IS NOT NULL"#,
     r#"boolean IS NULL"#,                          r#"pop_other=1038288"#, 1),
    (r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"pop_other=1038288"#,
     r#"pop_other<1038288"#,                        r#"name<>'København'"#, 122),
    (r#"pop_other<>1038288"#,                       r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"start IS NOT NULL"#,                        r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#, 3),
    (r#"name<>'København'"#,                        r#"pop_other<>1038288"#,
     r#"pop_other IS NOT NULL"#,                    r#"name IS NOT NULL"#, 243),
    (r#"name='København'"#,                         r#"pop_other<1038288"#,
     r#"start IS NOT NULL"#,                        r#"pop_other<>1038288"#, 3),
    (r#"name<'København'"#,                         r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
    (r#"boolean=true"#,                             r#"pop_other<1038288"#,
     r#"name IS NOT NULL"#,                         r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#, 3),
    (r#"pop_other<=1038288"#,                       r#"name<'København'"#,
     r#"pop_other<1038288"#,                        r#"pop_other<1038288"#, 243),
    (r#"pop_other IS NULL"#,                        r#"name<='København'"#,
     r#"name='København'"#,                         r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
    (r#"pop_other<1038288"#,                        r#"name<>'København'"#,
     r#"pop_other<>1038288"#,                       r#"name<>'København'"#, 243),
    (r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"pop_other IS NULL"#,
     r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"name IS NOT NULL"#, 2),
    (r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"name='København'"#,
     r#"boolean IS NULL"#,                          r#"pop_other<>1038288"#, 241),
    (r#"boolean=true"#,                             r#"pop_other<=1038288"#,
     r#"name<>'København'"#,                        r#"pop_other IS NULL"#, 2),
    (r#"name IS NOT NULL"#,                         r#"pop_other<=1038288"#,
     r#"start IS NOT NULL"#,                        r#"boolean IS NOT NULL"#, 124),
    (r#"pop_other<=1038288"#,                       r#"pop_other<1038288"#,
     r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"pop_other>1038288"#, 1),
    (r#"start IS NOT NULL"#,                        r#"boolean IS NOT NULL"#,
     r#"name>='København'"#,                        r#"pop_other IS NOT NULL"#, 137),
    (r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"start IS NOT NULL"#,
     r#"pop_other>1038288"#,                        r#"pop_other<1038288"#, 1),
    (r#"pop_other<=1038288"#,                       r#"name<='København'"#,
     r#"boolean IS NULL"#,                          r#"start IS NOT NULL"#, 198),
    (r#"name>='København'"#,                        r#"name>='København'"#,
     r#"name<='København'"#,                        r#"name>='København'"#, 107),
    (r#"boolean=true"#,                             r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"boolean IS NOT NULL"#,                      r#"name<'København'"#, 2),
    (r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"pop_other IS NULL"#,                        r#"pop_other<=1038288"#, 1),
    (r#"pop_other<1038288"#,                        r#"name='København'"#,
     r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"name<'København'"#, 181),
    (r#"pop_other<1038288"#,                        r#"pop_other<=1038288"#,
     r#"pop_other IS NULL"#,                        r#"start IS NOT NULL"#, 121),
    (r#"name>='København'"#,                        r#"pop_other>=1038288"#,
     r#"boolean=true"#,                             r#"name IS NOT NULL"#, 79),
    (r#"boolean IS NULL"#,                          r#"name<>'København'"#,
     r#"boolean IS NULL"#,                          r#"pop_other IS NOT NULL"#, 240),
    (r#"pop_other<1038288"#,                        r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"name>'København'"#,                         r#"pop_other<=1038288"#, 199),
    (r#"name<='København'"#,                        r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"name<'København'"#,                         r#"boolean IS NULL"#, 106),
    (r#"pop_other IS NOT NULL"#,                    r#"name<>'København'"#,
     r#"pop_other<1038288"#,                        r#"pop_other<=1038288"#, 121),
    (r#"name>='København'"#,                        r#"start IS NOT NULL"#,
     r#"name>='København'"#,                        r#"name IS NOT NULL"#, 137),
    (r#"pop_other<1038288"#,                        r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"name IS NULL"#,                             r#"pop_other>=1038288"#, 1),
    (r#"pop_other>=1038288"#,                       r#"name>'København'"#,
     r#"boolean IS NOT NULL"#,                      r#"start IS NOT NULL"#, 184),
    (r#"start IS NOT NULL"#,                        r#"name<>'København'"#,
     r#"name<='København'"#,                        r#"name IS NULL"#, 241),
    (r#"name>='København'"#,                        r#"pop_other<>1038288"#,
     r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"name<>'København'"#, 2),
    (r#"boolean IS NOT NULL"#,                      r#"pop_other<=1038288"#,
     r#"pop_other=1038288"#,                        r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#, 1),
    (r#"name IS NOT NULL"#,                         r#"start IS NOT NULL"#,
     r#"start IS NOT NULL"#,                        r#"name>='København'"#, 241),
    (r#"pop_other=1038288"#,                        r#"pop_other IS NOT NULL"#,
     r#"start IS NOT NULL"#,                        r#"name<>'København'"#, 2),
    (r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"start IS NULL"#,
     r#"pop_other>1038288"#,                        r#"pop_other<=1038288"#, 1),
    (r#"name IS NULL"#,                             r#"start IS NOT NULL"#,
     r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"name IS NOT NULL"#, 1),
    (r#"boolean IS NOT NULL"#,                      r#"name='København'"#,
     r#"boolean IS NOT NULL"#,                      r#"name IS NOT NULL"#, 3),
    (r#"pop_other<>1038288"#,                       r#"pop_other<>1038288"#,
     r#"pop_other=1038288"#,                        r#"pop_other<=1038288"#, 1),
    (r#"pop_other IS NULL"#,                        r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"boolean IS NOT NULL"#, 241),
    (r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"boolean IS NULL"#,
     r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"name<'København'"#, 2),
    (r#"pop_other>1038288"#,                        r#"pop_other<>1038288"#,
     r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"name<>'København'"#, 2),
    (r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"pop_other=1038288"#,                        r#"name IS NOT NULL"#, 2),
    (r#"pop_other<=1038288"#,                       r#"start IS NOT NULL"#,
     r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#, r#"boolean IS NOT NULL"#, 242),
    (r#"boolean=true"#,                             r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"pop_other<1038288"#,                        r#"pop_other<>1038288"#, 122),
    (r#"pop_other>=1038288"#,                       r#"pop_other>1038288"#,
     r#"boolean IS NULL"#,                          r#"pop_other=1038288"#, 121),
    (r#"name<'København'"#,                         r#"pop_other>1038288"#,
     r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  r#"boolean=true"#, 44)
];

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let features = places()?;

    // let _c9_ = tracing::span!(tracing::Level::DEBUG, "L1").entered();
    // let _c9_start = _c9_.enter();

    let shared_ctx = Context::new().freeze();
    for (ndx, (p1, p2, p3, p4, expected)) in PREDICATES.iter().enumerate() {
        let input = format!("(NOT ({p2}) AND {p1}) OR ({p3} and {p4}) or not ({p1} OR {p4})");
        // tracing::debug!("input = {input}");
        let exp = Expression::try_from_text(&input)?;
        // tracing::debug!("exp = {exp:?}");

        let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
        evaluator.setup(exp)?;

        let mut actual = 0;
        for feat in &features {
            let res = evaluator
                .evaluate(&feat)
                .expect(&format!("Failed {feat:?}"));
            if matches!(res, Outcome::T) {
                // tracing::debug!(":) --- {feat:?}\n");
                actual += 1;
            } else {
                // tracing::debug!(":( --- {feat:?}\n");
            }
        }

        evaluator.teardown()?;

        tracing::debug!("Predicate #{ndx} - actual / expected: {actual} / {expected}");
        assert_eq!(actual, *expected)
    }

    Ok(())
}
