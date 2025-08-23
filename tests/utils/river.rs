// SPDX-License-Identifier: Apache-2.0

//! Code to handle representing records of `ne_110m_rivers_lake_centerlines`
//! conformance test data and logic to convert them to structures that
//! can be used by the library.
//!

use core::fmt;
use ogc_cql2::{Q, Resource};
use csv::StringRecord;
use serde::Deserialize;
use std::{collections::HashMap, error::Error, marker::PhantomData};

#[rustfmt::skip]
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ZRiver {
    /* 0 */ fid: i32,
    /* 1 */ geom: String,
    /* 2 */ name: String,
    #[serde(skip)] ignored: PhantomData<String>
}

impl fmt::Display for ZRiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.fid, self.name)
    }
}

impl TryFrom<ZRiver> for Resource {
    type Error = Box<dyn Error>;

    fn try_from(value: ZRiver) -> Result<Self, Self::Error> {
        Ok(HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkt(&value.geom)?),
            ("name".into(), Q::new_plain_str(&value.name)),
        ]))
    }
}

impl ZRiver {
    // Convenience constructor for a new instance from a CSV data set record.
    pub(crate) fn new_from_record(r: StringRecord) -> Result<Resource, Box<dyn Error>> {
        let row = ZRiver::try_from(r)?;
        Ok(Resource::try_from(row)?)
    }

    #[allow(dead_code)]
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<StringRecord> for ZRiver {
    type Error = Box<dyn Error>;

    fn try_from(value: StringRecord) -> Result<Self, Self::Error> {
        let fid = value.get(0).expect("Missing 'fid'").parse::<i32>()?;
        let geom = value.get(1).expect("Missing 'geom'").to_owned();
        let name = value.get(4).expect("Missing 'name'").to_owned();
        Ok(Self {
            fid,
            geom,
            name,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{DATASETS, RIVERS};
    use csv::Reader;
    use geos::{Geom, Geometry};
    use std::fs::File;

    #[test]
    fn test() -> Result<(), Box<dyn Error>> {
        let file = File::open(DATASETS[RIVERS])?;
        let mut rdr = Reader::from_reader(file);
        let mut count = 0;
        for record in rdr.deserialize() {
            let river: ZRiver = record?;
            count += 1;
            // all geometries are valid lines...
            let g = Geometry::new_from_wkt(&river.geom)?;
            assert_eq!(g.get_type()?, "LineString");
        }
        // set contains 13 rows...
        assert_eq!(count, 13);
        Ok(())
    }
}
