//! Ripgrep decoding preserves native names and fails visibly on invalid protocol or escaped project paths.

/// Standard encoding constructs byte-valued wire fixtures;
///  assertions compare the original bytes.
use base64::{Engine, engine::general_purpose::STANDARD};
/// Production result types and decoders are exercised without a native window.
/// `json_value!` builds the wire fixtures like `json!`,
///  with the one lint allowance the `unwrap` ban needs.
use ide_app::{
    json_value,
    search::{MAX_PREVIEW_GRAPHEMES, SearchKind},
    search_protocol::{content, filename},
    search_query::PathQuery,
};
/// Typed JSON fixture construction escapes quotes,
///  separators,
///  and newlines at the protocol boundary.
use serde_json::{Value, json};
/// Native filename bytes remain distinct from their lossy display representation.
use std::{os::unix::ffi::OsStrExt, path::Path};

/// Serialize a real match envelope rather than concatenating unescaped JSON strings.
fn record(path: Value, lines: Value, line: usize) -> Vec<u8> {
    return serde_json::to_vec(&json_value!({"type":"match", "data":{"path":path,"lines":lines,"line_number":line,"submatches":[]}}))
        .expect("serialize match fixture");
}

/// Text payloads preserve adversarial filenames and source whitespace before the matching text.
#[test]
fn text_records_keep_native_identity_and_source_line() {
    let root = Path::new("/project");
    let name = "/project/猫 '\";\\\n$.ts";
    let wire = record(
        json_value!({"text":name}),
        json_value!({"text":"  const 猫 = 1;  \r\n"}),
        7,
    );
    let hit = content(root, &wire)
        .expect("valid JSON match")
        .expect("content result");
    assert_eq!(hit.path, Path::new(name));
    assert_eq!(
        hit.kind,
        SearchKind::Content {
            line: 7,
            preview: "  const 猫 = 1;".to_string(),
            truncated: false
        }
    );
}

/// Byte payloads retain invalid-UTF-8 filenames and restrict replacement characters to the preview.
#[test]
fn byte_records_do_not_round_trip_paths_through_utf8() {
    let root = Path::new("/project");
    let path = b"/project/native-\xff.rs";
    let wire = record(
        json_value!({"bytes":STANDARD.encode(path)}),
        json_value!({"bytes":STANDARD.encode(b"a\xffb\n")}),
        1,
    );
    let hit = content(root, &wire)
        .expect("byte-valued JSON")
        .expect("content result");
    assert_eq!(hit.path.as_os_str().as_bytes(), path);
    assert_eq!(
        hit.kind,
        SearchKind::Content {
            line: 1,
            preview: "a\u{fffd}b".to_string(),
            truncated: false
        }
    );
    assert_eq!(
        filename(root, path)
            .expect("native filename record")
            .as_os_str()
            .as_bytes(),
        path
    );
    assert_eq!(
        filename(root, b"/project/line\nbreak.txt").expect("newline filename"),
        Path::new("/project/line\nbreak.txt")
    );
}

/// Preview clipping stops at a grapheme boundary and reports truncation independently of line navigation.
#[test]
fn preview_limit_preserves_combining_and_joined_graphemes() {
    for grapheme in ["e\u{301}", "👨‍👩‍👧‍👦"] {
        let text = grapheme.repeat(MAX_PREVIEW_GRAPHEMES + 1);
        let wire = record(
            json_value!({"text":"/project/source.txt"}),
            json_value!({"text":text}),
            2,
        );
        let hit = content(Path::new("/project"), &wire)
            .expect("long matching line")
            .expect("content result");
        assert_eq!(
            hit.kind,
            SearchKind::Content {
                line: 2,
                preview: grapheme.repeat(MAX_PREVIEW_GRAPHEMES),
                truncated: true
            }
        );
    }
    let exact = "x".repeat(MAX_PREVIEW_GRAPHEMES);
    let wire = record(
        json_value!({"text":"/project/source.txt"}),
        json_value!({"text":exact}),
        1,
    );
    let hit = content(Path::new("/project"), &wire)
        .expect("exact boundary")
        .expect("result");
    assert_eq!(
        hit.kind,
        SearchKind::Content {
            line: 1,
            preview: "x".repeat(MAX_PREVIEW_GRAPHEMES),
            truncated: false
        }
    );
}

/// Known protocol metadata is ignored;
///  unknown events are not silently treated as no matches.
#[test]
fn metadata_and_unknown_event_types_remain_distinct() {
    for kind in ["begin", "end", "context", "summary"] {
        let wire = serde_json::to_vec(&json_value!({"type":kind,"data":{"unused":[1,2,3]}}))
            .expect("metadata fixture");
        assert!(
            content(Path::new("/project"), &wire)
                .expect("known metadata")
                .is_none()
        );
    }
    assert!(
        content(
            Path::new("/project"),
            br#"{"type":"future-event","data":null}"#
        )
        .is_err()
    );
    assert!(content(Path::new("/project"), b"not JSON").is_err());
}

/// Ambiguous payloads,
///  invalid base64,
///  duplicate fields,
///  and missing line metadata fail visibly.
#[test]
fn malformed_matches_are_errors_not_empty_results() {
    let root = Path::new("/project");
    for path in [
        json_value!({}),
        json_value!({"text":"/project/source","bytes":"AA=="}),
        json_value!({"bytes":"***"}),
    ] {
        assert!(content(root, &record(path, json_value!({"text":"source"}), 1)).is_err());
    }
    assert!(
        content(
            root,
            &record(
                json_value!({"text":"/project/source"}),
                json_value!({"text":"source"}),
                0
            )
        )
        .is_err()
    );
    assert!(
        content(
            root,
            &record(json_value!({"text":"/project/source"}), json!({}), 1)
        )
        .is_err()
    );
    assert!(content(root, br#"{"type":"match","data":{"path":{"text":"/project/a","text":"/project/b"},"lines":{"text":"x"},"line_number":1}}"#).is_err());
    assert!(
        content(
            root,
            br#"{"type":"match","data":{"path":{"text":"/project/a"},"lines":{"text":"x"}}}"#
        )
        .is_err()
    );
}

/// Root siblings,
///  traversal components,
///  missing names,
///  and embedded NUL bytes are rejected before UI activation.
#[test]
fn decoded_paths_must_be_project_children() {
    let root = Path::new("/project");
    for path in [
        "",
        "/project",
        "/project-other/file",
        "/project/../outside",
        "relative.txt",
        "/project/a\0b",
    ] {
        assert!(
            filename(root, path.as_bytes()).is_err(),
            "accepted invalid filename {path:?}"
        );
        assert!(
            content(
                root,
                &record(
                    json_value!({"text":path}),
                    json_value!({"text":"source"}),
                    1
                )
            )
            .is_err()
        );
    }
}

/// Filename patterns are smart-case substrings,
///  including Unicode case and literal regex punctuation.
#[test]
fn filename_queries_match_editord_smart_case_semantics() {
    assert!(PathQuery::new("source").matches(Path::new("SRC/Source.ts")));
    assert!(PathQuery::new("Source").matches(Path::new("SRC/Source.ts")));
    assert!(!PathQuery::new("Source").matches(Path::new("src/source.ts")));
    assert!(PathQuery::new("ä").matches(Path::new("Ä/source.txt")));
    assert!(!PathQuery::new("Ä").matches(Path::new("ä/source.txt")));
    assert!(PathQuery::new("[x]").matches(Path::new("literal[x].txt")));
    assert!(!PathQuery::new("[x]").matches(Path::new("x.txt")));
    assert!(PathQuery::new("").matches(Path::new("any.txt")));
}
