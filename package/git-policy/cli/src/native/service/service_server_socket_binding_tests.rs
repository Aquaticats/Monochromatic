//! Controls of the text a wait and a failure give the address of a service:
//! a path a person can look at,
//! an abstract name that is said to be a name,
//! and the descriptor the service manager passed in.

use super::*;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// What:
///  Every kind of address is named in words a person can act on,
/// and an address that is no path never claims to be one.
/// Why:
///   The notice and the failure that end a wait name the address,
/// which is the one fact that tells two worktrees'
///  services apart;
/// an abstract name has no file to look at,
/// so naming it as a path would send the person to a file that does not
///  exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(addressText({ path: '/run/user/1000/x.sock' })).toBe('/run/user/1000/x.sock');
/// ```
#[test]
fn every_kind_of_address_is_named_in_words_a_person_can_act_on() {
    let socket: PathBuf = PathBuf::from("/run/user/1000/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock");
    assert_eq!(address_text(&Bind::Path(socket.clone())), socket.display().to_string(), "a path is the path");
    let name: String = String::from("monochromatic-git-policy-cli-0123");
    assert_eq!(address_text(&Bind::Abstract(name.clone())), format!("the name {name}"), "an abstract name is said to be a name and never a path");
    assert_eq!(
        address_text(&Bind::Inherited),
        String::from("the one this service was started with"),
        "the descriptor the service manager passed in names no address of its own"
    );
}

/// What:
///  A bind leaves a socket that accepts a connection to its service,
///  and takes over only one the kernel refuses.
/// Why:
///  The review of `53726ade2d` reproduced two live services for one worktree, because a socket that accepted and greeted
///  nothing within a second was taken over: a service that is recovering, starved or stopped in a debugger is silent and
///  alive. The refusal is the kernel's proof; a silence is not.
#[test]
fn a_socket_that_accepts_is_left_to_its_service_and_a_refused_one_is_taken_over() {
    let root: std::path::PathBuf = crate::test_support::short_directory_for_sockets("bind-over-silent");
    let path: std::path::PathBuf = root.join("3f9a.sock");
    // A service that holds the address and is stopped: it accepts into its backlog and greets nothing.
    let silent: UnixListener = UnixListener::bind(path.as_path()).expect("the service's address");
    let holding = std::thread::spawn(move || {
        let mut held: Vec<UnixStream> = Vec::new();
        while let Ok((stream, _)) = silent.accept() {
            held.push(stream);
            if held.len() > 8 {
                break;
            }
        }
    });
    let refused: std::io::Error = bind_path(path.as_path()).expect_err("a socket that accepts keeps its service's address");
    assert_eq!(refused.kind(), std::io::ErrorKind::AddrInUse, "{refused}");
    // The service's own address still answers.
    let _connected: UnixStream = UnixStream::connect(path.as_path()).expect("the service's address answers");
    std::mem::forget(holding);

    // A socket nothing holds refuses, and that one is taken over: the case a killed service leaves.
    let stale_root: std::path::PathBuf = crate::test_support::short_directory_for_sockets("bind-stale");
    let stale: std::path::PathBuf = stale_root.join("3f9a.sock");
    let killed: UnixListener = UnixListener::bind(stale.as_path()).expect("a socket a service left");
    drop(killed);
    let taken: UnixListener = bind_path(stale.as_path()).expect("a refused socket is taken over");
    let _reaches: UnixStream = UnixStream::connect(stale.as_path()).expect("and the new one answers");
    drop(taken);
}

/// What:
///  A bind against a listening socket whose queue is full ends at its bound, and takes nothing over.
/// Why:
///  The takeover probe used a plain connect, so a service stopped with a full queue held the probe for as long as it stayed
/// stopped: the review of `3d5c1a0d8c` found the self-healing start unable to run for that reason. A full queue is not a
/// refusal, so it is not a socket to take over either.
#[test]
fn a_bind_against_a_full_queue_ends_at_its_bound_and_takes_nothing() {
    let root: std::path::PathBuf = crate::test_support::short_directory_for_sockets("bind-full-queue");
    let path: std::path::PathBuf = root.join("3f9a.sock");
    let listener: UnixListener = UnixListener::bind(path.as_path()).expect("a listening socket");
    let queued = crate::test_support::fill_a_socket_backlog(path.as_path());
    assert!(!queued.is_empty(), "the kernel queued at least one connection");
    let began = std::time::Instant::now();
    let refused: std::io::Error = bind_path(path.as_path()).expect_err("a socket that holds a full queue keeps its address");
    let took = began.elapsed();
    assert_eq!(refused.kind(), std::io::ErrorKind::AddrInUse, "{refused}");
    assert!(took < crate::wait_ceilings::CONNECT_CEILING + std::time::Duration::from_secs(1), "the probe ends at its bound: {took:?}");
    drop(queued);
    drop(listener);
}
