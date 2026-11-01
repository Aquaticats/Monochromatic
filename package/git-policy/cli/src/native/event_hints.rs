//! What:
//!  The hint that follows a finding, an engine failure or a warn-unsafe notice: a guess at what the person was
//!  trying to do, and the concrete next step.
//! Why:
//!  A block that only states a rule leaves the next move to guesswork. Every block and every divergence from real
//!  Git's behavior names the likely intent and a way forward, so a rejected command is a step toward the goal.
//!  Messages built with their own hint (the discard, branch, add, root and commit-only rejections) are listed as
//!  self-hinted, and a guard test fails when a code has neither.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hint = hints.get(`${policy}/${code}`); if (hint) event.hint = hint;
//! ```

use super::json_record::quote;

/// The policy codes whose messages already carry their own intent guess and next step.
pub const SELF_HINTED: &[(&str, &str)] = &[
    ("require-root", "not-at-root"),
    ("linked-worktree-only", "linked-worktree-required"),
    ("branch-worktree-only", "branch-creation-requires-worktree"),
];

/// What:
///  The hint for a policy finding, or none when the message carries its own.
///  `&'static str` is text compiled into the program.
/// Why:
///   A finding's wording is the rule; the hint is the guess about the goal and the way to reach it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingHint(policy: string, code: string, fixAvailable: boolean): string | undefined;
/// ```
pub fn finding_hint(policy: &str, code: &str, fix_available: bool) -> Option<&'static str> {
    match (policy, code, fix_available) {
        ("add-explicit", "bulk-add-rejected", _) => {
            return Some(
                "Hint: if you meant to stage everything you changed, run `git status --short` to list it and name \
                 each path, as in `git add -- <path> <path>`; if you meant only files git already tracks, name those \
                 paths the same way.",
            );
        }
        ("final-newline", "noncanonical-final-newline", true) => {
            return Some(
                "Hint: if you are committing or checking this file, a commit corrects the ending for you; \
                 to correct it now without committing, run `git cli-git fix -- <path>`.",
            );
        }
        ("final-newline", "noncanonical-final-newline", false) => {
            return Some(
                "Hint: if you are committing or checking this file, edit it so it ends with exactly one newline, \
                 then stage it again.",
            );
        }
        ("markdown/autofix", "markdown-autofix", _) => {
            return Some(
                "Hint: if you are committing this Markdown, the commit applies the linter's corrections itself; \
                 to apply them now, run `git cli-git fix -- <path>`.",
            );
        }
        ("markdown/autofix", "markdown-violation", _) => {
            return Some(
                "Hint: if you are committing this Markdown, the linter cannot repair this rule by itself; \
                 edit the file at the reported line and column, stage it again, and retry.",
            );
        }
        ("mono/forbidden-root-context", "root-context-forbidden", _) => {
            return Some(
                "Hint: if you meant to keep project notes or terms for later readers, put them in a file under doc/; \
                 to drop CONTEXT.md from this commit and keep the file on disk, run `git rm --cached CONTEXT.md`.",
            );
        }
        ("mono/dependent-version-bump", "dependent-version-stale", _) => {
            return Some(
                "Hint: if you changed a package that others depend on, raise its dependents' version ranges to match \
                 the new version in the same commit, then stage those files; `git cli-git fix -- <path>` applies the \
                 correction when one is available.",
            );
        }
        ("mono/dependent-version-bump", "dependent-version-unsupported", _) => {
            return Some(
                "Hint: if you edited a dependency declaration of a shape this check does not read, update the \
                 dependents by hand in the same commit; the message names the file.",
            );
        }
        ("security/forbidden-strings", "forbidden-string", _) => {
            return Some(
                "Hint: if you are committing text or a path that contains a secret or a banned term, remove or \
                 replace it where the message points and stage the file again; if you believe the rule is wrong, \
                 review it in the rules file named by the configuration instead of splitting the commit.",
            );
        }
        _ => return None,
    }
}

/// What:
///  The hint for an engine failure, by the code's wire spelling.
/// Why:
///   An engine failure stops the command before Git runs; the person needs to know whether to retry, fix a file, or
///  report a defect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function engineFailureHint(code: string): string | undefined;
/// ```
pub fn engine_failure_hint(code: &str) -> Option<&'static str> {
    match code {
        "config-invalid" => {
            return Some(
                "Hint: if you just edited cli-git.config.jsonc, fix the key the message names or revert the edit; \
                 an unknown key, a wrong type or an unknown policy id fails before any policy runs.",
            );
        }
        "core-incomplete" => {
            return Some(
                "Hint: if another session was changing the repository at the same moment (a commit, a checkout or a \
                 rebase running elsewhere), run the command again; if it repeats, run `git status` to check that the \
                 repository is healthy.",
            );
        }
        "content-unavailable" => {
            return Some(
                "Hint: if you are committing or pushing content that is not in this clone (a missing object, a \
                 shallow history, or a file removed during the command), fetch the history with `git fetch`, make \
                 sure the files exist, and retry.",
            );
        }
        "policy-incomplete" => {
            return Some(
                "Hint: if you are committing or pushing, a policy could not finish reading the content, so it \
                 decided nothing; fix the cause the message names (an unreadable file, a path with a line break) \
                 and retry.",
            );
        }
        "patch-invalid" | "patch-conflict" => {
            return Some(
                "Hint: if you are committing files that changed while the command ran or that another policy also \
                 rewrites, save your edits, run `git cli-git fix -- <path>` to apply the corrections, review the \
                 result with `git diff`, and commit again.",
            );
        }
        "fix-cycle" | "fix-pass-limit" => {
            return Some(
                "Hint: two corrections keep undoing each other on this file; apply one of them by hand, stage the \
                 file, and commit again. Report the pair of policies if the file is ordinary text.",
            );
        }
        "transaction-failed" => {
            return Some(
                "Hint: if you were committing, run `git status` and `git log --oneline -3` to see whether the \
                 commit landed before you retry; staged files stay staged unless the message says otherwise, and the \
                 message names the step that failed.",
            );
        }
        "index-lock-unproven-owner" => {
            return Some(
                "Hint: if no other Git command is running in this repository, a crashed command left the lock \
                 behind: check with `ps` that no Git process uses this repository, then remove the .git/index.lock \
                 file the message names and retry.",
            );
        }
        "legacy-config-ignored" => {
            return Some(
                "Hint: if you are still keeping the old configuration files, delete them once \
                 cli-git.config.jsonc holds the settings; they are no longer read.",
            );
        }
        "traditional-hooks-unsupported" => {
            return Some(
                "Hint: if scripts in the hooks directory still do work for you, move each into a config hook with \
                 `git config hook.<name>.event <event>` and `git config hook.<name>.command <command>`; \
                 `git cli-git doctor` lists the scripts it finds and exits 1 while any remain.",
            );
        }
        _ => return None,
    }
}

/// What:
///  The hint for a warn-unsafe notice.
/// Why:
///   The person who set a policy to warn wanted commits to keep flowing; the notice says what that costs and what to
///  choose instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const warnUnsafeHint: string;
/// ```
pub const WARN_UNSAFE_HINT: &str = "Hint: if you set it to warn to keep commits flowing, note that this policy \
     changes or misses content when it only warns; set its severity to error to enforce it, or to off to disable it, \
     in cli-git.config.jsonc.";

/// The hint that follows a post-commit gate block.
pub const POST_COMMIT_BLOCKED_HINT: &str = "Hint: if you wanted this commit backed up, fix what the gate reported \
     and run `git push` yourself; the commit stays in your local history until then.";

/// The hint that follows a replay that dropped signature headers.
pub const REPLAY_HEADERS_DROPPED_HINT: &str = "Hint: if this commit must carry your original signature, run \
     `git commit --amend --no-edit -S` after the replay to sign the replayed commit again.";

/// What:
///  Append the `hint` field to a JSON event line when a hint exists.
///  `&mut String` lends the line for appending.
/// Why:
///   The `message` field stays byte-identical to the retired wrapper's; the hint is an additive field that readers of
///  the line see without any change to the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (hint !== undefined) event.hint = hint;
/// ```
pub fn push_hint_field(line: &mut String, hint: Option<&str>) {
    if let Some(text) = hint {
        line.push_str(",\"hint\":");
        line.push_str(quote(text).as_str());
    }
}

/// The guard that every code has a hint or its own.
#[cfg(test)]
#[path = "event_hints_tests.rs"]
mod tests;
