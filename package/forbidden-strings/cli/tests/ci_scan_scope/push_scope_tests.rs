//! What:
//!  The push-run scope,
//!  case by case:
//!  a canceled run's unvalidated change is caught by the next successful run,
//!  a successful head becomes the next baseline,
//!  a run from before the scope marker is not a validated tree,
//!  no run history at all falls back to the whole tree,
//!  and a head outside the checked-out ancestry is refused as a baseline.
//! Why:
//!  Push scopes are where issue #993 lived:
//!  a canceled run left a changed file unscanned forever,
//!  so every fallback here is the difference between coverage and a silent
//!  gap.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // push-scope.test.ts: baseline selection for push events
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

// What:     `#[test] fn surviving_push_scan_covers_changes_the_canceled_run_never_validated()`
//           models the issue #993 scenario end to end: push A to B plants a
//           forbidden literal in `x.txt` and its run is canceled before
//           scanning it; push B to C touches only `y.txt`; the surviving run
//           must still detect the literal in `x.txt`.
// Why:      This is the acceptance regression test. Pre-fix the surviving run
//           scanned only `B..C`, so `x.txt` stayed unscanned until someone
//           touched it again.
#[test]
fn surviving_push_scan_covers_changes_the_canceled_run_never_validated() {
    let mut fixture = Fixture::new("canceled-prior-push");
    fixture.write_workflow_revision(true);
    fixture.put("x.txt", "clean before the canceled push\n");
    fixture.put("y.txt", "clean before the canceled push\n");
    fixture.commit("A");
    fixture.put("x.txt", &format!("clean\n{}\n", SENTINEL));
    fixture.commit("B");
    fixture.put("y.txt", "clean\nchanged by the surviving push\n");
    fixture.commit("C");
    // What:     The run list is newest first and holds three near-miss rows
    //           at head B alongside the one real baseline at head A.
    // Why:      A canceled run never proved its head, a run of another
    //           workflow never proved this one, and a pull-request run only
    //           proved its own PR diff; each near-miss becomes the chosen
    //           baseline if the body drops the matching filter, and then
    //           `x.txt` is missed again.
    fixture.write_runs(&[
        RunRow {
            conclusion: "cancelled",
            event: "push",
            head_branch: "main",
            workflow: "forbidden-strings.yml",
            head_label: "B",
        },
        RunRow {
            conclusion: "success",
            event: "push",
            head_branch: "main",
            workflow: "other-gate.yml",
            head_label: "B",
        },
        RunRow {
            conclusion: "success",
            event: "pull_request",
            head_branch: "main",
            workflow: "forbidden-strings.yml",
            head_label: "B",
        },
        RunRow {
            conclusion: "success",
            event: "push",
            head_branch: "main",
            workflow: "forbidden-strings.yml",
            head_label: "A",
        },
    ]);

    let result = fixture.run_scan("push", "C", "main", "B");

    assert!(
        result.status == 1,
        "the surviving scan must report the planted violation; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stderr.contains("x.txt") && result.stderr.contains(FIXTURE_RULE),
        "findings must name the file and rule left behind by the canceled push\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("x.txt") && result.stdout.contains("y.txt"),
        "the surviving run must scan both the canceled push's file and its own\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result
            .stdout
            .contains(&format!("{}..{}", fixture.sha("A"), fixture.sha("C"))),
        "the scan range must start at the last validated commit A\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn successful_push_run_head_becomes_the_next_scan_baseline()`
//           runs a clean push to C against a baseline at B, then a clean push
//           to D against a baseline at C.
// Why:      Coverage must not regress into wide rescans: a successful run
//           proves its head tree, so the next run scans only the delta from
//           that head, and the newest validated head wins over older ones.
#[test]
fn successful_push_run_head_becomes_the_next_scan_baseline() {
    let mut fixture = Fixture::new("baseline-advance");
    fixture.write_workflow_revision(true);
    fixture.put("x.txt", "clean\n");
    fixture.put("y.txt", "clean\n");
    fixture.put("z.txt", "clean\n");
    fixture.commit("A");
    fixture.put("y.txt", "clean\nchanged by the push to B\n");
    fixture.commit("B");
    fixture.put("z.txt", "clean\nchanged by the push to C\n");
    fixture.commit("C");
    fixture.write_runs(&[
        RunRow {
            conclusion: "success",
            event: "push",
            head_branch: "main",
            workflow: "forbidden-strings.yml",
            head_label: "B",
        },
        RunRow {
            conclusion: "success",
            event: "push",
            head_branch: "main",
            workflow: "forbidden-strings.yml",
            head_label: "A",
        },
    ]);

    let first = fixture.run_scan("push", "C", "main", "B");

    assert!(
        first.status == 0,
        "a clean delta scan must pass; got status {}\nstdout: {}\nstderr: {}",
        first.status,
        first.stdout,
        first.stderr,
    );
    assert!(
        first.stdout.contains("z.txt")
            && !first.stdout.contains("x.txt")
            && !first.stdout.contains("y.txt"),
        "the run at C must scan only the delta from validated head B\nstdout: {}\nstderr: {}",
        first.stdout,
        first.stderr,
    );

    fixture.put("x.txt", "clean\nchanged by the push to D\n");
    fixture.commit("D");
    fixture.write_runs(&[
        RunRow {
            conclusion: "success",
            event: "push",
            head_branch: "main",
            workflow: "forbidden-strings.yml",
            head_label: "C",
        },
        RunRow {
            conclusion: "success",
            event: "push",
            head_branch: "main",
            workflow: "forbidden-strings.yml",
            head_label: "B",
        },
        RunRow {
            conclusion: "success",
            event: "push",
            head_branch: "main",
            workflow: "forbidden-strings.yml",
            head_label: "A",
        },
    ]);

    let second = fixture.run_scan("push", "D", "main", "C");

    assert!(
        second.status == 0,
        "a clean delta scan must pass; got status {}\nstdout: {}\nstderr: {}",
        second.status,
        second.stdout,
        second.stderr,
    );
    assert!(
        second.stdout.contains("x.txt")
            && !second.stdout.contains("y.txt")
            && !second.stdout.contains("z.txt"),
        "the run at D must scan only the delta from the nearer validated head C\nstdout: {}\nstderr: {}",
        second.stdout,
        second.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn run_from_before_the_scope_marker_is_not_a_validated_baseline()`
//           offers one successful run whose committed workflow revision lacks
//           the scope marker and expects the surviving run to scan the whole
//           tree instead of trusting it.
// Why:      Pre-marker runs scanned only their own push range, so their head
//          is not a validated tree; trusting one would hide exactly the
//           changes issue #993 is about.
#[test]
fn run_from_before_the_scope_marker_is_not_a_validated_baseline() {
    let mut fixture = Fixture::new("pre-marker-run");
    fixture.write_workflow_revision(false);
    fixture.put("x.txt", "clean\n");
    fixture.put("y.txt", "clean\n");
    fixture.commit("A");
    fixture.put("x.txt", &format!("clean\n{}\n", SENTINEL));
    fixture.put("y.txt", "clean\nchanged by the surviving push\n");
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
        "the fallback scan must report the planted violation; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stderr.contains("x.txt") && result.stderr.contains(FIXTURE_RULE),
        "the fallback scan must detect the violation a pre-marker baseline would hide\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("scanning the current tree")
            && !result.stdout.contains("changed file(s)"),
        "an unsuitable baseline must fall back to the whole tree, not to a delta scan\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn missing_run_history_falls_back_to_a_full_tree_scan()`
//           offers no listed runs at all and expects the surviving run to scan
//           the whole tree.
// Why:      The first marker-era run, and any run whose listing is
//           unavailable, has no validated baseline to trust; the current tree
//           is the only coverage-safe scope left.
#[test]
fn missing_run_history_falls_back_to_a_full_tree_scan() {
    let mut fixture = Fixture::new("no-run-history");
    fixture.write_workflow_revision(true);
    fixture.put("x.txt", "clean\n");
    fixture.put("y.txt", "clean\n");
    fixture.commit("A");
    fixture.put("x.txt", &format!("clean\n{}\n", SENTINEL));
    fixture.put("y.txt", "clean\nchanged by the surviving push\n");
    fixture.commit("B");
    fixture.write_runs(&[]);

    let result = fixture.run_scan("push", "B", "main", "A");

    assert!(
        result.status == 1,
        "the fallback scan must report the planted violation; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("scanning the current tree")
            && !result.stdout.contains("changed file(s)"),
        "with no baseline at all the run must scan the whole tree\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

// What:     `#[test] fn non_ancestor_run_head_is_not_a_validated_baseline()`
//           offers one successful run whose head is a commit that the checked
//           out history does not descend from, and expects a whole-tree scan.
// Why:      After a rewritten main the old heads are gone from the ancestry
//           of the new head, so the run must not diff against them; the
//           whole tree keeps coverage safe across rewrites.
#[test]
fn non_ancestor_run_head_is_not_a_validated_baseline() {
    let mut fixture = Fixture::new("non-ancestor-run");
    fixture.write_workflow_revision(true);
    fixture.put("x.txt", "clean\n");
    fixture.commit("A");
    fixture.put("x.txt", &format!("clean\n{}\n", SENTINEL));
    fixture.commit("B");
    fixture.git(&["switch", "--create", "side", &fixture.sha("A")]);
    fixture.put("y.txt", "clean\nchanged on the side history\n");
    fixture.commit("S");
    fixture.git(&["switch", "main"]);
    fixture.write_runs(&[RunRow {
        conclusion: "success",
        event: "push",
        head_branch: "main",
        workflow: "forbidden-strings.yml",
        head_label: "S",
    }]);

    let result = fixture.run_scan("push", "B", "main", "A");

    assert!(
        result.status == 1,
        "the fallback scan must report the planted violation; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("scanning the current tree")
            && !result.stdout.contains("changed file(s)"),
        "a baseline outside the checked-out ancestry must fall back to the whole tree\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}
