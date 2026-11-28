//! What:
//!  Integration tests for the scan-scope selection that
//!  `.github/workflows/forbidden-strings.yml` performs on CI.
//!  Each test
//!  extracts the exact `run` bodies of the workflow's steps from
//!  "List changed files" to the step that scans links and gitlinks,
//!  and executes them with `bash` in workflow order,
//!  against a disposable Git repository,
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
use crate::scan_step_command::{
    LINK_LIST_STEP_NAME,
    LINK_SCAN_STEP_NAME,
    SCAN_STEP_NAME,
    SCOPE_STEP_NAME,
    declared_step_body,
    step_body,
    step_runs_after_failure,
};

// What:     `const SCOPE_SCRIPT` is the fixture file holding the extracted
//           scope-step body.
// Why:      The fixture writes each extracted body once and runs it by name,
//           so a failing case points at the step that failed.
const SCOPE_SCRIPT: &str = "list-changed-files.sh";

// What:     `const SCAN_SCRIPT` is the fixture file holding the extracted
//           file-scan step body.
// Why:      Same reason as `SCOPE_SCRIPT`; this is the body that filters the
//           changed entries.
const SCAN_SCRIPT: &str = "scan-changed-files.sh";

// What:     `const LINK_LIST_SCRIPT` is the fixture file holding the extracted
//           body of the step that writes each link's stored target text and
//           each gitlink's name for the next body.
// Why:      Same reason as `SCOPE_SCRIPT`; this is the body issue #6640 adds,
//           and it is written only when the workflow declares it, so a
//           workflow without it runs its other steps and reports what they
//           hand over.
const LINK_LIST_SCRIPT: &str = "list-links.sh";

// What:     `const LINK_SCAN_SCRIPT` is the fixture file holding the extracted
//           body of the step that scans links and gitlinks under their own
//           names.
// Why:      This body is written only when the workflow declares it, so a
//           workflow without it runs its other steps and reports what they
//           hand over.
const LINK_SCAN_SCRIPT: &str = "scan-links.sh";

// What:     `const STEPS` is the workflow order of the bodies the fixture runs:
//           the scope selection,
//           the file scan,
//           the step that writes each link's target text,
//           and the step that scans links and gitlinks,
//           each with whether the workflow must declare it.
// Why:      The runner runs a job's steps in the workflow's order, stops at the
//           first one that fails unless a later step declares a status check,
//           and cancels the rest when the run is cancelled;
//           one ordered list keeps the fixture on that contract when a step
//           joins the workflow.
//           The two older bodies are required,
//           so a workflow that lost one fails the harness loudly;
//           the two newer ones are optional,
//           so a case about them reports the behaviour it found instead of a
//           harness panic.
//           Whether a step runs after a failure is read from the workflow's
//           own `if:` line, never assumed here.
const STEPS: [(&str, &str, bool); 4] = [
    (SCOPE_STEP_NAME, SCOPE_SCRIPT, true),
    (SCAN_STEP_NAME, SCAN_SCRIPT, true),
    (LINK_LIST_STEP_NAME, LINK_LIST_SCRIPT, false),
    (LINK_SCAN_STEP_NAME, LINK_SCAN_SCRIPT, false),
];

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
    steps: Vec<(&'static str, bool, bool)>,
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
        // What:     `steps` is, in workflow order, each body the workflow
        //           declares, with the fixture script that holds it and
        //           whether it runs after a failed step.
        // Why:      The fixture runs every declared body in order; a workflow
        //           without one of the later bodies still runs its other
        //           bodies, so a case about a missing step reports the
        //           behaviour it found instead of a harness panic.
        let mut steps: Vec<(&'static str, bool, bool)> = Vec::new();
        for (name, script, required) in STEPS {
            // What:     `after_failure` is the workflow's own answer to whether
            //           this step runs after an earlier step failed.
            // Why:      Assuming it here would let a plant that removed the
            //           status check still pass every case.
            let after_failure: bool = step_runs_after_failure(name);
            // What:     `body` is the step's shell, or `None` when the step is
            //           optional and the workflow does not declare it.
            // Why:      `step_body` panics on a missing step, and `declared_step_body`
            //           answers `None`; each kind of step gets the one its
            //           requirement asks for.
            let body: Option<String> = if required {
                Some(step_body(name))
            } else {
                declared_step_body(name)
            };
            match body {
                Some(body) => {
                    fs::write(root.join(script), body).expect("write extracted step body");
                    steps.push((script, true, after_failure));
                }
                None => steps.push((script, false, after_failure)),
            }
        }
        let runs_tsv = root.join("runs.tsv");
        let fixture = Fixture {
            root,
            repo,
            stub_bin,
            runner_temp,
            runs_tsv,
            cache_dir,
            steps,
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

    // What:     `fn link(&self, relative, target)` creates one symbolic link
    //           in the fixture repository.
    // Why:      Issue #6632: a tracked link to a folder is the entry the scan
    //           step must leave out; only a real symlink on disk reproduces
    //           what a CI checkout materializes.
    fn link(
        &self,
        relative: &str,
        target: &str,
    ) {
        let path = self.repo.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create link parent dirs");
        }
        // `std::os::unix::fs::symlink(target, path)` creates a link whose
        // stored text is `target`, resolved against the link's own folder.
        // The sibling `symlink_dir` is Windows-only and does the same thing.
        std::os::unix::fs::symlink(target, &path).expect("create fixture symlink");
    }

    // What:     `fn gitlink(&mut self, relative, target_label)` records a
    //           submodule entry at `relative`, pointing at an existing
    //           fixture commit.
    // Why:      A submodule entry is the other tree entry a content scan
    //           cannot open as a file; the index is the only way to plant one
    //           without a real submodule checkout.
    fn gitlink(
        &mut self,
        relative: &str,
        target_label: &str,
    ) {
        let target_sha = self.sha(target_label);
        // `160000,<sha>,<path>` is Git's `--cacheinfo` argument shape: the
        // tree mode, the recorded object, and the path, comma-separated.
        let cacheinfo = format!("160000,{},{}", target_sha, relative);
        self.git(&["update-index", "--add", "--cacheinfo", &cacheinfo]);
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
        return self.commit_index(label);
    }

    // What:     `fn commit_index(&mut self, label)` commits exactly what the
    //           index holds and records the commit under `label`.
    // Why:      `git add --all` cannot stage a submodule entry whose folder is
    //           absent, so the gitlink case plants it in the index first and
    //           then commits here without restaging the tree.
    fn commit_index(
        &mut self,
        label: &str,
    ) -> String {
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
    //           executes every extracted step body the workflow declares, in
    //           workflow order, and stops at the first body that fails unless a
    //           later body runs after a failure, as the runner does.
    // Why:      The contract under test is the workflow's own shell behavior,
    //           so the test runs those bodies unmodified and captures their
    //           joined streams and the exit status of the step that failed
    //           first. Each step gets a fresh `bash`, as the runner gives it.
    fn run_scan(
        &self,
        event: &str,
        head_label: &str,
        branch: &str,
        before_label: &str,
    ) -> ScanResult {
        // A CI job starts with an empty `RUNNER_TEMP`; clearing the state the
        // steps pass to each other keeps a second call in one fixture from
        // reading a previous call's decision.
        for state_file in [
            "forbidden-strings.comparison",
            "forbidden-strings.changed-raw",
            "forbidden-strings.whole-tree",
            "forbidden-strings.left-out-raw",
            "forbidden-strings.link-and-gitlink-raw",
        ] {
            let _ = fs::remove_file(self.runner_temp.join(state_file));
        }
        let mut stdout = String::new();
        let mut stderr = String::new();
        // What:     `failed_status` is the exit status of the first body that
        //           failed, or 0 while every body so far passed.
        // Why:      A body that runs after a failure still runs, and the job
        //           keeps the status of what failed first.
        let mut failed_status: i32 = 0;
        for (script, declared, after_failure) in &self.steps {
            if !declared {
                continue;
            }
            if failed_status != 0 && !after_failure {
                continue;
            }
            let step = self.run_step(script, event, head_label, branch, before_label);
            stdout.push_str(&step.stdout);
            stderr.push_str(&step.stderr);
            if step.status != 0 && failed_status == 0 {
                failed_status = step.status;
            }
        }
        return ScanResult {
            status: failed_status,
            stdout,
            stderr,
        };
    }

    // What:     `fn run_step(&self, script, ...)` runs one extracted step body
    //           and captures its streams and exit status.
    // Why:      Both steps need the same environment and PATH stubs, so the
    //           setup lives once and the pair runs in workflow order.
    fn run_step(
        &self,
        script: &str,
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
        command.arg(self.root.join(script));
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
        let output = command.output().expect("spawn extracted step body");
        return ScanResult {
            status: output.status.code().unwrap_or(-1),
            // `to_string()` allocates an owned copy (the sibling `&str` is a
            // borrow of `output`, which goes out of scope) so the result can
            // outlive the child process.
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
///  `#[path = "ci_scan_scope/folder_link_entry_tests.rs"]
///  mod folder_link_entry_tests;` registers the cases for changed entries that
///  are tree entries a content scan cannot open as a file.
/// Why:
///  Issue #6632:
///  every push since 2026-11-20 was red only because the step
///  handed tracked links to folders to the scanner.
#[path = "ci_scan_scope/folder_link_entry_tests.rs"]
mod folder_link_entry_tests;
/// What:
///  `#[path = "ci_scan_scope/link_target_text_tests.rs"]
///  mod link_target_text_tests;` registers the cases for what the steps hand the
///  scanner for every link and gitlink:
///  the entry's own name and,
///  for a link,
///  the target text Git stores for it.
/// Why:
///  Issue #6633:
///  leaving a folder link and a submodule gitlink out of the positional list
///  left their own names,
///  and every link's stored target text,
///  inspected by nothing.
///  Issue #6640:
///  a link to a file keeps its read-through and gains the same target-text scan.
#[path = "ci_scan_scope/link_target_text_tests.rs"]
mod link_target_text_tests;
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
///  registers the extraction of the workflow's own step shell bodies.
/// Why:
///  The tests must run what CI runs,
///  not a copy of it.
#[path = "ci_scan_scope/scan_step_command.rs"]
mod scan_step_command;
/// What:
///  `#[path = "ci_scan_scope/whole_tree_scan_tests.rs"]
///  mod whole_tree_scan_tests;` registers the cases for the whole-tree arm,
///  which runs when no validated baseline exists.
/// Why:
///  Issue #6640:
///  that arm ran `--all`,
///  whose walk leaves every link and gitlink out,
///  so a tree scanned that way had no link name and no stored target text
///  inspected at all.
#[path = "ci_scan_scope/whole_tree_scan_tests.rs"]
mod whole_tree_scan_tests;
