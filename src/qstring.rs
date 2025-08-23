// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! CQL2 friendly string type that caters for a literal character sequence to
//! be used as-is or in a case-insensitive way.
//!

use core::fmt;
use std::ops;
use unicase::UniCase;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

/// Flags to indicate how to handle a given literal string; i.e. whether to
/// ignore its case, its accents, ignore both, or use as is.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Ignoring(u8);

impl Ignoring {
    const NOTHING: Self = Self(0);
    const CASE: Self = Self(1);
    const ACCENT: Self = Self(2);
}

impl fmt::Display for Ignoring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Ignoring::NOTHING => write!(f, "/_"),
            Ignoring::CASE => write!(f, "/c"),
            Ignoring::ACCENT => write!(f, "/a"),
            _ => write!(f, "/b"),
        }
    }
}

impl ops::BitAnd for Ignoring {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl ops::BitOr for Ignoring {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// String based type used by [Queryables][1] to represent a plain string, and
/// a set of flags to indicate how to use it in case and/or accent insensitive
/// contexts.
///
/// [1]: crate::queryable::Q
#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct QString {
    /// String literal.
    inner: String,
    /// How to use in case and accent sensitive contexts.
    flags: Ignoring,
}

impl fmt::Display for QString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}{}", self.inner, self.flags)
    }
}

impl PartialEq for QString {
    fn eq(&self, other: &Self) -> bool {
        let to_icase = self.is_icase() || other.is_icase();
        let to_iaccent = self.is_iaccent() || other.is_iaccent();
        match (to_icase, to_iaccent) {
            (true, true) => {
                UniCase::new(QString::unaccent(&self.inner))
                    == UniCase::new(QString::unaccent(&other.inner))
            }
            (true, false) => UniCase::new(&self.inner) == UniCase::new(&other.inner),
            (false, true) => QString::unaccent(&self.inner) == QString::unaccent(&other.inner),
            (false, false) => self.inner == other.inner,
        }
    }
}

impl Eq for QString {}

impl QString {
    /// Constructor for a plain instance.
    pub fn plain(s: &str) -> Self {
        Self {
            inner: s.to_owned(),
            flags: Ignoring::NOTHING,
        }
    }

    /// Create a new instance from `self` w/ the added ICASE (ignore case) flag
    /// set.
    pub fn and_icase(&self) -> Self {
        Self {
            inner: self.inner.to_owned(),
            flags: self.flags.clone() | Ignoring::CASE,
        }
    }

    /// Create a new instance from `self` w/ the added IACCENT (ignore accent)
    /// flag set.
    pub fn and_iaccent(&self) -> Self {
        Self {
            inner: self.inner.to_owned(),
            flags: self.flags.clone() | Ignoring::ACCENT,
        }
    }

    /// Return the inner raw string.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Return a new string from the given argument w/ all Unicode 'Mn' (Combining
    /// Mark) codepoints removed.
    pub fn unaccent(s: &str) -> String {
        if s.is_empty() {
            return "".into();
        }

        let result: String = s.nfd().filter(|x| !is_combining_mark(*x)).nfc().collect();
        result
    }

    /// Return TRUE if this is a case-insensitive string; FALSE otherwise.
    fn is_icase(&self) -> bool {
        self.flags.0 % 2 == 1
    }

    /// Return TRUE if this is an accent-insensitive string; FALSE otherwise.
    fn is_iaccent(&self) -> bool {
        self.flags.0 >= 2
    }

    /// Whether `input` is matches the LIKE `pattern`.
    pub(crate) fn like(input: &Self, pattern: &Self) -> bool {
        // recursively compare 2 sub-strings, 1 char at a time...
        fn recursive(input: &[char], pattern: &[char]) -> bool {
            // w/ an empty pattern, only empty input matches...
            if pattern.is_empty() {
                return input.is_empty();
            }

            if input.is_empty() {
                return pattern.iter().all(|&x| x == '%');
            }

            if pattern[0] == '\\' && pattern.len() > 1 {
                let escaped = pattern[1];
                return recursive(&input[1..], &pattern[2..])
                    || (input[0] == escaped) && recursive(&input[1..], &pattern[2..]);
            }

            if pattern[0] == '%' {
                return recursive(&input[1..], pattern) || recursive(input, &pattern[1..]);
            }

            if pattern[0] == '_' {
                return recursive(&input[1..], &pattern[1..]);
            }

            (input[0] == pattern[0]) && recursive(&input[1..], &pattern[1..])
        }

        // case-insensitive mode kicks in when either arguments is unicase.
        let input_icase = input.is_icase();
        let pattern_icase = pattern.is_icase();
        let icase = input_icase || pattern_icase;
        // same deal w/ ignore-accents...
        let input_iaccent = input.is_iaccent();
        let pattern_iaccent = pattern.is_iaccent();
        let iaccent = input_iaccent || pattern_iaccent;

        let folded_input: Vec<char> = match (icase, iaccent) {
            (true, true) => {
                // compare ignoring case + accent...
                UniCase::unicode(QString::unaccent(&input.inner))
                    .to_folded_case()
                    .chars()
                    .collect()
            }
            (true, false) => {
                // compare ignoring case only...
                UniCase::unicode(input.inner.as_str())
                    .to_folded_case()
                    .chars()
                    .collect()
            }
            (false, true) => {
                // compare ignoring accents only...
                QString::unaccent(&input.inner).as_str().chars().collect()
            }
            (false, false) => {
                // plain strings all the way...
                input.inner.chars().collect()
            }
        };

        let folded_pattern: Vec<char> = match (icase, iaccent) {
            (true, true) => UniCase::unicode(QString::unaccent(&pattern.inner))
                .to_folded_case()
                .chars()
                .collect(),
            (true, false) => UniCase::unicode(&pattern.inner)
                .to_folded_case()
                .chars()
                .collect(),
            (false, true) => QString::unaccent(&pattern.inner).chars().collect(),
            (false, false) => pattern.inner.as_str().chars().collect(),
        };

        recursive(&folded_input, &folded_pattern)
    }

    /// Return TRUE if this is a plain string; FALSE otherwise.
    #[cfg(test)]
    fn is_plain(&self) -> bool {
        self.flags.0 == 0
    }

    /// Constructor for an accent-insensitive instance.
    #[cfg(test)]
    pub fn iaccent(s: &str) -> Self {
        Self {
            inner: s.to_owned(),
            flags: Ignoring::ACCENT,
        }
    }

    /// Constructor for a case-insensitive instance.
    #[cfg(test)]
    pub fn icase(s: &str) -> Self {
        Self {
            inner: s.to_owned(),
            flags: Ignoring::CASE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, distr::Alphanumeric};

    #[test]
    // #[tracing_test::traced_test]
    fn test_display() {
        const S1: &str = "/chișinău/_";
        const S2: &str = "/CHIȘINĂU/c";
        const S3: &str = "/CHIȘINĂU/a";
        const S5: &str = "/chișinău/b";

        let s1 = QString::plain("chișinău");
        assert!(s1.is_plain());
        assert_eq!(s1.to_string(), S1);

        let s2 = QString::icase("CHIȘINĂU");
        assert!(s2.is_icase());
        assert_eq!(s2.to_string(), S2);

        let s3 = QString::iaccent("CHIȘINĂU");
        assert!(s3.is_iaccent());
        assert_eq!(s3.to_string(), S3);

        let s4 = s1.and_icase();
        assert!(s1.is_plain());
        assert!(!s4.is_plain());
        assert!(s4.is_icase());

        let s5 = s4.and_iaccent();
        assert_eq!(s5.to_string(), S5);
        assert!(s5.is_icase());
        assert!(s5.is_iaccent());
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_equality() {
        let s1 = QString::plain("chisinau");
        let s2 = QString::icase("CHISINAU");
        let s3 = QString::iaccent("chișinău");
        let s4 = QString::iaccent("CHIȘINĂU").and_icase();
        let s5 = QString::plain("CHISINAU").and_iaccent();

        assert!(s1 == s2);
        assert!(s3 == s4);
        assert!(s4 == s5);

        // all together now...
        let s4 = s2.and_iaccent();
        let s5 = s3.and_icase();

        assert!(s1 == s3);
        assert!(s1 == s4);
        assert!(s1 == s5);

        // remain valid after same bit set multiple times...
        let s5 = s4.and_iaccent();
        assert_eq!(s2, s4);
        assert_eq!(s2, s5);
        assert_eq!(s4, s5);
        assert!(s5.is_icase());
        assert!(s5.is_iaccent());
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_unaccent() {
        let slo = "chisinau";
        let shi = "CHISINAU";

        let iaccented = QString::unaccent("chișinău");
        // tracing::debug!("iaccented = '{iaccented}'");
        assert_eq!(iaccented, slo);

        let iaccented = QString::unaccent("CHIȘINĂU");
        // tracing::debug!("iaccented = '{iaccented}'");
        assert_eq!(iaccented, shi);

        // now test when LIKE wildcard characters are included...

        let iaccented = QString::unaccent("Chiș%");
        tracing::debug!("iaccented = '{iaccented}'");
        assert_eq!(iaccented, "Chis%");

        let iaccented = QString::unaccent("cHis%");
        tracing::debug!("iaccented = '{iaccented}'");
        assert_eq!(iaccented, "cHis%");

        // ...and when combined w/ icase...

        let a = QString::unaccent(&UniCase::new("chișinău%").to_folded_case());
        tracing::debug!("a = '{a}'");
        let b = UniCase::new(QString::unaccent("chișinău%")).to_folded_case();
        tracing::debug!("b = '{b}'");
        assert_eq!(a, b);
    }

    fn starts_with_foo() -> String {
        let size: usize = rand::rng().random_range(5..15);
        let s = (0..size)
            .map(|_| rand::rng().sample(Alphanumeric) as char)
            .collect();
        let hit = rand::rng().random_bool(0.25);
        if hit { format!("Foo{s}") } else { s }
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_like_small() {
        let pattern = QString::icase("foo%");
        for _ in 0..1000 {
            let s = starts_with_foo();
            if s.starts_with("Foo") {
                let input = QString::icase(&s);
                let result = QString::like(&input, &pattern);
                if !result {
                    eprintln!("*** Was expecting '{s}' to succeed");
                    panic!("Ooops")
                }
            };
        }
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_like_capital() {
        let pattern = QString::icase("FOO%");
        for _ in 0..1000 {
            let s = starts_with_foo();
            if s.starts_with("Foo") {
                let input = QString::icase(&s);
                let result = QString::like(&input, &pattern);
                if !result {
                    eprintln!("*** Was expecting '{s}' to succeed");
                    panic!("Ooops")
                }
            };
        }
    }

    #[test]
    fn test_nfkd() {
        const S: &str = "ἄbc";

        let r1: String = S
            .chars()
            .map(|c| UnicodeNormalization::nfkd(c).nth(0).unwrap())
            .collect();
        tracing::debug!("'{r1}'");
        assert_eq!(r1, "αbc");

        assert_eq!(QString::unaccent(S), r1);
    }
}
