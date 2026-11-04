//! What:
//!  Where a plain `git push` of one named branch would send it, decided from the configuration and the local refs,
//!  or why Git would send it nowhere.
//! Why:
//!  The service pushes the branch a record names, which may not be the branch checked out, so it must name both sides of
//!  the push itself; naming them from a partial reading of Git's rules pushed to places a plain `git push` refuses
//!  or never goes (a `matching` push recreating a branch the remote deleted, one of several upstream branches chosen at
//!  random, a destination that another configured refspec had already taken).
//!  Everything here follows `setup_default_push_refspecs` in Git's `builtin/push.c` and `match_push_refs` in `remote.c`;
//!  a setting Git refuses or a form this module does not interpret becomes a skip with a note that names the setting,
//!  never a push.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const named: NamedPush = await namedPush({ git, branchRef }); // push | remote-must-have | skip
//! ```

use crate::auto_push_single_flight::{PushGit, SingleFlightError, run_query};
use crate::config_snapshot::{ConfigSnapshot, Lookup};
use std::collections::BTreeMap;
use std::ffi::OsString;

/// Start of every note this module words, so the terminal filter keeps it and a person sees which tool spoke.
const NOTE_PREFIX: &str = "cli-git: auto-push: ";

/// Where the plain push goes: the branch's push remote and the ref it updates there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Destination = { remote: string; mergeRef: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Destination {
    /// Remote name the push goes to.
    pub remote: String,
    /// Full ref the push updates there, for example `refs/heads/main`.
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
    /// `push.default=matching`: the plain `git push <remote>` with no refspec, so Git matches the branches itself.
    Matching {
        /// The remote the push goes to.
        remote: String,
    },
    /// Git pushes the branch nowhere or refuses; the note says why and what to do.
    Skip(String),
}

/// The settings that decide where one branch is pushed, read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushSettings = { branch: string; pushRemote: string; trackedRemote: string; merges: string[]; mode: string; refspecs: string[]; mirror: boolean };
/// ```
struct PushSettings {
    /// Short name of the branch.
    branch: String,
    /// The remote the push goes to, in Git's order (`branch.<name>.pushRemote`, `remote.pushDefault`, `branch.<name>.remote`).
    push_remote: String,
    /// `branch.<name>.remote`, empty when unset.
    tracked_remote: String,
    /// Every `branch.<name>.merge` value, in order.
    merges: Vec<String>,
    /// The value of `push.default`, empty when unset.
    mode: String,
    /// Every `remote.<push remote>.push` value, in order.
    refspecs: Vec<String>,
    /// Whether `remote.<push remote>.mirror` is true.
    mirror: bool,
}

/// Argument list from string literals and owned text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['config', '--list', ...]
/// ```
fn argv(parts: &[&str]) -> Vec<OsString> {
    return parts.iter().map(OsString::from).collect();
}

/// Every value of `key` in the snapshot, in listing order; a key written without `=` reads as an empty value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.entries.filter((entry) => entry.key === key).map((entry) => entry.value ?? '')
/// ```
fn values_of(snapshot: &ConfigSnapshot, key: &str) -> Vec<String> {
    return snapshot
        .entries()
        .iter()
        .filter(|entry| {
            return entry.key == key;
        })
        .map(|entry| {
            return entry.value.as_deref().map(|bytes| { return String::from_utf8_lossy(bytes).into_owned(); }).unwrap_or_default();
        })
        .collect();
}

/// The last value of `key`, empty when it is not set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// valuesOf(snapshot, key).at(-1) ?? ''
/// ```
fn last_of(snapshot: &ConfigSnapshot, key: &str) -> String {
    return values_of(snapshot, key).pop().unwrap_or_default();
}

/// What:
///  The settings that decide where the plain push of a branch goes, or nothing for a branch outside `refs/heads`
///  and for the remote `.` or no remote.
/// Why:
///   One listing of the configuration (`config --list -z --show-scope`, answered without a Git process where the
///  in-process reader can) gives every value of the multi-valued keys, which a `config --get` (the last value only)
///  cannot: `branch.<name>.merge` and `remote.<name>.push` hold several.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readPushSettings({ git, branchRef }): Promise<PushSettings | undefined>;
/// ```
fn read_push_settings(git: &PushGit, branch_ref: &str) -> Result<Option<PushSettings>, SingleFlightError> {
    let Some(name) = branch_ref.strip_prefix("refs/heads/") else {
        return Ok(None);
    };
    let arguments = argv(&["config", "--list", "-z", "--show-scope"]);
    let listing = run_query(git, &arguments)?;
    let unreadable = |stderr: String| -> SingleFlightError {
        return SingleFlightError::Git { arguments: String::from("config --list -z --show-scope"), code: listing.code, stderr };
    };
    if listing.code != Some(0) {
        return Err(unreadable(String::from_utf8_lossy(&listing.stderr).into_owned()));
    }
    let Some(snapshot) = ConfigSnapshot::parse(&listing.stdout, true) else {
        return Err(unreadable(String::from("the listing is not in the shape `scope NUL key LF value NUL`")));
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
        mirror: matches!(snapshot.bool_value(&format!("remote.{push_remote}.mirror")), Lookup::Found(true)),
        merges: values_of(&snapshot, &format!("branch.{name}.merge")),
        mode: last_of(&snapshot, "push.default"),
        push_remote,
        tracked_remote,
    }));
}

/// A note for a branch that is not pushed: what was seen, and what to do about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: auto-push: ${branch} was not pushed: ${reason} Nothing was pushed or changed.`
/// ```
fn note(branch: &str, reason: &str) -> String {
    return format!("{NOTE_PREFIX}{branch} was not pushed: {reason} Nothing was pushed or changed; the commit is saved locally.");
}

/// The skip of a branch, with its note.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ kind: 'skip', note: note(branch, reason) })
/// ```
fn skip(branch: &str, reason: &str) -> NamedPush {
    return NamedPush::Skip(note(branch, reason));
}

/// How `push.default` sends the branch when no `remote.<name>.push` refspec does.
enum ByMode {
    /// The ref the branch updates on the push remote.
    Target(String),
    /// `matching`: Git pairs the branches itself, so the plain push is run.
    Matching,
    /// Git pushes nothing or refuses; the note says why.
    Refused(NamedPush),
}

/// What:
///  The ref `push.default` sends the branch to, or why Git sends it nowhere.
/// Why:
///   `current` and a push remote other than the tracked one (a triangular `simple`) use the branch's own name.
///  `upstream` and `simple` on the tracked remote use the upstream branch, and Git dies when the branch has none or
///  several (`get_upstream_ref` in `builtin/push.c`); a differently named single upstream, which `simple` also refuses,
///  is where the person tracked the branch (a linked worktree branch created from `origin/main`), so it is pushed.
///   `nothing` and a triangular `upstream` name no single ref; `matching` names none either, and Git is left to match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function byMode(branchRef: string, settings: PushSettings): ByMode;
/// ```
fn by_mode(branch_ref: &str, settings: &PushSettings) -> ByMode {
    let branch = settings.branch.as_str();
    let mode = settings.mode.as_str();
    let triangular = settings.push_remote != settings.tracked_remote;
    let upstream = |mode_name: &str| -> ByMode {
        if triangular {
            return ByMode::Refused(skip(branch, &format!(
                "`push.default` is `{mode_name}` but its push remote `{}` is not the remote it tracks (`{}`), so Git refuses to say which \
                 branch to update. Set `push.default` to `simple` or `current`, or push it with `git push {} {branch}`.",
                settings.push_remote, settings.tracked_remote, settings.push_remote
            )));
        }
        let [merge] = settings.merges.as_slice() else {
            return ByMode::Refused(skip(branch, &format!(
                "`branch.{branch}.merge` has {} values and `push.default` is `{}`, so Git refuses to pick one upstream branch (\"multiple upstream \
                 branches, refusing to push\"). Keep one `branch.{branch}.merge` value, or set `push.default` to `current`.",
                settings.merges.len(),
                if mode_name.is_empty() { "simple" } else { mode_name }
            )));
        };
        if !merge.starts_with("refs/heads/") || merge == "refs/heads/" {
            return ByMode::Refused(skip(branch, &format!(
                "it tracks `{merge}`, which is not a branch under `refs/heads`. Push it yourself with `git push {} {branch}`.",
                settings.push_remote
            )));
        }
        return ByMode::Target(merge.clone());
    };
    return match mode {
        "current" => ByMode::Target(branch_ref.to_owned()),
        "" | "simple" if triangular => ByMode::Target(branch_ref.to_owned()),
        "" | "simple" => upstream(""),
        "upstream" | "tracking" => upstream(mode),
        "matching" => ByMode::Matching,
        "nothing" => ByMode::Refused(skip(branch, &format!(
            "`push.default` is `nothing`, so Git pushes no branch without a refspec. Set `push.default` to `simple` or `current`, or push it \
             with `git push {} {branch}`.",
            settings.push_remote
        ))),
        other => ByMode::Refused(skip(branch, &format!(
            "`push.default` is `{other}`, a value Git does not accept. Set it to `simple`, `current`, `upstream` or `matching`."
        ))),
    };
}

/// What:
///  The remote and the branch ref that the plain `git push` of this branch updates, when the settings name one.
/// Why:
///   The reconcile of a rejected push merges the tip of the branch the rejected push targeted, which with a push remote,
///  a triangular setup or `push.default=current` is not the tracked branch.
///   The remote `.`, a `remote.<name>.push` refspec, `matching`, `nothing`, several upstream branches and a target
///  outside `refs/heads` name none, because the rejection then came from a destination this module cannot name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readDestination({ git, branchRef }): Promise<Destination | undefined>;
/// ```
pub fn read_destination(git: &PushGit, branch_ref: &str) -> Result<Option<Destination>, SingleFlightError> {
    let Some(settings) = read_push_settings(git, branch_ref)? else {
        return Ok(None);
    };
    if settings.mirror || !settings.refspecs.is_empty() {
        return Ok(None);
    }
    return Ok(match by_mode(branch_ref, &settings) {
        ByMode::Target(merge_ref) => Some(Destination { remote: settings.push_remote, merge_ref }),
        ByMode::Matching | ByMode::Refused(_) => None,
    });
}

/// What:
///  What the service does for a named branch whose upstream resolves: the remote and refs it pushes to, the one remote
///  question that decides, or the note that explains why it pushes nothing.
/// Why:
///   The plain `git push` pushes the checked-out branch, so a push of a branch that is not checked out names both sides;
///  the destination follows Git's rules for the branch as if it were checked out, and a setting under which Git
///  refuses or pushes nothing for it is a skip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function namedPush({ git, branchRef }): Promise<NamedPush>;
/// ```
pub fn named_push(git: &PushGit, branch_ref: &str) -> Result<NamedPush, SingleFlightError> {
    let Some(settings) = read_push_settings(git, branch_ref)? else {
        let branch = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
        return Ok(skip(branch, "it has no remote to push to (its remote is `.`, a local branch, or none is set). Push it yourself with `git push`."));
    };
    if settings.mirror {
        return Ok(skip(&settings.branch, &format!(
            "`remote.{0}.mirror` is true: Git mirrors every ref to `{0}` (and deletes the ones it lacks) and refuses a branch pushed by name, \
             which is all this service pushes. Run `git push` yourself to mirror, or unset `remote.{0}.mirror`.",
            settings.push_remote
        )));
    }
    if !settings.refspecs.is_empty() {
        return named_by_refspecs(git, branch_ref, &settings);
    }
    return Ok(match by_mode(branch_ref, &settings) {
        ByMode::Target(target) => NamedPush::Push { remote: settings.push_remote.clone(), targets: vec![target] },
        ByMode::Matching => NamedPush::Matching { remote: settings.push_remote.clone() },
        ByMode::Refused(skipped) => skipped,
    });
}

/// One `remote.<name>.push` refspec this module interprets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refspec = { negative: boolean; source: string; destination?: string; pattern: boolean };
/// ```
struct Refspec {
    /// A `^` refspec: it removes matching destinations.
    negative: bool,
    /// The local side.
    source: String,
    /// The remote side; the source when absent.
    destination: Option<String>,
    /// Whether the sides hold one `*`.
    pattern: bool,
}

/// What:
///  Read a configured push refspec, or nothing when it is a form this module does not interpret.
/// Why:
///   Only full ref names and one `*` per side are interpreted; `HEAD`, short names, object names, deletions and the
///  `:` of matching depend on more than the local ref names, so they are never guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRefspec(text: string): Refspec | undefined;
/// ```
fn parse_refspec(text: &str) -> Option<Refspec> {
    let stars = |side: &str| -> usize { return side.matches('*').count(); };
    if let Some(source) = text.strip_prefix('^') {
        let negative_ok = source.starts_with("refs/") && !source.contains(':') && stars(source) <= 1;
        return negative_ok.then(|| { return Refspec { negative: true, source: source.to_owned(), destination: None, pattern: stars(source) == 1 }; });
    }
    let spec = text.strip_prefix('+').unwrap_or(text);
    let (source, destination) = match spec.split_once(':') {
        Some((source, destination)) => (source, Some(destination)),
        None => (spec, None),
    };
    if !source.starts_with("refs/") || destination.is_some_and(|side| { return !side.starts_with("refs/"); }) {
        return None;
    }
    let pattern = stars(source) == 1;
    let sides_agree = stars(source) == destination.map_or(stars(source), stars);
    if stars(source) > 1 || !sides_agree {
        return None;
    }
    return Some(Refspec { negative: false, source: source.to_owned(), destination: destination.map(str::to_owned), pattern });
}

/// The text a `*` pattern captured in `name`, or nothing when `name` does not match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function capture(pattern: string, name: string): string | undefined;
/// ```
fn capture<'a>(pattern: &str, name: &'a str) -> Option<&'a str> {
    let (prefix, suffix) = pattern.split_once('*')?;
    let middle = name.strip_prefix(prefix)?.strip_suffix(suffix)?;
    return Some(middle);
}

/// Whether a negative refspec removes the remote ref `name`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function excludes(negative: Refspec, name: string): boolean;
/// ```
fn excludes(negative: &Refspec, name: &str) -> bool {
    if negative.pattern {
        return capture(&negative.source, name).is_some();
    }
    return negative.source == name;
}

/// What the configured push refspecs make of one branch.
#[derive(Debug, Eq, PartialEq)]
enum Judged {
    /// The refs on the remote that receive the branch, possibly none.
    Targets(Vec<String>),
    /// Git refuses the whole push; the sentence says why.
    Refused(String),
    /// A refspec of a form not interpreted; its text.
    Unjudged(String),
}

/// What:
///  Apply the configured push refspecs to the local refs the way Git's `match_push_refs` does, and report where the branch goes.
/// Why:
///   Git matches every explicit refspec first (a source that is not a local ref, or two sources for one destination, fail
///  the whole push), then lets each local ref take the first matching pattern unless an earlier pairing already took its
///  destination, then drops the destinations a negative refspec names.
///  The `%(push:remoteref)` atom of `git for-each-ref` maps one branch alone, so it misses all three and cannot be used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judge({ branchRef, refspecs, localRefs }): Judged;
/// ```
fn judge(branch_ref: &str, remote: &str, refspecs: &[String], local_refs: &[String]) -> Judged {
    let mut items: Vec<Refspec> = Vec::new();
    for text in refspecs {
        match parse_refspec(text) {
            Some(item) => items.push(item),
            None => return Judged::Unjudged(text.clone()),
        }
    }
    let mut taken: BTreeMap<String, String> = BTreeMap::new();
    for item in items.iter().filter(|item| { return !item.negative && !item.pattern; }) {
        if !local_refs.contains(&item.source) {
            return Judged::Refused(format!(
                "`remote.{remote}.push` names `{}`, which is not a ref here, and Git refuses to push anything while it is not.",
                item.source
            ));
        }
        let destination: String = item.destination.clone().unwrap_or_else(|| { return item.source.clone(); });
        if let Some(earlier) = taken.insert(destination.clone(), item.source.clone()) {
            return Judged::Refused(format!(
                "`remote.{remote}.push` sends both `{earlier}` and `{}` to `{destination}`, and Git refuses that (\"receives from more than one src\").",
                item.source
            ));
        }
    }
    let mut sorted: Vec<&String> = local_refs.iter().collect();
    sorted.sort();
    for name in sorted {
        let mapped = items.iter().filter(|item| { return !item.negative && item.pattern; }).find_map(|item| {
            let middle = capture(&item.source, name)?;
            let side: &str = item.destination.as_deref().unwrap_or(item.source.as_str());
            return Some(side.replacen('*', middle, 1));
        });
        if let Some(destination) = mapped {
            taken.entry(destination).or_insert_with(|| { return name.clone(); });
        }
    }
    for negative in items.iter().filter(|item| { return item.negative; }) {
        taken.retain(|destination, _| { return !excludes(negative, destination); });
    }
    return Judged::Targets(
        taken
            .into_iter()
            .filter(|(_, source)| { return source == branch_ref; })
            .map(|(destination, _)| { return destination; })
            .collect(),
    );
}

/// What:
///  The destinations of the configured push refspecs for the branch, or the note for why there are none.
/// Why:
///   The local refs the refspecs can name are asked for in one `for-each-ref`; everything else is decided here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function namedByRefspecs({ git, branchRef, settings }): Promise<NamedPush>;
/// ```
fn named_by_refspecs(git: &PushGit, branch_ref: &str, settings: &PushSettings) -> Result<NamedPush, SingleFlightError> {
    let branch = settings.branch.as_str();
    let remote = settings.push_remote.as_str();
    let mut patterns: Vec<String> = settings
        .refspecs
        .iter()
        .filter_map(|text| { return parse_refspec(text).filter(|item| { return !item.negative; }).map(|item| { return item.source; }); })
        .collect();
    patterns.sort();
    patterns.dedup();
    let mut query: Vec<&str> = vec!["for-each-ref", "--format=%(refname)"];
    query.extend(patterns.iter().map(String::as_str));
    let asked = run_query(git, &argv(&query))?;
    if asked.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: query.join(" "),
            code: asked.code,
            stderr: String::from_utf8_lossy(&asked.stderr).into_owned(),
        });
    }
    let local_refs: Vec<String> = String::from_utf8_lossy(&asked.stdout).lines().map(str::to_owned).collect();
    return Ok(match judge(branch_ref, remote, &settings.refspecs, &local_refs) {
        Judged::Targets(targets) if !targets.is_empty() => NamedPush::Push { remote: remote.to_owned(), targets },
        Judged::Targets(_) => skip(branch, &format!(
            "no `remote.{remote}.push` refspec sends it anywhere, so Git pushes other refs or nothing for it. Add a refspec for it, or push it \
             with `git push {remote} {branch}`."
        )),
        Judged::Refused(reason) => skip(branch, &format!("{reason} Fix `remote.{remote}.push`, or push it with `git push {remote} {branch}`.")),
        Judged::Unjudged(text) => skip(branch, &format!(
            "`remote.{remote}.push` has `{text}`, a form this service does not interpret (it reads full ref names and one `*` per side). \
             Push it yourself with `git push`."
        )),
    });
}

/// The destination logic is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_destination_tests.rs"]
mod tests;
