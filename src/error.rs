// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Errors raised from this library.
//!

use peg::{error::ParseError, str::LineCol};
use std::borrow::Cow;
use thiserror::Error;

/// Variants of error raised from this library.
#[derive(Debug, Error)]
pub enum MyError {
    /// Data serialization/deserialization, parsing + validation errors.
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// Date, time + timestamp (`jiff`) parsing error.
    #[error("Date-Time error: {0}")]
    Time(#[from] jiff::Error),

    /// Text-encoding (`peg`) related error.
    #[error("PEG error: {0:?}")]
    Text(ParseError<LineCol>),

    /// JSON-encoding (`serde`) related error
    #[error("Json [Try]From error: {0}")]
    Json(#[from] serde_json::Error),

    /// Geometry (`geos`) related error.
    #[error("Geos error: {0}")]
    Geos(#[from] geos::Error),

    /// Coordinate conversion results in loss of precision.
    #[error("Converting {0} to `f64` will result in loss of precision")]
    PrecisionLoss(Cow<'static, str>),

    /// CRS construction error.
    #[error("CRS creation error: {0}")]
    CRS(#[from] proj::ProjCreateError),

    /// Coordinate transformation (`proj`) related error.
    #[error("Proj error: {0}")]
    Proj(#[from] proj::ProjError),

    /// Runtime error.
    #[error("Runtime error: {0}")]
    Runtime(Cow<'static, str>),
}
