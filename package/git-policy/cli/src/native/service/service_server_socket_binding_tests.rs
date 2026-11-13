//! Controls of the text a wait and a failure give the address of a service:
//! a path a person can look at,
//! an abstract name that is said to be a name,
//! and the descriptor the service manager passed in.

use super::*;
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
///  A bind takes the address over from a socket that accepts a connection and greets nothing,
///  which is a service that is exiting.
/// Why:
///  Such a service held the address for 33 seconds on 2026-11-12 while a commit waited 37 seconds on it;
/// the next server takes the name within one greeting's read, and the old one keeps its own socket until it ends.
#[test]
fn a_socket_that_greets_nothing_is_taken_over() {
    // A socket path must fit in a socket address, which the fixture tree's depth does not allow.
    let root: std::path::PathBuf = crate::test_support::short_directory_for_sockets("bind-over-silent");
    let path: std::path::PathBuf = root.join("3f9a.sock");
    let dying: UnixListener = UnixListener::bind(path.as_path()).expect("the dying service's address");
    let holding = std::thread::spawn(move || {
        let mut held: Vec<UnixStream> = Vec::new();
        while let Ok((stream, _)) = dying.accept() {
            held.push(stream);
            if held.len() > 8 {
                break;
            }
        }
    });
    let began: std::time::Instant = std::time::Instant::now();
    let taken: UnixListener = bind_path(path.as_path()).expect("the next server takes the address");
    assert!(began.elapsed() < std::time::Duration::from_secs(5), "the takeover is one greeting's read, not a wait on the dying process");
    // The new server is the one a connection reaches now.
    let _connected: UnixStream = UnixStream::connect(path.as_path()).expect("a connection to the new address");
    drop(taken);
    std::mem::forget(holding);
}
