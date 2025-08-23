// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Configuration parameters affecting the behaviour of this library.
//!

use crate::crs::{self, CRS};
use dotenvy::var;
use std::sync::OnceLock;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Config {
    default_crs: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
/// This library configuration Singleton.
#[allow(dead_code)]
pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::default)
}

impl Default for Config {
    fn default() -> Self {
        let default_crs: String = var("DEFAULT_CRS").unwrap_or(crs::DEFAULT_CRS.to_owned());
        // ensure it's valid...
        let _ = CRS::new(&default_crs).expect("Invalid default CRS. Abort");

        Self { default_crs }
    }
}

impl Config {
    /// Return the configured default CRS to use when validating + interpreting
    /// geometry coordinates.
    pub fn default_crs(&self) -> &str {
        &self.default_crs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_default_crs() {
        let default_crs = config().default_crs();
        assert_eq!(default_crs, crs::DEFAULT_CRS);
    }
}
