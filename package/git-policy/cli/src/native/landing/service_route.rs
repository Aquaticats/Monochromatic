//! What:
//!  How a commit reaches the resident service:
//! already listening,
//! or set up first by the wrapper.
//! Why:
//!  There is no switch and no install state for the person to manage (`doc/planning/cli-git-resident-service.md`,
//!  "Provisioning").
//! The route follows the machine:
//! a user session bus and the wrapper's own executable (which is the service
//!  program too) mean the wrapper can set the service up itself,
//! and keeps it set up (the socket is the manager's and outlives every upgrade,
//! so
//!  its presence never says the installed units name this release's program);
//! a socket without them is used as it is;
//!  anything else has no route,
//! and the commit stops with the plain message of its platform,
//! because the service is the only
//!  writer of a commit.
//! The decision reads the file system only and starts no process.
//! The Git directory has two faces:
//! the synchronous one a caller on a thread asks (`worktree_git_dir`),
//! and the awaited one a task of the runtime
//!  asks (`worktree_git_dir_awaiting`),
//! which reads the repository's layout as a job of the blocking pool,
//! awaited no longer than
//!  `ROUTE_STATE_CEILING`,
//! and awaits its Git child on the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const route = chooseRoute({ runtimeDir, busAddress, configHome, homeDir, ownExecutable, realGit }); // undefined: no route
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::repo_read_layout::{RepoLayout, read_layout_in};
/// A pathname from the bytes Git printed for it,
///  and one answer without its terminating line feed.
use crate::hooks_path_values::{path_answer, without_the_terminator};
use crate::service_address_of_a_worktree::{socket_address_of, the_address_is_there};
use crate::service_client_listen_and_ask_to_retire::Reached;
use crate::service_instance::{SOCKET_DIRECTORY, service_unit_instance, try_instance_of};
use crate::service_provision::{ProvisionHost, RETIREMENT_WAIT};
use crate::service_server::Bind;
use crate::systemd_dbus::session_bus_path;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::wait_ceilings::{HAND_OVER_GREETING_WAIT, ROUTE_STATE_CEILING};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

static OWN_EXECUTABLE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Remembers where the running wrapper is,
/// once,
/// so the unit of the service can name it (the service is a mode of the same
///  executable).
///  Identification only:
/// nothing is started from this path (`doc/planning/cli-git-no-self-invocation.md`);
/// the path is
///  read to name this executable in a unit file that the operating system's manager runs,
/// as the program of its service mode.
pub fn remember_own_executable(path: &Path) {
    let _first = OWN_EXECUTABLE.set(path.to_path_buf());
}

/// The wrapper's own path,
/// when the process remembered it.
pub fn remembered_own_executable() -> Option<&'static Path> {
    return OWN_EXECUTABLE.get().map(PathBuf::as_path);
}

/// What the route decision reads.
#[derive(Clone, Copy, Debug)]
pub struct RouteInputs<'a> {
    /// `XDG_RUNTIME_DIR`.
    pub runtime_dir: Option<&'a Path>,
    /// `DBUS_SESSION_BUS_ADDRESS`.
    pub bus_address: Option<&'a str>,
    /// `XDG_CONFIG_HOME`.
    pub config_home: Option<&'a Path>,
    /// `HOME`.
    pub home: Option<&'a Path>,
    /// The running wrapper's own executable.
    pub own_executable: Option<&'a Path>,
    /// The real Git the service will run.
    pub real_git: &'a Path,
    /// The worktree's own Git directory,
    /// which names the service instance the commit goes to.
    pub git_dir: &'a Path,
}

/// How a commit reaches the service:
/// through the service at `address`,
/// after provisioning when `provision` is present.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Route {
    /// Where the service listens.
    pub address: Bind,
    /// What to set up,
    /// or to check is set up,
    /// first;
    /// absent only when a socket exists and the wrapper cannot set up
    ///  a service itself (no session bus,
    /// or no executable of its own to name).
    pub provision: Option<ProvisionHost>,
}

/// The directory a user's own unit files live in.
pub fn user_unit_dir(config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    // The configuration home when one is set, else `.config` under the home directory.
    if let Some(directory) = config_home {
        return Some(directory.join("systemd").join("user"));
    }
    let directory: &Path = home?;
    return Some(directory.join(".config").join("systemd").join("user"));
}

/// What:
///  Decides the route.
/// Why:
///   Provisioning needs the session bus socket and the wrapper's own executable,
/// and it is always handed back when both are there,
///  even when the socket exists:
/// the socket outlives an upgrade,
/// and the units it activates may name a program the upgrade
///  removed.
/// Whether anything must be written is decided where the units are read (`provision`),
/// so this decision stays
///  free of reads.
/// A socket that exists without the means to provision is used as it is,
/// whoever made it;
/// with neither
///  the socket nor the means there is no route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chooseRoute(inputs) // undefined when nothing can reach or start a service
/// ```
pub fn choose_route(inputs: &RouteInputs<'_>) -> Option<Route> {
    let runtime_dir = inputs.runtime_dir?;
    // A Git directory that cannot be resolved (a link to a drive that is not mounted) names no instance: no socket path or
    // unit instance is made for it.
    let instance: String = try_instance_of(inputs.git_dir).ok()?;
    let socket: Bind = socket_address_of(runtime_dir, instance.as_str());
    let socket_exists: bool = the_address_is_there(&socket);
    // A bus socket that is not there is no bus to provision through.
    let bus: PathBuf = match session_bus_path(inputs.bus_address, Some(runtime_dir)) {
        Some(path) if path.exists() => path,
        _ => return if socket_exists { Some(Route { address: socket, provision: None }) } else { None },
    };
    // The service is a mode of the wrapper's own executable (`service_main.rs`), so the unit names that file; a path that
    //  is not a file (a deleted executable) is no program for a unit to name.
    let service_binary: PathBuf = match inputs.own_executable {
        Some(executable) if executable.is_file() => executable.to_path_buf(),
        _ => return if socket_exists { Some(Route { address: socket, provision: None }) } else { None },
    };
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: runtime_dir.to_path_buf(),
        user_unit_dir: user_unit_dir(inputs.config_home, inputs.home),
        bus,
        service_binary,
        git: inputs.real_git.to_path_buf(),
        instance,
        git_dir: inputs.git_dir.to_path_buf(),
        retirement_wait: RETIREMENT_WAIT,
    };
    return Some(Route { address: socket, provision: Some(host) });
}

/// What `git monochromatic-git-policy-cli service status` prints and exits with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReport {
    /// Exit status:
    /// 0 when the service answered.
    pub code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// What:
///  The two facts one layout question names,
/// owned so that a job of the blocking pool can take them.
/// Why:
///  The reader walks up from the start directory and reads the configuration and the directory entries it finds,
/// so it
///  runs as a job of the pool and never on the task that asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LayoutQuestion = { directory: string; overlay: [string, string][] };
/// ```
struct LayoutQuestion {
    /// The directory the reader starts walking up from.
    directory: PathBuf,
    /// The environment the reader judges the layout under.
    overlay: Vec<(OsString, OsString)>,
}

/// The reading one job of the pool does:
/// `read_layout_in` over the question's own directory and environment.
fn layout_of(question: LayoutQuestion) -> Option<RepoLayout> {
    return read_layout_in(question.directory.as_path(), question.overlay.as_slice());
}

/// What:
///  The repository's layout as a job of the blocking pool,
/// awaited no longer than `ceiling`.
/// Why:
///   A task of the runtime may not read the file system:
/// the walk up from the start directory,
/// the configuration
///  files it reads and the directory entries it checks wait in the kernel on a mount that stopped answering,
/// which would
///  hold a worker the runtime needs,
/// and no timer on that worker can end it.
/// A reading the ceiling or a refused
///  thread leaves without an answer answers nothing,
/// which is what the files answer when the layout is not one the reader
///  models,
/// so the caller asks Git and the person reads Git's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function layoutWithin(ceilingMs: number, question: LayoutQuestion): Promise<RepoLayout | undefined>;
/// ```
async fn layout_within(ceiling: Duration, directory: &Path, overlay: &[(OsString, OsString)]) -> Option<RepoLayout> {
    let question: LayoutQuestion = LayoutQuestion { directory: directory.to_path_buf(), overlay: overlay.to_vec() };
    return match run_a_job_awaited_no_longer_than(ceiling, question, layout_of).await {
        TheAnswerOfAJobAwaited::Ran(answer) => answer,
        // No thread was given, or the job outlasted its ceiling: the files said nothing this task may wait for.
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}

/// What:
///  The question both faces ask Git for the worktree's Git directory.
/// Why:
///   A command line that names its repository selects the instance of that one,
/// so the question carries the selection and
///  asks without the inherited prefix;
/// one request serves both faces,
/// so neither can select another repository than the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd, arguments: ['rev-parse', '--path-format=absolute', '--absolute-git-dir'], withoutPrefix: named, repositorySelection: selection };
/// ```
fn git_directory_request(cwd: &Path, selection: &[OsString], named: bool) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(cwd, &["rev-parse", "--path-format=absolute", "--absolute-git-dir"]);
    if named {
        request.without_prefix = true;
        request.repository_selection = selection.to_vec();
    }
    return request;
}

/// What:
///  The Git directory Git's answer to `git_directory_request` names,
///  and nothing when it named none.
/// Why:
///  Both faces read Git's answer through this one function,
/// so a question Git could not answer and an answer that
///  named nothing leave both of them without a directory.
///  The answer is a pathname,
/// so it is read as the bytes Git printed it as,
/// with only Git's one terminating line
///  feed removed:
/// a spelling of it named a directory that does not exist when the name was not UTF-8,
/// and taking
///  every ASCII whitespace byte after that terminator read a directory named `metadata `,
/// or one whose name ends in
///  a line feed,
/// as `metadata`,
/// so the route named a directory that does not exist and the commit found no instance
///  to hand its work to instead of reaching the service of the repository the person selected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitDirectoryAnswered(answered: GitOutput): string | undefined { if (!answered.succeeded) return undefined; const printed = withoutTheTerminator(answered.out); return printed.length ? pathAnswer(printed) : undefined; }
/// ```
fn git_directory_answered(output: &GitOutput) -> Option<PathBuf> {
    if !output.succeeded() {
        return None;
    }
    let printed: &[u8] = without_the_terminator(output.stdout.as_slice());
    return if printed.is_empty() { None } else { Some(path_answer(printed)) };
}

/// What:
///  The real path of the Git directory of the worktree a command runs in,
/// read from the file system when the layout is a plain
///  one and from Git otherwise.
/// Why:
///   The Git directory names the service instance,
/// so the route cannot be chosen without it;
/// the preparation of the commit
///  asks the same question later and shares the answer's cost when the file system can give it.
///  A command line that names its repository selects the instance of that one:
///  the reader answers for the directory it is given and cannot honour a Git directory beside the work tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeGitDir(context, cwd) // undefined outside a repository
/// ```
pub fn worktree_git_dir(context: &GitContext, cwd: &Path) -> Option<PathBuf> {
    let selection: Vec<std::ffi::OsString> = crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    let named: bool = !selection.is_empty();
    if !named
        && let Some(layout) = read_layout_in(cwd, context.overlay.as_slice())
    {
        return Some(layout.git_dir);
    }
    let output = run_git(context, &git_directory_request(cwd, selection.as_slice(), named)).ok()?;
    return git_directory_answered(&output);
}

/// What:
///  `worktree_git_dir` with the layout read as a job of the blocking pool and its Git child awaited on the
///  runtime.
/// Why:
///   A task of the runtime may wait on neither a file system nor a child it starts itself,
/// and the Git directory is what the
///  route to the service is chosen from once the commit path is a task.
/// The order of the two answers (the files
///  first,
/// then Git's own) and the selection each of them honours are the synchronous face's,
/// so the two cannot name different
///  directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDir = await worktreeGitDirAwaiting(context, cwd); // undefined outside a repository
/// ```
pub async fn worktree_git_dir_awaiting(context: &GitContext, cwd: &Path) -> Option<PathBuf> {
    let selection: Vec<std::ffi::OsString> = crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    let named: bool = !selection.is_empty();
    if !named
        && let Some(layout) = layout_within(ROUTE_STATE_CEILING, cwd, context.overlay.as_slice()).await
    {
        return Some(layout.git_dir);
    }
    let output = run_git_awaiting(context, &git_directory_request(cwd, selection.as_slice(), named)).await.ok()?;
    return git_directory_answered(&output);
}

/// What:
///  The path an environment variable holds,
/// nothing when it is unset or empty.
/// Why:
///  An empty runtime,
/// configuration or home directory names no place,
/// so the route treats it as unset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nonEmptyPathVariable = (name: string) => process.env[name] ? process.env[name] : undefined;
/// ```
pub(crate) fn non_empty_path_variable(name: &str) -> Option<PathBuf> {
    return match std::env::var_os(name) {
        Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
        _ => None,
    };
}

/// What:
///  The route to the service of a worktree from this process's environment.
/// Why:
///   The commit and the commands that hand a push over after they moved a branch reach the service the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// routeFromEnvironment(realGit, gitDir) // undefined: no way to reach a service
/// ```
pub fn route_from_environment(real_git: &Path, git_dir: &Path) -> Option<Route> {
    // The read of this process's environment is the read of the invocation's: `main` hands the entry `std::env::vars_os()` as
    //  that environment, and nothing in the crate changes the wrapper's own environment, so a command started with another
    //  `HOME` reads that home here. No service chooses a route (a service start goes to `service_main` before any command's
    //  route is picked), so this read never answers with a manager's home while serving.
    let runtime: Option<PathBuf> = non_empty_path_variable("XDG_RUNTIME_DIR");
    let config_home: Option<PathBuf> = non_empty_path_variable("XDG_CONFIG_HOME");
    let home: Option<PathBuf> = non_empty_path_variable("HOME");
    let bus_address: Option<String> = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
    return choose_route(&RouteInputs {
        runtime_dir: runtime.as_deref(),
        bus_address: bus_address.as_deref(),
        config_home: config_home.as_deref(),
        home: home.as_deref(),
        own_executable: remembered_own_executable(),
        real_git,
        git_dir,
    });
}

/// What a socket of the commit service did when the status asked it.
/// Why:
///  A socket that takes a connection is not a service that answers:
/// the status is what the message of a failed commit sends a person to,
/// so it tells a service that works from one
///  that is running and has stopped inside itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answered = 'greeted' | 'said-nothing' | 'nothing'
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Answered {
    /// A service took the connection and sent its greeting.
    Greeted,
    /// Something took the connection and sent nothing within the bound.
    SaidNothing,
    /// Nothing took the connection.
    Nothing,
}

/// What:
///  What the commit service at `address` does when it is asked,
/// waited for no longer than the wait a hand-over grants a greeting.
/// Why:
///   A service greets a connection before it restores anything,
/// so a connection taken and greeted is a service that runs,
/// and a connection taken and never greeted is a program that stopped inside itself
/// (`wait_ceilings.rs`,
/// `HAND_OVER_GREETING_WAIT`).
/// The address is the one the commit itself uses, a socket file or an abstract name (`service_address_of_a_worktree.rs`),
/// and a connection another account's process took is no service of this command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = asking(address); // 'greeted' | 'said-nothing' | 'nothing'
/// ```
fn asking(address: &Bind) -> Answered {
    let Reached::Ours(stream) = crate::service_client_listen_and_ask_to_retire::reach(address) else {
        // Nothing took the connection, or a process of another account did, which is no service of this command.
        return Answered::Nothing;
    };
    let _timeout = stream.set_read_timeout(Some(HAND_OVER_GREETING_WAIT));
    let mut reader: std::io::BufReader<std::os::unix::net::UnixStream> = std::io::BufReader::new(stream);
    return match crate::service_client_listen_and_ask_to_retire::listen(&mut reader) {
        crate::service_client_listen_and_ask_to_retire::Heard::Message(_) => Answered::Greeted,
        crate::service_client_listen_and_ask_to_retire::Heard::Silent => Answered::SaidNothing,
        crate::service_client_listen_and_ask_to_retire::Heard::Closed => Answered::Nothing,
    };
}

/// What:
///  Whether any commit service of the account greets a connection at a runtime directory.
/// Why:
///   Without a worktree to ask about,
/// any service of the account counts,
/// and one that takes a connection and says nothing does not.
/// A service holds either a socket file under the folder of the sockets or, where that folder's path is too long for one
/// socket address, an abstract name, which has no file to be found by
/// (`service_address_of_a_worktree.rs`).
/// The instances of an account are the ones the notes name, which provisioning writes for every instance and the sweep
/// reads for the same reason (`instances_with_units`), and each names its own address;
/// the files still answer for an instance whose note is gone, which a process killed in provisioning leaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdir(directory).some((name) => name.endsWith('.sock') && asking(join(directory, name)) === 'greeted')
///   || instancesWithUnits(runtime).some((instance) => asking(socketAddressOf(runtime, instance)) === 'greeted')
/// ```
fn any_service_greets(runtime_dir: &Path) -> bool {
    // What: A folder of sockets that is not there holds no socket file, and is not the end of the question.
    // Why:  An account served at abstract names has no such folder until something makes it (the fixture of a control does,
    //       a person's runtime directory of a long path never does), and the names below are where its services are found.
    if let Ok(entries) = std::fs::read_dir(runtime_dir.join(SOCKET_DIRECTORY)) {
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let is_socket_file: bool = match path.extension() {
                Some(extension) => extension == "sock",
                None => false,
            };
            if is_socket_file && asking(&Bind::Path(path)) == Answered::Greeted {
                return true;
            }
        }
    }
    // What: The names this runtime directory's services hold, asked at each of them.
    // Why:  A name has no file to be found by, and the table of unix sockets is where it is visible without connecting;
    //       `listening_names_of` reads only the names whose account and place are this runtime directory's, so a service of
    //       another account, or of another runtime directory, is never counted. A path-form instance is the socket file
    //       asked above and is not in that list.
    for name in crate::service_address_of_a_worktree::listening_names_of(runtime_dir, Path::new("/proc")) {
        if asking(&Bind::Abstract(name)) == Answered::Greeted {
            return true;
        }
    }
    return false;
}

/// What:
///  Says what the commit service of a worktree does,
///  names the unit that serves it,
///  and gives the step that ends a service which stopped inside itself.
/// Why:
///   A diagnostic a person reaches for when a commit failed:
/// connecting is what a commit does,
/// so a service the manager would start on demand is started by this check too,
/// and the unit its restart
///  names is the one of this worktree and no other.
/// A restart of every service of the account is what the failure this check answers must never ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceStatus(runtimeDir, gitDir)
/// ```
pub fn service_status(runtime_dir: Option<&Path>, git_dir: Option<&Path>) -> StatusReport {
    let what: &str = if git_dir.is_some() { "The commit service of this worktree" } else { "A commit service" };
    let instance: Option<String> = match (runtime_dir, git_dir) {
        (Some(_runtime), Some(git)) => match try_instance_of(git) {
            Ok(found) => Some(found),
            Err(unreachable) => {
                return StatusReport {
                    code: 1,
                    stdout: String::new(),
                    stderr: format!(
                        "The commit service of this worktree cannot be asked: {unreachable}. Nothing was connected to or started; \
                         if the repository is on a drive that is not mounted, mount it and run this again.\n"
                    ),
                };
            }
        },
        _ => None,
    };
    let answered: Answered = match (runtime_dir, instance.as_deref()) {
        (Some(runtime), Some(found)) => asking(&socket_address_of(runtime, found)),
        (Some(runtime), None) => {
            if any_service_greets(runtime) {
                Answered::Greeted
            } else {
                Answered::Nothing
            }
        }
        (None, _) => Answered::Nothing,
    };
    let unit: String = match instance.as_deref() {
        Some(found) => format!(" Its unit is `{}`.", service_unit_instance(found)),
        None => String::new(),
    };
    return match answered {
        Answered::Greeted => StatusReport { code: 0, stdout: format!("{what} is running and answers.{unit}\n"), stderr: String::new() },
        Answered::SaidNothing => StatusReport {
            code: 1,
            stdout: String::new(),
            stderr: format!(
                "{what} took a connection and sent nothing for {} seconds, so it is running and has stopped inside itself: it was \
                 frozen, a debugger holds it, or it waits for a disk that never answers. Starting it again does not help, because the \
                 service manager believes it runs.{unit} Restart that unit with `systemctl --user restart` and the unit name printed here, \
                 then run the commit again.\n",
                HAND_OVER_GREETING_WAIT.as_secs()
            ),
        },
        Answered::Nothing => StatusReport {
            code: 1,
            stdout: String::new(),
            stderr: format!(
                "{what} does not answer here. It sets itself up the first time a commit needs it, so there is nothing to \
                 install; if a commit fails, its message says what to do.{unit}\n"
            ),
        },
    };
}

/// Route controls stay out of the release executable.
#[cfg(test)]
#[path = "service_route_tests.rs"]
mod tests;

/// Controls for the awaited Git directory stay out of the release executable.
#[cfg(test)]
#[path = "service_route_awaiting_tests.rs"]
mod awaiting_tests;
