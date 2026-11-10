//! What:
//!  Tests for the JSON text operations the `json-keys` kind is built from.
//! Why:
//!  Splicing keeps every byte it does not own,
//! which is what lets a strict-JSON manifest keep its own layout,
//! and a comparison against a config value must use the decoded text.
//!  The kind's tests drive the ordinary shapes;
//! these name the shapes a package manifest does not have.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('json text', () => { it('replaces only the value', () => { ... }); });
//! ```

use crate::json_ops::{Comparable, parse_json_text, token_text, value_matches};
use crate::value_eval::Evaluated;

/// What:
///  The text of a document after one member was added at a path.
/// Why:
///  Every assertion about an added member goes through the same lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addMember(source: string, record: string[], key: string, token: string): string
/// ```
fn add_member(source: &str, record: &[&str], key: &str, token: &str) -> String {
    let path: Vec<String> = record.iter().map(|segment| return (*segment).to_string()).collect();
    let document = parse_json_text(source).expect("the fixture is valid JSON");
    let node = document.node(&path).expect("the fixture has the record");
    return document.with_member(node, key, token).expect("a record takes a member");
}

/// What:
///  A config value for each `Evaluated` variant.
/// Why:
///  The comparison and the token text are only exercised by every variant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = (value: string) => ({ kind: 'text', value });
/// ```
fn text(value: &str) -> Evaluated {
    return Evaluated::Text(value.to_string());
}

#[test]
fn an_added_member_takes_the_indentation_its_siblings_use() {
    let source = "{\n  \"a\": 1,\n  \"b\": {\n      \"c\": 2\n  }\n}\n";
    assert_eq!(
        add_member(source, &[], "homepage", "\"https://example\""),
        "{\n  \"a\": 1,\n  \"b\": {\n      \"c\": 2\n  },\n  \"homepage\": \"https://example\"\n}\n"
    );
    assert_eq!(
        add_member(source, &["b"], "directory", "\"d\""),
        "{\n  \"a\": 1,\n  \"b\": {\n      \"c\": 2,\n      \"directory\": \"d\"\n  }\n}\n"
    );
}

#[test]
fn an_added_member_takes_the_comma_the_last_member_already_has() {
    let source = "{\n  \"a\": 1,\n}\n";
    assert_eq!(add_member(source, &[], "homepage", "\"h\""), "{\n  \"a\": 1,\n  \"homepage\": \"h\"\n}\n");
}

#[test]
fn an_added_member_of_an_empty_record_sits_between_its_own_braces() {
    assert_eq!(add_member("{\n  \"r\": {}\n}\n", &["r"], "url", "\"u\""), "{\n  \"r\": {\n    \"url\": \"u\"\n  }\n}\n");
}

#[test]
fn a_value_is_replaced_and_every_other_byte_is_kept() {
    let source = "{ \"a\": \"old\",\n  \"b\": [1, 2] }";
    let document = parse_json_text(source).expect("the fixture is valid JSON");
    let path: Vec<String> = vec!["a".to_string()];
    let node = document.node(&path).expect("the fixture has the member");
    assert_eq!(document.replaced(node, "\"new\""), "{ \"a\": \"new\",\n  \"b\": [1, 2] }");
}

#[test]
fn a_key_path_walks_nested_records_and_arrays() {
    let source = "{\"a\": [{\"b\": 3}], \"c\": {\"d\": true}}";
    let document = parse_json_text(source).expect("the fixture is valid JSON");
    let nested: Vec<String> = vec!["c".to_string(), "d".to_string()];
    assert_eq!(document.node(&nested).map(|node| return node.comparable.clone()), Some(Comparable::Bool(true)));
    let element: Vec<String> = vec!["a".to_string(), "[0]".to_string(), "b".to_string()];
    assert_eq!(document.node(&element).map(|node| return node.comparable.clone()), Some(Comparable::Integer(3)));
    let absent: Vec<String> = vec!["c".to_string(), "e".to_string()];
    assert_eq!(document.node(&absent), None);
    let repeated: Vec<String> = vec!["a".to_string(), "[0]".to_string(), "[0]".to_string()];
    assert_eq!(document.node(&repeated), None, "an array element has no named members");
}

#[test]
fn a_decoded_value_compares_by_value_and_not_by_spelling() {
    let document = parse_json_text("{\"u\": \"gi\\u0074\"}").expect("the fixture is valid JSON");
    let path: Vec<String> = vec!["u".to_string()];
    let node = document.node(&path).expect("the fixture has the member");
    assert!(value_matches(&node.comparable, &text("git")), "an escaped spelling holds the same text");
}

#[test]
fn a_value_a_config_cannot_express_never_matches() {
    let document = parse_json_text("{\"a\": null, \"b\": 1.5, \"c\": {\"d\": 1}, \"e\": \"\\ud800\"}")
        .expect("the fixture is valid JSON");
    for name in ["a", "b", "c", "e"] {
        let path: Vec<String> = vec![name.to_string()];
        let node = document.node(&path).expect("the fixture has the member");
        assert_eq!(node.comparable, Comparable::Other, "{name}");
        assert!(!value_matches(&node.comparable, &text("")), "{name}");
        assert!(!value_matches(&node.comparable, &Evaluated::Bool(false)), "{name}");
    }
}

#[test]
fn a_list_matches_element_by_element() {
    let document = parse_json_text("{\"a\": [\"x\", true, 2], \"b\": [\"x\"]}").expect("the fixture is valid JSON");
    let path: Vec<String> = vec!["a".to_string()];
    let node = document.node(&path).expect("the fixture has the member");
    let wanted = Evaluated::List(vec![text("x"), Evaluated::Bool(true), Evaluated::Integer(2)]);
    assert!(value_matches(&node.comparable, &wanted));
    let shorter = Evaluated::List(vec![text("x")]);
    assert!(!value_matches(&node.comparable, &shorter));
    let other_element = Evaluated::List(vec![text("x"), Evaluated::Integer(2), Evaluated::Integer(2)]);
    assert!(!value_matches(&node.comparable, &other_element));
    let shorter_path: Vec<String> = vec!["b".to_string()];
    let shorter_node = document.node(&shorter_path).expect("the fixture has the member");
    assert!(!value_matches(&shorter_node.comparable, &Evaluated::List(vec![])));
}

#[test]
fn a_computed_value_becomes_the_token_that_stores_it() {
    assert_eq!(token_text(&text("a\"b\\c\n")), "\"a\\\"b\\\\c\\n\"");
    assert_eq!(token_text(&Evaluated::Bool(false)), "false");
    assert_eq!(token_text(&Evaluated::Integer(-7)), "-7");
    assert_eq!(
        token_text(&Evaluated::List(vec![text("x"), Evaluated::Integer(1), Evaluated::List(vec![Evaluated::Bool(true)])])),
        "[\"x\", 1, [true]]"
    );
}
