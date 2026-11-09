//! What:
//!  The small steps of a service commit:
//! the arguments after the subcommand,
//! the name of a switched branch,
//! the standard input
//!  the commit reads,
//! the message when no repository is found,
//! the lines printed after a landing,
//! and the commit the service
//!  builds itself from the message embedded in the command line.
//! Why:
//!  `service_commit.rs` holds the retry loop around them;
//! none of them uses the loop.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! argumentsAfterSubcommand(args, index); standardInputFor(invocation); await embeddedCommit(invocation, parsed)
//! ```

use crate::service_client::{Activation, ClientConfig, START_WAIT, manager_watch_for};
use crate::service_instance::try_instance_of;
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_messages::{ClientFailure, render};
use crate::service_provision::platform_without_manager;
use std::path::{Path, PathBuf};
use crate::service_server::Bind;
use crate::service_provision::ProvisionHost;
use crate::service_route::{Route, RouteInputs, choose_route, non_empty_path_variable, remembered_own_executable, worktree_git_dir};
use crate::commit_timing::CommitTiming;
use crate::commit_transaction::{CommitInvocation, TransactionStop, failed};
use crate::diagnostic_log::warn;
use crate::service_amend::{AmendSource, amended_author, read_amend_source};
use crate::service_commit_args::ServiceCommitArgs;
use crate::service_message::{author_overrides, build_message, identity};
use crate::transaction_git::{GitRequest, run_git};
use std::ffi::OsString;
use std::io::{Read, Write};
/// What:
///  The words of the command line after the subcommand:
/// the options and the paths of the command itself.
/// Why:
///   The subcommand's own name is not one of them,
/// and a command line that ends at the subcommand has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// arguments.slice(subcommandIndex + 1)
/// ```
pub(crate) fn arguments_after_subcommand(arguments: &[OsString], subcommand_index: usize) -> &[OsString] {
    return arguments.get(subcommand_index + 1..).unwrap_or(&[]);
}

/// What:
///  How a branch is named to the person when the worktree switched under a commit:
/// a detached head by those words,
/// any other
///  reference by its branch name.
/// Why:
///   `HEAD` is not a branch name,
/// and the full reference of a branch is not what the person typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reference === 'HEAD' ? 'a detached head' : reference.replace(/^refs\/heads\//, '')
/// ```
pub(crate) fn switched_branch_name(reference: &str) -> String {
    return if reference == "HEAD" { String::from("a detached head") } else { reference.trim_start_matches("refs/heads/").to_string() };
}

/// The standard input of the command when a pathspec file is read from it,
/// read once.
pub(crate) fn standard_input_for(invocation: &CommitInvocation<'_>) -> Result<Option<Vec<u8>>, TransactionStop> {
    let after: &[OsString] = arguments_after_subcommand(invocation.arguments, invocation.subcommand_index);
    let Some(value) = invocation.region.pathspec_from_file else { return Ok(None) };
    if crate::command_options_query::value_bytes(after, value) != b"-" {
        return Ok(None);
    }
    let mut bytes: Vec<u8> = Vec::new();
    if let Err(error) = std::io::stdin().read_to_end(&mut bytes) {
        return Err(failed(format!("reading the pathspec file failed: {error}")));
    }
    return Ok(Some(bytes));
}

/// What:
///  What a commit run outside any repository says:
/// Git's own words and status for the same question.
/// Why:
///   No worktree means no service instance to send the commit to.
/// Asking Git which repository the directory belongs to gives
///  the person the text and the exit status native `git commit` gives there,
/// and that question can make no commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryNotFound(invocation) // Git's stderr, then its exit status
/// ```
pub(crate) fn repository_not_found(invocation: &CommitInvocation<'_>) -> TransactionStop {
    let asked = run_git(invocation.context, &GitRequest::new(invocation.cwd, &["rev-parse", "--git-dir"]));
    let output = match asked {
        Ok(found) => found,
        Err(error) => return failed(format!("asking Git which repository this directory belongs to failed: {error}")),
    };
    if output.succeeded() {
        return failed(String::from("this directory belongs to a repository whose Git directory could not be located"));
    }
    let _reported = std::io::stderr().write_all(output.stderr.as_slice());
    return TransactionStop::NativeExit(output.code.unwrap_or(1));
}

/// What:
///  The text of a message of a commit that is being amended.
/// Why:
///  The message of the commit to replace is the previous message the new one is built from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const messageOfSource = (source: AmendSource) => source.message;
/// ```
pub(crate) fn message_of_source(found: &AmendSource) -> &str {
    return found.message.as_str();
}

/// What:
///  Writes the summary line of a landed commit and ends the command as Git does when it cannot be written.
/// Why:
///   Git prints its summary after the branch moved and dies with `SIGPIPE` when the reader is gone and with a write error when
///  the device is full or closed,
/// so a command whose output is broken ends the same way whichever program built the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// printSummary(line)
/// ```
pub(crate) fn print_summary(line: &str) {
    let mut out = std::io::stdout();
    let wrote: std::io::Result<()> = writeln!(out, "{line}");
    let written: std::io::Result<()> = match wrote {
        Ok(()) => out.flush(),
        Err(error) => Err(error),
    };
    let Err(error) = written else { return };
    if error.kind() == std::io::ErrorKind::BrokenPipe {
        // SAFETY: restoring the default action of a signal and raising it takes no pointers.
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
            libc::raise(libc::SIGPIPE);
        }
    }
    let _reported = writeln!(std::io::stderr(), "fatal: write error: {error}");
    std::process::exit(128);
}

/// What:
///  Writes one note of the wrapper's to standard error.
/// Why:
///   The note is the wrapper's own words beside Git's output on standard output;
/// a write that fails is logged and does not fail
///  the commit that has already landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// printNote(line)
/// ```
pub(crate) fn print_note(line: &str) {
    if let Err(error) = writeln!(std::io::stderr(), "{line}") {
        warn("serviceCommit", format!("writing the note about the rebuilt commit failed: {error}").as_str());
    }
}

/// What the embedded route made of a command line:
/// the message,
/// the identities and,
/// for an amend,
/// the replaced commit.
pub(crate) struct Embedded {
    pub(crate) message: String,
    pub(crate) author: String,
    pub(crate) committer: String,
    pub(crate) source: Option<AmendSource>,
    pub(crate) quiet: bool,
}

/// What:
///  Builds the message and the identities of a command line the service covers,
/// the way native Git would.
/// Why:
///   Returns nothing when Git alone can say what the line means (an unreadable message file,
/// a date it does not read,
/// no
///  identity configured,
/// an empty message),
/// and Git then builds the commit and gives the person its own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await embeddedCommit(invocation, parsed) // undefined when Git must build it
/// ```
pub(crate) fn embedded_commit(invocation: &CommitInvocation<'_>, parsed: ServiceCommitArgs) -> Result<Option<Embedded>, TransactionStop> {
    let Some(committer) = identity(invocation.context, invocation.cwd, "GIT_COMMITTER_IDENT", &[])? else { return Ok(None) };
    let source: Option<AmendSource> = if parsed.amend {
        let Some(read) = read_amend_source(invocation.context, invocation.cwd)? else { return Ok(None) };
        Some(read)
    } else {
        None
    };
    let previous: Option<&str> = source.as_ref().map(message_of_source);
    let Some(built) = build_message(invocation, &parsed, committer.as_str(), previous)? else { return Ok(None) };
    let author: String = match built.reused_author {
        Some(reused) => reused,
        None => match source.as_ref() {
            Some(replaced) => {
                let Some(made) = amended_author(invocation.context, invocation.cwd, replaced, &parsed)? else { return Ok(None) };
                made
            }
            None => {
                let Some(made) = identity(invocation.context, invocation.cwd, "GIT_AUTHOR_IDENT", author_overrides(&parsed).as_slice())? else {
                    return Ok(None);
                };
                made
            }
        },
    };
    return Ok(Some(Embedded { message: built.message, author, committer, source, quiet: parsed.quiet }));
}

/// What:
///  Where the service of this worktree is reached and,
/// when it may have to be started,
/// how to provision it.
/// Why:
///  The route depends on the process environment and on the worktree's own Git directory,
/// which names the service instance;
///  a commit outside a repository,
/// or on a machine with no way to start a service,
/// stops here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { address, provision } = routeForThisWorktree(invocation, timing); // throws outside a repository
/// ```
pub(crate) fn route_for_this_worktree(invocation: &CommitInvocation<'_>, timing: &mut CommitTiming) -> Result<(Bind, Option<ProvisionHost>), TransactionStop> {
    // The environment read here is the invocation's own, on the same terms as `route_from_environment` reads it: `main`
    //  hands the entry `std::env::vars_os()` and nothing changes the wrapper's own environment, so the home this commit was
    //  started with is the home its service route is chosen from.
    let runtime: Option<PathBuf> = non_empty_path_variable("XDG_RUNTIME_DIR");
    let config_home: Option<PathBuf> = non_empty_path_variable("XDG_CONFIG_HOME");
    let home: Option<PathBuf> = non_empty_path_variable("HOME");
    let bus_address: Option<String> = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
    // The worktree's own Git directory names the service instance, so no route exists outside a repository.
    let Some(git_dir) = worktree_git_dir(invocation.context, invocation.cwd) else { return Err(repository_not_found(invocation)) };
    let route: Option<Route> = choose_route(&RouteInputs {
        runtime_dir: runtime.as_deref(),
        bus_address: bus_address.as_deref(),
        config_home: config_home.as_deref(),
        home: home.as_deref(),
        own_executable: remembered_own_executable(),
        real_git: invocation.context.real_git.as_path(),
        git_dir: git_dir.as_path(),
    });
    let Some(Route { address, provision: provisioning }) = route else {
        return Err(refusal_without_route(git_dir.as_path()));
    };
    // From here on the commit is recorded; one that stopped above, on a machine with no way to start a service, took no time
    //  worth splitting and leaves no record (nothing it does changes the repository).
    timing.place_at(git_dir.as_path());
    return Ok((address, provisioning));
}

/// What:
///  The settings of the client that submits a commit to the service:
/// the address,
/// the waits and ceilings,
/// and the watch on the
///  service manager when the service may have to be started.
/// Why:
///  The retry loop copies it for each wait and adds the scratch directory to keep fresh.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const config = managedClientConfig(address, provisioning)
/// ```
pub(crate) fn managed_client_config(address: Bind, provisioning: Option<&ProvisionHost>) -> ClientConfig {
    return ClientConfig {
        address,
        activation: Activation::Managed,
        start_wait: START_WAIT,
        heartbeat_silence: HEARTBEAT_SILENCE,
        start_watch: provisioning.map(manager_watch_for),
        keep_fresh: None, answer_ceiling: crate::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: crate::wait_ceilings::WITHDRAW_CEILING, extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: crate::wait_ceilings::START_CEILING, waiting_notice_after: crate::wait_ceilings::waiting_notice_after(),
        units_record_dir: provisioning.map(crate::service_provision_host_and_manager::units_record_dir_of),
    };
}

/// What:
///  The stop for a worktree that has no route to a service.
/// Why:
///  The cause is named:
/// a Git directory that cannot be resolved (a link to a drive that is not mounted) is not a machine without a service manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stop = refusalWithoutRoute(gitDir);
/// ```
pub(crate) fn refusal_without_route(git_dir: &Path) -> TransactionStop {
    if let Err(unreachable) = try_instance_of(git_dir) {
        return TransactionStop::Message(format!(
            "{unreachable}, so this commit was not made and nothing was written. If the repository is on a drive that is not mounted, mount it and run the commit again."
        ));
    }
    return TransactionStop::Message(render(&ClientFailure::NoActivation(platform_without_manager())));
}

/// Controls of the stop for a worktree without a route.
#[cfg(test)]
#[path = "service_commit_input_notes_and_embedded_commit_tests.rs"]
mod tests;
