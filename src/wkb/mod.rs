// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Parse encoded GeoPackage WKB, and PostGIS EWKB binary values.
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
mod ewkb;
mod header;

pub(crate) use blob::GeoPackageBinary;
pub(crate) use byte_order::ByteOrder;
pub(crate) use envelope::Envelope;
pub(crate) use ewkb::PostGisBinary;
pub(crate) use header::GeoPackageBinaryHeader;

use crate::{MyError, XY1V, XY2V, XY3V, XY4V};

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

// a macro to generate blob parsing for the 3 basic geometry types.
macro_rules! gen_wkb_functions {
    ($suffix:expr, $wkb_type:expr) => {
        paste::paste! {
            pub(crate) fn [<wkb_ $suffix>](ba: &[u8], start: usize) -> Result<([<XY $wkb_type V>], usize), MyError> {
                let mut pos = start;
                let byte_order = &ByteOrder::from(ba[pos]);
                pos += 1;
                let geom_type = unsigned(byte_order, ba, pos)?;
                pos += 4;
                let type_ = $wkb_type;
                assert_eq!(
                    geom_type, $wkb_type,
                    "Expected wkbType to be {type_} but found {geom_type}"
                );
                let (xy, offset) = [<$suffix>](byte_order, ba, pos)?;
                Ok((xy, 5 + offset))
            }
        }
    };
}

gen_wkb_functions!("point", 1);
gen_wkb_functions!("line", 2);
gen_wkb_functions!("polygon", 3);

// Parse and return a pair of x y (double) coordinates.
fn point(bo: &ByteOrder, ba: &[u8], start: usize) -> Result<(Vec<f64>, usize), MyError> {
    let x = double(bo, ba, start)?;
    let mut span = 8;
    let y = double(bo, ba, start + span)?;
    span += 8;
    let xy = vec![x, y];
    Ok((xy, span))
}

// Parse and return an unsigned `numPoints` followed by as many pairs of x, y
// coordinates.
fn line(bo: &ByteOrder, ba: &[u8], start: usize) -> Result<(XY2V, usize), MyError> {
    let num_points = unsigned(bo, ba, start)?;
    let mut span = 4;
    let mut xy: XY2V = Vec::with_capacity(usize::try_from(num_points)?);
    for _ in 0..num_points {
        let (coord, offset) = point(bo, ba, start + span)?;
        xy.push(coord);
        span += offset;
    }
    Ok((xy, span))
}

// An unsigned `numRings` followed by as many linear-rings. For our purposes a
// linear ring is structurally the same as a `line`.
fn polygon(bo: &ByteOrder, ba: &[u8], start: usize) -> Result<(XY3V, usize), MyError> {
    let num_rings = unsigned(bo, ba, start)?;
    let mut span = 4;
    let mut xy: XY3V = Vec::with_capacity(usize::try_from(num_rings)?);
    for _ in 0..num_rings {
        let (coord, offset) = line(bo, ba, start + span)?;
        xy.push(coord);
        span += offset;
    }
    Ok((xy, span))
}

// An unsigned `numPoints` followed by as many `wkb_point`s.
fn points(bo: &ByteOrder, ba: &[u8], start: usize) -> Result<(Vec<Vec<f64>>, usize), MyError> {
    let num_points = unsigned(bo, ba, start)?;
    let mut span = 4;
    let mut xy: Vec<Vec<f64>> = Vec::with_capacity(usize::try_from(num_points)?);
    for _ in 0..num_points {
        let (coord, offset) = wkb_point(ba, start + span)?;
        xy.push(coord);
        span += offset;
    }
    Ok((xy, span))
}

fn lines(bo: &ByteOrder, ba: &[u8], start: usize) -> Result<(XY3V, usize), MyError> {
    let num_lines = unsigned(bo, ba, start)?;
    let mut span = 4;
    let mut xy: XY3V = Vec::with_capacity(usize::try_from(num_lines)?);
    for _ in 0..num_lines {
        let (coord, offset) = wkb_line(ba, start + span)?;
        xy.push(coord);
        span += offset;
    }
    Ok((xy, span))
}

fn polygons(bo: &ByteOrder, ba: &[u8], start: usize) -> Result<(XY4V, usize), MyError> {
    let num_polygons = unsigned(bo, ba, start)?;
    let mut span = 4;
    let mut xy: XY4V = Vec::with_capacity(usize::try_from(num_polygons)?);
    for _ in 0..num_polygons {
        let (coord, offset) = wkb_polygon(ba, start + span)?;
        xy.push(coord);
        span += offset;
    }
    Ok((xy, span))
}
