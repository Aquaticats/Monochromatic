//! Binary-boundary tests for the rules list: `--rules` and `FORBIDDEN_STRINGS_RULES` naming several files,
//! the optional marker, and the cache keyed over the joined text.
//! Every fixture is a disposable directory with its own cache root; no test reads the real environment.

/// Imports ordinary filesystem operations for isolated fixtures.
use std::fs;
/// Imports the absolute path type for disposable fixtures.
use std::path::{Path, PathBuf};
/// Imports the process runner to exercise the actual CLI.
use std::process::{Command, Output};

/// Binary artifact built by Cargo for this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");

/// Creates a fresh fixture root for one test.
fn fixture(label: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("rules-list-{}-{}", label, std::process::id()));
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
fn write(root: &Path, name: &str, body: &str) {
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
            } else if path.file_name().is_some_and(|name| return name == "rules.bin") {
                count += 1;
            }
        }
    }
    return count;
}

/// Two tail-format files and one target holding both files' needles.
fn two_file_fixture(label: &str) -> PathBuf {
    let root = fixture(label);
    write(&root, "shared.txt", "==> qqq-shared <==\nSHARED_NEEDLE_LONG_ENOUGH\n");
    write(&root, "local.txt", "==> qqq-local <==\nLOCAL_NEEDLE_LONG_ENOUGH\n");
    write(&root, "target.txt", "SHARED_NEEDLE_LONG_ENOUGH\nclean\nLOCAL_NEEDLE_LONG_ENOUGH\n");
    return root;
}

/// A single path behaves exactly as before lists existed.
#[test]
fn a_single_path_still_names_one_required_file() {
    let root = two_file_fixture("single");
    let output = scanner(&root).args(["--rules", "shared.txt", "target.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("target.txt:1 rule=qqq-shared"), "{stderr}");
    assert!(!stderr.contains("qqq-local"), "{stderr}");
    assert!(!stderr.contains("optional rules file"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// Both files' rules apply, in list order, as one rule set.
#[test]
fn two_files_scan_as_one_rule_set() {
    let root = two_file_fixture("two");
    let output = scanner(&root).args(["--rules", "shared.txt,local.txt", "target.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("target.txt:1 rule=qqq-shared"), "{stderr}");
    assert!(stderr.contains("target.txt:3 rule=qqq-local"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// The variable carries the same list, and the flag replaces it instead of adding to it.
#[test]
fn the_variable_carries_a_list_and_the_flag_replaces_it() {
    let root = two_file_fixture("variable");
    let from_variable = scanner(&root)
        .env("FORBIDDEN_STRINGS_RULES", "shared.txt,local.txt")
        .arg("target.txt")
        .output()
        .expect("run");
    let variable_stderr = stderr_of(&from_variable);
    assert_eq!(from_variable.status.code(), Some(1), "{variable_stderr}");
    assert!(variable_stderr.contains("rule=qqq-shared") && variable_stderr.contains("rule=qqq-local"), "{variable_stderr}");

    let replaced = scanner(&root)
        .env("FORBIDDEN_STRINGS_RULES", "shared.txt,local.txt")
        .args(["--rules", "local.txt", "target.txt"])
        .output()
        .expect("run");
    let replaced_stderr = stderr_of(&replaced);
    assert_eq!(replaced.status.code(), Some(1), "{replaced_stderr}");
    assert!(replaced_stderr.contains("rule=qqq-local"), "{replaced_stderr}");
    assert!(!replaced_stderr.contains("rule=qqq-shared"), "the flag must not merge with the variable: {replaced_stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A present optional entry is read like a required one.
#[test]
fn a_present_optional_entry_applies() {
    let root = two_file_fixture("optional-present");
    let output = scanner(&root).args(["--rules", "shared.txt,local.txt?", "target.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("rule=qqq-local"), "{stderr}");
    assert!(!stderr.contains("optional rules file"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// An absent optional entry is skipped and logged, and the scan still passes or fails on the rest.
#[test]
fn an_absent_optional_entry_is_skipped_and_logged() {
    let root = two_file_fixture("optional-absent");
    write(&root, "clean.txt", "nothing to see\n");
    let clean = scanner(&root).args(["--rules", "shared.txt,missing.txt?", "clean.txt"]).output().expect("run");
    let clean_stderr = stderr_of(&clean);
    assert_eq!(clean.status.code(), Some(0), "{clean_stderr}");
    assert!(clean_stderr.contains("optional rules file is absent; skipped"), "{clean_stderr}");
    assert!(clean_stderr.contains("missing.txt"), "{clean_stderr}");

    let dirty = scanner(&root).args(["--rules", "shared.txt,missing.txt?", "target.txt"]).output().expect("run");
    let dirty_stderr = stderr_of(&dirty);
    assert_eq!(dirty.status.code(), Some(1), "{dirty_stderr}");
    assert!(dirty_stderr.contains("rule=qqq-shared"), "{dirty_stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A missing required entry exits 2 and names its path, whatever precedes it.
#[test]
fn a_missing_required_entry_exits_2_naming_its_path() {
    let root = two_file_fixture("required-missing");
    let output = scanner(&root).args(["--rules", "shared.txt,gone.txt", "target.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("read rules gone.txt"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// Every malformed list exits 2 before any file is read.
#[test]
fn malformed_lists_exit_2() {
    let root = two_file_fixture("malformed");
    for (value, expected) in [
        ("shared.txt,,local.txt", "entry 2 names no file"),
        ("shared.txt,", "entry 2 names no file"),
        (",shared.txt", "entry 1 names no file"),
        ("?", "entry 1 names no file"),
        ("shared.txt,shared.txt?", "entry 2 names the same file as entry 1"),
        ("", "the rules list is empty"),
    ] {
        let output = scanner(&root).args(["--rules", value, "target.txt"]).output().expect("run");
        let stderr = stderr_of(&output);
        assert_eq!(output.status.code(), Some(2), "{value:?}: {stderr}");
        assert!(stderr.contains("invalid rules list"), "{value:?}: {stderr}");
        assert!(stderr.contains(expected), "{value:?}: {stderr}");
    }
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A path ending in a question mark is named as optional by doubling the mark.
#[test]
fn a_doubled_question_mark_names_a_path_that_ends_in_one() {
    let root = two_file_fixture("doubled");
    write(&root, "odd?.txt?", "==> qqq-odd <==\nODD_NEEDLE_LONG_ENOUGH\n");
    write(&root, "odd-target.txt", "ODD_NEEDLE_LONG_ENOUGH\n");
    let output = scanner(&root)
        .args(["--rules", "shared.txt,odd?.txt??", "odd-target.txt"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("rule=qqq-odd"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A comma separates entries, so a path holding one is not found as one file.
#[test]
fn a_comma_in_a_path_splits_the_entry() {
    let root = two_file_fixture("comma");
    write(&root, "a,b.txt", "==> qqq-comma <==\nCOMMA_NEEDLE_LONG_ENOUGH\n");
    let output = scanner(&root).args(["--rules", "a,b.txt", "target.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("read rules a:"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A list with no file present fails closed, and the baseline can stand in for them only when asked for.
#[test]
fn only_absent_optional_entries_fail_unless_the_baseline_is_on() {
    let root = two_file_fixture("all-absent");
    write(&root, "clean.txt", "nothing to see\n");
    let refused = scanner(&root).args(["--rules", "a.txt?,b.txt?", "clean.txt"]).output().expect("run");
    let refused_stderr = stderr_of(&refused);
    assert_eq!(refused.status.code(), Some(2), "{refused_stderr}");
    assert!(refused_stderr.contains("no rules loaded"), "{refused_stderr}");
    assert!(refused_stderr.contains("a.txt") && refused_stderr.contains("b.txt"), "{refused_stderr}");

    let with_baseline = scanner(&root)
        .args(["--builtin-rules", "--rules", "a.txt?,b.txt?", "clean.txt"])
        .output()
        .expect("run");
    let baseline_stderr = stderr_of(&with_baseline);
    assert_eq!(with_baseline.status.code(), Some(0), "{baseline_stderr}");
    assert!(baseline_stderr.contains("a.txt") && baseline_stderr.contains("b.txt"), "{baseline_stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// Files in different formats are refused, naming both.
#[test]
fn files_in_different_formats_are_refused() {
    let root = two_file_fixture("mixed");
    write(&root, "legacy.txt", "LEGACY_NEEDLE_LONG_ENOUGH\n");
    let output = scanner(&root).args(["--rules", "shared.txt,legacy.txt", "target.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("legacy.txt") && stderr.contains("shared.txt"), "{stderr}");
    assert!(stderr.contains("one format"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A section name repeated across files exits 2 and names both files.
#[test]
fn a_section_declared_in_two_files_is_refused() {
    let root = two_file_fixture("duplicate");
    write(&root, "other.txt", "==> qqq-shared <==\nOTHER_NEEDLE_LONG_ENOUGH\n");
    let output = scanner(&root).args(["--rules", "shared.txt,other.txt", "target.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("other.txt") && stderr.contains("shared.txt"), "{stderr}");
    assert!(stderr.contains("qqq-shared"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A BOM and a missing final newline in the first file leave the second file's rules intact.
#[test]
fn a_bom_and_a_missing_final_newline_do_not_disturb_the_next_file() {
    let root = fixture("join-edges");
    write(&root, "first.txt", "==> qqq-first <==\nFIRST_NEEDLE_LONG_ENOUGH");
    write(&root, "second.txt", "\u{feff}==> qqq-second <==\nSECOND_NEEDLE_LONG_ENOUGH");
    write(&root, "target.txt", "FIRST_NEEDLE_LONG_ENOUGH\nSECOND_NEEDLE_LONG_ENOUGH\n");
    let output = scanner(&root).args(["--rules", "first.txt,second.txt", "target.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("target.txt:1 rule=qqq-first"), "{stderr}");
    assert!(stderr.contains("target.txt:2 rule=qqq-second"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// The compiled artifact is reused while the list's text is unchanged and missed when any listed file changes.
#[test]
fn the_cache_follows_the_joined_text() {
    let root = two_file_fixture("cache");
    let run = || {
        return scanner(&root)
            .args(["--rules", "shared.txt,local.txt?", "target.txt"])
            .output()
            .expect("run");
    };
    let first = stderr_of(&run());
    assert!(first.contains("\"reason\":\"missing\""), "{first}");
    assert_eq!(artifacts(&root), 1);

    let second = stderr_of(&run());
    assert!(!second.contains("cache-warning"), "unchanged list must hit: {second}");
    assert_eq!(artifacts(&root), 1);

    write(&root, "local.txt", "==> qqq-local <==\nLOCAL_NEEDLE_LONG_ENOUGH\n# edited\n");
    let edited = stderr_of(&run());
    assert!(edited.contains("\"reason\":\"missing\""), "an edited second file must miss: {edited}");
    assert_eq!(artifacts(&root), 2);

    fs::remove_file(root.join("local.txt")).expect("remove optional file");
    let removed = run();
    let removed_stderr = stderr_of(&removed);
    assert!(removed_stderr.contains("optional rules file is absent; skipped"), "{removed_stderr}");
    assert!(removed_stderr.contains("\"reason\":\"missing\""), "a removed optional file must miss: {removed_stderr}");
    assert!(!removed_stderr.contains("rule=qqq-local"), "{removed_stderr}");
    assert_eq!(artifacts(&root), 3);

    write(&root, "local.txt", "==> qqq-local <==\nLOCAL_NEEDLE_LONG_ENOUGH\n");
    let restored = stderr_of(&run());
    assert!(!restored.contains("cache-warning"), "restored text must hit its earlier artifact: {restored}");
    assert!(restored.contains("rule=qqq-local"), "{restored}");
    assert_eq!(artifacts(&root), 3);
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A changed file order is a changed text and misses the cache.
#[test]
fn a_changed_order_is_a_cache_miss() {
    let root = two_file_fixture("order");
    let forward = scanner(&root).args(["--rules", "shared.txt,local.txt", "target.txt"]).output().expect("run");
    assert!(stderr_of(&forward).contains("\"reason\":\"missing\""));
    let reverse = scanner(&root).args(["--rules", "local.txt,shared.txt", "target.txt"]).output().expect("run");
    assert!(stderr_of(&reverse).contains("\"reason\":\"missing\""));
    assert_eq!(artifacts(&root), 2);
    fs::remove_dir_all(root).expect("remove fixture");
}

/// `compile-rules` takes the same list, warms the artifact a scan then reuses, and fails on a missing required entry.
#[test]
fn compile_rules_takes_a_list() {
    let root = two_file_fixture("compile");
    let compiled = scanner(&root)
        .args(["compile-rules", "--rules", "shared.txt,local.txt,absent.txt?"])
        .output()
        .expect("run");
    let compile_stderr = stderr_of(&compiled);
    assert!(compiled.status.success(), "{compile_stderr}");
    assert!(compiled.stdout.is_empty());
    assert!(compile_stderr.contains("absent.txt"), "{compile_stderr}");
    assert_eq!(artifacts(&root), 1);

    let scan = scanner(&root).args(["--rules", "shared.txt,local.txt", "target.txt"]).output().expect("run");
    let scan_stderr = stderr_of(&scan);
    assert_eq!(scan.status.code(), Some(1), "{scan_stderr}");
    assert!(!scan_stderr.contains("cache-warning"), "the scan must reuse the compiled artifact: {scan_stderr}");

    let missing = scanner(&root).args(["compile-rules", "--rules", "shared.txt,gone.txt"]).output().expect("run");
    assert_eq!(missing.status.code(), Some(2));
    assert!(stderr_of(&missing).contains("read rules gone.txt"), "{}", stderr_of(&missing));

    let malformed = scanner(&root).args(["compile-rules", "--rules", "shared.txt,,local.txt"]).output().expect("run");
    assert_eq!(malformed.status.code(), Some(2));

    let none = scanner(&root).args(["compile-rules", "--rules", "a.txt?,b.txt?"]).output().expect("run");
    assert_eq!(none.status.code(), Some(2));
    assert!(stderr_of(&none).contains("no rules loaded"), "{}", stderr_of(&none));
    fs::remove_dir_all(root).expect("remove fixture");
}

/// `--all` skips every listed rules file, not only the first, so a tracked second file does not self-match.
#[test]
fn all_mode_skips_every_listed_rules_file() {
    let root = two_file_fixture("all");
    fs::remove_file(root.join("target.txt")).expect("remove target");
    let git = if Path::new("/usr/bin/git").exists() { "/usr/bin/git" } else { "git" };
    let init = Command::new(git).args(["init", "-q"]).current_dir(&root).status().expect("init");
    assert!(init.success());
    let add = Command::new(git).args(["add", "shared.txt", "local.txt"]).current_dir(&root).status().expect("add");
    assert!(add.success());
    // The cache lives outside the walked tree, so only the rules files could self-match.
    let cache_root = fixture("all-cache");
    let output = scanner(&root)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache_root)
        .args(["--rules", "shared.txt,local.txt", "--all"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(0), "both rules files must be skipped by --all: {stderr}");
    fs::remove_dir_all(cache_root).expect("remove cache fixture");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A listed file that is not UTF-8 exits 2 naming its path.
#[test]
fn a_listed_file_that_is_not_utf8_exits_2() {
    let root = two_file_fixture("not-utf8");
    fs::write(root.join("bytes.txt"), [0xff, 0xfe, 0xfd]).expect("write bytes");
    let output = scanner(&root).args(["--rules", "shared.txt,bytes.txt", "target.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("bytes.txt") && stderr.contains("valid UTF-8"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// The implicit default file may still be absent under the baseline, with no log line for it.
#[test]
fn the_implicit_default_stays_quiet_when_absent() {
    let root = fixture("implicit-default");
    write(&root, "clean.txt", "nothing to see\n");
    let output = scanner(&root).args(["--builtin-rules", "clean.txt"]).output().expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert!(!stderr.contains("optional rules file"), "{stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}
