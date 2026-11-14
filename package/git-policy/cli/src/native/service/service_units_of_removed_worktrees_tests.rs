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

/// What:
///  The files a gone worktree's instance left go together, and a claim file goes with them.
/// Why:
///  A claim file is a file the wrapper wrote in a runtime directory it served, and the worktree that made the takeover
///  possible is gone (the review of `55f236a0a8`, finding 3).
#[test]
fn a_gone_worktrees_instance_loses_its_claim_file_with_its_other_files() {
    let runtime: PathBuf = machine("claim-gone");
    let instance: &str = "3f9a";
    let socket: PathBuf = socket_path(runtime.as_path(), instance);
    let claim: PathBuf = crate::service_address_takeover_claim::path(socket.as_path());
    // A socket a killed service left, and the claim file its takeover made.
    drop(std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a socket a killed service left"));
    std::fs::write(claim.as_path(), b"").expect("the claim file a takeover made");
    remove_what_a_stopped_instance_left(runtime.as_path(), instance);
    assert!(!socket.exists(), "the address goes");
    assert!(!claim.exists(), "and the claim file with it: {}", claim.display());
}

/// What:
///  A live address keeps its claim file, because a takeover of it may still be in flight.
/// Why:
///  The socket file goes only when nothing answers; the claim file of an instance whose service still runs is the one a
///  taking-over process holds, and removing it would let two claims be made on one address.
#[test]
fn a_live_address_keeps_its_claim_file() {
    let runtime: PathBuf = machine("claim-live");
    let instance: &str = "3f9a";
    let socket: PathBuf = socket_path(runtime.as_path(), instance);
    let claim: PathBuf = crate::service_address_takeover_claim::path(socket.as_path());
    let listening = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a live server's address");
    std::fs::write(claim.as_path(), b"").expect("a claim file");
    remove_what_a_stopped_instance_left(runtime.as_path(), instance);
    assert!(claim.exists(), "a live address keeps its claim file");
    drop(listening);
}

/// What:
///  The sweep that forgets a stopped instance removes its claim file too.
/// Why:
///  That sweep is the one a person runs for a worktree that is gone, and a claim file left there is a file the wrapper
///  leaves for good (the review of `55f236a0a8`, finding 3).
#[test]
fn forgetting_a_stopped_instance_removes_its_claim_file() {
    let runtime: PathBuf = machine("claim-forgotten");
    let instance: &str = "3f9a";
    let socket: PathBuf = socket_path(runtime.as_path(), instance);
    let claim: PathBuf = crate::service_address_takeover_claim::path(socket.as_path());
    let note: PathBuf = note_path(runtime.as_path(), instance);
    std::fs::create_dir_all(note.parent().unwrap_or(runtime.as_path())).expect("the folder the note sits in");
    std::fs::write(note.as_path(), b"gitdirHex=2f7265706f2f2e676974").expect("the note of the worktree");
    std::fs::write(socket.as_path(), b"").expect("a socket file");
    std::fs::write(claim.as_path(), b"").expect("a claim file");
    forget_stopped_instance(runtime.as_path(), instance);
    assert!(!note.exists() && !socket.exists(), "the note and the socket go");
    assert!(!claim.exists(), "and the claim file: {}", claim.display());
}

/// What:
///  A service's own end removes a claim file a killed command left,
///  and leaves one another process is holding.
/// Why:
///  The claim file exists only while a takeover does, and a takeover that was killed leaves it; the service's own end is
///  the one moment its worktree's address is provably free, so that is where a leftover goes (the review of `55f236a0a8`,
///  finding 3). A file another process holds is that process's to remove.
#[test]
fn a_service_removes_a_leftover_claim_file_and_leaves_a_held_one() {
    let runtime: PathBuf = machine("claim-leftover");
    let instance: &str = "3f9a";
    let socket: PathBuf = socket_path(runtime.as_path(), instance);
    let claim_path: PathBuf = crate::service_address_takeover_claim::path(socket.as_path());
    let listening = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("this service's address");
    let files = crate::service_server_files_of_a_running_instance::take(socket.as_path(), Some(instance)).expect("the address is this process's");
    // A file a killed command left: nobody holds it.
    std::fs::write(claim_path.as_path(), b"").expect("a claim file a killed taker left");
    files.remove();
    assert!(!socket.exists(), "the address goes");
    assert!(!claim_path.exists(), "and the claim file a killed command left goes with it: {}", claim_path.display());

    // A file another process holds: this claim stands where that process's would.
    let again = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("the address again");
    let files_again = crate::service_server_files_of_a_running_instance::take(socket.as_path(), Some(instance)).expect("the address is this process's again");
    let held = match crate::service_address_takeover_claim::claim(socket.as_path()).expect("the claim is made") {
        crate::service_address_takeover_claim::Claim::Held(held) => held,
        crate::service_address_takeover_claim::Claim::AnotherProcessTakesOver => panic!("nothing held this claim"),
    };
    files_again.remove();
    assert!(claim_path.exists(), "a claim file another process holds is left where it is");
    drop(held);
    assert!(!claim_path.exists(), "and it goes when that claim ends");
    drop(again);
    drop(listening);
}
