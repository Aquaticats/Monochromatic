//! What:
//!  The authoritative ref updates a `git push` would send, discovered by a private dry-run push.
//! Why:
//!  The manual-push policies must scan exactly what Git would publish, so the probe asks Git:
//!  a private `pre-push` hook in a private directory records the update lines Git negotiated
//!  (`src/policy-engine/manual-push-hook.ts`), and `git ls-remote --refs` confirms each destination value.
//!  A remote ref that changed between negotiation and confirmation stops the probe.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const updates = await probeManualPushUpdates({ gitPath, cwd, args });
//! ```

/// The probe's Git runner and its environment.
use super::auto_push_single_flight::{PushGit, git_command};
/// Unguessable names for the private probe directory.
use super::random_id::random_uuid;
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// Owned OS strings for the probe's Git arguments.
use std::ffi::OsString;
/// Reading and writing the private hook and its captures.
use std::fs;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Stdio` connects the probe's standard input and output.
use std::process::Stdio;

/// Environment variable naming the capture directory the hook writes to.
pub const CAPTURE_DIRECTORY_ENV: &str = "CLI_GIT_PUSH_FACTS_DIRECTORY";

/// Fields of one `pre-push` update record: local ref, local object, remote ref, remote object.
pub const PRE_PUSH_FIELD_COUNT: usize = 4;

/// Largest standard input the recording hook accepts, in bytes, as the incumbent's hook does.
pub const MAX_HOOK_INPUT_BYTES: u64 = 16 * 1024 * 1024;

/// Private directory mode for the probe directories, shared with the incumbent.
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;

/// Name of the recording hook inside the private hooks directory.
const HOOK_NAME: &str = "pre-push";

/// Suffix of a capture file the hook writes, one per invocation.
const CAPTURE_SUFFIX: &str = ".capture";

/// What:
///  Why the probe could not establish the updates Git would push.
/// Why:
///   Every variant stops the policy lifecycle with a content-unavailable failure; none guesses an update.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ManualPushProbeError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualPushProbeError {
    /// Diagnostic naming what could not be established.
    pub message: String,
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
/// new ManualPushProbeError(message)
/// ```
fn probe_error(message: String) -> ManualPushProbeError {
    return ManualPushProbeError { message };
}

/// What:
///  One update record a hook capture holds, before remote confirmation.
/// Why:
///   The advertised remote object is Git's negotiated value; the confirmation must agree with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProbedPushUpdate = { localRef; localOid; remoteName; remoteLocation; remoteRef; advertisedRemoteOid };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbedUpdate {
    /// Local ref Git pushes from.
    pub local_ref: String,
    /// Local object name, all zeros for a deletion.
    pub local_oid: String,
    /// Destination remote name, as Git passed it to the hook.
    pub remote_name: String,
    /// Destination location, as Git passed it to the hook.
    pub remote_location: String,
    /// Destination ref Git updates.
    pub remote_ref: String,
    /// Destination object Git negotiated, all zeros when the ref is new.
    pub advertised_remote_oid: String,
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
///  Whether a hexadecimal object name is the all-zero absence value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isZeroOid(oid: string): boolean { return /^0+$/.test(oid); }
/// ```
pub fn is_zero_oid(oid: &str) -> bool {
    return !oid.is_empty()
        && oid.bytes().all(|byte| {
            return byte == b'0';
        });
}

/// What:
///  Parse the hook's standard input, one update per line.
/// Why:
///   Git guarantees four space-separated fields per update record; anything else stops the probe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCapture(capture): ProbedPushUpdate[];
/// ```
pub fn parse_update_records(
    input: &str,
    remote_name: &str,
    remote_location: &str,
) -> Result<Vec<ProbedUpdate>, ManualPushProbeError> {
    let mut updates: Vec<ProbedUpdate> = Vec::new();
    for line in input.split('\n').filter(|line| {
        return !line.is_empty();
    }) {
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() != PRE_PUSH_FIELD_COUNT {
            return Err(probe_error(format!(
                "Malformed pre-push update record: {line}"
            )));
        }
        updates.push(ProbedUpdate {
            local_ref: parts[0].to_owned(),
            local_oid: parts[1].to_owned(),
            remote_name: remote_name.to_owned(),
            remote_location: remote_location.to_owned(),
            remote_ref: parts[2].to_owned(),
            advertised_remote_oid: parts[3].to_owned(),
        });
    }
    return Ok(updates);
}

/// What:
///  The probe's Git arguments: the invocation's global options, the hook path override, the switch that turns
///  configured `pre-push` hooks off, then `push` with `--dry-run --verify` placed before any `--` terminator.
/// Why:
///   The `--` terminator is the user's; the options Git interprets must stay before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// probeArgs({ args, hooksDirectory });
/// ```
pub fn probe_arguments(
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
    hooks_directory: &Path,
) -> Vec<OsString> {
    let separator = post_subcommand.iter().position(|argument| {
        return argument.as_os_str() == "--";
    });
    let insertion = separator.unwrap_or(post_subcommand.len());
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("-c"));
    arguments.push(OsString::from(format!(
        "core.hooksPath={}",
        hooks_directory.display()
    )));
    // The probe publishes nothing, so no configured `pre-push` hook may run for it (a configured hook would run once
    //  more per probe, and the LFS upload hook would upload during a dry run).
    //  This switch leaves the recording hook in the private directory alone: it disables configured hooks only.
    arguments.push(OsString::from("-c"));
    arguments.push(OsString::from("hook.pre-push.enabled=false"));
    arguments.push(OsString::from("push"));
    arguments.extend(post_subcommand[..insertion].iter().cloned());
    arguments.push(OsString::from("--dry-run"));
    arguments.push(OsString::from("--verify"));
    arguments.extend(post_subcommand[insertion..].iter().cloned());
    return arguments;
}

/// What:
///  The recording hook's shell program: write the remote name and location, then the update lines, to a
///  new capture file named by the hook's process ID.
/// Why:
///   `set -C` makes the redirection exclusive, so two destinations never overwrite each other's capture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hookSource(): string; // `#!${process.execPath}` plus the capture writer
/// ```
pub fn hook_source() -> String {
    return format!(
        "#!/bin/sh\nset -C\n{{ printf '%s\\n%s\\n' \"$1\" \"$2\"; head -c {}; }} > \"${CAPTURE_DIRECTORY_ENV}/$$.capture\" || exit 1\n",
        MAX_HOOK_INPUT_BYTES + 1
    );
}

/// What:
///  Create a private directory with mode `0700`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdtemp(join(tmpdir(), 'cli-git-push-facts-'));
/// ```
fn create_private_directory(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    return fs::DirBuilder::new()
        .mode(PRIVATE_DIRECTORY_MODE)
        .create(path);
}

/// What:
///  Whether a capture holds more update text than the limit allows.
/// Why:
///   The hook reads one byte past the limit, so exactly the limit is complete input and one more byte is proof
///  that input was cut short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exceedsCaptureLimit = (length: number) => length > MAX_HOOK_INPUT_BYTES;
/// ```
pub fn exceeds_capture_limit(length: usize) -> bool {
    return length as u64 > MAX_HOOK_INPUT_BYTES;
}

/// What:
///  Run the private dry-run push and read back the updates its hook captured.
/// Why:
///   Git may exit non-zero after the hook has captured the negotiated updates, so the captures decide,
///  and an empty capture set is a probe failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureProbedPushUpdates({ gitPath, cwd, args }): Promise<ProbedPushUpdate[]>;
/// ```
pub fn capture_probed_updates(
    git: &PushGit,
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
    scratch: &Path,
) -> Result<Vec<ProbedUpdate>, ManualPushProbeError> {
    let hooks = scratch.join("hooks");
    let captures = scratch.join("captures");
    let created = |directory: &Path| -> Result<(), ManualPushProbeError> {
        return create_private_directory(directory).map_err(|error| {
            return probe_error(format!(
                "private probe directory {} could not be created: {error}",
                directory.display()
            ));
        });
    };
    // The scratch directory is created first, and its guard follows at once, so every later failure removes it.
    // A scratch path that already exists is never adopted: creating it fails before any guard exists.
    created(scratch)?;
    let _scratch_guard = ScratchGuard(scratch.to_path_buf());
    created(hooks.as_path())?;
    created(captures.as_path())?;
    let hook_path = hooks.join(HOOK_NAME);
    fs::write(&hook_path, hook_source()).map_err(|error| {
        return probe_error(format!("hook write failed: {error}"));
    })?;
    set_executable(&hook_path)?;
    let arguments = probe_arguments(global_prefix, post_subcommand, &hooks);
    let mut command = git_command(git, &arguments);
    command
        .stdin(Stdio::null())
        .env(CAPTURE_DIRECTORY_ENV, &captures);
    let probe = command.output_retrying().map_err(|error| {
        return probe_error(format!("probe push could not start: {error}"));
    })?;
    let mut names: Vec<PathBuf> = fs::read_dir(&captures)
        .map_err(|error| {
            return probe_error(format!("captures unreadable: {error}"));
        })?
        .map(|entry| {
            return entry.map(|found| {
                return found.path();
            });
        })
        .collect::<Result<Vec<PathBuf>, std::io::Error>>()
        .map_err(|error| {
            return probe_error(format!("captures unreadable: {error}"));
        })?;
    names.retain(|path| {
        return path.to_string_lossy().ends_with(CAPTURE_SUFFIX);
    });
    names.sort();
    if names.is_empty() {
        return Err(probe_error(format!(
            "Git could not determine push updates: {}",
            String::from_utf8_lossy(&probe.stderr).trim_end()
        )));
    }
    let mut updates: Vec<ProbedUpdate> = Vec::new();
    for path in names {
        let bytes = fs::read(&path).map_err(|error| {
            return probe_error(format!("capture unreadable: {error}"));
        })?;
        let text = String::from_utf8_lossy(&bytes);
        let mut lines = text.splitn(3, '\n');
        let remote_name = lines.next().unwrap_or_default();
        let remote_location = lines.next().unwrap_or_default();
        let input = lines.next().unwrap_or_default();
        if exceeds_capture_limit(input.len()) {
            return Err(probe_error(
                "Pre-push input exceeded capture limit.".to_owned(),
            ));
        }
        updates.extend(parse_update_records(input, remote_name, remote_location)?);
    }
    return Ok(updates);
}

/// What:
///  Removes the private probe directory when the probe ends, on every path.
/// Why:
///   Captures hold the names and tips of unpublished refs and must not outlive the probe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using directory = await createProbeDirectory();
/// ```
struct ScratchGuard(PathBuf);

impl Drop for ScratchGuard {
    /// Removes the directory; a failure leaves only a private directory in the temporary area.
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

/// Mark a file executable by its owner only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await chmod(hookPath, 0o700);
/// ```
fn set_executable(path: &Path) -> Result<(), ManualPushProbeError> {
    use std::os::unix::fs::PermissionsExt;
    return fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE)).map_err(
        |error| {
            return probe_error(format!("hook mode failed: {error}"));
        },
    );
}

/// Parse `git ls-remote --refs` records into a ref-to-object map.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemoteRefs(output: string): ReadonlyMap<string, string>;
/// ```
pub fn parse_remote_refs(output: &str) -> Result<Vec<(String, String)>, ManualPushProbeError> {
    let mut refs: Vec<(String, String)> = Vec::new();
    for line in output.split('\n').filter(|line| {
        return !line.is_empty();
    }) {
        let Some((oid, name)) = line.split_once('\t') else {
            return Err(probe_error(format!("Malformed ls-remote record: {line}")));
        };
        refs.push((name.to_owned(), oid.to_owned()));
    }
    return Ok(refs);
}

/// What:
///  Confirm one destination's values with `git ls-remote --refs` and turn each probed update into a public update.
/// Why:
///   A destination that changed after negotiation would publish something the scan did not see,
///  so a mismatch between negotiation and confirmation is an error, never a silent repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveRemoteGroup({ gitPath, cwd, remoteLocation, updates }): Promise<PushUpdate[]>;
/// ```
pub fn confirm_group(
    git: &PushGit,
    remote_location: &str,
    updates: &[ProbedUpdate],
) -> Result<Vec<PushUpdate>, ManualPushProbeError> {
    let mut arguments: Vec<OsString> = vec![
        OsString::from("ls-remote"),
        OsString::from("--refs"),
        OsString::from(remote_location),
    ];
    arguments.extend(updates.iter().map(|update| {
        return OsString::from(&update.remote_ref);
    }));
    let output = git_command(git, &arguments)
        .stdin(Stdio::null())
        .output_retrying()
        .map_err(|error| {
            return probe_error(format!("ls-remote could not start: {error}"));
        })?;
    if !output.status.success() {
        return Err(probe_error(format!(
            "ls-remote of {remote_location} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim_end()
        )));
    }
    let remote_refs = parse_remote_refs(&String::from_utf8_lossy(&output.stdout))?;
    let mut confirmed: Vec<PushUpdate> = Vec::new();
    for update in updates {
        let authoritative: Option<&String> = remote_refs
            .iter()
            .find(|(name, _oid)| {
                return name == &update.remote_ref;
            })
            .map(|(_name, oid)| {
                return oid;
            });
        let advertised_absent = is_zero_oid(&update.advertised_remote_oid);
        if authoritative.is_none() != advertised_absent {
            return Err(probe_error(format!(
                "Remote ref changed during manual-push discovery: {}",
                update.remote_ref
            )));
        }
        if let Some(oid) = authoritative
            && *oid != update.advertised_remote_oid
        {
            return Err(probe_error(format!(
                "Remote ref changed during manual-push discovery: {}",
                update.remote_ref
            )));
        }
        confirmed.push(PushUpdate {
            local_oid: if is_zero_oid(&update.local_oid) {
                None
            } else {
                Some(update.local_oid.clone())
            },
            remote_oid: authoritative.cloned(),
            remote_name: update.remote_name.clone(),
            remote_ref: update.remote_ref.clone(),
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
    let scratch = std::env::temp_dir().join(format!(
        "cli-git-push-facts-{}",
        random_uuid().map_err(|error| {
            return probe_error(format!("random source failed: {}", error.0));
        })?
    ));
    let probed = capture_probed_updates(git, global_prefix, post_subcommand, &scratch)?;
    let mut locations: Vec<String> = Vec::new();
    for update in &probed {
        if !locations.contains(&update.remote_location) {
            locations.push(update.remote_location.clone());
        }
    }
    let mut confirmed: Vec<PushUpdate> = Vec::new();
    for location in &locations {
        let group: Vec<ProbedUpdate> = probed
            .iter()
            .filter(|update| {
                return &update.remote_location == location;
            })
            .cloned()
            .collect();
        confirmed.extend(confirm_group(git, location, &group)?);
    }
    return Ok(confirmed);
}

/// The probe's parsers and its confirmation rules are controlled in the test module.
#[cfg(test)]
#[path = "manual_push_probe_tests.rs"]
mod tests;
