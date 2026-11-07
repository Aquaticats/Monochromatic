//! Binary-boundary tests for the `url-userinfo-credential` rule of the repository's shared appendix.
//!
//! What:
//! the scanner runs with the real `forbidden-strings.append.txt` of the repository against fixture files.
//! Why:
//! a credential inside a URL (`scheme://user:password@host`) is copied by every file,
//! Git config value and log
//! that holds the URL,
//! and `git-lfs` once echoed such a URL into thousands of crash logs;
//! the rule must fire on
//! passwords and stay quiet on placeholders,
//! plain URLs and account-only SSH URLs,
//! or it blocks legitimate commits.
//! Every credential below is a made-up value assembled in this file.

/// Imports ordinary filesystem operations for isolated fixtures.
use std::fs;
/// Imports the absolute path type for disposable fixtures.
use std::path::{Path, PathBuf};
/// Imports the process runner to exercise the actual CLI.
use std::process::{Command, Output};

/// Binary artifact built by Cargo for this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");

/// Name the scanner reports for the rule under test.
const RULE_NAME: &str = "url-userinfo-credential";

/// Made-up password long enough for the rule's minimum.
const FAKE_PASSWORD: &str = "fake-password-for-tests";

/// Repository shared appendix,
/// relative to this package.
const APPENDIX_FROM_PACKAGE: &str = "../../../forbidden-strings.append.txt";

/// Fixture file the scanner is asked to scan.
const SCANNED: &str = "scanned.txt";

/// Creates a fresh repository-shaped root that holds a copy of the real appendix as its rules file.
fn fixture(label: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("appendix-userinfo-{}-{}", label, std::process::id()));
    fs::create_dir_all(root.join(".git")).expect("create repository marker");
    let appendix = Path::new(env!("CARGO_MANIFEST_DIR")).join(APPENDIX_FROM_PACKAGE);
    fs::copy(appendix, root.join("rules.txt")).expect("copy the repository appendix");
    return root;
}

/// Runtime cache directory beside the fixture root,
/// outside it so a walk never reads the cache files.
fn cache_directory(root: &Path) -> PathBuf {
    let mut name = root.file_name().expect("fixture root has a name").to_os_string();
    name.push("-cache");
    return root.with_file_name(name);
}

/// Scans one line of text with the appendix and returns the scanner output.
fn scan_line(label: &str, line: &str) -> Output {
    let root = fixture(label);
    fs::write(root.join(SCANNED), format!("{line}\n")).expect("write scanned file");
    let output = Command::new(BIN)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", cache_directory(&root))
        .current_dir(&root)
        .args(["--rules", "rules.txt", SCANNED])
        .output()
        .expect("run scanner");
    let cache = cache_directory(&root);
    if cache.exists() {
        fs::remove_dir_all(cache).expect("remove runtime cache");
    }
    fs::remove_dir_all(root).expect("remove fixture");
    return output;
}

/// Whether the scanner reported the rule under test for the line.
fn reports_rule(label: &str, line: &str) -> bool {
    let output = scan_line(label, line);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let hit = stderr.contains(RULE_NAME);
    if hit {
        assert_eq!(output.status.code(), Some(1), "{stderr}");
        assert!(!stderr.contains(FAKE_PASSWORD), "the finding must not print the matched text: {stderr}");
    }
    return hit;
}

/// A password in the userinfo of a web URL is reported,
/// and the finding never prints it.
#[test]
fn password_in_a_web_url_is_reported() {
    assert!(reports_rule("web", &format!("\turl = https://lfs:{FAKE_PASSWORD}@lfs.example.test")));
}

/// The same shape on another scheme is reported.
#[test]
fn password_in_an_ssh_url_is_reported() {
    assert!(reports_rule("ssh", &format!("ssh://git:{FAKE_PASSWORD}@host.example.test/repo.git")));
}

/// The Git config key `git-lfs` writes for a credentialed endpoint is reported.
#[test]
fn key_name_that_embeds_the_credential_is_reported() {
    assert!(reports_rule("key", &format!("[lfs \"https://lfs:{FAKE_PASSWORD}@lfs.example.test\"]")));
}

/// A placeholder in angle brackets,
/// as documentation writes it,
/// is not reported.
#[test]
fn angle_bracket_placeholder_is_not_reported() {
    assert!(!reports_rule("angle", "url = https://lfs:<LFS_WRITE_TOKEN>@lfs.example.test"));
}

/// A shell or template placeholder in braces is not reported.
#[test]
fn brace_placeholder_is_not_reported() {
    assert!(!reports_rule("brace", "url = https://lfs:${LFS_WRITE_TOKEN}@lfs.example.test"));
}

/// A plain URL is not reported.
#[test]
fn plain_url_is_not_reported() {
    assert!(!reports_rule("plain", "url = https://lfs.example.test/objects/batch"));
}

/// A user name alone on SSH names an account,
/// not a secret,
/// and is not reported.
#[test]
fn account_only_ssh_url_is_not_reported() {
    assert!(!reports_rule("account", "url = ssh://git@lfs.example.test/repo.git"));
}

/// An `@` in the path of a URL is not userinfo and is not reported.
#[test]
fn at_sign_in_a_path_is_not_reported() {
    assert!(!reports_rule("path", "https://lfs.example.test/a:long-enough-segment-here@b"));
}

/// A password shorter than the rule's minimum is not reported,
/// which keeps documentation examples quiet.
#[test]
fn short_example_password_is_not_reported() {
    assert!(!reports_rule("short", "https://user:pass@lfs.example.test"));
}
