//! Controls for the scratch locations,
//!  each in its own disposable directory under the target directory.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;

/// An environment of the given pairs.
fn environment(pairs: &[(&str, &Path)]) -> Vec<(OsString, OsString)> {
    return pairs
        .iter()
        .map(|(name, value)| { return (OsString::from(name), value.as_os_str().to_os_string()); })
        .collect();
}

/// The permission bits of a path without following a final link.
fn mode_of(path: &Path) -> u32 {
    return std::fs::symlink_metadata(path).expect("metadata").permissions().mode() & 0o7777;
}

/// Set a file or directory's modification time to `back` before now.
fn age(path: &Path, back: Duration) {
    std::fs::File::open(path)
        .expect("open")
        .set_modified(SystemTime::now() - back)
        .expect("set modification time");
}

/// An absolute `XDG_STATE_HOME` wins; a relative one is ignored as the XDG rules say; `HOME` is the fallback.
#[test]
fn the_state_directory_follows_the_xdg_rules() {
    assert_eq!(
        state_directory(&environment(&[("XDG_STATE_HOME", Path::new("/state")), ("HOME", Path::new("/home/u"))])),
        Some(PathBuf::from("/state/cli-git"))
    );
    assert_eq!(
        state_directory(&environment(&[("XDG_STATE_HOME", Path::new("relative")), ("HOME", Path::new("/home/u"))])),
        Some(PathBuf::from("/home/u/.local/state/cli-git"))
    );
    assert_eq!(
        state_directory(&environment(&[("HOME", Path::new("/home/u"))])),
        Some(PathBuf::from("/home/u/.local/state/cli-git"))
    );
}

/// Without an absolute `XDG_STATE_HOME` and without a non-empty `HOME` there is no directory, and the sentence names
/// both variables and says the system temporary directory is not a fallback.
#[test]
fn a_missing_home_is_named_and_never_falls_back_to_the_system_directory() {
    assert_eq!(state_directory(&[]), None);
    assert_eq!(state_directory(&environment(&[("HOME", Path::new(""))])), None);
    let reason: String = user_scratch_path(&[]).expect_err("no home");
    assert!(reason.contains("XDG_STATE_HOME") && reason.contains("HOME"), "{reason}");
    assert!(reason.contains("never in the system temporary directory"), "{reason}");
    assert_eq!(prepare_user_scratch(&[]).expect_err("no home"), reason);
}

/// The per-user scratch directory is made with every missing parent at mode `0700`, and a second preparation accepts it.
#[test]
fn the_user_scratch_directory_is_created_private_with_its_parents() {
    let root: PathBuf = fixture("scratch-user-create");
    let home: PathBuf = root.join("home");
    std::fs::create_dir(&home).expect("home");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    let scratch: PathBuf = prepare_user_scratch(env.as_slice()).expect("first preparation");
    assert_eq!(scratch, home.join(".local/state/cli-git/scratch"));
    for directory in [".local", ".local/state", ".local/state/cli-git", ".local/state/cli-git/scratch"] {
        assert_eq!(mode_of(home.join(directory).as_path()), 0o700, "{directory}");
    }
    assert_eq!(prepare_user_scratch(env.as_slice()).expect("second preparation"), scratch);
    remove(root.as_path());
}

/// A scratch path that is a link is refused and never followed, so no write can be redirected through it.
#[test]
fn a_linked_scratch_directory_is_refused() {
    let root: PathBuf = fixture("scratch-user-link");
    let home: PathBuf = root.join("home");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(home.join(".local/state/cli-git")).expect("state");
    std::fs::create_dir(&elsewhere).expect("elsewhere");
    std::os::unix::fs::symlink(&elsewhere, home.join(".local/state/cli-git/scratch")).expect("plant link");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    let reason: String = prepare_user_scratch(env.as_slice()).expect_err("a link is refused");
    assert!(reason.contains("not a directory"), "{reason}");
    let repository_link: PathBuf = root.join("common");
    std::fs::create_dir(&repository_link).expect("common");
    std::os::unix::fs::symlink(&elsewhere, repository_link.join(REPOSITORY_SCRATCH_NAME)).expect("plant link");
    assert!(prepare_repository_scratch(repository_link.as_path()).is_err());
    assert_eq!(std::fs::read_dir(&elsewhere).expect("elsewhere").count(), 0);
    remove(root.as_path());
}

/// An existing directory of this user with a looser mode is tightened to `0700`.
#[test]
fn a_loose_scratch_directory_of_this_user_is_tightened() {
    let root: PathBuf = fixture("scratch-user-loose");
    let home: PathBuf = root.join("home");
    let scratch: PathBuf = home.join(".local/state/cli-git/scratch");
    std::fs::create_dir_all(&scratch).expect("scratch");
    std::fs::set_permissions(&scratch, std::fs::Permissions::from_mode(0o755)).expect("loosen");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    prepare_user_scratch(env.as_slice()).expect("preparation");
    assert_eq!(mode_of(scratch.as_path()), 0o700);
    remove(root.as_path());
}

/// The repository scratch directory is made inside the common directory at mode `0700` and accepted again.
#[test]
fn the_repository_scratch_directory_is_created_private_beside_the_common_directory_entries() {
    let root: PathBuf = fixture("scratch-repository-create");
    let first: PathBuf = prepare_repository_scratch(root.as_path()).expect("first preparation");
    assert_eq!(first, root.join("cli-git-scratch"));
    assert_eq!(mode_of(first.as_path()), 0o700);
    assert_eq!(prepare_repository_scratch(root.as_path()).expect("second preparation"), first);
    let missing: PathBuf = root.join("no-such-common-directory");
    assert!(prepare_repository_scratch(missing.as_path()).is_err());
    remove(root.as_path());
}

/// The sweep removes files, directories with contents and links older than the bound, never follows a link, and keeps
/// fresh entries and entries dated in the future.
#[test]
fn the_sweep_removes_only_stale_entries_and_never_follows_links() {
    let root: PathBuf = fixture("scratch-sweep");
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    let outside: PathBuf = root.join("outside.txt");
    std::fs::write(&outside, b"outside").expect("outside file");
    age(outside.as_path(), SCRATCH_STALE_AFTER * 3);
    std::fs::write(scratch.join("cli-git-old-file"), b"old").expect("old file");
    std::fs::create_dir(scratch.join("cli-git-old-directory")).expect("old directory");
    std::fs::write(scratch.join("cli-git-old-directory/inner"), b"inner").expect("inner");
    std::fs::write(scratch.join("cli-git-fresh"), b"fresh").expect("fresh file");
    std::fs::write(scratch.join("cli-git-future"), b"future").expect("future file");
    std::os::unix::fs::symlink(&outside, scratch.join("cli-git-old-link")).expect("link");
    let long_ago: SystemTime = SystemTime::now() - SCRATCH_STALE_AFTER * 2;
    std::fs::File::open(scratch.join("cli-git-old-file")).expect("open").set_modified(long_ago).expect("age");
    std::fs::File::open(scratch.join("cli-git-old-directory/inner")).expect("open").set_modified(long_ago).expect("age");
    std::fs::File::open(scratch.join("cli-git-old-directory")).expect("open").set_modified(long_ago).expect("age");
    std::fs::File::open(scratch.join("cli-git-future"))
        .expect("open")
        .set_modified(SystemTime::now() + SCRATCH_STALE_AFTER * 2)
        .expect("future");
    // The link itself is fresh, so the first sweep keeps it: only its own time counts, never its target's.
    let removed: usize = sweep_stale(scratch.as_path(), SystemTime::now(), SCRATCH_STALE_AFTER);
    assert_eq!(removed, 2);
    let mut left: Vec<String> = std::fs::read_dir(&scratch)
        .expect("list")
        .map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); })
        .collect();
    left.sort();
    assert_eq!(left, ["cli-git-fresh", "cli-git-future", "cli-git-old-link"]);
    // Two days later the fresh file and the link are stale: both go, and the file the link points at stays.
    let later: SystemTime = SystemTime::now() + SCRATCH_STALE_AFTER * 2;
    assert_eq!(sweep_stale(scratch.as_path(), later, SCRATCH_STALE_AFTER), 2);
    assert!(std::fs::symlink_metadata(scratch.join("cli-git-old-link")).is_err(), "the link is gone");
    assert_eq!(std::fs::read(&outside).expect("target"), b"outside");
    remove(root.as_path());
}

/// A missing directory sweeps nothing and does not fail.
#[test]
fn sweeping_a_missing_directory_removes_nothing() {
    let root: PathBuf = fixture("scratch-sweep-missing");
    assert_eq!(sweep_stale(root.join("missing").as_path(), SystemTime::now(), SCRATCH_STALE_AFTER), 0);
    remove(root.as_path());
}

/// Both preparations collect what a killed process left, so stale files cannot accumulate without bound.
#[test]
fn preparing_a_directory_collects_what_a_killed_process_left() {
    let root: PathBuf = fixture("scratch-prepare-sweep");
    let home: PathBuf = root.join("home");
    let scratch: PathBuf = home.join(".local/state/cli-git/scratch");
    std::fs::create_dir_all(&scratch).expect("scratch");
    std::fs::write(scratch.join("cli-git-markdown-stale.jsonc"), b"[]").expect("stale");
    age(scratch.join("cli-git-markdown-stale.jsonc").as_path(), SCRATCH_STALE_AFTER * 2);
    std::fs::write(scratch.join("cli-git-markdown-live.jsonc"), b"[]").expect("live");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    prepare_user_scratch(env.as_slice()).expect("preparation");
    assert!(!scratch.join("cli-git-markdown-stale.jsonc").exists());
    assert!(scratch.join("cli-git-markdown-live.jsonc").exists());
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.join(REPOSITORY_SCRATCH_NAME)).expect("repository scratch");
    std::fs::write(common.join("cli-git-scratch/cli-git-input-stale"), b"x").expect("stale");
    age(common.join("cli-git-scratch/cli-git-input-stale").as_path(), SCRATCH_STALE_AFTER * 2);
    prepare_repository_scratch(common.as_path()).expect("preparation");
    assert!(!common.join("cli-git-scratch/cli-git-input-stale").exists());
    remove(root.as_path());
}
