// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Errors raised from this library.
//!

use peg::{error::ParseError, str::LineCol};
use std::{
    array::TryFromSliceError,
    borrow::Cow,
    num::{ParseIntError, TryFromIntError},
};
use thiserror::Error;

/// Variants of error raised from this library.
#[derive(Debug, Error)]
pub enum MyError {
    /// Input/Output related error.
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

    /// GEOS related error.
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

    /// CSV error.
    #[error("CSV error: {0}")]
    CSV(#[from] csv::Error),

    /// SqlX error.
    #[error("SQLx error: {0}")]
    SQL(#[from] sqlx::Error),

    /// Byte-array to integer conversion error.
    #[error("Conversion (bytes -> int) error: {0}")]
    Conv(#[from] TryFromIntError),

    /// String to integer conversion error.
    #[error("Conversion (str -> int) error: {0}")]
    Conv2(#[from] ParseIntError),

    /// Byte-array to float conversion error.
    #[error("Conversion (slice) error: {0}")]
    Slice(#[from] TryFromSliceError),
}
