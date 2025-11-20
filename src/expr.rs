// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! OGC CQL2 Text-encoded expression types used by the PEG parser.
//!

use crate::{
    Bound, Context, ExtDataType, MyError, Q, Resource,
    geom::{G, GTrait},
    op::Op,
    qstring::QString,
};
use core::fmt;
use jiff::Zoned;
use std::{any::Any, mem};
use tracing::{debug, error};

/// Expression variants...
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum E {
    #[default]
    Null,

    Unbounded,
    Bool(bool),
    Num(f64),
    Str(QString),
    Date(Zoned),
    Timestamp(Zoned),
    Spatial(G),
    Id(String),
    Monadic(Op, Box<E>),
    Dyadic(Op, Box<E>, Box<E>),
    Function(Call),
    Array(Vec<E>),
    Interval(Box<E>, Box<E>),
}

impl fmt::Display for E {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            E::Null => write!(f, "NULL"),
            E::Unbounded => write!(f, ".."),
            E::Bool(x) => write!(f, "{}", if *x { "TRUE" } else { "FALSE" }),
            E::Num(x) => write!(f, "{x}"),
            E::Str(x) => write!(f, "'{x}'"),
            E::Date(x) => write!(f, "{}", x.date()),
            E::Timestamp(x) => write!(f, "{}", x.datetime()),
            E::Spatial(x) => write!(f, "{x}"),
            E::Id(x) => write!(f, "{x}"),
            E::Monadic(op, x) if op.nullable() => write!(f, "{x} {op}"),
            E::Monadic(op, x) => write!(f, "{op}({x})"),
            E::Dyadic(op, a, b)
                if matches!(op, Op::IsBetween) || matches!(op, Op::IsNotBetween) =>
            {
                // RHS of [NOT] BETWEEN is now an array of 2 numeric expressions...
                match &**b {
                    E::Array(x) => {
                        let lo = &x[0];
                        let hi = &x[1];
                        write!(f, "{a} {op} {lo} AND {hi}")
                    }
                    _ => panic!("[NOT] BETWEEN's RHS expression is not an array :("),
                }
            }
            E::Dyadic(op, a, b) if op.array() || op.spatial() || op.temporal() => {
                write!(f, "{op}({a}, {b})")
            }
            E::Dyadic(op, a, b) => {
                // if 'b' is a literal, use as is; otherwise surround w/ parens...
                if b.is_literal() {
                    write!(f, "{a} {op} {b}")
                } else {
                    write!(f, "{a} {op} ({b})")
                }
            }
            E::Function(x) => {
                let params: Vec<_> = x.params.iter().map(|x| x.to_string()).collect();
                write!(f, "{}({})", x.name, params.join(", "))
            }
            E::Array(x) => {
                let items: Vec<_> = x.iter().map(|x| x.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            E::Interval(x, y) => write!(f, "[{x} .. {y}]"),
        }
    }
}

impl E {
    /// Return TRUE if this is a literal value; FALSE otherwise.
    pub(crate) fn is_literal(&self) -> bool {
        match self {
            E::Bool(_) | E::Num(_) | E::Str(_) | E::Date(_) | E::Timestamp(_) | E::Spatial(_) => {
                true
            }
            E::Array(x) => x.iter().all(|y| y.is_literal()),
            _ => false,
        }
    }

    // Return TRUE if this is an Identifier; FALSE otherwise.
    pub(crate) fn is_id(&self) -> bool {
        matches!(self, E::Id(_))
    }

    // Return TRUE if it's a literal value or just a property name.
    pub(crate) fn is_literal_or_id(&self) -> bool {
        self.is_literal() || self.is_id()
    }

    // Return TRUE if this is an Interval; FALSE otherwise.
    pub(crate) fn is_interval(&self) -> bool {
        matches!(self, E::Interval(_, _))
    }

    // Return this expression's value as `Some<Q>` if it's indeed a terminal;
    // `None` otherwise.
    pub(crate) fn as_literal(&self) -> Option<Q> {
        match self {
            E::Null => Some(Q::Null),
            E::Unbounded => Some(Q::Instant(Bound::None)),
            E::Bool(x) => Some(Q::Bool(*x)),
            E::Num(x) => Some(Q::Num(*x)),
            E::Str(s) => Some(Q::Str(s.to_owned())),
            E::Date(z) => Some(Q::Instant(Bound::Date(z.to_owned()))),
            E::Timestamp(z) => Some(Q::Instant(Bound::Timestamp(z.to_owned()))),
            E::Spatial(g) => Some(Q::Geom(g.to_owned())),
            _ => None,
        }
    }

    pub(crate) fn as_id(&self) -> Option<&str> {
        match self {
            E::Id(x) => Some(x),
            _ => None,
        }
    }

    // pub(crate) fn as_interval(&self) -> Option<(&E, &E)> {
    pub(crate) fn as_interval(&self) -> Option<(E, E)> {
        match self {
            E::Interval(x, y) => {
                let mut xx = *x.clone();
                let mut yy = *y.clone();
                let x_ = ::std::mem::take(&mut xx);
                let y_ = ::std::mem::take(&mut yy);
                Some((x_, y_))
            }
            _ => None,
        }
    }

    // Possible outcome values when evaluating an [Expression] against an
    // individual _Resource_ from a collection.
    //
    // From [OGC CQL2][1]:
    // > _Each resource instance in the source collection is evaluated against
    // > a filtering expression. The net effect of evaluating a filter
    // > [Expression] is a subset of resources that satisfy the predicate(s)
    // > in the [Expression]._
    //
    // Logically connected predicates are evaluated according to the following
    // truth table, where `T` is TRUE, `F` is FALSE and `N` is NULL.
    // ```text
    // +-----+-----+---------+---------+
    // | P1  | P2  | P1 & P2 | P1 | P2 |
    // +-----+-----+---------+---------+
    // |  T  |  T  |    T    |    T    |
    // |  T  |  F  |    F    |    T    |
    // |  F  |  T  |    F    |    T    |
    // |  F  |  F  |    F    |    F    |
    // |  T  |  N  |    N    |    T    | <-- IMPORTANT
    // |  F  |  N  |    F    |    N    |        |
    // |  N  |  T  |    N    |    T    |        |
    // |  N  |  F  |    F    |    N    |        |
    // |  N  |  N  |    N    |    N    | <------|
    // +-----+-----+---------+---------+
    // ```
    // [1]: https://docs.ogc.org/is/21-065r2/21-065r2.html
    // #[tracing::instrument(level = "trace", skip(ctx, f), ret)]
    pub(crate) fn eval(&self, ctx: &Context, feature: &Resource) -> Result<Q, MyError> {
        match self {
            E::Null => Ok(Q::Null),
            E::Unbounded => Ok(Q::Instant(Bound::None)),
            E::Bool(x) => Ok(Q::Bool(*x)),
            E::Num(x) => Ok(Q::Num(*x)),
            E::Str(x) => Ok(Q::Str(x.to_owned())),
            E::Date(x) => Ok(Q::Instant(Bound::Date(x.to_owned()))),
            E::Timestamp(x) => Ok(Q::Instant(Bound::Timestamp(x.to_owned()))),
            E::Spatial(x) => {
                // ensure geometry has valid coordinates w/in the configured
                // CRS's area-of-use...
                x.check_coordinates(ctx.crs())?;
                Ok(Q::Geom(x.to_owned()))
            }
            E::Id(x) => match feature.get(x) {
                Some(y) => Ok(y.clone()),
                None => {
                    // from <https://docs.ogc.org/is/21-065r2/21-065r2.html#basic-cql2_property>:
                    // "The property name reference SHALL evaluate to its
                    // corresponding value, or NULL if unset.
                    // NOTE (rsn) 20250719 - Text-encoded representation stores
                    // quoted identifiers even when they were not originally
                    if x.starts_with('"') && x.ends_with('"') {
                        let x_ = &x[1..x.len() - 1];
                        if let Some(y) = feature.get(x_) {
                            return Ok(y.clone());
                        } // else it's not in resource...
                    }
                    debug!("No queryable named '{x}' in Resource");
                    Ok(Q::Null)
                }
            },
            E::Monadic(op, x) => {
                let zx = x.eval(ctx, feature)?;
                match op {
                    Op::Minus => {
                        // x, when not Null, should resolve to a number...
                        match zx {
                            Q::Null => Ok(Q::Null),
                            Q::Num(x) => Ok(Q::Num(-x)),
                            _ => Err(MyError::Runtime(
                                format!("Expected a number: {zx:?}. Abort").into(),
                            )),
                        }
                    }
                    Op::Neg => {
                        // x, when not Null, should resolve to a boolean...
                        match zx {
                            Q::Null => Ok(Q::Null),
                            Q::Bool(x) => Ok(Q::Bool(!x)),
                            _ => Err(MyError::Runtime(
                                format!("Expected a boolean: {zx:?}. Abort").into(),
                            )),
                        }
                    }
                    Op::CaseI => match zx {
                        Q::Str(s) => Ok(Q::Str(s.and_icase())),
                        _ => Err(MyError::Runtime(
                            format!("Expected a string: {zx:?}. Abort").into(),
                        )),
                    },
                    Op::AccentI => match zx {
                        Q::Str(s) => Ok(Q::Str(s.and_iaccent())),
                        _ => Err(MyError::Runtime(
                            format!("Expected a string: {zx:?}. Abort").into(),
                        )),
                    },
                    Op::IsNull => match zx {
                        Q::Null => Ok(Q::Bool(true)),
                        _ => Ok(Q::Bool(false)),
                    },
                    Op::IsNotNull => match zx {
                        Q::Null => Ok(Q::Bool(false)),
                        _ => Ok(Q::Bool(true)),
                    },
                    _ => Err(MyError::Runtime(
                        format!("Unexpected ({op:?} w/ {zx:?}). Abort").into(),
                    )),
                }
            }
            // see truth table here...
            // https://docs.ogc.org/is/21-065r2/21-065r2.html#basic-cql2_filter-expression
            #[rustfmt::skip]
            E::Dyadic(Op::And, x, y) => {
                let zx = x.eval(ctx, feature)?;
                let zy = y.eval(ctx, feature)?;
                match (&zx, &zy) {
                    (Q::Bool(true), Q::Bool(true))   => Ok(Q::Bool(true)),
                    (Q::Bool(true), Q::Bool(false))  => Ok(Q::Bool(false)),
                    (Q::Bool(false), Q::Bool(true))  => Ok(Q::Bool(false)),
                    (Q::Bool(false), Q::Bool(false)) => Ok(Q::Bool(false)),
                    (Q::Bool(true), Q::Null)         => Ok(Q::Null),
                    (Q::Bool(false), Q::Null)        => Ok(Q::Bool(false)),
                    (Q::Null, Q::Bool(true))         => Ok(Q::Null),
                    (Q::Null, Q::Bool(false))        => Ok(Q::Bool(false)),
                    (Q::Null, Q::Null)               => Ok(Q::Null),
                    _ => Err(MyError::Runtime(
                        format!("Unexpected AND operands: {zx:?}, {zy:?}. Abort").into(),
                    )),
                }
            }
            #[rustfmt::skip]
            E::Dyadic(Op::Or, x, y) => {
                let zx = x.eval(ctx, feature)?;
                let zy = y.eval(ctx, feature)?;
                match (&zx, &zy) {
                    (Q::Bool(true), Q::Bool(true))    => Ok(Q::Bool(true)),
                    (Q::Bool(true), Q::Bool(false))   => Ok(Q::Bool(true)),
                    (Q::Bool(false), Q::Bool(true))   => Ok(Q::Bool(true)),
                    (Q::Bool(false), Q::Bool(false))  => Ok(Q::Bool(false)),
                    (Q::Bool(true), Q::Null)          => Ok(Q::Bool(true)),
                    (Q::Bool(false), Q::Null)         => Ok(Q::Null),
                    (Q::Null, Q::Bool(true))          => Ok(Q::Bool(true)),
                    (Q::Null, Q::Bool(false))         => Ok(Q::Null),
                    (Q::Null, Q::Null)                => Ok(Q::Null),
                    _ => Err(MyError::Runtime(
                        format!("Unexpected OR operands: {zx:?}, {zy:?}. Abort").into(),
                    )),
                }
            }
            E::Dyadic(op, x, y) if op.comparison() => {
                let zx = x.eval(ctx, feature)?;
                let zy = y.eval(ctx, feature)?;
                // from Requirement #3C
                // https://docs.ogc.org/is/21-065r2/21-065r2.html#basic-cql2_comparison-predicates
                // "If either scalar expression (rule scalarExpression) of the
                // predicate is NULL then the predicate SHALL evaluate to the
                // value NULL;"
                if zx.is_null() || zy.is_null() {
                    Ok(Q::Null)
                } else if Q::same_type(&zx, &zy) {
                    // from Requirement #3D of the same section
                    // "Both scalar expressions (rule scalarExpression) in rule
                    // binaryComparisonPredicate SHALL evaluate to the same type
                    // of literal."
                    match op {
                        Op::Eq => Ok(Q::Bool(zx.eq(&zy))),
                        Op::Neq => Ok(Q::Bool(!zx.eq(&zy))),
                        Op::Lt => Ok(Q::Bool(zx.lt(&zy))),
                        Op::Gt => Ok(Q::Bool(zx.gt(&zy))),
                        Op::Lte => Ok(Q::Bool(zx.le(&zy))),
                        Op::Gte => Ok(Q::Bool(zx.ge(&zy))),
                        _ => Err(MyError::Runtime(
                            format!("Unexpected comparison operator: {op:?}. Abort").into(),
                        )),
                    }
                } else {
                    Err(MyError::Runtime(
                        format!("Unexpected comparison ({op:?}) between {zx:?} and {zy:?}. Abort")
                            .into(),
                    ))
                }
            }
            E::Dyadic(op, x, y) if op.xtd_comparison() => {
                let zx = x.eval(ctx, feature)?;
                let zy = y.eval(ctx, feature)?;
                if zx.is_null() || zy.is_null() {
                    Ok(Q::Null)
                } else {
                    match op {
                        Op::IsLike | Op::IsNotLike => {
                            let input = zx.to_str()?;
                            let pattern = zy.to_str()?;
                            if matches!(op, Op::IsLike) {
                                Ok(Q::Bool(QString::like(&input, &pattern)))
                            } else {
                                Ok(Q::Bool(!QString::like(&input, &pattern)))
                            }
                        }
                        Op::IsBetween | Op::IsNotBetween => {
                            let a = zx.to_num()?;
                            let bounds = zy.to_list()?;
                            let (b0, b1) = (&bounds[0], &bounds[1]);
                            if b0.is_null() || b1.is_null() {
                                Ok(Q::Null)
                            } else {
                                let lo = b0.to_num()?;
                                let hi = b1.to_num()?;
                                let range = if lo <= hi { lo..=hi } else { hi..=lo };
                                if matches!(op, Op::IsBetween) {
                                    Ok(Q::Bool(range.contains(&a)))
                                } else {
                                    Ok(Q::Bool(!range.contains(&a)))
                                }
                            }
                        }
                        Op::IsInList | Op::IsNotInList => {
                            // x must be a literal...
                            if zx.literal_type().is_none() {
                                return Err(MyError::Runtime(
                                    "[NOT] IN LHS is not a literal value".into(),
                                ));
                            }
                            // y must be a list...
                            let list = zy.to_list()?;
                            // ...and every element must be of same-type as x...
                            let ok = list.iter().all(|e| Q::same_type(&zx, e));
                            if !ok {
                                return Err(MyError::Runtime(
                                    "Incompatible [NOT] IN predicate types".into(),
                                ));
                            }
                            if matches!(op, Op::IsInList) {
                                Ok(Q::Bool(zx.contained_by(list)?))
                            } else {
                                Ok(Q::Bool(!zx.contained_by(list)?))
                            }
                        }
                        _ => Err(MyError::Runtime(
                            format!("Unexpected extended comparison ({op:?}). Abort").into(),
                        )),
                    }
                }
            }
            #[rustfmt::skip]
            E::Dyadic(op, x, y) if op.arithmetic() => {
                let zx = x.eval(ctx, feature)?;
                let zy = y.eval(ctx, feature)?;
                if zx.is_null() || zy.is_null() {
                    Ok(Q::Null)
                } else {
                    let a = zx.to_num()?;
                    let b = zy.to_num()?;
                    match op {
                        Op::Plus   => Ok(Q::Num(a + b)),
                        Op::Minus  => Ok(Q::Num(a - b)),
                        Op::Mult   => Ok(Q::Num(a * b)),
                        Op::Div    => Ok(Q::Num(a / b)),
                        Op::IntDiv => Ok(Q::Num(a.rem_euclid(b))),
                        Op::Mod    => Ok(Q::Num(a % b)),
                        Op::Exp    => Ok(Q::Num(a.powf(b).round())),
                        _ => Err(MyError::Runtime(
                            format!("Unexpected arithmetic operator: {op:?}. Abort").into(),
                        )),
                    }
                }
            }
            #[rustfmt::skip]
            E::Dyadic(op, x, y) if op.spatial() => {
                let zx = x.eval(ctx, feature)?;
                let zy = y.eval(ctx, feature)?;
                if zx.is_null() || zy.is_null() {
                    Ok(Q::Null)
                } else {
                    let a = zx.to_geom()?;
                    let b = zy.to_geom()?;
                    match op {
                        Op::SIntersects => Ok(Q::Bool(a.intersects(&b)?)),
                        Op::SEquals     => Ok(Q::Bool(a.equals(&b)?)),
                        Op::SDisjoint   => Ok(Q::Bool(a.disjoint(&b)?)),
                        Op::STouches    => Ok(Q::Bool(a.touches(&b)?)),
                        Op::SWithin     => Ok(Q::Bool(a.within(&b)?)),
                        Op::SOverlaps   => Ok(Q::Bool(a.overlaps(&b)?)),
                        Op::SCrosses    => Ok(Q::Bool(a.crosses(&b)?)),
                        Op::SContains   => Ok(Q::Bool(a.contains(&b)?)),
                        _ => Err(MyError::Runtime(
                            format!("Unexpected spatial operator: {op:?}. Abort").into(),
                        )),
                    }
                }
            }
            E::Dyadic(op, x, y) if op.temporal() => {
                let zx = x.eval(ctx, feature)?;
                let zy = y.eval(ctx, feature)?;
                if zx.is_null() || zy.is_null() {
                    Ok(Q::Null)
                } else if op.instant_or_interval() {
                    let it = eval_temporal_fn(op, zx, zy)?;
                    Ok(Q::Bool(it))
                } else {
                    // expect intervals only...
                    let t1 = zx.to_interval()?;
                    let t2 = zy.to_interval()?;
                    match op {
                        Op::TContains => {
                            // If a proper interval T1 is intervalContains another
                            // proper interval T2, then the beginning of T1 is before
                            // the beginning of T2, and the end of T1 is after the
                            // end of T2.
                            Ok(Q::Bool((t1.0 < t2.0) && (t1.1 > t2.1)))
                        }
                        Op::TDuring => {
                            // If a proper interval T1 is intervalDuring another
                            // proper interval T2, then the beginning of T1 is after
                            // the beginning of T2, and the end of T1 is before the
                            // end of T2.
                            Ok(Q::Bool((t1.0 > t2.0) && (t1.1 < t2.1)))
                        }
                        Op::TFinishedBy => {
                            // If a proper interval T1 is intervalFinishedBy another
                            // proper interval T2, then the beginning of T1 is before
                            // the beginning of T2, and the end of T1 is coincident
                            // with the end of T2.
                            Ok(Q::Bool((t1.0 < t2.0) && (t1.1 == t2.1)))
                        }
                        Op::TFinishes => {
                            // If a proper interval T1 is intervalFinishes another
                            // proper interval T2, then the beginning of T1 is after
                            // the beginning of T2, and the end of T1 is coincident
                            // with the end of T2.
                            Ok(Q::Bool((t1.0 > t2.0) && (t1.1 == t2.1)))
                        }
                        Op::TMeets => {
                            // If a proper interval T1 is intervalMeets another
                            // proper interval T2, then the end of T1 is coincident
                            // with the beginning of T2.
                            Ok(Q::Bool(t1.1 == t2.0))
                        }
                        Op::TMetBy => {
                            // If a proper interval T1 is intervalMetBy another
                            // proper interval T2, then the beginning of T1 is
                            // coincident with the end of T2.
                            Ok(Q::Bool(t1.0 == t2.1))
                        }
                        Op::TOverlappedBy => {
                            // If a proper interval T1 is intervalOverlappedBy another
                            // proper interval T2, then the beginning of T1 is after
                            // the beginning of T2, the beginning of T1 is before the
                            // end of T2, and the end of T1 is after the end of T2.
                            Ok(Q::Bool((t1.0 > t2.0) && (t1.0 < t2.1) && (t1.1 > t2.1)))
                        }
                        Op::TOverlaps => {
                            // If a proper interval T1 is intervalOverlaps another
                            // proper interval T2, then the beginning of T1 is before
                            // the beginning of T2, the end of T1 is after the
                            // beginning of T2, and the end of T1 is before the end
                            // of T2.
                            Ok(Q::Bool((t1.0 < t2.0) && (t1.1 > t2.0) && (t1.1 < t2.1)))
                        }
                        Op::TStartedBy => {
                            // If a proper interval T1 is intervalStartedBy another
                            // proper interval T2, then the beginning of T1 is
                            // coincident with the beginning of T2, and the end of
                            // T1 is after the end of T2.
                            Ok(Q::Bool((t1.0 == t2.0) && (t1.1 > t2.1)))
                        }
                        Op::TStarts => {
                            // If a proper interval T1 is intervalStarts another
                            // proper interval T2, then the beginning of T1 is
                            // coincident with the beginning of T2, and the end of
                            // T1 is before the end of T2.
                            Ok(Q::Bool((t1.0 == t2.0) && (t1.1 < t2.1)))
                        }
                        _ => Err(MyError::Runtime(
                            format!("Unexpected interval operator: {op:?}. Abort").into(),
                        )),
                    }
                }
            }
            #[rustfmt::skip]
            E::Dyadic(op, x, y) if op.array() => {
                let zx = x.eval(ctx, feature)?;
                let a = zx.to_list()?;
                let zy = y.eval(ctx, feature)?;
                let b = zy.to_list()?;
                match op {
                    Op::AEquals      => Ok(Q::Bool(a.eq(&b))),
                    Op::AContains    => Ok(Q::Bool(b.iter().all(|p| a.contains(p)))),
                    Op::AContainedBy => Ok(Q::Bool(a.iter().all(|p| b.contains(p)))),
                    Op::AOverlaps    => Ok(Q::Bool(a.iter().zip(b).any(|(m, n)| m.eq(&n)))),
                    _ => Err(MyError::Runtime(
                        format!("Unexpected array operator: {op:?}. Abort").into(),
                    )),
                }
            }
            E::Dyadic(op, x, y) => Err(MyError::Runtime(
                format!("Unexpected (D) {op:?} between {x:?} and {y:?}. Abort").into(),
            )),
            E::Function(x) => Self::eval_fn_call(ctx, feature, x),
            E::Array(x) => {
                let v: Result<Vec<Q>, MyError> = x.iter().map(|x| x.eval(ctx, feature)).collect();
                Ok(Q::List(v?))
            }
            E::Interval(x, y) => {
                let xx = x.eval(ctx, feature)?;
                let a = match Bound::try_from(&xx) {
                    Ok(x) => Some(x),
                    Err(x) => {
                        error!("Failed converting {} to Bound: {x}", &xx);
                        None
                    }
                };
                let yy = y.eval(ctx, feature)?;
                let b = match Bound::try_from(&yy) {
                    Ok(x) => Some(x),
                    Err(x) => {
                        error!("Failed converting {} to Bound: {x}", &yy);
                        None
                    }
                };
                // if either a or b are none then return null...
                match a {
                    Some(x) => match b {
                        Some(y) => Ok(Q::Interval(x, y)),
                        None => Ok(Q::Null),
                    },
                    None => Ok(Q::Null),
                }
            }
        }
    }

    // partially evaluate expressions and sub-expressions if/when they contain
    // literals.  for example a construct like `1038290-2*2^0` should be replaced
    // by a single numeric value `1038288`.
    pub(crate) fn reduce(e: &mut E) -> Result<Self, MyError> {
        match e {
            // expressions consisting of terminal values cannot be reduced...
            E::Null
            | E::Unbounded
            | E::Bool(_)
            | E::Num(_)
            | E::Str(_)
            | E::Date(_)
            | E::Timestamp(_)
            | E::Spatial(_)
            | E::Id(_) => {
                let it = mem::take(e);
                Ok(it)
            }
            E::Monadic(op, x) => {
                // start by reducing operand...
                let v = E::reduce(x)?;
                // depending on the operator and specific operand type we can reduce...
                match op {
                    Op::Minus => match v {
                        E::Null => Ok(E::Null),
                        E::Num(x) => Ok(E::Num(-x)),
                        _ => Ok(E::Monadic(Op::Minus, Box::new(v))),
                    },
                    Op::Neg => match v {
                        E::Null => Ok(E::Null),
                        E::Bool(x) => Ok(E::Bool(!x)),
                        _ => Ok(E::Monadic(Op::Neg, Box::new(v))),
                    },
                    Op::IsNull => match v {
                        E::Null => Ok(E::Bool(true)),
                        _ => Ok(E::Monadic(Op::IsNull, Box::new(v))),
                    },
                    Op::IsNotNull => match v {
                        E::Null => Ok(E::Bool(false)),
                        _ => Ok(E::Monadic(Op::IsNotNull, Box::new(v))),
                    },
                    Op::CaseI => match v {
                        E::Str(s) => Ok(E::Str(s.and_icase())),
                        // ignoring case multiple times is superfluous...
                        E::Monadic(Op::CaseI, z) => Ok(E::Monadic(Op::CaseI, z)),
                        _ => Ok(E::Monadic(Op::CaseI, Box::new(v))),
                    },
                    Op::AccentI => match v {
                        E::Str(s) => Ok(E::Str(s.and_iaccent())),
                        // so is ignoring accents...
                        E::Monadic(Op::AccentI, z) => Ok(E::Monadic(Op::AccentI, z)),
                        _ => Ok(E::Monadic(Op::AccentI, Box::new(v))),
                    },
                    _ => Ok(E::Monadic(op.to_owned(), Box::new(v))),
                }
            }
            E::Dyadic(Op::And, x, y) => {
                let lhs = E::reduce(x)?;
                let rhs = E::reduce(y)?;
                match (&lhs, &rhs) {
                    // NULL requires special handling...
                    (E::Null, E::Null) => Ok(E::Null),
                    (E::Null, E::Bool(true)) => Ok(E::Null),
                    (E::Null, E::Bool(false)) => Ok(E::Bool(false)),
                    (E::Bool(true), E::Null) => Ok(E::Null),
                    (E::Bool(false), E::Null) => Ok(E::Bool(false)),

                    (E::Bool(true), E::Bool(true)) => Ok(E::Bool(true)),
                    (E::Bool(true), E::Bool(false)) => Ok(E::Bool(false)),
                    (E::Bool(false), E::Bool(true)) => Ok(E::Bool(false)),
                    (E::Bool(false), E::Bool(false)) => Ok(E::Bool(false)),

                    _ => Ok(E::Dyadic(Op::And, Box::new(lhs), Box::new(rhs))),
                }
            }
            E::Dyadic(Op::Or, x, y) => {
                let lhs = E::reduce(x)?;
                let rhs = E::reduce(y)?;
                match (&lhs, &rhs) {
                    // NULL requires special handling...
                    (E::Null, E::Null) => Ok(E::Null),
                    (E::Null, E::Bool(true)) => Ok(E::Bool(true)),
                    (E::Null, E::Bool(false)) => Ok(E::Null),
                    (E::Bool(true), E::Null) => Ok(E::Bool(true)),
                    (E::Bool(false), E::Null) => Ok(E::Null),

                    (E::Bool(true), E::Bool(true)) => Ok(E::Bool(true)),
                    (E::Bool(true), E::Bool(false)) => Ok(E::Bool(true)),
                    (E::Bool(false), E::Bool(true)) => Ok(E::Bool(true)),
                    (E::Bool(false), E::Bool(false)) => Ok(E::Bool(false)),

                    _ => Ok(E::Dyadic(Op::Or, Box::new(lhs), Box::new(rhs))),
                }
            }
            E::Dyadic(op, x, y) if op.comparison() => {
                let lhs = E::reduce(x)?;
                let rhs = E::reduce(y)?;
                if matches!(lhs, E::Null) || matches!(rhs, E::Null) {
                    Ok(E::Null)
                } else {
                    // only compare terminals...
                    let u = lhs.as_literal();
                    let v = rhs.as_literal();
                    match (u, v) {
                        (Some(a), Some(b)) => {
                            // ...that are similar to each other...
                            match op {
                                Op::Eq => Ok(E::Bool(a.eq(&b))),
                                Op::Neq => Ok(E::Bool(!a.eq(&b))),
                                Op::Lt => Ok(E::Bool(a.lt(&b))),
                                Op::Gt => Ok(E::Bool(a.gt(&b))),
                                Op::Lte => Ok(E::Bool(a.le(&b))),
                                Op::Gte => Ok(E::Bool(a.ge(&b))),
                                _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                            }
                        }
                        _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                    }
                }
            }
            E::Dyadic(op, x, y) if op.xtd_comparison() => {
                let lhs = E::reduce(x)?;
                let rhs = E::reduce(y)?;
                if matches!(lhs, E::Null) || matches!(rhs, E::Null) {
                    Ok(E::Null)
                } else {
                    let u = lhs.as_literal();
                    let v = rhs.as_literal();
                    match (u, v) {
                        (Some(a), Some(b)) => match op {
                            Op::IsLike | Op::IsNotLike => {
                                let input = a.to_str()?;
                                let pattern = b.to_str()?;
                                if matches!(op, Op::IsLike) {
                                    Ok(E::Bool(QString::like(&input, &pattern)))
                                } else {
                                    Ok(E::Bool(!QString::like(&input, &pattern)))
                                }
                            }
                            Op::IsBetween | Op::IsNotBetween => {
                                let z = a.to_num()?;
                                let bounds = b.to_list()?;
                                let (b0, b1) = (&bounds[0], &bounds[1]);
                                if b0.is_null() || b1.is_null() {
                                    Ok(E::Null)
                                } else {
                                    let lo = b0.to_num()?;
                                    let hi = b1.to_num()?;
                                    let range = if lo <= hi { lo..=hi } else { hi..=lo };
                                    if matches!(op, Op::IsBetween) {
                                        Ok(E::Bool(range.contains(&z)))
                                    } else {
                                        Ok(E::Bool(!range.contains(&z)))
                                    }
                                }
                            }
                            Op::IsInList | Op::IsNotInList => {
                                if a.literal_type().is_none() {
                                    return Err(MyError::Runtime(
                                        "[NOT] IN LHS is not a literal value".into(),
                                    ));
                                }
                                let list = b.to_list()?;
                                let ok = list.iter().all(|e| Q::same_type(&a, e));
                                if !ok {
                                    return Err(MyError::Runtime(
                                        "Incompatible [NOT] IN predicate types".into(),
                                    ));
                                }
                                if matches!(op, Op::IsInList) {
                                    Ok(E::Bool(a.contained_by(list)?))
                                } else {
                                    Ok(E::Bool(!a.contained_by(list)?))
                                }
                            }
                            _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                        },
                        _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                    }
                }
            }
            E::Dyadic(op, x, y) if op.arithmetic() => {
                let lhs = E::reduce(x)?;
                let rhs = E::reduce(y)?;
                if matches!(lhs, E::Null) || matches!(rhs, E::Null) {
                    Ok(E::Null)
                } else {
                    let u = lhs.as_literal();
                    let v = rhs.as_literal();
                    match (u, v) {
                        (Some(a), Some(b)) => {
                            let m = a.to_num()?;
                            let n = b.to_num()?;
                            match op {
                                Op::Plus => Ok(E::Num(m + n)),
                                Op::Minus => Ok(E::Num(m - n)),
                                Op::Mult => Ok(E::Num(m * n)),
                                Op::Div => Ok(E::Num(m / n)),
                                Op::IntDiv => Ok(E::Num(m.rem_euclid(n))),
                                Op::Mod => Ok(E::Num(m % n)),
                                Op::Exp => Ok(E::Num(m.powf(n).round())),
                                _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                            }
                        }
                        _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                    }
                }
            }
            E::Dyadic(op, x, y) if op.spatial() => {
                let lhs = E::reduce(x)?;
                let rhs = E::reduce(y)?;
                if matches!(lhs, E::Null) || matches!(rhs, E::Null) {
                    Ok(E::Null)
                } else {
                    let u = lhs.as_literal();
                    let v = rhs.as_literal();
                    match (u, v) {
                        (Some(a), Some(b)) => {
                            let m = a.to_geom()?;
                            let n = b.to_geom()?;
                            match op {
                                Op::SIntersects => Ok(E::Bool(m.intersects(&n)?)),
                                Op::SEquals => Ok(E::Bool(m.equals(&n)?)),
                                Op::SDisjoint => Ok(E::Bool(m.disjoint(&n)?)),
                                Op::STouches => Ok(E::Bool(m.touches(&n)?)),
                                Op::SWithin => Ok(E::Bool(m.within(&n)?)),
                                Op::SOverlaps => Ok(E::Bool(m.overlaps(&n)?)),
                                Op::SCrosses => Ok(E::Bool(m.crosses(&n)?)),
                                Op::SContains => Ok(E::Bool(m.contains(&n)?)),
                                _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                            }
                        }
                        _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                    }
                }
            }
            E::Dyadic(op, x, y) if op.temporal() => {
                let lhs = E::reduce(x)?;
                let rhs = E::reduce(y)?;
                if matches!(lhs, E::Null) || matches!(rhs, E::Null) {
                    Ok(E::Null)
                } else {
                    let u = lhs.as_literal();
                    let v = rhs.as_literal();
                    match (u, v) {
                        (Some(a), Some(b)) => {
                            if a.is_null() || b.is_null() {
                                Ok(E::Null)
                            } else if op.instant_or_interval() {
                                let it = eval_temporal_fn(op, a, b)?;
                                Ok(E::Bool(it))
                            } else {
                                // expect intervals only...
                                let t1 = a.to_interval()?;
                                let t2 = b.to_interval()?;
                                match op {
                                    Op::TContains => Ok(E::Bool((t1.0 < t2.0) && (t1.1 > t2.1))),
                                    Op::TDuring => Ok(E::Bool((t1.0 > t2.0) && (t1.1 < t2.1))),
                                    Op::TFinishedBy => Ok(E::Bool((t1.0 < t2.0) && (t1.1 == t2.1))),
                                    Op::TFinishes => Ok(E::Bool((t1.0 > t2.0) && (t1.1 == t2.1))),
                                    Op::TMeets => Ok(E::Bool(t1.1 == t2.0)),
                                    Op::TMetBy => Ok(E::Bool(t1.0 == t2.1)),
                                    Op::TOverlappedBy => {
                                        Ok(E::Bool((t1.0 > t2.0) && (t1.0 < t2.1) && (t1.1 > t2.1)))
                                    }
                                    Op::TOverlaps => {
                                        Ok(E::Bool((t1.0 < t2.0) && (t1.1 > t2.0) && (t1.1 < t2.1)))
                                    }
                                    Op::TStartedBy => Ok(E::Bool((t1.0 == t2.0) && (t1.1 > t2.1))),
                                    Op::TStarts => Ok(E::Bool((t1.0 == t2.0) && (t1.1 < t2.1))),
                                    _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                                }
                            }
                        }
                        _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                    }
                }
            }
            E::Dyadic(op, x, y) if op.array() => {
                let lhs = E::reduce(x)?;
                let rhs = E::reduce(y)?;
                if matches!(lhs, E::Null) || matches!(rhs, E::Null) {
                    Ok(E::Null)
                } else {
                    let u = lhs.as_literal();
                    let v = rhs.as_literal();
                    match (u, v) {
                        (Some(a), Some(b)) => {
                            let m = a.to_list()?;
                            let n = b.to_list()?;
                            match op {
                                Op::AEquals => Ok(E::Bool(m.eq(&n))),
                                Op::AContains => Ok(E::Bool(n.iter().all(|p| m.contains(p)))),
                                Op::AContainedBy => Ok(E::Bool(m.iter().all(|p| n.contains(p)))),
                                Op::AOverlaps => {
                                    Ok(E::Bool(m.iter().zip(n).any(|(i, j)| i.eq(&j))))
                                }
                                _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                            }
                        }
                        _ => Ok(E::Dyadic(op.to_owned(), Box::new(lhs), Box::new(rhs))),
                    }
                }
            }
            E::Dyadic(op, x, y) => unreachable!("Unexpected dyadic {op} between {x:?} and {y:?}"),
            E::Function(x) => {
                let z_params = mem::take(&mut x.params);
                let params: Result<Vec<E>, MyError> = z_params
                    .into_iter()
                    .map(|mut x| E::reduce(&mut x))
                    .collect();
                let call = Call {
                    name: x.name.to_owned(),
                    params: params?,
                };
                Ok(E::Function(call))
            }
            E::Array(x) => {
                let mut list = vec![];
                for e in x {
                    let i = E::reduce(e)?;
                    list.push(i);
                }
                Ok(E::Array(list))
            }
            E::Interval(x, y) => {
                let u = E::reduce(x)?;
                let v = E::reduce(y)?;
                let it = E::Interval(Box::new(u), Box::new(v));
                Ok(it)
            }
        }
    }

    fn eval_fn_call(ctx: &Context, feature: &Resource, c: &Call) -> Result<Q, MyError> {
        let fname = &c.name;
        if let Some(fn_meta) = ctx.fn_info(fname) {
            // check if number of declared args matches that of call params...
            if c.params.len() != fn_meta.arg_types.len() {
                let msg = format!(
                    "Function '{fname}()' has wrong ({}) arguments count; expected {}",
                    c.params.len(),
                    fn_meta.arg_types.len(),
                );
                error!("Failed: {}", msg);
                return Err(MyError::Runtime(msg.into()));
            }

            // evaluate the associated parameters so we can match them to the
            // function's arguments...
            let args: Result<Vec<Q>, MyError> =
                c.params.iter().map(|x| x.eval(ctx, feature)).collect();
            let args = args?;

            // Check if each argument type matches the expected one
            let mut z_args: Vec<Box<dyn Any>> = vec![];
            for (arg, type_) in args.iter().zip(&fn_meta.arg_types) {
                match *type_ {
                    ExtDataType::Str => z_args.push(Box::new(arg.to_str()?)),
                    ExtDataType::Num => z_args.push(Box::new(arg.to_num()?)),
                    ExtDataType::Bool => z_args.push(Box::new(arg.to_bool()?)),
                    ExtDataType::Timestamp | ExtDataType::Date => {
                        let b = arg.to_bound()?;
                        let z = b.to_zoned()?;
                        z_args.push(Box::new(z));
                    }
                    ExtDataType::Geom => z_args.push(Box::new(arg.to_geom()?)),
                }
            }

            // Call the closure
            let call = (fn_meta.closure)(z_args);
            // convert result to a Q and return it...
            match call {
                Some(x) => match fn_meta.result_type {
                    ExtDataType::Str => {
                        let result = x
                            .downcast_ref::<String>()
                            .unwrap_or_else(|| panic!("Expected '{fname}()' to return a string"));
                        debug!("Invoking '{fname}()' resulted in {result}");
                        let qs = QString::plain(result);
                        Ok(Q::Str(qs))
                    }
                    ExtDataType::Num => {
                        let result = x
                            .downcast_ref::<f64>()
                            .unwrap_or_else(|| panic!("Expected '{fname}()' to return a float"));
                        debug!("Invoking '{fname}()' resulted in {result}");
                        Ok(Q::Num(*result))
                    }
                    ExtDataType::Bool => {
                        let result = x
                            .downcast_ref::<bool>()
                            .unwrap_or_else(|| panic!("Expected '{fname}()' to return a boolean"));
                        debug!("Invoking '{fname}()' resulted in {result}");
                        Ok(Q::Bool(*result))
                    }
                    ExtDataType::Timestamp => {
                        let result = x
                            .downcast_ref::<Zoned>()
                            .unwrap_or_else(|| panic!("Expected '{fname}()' to return a Zoned"));
                        debug!("Invoking '{fname}()' resulted in {result}");
                        let b = Bound::Timestamp(result.to_owned());
                        Ok(Q::Instant(b))
                    }
                    ExtDataType::Date => {
                        let result = x
                            .downcast_ref::<Zoned>()
                            .unwrap_or_else(|| panic!("Expected '{fname}()' to return a Zoned"));
                        debug!("Invoking '{fname}()' resulted in {result}");
                        let b = Bound::Date(result.to_owned());
                        Ok(Q::Instant(b))
                    }
                    ExtDataType::Geom => {
                        let result = x
                            .downcast_ref::<G>()
                            .unwrap_or_else(|| panic!("Expected '{fname}()' to return a Geometry"));
                        debug!("Invoking '{fname}()' resulted in a geometry");
                        Ok(Q::Geom(result.to_owned()))
                    }
                },
                None => {
                    error!("ERROR: Failed invoking '{fname}()'");
                    Ok(Q::Null)
                }
            }
        } else {
            error!("Use of unknown '{fname}' Fn");
            Ok(Q::Null)
        }
    }

    #[cfg(test)]
    pub(crate) fn as_str(&self) -> Option<&QString> {
        match self {
            E::Str(x) => Some(x),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn as_zoned(&self) -> Option<&Zoned> {
        match self {
            E::Date(x) => Some(x),
            E::Timestamp(x) => Some(x),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn as_spatial(&self) -> Option<&G> {
        match self {
            E::Spatial(x) => Some(x),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn as_dyadic(&self) -> Option<(&Op, &E, &E)> {
        match self {
            E::Dyadic(op, a, b) => Some((op, a, b)),
            _ => None,
        }
    }

    #[cfg(test)]
    fn as_array(&self) -> Option<&Vec<E>> {
        match self {
            E::Array(x) => Some(x),
            _ => None,
        }
    }
}

// arguments are either intervals, or instants.
fn eval_temporal_fn(op: &Op, t1: Q, t2: Q) -> Result<bool, MyError> {
    let (t1_is_instant, t2_is_instant, b0, b1, b2, b3) = unfold_queryables(&t1, &t2)?;
    match op {
        // start of T1 is after end of T2
        Op::TAfter => match (t1_is_instant, t2_is_instant) {
            (true, true) => Ok(b0 > b2),
            (true, false) => Ok(b0 > b3),
            (false, true) => Ok(b0 > b2),
            (false, false) => Ok(b0 > b3),
        },
        // end of T1 is before start of T2
        Op::TBefore => match (t1_is_instant, t2_is_instant) {
            (true, true) => Ok(b0 < b2),
            (true, false) => Ok(b0 < b2),
            (false, true) => Ok(b1 < b2),
            (false, false) => Ok(b1 < b2),
        },
        // (t1 T_BEFORE t2) OR (t1 T_AFTER t2)
        Op::TDisjoint => match (t1_is_instant, t2_is_instant) {
            (true, true) => Ok(b0 != b2),
            (true, false) => Ok((b0 < b2) || (b0 > b3)),
            (false, true) => Ok((b1 < b2) || (b0 > b2)),
            (false, false) => Ok((b1 < b2) || (b0 > b3)),
        },
        // Start and end of t1 and t2 coincide
        Op::TEquals => match (t1_is_instant, t2_is_instant) {
            (true, true) => Ok(b0 == b2),
            (true, false) => Ok((b0 == b2) && (b0 == b3)),
            (false, true) => Ok((b0 == b2) && (b1 == b2)),
            (false, false) => Ok((b0 == b2) && (b1 == b3)),
        },
        // NOT (t1 T_DISJOINT t2)
        Op::TIntersects => match (t1_is_instant, t2_is_instant) {
            (true, true) => Ok(b0 == b2),
            (true, false) => Ok(!((b0 < b2) || (b0 > b3))),
            (false, true) => Ok(!((b1 < b2) || (b0 > b2))),
            (false, false) => Ok(!((b1 < b2) || (b0 > b3))),
        },
        _ => Err(MyError::Runtime(
            format!("Unexpected ({op}) temporal operator").into(),
        )),
    }
}

// ensure arguments are temporal operands.  return flags indicating whether
// they're Instants or Intervals, and their associated start and end Bounds.
fn unfold_queryables(a: &Q, b: &Q) -> Result<(bool, bool, Bound, Bound, Bound, Bound), MyError> {
    let a_is_instant = a.is_instant();
    let b_is_instant = b.is_instant();
    match (a_is_instant, b_is_instant) {
        (true, true) => {
            let t1 = a.to_bound()?;
            let t2 = b.to_bound()?;
            Ok((true, true, t1, Bound::None, t2, Bound::None))
        }
        (true, false) => {
            let t1 = a.to_bound()?;
            let t2 = b.to_interval()?;
            Ok((true, false, t1, Bound::None, t2.0, t2.1))
        }
        (false, true) => {
            let t1 = a.to_interval()?;
            let t2 = b.to_bound()?;
            Ok((false, true, t1.0, t1.1, t2, Bound::None))
        }
        (false, false) => {
            let t1 = a.to_interval()?;
            let t2 = b.to_interval()?;
            Ok((false, false, t1.0, t1.1, t2.0, t2.1))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Call {
    pub(crate) name: String,
    pub(crate) params: Vec<E>,
}

impl Call {
    pub(crate) fn from(name: &str, params: Vec<E>) -> Self {
        Call {
            name: name.to_owned(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::cql2;

    #[test]
    fn test_ex57() {
        const CQL: &str = r#"T_DURING(INTERVAL(starts_at, ends_at), INTERVAL('2005-01-10', '2010-02-10'))
        "#;
        const D1: &str = "2005-01-10";
        const D2: &str = "2010-02-10";

        let exp = cql2::expression(CQL);
        assert!(exp.is_ok());

        let binding = exp.unwrap();
        let (op, x1, x2) = binding.as_dyadic().expect("Not a dyadic expression");
        assert_eq!(op, &Op::TDuring);

        // both argumens are interval expressions...
        assert!(matches!(x1, E::Interval(_, _)));
        assert!(matches!(x2, E::Interval(_, _)));

        // only the 2nd one is resolvable...
        let (a, b) = x2.as_interval().expect("Not an interval");
        let a_date = a.as_zoned().expect("'a' is not fixed bound");
        let b_date = b.as_zoned().expect("'b' is not fixed bound");
        assert_eq!(a_date.date().to_string(), D1);
        assert_eq!(b_date.date().to_string(), D2);
    }

    #[test]
    fn test_c7_02() {
        const CQL: &str = r#"depth BETWEEN 100.0 and 150.0"#;

        let exp = cql2::expression(CQL);
        assert!(exp.is_ok());

        let binding = exp.unwrap();
        let (op, a, b) = binding.as_dyadic().expect("Not a dyadic expression");
        assert_eq!(op, &Op::IsBetween);

        // a is an identifier...
        assert!(matches!(a, E::Id(_)));
        assert_eq!(a.as_id().expect("Not an ID"), "depth");

        // b is a list of 2 numbers...
        assert!(matches!(b, E::Array(_)));
        let v = b.as_array().expect("Not an array expression");
        assert_eq!(v.len(), 2);

        assert!(matches!(v[0], E::Num(100.)));
        assert!(matches!(v[1], E::Num(150.)));
    }

    #[test]
    fn test_ex06a() {
        const CQL: &str = r#"    eo:cloud_cover BETWEEN 0.1 AND 0.2
AND landsat:wrs_row=28
AND landsat:wrs_path=203
"#;
        let exp = cql2::expression(CQL);
        assert!(exp.is_ok());
    }
}
