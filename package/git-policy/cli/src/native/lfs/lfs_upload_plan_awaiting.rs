//! What:
//!  The upload plan awaited by a caller that is itself a task of the runtime:
//! the same questions,
//! the same defaults and the same plan as `lfs_upload_plan`,
//!  with every wait on the runtime.
//! Why:
//!   The landing round of the service runs as tasks of the runtime
//! (`foundation/parallel_runner_on_the_runtime_awaiting.rs`),
//! and the plan's questions enter the runtime on the synchronous face and are refused from there.
//! A plan that could be
//!  worked out two ways would send other objects or refuse a push the synchronous face allows,
//! so every decision is the
//!  synchronous face's (`lfs_upload_plan`) and only the questions are awaited here.
//! The file exists beside the plan because
//!  the two together are more than one file of the package's length limit
//! (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = await planUploadOnTheRuntime(globalPrefix, region, env, directory, settings, query); // { remotes, refs, everything }
//! ```

/// Whether the destination confirms this clone's tracking refs.
use crate::lfs_destination_confirms_tracking_refs::{
    ConfirmedTrackingRefs,
    DestinationHeads,
    tracking_refs_confirmed_by_awaiting,
};
use crate::lfs_upload_plan::{
    Source,
    UploadPlan,
    directory_of,
    lfs_server_of,
    names_of,
    refspec_source,
    star_split,
};
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
/// The configuration the plan reads.
use crate::lfs_upload_settings::{
    PlanError,
    QueryAwaiting,
    current_branch_awaiting,
    git_says_true_awaiting,
};
/// Owned OS text for arguments.
use std::ffi::OsString;
/// Locations.
use std::path::Path;

/// What:
///  The same plan,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push plans its upload as a task,
/// and every question this plan asks enters the runtime on the synchronous face and is
///  refused from there.
/// Every decision,
/// every default and every destination is the synchronous face's,
/// and only the questions are awaited,
/// so the two cannot upload to different places or send different refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await planUploadOnTheRuntime(globalPrefix, region, environment, directory, settings, knownDestination, query);
/// ```
pub async fn plan_upload_awaiting(
    global_prefix: &[OsString],
    region: &[OsString],
    environment: &[(OsString, OsString)],
    directory: &Path,
    all: &[(String, String)],
    known_destination: Option<&DestinationHeads>,
    query: QueryAwaiting<'_>,
) -> Result<Option<UploadPlan>, PlanError> {
    // What:     `uploads_nothing(...)` reads the switch, the skip variable and the arguments, and answers the push's shape
    //           when it uploads anything at all; it is the synchronous face's function, so the two cannot differ here.
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
        current_branch_awaiting(query).await?
    } else {
        None
    };
    let remote_name: OsString = remote_name_of(&shape, branch.as_deref(), all);
    let remote_text: String = remote_name.to_string_lossy().into_owned();
    let mirror_is_set: bool = match a_bare_push(&shape, shape.mirror) {
        true => {
            git_says_true_awaiting(all, format!("remote.{remote_text}.mirror").as_str(), query)
                .await?
        }
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
            Source::Matching => refs.extend(list_awaiting(query, "refs/heads", None).await?),
            Source::Named(source) => match source.to_str() {
                Some(text) if text.contains('*') => {
                    refs.extend(expand_pattern_awaiting(query, text).await?)
                }
                _ => refs.push(source),
            },
        }
    }
    if shape.branches {
        refs.extend(list_awaiting(query, "refs/heads", None).await?);
    }
    if shape.tags {
        refs.extend(list_awaiting(query, "refs/tags", None).await?);
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
        tracking_refs_confirmed_by_awaiting(remote_text.as_str(), known_destination, query).await
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

/// What:
///  The same listing,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push asks which refs a refspec stands for as a task,
/// and the synchronous listing enters the runtime and is refused from there.
/// The question and the reading of its answer are the synchronous faces',
/// so the two cannot name different refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listOnTheRuntime(query, prefix, pattern);
/// ```
async fn list_awaiting(
    query: QueryAwaiting<'_>,
    prefix: &str,
    pattern: Option<(&str, &str)>,
) -> Result<Vec<OsString>, PlanError> {
    let listing: String = match query
        .ask(&["for-each-ref", "--format=%(refname)", prefix])
        .await
    {
        Ok(text) => text,
        Err(why) => return Err(PlanError(format!("git for-each-ref {prefix}: {why}"))),
    };
    return Ok(names_of(listing.as_str(), pattern));
}

/// What:
///  The refs a source with a `*` stands for,
/// awaited by a caller that is itself a task of the runtime,
/// by a refspec's own rule:
/// the text before the star,
/// the text after it,
/// and anything between,
/// `/` included (which a `for-each-ref` pattern does not do).
/// Why:
///   The synchronous expansion's rule,
/// with the wait on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await expandPatternOnTheRuntime(query, pattern);
/// ```
async fn expand_pattern_awaiting(
    query: QueryAwaiting<'_>,
    pattern: &str,
) -> Result<Vec<OsString>, PlanError> {
    return list_awaiting(query, directory_of(pattern), Some(star_split(pattern))).await;
}
