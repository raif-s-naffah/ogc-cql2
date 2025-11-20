// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Representation of WKB blob geometry header.
//!

use crate::{
    MyError,
    geom::{XY1V, XY2V, XY3V},
    srid::SRID,
    wkb::{ByteOrder, Envelope, double, signed},
};
use tracing::error;

/// Every GeoPackage binary value ([2.1.3.1.1. BLOB Format][1]) starts w/ a
/// header like this w/ the following consitituents:
/// * 2-byte magic = 0x4750 which represent the ASCII 'GP';
/// * 1-byte version, an 8-bit unsigned integer, `0` = version 1;
/// * 1-byte flags w/ the following layout (from left bit #7 to right bit #0)
///   bits #7 and #6 are reserved and are ignored.
///   bit #5: X = `0` StandardGeoPackageBinary, or `1` ExtendedGeoPackageBinary.
///   bit #4: Y = `0` for non-empty geometry, or `1` for an empty one.
///   bits #3 #2 #1: envelope content indicator. Tells how long is the
///   envelope array.
///   bit #0: byte-order of the next 2 fields. `0` for BE, and `1` for LE.
/// * int32 srs_id.  At least 3 values should be supported: `4326` for WGS'84,
///   and `-1` and `0` for "undefined" cartesian and geographic SRSes.
/// * double[] envelope.
///
/// Given that only Version 1 is defined, and that we're only interested in the
/// GeoPackage 2D geometry types, we only keep track in this structure of...
/// * the SRID as an unsigned 32-bit integer, and
/// * the envelope as an array of 64-bit floats.
///
/// [1]: https://www.geopackage.org/spec140/index.html
#[derive(Debug)]
pub(crate) struct GeoPackageBinaryHeader {
    srid: SRID,
    envelope: Envelope,
}

impl GeoPackageBinaryHeader {
    /// Return the SRID found in the header
    pub(crate) fn srid(&self) -> &SRID {
        &self.srid
    }

    pub(crate) fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    /// Return the number of bytes this header instance occupies.
    pub(crate) fn len(&self) -> usize {
        2 + 1 + 1 + 4 + 8 * self.envelope.len()
    }

    /// If this header's envelope is not trivial then ensure that coordinates
    /// are within its boundary. Raise [MyError] if they're not.
    pub(crate) fn check_point(&self, xy: &XY1V) -> Result<(), MyError> {
        self.envelope.check_point(xy)
    }

    pub(crate) fn check_line(&self, xy: &[XY1V]) -> Result<(), MyError> {
        self.envelope.check_line(xy)
    }

    pub(crate) fn check_polygon(&self, xy: &[XY2V]) -> Result<(), MyError> {
        self.envelope.check_polygon(xy)
    }

    pub(crate) fn check_points(&self, xy: &[XY1V]) -> Result<(), MyError> {
        self.envelope.check_points(xy)
    }

    pub(crate) fn check_lines(&self, xy: &[XY2V]) -> Result<(), MyError> {
        self.envelope.check_lines(xy)
    }

    pub(crate) fn check_polygons(&self, xy: &[XY3V]) -> Result<(), MyError> {
        self.envelope.check_polygons(xy)
    }
}

impl TryFrom<&[u8]> for GeoPackageBinaryHeader {
    type Error = MyError;

    fn try_from(ba: &[u8]) -> Result<Self, Self::Error> {
        if ba[..2] != *b"GP" {
            let msg = "Input does NOT start w/ expected ('GP') magic";
            error!("{msg}");
            return Err(MyError::Runtime(msg.into()));
        }

        let v = ba[2];
        if v != 0 {
            let msg = format!("Unexpected ({v}) format version");
            error!("{msg}");
            return Err(MyError::Runtime(msg.into()));
        }

        let flags = ba[3] & 0x3F;
        // bit #5 is X: 0 means 'standard' while 1 means 'extended'...
        if flags & 0x20 != 0 {
            let msg = "X flag set => NOT StandardGeoPackageBinary";
            error!("{msg}");
            return Err(MyError::Runtime(msg.into()));
        }

        // bit #4 is Y: 0 means non-empty geometry while 1 means empty one...
        if (flags & 0x10) >> 4 == 1 {
            let msg = "Y flag set => empty geometry";
            error!("{msg}");
            return Err(MyError::Runtime(msg.into()));
        }

        let bo = &ByteOrder::from(flags & 0x01);
        let eci = (flags >> 1) & 0x07;

        let srs_id = signed(bo, ba, 4)?;
        let srid = SRID::try_from(srs_id)?;

        let env_len = match eci {
            0 => 0,
            1 => 4,
            2 | 3 => 6,
            4 => 8,
            x => {
                let msg = format!("Invalid ({x}) E flag");
                error!("{msg}");
                return Err(MyError::Runtime(msg.into()));
            }
        };
        let mut bounds: Vec<f64> = Vec::with_capacity(env_len);
        for i in 0..env_len {
            let x = double(bo, ba, 8 + i * 8)?;
            bounds.push(x);
        }
        let envelope = Envelope::new(bounds);

        Ok(Self { srid, envelope })
    }
}
