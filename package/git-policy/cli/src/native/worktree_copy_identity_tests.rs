//! The registration token and the identity of the directory it is published into: whole publication, one winner, and no
//!  token in a directory that replaced the one that was observed.

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_copy_registry::registration_token;
use std::path::PathBuf;

/// An administrative root holding one directory named `topic`.
fn administration(root: &Path) -> PathBuf {
    let admin: PathBuf = root.join("worktrees");
    std::fs::create_dir_all(admin.join("topic")).expect("administrative directory");
    return admin;
}

/// The names in a directory.
fn names_in(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .expect("listing")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    return names;
}

/// The token is published whole under its final name, nothing else is left beside it, and a second call reads it back.
#[test]
fn a_token_is_published_whole_and_kept() {
    let root: PathBuf = fixture("identity-whole");
    let admin: PathBuf = administration(root.as_path());
    let token: String = ensure_registration_token(admin.as_path(), "topic", None).expect("published").expect("registered");
    assert!(!token.is_empty());
    assert_eq!(std::fs::read_to_string(admin.join("topic").join(REGISTRATION_NAME)).expect("file"), token);
    assert_eq!(names_in(admin.join("topic").as_path()), vec![REGISTRATION_NAME.to_string()], "no temporary stays");
    assert_eq!(ensure_registration_token(admin.as_path(), "topic", None).expect("again"), Some(token));
    remove(root.as_path());
}

/// A publisher killed before it linked its temporary leaves no token behind: the final name is absent, so the registration
/// reads as having none, and the next publisher publishes a whole one.
#[test]
fn a_publisher_killed_before_it_linked_leaves_no_token() {
    let root: PathBuf = fixture("identity-killed-publisher");
    let admin: PathBuf = administration(root.as_path());
    let leftover: PathBuf = admin.join("topic").join(format!("{REGISTRATION_NAME}.dead.tmp"));
    std::fs::write(leftover.as_path(), b"half of a tok").expect("leftover temporary");
    assert_eq!(registration_token(admin.as_path(), "topic").expect("read"), None);
    let token: String = ensure_registration_token(admin.as_path(), "topic", None).expect("published").expect("registered");
    assert_ne!(token, "half of a tok");
    assert_eq!(registration_token(admin.as_path(), "topic").expect("read"), Some(token));
    remove(root.as_path());
}

/// An empty token file, which an older release left when it was killed between creating the file and writing it, is
/// replaced instead of reading as no token for good.
#[test]
fn an_empty_token_from_an_older_release_is_replaced() {
    let root: PathBuf = fixture("identity-empty-token");
    let admin: PathBuf = administration(root.as_path());
    std::fs::write(admin.join("topic").join(REGISTRATION_NAME), b"").expect("empty token");
    let token: String = ensure_registration_token(admin.as_path(), "topic", None).expect("published").expect("registered");
    assert_eq!(registration_token(admin.as_path(), "topic").expect("read"), Some(token));
    remove(root.as_path());
}

/// Publishers that race agree on one token, whichever wins.
#[test]
fn racing_publishers_agree_on_one_token() {
    let root: PathBuf = fixture("identity-racing");
    let admin: PathBuf = administration(root.as_path());
    let tokens: Vec<Option<String>> = std::thread::scope(|threads| {
        let started: Vec<_> = (0..6)
            .map(|_number| return threads.spawn(|| return ensure_registration_token(admin.as_path(), "topic", None).expect("published")))
            .collect();
        return started.into_iter().map(|thread| return thread.join().expect("publisher")).collect();
    });
    assert!(tokens[0].is_some());
    assert!(tokens.iter().all(|token| return *token == tokens[0]), "{tokens:?}");
    remove(root.as_path());
}

/// A directory that replaced the one that was observed is another registration: it has another identity, and the token is
/// refused for it instead of being written into it. The renamed original keeps its inode, so the control does not depend
/// on whether the file system reuses inode numbers.
#[test]
fn a_token_is_never_written_into_a_replacement_directory() {
    let root: PathBuf = fixture("identity-replacement");
    let admin: PathBuf = administration(root.as_path());
    let observed: String = admin_identity(admin.as_path(), "topic").expect("identity").expect("there");
    std::fs::rename(admin.join("topic"), admin.join("topic-removed")).expect("the original goes away");
    std::fs::create_dir(admin.join("topic")).expect("a replacement under the same name");
    let replacement: String = admin_identity(admin.as_path(), "topic").expect("identity").expect("there");
    assert_ne!(observed, replacement);
    assert_eq!(ensure_registration_token(admin.as_path(), "topic", Some(observed.as_str())).expect("refused"), None);
    assert_eq!(names_in(admin.join("topic").as_path()), Vec::<String>::new(), "the replacement was tagged");
    assert!(ensure_registration_token(admin.as_path(), "topic", Some(replacement.as_str())).expect("own").is_some());
    remove(root.as_path());
}

/// A directory that is gone, a plain file, or a link in its place has no identity and takes no token.
#[test]
fn a_missing_or_linked_directory_has_no_identity() {
    let root: PathBuf = fixture("identity-missing");
    let admin: PathBuf = administration(root.as_path());
    assert_eq!(admin_identity(admin.as_path(), "absent").expect("absent"), None);
    assert_eq!(ensure_registration_token(admin.as_path(), "absent", None).expect("absent"), None);
    std::fs::write(admin.join("plain"), b"a file").expect("file");
    assert_eq!(admin_identity(admin.as_path(), "plain").expect("file"), None);
    assert_eq!(ensure_registration_token(admin.as_path(), "plain", None).expect("file"), None);
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("elsewhere");
    std::os::unix::fs::symlink(elsewhere.as_path(), admin.join("linked")).expect("link");
    assert_eq!(admin_identity(admin.as_path(), "linked").expect("link"), None);
    assert_eq!(ensure_registration_token(admin.as_path(), "linked", None).expect("link"), None);
    assert_eq!(names_in(elsewhere.as_path()), Vec::<String>::new(), "a link was followed");
    remove(root.as_path());
}
