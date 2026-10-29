//! What:
//!  `markdown/autofix` through the built executable,
//!  with the linter built from this
//!       repository found on the fixture's PATH,
//!  a missing linter,
//!  and a linter that fails.
//! Why:
//!  The policy's promises are about the program a person runs as `git`:
//!  an add warns
//!      about a Markdown file the linter would rewrite and stages it unchanged,
//!  `check`
//!      reports it,
//!  `fix` rewrites only the worktree file,
//!  an excluded file is left alone,
//!      and a linter that is missing or fails stops the command with exit status 2 and
//!      stages nothing.
//!  The real linter comes only from `GIT_POLICY_NATIVE_TEST_LINTER`,
//!      never from whatever is installed,
//!  and its absence fails the controls that need it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const added = spawnSync(join(fixture, 'bin/git'), ['add', '--', 'pkg/README.md'], { env: { PATH: `${bin}:${linterDir}:/usr/bin:/bin` } });
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{
    Fixture, Observed, bounded, bounded_for, executable, fixture, git, observe, porcelain, remove, repository,
};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The variable naming the linter built from this repository,
///  set in the gate image.
const LINTER_VARIABLE: &str = "GIT_POLICY_NATIVE_TEST_LINTER";

/// The object URL of `pkg/asset/shot.png`;
///  the digest is `sha256sum` of `image bytes`.
const SHOT_URL: &str = "https://lfs.example.com/org/repo/de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec/pkg/asset/shot.png";

/// A Markdown file whose relative image link the LFS rule rewrites.
const RELATIVE: &[u8] = b"# T\n\n![shot](asset/shot.png)\n";

/// The linter built from this repository,
///  by the variable the gate image sets.
fn real_linter() -> PathBuf {
    let Some(path) = std::env::var_os(LINTER_VARIABLE) else {
        panic!(
            "{LINTER_VARIABLE} is unset: build package/linter/monochromatic-lint and name its executable in \
             {LINTER_VARIABLE}; the gate image sets it to the linter it builds"
        );
    };
    return PathBuf::from(path);
}

/// Put a `monochromatic-lint` link to `program` in `<root>/linter` and return the PATH that
/// has the wrapper first,
///  then that directory,
///  then the system.
fn path_with_linter(fixture: &Fixture, program: &Path) -> OsString {
    let directory: PathBuf = fixture.root.join("linter");
    std::fs::create_dir_all(&directory).expect("linter directory");
    std::os::unix::fs::symlink(program, directory.join("monochromatic-lint")).expect("link");
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(directory);
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
        repo.join("cli-git.config.jsonc"),
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

/// With the real linter:
///  an add warns and stages the bytes as they are;
///  `check` reports;
/// `fix` rewrites the worktree file only and summarizes;
///  an excluded file and a file
/// that is already right report nothing;
///  at `error` the add stops and stages nothing.
#[test]
fn the_real_linter_rewrites_through_add_check_and_fix() {
    let fixture: Fixture = fixture("markdown-real");
    let path: OsString = path_with_linter(&fixture, real_linter().as_path());
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
    let checked: Observed = run_with(&fixture, &path, &repo, &["cli-git", "check", "--all"]);
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
    let fixed: Observed = run_with(&fixture, &path, &repo, &["cli-git", "fix", "--all"]);
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
        repo.join("cli-git.config.jsonc"),
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

/// The engine failure of a Markdown candidate the linter could not check.
fn unchecked(trigger: &str, path: &str, reason: &str) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"policy-incomplete\",\
         \"message\":\"{reason}. The Markdown candidate was not checked and nothing was changed; fix the cause, \
         or pass --no-enforce-markdown/autofix to skip this policy for one command.\",\
         \"trigger\":\"{trigger}\",\"policyId\":\"markdown/autofix\",\"path\":\"{path}\"}}\n"
    );
}

/// Without a linter on PATH a Markdown add stops with exit status 2 and stages nothing,
/// while an add with no Markdown file,
///  or with excluded ones only,
///  needs no linter;
/// escaping the policy lets the Markdown add through.
#[test]
fn a_missing_linter_stops_only_markdown_work() {
    let fixture: Fixture = fixture("markdown-missing");
    let repo: PathBuf = lfs_repository(&fixture, "[\"warn\", { \"exclude\": [\"skip/\"] }]");
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    std::fs::write(repo.join("a.md"), RELATIVE).expect("markdown");
    std::fs::write(repo.join("b.txt"), b"text\n").expect("text");
    let stopped: Observed = run_with(&fixture, &path, &repo, &["add", "--", "a.md"]);
    assert_eq!(
        (
            stopped.code,
            String::from_utf8_lossy(&stopped.stderr).into_owned()
        ),
        (
            Some(2),
            unchecked(
                "pre-forward",
                "a.md",
                "cli-git could not find monochromatic-lint in any absolute directory on PATH, and the policy \
                 markdown/autofix runs it for every Markdown candidate; install it so that it is on PATH"
            )
        )
    );
    let checked: Observed = run_with(&fixture, &path, &repo, &["cli-git", "check", "--", "a.md"]);
    assert_eq!(checked.code, Some(2));
    assert_eq!(
        String::from_utf8_lossy(&checked.stdout).into_owned(),
        unchecked(
            "direct-check",
            "a.md",
            "cli-git could not find monochromatic-lint in any absolute directory on PATH, and the policy \
             markdown/autofix runs it for every Markdown candidate; install it so that it is on PATH"
        )
    );
    let text: Observed = run_with(&fixture, &path, &repo, &["add", "--", "b.txt"]);
    assert_eq!((text.code, text.stderr), (Some(0), Vec::new()));
    std::fs::create_dir(repo.join("skip")).expect("excluded directory");
    std::fs::write(repo.join("skip/c.md"), RELATIVE).expect("excluded markdown");
    let excluded: Observed = run_with(&fixture, &path, &repo, &["add", "--", "skip/c.md"]);
    assert_eq!((excluded.code, excluded.stderr), (Some(0), Vec::new()));
    let escaped: Observed = run_with(
        &fixture,
        &path,
        &repo,
        &["add", "--no-enforce-markdown/autofix", "--", "a.md"],
    );
    assert_eq!((escaped.code, escaped.stderr), (Some(0), Vec::new()));
    assert_eq!(
        porcelain(&fixture, repo.as_path()),
        "A  a.md\nA  b.txt\nA  skip/c.md\n"
    );
    remove(&fixture);
}

/// A linter that exits with a status its configuration never produces stops the add and
/// the fix,
///  quoting its explanation,
///  and changes nothing.
#[test]
fn a_failing_linter_stops_the_command() {
    let fixture: Fixture = fixture("markdown-failing");
    let program: PathBuf = fixture.root.join("failing-linter");
    executable(
        program.as_path(),
        b"#!/bin/sh\necho 'monochromatic-lint: broken on purpose' >&2\nexit 3\n",
    );
    let path: OsString = path_with_linter(&fixture, program.as_path());
    let repo: PathBuf = lfs_repository(&fixture, "\"warn\"");
    std::fs::write(repo.join("a.md"), RELATIVE).expect("markdown");
    let reason: &str = "monochromatic-lint exited with status 3, which its one-rule configuration never produces: broken on purpose";
    let stopped: Observed = run_with(&fixture, &path, &repo, &["add", "--", "a.md"]);
    assert_eq!(
        (
            stopped.code,
            String::from_utf8_lossy(&stopped.stderr).into_owned()
        ),
        (Some(2), unchecked("pre-forward", "a.md", reason))
    );
    let fixed: Observed = run_with(&fixture, &path, &repo, &["cli-git", "fix", "--all"]);
    assert_eq!(
        (
            fixed.code,
            String::from_utf8_lossy(&fixed.stdout).into_owned()
        ),
        (Some(2), unchecked("direct-fix", "a.md", reason))
    );
    assert_eq!(
        std::fs::read(repo.join("a.md")).expect("unchanged"),
        RELATIVE
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), "?? a.md\n");
    remove(&fixture);
}

/// The bound of one commit under host load, in seconds; the shared bound is for single Git commands.
const COMMIT_BOUND_SECONDS: &str = "120";

/// The commit `main` names, read by real Git.
fn head(fixture: &Fixture, repo: &Path) -> String {
    return String::from_utf8_lossy(&git(fixture, repo, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
}

/// A commit of a Markdown file lands, and its post-commit stage finds the linter on the invocation's PATH
/// and runs it over the landed commit, so the command exits 0 with nothing on standard error.
/// The fake linter returns its input unchanged and records the commit `main` names each time it runs, so a run
/// made after the landing is told apart from the pre-commit runs.
#[test]
fn the_post_commit_stage_finds_the_linter_on_the_invocations_path() {
    let fixture: Fixture = fixture("markdown-post-commit");
    let log: PathBuf = fixture.root.join("linter-runs.log");
    let program: PathBuf = fixture.root.join("recording-linter");
    executable(
        program.as_path(),
        format!("#!/bin/sh\ncat .git/refs/heads/main >> '{}'\ncat\n", log.display()).as_bytes(),
    );
    let path: OsString = path_with_linter(&fixture, program.as_path());
    let repo: PathBuf = lfs_repository(&fixture, "\"warn\"");
    let base: String = head(&fixture, repo.as_path());
    std::fs::write(repo.join("doc.md"), b"# T\n\nplain\n").expect("markdown");
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
    let landed: String = head(&fixture, repo.as_path());
    let runs: String = std::fs::read_to_string(&log).expect("linter runs");
    assert_eq!(
        (committed.code, String::from_utf8_lossy(&committed.stderr).into_owned()),
        (Some(0), String::new()),
        "runs: {runs}"
    );
    assert_ne!(landed, base);
    assert_eq!(runs.lines().last(), Some(landed.as_str()), "runs: {runs}");
    remove(&fixture);
}

/// The per-user scratch directory relative to `HOME`, where the linter's configuration lives.
const SCRATCH_RELATIVE: &str = ".local/state/cli-git/scratch";

/// Markdown work needs no system temporary directory: with `TMPDIR` naming a path that does not exist, an add and
/// a direct check still run the linter, the linter's configuration file exists in the per-user scratch directory
/// while the linter runs, and nothing is left there afterwards.
#[test]
fn the_linter_configuration_lives_in_the_per_user_scratch_directory() {
    let fixture: Fixture = fixture("markdown-scratch-location");
    let log: PathBuf = fixture.root.join("linter-config.log");
    let program: PathBuf = fixture.root.join("recording-linter");
    executable(
        program.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$1\" >> '{log}'\nif [ -f \"${{1#--config=}}\" ]; then echo present >> '{log}'; \
             else echo missing >> '{log}'; fi\ncat\n",
            log = log.display()
        )
        .as_bytes(),
    );
    let path: OsString = path_with_linter(&fixture, program.as_path());
    let repo: PathBuf = lfs_repository(&fixture, "\"warn\"");
    std::fs::write(repo.join("a.md"), b"# T\n\nplain\n").expect("markdown");
    let unusable: PathBuf = fixture.root.join("no-such-tmpdir");
    let added: Observed = observe(
        bounded(&fixture, fixture.root.join("bin/git").as_path(), path.as_os_str())
            .current_dir(&repo)
            .env("TMPDIR", &unusable)
            .args(["add", "--", "a.md"]),
        b"",
    );
    assert_eq!((added.code, String::from_utf8_lossy(&added.stderr).into_owned()), (Some(0), String::new()));
    let checked: Observed = observe(
        bounded(&fixture, fixture.root.join("bin/git").as_path(), path.as_os_str())
            .current_dir(&repo)
            .env("TMPDIR", &unusable)
            .args(["cli-git", "check", "--", "a.md"]),
        b"",
    );
    assert_eq!(checked.code, Some(0), "{}", String::from_utf8_lossy(&checked.stdout));
    let scratch: PathBuf = fixture.root.join("home").join(SCRATCH_RELATIVE);
    let recorded: String = std::fs::read_to_string(&log).expect("linter runs");
    let lines: Vec<&str> = recorded.lines().collect();
    assert_eq!(lines.len(), 4, "two runs of two lines each: {recorded}");
    let prefix: String = format!("--config={}/cli-git-markdown-", scratch.display());
    for run in lines.chunks(2) {
        let [config, state] = run else { panic!("a run records two lines: {recorded}") };
        assert!(config.starts_with(prefix.as_str()) && config.ends_with(".jsonc"), "{config} under {prefix}");
        assert_eq!(*state, "present", "the configuration exists while the linter runs: {recorded}");
    }
    let left: Vec<OsString> = std::fs::read_dir(&scratch)
        .expect("the per-user scratch directory exists")
        .map(|entry| { return entry.expect("entry").file_name(); })
        .collect();
    assert!(left.is_empty(), "nothing is left in {}: {left:?}", scratch.display());
    assert!(!unusable.exists(), "nothing created the unusable temporary directory");
    remove(&fixture);
}

/// A commit needs no system temporary directory either: with `TMPDIR` naming a path that does not exist the commit
/// lands, the private scratch directory exists beside the repository's other private storage, and it is empty.
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
    let scratch: PathBuf = repo.join(".git").join("cli-git-scratch");
    let left: Vec<OsString> = std::fs::read_dir(&scratch)
        .expect("the repository scratch directory exists")
        .map(|entry| { return entry.expect("entry").file_name(); })
        .collect();
    assert!(left.is_empty(), "nothing is left in {}: {left:?}", scratch.display());
    assert!(!unusable.exists(), "nothing created the unusable temporary directory");
    remove(&fixture);
}
