// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Artifacts specific to handling geospatial data stored in a PostGIS database
//! table.
//!

use crate::{
    DataSource, Expression, MyError, QString, config::config, ds::sql::MIN_DATE_SQL, expr::E,
    op::Op,
};
use sqlx::{
    AssertSqlSafe, FromRow, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use tracing::debug;

const FIND_TABLE: &str = "SELECT * FROM geometry_columns WHERE f_table_name = $1";
// Name of a collation that is case-insensitive. For PostgreSQL that's level 2.
const CQL2_CI: &str = "cql2_ci";
// Name of collation that is accent-insensitive. For PostgreSQL that's level 2.5
const CQL2_AI: &str = "cql2_ai";
// Name of a collation that is both case- and accent-insensitive. For
// PostgreSQL that's level 1.
const CQL2_CAI: &str = "cql2_ci_ai";
// Name of PostgreSQL builtin collation that correctly orders Unicode strings
// comparisons...
const PG_UNICODE: &str = "pg_unicode_fast";

// structure to read back a textual value.
#[derive(Debug, FromRow)]
struct Pragma(String);

// Partial representation of a PostGIS `geometry_columns` table row.
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub(crate) struct TGeometryColumn {
    f_table_name: String,
    f_geometry_column: String,
    coord_dimension: i32,
    srid: i32,
    #[sqlx(rename = "type")]
    type_: String,
}

/// [`DataSource`] binding a _PostGIS_ enabled database + a table name that
/// maps rows to _Features_ and [Resources][crate::Resource].
#[derive(Debug)]
#[allow(dead_code)]
pub struct PGDataSource {
    db_name: String,
    table: String,
    pool: PgPool,
    srid: u32,
}

impl DataSource for PGDataSource {}

impl PGDataSource {
    /// Constructor.
    pub async fn from(db_name: &str, table: &str) -> Result<Self, MyError> {
        // concatenate server URL w/ DB name...
        let pg_server_url = config().pg_url();
        let url = format!("{pg_server_url}/{db_name}");
        // parse it to start w/ a useful connection options instance...
        let pool_opts = url
            .parse::<PgConnectOptions>()?
            .application_name(config().pg_appname());
        // configure connection parameters + make a pool...
        let pool = PgPoolOptions::new()
            .min_connections(config().pg_min_connections())
            .max_connections(config().pg_max_connections())
            .acquire_timeout(config().pg_acquire_timeout())
            .idle_timeout(config().pg_idle_timeout())
            .max_lifetime(config().pg_max_lifetime())
            .connect_with(pool_opts)
            .await?;

        // ensure DB has PostGIS extension installed...  do this by selecting
        // the PostGIS_Version() function.  an OK result will suffice for now...
        let pargma = sqlx::query_as::<_, Pragma>("SELECT PostGIS_Version();")
            .fetch_one(&pool)
            .await?;
        let v = pargma.0;
        debug!("PostGIS Version = {v}");

        // ensure table exists, has a geometry column and an SRID.
        let row = sqlx::query_as::<_, TGeometryColumn>(FIND_TABLE)
            .bind(table)
            .fetch_one(&pool)
            .await?;
        debug!("geometry_column = {row:?}");
        // ...
        let srid = u32::try_from(row.srid)?;

        // set time zone to UTC...
        let sql = "SET TIME ZONE 'UTC';";
        let safe_sql = AssertSqlSafe(sql);
        sqlx::query(safe_sql).execute(&pool).await?;

        // create collations...
        // ignore case
        let sql = format!(
            r#"CREATE COLLATION IF NOT EXISTS "{CQL2_CI}" (provider = icu, deterministic = false, locale = 'und-u-ks-level2');"#
        );
        let safe_sql = AssertSqlSafe(sql);
        sqlx::query(safe_sql).execute(&pool).await?;
        // ignore accents...
        let sql = format!(
            r#"CREATE COLLATION IF NOT EXISTS "{CQL2_AI}" (provider = icu, deterministic = false, locale = 'und-u-ks-level1-kc-true');"#
        );
        let safe_sql = AssertSqlSafe(sql);
        sqlx::query(safe_sql).execute(&pool).await?;
        // ignore both...
        let sql = format!(
            r#"CREATE COLLATION IF NOT EXISTS "{CQL2_CAI}" (provider = icu, deterministic = false, locale = 'und-u-ks-level1');"#
        );
        let safe_sql = AssertSqlSafe(sql);
        sqlx::query(safe_sql).execute(&pool).await?;

        Ok(Self {
            db_name: db_name.to_owned(),
            table: table.to_owned(),
            pool,
            srid,
        })
    }

    /// Return this pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Return name of the table housing the data.
    pub fn table(&self) -> &str {
        &self.table
    }

    /// Transform given [Expression] to an SQL _WHERE_ clause that can be used
    /// for selecting a subset of this data source items.
    pub fn to_sql(&self, exp: &Expression) -> Result<String, MyError> {
        let mut e = exp.to_inner()?;
        let reduced = E::reduce(&mut e)?;
        self.to_sql_impl(reduced)
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
            E::Id(x) => Ok(double_quoted(x)),
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
            E::Array(x) => {
                let items: Result<Vec<String>, MyError> =
                    x.into_iter().map(|x| self.to_sql_impl(x)).collect();
                let items_ = items?;
                Ok(format!("({})", items_.join(", ")))
            }
            x => unreachable!("{x:?} cannot be transformed to SQL"),
        }
    }

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

/// Render a given string as surrounded by double-quotes unless it already is.
fn double_quoted(s: String) -> String {
    // if already surrounded by double-quotes, return as is...
    if s.starts_with('"') && s.ends_with('"') {
        s
    } else {
        format!("\"{s}\"")
    }
}

fn qstr_to_sql(qs: QString) -> Result<String, MyError> {
    match qs.flags() {
        0 => Ok(format!(r#"'{}' COLLATE "{PG_UNICODE}""#, qs.inner())),
        1 => Ok(format!(r#"'{}' COLLATE "{CQL2_CI}""#, qs.inner())),
        2 => Ok(format!(r#"'{}' COLLATE "{CQL2_AI}""#, qs.inner())),
        3 => Ok(format!(r#"'{}' COLLATE "{CQL2_CAI}""#, qs.inner())),
        x => {
            let msg = format!("String w/ '{x}' flags has NO direct SQL representation");
            Err(MyError::Runtime(msg.into()))
        }
    }
}

/// Macro to generate a concrete [PGDataSource].
///
/// Caller must provide the following parameters:
/// * `$vis`: Visibility specifier of the generated artifacts; e.g. `pub(crate)`.
/// * `$name`: Prefix of the concrete data source structure name to materialize.
///   The final name will have a 'PG' suffix appended; eg. `Foo` -> `FooPG`.
/// * `$db_url`: Database URL to an accessible _PostgreSQL_ DB; e.g.
///   `postgres:user:password@host:port/db_name`
/// * `$table`: Name of the table containing the features' data.
/// * `$feature`: `sqlx` _FromRow_ convertible structure to map database table
///   rows to _Features_.
#[macro_export]
macro_rules! gen_pg_ds {
    ($vis:vis, $name:expr, $db_url:expr, $table:expr, $feature:expr) => {
        ::paste::paste! {
            /// Concrete PostgreSQL+PostGIS source.
            $vis struct [<$name PG>](PGDataSource);

            impl [<$name PG>] {
                /// Constructor.
                $vis async fn new() -> Result<Self, MyError> {
                    let ds = PGDataSource::from($db_url, $table).await?;
                    Ok(Self(ds))
                }

                /// Convert a row (aka Feature) to a generic Resource.
                $vis fn to_resource(r: $feature) -> Result<Resource, Box<dyn Error>> {
                    let row = $feature::try_from(r)?;
                    Ok(Resource::try_from(row)?)
                }

                /// Convenience method. Calls inner's samilarly named method.
                $vis fn table(&self) -> &str {
                    self.0.table()
                }

                /// Return a reference to the inner model data source.
                $vis fn inner(&self) -> &PGDataSource {
                    &self.0
                }
            }

            impl ::core::fmt::Display for [<$name PG>] {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{}PG({})", $name, $table)
                }
            }

            #[::async_trait::async_trait]
            impl StreamableDS for [<$name PG>] {
                type Item = $feature;
                type Err = MyError;

                async fn fetch(
                    &self
                ) -> Result<::futures::stream::BoxStream<'_, Result<$feature, MyError>>, MyError> {
                    let sql = format!("SELECT * FROM {};", $table);
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
                    let sql = format!(r#"SELECT * FROM "{}" WHERE {};"#, self.table(), where_clause);
                    tracing::debug!("-- sql = {sql}");
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
