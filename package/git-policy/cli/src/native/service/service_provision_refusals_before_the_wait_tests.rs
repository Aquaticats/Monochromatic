//! What:
//!  Controls for the refusals provisioning makes before the client waits for a start:
//! a socket address this system can only name as a path when that path is too long,
//! a service the manager made listen where this command does not look,
//! and the operating system's words without its error number.
//! Why:
//!  Each of these spent two minutes of waiting before the command said only that the service did not start,
//! so each control reads the same two facts the person reads:
//! the command ends at once,
//! and its message names the input that has to change.
//! Every check also has a control that proves it lets a working arrangement through,
//! because a refusal that is wrong fails a commit that would have landed.
//! A runtime directory too long for a path is served at an abstract name instead,
//! so the controls here hold the two forms against each other: what the unit names,
//! what the manager answers,
//! and what two spellings of one directory make.

use super::*;
use crate::service_address_of_a_worktree::{abstract_name_of, socket_address_of, the_address_is_there};
use crate::service_instance::{LONGEST_SOCKET_PATH, longest_runtime_directory, socket_path, socket_path_suffix_bytes};
use crate::service_provision::{in_place, provision_with};
use crate::service_provision_host_and_manager::Manager;
use crate::service_server::Bind;
use crate::systemd_dbus::CallError;
use std::path::{Path, PathBuf};

/// The instance a fixture's worktree has.
const INSTANCE: &str = "0123456789abcdef0123456789abcdef";

/// How long a refusal may take:
/// far below the start ceiling it replaces (one minute,
/// twice with the reload),
/// and above any reason for a slow check.
const AT_ONCE: std::time::Duration = std::time::Duration::from_secs(2);

/// A manager that records the calls it is asked for and answers the question about a unit's addresses as a control
///  sets it to.
#[derive(Default)]
struct Addressing {
    calls: Vec<String>,
    /// The addresses the manager says a unit listens at;
    /// nothing when it holds no such unit.
    addresses: Option<Vec<String>>,
    /// What the question is answered with instead of addresses,
    /// when it is made to fail.
    refusal: Option<CallError>,
}

impl Manager for Addressing {
    fn within(&mut self, wait: std::time::Duration) {
        self.calls.push(format!("within {}", wait.as_secs()));
    }

    fn reload(&mut self) -> Result<(), CallError> {
        self.calls.push(String::from("reload"));
        return Ok(());
    }

    fn reset_failed(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.push(format!("reset-failed {unit}"));
        return Ok(());
    }

    fn start(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.push(format!("start {unit}"));
        return Ok(());
    }

    fn stop(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.push(format!("stop {unit}"));
        return Ok(());
    }

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.push(format!("stop-alone {unit}"));
        return Ok(());
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        self.calls.push(format!("state {unit}"));
        // An earlier release's unit for this worktree never runs in these controls.
        return Ok(if unit.contains("cli-git@") { None } else { Some(String::from("inactive")) });
    }

    fn listen(&mut self, unit: &str) -> Result<Option<Vec<String>>, CallError> {
        return match self.refusal.clone() {
            Some(failure) => Err(failure),
            None => {
                let _asked: &str = unit;
                Ok(self.addresses.clone())
            }
        };
    }
}

/// What:
///  The directory of one test process's runtime directories for these controls,
/// short enough to leave room for a socket name in what it holds.
/// Why:
///   One socket address holds 107 bytes,
/// the runtime directory may take 40 of them,
/// and the fixture root under the build directory is far longer.
/// The process number keeps two runs of the gate from sharing one,
/// and each control removes its own runtime directory (never this one,
/// which a control running beside it still writes in).
fn short_parent() -> PathBuf {
    let directory: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory"))
        .join("temp/agent")
        .join(format!("pr{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("the scratch directory");
    return directory;
}

/// What:
///  A runtime directory of exactly `length` bytes,
/// standing in this control's scratch directory.
/// Why:
///   The room the path of one socket needs is what these controls move past,
/// so the directory is built to the byte and the length is asserted rather than assumed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runtimeDirectoryOfLength(41, 'past') // '/home/user/temp/agent/pr4242/past/xxxxxxxxx'
/// ```
fn runtime_directory_of_length(length: usize, tag: &str) -> PathBuf {
    let parent: PathBuf = short_parent();
    let room: usize = length.checked_sub(parent.as_os_str().len() + 1).expect("a name long enough to build");
    // The name holds the tag and then padding, never nothing: a path that ends in the separator has one byte less than it
    //  says it has when anything is joined to it. A runtime directory of thirty-nine bytes leaves a handful of bytes past this
    //  process's own scratch folder, and a longer process number narrows that, so a tag that does not fit is replaced by a
    //  digest of itself: two controls still get two directories, whichever number this process runs as.
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

/// What:
///  A runtime directory one byte shorter than the longest,
/// which these controls can use and whose socket path still fits one address.
/// Why:
///   These controls are about where the manager made the service listen,
/// not about the room the address needs,
/// so their runtime directory is inside the longest by a byte and the address refuses nothing.
fn a_runtime_directory_with_room(tag: &str) -> PathBuf {
    return runtime_directory_of_length(longest_runtime_directory(INSTANCE) - 1, tag);
}

/// What:
///  The bytes of the name of a runtime directory whose own spelling and its spelling through one `..` both fit
///  inside one socket address.
/// Why:
///   The spelling through a relative part holds the name twice,
///  once as the directory and once after `..`,
/// and `..` with the two slashes around it takes four bytes between the copies.
/// One byte of the room goes to the separator after the parent and one to the separator after `..`,
/// and what is left is halved,
/// so the two copies together never pass the longest runtime directory an address can carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nameLengthToSpell(30) // the name of a runtime directory under a parent of 30 bytes
/// ```
fn name_length_to_spell(parent: usize) -> usize {
    let longest: usize = longest_runtime_directory(INSTANCE);
    let room: usize = longest - parent - 1 - 4;
    return room / 2;
}

/// What:
///  A runtime directory whose name leaves room for the other spellings of it,
/// inside the longest runtime directory a socket address can carry.
/// Why:
///   The check about the room an address needs cannot be what refuses a control of spellings,
/// so the name is measured by `name_length_to_spell`,
/// which keeps both spellings inside one address.
fn a_runtime_directory_to_spell(tag: &str) -> PathBuf {
    let parent: PathBuf = short_parent();
    let name_length: usize = name_length_to_spell(parent.as_os_str().len());
    assert!(name_length > tag.len(), "the tag {tag:?} leaves room to pad a name of {name_length} bytes");
    return runtime_directory_of_length(parent.as_os_str().len() + 1 + name_length, tag);
}

/// What:
///  Every parent length from a scratch directory's to the longest leaves both spellings inside one socket address.
/// Why:
///   The room is halved,
/// and an odd room halves down by one,
/// so a construction that assumed three bytes between the copies of the name spent the room exactly for every second
/// parent length and produced an address one byte past what the kernel carries:
/// `two_spellings_of_one_runtime_directory_are_one_service` failed whenever the test process's number of digits made
/// the parent even.
/// Measured 2026-11-11T10:05-05:00 with pid 919300:
/// the dotted address took 108 bytes,
/// and a run whose pid made the parent odd passed.
#[test]
fn the_two_spellings_of_a_runtime_directory_fit_one_socket_address_at_every_parent_length() {
    let longest: usize = longest_runtime_directory(INSTANCE);
    let suffix: usize = socket_path_suffix_bytes(INSTANCE);
    // Up to the longest parent whose halved room still leaves a name of two bytes, which both parities cover.
    for parent in 10..=(longest - 9) {
        let name: usize = name_length_to_spell(parent);
        assert!(name > 1, "a parent of {parent} bytes leaves a name of {name} bytes");
        // The name, the separator after the parent, `..` between the copies, and the name again.
        let dotted: usize = parent + 1 + name + 4 + name;
        assert!(
            dotted + suffix <= LONGEST_SOCKET_PATH,
            "a parent of {parent} bytes spells the directory in {dotted} bytes, which with the {suffix} of a socket path takes more than the {} one address carries",
            LONGEST_SOCKET_PATH
        );
    }
}

/// What:
///  A symbolic link beside `runtime` that resolves to it.
/// Why:
///  A person's runtime directory may be reached through a link,
/// which this machine has in `/home` reaching `/var/home`,
/// and the file system resolves the link while the texts stay different.
fn a_link_beside(runtime: &Path, name: &str) -> PathBuf {
    let link: PathBuf = runtime.parent().expect("a runtime directory stands in a folder").join(name);
    std::os::unix::fs::symlink(runtime, &link).expect("a link to the runtime directory");
    crate::test_support::track_for_this_thread(link.as_path());
    return link;
}

/// What:
///  Another spelling of the directory `runtime` names,
/// `rounds` times over:
/// the directory,
/// `..`,
/// and its own name again.
/// Why:
///  The file system walks every `..` back and reaches the same directory,
/// while each round adds bytes to the text,
/// so a control can spell one directory at the length its case needs.
fn a_spelling_through_relative_parts(runtime: &Path, rounds: usize) -> PathBuf {
    let name: &std::ffi::OsStr = runtime.file_name().expect("a runtime directory has a name");
    let mut spelled: PathBuf = runtime.to_path_buf();
    for _round in 0..rounds {
        spelled = spelled.join("..").join(name);
    }
    return spelled;
}

/// What:
///  A runtime directory one byte past the longest a path may carry is served at an abstract name,
///  the socket unit names that same address with `%i` for the instance,
///  and the manager's answer about it is not a service out of reach.
/// Why:
///  The kernel refuses a socket path of 108 bytes,
/// so before this form existed the manager bound nothing and the commit failed,
///  where real Git made the commit (owner's rule, "Commits should never fail").
/// The unit is what the manager binds,
/// and the answer is what this command compares with the address it connects to,
/// so both are read here.
#[test]
#[cfg(target_os = "linux")]
fn a_runtime_directory_past_the_longest_is_served_at_an_abstract_name() {
    let longest: usize = longest_runtime_directory(INSTANCE);
    let too_long: PathBuf = runtime_directory_of_length(longest + 1, "abstract");
    let Bind::Abstract(name) = socket_address_of(too_long.as_path(), INSTANCE) else {
        panic!("a runtime directory past the longest is an abstract name")
    };
    let mut manager: Addressing = Addressing { addresses: Some(vec![format!("@{name}")]), ..Addressing::default() };
    provision_with(&host_with(too_long.clone()), &mut manager).expect("a runtime directory past the longest is set up");
    let unit: String = std::fs::read_to_string(too_long.join("systemd/user/monochromatic-git-policy-cli@.socket")).expect("the socket unit was written");
    let line: String = format!("ListenStream=@{}", name.replace(INSTANCE, "%i"));
    assert!(unit.contains(line.as_str()), "the unit names the abstract address, with the instance to the manager's `%i`: {unit}");
    assert!(!unit.contains("%t/"), "the unit names no path under a runtime directory: {unit}");
    assert!(manager.calls.iter().any(|call| { return call.starts_with("start "); }), "the socket was started: {:?}", manager.calls);
}

/// What:
///  A service the manager made listen at a socket file is out of reach of a command whose runtime directory is too
///  long for a path,
///  and the refusal names the abstract address this command waits at.
/// Why:
///   The two forms are two addresses,
/// and the commit must not wait a start for the one the manager never made.
/// The message names both,
/// which is what tells a person that `XDG_RUNTIME_DIR` is the setting that decides it.
#[test]
#[cfg(target_os = "linux")]
fn a_service_the_manager_made_at_a_path_is_out_of_reach_of_a_command_that_waits_at_a_name() {
    let runtime: PathBuf = runtime_directory_of_length(longest_runtime_directory(INSTANCE) + 1, "name-and-path");
    let Bind::Abstract(name) = socket_address_of(runtime.as_path(), INSTANCE) else { panic!("an abstract name") };
    let made: String = socket_path(Path::new("/run/user/1000"), INSTANCE).display().to_string();
    let mut manager: Addressing = Addressing { addresses: Some(vec![made.clone()]), ..Addressing::default() };
    let began: std::time::Instant = std::time::Instant::now();
    let refused: ClientFailure = provision_with(&host_with(runtime.clone()), &mut manager).expect_err("the manager made another address");
    let took: std::time::Duration = began.elapsed();
    assert!(took < AT_ONCE, "the refusal takes {took:?}, not a wait");
    assert_eq!(
        refused,
        ClientFailure::ServiceElsewhere {
            made: made.clone(),
            looked_for: format!("the name {name}"),
            manager_runtime: String::from("/run/user/1000"),
            client_runtime: runtime.display().to_string(),
        }
    );
    let said: String = message_of(&refused);
    for part in [made.as_str(), name.as_str(), "XDG_RUNTIME_DIR"] {
        assert!(said.contains(part), "the message names {part:?}: {said}");
    }
    assert!(!manager.calls.iter().any(|call| { return call.starts_with("start "); }), "nothing was started: {:?}", manager.calls);
}

/// What:
///  The socket the manager made,
/// bound so the installation is whole.
/// Why:
///  `in_place` looks for the address,
/// and a control of the address a commit connects to needs the socket a real manager would have made.
fn the_socket_the_manager_made(runtime: &Path) {
    use std::os::linux::net::SocketAddrExt;
    if let Bind::Abstract(name) = socket_address_of(runtime, INSTANCE) {
        let address = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("the name fits");
        let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind_addr(&address).expect("the abstract socket the manager made");
        // The listener stays for the rest of the test process: the table of unix sockets is what `in_place` reads, and a
        //  control binds a name of its own, so nothing else can take it.
        std::mem::forget(listener);
        return;
    }
    let socket: PathBuf = socket_path(runtime, INSTANCE);
    std::fs::create_dir_all(socket.parent().expect("the folder of the sockets")).expect("the folder of the sockets");
    let _listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind(&socket).expect("the socket the manager made");
    crate::test_support::track_for_this_thread(socket.as_path());
}

/// A host whose runtime directory is `runtime`.
fn host_with(runtime: PathBuf) -> ProvisionHost {
    return ProvisionHost {
        bus: runtime.join("bus"),
        runtime_dir: runtime,
        user_unit_dir: None,
        service_binary: PathBuf::from("/opt/monochromatic-git-policy-cli/monochromatic-git-policy-cli"),
        git: PathBuf::from("/usr/bin/git"),
        instance: String::from(INSTANCE),
        git_dir: short_parent().join("repository/.git/worktrees/topic"),
        retirement_wait: std::time::Duration::from_millis(400),
    };
}

/// What a failure says,
/// for a control that reads the message as the person reads it.
fn message_of(failure: &ClientFailure) -> String {
    return crate::service_messages::render(failure);
}

/// What:
///  The address form is decided by the runtime directory alone:
///  the longest that a path can carry keeps its socket file,
///  and one byte past it is an abstract name,
///  so no commit refuses over a length and none waits for a path no address can carry.
/// Why:
///   This is the boundary the kernel has,
///  and a form chosen a byte early or late would move the socket of an ordinary session
///  or leave a long one to fail as it did.
#[test]
#[cfg(target_os = "linux")]
fn the_longest_runtime_directory_that_keeps_a_path_and_the_one_past_it_that_does_not() {
    let longest: usize = longest_runtime_directory(INSTANCE);
    let at_the_longest: PathBuf = runtime_directory_of_length(longest, "boundary");
    assert!(
        matches!(socket_address_of(at_the_longest.as_path(), INSTANCE), Bind::Path(_)),
        "a runtime directory of exactly {longest} bytes keeps the socket file"
    );
    let past: PathBuf = runtime_directory_of_length(longest + 1, "past-boundary");
    assert!(
        matches!(socket_address_of(past.as_path(), INSTANCE), Bind::Abstract(_)),
        "one byte past it is an abstract name"
    );
}

/// What:
///  A runtime directory of exactly the longest is used,
/// and its socket path fills one address to its last byte.
/// Why:
///   A refusal one byte too eager would fail every commit whose runtime directory is the longest that works,
/// and this is the only control that holds the check to the boundary the kernel allows.
#[test]
fn a_runtime_directory_of_exactly_the_longest_has_room_to_the_last_byte() {
    let longest: usize = longest_runtime_directory(INSTANCE);
    let runtime: PathBuf = runtime_directory_of_length(longest, "longest");
    assert_eq!(socket_path(runtime.as_path(), INSTANCE).as_os_str().len(), LONGEST_SOCKET_PATH, "the address fills the room there is");
    let mut manager: Addressing = Addressing { addresses: Some(vec![socket_path(runtime.as_path(), INSTANCE).display().to_string()]), ..Addressing::default() };
    provision_with(&host_with(runtime.clone()), &mut manager).expect("a service with room to the last byte is set up");
    assert!(runtime.join("systemd/user").exists(), "the units were written");
    assert!(manager.calls.iter().any(|call| { return call.starts_with("start "); }), "the socket was started: {:?}", manager.calls);
}

/// What:
///  A service the manager made listen at another address than this command waits at is refused at once,
/// with both addresses,
/// both runtime directories and the remedy.
/// Why:
///   The unit names its address with `%t`,
/// which the manager reads as its own runtime directory,
/// so a command given another one (a unit of its own,
/// a container,
/// a script with its own `XDG_RUNTIME_DIR`)
/// waited two minutes for a service that was running and healthy,
/// and the message said nothing about the setting that caused it.
#[test]
fn a_service_the_manager_made_elsewhere_is_refused_with_both_addresses() {
    let runtime: PathBuf = a_runtime_directory_with_room("there");
    let made: PathBuf = socket_path(Path::new("/run/user/1000"), INSTANCE);
    let mut manager: Addressing = Addressing { addresses: Some(vec![made.display().to_string()]), ..Addressing::default() };
    let began: std::time::Instant = std::time::Instant::now();
    let refused: ClientFailure = provision_with(&host_with(runtime.clone()), &mut manager).expect_err("the service answers where this command does not look");
    let took: std::time::Duration = began.elapsed();
    assert!(took < AT_ONCE, "the refusal takes {took:?}, not a wait");
    assert_eq!(
        refused,
        ClientFailure::ServiceElsewhere {
            made: made.display().to_string(),
            looked_for: socket_path(runtime.as_path(), INSTANCE).display().to_string(),
            manager_runtime: String::from("/run/user/1000"),
            client_runtime: runtime.display().to_string(),
        }
    );
    let said: String = message_of(&refused);
    for part in [made.display().to_string(), socket_path(runtime.as_path(), INSTANCE).display().to_string(), String::from("/run/user/1000"), String::from("XDG_RUNTIME_DIR")] {
        assert!(said.contains(part.as_str()), "the message names {part:?}: {said}");
    }
    // The socket the manager already holds is another runtime directory's service for this worktree, and this command starts
    //  nothing: a unit started here would listen for a worktree no command of this runtime directory ever looks at.
    assert!(
        !manager.calls.iter().any(|call| { return call.starts_with("start "); }),
        "nothing was started in a manager that serves another runtime directory: {:?}",
        manager.calls
    );
}

/// What:
///  A service the manager made exactly where this command waits is used,
/// and the commit goes on.
/// Why:
///   This is the arrangement every healthy commit has,
/// and a check that refused it would fail all of them.
#[test]
fn a_service_the_manager_made_where_this_command_waits_is_used() {
    let runtime: PathBuf = a_runtime_directory_with_room("where");
    let mut manager: Addressing = Addressing { addresses: Some(vec![socket_path(runtime.as_path(), INSTANCE).display().to_string()]), ..Addressing::default() };
    provision_with(&host_with(runtime.clone()), &mut manager).expect("the service answers where this command looks");
}

/// What:
///  Two spellings of one runtime directory are one service,
/// and the commit goes on.
/// Why:
///   The address in the socket unit is `%t`,
/// the manager's own spelling of its runtime directory,
/// while the address a command connects to is built from the spelling of `XDG_RUNTIME_DIR` it was given.
/// A link to the same directory,
/// a `/run/user/1000/../1000` part,
/// and a runtime directory already reached through a link,
/// which this account has in `/home` reaching `/var/home` and in `/var/run` reaching `/run`,
/// all reach the one socket the manager made,
/// so a comparison of the two texts alone refused a service that answers on the very socket this command connects to.
/// Worse,
/// the refusal came before the record of what the manager was told,
/// so every later commit was refused the same way.
#[test]
fn two_spellings_of_one_runtime_directory_are_one_service() {
    let runtime: PathBuf = a_runtime_directory_to_spell("a");
    let made: String = socket_path(runtime.as_path(), INSTANCE).display().to_string();
    the_socket_the_manager_made(runtime.as_path());
    let linked: PathBuf = a_link_beside(runtime.as_path(), "lnk");
    let dotted: PathBuf = a_spelling_through_relative_parts(runtime.as_path(), 1);
    for (tag, spelled) in [("a link to it", linked), ("a relative part in it", dotted)] {
        assert_eq!(
            std::fs::canonicalize(&spelled).expect("the spelling resolves"),
            std::fs::canonicalize(&runtime).expect("the runtime directory"),
            "{tag}: the two spellings reach one directory",
        );
        assert_ne!(spelled, runtime, "{tag}: the spellings are different texts");
        let mut manager: Addressing = Addressing { addresses: Some(vec![made.clone()]), ..Addressing::default() };
        provision_with(&host_with(spelled.clone()), &mut manager).unwrap_or_else(|failure| {
            panic!("{tag}: the service answers on the socket this command connects to, and provisioning returned {failure:?}")
        });
        assert!(manager.calls.iter().any(|call| { return call.starts_with("start "); }), "{tag}: the socket was started: {:?}", manager.calls);
        assert!(in_place(&host_with(spelled.clone())), "{tag}: the next commit finds the installation this one made");
    }
}

/// What:
///  A runtime directory too long for a path is one service however its spelling is written,
///  and the next commit finds the installation the first one made.
/// Why:
///   The abstract name is made from the place the runtime directory names,
/// never from the text of the spelling that reached it,
/// which is what the kernel does for a socket path.
/// A `..` part makes the text longer than one address can carry,
/// and a name made from that text would give the manager one address and the client another,
///  which is the ten-second wait this change ends.
#[test]
#[cfg(target_os = "linux")]
fn a_runtime_directory_too_long_for_a_path_is_one_service_however_it_is_spelled() {
    let runtime: PathBuf = runtime_directory_of_length(longest_runtime_directory(INSTANCE) + 1, "spelled");
    let dotted: PathBuf = a_spelling_through_relative_parts(runtime.as_path(), 2);
    let linked: PathBuf = a_link_beside(runtime.as_path(), "link-to-a-long-runtime-directory");
    assert!(socket_path(dotted.as_path(), INSTANCE).as_os_str().len() > LONGEST_SOCKET_PATH, "the longer spelling is no path any address carries");
    let name: String = abstract_name_of(runtime.as_path(), INSTANCE);
    let mut manager: Addressing = Addressing { addresses: Some(vec![format!("@{name}")]), ..Addressing::default() };
    provision_with(&host_with(runtime.clone()), &mut manager).expect("the units are written and the manager was told");
    the_socket_the_manager_made(runtime.as_path());
    assert!(in_place(&host_with(runtime.clone())), "the installation looks current under its own spelling");
    for spelled in [dotted, linked] {
        assert_eq!(abstract_name_of(spelled.as_path(), INSTANCE), name, "one place, one name, whatever the spelling is");
        assert!(
            the_address_is_there(&socket_address_of(spelled.as_path(), INSTANCE)),
            "the socket the manager made before answers this spelling too: {} is reached through it",
            spelled.display()
        );
        let mut spelled_manager: Addressing = Addressing { addresses: Some(vec![format!("@{name}")]), ..Addressing::default() };
        provision_with(&host_with(spelled.clone()), &mut spelled_manager).expect("the same installation under another spelling is used, not refused");
        // The units name the program's link by the runtime directory's spelling, so a spelling that differs from the one the
        //  record was written under has the units written again and the manager asked to reload them: one write and one
        //  reload, and never a second service.
        assert!(
            spelled_manager.calls.iter().any(|call| { return call.starts_with("start "); }),
            "the same socket unit is started, not another: {:?}",
            spelled_manager.calls
        );
    }
}

/// What:
///  A manager that cannot say where a unit listens proves nothing,
/// so the commit goes on as it did before there was a check.
/// Why:
///   The address is the manager's answer,
/// and an answer that could not be read (an older manager,
/// a property of another shape)
/// is not evidence that the service is out of reach;
/// refusing on it would fail commits that work.
#[test]
fn a_manager_that_cannot_say_where_a_unit_listens_is_no_proof() {
    let runtime: PathBuf = a_runtime_directory_with_room("quiet");
    let mut manager: Addressing = Addressing { refusal: Some(CallError::Malformed(String::from("no address in the answer"))), ..Addressing::default() };
    provision_with(&host_with(runtime), &mut manager).expect("an answer that cannot be read is not a refusal");
    let mut absent: Addressing = Addressing::default();
    let second: PathBuf = a_runtime_directory_with_room("none");
    provision_with(&host_with(second), &mut absent).expect("a manager that holds no unit yet is not a refusal");
}

/// What:
///  The operating system's words are kept and its error number is not.
/// Why:
///   The number is implementation vocabulary that the wrapper's message rules keep from a person,
/// and the words before it are what the person acts on.
#[test]
fn the_words_of_a_failed_file_operation_lose_only_the_error_number() {
    let denied: std::io::Error = std::io::Error::from_raw_os_error(libc::EACCES);
    assert_eq!(in_plain_words(&denied), "Permission denied", "the words stay and the number does not");
    let full: std::io::Error = std::io::Error::from_raw_os_error(libc::ENOSPC);
    assert_eq!(in_plain_words(&full), "No space left on device");
    for error in [denied, full] {
        assert!(!in_plain_words(&error).contains("os error"), "no error number reaches the person");
    }
    let ours: std::io::Error = std::io::Error::other("the record could not be written");
    assert_eq!(in_plain_words(&ours), "the record could not be written", "an error of this program is already in plain words");
}

/// What:
///  The runtime directory of a service's address is the one the manager read `%t` as.
/// Why:
///   The message points `XDG_RUNTIME_DIR` at it,
/// so a wrong derivation would send the person somewhere that cannot work.
#[test]
fn the_runtime_directory_of_an_address_is_the_one_it_stands_in() {
    let address: PathBuf = socket_path(Path::new("/run/user/1000"), INSTANCE);
    assert_eq!(runtime_directory_of_an_address(address.as_path()), PathBuf::from("/run/user/1000"));
    assert_eq!(runtime_directory_of_an_address(Path::new("/run/a/b/c")), PathBuf::from("/run/a"));
    assert_eq!(runtime_directory_of_an_address(Path::new("/only")), PathBuf::from("/only"), "an address with no folder of its own is named as it is");
}

/// What:
///  The bytes a socket path adds to its runtime directory are the ones `socket_path` adds.
/// Why:
///   The longest runtime directory a person is told comes from that number,
/// so a change to the shape of the path that left it behind would tell people the wrong length.
#[test]
fn the_bytes_a_socket_path_adds_are_the_ones_it_really_adds() {
    for runtime in [PathBuf::from("/r"), PathBuf::from("/run/user/1000")] {
        for instance in [INSTANCE, "abc"] {
            let built: PathBuf = socket_path(runtime.as_path(), instance);
            assert_eq!(built.as_os_str().len(), runtime.as_os_str().len() + socket_path_suffix_bytes(instance));
        }
    }
    assert_eq!(longest_runtime_directory(INSTANCE) + socket_path_suffix_bytes(INSTANCE), LONGEST_SOCKET_PATH);
}

/// What:
///  A runtime directory no spelling can make — a path through a file, a path through a link that points nowhere — stops
///  provisioning at once and names the setting, rather than waiting for a start that cannot come.
/// Why:
///   Such a spelling stands in no place a service could listen at,
/// so there is no address to bind and no service to be had;
/// the address function still gives it one name before and after
/// (`a_spelling_that_cannot_be_resolved_still_gives_one_name` in `service_address_of_a_worktree_tests.rs`),
/// and what is left is that the commit says what to change and does not wait.
/// `SPEC.md`, "Machines with no way to reach the service", is the rule: the service is the only writer, so a command
/// that cannot reach it changes nothing and stops, with the input and every way to change it named.
#[test]
#[cfg(target_os = "linux")]
fn a_runtime_directory_that_cannot_be_made_stops_at_once_and_names_it() {
    let parent: PathBuf = short_parent();
    // Long enough that only a name can carry the service's address, which is the branch a long runtime directory takes.
    let through_a_file: PathBuf = parent.join("a-file-with-room").join("run");
    std::fs::write(parent.join("a-file-with-room"), b"not a directory\n").expect("a file where a directory has to be");
    let through_a_link: PathBuf = parent.join("dangling-link").join("run");
    std::os::unix::fs::symlink(parent.join("nowhere"), parent.join("dangling-link")).expect("a link that points nowhere");
    crate::test_support::track_for_this_thread(through_a_file.as_path());
    crate::test_support::track_for_this_thread(through_a_link.as_path());
    for (tag, runtime) in [("a file in the path", through_a_file), ("a link that points nowhere", through_a_link)] {
        assert!(matches!(socket_address_of(runtime.as_path(), INSTANCE), Bind::Abstract(_)), "{tag}: the runtime directory is long enough for a name");
        let mut manager: Addressing = Addressing::default();
        let began: std::time::Instant = std::time::Instant::now();
        let refused: ClientFailure = provision_with(&host_with(runtime.clone()), &mut manager).expect_err("{tag}: nothing can make this directory");
        let took: std::time::Duration = began.elapsed();
        assert!(took < AT_ONCE, "{tag}: the refusal takes {took:?}, not a wait");
        let ClientFailure::CouldNotStart(said) = &refused else { panic!("{tag}: a directory that cannot be made is a start that cannot be, not {refused:?}") };
        assert!(said.contains(runtime.to_string_lossy().as_ref()), "{tag}: the message names the directory: {said}");
        assert!(said.contains("XDG_RUNTIME_DIR"), "{tag}: the message names the setting that decides it: {said}");
        assert!(said.contains("make that one writable"), "{tag}: the message names the way out: {said}");
        assert!(!manager.calls.iter().any(|call| { return call.starts_with("start "); }), "{tag}: nothing was started: {:?}", manager.calls);
    }
}
