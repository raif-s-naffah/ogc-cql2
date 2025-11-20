// SPDX-License-Identifier: Apache-2.0

//! Code to handle representing records of `ne_110m_populated_places_simple`
//! conformance test data and logic to convert them to structures that
//! can be used by the library.
//!

use crate::utils::GPKG_URL;
use core::fmt;
use csv::StringRecord;
use futures::{StreamExt, TryStreamExt};
use ogc_cql2::prelude::*;
use serde::Deserialize;
use sqlx::{AssertSqlSafe, FromRow};
use std::{collections::HashMap, error::Error, fs::File};

const PLACES_CSV: &str = "./tests/samples/data/ne_110m_populated_places_simple.csv";
const PLACES_TBL: &str = "ne_110m_populated_places_simple";

#[allow(dead_code)]
#[rustfmt::skip]
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ZPlace {
    /*  0 */ fid: i32,
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

/// Conversion logic to map a [ZPlace] instance to a [Resource].
impl TryFrom<ZPlace> for Resource {
    type Error = MyError;

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

/// Conversion logic to map a generic [StringRecord] instance to a [ZPlace].
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

gen_csv_ds!(pub(crate), "Place", PLACES_CSV, ZPlace);

/// Read all _Simple Places_ CSV test data-set rows, convert each to a
/// [Resource] and return the lot.
pub(crate) fn places() -> Result<Vec<Resource>, MyError> {
    let csv = PlaceCSV::new();
    let mut result = vec![];
    for x in csv.iter()? {
        let row = x?;
        let resource = Resource::try_from(row)?;
        result.push(resource);
    }

    Ok(result)
}

// ============================================================================

#[derive(Debug, FromRow)]
pub(crate) struct TPlace {
    fid: i32,
    geom: Vec<u8>,
    name: String,
    nameascii: String,
    pop_max: u32,
    pop_min: u32,
    pop_other: u32,
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
    boolean: Option<bool>,
}

impl TryFrom<TPlace> for Resource {
    type Error = MyError;

    fn try_from(value: TPlace) -> Result<Self, Self::Error> {
        let mut map = HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkb(&value.geom)?),
            ("name".into(), Q::new_plain_str(&value.name)),
            ("nameascii".into(), Q::new_plain_str(&value.nameascii)),
            ("pop_max".into(), Q::Num(value.pop_max.into())),
            ("pop_min".into(), Q::Num(value.pop_min.into())),
            ("pop_other".into(), Q::Num(value.pop_other.into())),
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
        if let Some(x) = value.boolean {
            map.insert("boolean".into(), Q::Bool(x));
        }

        Ok(map)
    }
}

gen_gpkg_ds!(
    pub(crate),
    "Place",
    GPKG_URL,
    PLACES_TBL,
    TPlace
);

#[cfg(test)]
mod tests {
    use super::*;
    use ogc_cql2::{G, GTrait};
    use sqlx::any::install_default_drivers;

    #[test]
    fn test_iter() -> Result<(), Box<dyn Error>> {
        let csv = PlaceCSV::new();
        let (mut count, mut count_temporals) = (0, 0);
        for x in csv.iter()? {
            let place = x?;
            count += 1;
            // all geometries are valid points...
            let g = G::try_from(place.geom.as_str())?;
            assert_eq!(g.type_(), "Point");

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

    #[tokio::test]
    async fn test_fetch() -> Result<(), Box<dyn Error>> {
        install_default_drivers();

        let (mut count, mut count_temporals) = (0, 0);
        let gpkg = PlaceGPkg::new().await?;
        let mut stream = gpkg.fetch().await?;
        while let Some(p) = stream.try_next().await? {
            count += 1;
            // all geometries are valid points...
            let wkb: &[u8] = &p.geom;
            let g = G::try_from(wkb)?;
            assert_eq!(g.type_(), "Point");

            // all 3 temporal fields are present or not together...
            if let Some(_) = p.date {
                assert!(p.start.is_some());
                assert!(p.end.is_some());
                count_temporals += 1;
            } else {
                assert!(p.start.is_none());
                assert!(p.end.is_none());
            }
        }

        // set contains 243 rows...
        assert_eq!(count, 243);
        // ... but only 3 contain non-trivial temporal values...
        assert_eq!(count_temporals, 3);
        Ok(())
    }

    #[tokio::test]
    async fn test_stream() -> Result<(), Box<dyn Error>> {
        install_default_drivers();

        let (mut count, mut count_temporals) = (0, 0);
        let gpkg = PlaceGPkg::new().await?;
        let mut stream = gpkg.stream().await?;
        while let Some(p) = stream.try_next().await? {
            count += 1;
            // all geometries are valid points...
            let queryable = p.get("geom").expect("Missing 'geom'");
            let g = queryable.to_geom()?;
            assert_eq!(g.type_(), "Point");

            // all 3 temporal fields are present or not together...
            if let Some(_) = p.get("date") {
                assert!(p.get("start").is_some());
                assert!(p.get("end").is_some());
                count_temporals += 1;
            } else {
                assert!(p.get("start").is_none());
                assert!(p.get("end").is_none());
            }
        }

        // set contains 243 rows...
        assert_eq!(count, 243);
        // ... but only 3 contain non-trivial temporal values...
        assert_eq!(count_temporals, 3);
        Ok(())
    }

    #[tokio::test]
    async fn test_sql() -> Result<(), Box<dyn Error>> {
        #[rustfmt::skip]
        const PREDICATES: [(&str, u32); 36] = [
            (r#"name IS NOT NULL"#,  243),
            (r#"name IS NULL"#,        0),
            (r#"name='København'"#,    1),
            (r#"name>='København'"#, 137),
            (r#"name>'København'"#,  136),
            (r#"name<='København'"#, 107),
            (r#"name<'København'"#,  106),
            (r#"name<>'København'"#, 242),
            // -----
            (r#"pop_other IS NOT NULL"#, 243),
            (r#"pop_other IS NULL"#,       0),
            (r#"pop_other=1038288"#,       1),
            (r#"pop_other>=1038288"#,    123),
            (r#"pop_other>1038288"#,     122),
            (r#"pop_other<=1038288"#,    121),
            (r#"pop_other<1038288"#,     120),
            (r#"pop_other<>1038288"#,    242),
            // -----
            (r#""date" IS NOT NULL"#,         3),
            (r#""date" IS NULL"#,           240),
            (r#""date"=DATE('2022-04-16')"#,  1),
            (r#""date">=DATE('2022-04-16')"#, 2),
            (r#""date">DATE('2022-04-16')"#,  1),
            (r#""date"<=DATE('2022-04-16')"#, 2),
            (r#""date"<DATE('2022-04-16')"#,  1),
            (r#""date"<>DATE('2022-04-16')"#, 2),
            // -----
            (r#"start IS NOT NULL"#,                        3),
            (r#"start IS NULL"#,                          240),
            (r#"start=TIMESTAMP('2022-04-16T10:13:19Z')"#,  1),
            (r#"start>=TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
            (r#"start>TIMESTAMP('2022-04-16T10:13:19Z')"#,  1),
            (r#"start<=TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
            (r#"start<TIMESTAMP('2022-04-16T10:13:19Z')"#,  1),
            (r#"start<>TIMESTAMP('2022-04-16T10:13:19Z')"#, 2),
            // -----
            (r#"boolean IS NOT NULL"#, 3),
            (r#"boolean IS NULL"#,   240),
            (r#"boolean=true"#,        2),
            (r#"boolean=false"#,       1),
        ];

        let gpkg = PlaceGPkg::new().await?;
        // use the 'stream_where()' entry point -> TPlace...
        for (ndx, (filter, expected)) in PREDICATES.iter().enumerate() {
            let exp = Expression::try_from_text(&filter)?;
            let mut actual = 0;
            let mut stream = gpkg.fetch_where(&exp).await?;
            while let Some(_) = stream.try_next().await? {
                actual += 1;
            }
            assert_eq!(actual, *expected, "Failed predicate #{ndx}");
        }
        Ok(())
    }
}
