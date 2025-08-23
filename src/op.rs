// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! OGC CQL2 grammar operators...
//!

use core::fmt;

/// Operators...
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Op {
    // arithmetic operators + keywords...
    Plus,
    Minus, // both (M) negating a numeric, and (D) subtraction of 2 numerics
    Mult,
    Div,
    IntDiv, // integer division
    Mod,    // modulo
    Exp,    // exponentiation
    Neg,    // (M) negate of a boolean predicate
    // comparison operators...
    Eq,  // equal to
    Neq, // not equal to
    Lt,  // less than
    Gt,  // greater than
    Lte, // less than or equal to
    Gte, // greater than or equal to
    And,
    Or,
    // character stuff...
    CaseI,   // (M) case insensitive
    AccentI, // (M) accent insensitive
    // spatial stuff...
    SIntersects,
    SEquals,
    SDisjoint,
    STouches,
    SWithin,
    SOverlaps,
    SCrosses,
    SContains,
    // temporal stuff...
    TAfter,
    TBefore,
    TContains,
    TDisjoint,
    TDuring,
    TEquals,
    TFinishedBy,
    TFinishes,
    TIntersects,
    TMeets,
    TMetBy,
    TOverlappedBy,
    TOverlaps,
    TStartedBy,
    TStarts,
    // array stuff...
    AEquals,
    AContains,
    AContainedBy,
    AOverlaps,
    // others...
    IsLike,
    IsNotLike,
    IsBetween,
    IsNotBetween,
    IsInList,
    IsNotInList,
    IsNull,    // (M)
    IsNotNull, // (M)
}

impl Op {
    pub(crate) fn arithmetic(&self) -> bool {
        matches!(
            self,
            Op::Plus | Op::Minus | Op::Mult | Op::Div | Op::Mod | Op::IntDiv | Op::Exp
        )
    }

    pub(crate) fn array(&self) -> bool {
        matches!(
            self,
            Op::AContainedBy | Op::AContains | Op::AEquals | Op::AOverlaps
        )
    }

    // basic comparison operators...
    pub(crate) fn comparison(&self) -> bool {
        matches!(
            self,
            // basic
            Op::Eq | Op::Neq | Op::Lt | Op::Lte | Op::Gt | Op::Gte | Op::IsNull | Op::IsNotNull
        )
    }

    // extended comparison opertors...
    pub(crate) fn xtd_comparison(&self) -> bool {
        matches!(
            self,
            Op::IsLike
                | Op::IsNotLike
                | Op::IsBetween
                | Op::IsNotBetween
                | Op::IsInList
                | Op::IsNotInList
        )
    }

    pub(crate) fn spatial(&self) -> bool {
        matches!(
            self,
            // basic
            Op::SIntersects
            // other...
            | Op::SContains
                | Op::SCrosses
                | Op::SDisjoint
                | Op::SEquals
                | Op::SOverlaps
                | Op::STouches
                | Op::SWithin
        )
    }

    pub(crate) fn temporal(&self) -> bool {
        matches!(
            self,
            Op::TAfter
                | Op::TBefore
                | Op::TContains
                | Op::TDisjoint
                | Op::TDuring
                | Op::TEquals
                | Op::TFinishedBy
                | Op::TFinishes
                | Op::TIntersects
                | Op::TMeets
                | Op::TMetBy
                | Op::TOverlappedBy
                | Op::TOverlaps
                | Op::TStartedBy
                | Op::TStarts
        )
    }

    /// Return TRUE if this is applicable to either _Instants_ or _Intervals_;
    /// FALSE otherwise.
    pub(crate) fn instant_or_interval(&self) -> bool {
        matches!(
            self,
            Op::TAfter | Op::TBefore | Op::TDisjoint | Op::TEquals | Op::TIntersects
        )
    }

    pub(crate) fn nullable(&self) -> bool {
        matches!(self, Op::IsNull | Op::IsNotNull)
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Plus => write!(f, "+"),
            Op::Minus => write!(f, "-"),
            Op::Mult => write!(f, "*"),
            Op::Div => write!(f, "/"),
            Op::IntDiv => write!(f, "div"),
            Op::Mod => write!(f, "%"),
            Op::Exp => write!(f, "^"),
            Op::Neg => write!(f, "!"),
            Op::Eq => write!(f, "=="),
            Op::Neq => write!(f, "!="),
            Op::Lt => write!(f, "<"),
            Op::Gt => write!(f, ">"),
            Op::Lte => write!(f, "<="),
            Op::Gte => write!(f, ">="),
            Op::And => write!(f, "&&"),
            Op::Or => write!(f, "||"),
            Op::CaseI => write!(f, "CASEI"),
            Op::AccentI => write!(f, "ACCENTI"),
            Op::SIntersects => write!(f, "S_INTERSECTS"),
            Op::SEquals => write!(f, "S_EQUALS"),
            Op::SDisjoint => write!(f, "S_DISJOINT"),
            Op::STouches => write!(f, "S_TOUCHES"),
            Op::SWithin => write!(f, "S_WITHIN"),
            Op::SOverlaps => write!(f, "S_OVERLAPS"),
            Op::SCrosses => write!(f, "S_CROSSES"),
            Op::SContains => write!(f, "S_CONTAINS"),
            Op::TAfter => write!(f, "T_AFTER"),
            Op::TBefore => write!(f, "T_BEFORE"),
            Op::TContains => write!(f, "T_CONTAINS"),
            Op::TDisjoint => write!(f, "T_DISJOINT"),
            Op::TDuring => write!(f, "T_DURING"),
            Op::TEquals => write!(f, "T_EQUALS"),
            Op::TFinishedBy => write!(f, "T_FINISHEDBY"),
            Op::TFinishes => write!(f, "T_FINISHES"),
            Op::TIntersects => write!(f, "T_INTERSECTS"),
            Op::TMeets => write!(f, "T_MEETS"),
            Op::TMetBy => write!(f, "T_METBY"),
            Op::TOverlappedBy => write!(f, "T_OVERLAPPEDBY"),
            Op::TOverlaps => write!(f, "T_OVERLAPS"),
            Op::TStartedBy => write!(f, "T_STARTEDBY"),
            Op::TStarts => write!(f, "T_STARTS"),
            Op::AEquals => write!(f, "A_EQUALS"),
            Op::AContains => write!(f, "A_CONTAINS"),
            Op::AContainedBy => write!(f, "A_CONTAINEDBY"),
            Op::AOverlaps => write!(f, "A_OVERLAPS"),
            Op::IsLike => write!(f, "LIKE"),
            Op::IsNotLike => write!(f, "NOT LIKE"),
            Op::IsBetween => write!(f, "BETWEEN"),
            Op::IsNotBetween => write!(f, "NOT BETWEEN"),
            Op::IsInList => write!(f, "IN"),
            Op::IsNotInList => write!(f, "NOT IN"),
            Op::IsNull => write!(f, "IS NULL"),
            Op::IsNotNull => write!(f, "IS NOT NULL"),
        }
    }
}
