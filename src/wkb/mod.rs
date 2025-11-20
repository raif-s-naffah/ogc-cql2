// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Parse WKB encoded GeoPackage BLOB values.
//!
//! See [BLOB Format][1] section of the _OGC® GeoPackage Encoding Standard_ in
//! general, and the [OpenGIS® Implementation Standard for Geographic information
//! - Simple feature access - Part 1: Common architecture][2] for references.
//!
//! Especially note the following stated in the _GeoPackage_ standard:
//!
//! > WKB geometry types are are restricted to 0, 1 and 2-dimensional geometric
//! > objects that exist in 2, 3 or 4-dimensional coordinate space; they are
//! > not geographic or geodesic geometry types.
//!
//! [1]: https://www.geopackage.org/spec140/index.html#gpb_format
//! [2]: http://portal.opengeospatial.org/files/?artifact_id=25355

mod blob;
mod byte_order;
mod envelope;
mod header;

pub(crate) use blob::StandardGeoPackageBinary;
pub(crate) use byte_order::ByteOrder;
pub(crate) use envelope::Envelope;
pub(crate) use header::GeoPackageBinaryHeader;

use crate::MyError;

// Parse 4 bytes, starting at index `start`, as LE or BE and return an `i32`.
fn signed(bo: &ByteOrder, buffer: &[u8], start: usize) -> Result<i32, MyError> {
    // prefer 4 to `size_of::<i32>()` to ensure only 4 bytes are processed.
    let end = start + 4;
    let b4 = &buffer[start..end];
    let it = if bo.is_le() {
        i32::from_le_bytes(b4.try_into()?)
    } else {
        i32::from_be_bytes(b4.try_into()?)
    };
    Ok(it)
}

// Parse 4 bytes, starting at index `start`, as LE or BE and return a `u32`.
fn unsigned(bo: &ByteOrder, buffer: &[u8], start: usize) -> Result<u32, MyError> {
    // prefer 4 to `size_of::<u32>()` to ensure only 4 bytes are processed.
    let end = start + 4;
    let b4 = &buffer[start..end];
    let it = if bo.is_le() {
        u32::from_le_bytes(b4.try_into()?)
    } else {
        u32::from_be_bytes(b4.try_into()?)
    };
    Ok(it)
}

// Parse 8 bytes, starting at index `start`, as LE or BE and return an `f64`.
fn double(bo: &ByteOrder, buffer: &[u8], start: usize) -> Result<f64, MyError> {
    // prefer 8 to `size_of::<f64>()` to ensure only 8 bytes are processed.
    let end = start + 8;
    let b8 = &buffer[start..end];
    let it = if bo.is_le() {
        f64::from_le_bytes(b8.try_into()?)
    } else {
        f64::from_be_bytes(b8.try_into()?)
    };
    Ok(it)
}
