//! Controls of the proof that a stopped instance serves nothing any more.

use super::*;
use std::os::unix::fs::PermissionsExt;

fn folder(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("gone-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}


/// What:
///  Binds whatever address the fixture instance's spelling makes,
///  a path where it fits and an abstract name where it does not.
/// Why:
///   A fixture root deep enough to make the path too long is a runtime directory this program serves at an abstract name,
/// and a control of the address must hold for both forms.
fn bind_the_instance(address: &crate::service_server::Bind) -> std::os::unix::net::UnixListener {
    return match address {
        crate::service_server::Bind::Path(socket) => std::os::unix::net::UnixListener::bind(socket).expect("a service binds the path"),
        crate::service_server::Bind::Abstract(name) => {
            use std::os::linux::net::SocketAddrExt;
            let named: std::os::unix::net::SocketAddr = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("an abstract name");
            std::os::unix::net::UnixListener::bind_addr(&named).expect("a service binds the name")
        }
        crate::service_server::Bind::Inherited => panic!("an instance's address is never a descriptor another program passed in"),
    };
}

/// The socket file an address has, for the controls that change its permissions.
fn socket_file(address: &crate::service_server::Bind) -> Option<std::path::PathBuf> {
    return match address {
        crate::service_server::Bind::Path(socket) => Some(socket.clone()),
        crate::service_server::Bind::Abstract(_) | crate::service_server::Bind::Inherited => None,
    };
}

/// An instance with no note is gone, and so is one whose number carries another birth.
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
    crate::service_server_pid_note_of_an_instance::write(note.as_path(), i32::try_from(std::process::id()).expect("a process number")).expect("the note");
    assert!(!service_is_gone(root.as_path(), "3f9a"), "the process the note names is this one");
}

/// What:
///  A service whose worktree was removed keeps its note while it can still write,
///  so a sweep never reads it as stopped.
/// Why:
///  The review of `567ae37198` (finding 1), and the sixth review of issue 6293 (finding 2) before it:
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
    crate::service_server_pid_note_of_an_instance::write(note.as_path(), i32::try_from(std::process::id()).expect("a process number")).expect("this process serves the instance");
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

/// What:
///  A service that has bound its address and has not written its note yet is not gone.
/// Why:
///  The review of `567ae37198` (finding 1):
///  a worktree is made again, a new service binds the address and is stopped before it writes its note,
///  and a sweep that read the missing note as an end then removed the journal under a live writer.
///  The address is the fact that reading was missing: a bound socket answers whether or not it will greet.
#[test]
fn a_service_that_bound_its_address_and_wrote_no_note_is_not_gone() {
    let root = folder("bound-no-note");
    let runtime: std::path::PathBuf = root.join("run");
    let sockets: std::path::PathBuf = runtime.join(crate::service_instance::SOCKET_DIRECTORY);
    std::fs::create_dir_all(sockets.as_path()).expect("socket folder");
    let instance: String = "3f9a".to_string();
    let address: crate::service_server::Bind = crate::service_address_of_a_worktree::socket_address_of(runtime.as_path(), instance.as_str());
    let _listener: std::os::unix::net::UnixListener = bind_the_instance(&address);
    assert!(!crate::service_server_pid_note_of_an_instance::path(sockets.as_path(), instance.as_str()).exists(), "the premise: no note");
    assert!(!service_is_gone(sockets.as_path(), instance.as_str()), "a bound address is a service that is there, note or no note");
    assert_eq!(address_state(&address), AddressState::Serves, "the address is what says it");
    // With the address free and still no note, the instance is gone.
    drop(_listener);
    if let Some(socket) = socket_file(&address) {
        std::fs::remove_file(socket).expect("the service ended");
    }
    assert!(service_is_gone(sockets.as_path(), instance.as_str()), "nothing bound and no note names no writer");
    assert_eq!(address_state(&address), AddressState::Nothing, "the same address now holds nothing");
}

/// What:
///  An address this command cannot read is not an end: the sweep keeps the journal.
/// Why:
///  The review of `567ae37198` (finding 1), the other half:
///  reading "nothing answers" out of an error that only says "this command could not look" would remove a journal on a
///  fact that says nothing about the writer.
#[test]
fn an_address_this_command_cannot_read_is_not_gone() {
    // A runtime directory short enough that the address is a path, and a change of its socket's mode is what makes the
    // kernel's answer one this command cannot read: the harness's own root is deep enough for an abstract name.
    let root: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join(format!(".cache/gone-cannot-read-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(root.as_path());
    std::fs::create_dir_all(root.as_path()).expect("fixture folder");
    let runtime: std::path::PathBuf = root.join("run");
    std::fs::create_dir_all(runtime.join(crate::service_instance::SOCKET_DIRECTORY).as_path()).expect("socket folder");
    let instance: String = "3f9a".to_string();
    let address: crate::service_server::Bind = crate::service_address_of_a_worktree::socket_address_of(runtime.as_path(), instance.as_str());
    let Some(socket): Option<std::path::PathBuf> = socket_file(&address) else { panic!("a short runtime directory makes a path address") };
    let _listener: std::os::unix::net::UnixListener = bind_the_instance(&address);
    // A socket file without write permission refuses a connection with a refusal about permission, not about a holder.
    let mut mode: std::fs::Permissions = std::fs::metadata(socket.as_path()).expect("the socket file").permissions();
    mode.set_mode(0o000);
    std::fs::set_permissions(socket.as_path(), mode).expect("a socket this command may not open");
    assert_eq!(address_state(&address), AddressState::CannotTell, "the kernel's answer says neither that it is held nor that it is free");
    assert!(!service_is_gone(runtime.join(crate::service_instance::SOCKET_DIRECTORY).as_path(), instance.as_str()), "an answer this command cannot read keeps the journal");
    let mut readable: std::fs::Permissions = std::fs::metadata(socket.as_path()).expect("the socket file").permissions();
    readable.set_mode(0o600);
    std::fs::set_permissions(socket.as_path(), readable).expect("the socket file is readable again for the fixture's own removal");
    let _ = std::fs::remove_dir_all(root.as_path());
}
