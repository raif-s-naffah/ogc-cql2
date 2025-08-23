// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! OGC CQL2 parser and runtime interpreter...
//!
//! # Third-party crates
//!
//! This project relies on few 3<sup>rd</sup> party crates to satisfy the requirements of
//! the CQL2 standard. These are...
//!
//! 1. Geometry
//!    * [`geos`][1]: Rust bindings for [GEOS][2] C API for handling geometries.
//!
//! 2. JSON Deserialization:
//!    * [serde][3]: for the basic capabilities.
//!    * [serde_json][4]: for the JSON format bindings.
//!    * [serde_with][5]: for custom helpers.
//!
//! 3. Date + Time:
//!    * [jiff][6]: for time-zone-aware date and timestamp handling.
//!
//! 4. Case + Accent Insensitive Strings:
//!    * [unicase][7]: for comparing strings when case is not important.
//!    * [unicode-normalization][8]: for un-accenting strings w/ Unicode
//!      decomposition.
//!
//! 5. CRS Transformation:
//!    * [proj][9]: for coordinate transformation via bindings to the [PROJ][10]
//!      API.
//!
//!
//!
//! [1]: https://crates.io/crates/geos
//! [2]: https://libgeos.org/
//! [3]: https://crates.io/crates/serde
//! [4]: https://crates.io/crates/serde_json
//! [5]: https://crates.io/crates/serde_with
//! [6]: https://crates.io/crates/jiff
//! [7]: https://crates.io/crates/unicase
//! [8]: https://crates.io/crates/unicode-normalization
//! [9]: https://crates.io/crates/proj
//! [10]: https://proj.org/
//!

#![doc = include_str!("../doc/FUNCTION.md")]

mod bound;
mod config;
mod context;
mod crs;
mod error;
mod evaluator;
mod expr;
mod function;
mod geom;
mod json;
mod op;
mod qstring;
mod queryable;
mod text;

pub mod prelude;

use crate::{expr::E, text::cql2::expression};
use core::fmt;
pub use error::MyError;
use std::collections::HashMap;

pub use bound::*;
pub use context::*;
pub use crs::*;
pub use evaluator::*;
pub use function::*;
pub use geom::*;
pub use qstring::QString;
pub use queryable::*;

/// Default number of decimals to show in coordinates. For coordinates in
/// WGS 84 this translates to approx. `11.1` cm. accuracy when projecting
/// coordinates in that reference system to EPSG:3857 (Web Mercator in m.).
pub const DEFAULT_PRECISION: usize = 6;

#[derive(Debug)]
/// An instance of an OGC CQL2 filter.
pub enum Expression {
    /// Instance generated from a valid text-encoded input string.
    Text(TextEncoded),
    /// Instance generated from a valid JSON-encoded input string.
    Json(Box<JsonEncoded>),
}

impl Expression {
    /// Try to construct from a text-encoded string.
    pub fn try_from_text(s: &str) -> Result<Self, MyError> {
        let x = expression(s).map_err(MyError::Text)?;
        Ok(Expression::Text(TextEncoded(x)))
    }

    /// Try to construct from a JSON-encoded string.
    pub fn try_from_json(s: &str) -> Result<Self, MyError> {
        let x = serde_json::from_str::<json::Expression>(s).map_err(MyError::Json)?;
        Ok(Expression::Json(Box::new(JsonEncoded(x))))
    }

    /// Return a reference to the text-encoded variant as an `Option`.
    pub fn as_text_encoded(&self) -> Option<&TextEncoded> {
        match self {
            Expression::Text(x) => Some(x),
            Expression::Json(_) => None,
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Text(x) => write!(f, "{}", x.0),
            Expression::Json(x) => write!(f, "{}", x.0),
        }
    }
}

/// Text-encoded CQL2 Expression.
#[derive(Debug, PartialEq)]
pub struct TextEncoded(expr::E);

/// JSON-encoded CQL2 Expression.
#[derive(Debug)]
pub struct JsonEncoded(json::Expression);

/// Possible outcome values when evaluating an [Expression] against an
/// individual _Resource_ from a collection.
///
/// From [OGC CQL2][1]:
/// > _Each resource instance in the source collection is evaluated against
/// > a filtering expression. The net effect of evaluating a filter
/// > [Expression] is a subset of resources that satisfy the predicate(s)
/// > in the [Expression]._
///
/// Logically connected predicates are evaluated according to the following
/// truth table, where `T` is TRUE, `F` is FALSE and `N` is NULL.
/// ```text
/// +-----+-----+---------+---------+
/// | P1  | P2  | P1 & P2 | P1 | P2 |
/// +-----+-----+---------+---------+
/// |  T  |  T  |    T    |    T    |
/// |  T  |  F  |    F    |    T    |
/// |  F  |  T  |    F    |    T    |
/// |  F  |  F  |    F    |    F    |
/// |  T  |  N  |    N    |    T    |
/// |  F  |  N  |    F    |    N    |
/// |  N  |  T  |    N    |    T    |
/// |  N  |  F  |    F    |    N    |
/// |  N  |  N  |    N    |    N    |
/// +-----+-----+---------+---------+
/// ```
/// [1]: https://docs.ogc.org/is/21-065r2/21-065r2.html
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The input satisfies the [Expression] and should be marked as being in
    /// the result set.
    T,
    /// The input does not satisfy the filter [Expression] and should not be
    /// included the result set.
    F,
    /// Likewise.
    N,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::T => write!(f, "T"),
            Outcome::F => write!(f, "F",),
            Outcome::N => write!(f, "N"),
        }
    }
}

impl Outcome {
    /// Constructor from an optional boolean.
    pub fn new(flag: Option<&bool>) -> Self {
        match flag {
            Some(b) => match b {
                true => Self::T,
                false => Self::F,
            },
            None => Self::N,
        }
    }
}

/// A dictionary of queryable property names (strings) to [Q] values.
/// [Queryables][Q] have same lifetime as their parent.
pub type Resource = HashMap<String, Q>;

/// Internal Queryable type variants.
#[derive(Debug)]
pub(crate) enum DataType {
    /// A Unicode UTF-8 string.
    Str,
    /// A numeric value including integers and floating points.
    Num,
    /// A boolean value.
    Bool,
    /// An _Instant_ with a granularity of a second or smaller. Timestamps are
    /// always in the time zone UTC ("Z").
    Timestamp,
    /// An _Instant_ with a granularity of a day. Dates are local without an
    /// associated time zone.
    Date,
    /// A temporal range of 2 _Instants_ each either _fixed_ or _unbounded_.
    #[allow(dead_code)]
    Interval,
    /// A spatial (geometry) value.
    Geom,
    /// A collection of homogeneous values.
    #[allow(dead_code)]
    List,
}
