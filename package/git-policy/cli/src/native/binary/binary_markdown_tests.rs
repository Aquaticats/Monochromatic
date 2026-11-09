//! What:
//!  `markdown/autofix` through the built executable,
//!  with the linter linked into the executable,
//!  and a policy option that stops Markdown work.
//! Why:
//!  The policy's promises are about the program a person runs as `git`:
//!  an add warns
//!      about a Markdown file the linter would rewrite and stages it unchanged,
//!  `check`
//!      reports it,
//!  `fix` rewrites only the worktree file,
//!  an excluded file is left alone,
//!      and a candidate the policy cannot check stops the command with exit status 2 and
//!      stages nothing.
//!  No linter is installed or found on PATH:
//! the controls run with only the system programs after the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const added = spawnSync(join(fixture, 'bin/git'), ['add', '--', 'pkg/README.md'], { env: { PATH: `${bin}:/usr/bin:/bin` } });
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, bounded, bounded_for, fixture, git, observe, porcelain, remove, repository};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The object URL of `pkg/asset/shot.png`;
///  the digest is `sha256sum` of `image bytes`.
const SHOT_URL: &str = "https://lfs.example.com/org/repo/de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec/pkg/asset/shot.png";

/// A Markdown file whose relative image link the LFS rule rewrites.
const RELATIVE: &[u8] = b"# T\n\n![shot](asset/shot.png)\n";

/// The PATH that has the wrapper first and then the system:
///  no linter is on it.
fn plain_path(fixture: &Fixture) -> OsString {
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    return path;
}

/// Run the wrapper in `repo` under `path` and return what the caller saw.
fn run_with(fixture: &Fixture, path: &OsStr, repo: &Path, arguments: &[&str]) -> Observed {
    return observe(
        bounded(fixture, fixture.root.join("bin/git").as_path(), path)
            .current_dir(repo)
            .args(arguments),
        b"",
    );
}

/// A repository with an LFS endpoint,
///  the PNG rule,
///  one tracked image,
///  the Markdown policy
/// configured with `setting` and `final-newline` off,
///  and all of that committed.
fn lfs_repository(fixture: &Fixture, setting: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::create_dir_all(repo.join("pkg/asset")).expect("tree");
    std::fs::write(
        repo.join(".lfsconfig"),
        b"[lfs]\n\turl = https://lfs.example.com/org/repo\n",
    )
    .expect("lfsconfig");
    std::fs::write(
        repo.join(".gitattributes"),
        b"*.png filter=lfs diff=lfs merge=lfs -text\n",
    )
    .expect("attributes");
    std::fs::write(repo.join("pkg/asset/shot.png"), b"image bytes").expect("image");
    std::fs::write(
        repo.join("monochromatic-git-policy-cli.config.jsonc"),
        // The image is text without a final newline, so the built-in policy is turned off.
        format!(
            "{{ \"policies\": {{ \"final-newline\": \"off\", \"markdown/autofix\": {setting} }} }}\n"
        ),
    )
    .expect("configuration");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=base"],
    );
    return repo;
}

/// The autofix finding about `path` for one trigger,
///  severity and fix state.
fn autofix(sequence: u64, trigger: &str, severity: &str, path: &str, fix: &str) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"{trigger}\",\
         \"policyId\":\"markdown/autofix\",\"severity\":\"{severity}\",\"code\":\"markdown/autofix/markdown-autofix\",\
         \"message\":\"monochromatic-lint --fix (markdown/lfs-image-url) rewrites {path}.\",\"path\":\"{path}\",\"fix\":\"{fix}\"}}\n"
    );
}

/// The staged bytes of `path`,
///  read from the index by real Git.
fn staged(fixture: &Fixture, repo: &Path, path: &str) -> Vec<u8> {
    return git(fixture, repo, &["show", format!(":{path}").as_str()]).stdout;
}

/// With the linked linter:
///  an add warns and stages the bytes as they are;
///  `check` reports;
/// `fix` rewrites the worktree file only and summarizes;
///  an excluded file and a file
/// that is already right report nothing;
///  at `error` the add stops and stages nothing.
#[test]
fn the_linked_linter_rewrites_through_add_check_and_fix() {
    let fixture: Fixture = fixture("markdown-real");
    let path: OsString = plain_path(&fixture);
    let repo: PathBuf = lfs_repository(&fixture, "[\"warn\", { \"exclude\": [\"package/ssg/\"] }]");
    std::fs::write(repo.join("pkg/README.md"), RELATIVE).expect("relative link");
    std::fs::create_dir_all(repo.join("package/ssg")).expect("excluded tree");
    std::fs::write(
        repo.join("package/ssg/a.md"),
        b"![shot](../../pkg/asset/shot.png)\n",
    )
    .expect("excluded");
    std::fs::write(
        repo.join("pkg/done.md"),
        format!("![shot]({SHOT_URL})\n").as_bytes(),
    )
    .expect("already rewritten");
    let added: Observed = run_with(&fixture, &path, &repo, &["add", "--", "pkg/README.md"]);
    assert_eq!(
        (
            added.code,
            String::from_utf8_lossy(&added.stderr).into_owned()
        ),
        (
            Some(0),
            autofix(0, "pre-forward", "warn", "pkg/README.md", "none")
        )
    );
    assert_eq!(staged(&fixture, repo.as_path(), "pkg/README.md"), RELATIVE);
    let quiet: Observed = run_with(
        &fixture,
        &path,
        &repo,
        &["add", "--", "package/ssg/a.md", "pkg/done.md"],
    );
    assert_eq!((quiet.code, quiet.stderr), (Some(0), Vec::new()));
    let checked: Observed = run_with(&fixture, &path, &repo, &["monochromatic-git-policy-cli", "check", "--all"]);
    assert_eq!(
        (
            checked.code,
            String::from_utf8_lossy(&checked.stdout).into_owned()
        ),
        (
            Some(0),
            autofix(0, "direct-check", "warn", "pkg/README.md", "none")
        )
    );
    let index_before: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("index");
    let fixed: Observed = run_with(&fixture, &path, &repo, &["monochromatic-git-policy-cli", "fix", "--all"]);
    assert_eq!(
        (
            fixed.code,
            String::from_utf8_lossy(&fixed.stdout).into_owned()
        ),
        (
            Some(0),
            String::from(
                "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"direct-fix\",\
                 \"passes\":1,\"changedPaths\":[\"pkg/README.md\"]}\n"
            )
        )
    );
    assert_eq!(
        std::fs::read(repo.join("pkg/README.md")).expect("rewritten"),
        format!("# T\n\n![shot]({SHOT_URL})\n").into_bytes()
    );
    assert_eq!(
        std::fs::read(repo.join(".git/index")).expect("index"),
        index_before
    );
    assert_eq!(staged(&fixture, repo.as_path(), "pkg/README.md"), RELATIVE);
    // At `error` an add of a file the linter would rewrite stops before Git runs.
    std::fs::write(
        repo.join("monochromatic-git-policy-cli.config.jsonc"),
        "{ \"policies\": { \"final-newline\": \"off\", \"markdown/autofix\": \"error\" } }\n",
    )
    .expect("configuration");
    std::fs::write(repo.join("pkg/new.md"), RELATIVE).expect("new file");
    let stopped: Observed = run_with(&fixture, &path, &repo, &["add", "--", "pkg/new.md"]);
    assert_eq!(
        (
            stopped.code,
            String::from_utf8_lossy(&stopped.stderr).into_owned()
        ),
        (
            Some(1),
            autofix(0, "pre-forward", "error", "pkg/new.md", "none")
        )
    );
    assert!(
        porcelain(&fixture, repo.as_path()).contains("?? pkg/new.md\n"),
        "{}",
        porcelain(&fixture, repo.as_path())
    );
    remove(&fixture);
}

/// The engine failure of a Markdown candidate the policy could not check,
/// with no path because the failure is the
/// policy's own option.
fn unchecked_without_path(trigger: &str, reason: &str) -> String {
    // The message is a JSON string value, so the quotes in the pattern's reason are escaped as the event escapes them.
    let message: String = serde_json::to_string(&format!(
        "{reason}. The Markdown candidate was not checked and nothing was changed; fix the cause, \
         or pass --no-enforce-markdown/autofix to skip this policy for one command."
    ))
    .expect("message as JSON");
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"policy-incomplete\",\
         \"message\":{message},\
         \"trigger\":\"{trigger}\",\"policyId\":\"markdown/autofix\"}}\n"
    );
}

/// An exclude pattern the matcher cannot compile leaves a Markdown candidate unchecked:
/// an add and a direct check
/// stop with exit status 2 naming the pattern's reason,
/// while an add with no Markdown file needs no pattern;
/// escaping the policy lets the Markdown add through.
/// A trailing escape is the shape that still cannot compile:
/// a character range that runs backwards is read the way Git reads it.
#[test]
fn an_exclude_that_does_not_compile_stops_only_markdown_work() {
    let fixture: Fixture = fixture("markdown-unchecked");
    let path: OsString = plain_path(&fixture);
    let repo: PathBuf = lfs_repository(&fixture, "[\"warn\", { \"exclude\": [\"a\\\\\"] }]");
    std::fs::write(repo.join("a.md"), RELATIVE).expect("markdown");
    std::fs::write(repo.join("b.txt"), b"text\n").expect("text");
    let reason: String = git_policy_cli::markdown_exclude::compile_exclude(&[String::from("a\\")])
        .map(|_| { return String::new(); })
        .expect_err("the pattern does not compile");
    let stopped: Observed = run_with(&fixture, &path, &repo, &["add", "--", "a.md"]);
    assert_eq!(
        (
            stopped.code,
            String::from_utf8_lossy(&stopped.stderr).into_owned()
        ),
        (Some(2), unchecked_without_path("pre-forward", reason.as_str()))
    );
    let checked: Observed = run_with(&fixture, &path, &repo, &["monochromatic-git-policy-cli", "check", "--", "a.md"]);
    assert_eq!(checked.code, Some(2));
    assert_eq!(
        String::from_utf8_lossy(&checked.stdout).into_owned(),
        unchecked_without_path("direct-check", reason.as_str())
    );
    let text: Observed = run_with(&fixture, &path, &repo, &["add", "--", "b.txt"]);
    assert_eq!((text.code, text.stderr), (Some(0), Vec::new()));
    let escaped: Observed = run_with(
        &fixture,
        &path,
        &repo,
        &["add", "--no-enforce-markdown/autofix", "--", "a.md"],
    );
    assert_eq!((escaped.code, escaped.stderr), (Some(0), Vec::new()));
    assert_eq!(porcelain(&fixture, repo.as_path()), "A  a.md\nA  b.txt\n");
    remove(&fixture);
}

/// The bound of one commit under host load,
/// in seconds;
/// the shared bound is for single Git commands.
const COMMIT_BOUND_SECONDS: &str = "120";

/// The commit `main` names,
/// read by real Git.
fn head(fixture: &Fixture, repo: &Path) -> String {
    return String::from_utf8_lossy(&git(fixture, repo, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
}

/// A commit of a Markdown file lands with the linter linked in and no linter on the PATH:
/// the stale object URL in the file comes back as the policy's warning on standard error,
/// which only a run of the
/// linter produces,
/// and the commit still lands with exit status 0.
#[test]
fn a_markdown_commit_is_checked_in_process_with_no_linter_on_the_path() {
    let fixture: Fixture = fixture("markdown-post-commit");
    let path: OsString = plain_path(&fixture);
    let repo: PathBuf = lfs_repository(&fixture, "\"warn\"");
    let base: String = head(&fixture, repo.as_path());
    let stale: String = format!("![shot](https://lfs.example.com/org/repo/{}/pkg/asset/gone.png)\n", "b".repeat(64));
    std::fs::write(repo.join("doc.md"), stale.as_bytes()).expect("markdown");
    git(&fixture, repo.as_path(), &["add", "--", "doc.md"]);
    let committed: Observed = observe(
        bounded_for(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path.as_os_str(),
            COMMIT_BOUND_SECONDS,
        )
        .current_dir(&repo)
        .args(["commit", "--quiet", "--message=doc", "--", "doc.md"]),
        b"",
    );
    let stderr: String = String::from_utf8_lossy(&committed.stderr).into_owned();
    assert_eq!(committed.code, Some(0), "{stderr}");
    assert!(
        stderr.contains(format!("markdown/autofix/{}", git_policy_cli::policy_markdown::MARKDOWN_VIOLATION_CODE).as_str()),
        "{stderr}"
    );
    assert_ne!(head(&fixture, repo.as_path()), base);
    remove(&fixture);
}

/// A commit needs no system temporary directory either:
/// with `TMPDIR` naming a path that does not exist the commit
/// lands,
/// the private scratch directory exists beside the repository's other private storage,
/// and it is empty.
#[test]
fn a_commit_keeps_its_git_input_in_the_repository_scratch_directory() {
    let fixture: Fixture = fixture("commit-scratch-location");
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("base.txt"), b"base\n").expect("base");
    git(&fixture, repo.as_path(), &["add", "--", "base.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = head(&fixture, repo.as_path());
    std::fs::write(repo.join("doc.txt"), b"plain\n").expect("text");
    git(&fixture, repo.as_path(), &["add", "--", "doc.txt"]);
    let unusable: PathBuf = fixture.root.join("no-such-tmpdir");
    let committed: Observed = observe(
        bounded_for(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path.as_os_str(),
            COMMIT_BOUND_SECONDS,
        )
        .current_dir(&repo)
        .env("TMPDIR", &unusable)
        .args(["commit", "--quiet", "--message=doc", "--", "doc.txt"]),
        b"",
    );
    assert_eq!(
        (committed.code, String::from_utf8_lossy(&committed.stderr).into_owned()),
        (Some(0), String::new())
    );
    assert_ne!(head(&fixture, repo.as_path()), base, "the commit landed");
    let scratch: PathBuf = repo.join(".git").join("monochromatic-git-policy-cli-scratch");
    let left: Vec<OsString> = std::fs::read_dir(&scratch)
        .expect("the repository scratch directory exists")
        .map(|entry| { return entry.expect("entry").file_name(); })
        .collect();
    assert!(left.is_empty(), "nothing is left in {}: {left:?}", scratch.display());
    assert!(!unusable.exists(), "nothing created the unusable temporary directory");
    remove(&fixture);
}
