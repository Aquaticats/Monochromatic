// Precedence, rule-identity, and offset tests for the frx loader.

use super::load;
use crate::frx_scan::scan_file;

// Creates a fresh temp dir under the test scratch root, keyed by label and pid.
fn temp_dir(label: &str) -> std::path::PathBuf {
    let dir = crate::test_scratch::scratch_root().join(format!("frx-load-{}-{}", label, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    return dir
}

// Writes a rules file into a temp dir and returns its path (dir leaks per-test).
fn rules_file(label: &str, body: &str) -> std::path::PathBuf {
    let dir = temp_dir(label);
    let path = dir.join("rules.txt");
    std::fs::write(&path, body).expect("write rules");
    return path
}

// Serializes a compiled set into precompiled bytes for the builtin-baseline slot.
fn precompiled(body: &str) -> Vec<u8> {
    return crate::compile_from_text(body)
        .expect("compile precompiled fixture")
        .to_bytes()
        .expect("serialize precompiled fixture")
}

// Builds a name sidecar of `count` unnamed entries (one empty line per rule).
fn unnamed_sidecar(count: usize) -> String {
    return "\n".repeat(count)
}

#[test]
fn runtime_only_ids_start_at_zero() {
    let path = rules_file("runtime-only", "AAA_USER_ONE_LONG\n");
    let loaded = load(path.to_str().expect("utf8"), false, true, b"", "").expect("load");
    let hits = scan_file("t.txt", b"AAA_USER_ONE_LONG\n", &loaded);
    assert_eq!(hits, vec!["t.txt:1 rule=0".to_string()]);
}

#[test]
fn builtin_ids_offset_past_runtime_rules() {
    // Two unnamed runtime rules take ids 0 and 1; the unnamed builtin rule takes id 2.
    let path = rules_file("offset", "AAA_USER_ONE_LONG\nBBB_USER_TWO_LONG\n");
    let bytes = precompiled("BUILTIN_MARKER_LONG_ENOUGH\n");
    let loaded =
        load(path.to_str().expect("utf8"), true, true, &bytes, &unnamed_sidecar(1))
            .expect("load");
    let buf = b"AAA_USER_ONE_LONG\nBBB_USER_TWO_LONG\nBUILTIN_MARKER_LONG_ENOUGH\n";
    let hits = scan_file("t.txt", buf, &loaded);
    assert_eq!(
        hits,
        vec![
            "t.txt:1 rule=0".to_string(),
            "t.txt:2 rule=1".to_string(),
            "t.txt:3 rule=2".to_string(),
        ],
    );
}

#[test]
fn named_runtime_rules_render_section_names() {
    // A tail-format runtime file renders every finding through its section name,
    // regardless of the rule's position.
    let path = rules_file(
        "named-runtime",
        "==> qqq-alpha <==\nAAA_USER_ONE_LONG\n==> qqq-beta <==\nBBB_USER_TWO_LONG\n",
    );
    let loaded = load(path.to_str().expect("utf8"), false, true, b"", "").expect("load");
    let hits = scan_file("t.txt", b"BBB_USER_TWO_LONG\nAAA_USER_ONE_LONG\n", &loaded);
    assert_eq!(
        hits,
        vec!["t.txt:1 rule=qqq-beta".to_string(), "t.txt:2 rule=qqq-alpha".to_string()],
    );
}

#[test]
fn builtin_names_sidecar_renders_baseline_names() {
    // A named sidecar entry renders the baseline finding as its name; the numeric
    // offset applies only to unnamed entries, so the name is position-independent.
    let path = rules_file("named-builtin", "AAA_USER_ONE_LONG\n");
    let bytes = precompiled("BUILTIN_MARKER_LONG_ENOUGH\n");
    let loaded =
        load(path.to_str().expect("utf8"), true, true, &bytes, "qqq-baseline-rule\n")
            .expect("load");
    let buf = b"AAA_USER_ONE_LONG\nBUILTIN_MARKER_LONG_ENOUGH\n";
    let hits = scan_file("t.txt", buf, &loaded);
    assert_eq!(
        hits,
        vec!["t.txt:1 rule=0".to_string(), "t.txt:2 rule=qqq-baseline-rule".to_string()],
    );
}

#[test]
fn runtime_name_colliding_with_builtin_name_fails_closed() {
    // The same section name in the runtime file and the baseline would render two
    // ambiguous `rule=<name>` findings, so the load rejects it, naming the culprit.
    let path = rules_file("collision", "==> qqq-shared <==\nAAA_USER_ONE_LONG\n");
    let bytes = precompiled("BUILTIN_MARKER_LONG_ENOUGH\n");
    let error = load(path.to_str().expect("utf8"), true, true, &bytes, "qqq-shared\n")
        .err()
        .expect("must error");
    let rendered = format!("{error}");
    assert!(rendered.contains("collides"), "{rendered}");
    assert!(rendered.contains("qqq-shared"), "{rendered}");
}

#[test]
fn builtin_sidecar_count_mismatch_fails_closed() {
    // A sidecar whose entry count disagrees with the decoded set is a build
    // invariant break; misattributing names would be worse than failing.
    let path = rules_file("mismatch", "AAA_USER_ONE_LONG\n");
    let bytes = precompiled("BUILTIN_MARKER_LONG_ENOUGH\n");
    let error = load(path.to_str().expect("utf8"), true, true, &bytes, "qqq-a\nqqq-b\n")
        .err()
        .expect("must error");
    assert!(format!("{error}").contains("name sidecar"), "{error}");
}

#[test]
fn missing_default_with_builtin_scans_baseline_alone() {
    // A missing implicit default under --builtin-rules leaves the baseline alone at
    // offset 0.
    let dir = temp_dir("baseline-alone");
    let missing = dir.join("nope.txt");
    let bytes = precompiled("BUILTIN_MARKER_LONG_ENOUGH\n");
    let loaded =
        load(missing.to_str().expect("utf8"), true, false, &bytes, &unnamed_sidecar(1))
            .expect("load");
    let hits = scan_file("t.txt", b"BUILTIN_MARKER_LONG_ENOUGH\n", &loaded);
    assert_eq!(hits, vec!["t.txt:1 rule=0".to_string()]);
}

#[test]
fn explicit_missing_file_errors_with_read_rules() {
    let dir = temp_dir("explicit-missing");
    let missing = dir.join("nope.txt");
    let bytes = precompiled("BUILTIN_MARKER_LONG_ENOUGH\n");
    let error = load(missing.to_str().expect("utf8"), true, true, &bytes, &unnamed_sidecar(1))
        .err()
        .expect("must error");
    assert!(format!("{error}").contains("read rules"), "{error}");
}

// Builds a list entry for a file in a fixture directory.
fn entry(dir: &std::path::Path, name: &str, optional: bool) -> crate::rules_list::RulesEntry {
    return crate::rules_list::RulesEntry { path: dir.join(name), optional }
}

#[test]
fn listed_files_keep_list_order_and_the_builtin_follows_the_last_of_them() {
    let dir = temp_dir("list-order");
    std::fs::write(dir.join("a.txt"), "AAA_USER_ONE_LONG\n").expect("write a");
    std::fs::write(dir.join("b.txt"), "BBB_USER_TWO_LONG\n").expect("write b");
    let bytes = precompiled("BUILTIN_MARKER_LONG_ENOUGH\n");
    let entries = [entry(&dir, "a.txt", false), entry(&dir, "b.txt", false)];
    let loaded = super::load_listed(&entries, true, false, &bytes, &unnamed_sidecar(1)).expect("load");
    let buf = b"BBB_USER_TWO_LONG\nAAA_USER_ONE_LONG\nBUILTIN_MARKER_LONG_ENOUGH\n";
    let hits = scan_file("t.txt", buf, &loaded);
    assert_eq!(
        hits,
        vec![
            "t.txt:1 rule=1".to_string(),
            "t.txt:2 rule=0".to_string(),
            "t.txt:3 rule=2".to_string(),
        ],
    );
    assert!(loaded.skipped_optional().is_empty());
}

#[test]
fn an_absent_optional_entry_is_reported_and_its_rules_are_not_loaded() {
    let dir = temp_dir("list-optional");
    std::fs::write(dir.join("a.txt"), "AAA_USER_ONE_LONG\n").expect("write a");
    let entries = [entry(&dir, "a.txt", false), entry(&dir, "absent.txt", true)];
    let loaded = super::load_listed(&entries, false, false, b"", "").expect("load");
    assert_eq!(loaded.skipped_optional(), &[dir.join("absent.txt")]);
    let hits = scan_file("t.txt", b"AAA_USER_ONE_LONG\n", &loaded);
    assert_eq!(hits, vec!["t.txt:1 rule=0".to_string()]);
}

#[test]
fn a_required_entry_after_a_present_one_still_errors_naming_it() {
    let dir = temp_dir("list-required");
    std::fs::write(dir.join("a.txt"), "AAA_USER_ONE_LONG\n").expect("write a");
    let entries = [entry(&dir, "a.txt", false), entry(&dir, "gone.txt", false)];
    let error = super::load_listed(&entries, true, false, b"", "").err().expect("must error");
    let rendered = format!("{error}");
    assert!(rendered.contains("read rules") && rendered.contains("gone.txt"), "{rendered}");
}

#[test]
fn only_absent_optional_entries_scan_with_the_baseline_alone_or_fail_closed() {
    let dir = temp_dir("list-all-absent");
    let entries = [entry(&dir, "one.txt", true), entry(&dir, "two.txt", true)];
    let bytes = precompiled("BUILTIN_MARKER_LONG_ENOUGH\n");
    let loaded =
        super::load_listed(&entries, true, false, &bytes, &unnamed_sidecar(1)).expect("baseline alone");
    assert_eq!(loaded.skipped_optional().len(), 2);
    let hits = scan_file("t.txt", b"BUILTIN_MARKER_LONG_ENOUGH\n", &loaded);
    assert_eq!(hits, vec!["t.txt:1 rule=0".to_string()]);

    let error = super::load_listed(&entries, false, false, b"", "").err().expect("must error");
    let rendered = format!("{error}");
    assert!(rendered.contains("no rules loaded"), "{rendered}");
    assert!(rendered.contains("one.txt") && rendered.contains("two.txt"), "{rendered}");
}

#[test]
fn a_name_in_a_later_file_colliding_with_a_builtin_name_fails_closed() {
    let dir = temp_dir("list-collision");
    std::fs::write(dir.join("a.txt"), "==> qqq-first <==\nAAA_USER_ONE_LONG\n").expect("write a");
    std::fs::write(dir.join("b.txt"), "==> qqq-shared <==\nBBB_USER_TWO_LONG\n").expect("write b");
    let bytes = precompiled("BUILTIN_MARKER_LONG_ENOUGH\n");
    let entries = [entry(&dir, "a.txt", false), entry(&dir, "b.txt", false)];
    let error = super::load_listed(&entries, true, false, &bytes, "qqq-shared\n").err().expect("must error");
    let rendered = format!("{error}");
    assert!(rendered.contains("collides") && rendered.contains("qqq-shared"), "{rendered}");
}

#[test]
fn missing_default_without_builtin_errors_with_read_rules() {
    let dir = temp_dir("no-builtin-missing");
    let missing = dir.join("nope.txt");
    let error = load(missing.to_str().expect("utf8"), false, false, b"", "")
        .err()
        .expect("must error");
    assert!(format!("{error}").contains("read rules"), "{error}");
}
