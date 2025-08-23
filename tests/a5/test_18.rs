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

use crate::utils::{PLACES, harness};
use std::error::Error;
use tracing_test::traced_test;

#[rustfmt::skip]
const PREDICATES: [(&str, &str, &str, &str, u32); 10] = [
    (r#"CASEI(name)=casei('KIEV')"#,         r#"CASEI(name)=casei('København')"#,
     r#"CASEI(name) LIKE casei('B_r%')"#,    r#"CASEI(name) LIKE casei('B_R%')"#,    243),

    (r#"CASEI(name) LIKE casei('B_R%')"#,    r#"CASEI(name)=casei('KIEV')"#,
     r#"CASEI(name)=casei('København')"#,    r#"CASEI(name) LIKE casei('B_r%')"#,    240),

    (r#"CASEI(name) LIKE casei('B_r%')"#,    r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#,
     r#"CASEI(name) LIKE casei('B_R%')"#,    r#"CASEI(name)=casei('København')"#,    243),

    (r#"CASEI(name)=casei('københavn')"#,    r#"CASEI(name) LIKE casei('B_r%')"#,
     r#"CASEI(name)=casei('KIEV')"#,         r#"CASEI(name) LIKE casei('BERliN')"#,  243),

    (r#"CASEI(name) LIKE casei('BERli_')"#,  r#"CASEI(name)=casei('københavn')"#,
     r#"CASEI(name) LIKE casei('b_r%')"#,    r#"CASEI(name)=casei('KIEV')"#,         243),

    (r#"CASEI(name)=casei('kiev')"#,         r#"name<'København'"#,
     r#"CASEI(name)=casei('københavn')"#,    r#"CASEI(name) LIKE casei('b_r%')"#,    243),

    (r#"CASEI(name) LIKE casei('b_r%')"#,    r#"CASEI(name)=casei('kiev')"#,
     r#"CASEI(name)=casei('ATHENS')"#,       r#"CASEI(name)=casei('københavn')"#,   243),

    (r#"CASEI(name)=casei('KØBENHAVN')"#,    r#"CASEI(name) LIKE casei('b_r%')"#,
     r#"CASEI(name)=casei('kiev')"#,         r#"boolean IS NULL"#,                  243),

    (r#"name>='København'"#,                 r#"CASEI(name)=casei('KØBENHAVN')"#,
     r#"CASEI(name) LIKE casei('BERliN')"#,  r#"CASEI(name)=casei('kiev')"#,        243),

    (r#"CASEI(name)=casei('København')"#,    r#"pop_other>1038288"#,
     r#"CASEI(name)=casei('KØBENHAVN')"#,    r#"pop_other=1038288"#,                242),
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

    harness(PLACES, predicates)
}
