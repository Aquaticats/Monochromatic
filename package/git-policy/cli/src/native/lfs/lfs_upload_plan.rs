//! What:
//!  What the Git LFS upload before a push sends: the destinations `git lfs push` takes and the refs whose objects it
//!  uploads, with `--all` wherever a remote-tracking ref of this clone may not describe the destination, so that it
//!  cannot hide an object the destination lacks, and in git-lfs's bounded form (no `--all`) everywhere else.
//! Why:
//!  `git lfs push` takes refs and no ref-update lines, so the plan is read from the push's own arguments with the same
//!  option table as the rest of the wrapper, and from the configuration Git itself would read for the parts the
//!  arguments leave out (the default remote, `remote.<name>.push`, `remote.<name>.mirror`, the push URLs,
//!  `push.default`). The configuration comes from the snapshot the caller already holds, or from one query.
//!  Refs that Git expands from a pattern are listed with `git for-each-ref` and matched here with a refspec's own rule
//!  (`*` crosses `/`). The plan fails closed: a query that fails is an error the caller turns into a refusal, never
//!  into "nothing to upload". Only a mirror push uploads every ref.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planUpload(globalPrefix, region, env, directory, settings, query); // { remotes, refs, everything }
//! ```

/// The destination and the named refs of a push.
use crate::command_push::{parse_push_region, push_shape};
/// The configuration the plan reads.
use crate::lfs_upload_settings::{current_branch, default_remote, reads_as_true, setting, values};
pub use crate::lfs_upload_settings::{PlanError, Query, Settings, read_settings, resolve_push_remote, settings_from_snapshot};
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
    /// The destinations `git lfs push` is run for, one run each: the remote name, or one URL per push URL when the
    ///  remote has several (each may keep its objects in its own place).
    pub remotes: Vec<OsString>,
    /// The refs whose objects are uploaded; empty with `everything`.
    pub refs: Vec<OsString>,
    /// Upload everything reachable from every local ref.
    pub everything: bool,
    /// The push also pushes submodules, whose objects this upload does not cover.
    pub submodules: bool,
    /// Scan every object reachable from the refs (`--all`) instead of the ones the remote's tracking refs do not cover.
    ///  Set for every destination whose tracking refs cannot be trusted to describe it, and for a mirror push.
    pub full_scan: bool,
    /// The exact objects to upload (`git-lfs push --object-id`), when the wrapper found them itself; `None` leaves the
    ///  scan to `git-lfs` with the refs.
    pub objects: Option<Vec<String>>,
}

/// How many object names one `git-lfs push --object-id` run takes, so the argument list stays far under the system limit.
pub const OBJECTS_PER_RUN: usize = 500;

impl UploadPlan {
    /// The argument lists of the runs for one destination: one run that scans the refs, or, with named objects, one
    ///  `push --object-id <destination> <oid>...` run per [`OBJECTS_PER_RUN`] objects.
    pub fn runs(&self, remote: &OsStr) -> Vec<Vec<OsString>> {
        let Some(objects) = self.objects.as_ref() else {
            return vec![self.arguments(remote)];
        };
        return objects
            .chunks(OBJECTS_PER_RUN)
            .map(|chunk| {
                let mut arguments: Vec<OsString> = vec![OsString::from("push"), OsString::from("--object-id"), remote.to_owned()];
                arguments.extend(chunk.iter().map(OsString::from));
                return arguments;
            })
            .collect();
    }

    /// The arguments of one `git-lfs` run: `push`, with `--all` for a full scan, the destination, then the refs.
    ///  `--all` with refs uploads every object reachable from them, so objects that remote-tracking refs of this clone
    ///  mention are not skipped; without it `git-lfs` skips them, and a push that adds no object sends no batch request.
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

/// What the source side of a refspec is: nothing for a deletion (`:dst`), the matching marker for `:`, else the part
///  before the last colon (Git splits at the last one, so a source such as `:/message` survives); a leading `+` is dropped.
enum Source {
    /// The refspec only deletes.
    Deletion,
    /// The refspec `:` (or `+:`): every branch that exists on both sides.
    Matching,
    /// A ref, a revision expression or a pattern.
    Named(OsString),
}

/// Read the source of one refspec.
fn refspec_source(spec: &OsStr) -> Source {
    let bytes: &[u8] = spec.as_encoded_bytes();
    let unforced: &[u8] = bytes.strip_prefix(b"+").unwrap_or(bytes);
    if unforced == b":" {
        return Source::Matching;
    }
    let source: &[u8] = match unforced.iter().rposition(|byte| {
        return *byte == b':';
    }) {
        Some(position) => &unforced[..position],
        None => unforced,
    };
    if source.is_empty() {
        return Source::Deletion;
    }
    return Source::Named(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(source.to_vec()));
}

/// Keeps every ref name.
fn keep_all(_name: &str) -> bool {
    return true;
}

/// The refs under a prefix, listed by one `for-each-ref`, kept when `keep` accepts the full name.
fn list(query: Query<'_>, prefix: &str, keep: &dyn Fn(&str) -> bool) -> Result<Vec<OsString>, PlanError> {
    let listing: String = query(&["for-each-ref", "--format=%(refname)", prefix]).map_err(|why| {
        return PlanError(format!("git for-each-ref {prefix}: {why}"));
    })?;
    return Ok(listing
        .lines()
        .filter(|name| {
            return keep(name);
        })
        .map(OsString::from)
        .collect());
}

/// The refs a source with a `*` stands for, by a refspec's own rule: the text before the star, the text after it, and
///  anything between, `/` included (which a `for-each-ref` pattern does not do).
fn expand_pattern(query: Query<'_>, pattern: &str) -> Result<Vec<OsString>, PlanError> {
    let (front, back) = pattern.split_once('*').unwrap_or((pattern, ""));
    let directory: &str = front.rsplit_once('/').map_or("refs", |(head, _)| {
        return head;
    });
    return list(query, directory, &|name| {
        return name.len() >= front.len() + back.len() && name.starts_with(front) && name.ends_with(back);
    });
}

/// `file://` and percent-encoding for a path, where `git-lfs` accepts only a remote name or an URL.
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
///  Turn a destination typed in place of a remote name into what `git lfs push` accepts. A remote name and an URL stay as
///  they are; a path (it starts with `/`, `./` or `../`, or names an existing file or directory relative to `directory`
///  and is not a configured remote) becomes a percent-encoded `file://` URL.
/// Why:
///   Git pushes to a path directly, and the old hook flow handed `git-lfs` the URL Git had made of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isPath(remote) ? pathToFileURL(resolve(directory, remote)) : remote
/// ```
fn lfs_destination(remote: &OsStr, directory: &Path, configured: bool) -> OsString {
    let bytes: &[u8] = remote.as_encoded_bytes();
    let has_scheme: bool = bytes.windows(3).any(|window| {
        return window == b"://";
    });
    let scp_like: bool = bytes.iter().position(|byte| {
        return *byte == b':';
    }).is_some_and(|colon| {
        return !bytes[..colon].contains(&b'/');
    });
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
fn configured_remotes(all: &[(String, String)]) -> Vec<&str> {
    return all
        .iter()
        .filter_map(|(key, _)| {
            let rest: &str = key.strip_prefix("remote.")?;
            let (name, variable) = rest.rsplit_once('.')?;
            return (variable == "url" || variable == "pushurl").then_some(name);
        })
        .collect();
}

/// Whether any `url.<base>.pushInsteadOf` rewrite is configured, which can send the push elsewhere than the fetch.
fn has_push_rewrite(all: &[(String, String)]) -> bool {
    return all.iter().any(|(key, _)| {
        return key.starts_with("url.") && key.ends_with(".pushinsteadof");
    });
}

/// What:
///  What to upload for a push, or nothing when the push uploads nothing, or why the plan cannot be worked out.
/// Why:
///   A push that bypasses the upload (`--no-verify`, `GIT_LFS_SKIP_PUSH`, the caller's switch), sends nothing (a dry run,
///  only deletions) uploads nothing. A push that names refs uploads those, with patterns, `--all`, `--tags`, the matching
///  refspec `:` and the configured `remote.<name>.push` expanded to the refs they stand for; a bare push uploads what the
///  remote's configuration names, else the branch Git would send (`HEAD` when it is not a branch); a mirror push uploads
///  everything. A remote with several push URLs is uploaded to once per URL. `directory` is where relative paths are
///  resolved; `settings` is the configuration the plan reads.
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
    let submodules: bool = shape.recurse_submodules.as_ref().is_some_and(|value| {
        return matches!(value.to_str(), Some("on-demand" | "only"));
    });
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
            Source::Matching => refs.extend(list(query, "refs/heads", &keep_all)?),
            Source::Named(source) => match source.to_str() {
                Some(text) if text.contains('*') => refs.extend(expand_pattern(query, text)?),
                _ => refs.push(source),
            },
        }
    }
    if shape.branches {
        refs.extend(list(query, "refs/heads", &keep_all)?);
    }
    if shape.tags {
        refs.extend(list(query, "refs/tags", &keep_all)?);
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
        push_urls
            .iter()
            .copied()
            .map(|url| {
                return lfs_destination(OsStr::new(url), directory, false);
            })
            .collect()
    } else {
        vec![lfs_destination(remote_name.as_os_str(), directory, configured)]
    };
    // The tracking refs describe the destination only for a configured remote that is fetched from and pushed to the
    //  same place: a push URL, a `pushInsteadOf` rewrite, a URL or path destination or a mirror breaks that.
    let tracking_trusted: bool = configured && push_urls.is_empty() && !has_push_rewrite(all);
    let full_scan: bool = everything || !tracking_trusted;
    return Ok(Some(UploadPlan { remotes, refs: unique, everything, submodules, full_scan, objects: None }));
}

/// The plan is controlled in the test module.
#[cfg(test)]
#[path = "lfs_upload_plan_tests.rs"]
mod tests;
