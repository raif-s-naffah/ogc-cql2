// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Enough code to ease integration of PostgreSQL + PostGIS types w/ `sqlx`.
//!

use crate::{G, wkb::PostGisBinary};
use jiff::{
    Span, Zoned,
    civil::{Date, DateTime},
    fmt::temporal::DateTimeParser,
    tz::TimeZone,
};
use sqlx::{
    Decode, Postgres, Type,
    error::BoxDynError,
    postgres::{PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef},
};

static PARSER: DateTimeParser = DateTimeParser::new();

// ===== PostGIS geometry type ================================================

impl Type<Postgres> for G {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("geometry")
    }
}

impl PgHasArrayType for G {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_geometry")
    }
}

impl<'r> Decode<'r, Postgres> for G {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let bytes = value.as_bytes()?;
                let ewkb = PostGisBinary::try_from(bytes)?;
                Ok(ewkb.geom())
            }
            PgValueFormat::Text => Err("Failed decoding a PostGIS geometry column".into()),
        }
    }
}

// ===== PostgreSQL DATE type =================================================

/// Our representation of a PostgreSQL DATE type to ease w/ sqlx and jiff.
#[derive(Debug)]
pub struct PgDate(pub Date);

impl Type<Postgres> for PgDate {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("date")
    }
}

impl PgHasArrayType for PgDate {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_date")
    }
}

impl<'r> Decode<'r, Postgres> for PgDate {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // DATE is encoded as number of days since 2000-01-01...
                let days: i32 = Decode::<Postgres>::decode(value)?;
                let span = Span::new().try_days(days)?;
                let pg_epoch = Date::new(2000, 1, 1)?;
                PgDate(pg_epoch.checked_add(span)?)
            }
            PgValueFormat::Text => {
                let it = PARSER.parse_date(value.as_str()?)?;
                PgDate(it)
            }
        })
    }
}

// ===== PostgreSQL TIMESTAMPTZ type ==========================================

/// Our representation of a PostgreSQL TIMESTAMP type to use w/ sqlx and jiff.
#[derive(Debug)]
pub struct PgTimestamp(pub Zoned);

impl Type<Postgres> for PgTimestamp {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("timestamp")
    }
}

impl PgHasArrayType for PgTimestamp {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_timestamp")
    }
}

impl<'r> Decode<'r, Postgres> for PgTimestamp {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIMESTAMP is encoded as number of microseconds since 2000-01-01...
                let micro_secs: i64 = Decode::<Postgres>::decode(value)?;
                let span = Span::new().try_microseconds(micro_secs)?;
                let pg_epoch = DateTime::new(2000, 1, 1, 0, 0, 0, 0)?;
                let z = pg_epoch.checked_add(span)?.to_zoned(TimeZone::UTC)?;
                PgTimestamp(z)
            }
            PgValueFormat::Text => {
                let it = format!("{}Z", value.as_str()?).parse()?;
                PgTimestamp(it)
            }
        })
    }
}
