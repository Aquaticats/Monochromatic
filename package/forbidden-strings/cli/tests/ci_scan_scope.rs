//! What: Integration tests for the scan-scope selection that
//! `.github/workflows/forbidden-strings.yml` performs on CI. Each test
//! extracts the exact `run` body of the workflow's "Scan changed files"
//! step and executes it with `bash` against a disposable Git repository,
//! a fake `gh run list`, and the real scanner binary.
//! Why: Issue #993: a push run canceled before scanning a changed file left
//! that file unscanned forever, because the push scope was the event's own
//! `before..after` range and the next run only scanned its own range. The
//! workflow now scopes from the last validated tree; these tests pin that
//! behavior against the workflow text itself, so no second hand-maintained
//! copy of the shell logic can drift from what CI runs.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

// What:     `const GIT: &str = "/usr/bin/git"` names the system Git.
// Why:      The workflow runs on ubuntu-latest with the runner's stock Git.
//           The developer's `git` on PATH may be the cli-git policy wrapper,
//           which would add policy hooks to fixture commits that CI never
//           runs.
const GIT: &str = "/usr/bin/git";

// What:     `const SCAN_STEP_NAME` is the step heading the extraction keys on.
// Why:      The test must execute the workflow's real shell body; keying on
//           the human-facing step name keeps the extraction independent of
//           YAML formatting while failing loudly when the step is renamed.
const SCAN_STEP_NAME: &str = "    - name: Scan changed files\n";

// What:     `const FIXTURE_RULE` is the rule name in the fixture deny-list.
// Why:      Findings report `rule=<name>`, so asserting on the name proves
//           the detecting rule is the fixture one, not the embedded baseline.
const FIXTURE_RULE: &str = "ci-scope-fixture-sentinel";

// What:     `const SENTINEL` is the literal the fixture deny-list forbids.
// Why:      A test file content marker that the repository's real rule set
//           does not match keeps the fixture violation out of CI findings on
//           this test source itself.
const SENTINEL: &str = "zzq-scope-violation-sentinel";

// What:     `const MARKER_FREE_WORKFLOW` is a pre-marker workflow revision.
// Why:      Runs that executed a revision without the watermark marker
//           scanned only their own push range, so their head commit is not a
//           validated tree; the fixture needs a committed revision that the
//           marker check must reject.
const MARKER_FREE_WORKFLOW: &str = "name: forbidden-strings\n\
                                   # Pre-marker revision: push runs here scan only their own push range.\n";

// What:     `const FIXTURE_RULES` is one named tail-format rule forbidding
//           `SENTINEL`.
// Why:      The scan step reads `$RUNNER_TEMP/forbidden-strings.rules.txt`,
//           produced by the preceding workflow step in CI; the fixture writes
//           the same path with a rule set whose only finding is the planted
//           violation.
const FIXTURE_RULES: &str = "# Fixture deny-list for the CI scan-scope regression tests.\n\
                             ==> ci-scope-fixture-sentinel <==\n\
                             zzq-scope-violation-sentinel\n";

// What:     `const GH_RUN_LIST_STUB` is a fake `gh run list` built from
//           shell, installed on the fixture PATH ahead of any real `gh`.
// Why:      It implements the documented server-side filter semantics
//           honestly: a filter the caller omits is not applied, so dropping
//           `--status`, `--event`, `--branch`, or `--workflow` from the
//           workflow body leaks the extra fixture rows into the candidate
//           list and the tests fail. The stub also rejects any argument or
//           output shape it does not model, so body changes surface loudly.
const GH_RUN_LIST_STUB: &str = r#"#!/bin/sh
set -eu

status_filter=''
event_filter=''
branch_filter=''
workflow_filter=''
json_fields=''
jq_expr=''
while [ "$#" -gt 0 ]; do
  case "$1" in
    run|list)
      shift
      ;;
    --repo|--status|--event|--branch|--workflow|--json|--jq|--limit)
      flag=$1
      value=$2
      if [ "$flag" = '--status' ]; then status_filter=$value; fi
      if [ "$flag" = '--event' ]; then event_filter=$value; fi
      if [ "$flag" = '--branch' ]; then branch_filter=$value; fi
      if [ "$flag" = '--workflow' ]; then workflow_filter=$value; fi
      if [ "$flag" = '--json' ]; then json_fields=$value; fi
      if [ "$flag" = '--jq' ]; then jq_expr=$value; fi
      shift 2
      ;;
    *)
      echo "gh run list stub: unsupported argument: $1" >&2
      exit 2
      ;;
  esac
done

if [ "$json_fields" != 'headSha' ]; then
  echo "gh run list stub: unsupported --json value: $json_fields" >&2
  exit 2
fi
if [ "$jq_expr" != '.[].headSha' ]; then
  echo "gh run list stub: unsupported --jq expression: $jq_expr" >&2
  exit 2
fi

tab=$(printf '\t')
while IFS=$tab read -r conclusion event head_branch workflow head_sha; do
  if [ -n "$status_filter" ] && [ "$conclusion" != "$status_filter" ]; then continue; fi
  if [ -n "$event_filter" ] && [ "$event" != "$event_filter" ]; then continue; fi
  if [ -n "$branch_filter" ] && [ "$head_branch" != "$branch_filter" ]; then continue; fi
  if [ -n "$workflow_filter" ]; then
    matched=''
    if [ "$workflow_filter" = "$workflow" ]; then matched=yes; fi
    if [ "$workflow_filter" = "${workflow%.yml}" ]; then matched=yes; fi
    if [ -z "$matched" ]; then continue; fi
  fi
  printf '%s\n' "$head_sha"
done < "$GH_RUN_FIXTURE"
"#;

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
        let root = std::env::temp_dir().join(format!(
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
        write_executable(&stub_bin.join("git"), "#!/bin/sh\nexec /usr/bin/git \"$@\"\n");
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
        return fixture
    }

    // What:     `fn git(&self, args)` runs one system Git command inside the
    //           fixture repository and returns its trimmed stdout.
    // Why:      Fixture history must behave like the CI checkout: a plain
    //           repository with the workflow's commits, no policy wrapper
    //           hooks, and fixed identity so commit hashes stay stable.
    fn git(&self, args: &[&str]) -> String {
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
        return String::from_utf8_lossy(&output.stdout).trim_end().to_string()
    }

    // What:     `fn put(&self, relative, content)` writes one repository file.
    // Why:      Tests describe history as file contents per commit; creating
    //           parent directories here lets cases add nested paths without
    //           extra setup.
    fn put(&self, relative: &str, content: &str) {
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
    fn write_workflow_revision(&self, marked: bool) {
        if marked {
            let workflow_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.github/workflows/forbidden-strings.yml");
            let workflow = fs::read_to_string(&workflow_path).expect("read real workflow file");
            self.put(".github/workflows/forbidden-strings.yml", &workflow);
        } else {
            self.put(".github/workflows/forbidden-strings.yml", MARKER_FREE_WORKFLOW);
        }
    }

    // What:     `fn commit(&mut self, label)` stages the tree, commits it,
    //           and records the commit under `label`.
    // Why:      Cases reference history positions by label when planting run
    //           rows and choosing the checked-out head.
    fn commit(&mut self, label: &str) -> String {
        self.git(&["add", "--all"]);
        self.git(&["commit", "--message", label]);
        let sha = self.git(&["rev-parse", "HEAD"]);
        self.shas.push((label.to_string(), sha.clone()));
        return sha
    }

    // What:     `fn sha(&self, label)` resolves a recorded commit label.
    // Why:      Run rows and the scan environment need real commit ids; a
    //      missing label is a test authoring error and must fail loudly.
    fn sha(&self, label: &str) -> String {
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
    fn write_runs(&self, rows: &[RunRow]) {
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
        }
    }
}

// What:     `fn write_executable(path, body)` writes a shell file with the
//           executable bit set.
// Why:      The extracted body invokes `gh`, `git`, and `forbidden-strings`
//           by name, so the fixture must present them on `PATH` as
//           executables.
fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).expect("write stub script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod stub script");
}

// What:     `fn scan_step_body()` returns the workflow's scan-step shell body
//           exactly as the Actions runner executes it.
// Why:      Testing a re-implementation would prove nothing about what CI
//           runs; the workflow file is the single source of the logic.
fn scan_step_body() -> String {
    let workflow_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../.github/workflows/forbidden-strings.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("read workflow file");
    let step_start = workflow
        .find(SCAN_STEP_NAME)
        .expect("workflow must keep its `Scan changed files` step");
    let from_step = &workflow[step_start..];
    let step_end = match from_step.find("\n    - name: ") {
        Some(index) => index + 1,
        None => from_step.len(),
    };
    let step = &from_step[..step_end];
    let run_key = "      run: |\n";
    let run_start = step
        .find(run_key)
        .expect("scan step must keep a `run: |` body");
    let body = &step[run_start + run_key.len()..];
    let mut lines: Vec<String> = Vec::new();
    for line in body.split('\n') {
        if line.is_empty() {
            lines.push(String::new());
            continue;
        }
        let Some(stripped) = line.strip_prefix("        ") else {
            panic!("unexpected scan-step line outside the run body: {}", line);
        };
        lines.push(stripped.to_string());
    }
    while !lines.is_empty() && lines[lines.len() - 1].is_empty() {
        lines.pop();
    }
    return lines.join("\n");
}

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
        result.stdout.contains(&format!("{}..{}", fixture.sha("A"), fixture.sha("C"))),
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
        result.stdout.contains("scanning the current tree") && !result.stdout.contains("changed file(s)"),
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
        result.stdout.contains("scanning the current tree") && !result.stdout.contains("changed file(s)"),
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
        result.stdout.contains("scanning the current tree") && !result.stdout.contains("changed file(s)"),
        "a baseline outside the checked-out ancestry must fall back to the whole tree\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr,
    );
    let _ = fs::remove_dir_all(&fixture.root);
}

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
