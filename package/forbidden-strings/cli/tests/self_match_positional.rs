//! Binary-boundary tests for the generated rule-source files when they are named as positional arguments.
//!
//! What:
//! CI hands the scanner the changed files positionally
//! (`forbidden-strings --builtin-rules --rules ... <changed>...`).
//! Why:
//! the three generated rule-source files hold literal copies of rule bodies,
//! so they always match their own
//! rules.
//! The `--all` walk skips them;
//! the positional path skipped only the `forbidden-strings.*.txt` files at the
//! working directory,
//! so a push that renamed or edited a rule source failed the workflow on every later push
//! (the validated-tree watermark only advances on success).

/// Imports ordinary filesystem operations for isolated fixtures.
use std::fs;
/// Imports the absolute path type for disposable fixtures.
use std::path::{Path, PathBuf};
/// Imports the process runner to exercise the actual CLI.
use std::process::{Command, Output};

/// Binary artifact built by Cargo for this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");

/// Literal the fixture rules file forbids;
/// it appears in every fixture file that should hit.
const SENTINEL: &str = "VAULTTOKEN_LONG";

/// The three generated rule-source files,
/// relative to the repository root.
const RULE_SOURCES: [&str; 3] = [
    "package/forbidden-strings/cli/data/betterleaks-default-config.toml",
    "package/forbidden-strings/cli/data/builtin-rules.txt",
    "package/forbidden-strings/cli/src/port-betterleaks-relaxations.ts",
];

/// Creates a fresh repository-shaped root holding the rules file and the three rule sources.
fn fixture(label: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("self-match-positional-{}-{}", label, std::process::id()));
    fs::create_dir_all(root.join(".git")).expect("create repository marker");
    fs::write(root.join("rules.txt"), format!("{SENTINEL}\n")).expect("write rules");
    for source in RULE_SOURCES {
        write_file(&root, source, &format!("line {SENTINEL} line\n"));
    }
    return root;
}

/// Writes a file below the fixture root,
/// creating its directories.
fn write_file(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("file has a parent")).expect("create parent directories");
    fs::write(path, text).expect("write fixture file");
}

/// Removes the fixture root and its runtime cache directory.
fn remove_fixture(root: PathBuf) {
    // The cache directory exists only when the scanner compiled rules there.
    let cache = cache_directory(&root);
    if cache.exists() {
        fs::remove_dir_all(cache).expect("remove runtime cache");
    }
    fs::remove_dir_all(root).expect("remove fixture");
}

/// Runtime cache directory beside the fixture root,
/// outside it so an `--all` walk never reads the cache files.
fn cache_directory(root: &Path) -> PathBuf {
    let mut name = root.file_name().expect("fixture root has a name").to_os_string();
    name.push("-cache");
    return root.with_file_name(name);
}

/// Runs the scanner in the fixture root with the fixture rules and a test-owned runtime cache.
fn scan(root: &Path, arguments: &[&str]) -> Output {
    return Command::new(BIN)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", cache_directory(root))
        .current_dir(root)
        .args(["--rules", "rules.txt"])
        .args(arguments)
        .output()
        .expect("run scanner");
}

/// The rule sources named positionally with relative paths,
/// as CI's `git diff --name-only` lists them,
/// pass.
#[test]
fn positional_rule_sources_are_skipped() {
    let root = fixture("relative");
    let output = scan(&root, &RULE_SOURCES);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    remove_fixture(root);
}

/// The same files named by absolute path resolve to the same canonical file and pass.
#[test]
fn positional_rule_sources_are_skipped_by_absolute_path() {
    let root = fixture("absolute");
    let absolute: Vec<String> = RULE_SOURCES
        .iter()
        .map(|source| return root.join(source).to_string_lossy().into_owned())
        .collect();
    let arguments: Vec<&str> = absolute.iter().map(String::as_str).collect();
    let output = scan(&root, &arguments);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    remove_fixture(root);
}

/// A same-named copy elsewhere is a different file and is still scanned (the path anchor stays exact).
#[test]
fn same_named_copy_elsewhere_is_still_scanned() {
    let root = fixture("copy");
    write_file(&root, "vendor/package/forbidden-strings/cli/data/builtin-rules.txt", &format!("{SENTINEL}\n"));
    let output = scan(&root, &["vendor/package/forbidden-strings/cli/data/builtin-rules.txt"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("vendor/package/forbidden-strings/cli/data/builtin-rules.txt:1"), "{stderr}");
    remove_fixture(root);
}

/// Positive control:
/// an ordinary file with a hit,
/// named beside the rule sources,
/// still fails the run.
#[test]
fn ordinary_file_with_a_hit_still_fails_beside_the_skipped_sources() {
    let root = fixture("control");
    write_file(&root, "src/leak.ts", &format!("const value = '{SENTINEL}';\n"));
    let mut arguments = RULE_SOURCES.to_vec();
    arguments.push("src/leak.ts");
    let output = scan(&root, &arguments);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("src/leak.ts:1"), "{stderr}");
    assert!(!stderr.contains("builtin-rules.txt"), "a skipped rule source was reported: {stderr}");
    remove_fixture(root);
}

/// A rules file the user names positionally is still scanned:
/// only the three generated sources are skipped.
#[test]
fn positional_rules_file_is_still_scanned() {
    let root = fixture("rules-file");
    let output = scan(&root, &["rules.txt"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("rules.txt:1"), "{stderr}");
    remove_fixture(root);
}

/// The `--all` walk keeps skipping the three sources and the rules file.
#[test]
fn all_mode_still_skips_the_rule_sources() {
    let root = fixture("walk");
    let output = scan(&root, &["--all"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    remove_fixture(root);
}
