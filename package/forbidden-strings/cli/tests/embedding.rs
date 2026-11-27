//! What:
//!  Public scanner API exercised from a real library consumer with disposable cache state.
//! Why:
//!  Private matcher fixtures do not prove exported loading,
//!  cache warnings or native rule-file paths work.
//!  Where the file system under the build directory refuses a name that is not valid UTF-8,
//!  the driver re-runs itself inside a namespace of its own with a tmpfs there,
//!  so the native rule-file control still runs instead of failing or skipping (issue 6491).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Start a bounded child test with its own home/cache, then call the exported Scanner interface.
//! ```

/// Import only the published library boundary,
///  not private scanner internals.
use forbidden_strings::{
    CandidateScan,
    ScanFinding,
    Scanner,
};
/// What:
///  `Cow` is "a borrowed value or an owned one",
/// which reads the same either way;
///  the siblings a reader might expect are `String` (always owned,
/// so always a copy) and `&str` (borrowed only).
/// Why:
///  A child's output is usually valid UTF-8 and so is borrowed from the bytes already read,
///  and only an output with an invalid byte is copied and repaired.
use std::borrow::Cow;
/// Import native filesystem and child-process primitives for disposable verification.
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

/// What:
///  The fixture-root question this control asks before it makes a rule file whose name is not valid UTF-8,
///  and the re-execution that puts the control on a tmpfs where the root refuses that name.
/// Why:
///  `CARGO_TARGET_TMPDIR` sits on this host's home dataset,
/// which refuses such a name under `utf8only=on`
///  and answers `EILSEQ` (issue 6491),
///  so the control would fail for a file system's reason rather than the scanner's;
///  the mechanism lives in its own file so this driver reads as what it checks.
#[cfg(unix)]
#[path = "support/fixture_root_taking_any_name.rs"]
mod fixture_root_taking_any_name;

/// What:
///  The name libtest answers to for this file's driver control,
/// which is the function's own name.
/// Why:
///  The half that runs where the fixture root refuses the byte name starts that same test by this name,
/// with `--exact`,
///  and a platform whose paths are not bytes asks nothing,
/// so the name is only written where a path is bytes.
#[cfg(unix)]
const THE_DRIVER_CONTROL: &str = "public_embedding_runs_without_real_user_state";

/// What:
///  The line the native rule-file control prints once it has run.
/// Why:
///  A control that returned early on a file system that refuses its name would leave the line out,
///  and the driver fails on its absence instead of passing a run in which the control never ran.
#[cfg(unix)]
const NATIVE_RULE_PATH_CONTROL_RAN: &str = "NATIVE RULE-FILE NAME CONTROL RAN";

/// Child consumer selected explicitly by the driver;
///  ordinary test enumeration performs no state mutation here.
#[test]
fn embedded_consumer_probe() {
    // The driver controls this variable in a separate process instead of mutating shared process environment.
    let Some(directory) = std::env::var_os("FORBIDDEN_STRINGS_EMBEDDED_FIXTURE") else {
        return;
    };
    let root: PathBuf = PathBuf::from(directory);
    let rules: PathBuf = root.join("rules.txt");
    let scanner: Scanner = Scanner::load(rules.as_path(), false, true).expect("public load");
    assert_eq!(scanner.cache_warnings().len(), 1);
    assert_eq!(scanner.cache_warnings()[0].reason(), "missing");
    assert_eq!(scanner.cache_warnings()[0].recovery(), "compile-from-text");
    let report: CandidateScan = scanner.scan(
        73,
        Path::new("private/EMBEDDED_NEEDLE_LONG.txt"),
        b"first\nEMBEDDED_NEEDLE_LONG\n",
    );
    assert_eq!(report.identity, 73);
    assert_eq!(report.display_path, "private/[REDACTED]");
    assert_eq!(
        report.findings,
        vec![
            ScanFinding::Name {
                component: 2,
                rule: String::from("0")
            },
            ScanFinding::Content {
                line: 2,
                rule: String::from("0")
            },
        ]
    );
    let cached: Scanner = Scanner::load(rules.as_path(), false, true).expect("cached public load");
    assert!(cached.cache_warnings().is_empty());
    let missing: PathBuf = root.join("missing.txt");
    assert!(Scanner::load(missing.as_path(), false, true).is_err());
    assert!(Scanner::load(missing.as_path(), true, true).is_err());
    let builtin: Scanner = Scanner::load(missing.as_path(), true, false)
        .expect("implicit missing runtime file uses shipped baseline");
    assert!(
        builtin
            .scan(74, Path::new("clean.txt"), b"plain")
            .findings
            .is_empty()
    );
    let invalid: PathBuf = root.join("invalid.txt");
    std::fs::write(&invalid, "/PRIVATE_PATTERN_LONG/g\n").expect("invalid rule fixture");
    let error = match Scanner::load(invalid.as_path(), false, true) {
        Ok(_) => panic!("invalid rule must fail loading"),
        Err(failure) => failure,
    };
    assert!(!error.to_string().contains("PRIVATE_PATTERN_LONG"));
    native_rule_path(&root);
}

/// Native rule-file paths retain their original bytes through cache repair's source revalidation.
#[cfg(unix)]
fn native_rule_path(root: &Path) {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let filename: OsString = OsString::from_vec(b"rules-\xff.txt".to_vec());
    let path: PathBuf = root.join(filename);
    std::fs::write(&path, "NATIVE_NEEDLE_LONG\n").expect("native rule file");
    let scanner: Scanner = Scanner::load(path.as_path(), false, true).expect("native rules load");
    let report: CandidateScan = scanner.scan(75, Path::new("clean.txt"), b"NATIVE_NEEDLE_LONG\n");
    assert_eq!(
        report.findings,
        vec![ScanFinding::Content {
            line: 1,
            rule: String::from("0")
        }]
    );
    let cached: Scanner =
        Scanner::load(path.as_path(), false, true).expect("native rules cache reuse");
    assert!(cached.cache_warnings().is_empty());
    eprintln!(
        "{NATIVE_RULE_PATH_CONTROL_RAN}: the rule file kept its bytes through the cache repair"
    );
}

/// This byte-construction control is Unix-specific;
///  other platforms retain the public API controls.
#[cfg(not(unix))]
fn native_rule_path(_root: &Path) {}

/// The actual driver proves the public consumer runs with an isolated cache,
///  not just an early-returning probe.
///  Where `CARGO_TARGET_TMPDIR` refuses a name that is not valid UTF-8,
///  this test re-runs itself in a namespace of its own with a tmpfs there
///  (`fixture_root_taking_any_name`),
/// so the native rule-file control still runs.
#[test]
fn public_embedding_runs_without_real_user_state() {
    let fixture_root: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    // What: ask whether the file system under the build directory takes the byte name the native control writes.
    // Why: where it does not, this half has been replaced by one that mounted a tmpfs there and ran this test.
    #[cfg(unix)]
    if !fixture_root_taking_any_name::in_a_fixture_root_taking_any_name(
        fixture_root.as_path(),
        THE_DRIVER_CONTROL,
    ) {
        return;
    }
    let root: PathBuf = fixture_root.join(format!("scanner-embedded-{}", std::process::id()));
    std::fs::create_dir(&root).expect("fresh disposable fixture");
    std::fs::write(root.join("rules.txt"), "EMBEDDED_NEEDLE_LONG\n").expect("runtime rules");
    let executable: PathBuf = std::env::current_exe().expect("test consumer executable");
    let output = Command::new(executable)
        .args(["--exact", "embedded_consumer_probe", "--nocapture"])
        .env("FORBIDDEN_STRINGS_EMBEDDED_FIXTURE", &root)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", root.join("cache"))
        .env("HOME", &root)
        .output()
        .expect("spawn isolated public consumer");
    let said: Cow<'_, str> = String::from_utf8_lossy(output.stdout.as_slice());
    let heard: Cow<'_, str> = String::from_utf8_lossy(output.stderr.as_slice());
    std::fs::remove_dir_all(&root).expect("remove only this fixture");
    assert!(output.status.success(), "{said}\n{heard}");
    assert!(
        said.contains("1 passed"),
        "the child ran no single passing test:\n{said}\n{heard}"
    );
    #[cfg(unix)]
    assert!(
        heard.contains(NATIVE_RULE_PATH_CONTROL_RAN),
        "the native rule-file control did not run in the child, so a file system that refuses its name would go \
         unnoticed:\n{said}\n{heard}"
    );
}
