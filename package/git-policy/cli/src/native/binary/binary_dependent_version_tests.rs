//! What:
//! `mono/dependent-version-bump` through the built executable:
//! `git monochromatic-git-policy-cli check`
//!       reports the dependents of a raised manifest,
//! `git monochromatic-git-policy-cli fix` bumps them in the
//!       worktree,
//! and a narrow fix changes an unselected dependent only when it matches
//!       `HEAD`.
//! Why:
//! These are the commands the release workflow will run;
//! the controls read the
//!      events,
//! the worktree and the index exactly as a person would see them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['monochromatic-git-policy-cli', 'fix', '--all']).stdout).toContain('fix-summary');
//! ```

/// The deepest indentation the generated configuration writes,
/// which the chain below must pass.
use git_policy_cli::dependent_version_yaml_lines::GENERATED_MAX_INDENT;
/// Import the shared fixtures and bounded process helpers.
use super::commit::{git_text, run_commit};
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

/// Write a file below the repository,
/// creating its directory.
fn write(repo: &Path, path: &str, text: &str) {
    let file: PathBuf = repo.join(path);
    std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
    std::fs::write(file, text).expect("file");
}

/// The registry configuration the generator writes around one publish set,
/// four levels down at `auth.oidc[].workloads[].packages`;
/// an empty slice writes the list with no item.
fn publish_set(names: &[&str]) -> String {
    let items: Vec<String> = names
        .iter()
        .map(|name: &&str| return format!("            - '{name}'"))
        .collect();
    return format!(
        "auth:\n  oidc:\n    - name: github\n      workloads:\n        - identity:\n            subject: 'repo:example/main'\n          registry: r\n          packages:\n{}\n\nweb:\n  enable: false\n",
        items.join("\n")
    );
}

/// A committed workspace of `@s/a` and its dependents `@s/b` and `@s/c`,
/// configured for
/// the policy,
/// with `@s/a` raised to `1.1.0` in the worktree.
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
        &publish_set(&["@s/a", "@s/b", "@s/c"]),
    );
    write(repo.as_path(), "monochromatic-git-policy-cli.config.jsonc", CONFIGURATION);
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

/// A committed workspace whose packages form the chain `@s/a` ← `@s/b` ← `@s/c`,
/// configured for the policy,
/// with `@s/a` raised to `1.1.0` in the worktree:
/// `@s/c` reaches the raised `@s/a` only through `@s/b`,
/// so its bump proves the ripple is transitive.
fn chain_workspace(fixture: &Fixture) -> PathBuf {
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
        &manifest("@s/c", "3.0.0", Some("@s/b")),
    );
    write(
        repo.as_path(),
        "package/config/pnpr/config.yaml",
        &publish_set(&["@s/a", "@s/b", "@s/c"]),
    );
    write(repo.as_path(), "monochromatic-git-policy-cli.config.jsonc", CONFIGURATION);
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=chain"],
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

/// A committed workspace of `@s/base` and its runtime dependent `app`,
/// configured for the policy,
/// with `@s/base` raised to `1.1.0` in the worktree and no registry configuration yet.
fn app_workspace(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    write(
        repo.as_path(),
        "package/module/base/package.json",
        &manifest("@s/base", "1.0.0", None),
    );
    write(
        repo.as_path(),
        "package/module/app/package.json",
        &manifest("app", "2.0.0", Some("@s/base")),
    );
    write(repo.as_path(), "monochromatic-git-policy-cli.config.jsonc", CONFIGURATION);
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=workspace"],
    );
    write(
        repo.as_path(),
        "package/module/base/package.json",
        &manifest("@s/base", "1.1.0", None),
    );
    return repo;
}

/// A tagged workload key ends the check with the line that holds it,
/// instead of hiding the
///       publishable package the other workload already listed a name beside.
#[test]
fn a_tagged_key_stops_the_check_by_line() {
    let fixture: Fixture = fixture("dependent-version-tagged-key");
    let repo: PathBuf = app_workspace(&fixture);
    write(
        repo.as_path(),
        "package/config/pnpr/config.yaml",
        concat!(
            "auth:\n",
            "  oidc:\n",
            "    - name: github\n",
            "      workloads:\n",
            "        - registry: r\n",
            "          packages:\n",
            "            - '@s/base'\n",
            "        - registry: monochromatic-dev\n",
            "          !!str packages:\n",
            "            - 'app'\n",
        ),
    );
    let refused: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["monochromatic-git-policy-cli", "check", "--all"],
    );
    assert_eq!(refused.code, Some(2), "{}", String::from_utf8_lossy(&refused.stdout));
    let events: String = String::from_utf8_lossy(&refused.stdout).into_owned();
    assert!(events.contains("\"code\":\"policy-incomplete\""), "{events}");
    assert!(
        events.contains("begins with '!', which opens a key this reader does not read on line 9"),
        "{events}"
    );
    remove(&fixture);
}

/// A '#' on a list item stops the check with the line that holds it,
/// instead of reading the item
///       as a mapping or as another name:
/// the generator writes no '#' on an item.
#[test]
fn a_hash_on_a_list_item_stops_the_check() {
    let fixture: Fixture = fixture("dependent-version-item-hash");
    let repo: PathBuf = app_workspace(&fixture);
    let text: String = concat!(
        "auth:\n",
        "  oidc:\n",
        "    - name: github\n",
        "      workloads:\n",
        "        - registry: r\n",
        "          packages:\n",
        "            - '@s/base'\n",
        "            - app # publication: enabled\n",
    )
    .to_string();
    let hashed: usize = text.lines().count();
    write(repo.as_path(), "package/config/pnpr/config.yaml", &text);
    let refused: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["monochromatic-git-policy-cli", "check", "--all"],
    );
    assert_eq!(
        refused.code,
        Some(2),
        "{}",
        String::from_utf8_lossy(&refused.stdout)
    );
    let events: String = String::from_utf8_lossy(&refused.stdout).into_owned();
    assert!(events.contains("\"code\":\"policy-incomplete\""), "{events}");
    assert!(
        events.contains(
            format!(
                "holds a '#' on a list item, where the generated configuration writes none on line {hashed}"
            )
            .as_str()
        ),
        "{events}"
    );
    remove(&fixture);
}

/// A commit that raises a version lands whether the publish set is absent or empty:
/// the pre-forward lifecycle plans no dependent bump,
/// so a publish set it cannot read never fails a commit.
#[test]
fn a_version_raising_commit_lands_without_a_publish_set() {
    for (name, configuration) in [
        ("dependent-version-commit-untracked", None),
        ("dependent-version-commit-empty", Some(publish_set(&[]))),
    ] {
        let fixture: Fixture = fixture(name);
        let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
        write(
            repo.as_path(),
            "package/module/base/package.json",
            &manifest("@s/base", "1.0.0", None),
        );
        write(repo.as_path(), "monochromatic-git-policy-cli.config.jsonc", CONFIGURATION);
        if let Some(text) = configuration {
            write(repo.as_path(), "package/config/pnpr/config.yaml", &text);
        }
        git(&fixture, repo.as_path(), &["add", "--all"]);
        git(
            &fixture,
            repo.as_path(),
            &["commit", "--quiet", "--message=initial"],
        );
        write(
            repo.as_path(),
            "package/module/base/package.json",
            &manifest("@s/base", "1.1.0", None),
        );
        git(
            &fixture,
            repo.as_path(),
            &["add", "--", "package/module/base/package.json"],
        );
        let landed: Observed = run_commit(
            &fixture,
            repo.as_path(),
            &[
                "commit",
                "--message=bump",
                "--",
                "package/module/base/package.json",
            ],
        );
        assert_eq!(
            landed.code,
            Some(0),
            "{name}: {}{}",
            String::from_utf8_lossy(&landed.stdout),
            String::from_utf8_lossy(&landed.stderr)
        );
        assert_eq!(
            git_text(
                &fixture,
                repo.as_path(),
                &["show", "HEAD:package/module/base/package.json"],
            ),
            manifest("@s/base", "1.1.0", None),
            "{name}"
        );
        remove(&fixture);
    }
}

/// A check reports both dependents with a fix available and exits 1;
/// `fix --all` bumps
/// both in one pass without changing the index;
/// the check is then clean.
#[test]
fn check_reports_and_fix_bumps_every_dependent() {
    let fixture: Fixture = fixture("dependent-version-all");
    let repo: PathBuf = workspace(&fixture);
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "check", "--all"]),
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
        run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "fix", "--all"]),
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
        run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "check", "--all"]),
        Observed {
            code: Some(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    );
    remove(&fixture);
}

/// A fix of the raised manifest alone bumps the clean dependents it did not select,
/// and
/// refuses one whose worktree copy changed,
/// changing nothing.
#[test]
fn a_narrow_fix_changes_unselected_dependents_only_when_they_match_head() {
    let fixture: Fixture = fixture("dependent-version-narrow");
    let repo: PathBuf = workspace(&fixture);
    let narrow: [&str; 4] = ["monochromatic-git-policy-cli", "fix", "--", "package/module/a/package.json"];
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

/// `git add` of the raised manifest,
/// with the policy enabled and that manifest a candidate,
/// stages it and says nothing:
/// the pre-forward trigger plans only for `commit`,
/// as the
/// incumbent does,
/// so the command is never refused.
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
/// dependents of a raised manifest:
/// the check reports them and exits 1,
/// as it does without it.
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
        run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "check", "--all"]),
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

/// Under `GIT_LITERAL_PATHSPECS=1`,
/// a direct check that names one raised manifest reports
/// its dependents exactly as plain mode does:
/// the listing and the staging read the pathspec alike.
#[test]
fn literal_pathspecs_check_a_named_manifest_like_plain_mode() {
    let fixture: Fixture = fixture("dependent-version-literal-named");
    let repo: PathBuf = workspace(&fixture);
    let arguments: [&str; 4] = ["monochromatic-git-policy-cli", "check", "--", "package/module/a/package.json"];
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

/// Under `GIT_LITERAL_PATHSPECS=1`,
/// `check --all` reports the same dependents as plain mode:
/// the `:/` pathspec is magic that literal mode would otherwise read as a file name.
#[test]
fn literal_pathspecs_check_all_like_plain_mode() {
    let fixture: Fixture = fixture("dependent-version-literal-all");
    let repo: PathBuf = workspace(&fixture);
    let arguments: [&str; 3] = ["monochromatic-git-policy-cli", "check", "--all"];
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

/// A configuration nested deeper than the generated one stops the policy with a
/// diagnostic naming the first line past its deepest key,
/// instead of reading it and judging:
/// the reader's block recursion is bounded by that
///       indentation,
/// so no input can grow the stack.
#[test]
fn a_configuration_deeper_than_the_generated_one_stops_the_check() {
    let fixture: Fixture = fixture("dependent-version-too-deep");
    let repo: PathBuf = app_workspace(&fixture);
    let mut text: String = publish_set(&["@s/base", "app"]);
    text.push_str("deeper:\n");
    for level in 1..=GENERATED_MAX_INDENT + 1 {
        text.push_str(&" ".repeat(level));
        text.push_str("a:\n");
    }
    let deepest: usize = text.lines().count();
    write(repo.as_path(), "package/config/pnpr/config.yaml", &text);
    let refused: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["monochromatic-git-policy-cli", "check", "--all"],
    );
    assert_eq!(
        refused.code,
        Some(2),
        "{}",
        String::from_utf8_lossy(&refused.stdout)
    );
    let events: String = String::from_utf8_lossy(&refused.stdout).into_owned();
    assert!(events.contains("\"code\":\"policy-incomplete\""), "{events}");
    assert!(
        events.contains(
            format!(
                "is indented deeper than the generated configuration's deepest key, which sits {GENERATED_MAX_INDENT} spaces in on line {deepest}"
            )
            .as_str()
        ),
        "{events}"
    );
    remove(&fixture);
}

/// The manifest the commit at `HEAD` holds for dependent `name`.
fn committed_manifest(fixture: &Fixture, repo: &Path, name: &str) -> String {
    return git_text(
        fixture,
        repo,
        &[
            "show",
            format!("HEAD:package/module/{name}/package.json").as_str(),
        ],
    );
}

/// A wrapped commit of a hand-raised manifest carries its dependents' patch bumps in the
/// same commit,
/// through the whole chain,
/// and brings their worktree copies to the committed bytes:
/// both copies still matched `HEAD`,
/// so the update discards nothing.
#[test]
fn a_commit_adds_and_updates_the_dependents_of_a_hand_bump() {
    let fixture: Fixture = fixture("dependent-version-commit-adds");
    let repo: PathBuf = chain_workspace(&fixture);
    git(
        &fixture,
        repo.as_path(),
        &["add", "--", "package/module/a/package.json"],
    );
    let landed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &[
            "commit",
            "--message=bump",
            "--",
            "package/module/a/package.json",
        ],
    );
    assert_eq!(
        landed.code,
        Some(0),
        "{}{}",
        String::from_utf8_lossy(&landed.stdout),
        String::from_utf8_lossy(&landed.stderr)
    );
    assert_eq!(
        committed_manifest(&fixture, repo.as_path(), "a"),
        manifest("@s/a", "1.1.0", None)
    );
    assert_eq!(
        committed_manifest(&fixture, repo.as_path(), "b"),
        manifest("@s/b", "2.0.1", Some("@s/a"))
    );
    assert_eq!(
        committed_manifest(&fixture, repo.as_path(), "c"),
        manifest("@s/c", "3.0.1", Some("@s/b"))
    );
    assert_eq!(
        read(repo.as_path(), "b"),
        manifest("@s/b", "2.0.1", Some("@s/a"))
    );
    assert_eq!(
        read(repo.as_path(), "c"),
        manifest("@s/c", "3.0.1", Some("@s/b"))
    );
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["status", "--porcelain"]),
        ""
    );
    remove(&fixture);
}

/// A dependent whose worktree copy no longer matches `HEAD` holds an edit the commit does
/// not carry:
/// the commit keeps that copy,
/// leaves the dependent out of the commit,
/// lands the dependents it can,
/// and says which dependent it left behind and why.
#[test]
fn a_commit_leaves_a_dependent_whose_worktree_copy_changed() {
    let fixture: Fixture = fixture("dependent-version-commit-kept-copy");
    let repo: PathBuf = chain_workspace(&fixture);
    let edited: String = format!("{} ", manifest("@s/b", "2.0.0", Some("@s/a")));
    write(repo.as_path(), "package/module/b/package.json", &edited);
    git(
        &fixture,
        repo.as_path(),
        &["add", "--", "package/module/a/package.json"],
    );
    let landed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &[
            "commit",
            "--message=bump",
            "--",
            "package/module/a/package.json",
        ],
    );
    assert_eq!(
        landed.code,
        Some(0),
        "{}{}",
        String::from_utf8_lossy(&landed.stdout),
        String::from_utf8_lossy(&landed.stderr)
    );
    assert_eq!(
        committed_manifest(&fixture, repo.as_path(), "a"),
        manifest("@s/a", "1.1.0", None),
        "the hand bump lands"
    );
    assert_eq!(
        committed_manifest(&fixture, repo.as_path(), "b"),
        manifest("@s/b", "2.0.0", Some("@s/a")),
        "the dependent whose worktree copy holds an edit keeps its version"
    );
    assert_eq!(
        committed_manifest(&fixture, repo.as_path(), "c"),
        manifest("@s/c", "3.0.1", Some("@s/b")),
        "the dependents whose copies match HEAD are added"
    );
    assert_eq!(
        read(repo.as_path(), "b"),
        edited,
        "the edit in the worktree is kept"
    );
    assert_eq!(
        read(repo.as_path(), "c"),
        manifest("@s/c", "3.0.1", Some("@s/b")),
        "a copy that matched HEAD takes the committed bytes"
    );
    let errors: String = String::from_utf8_lossy(&landed.stderr).into_owned();
    assert!(
        errors.contains("package/module/b/package.json"),
        "the report names the dependent: {errors}"
    );
    assert!(
        errors.contains("its worktree copy has unstaged changes"),
        "the report names the reason: {errors}"
    );
    remove(&fixture);
}

/// A dependent whose manifest is staged but not part of the commit is left behind:
/// the landing writes the commit's entry into the real index,
/// so adding it would replace what the person
///  staged.
/// The commit lands with only the hand bump,
/// the staging survives,
/// and the report names both.
#[test]
fn a_commit_leaves_behind_a_dependent_the_real_index_already_changes() {
    let fixture: Fixture = fixture("dependent-version-commit-staged");
    let repo: PathBuf = chain_workspace(&fixture);
    let staged_on_top: String = format!("{} ", manifest("@s/b", "2.0.0", Some("@s/a")));
    write(repo.as_path(), "package/module/b/package.json", &staged_on_top);
    git(
        &fixture,
        repo.as_path(),
        &["add", "--", "package/module/b/package.json"],
    );
    git(
        &fixture,
        repo.as_path(),
        &["add", "--", "package/module/a/package.json"],
    );
    let landed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &[
            "commit",
            "--message=bump",
            "--",
            "package/module/a/package.json",
        ],
    );
    assert_eq!(
        landed.code,
        Some(0),
        "{}{}",
        String::from_utf8_lossy(&landed.stdout),
        String::from_utf8_lossy(&landed.stderr)
    );
    assert_eq!(
        committed_manifest(&fixture, repo.as_path(), "b"),
        manifest("@s/b", "2.0.0", Some("@s/a")),
        "the dependent stays at the version the branch holds"
    );
    assert_eq!(
        git_text(
            &fixture,
            repo.as_path(),
            &["diff", "--cached", "--name-only"]
        ),
        "package/module/b/package.json\n",
        "the staging the person made is still staged"
    );
    assert_eq!(
        read(repo.as_path(), "b"),
        staged_on_top,
        "the staged copy is untouched"
    );
    let errors: String = String::from_utf8_lossy(&landed.stderr).into_owned();
    assert!(
        errors.contains("package/module/b/package.json"),
        "the report names the dependent: {errors}"
    );
    assert!(
        errors.contains("it has staged changes"),
        "the report names the reason: {errors}"
    );
    remove(&fixture);
}
