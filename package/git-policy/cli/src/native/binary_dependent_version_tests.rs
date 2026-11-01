//! What: `mono/dependent-version-bump` through the built executable: `git cli-git check`
//!       reports the dependents of a raised manifest, `git cli-git fix` bumps them in the
//!       worktree, and a narrow fix changes an unselected dependent only when it matches
//!       `HEAD`.
//! Why: These are the commands the release workflow will run; the controls read the
//!      events, the worktree and the index exactly as a person would see them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['cli-git', 'fix', '--all']).stdout).toContain('fix-summary');
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, fixture, git, observe, remove, repository, wrapped};
/// Operating-system text for the repository name.
use std::ffi::OsStr;
/// Borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// The captured result of a finished process.
use std::process::Output;

/// The configuration that turns the policy on.
const CONFIGURATION: &str = "{ \"policies\": { \"mono/dependent-version-bump\": \"error\" } }\n";

/// Run a wrapped command in a repository and return what the caller saw.
fn run_wrapped(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    return observe(wrapped(fixture).current_dir(repo).args(arguments), b"");
}

/// A manifest as two-space JSON with a final newline.
fn manifest(name: &str, version: &str, dependency: Option<&str>) -> String {
    let dependencies: String = dependency.map_or_else(String::new, |depended: &str| {
        return format!(",\n  \"dependencies\": {{\n    \"{depended}\": \"workspace:*\"\n  }}");
    });
    return format!(
        "{{\n  \"name\": \"{name}\",\n  \"version\": \"{version}\"{dependencies}\n}}\n"
    );
}

/// Write a file below the repository, creating its directory.
fn write(repo: &Path, path: &str, text: &str) {
    let file: PathBuf = repo.join(path);
    std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
    std::fs::write(file, text).expect("file");
}

/// A committed workspace of `@s/a` and its dependents `@s/b` and `@s/c`, configured for
/// the policy, with `@s/a` raised to `1.1.0` in the worktree.
fn workspace(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    write(
        repo.as_path(),
        "package/module/a/package.json",
        &manifest("@s/a", "1.0.0", None),
    );
    write(
        repo.as_path(),
        "package/module/b/package.json",
        &manifest("@s/b", "2.0.0", Some("@s/a")),
    );
    write(
        repo.as_path(),
        "package/module/c/package.json",
        &manifest("@s/c", "3.0.0", Some("@s/a")),
    );
    write(
        repo.as_path(),
        "package/config/pnpr/config.yaml",
        "packages:\n  - '@s/a'\n  - '@s/b'\n  - '@s/c'\n",
    );
    write(repo.as_path(), "cli-git.config.jsonc", CONFIGURATION);
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=workspace"],
    );
    write(
        repo.as_path(),
        "package/module/a/package.json",
        &manifest("@s/a", "1.1.0", None),
    );
    return repo;
}

/// The stale finding event about dependent `name` at `sequence`.
fn stale(sequence: u64, name: &str, from: &str, to: &str) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"direct-check\",\
         \"policyId\":\"mono/dependent-version-bump\",\"severity\":\"error\",\
         \"code\":\"mono/dependent-version-bump/dependent-version-stale\",\
         \"message\":\"@s/{name} reaches a package bumped in this commit (@s/a); bump it from {from} to {to} in the same commit.\",\
         \"path\":\"package/module/{name}/package.json\",\"fix\":\"available\"}}\n"
    );
}

/// The worktree text of dependent `name`.
fn read(repo: &Path, name: &str) -> String {
    return std::fs::read_to_string(repo.join(format!("package/module/{name}/package.json")))
        .expect("manifest");
}

/// A check reports both dependents with a fix available and exits 1; `fix --all` bumps
/// both in one pass without changing the index; the check is then clean.
#[test]
fn check_reports_and_fix_bumps_every_dependent() {
    let fixture: Fixture = fixture("dependent-version-all");
    let repo: PathBuf = workspace(&fixture);
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["cli-git", "check", "--all"]),
        Observed {
            code: Some(1),
            stdout: format!(
                "{}{}",
                stale(0, "b", "2.0.0", "2.0.1"),
                stale(1, "c", "3.0.0", "3.0.1")
            )
            .into_bytes(),
            stderr: Vec::new(),
        }
    );
    let index_before: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("index");
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["cli-git", "fix", "--all"]),
        Observed {
            code: Some(0),
            stdout: b"{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"direct-fix\",\"passes\":1,\"changedPaths\":[\"package/module/b/package.json\",\"package/module/c/package.json\"]}\n".to_vec(),
            stderr: Vec::new(),
        }
    );
    assert_eq!(
        std::fs::read(repo.join(".git/index")).expect("index"),
        index_before
    );
    assert_eq!(
        read(repo.as_path(), "b"),
        manifest("@s/b", "2.0.1", Some("@s/a"))
    );
    assert_eq!(
        read(repo.as_path(), "c"),
        manifest("@s/c", "3.0.1", Some("@s/a"))
    );
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["cli-git", "check", "--all"]),
        Observed {
            code: Some(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    );
    remove(&fixture);
}

/// A fix of the raised manifest alone bumps the clean dependents it did not select, and
/// refuses one whose worktree copy changed, changing nothing.
#[test]
fn a_narrow_fix_changes_unselected_dependents_only_when_they_match_head() {
    let fixture: Fixture = fixture("dependent-version-narrow");
    let repo: PathBuf = workspace(&fixture);
    let narrow: [&str; 4] = ["cli-git", "fix", "--", "package/module/a/package.json"];
    let edited: String = format!("{} ", manifest("@s/b", "2.0.0", Some("@s/a")));
    write(repo.as_path(), "package/module/b/package.json", &edited);
    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &narrow);
    assert_eq!(refused.code, Some(2));
    assert_eq!(
        String::from_utf8_lossy(&refused.stdout),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"patch-conflict\",\
         \"message\":\"A policy fix needs to change package/module/b/package.json, which this fix did not select, \
         but its worktree copy has unstaged changes. Include package/module/b/package.json in the fix pathspecs \
         so the fix applies to its worktree copy, or restore it to match HEAD \
         (git restore --staged --worktree -- package/module/b/package.json), then run the fix again.\",\
         \"trigger\":\"direct-fix\",\"policyId\":\"mono/dependent-version-bump\"}\n"
    );
    assert_eq!(read(repo.as_path(), "b"), edited);
    assert_eq!(
        read(repo.as_path(), "c"),
        manifest("@s/c", "3.0.0", Some("@s/a"))
    );
    git(
        &fixture,
        repo.as_path(),
        &["restore", "--", "package/module/b/package.json"],
    );
    let fixed: Observed = run_wrapped(&fixture, repo.as_path(), &narrow);
    assert_eq!(
        fixed.code,
        Some(0),
        "{}",
        String::from_utf8_lossy(&fixed.stdout)
    );
    assert_eq!(
        read(repo.as_path(), "b"),
        manifest("@s/b", "2.0.1", Some("@s/a"))
    );
    assert_eq!(
        read(repo.as_path(), "c"),
        manifest("@s/c", "3.0.1", Some("@s/a"))
    );
    remove(&fixture);
}

/// `git add` of the raised manifest, with the policy enabled and that manifest a candidate,
/// stages it and says nothing: the pre-forward trigger plans only for `commit`, as the
/// incumbent does, so the command is never refused.
#[test]
fn add_of_a_raised_manifest_is_not_refused() {
    let fixture: Fixture = fixture("dependent-version-add");
    let repo: PathBuf = workspace(&fixture);
    let added: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["add", "--", "package/module/a/package.json"],
    );
    assert_eq!(
        added,
        Observed {
            code: Some(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    );
    let staged: Output = git(
        &fixture,
        repo.as_path(),
        &["diff", "--cached", "--name-only"],
    );
    assert_eq!(
        String::from_utf8_lossy(&staged.stdout),
        "package/module/a/package.json\n"
    );
    remove(&fixture);
}

/// A staged deletion of an unrelated file does not stop `check --all` from reporting both
/// dependents of a raised manifest: the check reports them and exits 1, as it does without it.
#[test]
fn a_staged_deletion_does_not_block_the_check() {
    let fixture: Fixture = fixture("dependent-version-deletion");
    let repo: PathBuf = workspace(&fixture);
    write(repo.as_path(), "package/module/old/notes.txt", "retired\n");
    git(
        &fixture,
        repo.as_path(),
        &["add", "--", "package/module/old/notes.txt"],
    );
    git(
        &fixture,
        repo.as_path(),
        &[
            "commit",
            "--quiet",
            "--message=notes",
            "--",
            "package/module/old/notes.txt",
        ],
    );
    git(
        &fixture,
        repo.as_path(),
        &["rm", "--quiet", "--", "package/module/old/notes.txt"],
    );
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["cli-git", "check", "--all"]),
        Observed {
            code: Some(1),
            stdout: format!(
                "{}{}",
                stale(0, "b", "2.0.0", "2.0.1"),
                stale(1, "c", "3.0.0", "3.0.1")
            )
            .into_bytes(),
            stderr: Vec::new(),
        }
    );
    remove(&fixture);
}

/// Under `GIT_LITERAL_PATHSPECS=1`, a direct check that names one raised manifest reports
/// its dependents exactly as plain mode does: the listing and the staging read the pathspec alike.
#[test]
fn literal_pathspecs_check_a_named_manifest_like_plain_mode() {
    let fixture: Fixture = fixture("dependent-version-literal-named");
    let repo: PathBuf = workspace(&fixture);
    let arguments: [&str; 4] = ["cli-git", "check", "--", "package/module/a/package.json"];
    let plain: Observed = observe(
        wrapped(&fixture).current_dir(repo.as_path()).args(arguments),
        b"",
    );
    let literal: Observed = observe(
        wrapped(&fixture)
            .current_dir(repo.as_path())
            .env("GIT_LITERAL_PATHSPECS", "1")
            .args(arguments),
        b"",
    );
    assert_eq!(plain.code, Some(1));
    assert_eq!(literal, plain);
    remove(&fixture);
}

/// Under `GIT_LITERAL_PATHSPECS=1`, `check --all` reports the same dependents as plain mode:
/// the `:/` pathspec is magic that literal mode would otherwise read as a file name.
#[test]
fn literal_pathspecs_check_all_like_plain_mode() {
    let fixture: Fixture = fixture("dependent-version-literal-all");
    let repo: PathBuf = workspace(&fixture);
    let arguments: [&str; 3] = ["cli-git", "check", "--all"];
    let plain: Observed = observe(
        wrapped(&fixture).current_dir(repo.as_path()).args(arguments),
        b"",
    );
    let literal: Observed = observe(
        wrapped(&fixture)
            .current_dir(repo.as_path())
            .env("GIT_LITERAL_PATHSPECS", "1")
            .args(arguments),
        b"",
    );
    assert_eq!(plain.code, Some(1));
    assert_eq!(literal, plain);
    remove(&fixture);
}
