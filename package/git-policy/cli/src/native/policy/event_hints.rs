//! What:
//!  The hint that follows a finding,
//! an engine failure or a warn-unsafe notice:
//! a guess at what the person was
//!  trying to do,
//! and the concrete next step.
//! Why:
//!  A block that only states a rule leaves the next move to guesswork.
//! Every block and every divergence from real
//!  Git's behavior names the likely intent and a way forward,
//! so a rejected command is a step toward the goal.
//!  Messages built with their own hint are listed as
//!  self-hinted,
//! and a guard test fails when a code has neither.
//!  The three families in that list are the discard
//!  (`linked-worktree-only`),
//! the root (`require-root`) and the branch-worktree
//!  (`branch-worktree-only`) rejections;
//! the shared-remote-settings refusal of `linked-worktree-only` carries its own
//!  text in `rule_shared_remote_messages.rs`,
//! the add rejection's hint is in the table of
//!  `finding_hint`,
//! and the commit-only diagnostics carry their own text in
//!  `rule_commit_only_message.rs`.
//!  The hint an engine failure and a configuration warning carry is worded beside the events
//!  `invocation_config` renders,
//!  in `invocation_config` itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hint = hints.get(`${policy}/${code}`); if (hint) event.hint = hint;
//! ```

/// The hint of one engine-failure code,
/// which a manual push's missing content answers with.
use crate::invocation_config::engine_failure_hint;
use crate::manual_push_cause::ManualPushCause;

/// The policy codes whose messages already carry their own intent guess and next step.
pub const SELF_HINTED: &[(&str, &str)] = &[
    ("require-root", "not-at-root"),
    ("linked-worktree-only", "linked-worktree-required"),
    ("linked-worktree-only", "shared-remote-settings"),
    ("branch-worktree-only", "branch-creation-requires-worktree"),
];

/// What:
///  The hint for a policy finding,
/// or none when the message carries its own.
///  `&'static str` is text compiled into the program.
/// Why:
///   A finding's wording is the rule;
/// the hint is the guess about the goal and the way to reach it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingHint(policy: string, code: string, fixAvailable: boolean): string | undefined;
/// ```
pub fn finding_hint(
    policy: &str,
    code: &str,
    fix_available: bool,
) -> Option<&'static str> {
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
                 to correct it now without committing, run `git monochromatic-git-policy-cli fix -- <path>`.",
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
                 to apply them now, run `git monochromatic-git-policy-cli fix -- <path>`.",
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
                 the new version in the same commit, then stage those files; `git monochromatic-git-policy-cli fix -- <path>` applies the \
                 correction when one is available.",
            );
        }
        ("mono/dependent-version-bump", "dependent-version-unsupported", _) => {
            return Some(
                "Hint: if you edited a dependency declaration of a shape this check does not read, update the \
                 dependents by hand in the same commit; the message names the file.",
            );
        }
        ("mono/catalog-specifiers", "disallowed-specifier", _) => {
            return Some(
                "Hint: if you are adding or changing a dependency, move its version into the catalog section of \
                 pnpm-workspace.yaml and write catalog: here, or write workspace:* for a package of this workspace; \
                 peerDependencies take * or catalog:.",
            );
        }
        ("mono/catalog-specifiers", "non-string-specifier", _) => {
            return Some(
                "Hint: if you are writing a dependency version, make it a string such as catalog: or workspace:*; \
                 a number, list, object or null is not a specifier.",
            );
        }
        ("mono/catalog-specifiers", "malformed-section", _) => {
            return Some(
                "Hint: if you are editing a dependency section, write it as an object that maps package names to \
                 specifiers; the message names the section.",
            );
        }
        ("mono/catalog-specifiers", "duplicate-key", _) => {
            return Some(
                "Hint: if you are editing this manifest by hand, keep one copy of the repeated section or \
                 dependency name; pnpm keeps the last copy, which may not be the one you meant.",
            );
        }
        ("mono/catalog-specifiers", "invalid-manifest", _) => {
            return Some(
                "Hint: if you are committing this package.json, make it a JSON object that parses and is UTF-8 \
                 text (a stray character, an empty file or a link instead of a file shows here), then stage it again.",
            );
        }
        ("mono/catalog-specifiers", "unsupported-manifest-format", _) => {
            return Some(
                "Hint: if you are adding a package.json5, package.yaml or package.yml, use package.json instead, \
                 because the catalog rule reads only that format; rename or convert the file and stage it again.",
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
///  The hint for a manual push whose scope could not be established,
/// by the cause the failure carries.
/// Why:
///   One engine-failure code covers causes the person answers differently,
/// so the hint names the cause and the move for it:
///  the existing missing-content hint stays where content is what is missing,
/// and a message that already says what to do
///  (a deadline with its setting,
/// a repository that cannot be located) gets none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function manualPushFailureHint(cause: ManualPushCause): string | undefined;
/// ```
pub fn manual_push_failure_hint(cause: ManualPushCause) -> Option<&'static str> {
    return match cause {
        ManualPushCause::ContentMissing => engine_failure_hint("content-unavailable"),
        ManualPushCause::RemoteMoved => Some(
            "Hint: the remote changed while the wrapper was reading it, so nothing was pushed; run the same push again.",
        ),
        ManualPushCause::RemoteUnreachable => Some(
            "Hint: the wrapper could not read the remote's branches before pushing, so nothing was pushed; check the \
             network connection, the remote's address and your credentials (`git ls-remote <remote>` shows Git's own \
             answer), then run the push again.",
        ),
        ManualPushCause::GitNotStarted => Some(
            "Hint: the wrapper could not start Git to check this push, so nothing was pushed; check that the Git \
             installation the wrapper runs exists and can be executed, then run the push again.",
        ),
        ManualPushCause::GitSilent => Some(
            "Hint: Git ended the wrapper's check of this push without a message, so nothing was pushed; run the same \
             command with `--dry-run` to see Git's own answer, and run the push again.",
        ),
        ManualPushCause::UnexpectedOutput => Some(
            "Hint: Git answered in a form the wrapper does not read, so nothing was pushed; this is a defect of the \
             wrapper, not of the repository. Report it with this message and the output of `git --version`.",
        ),
        ManualPushCause::UnsupportedObject => Some(
            "Hint: the manual-push policies read commits only; push a branch, or a tag that points at a commit.",
        ),
        ManualPushCause::SelfExplained => None,
    };
}

/// What:
///  The hint for a warn-unsafe notice.
/// Why:
///   The person who set a policy to warn wanted commits to keep flowing;
/// the notice says what that costs and what to
///  choose instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const warnUnsafeHint: string;
/// ```
pub const WARN_UNSAFE_HINT: &str = "Hint: if you set it to warn to keep commits flowing, note that this policy \
     changes or misses content when it only warns; set its severity to error to enforce it, or to off to disable it, \
     in monochromatic-git-policy-cli.config.jsonc.";

/// The hint that follows a post-commit gate block.
pub const POST_COMMIT_BLOCKED_HINT: &str = "Hint: if you wanted this commit backed up, fix what the gate reported \
     and run `git push` yourself; the commit stays in your local history until then.";

/// The guard that every code has a hint or its own.
#[cfg(test)]
#[path = "event_hints_tests.rs"]
mod tests;
