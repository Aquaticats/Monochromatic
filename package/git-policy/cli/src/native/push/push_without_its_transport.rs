//! What:
//!  The words a service push stops with when the transport settings of the command that asked for it are gone.
//! Why:
//!  A service holds those settings only in its memory:
//! `GIT_SSH`,
//! `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND` are whole
//!  command lines that can hold a password,
//! so nothing of them is written and a service that restarted has none of
//!  them (`service_push_programs_held_in_memory`).
//! A push whose command chose its transport must then stop rather than
//!  run through the manager's environment:
//! that would speak a way its person did not ask for,
//! and may carry none of
//!  the credentials the chosen way needs.
//!  The words therefore say what is gone,
//! that nothing was sent through another transport,
//! and the one command that
//!  pushes the branch:
//! the same refspec the service would have used,
//! run from the shell that still has the settings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = await wordsOfAPushWithoutItsTransport(git, item); // a skip's text, shown by the person's next command
//! ```

use crate::auto_push::{Planned, push_arguments};
use crate::auto_push_awaiting::plan_named_push_awaiting;
use crate::auto_push_single_flight::PushGit;
use crate::shell_quote::shell_quote;
use crate::work_item::WorkItem;

/// What:
///  The words a push is skipped with,
///  because the transport settings of the command that asked for it are gone.
/// Why:
///   The person must learn three things:
/// which push stopped,
/// why its settings are gone rather than merely unavailable,
/// and the command that pushes without them.
/// The command is the push the service would have made,
/// against the branch as it stands,
/// so the person runs exactly what they asked for in the first place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the push of ${branch} of ${repository} was not made: ... run: ${git push ...}`
/// ```
pub async fn words(git: &PushGit, item: &WorkItem) -> String {
    let command: Option<String> = push_command(git, item).await;
    // The command is the last line and on its own, so a person copies one line and a control can run exactly what is printed.
    let repository: String = item.repository.display().to_string();
    let run: String = match command {
        Some(printed) => format!("\nRun this in {repository}: {printed}"),
        // A branch with no upstream and no `origin` has nowhere to be pushed, so no command names a way out and the words say
        //  only what happened; the push this replaces would have been skipped for the same reason.
        None => String::from("\nThis branch names no remote to push to, so give it one and push it yourself."),
    };
    return format!(
        "the push of {branch} of {repository} was not made. The command that asked for it set GIT_SSH, GIT_SSH_COMMAND or \
         GIT_PROXY_COMMAND, and those settings are held only in the memory of the commit service; the service that holds this \
         push has been started again since, or the command that asked could not reach it, so the settings are gone, and nothing \
         was sent through another transport.{run}",
        branch = item.branch_ref,
        repository = repository,
        run = run,
    );
}

/// What:
///  The command that pushes this branch from a shell,
///  with its arguments quoted for a shell.
/// Why:
///  The person pastes it,
/// so every argument is quoted as the shell reads it,
/// and the arguments are the ones the service's own push would have
///  run (`push_arguments`),
/// so the command sends what that push would have sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = await planNamedPush(git, branchRef); return plan.kind === 'push' ? 'git ' + pushArguments(branchRef, plan.sent, plan.setUpstream).map(quote).join(' ') : undefined;
/// ```
async fn push_command(git: &PushGit, item: &WorkItem) -> Option<String> {
    // The plan is the service's own, asked of the local repository only: it names the remote and the refspec, and it starts
    //  nothing that reaches the network.
    let planned: Planned = plan_named_push_awaiting(git, item.branch_ref.as_str(), &mut std::io::sink()).await.ok()?;
    let (sent, set_upstream): (crate::auto_push::Sent, bool) = match planned {
        Planned::Skip(_) => return None,
        Planned::Push { sent, set_upstream, .. } => (sent, set_upstream),
    };
    let arguments: Vec<String> = push_arguments(item.branch_ref.as_str(), &sent, set_upstream);
    let mut quoted: Vec<String> = vec![String::from("git")];
    quoted.extend(arguments.iter().map(|argument| { return shell_quote(argument.as_bytes()); }));
    return Some(quoted.join(" "));
}

/// The controls of the words and of the command they name are in the sidecar.
#[cfg(test)]
#[path = "push_without_its_transport_tests.rs"]
mod tests;
