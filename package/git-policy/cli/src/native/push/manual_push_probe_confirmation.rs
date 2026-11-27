//! What:
//!  One destination's confirmation:
//!  the updates it confirmed,
//!  and everything its one `ls-remote` reported.
//! Why:
//!   The upload plan for a typed push asks the destination which of this clone's tracking refs it
//!  reports;
//! that question is answered by the same connection if the confirmation lists every branch head,
//!  and the listing is carried so the plan asks nothing more.
//!  The confirmation sits in a file of its own
//!  because the probe's own file is more than the package's length limit allows one (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ConfirmedDestination = { updates: PushUpdate[]; listing: string };
//! async function confirmGroup({ git, remoteLocation, remoteName, updates }): Promise<ConfirmedDestination>;
//! ```

/// The probe's Git runner,
/// its bounded queries and its environment.
use crate::auto_push_single_flight::{
    PushGit,
    run_query,
};
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
/// Removes userinfo from text that may name a remote.
use crate::lfs_output_mask::mask_url_secrets_text;
/// Why a manual push's scope could not be established,
/// as the cause its hint is chosen by.
use crate::manual_push_cause::ManualPushCause;
/// One update Git's own report printed for a destination,
/// and the value the remote held before the push.
use crate::manual_push_porcelain::{
    PorcelainUpdate,
    RemoteBefore,
};
/// The probe's failure type,
/// the cause a failed query carries,
/// and the two ways a failure is built.
use crate::manual_push_probe_error::{
    ManualPushProbeError,
    cause_of_query,
    probe_error,
};
/// The parser of the ls-remote listing,
/// beside the other listing parsers.
use crate::manual_push_range_published_tips::oids_by_ref;
/// The public update of one pushed reference.
use crate::manual_push_update_and_query_limits::PushUpdate;
/// Maps each ref the remote listed to the object name it reported.
use std::collections::HashMap;
/// Owned OS strings for the probe's Git arguments.
use std::ffi::OsString;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// The pattern that makes one confirmation query list every branch head too,
/// which is the answer the Git LFS upload plan
///  needs about this clone's tracking refs.
/// `git ls-remote` matches a pattern against the tail of a ref name,
/// so this one adds the destination's
///  heads to the refs the report named.
pub const ALL_HEADS_PATTERN: &str = "refs/heads/*";

/// What:
///  One destination's confirmation:
///  the updates it confirmed,
///  and everything its one `ls-remote` reported.
/// Why:
///   The upload plan for a typed push asks the destination which of this clone's tracking refs it
///  reports;
/// that question is answered by the same connection if the confirmation lists every branch head,
///  and the listing is carried so the plan asks nothing more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfirmedDestination = { updates: PushUpdate[]; listing: string };
/// ```
#[derive(Clone, Debug)]
pub struct ConfirmedDestination {
    /// The updates this destination confirmed.
    pub updates: Vec<PushUpdate>,
    /// The listing its one `ls-remote` printed,
    /// which names every branch head as well as the pushed refs.
    pub listing: String,
}

pub fn confirm_group(
    git: &PushGit,
    remote_location: &str,
    remote_name: &str,
    updates: &[(PorcelainUpdate, Option<String>)],
) -> Result<ConfirmedDestination, ManualPushProbeError> {
    let mut arguments: Vec<OsString> = vec![
        OsString::from("ls-remote"),
        OsString::from("--refs"),
        OsString::from(remote_location),
    ];
    for (update, _local) in updates {
        arguments.push(OsString::from(&update.to));
    }
    // The branch heads of the destination answer the LFS upload plan's question about this clone's tracking refs, so they
    //  travel in this query rather than in one of their own.
    arguments.push(OsString::from(ALL_HEADS_PATTERN));
    let output = match run_query(git, arguments.as_slice()) {
        Ok(found) => found,
        Err(error) => {
            return Err(probe_error(
                cause_of_query(&error),
                mask_url_secrets_text(
                    format!("ls-remote of {remote_location} failed: {error}").as_str(),
                ),
            ));
        }
    };
    if output.code != Some(0) {
        return Err(probe_error(
            ManualPushCause::RemoteUnreachable,
            mask_url_secrets_text(
                format!(
                    "ls-remote of {remote_location} failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim_end()
                )
                .as_str(),
            ),
        ));
    }
    let listing: String = String::from_utf8_lossy(&output.stdout).into_owned();
    let listed: HashMap<&str, &str> = oids_by_ref(listing.as_str());
    let mut confirmed: Vec<PushUpdate> = Vec::new();
    for (update, local) in updates {
        crate::collected_elements_looked_at::note_looked_at(1);
        let authoritative: Option<String> =
            listed.get(update.to.as_str()).copied().map(str::to_owned);
        let changed: bool = match &update.before {
            RemoteBefore::Absent => authoritative.is_some(),
            RemoteBefore::Known(reported) => authoritative.as_ref() != Some(reported),
            RemoteBefore::Unknown => false,
        };
        // What: a destination that moved to exactly the object this update sends asks for nothing.
        // Why:  the commit's own auto-push publishes the same commit while a typed push is on its way, so the one
        //       change a destination can take between Git's dry run and this read is that push landing; the person asked
        //       for this object, the destination holds it, and the update adds no commit to the range the policies read.
        // A destination that moved to any other value is a change this push cannot account for.
        if changed && authoritative.as_ref() != local.as_ref() {
            return Err(probe_error(
                ManualPushCause::RemoteMoved,
                format!(
                    "Remote ref changed during manual-push discovery: {}",
                    update.to
                ),
            ));
        }
        if changed {
            debug(
                TAG,
                &format!(
                    "{} already holds the object this push sends, so nothing is left to publish from it",
                    update.to
                ),
            );
        }
        confirmed.push(PushUpdate {
            local_oid: local.clone(),
            remote_oid: authoritative,
            remote_name: remote_name.to_owned(),
            remote_ref: update.to.clone(),
        });
    }
    return Ok(ConfirmedDestination {
        updates: confirmed,
        listing,
    });
}
