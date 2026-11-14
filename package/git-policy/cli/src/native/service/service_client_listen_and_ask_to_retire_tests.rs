//! What:
//!  Controls for what a client learns about the process that took its connection:
//!  that the comparison accepts this account and refuses another,
//!  and that the kernel really reports a user id for a live connection.
//! Why:
//!  A socket file is protected by the directory it stands in,
//! and an abstract name has no file and no permission bits,
//! so the kernel's word about the peer is what keeps another account out of a service that answers at one.
//! Another account's process cannot be started from a control,
//! which runs as the account that started the test suite,
//!  so the comparison is held at both answers here
//!  and the read is proven to give this account's own id for a connection of this account's:
//!  a check that answered "no" to everything would send every commit of a long runtime directory away.
//! The service asks the same question of every client before it reads a byte (`service_server_connection_handling.rs`).

use super::*;

/// What:
///  The comparison accepts this account's own id and refuses any other.
/// Why:
///  This is the whole of what stands in for a directory's mode on an address that has none,
/// and both answers have to be right:
///  a comparison that refused another id would let a foreign process be taken for the service,
///  and one that refused the account's own would fail every commit under a long runtime directory.
#[test]
fn a_connection_of_this_account_is_accepted_and_another_is_not() {
    let own: u32 = own_users_id();
    assert!(a_connection_of_the_account(Some(own), own), "this account's own connection");
    assert!(!a_connection_of_the_account(Some(own + 1), own), "a connection another account's process took");
    assert!(!a_connection_of_the_account(Some(0), own), "a connection of the root account");
    assert!(!a_connection_of_the_account(None, own), "a connection the kernel would not speak about");
}

/// What:
///  A directory under the short scratch folder of this test process,
///  short enough for a socket name.
/// Why:
///   A socket address carries 107 bytes of path,
/// and a fixture root under the build directory is far longer,
///  so a control of a socket stands where the package's own fixtures keep their runtime directories.
fn short_scratch(tag: &str) -> std::path::PathBuf {
    let directory: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os("HOME").expect("a home directory"))
        .join("temp/agent")
        .join(format!("cu{}-{tag}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("the scratch directory");
    crate::test_support::track_for_this_thread(directory.as_path());
    return directory;
}

/// What:
///  The kernel reports this account's id for a connection this account's own process took,
///  and `connect` says the connection is ours.
/// Why:
///  The comparison above is only as good as the read behind it,
///  and a read that answered nothing would refuse every service this program starts.
#[test]
fn the_kernel_reports_this_account_for_a_connection_of_this_account() {
    let socket: std::path::PathBuf = short_scratch("account").join("s.sock");
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind(&socket).expect("a listening socket");
    let Reached::Ours(stream) = connect(&Bind::Path(socket)) else {
        panic!("a connection this account's own listener took is ours")
    };
    assert_eq!(the_users_id_on_the_other_end(&stream), Some(own_users_id()), "the kernel named this account");
    assert!(connection_is_this_account(&stream), "and the comparison accepts it");
    drop(listener);
}

/// What:
///  Nothing listens at an address whose socket file is not there,
///  and nothing listens at an abstract name nobody bound.
/// Why:
///  Every wait of a commit starts from what a connection came to,
/// and an address nothing answers must read as that and not as another account's.
#[test]
fn a_connection_to_nothing_is_nothing() {
    assert!(!matches!(connect(&Bind::Path(short_scratch("nothing").join("no-service-here.sock"))), Reached::Ours(_)), "no socket file, nothing");
    assert!(
        matches!(connect(&Bind::Abstract(String::from("monochromatic-git-policy-cli/nobody/listens/here"))), Reached::Nothing),
        "no such name, nothing"
    );
}
