//! Binary-boundary pathname tests with disposable inputs and rule files.

/// Imports ordinary filesystem operations for isolated fixtures.
use std::fs;
/// Imports the absolute path type for disposable fixtures.
use std::path::PathBuf;
/// Imports the process runner to exercise the actual CLI.
use std::process::Command;

/// Binary artifact built by Cargo for this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");

/// Creates a new temporary root for each test process.
fn fixture(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("path-names-{}-{}", label, std::process::id()));
    fs::create_dir_all(&dir).expect("create test directory");
    return dir;
}

/// Starts a scanner with a test-owned disposable runtime cache.
fn scanner(root: &std::path::Path) -> Command {
    let mut command = Command::new(BIN);
    command.env("FORBIDDEN_STRINGS_CACHE_DIR", root.join("cache"));
    return command;
}

/// Exercises name-only,
///  content-only,
///  and combined redaction at the CLI boundary.
#[test]
fn pathname_hit_masks_the_name_in_every_finding() {
    let root = fixture("name-and-content");
    let rules = root.join("rules.txt");
    fs::write(&rules, "VAULTTOKEN_LONG\n").expect("write rules");
    let source = root.join("content-only.txt");
    fs::write(&source, "VAULTTOKEN_LONG\n").expect("write candidate bytes");
    let output = scanner(&root)
        .current_dir(&root)
        .args(["--rules", "rules.txt", "--name-path", "private/VAULTTOKEN_LONG.txt"])
        .arg(&source)
        .output()
        .expect("run scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("private/[REDACTED]:name:2 rule=0 input=0"), "{stderr}");
    assert!(stderr.contains("private/[REDACTED]:1 rule=0 input=0"), "{stderr}");
    assert!(!stderr.contains("VAULTTOKEN_LONG"), "matched name leaked: {stderr}");
    assert!(!stderr.contains("content-only.txt"), "temporary path leaked: {stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// The walker picks a real forbidden directory name even when file content is clean.
#[test]
fn walked_directory_name_fails_with_clean_content() {
    let root = fixture("walked-directory");
    fs::create_dir(root.join(".git")).expect("create Git marker");
    fs::write(root.join("rules.txt"), "VAULTTOKEN_LONG\n").expect("write rules");
    let directory = root.join("VAULTTOKEN_LONG");
    fs::create_dir(&directory).expect("create forbidden directory");
    fs::write(directory.join("clean.txt"), "harmless\n").expect("write clean file");
    let output = scanner(&root)
        .current_dir(&root)
        .args(["--rules", "rules.txt", "--all"])
        .output()
        .expect("run scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("[REDACTED]/clean.txt:name:1 rule=0"), "{stderr}");
    assert!(!stderr.contains("VAULTTOKEN_LONG"), "matched directory leaked: {stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// Every supplied directory name participates outside a Git repository.
#[test]
fn explicit_external_parent_is_scanned() {
    let root = fixture("external-parent");
    fs::write(root.join("rules.txt"), "VAULTTOKEN_LONG\n").expect("write rules");
    let parent = root.join("VAULTTOKEN_LONG");
    fs::create_dir(&parent).expect("create external directory");
    let file = parent.join("clean.txt");
    fs::write(&file, "harmless\n").expect("write clean file");
    let output = scanner(&root)
        .current_dir(&root)
        .args(["--rules", "rules.txt"])
        .arg(&file)
        .output()
        .expect("run scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("[REDACTED]/clean.txt:name:"), "{stderr}");
    assert!(!stderr.contains("VAULTTOKEN_LONG"), "external path leaked: {stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// An invalid logical-path mapping exits as a usage error rather than skipping files.
#[test]
fn name_override_count_must_equal_content_file_count() {
    let root = fixture("mapping-count");
    fs::write(root.join("rules.txt"), "VAULTTOKEN_LONG\n").expect("write rules");
    let output = scanner(&root)
        .current_dir(&root)
        .args(["--rules", "rules.txt", "--name-path", "one.txt"])
        .output()
        .expect("run scanner");
    assert_eq!(output.status.code(), Some(2));
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A real filename is scanned even when no content bytes exist.
#[test]
fn actual_filename_matches_in_empty_file() {
    let root = fixture("empty-filename");
    fs::create_dir(root.join(".git")).expect("create repository marker");
    fs::write(root.join("rules.txt"), "VAULTTOKEN_LONG\n").expect("write rules");
    let file = root.join("VAULTTOKEN_LONG.txt");
    fs::write(&file, b"").expect("create empty file");
    let output = scanner(&root)
        .current_dir(&root)
        .args(["--rules", "rules.txt"])
        .arg(&file)
        .output()
        .expect("run scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("[REDACTED]:name:1 rule=0"), "{stderr}");
    assert!(!stderr.contains("VAULTTOKEN_LONG"), "filename leaked: {stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// Distinct candidate operands remain attributable after identical masking.
#[test]
fn masked_override_paths_keep_separate_operand_indexes() {
    let root = fixture("identical-masks");
    fs::write(root.join("rules.txt"), "VAULTTOKEN_LONG\n\nSECOND_SECRET_LONG\n")
        .expect("write rules");
    let first = root.join("first");
    let second = root.join("second");
    fs::write(&first, "clean").expect("write first");
    fs::write(&second, "clean").expect("write second");
    let output = scanner(&root)
        .current_dir(&root)
        .args(["--rules", "rules.txt", "--name-path", "VAULTTOKEN_LONG", "--name-path", "SECOND_SECRET_LONG"])
        .arg(&first)
        .arg(&second)
        .output()
        .expect("run scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("[REDACTED]:name:1 rule=0 input=0"), "{stderr}");
    assert!(stderr.contains("[REDACTED]:name:1 rule=1 input=1"), "{stderr}");
    assert!(!stderr.contains("VAULTTOKEN_LONG"), "first name leaked: {stderr}");
    assert!(!stderr.contains("SECOND_SECRET_LONG"), "second name leaked: {stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A missing input still reports the forbidden name without printing it.
#[test]
fn unreadable_named_input_fails_closed_and_redacts_both_findings() {
    let root = fixture("missing-named-input");
    fs::write(root.join("rules.txt"), "VAULTTOKEN_LONG\n").expect("write rules");
    let output = scanner(&root)
        .current_dir(&root)
        .args(["--rules", "rules.txt", "--name-path", "VAULTTOKEN_LONG.txt"])
        .arg(root.join("missing"))
        .output()
        .expect("run scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("[REDACTED]:name:1 rule=0 input=0"), "{stderr}");
    assert!(stderr.contains("[REDACTED]: read error:"), "{stderr}");
    assert!(!stderr.contains("VAULTTOKEN_LONG"), "missing name leaked: {stderr}");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A synthetic logical path and `--all` cannot be combined.
#[test]
fn override_is_rejected_in_walker_mode() {
    let root = fixture("mapping-walker");
    fs::write(root.join("rules.txt"), "VAULTTOKEN_LONG\n").expect("write rules");
    let output = scanner(&root)
        .current_dir(&root)
        .args(["--rules", "rules.txt", "--all", "--name-path", "one.txt"])
        .output()
        .expect("run scanner");
    assert_eq!(output.status.code(), Some(2));
    fs::remove_dir_all(root).expect("remove fixture");
}
