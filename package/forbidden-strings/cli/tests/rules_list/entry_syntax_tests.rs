//! What:
//!  The grammar of one rules list at the binary boundary:
//!  the split on every comma,
//!  the one trailing `?` that marks an entry optional,
//!  the order the entries apply in,
//!  and the exit-2 refusals for an empty,
//!  repeated,
//!  or malformed list.
//! Why:
//!  The grammar is the whole definition and there is no escape syntax,
//!  so every value a shell or a CI secret can carry either names files
//!  exactly as documented or is refused before any file is read;
//!  a silently different reading would scan a different rule set.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // entry-syntax.test.ts: split(',') then one trailing '?' per entry
//! ```

use std::fs;

use crate::{
    fixture,
    scanner,
    stderr_of,
    two_file_fixture,
    write,
};

/// A single path behaves exactly as before lists existed.
#[test]
fn a_single_path_still_names_one_required_file() {
    let root = two_file_fixture("single");
    let output = scanner(&root)
        .args(["--rules", "shared.txt", "target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("target.txt:1 rule=qqq-shared"), "{stderr}");
    assert!(!stderr.contains("qqq-local"), "{stderr}");
    assert!(!stderr.contains("optional rules file"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// Both files' rules apply,
/// in list order,
/// as one rule set.
#[test]
fn two_files_scan_as_one_rule_set() {
    let root = two_file_fixture("two");
    let output = scanner(&root)
        .args(["--rules", "shared.txt,local.txt", "target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("target.txt:1 rule=qqq-shared"), "{stderr}");
    assert!(stderr.contains("target.txt:3 rule=qqq-local"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// The variable carries the same list,
/// and the flag replaces it instead of adding to it.
#[test]
fn the_variable_carries_a_list_and_the_flag_replaces_it() {
    let root = two_file_fixture("variable");
    let from_variable = scanner(&root)
        .env("FORBIDDEN_STRINGS_RULES", "shared.txt,local.txt")
        .arg("target.txt")
        .output()
        .expect("run");
    let variable_stderr = stderr_of(&from_variable);
    assert_eq!(from_variable.status.code(), Some(1), "{variable_stderr}");
    assert!(
        variable_stderr.contains("rule=qqq-shared") && variable_stderr.contains("rule=qqq-local"),
        "{variable_stderr}"
    );

    let replaced = scanner(&root)
        .env("FORBIDDEN_STRINGS_RULES", "shared.txt,local.txt")
        .args(["--rules", "local.txt", "target.txt"])
        .output()
        .expect("run");
    let replaced_stderr = stderr_of(&replaced);
    assert_eq!(replaced.status.code(), Some(1), "{replaced_stderr}");
    assert!(
        replaced_stderr.contains("rule=qqq-local"),
        "{replaced_stderr}"
    );
    assert!(
        !replaced_stderr.contains("rule=qqq-shared"),
        "the flag must not merge with the variable: {replaced_stderr}"
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A present optional entry is read like a required one.
#[test]
fn a_present_optional_entry_applies() {
    let root = two_file_fixture("optional-present");
    let output = scanner(&root)
        .args(["--rules", "shared.txt,local.txt?", "target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("rule=qqq-local"), "{stderr}");
    assert!(!stderr.contains("optional rules file"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// An absent optional entry is skipped and logged,
/// and the scan still passes or fails on the rest.
#[test]
fn an_absent_optional_entry_is_skipped_and_logged() {
    let root = two_file_fixture("optional-absent");
    write(&root, "clean.txt", "nothing to see\n");
    let clean = scanner(&root)
        .args(["--rules", "shared.txt,missing.txt?", "clean.txt"])
        .output()
        .expect("run");
    let clean_stderr = stderr_of(&clean);
    assert_eq!(clean.status.code(), Some(0), "{clean_stderr}");
    assert!(
        clean_stderr.contains("optional rules file is absent; skipped"),
        "{clean_stderr}"
    );
    assert!(clean_stderr.contains("missing.txt"), "{clean_stderr}");

    let dirty = scanner(&root)
        .args(["--rules", "shared.txt,missing.txt?", "target.txt"])
        .output()
        .expect("run");
    let dirty_stderr = stderr_of(&dirty);
    assert_eq!(dirty.status.code(), Some(1), "{dirty_stderr}");
    assert!(dirty_stderr.contains("rule=qqq-shared"), "{dirty_stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A missing required entry exits 2 and names its path,
/// whatever precedes it.
#[test]
fn a_missing_required_entry_exits_2_naming_its_path() {
    let root = two_file_fixture("required-missing");
    let output = scanner(&root)
        .args(["--rules", "shared.txt,gone.txt", "target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("read rules gone.txt"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// Every malformed list exits 2 before any file is read.
#[test]
fn malformed_lists_exit_2() {
    let root = two_file_fixture("malformed");
    for (value, expected) in [
        ("shared.txt,,local.txt", "entry 2 names no file"),
        ("shared.txt,", "entry 2 names no file"),
        (",shared.txt", "entry 1 names no file"),
        ("?", "entry 1 names no file"),
        (
            "shared.txt,shared.txt?",
            "entry 2 names the same file as entry 1",
        ),
        ("", "the rules list is empty"),
    ] {
        let output = scanner(&root)
            .args(["--rules", value, "target.txt"])
            .output()
            .expect("run");
        let stderr = stderr_of(&output);
        assert_eq!(output.status.code(), Some(2), "{value:?}: {stderr}");
        assert!(stderr.contains("invalid rules list"), "{value:?}: {stderr}");
        assert!(stderr.contains(expected), "{value:?}: {stderr}");
    }
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A path ending in a question mark is named as optional by doubling the mark.
#[test]
fn a_doubled_question_mark_names_a_path_that_ends_in_one() {
    let root = two_file_fixture("doubled");
    write(
        &root,
        "odd?.txt?",
        "==> qqq-odd <==\nODD_NEEDLE_LONG_ENOUGH\n",
    );
    write(&root, "odd-target.txt", "ODD_NEEDLE_LONG_ENOUGH\n");
    let output = scanner(&root)
        .args(["--rules", "shared.txt,odd?.txt??", "odd-target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("rule=qqq-odd"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A comma separates entries,
/// so a path holding one is not found as one file.
#[test]
fn a_comma_in_a_path_splits_the_entry() {
    let root = two_file_fixture("comma");
    write(
        &root,
        "a,b.txt",
        "==> qqq-comma <==\nCOMMA_NEEDLE_LONG_ENOUGH\n",
    );
    let output = scanner(&root)
        .args(["--rules", "a,b.txt", "target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("read rules a:"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}
