// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]
#![allow(dead_code)]

//! Coordinate Reference System (CRS) types and traits in this library.
//!

use crate::{MyError, config::config};
use core::fmt;
use proj::Proj;
use tracing::info;

#[derive(Debug)]
struct EoV {
    /// horizontal or longitudinal extent bounds.
    x_range: (f64, f64),
    /// vertical or latitudinal extent bounds.
    y_range: (f64, f64),
}

/// Representation of a Coordinate Reference System
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct CRS {
    definition: String,
    inner: Proj,
    extent_of_validity: EoV,
}

// FIXME (rsn) 20250807 - add LRU cache to store instances + reduce duplication.

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
            x_range: (eov.west, eov.east),
            y_range: (eov.south, eov.north),
        };
        let crs = CRS {
            definition,
            inner,
            extent_of_validity,
        };

        // FIXME - cache it...

        Ok(crs)
    }

    /// Check if the given point coordinates are w/in the area-of-validity of this.
    pub fn check_point(&self, coord: &[f64]) -> Result<(), MyError> {
        // FIXME (rsn) 2250807 - so far we only handle 2D coordinates...
        let (x, y) = (&coord[0], &coord[1]);
        let (x_min, x_max) = self.extent_of_validity.x_range;
        let x_ok = *x >= x_min && *x <= x_max;
        if !x_ok {
            return Err(MyError::Runtime(
                "Point x (longitude) coordinate is out-of-bounds".into(),
            ));
        }
        let (y_min, y_max) = self.extent_of_validity.y_range;
        let y_ok = *y >= y_min && *y <= y_max;
        if !y_ok {
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
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_name() {
        let epsg_4326 = Proj::new("EPSG:4326").unwrap();
        // tracing::debug!("EPSG:4326 = {epsg_4326:?}");

        let (aou, _) = epsg_4326.area_of_use().unwrap();
        // tracing::debug!("EPSG:4326 area of use = {aou:?}");

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
