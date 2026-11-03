//! The third review of the Git reader (3a): symbolic-link `HEAD`, remote `.`, path text, object validity, configuration scope,
//! the other system configuration places, ownership, and the one Git start that answers a declined choice.
//! Git itself is the oracle, run with an isolated home and the same program as the fallback.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::git_base::resolve_trust_base_with;
use crate::git_differential_tests::{Fixture, TOPIC_TRACKS_BASE, assert_agrees, assert_agrees_at, assert_declines, base_branch_at_first, environment, fixture, git, oracle, write_config, write_ref};
use crate::git_refs::{GitProgram, NativeEnvironment};
use crate::test_scratch::scratch;

/// The real Git, so the fallback runs the program the oracle runs.
const SYSTEM_GIT: &str = "/usr/bin/git";

/// The reader's environment whose fallback runs the real Git with the isolated home, like the oracle.
fn controlled(fixture: &Fixture) -> NativeEnvironment {
    let mut base: NativeEnvironment = environment(fixture.root.path());
    base.git = GitProgram {
        program: PathBuf::from(SYSTEM_GIT),
        bound: Duration::from_secs(30),
        clean_environment: true,
        environment: vec![
            ("PATH".to_string(), "/usr/bin:/bin".to_string()),
            ("HOME".to_string(), fixture.root.path().join("home").display().to_string()),
            ("GIT_CONFIG_NOSYSTEM".to_string(), "1".to_string()),
            ("LC_ALL".to_string(), "C".to_string()),
        ],
    };
    return base;
}

/// Requires the whole choice (the reader, then the fallback) to equal Git's.
fn assert_end_to_end(fixture: &Fixture, label: &str) {
    let expected: Option<(String, String)> = oracle(fixture, &fixture.work);
    let base = resolve_trust_base_with(&fixture.work, false, None, &controlled(fixture));
    let found: Option<(String, String)> = base.commit.clone().ok().map(|commit| { return (base.spelling.clone(), commit); });
    assert_eq!(found, expected, "{label}: the choice is {base:?} but Git chooses {expected:?}");
}

/// Writes a configuration whose upstream lives in an included file, so the reader must decline and Git answers.
fn include_with(fixture: &Fixture, included_text: &str) {
    let included: PathBuf = fixture.root.path().join("included.conf");
    fs::write(&included, included_text).expect("included file");
    write_config(fixture, &format!("[include]\n\tpath = {}\n", included.display()));
}

/// Review finding (3a 1): a symbolic-link `HEAD` is a symbolic reference, not a detached object.
#[test]
fn a_symbolic_link_head_is_not_a_detached_head() {
    let fixture = fixture("topic");
    base_branch_at_first(&fixture);
    write_config(&fixture, &format!("[core]\n\tpreferSymlinkRefs = true\n{TOPIC_TRACKS_BASE}"));
    let head: PathBuf = fixture.work.join(".git/HEAD");
    fs::remove_file(&head).expect("remove HEAD");
    symlink("refs/heads/topic", &head).expect("a symbolic-link HEAD");

    assert_agrees(&fixture, "symbolic-link HEAD");
}

/// Review finding (3a 2): the remote `.` maps through its fetch refspecs like any other remote.
#[test]
fn the_remote_dot_maps_through_its_fetch_refspecs() {
    let fixture = fixture("topic");
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.second]);
    write_ref(&fixture.work.join(".git"), "refs/remotes/local/base", &fixture.first);
    write_config(&fixture, &format!("{TOPIC_TRACKS_BASE}[remote \".\"]\n\tfetch = +refs/heads/*:refs/remotes/local/*\n"));

    assert_agrees(&fixture, "remote dot with a fetch refspec");
}

/// Review finding (3a 2): a merge name that exists only through Git's name expansion is Git's to resolve.
#[test]
fn a_merge_name_that_needs_name_expansion_is_left_to_git() {
    let fixture = fixture("topic");
    git(fixture.root.path(), &fixture.work, &["branch", "refs/heads/base", &fixture.first]);
    write_config(&fixture, TOPIC_TRACKS_BASE);

    assert_agrees(&fixture, "merge name that needs expansion");
}

/// Review finding (3a 3): the `.git` file's path keeps trailing spaces; only the line ending is dropped.
#[test]
fn a_gitdir_pointer_keeps_trailing_spaces() {
    let fixture = fixture("topic");
    let outer: PathBuf = fixture.root.path().join("outer");
    fs::create_dir_all(&outer).expect("outer");
    git(fixture.root.path(), &outer, &["clone", "--quiet", "--bare", fixture.work.to_str().expect("utf8"), "git-a"]);
    git(fixture.root.path(), &outer, &["clone", "--quiet", "--bare", fixture.work.to_str().expect("utf8"), "git-a "]);
    git(fixture.root.path(), &outer.join("git-a "), &["update-ref", "refs/heads/topic", &fixture.first]);
    fs::write(outer.join(".git"), "gitdir: git-a \n").expect("pointer");

    assert_agrees_at(&fixture, &outer, "gitdir with a trailing space");
}

/// Review finding (3a 4): a loose object with a readable prefix and a damaged rest is not a commit.
#[test]
fn a_truncated_or_malformed_loose_object_is_not_confirmed_as_a_commit() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    let common: PathBuf = fixture.work.join(".git");
    let loose: PathBuf = common.join("objects").join(&fixture.second[..2]).join(&fixture.second[2..]);
    let whole: Vec<u8> = fs::read(&loose).expect("loose commit");
    assert_eq!(confirm_commit(&common, &fixture.second), Ok(()), "control: the whole object is a commit");

    fs::set_permissions(&loose, fs::Permissions::from_mode(0o644)).expect("writable");
    fs::write(&loose, &whole[..whole.len() / 2]).expect("truncate");
    assert!(confirm_commit(&common, &fixture.second).is_err(), "a truncated stream");

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut encoder, b"commit 5\0hello").expect("compress");
    fs::write(&loose, encoder.finish().expect("finish")).expect("write a malformed commit");
    assert!(confirm_commit(&common, &fixture.second).is_err(), "a body that is not a commit");
}

/// Review finding (3a 4): a pack entry that is cut off after its type byte is not a commit.
#[test]
fn a_truncated_pack_entry_is_not_confirmed_as_a_commit() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    git(fixture.root.path(), &fixture.work, &["gc", "--quiet"]);
    let common: PathBuf = fixture.work.join(".git");
    assert_eq!(confirm_commit(&common, &fixture.second), Ok(()), "control: the packed commit is confirmed");
    let pack: PathBuf = fs::read_dir(common.join("objects/pack")).expect("packs").flatten().map(|entry| { return entry.path(); }).find(|path| { return path.extension().is_some_and(|extension| { return extension == "pack"; }); }).expect("a pack");
    fs::set_permissions(&pack, fs::Permissions::from_mode(0o644)).expect("writable");
    fs::OpenOptions::new().write(true).open(&pack).expect("open").set_len(13).expect("cut the pack after one entry byte");

    assert!(confirm_commit(&common, &fixture.second).is_err());
}

/// Review finding (3a 5): a corrupt lower-priority candidate must not hide an existing upstream.
#[test]
fn a_corrupt_default_branch_does_not_hide_the_upstream() {
    let fixture = fixture("topic");
    base_branch_at_first(&fixture);
    include_with(&fixture, TOPIC_TRACKS_BASE);
    let bad: &str = "9999999999999999999999999999999999999999";
    write_ref(&fixture.work.join(".git"), "refs/remotes/origin/master", bad);
    let loose: PathBuf = fixture.work.join(".git/objects/99");
    fs::create_dir_all(&loose).expect("object directory");
    fs::write(loose.join(&bad[2..]), b"this is not a zlib stream").expect("a corrupt loose object");

    assert_end_to_end(&fixture, "corrupt origin/master with a valid upstream");
}

/// The declined choice agrees with Git end to end for the ordinary shapes.
#[test]
fn a_declined_choice_agrees_with_git_for_the_ordinary_shapes() {
    let upstream = fixture("topic");
    base_branch_at_first(&upstream);
    include_with(&upstream, TOPIC_TRACKS_BASE);
    assert_end_to_end(&upstream, "upstream through an include");

    let default_branch = fixture("topic");
    write_ref(&default_branch.work.join(".git"), "refs/remotes/origin/main", &default_branch.first);
    include_with(&default_branch, "");
    assert_end_to_end(&default_branch, "origin/main through an include");

    let head_only = fixture("topic");
    include_with(&head_only, "");
    assert_end_to_end(&head_only, "HEAD only");
}

/// Review finding (3a 6): a bare repository format version is Git's to reject.
#[test]
fn a_bare_repository_format_version_is_left_to_git() {
    let fixture = fixture("topic");
    fs::write(fixture.work.join(".git/config"), "[core]\n\trepositoryformatversion\n").expect("config");

    assert_agrees(&fixture, "bare format version");
    assert_declines(&fixture, "bare format version");
}

/// Review finding (3a 6): an invalid refspec of a remote that is not used still fails Git's remote configuration.
#[test]
fn an_invalid_refspec_of_an_unused_remote_is_left_to_git() {
    let fixture = fixture("topic");
    base_branch_at_first(&fixture);
    write_config(&fixture, &format!("{TOPIC_TRACKS_BASE}[remote \"unused\"]\n\tfetch = refs/heads/**:refs/remotes/unused/**\n"));

    assert_agrees(&fixture, "invalid refspec of an unused remote");
}

/// Review finding (3a 6): a required string written without a value on another branch breaks Git's remote configuration.
#[test]
fn a_bare_remote_key_on_another_branch_is_left_to_git() {
    let fixture = fixture("topic");
    base_branch_at_first(&fixture);
    write_config(&fixture, &format!("{TOPIC_TRACKS_BASE}[branch \"other\"]\n\tremote\n"));

    assert_agrees(&fixture, "bare remote on another branch");
}

/// Review finding (3a 7): a system configuration in a place a differently built Git reads is Git's to apply.
#[test]
fn a_system_configuration_in_another_known_place_is_left_to_git() {
    let fixture = fixture("topic");
    base_branch_at_first(&fixture);
    write_config(&fixture, "");
    let other: PathBuf = fixture.root.path().join("other-etc-gitconfig");
    fs::write(&other, TOPIC_TRACKS_BASE).expect("the other system configuration");
    let mut environment: NativeEnvironment = environment(fixture.root.path());
    environment.other_system_configs = vec![other];

    let answered = crate::git_native::native_base(&fixture.work, false, &environment);

    assert!(answered.is_err(), "{answered:?}");
}

/// Review finding (3a 8): a repository that another user owns is Git's to accept or refuse.
#[test]
fn a_repository_owned_by_another_user_is_left_to_git() {
    let fixture = fixture("topic");
    let mut environment: NativeEnvironment = environment(fixture.root.path());
    environment.user_id = environment.user_id.wrapping_add(1);

    let answered = crate::git_native::native_base(&fixture.work, false, &environment);

    assert!(answered.is_err(), "{answered:?}");
}

/// Writes an executable shell script.
fn script(directory: &Path, name: &str, body: &str) -> PathBuf {
    let path: PathBuf = directory.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("script");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
    return path;
}

/// Review finding (3a 9) and the owner's rule of 2026-11-03: a Git that outlives its bound is stopped and named.
#[test]
fn a_git_that_outlives_its_bound_is_stopped_and_named() {
    let fixture = fixture("topic");
    include_with(&fixture, "");
    let mut environment: NativeEnvironment = controlled(&fixture);
    let directory = scratch();
    environment.git.program = script(directory.path(), "slow-git", "sleep 5");
    environment.git.bound = Duration::from_millis(300);
    let started = Instant::now();

    let base = resolve_trust_base_with(&fixture.work, false, None, &environment);

    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    let message: String = base.commit.expect_err("a stopped git has no answer");
    assert!(message.contains("did not finish"), "{message}");
}

/// Review finding (3a 9): a Git that exits without reading its input is reaped, and the choice reports the failure.
#[test]
fn a_git_that_exits_without_reading_is_reaped_and_reported() {
    let fixture = fixture("topic");
    include_with(&fixture, "");
    let mut environment: NativeEnvironment = controlled(&fixture);
    let directory = scratch();
    environment.git.program = script(directory.path(), "quitting-git", "exit 1");

    let base = resolve_trust_base_with(&fixture.work, false, None, &environment);

    assert!(base.commit.is_err(), "{base:?}");
}
