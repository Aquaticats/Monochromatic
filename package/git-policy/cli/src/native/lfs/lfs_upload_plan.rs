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
/// The configuration the plan reads.
use crate::lfs_upload_settings::{current_branch, default_remote, reads_as_true, setting, values};
/// The host a URL names,
/// with any userinfo left out.
use crate::lfs_output_mask::host_of;
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
    /// The server the upload is connected to,
    /// as its host and port with any userinfo left out,
    /// when the configuration names one:
    /// what a stop that held the connection names,
    /// so its reader knows which server did not answer.
    pub lfs_server: Option<String>,
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

/// The source of one refspec,
/// and the refs a pattern of it names,
/// which `lfs_upload_refspecs.rs` owns.
pub(crate) use crate::lfs_upload_refspecs::{Source, directory_of, expand_pattern, list, names_of, refspec_source, star_split};
/// The one test of a colon,
/// which the destination of an upload reads too.
use crate::lfs_upload_refspecs::is_colon_byte;

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

/// What:
///  The server one upload is connected to,
///  as its host and port with any userinfo left out,
///  in the order git-lfs 3.8.1 itself picks that server for an upload.
/// Why:
///   A stop of an upload says which server held the connection,
///  and the server of an upload is not the push's destination:
///  git-lfs's own order,
/// read at `lfsapi/endpoint_finder.go:93-140` (`getEndpoint`,
/// `RemoteEndpoint`)
///  and `:190-201` (`GitRemoteURL`),
/// is
///  `lfs.pushurl`,
///  then `lfs.url`,
///  then the named remote's `lfspushurl`,
/// `lfsurl`,
/// `pushurl` and `url`,
///  then the same four of `origin` when the named remote answered nothing.
///  Measured against git-lfs 3.8.1:
/// with `lfs.pushurl` and `lfs.url` both set the batch request went to `lfs.pushurl`,
///  with `lfs.url` and `remote.origin.lfsurl` both set it went to `lfs.url`,
///  and with only `remote.origin.lfsurl` set git-lfs sent nothing there.
///  A destination that is a URL is not an endpoint of its own:
///  `git lfs push` takes a remote name and not a URL (`commands/command_push.go`),
///  so the configuration answers for the upload and the URL only names where Git pushes.
///  A key holding no URL (a remote name that is no remote,
///  a path) answers nothing,
///  so the stop falls back to the words of the program that saw the server.
///  Every endpoint is rewritten through the URL alias tables before its host is read,
///  as git-lfs rewrites it before parsing (`NewEndpoint` calls it,
/// `lfsapi/endpoint_finder.go:234-235`;
///  `ReplaceUrlAlias`,
/// `:345-357`):
///  `url.<base>.pushInsteadOf` first for an upload,
///  `url.<base>.insteadOf` after it,
///  so the stop names the host the upload contacted rather than the alias it was configured with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lfsServerOf(remote: string, all: Settings): string | undefined
/// ```
pub fn lfs_server_of(remote: &OsStr, all: &[(String, String)]) -> Option<String> {
    let text: String = remote.to_string_lossy().into_owned();
    let push_aliases: Vec<(&str, &str)> = url_aliases(all, true);
    let fetch_aliases: Vec<(&str, &str)> = url_aliases(all, false);
    let endpoint_host = |value: &str| -> Option<String> {
        let rewritten: Option<String> = replace_url_alias(value, push_aliases.as_slice())
            .or_else(|| { return replace_url_alias(value, fetch_aliases.as_slice()); });
        return host_of(rewritten.as_deref().unwrap_or(value));
    };
    // The two global keys answer first, whatever the remote's own keys say (git-lfs `getEndpoint`, lines 99 and 104).
    for key in ["lfs.pushurl", "lfs.url"] {
        if let Some(value) = setting(all, key)
            && let Some(host) = endpoint_host(value)
        {
            return Some(host);
        }
    }
    // A named remote other than `origin` answers with its own keys, and `origin` answers behind it when they are silent.
    let mut names: Vec<&str> = Vec::new();
    if !text.is_empty() && text != "origin" {
        names.push(text.as_str());
    }
    names.push("origin");
    for name in names {
        for key in [format!("remote.{name}.lfspushurl"), format!("remote.{name}.lfsurl")] {
            if let Some(value) = setting(all, key.as_str())
                && let Some(host) = endpoint_host(value)
            {
                return Some(host);
            }
        }
        // `git-lfs` reads one value per key and the last one written wins (`config/git_fetcher.go:121-128`).
        for value in [setting(all, &format!("remote.{name}.pushurl")), setting(all, &format!("remote.{name}.url"))] {
            if let Some(host) = value.and_then(|endpoint| { return endpoint_host(endpoint); }) {
                return Some(host);
            }
        }
    }
    return None;
}

/// What:
///  The alias table git-lfs applies to an endpoint of one operation,
///  as `(alias, replacement)` pairs.
/// Why:
///  `url.<base>.pushInsteadOf` and `url.<base>.insteadOf` rewrite a URL before git-lfs parses it
///  (`lfsapi/endpoint_finder.go:234-235`),
///  and an upload reads the push table first and the fetch table after it (`ReplaceUrlAlias`,
/// `:345-357`).
///  A key's base is the replacement and its value is the alias;
///  a later value of one alias wins,
/// as it does in git-lfs's own table (`storeAlias`,
/// `:399-407`).
///  An empty alias is left out because git-lfs never applies one:
///  it keeps the empty value and rewrites only when the longest matching alias is longer than nothing
///  (`replaceUrlAlias`,
/// `:361-378`),
///  so the endpoint it contacts is the unrewritten URL;
///  Git does rewrite it (`git remote get-url --push` shows the replacement),
/// and this table follows git-lfs because
///  git-lfs is the program that makes the request.
///  An empty base is kept,
/// because it is a replacement of nothing and git-lfs applies it:
///  the endpoint is the URL with the alias taken off its front,
///  and the host read from it is the one inside whatever follows the alias
///  (measured with git-lfs 3.8.1:
/// `lfs.url=https://alias.invalid/https://127.0.0.1:39506/lfs` under an empty base
///  `https://alias.invalid/` gave `Endpoint=https://127.0.0.1:39506/lfs`),
///  so leaving it out would name the alias's host,
/// which git-lfs never contacts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function urlAliases(all: Settings, push: boolean): Array<[alias: string, replacement: string]>
/// ```
fn url_aliases<'a>(all: &'a [(String, String)], push: bool) -> Vec<(&'a str, &'a str)> {
    let suffix: &str = if push { ".pushinsteadof" } else { ".insteadof" };
    let mut aliases: Vec<(&'a str, &'a str)> = Vec::new();
    // The last entry of a key wins, so the entries are read backwards and one alias is kept once.
    for (key, value) in all.iter().rev() {
        let Some(base) = key.strip_prefix("url.").and_then(|rest| { return rest.strip_suffix(suffix); }) else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        if aliases.iter().any(|(alias, _replacement)| { return *alias == value.as_str(); }) {
            continue;
        }
        aliases.push((value.as_str(), base));
    }
    return aliases;
}

/// What:
///  `rawurl` with the longest matching alias's replacement in place of its prefix,
///  or nothing when no alias matches.
/// Why:
///  git-lfs chooses the longest alias that prefixes the URL (`replaceUrlAlias`,
/// `:361-378`),
///  so a narrower rule can rewrite a subpath of a broader one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceUrlAlias(rawurl: string, aliases: Array<[alias: string, replacement: string]>): string | undefined
/// ```
fn replace_url_alias(rawurl: &str, aliases: &[(&str, &str)]) -> Option<String> {
    let mut longest: Option<(&str, &str)> = None;
    for (alias, replacement) in aliases {
        if !rawurl.starts_with(alias) {
            continue;
        }
        if longest.is_none_or(|(found, _replacement)| { return alias.len() > found.len(); }) {
            longest = Some((alias, replacement));
        }
    }
    return longest.map(|(alias, replacement)| { return format!("{replacement}{}", &rawurl[alias.len()..]); });
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
    let lfs_server: Option<String> = lfs_server_of(remote_name.as_os_str(), all);
    return Ok(Some(UploadPlan { remotes, refs: unique, everything, submodules, full_scan, objects: None, lfs_server }));
}

/// The plan is controlled in the test module.
#[cfg(test)]
#[path = "lfs_upload_plan_tests.rs"]
mod tests;
