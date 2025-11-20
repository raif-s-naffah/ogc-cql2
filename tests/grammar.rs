// SPDX-License-Identifier: Apache-2.0

use ogc_cql2::Expression;
use std::{error::Error, fs};
use walkdir::WalkDir;

/// Parse all 120 .txt files in "test/samples/text" and print outcome.
#[test]
// #[tracing_test::traced_test]
pub(crate) fn test_text_samples() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for entry in WalkDir::new("tests/samples/text") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        // tracing::debug!("===== {} =====", entry.path().display());
        let src = fs::read_to_string(entry.path()).expect("Failed reading sample text");
        // tracing::debug!("(TEXT) {src}");

        let _expr = Expression::try_from_text(&src).expect("Failed...");
        // tracing::debug!("{_expr}");

        count += 1;
    }

    assert_eq!(count, 120);
    Ok(())
}

/// Parse all 109 .json files in "test/samples/json" and print outcome.
#[test]
#[tracing_test::traced_test]
pub(crate) fn test_json_samples() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for entry in WalkDir::new("tests/samples/json") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        if entry.path().ends_with("validate.sh") {
            continue;
        }

        // tracing::debug!("===== {} =====", entry.path().display());
        let src = fs::read_to_string(entry.path()).expect("Failed reading sample json");
        // tracing::debug!("(JSON) {src}");

        let expr = Expression::try_from_json(&src).expect("Failed...");
        let _text = expr.to_string();
        // tracing::debug!("{_text}");

        count += 1;
    }

    assert_eq!(count, 109);
    Ok(())
}

/// Ensure JSON-encoded parse result is usable by Text-encoded parser.
#[test]
#[tracing_test::traced_test]
fn test_json_compat() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for entry in WalkDir::new("tests/samples/json") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        if entry.path().ends_with("validate.sh") {
            continue;
        }

        // tracing::debug!("===== {} =====", entry.path().display());
        let src = fs::read_to_string(entry.path()).expect("Failed reading sample json");
        // tracing::debug!("(JSON) {src}");

        let expr = Expression::try_from_json(&src).expect("Failed...");
        let text = expr.to_string();
        // tracing::debug!("{text}");

        // also, the to_string() output of the JSON version should be text-encoded
        // friendly...
        let expr2 = Expression::try_from_text(&text);
        // tracing::debug!("expr2 = {expr2:?}");
        assert!(expr2.is_ok());
        // tracing::debug!("(TEXT) {}", expr2.unwrap());

        count += 1;
    }

    assert_eq!(count, 109);
    Ok(())
}
