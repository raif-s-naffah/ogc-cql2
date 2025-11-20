// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Configuration parameters affecting the behaviour of this library.
//!
//! 1. _Default CRS_: help determine if geometry coordinates w/o explicit CRS
//!    are valid or not. By valid we mean they fall w/in the extent of validity
//!    of said CRS.
//!
//! 2. _Default precision_: determine the number of decimal digits after the
//!    decimal point to output when rendering geometry coordinates in a WKT.
//!    also used when generating SQL for certain ST functions.
//!

use crate::{crs::CRS, srid::SRID};
use dotenvy::var;
use std::sync::OnceLock;

const DEFAULT_SRID: usize = 4326;
const DEFAULT_PRECISION: &str = "7";
const MAX_PRECISION: usize = 32;

#[derive(Debug)]
pub(crate) struct Config {
    #[allow(dead_code)]
    default_srid: SRID,
    default_crs: String,
    default_precision: usize,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
/// This library configuration Singleton.
pub(crate) fn config() -> &'static Config {
    CONFIG.get_or_init(Config::default)
}

impl Default for Config {
    fn default() -> Self {
        let srid = var("DEFAULT_SRID")
            .unwrap_or(DEFAULT_SRID.to_string())
            .parse::<usize>()
            .expect("Invalid DEFAULT_SRID");
        let default_srid = SRID::try_from(srid).expect("Invalid EPSG SRS identifier");
        let default_crs: String = format!("EPSG:{srid}");
        // ensure it's valid...
        let _ = CRS::new(&default_crs).expect("Invalid default CRS");

        let value: usize = var("DEFAULT_PRECISION")
            .unwrap_or(DEFAULT_PRECISION.to_owned())
            .parse()
            .expect("Failed parsing DEFAULT_PRECISION");
        // ensure it's valid...
        if value > MAX_PRECISION {
            panic!("Invalid ({value}) default precision. MUST be <= {MAX_PRECISION}");
        }

        Self {
            default_srid,
            default_crs,
            default_precision: value,
        }
    }
}

impl Config {
    /// Return the configured default SRID (EPSG) code to use when validating
    /// geometry coordinates.
    #[allow(dead_code)]
    pub(crate) fn default_srid(&self) -> &SRID {
        &self.default_srid
    }

    /// Return the configured default CRS code to use when validating
    /// geometry coordinates.
    pub(crate) fn default_crs(&self) -> &str {
        &self.default_crs
    }

    /// Return the configured default to use when generating geometry WKT w/o a
    /// specific precision parameter.
    pub(crate) fn default_precision(&self) -> usize {
        self.default_precision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_default_srid() -> Result<(), Box<dyn Error>> {
        // should be the same as that of the corresponding env. variable...
        let actual = config().default_srid();
        let expected = var("DEFAULT_SRID")?;
        assert_eq!(actual.as_usize()?.to_string(), expected);

        Ok(())
    }

    #[test]
    fn test_default_precision() -> Result<(), Box<dyn Error>> {
        // should be the same as that of the corresponding env. variable...
        let actual = config().default_precision();
        let expected: usize = var("DEFAULT_PRECISION")?.parse()?;
        assert_eq!(actual, expected);

        Ok(())
    }
}
