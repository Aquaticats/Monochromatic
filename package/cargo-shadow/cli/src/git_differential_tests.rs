//! The differential tests of the plain-file base reader: Git itself is the oracle.
//! Each case builds a disposable repository with an isolated home, asks `/usr/bin/git` which base it would choose, and requires the
//! reader to either decline (so Git is asked) or give exactly Git's spelling and commit.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::git_base::resolve_trust_base_with;
use crate::git_native::native_base;
use crate::git_refs::{NativeBase, NativeEnvironment};
use crate::test_scratch::scratch;

/// The real Git binary; the `git` on the search path may be a policy wrapper.
const SYSTEM_GIT: &str = "/usr/bin/git";

/// The spellings the base choice tries, in order.
const CANDIDATES: [&str; 4] = ["@{upstream}", "origin/main", "origin/master", "HEAD"];

/// A disposable repository with two commits: `first` (the older) and `second` (the one `HEAD` points at on the branch `topic`).
struct Fixture {
    /// The directory that owns everything the case creates.
    root: tempfile::TempDir,
    /// The working directory of the repository.
    work: PathBuf,
    /// The older commit.
    first: String,
    /// The newer commit, `HEAD` of `topic`.
    second: String,
}

/// Runs the real Git in a directory with an environment that contains nothing but a disposable home.
fn run_git(root: &Path, directory: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new(SYSTEM_GIT)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("LC_ALL", "C")
        .args(["-c", "user.name=tester", "-c", "user.email=tester@example.invalid", "-c", "commit.gpgsign=false", "-c", "protocol.file.allow=always"])
        .args(arguments)
        .current_dir(directory)
        .output()
        .expect("start git");
    if !output.status.success() {
        return None;
    }
    return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
}

/// Runs Git and requires success.
fn git(fixture_root: &Path, directory: &Path, arguments: &[&str]) -> String {
    return run_git(fixture_root, directory, arguments).unwrap_or_else(|| { return panic!("git {arguments:?} failed in {}", directory.display()); });
}

/// The reader's environment: the disposable home and no system configuration.
fn environment(root: &Path) -> NativeEnvironment {
    return NativeEnvironment { home: Some(root.join("home")), xdg_config_home: None, system_config: root.join("no-system-config"), blocking_variable: None };
}

/// Builds the repository on the branch `branch`.
fn fixture(branch: &str) -> Fixture {
    let root = scratch();
    fs::create_dir_all(root.path().join("home")).expect("home");
    let work: PathBuf = root.path().join("work");
    fs::create_dir_all(&work).expect("work");
    git(root.path(), &work, &["init", "--quiet", "--initial-branch", branch]);
    fs::write(work.join("file"), "one\n").expect("file");
    git(root.path(), &work, &["add", "file"]);
    git(root.path(), &work, &["commit", "--quiet", "--message", "one"]);
    let first: String = git(root.path(), &work, &["rev-parse", "HEAD"]);
    fs::write(work.join("file"), "two\n").expect("file");
    git(root.path(), &work, &["commit", "--quiet", "--all", "--message", "two"]);
    let second: String = git(root.path(), &work, &["rev-parse", "HEAD"]);
    return Fixture { root, work, first, second };
}

/// Replaces the repository configuration with the plain core section followed by `text`.
fn write_config(fixture: &Fixture, text: &str) {
    let core: &str = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n";
    fs::write(fixture.work.join(".git/config"), format!("{core}{text}")).expect("config");
}

/// Writes a loose reference by hand, the way a fetch or a broken tool could.
fn write_ref(directory: &Path, name: &str, object: &str) {
    let path: PathBuf = directory.join(name);
    fs::create_dir_all(path.parent().expect("parent")).expect("ref directory");
    fs::write(path, format!("{object}\n")).expect("ref");
}

/// What Git chooses: the first candidate that names a commit, with its commit.
fn oracle(fixture: &Fixture, directory: &Path) -> Option<(String, String)> {
    for spelling in CANDIDATES {
        let revision: String = format!("{spelling}^{{commit}}");
        if let Some(commit) = run_git(fixture.root.path(), directory, &["rev-parse", "--verify", "--quiet", "--end-of-options", &revision]) {
            return Some((spelling.to_string(), commit));
        }
    }
    return None;
}

/// Requires the reader to decline or to agree with Git, in the directory `directory`.
fn assert_agrees_at(fixture: &Fixture, directory: &Path, label: &str) {
    let expected: Option<(String, String)> = oracle(fixture, directory);
    let native: Result<NativeBase, String> = native_base(directory, false, &environment(fixture.root.path()));
    match native {
        Err(_) => {}
        Ok(base) => {
            let found: Option<(String, String)> = base.commit.clone().ok().map(|commit| { return (base.spelling.clone(), commit); });
            assert_eq!(found, expected, "{label}: the reader answered {base:?} but Git chooses {expected:?}");
        }
    }
}

/// Requires the reader to decline or to agree with Git, in the repository's working directory.
fn assert_agrees(fixture: &Fixture, label: &str) {
    assert_agrees_at(fixture, &fixture.work, label);
}

/// Requires the reader to decline.
fn assert_declines(fixture: &Fixture, label: &str) {
    let native: Result<NativeBase, String> = native_base(&fixture.work, false, &environment(fixture.root.path()));
    assert!(native.is_err(), "{label}: the reader answered {native:?} where only Git can say");
}

/// Makes the branch `base` point at the older commit, so a correct upstream selection is distinguishable from `HEAD`.
fn base_branch_at_first(fixture: &Fixture) {
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.first]);
}

/// The branch section that makes `topic` track `refs/heads/base` of this repository.
const TOPIC_TRACKS_BASE: &str = "[branch \"topic\"]\n\tremote = .\n\tmerge = refs/heads/base\n";

/// A control: the plain upstream case agrees, so the other cases test the difference and not the harness.
#[test]
fn a_plain_local_upstream_agrees_with_git() {
    let fixture = fixture("topic");
    base_branch_at_first(&fixture);
    write_config(&fixture, TOPIC_TRACKS_BASE);

    assert_agrees(&fixture, "plain upstream");
    let answered = native_base(&fixture.work, false, &environment(fixture.root.path())).expect("the plain case is answered from the files");
    assert_eq!(answered.commit, Ok(fixture.first.clone()));
}

/// Review finding (2a 1): a comment that ends in a backslash does not continue onto the next line, so the include below it is real.
#[test]
fn a_comment_ending_in_a_backslash_does_not_hide_the_include_below_it() {
    let fixture = fixture("topic");
    base_branch_at_first(&fixture);
    let included: PathBuf = fixture.root.path().join("upstream.conf");
    fs::write(&included, TOPIC_TRACKS_BASE).expect("included file");
    write_config(&fixture, &format!("\t# a comment \\\n[include]\n\tpath = {}\n", included.display()));

    assert_agrees(&fixture, "comment then include");
    assert_declines(&fixture, "comment then include");
}

/// Review finding (2a 2): every Git spelling of true for `extensions.worktreeConfig` makes the reader leave the answer to Git.
#[test]
fn every_true_spelling_of_the_worktree_configuration_extension_is_left_to_git() {
    for spelling in ["true", "TRUE", "yes", "on", "1", "2", "-1"] {
        let fixture = fixture("topic");
        base_branch_at_first(&fixture);
        write_config(&fixture, "");
        fs::write(
            fixture.work.join(".git/config"),
            format!("[core]\n\trepositoryformatversion = 1\n[extensions]\n\tworktreeConfig = {spelling}\n"),
        )
        .expect("config");
        fs::write(fixture.work.join(".git/config.worktree"), TOPIC_TRACKS_BASE).expect("worktree config");

        assert_agrees(&fixture, spelling);
        assert_declines(&fixture, spelling);
    }
}

/// Review finding (2a 3): with several `merge` values Git uses the first.
#[test]
fn the_first_of_several_merge_values_is_the_upstream() {
    let fixture = fixture("topic");
    base_branch_at_first(&fixture);
    write_config(&fixture, "[branch \"topic\"]\n\tremote = .\n\tmerge = refs/heads/base\n\tmerge = refs/heads/topic\n");

    assert_agrees(&fixture, "two merge values");
    let answered = native_base(&fixture.work, false, &environment(fixture.root.path())).expect("answered");
    assert_eq!(answered.commit, Ok(fixture.first.clone()));
}

/// Review finding (2a 4): a dotted section name followed by a quoted subsection is one flat name in Git.
#[test]
fn a_dotted_section_with_a_quoted_subsection_names_the_dotted_branch() {
    let fixture = fixture("topic.sub");
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.first]);
    write_config(&fixture, "[branch.topic \"sub\"]\n\tremote = .\n\tmerge = refs/heads/base\n");

    assert_agrees(&fixture, "dotted section and quoted subsection");
}

/// Review finding (2a 4): a quoted subsection may contain a closing bracket, an escaped quote and a hash sign.
#[test]
fn a_quoted_subsection_may_hold_a_bracket_an_escaped_quote_and_a_hash() {
    let fixture = fixture("topic\"]#suffix");
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.first]);
    write_config(&fixture, "[branch \"topic\\\"]#suffix\"]\n\tremote = .\n\tmerge = refs/heads/base\n");

    assert_agrees(&fixture, "bracket in subsection");
}

/// Review finding (2a 5): a quoted value keeps its trailing characters, even a no-break space that is a legal reference-name character.
#[test]
fn a_quoted_merge_value_keeps_a_trailing_no_break_space() {
    let fixture = fixture("topic");
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.second]);
    git(fixture.root.path(), &fixture.work, &["branch", "base\u{a0}", &fixture.first]);
    write_config(&fixture, "[branch \"topic\"]\n\tremote = .\n\tmerge = \"refs/heads/base\u{a0}\"\n");

    assert_agrees(&fixture, "no-break space");
}

/// Review finding (2a 6): a key without a value is not the string `true`; Git refuses the missing value.
#[test]
fn a_remote_key_without_a_value_is_not_a_remote_named_true() {
    let fixture = fixture("topic");
    write_ref(&fixture.work.join(".git"), "refs/remotes/true/base", &fixture.first);
    write_config(&fixture, "[branch \"topic\"]\n\tremote\n\tmerge = refs/heads/base\n[remote \"true\"]\n\tfetch = +refs/heads/*:refs/remotes/true/*\n");

    assert_agrees(&fixture, "bare remote key");
}

/// Review finding (2a 7): references of a linked worktree's private namespaces live in that worktree's own directory.
#[test]
fn a_private_reference_of_a_linked_worktree_is_not_looked_for_in_the_common_directory() {
    let fixture = fixture("topic");
    let linked: PathBuf = fixture.root.path().join("linked");
    git(fixture.root.path(), &fixture.work, &["worktree", "add", "--quiet", "-b", "topic2", linked.to_str().expect("utf8"), "HEAD"]);
    write_config(&fixture, "[branch \"topic2\"]\n\tremote = .\n\tmerge = refs/worktree/base\n");
    git(fixture.root.path(), &linked, &["update-ref", "refs/worktree/base", &fixture.first]);

    assert_agrees_at(&fixture, &linked, "private reference");
}

/// Review finding (2a 8): the short name `origin/main` also resolves through `refs/remotes/origin/main/HEAD`.
#[test]
fn a_short_name_resolves_through_the_remote_head_of_a_remote_named_like_a_branch() {
    let fixture = fixture("topic");
    let dot_git: PathBuf = fixture.work.join(".git");
    git(fixture.root.path(), &fixture.work, &["update-ref", "refs/remotes/origin/main/stable", &fixture.first]);
    git(fixture.root.path(), &fixture.work, &["symbolic-ref", "refs/remotes/origin/main/HEAD", "refs/remotes/origin/main/stable"]);
    write_config(&fixture, "");

    assert!(dot_git.join("refs/remotes/origin/main/HEAD").exists());
    assert_agrees(&fixture, "remote HEAD of a remote named origin/main");
}

/// Review finding (2a 9): an object name in a reference is evidence of a commit only when the object is one (tags peel, blobs and
/// missing objects are refused).
#[test]
fn an_annotated_tag_a_blob_and_a_missing_object_in_a_remote_reference_are_not_taken_for_commits() {
    let tag_fixture = fixture("topic");
    git(tag_fixture.root.path(), &tag_fixture.work, &["tag", "-a", "-m", "tag", "v1", &tag_fixture.first]);
    let tag_object: String = git(tag_fixture.root.path(), &tag_fixture.work, &["rev-parse", "v1"]);
    write_ref(&tag_fixture.work.join(".git"), "refs/remotes/origin/main", &tag_object);
    assert_agrees(&tag_fixture, "annotated tag");

    let blob_fixture = fixture("topic");
    let blob: String = git(blob_fixture.root.path(), &blob_fixture.work, &["rev-parse", "HEAD:file"]);
    write_ref(&blob_fixture.work.join(".git"), "refs/remotes/origin/main", &blob);
    assert_agrees(&blob_fixture, "blob");

    let missing_fixture = fixture("topic");
    write_ref(&missing_fixture.work.join(".git"), "refs/remotes/origin/main", "1234567890123456789012345678901234567890");
    assert_agrees(&missing_fixture, "missing object");
}

/// Review finding (2a 11): the path after `gitdir: ` keeps its leading space, and a `commondir` that cannot be read is not absent.
#[test]
fn a_gitdir_pointer_keeps_the_space_in_its_path() {
    let fixture = fixture("topic");
    let outer: PathBuf = fixture.root.path().join("outer");
    fs::create_dir_all(&outer).expect("outer");
    git(fixture.root.path(), &outer, &["clone", "--quiet", "--bare", fixture.work.to_str().expect("utf8"), "git-a"]);
    git(fixture.root.path(), &outer, &["clone", "--quiet", "--bare", fixture.work.to_str().expect("utf8"), " git-a"]);
    git(fixture.root.path(), &outer.join(" git-a"), &["update-ref", "refs/heads/topic", &fixture.first]);
    fs::write(outer.join(".git"), "gitdir:  git-a\n").expect("pointer");

    assert_agrees_at(&fixture, &outer, "gitdir with a leading space");
}

/// Review finding (2a 11): an unreadable `commondir` is a failure, not the absence of one.
#[test]
fn an_unreadable_commondir_is_not_an_absent_one() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = fixture("topic");
    let linked: PathBuf = fixture.root.path().join("linked");
    git(fixture.root.path(), &fixture.work, &["worktree", "add", "--quiet", "--detach", linked.to_str().expect("utf8"), "HEAD"]);
    let commondir: PathBuf = fixture.work.join(".git/worktrees/linked/commondir");
    fs::set_permissions(&commondir, fs::Permissions::from_mode(0o000)).expect("chmod");

    assert_agrees_at(&fixture, &linked, "unreadable commondir");
    fs::set_permissions(&commondir, fs::Permissions::from_mode(0o644)).expect("chmod back");
}

/// Review finding (2a 12): an unknown repository extension makes Git refuse the repository, so the reader must not answer.
#[test]
fn an_unknown_repository_extension_is_left_to_git() {
    let fixture = fixture("topic");
    fs::write(fixture.work.join(".git/config"), "[core]\n\trepositoryformatversion = 1\n[extensions]\n\tsomethingnew = yes\n").expect("config");

    assert_agrees(&fixture, "unknown extension");
    assert_declines(&fixture, "unknown extension");
}

/// Review finding (2a 12): a fetch refspec that Git rejects is not skipped because an earlier one matched.
#[test]
fn an_invalid_refspec_after_a_matching_one_is_left_to_git() {
    let fixture = fixture("topic");
    write_ref(&fixture.work.join(".git"), "refs/remotes/origin/tracked/base", &fixture.first);
    write_config(
        &fixture,
        "[branch \"topic\"]\n\tremote = origin\n\tmerge = refs/heads/base\n[remote \"origin\"]\n\tfetch = +refs/heads/*:refs/remotes/origin/tracked/*\n\tfetch = refs/heads/**:refs/remotes/origin/**\n",
    );

    assert_agrees(&fixture, "invalid second refspec");
}

/// Review finding (2a 13): when the reader declines, one Git start answers every candidate.
#[test]
fn a_declined_choice_costs_exactly_one_git_start() {
    let fixture = fixture("topic");
    let included: PathBuf = fixture.root.path().join("empty.conf");
    fs::write(&included, "").expect("included");
    write_config(&fixture, &format!("[include]\n\tpath = {}\n", included.display()));

    let base = resolve_trust_base_with(&fixture.work, false, None, &environment(fixture.root.path()));

    assert_eq!(base.git_starts, 1, "{base:?}");
    assert_eq!(base.spelling, "HEAD");
    assert_eq!(base.commit, Ok(fixture.second.clone()));
}
