//! What:
//!  The changed-entry kind rule of the workflow's scan step,
//!  case by case:
//!  a tracked link to a folder and a submodule gitlink leave the list,
//!  every other entry reaches the scanner as before,
//!  and the step says in one line how many entries it left out and why.
//! Why:
//!  Issue #6632:
//!  a push whose delta held a tracked link to a folder ended red on
//!  `read error: Is a directory (os error 21)`,
//!  a failure that says nothing about secrets and kept every later push red,
//!  because no run was green to advance the scan baseline.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // folder-link-entry.test.ts: which changed entries reach the scanner
//! ```

use std::fs;

use crate::fixture_inputs::{
    FIXTURE_RULE,
    SENTINEL,
};
use crate::{
    Fixture,
    RunRow,
};

// What:     `#[test] fn push_with_a_folder_link_and_no_forbidden_string_ends_green()`
//           pushes one clean file change and one new tracked link to a folder,
//           and expects the step to pass.
// Why:      This is the acceptance regression of issue #6632: the link is the
//           only entry the step must leave out, and no forbidden string sits
//           anywhere in the delta.
#[test]
fn push_with_a_folder_link_and_no_forbidden_string_ends_green() {
    let mut fixture = Fixture::new("folder-link-clean-push");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean before the push\n");
    fixture.put("consumer/main.txt", "clean before the push\n");
    fixture.commit("A");
    fixture.put("consumer/main.txt", "clean\nchanged by the push\n");
    // `consumer/uses/base` is a link to a folder, the shape `AGENTS.md` gives
    // every `uses/` folder entry; its target text is relative to `uses/`.
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
        "a push whose only unusual entry is a folder link must pass; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        !result.stderr.contains("read error"),
        "the step must not hand the scanner an entry it cannot open as a file\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    // What:     The step's own line states the count, the total, and why the
    //           entries left; nothing may leave the list silently.
    // Why:      A gate that quietly drops entries looks the same as one that
    //           inspected them.
    assert!(
        result
            .stdout
            .contains("Left out 1 of 2 changed entries the scanner cannot open as a file"),
        "the step must say how many entries it left out and why\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("folder links 1, submodule gitlinks 0"),
        "the step must break the left-out count down by entry kind\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains(
            "A link to a file, a broken link and an unreadable file stay in the scan"
        ),
        "the line must name the entries that stay in the scan\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("consumer/main.txt") && !result.stdout.contains("consumer/uses/base"),
        "the regular file must reach the scanner while the folder link stays out of its list\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn push_still_reports_a_forbidden_string_beside_a_folder_link()`
//           pushes the same folder link beside a regular file holding the
//           forbidden literal, and expects the step to fail with that finding.
// Why:      The positive control for the acceptance case: leaving the link out
//           must not weaken the gate for the files in the same push.
#[test]
fn push_still_reports_a_forbidden_string_beside_a_folder_link() {
    let mut fixture = Fixture::new("folder-link-violation-beside");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean before the push\n");
    fixture.put("consumer/main.txt", "clean before the push\n");
    fixture.commit("A");
    fixture.put(
        "consumer/main.txt",
        &format!("clean\n{}\n", SENTINEL),
    );
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
        result.status == 1,
        "a forbidden string in a regular file must still fail the step; got status {}\nstdout: {}\nstderr: {}",
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
    assert!(
        result
            .stdout
            .contains("Left out 1 of 2 changed entries the scanner cannot open as a file"),
        "the failing run must still say what it left out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn changed_link_to_a_file_still_reads_its_target_content()`
//           pushes a new tracked link to a regular file whose second line holds
//           the forbidden literal, and expects the finding.
// Why:      Only folders leave the list; a link to a file is content the
//           scanner reads through today, and that inspection must survive.
#[test]
fn changed_link_to_a_file_still_reads_its_target_content() {
    let mut fixture = Fixture::new("link-to-file-read-through");
    fixture.write_workflow_revision(true);
    fixture.put(
        "base/lib.txt",
        &format!("clean line\n{}\n", SENTINEL),
    );
    fixture.put("consumer/main.txt", "clean\n");
    fixture.commit("A");
    // The link resolves to the sentinel-bearing file; `fs::read` on it follows
    // the link, so the finding names the link's own path.
    fixture.link("consumer/uses/lib", "../../base/lib.txt");
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
        "a forbidden string reached through a link to a file must fail; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stderr
            .contains(&format!("consumer/uses/lib:2 rule={}", FIXTURE_RULE)),
        "the finding must name the link and the target's line\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        !result.stdout.contains("Left out"),
        "a link to a file is not left out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("Scanning 1 changed file(s)"),
        "the link must reach the scanner as its only changed entry\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn broken_link_still_ends_the_step_with_its_read_error()`
//           pushes a new tracked link whose target does not exist, and expects
//           the scanner's read error and exit 1.
// Why:      A gate must not pass silently on a file it could not inspect, and
//           a broken link is not a folder, so the step keeps handing it over.
#[test]
fn broken_link_still_ends_the_step_with_its_read_error() {
    let mut fixture = Fixture::new("broken-link-read-error");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.commit("A");
    // `../../nowhere` names no tracked or untracked path, so the link is
    // broken on disk exactly as a checked-out broken link is.
    fixture.link("consumer/uses/gone", "../../nowhere");
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
        "a broken link must still fail the step; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stderr
            .contains("consumer/uses/gone: read error: No such file or directory"),
        "the step must report the broken link's read error\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        !result.stdout.contains("Left out"),
        "a broken link is not left out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn push_of_folder_links_only_ends_green_and_says_what_it_left_out()`
//           pushes two new folder links and nothing else, and expects exit 0
//           after the step says it left both out.
// Why:      A delta made only of such links has nothing the scanner can read;
//           the step must not call the scanner with zero paths and must not
//           report a scan it did not do.
#[test]
fn push_of_folder_links_only_ends_green_and_says_what_it_left_out() {
    let mut fixture = Fixture::new("folder-links-only-push");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.put("other/lib.txt", "clean\n");
    fixture.commit("A");
    fixture.link("consumer/uses/base", "../../base");
    fixture.link("consumer/uses/other", "../../other");
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
        "a delta of folder links alone must pass; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stdout
            .contains("Left out 2 of 2 changed entries the scanner cannot open as a file"),
        "the step must say it left out every changed entry\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("folder links 2, submodule gitlinks 0"),
        "the step must count both folder links\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        !result.stdout.contains("Scanning")
            && result.stdout.contains("nothing to scan")
            && result.stderr.is_empty(),
        "no scanner run may be announced when every entry was left out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn push_of_a_submodule_gitlink_ends_green_and_says_what_it_left_out()`
//           pushes one new submodule entry whose folder is not checked out,
//           and expects exit 0 after the step says it left it out.
// Why:      A submodule entry is the other tree entry a content scan cannot
//           open as a file; the walk already leaves it out, so the step must
//           match the walk instead of reporting a read error.
#[test]
fn push_of_a_submodule_gitlink_ends_green_and_says_what_it_left_out() {
    let mut fixture = Fixture::new("submodule-gitlink-push");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.commit("A");
    // The gitlink records commit A at `vendor/sub`; no folder is checked out
    // there, so only the tree mode tells the step what the entry is.
    fixture.gitlink("vendor/sub", "A");
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
        result.status == 0,
        "a submodule entry must not fail the step; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stdout
            .contains("Left out 1 of 1 changed entries the scanner cannot open as a file"),
        "the step must say it left the submodule entry out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("folder links 0, submodule gitlinks 1"),
        "the step must count the entry as a submodule gitlink\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn pull_request_scan_leaves_a_folder_link_out_too()`
//           runs the step for a pull-request event whose branch changes one
//           regular file and adds one folder link, and expects exit 0.
// Why:      The step builds its list in two places; the merge-base arm must
//           apply the same entry rule as the push arm.
#[test]
fn pull_request_scan_leaves_a_folder_link_out_too() {
    let mut fixture = Fixture::new("pull-request-folder-link");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.put("consumer/main.txt", "clean\n");
    fixture.commit("A");
    fixture.git(&["switch", "--create", "feature", &fixture.sha("A")]);
    fixture.put("consumer/main.txt", "clean\nchanged by the pull request\n");
    fixture.link("consumer/uses/base", "../../base");
    fixture.commit("P");
    fixture.git(&["switch", "main"]);
    fixture.put("other.txt", "clean\nchanged on main after the fork\n");
    fixture.commit("M");
    fixture
        .git(&["update-ref", "refs/remotes/origin/main", &fixture.sha("M")]);
    fixture.git(&["switch", "feature"]);
    fixture.write_runs(&[]);

    let result = fixture.run_scan("pull_request", "P", "feature", "");

    assert!(
        result.status == 0,
        "a pull request adding a folder link must pass; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stdout
            .contains("Left out 1 of 2 changed entries the scanner cannot open as a file"),
        "the pull-request arm must say it left the folder link out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("origin/main...HEAD")
            && result.stdout.contains("consumer/main.txt")
            && !result.stdout.contains("consumer/uses/base"),
        "the pull-request arm must keep its merge-base scope while dropping the link\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn pull_request_scan_still_reports_a_forbidden_string_beside_a_folder_link()`
//           runs the pull-request arm with the forbidden literal in a regular
//           file changed beside the folder link, and expects exit 1.
// Why:      The pull-request arm's control: dropping the link must not weaken
//           the gate for the branch's own files.
#[test]
fn pull_request_scan_still_reports_a_forbidden_string_beside_a_folder_link() {
    let mut fixture = Fixture::new("pull-request-folder-link-violation");
    fixture.write_workflow_revision(true);
    fixture.put("base/lib.txt", "clean\n");
    fixture.put("consumer/main.txt", "clean\n");
    fixture.commit("A");
    fixture.git(&["switch", "--create", "feature", &fixture.sha("A")]);
    fixture.put(
        "consumer/main.txt",
        &format!("clean\n{}\n", SENTINEL),
    );
    fixture.link("consumer/uses/base", "../../base");
    fixture.commit("P");
    fixture.git(&["switch", "main"]);
    fixture.put("other.txt", "clean\nchanged on main after the fork\n");
    fixture.commit("M");
    fixture
        .git(&["update-ref", "refs/remotes/origin/main", &fixture.sha("M")]);
    fixture.git(&["switch", "feature"]);
    fixture.write_runs(&[]);

    let result = fixture.run_scan("pull_request", "P", "feature", "");

    assert!(
        result.status == 1,
        "the pull-request arm must report the forbidden string; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stderr
            .contains(&format!("consumer/main.txt:2 rule={}", FIXTURE_RULE)),
        "the finding must name the branch's regular file\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stdout
            .contains("Left out 1 of 2 changed entries the scanner cannot open as a file"),
        "the failing pull-request run must still say what it left out\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}
