//! Controls of the files a running service owns:
//! the claim that names this process before the address is bound,
//! the mode the address keeps once it is bound,
//! and the removal that leaves another service's files alone.

use super::*;
use crate::service_server_pid_note_of_an_instance::read as read_note;

fn folder(name: &str) -> PathBuf {
    return crate::test_support::short_directory_for_sockets(format!("own-files-{name}").as_str());
}

/// What this process's number is, as the note holds it.
fn our_pid() -> i32 {
    return i32::try_from(std::process::id()).expect("a process number");
}

/// What:
///  Claiming an instance names this process before anything is bound,
///  and removing the files leaves nothing behind.
/// Why:
///   A note that exists before the address does is what tells a second service that this instance already has one;
/// a note written only after the bind leaves the moment of the bind unnamed,
/// which is what the fourth review of issue 6293 (finding 1) read the chain in.
#[test]
fn the_instance_is_claimed_before_the_address_exists_and_the_files_are_removed_after() {
    let root = folder("claim");
    let socket = root.join("3f9a.sock");
    let files = claim(socket.as_path(), "3f9a").expect("the instance is claimed");
    let note = files.note().expect("the note is beside the address").to_path_buf();
    assert!(note.exists(), "the note is written before the address is bound");
    assert!(!socket.exists(), "and no address is bound yet");
    assert_eq!(read_note(note.as_path()).expect("the note reads").pid, our_pid(), "the note names this process");
    let _listener = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("the address is bound after the claim");
    files.keep_the_address_to_this_account().expect("the address's mode is set once it exists");
    assert!(socket.exists(), "the address is there");
    files.remove();
    assert!(!note.exists(), "the note goes with the service");
    assert!(!socket.exists(), "the address goes with the service");
}

/// What:
///  A claim is refused while another live process holds the instance,
///  and taken over when the note names a process that is gone.
/// Why:
///   This is the rule that keeps one instance to one service:
/// a live holder keeps its instance,
/// and a killed service's note is nobody's,
/// which the birth identity decides.
#[test]
fn a_live_holder_keeps_its_instance_and_a_dead_ones_note_is_taken_over() {
    let root = folder("holder");
    let note = crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a");
    let other = crate::test_support::a_live_process_that_is_not_this_one();
    // The note names a process that is alive and is not this one, as a service of this build writes it.
    let identity: String = crate::process_identity::process_birth_identity(i64::from(other.pid())).expect("a live process's birth").expect("it is there");
    std::fs::write(note.as_path(), format!("{} {identity}\n", other.pid())).expect("a live service's note");
    let refused = claim(root.join("3f9a.sock").as_path(), "3f9a").expect_err("a live holder keeps its instance");
    assert!(refused.to_string().contains(format!("process {}", other.pid()).as_str()), "the refusal names the process that holds it: {refused}");
    assert_eq!(read_note(note.as_path()).expect("the note still reads").pid, other.pid(), "and the note is untouched");
    // The same note with a birth no process has is a killed service's leftovers, which the next claim takes over.
    std::fs::write(note.as_path(), format!("{} linux:000000\n", other.pid())).expect("a killed service's note");
    let claimed = claim(root.join("3f9a.sock").as_path(), "3f9a").expect("a dead holder's note is taken over");
    assert_eq!(read_note(note.as_path()).expect("the note reads").pid, our_pid(), "and it now names this process");
    claimed.remove();
}

/// What:
///  A service that ends removes only the files that still name it.
/// Why:
///   The fourth review of issue 6293 (finding 1) measured the opposite:
/// an instance that ended deleted the address and the note of the service that had taken the instance over,
/// by path and without looking,
/// so a live successor lost its address and the next command started yet another service.
/// The note is the identity of the owner,
/// so a note that names another process is that process's to remove.
#[test]
fn an_ending_service_leaves_the_files_of_the_service_that_took_its_instance_over() {
    let root = folder("successor");
    let socket = root.join("3f9a.sock");
    let note = crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a");
    let successor = crate::test_support::a_live_process_that_is_not_this_one();
    let identity: String = crate::process_identity::process_birth_identity(i64::from(successor.pid())).expect("a live process's birth").expect("it is there");
    std::fs::write(note.as_path(), format!("{} {identity}\n", successor.pid())).expect("the successor's note");
    let listener = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("the successor's address");
    let files = OwnFiles { socket: Some(socket.clone()), pid_note: Some(note.clone()) };
    files.remove();
    assert!(socket.exists(), "the successor's address is left alone");
    assert!(note.exists(), "and its note with it");
    assert_eq!(read_note(note.as_path()).expect("the note still reads").pid, successor.pid(), "which still names the successor");
    drop(listener);
    // A note that is not there names nobody, so nothing beside it is certainly this process's: the sweep's to remove, never this
    //  service's to take from another.
    std::fs::remove_file(note.as_path()).expect("the note is gone");
    files.remove();
    assert!(socket.exists(), "an address whose note names nobody is left for the sweep");
    std::fs::remove_file(socket.as_path()).expect("the fixture is cleaned");
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
    assert!(socket.exists() && note.exists(), "a stale note names no process, so the files are the sweep's rather than another's to lose");
    assert!(matches!(read_note(note.as_path()), Some(named) if !crate::service_server_pid_note_of_an_instance::is_still_the_process(&named)), "the premise: the note names no process");
}
