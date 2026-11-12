//! What:
//!  What the Git LFS upload before a push sends:
//! the destinations `git lfs push` takes and the refs whose objects it
//!  uploads,
//! with `--all` wherever a remote-tracking ref of this clone may not describe the destination,
//! so that it
//!  cannot hide an object the destination lacks,
//! and in git-lfs's bounded form (no `--all`) everywhere else.
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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planUpload(globalPrefix, region, env, directory, settings, query); // { remotes, refs, everything }
//! ```

/// The destination and the named refs of a push.
use crate::command_push::{parse_push_region, push_shape};
/// Whether the destination confirms this clone's tracking refs.
use crate::lfs_destination_confirms_tracking_refs::tracking_refs_describe_destination;
/// The configuration the plan reads.
use crate::lfs_upload_settings::{current_branch, default_remote, git_says_true, setting, values};
/// The whole plan awaited by a caller that is itself a task of the runtime.
pub use crate::lfs_upload_plan_awaiting::plan_upload_awaiting;
pub use crate::lfs_upload_settings::{
    GitQuery, GitQueryAwaiting, PlanError, Query, QueryAwaiting, Settings, read_settings, read_settings_awaiting, resolve_push_remote, settings_from_snapshot,
};
/// The switch and the skip variable.
use crate::lfs_pre_push::{hook_switched_off, skip_requested};
/// Owned OS text for arguments.
use std::ffi::{OsStr, OsString};
/// Locations.
use std::path::Path;

/// What an upload sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UploadPlan = { remotes: string[]; refs: string[]; everything: boolean; submodules: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadPlan {
    /// The destinations `git lfs push` is run for,
    /// one run each:
    /// the remote name,
    /// or one URL per push URL when the
    ///  remote has several (each may keep its objects in its own place).
    pub remotes: Vec<OsString>,
    /// The refs whose objects are uploaded;
    /// empty with `everything`.
    pub refs: Vec<OsString>,
    /// Upload everything reachable from every local ref.
    pub everything: bool,
    /// The push also pushes submodules,
    /// whose objects this upload does not cover.
    pub submodules: bool,
    /// Scan every object reachable from the refs (`--all`) instead of the ones the remote's tracking refs do not cover.
    ///  Set for every destination whose tracking refs cannot be trusted to describe it,
    /// and for a mirror push.
    pub full_scan: bool,
    /// The exact objects to upload (`git-lfs push --object-id`),
    /// when the wrapper found them itself;
    /// `None` leaves the
    ///  scan to `git-lfs` with the refs.
    pub objects: Option<Vec<String>>,
}

/// How many object names one `git-lfs push --object-id` run takes,
/// so the argument list stays far under the system limit.
pub const OBJECTS_PER_RUN: usize = 500;

impl UploadPlan {
    /// The argument lists of the runs for one destination:
    /// one run that scans the refs,
    /// or,
    /// with named objects,
    /// one
    ///  `push --object-id <destination> <oid>...` run per [`OBJECTS_PER_RUN`] objects.
    pub fn runs(&self, remote: &OsStr) -> Vec<Vec<OsString>> {
        let Some(objects) = self.objects.as_ref() else {
            return vec![self.arguments(remote)];
        };
        let mut runs: Vec<Vec<OsString>> = Vec::new();
        for chunk in objects.chunks(OBJECTS_PER_RUN) {
            let mut arguments: Vec<OsString> = vec![OsString::from("push"), OsString::from("--object-id"), remote.to_owned()];
            arguments.extend(chunk.iter().map(OsString::from));
            runs.push(arguments);
        }
        return runs;
    }

    /// The arguments of one `git-lfs` run:
    /// `push`,
    /// with `--all` for a full scan,
    /// the destination,
    /// then the refs.
    ///  `--all` with refs uploads every object reachable from them,
    /// so objects that remote-tracking refs of this clone
    ///  mention are not skipped;
    /// without it `git-lfs` skips them,
    /// and a push that adds no object sends no batch request.
    pub fn arguments(&self, remote: &OsStr) -> Vec<OsString> {
        let mut arguments: Vec<OsString> = vec![OsString::from("push")];
        if self.full_scan {
            arguments.push(OsString::from("--all"));
        }
        arguments.push(remote.to_owned());
        if !self.everything {
            arguments.extend(self.refs.iter().cloned());
        }
        return arguments;
    }
}

/// What the source side of a refspec is:
/// nothing for a deletion (`:dst`),
/// the matching marker for `:`,
/// else the part
///  before the last colon (Git splits at the last one,
/// so a source such as `:/message` survives);
/// a leading `+` is dropped.
pub(crate) enum Source {
    /// The refspec only deletes.
    Deletion,
    /// The refspec `:` (or `+:`):
    /// every branch that exists on both sides.
    Matching,
    /// A ref,
    /// a revision expression or a pattern.
    Named(OsString),
}

/// Read the source of one refspec.
pub(crate) fn refspec_source(spec: &OsStr) -> Source {
    let bytes: &[u8] = spec.as_encoded_bytes();
    let unforced: &[u8] = bytes.strip_prefix(b"+").unwrap_or(bytes);
    if unforced == b":" {
        return Source::Matching;
    }
    let source: &[u8] = match unforced.iter().rposition(is_colon_byte) {
        Some(position) => &unforced[..position],
        None => unforced,
    };
    if source.is_empty() {
        return Source::Deletion;
    }
    return Source::Named(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(source.to_vec()));
}

/// Whether one byte is the colon that separates the source of a refspec from its destination,
/// or the host of an
///  scp-like remote from its path.
/// `&u8` borrows the byte,
/// as `position` hands it over.
fn is_colon_byte(byte: &u8) -> bool {
    return *byte == b':';
}

/// The refs under a prefix,
/// listed by one `for-each-ref`;
/// with a `(front, back)` pattern,
/// only the full names that
///  start with `front` and end with `back` without the two overlapping,
/// and with `None` every name.
fn list(query: Query<'_>, prefix: &str, pattern: Option<(&str, &str)>) -> Result<Vec<OsString>, PlanError> {
    let listing: String = match query.ask(&["for-each-ref", "--format=%(refname)", prefix]) {
        Ok(text) => text,
        Err(why) => return Err(PlanError(format!("git for-each-ref {prefix}: {why}"))),
    };
    return Ok(names_of(listing.as_str(), pattern));
}

/// What:
///  The names one finished listing keeps,
/// by the pattern the caller asked for.
/// Why:
///   Both faces of the listing keep the same names here,
/// so a pattern either face reads means the same refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listing.split('\n').filter((name) => !pattern || (name.length >= front.length + back.length && name.startsWith(front) && name.endsWith(back)));
/// ```
pub(crate) fn names_of(listing: &str, pattern: Option<(&str, &str)>) -> Vec<OsString> {
    let mut names: Vec<OsString> = Vec::new();
    for name in listing.lines() {
        let kept: bool = match pattern {
            Some((front, back)) => {
                name.len() >= front.len() + back.len() && name.starts_with(front) && name.ends_with(back)
            }
            None => true,
        };
        if kept {
            names.push(OsString::from(name));
        }
    }
    return names;
}

/// What:
///  The directory a `for-each-ref` pattern of a refspec is listed under.
/// Why:
///   Both faces of the expansion list the same directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// front.lastIndexOf('/') >= 0 ? front.slice(0, front.lastIndexOf('/')) : 'refs';
/// ```
pub(crate) fn directory_of(pattern: &str) -> &str {
    let (front, _back) = star_split(pattern);
    return match front.rsplit_once('/') {
        Some((head, _)) => head,
        None => "refs",
    };
}

/// What:
///  The text before a refspec pattern's star and the text after it.
/// Why:
///   Both faces of the expansion filter on the same two pieces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pattern.includes('*') ? pattern.split('*') : [pattern, ''];
/// ```
pub(crate) fn star_split(pattern: &str) -> (&str, &str) {
    return pattern.split_once('*').unwrap_or((pattern, ""));
}

/// The refs a source with a `*` stands for,
/// by a refspec's own rule:
/// the text before the star,
/// the text after it,
/// and
///  anything between,
/// `/` included (which a `for-each-ref` pattern does not do).
pub(crate) fn expand_pattern(query: Query<'_>, pattern: &str) -> Result<Vec<OsString>, PlanError> {
    return list(query, directory_of(pattern), Some(star_split(pattern)));
}

/// `file://` and percent-encoding for a path,
/// where `git-lfs` accepts only a remote name or an URL.
fn file_url(absolute: &Path) -> OsString {
    let mut url: Vec<u8> = b"file://".to_vec();
    for byte in absolute.as_os_str().as_encoded_bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(byte) {
            url.push(*byte);
        } else {
            url.extend_from_slice(format!("%{byte:02X}").as_bytes());
        }
    }
    return <OsString as std::os::unix::ffi::OsStringExt>::from_vec(url);
}

/// What:
///  Turn a destination typed in place of a remote name into what `git lfs push` accepts.
/// A remote name and an URL stay as
///  they are;
/// a path (it starts with `/`,
/// `./` or `../`,
/// or names an existing file or directory relative to `directory`
///  and is not a configured remote) becomes a percent-encoded `file://` URL.
/// Why:
///   Git pushes to a path directly,
/// and the old hook flow handed `git-lfs` the URL Git had made of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isPath(remote) ? pathToFileURL(resolve(directory, remote)) : remote
/// ```
pub(crate) fn lfs_destination(remote: &OsStr, directory: &Path, configured: bool) -> OsString {
    let bytes: &[u8] = remote.as_encoded_bytes();
    let mut has_scheme: bool = false;
    for window in bytes.windows(3) {
        if window == b"://" {
            has_scheme = true;
            break;
        }
    }
    let scp_like: bool = match bytes.iter().position(is_colon_byte) {
        Some(colon) => !bytes[..colon].contains(&b'/'),
        None => false,
    };
    let explicit_path: bool = bytes.starts_with(b"/") || bytes.starts_with(b"./") || bytes.starts_with(b"../");
    if configured || has_scheme || (scp_like && !explicit_path) {
        return remote.to_owned();
    }
    let candidate = directory.join(Path::new(remote));
    if !explicit_path && !candidate.exists() {
        return remote.to_owned();
    }
    return match std::path::absolute(&candidate) {
        Ok(absolute) => file_url(absolute.as_path()),
        Err(_) => remote.to_owned(),
    };
}

/// The remotes the configuration defines (a `url` or a `pushurl`).
pub(crate) fn configured_remotes(all: &[(String, String)]) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    for (key, _) in all {
        let Some(rest) = key.strip_prefix("remote.") else {
            continue;
        };
        let Some((name, variable)) = rest.rsplit_once('.') else {
            continue;
        };
        if variable == "url" || variable == "pushurl" {
            names.push(name);
        }
    }
    return names;
}

/// Whether any `url.<base>.pushInsteadOf` rewrite is configured,
/// which can send the push elsewhere than the fetch.
pub(crate) fn has_push_rewrite(all: &[(String, String)]) -> bool {
    for (key, _) in all {
        if key.starts_with("url.") && key.ends_with(".pushinsteadof") {
            return true;
        }
    }
    return false;
}

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
/// `settings` is the configuration the plan reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planUpload(globalPrefix, region, environment, directory, settings, query): UploadPlan | undefined; // throws PlanError
/// ```
pub fn plan_upload(
    global_prefix: &[OsString],
    region: &[OsString],
    environment: &[(OsString, OsString)],
    directory: &Path,
    all: &[(String, String)],
    query: Query<'_>,
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
    let branch: Option<String> = if branch_needed { current_branch(query)? } else { None };
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
        if git_says_true(all, format!("remote.{remote_text}.mirror").as_str(), query)? {
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
    let urls: Vec<&str> = if configured { values(all, &format!("remote.{remote_text}.url")) } else { Vec::new() };
    // Git pushes to every push URL of a remote, and to every URL of a remote that has no push URL, so each of those is a
    //  destination of its own: each may keep its objects in its own place.
    let places: &[&str] = if push_urls.is_empty() { urls.as_slice() } else { push_urls.as_slice() };
    let remotes: Vec<OsString> = if places.len() > 1 {
        let mut destinations: Vec<OsString> = Vec::new();
        for url in places {
            destinations.push(lfs_destination(OsStr::new(url), directory, false));
        }
        destinations
    } else {
        vec![lfs_destination(remote_name.as_os_str(), directory, configured)]
    };
    // The tracking refs describe the destination only for a configured remote that is fetched from and pushed to the
    //  same place: a push URL, several URLs, a `pushInsteadOf` rewrite, a URL or path destination, a mirror, and a
    //  destination that does not report the same branches at the same commits break that.
    let tracking_trusted: bool = !everything
        && configured
        && push_urls.is_empty()
        && urls.len() == 1
        && !has_push_rewrite(all)
        && tracking_refs_describe_destination(remote_text.as_str(), query);
    let full_scan: bool = everything || !tracking_trusted;
    return Ok(Some(UploadPlan { remotes, refs: unique, everything, submodules, full_scan, objects: None }));
}

/// The plan is controlled in the test module.
#[cfg(test)]
#[path = "lfs_upload_plan_tests.rs"]
mod tests;
