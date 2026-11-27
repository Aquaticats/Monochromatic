//! What:
//!  Integration tests for the scan-scope selection that
//!  `.github/workflows/forbidden-strings.yml` performs on CI.
//!  Each test
//!  extracts the exact `run` body of the workflow's "Scan changed files"
//!  step and executes it with `bash` against a disposable Git repository,
//!  a fake `gh run list`,
//!  and the real scanner binary.
//! Why:
//!  Issue #993:
//!  a push run canceled before scanning a changed file left
//!  that file unscanned forever,
//!  because the push scope was the event's own
//!  `before..after` range and the next run only scanned its own range.
//!  The
//!  workflow now scopes from the last validated tree;
//!  these tests pin that
//!  behavior against the workflow text itself,
//!  so no second hand-maintained
//!  copy of the shell logic can drift from what CI runs.
//!
//!  The cases sit in the role-named modules of this folder;
//!  this file holds the fixture the cases drive,
//!  then lists the parts in the order they build on it.
//!  Each part needs `#[path]` because a crate root always looks for its
//!  modules beside itself,
//!  in `tests/`,
//!  not in this folder.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command as ProcessCommand;

use crate::fixture_inputs::{
    FIXTURE_RULES,
    GH_RUN_LIST_STUB,
    GIT,
    MARKER_FREE_WORKFLOW,
};
use crate::scan_step_command::scan_step_body;

// What:     `struct RunRow` is one workflow run listed by the stub.
// Why:      The candidate baseline must come from a run that proved its head
//           tree: a successful push run of this workflow. Each field exists so
//           a test can plant a near-miss row that only leaks into the
//           candidate list when the workflow body drops the matching filter.
struct RunRow {
    conclusion: &'static str,
    event: &'static str,
    head_branch: &'static str,
    workflow: &'static str,
    head_label: &'static str,
}

// What:     `struct ScanResult` is one captured scan-step execution.
// Why:      Assertions need the exit status together with both streams; the
//           scanner reports findings on stderr and the step's own narration
//           on stdout.
struct ScanResult {
    status: i32,
    stdout: String,
    stderr: String,
}

// What:     `struct Fixture` owns a disposable repository, stub PATH
//           directory, runner temp directory, and run-list fixture.
// Why:      The scan step is a real shell program touching Git state,
//           `$RUNNER_TEMP`, `gh`, and `forbidden-strings`; one owning struct
//           keeps every test's environment disjoint and removable.
struct Fixture {
    root: PathBuf,
    repo: PathBuf,
    stub_bin: PathBuf,
    runner_temp: PathBuf,
    runs_tsv: PathBuf,
    cache_dir: PathBuf,
    shas: Vec<(String, String)>,
}

impl Fixture {
    // What:     `fn new(label)` creates the fixture tree and installs the
    //           stubs, the extracted scan script, and the fixture rule file.
    // Why:      Every test needs identical plumbing; constructing it in one
    //           place keeps the tests about scope selection, not setup.
    fn new(label: &str) -> Fixture {
        let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "forbidden-strings-ci-scan-{}-{}",
            label,
            std::process::id(),
        ));
        let _ = fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let stub_bin = root.join("bin");
        let runner_temp = root.join("runner-temp");
        let cache_dir = root.join("cache");
        fs::create_dir_all(&repo).expect("create fixture repo dir");
        fs::create_dir_all(&stub_bin).expect("create stub bin dir");
        fs::create_dir_all(&runner_temp).expect("create runner temp dir");
        fs::create_dir_all(&cache_dir).expect("create scanner cache dir");
        write_executable(&stub_bin.join("gh"), GH_RUN_LIST_STUB);
        write_executable(
            &stub_bin.join("forbidden-strings"),
            &format!(
                "#!/bin/sh\nexec \"{}\" \"$@\"\n",
                env!("CARGO_BIN_EXE_forbidden-strings"),
            ),
        );
        write_executable(
            &stub_bin.join("git"),
            "#!/bin/sh\nexec /usr/bin/git \"$@\"\n",
        );
        fs::write(
            runner_temp.join("forbidden-strings.rules.txt"),
            FIXTURE_RULES,
        )
        .expect("write fixture rules file");
        fs::write(root.join("scan.sh"), scan_step_body()).expect("write extracted scan body");
        let runs_tsv = root.join("runs.tsv");
        let fixture = Fixture {
            root,
            repo,
            stub_bin,
            runner_temp,
            runs_tsv,
            cache_dir,
            shas: Vec::new(),
        };
        fixture.git(&["init", "--initial-branch=main"]);
        return fixture;
    }

    // What:     `fn git(&self, args)` runs one system Git command inside the
    //           fixture repository and returns its trimmed stdout.
    // Why:      Fixture history must behave like the CI checkout: a plain
    //           repository with the workflow's commits, no policy wrapper
    //           hooks, and fixed identity so commit hashes stay stable.
    fn git(
        &self,
        args: &[&str],
    ) -> String {
        let mut command = ProcessCommand::new(GIT);
        command.current_dir(&self.repo);
        command.args([
            "-c",
            "user.name=forbidden-strings-fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ]);
        command.args(args);
        let output = command.output().expect("spawn git");
        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr),
            );
        }
        return String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string();
    }

    // What:     `fn put(&self, relative, content)` writes one repository file.
    // Why:      Tests describe history as file contents per commit; creating
    //           parent directories here lets cases add nested paths without
    //           extra setup.
    fn put(
        &self,
        relative: &str,
        content: &str,
    ) {
        let path = self.repo.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create file parent dirs");
        }
        fs::write(&path, content).expect("write fixture file");
    }

    // What:     `fn write_workflow_revision(&self, marked)` installs the
    //           workflow file at the current head, with or without the
    //           scope-semantics marker.
    // Why:      The scan step rejects runs whose committed workflow revision
    //           lacks the marker, because those runs scanned only their own
    //           push range; the fixture must be able to present both kinds of
    //           history.
    fn write_workflow_revision(
        &self,
        marked: bool,
    ) {
        if marked {
            let workflow_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.github/workflows/forbidden-strings.yml");
            let workflow = fs::read_to_string(&workflow_path).expect("read real workflow file");
            self.put(".github/workflows/forbidden-strings.yml", &workflow);
        } else {
            self.put(
                ".github/workflows/forbidden-strings.yml",
                MARKER_FREE_WORKFLOW,
            );
        }
    }

    // What:     `fn commit(&mut self, label)` stages the tree, commits it,
    //           and records the commit under `label`.
    // Why:      Cases reference history positions by label when planting run
    //           rows and choosing the checked-out head.
    fn commit(
        &mut self,
        label: &str,
    ) -> String {
        self.git(&["add", "--all"]);
        self.git(&["commit", "--message", label]);
        let sha = self.git(&["rev-parse", "HEAD"]);
        self.shas.push((label.to_string(), sha.clone()));
        return sha;
    }

    // What:     `fn sha(&self, label)` resolves a recorded commit label.
    // Why:      Run rows and the scan environment need real commit ids; a
    //      missing label is a test authoring error and must fail loudly.
    fn sha(
        &self,
        label: &str,
    ) -> String {
        for (recorded_label, sha) in &self.shas {
            if recorded_label == label {
                return sha.clone();
            }
        }
        panic!("fixture has no commit labeled {}", label);
    }

    // What:     `fn write_runs(&self, rows)` writes the stub's run-list
    //           fixture, newest first, with resolved head commit ids.
    // Why:      The candidate baseline comes from listed runs; planting the
    //           rows here lets each case model exactly which runs GitHub
    //           would report for its history.
    fn write_runs(
        &self,
        rows: &[RunRow],
    ) {
        let mut text = String::new();
        for row in rows {
            text.push_str(row.conclusion);
            text.push('\t');
            text.push_str(row.event);
            text.push('\t');
            text.push_str(row.head_branch);
            text.push('\t');
            text.push_str(row.workflow);
            text.push('\t');
            text.push_str(&self.sha(row.head_label));
            text.push('\n');
        }
        fs::write(&self.runs_tsv, text).expect("write run-list fixture");
    }

    // What:     `fn run_scan(&self, event, head_label, branch, before_label)`
    //           executes the extracted scan body with the environment the
    //           workflow step receives, including the push event's `before`
    //           commit when the event carries one.
    // Why:      The contract under test is the workflow's own shell behavior,
    //           so the test runs that body unmodified and captures its
    //           streams and exit status.
    fn run_scan(
        &self,
        event: &str,
        head_label: &str,
        branch: &str,
        before_label: &str,
    ) -> ScanResult {
        let mut before_sha = String::new();
        if !before_label.is_empty() {
            before_sha = self.sha(before_label);
        }
        let mut command = ProcessCommand::new("bash");
        command.current_dir(&self.repo);
        command.arg(self.root.join("scan.sh"));
        command.env("EVENT_NAME", event);
        command.env("HEAD_SHA", self.sha(head_label));
        command.env("PUSH_BASE_SHA", before_sha);
        command.env("BRANCH", branch);
        command.env("REPO", "fixture/fixture");
        command.env("GH_TOKEN", "fixture-token");
        command.env("RUNNER_TEMP", &self.runner_temp);
        command.env("GH_RUN_FIXTURE", &self.runs_tsv);
        command.env("FORBIDDEN_STRINGS_CACHE_DIR", &self.cache_dir);
        command.env(
            "PATH",
            format!(
                "{}:{}",
                self.stub_bin.display(),
                std::env::var("PATH").unwrap_or_default(),
            ),
        );
        let output = command.output().expect("spawn extracted scan body");
        return ScanResult {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        };
    }
}

// What:     `fn write_executable(path, body)` writes a shell file with the
//           executable bit set.
// Why:      The extracted body invokes `gh`, `git`, and `forbidden-strings`
//           by name, so the fixture must present them on `PATH` as
//           executables.
fn write_executable(
    path: &Path,
    body: &str,
) {
    fs::write(path, body).expect("write stub script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod stub script");
}
/// What:
///  `#[path = "ci_scan_scope/fixture_inputs.rs"] mod fixture_inputs;` registers
///  the text and shell every fixture installs.
/// Why:
///  Each constant exists so a near-miss baseline leaks into the candidate list
///  when the workflow body drops the filter that excludes it.
#[path = "ci_scan_scope/fixture_inputs.rs"]
mod fixture_inputs;
/// What:
///  `#[path = "ci_scan_scope/pull_request_scope_tests.rs"]
///  mod pull_request_scope_tests;` registers the case that keeps the
///  pull-request merge-base scope as it was.
/// Why:
///  The fix must not widen a path that already survives cancellation.
#[path = "ci_scan_scope/pull_request_scope_tests.rs"]
mod pull_request_scope_tests;
/// What:
///  `#[path = "ci_scan_scope/push_scope_tests.rs"] mod push_scope_tests;`
///  registers the five push-run baseline cases.
/// Why:
///  Push scopes are where issue #993 lived.
#[path = "ci_scan_scope/push_scope_tests.rs"]
mod push_scope_tests;
/// What:
///  `#[path = "ci_scan_scope/scan_step_command.rs"] mod scan_step_command;`
///  registers the extraction of the workflow's own scan-step shell body.
/// Why:
///  The tests must run what CI runs,
///  not a copy of it.
#[path = "ci_scan_scope/scan_step_command.rs"]
mod scan_step_command;
