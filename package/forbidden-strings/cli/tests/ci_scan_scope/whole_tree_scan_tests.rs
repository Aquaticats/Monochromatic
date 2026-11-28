//! What:
//!  The whole-tree arm of the scan steps:
//!  when no validated baseline exists the file scan runs the scanner's `--all`
//!  walk,
//!  which leaves every link and every gitlink out,
//!  and these cases pin that the index's links and gitlinks still reach the
//!  scanner,
//!  each by its own name and,
//!  for a link,
//!  by its stored target text.
//! Why:
//!  Issue #6640's second half:
//!  that arm runs on the first scan and after a baseline is lost,
//!  which is exactly when a link added earlier and never delta-scanned is the
//!  only place a forbidden string could hide.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // whole-tree-scan.test.ts: the --all arm still inspects the index's links and gitlinks
//! ```

use std::fs;

use crate::Fixture;
use crate::fixture_inputs::{
    FIXTURE_RULE,
    NAME_RULE,
    NAME_SENTINEL,
    SENTINEL,
};

// What:     `const LINK_SCAN_LINE` is the opening of the link-scan step's
//           narration, up to the count it prints.
// Why:      A case must show the whole-tree arm reached the step that inspects
//           links, not only that some finding appeared.
const LINK_SCAN_LINE: &str = "link and gitlink entries: each one by its own name";

// What:     `const WHOLE_TREE_LINE` is the part of the whole-tree arm's line
//           that names what it handed over.
// Why:      The arm must say what the walk leaves out and what the index gives
//           the scanner instead, so a reader can tell a covered link from a
//           skipped one.
const WHOLE_TREE_LINE: &str = "Whole-tree scan:";

// What:     `#[test] fn whole_tree_scan_reports_a_link_whose_own_name_is_forbidden()`
//           runs the whole-tree arm over a tree whose one tracked folder link
//           holds the string a pathname rule forbids in its own name, and
//           expects exit 1 with that finding.
// Why:      The `--all` walk leaves `Mode::SYMLINK` out, so before this change
//           nothing inspected a link's name in that arm.
#[test]
fn whole_tree_scan_reports_a_link_whose_own_name_is_forbidden() {
    let mut fixture = Fixture::new("whole-tree-link-name");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.commit("A");
    fixture.link(
        &format!("consumer/uses/{}-folder", NAME_SENTINEL),
        "../../base",
    );
    fixture.commit("B");
    // No listed run is a validated baseline, so the arm scans the whole tree.
    fixture.write_runs(&[]);

    let result = fixture.run_scan("push", "B", "main", "");

    assert!(
        result.status == 1,
        "a forbidden string in a whole-tree-scanned link's own name must fail the step; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stderr.contains(&format!(
            "consumer/uses/[REDACTED]:name:3 rule={} input=0",
            NAME_RULE,
        )),
        "the finding must name the link's redacted third segment and its input index\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains(WHOLE_TREE_LINE)
            && result
                .stdout
                .contains("tracked links 1, submodule gitlinks 0"),
        "the arm must say it handed the index's one link over\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn whole_tree_scan_reports_a_link_whose_stored_target_text_is_forbidden()`
//           runs the whole-tree arm over a tree whose one tracked link stores a
//           target text holding the string a content rule forbids, and expects
//           exit 1 with that finding.
// Why:      A link's stored target text is the `120000` blob's bytes; neither
//           the walk nor any positional list holds them, so before this change
//           a forbidden string there was invisible in every arm.
#[test]
fn whole_tree_scan_reports_a_link_whose_stored_target_text_is_forbidden() {
    let mut fixture = Fixture::new("whole-tree-link-target-text");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.commit("A");
    // The link dangles on purpose: its stored text is the only bytes that hold
    // the forbidden string, so a step that read the path it points at would
    // find nothing and report a read error instead.
    fixture.link("consumer/uses/clean-name", &format!("../../{}", SENTINEL));
    fixture.commit("B");
    fixture.write_runs(&[]);

    let result = fixture.run_scan("push", "B", "main", "");

    assert!(
        result.status == 1,
        "a forbidden string in a whole-tree-scanned link's stored target text must fail the step; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stderr.contains(&format!(
            "consumer/uses/clean-name:1 rule={} input=0",
            FIXTURE_RULE,
        )),
        "the finding must name the link, line one of its stored target text, and its input index\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        !result.stderr.contains("read error"),
        "nothing behind the link may be opened\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn whole_tree_scan_reports_a_gitlink_whose_own_name_is_forbidden()`
//           runs the whole-tree arm over a tree whose one submodule entry holds
//           the string a pathname rule forbids in its own name, and expects
//           exit 1 with that finding.
// Why:      The walk leaves `Mode::COMMIT` out because a gitlink's path is a
//           folder or absent; its name is the whole of what a scan of it can
//           cover, and nothing covered it in this arm.
#[test]
fn whole_tree_scan_reports_a_gitlink_whose_own_name_is_forbidden() {
    let mut fixture = Fixture::new("whole-tree-gitlink-name");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.commit("A");
    fixture.gitlink(&format!("vendor/{}-sub", NAME_SENTINEL), "A");
    fixture.commit_index("B");
    fixture.write_runs(&[]);

    let result = fixture.run_scan("push", "B", "main", "");

    assert!(
        result.status == 1,
        "a forbidden string in a whole-tree-scanned gitlink's own name must fail the step; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stderr.contains(&format!(
            "vendor/[REDACTED]:name:2 rule={} input=0",
            NAME_RULE,
        )),
        "the finding must name the gitlink's redacted second segment and its input index\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stdout
            .contains("tracked links 0, submodule gitlinks 1"),
        "the arm must say it handed the index's gitlink over\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn whole_tree_scan_says_which_links_and_gitlinks_it_hands_over()`
//           runs the whole-tree arm over a clean tree holding one folder link,
//           one link to a file and one submodule entry, and expects exit 0
//           after the arm's line names both counts and the link-scan step says
//           what it handed the scanner.
// Why:      A coverage change that reports nothing looks the same as one that
//           inspected everything; the line is what a reader uses to tell them
//           apart, and the clean run is the control for the three red cases.
#[test]
fn whole_tree_scan_says_which_links_and_gitlinks_it_hands_over() {
    let mut fixture = Fixture::new("whole-tree-clean");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.commit("A");
    fixture.link("consumer/uses/base", "../../base");
    fixture.link("consumer/uses/lib", "../../base/lib.txt");
    // The two links are staged by name because the commit below commits the
    // index as it stands: `git add --all` cannot stage a submodule entry whose
    // folder is absent, so the gitlink is recorded in the index first.
    fixture.git(&["add", "--", "consumer/uses/base", "consumer/uses/lib"]);
    fixture.gitlink("vendor/sub", "A");
    fixture.commit_index("B");
    fixture.write_runs(&[]);

    let result = fixture.run_scan("push", "B", "main", "");

    assert!(
        result.status == 0,
        "a clean whole-tree scan must pass; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        !result.stderr.contains("rule=") && !result.stderr.contains("read error"),
        "a clean tree may report nothing\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains(WHOLE_TREE_LINE)
            && result
                .stdout
                .contains("tracked links 2, submodule gitlinks 1"),
        "the arm's line must count the links and gitlinks it handed over\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains(LINK_SCAN_LINE),
        "the link-scan step must say what it scanned\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn whole_tree_scan_hands_over_a_name_holding_a_space_a_comma_and_a_line_break()`
//           runs the whole-tree arm over a tree whose links' names hold a
//           space, a comma and a line break, and expects each name to arrive
//           whole.
// Why:      The index listing is NUL-separated and the names are re-emitted into
//           another NUL-separated list before `--name-path` sees them; a split
//           at any of those bytes would stop a rule from matching instead of
//           failing loudly, so the rules span the byte.
#[test]
fn whole_tree_scan_hands_over_a_name_holding_a_space_a_comma_and_a_line_break() {
    let mut fixture = Fixture::new("whole-tree-special-byte-names");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.commit("A");
    fixture.link("consumer/uses/zzq name-link", "../../base");
    fixture.link("consumer/uses/zzq,name-link", "../../base");
    // A name holding a line break cannot be matched by a single-line rule, so
    // `forbidden-strings` 0.6.2 answers
    // `<redacted>: unsupported pathname line break` (built at
    // `package/forbidden-strings/cli/src/scan_finding.rs:76`) and counts it
    // toward exit 1; the case pins that the name arrives in one piece.
    fixture.link("consumer/uses/zzqline\nbreak-link", "../../base");
    fixture.commit("B");
    fixture.write_runs(&[]);

    let result = fixture.run_scan("push", "B", "main", "");

    assert!(
        result.status == 1,
        "names holding a space, a comma or a line break must still be inspected; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stderr.contains("rule=ci-scope-fixture-space"),
        "the rule spanning the space must match, so the name was not split there\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stderr.contains("rule=ci-scope-fixture-comma"),
        "the rule spanning the comma must match, so the name arrived whole\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stderr
            .contains("[REDACTED]: unsupported pathname line break"),
        "the name holding a line break must reach the scanner in one piece\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn whole_tree_scan_still_reports_a_regular_file()`
//           runs the whole-tree arm over a tree whose regular file holds the
//           forbidden string, and expects the finding beside the arm's line.
// Why:      The positive control for the arm: handing links and gitlinks over
//           must not replace the `--all` walk, which is what inspects the
//           tree's regular files.
#[test]
fn whole_tree_scan_still_reports_a_regular_file() {
    let mut fixture = Fixture::new("whole-tree-regular-file");
    fixture.write_workflow_revision(true);
    fixture.put("consumer/main.txt", &format!("clean\n{}\n", SENTINEL));
    fixture.commit("A");
    // The finding is in the tree the arm walks, not in any changed entry: the
    // same tree is both the baseline and the head here.
    fixture.write_runs(&[]);

    let result = fixture.run_scan("push", "A", "main", "");

    assert!(
        result.status == 1,
        "the arm must still report a regular file's forbidden string; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stderr
            .contains(&format!("consumer/main.txt:2 rule={}", FIXTURE_RULE)),
        "the finding must name the regular file and its line\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}
