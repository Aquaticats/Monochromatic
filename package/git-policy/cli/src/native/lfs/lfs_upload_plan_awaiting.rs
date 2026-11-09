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

/// The destination and the named refs of a push.
use crate::command_push::{parse_push_region, push_shape};
/// The configuration the plan reads.
use crate::lfs_upload_settings::{PlanError, QueryAwaiting, current_branch_awaiting, default_remote, reads_as_true, setting, values};
/// The switch and the skip variable.
use crate::lfs_pre_push::{hook_switched_off, skip_requested};
use crate::lfs_upload_plan::{
    Source, UploadPlan, configured_remotes, directory_of, has_push_rewrite, lfs_destination, names_of, refspec_source, star_split,
};
/// Owned OS text for arguments.
use std::ffi::{OsStr, OsString};
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
/// await planUploadOnTheRuntime(globalPrefix, region, environment, directory, settings, query);
/// ```
pub async fn plan_upload_awaiting(
    global_prefix: &[OsString],
    region: &[OsString],
    environment: &[(OsString, OsString)],
    directory: &Path,
    all: &[(String, String)],
    query: QueryAwaiting<'_>,
) -> Result<Option<UploadPlan>, PlanError> {
    if hook_switched_off(global_prefix, environment) || skip_requested(environment) {
        return Ok(None);
    }
    let Some((region_facts, shape)) = parse_push_region(region, &[]).ok().zip(push_shape(region)) else {
        return Ok(None);
    };
    if region_facts.no_verify || region_facts.dry_run || shape.deletes {
        return Ok(None);
    }
    let submodules: bool = match shape.recurse_submodules.as_ref() {
        Some(value) => matches!(value.to_str(), Some("on-demand" | "only")),
        None => false,
    };
    let branch_needed: bool = shape.repository.is_none() || shape.refspecs.is_empty();
    let branch: Option<String> = if branch_needed { current_branch_awaiting(query).await? } else { None };
    let remote_name: OsString = match shape.repository.clone() {
        Some(typed) => typed,
        None => OsString::from(default_remote(branch.as_deref(), all)),
    };
    let remote_text: String = remote_name.to_string_lossy().into_owned();
    let mut everything: bool = shape.mirror;
    let mut specs: Vec<OsString> = shape.refspecs.clone();
    let bare: bool = specs.is_empty() && !shape.branches && !shape.tags && !everything;
    if bare {
        let configured: Vec<&str> = values(all, &format!("remote.{remote_text}.push"));
        if setting(all, &format!("remote.{remote_text}.mirror")).is_some_and(reads_as_true) {
            everything = true;
        } else if !configured.is_empty() {
            specs.extend(configured.into_iter().map(OsString::from));
        } else if setting(all, "push.default") == Some("matching") {
            specs.push(OsString::from(":"));
        } else {
            specs.push(OsString::from(match branch.as_deref() {
                Some(name) => format!("refs/heads/{name}"),
                None => String::from("HEAD"),
            }));
        }
    }
    let mut refs: Vec<OsString> = Vec::new();
    for spec in &specs {
        match refspec_source(spec.as_os_str()) {
            Source::Deletion => {}
            Source::Matching => refs.extend(list_awaiting(query, "refs/heads", None).await?),
            Source::Named(source) => match source.to_str() {
                Some(text) if text.contains('*') => refs.extend(expand_pattern_awaiting(query, text).await?),
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
    let mut unique: Vec<OsString> = Vec::new();
    for found in refs {
        if !unique.contains(&found) {
            unique.push(found);
        }
    }
    if unique.is_empty() && !everything {
        return Ok(None);
    }
    let configured: bool = configured_remotes(all).contains(&remote_text.as_str());
    let push_urls: Vec<&str> = if configured { values(all, &format!("remote.{remote_text}.pushurl")) } else { Vec::new() };
    let push_url_count: usize = push_urls.len();
    let remotes: Vec<OsString> = if push_url_count > 1 {
        let mut destinations: Vec<OsString> = Vec::new();
        for url in &push_urls {
            destinations.push(lfs_destination(OsStr::new(url), directory, false));
        }
        destinations
    } else {
        vec![lfs_destination(remote_name.as_os_str(), directory, configured)]
    };
    // The tracking refs describe the destination only for a configured remote that is fetched from and pushed to the
    //  same place: a push URL, a `pushInsteadOf` rewrite, a URL or path destination or a mirror breaks that.
    let tracking_trusted: bool = configured && push_urls.is_empty() && !has_push_rewrite(all);
    let full_scan: bool = everything || !tracking_trusted;
    return Ok(Some(UploadPlan { remotes, refs: unique, everything, submodules, full_scan, objects: None }));
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
async fn list_awaiting(query: QueryAwaiting<'_>, prefix: &str, pattern: Option<(&str, &str)>) -> Result<Vec<OsString>, PlanError> {
    let listing: String = match query.ask(&["for-each-ref", "--format=%(refname)", prefix]).await {
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
async fn expand_pattern_awaiting(query: QueryAwaiting<'_>, pattern: &str) -> Result<Vec<OsString>, PlanError> {
    return list_awaiting(query, directory_of(pattern), Some(star_split(pattern))).await;
}
