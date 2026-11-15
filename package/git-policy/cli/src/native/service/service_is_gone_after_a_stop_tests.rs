//! Controls of the proof that a stopped instance serves nothing any more.

use super::*;

fn folder(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("gone-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}

/// An instance with no note is gone,
/// and so is one whose number carries another birth.
#[test]
fn a_note_that_names_no_living_process_is_gone() {
    let root = folder("stale");
    assert!(service_is_gone(root.as_path(), "3f9a"), "no note at all is no service");
    let note = crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a");
    std::fs::write(note.as_path(), "1 linux:000000\n").expect("a stale note");
    assert!(service_is_gone(root.as_path(), "3f9a"), "a note of another birth names no service");
}

/// An instance whose note names a living process is not gone.
#[test]
fn a_note_that_names_a_living_process_is_not_gone() {
    let root = folder("living");
    let note = crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a");
    crate::service_server_pid_note_of_an_instance::claim(note.as_path(), i32::try_from(std::process::id()).expect("a process number")).expect("the note");
    assert!(!service_is_gone(root.as_path(), "3f9a"), "the process the note names is this one");
}

/// What:
///  A service whose worktree was removed keeps its note while it can still write,
///  so a sweep never reads it as stopped.
/// Why:
///  The sixth review of issue 6293 (finding 2):
///  the path a service takes when it finds its own worktree gone unlinked the note as well as the socket,
///  and then `service_is_gone` read the missing note as proof that the writer was gone
///  while the serving function had not returned and its after-commit task could still publish a watch, a plan or a record.
///  The note is removed only as the process ends,
///  so the proof is a note that names no living process, and never the absence of one.
#[test]
fn a_service_whose_worktree_went_keeps_its_note_and_is_not_read_as_stopped() {
    let root = folder("worktree-gone");
    let runtime: std::path::PathBuf = root.join("run");
    let sockets: std::path::PathBuf = runtime.join(crate::service_instance::SOCKET_DIRECTORY);
    std::fs::create_dir_all(sockets.as_path()).expect("socket folder");
    let common: std::path::PathBuf = root.join("common/.git");
    let gone: std::path::PathBuf = common.join("worktrees/gone");
    std::fs::create_dir_all(gone.parent().expect("worktrees parent")).expect("common worktrees parent");
    let instance: String = crate::service_instance::instance_of(gone.as_path());
    crate::service_units_of_removed_worktrees::record_worktree(runtime.as_path(), instance.as_str(), gone.as_path()).expect("the worktree note");
    std::fs::write(crate::service_instance::socket_path(runtime.as_path(), instance.as_str()), b"socket").expect("socket placeholder");
    let note: std::path::PathBuf = crate::service_server_pid_note_of_an_instance::path(sockets.as_path(), instance.as_str());
    crate::service_server_pid_note_of_an_instance::claim(note.as_path(), i32::try_from(std::process::id()).expect("a process number")).expect("this process serves the instance");
    let journal: std::path::PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("journal folder");
    // The path a service takes when its own worktree is gone runs here, with this process still alive and still able to write.
    // SAFETY: no other control of this module sets this variable, and it is restored before this one ends.
    let was: Option<std::ffi::OsString> = std::env::var_os("XDG_RUNTIME_DIR");
    unsafe { std::env::set_var("XDG_RUNTIME_DIR", runtime.as_os_str()) };
    assert!(
        crate::service_units_of_removed_worktrees::stop_own_socket_when_the_worktree_is_gone(instance.as_str(), journal.as_path()),
        "the premise: the worktree is gone"
    );
    assert!(note.exists(), "the note stays while this process can still write");
    assert!(!service_is_gone(sockets.as_path(), instance.as_str()), "and the instance is not read as stopped");
    // The address stopped answering: a socket file is gone, and a name has no file and goes with the process that held it.
    match crate::service_address_of_a_worktree::socket_address_of(runtime.as_path(), instance.as_str()) {
        crate::service_server::Bind::Path(socket) => assert!(!socket.exists(), "the address stopped answering"),
        crate::service_server::Bind::Abstract(_) => {}
        crate::service_server::Bind::Inherited => panic!("an instance's address is never a descriptor another program passed in"),
    }
    match was {
        Some(value) => unsafe { std::env::set_var("XDG_RUNTIME_DIR", value) },
        None => unsafe { std::env::remove_var("XDG_RUNTIME_DIR") },
    }
    // A note whose process is gone is the answer a sweep acts on, and it is the one a killed service leaves.
    std::fs::write(note.as_path(), "1 linux:000000\n").expect("a killed service's note");
    assert!(service_is_gone(sockets.as_path(), instance.as_str()), "a note that names no process is a stopped instance");
}
