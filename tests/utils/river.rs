// SPDX-License-Identifier: Apache-2.0

//! Code to handle representing records of `ne_110m_rivers_lake_centerlines`
//! conformance test data and logic to convert them to structures that
//! can be used by the library.
//!

use crate::utils::{GPKG_URL, PG_DB_NAME};
use core::fmt;
use futures::{StreamExt, TryStreamExt};
use ogc_cql2::prelude::*;
use serde::Deserialize;
use sqlx::FromRow;
use std::{collections::HashMap, error::Error, marker::PhantomData};

const RIVERS_CSV: &str = "./tests/samples/data/ne_110m_rivers_lake_centerlines.csv";
const RIVERS_TBL: &str = "ne_110m_rivers_lake_centerlines";

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

/// Conversion logic to map a [ZRiver] instance to a [Resource].
impl TryFrom<ZRiver> for Resource {
    type Error = MyError;

    fn try_from(value: ZRiver) -> Result<Self, Self::Error> {
        Ok(HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkt(&value.geom)?),
            ("name".into(), Q::new_plain_str(&value.name)),
        ]))
    }
}

gen_csv_ds!(pub(crate), "River", RIVERS_CSV, ZRiver);

#[cfg(test)]
fn rivers() -> Result<Vec<ZRiver>, MyError> {
    let csv = RiverCSV::new();
    let it: Result<Vec<ZRiver>, MyError> = csv.iter()?.collect();
    Ok(it?)
}

// ============================================================================

#[derive(Debug, FromRow)]
pub(crate) struct TRiver {
    fid: i32,
    geom: Vec<u8>,
    name: String,
}

impl TryFrom<TRiver> for Resource {
    type Error = MyError;

    fn try_from(value: TRiver) -> Result<Self, Self::Error> {
        Ok(HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkb(&value.geom)?),
            ("name".into(), Q::new_plain_str(&value.name)),
        ]))
    }
}

gen_gpkg_ds!(
    pub(crate),
    "River",
    GPKG_URL,
    RIVERS_TBL,
    TRiver
);

// ============================================================================

#[derive(Debug, FromRow)]
pub(crate) struct LRiver {
    fid: i32,
    name: String,
    geom: G,
}

impl TryFrom<LRiver> for Resource {
    type Error = MyError;

    fn try_from(value: LRiver) -> Result<Self, Self::Error> {
        Ok(HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("name".into(), Q::new_plain_str(&value.name)),
            ("geom".into(), Q::Geom(value.geom)),
        ]))
    }
}

gen_pg_ds!(pub(crate), "River", PG_DB_NAME, RIVERS_TBL, LRiver);

#[cfg(test)]
mod tests {
    use super::*;
    use ogc_cql2::{G, GTrait};
    use sqlx::any::install_default_drivers;

    #[test]
    fn test_iter() -> Result<(), Box<dyn Error>> {
        let csv = RiverCSV::new();
        let mut count = 0;
        for x in csv.iter()? {
            let river = x?;
            count += 1;
            // all geometries are valid lines...
            let g = G::try_from(river.geom.as_str())?;
            assert_eq!(g.type_(), "LineString");
        }

        // set contains 13 rows...
        assert_eq!(count, 13);
        Ok(())
    }

    #[test]
    fn test_collect() -> Result<(), Box<dyn Error>> {
        let rivers = rivers()?;
        assert_eq!(rivers.len(), 13);

        Ok(())
    }

    #[tokio::test]
    async fn test_fetch() -> Result<(), Box<dyn Error>> {
        install_default_drivers();

        let mut count = 0;
        let gpkg = RiverGPkg::new().await?;
        let mut stream = gpkg.fetch().await?;
        while let Some(r) = stream.try_next().await? {
            count += 1;
            // all geometries are valid mulyi-polygons...
            let wkb: &[u8] = &r.geom;
            let g = G::try_from(wkb)?;
            assert_eq!(g.type_(), "LineString");
        }

        // layer contains 13 features...
        assert_eq!(count, 13);
        Ok(())
    }

    #[tokio::test]
    async fn test_stream() -> Result<(), Box<dyn Error>> {
        install_default_drivers();

        let mut count = 0;
        let gpkg = RiverGPkg::new().await?;
        let mut stream = gpkg.stream().await?;
        while let Some(r) = stream.try_next().await? {
            count += 1;
            // all geometries are valid mulyi-polygons...
            let queryable = r.get("geom").expect("Missing 'geom'");
            let g = queryable.to_geom()?;
            assert_eq!(g.type_(), "LineString");
        }

        // layer contains 13 features...
        assert_eq!(count, 13);
        Ok(())
    }

    #[tokio::test]
    async fn test_pg() -> Result<(), Box<dyn Error>> {
        let mut count = 0;
        let ds = RiverPG::new().await?;
        let mut stream = ds.stream().await?;
        while let Some(c) = stream.try_next().await? {
            count += 1;
            let queryable = c.get("geom").expect("Missing 'geom'");
            let g = queryable.to_geom()?;
            // all geometries are valid line-strings...
            assert_eq!(g.type_(), "LineString");
        }

        // layer contains 13 features...
        assert_eq!(count, 13);
        Ok(())
    }
}
