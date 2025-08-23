// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]
#![allow(clippy::redundant_closure_call)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::enum_variant_names)]

use crate::{MyError, geom};
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::Value;
use serde_with::{DisplayFromStr, serde_as};
use std::{ops::Deref, str::FromStr};

/// JSON-encoded CQL2 Expression.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum Expression {
    AndOrExpression(AndOrExpression),
    NotExpression(NotExpression),
    ComparisonPredicate(ComparisonPredicate),
    SpatialPredicate(SpatialPredicate),
    TemporalPredicate(TemporalPredicate),
    ArrayPredicate(ArrayPredicate),
    FunctionRef(FunctionRef),
    Boolean(bool),
    // NOTE (rsn) 20250620 - added to cater for a filter being only something
    // like... `{ "property": "foo" }` or any other atom...
    PropertyRef(PropertyRef),
    // caters for array items that are temporal | spatial | char literals...
    TemporalInstance(TemporalInstance),
    SpatialInstance(SpatialInstance),
    CharacterExpression(CharacterExpression),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::AndOrExpression(x) => write!(f, "{x}"),
            Expression::NotExpression(x) => write!(f, "{x}"),
            Expression::ComparisonPredicate(x) => write!(f, "{x}"),
            Expression::SpatialPredicate(x) => write!(f, "{x}"),
            Expression::TemporalPredicate(x) => write!(f, "{x}"),
            Expression::ArrayPredicate(x) => write!(f, "{x}"),
            Expression::FunctionRef(x) => write!(f, "{x}"),
            Expression::Boolean(x) => write!(f, "{}", if *x { "TRUE" } else { "FALSE" }),

            Expression::PropertyRef(x) => write!(f, "{x}"),
            Expression::TemporalInstance(x) => write!(f, "{x}"),
            Expression::SpatialInstance(x) => write!(f, "{x}"),
            Expression::CharacterExpression(x) => write!(f, "{x}"),
        }
    }
}

#[doc = "`Accenti`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"oneOf\": ["]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/functionRef\""]
#[doc = "          }"]
#[doc = "        ]"]
#[doc = "      },"]
#[doc = "      \"maxItems\": 1,"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"accenti\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Accenti {
    pub(crate) args: [AccentiArgsItem; 1usize],
    pub(crate) op: AccentiOp,
}

impl fmt::Display for Accenti {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ACCENTI({})", self.args[0])
    }
}

impl From<&Accenti> for Accenti {
    fn from(value: &Accenti) -> Self {
        value.clone()
    }
}

#[doc = "`AccentiArgsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/functionRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum AccentiArgsItem {
    CharacterExpression(CharacterExpression),
    PropertyRef(PropertyRef),
    FunctionRef(FunctionRef),
}

impl fmt::Display for AccentiArgsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccentiArgsItem::CharacterExpression(x) => write!(f, "{x}"),
            AccentiArgsItem::PropertyRef(x) => write!(f, "{x}"),
            AccentiArgsItem::FunctionRef(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for AccentiArgsItem {
    fn from(value: &AccentiArgsItem) -> Self {
        value.clone()
    }
}
impl From<CharacterExpression> for AccentiArgsItem {
    fn from(value: CharacterExpression) -> Self {
        Self::CharacterExpression(value)
    }
}
impl From<PropertyRef> for AccentiArgsItem {
    fn from(value: PropertyRef) -> Self {
        Self::PropertyRef(value)
    }
}
impl From<FunctionRef> for AccentiArgsItem {
    fn from(value: FunctionRef) -> Self {
        Self::FunctionRef(value)
    }
}
#[doc = "`AccentiOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"accenti\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum AccentiOp {
    #[serde(rename = "accenti")]
    Accenti,
}
impl From<&Self> for AccentiOp {
    fn from(value: &AccentiOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for AccentiOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Accenti => write!(f, "accenti"),
        }
    }
}
impl FromStr for AccentiOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "accenti" => Ok(Self::Accenti),
            _ => Err(MyError::Runtime("Expected ACCENTI".into())),
        }
    }
}
impl TryFrom<&str> for AccentiOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for AccentiOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for AccentiOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`AndOrExpression`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$dynamicRef\": \"#cql2expression\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 2"]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"and\","]
#[doc = "        \"or\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AndOrExpression {
    pub(crate) args: Vec<Value>,
    pub(crate) op: AndOrExpressionOp,
}

impl fmt::Display for AndOrExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let zargs: Vec<String> = self
            .args
            .iter()
            .map(|x| {
                let y: Expression = serde_json::from_value(x.clone()).expect("Expected expression");
                y.to_string()
            })
            .collect();
        let zop = format!(" {} ", self.op);
        write!(f, "{}", zargs.join(zop.as_str()))
    }
}

impl From<&AndOrExpression> for AndOrExpression {
    fn from(value: &AndOrExpression) -> Self {
        value.clone()
    }
}

#[doc = "`AndOrExpressionOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"and\","]
#[doc = "    \"or\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum AndOrExpressionOp {
    #[serde(rename = "and")]
    And,
    #[serde(rename = "or")]
    Or,
}
impl From<&Self> for AndOrExpressionOp {
    fn from(value: &AndOrExpressionOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for AndOrExpressionOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
        }
    }
}
impl FromStr for AndOrExpressionOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "and" => Ok(Self::And),
            "or" => Ok(Self::Or),
            _ => Err(MyError::Runtime("Expected AND | OR".into())),
        }
    }
}
impl TryFrom<&str> for AndOrExpressionOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for AndOrExpressionOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for AndOrExpressionOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`ArithmeticExpression`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"$ref\": \"#/$defs/arithmeticOperands\""]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"Plus\","]
#[doc = "        \"Minus\","]
#[doc = "        \"Mult\","]
#[doc = "        \"Div\","]
#[doc = "        \"Exp\","]
#[doc = "        \"Mod\","]
#[doc = "        \"IntDiv\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct ArithmeticExpression {
    pub(crate) args: ArithmeticOperands,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) op: ArithmeticExpressionOp,
}

impl fmt::Display for ArithmeticExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.args.0[0], self.op, self.args.0[1])
    }
}

impl From<&ArithmeticExpression> for ArithmeticExpression {
    fn from(value: &ArithmeticExpression) -> Self {
        value.clone()
    }
}

#[doc = "`ArithmeticExpressionOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"Plus\","]
#[doc = "    \"Minus\","]
#[doc = "    \"Mult\","]
#[doc = "    \"Div\","]
#[doc = "    \"Exp\","]
#[doc = "    \"Mod\","]
#[doc = "    \"IntDiv\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ArithmeticExpressionOp {
    Plus,
    Minus,
    Mult,
    Div,
    Exp,
    Mod,
    IntDiv,
}
impl From<&Self> for ArithmeticExpressionOp {
    fn from(value: &ArithmeticExpressionOp) -> Self {
        value.clone()
    }
}

impl fmt::Display for ArithmeticExpressionOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Mult => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Exp => write!(f, "^"),
            Self::Mod => write!(f, "%"),
            Self::IntDiv => write!(f, "div"),
        }
    }
}

impl FromStr for ArithmeticExpressionOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "+" => Ok(Self::Plus),
            "-" => Ok(Self::Minus),
            "*" => Ok(Self::Mult),
            "/" => Ok(Self::Div),
            "^" => Ok(Self::Exp),
            "%" => Ok(Self::Mod),
            "div" => Ok(Self::IntDiv),
            _ => Err(MyError::Runtime(
                "Expected + | - | * | / | ^ | % | div".into(),
            )),
        }
    }
}
impl TryFrom<&str> for ArithmeticExpressionOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for ArithmeticExpressionOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for ArithmeticExpressionOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`ArithmeticOperands`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"oneOf\": ["]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/arithmeticExpression\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/functionRef\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      }"]
#[doc = "    ]"]
#[doc = "  },"]
#[doc = "  \"maxItems\": 2,"]
#[doc = "  \"minItems\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct ArithmeticOperands(pub(crate) [ArithmeticOperandsItem; 2usize]);
impl Deref for ArithmeticOperands {
    type Target = [ArithmeticOperandsItem; 2usize];
    fn deref(&self) -> &[ArithmeticOperandsItem; 2usize] {
        &self.0
    }
}
impl From<ArithmeticOperands> for [ArithmeticOperandsItem; 2usize] {
    fn from(value: ArithmeticOperands) -> Self {
        value.0
    }
}
impl From<&ArithmeticOperands> for ArithmeticOperands {
    fn from(value: &ArithmeticOperands) -> Self {
        value.clone()
    }
}
impl From<[ArithmeticOperandsItem; 2usize]> for ArithmeticOperands {
    fn from(value: [ArithmeticOperandsItem; 2usize]) -> Self {
        Self(value)
    }
}
#[doc = "`ArithmeticOperandsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/arithmeticExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/functionRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum ArithmeticOperandsItem {
    Variant0(Box<ArithmeticExpression>),
    Variant1(PropertyRef),
    Variant2(FunctionRef),
    Variant3(f64),
}

impl fmt::Display for ArithmeticOperandsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticOperandsItem::Variant0(x) => write!(f, "({x})"),
            ArithmeticOperandsItem::Variant1(x) => write!(f, "{x}"),
            ArithmeticOperandsItem::Variant2(x) => write!(f, "{x}"),
            ArithmeticOperandsItem::Variant3(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for ArithmeticOperandsItem {
    fn from(value: &ArithmeticOperandsItem) -> Self {
        value.clone()
    }
}
impl From<Box<ArithmeticExpression>> for ArithmeticOperandsItem {
    fn from(value: Box<ArithmeticExpression>) -> Self {
        Self::Variant0(value)
    }
}
impl From<PropertyRef> for ArithmeticOperandsItem {
    fn from(value: PropertyRef) -> Self {
        Self::Variant1(value)
    }
}
impl From<FunctionRef> for ArithmeticOperandsItem {
    fn from(value: FunctionRef) -> Self {
        Self::Variant2(value)
    }
}
impl From<f64> for ArithmeticOperandsItem {
    fn from(value: f64) -> Self {
        Self::Variant3(value)
    }
}
#[doc = "`Array`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"oneOf\": ["]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/numericExpression\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$dynamicRef\": \"#cql2expression\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/spatialInstance\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/temporalInstance\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/array\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "      }"]
#[doc = "    ]"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct Array(pub(crate) Vec<ArrayItem>);
impl Deref for Array {
    type Target = Vec<ArrayItem>;
    fn deref(&self) -> &Vec<ArrayItem> {
        &self.0
    }
}

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> = self.0.iter().map(|x| x.to_string()).collect();
        write!(f, "{}", items.join(", "))
    }
}

impl From<Array> for Vec<ArrayItem> {
    fn from(value: Array) -> Self {
        value.0
    }
}
impl From<&Array> for Array {
    fn from(value: &Array) -> Self {
        value.clone()
    }
}
impl From<Vec<ArrayItem>> for Array {
    fn from(value: Vec<ArrayItem>) -> Self {
        Self(value)
    }
}
#[doc = "`ArrayExpression`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"oneOf\": ["]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/array\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/functionRef\""]
#[doc = "      }"]
#[doc = "    ]"]
#[doc = "  },"]
#[doc = "  \"maxItems\": 2,"]
#[doc = "  \"minItems\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct ArrayExpression(pub(crate) [ArrayExpressionItem; 2usize]);
impl Deref for ArrayExpression {
    type Target = [ArrayExpressionItem; 2usize];
    fn deref(&self) -> &[ArrayExpressionItem; 2usize] {
        &self.0
    }
}
impl From<ArrayExpression> for [ArrayExpressionItem; 2usize] {
    fn from(value: ArrayExpression) -> Self {
        value.0
    }
}
impl From<&ArrayExpression> for ArrayExpression {
    fn from(value: &ArrayExpression) -> Self {
        value.clone()
    }
}
impl From<[ArrayExpressionItem; 2usize]> for ArrayExpression {
    fn from(value: [ArrayExpressionItem; 2usize]) -> Self {
        Self(value)
    }
}
#[doc = "`ArrayExpressionItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/array\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/functionRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum ArrayExpressionItem {
    Array(Array),
    PropertyRef(PropertyRef),
    FunctionRef(FunctionRef),
}

impl fmt::Display for ArrayExpressionItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayExpressionItem::Array(x) => write!(f, "({x})"),
            ArrayExpressionItem::PropertyRef(x) => write!(f, "{x}"),
            ArrayExpressionItem::FunctionRef(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for ArrayExpressionItem {
    fn from(value: &ArrayExpressionItem) -> Self {
        value.clone()
    }
}
impl From<Array> for ArrayExpressionItem {
    fn from(value: Array) -> Self {
        Self::Array(value)
    }
}
impl From<PropertyRef> for ArrayExpressionItem {
    fn from(value: PropertyRef) -> Self {
        Self::PropertyRef(value)
    }
}
impl From<FunctionRef> for ArrayExpressionItem {
    fn from(value: FunctionRef) -> Self {
        Self::FunctionRef(value)
    }
}
#[doc = "`ArrayItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/numericExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$dynamicRef\": \"#cql2expression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/spatialInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/temporalInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/array\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum ArrayItem {
    Variant0(CharacterExpression),
    Variant1(NumericExpression),
    Variant2(Value),
    Variant3(SpatialInstance),
    Variant4(TemporalInstance),
    Variant5(Array),
    Variant6(PropertyRef),
}

impl fmt::Display for ArrayItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayItem::Variant0(x) => write!(f, "{x}"),
            ArrayItem::Variant1(x) => write!(f, "{x}"),
            ArrayItem::Variant2(x) => {
                let x: Expression = serde_json::from_value(x.clone()).expect("Expected expression");
                write!(f, "{x}")
            }
            ArrayItem::Variant3(x) => write!(f, "{x}"),
            ArrayItem::Variant4(x) => write!(f, "{x}"),
            ArrayItem::Variant5(x) => write!(f, "{x}"),
            ArrayItem::Variant6(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for ArrayItem {
    fn from(value: &ArrayItem) -> Self {
        value.clone()
    }
}
impl From<CharacterExpression> for ArrayItem {
    fn from(value: CharacterExpression) -> Self {
        Self::Variant0(value)
    }
}
impl From<NumericExpression> for ArrayItem {
    fn from(value: NumericExpression) -> Self {
        Self::Variant1(value)
    }
}
impl From<Value> for ArrayItem {
    fn from(value: Value) -> Self {
        Self::Variant2(value)
    }
}
impl From<SpatialInstance> for ArrayItem {
    fn from(value: SpatialInstance) -> Self {
        Self::Variant3(value)
    }
}
impl From<TemporalInstance> for ArrayItem {
    fn from(value: TemporalInstance) -> Self {
        Self::Variant4(value)
    }
}
impl From<Array> for ArrayItem {
    fn from(value: Array) -> Self {
        Self::Variant5(value)
    }
}
impl From<PropertyRef> for ArrayItem {
    fn from(value: PropertyRef) -> Self {
        Self::Variant6(value)
    }
}
#[doc = "`ArrayPredicate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"$ref\": \"#/$defs/arrayExpression\""]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"a_containedBy\","]
#[doc = "        \"a_contains\","]
#[doc = "        \"a_equals\","]
#[doc = "        \"a_overlaps\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ArrayPredicate {
    pub(crate) args: ArrayExpression,
    pub(crate) op: ArrayPredicateOp,
}

impl fmt::Display for ArrayPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({}, {})", self.op, self.args.0[0], self.args.0[1])
    }
}

impl From<&ArrayPredicate> for ArrayPredicate {
    fn from(value: &ArrayPredicate) -> Self {
        value.clone()
    }
}

#[doc = "`ArrayPredicateOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"a_containedBy\","]
#[doc = "    \"a_contains\","]
#[doc = "    \"a_equals\","]
#[doc = "    \"a_overlaps\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ArrayPredicateOp {
    #[serde(rename = "a_containedBy")]
    AContainedBy,
    #[serde(rename = "a_contains")]
    AContains,
    #[serde(rename = "a_equals")]
    AEquals,
    #[serde(rename = "a_overlaps")]
    AOverlaps,
}
impl From<&Self> for ArrayPredicateOp {
    fn from(value: &ArrayPredicateOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for ArrayPredicateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::AContainedBy => write!(f, "a_containedBy"),
            Self::AContains => write!(f, "a_contains"),
            Self::AEquals => write!(f, "a_equals"),
            Self::AOverlaps => write!(f, "a_overlaps"),
        }
    }
}
impl FromStr for ArrayPredicateOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "a_containedBy" => Ok(Self::AContainedBy),
            "a_contains" => Ok(Self::AContains),
            "a_equals" => Ok(Self::AEquals),
            "a_overlaps" => Ok(Self::AOverlaps),
            _ => Err(MyError::Runtime("Expected array function".into())),
        }
    }
}
impl TryFrom<&str> for ArrayPredicateOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for ArrayPredicateOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for ArrayPredicateOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`Bbox`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"maxItems\": 4,"]
#[doc = "      \"minItems\": 4"]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"maxItems\": 6,"]
#[doc = "      \"minItems\": 6"]
#[doc = "    }"]
#[doc = "  ],"]
#[doc = "  \"items\": {"]
#[doc = "    \"type\": \"number\""]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum Bbox {
    Variant0(Vec<f64>),
    Variant1(Vec<f64>),
}

impl fmt::Display for Bbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let coords: Vec<String> = match self {
            Bbox::Variant0(x) | Bbox::Variant1(x) => x.iter().map(|x| x.to_string()).collect(),
        };
        write!(f, "({})", coords.join(", "))
    }
}

impl From<&Self> for Bbox {
    fn from(value: &Bbox) -> Self {
        value.clone()
    }
}
#[doc = "`BboxLiteral`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"bbox\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bbox\": {"]
#[doc = "      \"$ref\": \"#/$defs/bbox\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct BboxLiteral {
    pub(crate) bbox: Bbox,
}

impl fmt::Display for BboxLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bbox)
    }
}

impl From<&BboxLiteral> for BboxLiteral {
    fn from(value: &BboxLiteral) -> Self {
        value.clone()
    }
}

#[doc = "`BinaryComparisonPredicate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"$ref\": \"#/$defs/scalarOperands\""]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"Eq\","]
#[doc = "        \"Neq\","]
#[doc = "        \"Lt\","]
#[doc = "        \"Gt\","]
#[doc = "        \"Lte\","]
#[doc = "        \"Gte\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BinaryComparisonPredicate {
    pub(crate) args: ScalarOperands,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) op: BinaryComparisonPredicateOp,
}

impl fmt::Display for BinaryComparisonPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE (rsn) 20250706 - if 'b' is a literal output as is; otherwise
        // surround it w/ parentheses
        let b = &self.args.0[1];

        write!(f, "{} {} {}", self.args.0[0], self.op, b)
    }
}

impl From<&BinaryComparisonPredicate> for BinaryComparisonPredicate {
    fn from(value: &BinaryComparisonPredicate) -> Self {
        value.clone()
    }
}

#[doc = "`BinaryComparisonPredicateOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"Eq\","]
#[doc = "    \"Neq\","]
#[doc = "    \"Lt\","]
#[doc = "    \"Gt\","]
#[doc = "    \"Lte\","]
#[doc = "    \"Gte\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]

pub(crate) enum BinaryComparisonPredicateOp {
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
}
impl From<&Self> for BinaryComparisonPredicateOp {
    fn from(value: &BinaryComparisonPredicateOp) -> Self {
        value.clone()
    }
}

impl fmt::Display for BinaryComparisonPredicateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Eq => write!(f, "="),
            Self::Neq => write!(f, "<>"),
            Self::Lt => write!(f, "<"),
            Self::Gt => write!(f, ">"),
            Self::Lte => write!(f, "<="),
            Self::Gte => write!(f, ">="),
        }
    }
}

impl FromStr for BinaryComparisonPredicateOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "=" => Ok(Self::Eq),
            "<>" => Ok(Self::Neq),
            "<" => Ok(Self::Lt),
            ">" => Ok(Self::Gt),
            "<=" => Ok(Self::Lte),
            ">=" => Ok(Self::Gte),
            _ => Err(MyError::Runtime("Expected = | <> | < | > | <= | >=".into())),
        }
    }
}

impl TryFrom<&str> for BinaryComparisonPredicateOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for BinaryComparisonPredicateOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for BinaryComparisonPredicateOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`Casei`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"oneOf\": ["]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/functionRef\""]
#[doc = "          }"]
#[doc = "        ]"]
#[doc = "      },"]
#[doc = "      \"maxItems\": 1,"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"casei\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Casei {
    pub(crate) args: [CaseiArgsItem; 1usize],
    pub(crate) op: CaseiOp,
}

impl fmt::Display for Casei {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CASEI({})", self.args[0])
    }
}

impl From<&Casei> for Casei {
    fn from(value: &Casei) -> Self {
        value.clone()
    }
}

#[doc = "`CaseiArgsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/functionRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum CaseiArgsItem {
    CharacterExpression(Box<CharacterExpression>),
    PropertyRef(PropertyRef),
    FunctionRef(FunctionRef),
}

impl fmt::Display for CaseiArgsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaseiArgsItem::CharacterExpression(x) => write!(f, "{x}"),
            CaseiArgsItem::PropertyRef(x) => write!(f, "{x}"),
            CaseiArgsItem::FunctionRef(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for CaseiArgsItem {
    fn from(value: &CaseiArgsItem) -> Self {
        value.clone()
    }
}
impl From<Box<CharacterExpression>> for CaseiArgsItem {
    fn from(value: Box<CharacterExpression>) -> Self {
        Self::CharacterExpression(value)
    }
}
impl From<PropertyRef> for CaseiArgsItem {
    fn from(value: PropertyRef) -> Self {
        Self::PropertyRef(value)
    }
}
impl From<FunctionRef> for CaseiArgsItem {
    fn from(value: FunctionRef) -> Self {
        Self::FunctionRef(value)
    }
}
#[doc = "`CaseiOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"casei\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum CaseiOp {
    #[serde(rename = "casei")]
    Casei,
}
impl From<&Self> for CaseiOp {
    fn from(value: &CaseiOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for CaseiOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Casei => write!(f, "casei"),
        }
    }
}
impl FromStr for CaseiOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "casei" => Ok(Self::Casei),
            _ => Err(MyError::Runtime("Expected CASEI".into())),
        }
    }
}
impl TryFrom<&str> for CaseiOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for CaseiOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for CaseiOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`CharacterExpression`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/casei\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/accenti\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum CharacterExpression {
    Variant0(Casei),
    Variant1(Box<Accenti>),
    Variant2(String),
}

impl fmt::Display for CharacterExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CharacterExpression::Variant0(x) => write!(f, "{x}"),
            CharacterExpression::Variant1(x) => write!(f, "{x}"),
            CharacterExpression::Variant2(x) => write!(f, "'{x}'"),
        }
    }
}

impl From<&Self> for CharacterExpression {
    fn from(value: &CharacterExpression) -> Self {
        value.clone()
    }
}
impl From<Casei> for CharacterExpression {
    fn from(value: Casei) -> Self {
        Self::Variant0(value)
    }
}
impl From<Box<Accenti>> for CharacterExpression {
    fn from(value: Box<Accenti>) -> Self {
        Self::Variant1(value)
    }
}
#[doc = "`ComparisonPredicate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/binaryComparisonPredicate\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/isLikePredicate\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/isBetweenPredicate\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/isInListPredicate\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/isNullPredicate\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComparisonPredicate {
    BinaryComparisonPredicate(BinaryComparisonPredicate),
    IsLikePredicate(IsLikePredicate),
    IsBetweenPredicate(IsBetweenPredicate),
    IsInListPredicate(IsInListPredicate),
    IsNullPredicate(IsNullPredicate),
}

impl fmt::Display for ComparisonPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonPredicate::BinaryComparisonPredicate(x) => write!(f, "{x}"),
            ComparisonPredicate::IsLikePredicate(x) => write!(f, "{x}"),
            ComparisonPredicate::IsBetweenPredicate(x) => write!(f, "{x}"),
            ComparisonPredicate::IsInListPredicate(x) => write!(f, "{x}"),
            ComparisonPredicate::IsNullPredicate(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for ComparisonPredicate {
    fn from(value: &ComparisonPredicate) -> Self {
        value.clone()
    }
}
impl From<BinaryComparisonPredicate> for ComparisonPredicate {
    fn from(value: BinaryComparisonPredicate) -> Self {
        Self::BinaryComparisonPredicate(value)
    }
}
impl From<IsLikePredicate> for ComparisonPredicate {
    fn from(value: IsLikePredicate) -> Self {
        Self::IsLikePredicate(value)
    }
}
impl From<IsBetweenPredicate> for ComparisonPredicate {
    fn from(value: IsBetweenPredicate) -> Self {
        Self::IsBetweenPredicate(value)
    }
}
impl From<IsInListPredicate> for ComparisonPredicate {
    fn from(value: IsInListPredicate) -> Self {
        Self::IsInListPredicate(value)
    }
}
impl From<IsNullPredicate> for ComparisonPredicate {
    fn from(value: IsNullPredicate) -> Self {
        Self::IsNullPredicate(value)
    }
}
#[doc = "`DateInstant`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"date\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"date\": {"]
#[doc = "      \"$ref\": \"#/$defs/dateString\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct DateInstant {
    pub(crate) date: DateString,
}

impl fmt::Display for DateInstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DATE({})", self.date)
    }
}

impl From<&DateInstant> for DateInstant {
    fn from(value: &DateInstant) -> Self {
        value.clone()
    }
}

#[doc = "`DateString`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^\\\\d{4}-\\\\d{2}-\\\\d{2}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub(crate) struct DateString(String);
impl Deref for DateString {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl fmt::Display for DateString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

impl From<DateString> for String {
    fn from(value: DateString) -> Self {
        value.0
    }
}
impl From<&DateString> for DateString {
    fn from(value: &DateString) -> Self {
        value.clone()
    }
}
impl FromStr for DateString {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^\\d{4}-\\d{2}-\\d{2}$").unwrap());
        if (*PATTERN).find(value).is_none() {
            return Err(MyError::Runtime("Expected date string".into()));
        }
        Ok(Self(value.to_string()))
    }
}
impl TryFrom<&str> for DateString {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for DateString {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for DateString {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for DateString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: MyError| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`FunctionRef`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"oneOf\": ["]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/numericExpression\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$dynamicRef\": \"#cql2expression\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/spatialInstance\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/temporalInstance\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/array\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "          }"]
#[doc = "        ]"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"not\": {"]
#[doc = "        \"enum\": ["]
#[doc = "          \"and\","]
#[doc = "          \"or\","]
#[doc = "          \"not\","]
#[doc = "          \"=\","]
#[doc = "          \"<>\","]
#[doc = "          \"<\","]
#[doc = "          \">\","]
#[doc = "          \"<=\","]
#[doc = "          \">=\","]
#[doc = "          \"like\","]
#[doc = "          \"between\","]
#[doc = "          \"in\","]
#[doc = "          \"isNull\","]
#[doc = "          \"casei\","]
#[doc = "          \"accenti\","]
#[doc = "          \"s_contains\","]
#[doc = "          \"s_crosses\","]
#[doc = "          \"s_disjoint\","]
#[doc = "          \"s_equals\","]
#[doc = "          \"s_intersects\","]
#[doc = "          \"s_overlaps\","]
#[doc = "          \"s_touches\","]
#[doc = "          \"s_within\","]
#[doc = "          \"t_after\","]
#[doc = "          \"t_before\","]
#[doc = "          \"t_contains\","]
#[doc = "          \"t_disjoint\","]
#[doc = "          \"t_during\","]
#[doc = "          \"t_equals\","]
#[doc = "          \"t_finishedBy\","]
#[doc = "          \"t_finishes\","]
#[doc = "          \"t_intersects\","]
#[doc = "          \"t_meets\","]
#[doc = "          \"t_metBy\","]
#[doc = "          \"t_overlappedBy\","]
#[doc = "          \"t_overlaps\","]
#[doc = "          \"t_startedBy\","]
#[doc = "          \"t_starts\","]
#[doc = "          \"a_containedBy\","]
#[doc = "          \"a_contains\","]
#[doc = "          \"a_equals\","]
#[doc = "          \"a_overlaps\","]
#[doc = "          \"+\","]
#[doc = "          \"-\","]
#[doc = "          \"*\","]
#[doc = "          \"/\","]
#[doc = "          \"^\","]
#[doc = "          \"%\","]
#[doc = "          \"div\""]
#[doc = "        ]"]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FunctionRef {
    pub(crate) args: Vec<FunctionRefArgsItem>,
    pub(crate) op: FunctionRefOp,
}

impl fmt::Display for FunctionRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<String> = self.args.iter().map(|x| x.to_string()).collect();
        write!(f, "{}({})", self.op.0, args.join(", "))
    }
}

impl From<&FunctionRef> for FunctionRef {
    fn from(value: &FunctionRef) -> Self {
        value.clone()
    }
}

#[doc = "`FunctionRefArgsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/numericExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$dynamicRef\": \"#cql2expression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/spatialInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/temporalInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/array\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum FunctionRefArgsItem {
    Variant0(CharacterExpression),
    Variant1(NumericExpression),
    Variant2(Value),
    Variant3(SpatialInstance),
    Variant4(TemporalInstance),
    Variant5(Array),
    Variant6(PropertyRef),
}

impl fmt::Display for FunctionRefArgsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionRefArgsItem::Variant0(x) => write!(f, "{x}"),
            FunctionRefArgsItem::Variant1(x) => write!(f, "{x}"),
            FunctionRefArgsItem::Variant2(x) => {
                let x: Expression = serde_json::from_value(x.clone()).expect("Expected expression");
                write!(f, "{x}")
            }
            FunctionRefArgsItem::Variant3(x) => write!(f, "{x}"),
            FunctionRefArgsItem::Variant4(x) => write!(f, "{x}"),
            FunctionRefArgsItem::Variant5(x) => write!(f, "{x}"),
            FunctionRefArgsItem::Variant6(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for FunctionRefArgsItem {
    fn from(value: &FunctionRefArgsItem) -> Self {
        value.clone()
    }
}
impl From<CharacterExpression> for FunctionRefArgsItem {
    fn from(value: CharacterExpression) -> Self {
        Self::Variant0(value)
    }
}
impl From<NumericExpression> for FunctionRefArgsItem {
    fn from(value: NumericExpression) -> Self {
        Self::Variant1(value)
    }
}
impl From<Value> for FunctionRefArgsItem {
    fn from(value: Value) -> Self {
        Self::Variant2(value)
    }
}
impl From<SpatialInstance> for FunctionRefArgsItem {
    fn from(value: SpatialInstance) -> Self {
        Self::Variant3(value)
    }
}
impl From<TemporalInstance> for FunctionRefArgsItem {
    fn from(value: TemporalInstance) -> Self {
        Self::Variant4(value)
    }
}
impl From<Array> for FunctionRefArgsItem {
    fn from(value: Array) -> Self {
        Self::Variant5(value)
    }
}
impl From<PropertyRef> for FunctionRefArgsItem {
    fn from(value: PropertyRef) -> Self {
        Self::Variant6(value)
    }
}
#[doc = "`FunctionRefOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"not\": {"]
#[doc = "    \"enum\": ["]
#[doc = "      \"and\","]
#[doc = "      \"or\","]
#[doc = "      \"not\","]
#[doc = "      \"=\","]
#[doc = "      \"<>\","]
#[doc = "      \"<\","]
#[doc = "      \">\","]
#[doc = "      \"<=\","]
#[doc = "      \">=\","]
#[doc = "      \"like\","]
#[doc = "      \"between\","]
#[doc = "      \"in\","]
#[doc = "      \"isNull\","]
#[doc = "      \"casei\","]
#[doc = "      \"accenti\","]
#[doc = "      \"s_contains\","]
#[doc = "      \"s_crosses\","]
#[doc = "      \"s_disjoint\","]
#[doc = "      \"s_equals\","]
#[doc = "      \"s_intersects\","]
#[doc = "      \"s_overlaps\","]
#[doc = "      \"s_touches\","]
#[doc = "      \"s_within\","]
#[doc = "      \"t_after\","]
#[doc = "      \"t_before\","]
#[doc = "      \"t_contains\","]
#[doc = "      \"t_disjoint\","]
#[doc = "      \"t_during\","]
#[doc = "      \"t_equals\","]
#[doc = "      \"t_finishedBy\","]
#[doc = "      \"t_finishes\","]
#[doc = "      \"t_intersects\","]
#[doc = "      \"t_meets\","]
#[doc = "      \"t_metBy\","]
#[doc = "      \"t_overlappedBy\","]
#[doc = "      \"t_overlaps\","]
#[doc = "      \"t_startedBy\","]
#[doc = "      \"t_starts\","]
#[doc = "      \"a_containedBy\","]
#[doc = "      \"a_contains\","]
#[doc = "      \"a_equals\","]
#[doc = "      \"a_overlaps\","]
#[doc = "      \"+\","]
#[doc = "      \"-\","]
#[doc = "      \"*\","]
#[doc = "      \"/\","]
#[doc = "      \"^\","]
#[doc = "      \"%\","]
#[doc = "      \"div\""]
#[doc = "    ]"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub(crate) struct FunctionRefOp(String);
impl Deref for FunctionRefOp {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FunctionRefOp> for String {
    fn from(value: FunctionRefOp) -> Self {
        value.0
    }
}
impl From<&FunctionRefOp> for FunctionRefOp {
    fn from(value: &FunctionRefOp) -> Self {
        value.clone()
    }
}
impl TryFrom<String> for FunctionRefOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        if [
            "and".to_string(),
            "or".to_string(),
            "not".to_string(),
            "=".to_string(),
            "<>".to_string(),
            "<".to_string(),
            ">".to_string(),
            "<=".to_string(),
            ">=".to_string(),
            "like".to_string(),
            "between".to_string(),
            "in".to_string(),
            "isNull".to_string(),
            "casei".to_string(),
            "accenti".to_string(),
            "s_contains".to_string(),
            "s_crosses".to_string(),
            "s_disjoint".to_string(),
            "s_equals".to_string(),
            "s_intersects".to_string(),
            "s_overlaps".to_string(),
            "s_touches".to_string(),
            "s_within".to_string(),
            "t_after".to_string(),
            "t_before".to_string(),
            "t_contains".to_string(),
            "t_disjoint".to_string(),
            "t_during".to_string(),
            "t_equals".to_string(),
            "t_finishedBy".to_string(),
            "t_finishes".to_string(),
            "t_intersects".to_string(),
            "t_meets".to_string(),
            "t_metBy".to_string(),
            "t_overlappedBy".to_string(),
            "t_overlaps".to_string(),
            "t_startedBy".to_string(),
            "t_starts".to_string(),
            "a_containedBy".to_string(),
            "a_contains".to_string(),
            "a_equals".to_string(),
            "a_overlaps".to_string(),
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "/".to_string(),
            "^".to_string(),
            "%".to_string(),
            "div".to_string(),
        ]
        .contains(&value)
        {
            Err(MyError::Runtime("Reserved keyword".into()))
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for FunctionRefOp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<String>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`GeometryLiteral`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/point\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/linestring\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/polygon\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/multipoint\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/multilinestring\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/multipolygon\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/geometrycollection\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum GeometryLiteral {
    Point(Point),
    Linestring(Linestring),
    Polygon(Polygon),
    Multipoint(Multipoint),
    Multilinestring(Multilinestring),
    Multipolygon(Multipolygon),
    Geometrycollection(Geometrycollection),
}

impl fmt::Display for GeometryLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryLiteral::Point(x) => write!(f, "{x}"),
            GeometryLiteral::Linestring(x) => write!(f, "{x}"),
            GeometryLiteral::Polygon(x) => write!(f, "{x}"),
            GeometryLiteral::Multipoint(x) => write!(f, "{x}"),
            GeometryLiteral::Multilinestring(x) => write!(f, "{x}"),
            GeometryLiteral::Multipolygon(x) => write!(f, "{x}"),
            GeometryLiteral::Geometrycollection(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for GeometryLiteral {
    fn from(value: &GeometryLiteral) -> Self {
        value.clone()
    }
}
impl From<Point> for GeometryLiteral {
    fn from(value: Point) -> Self {
        Self::Point(value)
    }
}
impl From<Linestring> for GeometryLiteral {
    fn from(value: Linestring) -> Self {
        Self::Linestring(value)
    }
}
impl From<Polygon> for GeometryLiteral {
    fn from(value: Polygon) -> Self {
        Self::Polygon(value)
    }
}
impl From<Multipoint> for GeometryLiteral {
    fn from(value: Multipoint) -> Self {
        Self::Multipoint(value)
    }
}
impl From<Multilinestring> for GeometryLiteral {
    fn from(value: Multilinestring) -> Self {
        Self::Multilinestring(value)
    }
}
impl From<Multipolygon> for GeometryLiteral {
    fn from(value: Multipolygon) -> Self {
        Self::Multipolygon(value)
    }
}
impl From<Geometrycollection> for GeometryLiteral {
    fn from(value: Geometrycollection) -> Self {
        Self::Geometrycollection(value)
    }
}
#[doc = "`Geometrycollection`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GeoJSON GeometryCollection\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"geometries\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"geometries\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"oneOf\": ["]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/point\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/linestring\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/polygon\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/multipoint\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/multilinestring\""]
#[doc = "          },"]
#[doc = "          {"]
#[doc = "            \"$ref\": \"#/$defs/multipolygon\""]
#[doc = "          }"]
#[doc = "        ]"]
#[doc = "      },"]
#[doc = "      \"minItems\": 2"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"GeometryCollection\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Geometrycollection {
    pub(crate) geometries: Vec<GeometrycollectionGeometriesItem>,
    #[serde(rename = "type")]
    pub(crate) type_: GeometrycollectionType,
}

impl fmt::Display for Geometrycollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> = self.geometries.iter().map(|x| x.to_string()).collect();
        write!(f, "GEOMETRYCOLLECTION ({})", items.join(", "))
    }
}

impl From<&Geometrycollection> for Geometrycollection {
    fn from(value: &Geometrycollection) -> Self {
        value.clone()
    }
}

#[doc = "`GeometrycollectionGeometriesItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/point\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/linestring\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/polygon\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/multipoint\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/multilinestring\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/multipolygon\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum GeometrycollectionGeometriesItem {
    Point(Point),
    Linestring(Linestring),
    Polygon(Polygon),
    Multipoint(Multipoint),
    Multilinestring(Multilinestring),
    Multipolygon(Multipolygon),
}

impl fmt::Display for GeometrycollectionGeometriesItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometrycollectionGeometriesItem::Point(x) => write!(f, "{x}"),
            GeometrycollectionGeometriesItem::Linestring(x) => write!(f, "{x}"),
            GeometrycollectionGeometriesItem::Polygon(x) => write!(f, "{x}"),
            GeometrycollectionGeometriesItem::Multipoint(x) => write!(f, "{x}"),
            GeometrycollectionGeometriesItem::Multilinestring(x) => write!(f, "{x}"),
            GeometrycollectionGeometriesItem::Multipolygon(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for GeometrycollectionGeometriesItem {
    fn from(value: &GeometrycollectionGeometriesItem) -> Self {
        value.clone()
    }
}
impl From<Point> for GeometrycollectionGeometriesItem {
    fn from(value: Point) -> Self {
        Self::Point(value)
    }
}
impl From<Linestring> for GeometrycollectionGeometriesItem {
    fn from(value: Linestring) -> Self {
        Self::Linestring(value)
    }
}
impl From<Polygon> for GeometrycollectionGeometriesItem {
    fn from(value: Polygon) -> Self {
        Self::Polygon(value)
    }
}
impl From<Multipoint> for GeometrycollectionGeometriesItem {
    fn from(value: Multipoint) -> Self {
        Self::Multipoint(value)
    }
}
impl From<Multilinestring> for GeometrycollectionGeometriesItem {
    fn from(value: Multilinestring) -> Self {
        Self::Multilinestring(value)
    }
}
impl From<Multipolygon> for GeometrycollectionGeometriesItem {
    fn from(value: Multipolygon) -> Self {
        Self::Multipolygon(value)
    }
}
#[doc = "`GeometrycollectionType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"GeometryCollection\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum GeometrycollectionType {
    GeometryCollection,
}
impl From<&Self> for GeometrycollectionType {
    fn from(value: &GeometrycollectionType) -> Self {
        value.clone()
    }
}
impl fmt::Display for GeometrycollectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::GeometryCollection => write!(f, "GeometryCollection"),
        }
    }
}
impl FromStr for GeometrycollectionType {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "GeometryCollection" => Ok(Self::GeometryCollection),
            _ => Err(MyError::Runtime("Expected GEOMETRYCOLLECTION".into())),
        }
    }
}
impl TryFrom<&str> for GeometrycollectionType {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for GeometrycollectionType {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for GeometrycollectionType {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`InListOperands`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"maxItems\": 2,"]
#[doc = "  \"minItems\": 2,"]
#[doc = "  \"prefixItems\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/scalarExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/scalarExpression\""]
#[doc = "      },"]
#[doc = "      \"type\": \"array\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct InListOperands(pub(crate) [Value; 2usize]);
impl Deref for InListOperands {
    type Target = [Value; 2usize];
    fn deref(&self) -> &[Value; 2usize] {
        &self.0
    }
}
impl From<InListOperands> for [Value; 2usize] {
    fn from(value: InListOperands) -> Self {
        value.0
    }
}
impl From<&InListOperands> for InListOperands {
    fn from(value: &InListOperands) -> Self {
        value.clone()
    }
}
impl From<[Value; 2usize]> for InListOperands {
    fn from(value: [Value; 2usize]) -> Self {
        Self(value)
    }
}
#[doc = "`InstantInstance`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/dateInstant\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/timestampInstant\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum InstantInstance {
    DateInstant(DateInstant),
    TimestampInstant(TimestampInstant),
}

impl fmt::Display for InstantInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstantInstance::DateInstant(x) => write!(f, "{x}"),
            InstantInstance::TimestampInstant(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for InstantInstance {
    fn from(value: &InstantInstance) -> Self {
        value.clone()
    }
}
impl From<DateInstant> for InstantInstance {
    fn from(value: DateInstant) -> Self {
        Self::DateInstant(value)
    }
}
impl From<TimestampInstant> for InstantInstance {
    fn from(value: TimestampInstant) -> Self {
        Self::TimestampInstant(value)
    }
}
#[doc = "`InstantString`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/dateString\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/timestampString\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum InstantString {
    DateString(DateString),
    TimestampString(TimestampString),
}
impl From<&Self> for InstantString {
    fn from(value: &InstantString) -> Self {
        value.clone()
    }
}
impl FromStr for InstantString {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        if let Ok(v) = value.parse() {
            Ok(Self::DateString(v))
        } else if let Ok(v) = value.parse() {
            Ok(Self::TimestampString(v))
        } else {
            Err(MyError::Runtime("Expected date | timestamp string".into()))
        }
    }
}
impl TryFrom<&str> for InstantString {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for InstantString {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for InstantString {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl fmt::Display for InstantString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DateString(x) => x.fmt(f),
            Self::TimestampString(x) => x.fmt(f),
        }
    }
}
impl From<DateString> for InstantString {
    fn from(value: DateString) -> Self {
        Self::DateString(value)
    }
}
impl From<TimestampString> for InstantString {
    fn from(value: TimestampString) -> Self {
        Self::TimestampString(value)
    }
}
#[doc = "`IntervalArray`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"oneOf\": ["]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/instantString\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"enum\": ["]
#[doc = "          \"..\""]
#[doc = "        ]"]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/functionRef\""]
#[doc = "      }"]
#[doc = "    ]"]
#[doc = "  },"]
#[doc = "  \"maxItems\": 2,"]
#[doc = "  \"minItems\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct IntervalArray(pub(crate) [IntervalArrayItem; 2usize]);
impl Deref for IntervalArray {
    type Target = [IntervalArrayItem; 2usize];
    fn deref(&self) -> &[IntervalArrayItem; 2usize] {
        &self.0
    }
}
impl From<IntervalArray> for [IntervalArrayItem; 2usize] {
    fn from(value: IntervalArray) -> Self {
        value.0
    }
}
impl From<&IntervalArray> for IntervalArray {
    fn from(value: &IntervalArray) -> Self {
        value.clone()
    }
}
impl From<[IntervalArrayItem; 2usize]> for IntervalArray {
    fn from(value: [IntervalArrayItem; 2usize]) -> Self {
        Self(value)
    }
}
#[doc = "`IntervalArrayItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/instantString\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"..\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/functionRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum IntervalArrayItem {
    Variant0(InstantString),
    Variant1(IntervalArrayItemVariant1),
    Variant2(PropertyRef),
    Variant3(FunctionRef),
}

impl fmt::Display for IntervalArrayItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntervalArrayItem::Variant0(x) => write!(f, "{x}"),
            IntervalArrayItem::Variant1(_) => write!(f, "'..'"),
            IntervalArrayItem::Variant2(x) => write!(f, "{x}"),
            IntervalArrayItem::Variant3(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for IntervalArrayItem {
    fn from(value: &IntervalArrayItem) -> Self {
        value.clone()
    }
}
impl From<InstantString> for IntervalArrayItem {
    fn from(value: InstantString) -> Self {
        Self::Variant0(value)
    }
}
impl From<IntervalArrayItemVariant1> for IntervalArrayItem {
    fn from(value: IntervalArrayItemVariant1) -> Self {
        Self::Variant1(value)
    }
}
impl From<PropertyRef> for IntervalArrayItem {
    fn from(value: PropertyRef) -> Self {
        Self::Variant2(value)
    }
}
impl From<FunctionRef> for IntervalArrayItem {
    fn from(value: FunctionRef) -> Self {
        Self::Variant3(value)
    }
}
#[doc = "`IntervalArrayItemVariant1`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"..\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum IntervalArrayItemVariant1 {
    #[serde(rename = "..")]
    X,
}
impl From<&Self> for IntervalArrayItemVariant1 {
    fn from(value: &IntervalArrayItemVariant1) -> Self {
        value.clone()
    }
}
impl fmt::Display for IntervalArrayItemVariant1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::X => write!(f, ".."),
        }
    }
}
impl FromStr for IntervalArrayItemVariant1 {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            ".." => Ok(Self::X),
            _ => Err(MyError::Runtime("Expected ..".into())),
        }
    }
}
impl TryFrom<&str> for IntervalArrayItemVariant1 {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for IntervalArrayItemVariant1 {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for IntervalArrayItemVariant1 {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`IntervalInstance`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"interval\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"interval\": {"]
#[doc = "      \"$ref\": \"#/$defs/intervalArray\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct IntervalInstance {
    pub(crate) interval: IntervalArray,
}

impl fmt::Display for IntervalInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "INTERVAL({}, {})",
            self.interval.0[0], self.interval.0[1]
        )
    }
}

impl From<&IntervalInstance> for IntervalInstance {
    fn from(value: &IntervalInstance) -> Self {
        value.clone()
    }
}

#[doc = "`IsBetweenOperands`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"oneOf\": ["]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/numericExpression\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/functionRef\""]
#[doc = "      }"]
#[doc = "    ]"]
#[doc = "  },"]
#[doc = "  \"maxItems\": 3,"]
#[doc = "  \"minItems\": 3"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct IsBetweenOperands(pub(crate) [IsBetweenOperandsItem; 3usize]);
impl Deref for IsBetweenOperands {
    type Target = [IsBetweenOperandsItem; 3usize];
    fn deref(&self) -> &[IsBetweenOperandsItem; 3usize] {
        &self.0
    }
}
impl From<IsBetweenOperands> for [IsBetweenOperandsItem; 3usize] {
    fn from(value: IsBetweenOperands) -> Self {
        value.0
    }
}
impl From<&IsBetweenOperands> for IsBetweenOperands {
    fn from(value: &IsBetweenOperands) -> Self {
        value.clone()
    }
}
impl From<[IsBetweenOperandsItem; 3usize]> for IsBetweenOperands {
    fn from(value: [IsBetweenOperandsItem; 3usize]) -> Self {
        Self(value)
    }
}
#[doc = "`IsBetweenOperandsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/numericExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/functionRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum IsBetweenOperandsItem {
    NumericExpression(NumericExpression),
    PropertyRef(PropertyRef),
    FunctionRef(FunctionRef),
}

impl fmt::Display for IsBetweenOperandsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsBetweenOperandsItem::NumericExpression(x) => write!(f, "{x}"),
            IsBetweenOperandsItem::PropertyRef(x) => write!(f, "{x}"),
            IsBetweenOperandsItem::FunctionRef(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for IsBetweenOperandsItem {
    fn from(value: &IsBetweenOperandsItem) -> Self {
        value.clone()
    }
}
impl From<NumericExpression> for IsBetweenOperandsItem {
    fn from(value: NumericExpression) -> Self {
        Self::NumericExpression(value)
    }
}
impl From<PropertyRef> for IsBetweenOperandsItem {
    fn from(value: PropertyRef) -> Self {
        Self::PropertyRef(value)
    }
}
impl From<FunctionRef> for IsBetweenOperandsItem {
    fn from(value: FunctionRef) -> Self {
        Self::FunctionRef(value)
    }
}
#[doc = "`IsBetweenPredicate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"$ref\": \"#/$defs/isBetweenOperands\""]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"between\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IsBetweenPredicate {
    pub(crate) args: IsBetweenOperands,
    pub(crate) op: IsBetweenPredicateOp,
}

impl fmt::Display for IsBetweenPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} BETWEEN {} AND {}",
            self.args.0[0], self.args.0[1], self.args.0[2]
        )
    }
}

impl From<&IsBetweenPredicate> for IsBetweenPredicate {
    fn from(value: &IsBetweenPredicate) -> Self {
        value.clone()
    }
}

#[doc = "`IsBetweenPredicateOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"between\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum IsBetweenPredicateOp {
    #[serde(rename = "between")]
    Between,
}
impl From<&Self> for IsBetweenPredicateOp {
    fn from(value: &IsBetweenPredicateOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for IsBetweenPredicateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Between => write!(f, "between"),
        }
    }
}
impl FromStr for IsBetweenPredicateOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "between" => Ok(Self::Between),
            _ => Err(MyError::Runtime("invalid value".into())),
        }
    }
}
impl TryFrom<&str> for IsBetweenPredicateOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for IsBetweenPredicateOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for IsBetweenPredicateOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`IsInListPredicate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"$ref\": \"#/$defs/inListOperands\""]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"in\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IsInListPredicate {
    pub(crate) args: InListOperands,
    pub(crate) op: IsInListPredicateOp,
}

impl fmt::Display for IsInListPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x = self.args.0[0].clone();
        let x: ScalarExpression = serde_json::from_value(x).expect("Expected scalar expression");
        let y = self.args.0[1].clone();
        let y: Array = serde_json::from_value(y).expect("Expected array");
        write!(f, "{x} IN ({y})")
    }
}

impl From<&IsInListPredicate> for IsInListPredicate {
    fn from(value: &IsInListPredicate) -> Self {
        value.clone()
    }
}

#[doc = "`IsInListPredicateOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"in\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum IsInListPredicateOp {
    #[serde(rename = "in")]
    In,
}
impl From<&Self> for IsInListPredicateOp {
    fn from(value: &IsInListPredicateOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for IsInListPredicateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::In => write!(f, "in"),
        }
    }
}
impl FromStr for IsInListPredicateOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "in" => Ok(Self::In),
            _ => Err(MyError::Runtime("Expected IN".into())),
        }
    }
}
impl TryFrom<&str> for IsInListPredicateOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for IsInListPredicateOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for IsInListPredicateOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`IsLikeOperands`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"maxItems\": 2,"]
#[doc = "  \"minItems\": 2,"]
#[doc = "  \"prefixItems\": ["]
#[doc = "    {"]
#[doc = "      \"oneOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/functionRef\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/patternExpression\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct IsLikeOperands(pub(crate) [Value; 2usize]);
impl Deref for IsLikeOperands {
    type Target = [Value; 2usize];
    fn deref(&self) -> &[Value; 2usize] {
        &self.0
    }
}
impl From<IsLikeOperands> for [Value; 2usize] {
    fn from(value: IsLikeOperands) -> Self {
        value.0
    }
}
impl From<&IsLikeOperands> for IsLikeOperands {
    fn from(value: &IsLikeOperands) -> Self {
        value.clone()
    }
}
impl From<[Value; 2usize]> for IsLikeOperands {
    fn from(value: [Value; 2usize]) -> Self {
        Self(value)
    }
}
#[doc = "`IsLikePredicate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"$ref\": \"#/$defs/isLikeOperands\""]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"like\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IsLikePredicate {
    pub(crate) args: IsLikeOperands,
    pub(crate) op: IsLikePredicateOp,
}

impl fmt::Display for IsLikePredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x = self.args.0[0].clone();
        let x: Expression = serde_json::from_value(x).expect("Expected expression)");
        let y = self.args.0[1].clone();
        let y: Expression = serde_json::from_value(y).expect("Expected expression");
        write!(f, "{x} LIKE {y}")
    }
}

impl From<&IsLikePredicate> for IsLikePredicate {
    fn from(value: &IsLikePredicate) -> Self {
        value.clone()
    }
}

#[doc = "`IsLikePredicateOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"like\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum IsLikePredicateOp {
    #[serde(rename = "like")]
    Like,
}
impl From<&Self> for IsLikePredicateOp {
    fn from(value: &IsLikePredicateOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for IsLikePredicateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Like => write!(f, "like"),
        }
    }
}
impl FromStr for IsLikePredicateOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "like" => Ok(Self::Like),
            _ => Err(MyError::Runtime("Expected LIKE".into())),
        }
    }
}
impl TryFrom<&str> for IsLikePredicateOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for IsLikePredicateOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for IsLikePredicateOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`IsNullOperand`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"oneOf\": ["]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/numericExpression\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$dynamicRef\": \"#cql2expression\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/spatialInstance\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/temporalInstance\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "      }"]
#[doc = "    ]"]
#[doc = "  },"]
#[doc = "  \"maxItems\": 1,"]
#[doc = "  \"minItems\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct IsNullOperand(pub(crate) [IsNullOperandItem; 1usize]);
impl Deref for IsNullOperand {
    type Target = [IsNullOperandItem; 1usize];
    fn deref(&self) -> &[IsNullOperandItem; 1usize] {
        &self.0
    }
}
impl From<IsNullOperand> for [IsNullOperandItem; 1usize] {
    fn from(value: IsNullOperand) -> Self {
        value.0
    }
}
impl From<&IsNullOperand> for IsNullOperand {
    fn from(value: &IsNullOperand) -> Self {
        value.clone()
    }
}
impl From<[IsNullOperandItem; 1usize]> for IsNullOperand {
    fn from(value: [IsNullOperandItem; 1usize]) -> Self {
        Self(value)
    }
}
#[doc = "`IsNullOperandItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/numericExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$dynamicRef\": \"#cql2expression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/spatialInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/temporalInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum IsNullOperandItem {
    Variant0(CharacterExpression),
    Variant1(NumericExpression),
    Variant2(Value),
    Variant3(SpatialInstance),
    Variant4(TemporalInstance),
    Variant5(PropertyRef),
}

impl fmt::Display for IsNullOperandItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsNullOperandItem::Variant0(x) => write!(f, "{x}"),
            IsNullOperandItem::Variant1(x) => write!(f, "{x}"),
            IsNullOperandItem::Variant2(x) => {
                let x: Expression = serde_json::from_value(x.clone()).expect("Expected expression");
                write!(f, "{x}")
            }
            IsNullOperandItem::Variant3(x) => write!(f, "{x}"),
            IsNullOperandItem::Variant4(x) => write!(f, "{x}"),
            IsNullOperandItem::Variant5(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for IsNullOperandItem {
    fn from(value: &IsNullOperandItem) -> Self {
        value.clone()
    }
}
impl From<CharacterExpression> for IsNullOperandItem {
    fn from(value: CharacterExpression) -> Self {
        Self::Variant0(value)
    }
}
impl From<NumericExpression> for IsNullOperandItem {
    fn from(value: NumericExpression) -> Self {
        Self::Variant1(value)
    }
}
impl From<Value> for IsNullOperandItem {
    fn from(value: Value) -> Self {
        Self::Variant2(value)
    }
}
impl From<SpatialInstance> for IsNullOperandItem {
    fn from(value: SpatialInstance) -> Self {
        Self::Variant3(value)
    }
}
impl From<TemporalInstance> for IsNullOperandItem {
    fn from(value: TemporalInstance) -> Self {
        Self::Variant4(value)
    }
}
impl From<PropertyRef> for IsNullOperandItem {
    fn from(value: PropertyRef) -> Self {
        Self::Variant5(value)
    }
}
#[doc = "`IsNullPredicate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"$ref\": \"#/$defs/isNullOperand\""]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"isNull\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IsNullPredicate {
    pub(crate) args: IsNullOperand,
    pub(crate) op: IsNullPredicateOp,
}

impl fmt::Display for IsNullPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} IS NULL", self.args.0[0])
    }
}

impl From<&IsNullPredicate> for IsNullPredicate {
    fn from(value: &IsNullPredicate) -> Self {
        value.clone()
    }
}

#[doc = "`IsNullPredicateOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"isNull\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum IsNullPredicateOp {
    #[serde(rename = "isNull")]
    IsNull,
}
impl From<&Self> for IsNullPredicateOp {
    fn from(value: &IsNullPredicateOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for IsNullPredicateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::IsNull => write!(f, "isNull"),
        }
    }
}
impl FromStr for IsNullPredicateOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "isNull" => Ok(Self::IsNull),
            _ => Err(MyError::Runtime("Expected IS NULL".into())),
        }
    }
}
impl TryFrom<&str> for IsNullPredicateOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for IsNullPredicateOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for IsNullPredicateOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`Linestring`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GeoJSON LineString\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coordinates\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bbox\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 4"]
#[doc = "    },"]
#[doc = "    \"coordinates\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"array\","]
#[doc = "        \"items\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"minItems\": 2"]
#[doc = "      },"]
#[doc = "      \"minItems\": 2"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"LineString\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Linestring {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bbox: Vec<f64>,
    pub(crate) coordinates: Vec<Vec<f64>>,
    #[serde(rename = "type")]
    pub(crate) type_: LinestringType,
}

impl fmt::Display for Linestring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LINESTRING {}",
            geom::Line::coords_as_txt(&self.coordinates)
        )
    }
}

impl From<&Linestring> for Linestring {
    fn from(value: &Linestring) -> Self {
        value.clone()
    }
}

#[doc = "`LinestringType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"LineString\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum LinestringType {
    LineString,
}
impl From<&Self> for LinestringType {
    fn from(value: &LinestringType) -> Self {
        value.clone()
    }
}
impl fmt::Display for LinestringType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::LineString => write!(f, "LineString"),
        }
    }
}
impl FromStr for LinestringType {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "LineString" => Ok(Self::LineString),
            _ => Err(MyError::Runtime("Expected LINESTRING".into())),
        }
    }
}
impl TryFrom<&str> for LinestringType {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for LinestringType {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for LinestringType {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`Multilinestring`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GeoJSON MultiLineString\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coordinates\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bbox\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 4"]
#[doc = "    },"]
#[doc = "    \"coordinates\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"array\","]
#[doc = "        \"items\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"number\""]
#[doc = "          },"]
#[doc = "          \"minItems\": 2"]
#[doc = "        },"]
#[doc = "        \"minItems\": 2"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"MultiLineString\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Multilinestring {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bbox: Vec<f64>,
    pub(crate) coordinates: Vec<Vec<Vec<f64>>>,
    #[serde(rename = "type")]
    pub(crate) type_: MultilinestringType,
}

impl fmt::Display for Multilinestring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MULTILINESTRING {}",
            geom::Lines::coords_as_txt(&self.coordinates)
        )
    }
}

impl From<&Multilinestring> for Multilinestring {
    fn from(value: &Multilinestring) -> Self {
        value.clone()
    }
}

#[doc = "`MultilinestringType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"MultiLineString\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum MultilinestringType {
    MultiLineString,
}
impl From<&Self> for MultilinestringType {
    fn from(value: &MultilinestringType) -> Self {
        value.clone()
    }
}
impl fmt::Display for MultilinestringType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::MultiLineString => write!(f, "MultiLineString"),
        }
    }
}
impl FromStr for MultilinestringType {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "MultiLineString" => Ok(Self::MultiLineString),
            _ => Err(MyError::Runtime("Expected MULTILINESTRING".into())),
        }
    }
}
impl TryFrom<&str> for MultilinestringType {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for MultilinestringType {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for MultilinestringType {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`Multipoint`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GeoJSON MultiPoint\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coordinates\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bbox\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 4"]
#[doc = "    },"]
#[doc = "    \"coordinates\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"array\","]
#[doc = "        \"items\": {"]
#[doc = "          \"type\": \"number\""]
#[doc = "        },"]
#[doc = "        \"minItems\": 2"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"MultiPoint\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Multipoint {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bbox: Vec<f64>,
    pub(crate) coordinates: Vec<Vec<f64>>,
    #[serde(rename = "type")]
    pub(crate) type_: MultipointType,
}

impl fmt::Display for Multipoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MULTIPOINT {}",
            geom::Points::coords_as_txt(&self.coordinates)
        )
    }
}

impl From<&Multipoint> for Multipoint {
    fn from(value: &Multipoint) -> Self {
        value.clone()
    }
}

#[doc = "`MultipointType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"MultiPoint\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum MultipointType {
    MultiPoint,
}
impl From<&Self> for MultipointType {
    fn from(value: &MultipointType) -> Self {
        value.clone()
    }
}
impl fmt::Display for MultipointType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::MultiPoint => write!(f, "MultiPoint"),
        }
    }
}
impl FromStr for MultipointType {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "MultiPoint" => Ok(Self::MultiPoint),
            _ => Err(MyError::Runtime("Expected MULTIPOINT".into())),
        }
    }
}
impl TryFrom<&str> for MultipointType {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for MultipointType {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for MultipointType {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`Multipolygon`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GeoJSON MultiPolygon\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coordinates\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bbox\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 4"]
#[doc = "    },"]
#[doc = "    \"coordinates\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"array\","]
#[doc = "        \"items\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"array\","]
#[doc = "            \"items\": {"]
#[doc = "              \"type\": \"number\""]
#[doc = "            },"]
#[doc = "            \"minItems\": 2"]
#[doc = "          },"]
#[doc = "          \"minItems\": 4"]
#[doc = "        }"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"MultiPolygon\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Multipolygon {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bbox: Vec<f64>,
    pub(crate) coordinates: Vec<Vec<Vec<Vec<f64>>>>,
    #[serde(rename = "type")]
    pub(crate) type_: MultipolygonType,
}

impl fmt::Display for Multipolygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MULTIPOLYGON {}",
            geom::Polygons::coords_as_txt(&self.coordinates)
        )
    }
}

impl From<&Multipolygon> for Multipolygon {
    fn from(value: &Multipolygon) -> Self {
        value.clone()
    }
}

#[doc = "`MultipolygonType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"MultiPolygon\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum MultipolygonType {
    MultiPolygon,
}
impl From<&Self> for MultipolygonType {
    fn from(value: &MultipolygonType) -> Self {
        value.clone()
    }
}
impl fmt::Display for MultipolygonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::MultiPolygon => write!(f, "MultiPolygon"),
        }
    }
}
impl FromStr for MultipolygonType {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "MultiPolygon" => Ok(Self::MultiPolygon),
            _ => Err(MyError::Runtime("Expected MULTIPOLYGON".into())),
        }
    }
}
impl TryFrom<&str> for MultipolygonType {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for MultipolygonType {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for MultipolygonType {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`NotExpression`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$dynamicRef\": \"#cql2expression\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 1,"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"not\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NotExpression {
    pub(crate) args: [Value; 1usize],
    pub(crate) op: NotExpressionOp,
}

impl fmt::Display for NotExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x = self.args[0].clone();
        let x: Expression = serde_json::from_value(x).expect("Expected expression");
        write!(f, "NOT( {x} )")
    }
}

impl From<&NotExpression> for NotExpression {
    fn from(value: &NotExpression) -> Self {
        value.clone()
    }
}

#[doc = "`NotExpressionOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"not\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum NotExpressionOp {
    #[serde(rename = "not")]
    Not,
}
impl From<&Self> for NotExpressionOp {
    fn from(value: &NotExpressionOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for NotExpressionOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Not => write!(f, "not"),
        }
    }
}
impl FromStr for NotExpressionOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "not" => Ok(Self::Not),
            _ => Err(MyError::Runtime("Expected NOT".into())),
        }
    }
}
impl TryFrom<&str> for NotExpressionOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for NotExpressionOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for NotExpressionOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`NumericExpression`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/arithmeticExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum NumericExpression {
    Variant0(ArithmeticExpression),
    Variant1(f64),
}

impl fmt::Display for NumericExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumericExpression::Variant0(x) => write!(f, "{x}"),
            NumericExpression::Variant1(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for NumericExpression {
    fn from(value: &NumericExpression) -> Self {
        value.clone()
    }
}
impl From<ArithmeticExpression> for NumericExpression {
    fn from(value: ArithmeticExpression) -> Self {
        Self::Variant0(value)
    }
}
impl From<f64> for NumericExpression {
    fn from(value: f64) -> Self {
        Self::Variant1(value)
    }
}
#[doc = "`PatternExpression`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"args\","]
#[doc = "        \"op\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"args\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/patternExpression\""]
#[doc = "          },"]
#[doc = "          \"maxItems\": 1,"]
#[doc = "          \"minItems\": 1"]
#[doc = "        },"]
#[doc = "        \"op\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"casei\""]
#[doc = "          ]"]
#[doc = "        }"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"args\","]
#[doc = "        \"op\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"args\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/patternExpression\""]
#[doc = "          },"]
#[doc = "          \"maxItems\": 1,"]
#[doc = "          \"minItems\": 1"]
#[doc = "        },"]
#[doc = "        \"op\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"enum\": ["]
#[doc = "            \"accenti\""]
#[doc = "          ]"]
#[doc = "        }"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum PatternExpression {
    Variant0 {
        args: [Box<PatternExpression>; 1usize],
        op: PatternExpressionVariant0Op,
    },
    Variant1 {
        args: [Box<PatternExpression>; 1usize],
        op: PatternExpressionVariant1Op,
    },
    Variant2(String),
}
impl From<&Self> for PatternExpression {
    fn from(value: &PatternExpression) -> Self {
        value.clone()
    }
}
#[doc = "`PatternExpressionVariant0Op`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"casei\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum PatternExpressionVariant0Op {
    #[serde(rename = "casei")]
    Casei,
}
impl From<&Self> for PatternExpressionVariant0Op {
    fn from(value: &PatternExpressionVariant0Op) -> Self {
        value.clone()
    }
}
impl fmt::Display for PatternExpressionVariant0Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Casei => write!(f, "casei"),
        }
    }
}
impl FromStr for PatternExpressionVariant0Op {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "casei" => Ok(Self::Casei),
            _ => Err(MyError::Runtime("Expected CASEI".into())),
        }
    }
}
impl TryFrom<&str> for PatternExpressionVariant0Op {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for PatternExpressionVariant0Op {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for PatternExpressionVariant0Op {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`PatternExpressionVariant1Op`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"accenti\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum PatternExpressionVariant1Op {
    #[serde(rename = "accenti")]
    Accenti,
}
impl From<&Self> for PatternExpressionVariant1Op {
    fn from(value: &PatternExpressionVariant1Op) -> Self {
        value.clone()
    }
}
impl fmt::Display for PatternExpressionVariant1Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Accenti => write!(f, "accenti"),
        }
    }
}
impl FromStr for PatternExpressionVariant1Op {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "accenti" => Ok(Self::Accenti),
            _ => Err(MyError::Runtime("Expected ACCENTI".into())),
        }
    }
}
impl TryFrom<&str> for PatternExpressionVariant1Op {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for PatternExpressionVariant1Op {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for PatternExpressionVariant1Op {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`Point`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GeoJSON Point\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coordinates\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bbox\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 4"]
#[doc = "    },"]
#[doc = "    \"coordinates\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 2"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"Point\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Point {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bbox: Vec<f64>,
    pub(crate) coordinates: Vec<f64>,
    #[serde(rename = "type")]
    pub(crate) type_: PointType,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "POINT ({})",
            geom::Point::coords_as_txt(&self.coordinates)
        )
    }
}

impl From<&Point> for Point {
    fn from(value: &Point) -> Self {
        value.clone()
    }
}

#[doc = "`PointType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"Point\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum PointType {
    Point,
}
impl From<&Self> for PointType {
    fn from(value: &PointType) -> Self {
        value.clone()
    }
}
impl fmt::Display for PointType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Point => write!(f, "Point"),
        }
    }
}
impl FromStr for PointType {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "Point" => Ok(Self::Point),
            _ => Err(MyError::Runtime("Expected POINT".into())),
        }
    }
}
impl TryFrom<&str> for PointType {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for PointType {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for PointType {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`Polygon`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GeoJSON Polygon\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"coordinates\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"bbox\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"number\""]
#[doc = "      },"]
#[doc = "      \"minItems\": 4"]
#[doc = "    },"]
#[doc = "    \"coordinates\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"array\","]
#[doc = "        \"items\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"number\""]
#[doc = "          },"]
#[doc = "          \"minItems\": 2"]
#[doc = "        },"]
#[doc = "        \"minItems\": 4"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"Polygon\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Polygon {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bbox: Vec<f64>,
    pub(crate) coordinates: Vec<Vec<Vec<f64>>>,
    #[serde(rename = "type")]
    pub(crate) type_: PolygonType,
}

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "POLYGON {}",
            geom::Polygon::coords_as_txt(&self.coordinates)
        )
    }
}

impl From<&Polygon> for Polygon {
    fn from(value: &Polygon) -> Self {
        value.clone()
    }
}

#[doc = "`PolygonType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"Polygon\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum PolygonType {
    Polygon,
}
impl From<&Self> for PolygonType {
    fn from(value: &PolygonType) -> Self {
        value.clone()
    }
}
impl fmt::Display for PolygonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Polygon => write!(f, "Polygon"),
        }
    }
}
impl FromStr for PolygonType {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "Polygon" => Ok(Self::Polygon),
            _ => Err(MyError::Runtime("Expected POLYGON".into())),
        }
    }
}
impl TryFrom<&str> for PolygonType {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for PolygonType {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for PolygonType {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`PropertyRef`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"property\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"property\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct PropertyRef {
    pub(crate) property: String,
}

impl fmt::Display for PropertyRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.property)
    }
}

impl From<&PropertyRef> for PropertyRef {
    fn from(value: &PropertyRef) -> Self {
        value.clone()
    }
}

#[doc = "`ScalarExpression`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/characterExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/numericExpression\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/instantInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/functionRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum ScalarExpression {
    Variant0(CharacterExpression),
    Variant1(NumericExpression),
    Variant2(bool),
    Variant3(InstantInstance),
    Variant4(FunctionRef),
    Variant5(PropertyRef),
}

impl fmt::Display for ScalarExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalarExpression::Variant0(x) => write!(f, "{x}"),
            ScalarExpression::Variant1(x) => write!(f, "{x}"),
            // ScalarExpression::Variant1(x) => write!(f, "({x})"),
            ScalarExpression::Variant2(x) => write!(f, "{x}"),
            ScalarExpression::Variant3(x) => write!(f, "{x}"),
            ScalarExpression::Variant4(x) => write!(f, "{x}"),
            ScalarExpression::Variant5(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for ScalarExpression {
    fn from(value: &ScalarExpression) -> Self {
        value.clone()
    }
}
impl From<CharacterExpression> for ScalarExpression {
    fn from(value: CharacterExpression) -> Self {
        Self::Variant0(value)
    }
}
impl From<NumericExpression> for ScalarExpression {
    fn from(value: NumericExpression) -> Self {
        Self::Variant1(value)
    }
}
impl From<bool> for ScalarExpression {
    fn from(value: bool) -> Self {
        Self::Variant2(value)
    }
}
impl From<InstantInstance> for ScalarExpression {
    fn from(value: InstantInstance) -> Self {
        Self::Variant3(value)
    }
}
impl From<FunctionRef> for ScalarExpression {
    fn from(value: FunctionRef) -> Self {
        Self::Variant4(value)
    }
}
impl From<PropertyRef> for ScalarExpression {
    fn from(value: PropertyRef) -> Self {
        Self::Variant5(value)
    }
}
#[doc = "`ScalarOperands`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"$ref\": \"#/$defs/scalarExpression\""]
#[doc = "  },"]
#[doc = "  \"maxItems\": 2,"]
#[doc = "  \"minItems\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct ScalarOperands(pub(crate) [ScalarExpression; 2usize]);
impl Deref for ScalarOperands {
    type Target = [ScalarExpression; 2usize];
    fn deref(&self) -> &[ScalarExpression; 2usize] {
        &self.0
    }
}
impl From<ScalarOperands> for [ScalarExpression; 2usize] {
    fn from(value: ScalarOperands) -> Self {
        value.0
    }
}
impl From<&ScalarOperands> for ScalarOperands {
    fn from(value: &ScalarOperands) -> Self {
        value.clone()
    }
}
impl From<[ScalarExpression; 2usize]> for ScalarOperands {
    fn from(value: [ScalarExpression; 2usize]) -> Self {
        Self(value)
    }
}
#[doc = "`SpatialInstance`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/geometryLiteral\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/bboxLiteral\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum SpatialInstance {
    GeometryLiteral(GeometryLiteral),
    BboxLiteral(BboxLiteral),
}

impl fmt::Display for SpatialInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpatialInstance::GeometryLiteral(x) => write!(f, "{x}"),
            SpatialInstance::BboxLiteral(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for SpatialInstance {
    fn from(value: &SpatialInstance) -> Self {
        value.clone()
    }
}
impl From<GeometryLiteral> for SpatialInstance {
    fn from(value: GeometryLiteral) -> Self {
        Self::GeometryLiteral(value)
    }
}
impl From<BboxLiteral> for SpatialInstance {
    fn from(value: BboxLiteral) -> Self {
        Self::BboxLiteral(value)
    }
}
#[doc = "`SpatialOperands`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"oneOf\": ["]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/spatialInstance\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/functionRef\""]
#[doc = "      }"]
#[doc = "    ]"]
#[doc = "  },"]
#[doc = "  \"maxItems\": 2,"]
#[doc = "  \"minItems\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct SpatialOperands(pub(crate) [SpatialOperandsItem; 2usize]);
impl Deref for SpatialOperands {
    type Target = [SpatialOperandsItem; 2usize];
    fn deref(&self) -> &[SpatialOperandsItem; 2usize] {
        &self.0
    }
}
impl From<SpatialOperands> for [SpatialOperandsItem; 2usize] {
    fn from(value: SpatialOperands) -> Self {
        value.0
    }
}
impl From<&SpatialOperands> for SpatialOperands {
    fn from(value: &SpatialOperands) -> Self {
        value.clone()
    }
}
impl From<[SpatialOperandsItem; 2usize]> for SpatialOperands {
    fn from(value: [SpatialOperandsItem; 2usize]) -> Self {
        Self(value)
    }
}
#[doc = "`SpatialOperandsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/spatialInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/functionRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum SpatialOperandsItem {
    SpatialInstance(SpatialInstance),
    PropertyRef(PropertyRef),
    FunctionRef(FunctionRef),
}

impl fmt::Display for SpatialOperandsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpatialOperandsItem::SpatialInstance(x) => write!(f, "{x}"),
            SpatialOperandsItem::PropertyRef(x) => write!(f, "{x}"),
            SpatialOperandsItem::FunctionRef(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for SpatialOperandsItem {
    fn from(value: &SpatialOperandsItem) -> Self {
        value.clone()
    }
}
impl From<SpatialInstance> for SpatialOperandsItem {
    fn from(value: SpatialInstance) -> Self {
        Self::SpatialInstance(value)
    }
}
impl From<PropertyRef> for SpatialOperandsItem {
    fn from(value: PropertyRef) -> Self {
        Self::PropertyRef(value)
    }
}
impl From<FunctionRef> for SpatialOperandsItem {
    fn from(value: FunctionRef) -> Self {
        Self::FunctionRef(value)
    }
}
#[doc = "`SpatialPredicate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"$ref\": \"#/$defs/spatialOperands\""]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"s_contains\","]
#[doc = "        \"s_crosses\","]
#[doc = "        \"s_disjoint\","]
#[doc = "        \"s_equals\","]
#[doc = "        \"s_intersects\","]
#[doc = "        \"s_overlaps\","]
#[doc = "        \"s_touches\","]
#[doc = "        \"s_within\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SpatialPredicate {
    pub(crate) args: SpatialOperands,
    pub(crate) op: SpatialPredicateOp,
}

impl fmt::Display for SpatialPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({}, {})", self.op, self.args.0[0], self.args.0[1])
    }
}

impl From<&SpatialPredicate> for SpatialPredicate {
    fn from(value: &SpatialPredicate) -> Self {
        value.clone()
    }
}

#[doc = "`SpatialPredicateOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"s_contains\","]
#[doc = "    \"s_crosses\","]
#[doc = "    \"s_disjoint\","]
#[doc = "    \"s_equals\","]
#[doc = "    \"s_intersects\","]
#[doc = "    \"s_overlaps\","]
#[doc = "    \"s_touches\","]
#[doc = "    \"s_within\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum SpatialPredicateOp {
    #[serde(rename = "s_contains")]
    SContains,
    #[serde(rename = "s_crosses")]
    SCrosses,
    #[serde(rename = "s_disjoint")]
    SDisjoint,
    #[serde(rename = "s_equals")]
    SEquals,
    #[serde(rename = "s_intersects")]
    SIntersects,
    #[serde(rename = "s_overlaps")]
    SOverlaps,
    #[serde(rename = "s_touches")]
    STouches,
    #[serde(rename = "s_within")]
    SWithin,
}
impl From<&Self> for SpatialPredicateOp {
    fn from(value: &SpatialPredicateOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for SpatialPredicateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::SContains => write!(f, "s_contains"),
            Self::SCrosses => write!(f, "s_crosses"),
            Self::SDisjoint => write!(f, "s_disjoint"),
            Self::SEquals => write!(f, "s_equals"),
            Self::SIntersects => write!(f, "s_intersects"),
            Self::SOverlaps => write!(f, "s_overlaps"),
            Self::STouches => write!(f, "s_touches"),
            Self::SWithin => write!(f, "s_within"),
        }
    }
}
impl FromStr for SpatialPredicateOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "s_contains" => Ok(Self::SContains),
            "s_crosses" => Ok(Self::SCrosses),
            "s_disjoint" => Ok(Self::SDisjoint),
            "s_equals" => Ok(Self::SEquals),
            "s_intersects" => Ok(Self::SIntersects),
            "s_overlaps" => Ok(Self::SOverlaps),
            "s_touches" => Ok(Self::STouches),
            "s_within" => Ok(Self::SWithin),
            _ => Err(MyError::Runtime("Expected spatial function".into())),
        }
    }
}
impl TryFrom<&str> for SpatialPredicateOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for SpatialPredicateOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for SpatialPredicateOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`TemporalInstance`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/instantInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/intervalInstance\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum TemporalInstance {
    InstantInstance(InstantInstance),
    IntervalInstance(IntervalInstance),
}

impl fmt::Display for TemporalInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemporalInstance::InstantInstance(x) => write!(f, "{x}"),
            TemporalInstance::IntervalInstance(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for TemporalInstance {
    fn from(value: &TemporalInstance) -> Self {
        value.clone()
    }
}
impl From<InstantInstance> for TemporalInstance {
    fn from(value: InstantInstance) -> Self {
        Self::InstantInstance(value)
    }
}
impl From<IntervalInstance> for TemporalInstance {
    fn from(value: IntervalInstance) -> Self {
        Self::IntervalInstance(value)
    }
}
#[doc = "`TemporalOperands`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"oneOf\": ["]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/temporalInstance\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "      },"]
#[doc = "      {"]
#[doc = "        \"$ref\": \"#/$defs/functionRef\""]
#[doc = "      }"]
#[doc = "    ]"]
#[doc = "  },"]
#[doc = "  \"maxItems\": 2,"]
#[doc = "  \"minItems\": 2"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct TemporalOperands(pub(crate) [TemporalOperandsItem; 2usize]);
impl Deref for TemporalOperands {
    type Target = [TemporalOperandsItem; 2usize];
    fn deref(&self) -> &[TemporalOperandsItem; 2usize] {
        &self.0
    }
}
impl From<TemporalOperands> for [TemporalOperandsItem; 2usize] {
    fn from(value: TemporalOperands) -> Self {
        value.0
    }
}
impl From<&TemporalOperands> for TemporalOperands {
    fn from(value: &TemporalOperands) -> Self {
        value.clone()
    }
}
impl From<[TemporalOperandsItem; 2usize]> for TemporalOperands {
    fn from(value: [TemporalOperandsItem; 2usize]) -> Self {
        Self(value)
    }
}
#[doc = "`TemporalOperandsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/temporalInstance\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/propertyRef\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/functionRef\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum TemporalOperandsItem {
    TemporalInstance(TemporalInstance),
    PropertyRef(PropertyRef),
    FunctionRef(FunctionRef),
}

impl fmt::Display for TemporalOperandsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemporalOperandsItem::TemporalInstance(x) => write!(f, "{x}"),
            TemporalOperandsItem::PropertyRef(x) => write!(f, "{x}"),
            TemporalOperandsItem::FunctionRef(x) => write!(f, "{x}"),
        }
    }
}

impl From<&Self> for TemporalOperandsItem {
    fn from(value: &TemporalOperandsItem) -> Self {
        value.clone()
    }
}
impl From<TemporalInstance> for TemporalOperandsItem {
    fn from(value: TemporalInstance) -> Self {
        Self::TemporalInstance(value)
    }
}
impl From<PropertyRef> for TemporalOperandsItem {
    fn from(value: PropertyRef) -> Self {
        Self::PropertyRef(value)
    }
}
impl From<FunctionRef> for TemporalOperandsItem {
    fn from(value: FunctionRef) -> Self {
        Self::FunctionRef(value)
    }
}
#[doc = "`TemporalPredicate`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"args\","]
#[doc = "    \"op\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"args\": {"]
#[doc = "      \"$ref\": \"#/$defs/temporalOperands\""]
#[doc = "    },"]
#[doc = "    \"op\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"t_after\","]
#[doc = "        \"t_before\","]
#[doc = "        \"t_contains\","]
#[doc = "        \"t_disjoint\","]
#[doc = "        \"t_during\","]
#[doc = "        \"t_equals\","]
#[doc = "        \"t_finishedBy\","]
#[doc = "        \"t_finishes\","]
#[doc = "        \"t_intersects\","]
#[doc = "        \"t_meets\","]
#[doc = "        \"t_metBy\","]
#[doc = "        \"t_overlappedBy\","]
#[doc = "        \"t_overlaps\","]
#[doc = "        \"t_startedBy\","]
#[doc = "        \"t_starts\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TemporalPredicate {
    pub(crate) args: TemporalOperands,
    pub(crate) op: TemporalPredicateOp,
}

impl fmt::Display for TemporalPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({}, {})", self.op, self.args.0[0], self.args.0[1])
    }
}

impl From<&TemporalPredicate> for TemporalPredicate {
    fn from(value: &TemporalPredicate) -> Self {
        value.clone()
    }
}

#[doc = "`TemporalPredicateOp`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"t_after\","]
#[doc = "    \"t_before\","]
#[doc = "    \"t_contains\","]
#[doc = "    \"t_disjoint\","]
#[doc = "    \"t_during\","]
#[doc = "    \"t_equals\","]
#[doc = "    \"t_finishedBy\","]
#[doc = "    \"t_finishes\","]
#[doc = "    \"t_intersects\","]
#[doc = "    \"t_meets\","]
#[doc = "    \"t_metBy\","]
#[doc = "    \"t_overlappedBy\","]
#[doc = "    \"t_overlaps\","]
#[doc = "    \"t_startedBy\","]
#[doc = "    \"t_starts\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum TemporalPredicateOp {
    #[serde(rename = "t_after")]
    TAfter,
    #[serde(rename = "t_before")]
    TBefore,
    #[serde(rename = "t_contains")]
    TContains,
    #[serde(rename = "t_disjoint")]
    TDisjoint,
    #[serde(rename = "t_during")]
    TDuring,
    #[serde(rename = "t_equals")]
    TEquals,
    #[serde(rename = "t_finishedBy")]
    TFinishedBy,
    #[serde(rename = "t_finishes")]
    TFinishes,
    #[serde(rename = "t_intersects")]
    TIntersects,
    #[serde(rename = "t_meets")]
    TMeets,
    #[serde(rename = "t_metBy")]
    TMetBy,
    #[serde(rename = "t_overlappedBy")]
    TOverlappedBy,
    #[serde(rename = "t_overlaps")]
    TOverlaps,
    #[serde(rename = "t_startedBy")]
    TStartedBy,
    #[serde(rename = "t_starts")]
    TStarts,
}
impl From<&Self> for TemporalPredicateOp {
    fn from(value: &TemporalPredicateOp) -> Self {
        value.clone()
    }
}
impl fmt::Display for TemporalPredicateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::TAfter => write!(f, "t_after"),
            Self::TBefore => write!(f, "t_before"),
            Self::TContains => write!(f, "t_contains"),
            Self::TDisjoint => write!(f, "t_disjoint"),
            Self::TDuring => write!(f, "t_during"),
            Self::TEquals => write!(f, "t_equals"),
            Self::TFinishedBy => write!(f, "t_finishedBy"),
            Self::TFinishes => write!(f, "t_finishes"),
            Self::TIntersects => write!(f, "t_intersects"),
            Self::TMeets => write!(f, "t_meets"),
            Self::TMetBy => write!(f, "t_metBy"),
            Self::TOverlappedBy => write!(f, "t_overlappedBy"),
            Self::TOverlaps => write!(f, "t_overlaps"),
            Self::TStartedBy => write!(f, "t_startedBy"),
            Self::TStarts => write!(f, "t_starts"),
        }
    }
}
impl FromStr for TemporalPredicateOp {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        match value {
            "t_after" => Ok(Self::TAfter),
            "t_before" => Ok(Self::TBefore),
            "t_contains" => Ok(Self::TContains),
            "t_disjoint" => Ok(Self::TDisjoint),
            "t_during" => Ok(Self::TDuring),
            "t_equals" => Ok(Self::TEquals),
            "t_finishedBy" => Ok(Self::TFinishedBy),
            "t_finishes" => Ok(Self::TFinishes),
            "t_intersects" => Ok(Self::TIntersects),
            "t_meets" => Ok(Self::TMeets),
            "t_metBy" => Ok(Self::TMetBy),
            "t_overlappedBy" => Ok(Self::TOverlappedBy),
            "t_overlaps" => Ok(Self::TOverlaps),
            "t_startedBy" => Ok(Self::TStartedBy),
            "t_starts" => Ok(Self::TStarts),
            _ => Err(MyError::Runtime("Expected temporal function".into())),
        }
    }
}
impl TryFrom<&str> for TemporalPredicateOp {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for TemporalPredicateOp {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for TemporalPredicateOp {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
#[doc = "`TimestampInstant`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"timestamp\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"timestamp\": {"]
#[doc = "      \"$ref\": \"#/$defs/timestampString\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct TimestampInstant {
    pub(crate) timestamp: TimestampString,
}

impl fmt::Display for TimestampInstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TIMESTAMP({})", self.timestamp)
    }
}

impl From<&TimestampInstant> for TimestampInstant {
    fn from(value: &TimestampInstant) -> Self {
        value.clone()
    }
}

#[doc = "`TimestampString`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^\\\\d{4}-\\\\d{2}-\\\\d{2}T\\\\d{2}:\\\\d{2}:\\\\d{2}(?:\\\\.\\\\d+)?Z$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub(crate) struct TimestampString(String);
impl Deref for TimestampString {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl fmt::Display for TimestampString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

impl From<TimestampString> for String {
    fn from(value: TimestampString) -> Self {
        value.0
    }
}
impl From<&TimestampString> for TimestampString {
    fn from(value: &TimestampString) -> Self {
        value.clone()
    }
}
impl FromStr for TimestampString {
    type Err = MyError;
    fn from_str(value: &str) -> Result<Self, MyError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| {
                ::regress::Regex::new("^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}(?:\\.\\d+)?Z$")
                    .unwrap()
            });
        if (*PATTERN).find(value).is_none() {
            return Err(MyError::Runtime("Expected timestamp string".into()));
        }
        Ok(Self(value.to_string()))
    }
}
impl TryFrom<&str> for TimestampString {
    type Error = MyError;
    fn try_from(value: &str) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<&String> for TimestampString {
    type Error = MyError;
    fn try_from(value: &String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl TryFrom<String> for TimestampString {
    type Error = MyError;
    fn try_from(value: String) -> Result<Self, MyError> {
        value.parse()
    }
}
impl<'de> Deserialize<'de> for TimestampString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: MyError| <D::Error as de::Error>::custom(e.to_string()))
    }
}
