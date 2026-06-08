// SPDX-License-Identifier: Apache-2.0

//! Code to handle representing records of `ne_110m_admin_0_countries`
//! conformance test data and logic to convert them to structures that
//! can be used by the library.
//!

#[allow(unused_imports)]
use futures::{StreamExt, TryStreamExt};
// #[cfg(feature = "pg_ds")]
// use ogc_cql2::gen_pg_ds;
// #[cfg(any(feature = "gpkg_ds", feature = "pg_ds"))]
// use ogc_cql2::prelude::*;
// #[cfg(feature = "csv_ds")]
// use ogc_cql2::{IterableDS, gen_csv_ds};
// use ogc_cql2::{MyError, Q, Resource};
use ogc_cql2::prelude::*;
#[cfg(any(feature = "gpkg_ds", feature = "pg_ds"))]
use sqlx::FromRow;

#[cfg(feature = "csv_ds")]
const COUNTRIES_CSV: &str = "./tests/samples/data/ne_110m_admin_0_countries.csv";
#[cfg(any(feature = "gpkg_ds", feature = "pg_ds"))]
const COUNTRIES_TBL: &str = "ne_110m_admin_0_countries";

/// Type to easily represent a CSV data source.
#[cfg(feature = "csv_ds")]
#[derive(Debug, Default, serde::Deserialize)]
#[rustfmt::skip]
#[allow(dead_code)]
pub(crate) struct ZCountry {
    /*  0 */ fid: i32,
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

#[cfg(feature = "csv_ds")]
impl core::fmt::Display for ZCountry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.fid, self.name, self.pop_est)
    }
}

/// Conversion logic to map a [ZCountry] instance to a [Resource].
#[cfg(feature = "csv_ds")]
impl TryFrom<ZCountry> for Resource {
    type Error = MyError;

    fn try_from(value: ZCountry) -> Result<Self, Self::Error> {
        Ok(std::collections::HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkt(&value.geom)?),
            ("NAME".into(), Q::new_plain_str(&value.name)),
            ("POP_EST".into(), Q::from(value.pop_est)),
        ]))
    }
}

#[cfg(feature = "csv_ds")]
impl ZCountry {
    /// Return the `name` value of this.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(feature = "csv_ds")]
gen_csv_ds!(pub(crate), "Country", COUNTRIES_CSV, ZCountry);

/// Read all _Countries_ CSV test data-set rows, convert each to a [Resource]
/// and return the lot.
#[cfg(feature = "csv_ds")]
pub(crate) fn countries() -> Result<Vec<Resource>, MyError> {
    let csv = CountryCSV::new();
    let mut result = vec![];
    for x in csv.iter()? {
        let row = x?;
        let resource = Resource::try_from(row)?;
        result.push(resource);
    }

    Ok(result)
}

// ============================================================================

#[cfg(feature = "gpkg_ds")]
#[derive(Debug, FromRow)]
#[rustfmt::skip]
pub(crate) struct TCountry {
    fid: i32,
    geom: Vec<u8>,
    #[sqlx(rename = "NAME")] name: String,
    #[sqlx(rename = "POP_EST")] pop_est: f64,
}

#[cfg(feature = "gpkg_ds")]
impl TryFrom<TCountry> for Resource {
    type Error = MyError;

    fn try_from(value: TCountry) -> Result<Self, Self::Error> {
        Ok(std::collections::HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkb(&value.geom)?),
            ("NAME".into(), Q::new_plain_str(&value.name)),
            ("POP_EST".into(), Q::from(value.pop_est)),
        ]))
    }
}

#[cfg(feature = "gpkg_ds")]
gen_gpkg_ds!(
    pub(crate),
    "Country",
    super::GPKG_URL,
    COUNTRIES_TBL,
    TCountry
);

// similar to `countries()` but source its features from the GeoPackage data source.
#[cfg(feature = "gpkg_ds")]
pub(crate) async fn countries_gpkg() -> Result<Vec<Resource>, MyError> {
    let gpkg = CountryGPkg::new().await?;
    let stream = gpkg.stream().await?;
    let result = futures::TryStreamExt::try_collect::<Vec<Resource>>(stream).await?;
    Ok(result)
}

// ============================================================================

#[cfg(feature = "pg_ds")]
#[derive(Debug, FromRow)]
#[rustfmt::skip]
pub(crate) struct LCountry {
    fid: i32,
    #[sqlx(rename = "NAME")] name: String,
    #[sqlx(rename = "POP_EST")] pop_est: f64,
    geom: G,
}

#[cfg(feature = "pg_ds")]
impl TryFrom<LCountry> for Resource {
    type Error = MyError;

    fn try_from(value: LCountry) -> Result<Self, Self::Error> {
        Ok(std::collections::HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("NAME".into(), Q::new_plain_str(&value.name)),
            ("POP_EST".into(), Q::from(value.pop_est)),
            ("geom".into(), Q::Geom(value.geom)),
        ]))
    }
}

#[cfg(feature = "pg_ds")]
gen_pg_ds!(pub(crate), "Country", super::PG_DB_NAME, COUNTRIES_TBL, LCountry);

#[cfg(test)]
mod tests {
    #[cfg(any(feature = "csv_ds", feature = "gpkg_ds", feature = "pg_ds"))]
    use std::error::Error;

    #[cfg(feature = "csv_ds")]
    #[test]
    fn test_iter() -> Result<(), Box<dyn Error>> {
        use crate::utils::CountryCSV;
        use ogc_cql2::prelude::*;

        let csv = CountryCSV::new();
        let mut count = 0;
        for x in csv.iter()? {
            let country = x?;
            count += 1;
            // all geometries are valid multi-polygons...
            let g = G::try_from(country.geom.as_str())?;
            assert_eq!(g.type_(), "MultiPolygon");
        }

        // set contains 177 rows...
        assert_eq!(count, 177);
        Ok(())
    }

    #[cfg(feature = "gpkg_ds")]
    #[tokio::test]
    async fn test_fetch() -> Result<(), Box<dyn Error>> {
        use crate::utils::CountryGPkg;
        use futures::TryStreamExt;
        use ogc_cql2::prelude::*;

        sqlx::any::install_default_drivers();

        let mut count = 0;
        let gpkg = CountryGPkg::new().await?;
        // use the 'fetch()' entry point -> TCountry...
        let mut stream = gpkg.fetch().await?;
        while let Some(c) = stream.try_next().await? {
            count += 1;
            // all geometries are valid multi-polygons...
            let wkb: &[u8] = &c.geom;
            let g = G::try_from(wkb)?;
            assert_eq!(g.type_(), "MultiPolygon");
        }

        // layer contains 177 features...
        assert_eq!(count, 177);
        Ok(())
    }

    #[cfg(feature = "gpkg_ds")]
    #[tokio::test]
    async fn test_fetch_where() -> Result<(), Box<dyn Error>> {
        use crate::utils::CountryGPkg;
        use futures::TryStreamExt;
        use ogc_cql2::prelude::*;

        #[rustfmt::skip]
        const PREDICATES: [(&str, u32); 12] = [
            (r#"NAME='Luxembourg'"#,    1),
            (r#"NAME>='Luxembourg'"#,  84),
            (r#"NAME>'Luxembourg'"#,   83),
            (r#"NAME<='Luxembourg'"#,  94),
            (r#"NAME<'Luxembourg'"#,   93),
            (r#"NAME<>'Luxembourg'"#, 176),
            // -----
            (r#"POP_EST=37589262"#,    1),
            (r#"POP_EST>=37589262"#,  39),
            (r#"POP_EST>37589262"#,   38),
            (r#"POP_EST<=37589262"#, 139),
            (r#"POP_EST<37589262"#,  138),
            (r#"POP_EST<>37589262"#, 176),
        ];

        sqlx::any::install_default_drivers();

        let gpkg = CountryGPkg::new().await?;
        // use the 'fetch_where()' entry point -> TCountry...
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

    #[cfg(feature = "gpkg_ds")]
    #[tokio::test]
    async fn test_streamable() -> Result<(), Box<dyn Error>> {
        use crate::utils::CountryGPkg;
        use futures::TryStreamExt;
        use ogc_cql2::prelude::*;

        let mut count = 0;
        let gpkg = CountryGPkg::new().await?;
        // use the 'stream()' entry point -> Resource...
        let mut stream = gpkg.stream().await?;
        while let Some(c) = stream.try_next().await? {
            count += 1;
            let queryable = c.get("geom").expect("Missing 'geom'");
            let g = queryable.to_geom()?;
            // all geometries are valid multi-polygons...
            assert_eq!(g.type_(), "MultiPolygon");
        }

        // layer contains 177 features...
        assert_eq!(count, 177);
        Ok(())
    }

    #[cfg(feature = "pg_ds")]
    #[tokio::test]
    async fn test_pg() -> Result<(), Box<dyn Error>> {
        use crate::utils::CountryPG;
        use futures::TryStreamExt;
        use ogc_cql2::prelude::*;

        let mut count = 0;
        let ds = CountryPG::new().await?;
        let mut stream = ds.stream().await?;
        while let Some(c) = stream.try_next().await? {
            count += 1;
            let queryable = c.get("geom").expect("Missing 'geom'");
            let g = queryable.to_geom()?;
            // all geometries are valid multi-polygons...
            assert_eq!(g.type_(), "MultiPolygon");
        }

        // layer contains 177 features...
        assert_eq!(count, 177);
        Ok(())
    }
}
