//! What:
//!  Controls for the address of one worktree's service:
//! which form a runtime directory gets,
//! that the abstract name fits and is one name per runtime directory and instance,
//! that the kernel really binds and reaches one,
//! and what a unit file and a table of sockets say about it.
//! Why:
//!  The address is the whole answer to a runtime directory too long for a path,
//! so a name that did not fit,
//! two spellings of one directory that made two services,
//! or a table looked at wrongly would put the failure back where this change takes it from.
//! The kernel's own boundary is held here rather than a number copied from a document:
//! the longest path and the longest abstract name both bind,
//! and one byte each past them is refused.

use super::*;
use crate::service_instance::{
    INSTANCE_LENGTH, LONGEST_SOCKET_PATH, SOCKET_DIRECTORY, SOCKET_ADDRESS_BYTES, longest_runtime_directory, socket_path,
    socket_path_suffix_bytes,
};
use crate::test_support::{fixture, remove};

/// An instance name of the shape this program writes,
/// whose length every control here can rely on.
const INSTANCE: &str = "0123456789abcdef0123456789abcdef";

/// What:
///  A runtime directory with a path of exactly `length` bytes,
/// under the short scratch directory of this test process.
/// Why:
///   The boundary is a count of bytes,
/// so a control that guessed the length would prove nothing about it.
/// One socket address holds 107 bytes,
/// and the fixture root under the build directory is far longer than the room the runtime directory has,
/// so the fixture stands where the package's own binary fixtures put their runtime directories.
fn runtime_directory_of_length(length: usize, tag: &str) -> PathBuf {
    let parent: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory"))
        .join("temp/agent")
        .join(format!("sa{}", std::process::id()));
    std::fs::create_dir_all(&parent).expect("the scratch directory");
    let room: usize = length.checked_sub(parent.as_os_str().len() + 1).expect("a name long enough to build");
    // The name holds the tag where there is room for it and padding to the byte the control asks for. A runtime directory of
    //  thirty-nine bytes leaves seven or eight bytes past this process's own scratch folder, and a process number with another
    //  digit narrows that, so a tag that does not fit is replaced by a digest of itself: two controls still get two
    //  directories, whichever number this process runs as.
    // A tag of words needs room for itself; where a control's directory leaves none, a digest of the tag keeps two
    //  controls apart, and the digest needs eight bytes.
    assert!(room >= 2, "a runtime directory of {length} bytes leaves {room} bytes past this process's scratch folder, too little to name a control in");
    let name: String = if room > tag.len() {
        format!("{tag}{}", "x".repeat(room - tag.len()))
    } else {
        assert!(room >= 8, "the tag {tag:?} does not fit {room} bytes, and a digest of it needs eight");
        let mut digest: String = crate::service_instance::digest_of_bytes(tag.as_bytes());
        digest.truncate(room);
        digest
    };
    let runtime: PathBuf = parent.join(name);
    std::fs::create_dir_all(&runtime).expect("the runtime directory");
    assert_eq!(runtime.as_os_str().len(), length, "the directory is exactly as long as the control asks");
    crate::test_support::track_for_this_thread(runtime.as_path());
    return runtime;
}

/// A runtime directory whose socket path still fits one address,
/// with a byte of room to spare.
fn a_runtime_directory_with_room(tag: &str) -> PathBuf {
    return runtime_directory_of_length(longest_runtime_directory(INSTANCE) - 1, tag);
}

/// A runtime directory one byte too long for a socket path.
fn a_runtime_directory_past_the_path(tag: &str) -> PathBuf {
    return runtime_directory_of_length(longest_runtime_directory(INSTANCE) + 1, tag);
}

/// What:
///  A runtime directory inside the room of a path keeps the socket file it has always had.
/// Why:
///   Every account whose runtime directory is short enough must be served exactly as before,
/// and a change of form a byte early would move the socket of the ordinary session.
#[test]
fn a_runtime_directory_with_room_keeps_the_socket_file() {
    let runtime: PathBuf = a_runtime_directory_with_room("room");
    assert_eq!(socket_address_of(runtime.as_path(), INSTANCE), Bind::Path(socket_path(runtime.as_path(), INSTANCE)));
    assert_eq!(listen_stream_of(runtime.as_path(), INSTANCE), format!("%t/{SOCKET_DIRECTORY}/%i.sock"), "the unit names the file it always named");
}

/// What:
///  A runtime directory whose socket path would take one byte more than an address holds is served at an abstract name.
/// Why:
///   This is the case the whole change is for:
/// the kernel refuses the path,
/// so the commit had nothing to connect to,
/// while real Git made the commit.
#[test]
fn a_runtime_directory_past_the_path_is_served_at_an_abstract_name() {
    let runtime: PathBuf = a_runtime_directory_past_the_path("past");
    assert!(socket_path(runtime.as_path(), INSTANCE).as_os_str().len() > LONGEST_SOCKET_PATH, "the path form does not fit");
    assert_eq!(socket_address_of(runtime.as_path(), INSTANCE), Bind::Abstract(abstract_name_of(runtime.as_path(), INSTANCE)));
}

/// What:
///  An abstract name fits the field one abstract name may take, for the longest runtime directory there can be,
///  and it carries the account, the runtime directory and the instance.
/// Why:
///   A name that did not fit would fail the commit at the moment the kernel refused it,
/// which is the failure this change removes,
/// and the parts are what tell two services apart.
#[test]
fn an_abstract_name_fits_and_names_its_account_its_directory_and_its_instance() {
    let deep: PathBuf = PathBuf::from("/home/someone").join("a/deep/scratch/folder".repeat(40));
    let name: String = abstract_name_of(deep.as_path(), INSTANCE);
    assert!(name.len() <= LONGEST_SOCKET_PATH, "an abstract name of {} bytes fits the {LONGEST_SOCKET_PATH} one may take: {name}", name.len());
    assert!(name.starts_with(format!("{SOCKET_DIRECTORY}/").as_str()), "the service's own name comes first: {name}");
    assert!(name.ends_with(format!("/{INSTANCE}").as_str()), "the instance is the last part: {name}");
    let account: u32 = rustix::process::geteuid().as_raw();
    assert!(name.contains(format!("/{account}/").as_str()), "the account is named: {name}");
    assert_eq!(INSTANCE.len(), INSTANCE_LENGTH, "the instance is the length this program writes");
    // The digest is the only part that may change with the directory, and it does.
    let other: String = abstract_name_of(Path::new("/home/someone/another/folder"), INSTANCE);
    assert_ne!(name, other, "two runtime directories are two names");
    assert_ne!(abstract_name_of(deep.as_path(), INSTANCE), abstract_name_of(deep.as_path(), "fedcba9876543210fedcba9876543210"), "two instances are two names");
}

/// What:
///  Two spellings of one runtime directory are one name,
///  and a directory that does not exist yet gets the name it will have once it does.
/// Why:
///   The command's `XDG_RUNTIME_DIR` is the person's to spell,
/// and provisioning creates the directory a moment after the route named it:
/// two names for one directory would give the manager one address and the client another,
/// which is the ten-second wait this file exists to end.
#[test]
fn two_spellings_of_one_directory_are_one_name_and_a_directory_to_be_made_keeps_its_name() {
    let runtime: PathBuf = a_runtime_directory_past_the_path("spellings");
    let runtime_name_before: String = abstract_name_of(runtime.as_path(), INSTANCE);
    let linked: PathBuf = runtime.parent().expect("a parent").join("link-to-the-runtime-directory");
    std::os::unix::fs::symlink(runtime.as_path(), &linked).expect("a link to the runtime directory");
    crate::test_support::track_for_this_thread(linked.as_path());
    let dotted: PathBuf = runtime.join("..").join(runtime.file_name().expect("a name"));
    assert_eq!(abstract_name_of(linked.as_path(), INSTANCE), abstract_name_of(runtime.as_path(), INSTANCE), "a link names the same service");
    assert_eq!(abstract_name_of(dotted.as_path(), INSTANCE), abstract_name_of(runtime.as_path(), INSTANCE), "a relative part names the same service");
    // A directory below the one that exists, which provisioning makes with `mkdir`, keeps the name it had before it was made.
    let to_be_made: PathBuf = runtime.join("systemd/user");
    let before_making_it: String = abstract_name_of(to_be_made.as_path(), INSTANCE);
    std::fs::create_dir_all(&to_be_made).expect("the directory provisioning makes");
    assert_eq!(
        the_place_a_directory_names(to_be_made.as_path()),
        the_place_a_directory_names(runtime.as_path()).join("systemd").join("user"),
        "the place of a directory to be made is the real path of the one above it, with the names below still to make"
    );
    assert_eq!(abstract_name_of(to_be_made.as_path(), INSTANCE), before_making_it, "making a directory does not rename the service it names");
    assert_eq!(abstract_name_of(&runtime, INSTANCE), runtime_name_before, "and the runtime directory's own name never moved");
}

/// What:
///  The kernel binds an abstract name of the length this program writes and reaches it,
///  and it refuses the name the socket unit would refuse.
/// Why:
///  The name has to be one the kernel carries:
/// an abstract name takes the same field as a path,
/// and neither this file nor the unit's parser may assume a number the kernel decides.
#[test]
#[cfg(target_os = "linux")]
fn the_kernel_binds_and_reaches_an_abstract_name_of_this_length() {
    use std::os::linux::net::SocketAddrExt;
    let runtime: PathBuf = a_runtime_directory_past_the_path("binds");
    let name: String = abstract_name_of(runtime.as_path(), INSTANCE);
    let address = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("the name fits");
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind_addr(&address).expect("the kernel binds this name");
    assert!(std::os::unix::net::UnixStream::connect_addr(&address).is_ok(), "the kernel reaches the name it bound");
    assert!(the_address_is_there(&Bind::Abstract(name)), "the table of unix sockets holds the name while the socket listens");
    drop(listener);
    // A name at the field's own length is refused, which is the limit the path form has as well.
    let too_long: String = "n".repeat(SOCKET_ADDRESS_BYTES);
    assert!(
        std::os::unix::net::SocketAddr::from_abstract_name(too_long.as_bytes()).is_err(),
        "a name of {SOCKET_ADDRESS_BYTES} bytes is refused before any bind"
    );
    assert!(!the_address_is_there(&Bind::Abstract(too_long)), "and no table holds it");
}

/// What:
///  A name the table lists is found,
///  a name it does not is not,
///  and a name that is the ending of another is not the other.
/// Why:
///  The table is the only way to see an abstract socket without connecting to it,
/// and a connection would start the service the caller is deciding whether to start.
/// The kernel prints the name with `@` before it,
/// which this control holds against a table it writes itself,
/// so the reading is proved here and not only against a running kernel.
#[test]
fn a_name_is_found_in_the_table_of_unix_sockets_and_a_longer_name_holding_it_is_not_it() {
    let root: PathBuf = fixture("address-table");
    std::fs::create_dir_all(root.join("net")).expect("the table's directory");
    let name: String = String::from("monochromatic-git-policy-cli/1000/0123456789abcdef/0123456789abcdef0123456789abcdef");
    let wanted: Vec<u8> = abstract_name_column(name.as_str());
    // The kernel's own columns for a listening abstract socket.
    let listening: String = format!("0000000000000000: 00000002 00000000 00010000 0001 01 12345678 {}\n", String::from_utf8_lossy(&wanted));
    let not_listening: String = format!("0000000000000000: 00000002 00000000 00000000 0001 02 12345678 {}\n", String::from_utf8_lossy(&wanted));
    let longer: String = format!("0000000000000000: 00000002 00000000 00010000 0001 01 12345678 {}ed\n", String::from_utf8_lossy(&wanted));
    for (tag, table, there) in [("listening", listening.as_str(), true), ("not listening", not_listening.as_str(), false), ("a longer name holding it", longer.as_str(), false)] {
        std::fs::write(root.join("net/unix"), table.as_bytes()).expect("a table");
        assert_eq!(the_address_is_in(root.as_path(), &Bind::Abstract(name.clone())), there, "{tag}");
    }
    remove(root.as_path());
}

/// What:
///  A path address is there when its file is, and an abstract one is not there when nothing of its name listens.
/// Why:
///  The route hands an existing socket to the command,
/// and the path form's answer must stay the one it had (`service_route_tests.rs`).
#[test]
fn a_path_address_is_there_when_its_file_is() {
    let root: PathBuf = fixture("address-path-is-there");
    let socket: PathBuf = root.join("s.sock");
    assert!(!the_address_is_there(&Bind::Path(socket.clone())), "no file, no address");
    std::fs::write(&socket, b"a stand-in for the file a manager made").expect("the file");
    assert!(the_address_is_there(&Bind::Path(socket)), "a file at the path is an address that is there");
    remove(root.as_path());
}

/// What:
///  The bytes a socket path adds to its runtime directory are the ones the longest runtime directory comes from.
/// Why:
///  The two forms meet at one boundary,
/// and a check that counted other bytes would send a person whose runtime directory works to a form it does not need,
/// or keep the path form one byte past what the kernel carries.
#[test]
fn the_boundary_between_the_two_forms_is_the_one_the_kernel_has() {
    let longest: usize = longest_runtime_directory(INSTANCE);
    assert_eq!(longest + socket_path_suffix_bytes(INSTANCE), LONGEST_SOCKET_PATH);
    let at_the_boundary: PathBuf = runtime_directory_of_length(longest, "at");
    assert_eq!(socket_path(at_the_boundary.as_path(), INSTANCE).as_os_str().len(), LONGEST_SOCKET_PATH, "a path at the boundary fills the field");
    assert!(matches!(socket_address_of(at_the_boundary.as_path(), INSTANCE), Bind::Path(_)), "the boundary itself keeps the path form");
    let past: PathBuf = runtime_directory_of_length(longest + 1, "past-boundary");
    assert!(matches!(socket_address_of(past.as_path(), INSTANCE), Bind::Abstract(_)), "one byte past it is an abstract name");
}
