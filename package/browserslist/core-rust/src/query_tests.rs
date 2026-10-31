//! What:
//!  Tests for exact-text query matching and the "not implemented" refusals.
//! Why:
//!  The user's rule is that every non-conforming query throws "not implemented", so representative unsupported
//!  forms (other counts, spellings, browsers, composition, and whole other query families) are pinned here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('parseQuery', () => { /* the seven forms, then representative refusals */ });
//! ```

use crate::query::{Browser, Query, parse_query};

/// Each of the seven lines maps to its query.
#[test]
fn accepts_the_seven_lines() {
    assert_eq!(parse_query(1, "last 1 Chrome versions").expect("q"), Query::LastVersions(Browser::Chrome));
    assert_eq!(parse_query(2, "last 1 ChromeAndroid versions").expect("q"), Query::LastVersions(Browser::ChromeAndroid));
    assert_eq!(parse_query(3, "last 1 Android versions").expect("q"), Query::LastVersions(Browser::Android));
    assert_eq!(parse_query(4, "Firefox ESR").expect("q"), Query::FirefoxEsr);
    assert_eq!(parse_query(5, "last 1 FirefoxAndroid versions").expect("q"), Query::LastVersions(Browser::FirefoxAndroid));
    assert_eq!(parse_query(6, "last 1 node major versions").expect("q"), Query::LastNodeMajorVersions);
    assert_eq!(parse_query(7, "not dead").expect("q"), Query::NotDead);
}

/// The browser keys are the caniuse names browserslist resolves the written names to.
#[test]
fn browser_keys() {
    assert_eq!(Browser::Chrome.agent_key(), "chrome");
    assert_eq!(Browser::ChromeAndroid.agent_key(), "and_chr");
    assert_eq!(Browser::Android.agent_key(), "android");
    assert_eq!(Browser::FirefoxAndroid.agent_key(), "and_ff");
}

/// Spellings npm browserslist would accept are still refused, and the message names the query and the line.
#[test]
fn refuses_other_spellings_and_forms() {
    for text in [
        "last 2 Chrome versions",
        "last 0 Chrome versions",
        "last 01 Chrome versions",
        "last 1 chrome versions",
        "LAST 1 CHROME VERSIONS",
        "last 1 Chrome version",
        "last  1 Chrome versions",
        "last\t1 Chrome versions",
        "last\u{a0}1 Chrome versions",
        "firefox esr",
        "FX ESR",
        "Not Dead",
        "notdead",
        "not not dead",
        "dead",
        "last 1 Safari versions",
        "last 1 node versions",
        "last 2 node major versions",
        "last 1 Chrome major versions",
        "last 1 years",
        "> 0.5%",
        "defaults",
        "extends x",
        "chrome 100",
        "node 18",
        "supports es6-module",
        "cover 99.5%",
        "since 2020",
        "baseline widely available",
        "maintained node versions",
        "last 1 Chrome versions or last 1 Android versions",
        "last 1 Chrome versions and last 1 Android versions",
        "",
        " ",
    ] {
        let message = parse_query(9, text).expect_err("refused").to_string();
        assert!(message.starts_with("not implemented: "), "{text:?}: {message}");
        assert!(message.contains(&format!("{text:?}")), "{text:?}: {message}");
        assert!(message.contains("line 9"), "{text:?}: {message}");
    }
}
