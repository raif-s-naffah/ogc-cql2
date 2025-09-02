// SPDX-License-Identifier: Apache-2.0

//! Test escaping in string literals.
//!
//! Given:
//!     * One or more data sources containing string literals with embedded
//!       single quotation (') and/or BELL, and/or BACKSPACE, and/or HORIZONTAL
//!       TAB, and/or NEWLINE, and/or VERTICAL TAB, and/or FORM FEED, and/or
//!       CARRIAGE RETURN characters.
//! When:
//!     Decode each string literal.
//! Then:
//! * assert that the escaped embedded characters have been correctly recovered.
//!

use ogc_cql2::{Context, Evaluator, EvaluatorImpl, Expression, Outcome, Q, Resource};
use std::error::Error;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test() -> Result<(), Box<dyn Error>> {
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

    let shared_ctx = Context::new().freeze();
    for (ndx, (s, expected)) in TV.iter().enumerate() {
        let mut evaluator = EvaluatorImpl::new(shared_ctx.clone());
        let input = format!("x = {s}");
        let expr = Expression::try_from_text(&input);
        assert!(expr.is_ok());
        evaluator.setup(expr.unwrap())?;

        let feat = Resource::from([
            ("fid".into(), Q::try_from(ndx)?),
            ("x".into(), Q::new_plain_str(&expected)),
        ]);

        let res = evaluator.evaluate(&feat)?;
        assert!(matches!(res, Outcome::T));

        evaluator.teardown()?
    }

    Ok(())
}
