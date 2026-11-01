//! What:
//!  Controls of the closure step against a stub `cargo`.
//! Why:
//!  The step is where skip,
//! failure and findings are decided,
//! so each outcome is driven end to end with canned cargo
//!  answers:
//! no compiler,
//! nightly or network is involved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a cold cache skips', () => expect(runClosureStepWith(plans, cwd, false, stub.cargo).messages).toEqual([...]));
//! ```

use super::{ClosureOutcome, run_closure_step_at, run_closure_step_with, with_messages};
use crate::{
    closure_driver::DriverSite,
    closure_manifest::PREREQUISITES_FILE,
    closure_stub_support::{Scenario, scenario, stub_cargo},
    diagnostic::Severity,
    run_plan::FilePlan,
    run_test_support::{CONFIG, plan, write},
    test_fs::Fixture,
};

/// The rule at warning level,
/// with the library next to the configuration.
const CONFIGURATION: &str = r#"[
  { "files": ["**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "warn", "library": "lib" } } }
]"#;

/// The source of the checked file,
/// with a closure starting at byte 23.
const SOURCE: &str = "fn a() -> bool {\n  run(|x| x)\n}\n";

/// One cargo record for `src/lib.rs`.
fn record(code: &str, start: u64) -> String {
    return format!(
        r#"{{"reason": "compiler-message", "message": {{"code": {{"code": "{code}", "explanation": null}}, "level": "warning", "message": "a closure", "spans": [{{"file_name": "src/lib.rs", "is_primary": true, "byte_start": {start}, "byte_end": {end}, "line_start": 2, "column_start": 7}}], "children": []}}}}"#,
        end = start + 7
    );
}

/// A fixture with the library,
/// a package whose directory has a space,
/// and the configuration.
fn fixture() -> Fixture {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "lib/Cargo.toml", "[package]\n");
    write(&fixture.path, "my pkg/Cargo.toml", "[package]\n");
    write(&fixture.path, "my pkg/src/lib.rs", SOURCE);
    return fixture;
}

/// Plan the package's file.
fn plans(fixture: &Fixture) -> Vec<FilePlan> {
    return vec![plan(&fixture.path, "my pkg/src/lib.rs")];
}

/// Run the step with a scenario.
fn run(fixture: &Fixture, stub: &Scenario) -> ClosureOutcome {
    return run_closure_step_with(plans(fixture).as_slice(), &fixture.path, false, &stub.cargo);
}

/// The kinds of cargo call the stub saw,
/// in order.
fn kinds(stub: &Scenario) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    for call in stub.calls() {
        found.push(String::from(call.split('|').next().unwrap_or("")));
    }
    return found;
}

/// A clean run produces no finding,
/// one note and the three calls in order.
#[test]
fn a_clean_package_is_checked_in_order() {
    let fixture: Fixture = fixture();
    let stub = scenario(&[]);
    let outcome = run(&fixture, &stub);
    assert!(outcome.findings.is_empty() && outcome.messages.is_empty());
    assert_eq!(kinds(&stub), vec!["dylint-version", "metadata", "dylint"]);
    assert!(
        outcome
            .notes
            .iter()
            .any(|note| return note.contains("checked my pkg")),
        "{:?}",
        outcome.notes
    );
}

/// A report becomes a finding at the byte position of the file,
/// with the configured severity.
#[test]
fn a_report_becomes_a_finding() {
    let fixture: Fixture = fixture();
    let stub = scenario(&[(
        "dylint.stdout",
        format!("{}\n", record("avoidable_closure", 23)).as_str(),
    )]);
    let outcome = run(&fixture, &stub);
    assert_eq!(outcome.findings.len(), 1);
    let finding = &outcome.findings[0];
    assert_eq!(
        (finding.code.as_str(), finding.severity),
        ("rust/avoidable-closure", Severity::Warn)
    );
    assert_eq!(finding.filename, "my pkg/src/lib.rs");
    assert_eq!(
        (finding.labels[0].span.line, finding.labels[0].span.column),
        (2, 7)
    );
}

/// An expectation that suppressed nothing is reported like a closure.
#[test]
fn an_unfulfilled_expectation_is_reported() {
    let fixture: Fixture = fixture();
    let stub = scenario(&[(
        "dylint.stdout",
        format!("{}\n", record("unfulfilled_lint_expectations", 23)).as_str(),
    )]);
    assert_eq!(run(&fixture, &stub).findings.len(), 1);
}

/// A cold cache skips with a message on the standard-error channel and runs nothing else.
#[test]
fn a_cold_cache_skips_with_a_message() {
    let fixture: Fixture = fixture();
    let stub = scenario(&[
        ("metadata.status", "101"),
        ("metadata.stderr", "error: no matching package\n"),
    ]);
    let outcome = run(&fixture, &stub);
    assert!(outcome.findings.is_empty());
    assert_eq!(outcome.messages.len(), 1);
    assert!(
        outcome.messages[0].starts_with("rust/avoidable-closure skipped my pkg:"),
        "{:?}",
        outcome.messages
    );
    assert!(outcome.messages[0].contains("cargo fetch --locked"));
    assert_eq!(kinds(&stub), vec!["dylint-version", "metadata"]);
}

/// A missing prerequisite skips with the reason and never runs dylint.
#[test]
fn a_missing_prerequisite_skips() {
    let fixture: Fixture = fixture();
    write(
        &fixture.path,
        format!("my pkg/{PREREQUISITES_FILE}").as_str(),
        r#"{"executables": ["monochromatic-lint-absent-program"]}"#,
    );
    let stub = scenario(&[]);
    let outcome = run(&fixture, &stub);
    assert!(outcome.findings.is_empty());
    assert!(
        outcome.messages[0].contains("monochromatic-lint-absent-program"),
        "{:?}",
        outcome.messages
    );
    assert!(!kinds(&stub).contains(&String::from("dylint")));
}

/// An unknown prerequisite fails the run,
/// names the manifest and runs nothing.
#[test]
fn an_unknown_prerequisite_is_a_processing_failure() {
    let fixture: Fixture = fixture();
    write(
        &fixture.path,
        format!("my pkg/{PREREQUISITES_FILE}").as_str(),
        r#"{"shell": ["touch x"]}"#,
    );
    let stub = scenario(&[]);
    let outcome = run(&fixture, &stub);
    assert_eq!(outcome.findings.len(), 1);
    assert!(outcome.findings[0].processing_failure);
    assert_eq!(outcome.findings[0].filename, "my pkg/Cargo.toml");
    assert!(
        outcome.findings[0].message.contains("\"shell\""),
        "{}",
        outcome.findings[0].message
    );
    assert_eq!(kinds(&stub), vec!["dylint-version"]);
}

/// A library path without a Cargo.toml fails once and runs nothing.
#[test]
fn a_missing_library_is_a_processing_failure_and_runs_nothing() {
    let fixture: Fixture = fixture();
    std::fs::remove_file(fixture.path.join("lib/Cargo.toml")).expect("remove library manifest");
    let stub = scenario(&[]);
    let outcome = run(&fixture, &stub);
    assert_eq!(outcome.findings.len(), 1);
    assert!(outcome.findings[0].processing_failure);
    assert_eq!(outcome.findings[0].filename, CONFIG);
    assert!(
        outcome.findings[0].message.contains("holds no Cargo.toml"),
        "{}",
        outcome.findings[0].message
    );
    assert!(stub.calls().is_empty());
}

/// A run that exits nonzero without a record is a processing failure quoting cargo's last messages.
#[test]
fn a_non_diagnostic_failure_is_a_processing_failure() {
    let fixture: Fixture = fixture();
    let stub = scenario(&[
        ("dylint.status", "101"),
        (
            "dylint.stderr",
            "line one\nerror: could not find `cargo-dylint`\n",
        ),
    ]);
    let outcome = run(&fixture, &stub);
    assert_eq!(outcome.findings.len(), 1);
    let failure = &outcome.findings[0];
    assert!(failure.processing_failure && failure.severity == Severity::Error);
    assert_eq!(failure.filename, "my pkg/Cargo.toml");
    assert!(
        failure.message.contains("could not find `cargo-dylint`"),
        "{}",
        failure.message
    );
    assert!(
        failure.message.contains("incomplete"),
        "{}",
        failure.message
    );
}

/// A compiler error is quoted,
/// and the findings of the same run are kept beside the failure.
#[test]
fn a_compiler_error_is_quoted_and_findings_are_kept() {
    let fixture: Fixture = fixture();
    let error: &str = r#"{"reason": "compiler-message", "message": {"code": {"code": "E0425"}, "level": "error", "message": "cannot find value", "rendered": "error[E0425]: cannot find value `q`\n", "spans": [], "children": []}}"#;
    let stdout: String = format!("{}\n{error}\n", record("avoidable_closure", 23));
    let stub = scenario(&[("dylint.status", "101"), ("dylint.stdout", stdout.as_str())]);
    let outcome = run(&fixture, &stub);
    assert_eq!(outcome.findings.len(), 2);
    assert!(!outcome.findings[0].processing_failure);
    assert!(
        outcome.findings[1]
            .message
            .contains("cannot find value `q`"),
        "{}",
        outcome.findings[1].message
    );
}

/// A missing `cargo` skips every package once,
/// with one message.
#[test]
fn a_missing_cargo_skips_everything_with_one_message() {
    let fixture: Fixture = fixture();
    let outcome = run_closure_step_with(
        plans(&fixture).as_slice(),
        &fixture.path,
        false,
        std::path::Path::new("/nonexistent/monochromatic-lint/cargo"),
    );
    assert!(outcome.findings.is_empty());
    assert_eq!(outcome.messages.len(), 1);
    assert!(
        outcome.messages[0].contains("all 1 package(s)"),
        "{:?}",
        outcome.messages
    );
}

/// A failing `cargo dylint --version` skips everything and runs no package.
#[test]
fn a_missing_dylint_skips_everything() {
    let fixture: Fixture = fixture();
    let stub = scenario(&[("dylint-version.status", "101")]);
    let outcome = run(&fixture, &stub);
    assert!(
        outcome.messages[0].contains("cargo dylint --version"),
        "{:?}",
        outcome.messages
    );
    assert_eq!(kinds(&stub), vec!["dylint-version"]);
}

/// `--fix` runs nothing.
#[test]
fn fix_mode_runs_nothing() {
    let fixture: Fixture = fixture();
    let stub = scenario(&[]);
    let outcome =
        run_closure_step_with(plans(&fixture).as_slice(), &fixture.path, true, &stub.cargo);
    assert!(outcome.findings.is_empty() && outcome.messages.is_empty());
    assert!(outcome.notes[0].contains("--fix"));
    assert!(stub.calls().is_empty());
}

/// No selected package runs nothing,
/// not even the tool probe.
#[test]
fn no_package_runs_nothing() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "loose/script.rs", "");
    let stub = scenario(&[]);
    let selected = vec![plan(&fixture.path, "loose/script.rs")];
    let outcome = run_closure_step_with(selected.as_slice(), &fixture.path, false, &stub.cargo);
    assert!(outcome.findings.is_empty() && outcome.messages.is_empty());
    assert!(stub.calls().is_empty());
}

/// Two packages are checked independently:
/// a skip in one does not hide a finding in the other.
#[test]
fn packages_are_independent() {
    let fixture: Fixture = fixture();
    write(&fixture.path, "cold/Cargo.toml", "[package]\n");
    write(&fixture.path, "cold/src/lib.rs", SOURCE);
    let stub = scenario(&[
        ("metadata.cold.status", "101"),
        (
            "dylint.stdout",
            format!("{}\n", record("avoidable_closure", 23)).as_str(),
        ),
    ]);
    let selected = vec![
        plan(&fixture.path, "cold/src/lib.rs"),
        plan(&fixture.path, "my pkg/src/lib.rs"),
    ];
    let outcome = run_closure_step_with(selected.as_slice(), &fixture.path, false, &stub.cargo);
    assert_eq!(outcome.messages.len(), 1);
    assert!(
        outcome.messages[0].contains("cold"),
        "{:?}",
        outcome.messages
    );
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(outcome.findings[0].filename, "my pkg/src/lib.rs");
}

/// Messages are prefixed with the program name and put before the existing standard error.
#[test]
fn messages_are_prefixed_and_come_first() {
    let text: String = with_messages(
        "debug line\n",
        &[String::from("one"), String::from("two")],
        "monochromatic-lint",
    );
    assert_eq!(
        text,
        "monochromatic-lint: one\nmonochromatic-lint: two\ndebug line\n"
    );
    assert_eq!(with_messages("", &[], "p"), "");
}

//region Dylint driver check

/// A `cargo-dylint` that prints its version, so the driver check has a version to compare.
const VERSIONED: (&str, &str) = ("dylint-version.stdout", "cargo-dylint 6.3.0\n");

/// The answers that make the offline resolution of the throwaway driver crate fail.
const COLD_DRIVER: [(&str, &str); 2] = [
    ("metadata.dylint-driver-probe.status", "101"),
    (
        "metadata.dylint-driver-probe.stderr",
        "error: failed to select a version for the requirement `dylint_driver = \"=6.3.0\"`\n",
    ),
];

/// The facts of the machine for a stub `cargo`:
/// a `rustup` that names one toolchain,
/// and a driver directory and a scratch directory below the stub's scenario directory.
fn driver_site(stub: &Scenario, rustup: &Scenario) -> DriverSite {
    return DriverSite {
        rustup: rustup.cargo.clone(),
        drivers: Some(stub.directory.join("drivers")),
        scratch: Some(stub.directory.join("scratch")),
    };
}

/// A `rustup` stub that names the toolchain `nightly-fixture`.
fn toolchain_stub() -> Scenario {
    return scenario(&[("show.stdout", "nightly-fixture (default)\n")]);
}

/// Install a driver for the toolchain that reports the given version line.
fn install_driver(stub: &Scenario, version_line: &str) {
    let directory = stub.directory.join("drivers").join("nightly-fixture");
    std::fs::create_dir_all(&directory).expect("driver directory");
    std::fs::hard_link(stub_cargo(), directory.join("dylint-driver")).expect("driver link");
    std::fs::write(directory.join("-V.stdout"), version_line).expect("driver version");
}

/// Run the step with explicit machine facts.
fn run_at(fixture: &Fixture, stub: &Scenario, site: &DriverSite) -> ClosureOutcome {
    return run_closure_step_at(
        plans(fixture).as_slice(),
        &fixture.path,
        false,
        &stub.cargo,
        site,
    );
}

/// An outdated or absent driver with a cold cache skips the package with the warm command,
/// and neither the package's dependency probe nor the run starts.
#[test]
fn a_missing_driver_skips_with_the_warm_command() {
    let fixture: Fixture = fixture();
    let mut answers: Vec<(&str, &str)> = COLD_DRIVER.to_vec();
    answers.push(VERSIONED);
    let stub = scenario(answers.as_slice());
    let rustup = toolchain_stub();
    let outcome = run_at(&fixture, &stub, &driver_site(&stub, &rustup));
    assert!(outcome.findings.is_empty());
    assert_eq!(outcome.messages.len(), 1, "{:?}", outcome.messages);
    let message: &String = &outcome.messages[0];
    assert!(
        message.starts_with("rust/avoidable-closure skipped all 1 package(s):"),
        "{message}"
    );
    assert!(
        message.contains(
            "`env --unset=CARGO_NET_OFFLINE cargo dylint --path lib --manifest-path my pkg/Cargo.toml -- --locked`"
        ),
        "{message}"
    );
    assert!(message.contains("is absent"), "{message}");
    assert!(message.ends_with('.'), "{message}");
    assert_eq!(kinds(&stub), vec!["dylint-version", "metadata"]);
    assert!(
        stub.calls()[1].contains("driver-probe-"),
        "{:?}",
        stub.calls()
    );
}

/// A current driver is the control:
/// no resolution of the driver crate happens and the run goes ahead.
#[test]
fn a_current_driver_lets_the_package_run() {
    let fixture: Fixture = fixture();
    let mut answers: Vec<(&str, &str)> = COLD_DRIVER.to_vec();
    answers.push(VERSIONED);
    let stub = scenario(answers.as_slice());
    install_driver(&stub, "dylint-driver 6.3.0\n");
    let rustup = toolchain_stub();
    let outcome = run_at(&fixture, &stub, &driver_site(&stub, &rustup));
    assert!(outcome.findings.is_empty() && outcome.messages.is_empty());
    assert_eq!(kinds(&stub), vec!["dylint-version", "metadata", "dylint"]);
    assert!(
        !stub.calls()[1].contains("driver-probe-"),
        "{:?}",
        stub.calls()
    );
}

/// A stale driver is acceptable when the cargo cache resolves the pinned crate,
/// because `cargo dylint` then rebuilds it offline.
#[test]
fn a_stale_driver_with_a_warm_cache_lets_the_package_run() {
    let fixture: Fixture = fixture();
    let stub = scenario(&[VERSIONED]);
    install_driver(&stub, "dylint-driver 6.2.0\n");
    let rustup = toolchain_stub();
    let outcome = run_at(&fixture, &stub, &driver_site(&stub, &rustup));
    assert!(outcome.findings.is_empty() && outcome.messages.is_empty());
    assert_eq!(
        kinds(&stub),
        vec!["dylint-version", "metadata", "metadata", "dylint"]
    );
    assert!(
        stub.calls()[1].contains("driver-probe-"),
        "{:?}",
        stub.calls()
    );
}

/// A version that cannot be read, or facts that cannot be found, do not skip anything:
/// the check is noted for `--debug` and the run decides.
#[test]
fn a_driver_check_that_cannot_be_made_is_a_note_and_the_run_proceeds() {
    let fixture: Fixture = fixture();
    let stub = scenario(&[]);
    let rustup = toolchain_stub();
    let outcome = run_at(&fixture, &stub, &driver_site(&stub, &rustup));
    assert!(outcome.messages.is_empty());
    assert!(
        outcome
            .notes
            .iter()
            .any(|note| return note.contains("printed no readable version")),
        "{:?}",
        outcome.notes
    );
    assert_eq!(kinds(&stub), vec!["dylint-version", "metadata", "dylint"]);
    let located = scenario(&[VERSIONED]);
    let unlocated: DriverSite = DriverSite {
        drivers: None,
        ..driver_site(&located, &rustup)
    };
    let second = run_at(&fixture, &located, &unlocated);
    assert!(second.messages.is_empty());
    assert!(
        second
            .notes
            .iter()
            .any(|note| return note.contains("did not run: no driver directory")),
        "{:?}",
        second.notes
    );
    assert!(kinds(&located).contains(&String::from("dylint")));
}

//endregion
