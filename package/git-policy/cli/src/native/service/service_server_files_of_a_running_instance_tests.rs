//! Controls of the files a running service owns: the address and the note, made and removed.

use super::*;
use crate::service_server_pid_note_of_an_instance::read as read_note;

fn folder(name: &str) -> PathBuf {
    return crate::test_support::short_directory_for_sockets(format!("own-files-{name}").as_str());
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

/// What:
///  A service that ends leaves the address and the note of a *later* server where they are.
/// Why:
///  The review of `53726ade2d` reproduced the other behaviour with the real binary:
/// a second server had bound the same path, and the first one's exit unlinked the second's socket and note,
///  so the second was unreachable and the proof that guards a journal's removal read true of a live service.
/// The inode of the socket and the text of the note are what say whose each file is.
#[test]
fn a_service_leaves_files_that_are_not_its_own() {
    let root = folder("not-ours");
    let socket = root.join("3f9a.sock");
    // What this service bound and wrote.
    let mine = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("this service's socket");
    let files = take(socket.as_path(), Some("3f9a")).expect("the files are taken");
    let note = files.pid_note.clone().expect("the note is beside the address");
    // A later server's: another socket bound at the same path, and a note naming another process.
    std::fs::remove_file(socket.as_path()).expect("the address is freed for the later server");
    let theirs = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("the later server's socket");
    std::fs::write(note.as_path(), "1 linux:000000\n").expect("the later server's note");

    files.remove();
    assert!(socket.exists(), "the later server's address is left where it is");
    assert!(note.exists(), "and its note with it");
    assert_eq!(std::fs::read_to_string(note.as_path()).expect("the note reads"), "1 linux:000000\n");
    drop(mine);
    drop(theirs);
    // And what is still this service's own goes.
    std::fs::remove_file(socket.as_path()).expect("clean up the later socket");
    std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a socket to remove");
    let again = take(socket.as_path(), Some("3f9a")).expect("the files are taken again");
    again.remove();
    assert!(!socket.exists() && !note.exists(), "a service's own files are removed");
}
