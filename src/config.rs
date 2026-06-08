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
    // #[allow(dead_code)]
    default_srid: SRID,
    default_crs: String,
    default_precision: usize,

    // PostgreSQL parameters...
    #[cfg(feature = "pg_ds")]
    pg_url: String,
    #[cfg(feature = "pg_ds")]
    pg_max_connections: u32,
    #[cfg(feature = "pg_ds")]
    pg_min_connections: u32,
    #[cfg(feature = "pg_ds")]
    pg_acquire_timeout: std::time::Duration,
    #[cfg(feature = "pg_ds")]
    pg_idle_timeout: std::time::Duration,
    #[cfg(feature = "pg_ds")]
    pg_max_lifetime: std::time::Duration,
    #[cfg(feature = "pg_ds")]
    pg_appname: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
/// This library configuration Singleton.
pub(crate) fn config() -> &'static Config {
    CONFIG.get_or_init(Config::default)
}

impl Default for Config {
    fn default() -> Self {
        // ----- Common paramters...
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

            #[cfg(feature = "pg_ds")]
            pg_url: var("PG_URL").expect("Missing PG_URL"),
            #[cfg(feature = "pg_ds")]
            pg_max_connections: var("PG_MAX_CONNECTIONS")
                .unwrap_or("8".to_string())
                .parse()
                .expect("Failed parsing DB_MAX_CONNECTIONS"),
            #[cfg(feature = "pg_ds")]
            pg_min_connections: var("PG_MIN_CONNECTIONS")
                .unwrap_or("4".to_string())
                .parse()
                .expect("Failed parsing PG_MIN_CONNECTIONS"),
            #[cfg(feature = "pg_ds")]
            pg_acquire_timeout: std::time::Duration::from_secs(
                var("PG_ACQUIRE_TIMEOUT_SECS")
                    .unwrap_or("8".to_string())
                    .parse()
                    .expect("Failed parsing PG_ACQUIRE_TIMEOUT_SECS"),
            ),
            #[cfg(feature = "pg_ds")]
            pg_idle_timeout: std::time::Duration::from_secs(
                var("PG_IDLE_TIMEOUT_SECS")
                    .unwrap_or("8".to_string())
                    .parse()
                    .expect("Failed parsing PG_IDLE_TIMEOUT_SECS"),
            ),
            #[cfg(feature = "pg_ds")]
            pg_max_lifetime: std::time::Duration::from_secs(
                var("PG_MAX_LIFETIME_SECS")
                    .unwrap_or("8".to_string())
                    .parse()
                    .expect("Failed parsing PG_MAX_LIFETIME_SECS"),
            ),
            #[cfg(feature = "pg_ds")]
            pg_appname: "CQL2".into(),
        }
    }
}

impl Config {
    /// Return the configured default SRID (EPSG) code to use when validating
    /// geometry coordinates.
    // #[allow(dead_code)]
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

    #[cfg(feature = "pg_ds")]
    pub(crate) fn pg_url(&self) -> &str {
        &self.pg_url
    }

    #[cfg(feature = "pg_ds")]
    pub(crate) fn pg_max_connections(&self) -> u32 {
        self.pg_max_connections
    }

    #[cfg(feature = "pg_ds")]
    pub(crate) fn pg_min_connections(&self) -> u32 {
        self.pg_min_connections
    }

    #[cfg(feature = "pg_ds")]
    pub(crate) fn pg_acquire_timeout(&self) -> std::time::Duration {
        self.pg_acquire_timeout
    }

    #[cfg(feature = "pg_ds")]
    pub(crate) fn pg_idle_timeout(&self) -> std::time::Duration {
        self.pg_idle_timeout
    }

    #[cfg(feature = "pg_ds")]
    pub(crate) fn pg_max_lifetime(&self) -> std::time::Duration {
        self.pg_max_lifetime
    }

    #[cfg(feature = "pg_ds")]
    pub(crate) fn pg_appname(&self) -> &str {
        &self.pg_appname
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
        assert_eq!(actual.into_inner().to_string(), expected);

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
