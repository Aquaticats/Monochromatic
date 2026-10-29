//! What:
//!  Black-box controls of the closure check through the built `monochromatic-lint` executable.
//! Why:
//!  The check is wired through an environment variable,
//! a configuration option,
//! standard error and exit statuses;
//!  only a child process shows all of them together.
//!  The orchestration runs against a stub `cargo`,
//! so the tests need no network,
//! nightly toolchain or `cargo-dylint`
//!  and run in the bounded test container.
//!  The one test that runs the real tools is ignored by default and runs on the host (`mise run test:host`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { status, stdout, stderr } = spawnSync(binary, args, { cwd, env: { MONOCHROMATIC_LINT_CARGO: stub } });
//! ```

#[path = "closure_stub_support.rs"]
mod closure_stub_support;

use closure_stub_support::{Scenario, scenario};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

/// The executable that Cargo built for this test run.
const BINARY: &str = env!("CARGO_BIN_EXE_monochromatic-lint");

/// A process-wide sequence for unique fixture directory names.
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// The rule at warning level,
/// with the library at `lib`,
/// and off for tests.
const CONFIGURATION: &str = r#"[
  { "files": ["**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "warn", "library": "lib" } } },
  { "files": ["**/tests/**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "off" } } }
]"#;

/// A directory owned by one test.
struct Fixture {
    /// The root of the disposable tree.
    path: PathBuf,
}

impl Fixture {
    /// Create the root and the library,
    /// the package `my pkg` with its manifest and one source file.
    fn new() -> Fixture {
        let sequence: u64 = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let created: PathBuf = crate::test_scratch::scratch_root().join(format!(
            "monochromatic-lint-closure-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&created).expect("fixture root");
        let fixture: Fixture = Fixture {
            path: std::fs::canonicalize(&created).expect("resolved fixture root"),
        };
        fixture.write("monochromatic-lint.config.jsonc", CONFIGURATION);
        fixture.write("lib/Cargo.toml", "[package]\n");
        fixture.write("my pkg/Cargo.toml", "[package]\n");
        fixture.write("my pkg/src/lib.rs", "fn a() -> bool {\n  run(|x| x)\n}\n");
        return fixture;
    }

    /// Write a file below the root.
    fn write(&self, relative: &str, text: &str) {
        let path: PathBuf = self.path.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("directories");
        std::fs::write(path, text).expect("file");
    }
}

impl Drop for Fixture {
    /// Remove the tree,
    /// including after a failed assertion.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).expect("remove fixture");
    }
}

/// What one run produced.
struct Run {
    /// The exit status.
    status: i32,
    /// Standard output.
    stdout: String,
    /// Standard error.
    stderr: String,
}

/// Run the executable in the fixture with the stub as `cargo`.
fn run(fixture: &Fixture, stub: &Scenario, arguments: &[&str]) -> Run {
    return run_with(fixture, Some(stub.cargo.as_path()), arguments);
}

/// Run the executable with a chosen `cargo` program,
/// or with the variable unset.
fn run_with(fixture: &Fixture, cargo: Option<&Path>, arguments: &[&str]) -> Run {
    let mut command: Command = Command::new(BINARY);
    command
        .args(arguments)
        .current_dir(&fixture.path)
        .stdin(Stdio::null());
    match cargo {
        Some(program) => command.env("MONOCHROMATIC_LINT_CARGO", program),
        None => command.env_remove("MONOCHROMATIC_LINT_CARGO"),
    };
    let output: Output = command.output().expect("the executable runs");
    return Run {
        status: output.status.code().expect("the executable exits"),
        stdout: String::from_utf8(output.stdout).expect("UTF-8 output"),
        stderr: String::from_utf8(output.stderr).expect("UTF-8 error"),
    };
}

/// The records of a JSONL text.
fn records(text: &str) -> Vec<Value> {
    return text.lines().map(parse_line).collect::<Vec<Value>>();
}

/// One JSON line.
fn parse_line(line: &str) -> Value {
    return serde_json::from_str::<Value>(line).expect("one JSON object per line");
}

/// A cargo record for a file at a byte range.
fn record(code: &str, file: &str, start: u64) -> String {
    return format!(
        r#"{{"reason": "compiler-message", "message": {{"code": {{"code": "{code}", "explanation": null}}, "level": "warning", "message": "a closure", "spans": [{{"file_name": "{file}", "is_primary": true, "byte_start": {start}, "byte_end": {end}, "line_start": 2, "column_start": 7}}], "children": [{{"level": "help", "message": "use a function", "spans": [], "children": [], "code": null}}]}}}}
"#,
        end = start + 7
    );
}

/// A finding in a package with a space in its name is reported at warning level with exit status 0.
#[test]
fn a_report_is_a_warning_finding_and_exit_zero() {
    let fixture: Fixture = Fixture::new();
    let stub = scenario(&[(
        "dylint.stdout",
        record("avoidable_closure", "src/lib.rs", 23).as_str(),
    )]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!(output.status, 0, "{}", output.stderr);
    let found: Vec<Value> = records(&output.stdout);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["code"], "rust/avoidable-closure");
    assert_eq!(found[0]["severity"], "warn");
    assert_eq!(found[0]["filename"], "my pkg/src/lib.rs");
    assert_eq!(found[0]["labels"][0]["span"]["line"], 2);
    assert_eq!(found[0]["labels"][0]["span"]["column"], 7);
    assert_eq!(found[0]["help"], "use a function");
    assert_eq!(output.stderr, "");
}

/// The warning count is subject to `--max-warnings` like any other warning.
#[test]
fn max_warnings_applies() {
    let fixture: Fixture = Fixture::new();
    let stub = scenario(&[(
        "dylint.stdout",
        record("avoidable_closure", "src/lib.rs", 23).as_str(),
    )]);
    assert_eq!(
        run(&fixture, &stub, &["--max-warnings", "0", "my pkg"]).status,
        1
    );
    assert_eq!(
        run(&fixture, &stub, &["--max-warnings", "1", "my pkg"]).status,
        0
    );
}

/// A later block can raise the severity of one tree to error,
/// and the exit status follows.
#[test]
fn a_path_block_can_raise_the_severity_to_error() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "monochromatic-lint.config.jsonc",
        r#"[
          { "files": ["**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "warn", "library": "lib" } } },
          { "files": ["my pkg/**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "error" } } }
        ]"#,
    );
    let stub = scenario(&[(
        "dylint.stdout",
        record("avoidable_closure", "src/lib.rs", 23).as_str(),
    )]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!(output.status, 1);
    assert_eq!(records(&output.stdout)[0]["severity"], "error");
}

/// A report in an exempt directory or in a file outside the run is dropped.
#[test]
fn reports_in_exempt_or_unlisted_files_are_dropped() {
    let fixture: Fixture = Fixture::new();
    fixture.write("my pkg/tests/it.rs", "fn a() { run(|x| x); }\n");
    fixture.write("my pkg/src/other.rs", "fn a() { run(|x| x); }\n");
    let stdout: String = format!(
        "{}{}{}",
        record("avoidable_closure", "tests/it.rs", 11),
        record("avoidable_closure", "src/other.rs", 11),
        record("avoidable_closure", "src/lib.rs", 23)
    );
    let stub = scenario(&[("dylint.stdout", stdout.as_str())]);
    let output: Run = run(
        &fixture,
        &stub,
        &["my pkg/src/lib.rs", "my pkg/tests/it.rs"],
    );
    let found: Vec<Value> = records(&output.stdout);
    assert_eq!(found.len(), 1, "{}", output.stdout);
    assert_eq!(found[0]["filename"], "my pkg/src/lib.rs");
}

/// A stale expectation is a finding of the same rule.
#[test]
fn an_unfulfilled_expectation_is_reported() {
    let fixture: Fixture = Fixture::new();
    let stub = scenario(&[(
        "dylint.stdout",
        record("unfulfilled_lint_expectations", "src/lib.rs", 23).as_str(),
    )]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!(records(&output.stdout)[0]["code"], "rust/avoidable-closure");
}

/// A fuzz or bench crate is never run,
/// even when its files select the rule.
#[test]
fn fuzz_and_bench_crates_run_nothing() {
    let fixture: Fixture = Fixture::new();
    for directory in ["my pkg.fuzz", "my pkg.bench"] {
        fixture.write(format!("{directory}/Cargo.toml").as_str(), "[package]\n");
        fixture.write(format!("{directory}/src/lib.rs").as_str(), "fn a() {}\n");
    }
    let stub = scenario(&[]);
    let output: Run = run(&fixture, &stub, &["my pkg.fuzz", "my pkg.bench"]);
    assert_eq!(
        (
            output.status,
            output.stdout.as_str(),
            output.stderr.as_str()
        ),
        (0, "", "")
    );
    assert!(stub.calls().is_empty());
}

/// A Rust file with no Cargo.toml above it is not an error and runs nothing.
#[test]
fn a_package_without_cargo_toml_runs_nothing() {
    let fixture: Fixture = Fixture::new();
    fixture.write("loose/script.rs", "fn a() {}\n");
    let stub = scenario(&[]);
    let output: Run = run(&fixture, &stub, &["loose"]);
    assert_eq!(
        (
            output.status,
            output.stdout.as_str(),
            output.stderr.as_str()
        ),
        (0, "", "")
    );
    assert!(stub.calls().is_empty());
    let debug: Run = run(&fixture, &stub, &["--debug", "loose"]);
    assert!(
        debug
            .stderr
            .contains("loose/script.rs: not checked by rust/avoidable-closure"),
        "{}",
        debug.stderr
    );
}

/// A cold cache is a note on standard error and exit status 0,
/// and nothing is fetched.
#[test]
fn a_cold_cache_is_a_skip_note_and_exit_zero() {
    let fixture: Fixture = Fixture::new();
    let stub = scenario(&[("metadata.status", "101")]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!((output.status, output.stdout.as_str()), (0, ""));
    assert!(
        output
            .stderr
            .starts_with("monochromatic-lint: rust/avoidable-closure skipped my pkg:"),
        "{}",
        output.stderr
    );
    assert!(
        output.stderr.contains("cargo fetch --locked"),
        "{}",
        output.stderr
    );
    assert!(
        !stub
            .calls()
            .iter()
            .any(|call| return call.contains("fetch")),
        "the linter never fetches"
    );
    let silent: Run = run(&fixture, &stub, &["--silent", "my pkg"]);
    assert_eq!(silent.status, 0);
    assert!(
        silent.stderr.contains("skipped"),
        "a skip is not a finding, so --silent keeps it"
    );
}

/// A missing prerequisite is a skip note and exit status 0.
#[test]
fn a_missing_prerequisite_is_a_skip_note_and_exit_zero() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "my pkg/monochromatic-lint.prerequisites.jsonc",
        r#"{"executables": ["no-such-program-xyz"], "qmake": true}"#,
    );
    let stub = scenario(&[]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!(output.status, 0);
    assert!(
        output.stderr.contains("no-such-program-xyz"),
        "{}",
        output.stderr
    );
    assert!(
        !stub
            .calls()
            .iter()
            .any(|call| return call.starts_with("dylint|"))
    );
}

/// A manifest that names an unknown prerequisite fails the run with a finding that names the manifest.
#[test]
fn an_unknown_prerequisite_exits_two() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "my pkg/monochromatic-lint.prerequisites.jsonc",
        r#"{"run": ["rm -rf /"]}"#,
    );
    let stub = scenario(&[]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!(output.status, 2);
    let found: Vec<Value> = records(&output.stdout);
    assert_eq!(found[0]["code"], "core/processing-failure");
    assert_eq!(found[0]["filename"], "my pkg/Cargo.toml");
    assert!(
        found[0]["message"]
            .as_str()
            .expect("text")
            .contains("\"run\"")
    );
    assert!(
        !stub
            .calls()
            .iter()
            .any(|call| return call.starts_with("dylint|"))
    );
}

/// A library path that does not exist fails the run.
#[test]
fn a_missing_library_exits_two() {
    let fixture: Fixture = Fixture::new();
    std::fs::remove_file(fixture.path.join("lib/Cargo.toml")).expect("remove");
    let stub = scenario(&[]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!(output.status, 2);
    assert_eq!(
        records(&output.stdout)[0]["filename"],
        "monochromatic-lint.config.jsonc"
    );
    assert!(stub.calls().is_empty());
}

/// A cargo failure with no diagnostic fails the run and quotes cargo's last messages.
#[test]
fn a_non_diagnostic_cargo_failure_exits_two() {
    let fixture: Fixture = Fixture::new();
    let stub = scenario(&[
        ("dylint.status", "101"),
        (
            "dylint.stderr",
            "error: failed to run custom build command\n",
        ),
    ]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!(output.status, 2);
    let message: String = String::from(
        records(&output.stdout)[0]["message"]
            .as_str()
            .expect("text"),
    );
    assert!(
        message.contains("failed to run custom build command"),
        "{message}"
    );
}

/// A machine without the tools skips every package with one note and exit status 0.
#[test]
fn a_machine_without_cargo_dylint_skips_everything() {
    let fixture: Fixture = Fixture::new();
    let stub = scenario(&[("dylint-version.status", "1")]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!(output.status, 0);
    assert!(
        output.stderr.contains("skipped all 1 package(s)"),
        "{}",
        output.stderr
    );
}

/// The compile flags and the offline posture reach the process.
#[test]
fn the_process_gets_the_flags_and_the_offline_posture() {
    let fixture: Fixture = Fixture::new();
    let stub = scenario(&[]);
    run(&fixture, &stub, &["my pkg"]);
    let calls: Vec<String> = stub.calls();
    let dylint: &String = calls
        .iter()
        .find(|call| return call.starts_with("dylint|"))
        .expect("a dylint call");
    assert!(
        dylint.contains("DYLINT_RUSTFLAGS=-W avoidable_closure -W unfulfilled_lint_expectations"),
        "{dylint}"
    );
    assert!(dylint.contains("CARGO_NET_OFFLINE=true"), "{dylint}");
    assert!(dylint.contains("--path "), "{dylint}");
    assert!(
        dylint.contains(&format!("{}", fixture.path.join("lib").display())),
        "{dylint}"
    );
}

/// `--fix` and `--stdin` start no process.
#[test]
fn fix_and_stdin_start_no_process() {
    let fixture: Fixture = Fixture::new();
    let stub = scenario(&[]);
    assert_eq!(run(&fixture, &stub, &["--fix", "my pkg"]).status, 0);
    let mut command: Command = Command::new(BINARY);
    command
        .args(["--stdin", "--stdin-filename", "my pkg/src/lib.rs"])
        .current_dir(&fixture.path)
        .env("MONOCHROMATIC_LINT_CARGO", &stub.cargo)
        .stdin(Stdio::null());
    assert_eq!(command.output().expect("runs").status.code(), Some(0));
    assert!(stub.calls().is_empty());
}

/// A configuration without the rule starts no process.
#[test]
fn a_configuration_without_the_rule_starts_no_process() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "monochromatic-lint.config.jsonc",
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "error" } } }]"#,
    );
    let stub = scenario(&[]);
    assert_eq!(run(&fixture, &stub, &["my pkg"]).status, 0);
    assert!(stub.calls().is_empty());
}

/// A selected rule without a library is a configuration error before anything runs.
#[test]
fn a_rule_without_a_library_exits_two_before_running_anything() {
    let fixture: Fixture = Fixture::new();
    fixture.write("monochromatic-lint.config.jsonc", r#"[{ "files": ["**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "warn" } } }]"#);
    let stub = scenario(&[]);
    let output: Run = run(&fixture, &stub, &["my pkg"]);
    assert_eq!(output.status, 2);
    assert!(
        output.stderr.contains("needs the option library"),
        "{}",
        output.stderr
    );
    assert!(stub.calls().is_empty());
}

/// The rule is listed,
/// takes no suppression directive,
/// and an empty library is refused.
#[test]
fn the_rule_is_listed_as_unsuppressible_with_a_library_option() {
    let fixture: Fixture = Fixture::new();
    let listing: Run = run_with(&fixture, None, &["--rules"]);
    let rule: Value = records(&listing.stdout)
        .into_iter()
        .find(|value| return value["id"] == "rust/avoidable-closure")
        .expect("listed");
    assert_eq!(rule["suppressible"], false);
    assert_eq!(rule["options"].to_string(), r#"["library"]"#);
    fixture.write("monochromatic-lint.config.jsonc", r#"[{ "files": ["**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "warn", "library": "" } } }]"#);
    assert_eq!(run_with(&fixture, None, &["my pkg"]).status, 2);
}

/// `--debug` explains what was checked.
#[test]
fn debug_notes_name_the_checked_package() {
    let fixture: Fixture = Fixture::new();
    let stub = scenario(&[(
        "dylint.stdout",
        record("avoidable_closure", "src/lib.rs", 23).as_str(),
    )]);
    let output: Run = run(&fixture, &stub, &["--debug", "my pkg"]);
    assert!(
        output
            .stderr
            .contains("rust/avoidable-closure checked my pkg: 1 report(s)"),
        "{}",
        output.stderr
    );
}

/// Host-only:
/// the real tools over the real `package/wg/cli-quicker-exempt`,
/// compared with an `--all-targets` run of
/// the same library so that the absence of test findings is shown by a probe that could have found them.
#[test]
#[ignore = "needs cargo-dylint, the rustc-dev nightly and a warm cargo cache; run with `mise run test:host`"]
fn host_real_run_over_wg_quicker_exempt() {
    let root: PathBuf =
        std::fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."))
            .expect("repository root");
    let package: PathBuf = root.join("package/wg/cli-quicker-exempt");
    let linted: Output = Command::new(BINARY)
        .arg("package/wg/cli-quicker-exempt")
        .current_dir(&root)
        .env_remove("MONOCHROMATIC_LINT_CARGO")
        .output()
        .expect("the executable runs");
    let stderr: String = String::from_utf8_lossy(&linted.stderr).into_owned();
    assert_eq!(linted.status.code(), Some(0), "{stderr}");
    assert!(
        !stderr.contains("skipped"),
        "the package was skipped: {stderr}"
    );
    let findings: Vec<Value> = records(&String::from_utf8(linted.stdout).expect("UTF-8"));
    assert!(!findings.is_empty(), "the real lint reported nothing");
    for finding in &findings {
        assert_eq!(finding["code"], "rust/avoidable-closure");
        let name: &str = finding["filename"].as_str().expect("text");
        assert!(
            !name.contains("_tests.rs") && !name.contains("/tests/"),
            "{name} is a test file"
        );
    }
    // Positive control: with every target compiled, the same library does report test code.
    let everything: Output = Command::new("cargo")
        .args(["dylint", "--path"])
        .arg(root.join("package/linter/monochromatic-lint-closure"))
        .arg("--manifest-path")
        .arg(package.join("Cargo.toml"))
        .args([
            "--",
            "--locked",
            "--all-targets",
            "--message-format",
            "json",
        ])
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .expect("cargo dylint runs");
    let in_tests: bool = String::from_utf8_lossy(&everything.stdout).contains("_tests.rs");
    assert!(
        in_tests,
        "the --all-targets probe found no test finding, so it cannot show their absence"
    );
}

/// The unit-test scratch root, shared with the library test build and placed in the build directory.
#[path = "test_scratch.rs"]
mod test_scratch;
