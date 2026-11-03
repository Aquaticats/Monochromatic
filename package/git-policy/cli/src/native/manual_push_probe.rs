//! What:
//!  The authoritative ref updates a `git push` would send, discovered by a dry-run push whose report Git writes itself.
//! Why:
//!  The manual-push policies must scan exactly what Git would publish, so the probe asks Git:
//!  `git push --dry-run --porcelain` reports each destination and ref update (`manual_push_porcelain`), the local
//!  objects come from that report or from one `rev-parse`, and `git ls-remote --refs` confirms each destination value.
//!  A remote ref that changed between negotiation and confirmation stops the probe.
//!  No program of ours is started for this, and no hook or shell text is written: an earlier version recorded Git's
//!  update lines with a `#!/bin/sh` hook (`doc/planning/cli-git-no-self-invocation.md`).
//!  Every Git child here is bounded by the query deadline, so a remote that never answers cannot hold the push.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const updates = await probeManualPushUpdates({ gitPath, cwd, args });
//! ```

/// The probe's Git runner, its bounded queries and its environment.
use super::auto_push_single_flight::{PushGit, SingleFlightError, run_query};
/// The destination Git would pick for a bare push, and the porcelain report's parser.
use super::lfs_upload_plan::resolve_push_remote;
/// Removes userinfo from text that may name a remote, and the cause a failure carries.
use super::lfs_output_mask::mask_userinfo_text;
use super::manual_push_cause::ManualPushCause;
use super::manual_push_porcelain::{PorcelainUpdate, RemoteBefore, parse_porcelain};
/// The push's own reading of its destination.
use super::command_push::push_shape;
/// Owned OS strings for the probe's Git arguments.
use std::ffi::OsString;

/// What:
///  Git's own refusal of a push: the exit status and the standard error it ended with.
/// Why:
///   When the dry run exits with a failure before it reports any destination, Git has refused the push itself (a ref to
///  delete that the remote lacks, a refspec naming nothing, a remote that does not exist, an option it does not know).
///  Nothing could be published, so Git's words and status are the answer, and the wrapper adds none of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitRefusal = { code: number; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitRefusal {
    /// The exit status Git ended with; never zero.
    pub code: i32,
    /// What Git wrote to standard error, as Git wrote it.
    pub stderr: String,
}

/// What:
///  Why the probe could not establish the updates Git would push.
/// Why:
///   Every variant stops the policy lifecycle; none guesses an update. A failure of the wrapper's own reading is a
///  content-unavailable engine failure, and a refusal by Git carries Git's own report instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ManualPushProbeError extends Error { refusal?: GitRefusal }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualPushProbeError {
    /// Diagnostic naming what could not be established.
    pub message: String,
    /// Git's refusal when Git itself ended the dry run before it reported any destination; `None` for a failure of the
    ///  wrapper's own reading.
    pub refusal: Option<GitRefusal>,
    /// What the person can do about it, which the hint is chosen by.
    pub cause: ManualPushCause,
}

impl std::fmt::Display for ManualPushProbeError {
    /// Writes the diagnostic text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(&self.message);
    }
}

impl std::error::Error for ManualPushProbeError {}

/// Build a probe error from text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new ManualPushProbeError(cause, message)
/// ```
fn probe_error(cause: ManualPushCause, message: String) -> ManualPushProbeError {
    return ManualPushProbeError { message, refusal: None, cause };
}

/// What:
///  The cause of a Git query that could not produce an exit status.
/// Why:
///   A query that never started means the Git the wrapper runs is missing; one stopped at its deadline already says in its
///  message which setting changes the deadline; anything else Git did not explain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error instanceof StartError ? 'git-not-started' : error instanceof TimedOut ? 'self-explained' : 'git-silent'
/// ```
fn cause_of_query(error: &SingleFlightError) -> ManualPushCause {
    return match error {
        SingleFlightError::Start(_) => ManualPushCause::GitNotStarted,
        SingleFlightError::TimedOut { .. } => ManualPushCause::SelfExplained,
        SingleFlightError::Git { .. } | SingleFlightError::Io(_) => ManualPushCause::GitSilent,
    };
}

/// What:
///  One confirmed push update, the public fact the policies read.
/// Why:
///   An absent value is `None`, which replaces the incumbent's `ABSENT_GIT_VALUE` sentinel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushUpdate = { localOid: string | typeof ABSENT_GIT_VALUE; remoteOid: string | typeof ABSENT_GIT_VALUE; remoteName; remoteRef };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushUpdate {
    /// Local object name; `None` for a deletion.
    pub local_oid: Option<String>,
    /// Destination object name confirmed by `ls-remote`; `None` when the ref is absent.
    pub remote_oid: Option<String>,
    /// Destination remote name.
    pub remote_name: String,
    /// Destination ref.
    pub remote_ref: String,
}

/// What:
///  The probe's Git arguments: the invocation's global options, the switches that keep every hook and full object
///  names, then `push` with `--dry-run --porcelain` placed before any `--` terminator.
/// Why:
///   The `--` terminator is the user's; the options Git interprets must stay before it. The probe publishes nothing and
///  must not run the upload hook of the caller's configuration, so configured `pre-push` hooks are off and the
///  traditional hooks directory is one that never exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// probeArgs({ globalPrefix, args });
/// ```
pub fn probe_arguments(global_prefix: &[OsString], post_subcommand: &[OsString]) -> Vec<OsString> {
    let separator = post_subcommand.iter().position(|argument| {
        return argument.as_os_str() == "--";
    });
    let insertion = separator.unwrap_or(post_subcommand.len());
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for setting in [
        format!("core.hooksPath={}", super::hooks_off::NO_HOOKS_DIRECTORY),
        String::from("hook.pre-push.enabled=false"),
        String::from("core.abbrev=no"),
    ] {
        arguments.push(OsString::from("-c"));
        arguments.push(OsString::from(setting));
    }
    arguments.push(OsString::from("push"));
    arguments.extend(post_subcommand[..insertion].iter().cloned());
    // A `-q` written by the caller silences the report this probe reads, so it is cancelled here; the caller's own
    //  push keeps its `-q`.
    arguments.push(OsString::from("--no-quiet"));
    arguments.push(OsString::from("--dry-run"));
    arguments.push(OsString::from("--porcelain"));
    arguments.extend(post_subcommand[insertion..].iter().cloned());
    return arguments;
}

/// Whether text typed as the repository is an URL, an scp-style address or a path, not a remote name.
fn looks_like_location(typed: &str) -> bool {
    return typed.contains("://")
        || typed.starts_with('/')
        || typed.starts_with("./")
        || typed.starts_with("../")
        || typed.split_once(':').is_some_and(|(front, _)| {
            return !front.contains('/');
        });
}

/// One Git query of the probe: its trimmed standard output when it exits with status zero.
fn answer(git: &PushGit, arguments: &[OsString]) -> Option<String> {
    let output = run_query(git, arguments).ok()?;
    if output.code != Some(0) {
        return None;
    }
    return Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned());
}

/// The destination locations of a remote name, one per `To` block Git printed: the push URLs of a configured remote,
///  or the name itself when it is not one (a URL or a path typed in place of a remote).
fn locations_of(git: &PushGit, global_prefix: &[OsString], remote: &str) -> Vec<String> {
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.extend(["remote", "get-url", "--push", "--all", remote].map(OsString::from));
    return match answer(git, arguments.as_slice()) {
        Some(listing) if !listing.is_empty() => listing.lines().map(str::to_owned).collect(),
        Some(_) | None => vec![remote.to_owned()],
    };
}

/// What:
///  The object names of local refs, in order, from one `rev-parse`.
/// Why:
///   A new ref's report line has no object name; the ref's own value is what the push sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await git(['rev-parse', ...names])).split('\n')
/// ```
fn local_objects(git: &PushGit, global_prefix: &[OsString], names: &[&str]) -> Result<Vec<String>, ManualPushProbeError> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("rev-parse"));
    arguments.extend(names.iter().map(OsString::from));
    let listing: String = answer(git, arguments.as_slice())
        .ok_or_else(|| {
            return probe_error(ManualPushCause::ContentMissing, format!("the local objects of {names:?} could not be read"));
        })?;
    let objects: Vec<String> = listing.lines().map(str::to_owned).collect();
    if objects.len() != names.len() {
        return Err(probe_error(ManualPushCause::UnexpectedOutput, format!("rev-parse answered {} names for {}", objects.len(), names.len())));
    }
    return Ok(objects);
}

/// What:
///  Confirm one destination's values with `git ls-remote --refs` and turn each reported update into a public update.
/// Why:
///   A destination that changed after the dry run reported would publish something the scan did not see, so a
///  mismatch between the report and the confirmation is an error, never a silent repair. A report that did not state the
///  remote's value (a deletion, a refusal) is taken from the confirmation alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveRemoteGroup({ gitPath, cwd, remoteLocation, updates }): Promise<PushUpdate[]>;
/// ```
pub fn confirm_group(
    git: &PushGit,
    remote_location: &str,
    remote_name: &str,
    updates: &[(PorcelainUpdate, Option<String>)],
) -> Result<Vec<PushUpdate>, ManualPushProbeError> {
    let mut arguments: Vec<OsString> = vec![
        OsString::from("ls-remote"),
        OsString::from("--refs"),
        OsString::from(remote_location),
    ];
    arguments.extend(updates.iter().map(|(update, _local)| {
        return OsString::from(&update.to);
    }));
    let output = run_query(git, arguments.as_slice())
        .map_err(|error| {
            return probe_error(cause_of_query(&error), mask_userinfo_text(format!("ls-remote of {remote_location} failed: {error}").as_str()));
        })?;
    if output.code != Some(0) {
        return Err(probe_error(
            ManualPushCause::RemoteUnreachable,
            mask_userinfo_text(
                format!("ls-remote of {remote_location} failed: {}", String::from_utf8_lossy(&output.stderr).trim_end()).as_str(),
            ),
        ));
    }
    let listing: String = String::from_utf8_lossy(&output.stdout).into_owned();
    let mut confirmed: Vec<PushUpdate> = Vec::new();
    for (update, local) in updates {
        let authoritative: Option<String> = listing.lines().find_map(|line| {
            return line.split_once('\t').filter(|(_oid, name)| {
                return *name == update.to;
            }).map(|(oid, _name)| {
                return oid.to_owned();
            });
        });
        let changed: bool = match &update.before {
            RemoteBefore::Absent => authoritative.is_some(),
            RemoteBefore::Known(reported) => authoritative.as_ref() != Some(reported),
            RemoteBefore::Unknown => false,
        };
        if changed {
            return Err(probe_error(ManualPushCause::RemoteMoved, format!("Remote ref changed during manual-push discovery: {}", update.to)));
        }
        confirmed.push(PushUpdate {
            local_oid: local.clone(),
            remote_oid: authoritative,
            remote_name: remote_name.to_owned(),
            remote_ref: update.to.clone(),
        });
    }
    return Ok(confirmed);
}

/// What:
///  Discover the exact updates the push would send, confirmed against each destination.
/// Why:
///   The lifecycle's candidates and published-set reads both start from these updates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function probeManualPushUpdates({ gitPath, cwd, args }): Promise<PushUpdate[]>;
/// ```
pub fn probe_manual_push_updates(
    git: &PushGit,
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
) -> Result<Vec<PushUpdate>, ManualPushProbeError> {
    let arguments: Vec<OsString> = probe_arguments(global_prefix, post_subcommand);
    let output = run_query(git, arguments.as_slice()).map_err(|error| {
        return probe_error(cause_of_query(&error), format!("probe push could not run: {error}"));
    })?;
    let report: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let blocks = parse_porcelain(report.as_str()).map_err(|message| {
        return probe_error(ManualPushCause::UnexpectedOutput, message);
    })?;
    if blocks.is_empty() {
        if output.code == Some(0) {
            return Ok(Vec::new());
        }
        let said: String = String::from_utf8_lossy(&output.stderr).into_owned();
        let message: String = mask_userinfo_text(format!("Git could not determine push updates: {}", said.trim_end()).as_str());
        // Git ends `push --dry-run --porcelain` before any destination is reported only when it refuses the push itself
        //  (`transport_push` returns before it prints a block, and `die` and usage errors end the process); a status
        //  from a signal, or a refusal without a word, is not Git saying no.
        return Err(match output.code {
            Some(code) if !said.trim().is_empty() => {
                ManualPushProbeError { message, refusal: Some(GitRefusal { code, stderr: said }), cause: ManualPushCause::SelfExplained }
            }
            Some(_) | None => probe_error(ManualPushCause::GitSilent, message),
        });
    }
    let named = push_shape(post_subcommand).and_then(|shape| {
        return shape.repository;
    });
    let typed_location: Option<String> = named.as_ref().map(|typed| {
        return typed.to_string_lossy().into_owned();
    });
    let mut asking = |query: &[&str]| -> Result<String, String> {
        let mut owned: Vec<OsString> = global_prefix.to_vec();
        owned.extend(query.iter().map(OsString::from));
        let answered = run_query(git, owned.as_slice()).map_err(|error| {
            return error.to_string();
        })?;
        let text: String = String::from_utf8_lossy(answered.stdout.as_slice()).trim().to_owned();
        return match answered.code {
            Some(0) => Ok(text),
            Some(1) if text.is_empty() => Ok(text),
            code => Err(format!("exited with {code:?}")),
        };
    };
    let remote_name: String = resolve_push_remote(named, &mut asking)
        .map_err(|error| {
            return probe_error(ManualPushCause::SelfExplained, format!("Git could not tell which remote this push goes to: {}", error.0));
        })?
        .to_string_lossy()
        .into_owned();
    // A destination the caller typed as an URL or a path is its own location, with its credentials; Git's printed `To`
    //  line has them removed, so it is never used as an address. A remote name costs one query for its push URLs.
    let locations: Vec<String> = match typed_location {
        Some(typed) if looks_like_location(typed.as_str()) => vec![typed],
        _ => locations_of(git, global_prefix, remote_name.as_str()),
    };
    if locations.len() != blocks.len() {
        return Err(probe_error(
            ManualPushCause::SelfExplained,
            mask_userinfo_text(
                format!("Git reported {} destinations but {remote_name} has {} push locations", blocks.len(), locations.len()).as_str(),
            ),
        ));
    }
    let mut confirmed: Vec<PushUpdate> = Vec::new();
    for (block, location) in blocks.iter().zip(locations.iter()) {
        let unnamed: Vec<&str> = block
            .updates
            .iter()
            .filter(|update| {
                return !update.from.is_empty() && update.after.is_none();
            })
            .map(|update| {
                return update.from.as_str();
            })
            .collect();
        let mut resolved = local_objects(git, global_prefix, unnamed.as_slice())?.into_iter();
        let group: Vec<(PorcelainUpdate, Option<String>)> = block
            .updates
            .iter()
            .map(|update| {
                let local: Option<String> = if update.from.is_empty() {
                    None
                } else {
                    update.after.clone().or_else(|| {
                        return resolved.next();
                    })
                };
                return (update.clone(), local);
            })
            .collect();
        confirmed.extend(confirm_group(git, location.as_str(), remote_name.as_str(), group.as_slice())?);
    }
    return Ok(confirmed);
}

/// The probe's arguments and its confirmation rules are controlled in the test module.
#[cfg(test)]
#[path = "manual_push_probe_tests.rs"]
mod tests;
