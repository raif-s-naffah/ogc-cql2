// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Representation of WKB blob geometry envelope (part of the header).
//!

use crate::{
    MyError,
    geom::{XY1V, XY2V, XY3V},
};
use core::f64;
use std::ops::RangeInclusive;

// NOTE (rsn) 20251022 - we do not distinguish between XYZ and XYM for now
// and we only care about 2D vector geometries.
#[derive(Debug)]
pub(crate) struct Envelope {
    len: usize,
    x_range: RangeInclusive<f64>,
    y_range: RangeInclusive<f64>,
}

impl Envelope {
    pub(crate) fn new(bounds: Vec<f64>) -> Self {
        let len = bounds.len();
        let (x_range, y_range) = if bounds.is_empty() {
            (
                RangeInclusive::new(f64::MIN, f64::MAX),
                RangeInclusive::new(f64::MIN, f64::MAX),
            )
        } else {
            (
                // envelope is [minx, maxx, miny, maxy], 32 bytes
                RangeInclusive::new(bounds[0], bounds[1]),
                RangeInclusive::new(bounds[2], bounds[3]),
            )
        };
        Self {
            len,
            x_range,
            y_range,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn check_point(&self, xy: &XY1V) -> Result<(), MyError> {
        if !self.x_range.contains(&xy[0]) {
            return Err(MyError::Runtime(
                "Point x (longitude/easting) coordinate is out-of-bounds".into(),
            ));
        }

        if !self.y_range.contains(&xy[1]) {
            return Err(MyError::Runtime(
                "Point y (latitude/northing) coordinate is out-of-bounds".into(),
            ));
        }

        Ok(())
    }

    pub(crate) fn check_line(&self, coord: &[XY1V]) -> Result<(), MyError> {
        if coord.iter().all(|xy| self.check_point(xy).is_ok()) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "One or more line coordinates are out-of-bounds".into(),
            ))
        }
    }

    pub(crate) fn check_polygon(&self, rings: &[XY2V]) -> Result<(), MyError> {
        if rings.iter().all(|r| self.check_line(r).is_ok()) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "One or more polygon coordinates are out-of-bounds".into(),
            ))
        }
    }

    pub(crate) fn check_points(&self, coord: &[XY1V]) -> Result<(), MyError> {
        if coord.iter().all(|xy| self.check_point(xy).is_ok()) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "One or more point coordinates are out-of-bounds".into(),
            ))
        }
    }

    pub(crate) fn check_lines(&self, lines: &[XY2V]) -> Result<(), MyError> {
        if lines.iter().all(|l| self.check_line(l).is_ok()) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "At least one line has invalid coordinates".into(),
            ))
        }
    }

    pub(crate) fn check_polygons(&self, polygons: &[XY3V]) -> Result<(), MyError> {
        if polygons.iter().all(|poly| self.check_polygon(poly).is_ok()) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "At least one polygon has invalid coordinates".into(),
            ))
        }
    }

    #[cfg(test)]
    pub(crate) fn min_x(&self) -> f64 {
        *self.x_range.start()
    }

    #[cfg(test)]
    pub(crate) fn max_x(&self) -> f64 {
        *self.x_range.end()
    }

    #[cfg(test)]
    pub(crate) fn min_y(&self) -> f64 {
        *self.y_range.start()
    }

    #[cfg(test)]
    pub(crate) fn max_y(&self) -> f64 {
        *self.y_range.end()
    }
}
