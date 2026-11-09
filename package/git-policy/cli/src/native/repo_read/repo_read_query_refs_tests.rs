//! Differential controls for the reference and configuration command lines:
//!  standard output,
//! standard error and the exit code equal real Git's wherever the readers answer,
//!  and the states Git words by more than the files say are declined.

use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_query::tests::{answered, declined, mine, scope};
use crate::repo_read_query::{Query, answer_with};
use crate::test_support::{REAL_GIT, fixture, git, git_output, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A repository with a remote,
/// a tracking branch,
/// light and annotated tags,
/// a symbolic remote head and a deep branch.
fn referenced_repository(root: &Path, name: &str, format: &str) -> PathBuf {
    let repo = root.join(name);
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", &format!("--object-format={format}")]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=first"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=second"]);
    git(repo.as_path(), &["remote", "add", "origin", "https://example.invalid/a.git"]);
    git(repo.as_path(), &["update-ref", "refs/remotes/origin/main", "HEAD~1"]);
    git(repo.as_path(), &["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"]);
    git(repo.as_path(), &["tag", "light"]);
    git(repo.as_path(), &["tag", "-a", "-m", "annotated", "v1"]);
    git(repo.as_path(), &["branch", "feature/deep/name"]);
    return repo;
}

/// Pairs of text as environment variables.
fn pairs(items: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    return items.iter().map(|(name, value)| { return (OsString::from(*name), OsString::from(*value)); }).collect();
}

/// Configuration listings equal Git's with includes,
/// subsections,
/// bare names and injected command entries.
#[test]
fn configuration_listings_equal_git() {
    let root = fixture("repo-read-refs-config");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    let extra = root.join("extra.config");
    let global = root.join("global.config");
    std::fs::write(&extra, "[extra]\n\tvalue = from include\n[Mixed \"Sub Section\"]\n\tKey = V\n").expect("extra");
    std::fs::write(&global, format!("[user]\n\tname = Someone\n[include]\n\tpath = {}\n[flag]\n\tbare\n", extra.display())).expect("global");
    git(repo.as_path(), &["config", "--local", "include.path", extra.to_str().expect("UTF-8")]);
    let scope = ScopeEnvironment { home: Some(root.join("no-home")), no_system: true, global: Some(global.clone()), ..ScopeEnvironment::default() };
    let fixed = [("GIT_CONFIG_NOSYSTEM", "1"), ("GIT_CONFIG_GLOBAL", global.to_str().expect("UTF-8"))];
    let injected = [("GIT_CONFIG_COUNT", "2"), ("GIT_CONFIG_KEY_0", "core.hooksPath"), ("GIT_CONFIG_VALUE_0", "/dev/null/no-hooks"), ("GIT_CONFIG_KEY_1", "Hook.Pre-Commit.Enabled"), ("GIT_CONFIG_VALUE_1", "false")];
    let expected = std::process::Command::new(REAL_GIT)
        .current_dir(repo.as_path())
        .env_clear()
        .envs(fixed)
        .envs(injected)
        .args(["config", "--list", "-z", "--show-scope"])
        .output()
        .expect("config listing");
    let given: Vec<OsString> = ["config", "--list", "-z", "--show-scope"].iter().map(OsString::from).collect();
    let added = pairs(&injected);
    let found = answer_with(&Query { arguments: given.as_slice(), directory: Some(repo.as_path()), variables: added.as_slice(), input: None }, pairs(&fixed), &scope).expect("answered");
    assert_eq!(found.stdout, expected.stdout);
    assert!(expected.stdout.windows(5).any(|window| { return window == b"extra"; }), "the included file is listed");
    assert!(expected.stdout.windows(7).any(|window| { return window == b"command"; }), "the injected entries are listed");
    // A conditional include is evaluated by Git, so the reader leaves the listing to it.
    git(repo.as_path(), &["config", "--local", "includeIf.gitdir:/nowhere/.path", "extra.config"]);
    assert!(mine(root.as_path(), repo.as_path(), &["config", "--list", "-z", "--show-scope"]).is_none());
    remove(root.as_path());
}

/// With the per-worktree file on,
/// the listing carries its entries under the scope `worktree`,
/// after the local ones,
///  equal to Git's for the main and the linked worktree;
/// with the extension off the file is not listed,
/// and what the
///  reader cannot follow (a conditional include,
/// a value that is not a boolean) is left to Git.
#[test]
fn worktree_configuration_listings_equal_git() {
    let root = fixture("repo-read-refs-worktree-config");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    let linked = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked.to_str().expect("UTF-8"), "HEAD"]);
    git(repo.as_path(), &["config", "--local", "extensions.worktreeConfig", "true"]);
    std::fs::write(repo.join(".git/config.worktree"), "[user]\n\tname = Only Here\n[include]\n\tpath = wt-extra\n[flag]\n\tbare\n").expect("main file");
    std::fs::write(repo.join(".git/wt-extra"), "[extra]\n\tvalue = from include\n").expect("included");
    let linked_git = repo.join(".git/worktrees/linked");
    std::fs::write(linked_git.join("config.worktree"), "[user]\n\tname = Linked\n").expect("linked file");
    let listing = ["config", "--list", "-z", "--show-scope"];
    let expected = git_output(repo.as_path(), &listing);
    assert!(expected.stdout.windows(15).any(|window| { return window == b"worktree\0user.n"; }), "Git lists the per-worktree file under its own scope");
    answered(root.as_path(), repo.as_path(), &listing);
    answered(root.as_path(), linked.as_path(), &listing);
    // Off: Git does not read the file, so neither does the reader.
    git(repo.as_path(), &["config", "--local", "extensions.worktreeConfig", "false"]);
    assert!(!git_output(repo.as_path(), &listing).stdout.windows(9).any(|window| { return window == b"worktree\0"; }));
    answered(root.as_path(), repo.as_path(), &listing);
    git(repo.as_path(), &["config", "--local", "extensions.worktreeConfig", "true"]);
    // A conditional include in the per-worktree file is evaluated by Git.
    std::fs::write(repo.join(".git/config.worktree"), "[includeIf \"gitdir:/nowhere/\"]\n\tpath = wt-extra\n").expect("conditional");
    declined(root.as_path(), repo.as_path(), &listing);
    // The per-worktree file is the one beside the Git directory of the worktree asked, not the main one.
    std::fs::write(repo.join(".git/config.worktree"), "[user]\n\tname = Main only\n").expect("main file");
    std::fs::remove_file(linked_git.join("config.worktree")).expect("no linked file");
    answered(root.as_path(), linked.as_path(), &listing);
    remove(root.as_path());
}

/// Git clears the repository format when `core.repositoryformatversion` is absent (`read_repository_format` in
///  `setup.c`),
/// so `extensions.worktreeConfig` lists no `worktree` entries there;
/// the reader must list what Git lists.
#[test]
fn an_absent_format_version_lists_no_worktree_entries() {
    let root = fixture("repo-read-refs-wt-no-version");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    let listing = ["config", "--list", "-z", "--show-scope"];
    git(repo.as_path(), &["config", "--local", "extensions.worktreeConfig", "true"]);
    std::fs::write(repo.join(".git/config.worktree"), "[user]\n\tname = Only Here\n").expect("main file");
    let with_version = git_output(repo.as_path(), &listing);
    assert!(with_version.stdout.windows(9).any(|window| { return window == b"worktree\0"; }), "a version of 0 lists the file");
    git(repo.as_path(), &["config", "--local", "--unset", "core.repositoryformatversion"]);
    let without_version = git_output(repo.as_path(), &listing);
    assert!(!without_version.stdout.windows(9).any(|window| { return window == b"worktree\0"; }), "Git lists no worktree entries");
    answered(root.as_path(), repo.as_path(), &listing);
    remove(root.as_path());
}

/// The listing of a repository with only its own file equals Git's,
/// and so does a remote URL.
#[test]
fn local_listings_and_remote_urls_equal_git() {
    let root = fixture("repo-read-refs-remote");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    answered(root.as_path(), repo.as_path(), &["config", "--list", "-z", "--show-scope"]);
    answered(root.as_path(), repo.as_path(), &["remote", "get-url", "origin"]);
    // Git's refusal for a remote that does not exist is reproduced word for word and with its exit code.
    answered(root.as_path(), repo.as_path(), &["remote", "get-url", "missing"]);
    std::fs::create_dir_all(repo.join(".git/remotes")).expect("legacy directory");
    std::fs::write(repo.join(".git/remotes/legacy"), "URL: https://example.invalid/legacy.git\n").expect("legacy remote");
    declined(root.as_path(), repo.as_path(), &["remote", "get-url", "legacy"]);
    git(repo.as_path(), &["remote", "set-url", "--add", "origin", "https://example.invalid/b.git"]);
    declined(root.as_path(), repo.as_path(), &["remote", "get-url", "origin"]);
    let _ = git_output(repo.as_path(), &["config", "--unset-all", "remote.origin.url"]);
    git(repo.as_path(), &["config", "remote.origin.url", "https://example.invalid/a.git"]);
    answered(root.as_path(), repo.as_path(), &["remote", "get-url", "origin"]);
    git(repo.as_path(), &["config", "url.https://mirror.invalid/.insteadOf", "https://example.invalid/"]);
    declined(root.as_path(), repo.as_path(), &["remote", "get-url", "origin"]);
    remove(root.as_path());
}

/// Ref verification,
/// the ref listing and the upstream name equal Git's in loose and packed states.
#[test]
fn references_equal_git() {
    let root = fixture("repo-read-refs-all");
    for format in ["sha1", "sha256"] {
        let repo = referenced_repository(root.as_path(), format, format);
        git(repo.as_path(), &["config", "branch.main.remote", "origin"]);
        let shapes = |target: &Path| {
            for name in ["refs/heads/main", "refs/heads/feature/deep/name", "refs/tags/light", "refs/tags/v1", "refs/remotes/origin/main", "refs/remotes/origin/HEAD"] {
                answered(root.as_path(), target, &["rev-parse", "--verify", "--quiet", &format!("{name}^{{commit}}")]);
            }
            declined(root.as_path(), target, &["rev-parse", "--verify", "--quiet", "refs/heads/missing^{commit}"]);
            declined(root.as_path(), target, &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"]);
            answered(root.as_path(), target, &["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]);
        };
        shapes(repo.as_path());
        // A half-configured upstream is left to Git; with neither key Git's failure is reproduced word for word.
        declined(root.as_path(), repo.as_path(), &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"]);
        git(repo.as_path(), &["config", "--unset", "branch.main.remote"]);
        answered(root.as_path(), repo.as_path(), &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"]);
        git(repo.as_path(), &["config", "branch.main.remote", "origin"]);
        git(repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
        answered(root.as_path(), repo.as_path(), &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"]);
        git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
        shapes(repo.as_path());
        answered(root.as_path(), repo.as_path(), &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"]);
        // A branch spelled like the short name makes it ambiguous: Git answers `remotes/origin/main`.
        git(repo.as_path(), &["branch", "origin/main"]);
        declined(root.as_path(), repo.as_path(), &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"]);
        git(repo.as_path(), &["branch", "--delete", "origin/main"]);
        git(repo.as_path(), &["checkout", "--quiet", "--detach"]);
        declined(root.as_path(), repo.as_path(), &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"]);
    }
    remove(root.as_path());
}

/// `symbolic-ref --quiet HEAD` equals Git's for a branch,
/// an unborn branch and a detached `HEAD`.
#[test]
fn the_symbolic_head_equals_git() {
    let root = fixture("repo-read-refs-symbolic");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    answered(root.as_path(), repo.as_path(), &["symbolic-ref", "--quiet", "HEAD"]);
    git(repo.as_path(), &["checkout", "--quiet", "--detach"]);
    answered(root.as_path(), repo.as_path(), &["symbolic-ref", "--quiet", "HEAD"]);
    git(repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/unborn"]);
    answered(root.as_path(), repo.as_path(), &["symbolic-ref", "--quiet", "HEAD"]);
    declined(root.as_path(), repo.as_path(), &["symbolic-ref", "HEAD"]);
    remove(root.as_path());
}

/// Symbolic chains,
/// repeated upstream settings,
/// dangling tracking refs and a bare-name file that shadows the short name
///  are the cases where Git's answer differs from a plain read,
/// so the reader leaves them to Git.
#[test]
fn chains_repeats_and_shadows_are_declined() {
    let root = fixture("repo-read-refs-chains");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    git(repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    let upstream = ["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"];
    answered(root.as_path(), repo.as_path(), &upstream);
    // Two merge settings: Git takes the first.
    git(repo.as_path(), &["config", "--add", "branch.main.merge", "refs/heads/feature/deep/name"]);
    declined(root.as_path(), repo.as_path(), &upstream);
    git(repo.as_path(), &["config", "--unset-all", "branch.main.merge"]);
    git(repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    answered(root.as_path(), repo.as_path(), &upstream);
    // A bare-name file in the Git directory is Git's first lookup rule.
    std::fs::create_dir_all(repo.join(".git/origin")).expect("directory");
    std::fs::write(repo.join(".git/origin/main"), format!("{}\n", "1".repeat(40))).expect("file");
    declined(root.as_path(), repo.as_path(), &upstream);
    std::fs::remove_dir_all(repo.join(".git/origin")).expect("remove");
    // A tracking ref that is symbolic and points nowhere does not resolve.
    git(repo.as_path(), &["symbolic-ref", "refs/remotes/origin/main", "refs/remotes/origin/gone"]);
    declined(root.as_path(), repo.as_path(), &upstream);
    git(repo.as_path(), &["update-ref", "--no-deref", "-d", "refs/remotes/origin/main"]);
    git(repo.as_path(), &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    answered(root.as_path(), repo.as_path(), &upstream);
    // `HEAD` through an alias branch follows the chain in Git.
    git(repo.as_path(), &["symbolic-ref", "refs/heads/alias", "refs/heads/main"]);
    git(repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/alias"]);
    declined(root.as_path(), repo.as_path(), &["symbolic-ref", "--quiet", "HEAD"]);
    declined(root.as_path(), repo.as_path(), &upstream);
    remove(root.as_path());
}

/// A key injected through the environment that can change a diff makes the content queries decline,
///  while an injected key that cannot (the wrapper injects hook settings) leaves them answered.
#[test]
fn injected_content_keys_are_declined() {
    let root = fixture("repo-read-refs-injected");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    let given: Vec<OsString> = ["ls-files", "--stage", "-z", "--full-name"].iter().map(OsString::from).collect();
    let host = scope(root.as_path());
    let ask = |key: &str| -> bool {
        let injected = pairs(&[("GIT_CONFIG_COUNT", "1"), ("GIT_CONFIG_KEY_0", key), ("GIT_CONFIG_VALUE_0", "x")]);
        return answer_with(&Query { arguments: given.as_slice(), directory: Some(repo.as_path()), variables: injected.as_slice(), input: None }, Vec::new(), &host).is_some();
    };
    assert!(ask("core.hooksPath"), "a hook setting is plain");
    assert!(!ask("diff.orderFile"), "an injected order file");
    assert!(!ask("Core.IgnoreCase"), "an injected case rule, spelled with capitals");
    remove(root.as_path());
}

/// A listing with a ref whose object is missing,
/// and a ref file with an odd spelling,
/// are left to Git.
#[test]
fn broken_refs_are_declined() {
    let root = fixture("repo-read-refs-broken");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    answered(root.as_path(), repo.as_path(), &["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]);
    let missing = "1".repeat(40);
    std::fs::write(repo.join(".git/refs/heads/broken"), format!("{missing}\n")).expect("broken");
    declined(root.as_path(), repo.as_path(), &["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]);
    std::fs::remove_file(repo.join(".git/refs/heads/broken")).expect("remove");
    std::fs::write(repo.join(".git/refs/heads/spaced"), format!("{}  \n", "2".repeat(40))).expect("odd");
    declined(root.as_path(), repo.as_path(), &["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]);
    std::fs::remove_file(repo.join(".git/refs/heads/spaced")).expect("remove");
    let _ = scope(root.as_path());
    remove(root.as_path());
}

/// A tag and a branch whose names hold `@` and `+`,
/// as this repository's own release tags do,
/// are listed and verified without a Git start:
/// the ref listing is what the shadow's snapshot and the LFS plan take,
/// and a name outside the plain alphabet used to end the whole listing.
#[test]
fn scoped_tags_and_plus_branches_equal_git() {
    let root = fixture("repo-read-refs-scoped");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    git(repo.as_path(), &["tag", "-a", "-m", "scoped", "@scope/package@1.0.0"]);
    git(repo.as_path(), &["tag", "v1+meta"]);
    git(repo.as_path(), &["branch", "a+b"]);
    let shapes = |target: &Path| {
        answered(root.as_path(), target, &["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]);
        for name in ["refs/tags/@scope/package@1.0.0", "refs/tags/v1+meta", "refs/heads/a+b"] {
            answered(root.as_path(), target, &["rev-parse", "--verify", "--quiet", &format!("{name}^{{commit}}")]);
        }
    };
    shapes(repo.as_path());
    git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
    shapes(repo.as_path());
    remove(root.as_path());
}

/// A name Git refuses is still left to Git,
/// and so is a listing that holds one:
/// Git words a broken name on standard error and prints the other refs without it.
#[test]
fn refs_git_refuses_stay_declined() {
    let root = fixture("repo-read-refs-refused");
    let repo = referenced_repository(root.as_path(), "repo", "sha1");
    let tip = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8").trim_end().to_string();
    for name in ["refs/heads/a b", "refs/heads/a*b", "refs/heads/a:b", "refs/heads/a?b", "refs/heads/a[b", "refs/heads/a\\b", "refs/heads/a^b", "refs/heads/a~b"] {
        declined(root.as_path(), repo.as_path(), &["rev-parse", "--verify", "--quiet", &format!("{name}^{{commit}}")]);
    }
    std::fs::write(repo.join(".git/refs/heads/refused name"), format!("{tip}\n")).expect("broken name");
    declined(root.as_path(), repo.as_path(), &["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]);
    // A broken packed name beside a good ref: Git words the name for a listing and prints the other refs without it,
    // while a lookup of the good ref answers with nothing on standard error.
    std::fs::remove_file(repo.join(".git/refs/heads/refused name")).expect("remove the loose broken name");
    git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
    let packed = repo.join(".git/packed-refs");
    let original = std::fs::read_to_string(&packed).expect("packed refs");
    std::fs::write(&packed, format!("{original}{tip} refs/heads/refused name\n")).expect("broken packed name");
    declined(root.as_path(), repo.as_path(), &["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]);
    answered(root.as_path(), repo.as_path(), &["rev-parse", "--verify", "--quiet", "refs/heads/main^{commit}"]);
    remove(root.as_path());
}
