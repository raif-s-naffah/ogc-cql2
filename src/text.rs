// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! PEG parser rules and actions.
//!
//! Being a PEG parser means the following limits/constraints/directives
//! should be observed when writing and ordering rules...
//!
//! * From the [pest book](https://pest.rs/book/grammars/peg.html) (even though
//!   i'm not using the `pest` crate) states: **_In general, when writing a
//!   parser with choices, put the longest or most specific choice first, and
//!   the shortest or most general choice last._**
//! * From the same [book](https://pest.rs/book/grammars/peg.html): **_In a
//!   system with backtracking (like regular expressions), you would back up
//!   one step, "un-eating" a character, and then try again. But PEGs do not do
//!   this. In the rule `first _ second`, once `first` parses successfully, it
//!   has consumed some characters that will never come back. `second` can only
//!   run on the input that `first` did not consume._**
//!

use crate::{
    expr::{Call, E},
    geom::{BBox, G, Geometries, Line, Lines, Point, Points, Polygon, Polygons},
    op::Op,
    qstring::QString,
};
use jiff::Zoned;
use jiff::{civil::Date, tz::TimeZone};

peg::parser! {
    pub grammar cql2() for str {
        // howto handle case-insensitive tokens.  see
        // https://github.com/kevinmehall/rust-peg/issues/216
        rule i(literal: &'static str)
        = input:$([_]*<{literal.len()}>)
        {? if input.eq_ignore_ascii_case(literal) { Ok(()) } else { Err(literal) } }

        // ===== whitespace ===================================================
        rule _ = quiet! { [
            '\t'
            | '\u{0009}'      // Character tabulation
            | '\n'
            | '\u{000A}'   // Line feed
            | '\u{000B}'   // Line tabulation
            | '\u{000C}'   // Form feed
            | '\r'
            | '\u{000D}'   // Carriage return
            | '\u{0020}'   // Space
            | '\u{0085}'   // Next line
            | '\u{00A0}'   // No-break space
            | '\u{1680}'   // Ogham space mark
            | '\u{2000}'   // En quad
            | '\u{2001}'   // Em quad
            | '\u{2002}'   // En space
            | '\u{2003}'   // Em space
            | '\u{2004}'   // Three-per-em space
            | '\u{2005}'   // Four-per-em space
            | '\u{2006}'   // Six-per-em space
            | '\u{2007}'   // Figure space
            | '\u{2008}'   // Punctuation space
            | '\u{2009}'   // Thin space
            | '\u{200A}'   // Hair space
            | '\u{2028}'   // Line separator
            | '\u{2029}'   // Paragraph separator
            | '\u{202F}'   // Narrow no-break space
            | '\u{205F}'   // Medium mathematical space
            | '\u{3000}'   // Ideographic space
        ]* }

        // ignore whitespaces and EOL + EOF...
        pub rule expression() -> E = _ x:boolean_expression() _ ![_] { x }

        #[cache_left_rec]
        rule boolean_expression() -> E
        = x:boolean_term() _ y:or_term()* {
            match y.len() {
                0 => x,
                1 => E::Dyadic(Op::Or, Box::new(x), Box::new(y[0].clone())),
                _ => E::Dyadic(Op::Or, Box::new(x), Box::new(E::Array(y))),
            }
        }

        rule or_term() -> E = i("OR") _ x:boolean_expression()  { x }

        #[cache_left_rec]
        rule boolean_term() -> E = x:boolean_factor() _ y:and_term()* {
            match y.len() {
                0 => x,
                1 => E::Dyadic(Op::And, Box::new(x), Box::new(y[0].clone())),
                _ => E::Dyadic(Op::And, Box::new(x), Box::new(E::Array(y))),
            }
        }

        rule and_term() -> E = i("AND") _ y:boolean_expression() { y }

        #[cache_left_rec]
        rule boolean_factor() -> E = n:(i("NOT") _)? x:boolean_primary() {
            match n {
                Some(_) => E::Monadic(Op::Neg, Box::new(x)),
                None => x,
            }
        }

        #[cache_left_rec]
        rule boolean_primary() -> E
        = "(" _ x:boolean_expression() _ ")" { x }
        / x:comparison_predicate()           { x }
        / x:spatial_predicate()              { x }
        / x:temporal_predicate()             { x }
        / x:array_predicate()                { x }
        / x:function()                       { x }
        / x:boolean_literal()                { x }

        #[cache_left_rec]
        rule comparison_predicate() -> E
        = x:binary_comparison_predicate() { x }
        / x:is_like_predicate()           { x }
        / x:is_between_predicate()        { x }
        / x:is_in_list_predicate()        { x }
        / x:is_null_predicate()           { x }

        #[cache_left_rec]
        rule is_null_predicate() -> E
        = x:is_null_operand() _ i("IS") _ n:(i("NOT") _)? i("NULL") {
            if n.is_none() {
                E::Monadic(Op::IsNull, Box::new(x))
            } else {
                E::Monadic(Op::IsNotNull, Box::new(x))
            }
        }

        #[cache_left_rec]
        rule is_null_operand() -> E
        = x:temporal_instance()     { x }
        / x:spatial_instance()      { x }
        / x:character_clause()      { x }
        / x:arithmetic_expression() { x }
        / x:numeric_literal()       { x }
        / x:function()              { x }
        / x:property_name()         { x }
        / x:boolean_expression()    { x }

        rule binary_comparison_predicate() -> E
        = x:scalar_expression() _ op:comparison_operator() _ y:scalar_expression()
        { E::Dyadic(op, Box::new(x), Box::new(y)) }

        rule scalar_expression() -> E
        = x:boolean_literal()       { x }
        / x:character_clause()      { x }
        / x:instant_instance()      { x }
        / x:arithmetic_expression() { x }
        / x:numeric_literal()       { x }
        / x:function()              { x }
        / x:property_name()         { x }

        rule comparison_operator() -> Op
        = "=" { Op::Eq }
        / x:$("<" "="? ">"?) {
            match x {
                "<" =>  Op::Lt,
                "<=" => Op::Lte,
                "<>" => Op::Neq,
                _ => panic!("Expected < [= | >]")
            }
        }
        / x:$(">" "="?) {
            match x {
                ">" =>  Op::Gt,
                ">=" => Op::Gte,
                _ => panic!("Expected > [=]")
            }
        }

        pub(crate) rule is_like_predicate() -> E
        = x:character_expression() _ n:(i("NOT") _)? i("LIKE") _ y:pattern_expression() {
            match n {
                Some(_) => E::Dyadic(Op::IsNotLike, Box::new(x), Box::new(y)),
                None => E::Dyadic(Op::IsLike, Box::new(x), Box::new(y)),
            }
        }

        rule pattern_expression() -> E
        = i("CASEI") _ "(" _ x:pattern_expression() _ ")"   { E::Monadic(Op::CaseI, Box::new(x)) }
        / i("ACCENTI") _ "(" _ x:pattern_expression() _ ")" { E::Monadic(Op::AccentI, Box::new(x)) }
        / x:character_literal()                             { x }

        rule is_between_predicate() -> E
        = x:numeric_expression() _ n:(i("NOT") _)? i("BETWEEN") _ a:numeric_expression() _ i("AND") _ b:numeric_expression() {
            let op = if n.is_none() { Op::IsBetween } else { Op::IsNotBetween };
            E::Dyadic(op, Box::new(x), Box::new(E::Array(vec![a, b])))
        }

        #[cache]
        pub(crate) rule numeric_expression() -> E
        = x:arithmetic_expression() { x }
        / x:numeric_literal()       { x }
        / x:function()              { x }
        / x:property_name()         { x }

        rule is_in_list_predicate() -> E
        = x:scalar_expression() _ n:(i("NOT") _)? i("IN") _ "(" _ y:(in_list() ) _ ")" {
            let op = if n.is_none() { Op::IsInList } else { Op::IsNotInList };
            E::Dyadic(op, Box::new(x), Box::new(E::Array(y)))
        }

        rule in_list() -> Vec<E> = x:(scalar_expression() ++ (_ "," _)) { x }

        #[cache]
        rule spatial_predicate() -> E
        = op:spatial_function() _ "(" _ x:geom_expression() _ "," _ y:geom_expression() _ ")"
        { E::Dyadic(op, Box::new(x), Box::new(y)) }

        rule spatial_function() -> Op
        = i("S_INTERSECTS") { Op::SIntersects }
        / i("S_EQUALS")     { Op::SEquals }
        / i("S_DISJOINT")   { Op::SDisjoint }
        / i("S_TOUCHES")    { Op::STouches }
        / i("S_WITHIN")     { Op::SWithin }
        / i("S_OVERLAPS")   { Op::SOverlaps }
        / i("S_CROSSES")    { Op::SCrosses }
        / i("S_CONTAINS")   { Op::SContains }

        #[cache]
        pub(crate) rule geom_expression() -> E
        = x:spatial_instance() { x }
        / x:function()         { x }
        / x:property_name()    { x }

        #[cache]
        pub(crate) rule temporal_predicate() -> E
        = op:temporal_function() _ "(" _ x:temporal_expression() _ "," _ y:temporal_expression() _ ")"
        { E::Dyadic(op, Box::new(x), Box::new(y)) }

        #[cache]
        pub(crate) rule temporal_expression() -> E
        = x:temporal_instance() { x }
        / x:function()          { x }
        / x:property_name()     { x }

        rule temporal_function() -> Op
        = i("T_AFTER")        { Op::TAfter }
        / i("T_BEFORE")       { Op::TBefore }
        / i("T_CONTAINS")     { Op::TContains }
        / i("T_DISJOINT")     { Op::TDisjoint }
        / i("T_DURING")       { Op::TDuring }
        / i("T_EQUALS")       { Op::TEquals }
        / i("T_FINISHEDBY")   { Op::TFinishedBy }
        / i("T_FINISHES")     { Op::TFinishes }
        / i("T_INTERSECTS")   { Op::TIntersects }
        / i("T_MEETS")        { Op::TMeets }
        / i("T_METBY")        { Op::TMetBy }
        / i("T_OVERLAPPEDBY") { Op::TOverlappedBy }
        / i("T_OVERLAPS")     { Op::TOverlaps }
        / i("T_STARTEDBY")    { Op::TStartedBy }
        / i("T_STARTS")       { Op::TStarts }

        rule array_predicate() -> E
        = op:array_function() _ "(" _ x:array_expression() _ "," _ y:array_expression() _ ")"
        { E::Dyadic(op, Box::new(x), Box::new(y)) }

        rule array_expression() -> E
        = x:array()         { x }
        / x:function()      { x }
        / x:property_name() { x }

        #[cache]
        rule array() -> E = "(" _ x:(array_element() ** (_ "," _)) _ ")" { E::Array(x) }

        rule array_element() -> E
        = x:character_clause()      { x }
        / x:temporal_instance()     { x }
        / x:spatial_instance()      { x }
        / x:array()                 { x }
        / x:arithmetic_expression() { x }
        / x:numeric_literal()       { x }
        / x:boolean_expression()    { x }
        / x:function()              { x }
        / x:property_name()         { x }

        rule array_function() -> Op
        = i("A_EQUALS")      { Op::AEquals }
        / i("A_CONTAINS")    { Op::AContains }
        / i("A_CONTAINEDBY") { Op::AContainedBy }
        / i("A_OVERLAPS")    { Op::AOverlaps }

        #[cache]
        rule arithmetic_expression() -> E = x:arithmetic_term() _ v:add_term()* {
            match v.len() {
                0 => x,
                1 => {
                    let (op, y) = &v[0];
                    E::Dyadic(op.clone(), Box::new(x), Box::new(y.clone()))
                },
                _ => {
                    let (op, y) = &v[0];
                    let mut t = E::Dyadic(op.clone(), Box::new(x), Box::new(y.clone()));
                    for (op, y) in v[1..].iter() {
                        t = E::Dyadic(op.clone(), Box::new(t), Box::new(y.clone()))
                    }
                    t
                },
            }
        }

        rule add_term() -> (Op, E)
        = "+" _ y: arithmetic_term() { (Op::Plus, y) }
        / "-" _ y: arithmetic_term() { (Op::Minus, y) }

        rule arithmetic_term() -> E = x:power_term() _ v:mult_term()* {
            match v.len() {
                0 => x,
                1 => {
                    let (op, y) = &v[0];
                    E::Dyadic(op.clone(), Box::new(x), Box::new(y.clone()))
                },
                _ => {
                    let (op, y) = &v[0];
                    let mut t = E::Dyadic(op.clone(), Box::new(x), Box::new(y.clone()));
                    for (op, y) in v[1..].iter() {
                        t = E::Dyadic(op.clone(), Box::new(t), Box::new(y.clone()))
                    }
                    t
                },
            }
        }

        rule mult_term() -> (Op, E)
        = "*" _ y: power_term()   { (Op::Mult, y) }
        / "/" _ y: power_term()   { (Op::Div, y) }
        / "%" _ y: power_term()   { (Op::Mod, y) }
        / "div" _ y: power_term() { (Op::IntDiv, y) }

        rule power_term() -> E = x:arithmetic_factor() _ n:exp_term()? {
            match n {
                Some((op, y)) => E::Dyadic(op, Box::new(x), Box::new(y)),
                None => x,
            }
        }

        rule exp_term() -> (Op, E) = "^" _ y:arithmetic_factor() { (Op::Exp, y) }

        rule arithmetic_factor() -> E
        = "(" _ x:arithmetic_expression() _ ")" { x }
        / "-" _ x:arithmetic_operand()          { E::Monadic(Op::Minus, Box::new(x)) }
        / x:arithmetic_operand()                { x }

        #[cache]
        rule arithmetic_operand() -> E
        = x:numeric_literal() { x }
        / x:function()        { x }
        / x:property_name()   { x }

        #[cache]
        pub(crate) rule property_name() -> E
        = a:$("\"" (!"\"" [_])* "\"") { E::Id(a.into()) }
        / b:ident()                   { E::Id(b.into()) }

        #[cache]
        rule function() -> E
        = name:ident() _ "(" _ params:argument_list() _ ")" { E::Function(Call::from(name, params)) }

        rule argument_list() -> Vec<E> = x:(argument() ** (_ "," _)) { x }

        #[cache]
        rule argument() -> E
        = x:character_clause()      { x }
        / x:temporal_instance()     { x }
        / x:spatial_instance()      { x }
        / x:array()                 { x }
        / x:arithmetic_expression() { x }
        / x:numeric_literal()       { x }
        / x:boolean_expression()    { x }
        / x:function()              { x }
        / x:property_name()         { x }

        pub(crate) rule character_expression() -> E
        = x:character_clause() { x }
        / x:function()         { x }
        / x:property_name()    { x }

        #[cache]
        rule character_clause() -> E
        = i("CASEI") _ "(" _ x:character_expression() _ ")"   { E::Monadic(Op::CaseI, Box::new(x)) }
        / i("ACCENTI") _ "(" _ x:character_expression() _ ")" { E::Monadic(Op::AccentI, Box::new(x)) }
        / x:character_literal()                               { x }

        #[cache]
        rule character_literal() -> E = "'" s:character()* "'" {
            let plain: String = s.iter().collect();
            E::Str(QString::plain(plain))
        }

        rule character() -> char
        = "''"            { '\'' }
        / "\\'"           { '\'' }
        / c:not_a_quote() { c }

        rule not_a_quote() -> char = !"'" c:[_] { c }

        #[cache]
        rule numeric_literal() -> E = n:(unsigned_num() / signed_num()) { E::Num(n) }

        #[cache]
        rule signed_num() -> f64 = s:['+' | '-']? n:unsigned_num() {
            let sign = match s {
                Some('+') => 1.0,
                Some('-') => -1.0,
                _ => 1.0,
            };
            sign * n
        }

        #[cache]
        rule unsigned_num() -> f64
        = n:$(['0'..='9']+ ("." ['0'..='9']*)? ( ['e'|'E'] ['+'|'-']? ['0'..='9']+ )?)
        { n.parse().unwrap() }

        #[cache]
        rule boolean_literal() -> E
        = i("TRUE")  { E::Bool(true) }
        / i("FALSE") { E::Bool(false) }

        rule temporal_instance() -> E = x:(instant_instance() / interval_instance()) { x }

        rule instant_instance() -> E = x:(date_instant() / timestamp_instant()) { x }

        rule date_instant() -> E = i("DATE") _ "(" _ x:date_instant_string() _ ")" { x }

        rule date_instant_string() -> E = "'" x:full_date() "'" { E::Date(x) }

        rule timestamp_instant() -> E = i("TIMESTAMP") _ "(" _ x:timestamp_instant_string() _ ")" { x }

        rule timestamp_instant_string() -> E = "'" x:utc_time() "'" { E::Timestamp(x) }

        #[cache]
        rule interval_instance() -> E
        = i("INTERVAL") _ "(" _ x:instant_parameter() _ "," _ y:instant_parameter() _ ")"
        { E::Interval(Box::new(x), Box::new(y)) }

        #[cache]
        rule instant_parameter() -> E
        = x:date_instant_string()      { x }
        / x:timestamp_instant_string() { x }
        / "'..'"                       { E::Unbounded }
        / x:function()                 { x }
        / x:property_name()            { x }

        #[cache]
        rule ident() -> &'input str
        = quiet! { s:$([
            '\u{003A}'                  // colon
            | '\u{005F}'                // underscore
            | '\u{0041}'..='\u{005A}'   // A-Z
            | '\u{0061}'..='\u{007A}'   // a-z
            | '\u{00C0}'..='\u{00D6}'   // À-Ö Latin-1 Supplement Letters
            | '\u{00D8}'..='\u{00F6}'   // Ø-ö Latin-1 Supplement Letters
            | '\u{00F8}'..='\u{02FF}'   // ø-ÿ Latin-1 Supplement Letters
            | '\u{0370}'..='\u{037D}'   // Ͱ-ͽ Greek and Coptic (without ";")
            | '\u{037F}'..='\u{1FFE}'   // See note 1.
            | '\u{200C}'..='\u{200D}'   // zero width non-joiner and joiner
            | '\u{2070}'..='\u{218F}'   // See note 2.
            | '\u{2C00}'..='\u{2FEF}'   // See note 3.
            | '\u{3001}'..='\u{D7FF}'   // See note 4.
            | '\u{F900}'..='\u{FDCF}'   // See note 5.
            | '\u{FDF0}'..='\u{FFFD}'   // See note 6.
            | '\u{10000}'..='\u{EFFFF}' // See note 7.
        ] [
            '\u{003A}'                  // colon
            | '\u{005F}'                // underscore
            | '\u{0041}'..='\u{005A}'   // A-Z
            | '\u{0061}'..='\u{007A}'   // a-z
            | '\u{00C0}'..='\u{00D6}'   // À-Ö Latin-1 Supplement Letters
            | '\u{00D8}'..='\u{00F6}'   // Ø-ö Latin-1 Supplement Letters
            | '\u{00F8}'..='\u{02FF}'   // ø-ÿ Latin-1 Supplement Letters
            | '\u{0370}'..='\u{037D}'   // Ͱ-ͽ Greek and Coptic (without ";")
            | '\u{037F}'..='\u{1FFE}'   // See note 1.
            | '\u{200C}'..='\u{200D}'   // zero width non-joiner and joiner
            | '\u{2070}'..='\u{218F}'   // See note 2.
            | '\u{2C00}'..='\u{2FEF}'   // See note 3.
            | '\u{3001}'..='\u{D7FF}'   // See note 4.
            | '\u{F900}'..='\u{FDCF}'   // See note 5.
            | '\u{FDF0}'..='\u{FFFD}'   // See note 6.
            | '\u{10000}'..='\u{EFFFF}' // See note 7.
            | '\u{002E}'                // dot
            | '0'..='9'
            | '\u{0300}'..='\u{036F}'  // combining and diacritical marks
            | '\u{203F}'..='\u{2040}'  // ‿ and ⁀
        ]*) { s } }

        #[cache]
        rule spatial_instance() -> E = g:wkt() { E::Spatial(g) }

        #[cache]
        pub(crate) rule wkt() -> G
        = g:geo_literal()             { g }
        / g:geo_collection_tagd_txt() { g }
        / g:bbox_tagd_txt()           { g }

        #[cache]
        rule geo_literal() -> G
        = g:point_tagd_txt() { g }
        / g:line_tagd_txt() { g }
        / g:poly_tagd_txt() { g }
        / g:multipoint_tagd_txt() { g }
        / g:multiline_tagd_txt() { g }
        / g:multipolygon_tagd_txt() { g }

        rule point_tagd_txt() -> G = i("POINT") _ ("Z" _)? g:point_txt() { G::Point(Point::from_xy(g)) }

        rule point_txt() -> Vec<f64> = "(" _ x:point() _ ")" { x }

        rule point() -> Vec<f64> = x:signed_num() **<2, 3> ([' ' | '\t' | '\x0C']*) { x }

        rule line_tagd_txt() -> G = i("LINESTRING") _ ("Z" _)? _ x:line_txt() { G::Line(Line::from_xy(x)) }

        rule line_txt() -> Vec<Vec<f64>> = "(" _ x:point() **<2,> (_ "," _) _ ")" { x }

        rule poly_tagd_txt() -> G = i("POLYGON") _ ("Z" _)? _ x:poly_txt() { G::Polygon(Polygon::from_xy(x)) }

        rule poly_txt() -> Vec<Vec<Vec<f64>>> = "(" _ x:ring_txt() ++ (_ "," _) _ ")" { x }

        // a linear ring is a closed line w/ at least 4 points...
        rule ring_txt() -> Vec<Vec<f64>> = "(" _ x:point() **<4,> (_ "," _) _ ")" { x }

        rule multipoint_tagd_txt() -> G
        = i("MULTIPOINT") _ ("Z" _)? x:multipoint_txt() { G::Points(Points::from_xy(x)) }

        // rule multipoint_txt() -> Vec<Vec<f64>> = "(" _ x:(point_txt() ++ (_ "," _)) _ ")" { x }
        rule multipoint_txt() -> Vec<Vec<f64>> = "(" _ x:(point_txt_forms() ++ (_ "," _)) _ ")" { x }

        rule point_txt_forms() -> Vec<f64>
        = "(" _ x:point() _ ")" { x }
        / x:point()             { x}

        rule multiline_tagd_txt() -> G
        = i("MULTILINESTRING") _ ("Z" _)? x:multiline_txt() { G::Lines(Lines::from_xy(x)) }

        rule multiline_txt() -> Vec<Vec<Vec<f64>>> = "(" _ x:(line_txt() ++ (_ "," _)) _ ")" { x }

        rule multipolygon_tagd_txt() -> G
        = i("MULTIPOLYGON") _ ("Z" _)? x:multipolygon_txt() { G::Polygons(Polygons::from_xy(x)) }

        rule multipolygon_txt() -> Vec<Vec<Vec<Vec<f64>>>> = "(" _ x:(poly_txt() ++ (_ "," _)) _ ")" { x }

        rule geo_collection_tagd_txt() -> G
        = i("GEOMETRYCOLLECTION") _ ("Z" _)? x:geo_collection_txt() { G::Vec(Geometries::from_items(x)) }

        rule geo_collection_txt() -> Vec<G> = "(" _ x:(geo_literal() ++ (_ "," _)) _ ")" { x }

        rule bbox_tagd_txt() -> G = i("BBOX") _ x:bbox_txt() { G::BBox(BBox::from(x)) }

        rule bbox_txt() -> Vec<f64> = "(" _ x:(signed_num() **<4,6> (_ "," _)) _ ")" { x }

        #[cache]
        rule full_date() -> Zoned
        = d:$(['0'..='9']*<4,4> "-" ['0'..='9']*<2,2> "-" ['0'..='9']*<2,2>)
        { d.parse::<Date>().unwrap().to_zoned(TimeZone::UTC).unwrap() }

        #[cache]
        rule utc_time() -> Zoned
        = z:$( ['0'..='9']*<4> "-" ['0'..='9']*<2> "-" ['0'..='9']*<2> "T" ['0'..='9']*<2> ":" ['0'..='9']*<2> ":" ['0'..='9']*<2> ("." ['0'..='9']+ )? _ "Z" )
        { (z.to_owned() + "[UTC]").parse::<Zoned>().unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Context, Resource,
        text::cql2::{
            character_expression, expression, geom_expression, is_like_predicate,
            numeric_expression, property_name, temporal_expression, temporal_predicate,
        },
    };
    use jiff::civil::DateTime;
    use rand::{
        Rng,
        distr::{Alphanumeric, Distribution, StandardUniform},
    };
    use std::error::Error;

    #[test]
    fn test_boolean() {
        assert_eq!(expression("TRUE"), Ok(E::Bool(true)));
        assert_eq!(expression("trUE"), Ok(E::Bool(true)));
        assert_eq!(expression("FALSE"), Ok(E::Bool(false)));
        assert_eq!(expression("falsE"), Ok(E::Bool(false)));
    }

    #[test]
    fn test_numeric_literal() {
        assert_eq!(numeric_expression("1.0"), Ok(E::Num(1.0)));
        assert_eq!(numeric_expression("1.0e2"), Ok(E::Num(100.0)));
        assert_eq!(numeric_expression("1e2"), Ok(E::Num(100.0)));
        assert_eq!(numeric_expression("1E3"), Ok(E::Num(1000.0)));
        assert_eq!(numeric_expression("0.1E2"), Ok(E::Num(10.0)));
        assert_eq!(numeric_expression("0.1e2"), Ok(E::Num(10.0)));
        assert_eq!(numeric_expression("+0.1e2"), Ok(E::Num(10.0)));
        assert_eq!(
            numeric_expression("-0.1e2"),
            Ok(E::Monadic(Op::Minus, Box::new(E::Num(10.0))))
        );
        assert_eq!(numeric_expression("1.0E-2"), Ok(E::Num(0.01)));
        assert_eq!(numeric_expression("1.0e-0"), Ok(E::Num(1.)));
    }

    #[test]
    fn test_identifier() {
        assert_eq!(property_name("the_geom"), Ok(E::Id("the_geom".into())));
    }

    #[test]
    fn test_quoted_identifier() {
        assert_eq!(
            property_name("\"the_geom\""),
            Ok(E::Id("\"the_geom\"".into()))
        );
    }

    #[test]
    fn test_date() {
        const T: &str = "Date('2010-02-10')";

        let exp = temporal_expression(T);
        assert!(exp.is_ok());
        let t = exp.unwrap();
        assert!(matches!(t, E::Date(_)));
        let zoned = match t {
            E::Date(x) => x,
            _ => panic!("Expected a date expression"),
        };
        // NOTE (rsn) 20250721 - i always store dates in UTC time-zone
        assert_eq!(zoned.to_string(), "2010-02-10T00:00:00+00:00[UTC]");
    }

    #[test]
    fn test_jiff_utc() {
        let t1 = "2012-08-10T05:30:00.123000Z".parse::<Zoned>();
        assert!(t1.is_err());
        let t2 = "2012-08-10T05:30:00.123000".parse::<DateTime>();
        assert!(t2.is_ok());
        let t3 = "2012-08-10T05:30:00.123000Z[UTC]".parse::<Zoned>();
        assert!(t3.is_ok())
    }

    #[test]
    fn test_timestamp() {
        const T: &str = "TimeStamp('2012-08-10T05:30:00.123000Z')";

        let exp = temporal_expression(T);
        assert!(exp.is_ok());
        let t = exp.unwrap();
        assert!(matches!(t, E::Timestamp(_)));
        let zoned = match t {
            E::Timestamp(x) => x,
            _ => panic!("Expected a timestamp expression"),
        };
        assert_eq!(zoned.to_string(), "2012-08-10T05:30:00.123+00:00[UTC]");
    }

    #[test]
    fn test_precedence() {
        peg::parser! {
            pub grammar testing() for str {
                rule _ = quiet!{ [' ' | '\t' | '\r' | '\n']* }

                #[cache_left_rec]
                pub rule expr() -> E = precedence! {
                    a:(@) _ "+" _ b:@ { E::Dyadic(Op::Plus, Box::new(a), Box::new(b)) }
                    a:(@) _ "-" _ b:@ { E::Dyadic(Op::Minus, Box::new(a), Box::new(b)) }
                    --
                    a:(@) _ "*" _ b:@ { E::Dyadic(Op::Mult, Box::new(a), Box::new(b)) }
                    a:(@) _ "/" _ b:@ { E::Dyadic(Op::Div, Box::new(a), Box::new(b)) }
                    a:(@) _ "%" _ b:@ { E::Dyadic(Op::Mod, Box::new(a), Box::new(b)) }
                    a:(@) _ "div" _ b:@ { E::Dyadic(Op::IntDiv, Box::new(a), Box::new(b)) }
                    --
                    a:@ _ "^" _ b:(@) { E::Dyadic(Op::Exp, Box::new(a), Box::new(b)) }
                    --
                    "(" _ x:expr() _ ")" { x }
                    n:numeric_literal() { n }
                    x:id() { x }
                }

                rule numeric_literal() -> E
                = n:(unsigned_num() / signed_num()) { E::Num(n) }

                rule signed_num() -> f64
                = s:['+' | '-']? n:unsigned_num() {
                    let sign = match s {
                        Some('+') => 1.0,
                        Some('-') => -1.0,
                        _ => 1.0,
                    };
                    sign * n
                }

                rule unsigned_num() -> f64
                = n:$(['0'..='9']+ ("." ['0'..='9']*)? ( ['e'|'E'] ['+'|'-']? ['0'..='9']+ )?)
                { n.parse().unwrap() }

                #[cache]
                rule ident() -> &'input str
                = quiet! { s:$([
                    '\u{003A}'                  // colon
                    | '\u{005F}'                // underscore
                    | '\u{0041}'..='\u{005A}'   // A-Z
                    | '\u{0061}'..='\u{007A}'   // a-z
                    | '\u{00C0}'..='\u{00D6}'   // À-Ö Latin-1 Supplement Letters
                    | '\u{00D8}'..='\u{00F6}'   // Ø-ö Latin-1 Supplement Letters
                    | '\u{00F8}'..='\u{02FF}'   // ø-ÿ Latin-1 Supplement Letters
                    | '\u{0370}'..='\u{037D}'   // Ͱ-ͽ Greek and Coptic (without ";")
                    | '\u{037F}'..='\u{1FFE}'   // See note 1.
                    | '\u{200C}'..='\u{200D}'   // zero width non-joiner and joiner
                    | '\u{2070}'..='\u{218F}'   // See note 2.
                    | '\u{2C00}'..='\u{2FEF}'   // See note 3.
                    | '\u{3001}'..='\u{D7FF}'   // See note 4.
                    | '\u{F900}'..='\u{FDCF}'   // See note 5.
                    | '\u{FDF0}'..='\u{FFFD}'   // See note 6.
                    | '\u{10000}'..='\u{EFFFF}' // See note 7.
                ] [
                    '\u{003A}'                  // colon
                    | '\u{005F}'                // underscore
                    | '\u{0041}'..='\u{005A}'   // A-Z
                    | '\u{0061}'..='\u{007A}'   // a-z
                    | '\u{00C0}'..='\u{00D6}'   // À-Ö Latin-1 Supplement Letters
                    | '\u{00D8}'..='\u{00F6}'   // Ø-ö Latin-1 Supplement Letters
                    | '\u{00F8}'..='\u{02FF}'   // ø-ÿ Latin-1 Supplement Letters
                    | '\u{0370}'..='\u{037D}'   // Ͱ-ͽ Greek and Coptic (without ";")
                    | '\u{037F}'..='\u{1FFE}'   // See note 1.
                    | '\u{200C}'..='\u{200D}'   // zero width non-joiner and joiner
                    | '\u{2070}'..='\u{218F}'   // See note 2.
                    | '\u{2C00}'..='\u{2FEF}'   // See note 3.
                    | '\u{3001}'..='\u{D7FF}'   // See note 4.
                    | '\u{F900}'..='\u{FDCF}'   // See note 5.
                    | '\u{FDF0}'..='\u{FFFD}'   // See note 6.
                    | '\u{10000}'..='\u{EFFFF}' // See note 7.
                    | '\u{002E}'                // dot
                    | '0'..='9'
                    | '\u{0300}'..='\u{036F}'  // combining and diacritical marks
                    | '\u{203F}'..='\u{2040}'  // ‿ and ⁀
                ]*) { s } }

                #[cache]
                rule id() -> E
                = a:$("\"" (!"\"" [_])* "\"") { E::Id(a.into()) }
                / a:ident() { E::Id(a.into()) }
            }
        }

        // const CQL: &str = r#"value = - foo * 2.0 + "bar" / 6.1234 - "x" ^ 2.0"#;
        const CQL: &str = r#"foo * 2.0 + "bar" / 6.1234 - "x" ^ 2.0"#;

        let exp = testing::expr(CQL);
        tracing::debug!("exp = {exp:?}");
        assert!(exp.is_ok());
    }

    #[test]
    fn test_character_expression() {
        const S1: &str = "%Foo%";

        let input = format!("'{S1}'");
        let exp = character_expression(&input);
        assert!(exp.is_ok());
        let c = exp.unwrap();
        assert!(matches!(c, E::Str(_)));
        let pattern = c.as_str().expect("Not a string");
        assert!(pattern.is_plain());
        assert_eq!(pattern.as_str(), S1);
    }

    #[test]
    fn test_is_like_predicate() {
        const S1: &str = "%Bar%";
        const P: &str = "foo LIKE '%Bar%'";

        let exp = is_like_predicate(P);
        assert!(exp.is_ok());
        let c = exp.unwrap();
        assert!(matches!(c, E::Dyadic(Op::IsLike, _, _)));
        let (_, x, y) = c.as_dyadic().expect("Not a dyadic");
        assert!(matches!(x, E::Id(_)));
        assert!(matches!(y, E::Str(_)));

        assert_eq!("foo", x.as_id().expect("Not a property name"));

        let pattern = y.as_str().expect("Not a string");
        assert!(pattern.is_plain());
        assert_eq!(S1, pattern.as_str());
    }

    #[test]
    fn test_is_not_like_predicate() {
        const S1: &str = "_Foo%";
        const P: &str = "\"name\" NOT LIKE '_Foo%'";

        let exp = is_like_predicate(P);
        assert!(exp.is_ok());
        let c = exp.unwrap();
        assert!(matches!(c, E::Dyadic(Op::IsNotLike, _, _)));
        let (_, x, y) = c.as_dyadic().expect("Not a dyadic");
        assert!(matches!(x, E::Id(_)));
        assert!(matches!(y, E::Str(_)));

        assert_eq!(r#""name""#, x.as_id().expect("Not a property name"));

        let pattern = y.as_str().expect("Not a string");
        assert!(pattern.is_plain());
        assert_eq!(S1, pattern.as_str());
    }

    #[test]
    fn test_t_before() {
        const F: &str = r#"t_before(foo, date('2025-07-14'))"#;

        let f1 = expression(F);
        assert!(f1.is_ok());

        let f2 = temporal_predicate(F);
        assert!(f2.is_ok());
    }

    #[test]
    fn test_escape_apostophe() {
        const TV: [(&str, &str); 11] = [
            ("\'abcdef\'", "abcdef"),
            (r#"'abc''def'"#, "abc'def"),
            (r#"'abc\'def'"#, "abc'def"),
            ("\'abc\u{0007}def\'", "abc\u{7}def"), // bell
            ("\'abc\u{0008}def\'", "abc\u{8}def"), // backspace
            ("\'abc\u{0009}def\'", "abc\tdef"),    // (horizontal) tab
            ("\'abc\u{000A}def\'", "abc\ndef"),    // newline
            (
                r#"'abc
def'"#,
                "abc\ndef",
            ), // newline
            ("\'abc\u{000B}def\'", "abc\u{b}def"), // vertical tab
            ("\'abc\u{000C}def\'", "abc\u{c}def"), // form-feed
            ("\'abc\u{000D}def\'", "abc\rdef"),    // carriage-return
        ];

        for (s, expected) in TV {
            // build an expression using test vectors...
            let input = format!(r#"{s}"#);
            let exp = character_expression(&input);
            assert!(exp.is_ok());
            let e = exp.unwrap();
            let x = e.as_str();
            assert!(x.is_some());
            let actual = x.unwrap();

            assert_eq!(actual.as_str(), expected);
        }
    }

    #[test]
    // #[tracing_test::traced_test]
    fn fuzz_test_escape_apostrophe() {
        fn random_chars() -> Vec<char> {
            let mut rng = rand::rng();
            let size = rng.random_range(5..50);
            let size = 1 + size % 50;
            let mut result = Vec::with_capacity(size);
            for _ in 0..size {
                let c = match rng.random_range(0..15) {
                    0 => '\u{07}', // BELL
                    1 => '\u{08}', // BACKSPACE
                    2 => '\u{09}', // HORIZONTAL TAB
                    3 => '\u{0A}', // NEWLINE
                    4 => '\u{0B}', // VERTICAL TAB
                    5 => '\u{0C}', // FORM FEED
                    6 => '\u{0D}', // CARRIAGE RETURN
                    7 => '\'',
                    8 => StandardUniform.sample(&mut rng),
                    _ => Alphanumeric.sample(&mut rng) as char,
                };
                result.push(c);
            }
            result
        }

        fn escape_it(s: &Vec<char>) -> String {
            let mut rng = rand::rng();
            let mut result = String::new();
            for c in s.iter() {
                match c {
                    '\'' => match rng.random_bool(0.5) {
                        true => result.push_str(r#"\'"#),
                        false => result.push_str("''"),
                    },
                    _ => result.push(*c),
                }
            }
            result
        }

        let mut failures = 0;
        for _ in 0..1000 {
            let raw = random_chars();
            let escaped = escape_it(&raw);
            let cooked = format!("'{}'", escaped);

            let exp = character_expression(&cooked);
            let s_raw = String::from_iter(raw);
            match exp {
                Ok(x) => {
                    let actual = x.as_str();
                    assert!(actual.is_some());
                    let actual_plain_str = actual.unwrap().as_str();
                    assert_eq!(actual_plain_str, s_raw);
                }
                Err(x) => {
                    tracing::error!("Failed: {x}\n* raw\n|{s_raw}|,\n* escaped\n|{escaped}|");
                    failures += 1;
                }
            }
        }
        assert_eq!(failures, 0);
    }

    #[test]
    fn test_combined1() {
        const F: &str = r#"
        (NOT (name<>'København') AND pop_other<>1038288) 
        OR (pop_other IS NULL) 
        or not (pop_other<>1038288 OR name<'København')"#;

        let expr = expression(F);
        assert!(expr.is_ok());
    }

    #[test]
    fn test_combined2() {
        // const F: &str = r#"(NOT (name<>'København') AND pop_other<>1038288) OR (pop_other IS NULL and name<'København') or not (pop_other<>1038288 OR name<'København')"#;
        // const F: &str = r#"(pop_other IS NULL and name<'København') or not (pop_other<>1038288 OR name<'København')"#;
        // const F: &str = r#"(pop_other IS NULL and name<'København')"#;
        const F: &str = r#"pop_other IS NULL and name<'København'"#;

        let expr = expression(F);
        tracing::debug!("expr = {expr:?}");
        assert!(expr.is_ok());
    }

    // NOTE (rsn) 20250811 - the BNF imply that a MULTIPOINT tagged text must
    // have each point coordinates surrounded by parenthesis; e.g. E1 in the
    // test unit vector.  however the CQL2 specs when giving test vector data
    // do not abide by this.  furthermore, this Wikipedia link gives both forms
    // as valid representation of this type geometry; i.e. point coordinates
    // w/ and w/o surrounding parens...
    // today i changed the PEG rules to allow both...
    #[test]
    fn test_modified_multipoint() -> Result<(), Box<dyn Error>> {
        const E1: &str = "MULTIPOINT((7 50),(10 51))";
        const E2: &str = "MULTIPOINT(7 50, 10 51)";

        let expr1 = geom_expression(E1)?;
        let e1 = expr1.as_spatial().unwrap();
        assert!(matches!(e1, G::Points(_)));
        match e1 {
            G::Points(mp1) => {
                assert_eq!(mp1.num_points(), 2);
            }
            _ => panic!("Expected a multi-point geometry. Abort"),
        }

        let expr2 = geom_expression(E2)?;
        let e2 = expr2.as_spatial().unwrap();
        assert!(matches!(e2, G::Points(_)));
        match e2 {
            G::Points(mp2) => {
                assert_eq!(mp2.num_points(), 2);
            }
            _ => panic!("Expected a multi-point geometry. Abort"),
        }

        Ok(())
    }

    #[test]
    fn test_current_precedence() -> Result<(), Box<dyn Error>> {
        // * has higher precedence compared to + so w/o parenthesis
        // a * b + c must be evaluated as (a * b) + c...
        const E: &str = "3013259 = 30*100000+13259";

        let expr = expression(E)?;

        let res = expr.eval(&Context::new(), &Resource::new())?;
        assert!(res.to_bool()?);

        Ok(())
    }
}
