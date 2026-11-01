//! What:
//!  What the Git LFS upload before a push sends: the remote `git lfs push` takes and the refs whose objects it uploads.
//! Why:
//!  `git lfs push` takes refs and no ref-update lines, so the plan is read from the push's own arguments with the same
//!  option table as the rest of the wrapper, and from the configuration Git itself would read for the parts the
//!  arguments leave out (the default remote, `remote.<name>.push`, `push.default`). Refs that Git expands from a
//!  pattern are expanded with `git for-each-ref`, so the upload covers what the push sends and no more; only a
//!  mirror push and `push.default=matching` (which needs the remote's own refs to narrow) upload everything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planUpload(globalPrefix, region, env, directory, query); // { remote, refs, everything }
//! ```

/// The destination and the named refs of a push.
use super::command_push::{parse_push_region, push_shape};
/// The switch and the skip variable.
use super::lfs_pre_push::{hook_switched_off, skip_requested};
/// Owned OS text for arguments.
use std::ffi::{OsStr, OsString};
/// Locations.
use std::path::Path;

/// A read-only Git query: its trimmed standard output when it exits with status zero.
pub type Query<'a> = &'a mut dyn FnMut(&[&str]) -> Option<String>;

/// What an upload sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UploadPlan = { remote: string; refs: string[]; everything: boolean; submodules: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadPlan {
    /// The remote name or URL `git lfs push` takes first.
    pub remote: OsString,
    /// The refs whose objects are uploaded; empty with `everything`.
    pub refs: Vec<OsString>,
    /// Upload everything reachable from every local ref (`--all`).
    pub everything: bool,
    /// The push also pushes submodules, whose objects this upload does not cover.
    pub submodules: bool,
}

impl UploadPlan {
    /// The arguments of `git-lfs`: `push`, `--all` when everything is sent, the remote, then the refs.
    pub fn arguments(&self) -> Vec<OsString> {
        let mut arguments: Vec<OsString> = vec![OsString::from("push")];
        if self.everything {
            arguments.push(OsString::from("--all"));
        }
        arguments.push(self.remote.clone());
        if !self.everything {
            arguments.extend(self.refs.iter().cloned());
        }
        return arguments;
    }
}

/// The source side of a refspec, or nothing for a deletion (`:dst`); a leading `+` is dropped.
fn refspec_source(spec: &OsStr) -> Option<OsString> {
    let bytes: &[u8] = spec.as_encoded_bytes();
    let unforced: &[u8] = bytes.strip_prefix(b"+").unwrap_or(bytes);
    let source: &[u8] = unforced
        .split(|byte| {
            return *byte == b':';
        })
        .next()
        .unwrap_or_default();
    if source.is_empty() {
        return None;
    }
    return Some(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(source.to_vec()));
}

/// Settings Git reads to pick a push remote and a bare push's refs: `(key, value)` pairs from one configuration read.
fn settings(query: Query<'_>) -> Vec<(String, String)> {
    let listing: String = query(&[
        "config",
        "--get-regexp",
        "^(branch\\..*\\.(pushremote|remote)|remote\\.pushdefault|remote\\..*\\.push|push\\.default)$",
    ])
    .unwrap_or_default();
    return listing
        .lines()
        .filter_map(|line| {
            return line.split_once(' ').map(|(key, value)| {
                return (key.to_owned(), value.trim().to_owned());
            });
        })
        .collect();
}

/// The last value of a setting, the last one written winning as in Git.
fn setting<'a>(all: &'a [(String, String)], key: &str) -> Option<&'a str> {
    return all
        .iter()
        .rev()
        .find(|(name, _)| {
            return name == key;
        })
        .map(|(_, value)| {
            return value.as_str();
        });
}

/// The branch `HEAD` names, from the full ref so that a tag with the same name cannot change the answer.
fn current_branch(query: Query<'_>) -> Option<String> {
    let full: String = query(&["symbolic-ref", "--quiet", "HEAD"])?;
    return full.strip_prefix("refs/heads/").map(str::to_owned);
}

/// The remote Git pushes to when the caller names none: the branch's push remote, the default push remote, the
///  branch's remote, then `origin`.
fn default_remote(branch: Option<&str>, all: &[(String, String)]) -> String {
    let named = |variable: &str| -> Option<String> {
        return branch.and_then(|name| {
            return setting(all, &format!("branch.{name}.{variable}")).map(str::to_owned);
        });
    };
    return named("pushremote")
        .or_else(|| {
            return setting(all, "remote.pushdefault").map(str::to_owned);
        })
        .or_else(|| {
            return named("remote");
        })
        .unwrap_or_else(|| {
            return String::from("origin");
        });
}

/// What:
///  The remote a push goes to: the one the caller named, or the one Git picks for a bare push.
/// Why:
///   The manual-push probe and the upload plan must agree on the destination, so both read it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// named ?? defaultRemote(branch, settings)
/// ```
pub fn resolve_push_remote(named: Option<OsString>, query: Query<'_>) -> OsString {
    if let Some(found) = named {
        return found;
    }
    let branch: Option<String> = current_branch(query);
    let all: Vec<(String, String)> = settings(query);
    return OsString::from(default_remote(branch.as_deref(), all.as_slice()));
}

/// The refs a pattern or a prefix expands to in the local repository, one `for-each-ref` start.
fn expand(query: Query<'_>, patterns: &[&str]) -> Vec<OsString> {
    let mut arguments: Vec<&str> = vec!["for-each-ref", "--format=%(refname)"];
    arguments.extend(patterns);
    return query(arguments.as_slice())
        .unwrap_or_default()
        .lines()
        .map(OsString::from)
        .collect();
}

/// What:
///  Turn the destination a caller typed into what `git lfs push` accepts: a path (it starts with `/`, `./` or `../`)
///  becomes a `file://` URL, which `git-lfs` accepts where it refuses a bare path.
/// Why:
///   Git pushes to a path directly, and the old hook flow handed `git-lfs` the URL Git had made of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^\.{0,2}\//.test(remote) ? pathToFileURL(resolve(directory, remote)) : remote
/// ```
fn lfs_destination(remote: OsString, directory: &Path) -> OsString {
    let bytes: &[u8] = remote.as_encoded_bytes();
    let path_like: bool = bytes.starts_with(b"/") || bytes.starts_with(b"./") || bytes.starts_with(b"../");
    if !path_like {
        return remote;
    }
    let Ok(absolute) = std::path::absolute(directory.join(Path::new(&remote))) else {
        return remote;
    };
    let mut url: OsString = OsString::from("file://");
    url.push(absolute.as_os_str());
    return url;
}

/// What:
///  What to upload for a push, or nothing when the push uploads nothing.
/// Why:
///   A push that bypasses the upload (`--no-verify`, `GIT_LFS_SKIP_PUSH`, the caller's switch), sends nothing (a dry
///  run, only deletions) uploads nothing. A push that names refs uploads those; a pattern, `--all` and `--tags` are
///  expanded to the refs they stand for; a bare push uploads what `remote.<name>.push` names, else the branch Git
///  would send (`HEAD` when the branch has no name). `directory` is where relative paths are resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planUpload(globalPrefix, region, environment, directory, query): UploadPlan | undefined;
/// ```
pub fn plan_upload(
    global_prefix: &[OsString],
    region: &[OsString],
    environment: &[(OsString, OsString)],
    directory: &Path,
    query: Query<'_>,
) -> Option<UploadPlan> {
    if hook_switched_off(global_prefix, environment) || skip_requested(environment) {
        return None;
    }
    let region_facts = parse_push_region(region, &[]).ok()?;
    if region_facts.no_verify || region_facts.dry_run {
        return None;
    }
    let shape = push_shape(region)?;
    if shape.deletes {
        return None;
    }
    let submodules: bool = shape.recurse_submodules.as_ref().is_some_and(|value| {
        return matches!(value.to_str(), Some("on-demand" | "only"));
    });
    let mut everything: bool = shape.mirror;
    let mut specs: Vec<OsString> = shape.refspecs.clone();
    let mut patterns: Vec<String> = Vec::new();
    if shape.branches {
        patterns.push(String::from("refs/heads"));
    }
    if shape.tags {
        patterns.push(String::from("refs/tags"));
    }
    let bare: bool = specs.is_empty() && patterns.is_empty() && !everything;
    let mut remote: Option<OsString> = shape.repository.clone();
    if remote.is_none() || bare {
        let branch: Option<String> = current_branch(query);
        let all: Vec<(String, String)> = settings(query);
        if remote.is_none() {
            remote = Some(OsString::from(default_remote(branch.as_deref(), all.as_slice())));
        }
        if bare {
            let key: String = format!("remote.{}.push", remote.as_ref()?.to_string_lossy());
            let configured: Vec<&str> = all
                .iter()
                .filter(|(name, _)| {
                    return *name == key;
                })
                .map(|(_, value)| {
                    return value.as_str();
                })
                .collect();
            if !configured.is_empty() {
                specs.extend(configured.into_iter().map(OsString::from));
            } else if setting(all.as_slice(), "push.default") == Some("matching") {
                everything = true;
            } else {
                let head: String = match branch.as_deref() {
                    Some(name) => format!("refs/heads/{name}"),
                    None => String::from("HEAD"),
                };
                specs.push(OsString::from(head));
            }
        }
    }
    let mut refs: Vec<OsString> = Vec::new();
    for spec in &specs {
        let Some(source) = refspec_source(spec.as_os_str()) else {
            continue;
        };
        match source.to_str() {
            Some(text) if text.contains('*') => refs.extend(expand(query, &[text])),
            _ => refs.push(source),
        }
    }
    if !patterns.is_empty() {
        let borrowed: Vec<&str> = patterns.iter().map(String::as_str).collect();
        refs.extend(expand(query, borrowed.as_slice()));
    }
    if refs.is_empty() && !everything {
        return None;
    }
    let destination: OsString = lfs_destination(remote?, directory);
    return Some(UploadPlan { remote: destination, refs, everything, submodules });
}

/// The plan is controlled in the test module.
#[cfg(test)]
#[path = "lfs_upload_plan_tests.rs"]
mod tests;
