// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! OGC CQL2 [Queryable][Q] is a token that represents a property of a [Resource][1]
//! that can be used in a [filter expression][2].
//!
//! [1][crate::Resource]
//! [2][crate::Expression]
//!

use crate::{
    MyError,
    bound::Bound,
    geom::{G, GTrait},
    qstring::QString,
};
use core::fmt;
use jiff::{Timestamp, Zoned, civil::Date, tz::TimeZone};
use std::{cmp::Ordering, mem};
use tracing::error;

/// [Queryable][Q] type variants.
#[derive(Debug)]
pub enum DataType {
    /// A Unicode UTF-8 string.
    Str,
    /// A numeric value including integers and floating points.
    Num,
    /// A boolean value.
    Bool,
    /// An _Instant_ with a granularity of a second or smaller. Timestamps are
    /// always in the time zone UTC ("Z").
    Timestamp,
    /// An _Instant_ with a granularity of a day. Dates are local without an
    /// associated time zone.
    Date,
    /// A temporal range of 2 _Instants_ each either _fixed_ or _unbounded_.
    #[allow(dead_code)]
    Interval,
    /// A spatial (geometry) value.
    Geom,
    /// A collection of homogeneous values.
    #[allow(dead_code)]
    List,
}

/// A [`Resource`][crate::Resource] queryable property possible concrete value
/// variants.
#[derive(Clone)]
pub enum Q {
    /// Unknown or undefined w/in the current context.
    Null,
    /// A known boolean value.
    Bool(bool),
    /// A known numeric literal.
    Num(f64),
    /// Either a known UTF8 character string literal, or one that when used in
    /// comparisons, should be used ignoring its case and/or accent(s).
    Str(QString),
    /// A known geometry (spatial) instance.
    Geom(G),
    /// Either a known temporal instant or an unbounded value.
    Instant(Bound),
    /// A temporal interval.
    Interval(Bound, Bound),
    /// A list of other [Queryables][Q].
    List(Vec<Q>),
}

impl fmt::Debug for Q {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "Null"),
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::Num(arg0) => f.debug_tuple("Num").field(arg0).finish(),
            Self::Str(arg0) => f.debug_tuple("Str").field(arg0).finish(),
            Self::Geom(x) => write!(f, "Geom({})", x.to_wkt()),
            Self::Instant(arg0) => f.debug_tuple("Instant").field(arg0).finish(),
            Self::Interval(arg0, arg1) => {
                f.debug_tuple("Interval").field(arg0).field(arg1).finish()
            }
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
        }
    }
}

impl PartialEq for Q {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Num(l0), Self::Num(r0)) => l0 == r0,
            (Self::Str(l0), Self::Str(r0)) => l0 == r0,
            (Self::Geom(l0), Self::Geom(r0)) => l0 == r0,
            (Self::Instant(l0), Self::Instant(r0)) => l0 == r0,
            (Self::Interval(l0, l1), Self::Interval(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            _ => mem::discriminant(self) == mem::discriminant(other),
        }
    }
}

impl Eq for Q {}

impl PartialOrd for Q {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Q::Null, Q::Null) => Some(Ordering::Equal),
            (Q::Bool(a), Q::Bool(b)) => a.partial_cmp(b),
            (Q::Num(a), Q::Num(b)) => a.partial_cmp(b),
            (Q::Str(a), Q::Str(b)) => a.partial_cmp(b),
            (Q::Instant(a), Q::Instant(b)) => a.partial_cmp(b),
            (Q::Interval(a0, a1), Q::Interval(b0, b1)) => match a0.partial_cmp(b0) {
                Some(Ordering::Equal) => match (a1, b1) {
                    (Bound::None, Bound::None) => Some(Ordering::Equal),
                    (Bound::None, _) => Some(Ordering::Greater),
                    (_, Bound::None) => Some(Ordering::Less),
                    _ => a1.partial_cmp(b1),
                },
                x => x,
            },
            (Q::List(a), Q::List(b)) => a.partial_cmp(b),
            // anything else, incl. geometries are incomparable...
            _ => None,
        }
    }
}

impl fmt::Display for Q {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Q::Null => write!(f, "Null"),
            Q::Bool(x) => write!(f, "{x}"),
            Q::Num(x) => write!(f, "{x}"),
            Q::Str(x) => write!(f, "{x}"),
            Q::Geom(x) => write!(f, "{}", x.to_wkt()),
            Q::Instant(x) => write!(f, "{x}"),
            Q::Interval(Bound::None, Bound::None) => write!(f, "[...]"),
            Q::Interval(Bound::None, y) => write!(f, "[..{y}]"),
            Q::Interval(x, Bound::None) => write!(f, "[{x}..]"),
            Q::Interval(x, y) => write!(f, "[{x}..{y}]"),
            Q::List(x) => write!(f, "{x:?}"),
        }
    }
}

impl Q {
    /// Create a new instance as a plain literal string from given argument
    /// **after trimming it**.
    pub fn new_plain_str(value: &str) -> Self {
        Self::Str(QString::plain(value.trim()))
    }

    /// Try creating a new temporal timestamp variant instance from a string of
    /// the form _fullDate_ followed by "T", followed by _utcTime_.
    pub fn try_from_timestamp_str(value: &str) -> Result<Self, MyError> {
        let x = value.parse::<Timestamp>()?;
        let z = x.to_zoned(TimeZone::UTC);
        Ok(Q::Instant(Bound::Timestamp(z)))
    }

    /// Try creating a new temporal timestamp variant instance from a number of
    /// nanoseconds since the Unix epoch.
    pub fn try_from_timestamp_ns(value: i128) -> Result<Self, MyError> {
        let x = Timestamp::from_nanosecond(value)?;
        let z = x.to_zoned(TimeZone::UTC);
        Ok(Q::Instant(Bound::Timestamp(z)))
    }

    /// Try creating a new temporal date variant instance from a _fullDate_
    /// string.
    ///
    /// **IMPORTANT** - CQL2 specs state that dates are to be considered as
    /// local wrt. time zones. This implementation however always assigns a
    /// UTC time zone.
    pub fn try_from_date_str(value: &str) -> Result<Self, MyError> {
        let x = value.parse::<Date>()?;
        let z = x.to_zoned(TimeZone::UTC)?;
        Ok(Q::Instant(Bound::Date(z)))
    }

    /// Try creating a new temporal date variant instance from a number of
    /// nanoseconds since the Unix epoch.
    pub fn try_from_date_ns(value: i128) -> Result<Self, MyError> {
        let x = Timestamp::from_nanosecond(value)?;
        let z = x.to_zoned(TimeZone::UTC);
        Ok(Q::Instant(Bound::Date(z)))
    }

    /// Try creating a new instance from a Well Known Text encoded geometry.
    pub fn try_from_wkt(value: &str) -> Result<Self, MyError> {
        let g = G::try_from(value)?;
        Ok(Q::Geom(g))
    }

    /// Try creating a new instance from a Well Known Binary encoded geometry.
    pub fn try_from_wkb(value: &[u8]) -> Result<Self, MyError> {
        let g = G::try_from(value)?;
        Ok(Q::Geom(g))
    }

    /// Return TRUE if this is `Null`; FALSE otherwise.
    pub(crate) fn is_null(&self) -> bool {
        matches!(self, Q::Null)
    }

    /// Return TRUE if this is a temporal value; FALSE otherwise.
    pub(crate) fn is_instant(&self) -> bool {
        matches!(self, Q::Instant(_))
    }

    /// Return the current value of this if it's a boolean value.
    pub fn to_bool(&self) -> Result<bool, MyError> {
        match self {
            Q::Bool(x) => Ok(*x),
            _ => Err(MyError::Runtime(format!("{self} is not a boolean").into())),
        }
    }

    /// Return the current value of this if it's a [string][QString] value.
    pub fn to_str(&self) -> Result<QString, MyError> {
        match self {
            Q::Str(x) => Ok(x.to_owned()),
            _ => Err(MyError::Runtime(format!("{self} is not a string").into())),
        }
    }

    /// Return the current value of this if it's a number value.
    pub fn to_num(&self) -> Result<f64, MyError> {
        match self {
            Q::Num(x) => Ok(*x),
            _ => Err(MyError::Runtime(format!("{self} is not a number").into())),
        }
    }

    /// Return the current value of this if it's a [Geometry][G] value.
    pub fn to_geom(&self) -> Result<G, MyError> {
        match self {
            Q::Geom(x) => Ok(x.to_owned()),
            _ => Err(MyError::Runtime(format!("{self} is not a geometry").into())),
        }
    }

    /// Return the current value of this if it's a [Bound] value.
    pub fn to_bound(&self) -> Result<Bound, MyError> {
        match self {
            Q::Instant(x) => Ok(x.to_owned()),
            _ => Err(MyError::Runtime(
                format!("{self} is not a bounded instant").into(),
            )),
        }
    }

    /// Return the current value of this if it's a _Interval_ value as a pair
    /// of [Bound]s.
    pub fn to_interval(&self) -> Result<(Bound, Bound), MyError> {
        match self {
            Q::Interval(x, y) => Ok((x.to_owned(), y.to_owned())),
            _ => Err(MyError::Runtime(
                format!("{self} is not an interval").into(),
            )),
        }
    }

    /// Return the current value of this if it's a collection.
    pub fn to_list(&self) -> Result<Vec<Q>, MyError> {
        match self {
            Q::List(x) => Ok(x.to_owned()),
            _ => Err(MyError::Runtime(format!("{self} is not a list").into())),
        }
    }

    /// Return TRUE if both arguments are of the same type; FALSE otherwise.
    pub(crate) fn same_type(this: &Self, that: &Self) -> bool {
        mem::discriminant(this) == mem::discriminant(that)
    }

    // Return the optional literal data type of this.
    pub(crate) fn literal_type(&self) -> Option<DataType> {
        match self {
            Q::Bool(_) => Some(DataType::Bool),
            Q::Num(_) => Some(DataType::Num),
            Q::Str(_) => Some(DataType::Str),
            Q::Geom(_) => Some(DataType::Geom),
            Q::Instant(x) => match x {
                Bound::None => None,
                Bound::Date(_) => Some(DataType::Date),
                Bound::Timestamp(_) => Some(DataType::Timestamp),
            },
            // Q::Null | Q::Interval(_, _) | Q::List(qs)
            _ => None,
        }
    }

    pub(crate) fn contained_by(&self, list: Vec<Self>) -> Result<bool, MyError> {
        if list.is_empty() {
            return Ok(false);
        }

        if let Some(z_type) = self.literal_type() {
            if matches!(z_type, DataType::Bool) {
                let lhs = self.to_bool()?;
                let rhs: Result<Vec<bool>, MyError> = list.iter().map(|e| e.to_bool()).collect();
                let rhs = rhs?;
                Ok(rhs.contains(&lhs))
            } else if matches!(z_type, DataType::Num) {
                let lhs = self.to_num()?;
                let rhs: Result<Vec<f64>, MyError> = list.iter().map(|e| e.to_num()).collect();
                let rhs = rhs?;
                Ok(rhs.contains(&lhs))
            } else if matches!(z_type, DataType::Str) {
                let lhs = &self.to_str()?;
                let rhs: Result<Vec<QString>, MyError> = list.iter().map(|e| e.to_str()).collect();
                let rhs = rhs?;
                Ok(rhs.contains(lhs))
            } else if matches!(z_type, DataType::Date) || matches!(z_type, DataType::Timestamp) {
                let lhs = self.to_bound()?.as_zoned().unwrap();
                let rhs: Result<Vec<Zoned>, MyError> = list
                    .iter()
                    .map(|e| e.to_bound().and_then(|b| b.to_zoned()))
                    .collect();
                let rhs = rhs?;
                Ok(rhs.contains(&lhs))
            } else if matches!(z_type, DataType::Geom) {
                let lhs = self.to_geom()?;
                let rhs: Result<Vec<G>, MyError> = list.iter().map(|e| e.to_geom()).collect();
                let rhs = rhs?;
                Ok(rhs.contains(&lhs))
            } else {
                error!("Failed. self = {self:?}; list = {list:?}");
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}

impl From<bool> for Q {
    fn from(value: bool) -> Self {
        Q::Bool(value)
    }
}

// logic to convert integers, both signed and unsigned to E::Num...

trait TryToF64<T> {
    fn try_to_f64(self) -> Result<f64, MyError>;
}

// Implement trait for small integer types that involve safe conversion.
macro_rules! impl_safe_try_to_f64 {
    ($($t:ty),*) => {
        $(
            impl TryToF64<$t> for $t {
                fn try_to_f64(self) -> Result<f64, $crate::MyError> {
                    Ok(self as f64)
                }
            }
        )*
    };
}

// rinse + repeat...
impl_safe_try_to_f64!(u8, u16, u32, i8, i16, i32);

// Implement trait for unsigned integer types that may result in precision
// loss.
//
// Constant values used for ensuring safe conversion of integers to `f64` are
// based on the fact that in Rust `f64` numbers have a `52` bit long mantissa,
// which implies that integers with abstract values bit-length greater than
// `52` will not be accurately cast.
macro_rules! impl_try_unsigned_to_f64 {
    ($($t:ty),*) => {
        $(
            impl TryToF64<$t> for $t {
                fn try_to_f64(self) -> Result<f64, $crate::MyError> {
                    const MAX_LIMIT: $t = (1 << 53) - 1;

                    if self <= MAX_LIMIT {
                        Ok(self as f64)
                    } else {
                        Err(MyError::PrecisionLoss(self.to_string().into()))
                    }
                }
            }
        )*
    };
}

impl_try_unsigned_to_f64!(u64, u128);

// Implement trait for signed integer types that may result in precision loss.
macro_rules! impl_try_signed_to_f64 {
    ($($t:ty),*) => {
        $(
            impl TryToF64<$t> for $t {
                fn try_to_f64(self) -> Result<f64, $crate::MyError> {
                    const MAX_LIMIT: $t = (1 << 53) - 1;
                    // const MIN_LIMIT: $t = -((1 << 53) - 1);
                    const MIN_LIMIT: $t = - MAX_LIMIT;

                    if (MIN_LIMIT..=MAX_LIMIT).contains(&self) {
                        Ok(self as f64)
                    } else {
                        Err(MyError::PrecisionLoss(self.to_string().into()))
                    }
                }
            }
        )*
    };
}

impl_try_signed_to_f64!(i64, i128);

// special cases for both usize and isize to cater for platform-specific
// bit-length of those types.

impl TryToF64<usize> for usize {
    fn try_to_f64(self) -> Result<f64, MyError> {
        match usize::BITS {
            32 => (self as u32).try_to_f64(),
            _ => (self as u64).try_to_f64(),
        }
    }
}

impl TryToF64<isize> for isize {
    fn try_to_f64(self) -> Result<f64, MyError> {
        match isize::BITS {
            32 => (self as i32).try_to_f64(),
            _ => (self as i64).try_to_f64(),
        }
    }
}

// generate TryFrom<x> implementetation to Q...
macro_rules! impl_try_from_int {
    ($($t:ty),*) => {
        $(
            impl TryFrom<$t> for $crate::Q {
                type Error = MyError;
                fn try_from(value: $t) -> Result<Self, $crate::MyError> {
                    let x = value.try_to_f64()?;
                    Ok(Q::Num(x))
                }
            }
        )*
    };
}

impl_try_from_int!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl From<f64> for Q {
    fn from(value: f64) -> Self {
        Q::Num(value)
    }
}

impl TryFrom<Date> for Q {
    type Error = MyError;

    fn try_from(value: Date) -> Result<Self, Self::Error> {
        let z = value.to_zoned(TimeZone::UTC)?;
        Ok(Q::Instant(Bound::Date(z)))
    }
}

impl From<Timestamp> for Q {
    fn from(value: Timestamp) -> Self {
        let z = value.to_zoned(TimeZone::UTC);
        Q::Instant(Bound::Timestamp(z))
    }
}

impl From<Bound> for Q {
    fn from(value: Bound) -> Self {
        Q::Instant(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_usize_max() {
        let x = usize::MAX.try_to_f64();
        assert!(matches!(x.err(), Some(MyError::PrecisionLoss(_))))
    }

    #[test]
    fn test_usize() {
        let x: usize = (1 << 53) - 1;
        let y1 = x as f64;
        let y2 = x.try_to_f64().expect("Failed");
        assert_eq!(y1, y2)
    }

    #[test]
    fn test_u128_max() {
        let x = u128::MAX.try_to_f64();
        assert!(matches!(x.err(), Some(MyError::PrecisionLoss(_))))
    }

    #[test]
    fn test_u128() {
        let x: u128 = (1 << 53) - 1;
        let y1 = x as f64;
        let y2 = x.try_to_f64().expect("Failed");
        assert_eq!(y1, y2)
    }

    #[test]
    fn test_u64_max() {
        let x = u64::MAX.try_to_f64();
        assert!(matches!(x.err(), Some(MyError::PrecisionLoss(_))))
    }

    #[test]
    fn test_u64() {
        let x: u64 = (1 << 53) - 1;
        let y1 = x as f64;
        let y2 = x.try_to_f64().expect("Failed");
        assert_eq!(y1, y2)
    }

    #[test]
    fn test_isize_max() {
        let x = isize::MAX.try_to_f64();
        assert!(matches!(x.err(), Some(MyError::PrecisionLoss(_))))
    }

    #[test]
    fn test_isize() {
        let x1: isize = (1 << 53) - 1;
        let y1 = x1 as f64;
        let y2 = x1.try_to_f64().expect("Failed");
        assert_eq!(y1, y2);

        let x2 = -x1;
        let y1 = x2 as f64;
        let y2 = x2.try_to_f64().expect("Failed");
        assert_eq!(y1, y2)
    }

    #[test]
    fn test_isize_min() {
        let x = isize::MIN.try_to_f64();
        assert!(matches!(x.err(), Some(MyError::PrecisionLoss(_))))
    }

    #[test]
    fn test_i128_max() {
        let x = i128::MAX.try_to_f64();
        assert!(matches!(x.err(), Some(MyError::PrecisionLoss(_))))
    }

    #[test]
    fn test_i128() {
        let x1: i128 = (1 << 53) - 1;
        let y1 = x1 as f64;
        let y2 = x1.try_to_f64().expect("Failed");
        assert_eq!(y1, y2);

        let x2 = -x1;
        let y1 = x2 as f64;
        let y2 = x2.try_to_f64().expect("Failed");
        assert_eq!(y1, y2)
    }

    #[test]
    fn test_i128_min() {
        let x = i128::MIN.try_to_f64();
        assert!(matches!(x.err(), Some(MyError::PrecisionLoss(_))))
    }

    #[test]
    fn test_i64_max() {
        let x = i64::MAX.try_to_f64();
        assert!(matches!(x.err(), Some(MyError::PrecisionLoss(_))))
    }

    #[test]
    fn test_i64_min() {
        let x = i64::MIN.try_to_f64();
        assert!(matches!(x.err(), Some(MyError::PrecisionLoss(_))))
    }

    #[test]
    fn test_i64() {
        let x1: i64 = (1 << 53) - 1;
        let y1 = x1 as f64;
        let y2 = x1.try_to_f64().expect("Failed");
        assert_eq!(y1, y2);

        let x2 = -x1;
        let y1 = x2 as f64;
        let y2 = x2.try_to_f64().expect("Failed");
        assert_eq!(y1, y2)
    }

    #[test]
    fn fuzz_test_i64() {
        const LIMIT: i64 = (1 << 53) - 1;

        fn random_i64() -> i64 {
            let mut rng = rand::rng();
            match rng.random_bool(0.5) {
                true => LIMIT - rng.random_range(1..=LIMIT.abs()),
                false => LIMIT + rng.random_range(1..=LIMIT.abs()),
            }
        }

        let mut expected = 0;
        let mut actual = 0;
        for _ in 0..1000 {
            let x = random_i64();
            if !(-LIMIT..=LIMIT).contains(&x) {
                expected += 1;
            }
            // else as f64 is ok...
            match Q::try_from(x) {
                Ok(_) => (), // cool
                Err(MyError::PrecisionLoss(_)) => actual += 1,
                Err(x) => panic!("Unexpected {x}"),
            }
        }

        assert_eq!(expected, actual)
    }

    #[test]
    fn fuzz_test_u64() {
        const LIMIT: u64 = (1 << 53) - 1;

        fn random_u64() -> u64 {
            let mut rng = rand::rng();
            match rng.random_bool(0.5) {
                true => LIMIT.saturating_sub(rng.random_range(1..=LIMIT)),
                false => LIMIT + rng.random_range(1..=LIMIT),
            }
        }

        let mut expected = 0;
        let mut actual = 0;
        for _ in 0..1000 {
            let x = random_u64();
            if x > LIMIT {
                expected += 1;
            }
            // else as f64 is ok...
            match Q::try_from(x) {
                Ok(_) => (), // cool
                Err(MyError::PrecisionLoss(_)) => actual += 1,
                Err(x) => panic!("Unexpected {x}"),
            }
        }

        assert_eq!(expected, actual)
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_like() {
        // plain input and pattern.  no wildcards...
        let input = QString::plain("hello");
        let pattern = QString::plain("h%o");
        let r1 = QString::like(&input, &pattern);
        assert!(r1);

        // case-insensitive input, plain pattern.  multi wildcard...
        let input = QString::icase("HELLO");
        let pattern = QString::plain("h%o");
        let r2 = QString::like(&input, &pattern);
        assert!(r2);
        let input = QString::icase("HELLODOLLY");
        let pattern = QString::plain("h%odo%y");
        let r2p = QString::like(&input, &pattern);
        assert!(r2p);

        // plain input, case-insensitive pattern.  single wildcard...
        let input = QString::plain("hello");
        let pattern = QString::icase("h__lo");
        let r3 = QString::like(&input, &pattern);
        assert!(r3);
        // multi wildcard...
        let pattern = QString::icase("h%lo");
        let r3p = QString::like(&input, &pattern);
        assert!(r3p);

        // plain input and pattern.  escaped multi wildcard...
        let input = QString::plain("hello");
        let pattern = QString::plain("h\\%o");
        let r4 = QString::like(&input, &pattern);
        assert!(!r4);

        let input = QString::plain("h%llo");
        let pattern = QString::plain("h\\%llo");
        let r5 = QString::like(&input, &pattern);
        assert!(r5);

        // empty input and multi wildcard pattern should match
        let input = QString::plain("");
        let pattern = QString::plain("%");
        let r6 = QString::like(&input, &pattern);
        assert!(r6);

        // non-empty input and empty pattern should fail
        let input = QString::plain("abc");
        let pattern = QString::plain("");
        let r7 = QString::like(&input, &pattern);
        assert!(!r7);

        // w/ unicode... case-insensitive input and no wildcards...

        let input = QString::icase("ß"); // small sharp s
        let pattern = QString::icase("ẞ"); // capital sharp s
        let u1 = QString::like(&input, &pattern);
        assert!(u1);

        let input = QString::icase("Σ");
        let pattern = QString::plain("σ");
        let u2 = QString::like(&input, &pattern);
        assert!(u2);

        // unicase bug?  Turkish dotted i
        // let input = QString::plain("İ"); // capital dotted I
        // let pattern = QString::icase("i"); // small dotted i
        // let u3 = QString::like(&input, &pattern);
        // assert!(u3);

        // w/ unicode + wildcard...

        let input = QString::plain("こんにちは");
        let pattern = QString::plain("こ%は");
        let u4 = QString::like(&input, &pattern);
        assert!(u4);

        let pattern = QString::icase("こ_にちは");
        let u5 = QString::like(&input, &pattern);
        assert!(u5);
    }
}
