//! Controls of what an earlier release left:
//! its unit files are removed and nothing else is,
//! its socket units are stopped through the manager's own program and only when a manager of this session is there,
//! and the whole sweep is once per command.

use super::*;
use std::os::unix::fs::PermissionsExt;

/// A runtime directory of this test's own.
fn runtime(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture("earlier-units").join(name);
    crate::test_support::track_for_this_thread(root.as_path());
    let directory: PathBuf = root.join("systemd/user");
    std::fs::create_dir_all(directory.as_path()).expect("the unit directory");
    return root;
}

/// The text of one of the earlier release's templates, as it wrote it.
fn earlier_unit() -> String {
    return String::from("# Written by the git wrapper for its commit service (monochromatic-git-policy-cli-unit); the wrapper rewrites this file when it changes.\n[Unit]\nDescription=Commit service of one worktree for the git wrapper\n[Service]\nExecStart=@SERVICE_BINARY@ service run\n");
}

/// What:
///  Every file of the earlier release is removed, and a file of one of those names that it did not write is left.
/// Why:
///  A unit directory is shared: another program's file of the same name is never this program's to remove.
#[test]
fn the_earlier_releases_files_go_and_nothing_else_does() {
    let root: PathBuf = runtime("removes");
    let directory: PathBuf = root.join("systemd/user");
    let service: PathBuf = directory.join("monochromatic-git-policy-cli@.service");
    let socket: PathBuf = directory.join("monochromatic-git-policy-cli@.socket");
    let notes: PathBuf = directory.join("monochromatic-git-policy-cli-worktree-notes");
    let legacy: PathBuf = directory.join("cli-git.socket");
    let other: PathBuf = directory.join("cli-git.service");
    std::fs::write(service.as_path(), earlier_unit()).expect("the service template");
    std::fs::write(socket.as_path(), earlier_unit()).expect("the socket template");
    std::fs::write(legacy.as_path(), earlier_unit()).expect("the earlier single socket");
    std::fs::create_dir_all(notes.as_path()).expect("the notes folder");
    std::fs::write(notes.join("3f9a.git-dir"), hex_of("/repo/.git")).expect("a note");
    // Somebody else's file under one of the names this program also uses.
    std::fs::write(other.as_path(), "[Unit]\nDescription=another program's unit\n").expect("another program's unit");

    let removed: Vec<PathBuf> = remove_earlier_unit_files(root.as_path());
    assert!(!service.exists() && !socket.exists() && !legacy.exists(), "the earlier release's files are gone");
    assert!(!notes.exists(), "and its notes folder with them");
    assert!(other.exists(), "a file this program did not write is left: {:?}", removed);
    assert_eq!(removed.len(), 4, "four paths went: {removed:?}");
    // A second sweep finds nothing to do.
    assert!(remove_earlier_unit_files(root.as_path()).is_empty(), "the sweep is idempotent");
}

/// The note an earlier release wrote holds the Git directory as hex, as that release wrote it.
fn hex_of(text: &str) -> String {
    let mut hex: String = format!("{}{}", crate::path_bytes_in_records::GITDIR_HEX_PREFIX, "");
    for byte in text.as_bytes() {
        hex.push_str(format!("{byte:02x}").as_str());
    }
    return hex;
}

/// A stand-in for the manager's own client, which records the words it was given.
fn stand_in_systemctl(folder: &Path) -> PathBuf {
    let program: PathBuf = folder.join("systemctl");
    std::fs::write(program.as_path(), format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{calls}'\n", calls = folder.join("calls.txt").display())).expect("the stand-in is written");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the stand-in runs");
    return program;
}

/// What:
///  The socket units of this worktree and of the earlier single service are stopped,
///  in the words the manager's own client takes.
/// Why:
///  A loaded socket unit keeps this worktree's address bound;
/// the stop is the one call that frees it without a reload.
#[test]
fn the_socket_units_are_stopped_in_the_managers_own_words() {
    let folder: PathBuf = crate::test_support::fixture("earlier-units-stop");
    let _stand_in: PathBuf = stand_in_systemctl(folder.as_path());
    let held: Option<std::ffi::OsString> = std::env::var_os("PATH");
    // SAFETY: this control is the only one in this process that changes PATH, and it restores it before it ends.
    unsafe { std::env::set_var("PATH", folder.as_os_str()) };
    let asked: bool = stop_with_the_managers_client("3f9a");
    let words: String = std::fs::read_to_string(folder.join("calls.txt")).unwrap_or_default();
    match held {
        Some(path) => unsafe { std::env::set_var("PATH", path) },
        None => unsafe { std::env::remove_var("PATH") },
    }
    assert!(asked, "the manager's client answered");
    assert!(words.contains("--user stop"), "the words are the client's own: {words}");
    assert!(words.contains("monochromatic-git-policy-cli@3f9a.socket"), "this worktree's socket: {words}");
    assert!(words.contains("cli-git.socket"), "and the earlier single service's: {words}");
    crate::test_support::remove(folder.as_path());
}

/// What:
///  A runtime directory with nothing of an earlier release in it costs nothing and asks nothing.
/// Why:
///  That is every login after the first: the runtime directory is new, and the command touches nothing.
#[test]
fn a_runtime_directory_without_an_earlier_release_does_nothing() {
    let root: PathBuf = runtime("empty");
    let removed: Vec<PathBuf> = retire_earlier_units(root.as_path(), None, None, "3f9a");
    assert!(removed.is_empty(), "{removed:?}");
}

/// What:
///  The manager is asked only when the runtime directory is the one the environment named and its bus is that session's.
/// Why:
///  A command given a runtime directory of its own must leave the account's manager alone (issue 6293),
///  and a folder this program fell back to is no session.
#[test]
fn the_stop_is_asked_only_for_this_sessions_own_bus() {
    let folder: PathBuf = crate::test_support::fixture("earlier-units-gate");
    let _stand_in: PathBuf = stand_in_systemctl(folder.as_path());
    let runtime_dir: PathBuf = folder.join("runtime");
    std::fs::create_dir_all(runtime_dir.as_path()).expect("the runtime directory");
    let held: Option<std::ffi::OsString> = std::env::var_os("PATH");
    // SAFETY: this control is the only one in this process that changes PATH, and it restores it before it ends.
    unsafe { std::env::set_var("PATH", folder.as_os_str()) };
    // When: the named runtime directory is this one, and the bus's socket is inside it.
    let _removed: Vec<PathBuf> = retire_earlier_units(runtime_dir.as_path(), Some(runtime_dir.as_path()), Some(format!("unix:path={}", runtime_dir.join("bus").display()).as_str()), "3f9a");
    // When not: a bus of another directory, and a folder the program fell back to.
    let _unasked: Vec<PathBuf> = retire_earlier_units(runtime_dir.as_path(), Some(runtime_dir.as_path()), Some("unix:path=/run/user/9999/bus"), "3f9a");
    let _fell_back: Vec<PathBuf> = retire_earlier_units(runtime_dir.as_path(), None, Some(format!("unix:path={}", runtime_dir.join("bus").display()).as_str()), "3f9a");
    let words: String = std::fs::read_to_string(folder.join("calls.txt")).unwrap_or_default();
    match held {
        Some(path) => unsafe { std::env::set_var("PATH", path) },
        None => unsafe { std::env::remove_var("PATH") },
    }
    assert_eq!(words.lines().count(), 1, "one ask, for the session whose bus it is: {words}");
    crate::test_support::remove(folder.as_path());
}
