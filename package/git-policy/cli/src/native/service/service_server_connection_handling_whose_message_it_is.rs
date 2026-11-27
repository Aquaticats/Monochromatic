//! What:
//!  Whether a message a connection read is this instance's own:
//!  the instance of a Git directory,
//!  read on the blocking pool because naming it resolves the path,
//!  the refusal of a message about another worktree,
//!  and the words for a service that cannot take the request now.
//! Why:
//!  One service runs per worktree,
//!  so a message that names another worktree's Git directory reached the wrong socket
//!  and is refused where it can be seen;
//!  the check costs a filesystem read,
//!  which is why it is a step of its own and runs beside the beats.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const foreign = await isForeign(shared, gitDir); // true | false | undefined
//! ```

use crate::blocking_job::run_blocking_job;
use crate::service_instance::instance_of;
use crate::service_protocol::{
    FailureCode,
    ServiceMessage,
};
use crate::service_server_connection_handling_what_every_connection_shares::ConnectionShared;
use crate::service_server_queue_types_and_send::send;
use std::path::PathBuf;
use tokio::net::unix::OwnedWriteHalf;
/// What:
///  The point at which this file reads `git_dir`'s instance,
///  for a control that holds that read (`service_fault_hooks_test_support.rs`).
/// Why:
///   The point names the directory whose instance is read,
///  so two controls of one process hold the read of their own directory and neither answers the other (issue 6497),
///  as `collection_point` names the directory it collects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const instanceReadPoint = (gitDir: string) => `the-instance-read-of-${gitDir}`;
/// ```
#[cfg(test)]
pub(crate) fn instance_read_point(git_dir: &std::path::Path) -> String {
    return format!("the-instance-read-of-{}", git_dir.display());
}

/// Whether a Git directory belongs to another instance of the service than this one,
/// asked on the blocking pool because naming the instance resolves the path;
/// nothing when the pool could not run it.
/// Why:
///  A request that names this worktree's own Git directory exactly is this instance's without that ask:
/// the same text
///  names the same directory,
/// so the same instance,
/// and the filesystem is asked only for a request that spells it another way
///  or belongs to another worktree.
///  The ask resolves a real path,
/// which under a busy disk took thirty-two seconds in the record of
///  2026-11-10T17:50:19Z,
/// and it is on the answering path of every commit and every wake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isForeign(shared, gitDir) // shared.ownGitDir === gitDir ? false : await named(gitDir) !== shared.instance
/// ```
pub(crate) async fn is_foreign(
    shared: &ConnectionShared,
    git_dir: PathBuf,
) -> Option<bool> {
    let Some(mine) = shared.instance.clone() else {
        return Some(false);
    };
    // The text this service was named for: the same text names the same directory, whatever a resolution would say.
    if shared.own_git_dir.as_deref() == Some(git_dir.as_path()) {
        return Some(false);
    }
    return match run_blocking_job(move || {
        // A control holds this read on whichever thread runs it, which is a pool thread and not the thread that set the hold.
        #[cfg(test)]
        {
            let held_point: String = instance_read_point(git_dir.as_path());
            crate::service_fault::hooks::run_on_any_thread(held_point.as_str());
        }
        return instance_of(git_dir.as_path()) != mine;
    })
    .await
    {
        Ok(foreign) => Some(foreign),
        Err(error) => {
            crate::service_error!(
                "the instance of a Git directory could not be looked at: {error}"
            );
            None
        }
    };
}

/// Refuses a message about a Git directory that another instance serves.
pub(crate) async fn refuse_a_message_for(
    git_dir: &std::path::Path,
    write_half: &mut OwnedWriteHalf,
) {
    crate::service_warn!(
        "refused a message for {}, which another instance serves",
        git_dir.display()
    );
    send(
        write_half,
        &ServiceMessage::Failed {
            id: String::new(),
            code: FailureCode::Rejected,
        },
    )
    .await;
}

/// Tells a client that the service cannot take its request now.
pub(crate) async fn tell_unavailable(write_half: &mut OwnedWriteHalf) {
    send(
        write_half,
        &ServiceMessage::Failed {
            id: String::new(),
            code: FailureCode::Unavailable,
        },
    )
    .await;
}
