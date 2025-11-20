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
pub(crate) struct Ignoring(u8);

impl Ignoring {
    const NEITHER: Self = Self(0);
    const CASE: Self = Self(1);
    const ACCENT: Self = Self(2);
}

impl fmt::Display for Ignoring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Ignoring::NEITHER => write!(f, "/_"),
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
    pub fn plain<S: Into<String>>(s: S) -> Self {
        Self {
            inner: s.into(),
            flags: Ignoring::NEITHER,
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

    /// Return a this [`Ignoring`] flags as a byte.
    pub(crate) fn flags(&self) -> u8 {
        self.flags.0
    }

    /// Return a reference to this `inner` string.
    pub(crate) fn inner(&self) -> &str {
        &self.inner
    }

    /// Return TRUE if this is a plain string; FALSE otherwise.
    #[allow(dead_code)]
    pub(crate) fn is_plain(&self) -> bool {
        self.flags.0 == 0
    }

    /// Return TRUE if this is a case-insensitive string; FALSE otherwise.
    pub(crate) fn is_icase(&self) -> bool {
        self.flags.0 % 2 == 1
    }

    /// Return TRUE if this is an accent-insensitive string; FALSE otherwise.
    pub(crate) fn is_iaccent(&self) -> bool {
        self.flags.0 >= 2
    }

    /// Whether `input` matches the LIKE `pattern`.
    pub(crate) fn like(input: &Self, pattern: &Self) -> bool {
        const WC: char = '%';

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

        // reduce multiple occurences of unescaped wildcards (uwc) to just one.
        fn reduce_wildcards(pattern: &str) -> Vec<char> {
            let mut result: Vec<char> = Vec::with_capacity(pattern.len());
            let mut chars = pattern.chars();
            let mut saw_uwc = false;
            while let Some(c) = chars.next() {
                let state = if c == '\\' {
                    result.push('\\');
                    if let Some(n) = chars.next() {
                        result.push(n);
                    }
                    false
                } else if c == WC {
                    if !saw_uwc {
                        result.push(WC);
                    }
                    true
                } else {
                    result.push(c);
                    false
                };
                saw_uwc = state;
            }
            result
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
            (true, true) => UniCase::unicode(QString::unaccent(&input.inner))
                .to_folded_case()
                .chars()
                .collect(),
            (true, false) => UniCase::unicode(input.inner.as_str())
                .to_folded_case()
                .chars()
                .collect(),
            (false, true) => QString::unaccent(&input.inner).as_str().chars().collect(),
            (false, false) => input.inner.chars().collect(),
        };

        let binding1 = UniCase::unicode(QString::unaccent(&pattern.inner)).to_folded_case();
        let binding2 = UniCase::unicode(&pattern.inner).to_folded_case();
        let binding3 = QString::unaccent(&pattern.inner);
        let folded_pattern = match (icase, iaccent) {
            (true, true) => binding1.as_str(),
            (true, false) => binding2.as_str(),
            (false, true) => binding3.as_str(),
            (false, false) => pattern.inner.as_str(),
        };

        // replace repeated wildcards w/ one. mind escaped instances.
        let reduced_pattern = reduce_wildcards(folded_pattern);

        recursive(&folded_input, &reduced_pattern)
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
    use rand::{
        Rng,
        distr::{
            Alphanumeric,
            uniform::{UniformChar, UniformSampler},
        },
    };
    use tracing::debug;

    #[test]
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
    fn test_unaccent() {
        let slo = "chisinau";
        let shi = "CHISINAU";
        let aaaa = ["ẵ", "aͣ", "ą", "ǟ", "aₐ", "ắ"];
        let nota = ["ɑ", "Ⓐ", "ⓐ", "æ", "ǽ", "ⱥ", "ᶏ", "ₐ"];

        let iaccented = QString::unaccent("chișinău");
        assert_eq!(iaccented, slo);

        let iaccented = QString::unaccent("CHIȘINĂU");
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

        // test 'a' combos...
        for c in aaaa.into_iter() {
            let a = QString::unaccent(c);
            assert!(a.starts_with('a'));
        }
        for c in nota.into_iter() {
            let a = QString::unaccent(c);
            assert!(!a.starts_with('a'));
        }
    }

    fn starts_with_foo() -> String {
        let mut rng = rand::rng();
        let size: usize = rng.random_range(5..15);
        let s = (0..size)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();
        let hit = rng.random_bool(0.25);
        if hit { format!("Foo{s}") } else { s }
    }

    #[test]
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

    #[test]
    #[tracing_test::traced_test]
    fn test_like_bench() {
        // generate random word, 5 to 10 characters long from latin characters.
        fn random_latin_word() -> String {
            let mut rng = rand::rng();
            let len: usize = Rng::random_range(&mut rng, 5..10);
            let dist = UniformChar::new_inclusive('\u{0041}', '\u{024F}').unwrap();
            (0..len).map(|_| dist.sample(&mut rng)).collect()
        }

        const PATTERN: &str = "Ä%%";
        let pattern = QString::plain(PATTERN).and_iaccent().and_icase();
        for _ in 0..1000 {
            let raw = random_latin_word();
            let cooked = raw
                .nfd()
                .filter(|x| !is_combining_mark(*x))
                .nfc()
                .collect::<String>();
            let ricotta = UniCase::unicode(&cooked).to_folded_case();
            let expected = ricotta.starts_with('a');
            let input = QString::plain(&raw).and_icase().and_iaccent();
            let actual = QString::like(&input, &pattern);
            if actual != expected {
                debug!("    raw: '{raw}' {}", raw.escape_unicode());
                debug!("  cotta: '{cooked}' {}", cooked.escape_unicode());
                debug!("ricotta: '{ricotta}' {}", ricotta.escape_unicode());
                panic!(
                    "IA(IC({input})) LIKE IC(IA({pattern})) is {actual} but expected {expected}"
                );
            }
        }
    }
}
