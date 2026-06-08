// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Group imports of many common traits and types by adding a glob import for
//! use by clients of this library.
//!

pub use super::bound::*;
pub use super::context::*;
pub use super::crs::*;
pub use super::ds::*;
pub use super::error::*;
pub use super::evaluator::*;
pub use super::function::*;
pub use super::geom::*;
pub use super::qstring::*;
pub use super::queryable::*;
pub use super::srid::*;

pub use super::Expression;
pub use super::Outcome;

#[cfg(feature = "csv_ds")]
pub use super::{CSVDataSource, gen_csv_ds};
#[cfg(feature = "gpkg_ds")]
pub use super::{GPkgDataSource, gen_gpkg_ds};
#[cfg(feature = "pg_ds")]
pub use super::{PGDataSource, gen_pg_ds};
