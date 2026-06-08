// SPDX-License-Identifier: Apache-2.0

//! Code to handle representing records of `ne_110m_rivers_lake_centerlines`
//! conformance test data and logic to convert them to structures that
//! can be used by the library.
//!

#[cfg(any(feature = "gpkg_ds", feature = "pg_ds"))]
use futures::{StreamExt, TryStreamExt};
// #[cfg(feature = "csv_ds")]
// use ogc_cql2::gen_csv_ds;
// #[cfg(feature = "gpkg_ds")]
// use ogc_cql2::gen_gpkg_ds;
// #[cfg(feature = "pg_ds")]
// use ogc_cql2::gen_pg_ds;
// use ogc_cql2::{MyError, Q, Resource};
use ogc_cql2::prelude::*;

#[cfg(feature = "csv_ds")]
const RIVERS_CSV: &str = "./tests/samples/data/ne_110m_rivers_lake_centerlines.csv";
#[cfg(any(feature = "gpkg_ds", feature = "pg_ds"))]
const RIVERS_TBL: &str = "ne_110m_rivers_lake_centerlines";

#[cfg(feature = "csv_ds")]
#[derive(Debug, Default, serde::Deserialize)]
#[rustfmt::skip]
pub(crate) struct ZRiver {
    /* 0 */ fid: i32,
    /* 1 */ geom: String,
    /* 2 */ name: String,
    #[serde(skip)] ignored: std::marker::PhantomData<String>
}

#[cfg(feature = "csv_ds")]
impl core::fmt::Display for ZRiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.fid, self.name)
    }
}

/// Conversion logic to map a [ZRiver] instance to a [Resource].
#[cfg(feature = "csv_ds")]
impl TryFrom<ZRiver> for Resource {
    type Error = MyError;

    fn try_from(value: ZRiver) -> Result<Self, Self::Error> {
        Ok(std::collections::HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkt(&value.geom)?),
            ("name".into(), Q::new_plain_str(&value.name)),
        ]))
    }
}

#[cfg(feature = "csv_ds")]
gen_csv_ds!(pub(crate), "River", RIVERS_CSV, ZRiver);

#[cfg(feature = "csv_ds")]
#[cfg(test)]
pub(crate) fn rivers() -> Result<Vec<ZRiver>, MyError> {
    use ogc_cql2::IterableDS;

    let csv = RiverCSV::new();
    let it: Result<Vec<ZRiver>, MyError> = csv.iter()?.collect();
    Ok(it?)
}

// ============================================================================

#[cfg(feature = "gpkg_ds")]
#[derive(Debug, sqlx::prelude::FromRow)]
pub(crate) struct TRiver {
    fid: i32,
    geom: Vec<u8>,
    name: String,
}

#[cfg(feature = "gpkg_ds")]
impl TryFrom<TRiver> for Resource {
    type Error = MyError;

    fn try_from(value: TRiver) -> Result<Self, Self::Error> {
        Ok(std::collections::HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkb(&value.geom)?),
            ("name".into(), Q::new_plain_str(&value.name)),
        ]))
    }
}

#[cfg(feature = "gpkg_ds")]
gen_gpkg_ds!(
    pub(crate),
    "River",
    super::GPKG_URL,
    RIVERS_TBL,
    TRiver
);

// ============================================================================

#[cfg(feature = "pg_ds")]
#[derive(Debug, sqlx::prelude::FromRow)]
pub(crate) struct LRiver {
    fid: i32,
    name: String,
    geom: ogc_cql2::G,
}

#[cfg(feature = "pg_ds")]
impl TryFrom<LRiver> for Resource {
    type Error = MyError;

    fn try_from(value: LRiver) -> Result<Self, Self::Error> {
        Ok(std::collections::HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("name".into(), Q::new_plain_str(&value.name)),
            ("geom".into(), Q::Geom(value.geom)),
        ]))
    }
}

#[cfg(feature = "pg_ds")]
gen_pg_ds!(pub(crate), "River", super::PG_DB_NAME, RIVERS_TBL, LRiver);

#[cfg(test)]
mod tests {
    #[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
    use std::error::Error;

    #[cfg(feature = "csv_ds")]
    #[test]
    fn test_iter() -> Result<(), Box<dyn Error>> {
        use crate::utils::RiverCSV;
        use ogc_cql2::{G, GTrait, IterableDS};

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

    #[cfg(feature = "csv_ds")]
    #[test]
    fn test_collect() -> Result<(), Box<dyn Error>> {
        use crate::utils::river::rivers;

        let rivers = rivers()?;
        assert_eq!(rivers.len(), 13);

        Ok(())
    }

    #[cfg(feature = "gpkg_ds")]
    #[tokio::test]
    async fn test_fetch() -> Result<(), Box<dyn Error>> {
        use crate::utils::RiverGPkg;
        use futures::TryStreamExt;
        use ogc_cql2::{G, GTrait, StreamableDS};

        sqlx::any::install_default_drivers();

        let mut count = 0;
        let gpkg = RiverGPkg::new().await?;
        let mut stream = gpkg.fetch().await?;
        while let Some(r) = stream.try_next().await? {
            count += 1;
            // all geometries are valid lines...
            let wkb: &[u8] = &r.geom;
            let g = G::try_from(wkb)?;
            assert_eq!(g.type_(), "LineString");
        }

        // layer contains 13 features...
        assert_eq!(count, 13);
        Ok(())
    }

    #[cfg(feature = "gpkg_ds")]
    #[tokio::test]
    async fn test_stream() -> Result<(), Box<dyn Error>> {
        use crate::utils::RiverGPkg;
        use futures::TryStreamExt;
        use ogc_cql2::{GTrait, StreamableDS};

        sqlx::any::install_default_drivers();

        let mut count = 0;
        let gpkg = RiverGPkg::new().await?;
        let mut stream = gpkg.stream().await?;
        while let Some(r) = stream.try_next().await? {
            count += 1;
            // all geometries are valid lines...
            let queryable = r.get("geom").expect("Missing 'geom'");
            let g = queryable.to_geom()?;
            assert_eq!(g.type_(), "LineString");
        }

        // layer contains 13 features...
        assert_eq!(count, 13);
        Ok(())
    }

    #[cfg(feature = "pg_ds")]
    #[tokio::test]
    async fn test_pg() -> Result<(), Box<dyn Error>> {
        use crate::utils::RiverPG;
        use futures::TryStreamExt;
        use ogc_cql2::{GTrait, StreamableDS};

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
