// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! In OGC CQL2 a temporal _Bound_ is a value that is either bounded or not used
//! either on its own as an _Instant_ or a _limit_ in an _Interval_.
//!

use crate::{MyError, Q};
use core::fmt;
use jiff::{Timestamp, Zoned, civil::Date, tz::TimeZone};
use std::{cmp::Ordering, mem};

/// Possible variants of a CQL2 _Instant_ and _Interval_ limit.
#[derive(Debug, Clone)]
pub enum Bound {
    /// Unbounded temporal value used as lower, upper, or both limit(s);
    /// represented by the string `'..'` .
    None,
    /// Instant with a 1-day granularity, in UTC time-zone.
    Date(Zoned),
    /// Instant with a 1-second or less granularity in UTC time-zone.
    Timestamp(Zoned),
}

impl PartialEq for Bound {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Bound::Date(x), Bound::Date(y))
            | (Bound::Date(x), Bound::Timestamp(y))
            | (Bound::Timestamp(x), Bound::Date(y))
            | (Bound::Timestamp(x), Bound::Timestamp(y)) => x == y,
            _ => mem::discriminant(self) == mem::discriminant(other),
        }
    }
}

impl Eq for Bound {}

impl PartialOrd for Bound {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // if both are unbounded, the result is always equal.
            (Bound::None, Bound::None) => Some(Ordering::Equal),
            // if the LHS is unbounded and the RHS is not then the result is...
            (Bound::None, _) => Some(Ordering::Less),
            // and the opposite is true if it's the other way around...
            (_, Bound::None) => Some(Ordering::Greater),
            // if they're both bounded instants of the same type...
            (Bound::Date(z1), Bound::Date(z2)) | (Bound::Timestamp(z1), Bound::Timestamp(z2)) => {
                z1.partial_cmp(z2)
            }
            // IMPORTANT (rsn) 202511-19 - just make sure they're date/time based;
            // otherwise we may run into stack overflow
            (Bound::Date(z1), Bound::Timestamp(z2)) | (Bound::Timestamp(z1), Bound::Date(z2)) => {
                z1.partial_cmp(z2)
            }
        }
    }
}

impl fmt::Display for Bound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bound::None => write!(f, ".."),
            Bound::Date(x) => write!(f, "{x}/d"),
            Bound::Timestamp(x) => write!(f, "{x}/t"),
        }
    }
}

impl TryFrom<Q> for Bound {
    type Error = MyError;

    fn try_from(value: Q) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&Q> for Bound {
    type Error = MyError;

    fn try_from(value: &Q) -> Result<Self, Self::Error> {
        match value {
            Q::Str(x) => {
                let s = x.as_str();
                match s {
                    "'..'" => Ok(Bound::None),
                    _ => Err(MyError::Runtime(
                        "Only '..' string is allowed for interval bounds".into(),
                    )),
                }
            }
            Q::Instant(x) => Ok(x.to_owned()),
            _ => Err(MyError::Runtime("Expected a zoned timestamp | '..'".into())),
        }
    }
}

impl Bound {
    /// Try creating a new Bound::Date variant from a well-formed RFC-3339
    /// date string. Return [MyError] if an error occurs.
    pub fn try_new_date(s: &str) -> Result<Self, MyError> {
        let d = s.parse::<Date>()?;
        let z = d.to_zoned(TimeZone::UTC)?;
        Ok(Bound::Date(z))
    }

    /// Try creating a new Bound::Timestamp variant from a well-formed RFC-3339
    /// timestamp string. Return [MyError] if an error occurs.
    pub fn try_new_timestamp(s: &str) -> Result<Self, MyError> {
        let d = s.parse::<Timestamp>()?;
        let z = d.to_zoned(TimeZone::UTC);
        Ok(Bound::Timestamp(z))
    }

    /// Return inner value if it was a bounded instant.
    pub(crate) fn to_zoned(&self) -> Result<Zoned, MyError> {
        match self {
            Bound::Date(z) => Ok(z.to_owned()),
            Bound::Timestamp(z) => Ok(z.to_owned()),
            _ => Err(MyError::Runtime(
                format!("{self} is not a bounded instant").into(),
            )),
        }
    }

    // Return the inner value in `Some` if this is not the unbound variant.
    // Return `None` otherwise.
    pub(crate) fn as_zoned(&self) -> Option<Zoned> {
        match self {
            Bound::Date(x) => Some(x.to_owned()),
            Bound::Timestamp(x) => Some(x.to_owned()),
            Bound::None => None,
        }
    }

    // Return TRUE if this is an unbound variant, FALSE otherwise.
    #[cfg(test)]
    pub(crate) fn is_unbound(&self) -> bool {
        matches!(self, Bound::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bound() {
        const D: &str = "2015-01-01";
        const T: &str = "2015-01-01T00:00:00Z";

        let d = Bound::try_new_date(D);
        assert!(d.is_ok());
        let b1 = d.unwrap();
        assert!(!b1.is_unbound());
        let b1_ = b1.as_zoned();
        assert!(b1_.is_some());
        let z1 = b1_.unwrap();

        let t = Bound::try_new_timestamp(T);
        assert!(t.is_ok());
        let b2 = t.unwrap();
        assert!(!b2.is_unbound());
        let b2_ = b2.as_zoned();
        assert!(b2_.is_some());
        let z2 = b2_.unwrap();

        assert_eq!(z1, z2);
        assert!(z1 == z2);
    }
}
