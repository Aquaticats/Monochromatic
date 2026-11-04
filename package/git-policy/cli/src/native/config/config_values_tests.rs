//! What:
//!  Boundary controls for the key-naming JSONC value readers.
//! Why:
//!  Each reader must accept exactly its type and name the rejected key.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => boolean(parse('"true"'), 'a.b')).toThrow(/a\.b must be true or false/);
//! ```

/// Import the readers under test and the parser that builds their inputs.
use super::{
    MAX_SAFE_INTEGER, boolean, kind_name, member_keys, member_path, safe_integer, strings, text,
    wrong_kind,
};
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};

/// Parse one JSONC value fixture;
///  a fixture that does not parse is a broken test.
/// The parser accepts only an object or array as a document,
///  so scalars ride inside an array.
fn value(source: &str) -> JsoncValue {
    let document: JsoncValue =
        parse_jsonc(format!("[{source}]").as_str()).expect("fixture is valid JSONC");
    let elements: &[JsoncValue] = document.elements().expect("array wrapper");
    assert_eq!(elements.len(), 1, "one value per fixture");
    return elements[0].clone();
}

/// Dotted paths omit the separator only at the top level.
#[test]
fn member_paths_join_with_one_dot() {
    assert_eq!(member_path("", "policies"), "policies");
    assert_eq!(
        member_path("policies", "final-newline"),
        "policies.final-newline"
    );
    assert_eq!(member_path("a.b", ""), "a.b.");
}

/// Every JSON kind has its own diagnostic name.
#[test]
fn kind_names_cover_every_json_type() {
    for (source, expected) in [
        ("null", "null"),
        ("true", "a boolean"),
        ("1", "a number"),
        ("\"x\"", "a string"),
        ("[]", "an array"),
        ("{}", "an object"),
    ] {
        assert_eq!(kind_name(&value(source)), expected);
    }
}

/// Null is explained with its remedy;
///  other kinds name what was found;
///  the top level is described.
#[test]
fn wrong_kind_messages_name_key_found_kind_and_expectation() {
    assert_eq!(
        wrong_kind(&value("1"), "hooks", "an object").message,
        "Configuration key hooks must be an object, found a number."
    );
    assert_eq!(
        wrong_kind(&value("null"), "hooks", "an object").message,
        "Configuration key hooks must not be null; remove it to use the default or give it an object."
    );
    assert_eq!(
        wrong_kind(&value("[]"), "", "an object").message,
        "The configuration document must be an object, found an array."
    );
    assert_eq!(
        wrong_kind(&value("null"), "", "an object").message,
        "The configuration document must not be null; remove it to use the default or give it an object."
    );
}

/// Keys are returned decoded and in source order.
#[test]
fn member_keys_decode_escapes_in_source_order() {
    let document = value(r#"{ "b": 1, "a": 2, "": 3 }"#);
    let entries = document.entries().expect("object fixture");
    assert_eq!(
        member_keys(entries, "").expect("distinct keys"),
        vec![String::from("b"), String::from("a"), String::new()]
    );
}

/// A key repeated literally or through an escape alias is ambiguous and named.
#[test]
fn member_keys_reject_duplicates_after_decoding() {
    for (source, path, expected) in [
        (
            r#"{ "hooks": {}, "hooks": {} }"#,
            "",
            "Configuration key hooks is defined more than once; keep exactly one definition.",
        ),
        (
            r#"{ "a": 1, "b": 2, "a": 3 }"#,
            "policies",
            "Configuration key policies.a is defined more than once; keep exactly one definition.",
        ),
    ] {
        let document = value(source);
        let entries = document.entries().expect("object fixture");
        assert_eq!(
            member_keys(entries, path).expect_err("duplicate").message,
            expected
        );
    }
}

/// A lone surrogate key is rejected,
///  naming its container instead of printing invalid text.
#[test]
fn member_keys_reject_unpaired_surrogates() {
    let document = value(r#"{ "\ud800": 1 }"#);
    let entries = document.entries().expect("object fixture");
    let top = member_keys(entries, "")
        .expect_err("lone surrogate")
        .message;
    assert!(
        top.starts_with("A key under the configuration document is not valid Unicode text"),
        "{top}"
    );
    let nested = member_keys(entries, "policies")
        .expect_err("lone surrogate")
        .message;
    assert!(
        nested.starts_with("A key under policies is not valid Unicode text"),
        "{nested}"
    );
}

/// Only JSON booleans are switches.
#[test]
fn boolean_accepts_only_json_booleans() {
    assert_eq!(boolean(&value("true"), "k"), Ok(true));
    assert_eq!(boolean(&value("false"), "k"), Ok(false));
    for source in ["\"true\"", "1", "0", "[]", "{}", "null"] {
        let message = boolean(&value(source), "hooks.concurrentCommits")
            .expect_err(source)
            .message;
        assert!(
            message.starts_with("Configuration key hooks.concurrentCommits must"),
            "{message}"
        );
        assert!(message.contains("true or false"), "{message}");
    }
}

/// Strings decode escapes;
///  other kinds and lone surrogates are rejected by key.
#[test]
fn text_accepts_only_valid_unicode_strings() {
    assert_eq!(text(&value(r#""ab\n""#), "k"), Ok(String::from("ab\n")));
    assert_eq!(text(&value("\"\""), "k"), Ok(String::new()));
    assert_eq!(
        text(&value("3"), "policies.x").expect_err("number").message,
        "Configuration key policies.x must be a string, found a number."
    );
    let surrogate = text(&value(r#""\udfff""#), "policies.x")
        .expect_err("lone surrogate")
        .message;
    assert!(
        surrogate.starts_with("Configuration key policies.x is not valid Unicode text"),
        "{surrogate}"
    );
}

/// String arrays keep order and name the failing element's index.
#[test]
fn strings_require_an_array_of_strings() {
    assert_eq!(
        strings(&value(r#"["b", "a", "b"]"#), "k"),
        Ok(vec![
            String::from("b"),
            String::from("a"),
            String::from("b")
        ])
    );
    assert_eq!(strings(&value("[]"), "k"), Ok(Vec::<String>::new()));
    assert_eq!(
        strings(&value("\"a\""), "p.exclude")
            .expect_err("string")
            .message,
        "Configuration key p.exclude must be an array of strings, found a string."
    );
    assert_eq!(
        strings(&value(r#"["a", 2]"#), "p.exclude")
            .expect_err("number element")
            .message,
        "Configuration key p.exclude[1] must be a string, found a number."
    );
    assert_eq!(
        strings(&value(r#"[null]"#), "p.exclude")
            .expect_err("null element")
            .message,
        "Configuration key p.exclude[0] must not be null; remove it to use the default or give it a string."
    );
}

/// Exact integers are accepted in every spelling,
///  including both bounds.
#[test]
fn safe_integer_accepts_exact_whole_numbers_within_bounds() {
    for (source, minimum, expected) in [
        ("0", 0, 0),
        ("-0", 0, 0),
        ("0e9", 0, 0),
        ("0.0", 0, 0),
        ("1", 1, 1),
        ("1000", 0, 1000),
        ("1e3", 0, 1000),
        ("2.0", 0, 2),
        ("1.5e1", 0, 15),
        ("9007199254740991", 0, MAX_SAFE_INTEGER),
        ("9.007199254740991e15", 1, MAX_SAFE_INTEGER),
        ("5", 5, 5),
    ] {
        assert_eq!(
            safe_integer(&value(source), "k", minimum),
            Ok(expected),
            "{source}"
        );
    }
}

/// Fractions,
///  negatives,
///  out-of-range and non-number values are rejected with the bound in the message.
#[test]
fn safe_integer_rejects_inexact_or_out_of_range_values() {
    for (source, minimum) in [
        ("-1", 0),
        ("1.5", 0),
        ("1e-1", 0),
        ("0.1", 0),
        ("9007199254740992", 0),
        ("1e16", 0),
        ("1e400", 0),
        ("18446744073709551616", 0),
        ("99999999999999999999999999", 0),
        ("0", 1),
        ("4", 5),
        ("\"3\"", 0),
        ("true", 0),
        ("[1]", 0),
        ("null", 0),
    ] {
        let message = safe_integer(&value(source), "landing.reserveAfterLostRaces", minimum)
            .expect_err(source)
            .message;
        assert!(
            message.starts_with("Configuration key landing.reserveAfterLostRaces must"),
            "{source}: {message}"
        );
        assert!(
            message.contains(format!("a whole number from {minimum} to 9007199254740991").as_str()),
            "{source}: {message}"
        );
    }
    assert_eq!(
        safe_integer(&value("1.5"), "k", 0)
            .expect_err("fraction")
            .message,
        "Configuration key k must be a whole number from 0 to 9007199254740991, found 1.5."
    );
}
