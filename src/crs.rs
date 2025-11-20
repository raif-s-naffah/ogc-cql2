// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]
#![allow(dead_code)]

//! Coordinate Reference System (CRS) types and traits in this library.
//!

use crate::{MyError, config::config};
use core::fmt;
use proj::Proj;
use std::{num::NonZero, ops::RangeInclusive};
use tracing::info;

#[derive(Debug)]
struct EoV {
    /// horizontal/longitudinal/easting extent bounds.
    x_range: RangeInclusive<f64>,
    /// vertical/latitudinal/northing extent bounds.
    y_range: RangeInclusive<f64>,
}

/// Representation of a Coordinate Reference System
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct CRS {
    definition: String,
    inner: Proj,
    extent_of_validity: EoV,
}

impl fmt::Display for CRS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.definition)
    }
}

impl Default for CRS {
    fn default() -> Self {
        let default_crs = config().default_crs();
        info!("Will try using '{default_crs}' as the default CRS...");
        Self::new(default_crs).expect("Failed instantiating default CRS")
    }
}

impl CRS {
    /// Try constructing a new instance and ensure that CRS has a non-trivial
    /// extent of validity which will be later used to validate geometry
    /// coordinates.
    pub fn new(code: &str) -> Result<Self, MyError> {
        let inner = Proj::new(code)?;
        let definition = code.into();
        let (mb_eov, _mb_def) = inner.area_of_use()?;
        // tracing::trace!("area-of-use for '{definition}' = {mb_eov:?}, '{_mb_def:?}'");
        // for now reject input w/ no known validity-extent bounds...
        let eov = mb_eov.expect("CRSes w/ no known Area-of-Use are not supported. Abort");
        let extent_of_validity = EoV {
            x_range: RangeInclusive::new(eov.west, eov.east),
            y_range: RangeInclusive::new(eov.south, eov.north),
        };
        let crs = CRS {
            definition,
            inner,
            extent_of_validity,
        };

        Ok(crs)
    }

    /// Construct a new instance from the given code assuming EPSG Authority
    /// if it's a valid one (known by Proj).
    pub fn from_epsg(code: NonZero<usize>) -> Result<Self, MyError> {
        Self::new(&format!("EPSG:{code}"))
    }

    /// Check if the given point coordinates are w/in the area-of-validity of this.
    pub fn check_point(&self, coord: &[f64]) -> Result<(), MyError> {
        // FIXME (rsn) 2250807 - so far we only handle 2D coordinates...
        let (x, y) = (&coord[0], &coord[1]);
        if !self.extent_of_validity.x_range.contains(x) {
            return Err(MyError::Runtime(
                "Point x (longitude) coordinate is out-of-bounds".into(),
            ));
        }
        if !self.extent_of_validity.y_range.contains(y) {
            return Err(MyError::Runtime(
                "Point y (latitude) coordinate is out-of-bounds".into(),
            ));
        }
        Ok(())
    }

    /// Check if the given line coordinates are w/in the area-of-validity of this.
    pub fn check_line(&self, coord: &[Vec<f64>]) -> Result<(), MyError> {
        if coord.iter().all(|xy| self.check_point(xy).is_ok()) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "One or more line coordinates are out-of-bounds".into(),
            ))
        }
    }

    /// Check if the given polygon coordinates are w/in the area-of-validity of this.
    pub fn check_polygon(&self, rings: &[Vec<Vec<f64>>]) -> Result<(), MyError> {
        if rings.iter().all(|r| self.check_line(r).is_ok()) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "One or more polygon coordinates are out-of-bounds".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use proj::Proj;

    #[test]
    fn test_name() {
        let epsg_4326 = Proj::new("EPSG:4326").unwrap();
        let (aou, _) = epsg_4326.area_of_use().unwrap();

        if let Some(a) = aou {
            assert_eq!(a.west, -180.0);
            assert_eq!(a.east, 180.0);
            assert_eq!(a.south, -90.0);
            assert_eq!(a.north, 90.0);
        } else {
            panic!("Failed getting Area of Use for EPSG:4326")
        }
    }
}
