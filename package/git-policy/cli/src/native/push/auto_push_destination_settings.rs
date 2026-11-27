//! What:
//!  The settings that decide where one branch is pushed,
//!  read from one listing of the configuration:
//!  the remote the push goes to,
//!  the branches it tracks,
//!  the push mode and the configured refspecs.
//! Why:
//!  Every value of the multi-valued keys is needed,
//!  which a `config --get` (the last value only) cannot give;
//!  and both faces of the read,
//!  the synchronous one and the one a task of the runtime awaits,
//!  turn that one listing into one set of settings here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type PushSettings = { branch: string; pushRemote: string; trackedRemote: string; merges: string[]; mode: string; refspecs: string[]; mirror: boolean };
//! await readPushSettings({ git, branchRef }): Promise<PushSettings | undefined>;
//! ```

/// Import the probe's Git runner and its two faces of one query.
use crate::auto_push_single_flight::{
    PushGit,
    SingleFlightError,
    run_query,
    run_query_awaiting,
};
/// Import the finished answer of a bounded query.
use crate::bounded_query::QueryOutput;
/// Import the configuration listing and the lookup that answers whether a key is true.
use crate::config_snapshot::{
    ConfigSnapshot,
    Lookup,
};
/// Import the owned OS strings a query's arguments are made of.
use std::ffi::OsString;

/// Where the plain push goes:
/// the branch's push remote and the ref it updates there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Destination = { remote: string; mergeRef: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Destination {
    /// Remote name the push goes to.
    pub remote: String,
    /// Full ref the push updates there,
    /// for example `refs/heads/main`.
    pub merge_ref: String,
}

/// What the service does for one named branch whose upstream resolves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamedPush = { kind: 'push'; remote: string; targets: string[] } | { kind: 'matching'; remote: string }
///   | { kind: 'skip'; note: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedPush {
    /// Push the branch to each of these refs on the remote.
    Push {
        /// The remote the push goes to.
        remote: String,
        /// The full refs that receive the branch.
        targets: Vec<String>,
    },
    /// `push.default=matching`:
    /// the plain `git push <remote>` with no refspec,
    /// so Git matches the branches itself.
    Matching {
        /// The remote the push goes to.
        remote: String,
    },
    /// Git pushes the branch nowhere or refuses;
    /// the note says why and what to do.
    Skip(String),
}

/// The settings that decide where one branch is pushed,
/// read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushSettings = { branch: string; pushRemote: string; trackedRemote: string; merges: string[]; mode: string; refspecs: string[]; mirror: boolean };
/// ```
pub(super) struct PushSettings {
    /// Short name of the branch.
    pub(super) branch: String,
    /// The remote the push goes to,
    /// in Git's order (`branch.<name>.pushRemote`,
    /// `remote.pushDefault`,
    /// `branch.<name>.remote`).
    pub(super) push_remote: String,
    /// `branch.<name>.remote`,
    /// empty when unset.
    pub(super) tracked_remote: String,
    /// Every `branch.<name>.merge` value,
    /// in order.
    pub(super) merges: Vec<String>,
    /// The value of `push.default`,
    /// empty when unset.
    pub(super) mode: String,
    /// Every `remote.<push remote>.push` value,
    /// in order.
    pub(super) refspecs: Vec<String>,
    /// Whether `remote.<push remote>.mirror` is true.
    pub(super) mirror: bool,
}

/// Argument list from string literals and owned text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['config', '--list', ...]
/// ```
pub(super) fn argv(parts: &[&str]) -> Vec<OsString> {
    return parts.iter().map(OsString::from).collect();
}

/// Every value of `key` in the snapshot,
/// in listing order;
/// a key written without `=` reads as an empty value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.entries.filter((entry) => entry.key === key).map((entry) => entry.value ?? '')
/// ```
fn values_of(
    snapshot: &ConfigSnapshot,
    key: &str,
) -> Vec<String> {
    let mut values: Vec<String> = Vec::new();
    for entry in snapshot.entries() {
        if entry.key != key {
            continue;
        }
        values.push(match entry.value.as_deref() {
            Some(bytes) => String::from_utf8_lossy(bytes).into_owned(),
            None => String::new(),
        });
    }
    return values;
}

/// The last value of `key`,
/// empty when it is not set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// valuesOf(snapshot, key).at(-1) ?? ''
/// ```
fn last_of(
    snapshot: &ConfigSnapshot,
    key: &str,
) -> String {
    return values_of(snapshot, key).pop().unwrap_or_default();
}

///  The settings that decide where the plain push of a branch goes,
/// or nothing for a branch outside `refs/heads`
///  and for the remote `.` or no remote.
/// Why:
///   One listing of the configuration (`config --list -z --show-scope`,
/// answered without a Git process where the
///  in-process reader can) gives every value of the multi-valued keys,
/// which a `config --get` (the last value only)
///  cannot:
/// `branch.<name>.merge` and `remote.<name>.push` hold several.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readPushSettings({ git, branchRef }): Promise<PushSettings | undefined>;
/// ```
pub(super) fn read_push_settings(
    git: &PushGit,
    branch_ref: &str,
) -> Result<Option<PushSettings>, SingleFlightError> {
    let Some(name) = branch_ref.strip_prefix("refs/heads/") else {
        return Ok(None);
    };
    let arguments = argv(&["config", "--list", "-z", "--show-scope"]);
    let listing = run_query(git, &arguments)?;
    return push_settings_of(&listing, name);
}

///  The same settings,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push asks where it may send a branch as a task,
/// and the synchronous read enters the runtime and is refused from there.
/// The question and the reading of its answer are the synchronous face's,
/// so the two cannot decide differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readPushSettingsOnTheRuntime({ git, branchRef });
/// ```
pub(super) async fn read_push_settings_awaiting(
    git: &PushGit,
    branch_ref: &str,
) -> Result<Option<PushSettings>, SingleFlightError> {
    let Some(name) = branch_ref.strip_prefix("refs/heads/") else {
        return Ok(None);
    };
    let arguments = argv(&["config", "--list", "-z", "--show-scope"]);
    let listing = run_query_awaiting(git, &arguments).await?;
    return push_settings_of(&listing, name);
}

///  The settings one finished configuration listing answers for `name`,
/// or nothing for a branch with no remote to push to.
/// Why:
///   Both faces of the read turn one listing into one set of settings here,
/// so a value read by either is judged by the same code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settingsOf(listing, name); // PushSettings | undefined
/// ```
pub(super) fn push_settings_of(
    listing: &QueryOutput,
    name: &str,
) -> Result<Option<PushSettings>, SingleFlightError> {
    if listing.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: String::from("config --list -z --show-scope"),
            code: listing.code,
            stderr: String::from_utf8_lossy(&listing.stderr).into_owned(),
        });
    }
    let Some(snapshot) = ConfigSnapshot::parse(&listing.stdout, true) else {
        return Err(SingleFlightError::Git {
            arguments: String::from("config --list -z --show-scope"),
            code: listing.code,
            stderr: String::from("the listing is not in the shape `scope NUL key LF value NUL`"),
        });
    };
    let tracked_remote = last_of(&snapshot, &format!("branch.{name}.remote"));
    let mut push_remote = last_of(&snapshot, &format!("branch.{name}.pushremote"));
    if push_remote.is_empty() {
        push_remote = last_of(&snapshot, "remote.pushdefault");
    }
    if push_remote.is_empty() {
        push_remote.clone_from(&tracked_remote);
    }
    if push_remote.is_empty() || push_remote == "." {
        return Ok(None);
    }
    return Ok(Some(PushSettings {
        branch: name.to_owned(),
        refspecs: values_of(&snapshot, &format!("remote.{push_remote}.push")),
        mirror: matches!(
            snapshot.bool_value(&format!("remote.{push_remote}.mirror")),
            Lookup::Found(true)
        ),
        merges: values_of(&snapshot, &format!("branch.{name}.merge")),
        mode: last_of(&snapshot, "push.default"),
        push_remote,
        tracked_remote,
    }));
}
