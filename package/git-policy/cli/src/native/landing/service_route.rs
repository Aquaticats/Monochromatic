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

use crate::blocking_jobs_within_the_budget::{
    TheAnswerOfAJobAwaited,
    run_a_job_awaited_no_longer_than,
};
/// A pathname from the bytes Git printed for it,
///  and one answer without its terminating line feed.
use crate::hooks_path_values::{
    path_answer,
    without_the_terminator,
};
use crate::repo_read_layout::{
    RepoLayout,
    read_layout_in,
};
use crate::service_address_of_a_worktree::{
    socket_address_of,
    the_address_is_there,
};
use crate::service_instance::try_instance_of;
use crate::service_provision::{
    ProvisionHost,
    RETIREMENT_WAIT,
};
use crate::service_server::Bind;
use crate::systemd_dbus::session_bus_path;
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::wait_ceilings::ROUTE_STATE_CEILING;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// What the status of a worktree's service answers,
/// and the words it answers with,
/// are declared beside this file
/// (`service_route_what_the_status_answers.rs`),
/// because the route that names the address is at its line bound.
pub use crate::service_route_what_the_status_answers::{
    StatusReport,
    service_status,
};

/// What:
///  The one path a release build remembers of its own executable,
///  whose sibling in a build made for tests is no path at all.
/// Why:
///  A test process's executable is the test binary,
///  and offering it to the account's manager is what issue 6586 measured,
///  so the cell a release fills is not even compiled there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let ownExecutable: string | undefined; // only in a release build
/// ```
#[cfg(not(test))]
static OWN_EXECUTABLE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// What:
///  Remembers where the running wrapper is,
///  once,
///  so the unit of the service can name it (the service is a mode of the same
///  executable).
///  Identification only:
/// nothing is started from this path (`doc/planning/cli-git-no-self-invocation.md`);
/// the path is
///  read to name this executable in a unit file that the operating system's manager runs,
/// as the program of its service mode.
/// Why:
///  A command's own path is the program a later restart of its worktree's service runs,
///  so the route names it once and every later commit of the process reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// once(() => { ownExecutable = process.execPath; });
/// ```
#[cfg(not(test))]
pub fn remember_own_executable(path: &Path) {
    let _first = OWN_EXECUTABLE.set(path.to_path_buf());
}

/// What:
///  Remembers nothing,
///  so a build made for tests never offers the manager a program of its own.
/// Why:
///   The running executable of a test process is the test binary,
///  which is libtest and not a wrapper:
///  a route that named it in a unit file had this account's own service manager run libtest
///  (`error: Unrecognized option: 'journal'`) until that unit's start limit hit,
///  while the shared program link every worktree's service is started through named that binary
///  (issue 6586).
///  A control that wants a route with a program in it hands one to [`choose_route`] itself,
///  which is where a release's answer comes from too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rememberOwnExecutable(path: string): void { /* a test process offers no program of its own */ }
/// ```
#[cfg(test)]
pub fn remember_own_executable(_path: &Path) {}

/// What:
///  The wrapper's own path,
///  when the process remembered it,
///  which a release build answers after the first command that resolved the real Git.
/// Why:
///  The route names the program a restart of a worktree's service runs,
///  so the release reads the path its own process remembered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rememberedOwnExecutable = () => ownExecutable;
/// ```
#[cfg(not(test))]
pub fn remembered_own_executable() -> Option<&'static Path> {
    return OWN_EXECUTABLE.get().map(PathBuf::as_path);
}

/// What:
///  Nothing at all,
///  because a build made for tests remembered no program to offer the manager.
/// Why:
///   A test process's own executable is the test binary,
///  and a route that offered it had this account's manager run libtest for a fixture's service
///  and the shared program link pointed at that binary (issue 6586).
///  A control whose subject is a route with a program in it hands [`choose_route`] one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rememberedOwnExecutable = () => undefined;
/// ```
#[cfg(test)]
pub fn remembered_own_executable() -> Option<&'static Path> {
    return None;
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
pub fn user_unit_dir(
    config_home: Option<&Path>,
    home: Option<&Path>,
) -> Option<PathBuf> {
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
        _ => {
            return if socket_exists {
                Some(Route {
                    address: socket,
                    provision: None,
                })
            } else {
                None
            };
        }
    };
    // The service is a mode of the wrapper's own executable (`service_main.rs`), so the unit names that file; a path that
    //  is not a file (a deleted executable) is no program for a unit to name.
    let service_binary: PathBuf = match inputs.own_executable {
        Some(executable) if executable.is_file() => executable.to_path_buf(),
        _ => {
            return if socket_exists {
                Some(Route {
                    address: socket,
                    provision: None,
                })
            } else {
                None
            };
        }
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
    return Some(Route {
        address: socket,
        provision: Some(host),
    });
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
///
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
async fn layout_within(
    ceiling: Duration,
    directory: &Path,
    overlay: &[(OsString, OsString)],
) -> Option<RepoLayout> {
    let question: LayoutQuestion = LayoutQuestion {
        directory: directory.to_path_buf(),
        overlay: overlay.to_vec(),
    };
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
fn git_directory_request(
    cwd: &Path,
    selection: &[OsString],
    named: bool,
) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(
        cwd,
        &["rev-parse", "--path-format=absolute", "--absolute-git-dir"],
    );
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
    return if printed.is_empty() {
        None
    } else {
        Some(path_answer(printed))
    };
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
pub fn worktree_git_dir(
    context: &GitContext,
    cwd: &Path,
) -> Option<PathBuf> {
    let selection: Vec<std::ffi::OsString> =
        crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    let named: bool = !selection.is_empty();
    if !named && let Some(layout) = read_layout_in(cwd, context.overlay.as_slice()) {
        return Some(layout.git_dir);
    }
    let output = run_git(
        context,
        &git_directory_request(cwd, selection.as_slice(), named),
    )
    .ok()?;
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
pub async fn worktree_git_dir_awaiting(
    context: &GitContext,
    cwd: &Path,
) -> Option<PathBuf> {
    let selection: Vec<std::ffi::OsString> =
        crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    let named: bool = !selection.is_empty();
    if !named
        && let Some(layout) =
            layout_within(ROUTE_STATE_CEILING, cwd, context.overlay.as_slice()).await
    {
        return Some(layout.git_dir);
    }
    let output = run_git_awaiting(
        context,
        &git_directory_request(cwd, selection.as_slice(), named),
    )
    .await
    .ok()?;
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
///  The route to the service of a worktree from this session's own places.
/// Why:
///   The commit and the commands that hand a push over after they moved a branch reach the service the same way,
///  and a build made for tests reaches it only where a control handed a runtime directory in
///  (`foundation/session_runtime_directory_and_bus_address.rs`,
///  issue 6586).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// routeFromEnvironment(realGit, gitDir) // undefined: no way to reach a service
/// ```
pub fn route_from_environment(
    real_git: &Path,
    git_dir: &Path,
) -> Option<Route> {
    // What:     the runtime directory and the bus address, read in the one place this crate reads them
    //           (`foundation/session_runtime_directory_and_bus_address.rs`).
    // Why:      a build the release ships reads this session's own two facts, and a build made for tests reads
    //           only what the control that started its body handed in,
    //           so a test process reaches this account's manager only when a control said it may (issue 6586).
    //           Nothing in the crate changes the wrapper's own environment, so a command started with another
    //           `XDG_RUNTIME_DIR` reads that one here, and no service picks a route (a service start goes to
    //           `service_main` before any command's route is chosen).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const runtime = session.runtimeDirectory(); const bus = session.busAddress();
    // ```
    let runtime: Option<PathBuf> =
        crate::session_runtime_directory_and_bus_address::runtime_directory();
    let config_home: Option<PathBuf> = non_empty_path_variable("XDG_CONFIG_HOME");
    let home: Option<PathBuf> = non_empty_path_variable("HOME");
    let bus_address: Option<String> =
        crate::session_runtime_directory_and_bus_address::bus_address();
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

/// Route controls stay out of the release executable.
#[cfg(test)]
#[path = "service_route_tests.rs"]
mod tests;

/// Controls for the awaited Git directory stay out of the release executable.
#[cfg(test)]
#[path = "service_route_awaiting_tests.rs"]
mod awaiting_tests;
