//! What:
//!  What several files must agree on before they can scan as one rule set:
//!  one format across all of them,
//!  section names unique across the join,
//!  a BOM and a missing final newline handled per file,
//!  a path that is not UTF-8 refused by name,
//!  and the implicit default file staying quiet when absent under the
//!  baseline.
//! Why:
//!  The files are joined into one text and compiled once,
//!  so an agreement failure that slipped through would attribute a finding
//!  to the wrong file,
//!  or turn a legacy file's lines into the body of the last section.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // format-agreement.test.ts: one format per list, unique names, per-file BOM
//! ```

use std::fs;

use crate::{
    fixture,
    scanner,
    stderr_of,
    two_file_fixture,
    write,
};

/// A list with no file present fails closed,
/// and the baseline can stand in for them only when asked for.
#[test]
fn only_absent_optional_entries_fail_unless_the_baseline_is_on() {
    let root = two_file_fixture("all-absent");
    write(&root, "clean.txt", "nothing to see\n");
    let refused = scanner(&root)
        .args(["--rules", "a.txt?,b.txt?", "clean.txt"])
        .output()
        .expect("run");
    let refused_stderr = stderr_of(&refused);
    assert_eq!(refused.status.code(), Some(2), "{refused_stderr}");
    assert!(
        refused_stderr.contains("no rules loaded"),
        "{refused_stderr}"
    );
    assert!(
        refused_stderr.contains("a.txt") && refused_stderr.contains("b.txt"),
        "{refused_stderr}"
    );

    let with_baseline = scanner(&root)
        .args(["--builtin-rules", "--rules", "a.txt?,b.txt?", "clean.txt"])
        .output()
        .expect("run");
    let baseline_stderr = stderr_of(&with_baseline);
    assert_eq!(with_baseline.status.code(), Some(0), "{baseline_stderr}");
    assert!(
        baseline_stderr.contains("a.txt") && baseline_stderr.contains("b.txt"),
        "{baseline_stderr}"
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

/// Files in different formats are refused,
/// naming both.
#[test]
fn files_in_different_formats_are_refused() {
    let root = two_file_fixture("mixed");
    write(&root, "legacy.txt", "LEGACY_NEEDLE_LONG_ENOUGH\n");
    let output = scanner(&root)
        .args(["--rules", "shared.txt,legacy.txt", "target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("legacy.txt") && stderr.contains("shared.txt"),
        "{stderr}"
    );
    assert!(stderr.contains("one format"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A section name repeated across files exits 2 and names both files.
#[test]
fn a_section_declared_in_two_files_is_refused() {
    let root = two_file_fixture("duplicate");
    write(
        &root,
        "other.txt",
        "==> qqq-shared <==\nOTHER_NEEDLE_LONG_ENOUGH\n",
    );
    let output = scanner(&root)
        .args(["--rules", "shared.txt,other.txt", "target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("other.txt") && stderr.contains("shared.txt"),
        "{stderr}"
    );
    assert!(stderr.contains("qqq-shared"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A BOM and a missing final newline in the first file leave the second file's rules intact.
#[test]
fn a_bom_and_a_missing_final_newline_do_not_disturb_the_next_file() {
    let root = fixture("join-edges");
    write(
        &root,
        "first.txt",
        "==> qqq-first <==\nFIRST_NEEDLE_LONG_ENOUGH",
    );
    write(
        &root,
        "second.txt",
        "\u{feff}==> qqq-second <==\nSECOND_NEEDLE_LONG_ENOUGH",
    );
    write(
        &root,
        "target.txt",
        "FIRST_NEEDLE_LONG_ENOUGH\nSECOND_NEEDLE_LONG_ENOUGH\n",
    );
    let output = scanner(&root)
        .args(["--rules", "first.txt,second.txt", "target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("target.txt:1 rule=qqq-first"), "{stderr}");
    assert!(stderr.contains("target.txt:2 rule=qqq-second"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A listed file that is not UTF-8 exits 2 naming its path.
#[test]
fn a_listed_file_that_is_not_utf8_exits_2() {
    let root = two_file_fixture("not-utf8");
    fs::write(root.join("bytes.txt"), [0xff, 0xfe, 0xfd]).expect("write bytes");
    let output = scanner(&root)
        .args(["--rules", "shared.txt,bytes.txt", "target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("bytes.txt") && stderr.contains("valid UTF-8"),
        "{stderr}"
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

/// The implicit default file may still be absent under the baseline,
/// with no log line for it.
#[test]
fn the_implicit_default_stays_quiet_when_absent() {
    let root = fixture("implicit-default");
    write(&root, "clean.txt", "nothing to see\n");
    let output = scanner(&root)
        .args(["--builtin-rules", "clean.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert!(!stderr.contains("optional rules file"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}
