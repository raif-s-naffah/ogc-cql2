// SPDX-License-Identifier: Apache-2.0

//! Code to handle representing records of `ne_110m_populated_places_simple`
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
pub(crate) struct ZPlace {
    /*  0 */ fid: i32,
    // #[serde(skip)] geom: String,
    /*  1 */ geom: String,
    #[serde(skip)] featurecla: String,
    /*  3 */ name: String,
    #[serde(skip)] namepar: String,
    #[serde(skip)] namealt: String,
    /*  6 */ nameascii: String,
    #[serde(skip)] capin: String,
    #[serde(skip)] sov0name: String,
    #[serde(skip)] sov_a3: String,
    #[serde(skip)] adm0name: String,
    #[serde(skip)] adm0_a3: String,
    #[serde(skip)] adm1name: String,
    #[serde(skip)] note: String,
    /* 14 */ pop_max: f64,
    /* 15 */ pop_min: f64,
    /* 16 */ pop_other: f64,
    #[serde(skip)] meganame: f64,
    #[serde(skip)] ls_name: String,
    /* 19 */ date: Option<String>,
    /* 20 */ start: Option<String>,
    /* 21 */ end: Option<String>,
    /* 22 */ boolean: Option<u8>,
}

impl fmt::Display for ZPlace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{} ('{}', '{}') ({}, {}, {}) [{:?}, {:?}, {:?}] {:?}",
            self.fid,
            self.name,
            self.nameascii,
            self.pop_min,
            self.pop_max,
            self.pop_other,
            self.date,
            self.start,
            self.end,
            self.boolean
        )
    }
}

impl TryFrom<ZPlace> for Resource {
    type Error = Box<dyn Error>;

    fn try_from(value: ZPlace) -> Result<Self, Self::Error> {
        let mut map = HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkt(&value.geom)?),
            ("name".into(), Q::new_plain_str(&value.name)),
            ("nameascii".into(), Q::new_plain_str(&value.nameascii)),
            ("pop_max".into(), Q::from(value.pop_max)),
            ("pop_min".into(), Q::from(value.pop_min)),
            ("pop_other".into(), Q::from(value.pop_other)),
        ]);
        // now optional fields...
        if let Some(x) = value.date {
            map.insert("date".into(), Q::try_from_date_str(&x)?);
        }
        // time-stamp values lack a 'Z' suffix to be correctly recognized as timestamps...
        if let Some(mut x) = value.start {
            x.push('Z');
            map.insert("start".into(), Q::try_from_timestamp_str(&x)?);
        }
        if let Some(mut x) = value.end {
            x.push('Z');
            map.insert("end".into(), Q::try_from_timestamp_str(&x)?);
        }
        // booleans, when present, are encoded as 1 and 0...
        if let Some(x) = value.boolean {
            map.insert("boolean".into(), Q::Bool(x != 0));
        }

        Ok(map)
    }
}

impl ZPlace {
    // Convenience constructor for a new instance from a CSV data set record.
    pub(crate) fn new_from_record(r: StringRecord) -> Result<Resource, Box<dyn Error>> {
        let row = ZPlace::try_from(r)?;
        Ok(Resource::try_from(row)?)
    }
}

impl TryFrom<StringRecord> for ZPlace {
    type Error = Box<dyn Error>;

    fn try_from(value: StringRecord) -> Result<Self, Self::Error> {
        let fid = value.get(0).expect("Missing 'fid'").parse::<i32>()?;
        let geom = value.get(1).expect("Missing 'geom'").to_owned();
        let name = value.get(3).expect("Missing 'name'").to_owned();
        let nameascii = value.get(6).expect("Missing 'nameascii'").to_owned();
        let pop_max = value.get(14).expect("Missing 'pop_max'").parse::<f64>()?;
        let pop_min = value.get(15).expect("Missing 'pop_min'").parse::<f64>()?;
        let pop_other = value.get(16).expect("Missing 'pop_other'").parse::<f64>()?;
        let date = value
            .get(19)
            .map(|x| x.to_owned())
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string());
        let start = value
            .get(20)
            .map(|x| x.to_owned())
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string());
        let end = value
            .get(21)
            .map(|x| x.to_owned())
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string());
        let boolean = value
            .get(22)
            .map(|x| x.to_owned())
            .filter(|x| !x.is_empty())
            .map(|x| x.parse::<u8>().expect("Failed parsing 'boolean'"));
        Ok(Self {
            fid,
            geom,
            name,
            nameascii,
            pop_max,
            pop_min,
            pop_other,
            date,
            start,
            end,
            boolean,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{DATASETS, PLACES};
    use csv::Reader;
    use geos::{Geom, Geometry};
    use std::fs::File;

    #[test]
    fn test() -> Result<(), Box<dyn Error>> {
        let file = File::open(DATASETS[PLACES])?;
        let mut rdr = Reader::from_reader(file);
        let (mut count, mut count_temporals) = (0, 0);
        for record in rdr.deserialize() {
            let place: ZPlace = record?;
            count += 1;
            // all geometries are valid points...
            let g = Geometry::new_from_wkt(&place.geom)?;
            assert_eq!(g.get_type()?, "Point");
            // all 3 temporal fields are present or not together...
            if let Some(_) = place.date {
                assert!(place.start.is_some());
                assert!(place.end.is_some());
                count_temporals += 1;
            } else {
                assert!(place.start.is_none());
                assert!(place.end.is_none());
            }
        }
        // set contains 243 rows...
        assert_eq!(count, 243);
        // ... but only 3 contain non-trivial temporal values...
        assert_eq!(count_temporals, 3);
        Ok(())
    }
}
