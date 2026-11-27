//! What:
//!  The pull-request scope,
//!  kept as it was:
//!  a pull request scans its merge-base range and nothing else,
//!  whatever main did after the branch forked.
//! Why:
//!  That range is already cumulative per branch and survives a canceled run,
//!  so the fix that gave push runs validated baselines must leave this path
//!  untouched;
//!  widening it would rescan the whole tree on every pull request.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // pull-request-scope.test.ts: origin/main...HEAD only
//! ```

use std::fs;

use crate::Fixture;

// What:     `#[test] fn pull_request_scan_keeps_the_merge_base_scope()` runs
//           the body for a pull-request event after main moved on, and
//           expects only the pull request's own changed files.
// Why:      Pull-request and merge-group ranges are cumulative per branch, so
//           they already survive cancellation; the fix must leave that path
//           untouched while push runs switch to validated baselines.
#[test]
fn pull_request_scan_keeps_the_merge_base_scope() {
    let mut fixture = Fixture::new("pull-request-scope");
    fixture.write_workflow_revision(true);
    fixture.put("x.txt", "clean\n");
    fixture.put("z.txt", "clean\n");
    fixture.commit("A");
    fixture.git(&["switch", "--create", "feature", &fixture.sha("A")]);
    fixture.put("x.txt", "clean\nchanged by the pull request\n");
    fixture.commit("P");
    fixture.git(&["switch", "main"]);
    fixture.put("z.txt", "clean\nchanged on main after the fork\n");
    fixture.commit("M");
    fixture.git(&["update-ref", "refs/remotes/origin/main", &fixture.sha("M")]);
    fixture.git(&["switch", "feature"]);
    fixture.write_runs(&[]);

    let result = fixture.run_scan("pull_request", "P", "feature", "");

    assert!(
        result.status == 0,
        "the pull request touches no forbidden literal; got status {}\nstdout: {}\nstderr: {}",
        result.status,
        result.stdout,
        result.stderr,
    );
    assert!(
        result.stdout.contains("x.txt")
            && !result.stdout.contains("z.txt")
            && result.stdout.contains("origin/main...HEAD"),
        "the pull-request path must keep its merge-base scope\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}
