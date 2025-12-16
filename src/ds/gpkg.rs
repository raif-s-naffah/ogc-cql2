// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Artifacts specific to handling geospatial data stored in GeoPackage database
//! files.
//!

use crate::{
    CRS, E, Expression, MyError, QString,
    config::config,
    ds::{DataSource, sql::MIN_DATE_SQL},
    op::Op,
};
use sqlx::{AssertSqlSafe, FromRow, Pool, Sqlite, pool::PoolOptions, sqlite::SqliteConnectOptions};
use std::{cmp::Ordering, str::FromStr};
use tracing::{debug, info};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

const GPKG_APPLICATION_ID: i32 = 0x47504B47;
const FIND_TABLE: &str = "SELECT * FROM gpkg_contents WHERE table_name = $1";
const FIND_SRS: &str = "SELECT * FROM gpkg_spatial_ref_sys WHERE srs_id = $1";
const EPSG_AUTH: &str = "EPSG";

/// Name of a collation that is case-insensitive.
const CQL2_CI: &str = "CQL2_CI";
/// Name of a collation that is accent-insensitive.
const CQL2_AI: &str = "CQL2_AI";
/// Name of a collation that is both case- and accent-insensitive.
const CQL2_CAI: &str = "CQL2_CI_AI";

// structure to read back a textual PRAGMA value.
#[derive(Debug, FromRow)]
struct Pragma(String);

// Structure to use when SQL query is returning an integer be it a row ID or a
// numeric PRAGMA value.
#[derive(Debug, FromRow)]
struct RowID(i32);

// Partial representation of a `gpkg_spatial_ref_sys` table row.
#[derive(Debug, FromRow)]
struct TSpatialRefSys {
    organization: String,
    organization_coordsys_id: i32,
}

// Partial representation of a GeoPackage `gpkg_contents` table row.
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub(crate) struct TContents {
    table_name: String,
    data_type: String,
    srs_id: Option<i32>,
}

/// _GeoPackage_ [`DataSource`] binding a `.gpkg` database file + a layer name that
/// maps rows to _Features_ and [Resources][crate::Resource].
#[derive(Debug)]
#[allow(dead_code)]
pub struct GPkgDataSource {
    layer: String,
    pool: Pool<Sqlite>,
    srid: Option<u32>,
}

impl DataSource for GPkgDataSource {}

impl GPkgDataSource {
    /// Constructor.
    pub async fn from(gpkg_url: &str, layer_name: &str) -> Result<Self, MyError> {
        // FIXME (rsn) 20251023 - allow configuring the pool from environment
        // variables.

        // closure for case-insesnitive string comparisons.
        // let collate_ci = |a: &str, b: &str| QString::cmp_ci(a, b);
        let collate_ci = |a: &str, b: &str| cmp_ci(a, b);

        // closure for accent-insensitive string comparisons.
        let collate_ai = |a: &str, b: &str| cmp_ai(a, b);

        // closure for accent- and case-insensitive string comparisons.
        let collate_aci = |a: &str, b: &str| cmp_aci(a, b);

        // IMPORTANT - this is UNSAFE but i have no control over how to do it
        // differently since handling GeoPackage data sources is a no go w/o
        // `spatialite`...
        let pool_opts = unsafe {
            SqliteConnectOptions::from_str(gpkg_url)?
                .extension("mod_spatialite")
                .collation(CQL2_CI, collate_ci)
                .collation(CQL2_AI, collate_ai)
                .collation(CQL2_CAI, collate_aci)
        };

        let pool = PoolOptions::new().connect_with(pool_opts).await?;

        // GeoPackage SQLite DB files are expected to have 0x47504B47 (or 1196444487)
        // as their `application_id` in the DB header.
        let pragma = sqlx::query_as::<_, RowID>("PRAGMA application_id")
            .fetch_one(&pool)
            .await?;
        let application_id = pragma.0;
        if application_id != GPKG_APPLICATION_ID {
            return Err(MyError::Runtime("Unexpected application_id".into()));
        }

        // ensure it passes integerity checks...
        let pragma = sqlx::query_as::<_, Pragma>("PRAGMA integrity_check")
            .fetch_one(&pool)
            .await?;
        if pragma.0 != "ok" {
            return Err(MyError::Runtime("Failed integrity_check".into()));
        }

        // ensure it has no invalid foreign key values...
        let fk_values: Vec<_> = sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&pool)
            .await?;
        if !fk_values.is_empty() {
            return Err(MyError::Runtime("Found invalid FK value(s)".into()));
        }

        // ensure designated layer/table exists...
        let layer = sqlx::query_as::<_, TContents>(FIND_TABLE)
            .bind(layer_name)
            .fetch_one(&pool)
            .await?;
        // we only handle vector-based features, not tiles. check...
        if layer.data_type != "features" {
            return Err(MyError::Runtime("Layer is NOT vector-based".into()));
        }

        // also create a virtual table using `spatialite` _VirtualGPKG_...
        let sql = format!(
            r#"CREATE VIRTUAL TABLE IF NOT EXISTS "vgpkg_{0}" USING VirtualGPKG("{0}");"#,
            layer_name
        );
        let safe_sql = AssertSqlSafe(sql);
        sqlx::query(safe_sql).execute(&pool).await?;

        let srid = match layer.srs_id {
            // NOTE (rsn) 20251021 - the specs mandate the support for at least
            // 3 values: `4326`, `-1`, and `0` w/ the last 2 to indicate an
            // "undefined" cartesian or geographic system respectively.  ensure
            // we can represent it as a valid CRS but only if it's not an
            // undefined standard indicator...
            Some(fk) => match fk {
                -1 => {
                    info!("GeoPackage uses the undefined Cartesian SRS");
                    None
                }
                0 => {
                    info!("GeoPackage uses the undefined geographic SRS");
                    None
                }
                x => {
                    // NOTE (rsn) 20251023 - while the specs mandate the support
                    // for a `4326` value, there's no guarantee that this is in
                    // fact the EPSG:4326 SRS code.  what is guaranteed is that
                    // it's a foreign key into: `gpkg_spatial_ref_sys`.
                    let srs = sqlx::query_as::<_, TSpatialRefSys>(FIND_SRS)
                        .bind(x)
                        .fetch_one(&pool)
                        .await?;
                    // FIXME (rsn) 20251024 - handle other authorities.
                    let authority = srs.organization;
                    if !authority.eq_ignore_ascii_case(EPSG_AUTH) {
                        return Err(MyError::Runtime(
                            format!("Unexpected ({authority}) Authority").into(),
                        ));
                    }

                    let it = srs.organization_coordsys_id;
                    let epsg_code = format!("{authority}:{x}");
                    // raise an error if Proj cannot handle it...
                    let _ = CRS::new(&epsg_code)?;
                    Some(u32::try_from(it)?)
                }
            },
            None => None,
        };
        debug!("srid = {srid:?}");

        Ok(Self {
            layer: layer_name.to_owned(),
            pool,
            srid,
        })
    }

    /// Return a reference to the connection pool.
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Return name of the virtual table created for querying this
    /// GeoPackage table.
    ///
    /// This name is manufactured by pre-pending "vgpkg_" to the layer name
    /// in a similar way to how `spatialite` handles _GeoPackage_ files.
    pub fn vtable(&self) -> String {
        format!("vgpkg_{}", self.layer)
    }

    /// Transform given [Expression] to an SQL _WHERE_ clause that can be used
    /// for selecting a subset of this data source items.
    pub fn to_sql(&self, exp: &Expression) -> Result<String, MyError> {
        let mut e = exp.to_inner()?;
        let it = E::reduce(&mut e)?;
        let res = self.to_sql_impl(it);
        debug!("to_sql: {res:?}");
        res
    }

    fn to_sql_impl(&self, exp: E) -> Result<String, MyError> {
        match exp {
            E::Null => Ok("NULL".to_owned()),
            E::Unbounded => Ok(MIN_DATE_SQL.to_owned()),
            E::Bool(true) => Ok("TRUE".to_owned()),
            E::Bool(false) => Ok("FALSE".to_owned()),
            E::Num(x) => Ok(x.to_string()),
            E::Str(x) => qstr_to_sql(x),
            E::Date(x) => Ok(format!("'{}'", x.date())),
            E::Timestamp(x) => Ok(format!("'{}'", x.datetime())),
            E::Spatial(x) => Ok(x.to_sql()?),
            E::Id(x) => Ok(x.to_owned()),
            // some work need to be done when handling these options...
            E::Monadic(op, x) if op.nullable() => {
                let is_literal = x.is_literal_or_id();
                let lhs = self.to_sql_impl(*x)?;
                let z_op = op.to_sql();
                if is_literal {
                    Ok(format!("{lhs} {z_op}"))
                } else {
                    Ok(format!("({lhs}) {z_op}"))
                }
            }
            E::Monadic(op, x) => match op {
                Op::Neg | Op::Minus => {
                    let is_literal = x.is_literal_or_id();
                    let rhs = self.to_sql_impl(*x)?;
                    let z_op = op.to_sql();
                    if is_literal {
                        Ok(format!("{z_op} {rhs}"))
                    } else {
                        Ok(format!("{z_op} ({rhs})"))
                    }
                }
                Op::CaseI => match *x {
                    E::Monadic(Op::AccentI, y) => {
                        let rhs = self.to_sql_impl(*y)?;
                        Ok(format!("{rhs} COLLATE {CQL2_CAI}"))
                    }
                    _ => {
                        let rhs = self.to_sql_impl(*x)?;
                        Ok(format!("{rhs} COLLATE {CQL2_CI}"))
                    }
                },
                Op::AccentI => match *x {
                    E::Monadic(Op::CaseI, y) => {
                        let rhs = self.to_sql_impl(*y)?;
                        Ok(format!("{rhs} COLLATE {CQL2_CAI}"))
                    }
                    _ => {
                        let rhs = self.to_sql_impl(*x)?;
                        Ok(format!("{rhs} COLLATE {CQL2_AI}"))
                    }
                },
                x => unreachable!("Unexpected ({x}) monadic operator"),
            },
            E::Dyadic(op, a, b)
                if matches!(op, Op::IsBetween) || matches!(op, Op::IsNotBetween) =>
            {
                // RHS of [NOT] BETWEEN is an array of 2 numeric expressions...
                match *b {
                    E::Array(rhs) => {
                        let z_op = op.to_sql();
                        let lhs = self.to_sql_impl(*a)?;
                        let lo = self.to_sql_impl(rhs[0].to_owned())?;
                        let hi = self.to_sql_impl(rhs[1].to_owned())?;
                        Ok(format!("{lhs} {z_op} {lo} AND {hi}"))
                    }
                    _ => unreachable!("Expetced [NOT] BETWEEN's RHS expression to be an array"),
                }
            }
            E::Dyadic(op, a, b) if op.spatial() => match op {
                Op::SWithin | Op::SOverlaps | Op::STouches => self.reduce_precision(op, *a, *b),
                _ => {
                    let lhs = self.to_sql_impl(*a)?;
                    let rhs = self.to_sql_impl(*b)?;
                    let z_op = op.to_sql();
                    Ok(format!("{z_op}({lhs}, {rhs})"))
                }
            },
            E::Dyadic(op, a, b) if op.temporal() => match op {
                Op::TAfter => self.t_after_sql(*a, *b),
                Op::TBefore => self.t_before_sql(*a, *b),
                Op::TDisjoint => self.t_disjoint_sql(*a, *b),
                Op::TEquals => self.t_equals_sql(*a, *b),
                Op::TIntersects => self.t_intersects_sql(*a, *b),

                Op::TContains => self.t_contains_sql(*a, *b),
                Op::TDuring => self.t_during_sql(*a, *b),
                Op::TFinishedBy => self.t_finished_by_sql(*a, *b),
                Op::TFinishes => self.t_finishes_sql(*a, *b),
                Op::TMeets => self.t_meets_sql(*a, *b),
                Op::TMetBy => self.t_met_by_sql(*a, *b),
                Op::TOverlappedBy => self.t_overlapped_by_sql(*a, *b),
                Op::TOverlaps => self.t_overlaps_sql(*a, *b),
                Op::TStartedBy => self.t_started_by_sql(*a, *b),
                Op::TStarts => self.t_starts_sql(*a, *b),
                x => unreachable!("Unexpected ({x:?}) operator"),
            },
            E::Dyadic(op, a, b) if op.array() => {
                let z_op = op.to_sql();
                let lhs = self.to_sql_impl(*a)?;
                let rhs = self.to_sql_impl(*b)?;
                Ok(format!("{lhs} {z_op} {rhs}"))
            }
            E::Dyadic(op, a, b) if matches!(op, Op::IsLike) || matches!(op, Op::IsNotLike) => {
                let a_is_literal = a.is_literal_or_id();
                let lhs = self.to_sql_impl(*a)?;
                let rhs = self.to_sql_impl(*b)?;
                let z_op = op.to_sql();
                match a_is_literal {
                    true => Ok(format!("{lhs} {z_op} ({rhs})")),
                    false => Ok(format!("({lhs}) {z_op} ({rhs})")),
                }
            }
            E::Dyadic(op, a, b) => {
                let a_is_literal = a.is_literal_or_id();
                let b_is_literal = b.is_literal_or_id();
                let lhs = self.to_sql_impl(*a)?;
                let rhs = self.to_sql_impl(*b)?;
                let z_op = op.to_sql();
                match (a_is_literal, b_is_literal) {
                    (true, true) => Ok(format!("{lhs} {z_op} {rhs}")),
                    (true, false) => Ok(format!("{lhs} {z_op} ({rhs})")),
                    (false, true) => Ok(format!("({lhs}) {z_op} {rhs}")),
                    (false, false) => Ok(format!("({lhs}) {z_op} ({rhs})")),
                }
            }
            E::Function(x) => {
                let params: Result<Vec<String>, MyError> =
                    x.params.into_iter().map(|x| self.to_sql_impl(x)).collect();
                let params_ = params?;
                Ok(format!("{}({})", x.name, params_.join(", ")))
            }
            // NOTE (rsn) 20251105 - SQLite does not accept array elements w/in
            // square brackets; only parenthesis...
            E::Array(x) => {
                let items: Result<Vec<String>, MyError> =
                    x.into_iter().map(|x| self.to_sql_impl(x)).collect();
                let items_ = items?;
                Ok(format!("({})", items_.join(", ")))
            }
            x => unreachable!("{x:?} cannot be translated to SQL"),
        }
    }

    // NOTE (rsn) 20251120 - Some spatial functions (i.e. `ST_Within`, `ST_Covers`,
    // and `ST_Touches`) w/ GeoPackage data sources do NOT yield same results to
    // those obtained when directly using GEOS, when one of the arguments is a table
    // column.
    // we work around this by applying `ST_ReducePrecision` *before* calling those
    // functions. the precision value used in those instances is the same one
    // configured as the default (see DEFAULT_PRECISION in `config::config()`) which
    // we already use when outputing WKT strings...
    fn reduce_precision(&self, op: Op, a: E, b: E) -> Result<String, MyError> {
        let a_is_id = a.is_id();
        let b_is_id = b.is_id();
        let (lhs, rhs) = match (a_is_id, b_is_id) {
            (true, false) => {
                let lhs = self.reduce_precision_sql(a)?;
                let rhs = self.to_sql_impl(b)?;
                (lhs, rhs)
            }
            (false, true) => {
                let lhs = self.to_sql_impl(a)?;
                let rhs = self.reduce_precision_sql(b)?;
                (lhs, rhs)
            }
            _ => {
                let lhs = self.to_sql_impl(a)?;
                let rhs = self.to_sql_impl(b)?;
                (lhs, rhs)
            }
        };
        let z_op = op.to_sql();
        Ok(format!("{z_op}({lhs}, {rhs})"))
    }

    fn reduce_precision_sql(&self, a: E) -> Result<String, MyError> {
        let it = format!(
            "ST_ReducePrecision({}, 1E-{})",
            self.to_sql_impl(a)?,
            config().default_precision()
        );
        Ok(it)
    }

    // mixed (instant and interval) arguments...
    fn t_after_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (a_is_interval, b_is_interval, e0, e1, e2, e3) = crate::unfold_expressions!(a, b);
        match (a_is_interval, b_is_interval) {
            (false, false) => Ok(format!(
                "{} > {}",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e2)?
            )),
            // w/ the remaining cases, we may need additional xxx IS NOT NULL fragments...
            (false, true) => {
                let base = format!("{} > {}", self.to_sql_impl(e0)?, self.to_sql_impl(e3)?);
                let sql = crate::check_ids!(e2, base);
                Ok(sql)
            }
            (true, false) => {
                let base = format!("{} > {}", self.to_sql_impl(e0)?, self.to_sql_impl(e2)?);
                let sql = crate::check_ids!(e1, base);
                Ok(sql)
            }
            (true, true) => {
                let base = format!("{} > {}", self.to_sql_impl(e0)?, self.to_sql_impl(e3)?);
                let sql = crate::check_ids!(e1, e2, base);
                Ok(sql)
            }
        }
    }

    fn t_before_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (a_is_interval, b_is_interval, e0, e1, e2, e3) = crate::unfold_expressions!(a, b);
        match (a_is_interval, b_is_interval) {
            (false, false) => Ok(format!(
                "{} < {}",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e2)?
            )),
            (false, true) => {
                let base = format!("{} < {}", self.to_sql_impl(e0)?, self.to_sql_impl(e2)?);
                let sql = crate::check_ids!(e3, base);
                Ok(sql)
            }
            (true, false) => {
                let base = format!("{} < {}", self.to_sql_impl(e1)?, self.to_sql_impl(e2)?);
                let sql = crate::check_ids!(e0, base);
                Ok(sql)
            }
            (true, true) => {
                let base = format!("{} < {}", self.to_sql_impl(e1)?, self.to_sql_impl(e2)?);
                let sql = crate::check_ids!(e0, e3, base);
                Ok(sql)
            }
        }
    }

    fn t_disjoint_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (a_is_interval, b_is_interval, e0, e1, e2, e3) = crate::unfold_expressions!(a, b);
        match (a_is_interval, b_is_interval) {
            (false, false) => Ok(format!(
                "{} != {}",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e2)?
            )),
            (false, true) => {
                let e2_ = e2.clone();
                let e3_ = e3.clone();
                let s0 = self.to_sql_impl(e0)?;
                let s2 = self.to_sql_impl(e2)?;
                let s3 = self.to_sql_impl(e3)?;
                let base1 = format!("{s0} < {s2}");
                let sql1 = crate::check_ids!(e3_, base1);
                let base2 = format!("{s0} > {s3}");
                let sql2 = crate::check_ids!(e2_, base2);
                Ok(format!("({sql1}) OR ({sql2})"))
            }
            (true, false) => {
                let e0_ = e0.clone();
                let e1_ = e1.clone();
                let s0 = self.to_sql_impl(e0)?;
                let s1 = self.to_sql_impl(e1)?;
                let s2 = self.to_sql_impl(e2)?;
                let base1 = format!("{s1} < {s2}");
                let sql1 = crate::check_ids!(e0_, base1);
                let base2 = format!("{s0} > {s2}");
                let sql2 = crate::check_ids!(e1_, base2);
                Ok(format!("({sql1}) OR ({sql2})"))
            }
            (true, true) => {
                let e0_ = e0.clone();
                let e1_ = e1.clone();
                let e2_ = e2.clone();
                let e3_ = e3.clone();
                let s0 = self.to_sql_impl(e0)?;
                let s1 = self.to_sql_impl(e1)?;
                let s2 = self.to_sql_impl(e2)?;
                let s3 = self.to_sql_impl(e3)?;
                let base1 = format!("{s1} < {s2}");
                let sql1 = crate::check_ids!(e0_, e3_, base1);
                let base2 = format!("{s0} > {s3}");
                let sql2 = crate::check_ids!(e1_, e2_, base2);
                Ok(format!("({sql1}) OR ({sql2})"))
            }
        }
    }

    fn t_equals_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (a_is_interval, b_is_interval, e0, e1, e2, e3) = crate::unfold_expressions!(a, b);
        match (a_is_interval, b_is_interval) {
            (false, false) => Ok(format!(
                "{} = {}",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e2)?
            )),
            (false, true) => Ok(format!(
                "({0} = {1}) AND ({0} = {2})",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e2)?,
                self.to_sql_impl(e3)?
            )),
            (true, false) => Ok(format!(
                "({0} = {2}) AND ({1} = {2})",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e1)?,
                self.to_sql_impl(e2)?
            )),
            (true, true) => Ok(format!(
                "({0} = {2}) AND ({1} = {3})",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e1)?,
                self.to_sql_impl(e2)?,
                self.to_sql_impl(e3)?
            )),
        }
    }

    fn t_intersects_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (a_is_interval, b_is_interval, e0, e1, e2, e3) = crate::unfold_expressions!(a, b);
        match (a_is_interval, b_is_interval) {
            (false, false) => Ok(format!(
                "{} = {}",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e2)?
            )),
            (false, true) => Ok(format!(
                "NOT(({0} < {1}) OR ({0} > {2}))",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e2)?,
                self.to_sql_impl(e3)?
            )),
            (true, false) => Ok(format!(
                "NOT(({1} < {2}) OR ({0} > {2}))",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e1)?,
                self.to_sql_impl(e2)?
            )),
            (true, true) => Ok(format!(
                "NOT(({1} < {2}) OR ({0} > {3}))",
                self.to_sql_impl(e0)?,
                self.to_sql_impl(e1)?,
                self.to_sql_impl(e2)?,
                self.to_sql_impl(e3)?
            )),
        }
    }

    // intervals only...
    fn t_contains_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        Ok(format!(
            "({0} < {2}) AND ({1} > {3})",
            self.to_sql_impl(e0)?,
            self.to_sql_impl(e1)?,
            self.to_sql_impl(e2)?,
            self.to_sql_impl(e3)?
        ))
    }

    fn t_during_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        Ok(format!(
            "({0} > {2}) AND ({1} < {3})",
            self.to_sql_impl(e0)?,
            self.to_sql_impl(e1)?,
            self.to_sql_impl(e2)?,
            self.to_sql_impl(e3)?
        ))
    }

    fn t_finished_by_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        Ok(format!(
            "({0} < {2}) AND ({1} = {3})",
            self.to_sql_impl(e0)?,
            self.to_sql_impl(e1)?,
            self.to_sql_impl(e2)?,
            self.to_sql_impl(e3)?
        ))
    }

    fn t_finishes_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        Ok(format!(
            "({0} > {2}) AND ({1} = {3})",
            self.to_sql_impl(e0)?,
            self.to_sql_impl(e1)?,
            self.to_sql_impl(e2)?,
            self.to_sql_impl(e3)?
        ))
    }

    fn t_meets_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        let base = format!("{0} = {1}", self.to_sql_impl(e1)?, self.to_sql_impl(e2)?);
        let sql = crate::check_ids!(e0, e3, base);
        Ok(sql)
    }

    fn t_met_by_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        let base = format!("{0} = {1}", self.to_sql_impl(e0)?, self.to_sql_impl(e3)?);
        let sql = crate::check_ids!(e1, e2, base);
        Ok(sql)
    }

    fn t_overlapped_by_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        Ok(format!(
            "({0} > {2}) AND ({0} < {3}) AND ({1} > {3})",
            self.to_sql_impl(e0)?,
            self.to_sql_impl(e1)?,
            self.to_sql_impl(e2)?,
            self.to_sql_impl(e3)?
        ))
    }

    fn t_overlaps_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        Ok(format!(
            "({0} < {2}) AND ({1} > {2}) AND ({1} < {3})",
            self.to_sql_impl(e0)?,
            self.to_sql_impl(e1)?,
            self.to_sql_impl(e2)?,
            self.to_sql_impl(e3)?
        ))
    }

    fn t_started_by_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        Ok(format!(
            "({0} = {2}) AND ({1} > {3})",
            self.to_sql_impl(e0)?,
            self.to_sql_impl(e1)?,
            self.to_sql_impl(e2)?,
            self.to_sql_impl(e3)?
        ))
    }

    fn t_starts_sql(&self, a: E, b: E) -> Result<String, MyError> {
        let (e0, e1, e2, e3) = crate::unfold_intervals!(a, b);
        Ok(format!(
            "({0} = {2}) AND ({1} < {3})",
            self.to_sql_impl(e0)?,
            self.to_sql_impl(e1)?,
            self.to_sql_impl(e2)?,
            self.to_sql_impl(e3)?
        ))
    }
}

/// Return the [Ordering] when comparing `a` to `b` ignoring case.
fn cmp_ci(a: &str, b: &str) -> Ordering {
    a.to_lowercase().cmp(&b.to_lowercase())
}

/// Return the [Ordering] when comparing `a` to `b` ignoring accents.
fn cmp_ai(a: &str, b: &str) -> Ordering {
    let lhs = a.nfd().filter(|x| !is_combining_mark(*x)).nfc();
    let rhs = b.nfd().filter(|x| !is_combining_mark(*x)).nfc();
    lhs.cmp(rhs)
}

/// Return the [Ordering] when comparing `a` to `b` ignoring both accents
/// and case.
fn cmp_aci(a: &str, b: &str) -> Ordering {
    let x = a.to_lowercase();
    let y = b.to_lowercase();
    let lhs = x.nfd().filter(|x| !is_combining_mark(*x)).nfc();
    let rhs = y.nfd().filter(|x| !is_combining_mark(*x)).nfc();
    lhs.cmp(rhs)
}

/// Generate a string that can be used in composing an SQL WHERE clause.
fn qstr_to_sql(qs: QString) -> Result<String, MyError> {
    match qs.flags() {
        0 => Ok(format!("'{}'", qs.inner())),
        1 => Ok(format!("'{}' COLLATE {CQL2_CI}", qs.inner())),
        2 => Ok(format!("'{}' COLLATE {CQL2_AI}", qs.inner())),
        3 => Ok(format!("'{}' COLLATE {CQL2_CAI}", qs.inner())),
        x => {
            let msg = format!("String w/ '{x}' flags has NO direct SQL representation");
            Err(MyError::Runtime(msg.into()))
        }
    }
}

/// Macro to generate a concrete [GPkgDataSource].
///
/// Caller must provide the following parameters:
/// * `$vis`: Visibility specifier of the generated artifacts; e.g. `pub(crate)`.
/// * `$name`: Prefix of the concrete data source structure name to materialize.
///   The final name will have a 'GPkg' suffix appended; eg. `Foo` -> `FooGPkg`.
/// * `$gpkg_url`: Database URL to an accessible _GeoPackage_ DB; e.g.
///   `sqlite:path/to/a/geo_package.gpkg`
/// * `$layer`: Name of the table/layer containing the features' data.
/// * `$feature`: `sqlx` _FromRow_ convertible structure to map database layer
///   table rows to _Features_.
#[macro_export]
macro_rules! gen_gpkg_ds {
    ($vis:vis, $name:expr, $gpkg_url:expr, $layer:expr, $feature:expr) => {
        ::paste::paste! {
            /// Concrete GeoPackage source.
            $vis struct [<$name GPkg>](GPkgDataSource);

            impl [<$name GPkg>] {
                /// Constructor.
                $vis async fn new() -> Result<Self, MyError> {
                    let gpkp = GPkgDataSource::from($gpkg_url, $layer).await?;
                    Ok(Self(gpkp))
                }

                /// Convert a GeoPackage row (aka Feature) to a generic Resource.
                $vis fn to_resource(r: $feature) -> Result<Resource, Box<dyn Error>> {
                    let row = $feature::try_from(r)?;
                    Ok(Resource::try_from(row)?)
                }

                /// Convenience method. Calls inner's samilarly named method.
                $vis fn vtable(&self) -> String {
                    self.0.vtable()
                }

                /// Return a reference to the inner model data source.
                $vis fn inner(&self) -> &GPkgDataSource {
                    &self.0
                }
            }

            impl ::core::fmt::Display for [<$name GPkg>] {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{}GPkg({})", $name, $layer)
                }
            }

            #[::async_trait::async_trait]
            impl StreamableDS for [<$name GPkg>] {
                type Item = $feature;
                type Err = MyError;

                async fn fetch(
                    &self
                ) -> Result<::futures::stream::BoxStream<'_, Result<$feature, MyError>>, MyError> {
                    let sql = format!("SELECT * FROM {}", $layer);
                    let safe_sql = ::sqlx::AssertSqlSafe(sql);
                    let it = sqlx::query_as::<_, $feature>(safe_sql)
                        .fetch(self.0.pool())
                        .map_err(MyError::SQL);
                    Ok(Box::pin(it))
                }

                async fn stream(
                    &self
                ) -> Result<::futures::stream::BoxStream<'_, Result<Resource, MyError>>, MyError> {
                    let rows = self.fetch().await?;
                    let resources = rows
                        .try_filter_map(|row| async move {
                            match Resource::try_from(row) {
                                Ok(x) => Ok(Some(x)),
                                Err(x) => Err(x),
                            }
                        })
                        .boxed();
                    Ok(resources)
                }

                async fn fetch_where(
                    &self,
                    exp: &Expression,
                ) -> Result<::futures::stream::BoxStream<'_, Result<$feature, MyError>>, MyError> {
                    let where_clause = self.0.to_sql(exp)?;
                    let sql = format!(r#"SELECT * FROM "{}" WHERE {}"#, self.vtable(), where_clause);
                    let safe_sql = ::sqlx::AssertSqlSafe(sql);
                    let it = sqlx::query_as::<_, $feature>(safe_sql)
                        .fetch(self.0.pool())
                        .map_err(MyError::SQL);
                    Ok(Box::pin(it))
                }

                async fn stream_where(
                    &self,
                    exp: &Expression,
                ) -> Result<::futures::stream::BoxStream<'_, Result<Resource, MyError>>, MyError> {
                    let rows = self.fetch_where(exp).await?;
                    let resources = rows
                        .try_filter_map(|row| async move {
                            match Resource::try_from(row) {
                                Ok(x) => Ok(Some(x)),
                                Err(x) => Err(x),
                            }
                        })
                        .boxed();
                    Ok(resources)
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_ci() {
        let eq = cmp_ci("abc", "ABC");
        assert_eq!(eq, Ordering::Equal);

        let eq = cmp_ci("ABC", "abc");
        assert_eq!(eq, Ordering::Equal);

        let eq = cmp_ci("aBc", "AbC");
        assert_eq!(eq, Ordering::Equal);

        let eq = cmp_ci("abcd", "ABCe");
        assert_eq!(eq, Ordering::Less);

        let eq = cmp_ci("bcd", "ACz");
        assert_eq!(eq, Ordering::Greater);
    }

    #[test]
    fn test_cmp_ai() {
        let eq = cmp_ai("ÁBC", "ABC");
        assert_eq!(eq, Ordering::Equal);

        let eq = cmp_ai("ÁBC", "ABÇ");
        assert_eq!(eq, Ordering::Equal);
    }

    #[test]
    fn test_cmp_aci() {
        let eq = cmp_aci("ábc", "ABÇ");
        assert_eq!(eq, Ordering::Equal);
    }
}
