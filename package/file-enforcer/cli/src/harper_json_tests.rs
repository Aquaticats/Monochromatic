//! What:
//!  Tests for the Harper JSON helpers:
//! parsing,
//! linter switches,
//! the exclude-list union,
//! scoped derivation,
//! schema defaults and layout.
//! Why:
//!  The embedded JSON must keep unrelated keys in place and print as `JSON.stringify(value, null, 2)` does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('harper json', () => { it('unions excludes', () => {}); });
//! ```

/// Imports the helpers under test.
use crate::harper_json::{
    EXCLUDE_KEY, TITLE_CASE_DESCRIPTION, TITLE_CASE_PROPERTY, disable_linters, format_object, parse_object, scoped_config, union_excludes,
    with_schema_defaults,
};
/// Imports `Map`,
/// `Value` from `serde_json`.
use serde_json::{Map, Value};

/// Parse text that is known to be a JSON object.
fn object(text: &str) -> Map<String, Value> {
    return parse_object(text, "fixture", "f.xml").expect("an object");
}

/// The keys of an object in order.
fn keys(map: &Map<String, Value>) -> Vec<&str> {
    return map.keys().map(String::as_str).collect();
}

/// An object parses;
/// an array,
/// a scalar and broken text are refused with the file's path.
#[test]
fn only_a_json_object_is_accepted() {
    assert_eq!(keys(&object("{\"a\": 1, \"b\": 2}")), vec!["a", "b"]);
    for text in ["[]", "3", "null", "{\"a\": ", ""] {
        let error = parse_object(text, "configurationContent", "options/a.xml").expect_err("refused");
        assert_eq!(error.code, "harper-json-invalid", "{text}");
        assert_eq!(error.path.as_deref(), Some("options/a.xml"));
        assert!(error.message.contains("configurationContent"), "{error:?}");
    }
}

/// A new linter key is appended;
/// an existing one keeps its position and becomes false.
#[test]
fn disabling_a_linter_keeps_positions() {
    let mut config = object("{\"harper-ls.linters.B\": true, \"other\": 1}");
    disable_linters(&mut config, &["A".to_string(), "B".to_string()]);
    assert_eq!(keys(&config), vec!["harper-ls.linters.B", "other", "harper-ls.linters.A"]);
    assert_eq!(config["harper-ls.linters.B"], Value::Bool(false));
    assert_eq!(config["harper-ls.linters.A"], Value::Bool(false));
}

/// The union keeps the user's strings first,
/// drops non-strings and duplicates,
/// and appends the additions.
#[test]
fn the_exclude_union_keeps_user_strings_first() {
    let mut config = object("{\"harper-ls.excludePatterns\": [\"mine\", 3, \"**/AGENTS.md\", null, \"mine\"], \"z\": 0}");
    union_excludes(&mut config, &["**/AGENTS.md".to_string(), "**/CLAUDE.md".to_string()]);
    let members: Vec<&str> = config[EXCLUDE_KEY].as_array().expect("an array").iter().filter_map(Value::as_str).collect();
    assert_eq!(members, vec!["mine", "**/AGENTS.md", "**/CLAUDE.md"]);
    assert_eq!(keys(&config), vec![EXCLUDE_KEY, "z"], "the key keeps its position");
}

/// The union creates the key when absent,
/// and does nothing at all for an empty addition list.
#[test]
fn the_exclude_union_creates_the_key_and_ignores_empty_additions() {
    let mut config = object("{}");
    union_excludes(&mut config, &[]);
    assert!(config.is_empty(), "an empty list creates no key");
    union_excludes(&mut config, &["a".to_string()]);
    assert_eq!(config[EXCLUDE_KEY], Value::Array(vec![Value::String("a".to_string())]));
    let mut not_an_array = object("{\"harper-ls.excludePatterns\": \"text\"}");
    union_excludes(&mut not_an_array, &["a".to_string()]);
    assert_eq!(not_an_array[EXCLUDE_KEY], Value::Array(vec![Value::String("a".to_string())]), "a non-array value is replaced");
}

/// A scoped configuration drops the exclude list,
/// then switches its own linters off;
/// the base is not changed.
#[test]
fn a_scoped_config_drops_the_excludes_and_disables_its_linters() {
    let base = object("{\"harper-ls.linters.A\": false, \"harper-ls.excludePatterns\": [\"x\"], \"keep\": true}");
    let scoped = scoped_config(&base, &["M".to_string()]);
    assert_eq!(keys(&scoped), vec!["harper-ls.linters.A", "keep", "harper-ls.linters.M"]);
    assert!(base.contains_key(EXCLUDE_KEY), "the base config is untouched");
}

/// The title-case default is added only where absent;
/// other schema keys and an existing property survive.
#[test]
fn schema_defaults_fill_only_what_is_absent() {
    let filled = with_schema_defaults(object("{\"properties\": {\"x\": {}}, \"title\": \"t\"}"));
    assert_eq!(keys(&filled), vec!["properties", "title"], "`properties` keeps its position");
    let properties = filled["properties"].as_object().expect("properties");
    assert_eq!(keys(properties), vec!["x", TITLE_CASE_PROPERTY]);
    assert_eq!(properties[TITLE_CASE_PROPERTY]["type"], "boolean");
    assert_eq!(properties[TITLE_CASE_PROPERTY]["default"], Value::Bool(true));
    assert_eq!(properties[TITLE_CASE_PROPERTY]["description"], TITLE_CASE_DESCRIPTION);
    let kept = with_schema_defaults(object("{\"properties\": {\"harper-ls.linters.UseTitleCase\": {\"default\": false}}}"));
    assert_eq!(kept["properties"][TITLE_CASE_PROPERTY]["default"], Value::Bool(false), "an existing property is not replaced");
    let created = with_schema_defaults(object("{\"properties\": 3}"));
    assert!(created["properties"].as_object().expect("an object").contains_key(TITLE_CASE_PROPERTY));
}

/// Output uses two-space indentation,
/// no trailing newline,
/// and keeps non-ASCII text as is.
#[test]
fn output_matches_two_space_json_stringify() {
    let config = object("{\"a\": [\"é\", \"b\"], \"c\": {\"d\": false}, \"e\": []}");
    let expected = "{\n  \"a\": [\n    \"é\",\n    \"b\"\n  ],\n  \"c\": {\n    \"d\": false\n  },\n  \"e\": []\n}";
    assert_eq!(format_object(config), expected);
    assert_eq!(format_object(Map::new()), "{}");
}
