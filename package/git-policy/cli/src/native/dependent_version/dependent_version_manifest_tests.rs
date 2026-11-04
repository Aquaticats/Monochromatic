//! What: Strict decoding, strict JSON and fact extraction, with every case of
//!       `manifest-text.unit.test.ts` for `readManifestDependencyFacts`.
//! Why: The planner must accept exactly what `JSON.parse` accepts and read the same facts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(readManifestDependencyFacts({ path, text })).toEqual({ name, version, ... });
//! ```

/// The reader, the decoder and the failure type.
use super::{ManifestFacts, read_manifest_facts, shape, strict_text};
use crate::dependent_version_content::PolicyIncomplete;

/// The path every case reads.
const PATH: &[u8] = b"package/module/a/package.json";

/// Facts of a manifest that must parse.
fn facts(text: &str) -> ManifestFacts {
    return read_manifest_facts(PATH, text).unwrap_or_else(|error: PolicyIncomplete| {
        panic!("{text} should parse: {error:?}");
    });
}

/// The failure for a manifest that must not parse.
fn failure(text: &str) -> PolicyIncomplete {
    return read_manifest_facts(PATH, text)
        .err()
        .unwrap_or_else(|| panic!("{text} should fail"));
}

/// The syntax failure with its detail.
fn syntax(detail: &str) -> PolicyIncomplete {
    return PolicyIncomplete::ManifestSyntax {
        path: PATH.to_vec(),
        detail: String::from(detail),
    };
}

/// Code units of text.
fn units(text: &str) -> Vec<u16> {
    return text.encode_utf16().collect();
}

/// Names as owned strings.
fn names(values: &[&str]) -> Vec<String> {
    return values
        .iter()
        .map(|value: &&str| return String::from(*value))
        .collect();
}

/// Strict UTF-8 with one leading byte-order mark removed.
#[test]
fn decodes_strict_utf8_without_one_byte_order_mark() {
    assert_eq!(strict_text(PATH, b"{}"), Ok("{}"));
    assert_eq!(strict_text(PATH, b"\xef\xbb\xbf{}"), Ok("{}"));
    assert_eq!(
        strict_text(PATH, b"\xef\xbb\xbf\xef\xbb\xbf{}"),
        Ok("\u{feff}{}")
    );
    assert_eq!(
        strict_text(PATH, b"{\"name\":\"\xff\"}"),
        Err(PolicyIncomplete::NotUtf8 {
            path: PATH.to_vec()
        })
    );
}

/// Ported: name, version, runtime fields in order, and development names.
#[test]
fn reads_name_version_runtime_fields_in_order_and_development_names() {
    assert_eq!(
        facts(
            r#"{"name":"@scope/a","version":"1.0.0","dependencies":{"@scope/b":"workspace:*"},"peerDependencies":{"@scope/c":"*"},"optionalDependencies":{"@scope/d":"*"},"devDependencies":{"@scope/e":"workspace:*"}}"#
        ),
        ManifestFacts {
            name: String::from("@scope/a"),
            version: Some(units("1.0.0")),
            runtime_dependency_names: names(&["@scope/b", "@scope/c", "@scope/d"]),
            dev_dependency_names: names(&["@scope/e"]),
        }
    );
}

/// Ported: an absent version is omitted and non-object dependency fields are empty.
#[test]
fn omits_an_absent_version_and_reads_non_object_fields_as_empty() {
    assert_eq!(
        facts(r#"{"name":"a","dependencies":["b"]}"#),
        ManifestFacts {
            name: String::from("a"),
            version: None,
            runtime_dependency_names: Vec::new(),
            dev_dependency_names: Vec::new(),
        }
    );
}

/// Ported: a manifest that is not an object, has no string name, or a non-string version.
#[test]
fn rejects_the_incumbent_shapes() {
    assert_eq!(failure("[]"), shape(PATH, "is not a JSON object"));
    assert_eq!(
        failure(r#"{"version":"1.0.0"}"#),
        shape(PATH, "has no string \"name\"")
    );
    assert_eq!(
        failure(r#"{"name":"a","version":1}"#),
        shape(PATH, "has a non-string \"version\"")
    );
    assert_eq!(
        failure(r#"{"name":"a","version":null}"#),
        shape(PATH, "has a non-string \"version\"")
    );
    assert_eq!(
        failure(r#"{"name":1}"#),
        shape(PATH, "has no string \"name\"")
    );
    assert_eq!(
        shape(PATH, "is not a JSON object"),
        PolicyIncomplete::ManifestShape {
            path: PATH.to_vec(),
            problem: String::from("package/module/a/package.json is not a JSON object"),
        }
    );
}

/// A scalar root is JSON, so it fails as a shape, as under `JSON.parse`.
#[test]
fn reads_a_scalar_root_as_a_shape_problem() {
    assert_eq!(failure("\"x\""), shape(PATH, "is not a JSON object"));
    assert_eq!(failure(" 1 "), shape(PATH, "is not a JSON object"));
    assert_eq!(failure("null"), shape(PATH, "is not a JSON object"));
}

/// Comments and trailing commas, which JSONC allows and JSON does not, are refused.
#[test]
fn refuses_comments_and_trailing_commas() {
    assert_eq!(
        failure("{\"name\":\"a\"} // x"),
        syntax("JSON allows no comments")
    );
    assert_eq!(
        failure("/* x */ {\"name\":\"a\"}"),
        syntax("JSON allows no comments")
    );
    assert_eq!(
        failure("{\"name\":\"a\",}"),
        syntax("JSON allows no trailing commas")
    );
    assert_eq!(
        failure("{\"name\":\"a\", \r\n\t}"),
        syntax("JSON allows no trailing commas")
    );
    assert_eq!(
        failure("{\"name\":\"a\",\"d\":[1,]}"),
        syntax("JSON allows no trailing commas")
    );
    assert_eq!(
        failure("{\"name\":\"a\"},"),
        syntax("JSON allows no trailing commas")
    );
}

/// Slashes and commas inside strings, also after escaped quotes, are text, not syntax.
#[test]
fn reads_slashes_and_commas_inside_strings_as_text() {
    assert_eq!(facts(r#"{"name":"a/b,}"}"#).name, "a/b,}");
    assert_eq!(facts(r#"{"name":"a\"/,]"}"#).name, "a\"/,]");
    assert_eq!(facts(r#"{"name":"a\\","x":"/"}"#).name, "a\\");
    assert_eq!(facts(r#"{"x":[1, 2],"name":"a"}"#).name, "a");
}

/// Text that is not exactly one JSON value fails as syntax.
#[test]
fn refuses_text_that_is_not_one_value() {
    assert_eq!(
        failure(""),
        syntax("the text is not exactly one JSON value")
    );
    assert_eq!(
        failure("1, 2"),
        syntax("the text is not exactly one JSON value")
    );
    // `matches!` checks the variant; the parser's own detail is not pinned here.
    assert!(matches!(
        failure("{\"name\":}"),
        PolicyIncomplete::ManifestSyntax { .. }
    ));
    assert!(matches!(
        failure("{} {}"),
        PolicyIncomplete::ManifestSyntax { .. }
    ));
    assert!(matches!(
        failure("\u{feff}{}"),
        PolicyIncomplete::ManifestSyntax { .. }
    ));
}

/// The last duplicate of a key wins, as under `JSON.parse`.
#[test]
fn takes_the_last_duplicate_key() {
    let read: ManifestFacts = facts(
        r#"{"name":"first","version":"1.0.0","dependencies":{"x":"1"},"name":"second","version":"2.0.0","dependencies":{"y":"1"}}"#,
    );
    assert_eq!(read.name, "second");
    assert_eq!(read.version, Some(units("2.0.0")));
    assert_eq!(read.runtime_dependency_names, names(&["y"]));
    assert_eq!(
        failure(r#"{"name":"a","name":1}"#),
        shape(PATH, "has no string \"name\"")
    );
}

/// Dependency names are unique within a field and kept across fields.
#[test]
fn keeps_dependency_names_unique_within_a_field() {
    let read: ManifestFacts = facts(
        r#"{"name":"a","dependencies":{"b":"1","c":"1","b":"2"},"peerDependencies":{"b":"1"},"devDependencies":{"d":"1","d":"1"}}"#,
    );
    assert_eq!(read.runtime_dependency_names, names(&["b", "c", "b"]));
    assert_eq!(read.dev_dependency_names, names(&["d"]));
}

/// Escaped unpaired surrogates: refused in the name, dropped in dependency keys, kept in
/// the version.
#[test]
fn handles_unpaired_surrogates_by_field() {
    assert_eq!(
        failure(r#"{"name":"a\ud800"}"#),
        shape(PATH, "has a \"name\" holding an unpaired UTF-16 surrogate")
    );
    let read: ManifestFacts =
        facts(r#"{"name":"a😀","version":"1.0.\udc00","dependencies":{"\ud800":"1","b":"1"}}"#);
    assert_eq!(read.name, "a\u{1f600}");
    assert_eq!(read.version, Some(vec![0x31, 0x2e, 0x30, 0x2e, 0xdc00]));
    assert_eq!(read.runtime_dependency_names, names(&["b"]));
}
