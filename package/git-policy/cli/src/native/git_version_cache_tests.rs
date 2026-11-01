//! Controls for the Git version cache:
//!  the identity of an executable,
//!  a hit that is exact,
//!  and a replaced,
//!  rewritten,
//!  stale or foreign entry that is never trusted.

use super::*;
use crate::git_version::{GitRelease, ask_git_version, ask_git_version_cached};
use crate::test_support::{REAL_GIT, executable, fixture, git_context, remove};
use crate::transaction_git::GitContext;

/// A context whose Git is the executable at `path`.
fn context_for(path: &Path) -> GitContext {
    return GitContext { real_git: path.to_path_buf(), ..git_context() };
}

/// Copy the host's Git to `root/name` so the copy is a real ELF executable named as asked.
fn copy_git(root: &Path, name: &str) -> PathBuf {
    let target: PathBuf = root.join(name);
    std::fs::copy(REAL_GIT, &target).expect("fixture");
    return target;
}

/// The cache directory of a fixture,
///  two levels below the root so its parent does not exist yet.
fn cache_in(root: &Path) -> PathBuf {
    return root.join("xdg").join("cli-git");
}

/// The text of the host Git's answer, as the version check parses it.
fn host_release() -> (u64, u64, u64) {
    return ask_git_version(&git_context()).expect("the host Git is supported");
}

/// Record `text` for the executable at `path` so a later hit is visible by its text.
fn plant(cache: &Path, path: &Path, text: &str) {
    let identity: BinaryIdentity = binary_identity(path).expect("identity");
    store_version_text(cache, &identity, text.as_bytes());
}

/// A compiled executable named `git` has an identity, and the identity names its canonical path.
#[test]
#[cfg(unix)]
fn a_compiled_git_has_an_identity() {
    let root: PathBuf = fixture("version-cache-identity");
    let git_copy: PathBuf = copy_git(root.as_path(), "git");
    let identity: BinaryIdentity = binary_identity(git_copy.as_path()).expect("identity");
    assert_eq!(identity.path, git_copy.to_str().expect("fixture"));
    assert!(identity.text.starts_with(identity.path.as_str()));
    assert_eq!(binary_identity(git_copy.as_path()), Some(identity.clone()));
    remove(root.as_path());
}

/// A script,
///  an executable of another name,
///  and a link whose target is not named `git` have no identity:
///  their answer is not a property of their own bytes.
#[test]
#[cfg(unix)]
fn scripts_and_shims_have_no_identity() {
    let root: PathBuf = fixture("version-cache-shims");
    let script: PathBuf = root.join("git");
    executable(script.as_path(), b"#!/bin/sh\necho 'git version 2.56.0'\n");
    assert_eq!(binary_identity(script.as_path()), None);

    let other: PathBuf = copy_git(root.as_path(), "dispatcher");
    assert_eq!(binary_identity(other.as_path()), None);
    std::fs::create_dir(root.join("shims")).expect("fixture");
    std::os::unix::fs::symlink(&other, root.join("shims").join("git")).expect("fixture");
    assert_eq!(binary_identity(root.join("shims").join("git").as_path()), None);

    std::fs::create_dir(root.join("real")).expect("fixture");
    let real: PathBuf = copy_git(root.join("real").as_path(), "git");
    std::os::unix::fs::symlink(&real, root.join("shims").join("alias")).expect("fixture");
    std::fs::create_dir(root.join("links")).expect("fixture");
    std::os::unix::fs::symlink(&real, root.join("links").join("git")).expect("fixture");
    let through_link: BinaryIdentity = binary_identity(root.join("links").join("git").as_path()).expect("identity");
    assert_eq!(through_link.path, real.to_str().expect("fixture"));
    assert_eq!(binary_identity(root.join("missing").as_path()), None);
    remove(root.as_path());
}

/// A miss runs Git and records its text;
///  the next ask is answered by the cache alone, shown by a planted text that no Git prints.
#[test]
#[cfg(unix)]
fn a_miss_is_recorded_and_the_next_ask_hits() {
    let root: PathBuf = fixture("version-cache-hit");
    let git_copy: PathBuf = copy_git(root.as_path(), "git");
    let cache: PathBuf = cache_in(root.as_path());
    let context: GitContext = context_for(git_copy.as_path());
    assert_eq!(ask_git_version_cached(&context, Some(cache.as_path())), Ok(host_release()));
    assert!(std::fs::read_dir(&cache).expect("recorded").count() == 1);
    plant(cache.as_path(), git_copy.as_path(), "git version 9.8.7\n");
    assert_eq!(ask_git_version_cached(&context, Some(cache.as_path())), Ok((9, 8, 7)));
    remove(root.as_path());
}

/// The cache directory is private to the user and its file is not readable by others.
#[test]
#[cfg(unix)]
fn the_cache_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("version-cache-private");
    let git_copy: PathBuf = copy_git(root.as_path(), "git");
    let cache: PathBuf = cache_in(root.as_path());
    assert_eq!(ask_git_version_cached(&context_for(git_copy.as_path()), Some(cache.as_path())), Ok(host_release()));
    let directory_mode: u32 = std::fs::metadata(&cache).expect("recorded").permissions().mode() & 0o777;
    assert_eq!(directory_mode, 0o700);
    for entry in std::fs::read_dir(&cache).expect("recorded") {
        let file_mode: u32 = entry.expect("entry").metadata().expect("metadata").permissions().mode() & 0o777;
        assert_eq!(file_mode, 0o600);
    }
    remove(root.as_path());
}

/// An executable replaced by another file at the same path is noticed:
///  the planted answer of the old file is not returned.
#[test]
#[cfg(unix)]
fn a_replaced_executable_is_noticed() {
    let root: PathBuf = fixture("version-cache-replaced");
    let git_copy: PathBuf = copy_git(root.as_path(), "git");
    let cache: PathBuf = cache_in(root.as_path());
    let context: GitContext = context_for(git_copy.as_path());
    plant(cache.as_path(), git_copy.as_path(), "git version 9.8.7\n");
    assert_eq!(ask_git_version_cached(&context, Some(cache.as_path())), Ok((9, 8, 7)));
    // A new file of the same size and content moves into place: only the inode and times differ.
    let replacement: PathBuf = copy_git(root.as_path(), "git.new");
    std::fs::rename(&replacement, &git_copy).expect("fixture");
    assert_eq!(ask_git_version_cached(&context, Some(cache.as_path())), Ok(host_release()));
    remove(root.as_path());
}

/// A file rewritten in place with other content is noticed by its size.
#[test]
#[cfg(unix)]
fn an_executable_rewritten_with_other_size_is_noticed() {
    let root: PathBuf = fixture("version-cache-size");
    let git_copy: PathBuf = copy_git(root.as_path(), "git");
    let cache: PathBuf = cache_in(root.as_path());
    let context: GitContext = context_for(git_copy.as_path());
    plant(cache.as_path(), git_copy.as_path(), "git version 9.8.7\n");
    // The same file (same inode), now another program.
    std::fs::copy("/usr/bin/true", &git_copy).expect("fixture");
    let answer: Result<GitRelease, String> = ask_git_version_cached(&context, Some(cache.as_path()));
    let message: String = answer.expect_err("the replaced program prints no version");
    assert!(message.contains("an unreadable version"), "{message}");
    remove(root.as_path());
}

/// A file rewritten in place to the same size with its modification time put back is noticed by its change time.
#[test]
#[cfg(unix)]
fn a_same_size_rewrite_with_the_old_mtime_is_noticed() {
    let root: PathBuf = fixture("version-cache-ctime");
    let git_copy: PathBuf = copy_git(root.as_path(), "git");
    let cache: PathBuf = cache_in(root.as_path());
    let context: GitContext = context_for(git_copy.as_path());
    plant(cache.as_path(), git_copy.as_path(), "git version 9.8.7\n");
    let before: std::fs::Metadata = std::fs::metadata(&git_copy).expect("fixture");
    let inode: u64 = std::os::unix::fs::MetadataExt::ino(&before);
    let bytes: Vec<u8> = std::fs::read(&git_copy).expect("fixture");
    std::fs::write(&git_copy, bytes.as_slice()).expect("fixture");
    {
        // The handle is closed at the end of this block: Linux refuses to run a file open for writing.
        let file: std::fs::File = std::fs::OpenOptions::new().write(true).open(&git_copy).expect("fixture");
        file.set_modified(before.modified().expect("fixture")).expect("fixture");
    }
    let after: std::fs::Metadata = std::fs::metadata(&git_copy).expect("fixture");
    assert_eq!(std::os::unix::fs::MetadataExt::ino(&after), inode, "the same file");
    assert_eq!(after.len(), before.len());
    assert_eq!(after.modified().expect("fixture"), before.modified().expect("fixture"));
    assert_eq!(ask_git_version_cached(&context, Some(cache.as_path())), Ok(host_release()));
    remove(root.as_path());
}

/// An entry that is garbage,
///  in another layout,
///  of another identity,
///  too permissive,
///  a link or a directory is ignored,
///  and Git is asked.
#[test]
#[cfg(unix)]
fn stale_or_foreign_entries_are_not_trusted() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("version-cache-foreign");
    let git_copy: PathBuf = copy_git(root.as_path(), "git");
    let cache: PathBuf = cache_in(root.as_path());
    let context: GitContext = context_for(git_copy.as_path());
    let identity: BinaryIdentity = binary_identity(git_copy.as_path()).expect("identity");
    plant(cache.as_path(), git_copy.as_path(), "git version 9.8.7\n");
    let entry: PathBuf = std::fs::read_dir(&cache).expect("planted").next().expect("one entry").expect("entry").path();
    let good: Vec<u8> = std::fs::read(&entry).expect("planted");
    let mut cases: Vec<(&str, Vec<u8>)> = Vec::new();
    cases.push(("garbage", b"\x00\xff garbage".to_vec()));
    cases.push(("empty", Vec::new()));
    cases.push(("other layout", String::from_utf8_lossy(&good).replace("cache 1", "cache 2").into_bytes()));
    cases.push(("other identity", String::from_utf8_lossy(&good).replace(identity.text.as_str(), "other\t1\t2.3\t4.5\t6\t7").into_bytes()));
    cases.push(("unparseable text", String::from_utf8_lossy(&good).replace("git version 9.8.7", "hello").into_bytes()));
    cases.push(("identity only", format!("{FILE_TAG}{}", identity.text).into_bytes()));
    for (label, bytes) in cases {
        std::fs::remove_file(&entry).expect("fixture");
        std::fs::write(&entry, bytes).expect("fixture");
        std::fs::set_permissions(&entry, std::fs::Permissions::from_mode(0o600)).expect("fixture");
        assert_eq!(ask_git_version_cached(&context, Some(cache.as_path())), Ok(host_release()), "{label}");
    }
    // A readable-by-others entry, a link and a directory in the entry's place.
    std::fs::remove_file(&entry).expect("fixture");
    std::fs::write(&entry, &good).expect("fixture");
    std::fs::set_permissions(&entry, std::fs::Permissions::from_mode(0o644)).expect("fixture");
    assert_eq!(ask_git_version_cached(&context, Some(cache.as_path())), Ok(host_release()), "mode 0644");
    std::fs::remove_file(&entry).expect("fixture");
    std::fs::write(root.join("target"), &good).expect("fixture");
    std::fs::set_permissions(root.join("target"), std::fs::Permissions::from_mode(0o600)).expect("fixture");
    std::os::unix::fs::symlink(root.join("target"), &entry).expect("fixture");
    assert_eq!(ask_git_version_cached(&context, Some(cache.as_path())), Ok(host_release()), "link");
    std::fs::remove_file(&entry).expect("fixture");
    std::fs::create_dir(&entry).expect("fixture");
    assert_eq!(ask_git_version_cached(&context, Some(cache.as_path())), Ok(host_release()), "directory");
    remove(root.as_path());
}

/// A cached text that parses as an unsupported release is refused with the same message as a live answer.
#[test]
#[cfg(unix)]
fn a_cached_old_release_is_refused_like_a_live_one() {
    let root: PathBuf = fixture("version-cache-old");
    let git_copy: PathBuf = copy_git(root.as_path(), "git");
    let cache: PathBuf = cache_in(root.as_path());
    let context: GitContext = context_for(git_copy.as_path());
    plant(cache.as_path(), git_copy.as_path(), "git version 2.54.0\n");
    let refusal: String = ask_git_version_cached(&context, Some(cache.as_path())).expect_err("2.54.0 is refused");
    assert!(refusal.contains("version 2.54.0"), "{refusal}");
    assert!(refusal.contains("2.55.0 or newer"), "{refusal}");
    remove(root.as_path());
}

/// Without a cache directory,
///  or for a script,
///  Git is asked every time and nothing is written.
#[test]
#[cfg(unix)]
fn without_a_cache_or_an_identity_nothing_is_recorded() {
    let root: PathBuf = fixture("version-cache-none");
    let git_copy: PathBuf = copy_git(root.as_path(), "git");
    assert_eq!(ask_git_version_cached(&context_for(git_copy.as_path()), None), Ok(host_release()));
    let script_root: PathBuf = root.join("script");
    std::fs::create_dir(&script_root).expect("fixture");
    let script: PathBuf = script_root.join("git");
    executable(script.as_path(), b"#!/bin/sh\necho 'git version 2.55.0'\n");
    let cache: PathBuf = cache_in(root.as_path());
    assert_eq!(ask_git_version_cached(&context_for(script.as_path()), Some(cache.as_path())), Ok((2, 55, 0)));
    assert!(!cache.exists());
    remove(root.as_path());
}

/// A refusal is not recorded:
///  an executable that prints nothing leaves no entry.
#[test]
#[cfg(unix)]
fn a_failed_answer_is_not_recorded() {
    let root: PathBuf = fixture("version-cache-failed");
    let program: PathBuf = root.join("git");
    std::fs::copy("/usr/bin/true", &program).expect("fixture");
    let cache: PathBuf = cache_in(root.as_path());
    let refusal: String = ask_git_version_cached(&context_for(program.as_path()), Some(cache.as_path())).expect_err("no version");
    assert!(refusal.contains("an unreadable version"), "{refusal}");
    let recorded: usize = std::fs::read_dir(&cache).map(|entries| { return entries.count(); }).unwrap_or(0);
    assert_eq!(recorded, 0);
    remove(root.as_path());
}

/// The cached answer equals the live answer for the host Git.
#[test]
#[cfg(unix)]
fn the_cached_answer_equals_the_live_answer() {
    let root: PathBuf = fixture("version-cache-equal");
    let cache: PathBuf = cache_in(root.as_path());
    let live: Result<GitRelease, String> = ask_git_version(&git_context());
    let first: Result<GitRelease, String> = ask_git_version_cached(&git_context(), Some(cache.as_path()));
    let second: Result<GitRelease, String> = ask_git_version_cached(&git_context(), Some(cache.as_path()));
    assert_eq!(first, live);
    assert_eq!(second, live);
    remove(root.as_path());
}

/// The location follows the injected environment:
///  an absolute `XDG_CACHE_HOME`,
///  else `.cache` under `HOME`,
///  else none.
#[test]
fn the_location_follows_the_environment() {
    let pair = |name: &str, value: &str| { return (OsString::from(name), OsString::from(value)); };
    assert_eq!(
        cache_directory(&[pair("XDG_CACHE_HOME", "/xdg"), pair("HOME", "/home/a")]),
        Some(PathBuf::from("/xdg/cli-git"))
    );
    assert_eq!(
        cache_directory(&[pair("XDG_CACHE_HOME", "relative"), pair("HOME", "/home/a")]),
        Some(PathBuf::from("/home/a/.cache/cli-git"))
    );
    assert_eq!(cache_directory(&[pair("HOME", "/home/a")]), Some(PathBuf::from("/home/a/.cache/cli-git")));
    assert_eq!(cache_directory(&[pair("HOME", "")]), None);
    assert_eq!(cache_directory(&[]), None);
}
