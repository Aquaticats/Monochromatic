//! What:
//!  What the Git LFS upload before a push sends:
//! the destinations `git lfs push` takes and the refs whose objects it
//!  uploads,
//! with `--all` wherever a remote-tracking ref of this clone may not describe the destination,
//! so that it
//!  cannot hide an object the destination lacks,
//! and in git-lfs's bounded form (no `--all`) everywhere else.
//! A tracking ref
//!  describes the destination only where the destination itself reports it at the same commit,
//! which is asked of
//!  the destination rather than assumed (`lfs_destination_confirms_tracking_refs`),
//! and each confirmed ref is a place the scan may
//!  stop at,
//! so one stale tracking ref drops only itself.
//! Why:
//!  `git lfs push` takes refs and no ref-update lines,
//! so the plan is read from the push's own arguments with the same
//!  option table as the rest of the wrapper,
//! and from the configuration Git itself would read for the parts the
//!  arguments leave out (the default remote,
//! `remote.<name>.push`,
//! `remote.<name>.mirror`,
//! the push URLs,
//!  `push.default`).
//! The configuration comes from the snapshot the caller already holds,
//! or from one query.
//!  Refs that Git expands from a pattern are listed with `git for-each-ref` and matched here with a refspec's own rule
//!  (`*` crosses `/`).
//! The plan fails closed:
//! a query that fails is an error the caller turns into a refusal,
//! never
//!  into "nothing to upload".
//! Only a mirror push uploads every ref.
//! What the destination of an upload is,
//! and the server it is connected to,
//! are read in `lfs_upload_plan_destination.rs`,
//! beside this file so that each file passes the package's length limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planUpload(globalPrefix, region, env, directory, settings, query); // { remotes, refs, everything }
//! ```

/// Whether the destination confirms this clone's tracking refs.
use crate::lfs_destination_confirms_tracking_refs::{
    ConfirmedTrackingRefs,
    DestinationHeads,
    tracking_refs_confirmed_by,
};
/// The host a URL names,
/// with any userinfo left out.
/// What the arguments and the configuration decide before Git is asked anything.
use crate::lfs_upload_plan_arguments::{
    Destinations,
    a_bare_push,
    a_branch_is_needed,
    destinations_of,
    remote_name_of,
    same_place,
    specs_of,
    submodules_of,
    the_plan,
    unique_refs,
    uploads_nothing,
};
pub use crate::lfs_upload_settings::{
    GitQuery,
    GitQueryAwaiting,
    PlanError,
    Query,
    QueryAwaiting,
    Settings,
    read_settings,
    read_settings_awaiting,
    resolve_push_remote,
    settings_from_snapshot,
};
/// The configuration the plan reads.
use crate::lfs_upload_settings::{
    current_branch,
    git_says_true,
};
/// Owned OS text for arguments.
use std::ffi::OsString;
/// Locations.
use std::path::Path;

/// What an upload sends,
/// re-exported under the path this file had for it,
/// so the callers that name the plan here keep compiling.
pub use crate::lfs_upload_plan_itself::UploadPlan;

/// The source of one refspec,
/// and the refs a pattern of it names,
/// which `lfs_upload_refspecs.rs` owns.
use crate::lfs_upload_refspecs::{
    Source,
    expand_pattern,
    list,
    refspec_source,
};

/// The server an upload is connected to.
pub use crate::lfs_upload_plan_destination::lfs_server_of;

/// What:
///  What to upload for a push,
/// or nothing when the push uploads nothing,
/// or why the plan cannot be worked out.
/// Why:
///   A push that bypasses the upload (`--no-verify`,
/// `GIT_LFS_SKIP_PUSH`,
/// the caller's switch),
/// sends nothing (a dry run,
///  only deletions) uploads nothing.
/// A push that names refs uploads those,
/// with patterns,
/// `--all`,
/// `--tags`,
/// the matching
///  refspec `:` and the configured `remote.<name>.push` expanded to the refs they stand for;
/// a bare push uploads what the
///  remote's configuration names,
/// else the branch Git would send (`HEAD` when it is not a branch);
/// a mirror push uploads
///  everything.
/// A remote with several push URLs is uploaded to once per URL.
/// `directory` is where relative paths are
///  resolved;
/// `settings` is the configuration the plan reads;
/// `known_destination` is the destination's branches when this
///  invocation already read them,
/// so the question costs no connection of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planUpload(globalPrefix, region, environment, directory, settings, knownDestination, query): UploadPlan | undefined; // throws PlanError
/// ```
pub fn plan_upload(
    global_prefix: &[OsString],
    region: &[OsString],
    environment: &[(OsString, OsString)],
    directory: &Path,
    all: &[(String, String)],
    known_destination: Option<&DestinationHeads>,
    query: Query<'_>,
) -> Result<Option<UploadPlan>, PlanError> {
    // What:     `uploads_nothing(...)` reads the switch, the skip variable and the arguments, and answers the push's shape
    //           when it uploads anything at all.
    // Why:      A push that sends no object is answered before a destination is asked anything.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const shape = uploadsNothing(globalPrefix, environment, region); if (!shape) return undefined;
    // ```
    let Some(shape) = uploads_nothing(global_prefix, environment, region) else {
        return Ok(None);
    };
    let submodules: bool = submodules_of(&shape);
    let branch: Option<String> = if a_branch_is_needed(&shape) {
        current_branch(query)?
    } else {
        None
    };
    let remote_name: OsString = remote_name_of(&shape, branch.as_deref(), all);
    let remote_text: String = remote_name.to_string_lossy().into_owned();
    let mirror_is_set: bool = match a_bare_push(&shape, shape.mirror) {
        true => git_says_true(all, format!("remote.{remote_text}.mirror").as_str(), query)?,
        false => false,
    };
    let (everything, specs): (bool, Vec<OsString>) = specs_of(
        &shape,
        remote_text.as_str(),
        branch.as_deref(),
        all,
        shape.mirror,
        mirror_is_set,
    );
    let mut refs: Vec<OsString> = Vec::new();
    for spec in &specs {
        match refspec_source(spec.as_os_str()) {
            Source::Deletion => {}
            Source::Matching => refs.extend(list(query, "refs/heads", None)?),
            Source::Named(source) => match source.to_str() {
                Some(text) if text.contains('*') => refs.extend(expand_pattern(query, text)?),
                _ => refs.push(source),
            },
        }
    }
    if shape.branches {
        refs.extend(list(query, "refs/heads", None)?);
    }
    if shape.tags {
        refs.extend(list(query, "refs/tags", None)?);
    }
    let listed: Vec<OsString> = unique_refs(refs);
    if listed.is_empty() && !everything {
        return Ok(None);
    }
    let destinations: Destinations<'_> =
        destinations_of(&remote_name, remote_text.as_str(), directory, all);
    // The destination confirms this clone's tracking refs ref by ref, where it is one the clone both fetches from and
    //  pushes to; every other destination is asked nothing and its upload stops nowhere.
    let evidence: ConfirmedTrackingRefs = if same_place(&destinations, everything, all) {
        tracking_refs_confirmed_by(remote_text.as_str(), known_destination, query)
    } else {
        ConfirmedTrackingRefs::unknown()
    };
    let full_scan: bool = everything || !evidence.every_tracking_ref_is_confirmed();
    let lfs_server: Option<String> = lfs_server_of(remote_name.as_os_str(), all);
    return Ok(Some(the_plan(
        destinations,
        listed,
        everything,
        submodules,
        full_scan,
        evidence.names().to_vec(),
        lfs_server,
    )));
}

/// The plan is controlled in the test module.
#[cfg(test)]
#[path = "lfs_upload_plan_tests.rs"]
mod tests;
