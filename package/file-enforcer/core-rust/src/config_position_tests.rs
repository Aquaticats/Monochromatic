//! What:
//!  Tests that every config problem names `file:line:col` and the JSON pointer.
//! Why:
//!  A person fixing a config jumps to the line from the message,
//!  so a wrong column or a message with no position is a defect the golden decode tests would not catch.
//!  Expected positions are found by searching the config text,
//! not by calling the code under test.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('config positions', () => { it('names file:line:col and pointer', () => { ... }); });
//! ```

use crate::config_decode::{ConfigScope, decode_config};
use crate::config_load::parse_config_text;
use crate::config_locations::pointer_segment;

/// What:
///  The one-based line and column where `needle` first appears on one-based line `line` of `text`.
///  Columns count characters,
/// as the rule under test does.
/// Why:
///  The tests state their expectations from the text itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function where(text: string, line: number, needle: string): string // "line:col"
/// ```
fn where_is(text: &str, line: usize, needle: &str) -> String {
    let content = text.lines().nth(line - 1).unwrap_or_else(|| panic!("no line {line}"));
    let byte = content.find(needle).unwrap_or_else(|| panic!("{needle:?} is not on line {line}: {content:?}"));
    let column = content[..byte].chars().count() + 1;
    return format!("{line}:{column}");
}

/// What:
///  Decode a repository config and join every message into one string.
/// Why:
///  Most tests look for one message among several.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function messagesOf(text: string): string[]
/// ```
fn messages_of(text: &str) -> Vec<String> {
    let document = parse_config_text(text, "c.jsonc").expect("the config text parses");
    let errors = decode_config(&document, ConfigScope::Repository, "c.jsonc").expect_err("the config is invalid");
    return errors.into_iter().map(|error| return error.message).collect();
}

/// The config every positional test reads:
/// one problem per line,
/// so each line number names one expectation.
const INVALID: &str = r#"{
  "rules": [
    {"kind": "concat", "id": "a", "output": "/abs", "parts": [], "extra": 1},
    {"kind": "nope", "id": "b"},
    {"kind": "mirror", "id": "a", "from": "x", "to": "x/../y"},
    {"kind": "mirror", "id": "m", "from": "é", "to": 3},
    {"kind": "mode-only", "id": "n", "path": "p"}
  ]
}"#;

#[test]
fn every_decode_message_names_file_line_column_and_pointer() {
    let messages = messages_of(INVALID);
    assert!(messages.len() >= 5, "{messages:?}");
    for message in &messages {
        let mut parts = message.splitn(4, ':');
        assert_eq!(parts.next(), Some("c.jsonc"), "{message}");
        let line = parts.next().unwrap_or_default();
        let column = parts.next().unwrap_or_default();
        assert!(line.parse::<usize>().is_ok() && column.parse::<usize>().is_ok(), "no line:col in {message}");
        assert!(parts.next().unwrap_or_default().starts_with(" /"), "no pointer in {message}");
    }
}

#[test]
fn a_value_problem_points_at_the_value() {
    let messages = messages_of(INVALID).join("\n");
    let output = format!("c.jsonc:{}: /rules/0/output: ", where_is(INVALID, 3, "\"/abs\""));
    assert!(messages.contains(&output), "{messages}");
    let to = format!("c.jsonc:{}: /rules/2/to: ", where_is(INVALID, 5, "\"x/../y\""));
    assert!(messages.contains(&to), "{messages}");
}

#[test]
fn an_unknown_key_points_at_the_key() {
    let messages = messages_of(INVALID).join("\n");
    let extra = format!("c.jsonc:{}: /rules/0/extra: unknown key \"extra\"", where_is(INVALID, 3, "\"extra\""));
    assert!(messages.contains(&extra), "{messages}");
}

#[test]
fn an_unknown_kind_and_a_repeated_id_point_at_their_values() {
    let messages = messages_of(INVALID).join("\n");
    let kind = format!("c.jsonc:{}: /rules/1/kind: unknown kind \"nope\"", where_is(INVALID, 4, "\"nope\""));
    assert!(messages.contains(&kind), "{messages}");
    let id = format!("c.jsonc:{}: /rules/2/id: the id \"a\" is used by an earlier rule", where_is(INVALID, 5, "\"a\""));
    assert!(messages.contains(&id), "{messages}");
}

#[test]
fn a_missing_key_points_at_the_object_that_lacks_it() {
    let messages = messages_of(INVALID).join("\n");
    let rule = format!("c.jsonc:{}: /rules/4: is missing the required key \"mode\"", where_is(INVALID, 7, "{"));
    assert!(messages.contains(&rule), "{messages}");
}

#[test]
fn a_missing_rules_array_points_at_the_document_root() {
    let messages = messages_of("{}");
    assert_eq!(messages, vec!["c.jsonc:1:1: /: is missing the required key \"rules\"".to_string()]);
}

#[test]
fn columns_count_characters_not_bytes() {
    let messages = messages_of(INVALID).join("\n");
    let to = format!("c.jsonc:{}: /rules/3/to: must be a string", where_is(INVALID, 6, "3}"));
    assert!(messages.contains(&to), "{messages}");
    // `é` is two bytes and one character, so a byte column would read 55.
    assert_eq!(where_is(INVALID, 6, "3}"), "6:54");
}

#[test]
fn a_key_with_a_slash_is_escaped_in_the_pointer_and_still_located() {
    let text = "{\n  \"rules\": [{\"kind\": \"package-licenses\", \"id\": \"p\", \"manifests\": [],\n    \"directory\": \"L\", \"texts\": {\"a/b\": 1}, \"requires\": []}]\n}";
    let messages = messages_of(text).join("\n");
    let at = where_is(text, 3, "1}");
    assert!(messages.contains(&format!("c.jsonc:{at}: /rules/0/texts/a~1b: must be a string")), "{messages}");
    assert_eq!(pointer_segment("a/b~c"), "a~1b~0c");
}

#[test]
fn conversion_problems_carry_the_same_position_format() {
    let duplicate = "{\n  \"rules\": [],\n  \"rules\": []\n}";
    let error = parse_config_text(duplicate, "c.jsonc").expect_err("duplicate key");
    let expected = format!("c.jsonc:{}: /: the key \"rules\" appears twice in this object", where_is(duplicate, 3, "\"rules\""));
    assert_eq!(error.message, expected);
    let surrogate = "{\n  \"rules\": [],\n  \"x\": \"\\ud800\"\n}";
    let error = parse_config_text(surrogate, "c.jsonc").expect_err("lone surrogate");
    let expected = format!("c.jsonc:{}: /x: the string holds an unpaired surrogate", where_is(surrogate, 3, "\"\\ud800\""));
    assert_eq!(error.message, expected);
    let key_surrogate = "{\n  \"\\ud800\": 1\n}";
    let error = parse_config_text(key_surrogate, "c.jsonc").expect_err("lone surrogate key");
    assert_eq!(error.message, format!("c.jsonc:{}: /: a key holds an unpaired surrogate", where_is(key_surrogate, 2, "\"\\ud800\"")));
    let huge = "{\n  \"n\": 1e999\n}";
    let error = parse_config_text(huge, "c.jsonc").expect_err("unrepresentable number");
    assert_eq!(error.message, format!("c.jsonc:{}: /n: the number 1e999 is not representable", where_is(huge, 2, "1e999")));
    assert_eq!(error.code, "config-syntax");
    assert_eq!(error.path.as_deref(), Some("c.jsonc"));
}

#[test]
fn the_location_table_falls_back_to_the_nearest_existing_ancestor() {
    let document = parse_config_text("{\n  \"rules\": [\n    {\"kind\": \"mirror\"}\n  ]\n}", "c.jsonc").expect("parses");
    let table = &document.locations;
    assert_eq!((table.value_at("/rules/0/kind").line, table.value_at("/rules/0/kind").column), (3, 14));
    assert_eq!((table.value_at("/rules/0/id").line, table.value_at("/rules/0/id").column), (3, 5));
    assert_eq!((table.value_at("/rules/7/id").line, table.value_at("/rules/7/id").column), (2, 12));
    assert_eq!((table.key_at("/rules/0/kind").line, table.key_at("/rules/0/kind").column), (3, 6));
    assert_eq!((table.key_at("/rules/0").line, table.key_at("/rules/0").column), (3, 5));
    assert_eq!((table.value_at("").line, table.value_at("").column), (1, 1));
}
