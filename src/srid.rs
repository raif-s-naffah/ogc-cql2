// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Spatial Reference System identifier. Assumes implicitely the code is
//! under the "EPSG" Authority.
//!

use crate::{CRS, MyError};
use core::fmt;
use std::num::NonZero;

/// The constant representing the ubiquitous `EPSG:4326` or `WGS'84` SRID.
pub const EPSG_4326: SRID = SRID(4326);

/// Representation of a Spatial Reference IDentifier. For now the Authority
/// is implied to be EPSG.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::upper_case_acronyms)]
pub struct SRID(i32);

// invoked when parsing WKB blobs...
impl TryFrom<i32> for SRID {
    type Error = MyError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        // -1 and 0 are valid values for undefined cartesian and geographic SRS
        // as per GeoPackage specs...
        match value {
            x if x == -1 || x == 0 => Ok(Self(x)),
            x => {
                // ensure Proj knows it...
                let code = usize::try_from(value)?;
                let _ = CRS::from_epsg(
                    NonZero::new(code)
                        .ok_or(MyError::Runtime("Expected a non-zero EPSG code".into()))?,
                )?;
                Ok(Self(x))
            }
        }
    }
}

// invoked when parsing GEOS geometry instances...
impl TryFrom<usize> for SRID {
    type Error = MyError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self(0)),
            x => {
                // ensure Proj knows it...
                let _ = CRS::from_epsg(
                    NonZero::new(value)
                        .ok_or(MyError::Runtime("Expected a non-zero EPSG code".into()))?,
                )?;
                Ok(Self(x.try_into()?))
            }
        }
    }
}

impl fmt::Display for SRID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            -1 => write!(f, "Undefined (Cartesian)"),
            0 => write!(f, "Undefined (geographic)"),
            x => write!(f, "EPSG:{x}"),
        }
    }
}

impl SRID {
    pub(crate) fn as_usize(&self) -> Result<usize, MyError> {
        let it = usize::try_from(self.0)?;
        Ok(it)
    }
}
