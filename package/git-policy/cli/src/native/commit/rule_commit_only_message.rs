//! What:
//!  The user-facing diagnostics of the commit-only transform.
//! Why:
//!  Each message names the rejected form and every valid way forward;
//!  keeping the text
//!      apart from the decision keeps both files readable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const NO_PATHSPEC_MESSAGE = 'cli-git: git commit requires an explicit pathspec ...';
//! ```

/// Diagnostic for a commit with no pathspec source and no mode that permits a pathless
/// only-commit.
pub const NO_PATHSPEC_MESSAGE: &str = "cli-git: git commit requires an explicit pathspec when commit-only enforcement is active. \
Name the paths in the commit command (for example, git commit -m <msg> <path>), \
pass --pathspec-from-file, pass --no-only to commit the entire index, \
or pass --no-enforce-only to bypass for this invocation.";

/// Diagnostic for `-a`/`--all`,
///  which stages tracked modifications implicitly.
pub const ALL_FLAG_MESSAGE: &str = "cli-git: git commit rejects -a/--all because it stages every tracked modification before committing. \
Stage paths explicitly and commit with git commit -m <msg> <path>, \
or pass --no-enforce-only to bypass for this invocation.";

/// What:
///  Build the diagnostic for a pathless `--amend`/`--allow-empty` commit over a dirty
///       index.
///  `&str` borrows the flag text;
///  `String` is the owned result.
/// Why:
///   An injected `--only` would commit `HEAD`'s existing tree and leave the staged
///       changes staged without any warning from Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ignoredIndexMessage(flagText: string): string;
/// ```
pub fn ignored_index_message(flag_text: &str) -> String {
    // `format!` builds an owned `String`; `{flag_text}` inserts the borrowed text.
    return format!(
        "cli-git: git commit {flag_text} without pathspecs would silently ignore your staged changes. \
Commit-only enforcement injects --only, and a pathless --only commit reuses HEAD's existing tree, \
leaving staged changes staged with no warning; the index currently differs from HEAD. \
Choose explicitly: name the paths to include them (git commit --amend <path>), \
pass --only to proceed without them, \
or pass --no-only to commit the entire index."
    );
}
