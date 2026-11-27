//!  Binary-boundary tests for the rules list:
//!  `--rules` and `FORBIDDEN_STRINGS_RULES` naming several files,
//!  the optional marker,
//!  and the cache keyed over the joined text.
//!  Every fixture is a disposable directory with its own cache root;
//!  no test reads the real environment.
//!
//!  The suite outgrew one readable file,
//!  so the cases sit in the three role-named modules of this folder.
//!  This file holds the fixture harness every one of them calls,
//!  then lists the parts in the order they build on it.
//!  Each part needs `#[path]` because a crate root always looks for its
//!  modules beside itself,
//!  in `tests/`,
//!  not in this folder.

/// Imports ordinary filesystem operations for isolated fixtures.
use std::fs;
/// Imports the absolute path type for disposable fixtures.
use std::path::{
    Path,
    PathBuf,
};
/// Imports the process runner to exercise the actual CLI.
use std::process::{
    Command,
    Output,
};

/// Binary artifact built by Cargo for this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");

/// Creates a fresh fixture root for one test.
fn fixture(label: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "rules-list-{}-{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create fixture root");
    return dir;
}

/// Starts a scanner with a disposable cache root and no inherited rules variable.
fn scanner(root: &Path) -> Command {
    let mut command = Command::new(BIN);
    command
        .current_dir(root)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", root.join("cache"))
        .env_remove("FORBIDDEN_STRINGS_RULES");
    return command;
}

/// Writes a file under the fixture root.
fn write(
    root: &Path,
    name: &str,
    body: &str,
) {
    fs::write(root.join(name), body).expect("write fixture file");
}

/// Standard error text of a finished process.
fn stderr_of(output: &Output) -> String {
    return String::from_utf8_lossy(&output.stderr).into_owned();
}

/// Counts the compiled artifacts the fixture's cache holds.
fn artifacts(root: &Path) -> usize {
    let cache = root.join("cache");
    if !cache.exists() {
        return 0;
    }
    let mut count = 0;
    let mut pending = vec![cache];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).expect("read cache directory") {
            let path = entry.expect("cache entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .file_name()
                .is_some_and(|name| return name == "rules.bin")
            {
                count += 1;
            }
        }
    }
    return count;
}

/// Two tail-format files and one target holding both files' needles.
fn two_file_fixture(label: &str) -> PathBuf {
    let root = fixture(label);
    write(
        &root,
        "shared.txt",
        "==> qqq-shared <==\nSHARED_NEEDLE_LONG_ENOUGH\n",
    );
    write(
        &root,
        "local.txt",
        "==> qqq-local <==\nLOCAL_NEEDLE_LONG_ENOUGH\n",
    );
    write(
        &root,
        "target.txt",
        "SHARED_NEEDLE_LONG_ENOUGH\nclean\nLOCAL_NEEDLE_LONG_ENOUGH\n",
    );
    return root;
}

/// What:
///  `#[path = "rules_list/cache_and_compile_tests.rs"] mod
///  cache_and_compile_tests;` registers the module covering the artifact key
///  and the two commands that fill the cache.
/// Why:
///  A key that ignored one listed file would serve rules the user no longer
///  wrote.
#[path = "rules_list/cache_and_compile_tests.rs"]
mod cache_and_compile_tests;
/// What:
///  `#[path = "rules_list/entry_syntax_tests.rs"] mod entry_syntax_tests;`
///  registers the module that pins the list grammar and its exit-2 refusals.
/// Why:
///  The grammar is the whole definition;
///  a different reading scans different
///  rules.
#[path = "rules_list/entry_syntax_tests.rs"]
mod entry_syntax_tests;
/// What:
///  `#[path = "rules_list/format_agreement_tests.rs"] mod
///  format_agreement_tests;` registers the module that pins what several
///  listed files must agree on.
/// Why:
///  The files join into one text and compile once,
///  so a disagreement that
///  slipped through would misattribute findings.
#[path = "rules_list/format_agreement_tests.rs"]
mod format_agreement_tests;
