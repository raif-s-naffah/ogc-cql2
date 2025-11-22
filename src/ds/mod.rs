// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Groups artifacts providing Features and Resources from known types used in
//! the geospatial ecosystem such as GeoPackage files.
//!

mod csv;
mod gpkg;

pub use csv::*;
pub use gpkg::*;

use crate::{Expression, Q};
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::collections::HashMap;

/// A dictionary of queryable property names (strings) to [`Queryable`][Q] values.
pub type Resource = HashMap<String, Q>;

/// _Marker_ trait for a type that can act as a data source provider of _Features_
/// and [`Resource`]s in the context of processing CQL2 filter expressions.
///
/// A CSV file, and a GeoPackage autonomous database file are examples of this.
#[allow(dead_code)]
pub trait DataSource {}

/// Capability of a [`DataSource`] to provide an iterator over a collection of
/// _Features_ or [Resources][Resource].
pub trait IterableDS {
    /// Type representing a Feature which must be convertible to a [Resource].
    type Item: TryInto<Resource, Error = Self::Err>;
    /// Error raised by this.
    type Err;

    /// Return an iterator over this data source _Features_.
    fn iter(&self) -> Result<impl Iterator<Item = Result<Self::Item, Self::Err>>, Self::Err>;
}

/// Capability of a [`DataSource`] to asynchronously stream _Features_ or
/// [Resources][Resource].
#[async_trait]
pub trait StreamableDS {
    /// Type representing a _Feature_ which must be convertible to a [`Resource`].
    type Item: TryInto<Resource, Error = Self::Err>;
    /// Error raised by this.
    type Err;

    /// Return an unfiltered stream of all data source _Features_.
    async fn fetch(&self) -> Result<BoxStream<'_, Result<Self::Item, Self::Err>>, Self::Err>;

    /// Return an unfiltered stream of all data source _Resources_.
    async fn stream(&self) -> Result<BoxStream<'_, Result<Resource, Self::Err>>, Self::Err>;

    /// Return a filtered stream of _Features_ satisfying a CQL2 filter [Expression].
    async fn fetch_where(
        &self,
        exp: &Expression,
    ) -> Result<BoxStream<'_, Result<Self::Item, Self::Err>>, Self::Err>;

    /// Return a filtered stream of _Resources_ satisfying a CQL2 filter [Expression].
    async fn stream_where(
        &self,
        exp: &Expression,
    ) -> Result<BoxStream<'_, Result<Resource, Self::Err>>, Self::Err>;
}
