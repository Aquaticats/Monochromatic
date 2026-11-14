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
    std::fs::write(notes.join(format!("{}{NOTE_ENDING}", an_instance())), hex_of("/repo/.git")).expect("a note");
    // Somebody else's file under one of the names this program also uses.
    std::fs::write(other.as_path(), "[Unit]\nDescription=another program's unit\n").expect("another program's unit");

    let removed: Vec<PathBuf> = remove_earlier_unit_files(root.as_path());
    assert!(!service.exists() && !socket.exists() && !legacy.exists(), "the earlier release's files are gone");
    assert!(!notes.exists(), "and its notes folder with them");
    assert!(other.exists(), "a file this program did not write is left: {:?}", removed);
    // The two templates, the single service's socket, the note and the folder that held it and nothing else.
    assert_eq!(removed.len(), 5, "five paths went: {removed:?}");
    assert!(removed.iter().any(|path| { return path.ends_with(NOTES_FOLDER); }), "the folder goes too once nothing of a person's is in it: {removed:?}");
    // A second sweep finds nothing to do.
    assert!(remove_earlier_unit_files(root.as_path()).is_empty(), "the sweep is idempotent");
}

/// One instance name: the 32 hexadecimal digits of a worktree's hash.
fn an_instance() -> String {
    return String::from("0123456789abcdef0123456789abcdef");
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

/// What:
///  The notes folder loses this program's notes and nothing a person kept there,
///  and the folder stays while anything of theirs is in it.
/// Why:
///  The review of `76a08c9fac` reproduced a person's file going with the folder when the folder was removed whole:
///  `remove_dir_all` ran on any folder of that name, though the file's own doc promised a folder holding no note of this
///  program was left. The requirement is "never discard user data".
#[test]
fn the_notes_folder_keeps_what_this_program_did_not_write() {
    let root: PathBuf = runtime("notes-of-a-person");
    let notes: PathBuf = root.join("systemd/user").join(NOTES_FOLDER);
    std::fs::create_dir_all(notes.as_path()).expect("the notes folder");
    // This program's: one note of this release's shape, one of the earlier plain shape, and the program link.
    std::fs::write(notes.join(format!("{}{NOTE_ENDING}", an_instance())), hex_of("/repo/.git")).expect("a note in hexadecimal");
    let another_instance: String = an_instance().replace('a', "b");
    std::fs::write(notes.join(format!("{another_instance}{NOTE_ENDING}")), "/repo/.git\n").expect("a note as the earlier release wrote it");
    std::os::unix::fs::symlink("/usr/bin/monochromatic-git-policy-cli", notes.join(PROGRAM_LINK)).expect("the program link");
    // A person's: a file of their own, and a file whose name looks like a note and whose text is not one.
    let precious: PathBuf = notes.join("precious.txt");
    std::fs::write(precious.as_path(), "a person's own file, not a note of this program\n").expect("a person's file");
    let under_a_note_name: PathBuf = notes.join(format!("{}{NOTE_ENDING}", an_instance().replace('a', "c")));
    std::fs::write(under_a_note_name.as_path(), "a person's notes, under a name like this program's\n").expect("a person's file under a note's name");

    let removed: Vec<PathBuf> = remove_earlier_unit_files(root.as_path());
    assert!(removed.iter().any(|path| { return path.extension() == Some(std::ffi::OsStr::new("git-dir")) && path.file_name() != under_a_note_name.file_name(); }), "this program's notes are the ones that went: {removed:?}");
    assert!(!notes.join(format!("{another_instance}{NOTE_ENDING}")).exists(), "and the earlier release's, whose text is a path");
    assert!(!notes.join(PROGRAM_LINK).exists(), "and the link its units started the service through");
    assert!(precious.exists(), "a file a person kept there is left: {removed:?}");
    assert!(under_a_note_name.exists(), "and so is one whose name looks like a note and whose text is not one: {removed:?}");
    assert!(notes.exists(), "the folder itself stays while anything of a person's is in it");
    assert_eq!(removed.len(), 3, "three entries went, and the folder did not: {removed:?}");
    // Once nothing of a person's is left, the folder goes with the last note of this program.
    std::fs::remove_file(precious.as_path()).expect("the person's file");
    std::fs::remove_file(under_a_note_name.as_path()).expect("the person's other file");
    std::fs::write(notes.join(format!("{}{NOTE_ENDING}", an_instance().replace('a', "d"))), hex_of("/repo/.git")).expect("a note");
    let removed_now: Vec<PathBuf> = remove_earlier_unit_files(root.as_path());
    assert!(!notes.exists(), "an empty notes folder goes: {removed_now:?}");
    assert_eq!(removed_now.len(), 2, "the note and the folder: {removed_now:?}");
}

/// What:
///  A stop ask whose client cannot be reaped still answers within the ask's bound.
/// Why:
///  The review of `76a08c9fac` (finding 2) found `kill()` then `wait()` with no bound after the ask's `PROBE_WAIT`, on a
///  commit's path: a client inside an uninterruptible call is not ended by a signal until that call returns, and the
///  command waited for as long as that took. The call here is given an ending-and-reaping that sleeps far past the bound,
///  which is that client's state, and it must still answer in time.
#[test]
fn a_stop_ask_whose_client_cannot_be_reaped_answers_within_its_bound() {
    let folder: PathBuf = crate::test_support::fixture("earlier-units-unreapable");
    crate::test_support::track_for_this_thread(folder.as_path());
    let _stand_in: PathBuf = stand_in_systemctl(folder.as_path());
    // A client that outlasts the ask's bound, so the bound is what ends the ask.
    std::fs::write(folder.join("systemctl").as_path(), "#!/bin/sh\n/usr/bin/sleep 30\n").expect("a client that never answers");
    std::fs::set_permissions(folder.join("systemctl").as_path(), std::fs::Permissions::from_mode(0o700)).expect("the stand-in runs");
    let held: Option<std::ffi::OsString> = std::env::var_os("PATH");
    // SAFETY: this control is the only one in this process that changes PATH, and it restores it before it ends.
    unsafe { std::env::set_var("PATH", folder.as_os_str()) };
    let ending_began: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let began_at: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&ending_began);
    let began: std::time::Instant = std::time::Instant::now();
    let asked: bool = stop_with_the_managers_client_seamed(
        "3f9a",
        Box::new(move |mut child| {
            began_at.store(true, std::sync::atomic::Ordering::SeqCst);
            let _ended = child.kill();
            // The client inside an uninterruptible call: the wait for it takes far longer than the ask's bound.
            std::thread::sleep(std::time::Duration::from_secs(30));
            let _reaped = child.wait();
        }),
    );
    let took: std::time::Duration = began.elapsed();
    match held {
        Some(path) => unsafe { std::env::set_var("PATH", path) },
        None => unsafe { std::env::remove_var("PATH") },
    }
    assert!(!asked, "a client that has not answered by the bound is not an ask that was answered");
    assert!(took < PROBE_WAIT + std::time::Duration::from_secs(2), "the ask ends at its bound, not when the client can be reaped: {took:?}");
    // The ending and the reaping began, on a thread of their own rather than on this path.
    let mut waited: std::time::Duration = std::time::Duration::ZERO;
    while !ending_began.load(std::sync::atomic::Ordering::SeqCst) && waited < std::time::Duration::from_secs(2) {
        std::thread::sleep(std::time::Duration::from_millis(10));
        waited += std::time::Duration::from_millis(10);
    }
    assert!(ending_began.load(std::sync::atomic::Ordering::SeqCst), "the child is ended and reaped, just not on the command's path");
    crate::test_support::remove(folder.as_path());
}
