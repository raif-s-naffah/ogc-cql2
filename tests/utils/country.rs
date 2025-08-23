// SPDX-License-Identifier: Apache-2.0

//! Code to handle representing records of `ne_110m_admin_0_countries`
//! conformance test data and logic to convert them to structures that
//! can be used by the library.
//!

use core::fmt;
use ogc_cql2::{Q, Resource};
use csv::StringRecord;
use serde::Deserialize;
use std::{collections::HashMap, error::Error};

#[allow(dead_code)]
#[rustfmt::skip]
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ZCountry {
    /*  0 */ fid: i32,
    // #[serde(skip)] geom: String,
    /*  1 */ geom: String,
    #[serde(skip)] type_: String,
    #[serde(skip)] adm0_a3: String,
    /*  4 */ #[serde(rename(deserialize = "NAME"))] name: String,
    #[serde(skip)] name_long: String,
    #[serde(skip)] abbrev: String,
    #[serde(skip)] postal: String,
    #[serde(skip)] formal_en: String,
    #[serde(skip)] name_sort: String,
    /* 10 */ #[serde(rename(deserialize = "POP_EST"))] pop_est: f64,
    #[serde(skip)] enonomy: String,
    #[serde(skip)] income_grp: String,
    #[serde(skip)] continent: String,
    #[serde(skip)] region_un: String,
    #[serde(skip)] subregion: String,
    #[serde(skip)] region_wb: String,
    #[serde(skip)] wikidataid: String,
    #[serde(skip)] name_de: String,
    #[serde(skip)] name_en: String,
    #[serde(skip)] name_el: String,
}

impl fmt::Display for ZCountry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.fid, self.name, self.pop_est)
    }
}

impl TryFrom<ZCountry> for Resource {
    type Error = Box<dyn Error>;

    fn try_from(value: ZCountry) -> Result<Self, Self::Error> {
        Ok(HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkt(&value.geom)?),
            ("NAME".into(), Q::new_plain_str(&value.name)),
            ("POP_EST".into(), Q::from(value.pop_est)),
        ]))
    }
}

impl ZCountry {
    // Convenience constructor for a new instance from a CSV data set record.
    pub(crate) fn new_from_record(r: StringRecord) -> Result<Resource, Box<dyn Error>> {
        let row = ZCountry::try_from(r)?;
        Ok(Resource::try_from(row)?)
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<StringRecord> for ZCountry {
    type Error = Box<dyn Error>;

    fn try_from(value: StringRecord) -> Result<Self, Self::Error> {
        let fid = value.get(0).expect("Missing 'fid'").parse::<i32>()?;
        let geom = value.get(1).expect("Missing 'geom'").to_owned();
        let name = value.get(4).expect("Missing 'NAME'").to_owned();
        let pop_est = value.get(10).expect("Missing 'POP_EST'").parse::<f64>()?;
        Ok(Self {
            fid,
            geom,
            name,
            pop_est,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{COUNTRIES, DATASETS};
    use csv::Reader;
    use geos::{Geom, Geometry};
    use std::fs::File;

    #[test]
    fn test() -> Result<(), Box<dyn Error>> {
        let file = File::open(DATASETS[COUNTRIES])?;
        let mut rdr = Reader::from_reader(file);
        let mut count = 0;
        for record in rdr.deserialize() {
            let country: ZCountry = record?;
            count += 1;
            // all geometries are valid mulyi-polygons...
            let g = Geometry::new_from_wkt(&country.geom)?;
            assert_eq!(g.get_type()?, "MultiPolygon");
        }
        // set contains 177 rows...
        assert_eq!(count, 177);
        Ok(())
    }
}
