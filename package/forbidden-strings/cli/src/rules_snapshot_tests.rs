// Read, join, attribution, and fail-closed controls for the files of one rules list.

use super::RulesSnapshot;
use crate::rules_list::RulesEntry;
use std::path::{Path, PathBuf};

// Creates a fresh scratch directory keyed by label and pid.
fn temp_dir(label: &str) -> PathBuf {
    let dir = crate::test_scratch::scratch_root().join(format!("rules-snapshot-{}-{}", label, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    return dir
}

// Writes a file and returns a required entry for it.
fn required(dir: &Path, name: &str, body: &[u8]) -> RulesEntry {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write rules");
    return RulesEntry { path, optional: false }
}

// Returns an optional entry for a file that is not created.
fn absent(dir: &Path, name: &str) -> RulesEntry {
    return RulesEntry { path: dir.join(name), optional: true }
}

// Reads a list that must succeed.
fn read_ok(entries: &[RulesEntry]) -> RulesSnapshot {
    return RulesSnapshot::read(entries).expect("read list")
}

// Reads a list that must fail and returns the error text.
fn read_err(entries: &[RulesEntry]) -> String {
    return RulesSnapshot::read(entries).err().expect("list must fail").to_string()
}

// Compiles a snapshot that must fail and returns the error text.
fn compile_err(snapshot: &RulesSnapshot) -> String {
    return snapshot.compile().err().expect("compile must fail").to_string()
}

#[test]
fn one_file_joins_to_its_exact_text_including_a_bom_and_a_missing_final_newline() {
    let dir = temp_dir("one");
    let body = "\u{feff}FIRST_RULE_LONG_ENOUGH";
    let snapshot = read_ok(&[required(&dir, "a.txt", body.as_bytes())]);
    assert_eq!(snapshot.text(), body);
}

#[test]
fn two_files_join_in_list_order_with_a_newline_boundary() {
    let dir = temp_dir("two");
    let snapshot = read_ok(&[
        required(&dir, "a.txt", b"FIRST_RULE_LONG_ENOUGH"),
        required(&dir, "b.txt", b"SECOND_RULE_LONG_ENOUGH\n"),
    ]);
    assert_eq!(snapshot.text(), "FIRST_RULE_LONG_ENOUGH\nSECOND_RULE_LONG_ENOUGH\n");
    let compiled = snapshot.compile().expect("compile");
    assert_eq!(compiled.len(), 2);
}

#[test]
fn a_file_without_a_final_newline_cannot_fuse_with_the_next_files_first_line() {
    let dir = temp_dir("fuse");
    let snapshot = read_ok(&[
        required(&dir, "a.txt", b"==> qqq-first <==\nFIRST_RULE_LONG_ENOUGH"),
        required(&dir, "b.txt", b"==> qqq-second <==\nSECOND_RULE_LONG_ENOUGH"),
    ]);
    let compiled = snapshot.compile().expect("compile");
    assert_eq!(compiled.names(), &[Some("qqq-first".to_string()), Some("qqq-second".to_string())]);
}

#[test]
fn a_bom_at_the_start_of_a_later_file_is_stripped_before_the_join() {
    let dir = temp_dir("bom");
    let snapshot = read_ok(&[
        required(&dir, "a.txt", b"==> qqq-first <==\nFIRST_RULE_LONG_ENOUGH\n"),
        required(&dir, "b.txt", "\u{feff}==> qqq-second <==\nSECOND_RULE_LONG_ENOUGH\n".as_bytes()),
    ]);
    assert!(!snapshot.text()[1..].contains('\u{feff}'));
    let compiled = snapshot.compile().expect("a stripped BOM leaves a strict header");
    assert_eq!(compiled.names(), &[Some("qqq-first".to_string()), Some("qqq-second".to_string())]);
}

#[test]
fn a_legacy_file_after_a_tail_file_is_refused_naming_both_files() {
    let dir = temp_dir("mixed");
    let error = read_err(&[
        required(&dir, "tail.txt", b"==> qqq-first <==\nFIRST_RULE_LONG_ENOUGH\n"),
        required(&dir, "legacy.txt", b"SECOND_RULE_LONG_ENOUGH\n"),
    ]);
    assert!(error.contains("legacy.txt"), "{error}");
    assert!(error.contains("tail.txt"), "{error}");
    assert!(error.contains("one format"), "{error}");
}

#[test]
fn a_tail_file_after_a_legacy_file_is_refused() {
    let dir = temp_dir("mixed-reverse");
    let error = read_err(&[
        required(&dir, "legacy.txt", b"FIRST_RULE_LONG_ENOUGH\n"),
        required(&dir, "tail.txt", b"==> qqq-second <==\nSECOND_RULE_LONG_ENOUGH\n"),
    ]);
    assert!(error.contains("tail.txt") && error.contains("legacy.txt"), "{error}");
}

#[test]
fn two_legacy_files_join_and_a_comment_only_file_matches_either_format() {
    let dir = temp_dir("legacy-and-comments");
    let snapshot = read_ok(&[
        required(&dir, "a.txt", b"FIRST_RULE_LONG_ENOUGH\n"),
        required(&dir, "comments.txt", b"# only a comment\n\n"),
        required(&dir, "b.txt", b"SECOND_RULE_LONG_ENOUGH\n"),
    ]);
    assert_eq!(snapshot.compile().expect("compile").len(), 2);
    let tail = read_ok(&[
        required(&dir, "comments-first.txt", b"# local header only\n"),
        required(&dir, "tail.txt", b"==> qqq-only <==\nONLY_RULE_LONG_ENOUGH\n"),
    ]);
    assert_eq!(tail.compile().expect("compile").names(), &[Some("qqq-only".to_string())]);
}

#[test]
fn a_comment_only_file_after_a_tail_file_leaves_that_files_rules_unchanged() {
    let dir = temp_dir("comments-after-tail");
    let snapshot = read_ok(&[
        required(&dir, "shared.txt", b"==> qqq-only <==\nONLY_RULE_LONG_ENOUGH"),
        required(&dir, "local.txt", b"# local header only\n# second comment line\n"),
    ]);
    let compiled = snapshot.compile().expect("compile");
    assert_eq!(compiled.names(), &[Some("qqq-only".to_string())]);
}

#[test]
fn an_absent_optional_entry_is_skipped_and_listed() {
    let dir = temp_dir("optional-absent");
    let snapshot = read_ok(&[required(&dir, "a.txt", b"FIRST_RULE_LONG_ENOUGH\n"), absent(&dir, "local.txt")]);
    assert_eq!(snapshot.skipped(), vec![dir.join("local.txt")]);
    assert_eq!(snapshot.text(), "FIRST_RULE_LONG_ENOUGH\n");
    assert!(!snapshot.is_empty());
    assert_eq!(snapshot.label(), dir.join("a.txt").display().to_string());
}

#[test]
fn a_present_optional_entry_is_read_like_a_required_one() {
    let dir = temp_dir("optional-present");
    let mut local = required(&dir, "local.txt", b"SECOND_RULE_LONG_ENOUGH\n");
    local.optional = true;
    let snapshot = read_ok(&[required(&dir, "a.txt", b"FIRST_RULE_LONG_ENOUGH\n"), local]);
    assert!(snapshot.skipped().is_empty());
    assert_eq!(snapshot.compile().expect("compile").len(), 2);
}

#[test]
fn a_missing_required_entry_fails_naming_its_path_even_after_present_files() {
    let dir = temp_dir("required-missing");
    let mut missing = absent(&dir, "gone.txt");
    missing.optional = false;
    let error = read_err(&[required(&dir, "a.txt", b"FIRST_RULE_LONG_ENOUGH\n"), missing]);
    assert!(error.starts_with("read rules "), "{error}");
    assert!(error.contains("gone.txt"), "{error}");
}

#[test]
fn an_optional_entry_that_fails_for_another_reason_than_absence_still_fails() {
    let dir = temp_dir("optional-directory");
    std::fs::create_dir(dir.join("is-a-directory")).expect("directory");
    let error = read_err(&[
        required(&dir, "a.txt", b"FIRST_RULE_LONG_ENOUGH\n"),
        RulesEntry { path: dir.join("is-a-directory"), optional: true },
    ]);
    assert!(error.contains("is-a-directory"), "{error}");
}

#[test]
fn a_present_file_that_is_not_utf8_fails_naming_its_path() {
    let dir = temp_dir("not-utf8");
    let error = read_err(&[required(&dir, "bytes.txt", b"\xff\xfe\xfd")]);
    assert!(error.contains("bytes.txt") && error.contains("valid UTF-8"), "{error}");
}

#[test]
fn a_list_of_only_absent_optional_entries_is_empty_and_require_present_names_every_path() {
    let dir = temp_dir("all-absent");
    let snapshot = read_ok(&[absent(&dir, "one.txt"), absent(&dir, "two.txt")]);
    assert!(snapshot.is_empty());
    let error = snapshot.require_present().expect_err("must fail").to_string();
    assert!(error.contains("one.txt") && error.contains("two.txt"), "{error}");
    assert!(error.contains("no rules loaded"), "{error}");
}

#[test]
fn a_duplicate_section_name_across_files_names_both_files() {
    let dir = temp_dir("duplicate");
    let snapshot = read_ok(&[
        required(&dir, "shared.txt", b"==> qqq-same <==\nFIRST_RULE_LONG_ENOUGH\n"),
        required(&dir, "local.txt", b"==> qqq-same <==\nSECOND_RULE_LONG_ENOUGH\n"),
    ]);
    let error = compile_err(&snapshot);
    assert!(error.contains("local.txt") && error.contains("shared.txt"), "{error}");
    assert!(error.contains("qqq-same"), "{error}");
}

#[test]
fn a_failing_rule_in_a_later_file_is_attributed_to_that_file_with_its_local_position() {
    let dir = temp_dir("attribute");
    let snapshot = read_ok(&[
        required(&dir, "shared.txt", b"==> qqq-first <==\nFIRST_RULE_LONG_ENOUGH\n"),
        required(&dir, "local.txt", b"# header\n\n==> qqq-second <==\nSECOND_RULE_LONG_ENOUGH\n==> Bad Name <==\nx\n"),
    ]);
    let error = compile_err(&snapshot);
    assert!(error.contains("local.txt"), "{error}");
    assert!(!error.contains("shared.txt"), "{error}");
    // The header on local.txt line 5 is reported by its own line, not its offset in the joined text.
    assert!(error.contains("line 5"), "{error}");
}

#[test]
fn a_failure_of_a_single_file_list_keeps_the_one_path_message() {
    let dir = temp_dir("single-failure");
    let entry = required(&dir, "only.txt", b"==> qqq-a <==\nAAA_RULE_LONG_ENOUGH\n==> qqq-a <==\nBBB_RULE_LONG_ENOUGH\n");
    let snapshot = read_ok(std::slice::from_ref(&entry));
    let error = compile_err(&snapshot);
    assert!(error.starts_with(&format!("rules {}: ", entry.path.display())), "{error}");
    assert!(error.contains("duplicate section name"), "{error}");
}

#[test]
fn an_empty_joined_text_reports_no_rules_with_the_listed_files() {
    let dir = temp_dir("only-comments");
    let snapshot = read_ok(&[
        required(&dir, "a.txt", b"# nothing\n"),
        required(&dir, "b.txt", b"# still nothing\n"),
    ]);
    let error = compile_err(&snapshot);
    assert!(error.contains("no rules loaded"), "{error}");
    assert!(error.contains("a.txt") && error.contains("b.txt"), "{error}");
}

#[test]
fn the_label_lists_present_files_in_order() {
    let dir = temp_dir("label");
    let snapshot = read_ok(&[
        required(&dir, "a.txt", b"FIRST_RULE_LONG_ENOUGH\n"),
        absent(&dir, "skipped.txt"),
        required(&dir, "b.txt", b"SECOND_RULE_LONG_ENOUGH\n"),
    ]);
    assert_eq!(
        snapshot.label(),
        format!("{}, {}", dir.join("a.txt").display(), dir.join("b.txt").display()),
    );
}

#[test]
fn the_same_present_text_has_the_same_joined_text_whether_an_optional_file_was_listed_or_not() {
    let dir = temp_dir("same-text");
    let with_absent = read_ok(&[required(&dir, "a.txt", b"FIRST_RULE_LONG_ENOUGH\n"), absent(&dir, "local.txt")]);
    let alone = read_ok(&[required(&dir, "a.txt", b"FIRST_RULE_LONG_ENOUGH\n")]);
    assert_eq!(with_absent.text(), alone.text());
}
