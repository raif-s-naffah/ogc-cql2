// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! OGC CQL2 parser and runtime interpreter.
//!
//! The next paragraphs explain in more details the elements of this project
//! as well as the rationale behind some of the decisions that shaped its
//! components.
//!
//! # Expressions
//!
//! The kernel of this project is OGC CQL2 Expressions represented by the
//! [`Expression`] enumeration. The two variants: [`TextEncoded`] and [`JsonEncoded`]
//! respectively represent the text-based and json-based mandated representations.
//!
//! Parsing user-provided input is done by invoking one of the following two
//! methods: [`Expression::try_from_text()`] and [`Expression::try_from_json()`]
//! as shown in the following example:
//! ```rust
//! use ogc_cql2::prelude::*;
//! use std::error::Error;
//!
//! # fn test() -> Result<(), Box<dyn Error>> {
//! let expr = Expression::try_from_text(r#""name" NOT LIKE 'foo%' AND "value" > 10"#)?;
//! // ...
//! let expr = Expression::try_from_json(r#"
//! {
//!  "op": "t_finishes",
//!  "args": [
//!    { "interval": [ { "property": "starts_at" }, { "property": "ends_at" } ] },
//!    { "interval": [ "1991-10-07", "2010-02-10T05:29:20.073225Z" ] }
//!  ]
//! }"#)?;
//! #    Ok(())
//! # }
//! ```
//! An `Ok` result implies a syntactically correct parsed expression!
//!
//! For convenience, a standalone tool is included that can be used from the
//! command line to quickly test the vailidity of candidate expressions.
//!
//! Once the library is built (`cargo b↵`), it can be invoked by calling:
//! ```bash
//! cargo r --bin repl↵
//! ```
//! Read more about it [here](https://github.com/raif-s-naffah/ogc-cql2/blob/master/doc/REPL.md)
//!
//! # Evaluators
//!
//! An OGC CQL2 _Expression_ on its own is close to useless unless it is evaluated
//! against, what the (CQL2) standard refers to as [`Resource`]s. A [`Resource`]
//! here is essentially a _Map_ of property names (i.e. strings) to [queryable][Q]
//! values. More on that later.
//!
//! This library filters those [Resources][Resource] with objects that implement the
//! [`Evaluator`] trait. A simple example of an implementation of this trait is provided
//! --see [`ExEvaluator`].
//!
//! In an earlier incarnation an [`Evaluator`] used to have a `teardown()` hook.
//! Not anymore. Rust's [`Drop` trait](https://doc.rust-lang.org/std/ops/trait.Drop.html)
//! makes that method superfluous.
//!
//! # Data sources
//!
//! Data sources represent providers of data to be processed by [`Evaluator`]s to
//! filter (i.e. include or exclude) resources based on the result of [`Expression`]s.
//!
//! The [`DataSource`] (marker) trait represents those objects. Currently the
//! library provides three implementations: [`CSVDataSource`], [`GPkgDataSource`],
//! and [`PGDataSource`].  The first represents _Comma Separated Values_ (CSV) sourced
//! from tabular data where each row is mapped to a _Feature_ containing one geometry
//! (spatial) property and other non-geometry attributes. The second represents
//! [GeoPackage][gpkg] files. A _GeoPackage_ is
//! > ... _an open, standards-based, platform-independent, portable,
//! > self-describing, cocurrentlympact format for transferring geospatial information.
//! > It is a platform-independent SQLite database file_...
//!
//! The third [`PGDataSource`] allows processing [_PostGIS_](https://postgis.net/) enabled
//! [_PostgreSQL_](https://www.postgresql.org/) tables.
//!
//! Coding concrete implementations of those data source traits is facilitated
//! by the library macros: [gen_csv_ds!], [gen_gpkg_ds!], and [gen_pg_ds!] for the three
//! supported [`DataSource`] implementations.
//!
//! # GIS Features and Resources
//!
//! I frequently mention the term _GIS Feature_ in the documentation to refer
//! to an abstract GIS type that closely relates to its data source, as opposed
//! to a Rust _Conditional Compilation Feature_, or simply _Feature_. For a CSV
//! data source, a _GIS Feature_ is a structure that is [`serde`](https://serde.rs/)
//! deserializable.
//! For example, in the `tests/samples/data` folder, a CSV file named
//! `ne_110m_rivers_lake_centerlines` representing one of the 3 data sets
//! referred to in the standard for testing compliance is provided. The _GIS
//! Feature_ for that data source looks like this:
//! ```rust
//! use serde::Deserialize;
//! use std::marker::PhantomData;
//!
//! #[cfg(feature = "csv_ds")]
//! #[derive(Debug, Default, Deserialize)]
//! pub(crate) struct ZRiver {
//!     /* 0 */ fid: i32,
//!     /* 1 */ geom: String,
//!     /* 2 */ name: String,
//!     #[serde(skip)] ignored: PhantomData<String>
//! }
//! ```
//! This makes sense b/c the [csv crate](https://crates.io/crates/csv) used for
//! reading the _CSV_ data works smoothly with deserializable structures.
//! Worth noting here that the spatial data (the `geom` field) is expected to
//! be encoded as [WKT (Well Known Text)](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry).
//!
//! When dealing w/ a _GeoPackage_ version of the same data, this structure is used:
//! ```rust
//! #[cfg(feature = "gpkg_ds")]
//! #[derive(Debug, sqlx::FromRow)]
//! pub(crate) struct TRiver {
//!     fid: i32,
//!     geom: Vec<u8>,
//!     name: String,
//! }
//! ```
//! As one can see this best suits the [sqlx crate](https://crates.io/crates/sqlx)
//! used for reading _GeoPackage_ data. In this type of _Feature_ the same
//! `geom` spatial attribute is now expected to be a byte array containing the
//! WKB (Well Known Binary) encoded value of the vector geometry.
//!
//! Similarly, when dealing w/ a _PostGIS_ version of the same data, this structure
//! is used:
//!
//! ```rust
//! use ogc_cql2::prelude::*;
//!
//! #[cfg(feature = "pg_ds")]
//! #[derive(Debug, sqlx::FromRow)]
//! pub(crate) struct LRiver {
//!     fid: i32,
//!     name: String,
//!     geom: G,
//! }
//!
//! ```
//!
//! Finally on that note, a _GIS Feature_ implementation must provide a way of
//! converting an instance of `Self` to a [`Resource`]. Here it is for the
//! above _rivers_ CSV version, when feature `csv_ds` is activated:
//!
//! ```rust
//! # use ogc_cql2::prelude::*;
//! # use std::{collections::HashMap, error::Error, marker::PhantomData};
//! # use serde::Deserialize;
//! #
//! # #[derive(Debug, Default, Deserialize)]
//! # struct ZRiver {
//! #    /* 0 */ fid: i32,
//! #    /* 1 */ geom: String,
//! #    /* 2 */ name: String,
//! #    #[serde(skip)] ignored: PhantomData<String>
//! # }
//!
//! #[cfg(feature = "csv_ds")]
//! impl TryFrom<ZRiver> for Resource {
//!     type Error = MyError;
//!
//!     fn try_from(value: ZRiver) -> Result<Self, Self::Error> {
//!         use ogc_cql2::prelude::*;
//!
//!         Ok(HashMap::from([
//!             ("fid".into(), Q::try_from(value.fid)?),
//!             ("geom".into(), Q::try_from_wkt(&value.geom)?),
//!             ("name".into(), Q::new_plain_str(&value.name)),
//!         ]))
//!     }
//! }
//! ```
//!
//! A [`Resource`] on the other hand, as mentioned earlier, is generic in the
//! sense that it's a simple map of propery names to values in a similar vain
//! to how JSON objects are handled. In the same vain as how `serde` models
//! JSON values, the types of _value_ a _resource's_ queryable, property, or
//! attribute are embodied by the [Queryable][Q] enumeration.
//!
//! Note though that this _resource_ genericity is too expensive in terms of
//! performance.
//!
//! # Iterable and Streamable
//!
//! Access to the contents of a [`DataSource`] is possible by implementing
//! one or both of the two traits: [`IterableDS`] and [`StreamableDS`].
//!
//! The first exposes a method ([`iter()`][IterableDS::iter()]) that returns an
//! [_Iterator_](https://doc.rust-lang.org/std/iter/trait.Iterator.html) over
//! the _Features_ of the data source.
//!
//! Considering that the [`CSVDataSource`] related macro [`gen_csv_ds!`] does
//! exactly that, one can easily write something like this...
//!
//! ```rust
//! use ogc_cql2::prelude::*;
//! use std::error::Error;
//! # use std::fs::File;
//! # use std::collections::HashMap;
//! # use serde::Deserialize;
//! # use std::marker::PhantomData;
//! # #[derive(Debug, Default, Deserialize)]
//! # struct ZRiver {
//! #    /* 0 */ fid: i32,
//! #    /* 1 */ geom: String,
//! #    /* 2 */ name: String,
//! #    #[serde(skip)] ignored: PhantomData<String>
//! # }
//! # impl TryFrom<ZRiver> for Resource {
//! #    type Error = MyError;
//! #
//! #    fn try_from(value: ZRiver) -> Result<Self, Self::Error> {
//! #        Ok(HashMap::from([
//! #            ("fid".into(), Q::try_from(value.fid)?),
//! #            ("geom".into(), Q::try_from_wkt(&value.geom)?),
//! #            ("name".into(), Q::new_plain_str(&value.name)),
//! #        ]))
//! #    }
//! # }
//!
//! # #[cfg(feature = "csv_ds")]
//! # fn test() -> Result<(), Box<dyn Error>> {
//! // somewhere the macro is invoked to generate module-private artifacts...
//! gen_csv_ds!(pub(crate), "River", "...ne_110m_rivers_lake_centerlines.csv", ZRiver);
//!
//! // now we collect all the "rivers" in the collection...
//! let csv = RiverCSV::new();
//! let it: Result<Vec<ZRiver>, MyError> = csv.iter()?.collect();
//! // ...
//! #     Ok(())
//! # }
//! ```
//! The [`StreamableDS`] trait is more versatile. It exposes methods to stream
//! asynchronously the contents as _Features_ ([`fetch()`][StreamableDS::fetch()]
//! and [`fetch_where()`][StreamableDS::fetch_where()]) and _Resources_
//! ([`stream()`][StreamableDS::stream()] and [`stream_where()`][StreamableDS::stream_where()]).
//! The methods with the `_where` suffix expect an [`Expression`] argument that
//! will be delegated to the data source itself to use for _filtering_ the
//! contents in the best way it can; e.g. SQL WHERE clause for a _GeoPackage_
//! file, and _PostGIS_ DB tables, etc...
//!
//! Similar to the CSV data source, the [`gen_gpkg_ds!`] macro does the heavy
//! lifting generating the necessary artifcats for a _GeoPackage_ data-source.
//! As does the [`gen_pg_ds!`] macro for a _PostGIS_ data-source.
//!
//!
//! # Relative performance
//!
//! With the introduction of the [`DataSource`], [`IterableDS`] and [`StreamableDS`]
//! traits and the provided [`DataSource`] implementations, a User can
//! effectively process the data in 3 ways:
//!
//! * as _Features_ using the [`IterableDS`] trait --from a _CSV_ table.
//! * as either _Features_ or [`Resource`]s using the [`StreamableDS`] trait through
//!   the `fetch()` or `stream()` hooks --from a _GeoPackage_ database file,
//!   or a _PostGIS_ layer,
//! * as _Features_ or [`Resource`]s using the [`StreamableDS`] trait through the
//!   `fetch_where()` or `stream_where()` hooks --from a _GeoPackage_ DB or a
//!   _PostGIS_ layer.
//!
//! The last approach is by far the most effective since it delegates to a
//! DB engine the job of filtering the records, while the 2<sup>nd</sup> one
//! is the worst b/c it involves converting every _Feature_ to a [`Resource`]
//! even when we may not need all the queryables from that newly created
//! [`Resource`].
//!
//! As an example of relative performance of those approaches, consider the
//! timing of `test_points`, `test_points_gpkg` and `test_points_sql` in
//! `a9::test_37` which correspond to those 3 strategies respectively when
//! processing a data set of 243 records. On a 4×Intel® Core™ i7-3610QM CPU
//! @ 2.30GHz laptop, w/ the `profile [unoptimized + debuginfo]` I get...
//!
//! | # | test                 | time  |
//! |---|----------------------|-------|
//! | 1 | `test_points()`      | 0.09s |
//! | 2 | `test_points_gpkg()` | 5.35s |
//! | 3 | `test_points_sql()`  | 0.11s |
//!
//!
//! # Third-party crates
//!
//! This project, in addition to the external software mentioned in the [README][readme],
//! relies on few 3<sup>rd</sup> party crates. In addition to the `csv`, and `sqlx`
//! crates already mentioned, here are the most important ones...
//!
//! 1. PEG
//!    * [`peg`](https://crates.io/crates/peg): Provides a Rust macro that builds
//!      a recursive descent parser from a concise definition of a grammar.
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
//! # Rust (conditional compliation) Features
//!
//! This project currently allows users the ability to process GIS data
//! provided by different data-source types, gated by the following features:
//!
//! | Feature   | Data Source                                         |
//! |-----------|-----------------------------------------------------|
//! | `csv_ds`  | Comma Separated Values (CSV) textual data.<br/>Geometry columns are Well Known Text (WKT) strings.<br/>Properties are formatted strings. |
//! | `gpkg_ds` | _GeoPackage_ data.<br/>Geometry columns are Well Known Binary (WKB) bytes.<br/>Properties are standard SQLite types. |
//! | `pg_ds`   | _PostGIS_ tables.<br/>Geometry columns are Well Known Binary (WKB) bytes.<br/>Properties are standard PostgreSQL types. |
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
//! [gpkg]: https://www.geopackage.org/spec140/index.html
//! [shapefile]: https://en.wikipedia.org/wiki/Shapefile
//! [pgis]: https://en.wikipedia.org/wiki/PostGIS
//! [sqlx]: https://crates.io/crates/sqlx
//! [readme]: https://crates.io/crates/xapi-rs
//!

#![doc = include_str!("../doc/FUNCTION.md")]
#![doc = include_str!("../doc/CONFIGURATION.md")]

mod bound;
mod config;
mod context;
mod crs;
mod ds;
mod error;
mod evaluator;
mod expr;
mod function;
mod geom;
mod json;
mod op;
mod qstring;
mod queryable;
mod srid;
mod text;
mod wkb;

pub use bound::*;
pub use context::*;
pub use crs::*;
pub use ds::*;
pub use evaluator::*;
pub use function::*;
pub use geom::*;
pub use qstring::QString;
pub use queryable::*;
pub use srid::*;

pub mod prelude;

use crate::{expr::E, text::cql2::expression};
use core::fmt;
pub use error::MyError;

/// An instance of an OGC CQL2 filter.
#[derive(Debug)]
pub enum Expression {
    /// Instance generated from a successfully parsed text-encoded input string.
    Text(TextEncoded),
    /// Instance generated from a successfully parsed JSON-encoded input string.
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

    // convert both variants to the common `E` intermediary form.
    #[allow(dead_code)]
    pub(crate) fn to_inner(&self) -> Result<E, MyError> {
        match self {
            Expression::Text(x) => Ok(x.0.to_owned()),
            Expression::Json(x) => {
                let s = &x.0.to_string();
                let te = Self::try_from_text(s)?;
                let it = te
                    .as_text_encoded()
                    .ok_or_else(|| MyError::Runtime("Failed converting to TE".into()))?;
                Ok(it.0.to_owned())
            }
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

/// Text-encoded CQL2 [`Expression`].
#[derive(Debug, PartialEq)]
pub struct TextEncoded(expr::E);

/// JSON-encoded CQL2 [`Expression`].
#[derive(Debug)]
pub struct JsonEncoded(json::Expression);

/// Possible outcome values when evaluating an [`Expression`] against an
/// individual [`Resource`] from a collection.
///
/// From [OGC CQL2](https://docs.ogc.org/is/21-065r2/21-065r2.html):
/// > _Each resource instance in the source collection is evaluated against
/// > a filtering expression. The net effect of evaluating a filter
/// > [`Expression`] is a subset of resources that satisfy the predicate(s)
/// > in the [`Expression`]._
///
/// Logically connected predicates are evaluated according to the following
/// truth table, where `T` is TRUE, `F` is FALSE and `N` is NULL.
///
/// | P1  | P2  | P1 & P2 | P1 &#124; P2 |
/// |-----|-----|:-------:|:-------:|
/// |  T  |  T  |    T    |    T    |
/// |  T  |  F  |    F    |    T    |
/// |  F  |  T  |    F    |    T    |
/// |  F  |  F  |    F    |    F    |
/// |  T  |  N  |    N    |    T    |
/// |  F  |  N  |    F    |    N    |
/// |  N  |  T  |    N    |    T    |
/// |  N  |  F  |    F    |    N    |
/// |  N  |  N  |    N    |    N    |
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
