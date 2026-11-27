//! What:
//!  What the arguments of a `git push` and the configuration decide about an upload,
//!  before any question is put to Git or a destination:
//!  whether this push uploads at all,
//!  which refspecs it sends,
//!  which refs that makes,
//!  where those go,
//!  and the plan built from them.
//! Why:
//!   The plan has two faces,
//!  one that asks Git synchronously (`lfs_upload_plan.rs`) and one that awaits it on the runtime
//!  (`lfs_upload_plan_awaiting.rs`),
//!  and only the questions differ between them.
//!  Every decision below is the same on both,
//!  so it is written once here:
//!  a plan that could be worked out two ways would upload other objects or refuse a push the other face allows
//!  (debt ledger of 2026-11-25,
//! item 13).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const shape = uploadsNothing(prefix, env, region) ?? shapeOfPush(prefix, env, region);
//! const { everything, specs } = specsOf(shape, remote, branch, all, mirror);
//! const plan = thePlan(destinations, uniqueRefs(refs), ...);
//! ```

/// The push's own arguments,
/// and what they say the upload is.
use crate::command_push::{
    PushShape,
    parse_push_region,
    push_shape,
};
/// The switch and the skip variable that stop an upload.
use crate::lfs_pre_push::{
    hook_switched_off,
    skip_requested,
};
/// What the destination of an upload is,
/// and which remotes the configuration defines.
use crate::lfs_upload_plan_destination::{
    configured_remotes,
    has_push_rewrite,
    lfs_destination,
};
/// What an upload sends.
use crate::lfs_upload_plan_itself::UploadPlan;
/// The configuration the plan reads.
use crate::lfs_upload_settings::{
    default_remote,
    setting,
    values,
};
/// Owned OS text for arguments.
use std::ffi::{
    OsStr,
    OsString,
};
/// Locations.
use std::path::Path;

/// What:
///  The shape of a push that uploads something,
///  or nothing when this push uploads no object at all.
/// Why:
///   A push that bypasses the upload (`--no-verify`,
/// `GIT_LFS_SKIP_PUSH`,
/// the caller's switch),
/// sends nothing (a dry run,
/// only deletions),
///  so the plan is answered before anything is asked;
///  both faces ask this one question first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uploadsNothing(prefix, env, region): Shape | undefined;
/// ```
pub(crate) fn uploads_nothing(
    global_prefix: &[OsString],
    environment: &[(OsString, OsString)],
    region: &[OsString],
) -> Option<PushShape> {
    // What:     `hook_switched_off(...) || skip_requested(...)` reads the two ways a person stops the upload.
    // Why:      A push the person meant to skip must send nothing, and no destination is asked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (hookSwitchedOff(prefix, env) || skipRequested(env)) return undefined;
    // ```
    if hook_switched_off(global_prefix, environment) || skip_requested(environment) {
        return None;
    }
    // What:     `parse_push_region(...).ok().zip(push_shape(region))` keeps the parsed region's facts beside the push's shape,
    //           and answers nothing when either cannot be read.
    // Why:      The facts say whether the upload is bypassed or is a dry run; the shape says what it sends.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pair = parsePushRegion(region, []).ok && pushShape(region); if (!pair) return undefined;
    // ```
    // What:     `?` on the pair answers `None` here when either half is missing, which is the same "uploads nothing".
    // Why:      One way to leave with nothing, so the two readings of the arguments cannot differ.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [regionFacts, shape] = pair; if (!pair) return undefined;
    // ```
    let (region_facts, shape) = parse_push_region(region, &[])
        .ok()
        .zip(push_shape(region))?;
    // What:     `region_facts.no_verify`, `region_facts.dry_run` and `shape.deletes` are the three pushes that send no object.
    // Why:      Asking a destination about a push that sends nothing would cost a connection for no answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (facts.noVerify || facts.dryRun || shape.deletes) return undefined;
    // ```
    if region_facts.no_verify || region_facts.dry_run || shape.deletes {
        return None;
    }
    // What:     `Some(shape)` hands back the push's shape, an owned value.
    // Why:      The questions below read it and nothing else.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return shape;
    // ```
    return Some(shape);
}

/// What:
///  Whether `--recurse-submodules` makes the push carry submodules whose objects this upload does not cover.
/// Why:
///   `on-demand` and `only` are the two values that make Git recurse,
/// and the plan records it so a caller can say the upload does not cover them;
///  both faces read the same value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const submodulesOf = (shape) => ['on-demand', 'only'].includes(shape.recurseSubmodules ?? '');
/// ```
pub(crate) fn submodules_of(shape: &PushShape) -> bool {
    // What:     `match shape.recurse_submodules.as_ref()` opens the optional value the switch was given.
    // Why:      A push without the switch recurses never, which is not the same as a value that is not `on-demand`/`only`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return shape.recurseSubmodules === 'on-demand' || shape.recurseSubmodules === 'only';
    // ```
    return match shape.recurse_submodules.as_ref() {
        Some(value) => matches!(value.to_str(), Some("on-demand" | "only")),
        None => false,
    };
}

/// What:
///  Whether the branch's name has to be asked for:
///  a push that names no repository or no refspec sends the branch Git would,
///  which is what the caller is standing on.
/// Why:
///   The branch is one question of the runtime or of the command's own thread,
/// and both faces ask it only when the plan needs it;
///  a push that names its refspecs never reads `HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const branchNeeded = (shape) => shape.repository === undefined || shape.refspecs.length === 0;
/// ```
pub(crate) fn a_branch_is_needed(shape: &PushShape) -> bool {
    return shape.repository.is_none() || shape.refspecs.is_empty();
}

/// What:
///  The remote the push goes to:
///  the one the caller named,
///  or the one the configuration says a push of this branch goes to.
/// Why:
///   Git's own default for a bare push is `branch.<name>.pushRemote`,
/// then `remote.pushDefault`,
/// then the branch's remote,
///  then `origin`;
///  `default_remote` reads them from the settings the caller already holds,
///  so no face can pick a different destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remoteNameOf = (shape, branch, all) => shape.repository ?? defaultRemote(branch, all);
/// ```
pub(crate) fn remote_name_of(
    shape: &PushShape,
    branch: Option<&str>,
    all: &[(String, String)],
) -> OsString {
    // What:     `match shape.repository.clone()` reads the repository the caller typed, if any.
    // Why:      A named repository is the destination Git would use, and only a bare push falls back to the configuration.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return shape.repository?.clone() ?? OsString.from(defaultRemote(branch, all));
    // ```
    return match shape.repository.clone() {
        Some(typed) => typed,
        // What:     `OsString::from(default_remote(...))` is the configured remote as owned OS text, which may hold bytes no
        //           `String` does.
        // Why:      The remote name travels into Git's arguments, and the configuration is where a person writes it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // None: OsString.from(defaultRemote(branch, all));
        // ```
        None => OsString::from(default_remote(branch, all)),
    };
}

/// What:
///  Whether the push names no refspec of its own,
///  so what it sends is what the configuration decides.
/// Why:
///   A bare push takes `remote.<name>.push`,
/// a mirror's `--all`,
/// `push.default=matching`'s `:`,
/// or the branch;
///  both faces read that decision here rather than each deciding what a bare push means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aBarePush = (shape, everything) => shape.refspecs.length === 0 && !shape.branches && !shape.tags && !everything;
/// ```
pub(crate) fn a_bare_push(
    shape: &PushShape,
    everything: bool,
) -> bool {
    return shape.refspecs.is_empty() && !shape.branches && !shape.tags && !everything;
}

/// What:
///  The refspecs the push sends,
///  and whether it sends everything.
/// Why:
///   These are the arguments' own meaning:
/// `--all`,
/// `--tags`,
/// `--mirror`,
/// the matching refspec `:` and the configured `remote.<name>.push` all stand for refs;
///  a bare push that names none of them sends the branch Git would,
///  or `HEAD` where no branch is named;
///  both faces extend the same list from the same settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const specsOf = (shape, remote, branch, all, mirror) => { const specs = [...shape.refspecs]; if (!bare) return { everything: shape.mirror, specs }; if (mirror) return { everything: true, specs }; const configured = values(all, `remote.${remote}.push`); if (configured.length) specs.push(...configured); else if (setting(all, 'push.default') === 'matching') specs.push(':'); else specs.push(branch ? `refs/heads/${branch}` : 'HEAD'); return { everything: shape.mirror, specs } }
/// ```
pub(crate) fn specs_of(
    shape: &PushShape,
    remote_text: &str,
    branch: Option<&str>,
    all: &[(String, String)],
    everything: bool,
    mirror_is_set: bool,
) -> (bool, Vec<OsString>) {
    // What:     `shape.refspecs.clone()` copies the refspecs the caller typed, which is what a push that names any sends.
    // Why:      The plan reads the list and the arguments must stay readable, so a copy travels.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const specs = [...shape.refspecs];
    // ```
    let mut specs: Vec<OsString> = shape.refspecs.clone();
    // What:     `a_bare_push(shape, everything)` is the one case where the configuration decides.
    // Why:      Every other push sends exactly what it named, whatever the settings say.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!aBarePush(shape, everything)) return { everything, specs };
    // ```
    if !a_bare_push(shape, everything) {
        return (everything, specs);
    }
    // What:     `values(all, "remote.<name>.push")` is what the remote's configuration says a push of it sends.
    // Why:      Those refspecs are what Git would send, and the plan must send the same.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const configured = values(all, `remote.${remote_text}.push`);
    // ```
    let configured: Vec<&str> = values(all, &format!("remote.{remote_text}.push"));
    // What:     `if mirror_is_set` is the remote's own `remote.<name>.mirror`, which the face asked of Git or of the snapshot.
    // Why:      A mirror push sends everything reachable, whatever refspecs the settings hold.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (mirror_is_set) return { everything: true, specs };
    // ```
    if mirror_is_set {
        return (true, specs);
    }
    if !configured.is_empty() {
        specs.extend(configured.into_iter().map(OsString::from));
    } else if setting(all, "push.default") == Some("matching") {
        // What:     `OsString::from(":")` is the matching refspec: every branch of the remote the clone also has.
        // Why:      `push.default=matching` means exactly that, and Git would send it for a bare push too.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // specs.push(':');
        // ```
        specs.push(OsString::from(":"));
    } else {
        // What:     `format!("refs/heads/{name}")` is the branch the caller is on, or `HEAD` where none is named.
        // Why:      That is the branch a bare push sends when nothing else says otherwise.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // specs.push(branch ? `refs/heads/${branch}` : 'HEAD');
        // ```
        specs.push(OsString::from(match branch {
            Some(name) => format!("refs/heads/{name}"),
            None => String::from("HEAD"),
        }));
    }
    // What:     The refspecs and whether the push still sends everything are the answer.
    // Why:      The caller asks the destination about the refs these name, and nothing else.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { everything, specs };
    // ```
    return (everything, specs);
}

/// What:
///  The refs a list holds once,
///  in the order they were found.
/// Why:
///   `--all` and a named branch can name one ref twice,
/// and an upload of one ref twice is a second run of `git-lfs push` for no object;
///  both faces read the refs through this one list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const uniqueRefs = (refs) => refs.filter((ref, at) => refs.indexOf(ref) === at);
/// ```
pub(crate) fn unique_refs(refs: Vec<OsString>) -> Vec<OsString> {
    let mut unique: Vec<OsString> = Vec::new();
    for found in refs {
        if !unique.contains(&found) {
            unique.push(found);
        }
    }
    // What:     `unique` is the same refs with each name once, in the order they were first found.
    // Why:      The plan runs one upload per destination over these refs and no ref twice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return unique;
    // ```
    return unique;
}

/// What:
///  Where an upload's objects go:
///  the remote as the push names it,
///  and one destination per URL it pushes to.
/// Why:
///   Git pushes to every push URL of a remote,
/// and to every URL of a remote that has no push URL,
/// so each of those is a destination of its own:
/// each may keep its objects in its own place.
///  Both faces make the same list from the same settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Destinations = { configured: boolean; pushUrls: string[]; urls: string[]; remotes: OsString[] };
/// ```
pub(crate) struct Destinations<'a> {
    /// Whether the configuration defines this remote at all.
    pub(crate) configured: bool,
    /// The push URLs the remote is configured with.
    pub(crate) push_urls: Vec<&'a str>,
    /// The fetch URLs the remote is configured with.
    pub(crate) urls: Vec<&'a str>,
    /// The destinations `git lfs push` is run for,
    /// one run each.
    pub(crate) remotes: Vec<OsString>,
}

impl Destinations<'_> {
    /// Where the objects go:
    /// the push URLs when the remote has any,
    /// and its fetch URLs otherwise.
    pub(crate) fn places(&self) -> &[&str] {
        return if self.push_urls.is_empty() {
            self.urls.as_slice()
        } else {
            self.push_urls.as_slice()
        };
    }
}

/// What:
///  The destinations of one push of `remote_name`,
///  whose text is `remote_text`.
/// Why:
///   The remote is a destination of its own when it names one place,
/// and one per URL when it names several,
///  so the list is made here rather than on each face.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const destinationsOf = (remoteName, remoteText, directory, all): Destinations => { ... };
/// ```
pub(crate) fn destinations_of<'a>(
    remote_name: &OsString,
    remote_text: &str,
    directory: &Path,
    all: &'a [(String, String)],
) -> Destinations<'a> {
    // What:     `configured_remotes(all).contains(...)` says whether the settings define this remote at all.
    // Why:      A destination the configuration does not define is a URL or a path, and its own place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const configured = configuredRemotes(all).includes(remoteText);
    // ```
    let configured: bool = configured_remotes(all).contains(&remote_text);
    // What:     `values(all, "remote.<name>.pushurl")` and `.url` are the two lists Git takes destinations from.
    // Why:      A push URL is where a push goes, and a remote with none pushes to its fetch URLs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pushUrls = configured ? values(all, `remote.${remoteText}.pushurl`) : [];
    // const urls = configured ? values(all, `remote.${remoteText}.url`) : [];
    // ```
    let push_urls: Vec<&'a str> = if configured {
        values(all, &format!("remote.{remote_text}.pushurl"))
    } else {
        Vec::new()
    };
    let urls: Vec<&'a str> = if configured {
        values(all, &format!("remote.{remote_text}.url"))
    } else {
        Vec::new()
    };
    let mut destinations: Destinations<'a> = Destinations {
        configured,
        push_urls,
        urls,
        remotes: Vec::new(),
    };
    // What:     `destinations.places()` is every URL the objects go to, and it may hold several.
    // Why:      Each of several URLs is a destination of its own, because each may keep its objects in its own place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const places = destinations.places();
    // ```
    let places: &[&str] = destinations.places();
    // What:     `if places.len() > 1` makes one destination per URL, and otherwise the remote name is the one destination.
    // Why:      `git lfs push` takes one destination per run, and a single place is named by the remote itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // destinations.remotes = places.length > 1 ? places.map((url) => lfsDestination(url, directory, false)) : [lfsDestination(remoteName, directory, configured)];
    // ```
    destinations.remotes = if places.len() > 1 {
        let mut each: Vec<OsString> = Vec::new();
        for url in places {
            each.push(lfs_destination(OsStr::new(url), directory, false));
        }
        each
    } else {
        vec![lfs_destination(
            remote_name.as_os_str(),
            directory,
            configured,
        )]
    };
    // What:     `destinations` is the remote's places, with the names the upload is run for.
    // Why:      Both the confirmation question and the plan read this one value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return destinations;
    // ```
    return destinations;
}

/// What:
///  Whether this destination is one the clone both fetches from and pushes to,
///  which is the only kind whose tracking refs it can confirm.
/// Why:
///   A push URL,
/// several URLs,
/// a `pushInsteadOf` rewrite,
/// a URL or path destination and a mirror all break
///  the identity between what the clone fetched and where it pushes,
///  so such a destination is asked nothing and its upload stops nowhere;
///  both faces decide that here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const samePlace = (destinations, everything, all) => !everything && destinations.configured && destinations.pushUrls.length === 0 && destinations.urls.length === 1 && !hasPushRewrite(all);
/// ```
pub(crate) fn same_place(
    destinations: &Destinations<'_>,
    everything: bool,
    all: &[(String, String)],
) -> bool {
    return !everything
        && destinations.configured
        && destinations.push_urls.is_empty()
        && destinations.urls.len() == 1
        && !has_push_rewrite(all);
}

/// What:
///  The plan an upload of these refs to these destinations makes.
/// Why:
///   The last step of both faces:
///  the refs,
///  the destinations,
///  whether the scan is full,
/// the refs the destination confirmed,
///  the server the upload is connected to,
///  and no named objects;
///  assembled here so no field can be filled differently on one face.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thePlan = (destinations, refs, everything, submodules, fullScan, confirmed, lfsServer): UploadPlan => ({ remotes: destinations.remotes, refs, everything, submodules, fullScan, confirmedTrackingRefs: confirmed, objects: undefined, lfsServer });
/// ```
pub(crate) fn the_plan(
    destinations: Destinations<'_>,
    refs: Vec<OsString>,
    everything: bool,
    submodules: bool,
    full_scan: bool,
    confirmed_tracking_refs: Vec<String>,
    lfs_server: Option<String>,
) -> UploadPlan {
    // What:     `UploadPlan { .. }` is the whole plan: the destinations, the refs, whether everything is scanned, whether
    //           submodules are pushed, the full scan, the refs the destination confirmed, no named objects, and the server.
    // Why:      One value travels to the upload, and the two faces must fill it the same way.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { remotes: destinations.remotes, refs, everything, submodules, fullScan, confirmedTrackingRefs: confirmed, objects: undefined, lfsServer };
    // ```
    return UploadPlan {
        remotes: destinations.remotes,
        refs,
        everything,
        submodules,
        full_scan,
        confirmed_tracking_refs,
        objects: None,
        lfs_server,
    };
}
