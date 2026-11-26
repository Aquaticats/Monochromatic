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

/// Imports `Comparable`,
/// `token_text`,
/// `value_matches` from `crate::json_compare`.
use crate::json_compare::{
    Comparable,
    token_text,
    value_matches,
};
/// Imports `parse_json_text` from `crate::json_ops`.
use crate::json_ops::parse_json_text;
/// Imports `Evaluated` from `crate::value_eval`.
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
fn add_member(
    source: &str,
    record: &[&str],
    key: &str,
    token: &str,
) -> String {
    let path: Vec<String> = record
        .iter()
        .map(|segment| return (*segment).to_string())
        .collect();
    let document = parse_json_text(source).expect("the fixture is valid JSON");
    let node = document.node(&path).expect("the fixture has the record");
    return document
        .with_member(node, key, token)
        .expect("a record takes a member");
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
    assert_eq!(
        add_member(source, &[], "homepage", "\"h\""),
        "{\n  \"a\": 1,\n  \"homepage\": \"h\"\n}\n"
    );
}

#[test]
fn an_added_member_of_an_empty_record_sits_between_its_own_braces() {
    assert_eq!(
        add_member("{\n  \"r\": {}\n}\n", &["r"], "url", "\"u\""),
        "{\n  \"r\": {\n    \"url\": \"u\"\n  }\n}\n"
    );
}

#[test]
fn a_value_is_replaced_and_every_other_byte_is_kept() {
    let source = "{ \"a\": \"old\",\n  \"b\": [1, 2] }";
    let document = parse_json_text(source).expect("the fixture is valid JSON");
    let path: Vec<String> = vec!["a".to_string()];
    let node = document.node(&path).expect("the fixture has the member");
    assert_eq!(
        document.replaced(node, "\"new\""),
        "{ \"a\": \"new\",\n  \"b\": [1, 2] }"
    );
}

#[test]
fn a_key_path_walks_nested_records_and_arrays() {
    let source = "{\"a\": [{\"b\": 3}], \"c\": {\"d\": true}}";
    let document = parse_json_text(source).expect("the fixture is valid JSON");
    let nested: Vec<String> = vec!["c".to_string(), "d".to_string()];
    assert_eq!(
        document
            .node(&nested)
            .map(|node| return node.comparable.clone()),
        Some(Comparable::Bool(true))
    );
    let element: Vec<String> = vec!["a".to_string(), "[0]".to_string(), "b".to_string()];
    assert_eq!(
        document
            .node(&element)
            .map(|node| return node.comparable.clone()),
        Some(Comparable::Integer(3))
    );
    let absent: Vec<String> = vec!["c".to_string(), "e".to_string()];
    assert_eq!(document.node(&absent), None);
    let repeated: Vec<String> = vec!["a".to_string(), "[0]".to_string(), "[0]".to_string()];
    assert_eq!(
        document.node(&repeated),
        None,
        "an array element has no named members"
    );
}

#[test]
fn a_decoded_value_compares_by_value_and_not_by_spelling() {
    let document = parse_json_text("{\"u\": \"gi\\u0074\"}").expect("the fixture is valid JSON");
    let path: Vec<String> = vec!["u".to_string()];
    let node = document.node(&path).expect("the fixture has the member");
    assert!(
        value_matches(&node.comparable, &text("git")),
        "an escaped spelling holds the same text"
    );
}

#[test]
fn a_value_a_config_cannot_express_never_matches() {
    let document =
        parse_json_text("{\"a\": null, \"b\": 1.5, \"c\": {\"d\": 1}, \"e\": \"\\ud800\"}")
            .expect("the fixture is valid JSON");
    for name in ["a", "b", "c", "e"] {
        let path: Vec<String> = vec![name.to_string()];
        let node = document.node(&path).expect("the fixture has the member");
        assert_eq!(node.comparable, Comparable::Other, "{name}");
        assert!(!value_matches(&node.comparable, &text("")), "{name}");
        assert!(
            !value_matches(&node.comparable, &Evaluated::Bool(false)),
            "{name}"
        );
    }
}

#[test]
fn a_list_matches_element_by_element() {
    let document = parse_json_text("{\"a\": [\"x\", true, 2], \"b\": [\"x\"]}")
        .expect("the fixture is valid JSON");
    let path: Vec<String> = vec!["a".to_string()];
    let node = document.node(&path).expect("the fixture has the member");
    let wanted = Evaluated::List(vec![
        text("x"),
        Evaluated::Bool(true),
        Evaluated::Integer(2),
    ]);
    assert!(value_matches(&node.comparable, &wanted));
    let shorter = Evaluated::List(vec![text("x")]);
    assert!(!value_matches(&node.comparable, &shorter));
    let other_element = Evaluated::List(vec![
        text("x"),
        Evaluated::Integer(2),
        Evaluated::Integer(2),
    ]);
    assert!(!value_matches(&node.comparable, &other_element));
    let shorter_path: Vec<String> = vec!["b".to_string()];
    let shorter_node = document
        .node(&shorter_path)
        .expect("the fixture has the member");
    assert!(!value_matches(
        &shorter_node.comparable,
        &Evaluated::List(vec![])
    ));
}

/// What:
///  Environment variable that turns one run of this test binary into the child that measures a parse,
///  so no other test's allocation lands in the measurement.
/// Why:
///  Peak resident size is a property of a whole process,
/// and the tests of this crate share one process.
const MEMORY_CHILD_VARIABLE: &str = "FILE_ENFORCER_JSON_MEMORY_CHILD";

/// What:
///  The ceiling, in KiB, that parsing the review's deep shape may add to one process's peak resident size.
/// Why:
///  The release pass measured 181404 kB of peak resident size for a 15 KB manifest of 511 nested arrays,
///  because every node held its own comparable copy of its whole subtree;
///  the bound keeps that regression out while leaving room for the parsed tree and its spans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEEP_PARSE_PEAK_KIB = 16384;
/// ```
const DEEP_PARSE_PEAK_KIB: u64 = 16384;

/// What:
///  The peak resident size of this process in KiB, as the kernel records it (`VmHWM`).
/// Why:
///  The OS value needs no allocator hook and is what the release pass reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function peakResidentKib(): number | undefined
/// ```
fn peak_resident_kib() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").expect("the kernel provides it");
    for line in status.lines() {
        // `Some`/`let ... else` is how this code asks "is there a match?" without exceptions.
        let Some(rest) = line.strip_prefix("VmHWM:") else {
            continue;
        };
        let number = rest.split_whitespace().next()?;
        // `.ok()` turns the parse result into "a number or nothing".
        return number.parse::<u64>().ok();
    }
    // `None` is the "nothing found" answer this function's return type uses.
    return None;
}

/// What:
///  The review's deep shape: 511 nested arrays holding 3000 integers, inside a manifest that also owns a
///  repository object.
/// Why:
///  It is the document whose parse took the release pass's server to 181404 kB.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deepDocument(): string
/// ```
fn deep_document() -> String {
    let open: String = "[".repeat(511);
    let close: String = "]".repeat(511);
    let integers: Vec<String> = (0..3000).map(|value| return value.to_string()).collect();
    let joined: String = integers.join(",");
    return format!(
        "{{\n  \"name\": \"o\",\n  \"repository\": {{\n    \"url\": \"https://old.example/o.git\"\n  }},\n  \"deep\": {open}{joined}{close}\n}}\n"
    );
}

/// What:
///  What the `json-keys` kind does with the deep document: build the text, parse it, and resolve each path an
///  edit asks about.
/// Why:
///  The measurement must cover the work the rule does, not a bare parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deepWorkload(): number
/// ```
fn deep_workload() -> usize {
    let source = deep_document();
    let document = parse_json_text(&source).expect("the fixture is valid JSON");
    let mut resolved: usize = 0;
    for segments in [
        vec!["repository", "url"],
        vec!["repository"],
        vec!["homepage"],
        vec!["deep"],
        vec!["name"],
    ] {
        let path: Vec<String> = segments
            .iter()
            .map(|segment| return (*segment).to_string())
            .collect();
        let Some(node) = document.node(&path) else {
            continue;
        };
        // `let _ =` discards the answer: this workload exists for the memory it takes, not for the verdict.
        let _matched = value_matches(&node.comparable, &text("https://old.example/o.git"));
        resolved += 1;
    }
    return resolved;
}

#[test]
fn a_deeply_nested_document_parses_within_its_memory_bound() {
    if std::env::var_os(MEMORY_CHILD_VARIABLE).is_some() {
        let before = peak_resident_kib().expect("the kernel reports VmHWM");
        let resolved = deep_workload();
        assert_eq!(resolved, 4, "every path but the absent homepage resolves");
        let after = peak_resident_kib().expect("the kernel reports VmHWM");
        println!("deep-parse-peak-before-kib={before}");
        println!("deep-parse-peak-after-kib={after}");
        return;
    }
    let binary = std::env::current_exe().expect("the test binary has a path");
    let output = std::process::Command::new(binary)
        .args([
            "--exact",
            "json_ops_tests::a_deeply_nested_document_parses_within_its_memory_bound",
            "--nocapture",
        ])
        .env(MEMORY_CHILD_VARIABLE, "1")
        .output()
        .expect("the child of this test binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "the child failed: {stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut before: Option<u64> = None;
    let mut after: Option<u64> = None;
    for line in stdout.lines() {
        if let Some(value) = line.strip_prefix("deep-parse-peak-before-kib=") {
            before = value.trim().parse::<u64>().ok();
        }
        if let Some(value) = line.strip_prefix("deep-parse-peak-after-kib=") {
            after = value.trim().parse::<u64>().ok();
        }
    }
    let growth = after
        .expect("the child printed its peak after the parse")
        .saturating_sub(before.expect("the child printed its peak before the parse"));
    assert!(
        growth <= DEEP_PARSE_PEAK_KIB,
        "the parse added {growth} KiB of peak resident size, over the {DEEP_PARSE_PEAK_KIB} KiB bound"
    );
}

#[test]
fn a_later_duplicate_key_hides_every_path_of_the_earlier_one() {
    let document =
        parse_json_text("{\"repository\": {\"url\": \"old\"}, \"repository\": \"github:old/e\"}")
            .expect("the fixture is valid JSON");
    let url: Vec<String> = vec!["repository".to_string(), "url".to_string()];
    assert_eq!(
        document
            .node(&url)
            .map(|node| return node.comparable.clone()),
        None,
        "the string written last holds no members"
    );
    let repository: Vec<String> = vec!["repository".to_string()];
    assert_eq!(
        document
            .node(&repository)
            .map(|node| return node.comparable.clone()),
        Some(Comparable::Text("github:old/e".to_string()))
    );
    let flipped =
        parse_json_text("{\"repository\": \"github:old/f\", \"repository\": {\"url\": \"f\"}}")
            .expect("the fixture is valid JSON");
    assert_eq!(
        flipped
            .node(&url)
            .map(|node| return node.comparable.clone()),
        Some(Comparable::Text("f".to_string())),
        "the object written last is the one that answers"
    );
}

#[test]
fn an_added_member_keeps_the_line_ending_the_file_uses() {
    assert_eq!(
        add_member("{\r\n  \"a\": 1\r\n}\r\n", &[], "homepage", "\"h\""),
        "{\r\n  \"a\": 1,\r\n  \"homepage\": \"h\"\r\n}\r\n"
    );
    assert_eq!(
        add_member("{\r\n  \"r\": {}\r\n}\r\n", &["r"], "url", "\"u\""),
        "{\r\n  \"r\": {\r\n    \"url\": \"u\"\r\n  }\r\n}\r\n"
    );
}

#[test]
fn an_added_member_of_an_empty_record_takes_the_files_indentation_unit() {
    let source = "{\n\t\"name\": \"d\",\n\t\"repository\": {}\n}\n";
    assert_eq!(
        add_member(source, &["repository"], "url", "\"u\""),
        "{\n\t\"name\": \"d\",\n\t\"repository\": {\n\t\t\"url\": \"u\"\n\t}\n}\n"
    );
}

#[test]
fn a_computed_value_becomes_the_token_that_stores_it() {
    assert_eq!(token_text(&text("a\"b\\c\n")), "\"a\\\"b\\\\c\\n\"");
    assert_eq!(token_text(&Evaluated::Bool(false)), "false");
    assert_eq!(token_text(&Evaluated::Integer(-7)), "-7");
    assert_eq!(
        token_text(&Evaluated::List(vec![
            text("x"),
            Evaluated::Integer(1),
            Evaluated::List(vec![Evaluated::Bool(true)])
        ])),
        "[\"x\", 1, [true]]"
    );
}
