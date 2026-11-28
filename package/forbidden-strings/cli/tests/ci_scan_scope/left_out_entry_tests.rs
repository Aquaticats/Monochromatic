//! What:
//!  What the scan steps hand the scanner for each changed entry the file scan
//!  leaves out:
//!  the entry's own name,
//!  and,
//!  for a tracked link,
//!  the target text Git stores for it,
//!  taken from the link's own Git object and never from the path it points at.
//! Why:
//!  Issue #6633:
//!  the step that keeps `Is a directory (os error 21)` out of the scan left
//!  those entries inspected by nothing.
//!  Measured on `forbidden-strings` 0.6.2,
//!  `--name-path <link path> <materialized target text>` name-scans the link's
//!  own path and content-scans the stored target text in one invocation,
//!  so the workflow can cover both without a release of the scanner.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // left-out-entry.test.ts: what the step hands over for a folder link and a gitlink
//! ```

use std::fs;

use crate::fixture_inputs::{
    FIXTURE_RULE,
    NAME_RULE,
    NAME_SENTINEL,
    SENTINEL,
};
use crate::scan_step_command::{
    LEFT_OUT_STEP_NAME,
    declared_step_body,
};
use crate::{
    Fixture,
    RunRow,
};

// What:     `const WHICH_ENTRIES_SCAN_NEXT` is the sentence the file-scan step
//           prints to say what inspects the entries it left out.
// Why:      A gate that drops entries must say where they went; without this
//           sentence a reader cannot tell the left-out step from a silent loss.
const WHICH_ENTRIES_SCAN_NEXT: &str =
    "The next step scans their own names and, for each link, its stored target text.";

// What:     `const LEFT_OUT_SCAN_LINE` is the opening of the left-out step's
//           narration, up to the count it prints.
// Why:      The cases pin that the step ran and how many entries it handed
//           over, without depending on the count word or the pair indexes the
//           fixture's own diff order decides.
const LEFT_OUT_SCAN_LINE: &str = "left-out entries: each one by its own name";

// What:     `#[test] fn push_of_a_folder_link_whose_own_name_is_forbidden_ends_the_step_1()`
//           pushes one folder link whose own name holds the string a pathname
//           rule forbids, and expects exit 1 with that finding.
// Why:      This is the name half of issue #6633: before the fix nothing
//           inspected a left-out link's name once the link alone was the
//           delta, and the step stayed green.
#[test]
fn push_of_a_folder_link_whose_own_name_is_forbidden_ends_the_step_1() {
    let mut fixture = Fixture::new("left-out-link-name");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean before the push\n");
    fixture.put("consumer/main.txt", "clean before the push\n");
    fixture.commit("A");
    fixture.put("consumer/main.txt", "clean\nchanged by the push\n");
    // The link's stored text `../../base` holds no forbidden string and its
    // target is a folder, so the file scan leaves it out and only the name
    // remains for anything to inspect.
    fixture.link(
        &format!("consumer/uses/{}-folder-link", NAME_SENTINEL),
        "../../base",
    );
    fixture.commit("B");
    fixture.write_runs(&[RunRow {
        conclusion: "success",
        event: "push",
        head_branch: "main",
        workflow: "forbidden-strings.yml",
        head_label: "A",
    }]);

    let result = fixture.run_scan("push", "B", "main", "A");

    assert!(
        result.status == 1,
        "a forbidden string in a left-out folder link's own name must fail the step; got status {}\nstdout: {}\nstderr: {}",
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
        result.stdout.contains(WHICH_ENTRIES_SCAN_NEXT),
        "the file-scan step must say what scans the entries it left out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains(LEFT_OUT_SCAN_LINE),
        "the left-out step must say it handed the entry over by name\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn push_of_a_folder_link_whose_target_text_is_forbidden_ends_the_step_1()`
//           pushes one folder link whose stored target text holds the string a
//           content rule forbids, and expects exit 1 with that finding.
// Why:      This is the target-text half of issue #6633: no scan of any kind
//           has ever inspected the bytes Git stores for a link, so a forbidden
//           string there was invisible and the step stayed green.
#[test]
fn push_of_a_folder_link_whose_target_text_is_forbidden_ends_the_step_1() {
    let mut fixture = Fixture::new("left-out-link-target-text");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean before the push\n");
    // The target folder carries the forbidden string in its own name, so the
    // link's stored text holds it while the link's own name and the target
    // folder's file stay clean of it.
    fixture.put(
        &format!("base/{}/lib.txt", SENTINEL),
        "clean inside the target folder\n",
    );
    fixture.put("consumer/main.txt", "clean before the push\n");
    fixture.commit("A");
    fixture.put("consumer/main.txt", "clean\nchanged by the push\n");
    fixture.link(
        "consumer/uses/clean-target",
        &format!("../../base/{}", SENTINEL),
    );
    fixture.commit("B");
    fixture.write_runs(&[RunRow {
        conclusion: "success",
        event: "push",
        head_branch: "main",
        workflow: "forbidden-strings.yml",
        head_label: "A",
    }]);

    let result = fixture.run_scan("push", "B", "main", "A");

    assert!(
        result.status == 1,
        "a forbidden string in a left-out folder link's stored target text must fail the step; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stderr.contains(&format!(
            "consumer/uses/clean-target:1 rule={} input=0",
            FIXTURE_RULE,
        )),
        "the finding must name the link, line one of its stored target text, and its input index\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stdout
            .contains("folder links 1, submodule gitlinks 0"),
        "the file-scan step must count the link it left out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn push_of_a_submodule_gitlink_whose_own_name_is_forbidden_ends_the_step_1()`
//           pushes one submodule entry whose own name holds the string a
//           pathname rule forbids, and expects exit 1 with that finding.
// Why:      A gitlink has no content to read and no target text to scan, so
//           its own name is the whole of what a scan of it can cover; the file
//           scan leaves it out always.
#[test]
fn push_of_a_submodule_gitlink_whose_own_name_is_forbidden_ends_the_step_1() {
    let mut fixture = Fixture::new("left-out-gitlink-name");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.commit("A");
    // The gitlink records commit A at a path whose own folder name holds the
    // forbidden string; no folder is checked out there, so only the tree mode
    // tells the step what the entry is.
    fixture.gitlink(&format!("vendor/{}-sub", NAME_SENTINEL), "A");
    fixture.commit_index("B");
    fixture.write_runs(&[RunRow {
        conclusion: "success",
        event: "push",
        head_branch: "main",
        workflow: "forbidden-strings.yml",
        head_label: "A",
    }]);

    let result = fixture.run_scan("push", "B", "main", "A");

    assert!(
        result.status == 1,
        "a forbidden string in a submodule entry's own name must fail the step; got status {}\nstdout: {}\nstderr: {}",
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
        result.stdout.contains("submodule gitlinks 1"),
        "the file-scan step must count the gitlink it left out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn push_hands_over_a_link_whose_name_holds_a_space_a_newline_and_a_comma()`
//           pushes three folder links whose names hold a space, a comma and a
//           newline, and expects each one inspected whole.
// Why:      The shell and the scanner's argument list each treat a byte
//           specially: an unquoted expansion splits a name at a space or a
//           newline, and a comma-bearing rule only matches when the segment
//           arrives in one piece. The space and comma rules below span the
//           byte, so a split name stops matching instead of passing quietly.
#[test]
fn push_hands_over_a_link_whose_name_holds_a_space_a_newline_and_a_comma() {
    let mut fixture = Fixture::new("left-out-special-byte-names");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.put("consumer/main.txt", "clean before the push\n");
    fixture.commit("A");
    fixture.put("consumer/main.txt", "clean\nchanged by the push\n");
    fixture.link("consumer/uses/zzq name-link", "../../base");
    fixture.link("consumer/uses/zzq,name-link", "../../base");
    // A name holding a line break cannot be matched by a single-line rule, so
    // `forbidden-strings` 0.6.2 reports it as
    // `<redacted>: unsupported pathname line break` (built at
    // `package/forbidden-strings/cli/src/scan_finding.rs:76`) and counts it
    // toward exit 1; the case pins that the name arrives in one piece.
    fixture.link("consumer/uses/zzqline\nbreak-link", "../../base");
    fixture.commit("B");
    fixture.write_runs(&[RunRow {
        conclusion: "success",
        event: "push",
        head_branch: "main",
        workflow: "forbidden-strings.yml",
        head_label: "A",
    }]);

    let result = fixture.run_scan("push", "B", "main", "A");

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
    assert!(
        !result.stderr.contains("read error"),
        "no name may arrive as a path the scanner cannot open\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn push_of_a_folder_link_whose_target_folder_holds_a_forbidden_string_stays_green()`
//           pushes one clean folder link whose target folder holds a file with
//           the forbidden string, and expects exit 0.
// Why:      The target text must come from the link's own Git object and never
//           from the path it points at; a step that followed the link, or read
//           through it, would report a file this push never changed.
#[test]
fn push_of_a_folder_link_whose_target_folder_holds_a_forbidden_string_stays_green() {
    let mut fixture = Fixture::new("left-out-link-target-not-followed");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", &format!("clean line\n{}\n", SENTINEL));
    fixture.put("consumer/main.txt", "clean before the push\n");
    fixture.commit("A");
    fixture.put("consumer/main.txt", "clean\nchanged by the push\n");
    fixture.link("consumer/uses/base", "../../base");
    fixture.commit("B");
    fixture.write_runs(&[RunRow {
        conclusion: "success",
        event: "push",
        head_branch: "main",
        workflow: "forbidden-strings.yml",
        head_label: "A",
    }]);

    let result = fixture.run_scan("push", "B", "main", "A");

    assert!(
        result.status == 0,
        "a clean folder link must pass even when its target folder holds the string; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        !result.stderr.contains("rule=") && !result.stderr.contains("read error"),
        "nothing behind the link may be read or reported\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains(LEFT_OUT_SCAN_LINE),
        "the clean link must still be handed over by name and target text\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn pull_request_scan_reports_a_folder_link_whose_target_text_is_forbidden()`
//           runs the pull-request arm with the forbidden string in a folder
//           link's stored target text, and expects exit 1 with that finding.
// Why:      The step keeps two scope arms; the merge-base arm must hand each
//           entry it leaves out over the same way the push arm does.
#[test]
fn pull_request_scan_reports_a_folder_link_whose_target_text_is_forbidden() {
    let mut fixture = Fixture::new("pull-request-left-out-target-text");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.put(
        &format!("base/{}/lib.txt", SENTINEL),
        "clean inside the target folder\n",
    );
    fixture.put("consumer/main.txt", "clean\n");
    fixture.commit("A");
    fixture.git(&["switch", "--create", "feature", &fixture.sha("A")]);
    fixture.put("consumer/main.txt", "clean\nchanged by the pull request\n");
    fixture.link(
        "consumer/uses/clean-target",
        &format!("../../base/{}", SENTINEL),
    );
    fixture.commit("P");
    fixture.git(&["switch", "main"]);
    fixture.put("other.txt", "clean\nchanged on main after the fork\n");
    fixture.commit("M");
    fixture.git(&["update-ref", "refs/remotes/origin/main", &fixture.sha("M")]);
    fixture.git(&["switch", "feature"]);
    fixture.write_runs(&[]);

    let result = fixture.run_scan("pull_request", "P", "feature", "");

    assert!(
        result.status == 1,
        "the pull-request arm must report a forbidden string in a left-out link's target text; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stderr.contains(&format!(
            "consumer/uses/clean-target:1 rule={} input=0",
            FIXTURE_RULE,
        )),
        "the finding must name the link and line one of its stored target text\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("origin/main...HEAD")
            && result.stdout.contains(WHICH_ENTRIES_SCAN_NEXT),
        "the pull-request arm must keep its merge-base scope while saying what scans its left-out entries\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn push_of_a_file_named_like_an_option_is_still_scanned()`
//           pushes a regular file whose name starts with a dash and holds the
//           forbidden string, and expects the finding.
// Why:      A repository-relative name from `git diff --raw` can start with a
//           dash, and `forbidden-strings` reads such a word as an option
//           unless the positional list follows `--`; without the separator the
//           gate answered exit 2 instead of reporting the file.
#[test]
fn push_of_a_file_named_like_an_option_is_still_scanned() {
    let mut fixture = Fixture::new("dash-named-file");
    fixture.write_workflow_revision(true);
    fixture.put("consumer/main.txt", "clean\n");
    fixture.commit("A");
    fixture.put("-dashfile.txt", &format!("clean\n{}\n", SENTINEL));
    fixture.commit("B");
    fixture.write_runs(&[RunRow {
        conclusion: "success",
        event: "push",
        head_branch: "main",
        workflow: "forbidden-strings.yml",
        head_label: "A",
    }]);

    let result = fixture.run_scan("push", "B", "main", "A");

    assert!(
        result.status == 1,
        "a changed file named like an option must still be scanned; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stderr
            .contains(&format!("-dashfile.txt:2 rule={}", FIXTURE_RULE)),
        "the finding must name the dash-named file and its line\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn the_workflow_declares_a_step_that_scans_the_entries_it_leaves_out()`
//           reads the workflow and expects one step whose body takes each
//           left-out entry's name and each link's stored target text.
// Why:      Every other case here asserts behaviour, so a workflow that dropped
//           the step would make them report what the remaining steps hand over;
//           this case makes the missing step itself the finding, and pins the
//           two mechanisms the requirement names: the entry's own name, and the
//           link's target text read out of its Git object.
#[test]
fn the_workflow_declares_a_step_that_scans_the_entries_it_leaves_out() {
    // What:     `body` is the extracted shell of the left-out step, or `None`
    //           when the workflow declares no such step.
    // Why:      The three assertions below are about one step's text, and a
    //           missing step must fail here rather than take the binary down.
    let body: String = declared_step_body(LEFT_OUT_STEP_NAME).unwrap_or_else(|| {
        panic!(
            "the workflow must declare its `{}` step",
            LEFT_OUT_STEP_NAME.trim(),
        )
    });

    assert!(
        body.contains("--name-path"),
        "the step must hand each left-out entry over under its own name\nbody: {}",
        body,
    );
    assert!(
        body.contains("git cat-file blob"),
        "the step must read each link's stored target text from its own Git object\nbody: {}",
        body,
    );
}
