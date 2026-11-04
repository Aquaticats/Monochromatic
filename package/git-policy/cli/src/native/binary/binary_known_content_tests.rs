//! Binary-level controls for the forbidden-strings rule that content already in a parent commit's tree is not
//!  scanned again, driven through a real `git commit` of a rename.
//! Every control runs the built executable on a disposable repository with the host's Git.

use super::commit::{git_text, remove_settled, run_commit, wrapped_commit};
use super::support::{Fixture, Observed, fixture, git, observe, repository, stderr_of};
use std::ffi::OsStr;
use std::path::PathBuf;

/// A rename of a file that already holds a forbidden string lands, and the decision is logged with the masked
///  pathname, the blob and the parent commit; the same rename with one byte changed is stopped and nothing lands.
#[test]
fn a_rename_of_known_content_lands_and_a_rename_with_a_changed_byte_does_not() {
    let fixture: Fixture = fixture("known-content-rename");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let needle: String = ["PLANTED", "KNOWN", "NEEDLE"].join("_");
    let body: String = format!("first\n{needle}\n");
    std::fs::write(repo.join("a.txt"), body.as_bytes()).expect("known file");
    std::fs::create_dir(repo.join("rules")).expect("rules directory");
    std::fs::write(repo.join("rules/private.txt"), format!("{needle}\n")).expect("rules");
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        "{ \"policies\": { \"security/forbidden-strings\": [\"error\", { \"builtinRules\": false, \"rulesFile\": \"rules/private.txt\" }] } }\n",
    )
    .expect("configuration");
    git(&fixture, repo.as_path(), &["add", "--all"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    let base: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let blob: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD:a.txt"]).trim().to_owned();
    // A pure rename.
    std::fs::create_dir(repo.join("moved")).expect("moved directory");
    git(&fixture, repo.as_path(), &["mv", "a.txt", "moved/a.txt"]);
    let landed: Observed = observe(
        wrapped_commit(&fixture, repo.as_path(), &["commit", "--no-only", "--message=rename"])
            .env("MONOCHROMATIC_VERBOSE", "true"),
        b"",
    );
    let text: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "{text}");
    let decision: String = format!(
        "skipped the content of moved/a.txt: blob {blob} is already in the tree of commit {base}; only the pathname was checked."
    );
    assert!(text.contains(decision.as_str()), "{text}");
    assert!(!text.contains(needle.as_str()), "{text}");
    assert_ne!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), base);
    // The same rename with one byte changed is new content, so it is scanned and stops the commit.
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    std::fs::create_dir(repo.join("again")).expect("again directory");
    git(&fixture, repo.as_path(), &["mv", "moved/a.txt", "again/a.txt"]);
    std::fs::write(repo.join("again/a.txt"), body.replacen("first", "firsT", 1).as_bytes()).expect("edit");
    git(&fixture, repo.as_path(), &["add", "--", "again/a.txt"]);
    let stopped: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--no-only", "--message=edit"]);
    let stopped_text: String = stderr_of(&stopped);
    assert_eq!(stopped.code, Some(1), "{stopped_text}");
    assert!(stopped_text.contains("security/forbidden-strings/forbidden-string"), "{stopped_text}");
    assert!(stopped_text.contains("Forbidden string matched at line 2"), "{stopped_text}");
    assert!(!stopped_text.contains(needle.as_str()), "{stopped_text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), head);
    remove_settled(&fixture);
}
