// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! SQL generation capability for use by a data source.
//!

/// SQL string to use in lieue of unbounded interval limits.
/// NOTE (rsn) 2025112 - although PostgreSQL documentation states that that
/// "low" value is `4713 BC` this seems to be the lowest value i can use w/o
/// PostgreSQL 18.beta-1 throwing a tantrum.  it also works fine w/ SQLite.
pub(crate) const MIN_DATE_SQL: &str = "'-2021-01-01'";

// Given two _Expressions_ `$a` and `$b`, check whether they're _Intervals_ or
// not and compute a tuple that represents the result along w/ four expressions
// representing the entities to use in formulating comparison predicates that
// will reflect a desired CQL2 date/time function.
#[doc(hidden)]
#[macro_export]
macro_rules! unfold_expressions {
    ( $a: expr, $b: expr ) => {{
        let a_is_interval = $a.is_interval();
        let b_is_interval = $b.is_interval();
        match (a_is_interval, b_is_interval) {
            (false, false) => (false, false, $a, E::Null, $b, E::Null),
            (false, true) => {
                let t2 = $b.as_interval().expect("2nd argument is NOT an interval");
                (false, true, $a, E::Null, t2.0, t2.1)
            }
            (true, false) => {
                let t1 = $a.as_interval().expect("1st argument is NOT an interval");
                (true, false, t1.0, t1.1, $b, E::Null)
            }
            (true, true) => {
                let t1 = $a.as_interval().expect("1st argument is NOT an interval");
                let t2 = $b.as_interval().expect("2nd argument is NOT an interval");
                (true, true, t1.0, t1.1, t2.0, t2.1)
            }
        }
    }};
}

// Similar to `unfold_expressions!` except that it always expects the arguments
// to be _Intervals_.
#[doc(hidden)]
#[macro_export]
macro_rules! unfold_intervals {
    ( $a: expr, $b: expr ) => {{
        let t1 = $a.as_interval().expect("1st argument is NOT an interval");
        let t2 = $b.as_interval().expect("2nd argument is NOT an interval");
        (t1.0, t1.1, t2.0, t2.1)
    }};
}

// Augment a given `$sql` fragment by appending a `<x> IS NOT NULL` fragment(s)
// if either or both `$a` and `$b` are _Identifiers_.
#[doc(hidden)]
#[macro_export]
macro_rules! check_ids {
    ( $a: expr, $sql: expr ) => {{
        if $a.is_id() {
            let id = $a.as_id().expect("Argument is not an Identifier");
            format!("\"{}\" IS NOT NULL AND ({})", id, $sql)
        } else {
            $sql
        }
    }};

    ( $a: expr, $b: expr, $sql: expr ) => {{
        match ($a.is_id(), $b.is_id()) {
            (true, true) => {
                let id1 = $a.as_id().expect("1st argument is not an Identifier");
                let id2 = $b.as_id().expect("2nd argument is not an Identifier");
                format!(
                    "\"{}\" IS NOT NULL AND \"{}\" NOT NULL AND ({})",
                    id1, id2, $sql
                )
            }
            (true, false) => {
                let id = $a.as_id().expect("1st argument is not an Identifier");
                format!("\"{}\" IS NOT NULL AND ({})", id, $sql)
            }
            (false, true) => {
                let id = $b.as_id().expect("2nd argument is not an Identifier");
                format!("\"{}\" IS NOT NULL AND ({})", id, $sql)
            }
            (false, false) => $sql,
        }
    }};
}
