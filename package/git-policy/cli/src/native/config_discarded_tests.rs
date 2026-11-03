//! What:
//!  Controls of the `discardedCopies` section: defaults, accepted spellings and the rejection of every value that is no limit.
//! Why:
//!  A limit that reads as zero would remove every copy, and one that reads as "none" would never remove one; both are
//!  refused by name, like every other bad value of the file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseConfig('{"discardedCopies":{"maxAgeDays":0}}')).toThrow(/discardedCopies.maxAgeDays/);
//! ```

use crate::config_parse::parse_config;
use crate::discarded_prune::{DEFAULT_MAX_AGE_DAYS, DEFAULT_MAX_BYTES, Limits};

/// The limits a document gives.
fn limits_of(text: &str) -> Limits {
    return parse_config(text).expect("an accepted document").discarded;
}

/// The message a rejected document gives.
fn rejection_of(text: &str) -> String {
    return parse_config(text).expect_err("a rejected document").message;
}

/// Without the section, or with an empty one, the defaults apply: thirty days and one gibibyte.
#[test]
fn the_defaults_apply_without_the_section_or_its_keys() {
    let defaults: Limits = Limits { max_age_days: DEFAULT_MAX_AGE_DAYS, max_bytes: DEFAULT_MAX_BYTES };
    assert_eq!(limits_of("{}"), defaults);
    assert_eq!(limits_of(r#"{ "discardedCopies": {} }"#), defaults);
    assert_eq!(limits_of(r#"{ "discardedCopies": { "maxAgeDays": 7 } }"#), Limits { max_age_days: 7, ..defaults });
    assert_eq!(limits_of(r#"{ "discardedCopies": { "maxSize": "2k" } }"#), Limits { max_bytes: 2048, ..defaults });
}

/// Both keys together, a size as text with Git's suffixes or as a number of bytes, and comments as in any other section.
#[test]
fn the_keys_set_the_limits() {
    assert_eq!(
        limits_of("{ // keep a quarter of a gibibyte for three months\n \"discardedCopies\": { \"maxAgeDays\": 90, \"maxSize\": \"256m\" } }"),
        Limits { max_age_days: 90, max_bytes: 256 * 1024 * 1024 }
    );
    assert_eq!(limits_of(r#"{ "discardedCopies": { "maxSize": 4096 } }"#).max_bytes, 4096);
    assert_eq!(limits_of(r#"{ "discardedCopies": { "maxSize": "1G" } }"#).max_bytes, 1 << 30);
}

/// A value that is no limit is refused with the key named; zero never means "no limit" or "remove everything".
#[test]
fn a_value_that_is_no_limit_is_rejected_by_key() {
    for (text, key) in [
        (r#"{ "discardedCopies": { "maxAgeDays": 0 } }"#, "discardedCopies.maxAgeDays"),
        (r#"{ "discardedCopies": { "maxAgeDays": -3 } }"#, "discardedCopies.maxAgeDays"),
        (r#"{ "discardedCopies": { "maxAgeDays": 1.5 } }"#, "discardedCopies.maxAgeDays"),
        (r#"{ "discardedCopies": { "maxAgeDays": "30" } }"#, "discardedCopies.maxAgeDays"),
        (r#"{ "discardedCopies": { "maxAgeDays": null } }"#, "discardedCopies.maxAgeDays"),
        (r#"{ "discardedCopies": { "maxSize": 0 } }"#, "discardedCopies.maxSize"),
        (r#"{ "discardedCopies": { "maxSize": "0" } }"#, "discardedCopies.maxSize"),
        (r#"{ "discardedCopies": { "maxSize": "0k" } }"#, "discardedCopies.maxSize"),
        (r#"{ "discardedCopies": { "maxSize": "" } }"#, "discardedCopies.maxSize"),
        (r#"{ "discardedCopies": { "maxSize": "big" } }"#, "discardedCopies.maxSize"),
        (r#"{ "discardedCopies": { "maxSize": "1.5g" } }"#, "discardedCopies.maxSize"),
        (r#"{ "discardedCopies": { "maxSize": -1 } }"#, "discardedCopies.maxSize"),
        (r#"{ "discardedCopies": { "maxSize": true } }"#, "discardedCopies.maxSize"),
    ] {
        let message: String = rejection_of(text);
        assert!(message.contains(key), "{text}: {message}");
    }
}

/// An unknown key under the section, and a section that is not an object, are refused with the accepted keys named.
#[test]
fn an_unknown_key_or_a_wrong_shape_is_rejected() {
    let unknown: String = rejection_of(r#"{ "discardedCopies": { "maxCopies": 3 } }"#);
    assert!(unknown.contains("discardedCopies.maxCopies") && unknown.contains("maxAgeDays") && unknown.contains("maxSize"), "{unknown}");
    assert!(rejection_of(r#"{ "discardedCopies": 30 }"#).contains("discardedCopies must be an object"));
    assert!(rejection_of(r#"{ "discardedCopy": {} }"#).contains("discardedCopies"), "the accepted keys name the section");
}
