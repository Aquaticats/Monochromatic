//! What:
//!  Controls of what the sweep does about an instance whose worktree is gone:
//!  a live server keeps its address even when its note is stale,
//!  and a dead instance's files go.
//! Why:
//!  The review of `53726ade2d` found a live service whose note had been removed by another process;
//!  the sweep would then have unlinked the address of a service that was running. The note's death and the kernel's
//!  refusal of a connection are two proofs, and neither is a deadline.

use super::*;
use crate::service_server_pid_note_of_an_instance::{path as pid_note, write as write_note};

/// A short runtime directory of this test's own, and one instance in it.
fn machine(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::short_directory_for_sockets(format!("sweep-{name}").as_str());
    std::fs::create_dir_all(root.join(SOCKET_DIRECTORY).as_path()).expect("the socket folder");
    return root;
}

/// A socket a live process holds, and a note that names nobody: the state the review produced.
#[test]
fn a_live_address_whose_note_is_stale_is_left_alone() {
    let runtime: PathBuf = machine("live-stale-note");
    let instance: &str = "3f9a";
    let socket: PathBuf = socket_path(runtime.as_path(), instance);
    let listening = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a live server's address");
    std::fs::write(pid_note(runtime.join(SOCKET_DIRECTORY).as_path(), instance).as_path(), "1 linux:000000\n").expect("a stale note");
    remove_what_a_stopped_instance_left(runtime.as_path(), instance);
    assert!(socket.exists(), "the live server's address is not unlinked");
    drop(listening);
}

/// What:
///  A service's own exit removes its own address and note,
///  and leaves a note that names another process.
/// Why:
///  The removal is a comparison with this process, not with the path.
#[test]
fn a_service_removes_only_the_note_that_names_it() {
    let runtime: PathBuf = machine("own-note");
    let instance: &str = "3f9a";
    let directory: PathBuf = runtime.join(SOCKET_DIRECTORY);
    let socket: PathBuf = socket_path(runtime.as_path(), instance);
    let listening = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("this service's address");
    let folders: UnitFolders<'_> = UnitFolders { runtime_dir: runtime.as_path(), state_dir: None };
    // Another process's note at this instance's path: nothing of it is this service's to remove.
    std::fs::write(pid_note(directory.as_path(), instance).as_path(), "1 linux:000000\n").expect("another server's note");
    stop_own_socket(&folders, instance);
    assert!(socket.exists(), "the address stays while the note names another process");
    assert!(pid_note(directory.as_path(), instance).exists(), "and so does that note");
    // This process's own note: both go.
    write_note(pid_note(directory.as_path(), instance).as_path(), i32::try_from(std::process::id()).expect("a process number")).expect("our note");
    stop_own_socket(&folders, instance);
    assert!(!socket.exists() && !pid_note(directory.as_path(), instance).exists(), "this service's own files go");
    drop(listening);
}
