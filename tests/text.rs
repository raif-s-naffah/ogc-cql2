// SPDX-License-Identifier: Apache-2.0

use ogc_cql2::Expression;

#[test]
// #[tracing_test::traced_test]
fn test_ex13() {
    const CQL: &str = r#"owner NOT LIKE '%Mike%'
"#;
    let exp = Expression::try_from_text(CQL);
    // tracing::debug!("exp = {exp:?}");
    assert!(exp.is_ok());
}

#[test]
// #[tracing_test::traced_test]
fn test_ex38_alt01() {
    const CQL1: &str = r#""value" NOT BETWEEN 10 AND 20"#;
    const CQL2: &str = r#""value" Not BETWEEN 10 AND 20"#;

    let exp1 = Expression::try_from_text(CQL1);
    // tracing::debug!("exp = {exp:?}");
    assert!(exp1.is_ok());

    let exp2 = Expression::try_from_text(CQL2);
    // tracing::debug!("exp2 = {exp2:?}");
    assert!(exp2.is_ok());

    let x1 = exp1.unwrap();
    assert!(matches!(x1, Expression::Text(_)));
    let x2 = exp2.unwrap();
    assert!(matches!(x2, Expression::Text(_)));
    match (x1, x2) {
        (Expression::Text(x), Expression::Text(y)) => assert_eq!(x, y),
        _ => panic!("Expected 2 text-encoded expressions"),
    }
}

#[test]
// #[tracing_test::traced_test]
fn test_ex44_alt01() {
    const CQL: &str = r#""value" IS NULL OR "value" BETWEEN 10 AND 20
"#;
    let exp = Expression::try_from_text(CQL);
    // tracing::debug!("exp = {exp:?}");
    assert!(exp.is_ok());
}

#[test]
// #[tracing_test::traced_test]
fn test_ex49_alt01() {
    // Polygon 3D w/o Z...
    const CQL: &str = r#"S_WITHIN(POLYGON ((-49.88024 0.5 -75993.341684, -1.5 -0.99999 -100000.0, 0.0 0.5 -0.333333, -49.88024 0.5 -75993.341684), (-65.887123 2.00001 -100000.0, 0.333333 -53.017711 -79471.332949, 180.0 0.0 1852.616704, -65.887123 2.00001 -100000.0)), "geometry")
"#;
    let exp = Expression::try_from_text(CQL);
    // tracing::debug!("exp = {exp:?}");
    assert!(exp.is_ok());
}

#[test]
// #[tracing_test::traced_test]
fn test_ex68() {
    const CQL: &str = r#"Foo("geometry") = TRUE
"#;
    let exp = Expression::try_from_text(CQL);
    // tracing::debug!("exp = {exp:?}");
    assert!(exp.is_ok());
}

#[test]
// #[tracing_test::traced_test]
fn test_ex69() {
    const CQL: &str = r#"FALSE <> Bar("geometry", 100, 'a', 'b', FALSE)"#;

    let exp = Expression::try_from_text(CQL);
    tracing::debug!("exp = {exp:?}");
    assert!(exp.is_ok());
}

#[test]
// #[tracing_test::traced_test]
fn test_ex76() {
    const CQL: &str = r#""value" <= (2 ^ "foo")"#;

    let exp = Expression::try_from_text(CQL);
    // tracing::debug!("exp = {exp:?}");
    assert!(exp.is_ok());
}

#[test]
// #[tracing_test::traced_test]
fn test_ex85_alt01() {
    const CQL: &str = r#"value = - foo * 2.0 + "bar" / 6.1234 - "x" ^ 2.0"#;

    let exp = Expression::try_from_text(CQL);
    // tracing::debug!("exp = {exp:?}");
    assert!(exp.is_ok());
}

#[test]
// #[tracing_test::traced_test]
fn test_ex85_alt02() {
    // this is what the "example85.json" input parse result 'to_string()' spits
    // out...  it should result into successful text-encoding parse exactly
    // equivalent to (a) the one above (marked as alt01)...
    //   value = - foo * 2.0 + "bar" / 6.1234 - "x" ^ 2.0
    // as well as (b) the original "example85.text" input...
    //   "value" = ((((-1 * "foo") * 2.0) + ("bar" / 6.1234)) - ("x" ^ 2.0))
    // all 3 variants differ in the degree of excessive/superfluous (due to
    // operator precedence) surrounding sub-expressions w/ parenstheses.
    const CQL: &str = r#""value" = (((-1 * "foo") * 2) + ("bar" / 6.1234)) - ("x" ^ 2)"#;

    let exp = Expression::try_from_text(CQL);
    // tracing::debug!("exp = {exp:?}");
    assert!(exp.is_ok());
}
