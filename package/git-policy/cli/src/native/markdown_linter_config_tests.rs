//! What:
//!  Controls for the one-rule linter configuration and its temporary file.
//! Why:
//!  The document must select exactly the policy's rules at `warn` and carry every
//!      exclude pattern as data,
//!  whatever characters the pattern holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(JSON.parse(oneRuleConfiguration(rules, ['"]}'])).at(0).rules['markdown/lfs-image-url'].exclude).toEqual(['"]}']);
//! ```

/// The document builder and file writer.
use super::{MARKDOWN_FILE_PATTERNS, one_rule_configuration, write_configuration};
use crate::config_schema::MarkdownRule;
use crate::test_support::{fixture, remove};
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc, units_to_string};
use std::path::{Path, PathBuf};

/// The text of a JSON string value.
fn text_of(value: &JsoncValue) -> String {
    return units_to_string(value.text_units().expect("a string")).expect("valid text");
}

/// The member `key` of a record value.
fn member<'a>(value: &'a JsoncValue, key: &str) -> &'a JsoncValue {
    for entry in value.entries().expect("a record") {
        if text_of(&JsoncValue::text_from_units(entry.key.units.clone())) == key {
            return &entry.value;
        }
    }
    panic!("no member {key}");
}

/// The strings of an array value.
fn texts(value: &JsoncValue) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for element in value.elements().expect("an array") {
        found.push(text_of(element));
    }
    return found;
}

/// The document selects Markdown and MDX files and the LFS rule at `warn` with the patterns.
#[test]
fn the_document_selects_the_rule_at_warn() {
    let exclude: Vec<String> = vec![String::from("package/ssg/"), String::from("!keep.md")];
    let document: String = one_rule_configuration(&[MarkdownRule::LfsImageUrl], exclude.as_slice());
    let parsed: JsoncValue = parse_jsonc(document.as_str()).expect("valid JSON");
    let blocks: &[JsoncValue] = parsed.elements().expect("a list of blocks");
    assert_eq!(blocks.len(), 1);
    assert_eq!(texts(member(&blocks[0], "files")), MARKDOWN_FILE_PATTERNS);
    let rules: &JsoncValue = member(&blocks[0], "rules");
    assert_eq!(rules.entries().expect("rules").len(), 1);
    let lfs: &JsoncValue = member(rules, "markdown/lfs-image-url");
    assert_eq!(text_of(member(lfs, "severity")), "warn");
    assert_eq!(texts(member(lfs, "exclude")), exclude);
    assert_eq!(
        document,
        one_rule_configuration(&[MarkdownRule::LfsImageUrl], exclude.as_slice()),
        "the document is deterministic"
    );
}

/// No rule selects nothing;
///  no pattern is an empty list.
#[test]
fn empty_lists_stay_empty() {
    let none: JsoncValue = parse_jsonc(one_rule_configuration(&[], &[]).as_str()).expect("JSON");
    assert_eq!(
        member(&none.elements().expect("blocks")[0], "rules")
            .entries()
            .expect("rules")
            .len(),
        0
    );
    let bare: JsoncValue =
        parse_jsonc(one_rule_configuration(&[MarkdownRule::LfsImageUrl], &[]).as_str())
            .expect("JSON");
    let lfs: &JsoncValue = member(
        member(&bare.elements().expect("blocks")[0], "rules"),
        "markdown/lfs-image-url",
    );
    assert_eq!(texts(member(lfs, "exclude")), Vec::<String>::new());
}

/// Patterns holding JSON and JSONC syntax,
///  escapes,
///  line breaks,
///  control and astral
/// characters come back exactly as data.
#[test]
fn hostile_patterns_stay_data() {
    let exclude: Vec<String> = [
        "\"]}],{\"rules\":{}}",
        "back\\slash\\",
        "line\nbreak\r\n",
        "*/ /* // comment",
        "\u{0}\u{1f}\u{7f}",
        "😀/\u{2028}",
        "",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let document: String = one_rule_configuration(&[MarkdownRule::LfsImageUrl], exclude.as_slice());
    let parsed: JsoncValue = parse_jsonc(document.as_str()).expect("still JSON");
    let blocks: &[JsoncValue] = parsed.elements().expect("blocks");
    assert_eq!(blocks.len(), 1);
    let lfs: &JsoncValue = member(member(&blocks[0], "rules"), "markdown/lfs-image-url");
    assert_eq!(texts(member(lfs, "exclude")), exclude);
}

/// The file holds the document,
///  sits in the given directory with the policy's name,
///  is
/// readable by its owner only,
///  and is gone once dropped.
#[test]
fn the_file_is_private_and_removed_when_dropped() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("linter-config");
    let file = write_configuration("[]", root.as_path()).expect("written");
    let path: PathBuf = file.path().to_path_buf();
    assert_eq!(path.parent(), Some(root.as_path()));
    let name: String = path
        .file_name()
        .expect("name")
        .to_string_lossy()
        .into_owned();
    assert!(
        name.starts_with("cli-git-markdown-") && name.ends_with(".jsonc"),
        "{name}"
    );
    assert_eq!(std::fs::read_to_string(&path).expect("readable"), "[]");
    let mode: u32 = std::fs::metadata(&path)
        .expect("metadata")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600, "{mode:o}");
    drop(file);
    assert!(!path.exists());
    remove(root.as_path());
}

/// A directory that does not exist is reported,
///  naming it.
#[test]
fn a_missing_directory_is_reported() {
    let missing: &Path = Path::new("/nonexistent-cli-git-scratch");
    match write_configuration("[]", missing) {
        Err(reason) => assert!(
            reason.starts_with(
                "cli-git could not create the temporary linter configuration in /nonexistent-cli-git-scratch: "
            ),
            "{reason}"
        ),
        Ok(file) => panic!("created {:?}", file.path()),
    }
}
