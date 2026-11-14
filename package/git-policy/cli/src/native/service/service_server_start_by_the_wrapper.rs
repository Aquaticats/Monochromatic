//! What:
//!  Makes a worktree's commit service exist when nothing answers at its address:
//! one command starts it,
//! every other command of the same worktree waits for the one that is starting it.
//! Why:
//!   Without a service manager the first command that needs the service starts it,
//! and two commands that both find nothing must not both start one:
//! the loser would exit on a socket that is already bound,
//! after writing its own log and paying its own start.
//! The coordination is one exclusive publication,
//! which the kernel grants to exactly one process:
//! the starter marker is made whole in a private file and published with one `link`,
//! so a command that reads it always reads a name and never half of one,
//! and it names its owner by process birth identity,
//! so a marker a killed command left is recognised as nobody's and taken over,
//! and no lock is taken and no command ever waits on a lock
//! (`AGENTS.md`, the freedom from locks, and `single_owner_claim.rs` for the same idiom).
//! The socket itself stays the ownership:
//! a leaked marker costs a duplicated start and never two servers,
//! because the takeover of the address is one process's at a time
//! (`service_address_takeover_claim.rs`), so only one bind succeeds.
//! The wait for the greeting has its own bound,
//! so a service that starts and dies is reported rather than waited for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! startTheServer(host); // { greeted } | { anotherCommandStartsIt } | { didNotGreet }
//! ```

use crate::process_identity::{current_birth_identity, process_birth_identity};
use crate::service_client_listen_and_ask_to_retire::{Heard, START_WAIT, connect, listen};
use crate::service_server::Bind;
use crate::service_server_log_file_of_an_instance::{self as log_file, LOG_ENDING};
use crate::service_server_pid_note_of_an_instance::PID_NOTE_ENDING;
use crate::service_server_start_spawns_a_detached_process::{Detached, spawn, text_argument};
use crate::service_server_start_through_the_manager::{MANAGER_START_EXTRA, ManagerStart};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The ending of the marker the one command that starts a worktree's service creates.
pub const STARTER_MARKER_ENDING: &str = ".starting";

/// How often a wait looks for the service's greeting.
pub const GREETING_POLL: Duration = Duration::from_millis(25);

/// The most one connection waits for a greeting.
pub const GREETING_READ: Duration = Duration::from_secs(1);

/// What one call to `start` came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StartAttempt {
    /// The service answered,
    /// either before this call started anything or once it had.
    Greeted,
    /// Another command of this worktree holds the starter marker,
    /// so this one waits for the service that command is starting.
    AnotherCommandStartsIt,
    /// This command started the service and it sent no greeting within `START_WAIT`;
    /// the file the service's output went to.
    DidNotGreet(PathBuf),
}

/// What one start needs.
#[derive(Clone, Debug)]
pub struct ServerToStart {
    /// Where the service listens.
    pub socket: PathBuf,
    /// The worktree's instance name,
    /// which names its files beside the socket.
    pub instance: String,
    /// The folder the service keeps its journal in.
    pub journal: PathBuf,
    /// The real Git the service runs.
    pub git: PathBuf,
    /// The wrapper's own executable,
    /// which is the service's program too.
    pub program: PathBuf,
    /// What:
    ///  The manager this command may ask for a scope,
    ///  and nothing when the base starts the server itself.
    /// Why:
    ///  The decision belongs where the environment is read (`ManagerStart::in_this_session`),
    /// and a start here must not read it again:
    /// a control hands in the answer it pins.
    pub manager: Option<ManagerStart>,
    /// What:
    ///  The runtime directory the environment named,
    ///  and nothing when this program fell back to a folder of its own.
    /// Why:
    ///  Whether a manager may be asked at all is one question (`bus_is_ours` and this name),
    /// and whether a service can be *started* through it is another (its program being there):
    /// retiring an earlier release's units needs the first alone (`service_server_earlier_release_units.rs`).
    pub named_runtime_dir: Option<PathBuf>,
    /// The session bus address the environment gave this command,
    /// which tells whether that bus is this session's.
    pub bus_address: Option<String>,
}

impl ServerToStart {
    /// The folder the instance's files sit in.
    pub fn directory(&self) -> PathBuf {
        return self.socket.parent().map_or_else(
            || {
                return PathBuf::from(".");
            },
            |folder| {
                return folder.to_path_buf();
            },
        );
    }

    /// The file the service's output is appended to.
    pub fn log(&self) -> PathBuf {
        return log_file::path(self.directory().as_path(), self.instance.as_str());
    }

    /// The marker the one starting command holds.
    pub fn starter_marker(&self) -> PathBuf {
        return self.directory().join(format!("{}{STARTER_MARKER_ENDING}", self.instance));
    }

    /// The note the service writes when it listens.
    pub fn pid_note(&self) -> PathBuf {
        return self.directory().join(format!("{}{PID_NOTE_ENDING}", self.instance));
    }

    /// The arguments the service's mode is started with.
    /// The address is given, never inherited:
    /// nothing hands this process a socket.
    pub fn arguments(&self) -> Vec<std::ffi::OsString> {
        // The three words that select the service mode, then the ones it needs to serve this worktree.
        return vec![
            text_argument("monochromatic-git-policy-cli"),
            text_argument("service"),
            text_argument("run"),
            text_argument("--path"),
            crate::service_server_start_spawns_a_detached_process::argument(self.socket.as_path()),
            text_argument("--journal"),
            crate::service_server_start_spawns_a_detached_process::argument(self.journal.as_path()),
            text_argument("--git"),
            crate::service_server_start_spawns_a_detached_process::argument(self.git.as_path()),
            text_argument("--instance"),
            text_argument(self.instance.as_str()),
        ];
    }
}

/// What:
///  Starts the service unless it answers or another command is starting it.
/// Why:
///   A caller's loop connects,
/// calls this when nothing accepted the connection,
/// and connects again:
/// each answer says which of those to do next,
/// and a marker that cannot be taken is not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start(server) // greeted | another-command-starts-it | did-not-greet
/// ```
pub fn start(server: &ServerToStart) -> StartAttempt {
    return start_seamed(server, &mut ask_the_manager);
}

/// The manager's own start, as the caller and a control hand it in.
fn ask_the_manager(manager: &ManagerStart, server: &ServerToStart) -> std::io::Result<Option<i32>> {
    return manager.start(server);
}

/// `start` with the manager's start given by the caller,
/// so a control can hand in a scope that leaves no process number and starts its server late.
pub(crate) fn start_seamed(server: &ServerToStart, manager_start: &mut dyn FnMut(&ManagerStart, &ServerToStart) -> std::io::Result<Option<i32>>) -> StartAttempt {
    if answered(server.socket.as_path()) {
        return StartAttempt::Greeted;
    }
    // What: The folder the address and the marker live in is made before a marker is taken.
    // Why:  A marker cannot be created in a folder that is not there, and a start that could not even take its marker must
    //       not be reported as another command's start.
    if crate::service_socket_directory_only_this_account_may_open::ensure_private(server.directory().as_path()).is_err() {
        return StartAttempt::DidNotGreet(server.log());
    }
    let starter: StarterMarker = match StarterMarker::take(server.starter_marker().as_path()) {
        Ok(Some(taken)) => taken,
        Ok(None) => return StartAttempt::AnotherCommandStartsIt,
        Err(_refused) => return StartAttempt::DidNotGreet(server.log()),
    };
    let _held = starter;
    // The command that held the marker before this one may have finished between the two looks.
    if answered(server.socket.as_path()) {
        return StartAttempt::Greeted;
    }
    // What: Something that accepts a connection but greets nothing is a service that is alive and slow, and no server is started
    //       beside it.
    // Why:  Such a service holds its address (`bind_path` takes over only a socket the kernel refuses), so a server started here
    //       would fail to bind and exit; the command that needs the service waits instead, until the address frees or its own
    //       ceiling passes. This is the wait that keeps a slow recovery or a starved acceptor from failing a commit, and it is
    //       the reason the takeover that used to skip it is gone.
    if accepts(server.socket.as_path()) {
        return StartAttempt::AnotherCommandStartsIt;
    }
    let log: PathBuf = server.log();
    if let Err(_cannot_truncate) = log_file::truncate_when_past_its_cap(log.as_path()) {
        // A log that cannot be shortened still takes the server's lines, and the start is worth making.
    }
    // What: The manager is asked for a scope when it is this account's own and answers a probe in time; otherwise this command
    //       starts the server itself, which is the base.
    // Why:  A scope gives the server a cgroup of its own, and the base is what runs whenever the manager is absent, slow,
    //       refuses or answers with an error, with no message a person has to act on
    //       (`service_server_start_through_the_manager.rs`, and issue 6293 for the gate).
    if let Some(manager) = server.manager.as_ref()
        && manager.probe()
    {
        // One scope is made for this start, never two: the leader is kept so a scope that made nothing answer can be ended.
        let (leader, may_still_start): (Option<i32>, bool) = match manager_start(manager, server) {
            Ok(Some(number)) => (Some(number), false),
            // What: A scope whose number never came may still be starting its own server, so this command waits for that start
            //       before it makes one of its own.
            // Why:  The number is written by the middle process that forks the scope's program; one starved past its own wait
            //       leaves a `systemd-run` that starts a server seconds later, beside the one the base start would make (the
            //       review of `76a08c9fac`, finding 3, part 2). The wait is the enhancement's own extra and never the command's
            //       whole wait, so a manager that failed still leaves the base's start the time it needs.
            Ok(None) => (None, true),
            // A manager that refused made no scope, and nothing of its is coming.
            Err(_refused) => (None, false),
        };
        let bound: Duration = match (leader, may_still_start) {
            (Some(_waiting_for_it), _) => START_WAIT + MANAGER_START_EXTRA,
            (None, true) => MANAGER_START_EXTRA,
            (None, false) => Duration::ZERO,
        };
        let greeted: bool = bound > Duration::ZERO && waits_for_a_greeting(server.socket.as_path(), bound);
        if greeted {
            return StartAttempt::Greeted;
        }
        // What: A scope that made nothing answer is ended, and the base's start is what this command does instead.
        // Why:  A manager that made a scope and then failed costs one extra second and never a commit, and the scope's group is
        //       what this command started, so ending it is its own to do.
        if let Some(process) = leader {
            crate::service_server_start_through_the_manager::end_the_group_of(process);
        }
        if answered(server.socket.as_path()) {
            return StartAttempt::Greeted;
        }
    }
    let detached: Detached = Detached { program: server.program.clone(), arguments: server.arguments(), log: log.clone() };
    if spawn(&detached).is_err() {
        return StartAttempt::DidNotGreet(log);
    }
    return match waits_for_a_greeting(server.socket.as_path(), START_WAIT) {
        true => StartAttempt::Greeted,
        // What: A start that did not greet, while the address still takes a connection, is a service that was already there and
        //       is leaving: the server this command started could not bind and exited, and the caller keeps waiting.
        // Why:  Failing here would fail a commit whose service is about to be free, and the caller's own ceiling bounds the wait.
        false if accepts(server.socket.as_path()) => StartAttempt::AnotherCommandStartsIt,
        false => StartAttempt::DidNotGreet(log),
    };
}

/// What:
///  Whether something accepts a connection at this address,
///  which a listening socket does whether or not its process will greet.
/// Why:
///  It is how a command tells "nothing is bound here" from "a service is bound and not answering yet",
/// and the second is never a reason to bind the name again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// accepts(socket) // connect succeeds
/// ```
pub fn accepts(socket: &Path) -> bool {
    let address: Bind = Bind::Path(socket.to_path_buf());
    return match connect(&address) {
        // The connection was made: a process holds this address.
        Ok(_stream) => true,
        // What: A connection the bound ended is still a process holding the address: its queue is full and it takes no more.
        // Why:  A full queue and a socket nobody holds are different states of the world, and only the refusal says the second:
        //       a server started on the first would fail to bind, and the caller would be told a start that did not greet while a
        //       service was there (`accepts`' callers wait for it).
        Err(error) => error.kind() == std::io::ErrorKind::TimedOut,
    };
}

/// What:
///  Whether something at this address greets a connection in this program's protocol.
/// Why:
///   A service of another release greets too,
/// and that is a service the client may reach and ask to retire;
/// only silence and a closed connection are "nothing serves here".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// answered(socket) // connect + read one line in 1 s
/// ```
pub fn answered(socket: &Path) -> bool {
    let address: Bind = Bind::Path(socket.to_path_buf());
    let Ok(stream) = connect(&address) else { return false };
    let _timeout = stream.set_read_timeout(Some(GREETING_READ));
    let mut reader: BufReader<std::os::unix::net::UnixStream> = BufReader::new(stream);
    return matches!(listen(&mut reader), Heard::Message(crate::service_protocol::ServiceMessage::Welcome { .. }));
}

/// What:
///  Waits, bounded, for the service to greet.
/// Why:
///  The wait has a deadline of its own rather than a count of tries,
/// and each look inside it is bounded too:
///  a connection is made within `CONNECT_CEILING` and a greeting is read within `GREETING_READ`,
///  so one look can cost seconds and never the whole wait.
///  Without that, a look at a service whose queue is full waited for the service to move, and the wait's own
/// deadline sat after a call that never returned (`CONNECT_CEILING`, and the review of `3d5c1a0d8c`).
///  The wait can outlast its bound by at most one look, which is what the caller's own ceiling allows for.
pub fn waits_for_a_greeting(socket: &Path, bound: Duration) -> bool {
    let deadline: Instant = Instant::now() + bound;
    loop {
        if answered(socket) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(GREETING_POLL);
    }
}

/// What:
///  The right of one command to start one worktree's service,
/// held as a file only the kernel can publish once,
///  whole.
/// Why:
///   The file's content names its owner by process birth identity,
/// so a marker whose owner is gone is nobody's,
/// and taking it needs no lock and no wait.
///  It is published complete and never written through its own name:
/// a marker that appears empty to a second command is one that command reads as nobody's,
///  and removes while the first is still starting (the review of `76a08c9fac`, finding 3, part 1).
///  It is also removed only by the command whose own text it holds, for the same reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marker = publishWhole(`${instance}.starting`, `${pid} ${identity}`) ?? takeOverIfOwnerIsGone(path);
/// ```
///
/// `take` gives nothing when another live command holds the marker, and an error when the marker could not be created at all.
struct StarterMarker {
    /// The file to remove when this command's start is over.
    path: PathBuf,
    /// What:
    ///  The exact text this command published into it.
    /// Why:
    ///  The marker is removed only while it still holds this text,
    /// so a start that ends never removes a marker another command holds.
    content: String,
}

/// Distinguishes the private files one process publishes markers through at the same instant.
static MARKER_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A name beside `path` that no other publication can have:
/// the marker's own name, this process, the clock and a counter.
fn a_private_name_beside(path: &Path) -> PathBuf {
    let unique: u64 = MARKER_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|elapsed| { return elapsed.as_nanos(); }).unwrap_or(0);
    let name: String = format!(
        ".{}.{}.{nanos}.{unique}",
        path.file_name().map(std::ffi::OsStr::to_string_lossy).unwrap_or_default(),
        std::process::id()
    );
    return path.with_file_name(name);
}

/// What:
///  Makes the marker whole in a private file and publishes it under its own name with one `link`,
///  which the kernel grants to one process.
/// Why:
///   A reader then sees a marker that names its owner or no marker at all, never a marker that names nobody because its
/// owner has not written yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(private, content); link(private, path); rm(private) // AlreadyExists when another command holds the marker
/// ```
fn publish_a_whole_marker(path: &Path, content: &str, before_the_publish: &mut dyn FnMut()) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let private: PathBuf = a_private_name_beside(path);
    let written: std::io::Result<()> = (|| {
        let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&private)?;
        return file.write_all(content.as_bytes());
    })();
    if let Err(error) = written {
        let _removed: std::io::Result<()> = std::fs::remove_file(&private);
        return Err(error);
    }
    // The call `take` makes where a control may let another command publish its own marker.
    before_the_publish();
    let linked: std::io::Result<()> = std::fs::hard_link(&private, path);
    let _removed: std::io::Result<()> = std::fs::remove_file(&private);
    return linked;
}

/// What one attempt to take a marker another command left ended in.
/// Why:
///  A marker that names nobody is taken away and the publication tried again;
/// one that names a live command is that command's, and this one stops.
enum Taken {
    /// The marker named nobody, and it is gone now.
    NobodyIsNamed,
    /// The marker named a live command and was put back for it.
    ACommandIsNamed,
    /// The marker went between this command's look and its own move of it.
    ItWentAlready,
}

/// What:
///  Moves a marker away and asks, of the file this command now holds alone, whether it names a live command.
/// Why:
///   Asking about the path and then removing the path are two steps:
///  another command may have published its own whole marker between them, and removing that one takes the right to start
///  from a live command.
///  Renaming the marker away is the one step that says which file this command is asking about, and a marker that turns
///  out to name a live command is put back where it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename(path, grave) ?? return 'it went already'; if (alive(grave)) { link(grave, path); return 'a live command names it' } rm(grave); return 'nobody'
/// ```
fn take_away_a_marker_that_names_nobody(path: &Path) -> Taken {
    let grave: PathBuf = a_private_name_beside(path);
    if std::fs::rename(path, &grave).is_err() {
        return Taken::ItWentAlready;
    }
    if holder_is_alive(&grave) {
        // What: The marker this command moved names a live command, so it goes back where it was.
        // Why:  A publication that lands between the move and this restore means the path holds another command's whole marker
        //       by then, and that one names its own live owner: the moved marker is then dropped rather than taking its place.
        let _restored = std::fs::hard_link(&grave, path);
        let _removed: std::io::Result<()> = std::fs::remove_file(&grave);
        return Taken::ACommandIsNamed;
    }
    let _removed: std::io::Result<()> = std::fs::remove_file(&grave);
    return Taken::NobodyIsNamed;
}

impl StarterMarker {
    /// What:
    ///  Takes the marker,
    /// or nothing when another live command holds it.
    /// Why:
    ///   Three rounds are enough for the takeover:
    /// each one either publishes this command's marker or takes away a dead command's,
    /// and a live one ends the attempt.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// take(path) // { path } | undefined
    /// ```
    fn take(path: &Path) -> std::io::Result<Option<Self>> {
        return Self::take_seamed(path, &mut do_nothing);
    }

    /// `take` with a call where a control may let a second command act between one marker's making and its publication.
    fn take_seamed(path: &Path, before_the_publish: &mut dyn FnMut()) -> std::io::Result<Option<Self>> {
        // A platform that cannot name a process's birth cannot hold a marker another command may trust.
        let Ok(identity) = current_birth_identity() else {
            return Err(std::io::Error::other("this platform cannot name a process's birth, so no starter marker can be trusted"));
        };
        let content: String = format!("{} {identity}\n", std::process::id());
        for _round in 0..3 {
            match publish_a_whole_marker(path, content.as_str(), before_the_publish) {
                Ok(()) => return Ok(Some(Self { path: path.to_path_buf(), content })),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                // A folder that is not there, or one this account may not write, is a start that cannot happen, never another
                // command's start.
                Err(refused) => return Err(refused),
            }
            match take_away_a_marker_that_names_nobody(path) {
                Taken::NobodyIsNamed | Taken::ItWentAlready => {}
                Taken::ACommandIsNamed => return Ok(None),
            }
        }
        return Ok(None);
    }
}

impl Drop for StarterMarker {
    fn drop(&mut self) {
        // What: The marker goes only while it still holds this command's own text.
        // Why:  Removed by its path alone, a finishing start removed the marker another command had published by then, and a
        //       third command then took that empty place: two starts beside one worktree (the review of `76a08c9fac`,
        //       finding 3).
        if matches!(std::fs::read_to_string(&self.path), Ok(current) if current == self.content) {
            let _removed: std::io::Result<()> = std::fs::remove_file(&self.path);
        }
    }
}

/// The call `take` makes between a marker's making and its publication:
/// none.
fn do_nothing() {}

/// Whether the marker's owner is still a process with that birth identity.
/// A marker that cannot be read or parsed names nobody, so it is nobody's.
fn holder_is_alive(path: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else { return false };
    let Some((number, identity)) = text.trim_end().split_once(' ') else { return false };
    let Ok(pid) = number.parse::<i64>() else { return false };
    return matches!(process_birth_identity(pid), Ok(Some(held)) if held == identity);
}

/// A log file's name for an instance,
/// so a caller that only has the socket's folder and an instance can name it.
pub fn log_of(directory: &Path, instance: &str) -> PathBuf {
    return directory.join(format!("{instance}{LOG_ENDING}"));
}

/// Controls of the start stay out of the release executable.
#[cfg(test)]
#[path = "service_server_start_by_the_wrapper_tests.rs"]
mod tests;
