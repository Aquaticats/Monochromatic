//! What:
//!  The address of one worktree's commit service:
//! a socket file under the user's runtime directory when that path fits one socket address,
//! and a Linux abstract name derived from that path when it does not.
//! Why:
//!  A Unix socket address carries 107 bytes of path (`service_instance.rs`),
//! and a runtime directory is the person's to choose:
//! a scratch folder under an agent's home,
//! a container's private directory and a nested session's directory all reach past the 40 bytes a path form leaves,
//! and the kernel then refuses both the address the manager would bind and the one a commit would connect to.
//! A commit failed there where real Git made the commit,
//! which the owner's rule "Commits should never fail" does not allow.
//! An abstract name is the one address that fits both a socket unit
//! (`ListenStream=@name`,
//! measured against systemd 262 in a container: the unit is accepted,
//! the name is bound,
//! and `Listen` reports `@` and the name) and a client,
//! whatever working directory either of them runs in,
//! and it leaves nothing behind when its process dies,
//! so a killed service leaves no address to clear away.
//! The two ways a long path could be reached instead both lose to it:
//! an address relative to a held-open directory (`/proc/self/fd/<n>/<name>`) works for a process that holds the
//!  descriptor and for no one else,
//!  so a socket unit cannot name it and a client cannot guess the number;
//! a name relative to a working directory works only between two processes that chose the same directory,
//!  and a service unit binds its socket in the manager's own directory.
//! An abstract name has no permission bits,
//! so what protects it is what each end reads from the kernel about the other
//! (`SO_PEERCRED`):
//! the name is built from this account's own runtime directory and its user id,
//! and both ends refuse a connection whose peer is another account
//! (`service_client_listen_and_ask_to_retire.rs`, `service_server_connection_handling.rs`).
//! The path form stays for every runtime directory that can carry it,
//! so the ordinary commit pays nothing for this.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const address = socketAddressOf(runtimeDir, instance); // { path } | { abstractName }
//! ```

use crate::earlier_unix_sockets_that_listen::{ListeningSockets, listening_sockets_exactly};
use crate::service_instance::{LONGEST_SOCKET_PATH, SOCKET_DIRECTORY, digest_of_bytes, socket_path};
use crate::service_server::Bind;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

/// Whether an abstract name can be used at all:
/// the kernel of Linux carries one,
/// and other systems' unix sockets have no such namespace.
pub const AN_ABSTRACT_NAME_IS_AVAILABLE: bool = cfg!(target_os = "linux");

/// How many hexadecimal digits of the digest of the runtime directory an abstract name carries.
/// Why:
///   Eight bytes tell two runtime directories apart with no more room spent on them,
/// and a name that fits is what this whole file is about:
/// the prefix,
/// the user id,
/// the digest and the instance stay under the 107 bytes an abstract name, like a path, may take.
pub const DIGEST_DIGITS_IN_AN_ABSTRACT_NAME: usize = 16;

/// What:
///  The place a directory names,
/// as the file system resolves it:
/// its real path,
/// or the deepest directory above it that exists with what is spelled below that one folded onto it.
/// Why:
///   The name of an abstract socket is made once and then has to match what another process makes,
/// while two spellings of one directory (a link,
/// a `..` part,
/// a `.` part) name one place.
/// A directory that does not exist yet is the state a commit may find its own runtime directory in,
/// and provisioning creates it a moment later;
/// the client's name is made before that and the socket unit's after,
/// so the two are one name only if what is still to be made is folded the way the file system will walk it.
/// A component that does not exist yet is a directory `create_dir_all` makes and never a link,
/// so the `..` and `.` parts below it are folded by hand: a name joins,
/// `..` takes the last name away,
/// and `.` names nothing.
/// This is what the kernel would answer for those parts once they exist,
/// and the control `one_spelling_names_one_place_before_and_after_the_directory_exists` holds it.
/// A spelling nothing can resolve — a part that is a file,
/// a link that points nowhere,
/// a directory this account may not read — still gets one name from this fold,
/// the same name both ends of a commit make;
/// provisioning then cannot create the directory and stops at once, naming the setting
/// (`a_runtime_directory_that_cannot_be_made_stops_at_once_and_names_it`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePlaceADirectoryNames('/home/u/missing/../run') // '/var/home/u/run'
/// ```
fn the_place_a_directory_names(directory: &Path) -> PathBuf {
    // The whole spelling resolving is the ordinary case, and the file system itself decides it.
    if let Ok(resolved) = std::fs::canonicalize(directory) {
        return resolved;
    }
    // What is spelled above the deepest directory that exists, one entry per component and in the order spelled.
    let spelled: Vec<Component<'_>> = directory.components().collect();
    for take in (0..spelled.len()).rev() {
        let mut above: PathBuf = PathBuf::new();
        for component in &spelled[..take] {
            above.push(component.as_os_str());
        }
        // A relative spelling has its deepest directory in this process's working directory, which is what a relative
        // socket path is resolved against as well.
        let deepest: &Path = if above.as_os_str().is_empty() { Path::new(".") } else { above.as_path() };
        let Ok(resolved) = std::fs::canonicalize(deepest) else { continue };
        let mut built: PathBuf = resolved;
        for component in &spelled[take..] {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    let _popped = built.pop();
                }
                Component::Normal(name) => built.push(name),
                Component::RootDir | Component::Prefix(_) => {}
            }
        }
        return built;
    }
    // Nothing above the root resolves either, which a path from the root always does: the spelling stands as given.
    return directory.to_path_buf();
}

/// What:
///  The abstract name of one worktree's service in one runtime directory,
/// without the `@` a socket unit's `ListenStream=` puts before it.
/// Why:
///   The name carries the account,
/// so two accounts that share a runtime directory path never bind one name,
/// and the digest of the runtime directory,
/// so two runtime directories are two address spaces the way two socket files are today.
/// A service the manager starts and a client that connects compute the same name from the same three parts,
/// and neither can spell it past the 107 bytes an abstract name may take:
/// the prefix is the service's own name,
/// the user id takes at most ten digits,
/// the digest takes `DIGEST_DIGITS_IN_AN_ABSTRACT_NAME`,
/// and an instance is 32 digits (`INSTANCE_LENGTH`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// abstractNameOf('/home/u/temp/agent/some-long-scratch-folder/run', '3f9a…') // 'monochromatic-git-policy-cli/1000/0123456789abcdef/3f9a…'
/// ```
pub fn abstract_name_of(runtime_dir: &Path, instance: &str) -> String {
    return format!("{}/{instance}", abstract_name_prefix_of(runtime_dir));
}

/// What:
///  The part of an abstract name that is the same for every instance of one runtime directory: the service's own name,
///  the account, and the digest of the place the runtime directory names.
/// Why:
///   A unit file names the instance with `%i` and a client names it with the instance's text,
///  so the rest of the name is made in one place for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// abstractNamePrefixOf('/run/user/1000') // 'monochromatic-git-policy-cli/1000/0123456789abcdef'
/// ```
fn abstract_name_prefix_of(runtime_dir: &Path) -> String {
    let place: PathBuf = the_place_a_directory_names(runtime_dir);
    let digest: String = digest_of_bytes(place.as_os_str().as_bytes());
    let account: u32 = rustix::process::geteuid().as_raw();
    let kept: &str = digest.get(..DIGEST_DIGITS_IN_AN_ABSTRACT_NAME).unwrap_or(digest.as_str());
    return format!("{SOCKET_DIRECTORY}/{account}/{kept}");
}

/// What:
///  The address one worktree's service listens at, or would:
/// the socket file under the runtime directory when that path fits one socket address,
/// and an abstract name when it does not.
/// Why:
///   A runtime directory that can carry a path keeps it,
/// so every account whose runtime directory is short enough is served exactly as it was,
/// and the abstract name is the one address left for the rest.
/// A system without an abstract namespace has nothing to fall back to,
/// so the path form is named there and the caller refuses it if it is too long
/// (`service_provision_refusals_before_the_wait.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// socketAddressOf('/run/user/1000', '3f9a…') // { path: '/run/user/1000/monochromatic-git-policy-cli/3f9a….sock' }
/// ```
pub fn socket_address_of(runtime_dir: &Path, instance: &str) -> Bind {
    let path: PathBuf = socket_path(runtime_dir, instance);
    if !AN_ABSTRACT_NAME_IS_AVAILABLE || path.as_os_str().len() <= LONGEST_SOCKET_PATH {
        return Bind::Path(path);
    }
    return Bind::Abstract(abstract_name_of(runtime_dir, instance));
}

/// What:
///  The value the socket unit's `ListenStream=` takes for this runtime directory,
/// with the instance still to be filled in.
/// Why:
///   The manager binds the socket,
/// not the wrapper,
/// so the address has to be one a unit file can name:
/// an absolute path with `%t` for the manager's runtime directory,
/// or `@` and an abstract name.
/// Both forms name the instance with `%i`,
/// and which form this is was decided by `socket_address_of` for the same runtime directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listenStreamOf('/run/user/1000', '3f9a…') // '%t/monochromatic-git-policy-cli/%i.sock'
/// ```
pub fn listen_stream_of(runtime_dir: &Path, instance: &str) -> String {
    if matches!(socket_address_of(runtime_dir, instance), Bind::Abstract(_name)) {
        return format!("@{}/%i", abstract_name_prefix_of(runtime_dir));
    }
    return format!("%t/{SOCKET_DIRECTORY}/%i.sock");
}

/// What:
///  Whether the address has a socket on it right now:
/// a file at the path,
/// or a listening socket of that abstract name.
/// Why:
///   Provisioning counts a socket that is there as one step of an installation that is in place,
/// and the route hands a socket that is there to the command when nothing could set a service up.
/// A path is one `stat`;
/// an abstract name has no file,
/// and the table of unix sockets is the only way to see one without connecting,
///  because a connection would start the service the caller is deciding whether to start.
/// A table that cannot be read counts as "there",
/// as an unreadable table counts as a listener for an earlier release's socket:
/// the commit goes on to a client that waits with its own bound rather than fail here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theAddressIsThere({ abstractName }) // false when no socket of that name listens
/// ```
pub fn the_address_is_there(address: &Bind) -> bool {
    return the_address_is_in(Path::new("/proc"), address);
}

/// `the_address_is_there` with the table of unix sockets to read.
pub fn the_address_is_in(proc_root: &Path, address: &Bind) -> bool {
    return match address {
        Bind::Path(path) => path.exists(),
        Bind::Abstract(name) => listening_sockets_exactly(proc_root, &[abstract_name_column(name)]) != ListeningSockets::None,
        Bind::Inherited => false,
    };
}

/// The column the table of unix sockets prints for an abstract socket of this name:
/// `@` and the name,
/// as the kernel prints it (`sockaddr_pretty` of systemd's `socket-util.c`,
/// and `unix_seq_show` of the kernel's `net/unix/af_unix.c`).
pub fn abstract_name_column(name: &str) -> Vec<u8> {
    return format!("@{name}").into_bytes();
}

/// What:
///  The abstract names this runtime directory's services hold right now,
/// as the table of unix sockets prints them,
/// without the `@` that starts every one.
/// Why:
///   A status command outside a worktree counts the account's services that answer,
/// and under a long runtime directory a service has no socket file to be found by
/// (`any_service_greets` in `service_route.rs`).
/// The table is where a name is visible without connecting to it,
/// and the prefix is the account and the digest of the place,
/// so neither another account's services nor another runtime directory's are counted.
/// The names are printed and read back as the text they are,
/// because the rest of the wrapper names a service with that text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listeningNamesOf('/home/u/run') // ['monochromatic-git-policy-cli/1000/0123456789abcdef/3f9a…']
/// ```
pub fn listening_names_of(runtime_dir: &Path, proc_root: &Path) -> Vec<String> {
    let prefix: String = format!("@{}/", abstract_name_prefix_of(runtime_dir));
    return crate::earlier_unix_sockets_that_listen::listening_names_starting_with(proc_root, prefix.as_bytes())
        .into_iter()
        .map(|column| { return String::from(column.trim_start_matches('@')); })
        .collect();
}

/// Address controls stay out of the release executable.
#[cfg(test)]
#[path = "service_address_of_a_worktree_tests.rs"]
mod tests;
