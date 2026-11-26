//! What:
//!  Process isolation and rule fixtures shared by the scanner controls.
//! Why:
//!  Loading runtime rules reads the home directory and writes a per-user rule cache.
//!      A control must never touch the real ones,
//!  and changing this process's
//!      environment would race the other test threads.
//!  Each such control therefore
//!      re-runs itself in a child process whose home and cache are a disposable fixture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // spawnSync(process.execPath, ['--exact', name], { env: { HOME: fixture, FORBIDDEN_STRINGS_CACHE_DIR: `${fixture}/cache` } });
//! ```

/// Shared disposable-directory fixtures and native path and process types.
use crate::test_support::{
    fixture,
    remove,
};
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

/// Set only in the child:
///  names the fixture directory the control may use.
const ISOLATED_FIXTURE_VARIABLE: &str = "CLI_GIT_NATIVE_SCANNER_FIXTURE";

/// What:
///  Run a control's body in a child process with a disposable home and rule cache.
///       `fn(&Path)` is a plain function taking the fixture directory.
/// Why:
///   In the parent this starts the same test binary filtered to exactly this
///       control;
///  in that child the variable is set,
///  so the body runs there.
///  The
///       parent requires that exactly one test ran and passed,
///  which also catches a
///       mistyped control name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runIsolated(testPath: string, name: string, body: (fixture: string) => void): void;
/// ```
pub(crate) fn run_isolated(
    test_path: &str,
    name: &str,
    body: fn(&Path),
) {
    if let Some(directory) = std::env::var_os(ISOLATED_FIXTURE_VARIABLE) {
        body(Path::new(&directory));
        return;
    }
    let root: PathBuf = fixture(name);
    let home: PathBuf = root.join("home");
    std::fs::create_dir(&home).expect("disposable home");
    let work: PathBuf = root.join("work");
    std::fs::create_dir(&work).expect("control directory");
    let cache: PathBuf = root.join("cache");
    let environment: [(&str, &OsStr); 3] = [
        (ISOLATED_FIXTURE_VARIABLE, work.as_os_str()),
        ("HOME", home.as_os_str()),
        ("FORBIDDEN_STRINGS_CACHE_DIR", cache.as_os_str()),
    ];
    // The child's pass is this control's assertion, and its two cache variables must not come in from this process.
    crate::child_test_process::passed(crate::child_test_process::ChildTest {
        exact_test: test_path,
        environment: environment.as_slice(),
        cleared_environment: &["XDG_CACHE_HOME", "FORBIDDEN_STRINGS_RULES"],
        ..crate::child_test_process::ChildTest::default()
    });
    remove(root.as_path());
}

/// What:
///  The planted forbidden token,
///  assembled at run time.
/// Why:
///   This repository's own commit policy scans these sources;
///  a token that only
///       exists once the control runs can never be reported in the source itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needle = ['PLANTED', 'CANDIDATE', 'NEEDLE'].join('_');
/// ```
pub(crate) fn needle() -> String {
    return ["PLANTED", "CANDIDATE", "NEEDLE"].join("_");
}

/// Write a rules file holding the planted token as its only rule,
///  and return its path.
pub(crate) fn rules_file(directory: &Path) -> PathBuf {
    let path: PathBuf = directory.join("rules.txt");
    std::fs::write(&path, format!("{}\n", needle())).expect("rules file");
    return path;
}
