// SPDX-License-Identifier: Apache-2.0

use ogc_cql2::Expression;
use tracing_test::traced_test;

#[test]
fn test_property_expression() {
    const CQL1: &str = r#"{ "op": "avg", "args": [ { "property": "windSpeed" } ] }"#;
    // our JSON encoded representation ALWAYS quotes properties...
    const CQL2: &str = r#"avg("windSpeed")"#;

    let x1 = Expression::try_from_json(CQL1);
    assert!(x1.is_ok());
    let x1 = x1.unwrap();
    assert_eq!(x1.to_string(), CQL2);

    let x2 = Expression::try_from_text(CQL2);
    assert!(x2.is_ok());
    let x2 = x2.unwrap();

    assert_eq!(x1.to_string(), x2.to_string());
}

#[test]
fn test_c6_01() {
    const CQL1: &str = r#"{ "op": "avg", "args": [ { "property": "windSpeed" } ] }"#;
    // our JSON encoded representation ALWAYS quote properties...
    const CQL2: &str = r#"avg("windSpeed")"#;

    let x1 = Expression::try_from_json(CQL1);
    assert!(x1.is_ok());
    let x1 = x1.unwrap();
    assert_eq!(x1.to_string(), CQL2);

    let x2 = Expression::try_from_text(CQL2);
    assert!(x2.is_ok());
}

#[test]
fn test_c7_01() {
    const CQL1: &str = r#"{ "op": "like", "args": [ { "property": "name" }, "Smith%" ] }"#;
    // our JSON encoded representation ALWAYS quote properties...
    const CQL2: &str = r#""name" LIKE 'Smith%'"#;

    let x1 = Expression::try_from_json(CQL1);
    assert!(x1.is_ok());
    let x1 = x1.unwrap();
    assert_eq!(x1.to_string(), CQL2);

    let x2 = Expression::try_from_text(CQL2);
    assert!(x2.is_ok());
}

#[test]
fn test_c7_03b() {
    const CQL1: &str = r#"
{
  "op": "not",
  "args": [
    {
      "op": "in",
      "args": [
        { "property": "category" },
        [ 1, 2, 3, 4 ]
      ]
    }
  ]
}
"#;
    const CQL2: &str = r#"NOT( "category" IN (1, 2, 3, 4) )"#;

    let x1 = Expression::try_from_json(CQL1);
    assert!(x1.is_ok());
    let x1 = x1.unwrap();
    assert_eq!(x1.to_string(), CQL2);

    let x2 = Expression::try_from_text(CQL2);
    assert!(x2.is_ok());

    // NOTE (rsn) 20250728 - we now always defer the evaluation so wont be able
    // to see the equivalence that early...
    //
    // negating an IN list expression is equivalent to using infix NOT...
    // const CQL3: &str = r#""category" NOT IN (1, 2, 3, 4)"#;
    // let x3 = Expression::try_from_text(CQL3);
    // // tracing::debug!("x3 = {x3:?}");
    // assert!(x3.is_ok());
    // let x3 = x3.unwrap();

    // assert_eq!(x2.as_text_encoded(), x3.as_text_encoded());
}

#[test]
fn test_ex76() {
    const CQL1: &str = r#"
{
  "op": "<=",
  "args": [
    { "property": "value" },
    { "op": "^", "args": [ 2, { "property": "foo" } ] }
  ]
}
"#;
    const CQL2: &str = r#""value" <= 2 ^ "foo""#;
    // parentheses do not alter exponentiation operator priority...
    const CQL3: &str = r#""value" <= (2 ^ "foo")"#;

    let x1 = Expression::try_from_json(CQL1);
    assert!(x1.is_ok());
    let x1 = x1.unwrap();
    assert_eq!(x1.to_string(), CQL2);

    let x2 = Expression::try_from_text(CQL2);
    assert!(x2.is_ok());
    let x2 = x2.unwrap();

    let x3 = Expression::try_from_text(CQL3);
    assert!(x3.is_ok());
    let x3 = x3.unwrap();

    assert_eq!(x2.as_text_encoded(), x3.as_text_encoded());
}

#[test]
fn test_ex_82() {
    const CQL1: &str = r#"{
  "op": "a_overlaps",
  "args": [
    { "property": "values" },
    [ { "timestamp": "2012-08-10T05:30:00Z" }, { "date": "2010-02-10" }, false ]
  ]
}"#;
    // our JSON encoded representation ALWAYS quote properties...
    const CQL2: &str =
        r#"a_overlaps("values", (TIMESTAMP('2012-08-10T05:30:00Z'), DATE('2010-02-10'), FALSE))"#;

    let x1 = Expression::try_from_json(CQL1);
    assert!(x1.is_ok());
    let x1 = x1.unwrap();
    assert_eq!(x1.to_string(), CQL2);

    let x2 = Expression::try_from_text(CQL2);
    assert!(x2.is_ok());
}

#[test]
#[traced_test]
fn test_ex85() {
    const CQL1: &str = r#"{
"op": "=",
"args": [ {
    "property": "value"
}, {
    "op": "-",
    "args": [ {
        "op": "+",
        "args": [ {
            "op": "*",
            "args": [ { "op": "*", "args": [ -1, { "property": "foo" } ] }, 2.0 ]
        }, {
            "op": "/",
            "args": [ { "property": "bar" }, 6.1234 ]
        } ]
    }, {
        "op": "^", "args": [ { "property": "x" }, 2.0 ]
    }
    ]
} ] }"#;
    const CQL2: &str = r#""value" = (((-1 * "foo") * 2) + ("bar" / 6.1234)) - ("x" ^ 2)"#;

    let x1 = Expression::try_from_json(CQL1);
    assert!(x1.is_ok());
    let x1 = x1.unwrap();
    assert_eq!(x1.to_string(), CQL2);

    let x2 = Expression::try_from_text(CQL2);
    assert!(x2.is_ok());
}
