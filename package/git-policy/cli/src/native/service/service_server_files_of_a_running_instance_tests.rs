//! Controls of the files a running service owns: the address and the note, made and removed.

use super::*;
use crate::service_server_pid_note_of_an_instance::read as read_note;

fn folder(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture_root().join(format!("own-files-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}

/// Taking the files makes the folder private, keeps the socket to this account, names this process, and removing them leaves nothing.
#[test]
fn the_files_are_made_and_removed() {
    let root = folder("take");
    let socket = root.join("3f9a.sock");
    let _listener = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a socket");
    let files = take(socket.as_path(), Some("3f9a")).expect("the files are taken");
    let note = files.pid_note.clone().expect("the note is beside the address");
    assert_eq!(read_note(note.as_path()).expect("the note reads").pid, i32::try_from(std::process::id()).expect("a process number"));
    files.remove();
    assert!(!note.exists(), "the note goes with the service");
    assert!(!socket.exists(), "the address goes with the service");
}

/// The note of a service whose number has another birth names nothing.
#[test]
fn a_note_of_another_birth_names_nothing() {
    let root = folder("stale");
    let socket = root.join("3f9a.sock");
    let note = crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a");
    std::fs::write(note.as_path(), "1 linux:000000\n").expect("a stale note");
    std::fs::write(socket.as_path(), "").expect("a file where the address was");
    let files = OwnFiles { socket: Some(socket.clone()), pid_note: Some(note.clone()) };
    files.remove();
    assert!(!socket.exists() && !note.exists(), "a stale address and note are removed");
}
