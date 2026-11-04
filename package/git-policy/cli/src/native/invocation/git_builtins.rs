//! What:
//!  The names of the commands built into Git 2.56.0,
//!  and the test "is this word one
//!       of them".
//! Why:
//!  Git never expands an alias whose name is a built-in command,
//!  so a built-in word
//!      is the command that runs.
//!  Any other word may be an alias for anything,
//!  including
//!      `worktree add`,
//!  and resolving aliases is not ported;
//!  the caller needs to know
//!      which of the two it is looking at without starting Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // GIT_BUILTIN_COMMANDS.has('status') === true; GIT_BUILTIN_COMMANDS.has('st') === false
//! ```

/// What:
///  Every name `git --list-cmds=builtins` prints for Git 2.56.0,
///  in that order.
///       `&[&[u8]]` is a borrowed list of byte spellings baked into the program.
///  Sibling
///       the reader might expect:
///  `&[&str]`,
///  a list of UTF-8 texts.
/// Why:
///   Command words arrive as raw bytes and are compared without decoding,
///  so the
///       table holds bytes.
///  A test compares the whole table with real Git's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GIT_BUILTIN_COMMANDS: ReadonlySet<string> = new Set(['add', 'am', /* ... */]);
/// ```
pub const GIT_BUILTIN_COMMANDS: &[&[u8]] = &[
    b"add",
    b"am",
    b"annotate",
    b"apply",
    b"archive",
    b"backfill",
    b"bisect",
    b"blame",
    b"branch",
    b"bugreport",
    b"bundle",
    b"cat-file",
    b"check-attr",
    b"check-ignore",
    b"check-mailmap",
    b"check-ref-format",
    b"checkout",
    b"checkout--worker",
    b"checkout-index",
    b"cherry",
    b"cherry-pick",
    b"clean",
    b"clone",
    b"column",
    b"commit",
    b"commit-graph",
    b"commit-tree",
    b"config",
    b"count-objects",
    b"credential",
    b"credential-cache",
    b"credential-cache--daemon",
    b"credential-store",
    b"describe",
    b"diagnose",
    b"diff",
    b"diff-files",
    b"diff-index",
    b"diff-pairs",
    b"diff-tree",
    b"difftool",
    b"fast-export",
    b"fast-import",
    b"fetch",
    b"fetch-pack",
    b"fmt-merge-msg",
    b"for-each-ref",
    b"for-each-repo",
    b"format-patch",
    b"format-rev",
    b"fsck",
    b"fsck-objects",
    b"fsmonitor--daemon",
    b"gc",
    b"get-tar-commit-id",
    b"grep",
    b"hash-object",
    b"help",
    b"history",
    b"hook",
    b"index-pack",
    b"init",
    b"init-db",
    b"interpret-trailers",
    b"last-modified",
    b"log",
    b"ls-files",
    b"ls-remote",
    b"ls-tree",
    b"mailinfo",
    b"mailsplit",
    b"maintenance",
    b"merge",
    b"merge-base",
    b"merge-file",
    b"merge-index",
    b"merge-ours",
    b"merge-recursive",
    b"merge-recursive-ours",
    b"merge-recursive-theirs",
    b"merge-subtree",
    b"merge-tree",
    b"mktag",
    b"mktree",
    b"multi-pack-index",
    b"mv",
    b"name-rev",
    b"notes",
    b"pack-objects",
    b"pack-redundant",
    b"pack-refs",
    b"patch-id",
    b"pickaxe",
    b"prune",
    b"prune-packed",
    b"pull",
    b"push",
    b"range-diff",
    b"read-tree",
    b"rebase",
    b"receive-pack",
    b"reflog",
    b"refs",
    b"remote",
    b"remote-ext",
    b"remote-fd",
    b"repack",
    b"replace",
    b"replay",
    b"repo",
    b"rerere",
    b"reset",
    b"restore",
    b"rev-list",
    b"rev-parse",
    b"revert",
    b"rm",
    b"send-pack",
    b"shortlog",
    b"show",
    b"show-branch",
    b"show-index",
    b"show-ref",
    b"sparse-checkout",
    b"stage",
    b"stash",
    b"status",
    b"stripspace",
    b"submodule--helper",
    b"switch",
    b"symbolic-ref",
    b"tag",
    b"unpack-file",
    b"unpack-objects",
    b"update-index",
    b"update-ref",
    b"update-server-info",
    b"upload-archive",
    b"upload-archive--writer",
    b"upload-pack",
    b"url-parse",
    b"var",
    b"verify-commit",
    b"verify-pack",
    b"verify-tag",
    b"version",
    b"whatchanged",
    b"worktree",
    b"write-tree",
];

/// What:
///  Whether `word` is a command built into Git 2.56.0.
///  `&[u8]` borrows the word's
///       raw bytes;
///  `bool` is true or false.
/// Why:
///   A built-in word cannot be an alias,
///  so what the word says is what Git runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isGitBuiltin(word: string): boolean { return GIT_BUILTIN_COMMANDS.has(word); }
/// ```
pub fn is_git_builtin(word: &[u8]) -> bool {
    // `.contains(&word)` borrows the word for the lookup.
    return GIT_BUILTIN_COMMANDS.contains(&word);
}

/// The comparison with real Git stays out of the release executable.
#[cfg(test)]
#[path = "git_builtins_tests.rs"]
mod tests;
