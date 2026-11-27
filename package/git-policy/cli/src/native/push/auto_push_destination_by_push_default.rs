//! What:
//!  The ref `push.default` sends one branch to,
//!  or the note that says why Git sends it nowhere.
//! Why:
//!  Git's own rules decide this,
//!  and a setting it refuses,
//!  or several upstream branches,
//!  must become a skip with a note that names the setting,
//!  never a push;
//!  both faces of the push turn one decision here into one push.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function byMode(branchRef: string, settings: PushSettings): ByMode;
//! function namedByMode(branchRef: string, settings: PushSettings): NamedPush;
//! ```

/// Import the settings one branch is pushed by,
/// and the push or skip they name.
use super::auto_push_destination_settings::{
    Destination,
    NamedPush,
    PushSettings,
};

/// Start of every note this module words,
/// so the terminal filter keeps it and a person sees which tool spoke.
const NOTE_PREFIX: &str = "monochromatic-git-policy-cli: auto-push: ";

/// A note for a branch that is not pushed:
/// what was seen,
/// and what to do about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: auto-push: ${branch} was not pushed: ${reason} Nothing was pushed or changed.`
/// ```
fn note(
    branch: &str,
    reason: &str,
) -> String {
    return format!(
        "{NOTE_PREFIX}{branch} was not pushed: {reason} Nothing was pushed or changed; the commit is saved locally."
    );
}

/// The skip of a branch,
/// with its note.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ kind: 'skip', note: note(branch, reason) })
/// ```
pub(super) fn skip(
    branch: &str,
    reason: &str,
) -> NamedPush {
    return NamedPush::Skip(note(branch, reason));
}

/// How `push.default` sends the branch when no `remote.<name>.push` refspec does.
enum ByMode {
    /// The ref the branch updates on the push remote.
    Target(String),
    /// `matching`:
    /// Git pairs the branches itself,
    /// so the plain push is run.
    Matching,
    /// Git pushes nothing or refuses;
    /// the note says why.
    Refused(NamedPush),
}

/// The target when the push goes to the branch's upstream:
/// refused when the push remote is not the tracked remote or the
/// branch has not exactly one `merge` value that names a branch.
fn upstream_target(
    settings: &PushSettings,
    triangular: bool,
    mode_name: &str,
) -> ByMode {
    let branch = settings.branch.as_str();
    if triangular {
        return ByMode::Refused(skip(
            branch,
            &format!(
                "`push.default` is `{mode_name}` but its push remote `{}` is not the remote it tracks (`{}`), so Git refuses to say which \
             branch to update. Set `push.default` to `simple` or `current`, or push it with `git push {} {branch}`.",
                settings.push_remote, settings.tracked_remote, settings.push_remote
            ),
        ));
    }
    let [merge] = settings.merges.as_slice() else {
        return ByMode::Refused(skip(
            branch,
            &format!(
                "`branch.{branch}.merge` has {} values and `push.default` is `{}`, so Git refuses to pick one upstream branch (\"multiple upstream \
             branches, refusing to push\"). Keep one `branch.{branch}.merge` value, or set `push.default` to `current`.",
                settings.merges.len(),
                if mode_name.is_empty() {
                    "simple"
                } else {
                    mode_name
                }
            ),
        ));
    };
    if !merge.starts_with("refs/heads/") || merge == "refs/heads/" {
        return ByMode::Refused(skip(
            branch,
            &format!(
                "it tracks `{merge}`, which is not a branch under `refs/heads`. Push it yourself with `git push {} {branch}`.",
                settings.push_remote
            ),
        ));
    }
    return ByMode::Target(merge.clone());
}

///  The ref `push.default` sends the branch to,
/// or why Git sends it nowhere.
/// Why:
///   `current` and a push remote other than the tracked one (a triangular `simple`) use the branch's own name.
///  `upstream` and `simple` on the tracked remote use the upstream branch,
/// and Git dies when the branch has none or
///  several (`get_upstream_ref` in `builtin/push.c`);
/// a differently named single upstream,
/// which `simple` also refuses,
///  is where the person tracked the branch (a linked worktree branch created from `origin/main`),
/// so it is pushed.
///   `nothing` and a triangular `upstream` name no single ref;
/// `matching` names none either,
/// and Git is left to match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function byMode(branchRef: string, settings: PushSettings): ByMode;
/// ```
fn by_mode(
    branch_ref: &str,
    settings: &PushSettings,
) -> ByMode {
    let branch = settings.branch.as_str();
    let mode = settings.mode.as_str();
    let triangular = settings.push_remote != settings.tracked_remote;
    return match mode {
        "current" => ByMode::Target(branch_ref.to_owned()),
        "" | "simple" if triangular => ByMode::Target(branch_ref.to_owned()),
        "" | "simple" => upstream_target(settings, triangular, ""),
        "upstream" | "tracking" => upstream_target(settings, triangular, mode),
        "matching" => ByMode::Matching,
        "nothing" => ByMode::Refused(skip(
            branch,
            &format!(
                "`push.default` is `nothing`, so Git pushes no branch without a refspec. Set `push.default` to `simple` or `current`, or push it \
             with `git push {} {branch}`.",
                settings.push_remote
            ),
        )),
        other => ByMode::Refused(skip(
            branch,
            &format!(
                "`push.default` is `{other}`, a value Git does not accept. Set it to `simple`, `current`, `upstream` or `matching`."
            ),
        )),
    };
}

///  The destination one push's settings name,
/// or nothing when they name none.
/// Why:
///   Both faces of `read_destination` decide from the same settings here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settings.mirror || settings.refspecs.length ? undefined : byMode(branchRef, settings).kind === 'target' ? { remote, mergeRef } : undefined;
/// ```
pub(super) fn destination_of(
    branch_ref: &str,
    settings: &PushSettings,
) -> Option<Destination> {
    if settings.mirror || !settings.refspecs.is_empty() {
        return None;
    }
    return match by_mode(branch_ref, settings) {
        ByMode::Target(merge_ref) => Some(Destination {
            remote: settings.push_remote.clone(),
            merge_ref,
        }),
        ByMode::Matching | ByMode::Refused(_) => None,
    };
}

///  The skip of a branch whose remote mirrors every ref.
/// Why:
///   Both faces of `named_push` refuse a mirrored remote with the same note.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// skip(settings.branch, `remote.${settings.pushRemote}.mirror is true ...`);
/// ```
pub(super) fn mirror_skip(settings: &PushSettings) -> NamedPush {
    return skip(
        &settings.branch,
        &format!(
            "`remote.{0}.mirror` is true: Git mirrors every ref to `{0}` (and deletes the ones it lacks) and refuses a branch pushed by name, \
         which is all this service pushes. Run `git push` yourself to mirror, or unset `remote.{0}.mirror`.",
            settings.push_remote
        ),
    );
}

///  The push `push.default` names,
/// once the branch has no configured refspec to obey.
/// Why:
///   Both faces of `named_push` turn one decision of `push.default` into one push here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decided = byMode(branchRef, settings);
/// ```
pub(super) fn named_by_mode(
    branch_ref: &str,
    settings: &PushSettings,
) -> NamedPush {
    return match by_mode(branch_ref, settings) {
        ByMode::Target(target) => NamedPush::Push {
            remote: settings.push_remote.clone(),
            targets: vec![target],
        },
        ByMode::Matching => NamedPush::Matching {
            remote: settings.push_remote.clone(),
        },
        ByMode::Refused(skipped) => skipped,
    };
}
