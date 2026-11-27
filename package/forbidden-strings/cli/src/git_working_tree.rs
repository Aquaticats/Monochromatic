//! What:
//!  Where Git keeps the administrative data of the working tree a scan walks:
//!  the index recording that working tree's tracked files,
//!  and the configuration recording the index's object format.
//! Why:
//!  `<root>/.git` is the main repository's layout only.
//!  A linked worktree holds a `.git` file whose `gitdir:` line names
//!  `<main>/.git/worktrees/<name>`,
//!  a submodule holds one naming `<host>/.git/modules/<name>`,
//!  and a scan started in a subfolder has no `.git` of its own at all,
//!  so every case finds its index the way Git's discovery does:
//!  from the walked directory upward,
//!  reading a `.git` file's `gitdir:` line,
//!  and validating the directory that line names.
//!  Joining `.git/index` onto the walked directory found none of those
//!  (issue 1232).
//!
//!  What it does not do,
//!  and why the README says so:
//!  `GIT_DIR` and `GIT_WORK_TREE` name a different worktree root,
//!  `core.worktree` stores one in configuration,
//!  and Git's discovery also stops at mount points and honours
//!  `GIT_CEILING_DIRECTORIES`.
//!  Those need Git's own configuration and ownership machinery,
//!  which this package deliberately does not link,
//!  so the walked directory stays the worktree root here.
//!  `GIT_INDEX_FILE` is honoured,
//!  because it names the very file this module opens.
//!
//!  The parts,
//!  in the order they build on each other:
//!  `path_normalization` settles the spelling every comparison needs;
//!  `dot_git_entry` reads what one `.git` entry says;
//!  `object_format` reads the index's object format from configuration;
//!  `discovery` walks upward and answers with the working tree.
//!  The re-exports below keep this module's surface exactly what the walker
//!  and the tests already import.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const discovery = discoverWorkingTree(walkRoot);
//! // const index = discovery.kind === "working-tree"
//! //   ? openIndex(discovery.tree.indexPath, discovery.tree.hashKind())
//! //   : null;
//! ```

/// What:
///  `mod discovery;` registers the upward walk and the answer it returns to
///  the walker.
/// Why:
///  This is the entry point every scan reaches.
mod discovery;
/// What:
///  `mod dot_git_entry;` registers what one directory in the walk upward
///  holds as a `.git` entry,
///  and how its directories are read.
/// Why:
///  A linked worktree and a submodule name their Git directory in a file,
///  and only the entry itself says which shape it is.
mod dot_git_entry;
/// What:
///  `mod object_format;` registers the reader of `extensions.objectformat`.
/// Why:
///  `gix-index` needs the format before it parses one entry,
///  and a wrong
///  format corrupts paths silently.
mod object_format;
/// What:
///  `mod path_normalization;` registers the absolute form,
///  the lexical
///  normalization and the scope prefix of the walked directory.
/// Why:
///  Index paths are `/`-separated and relative to the worktree root,
///  so a
///  subfolder scan needs one normalized spelling first.
mod path_normalization;

/// What:
///  `pub(crate) use discovery::{Discovery, GitWorkingTree, discover};`
///  re-exports the three names the walker matches on.
/// Why:
///  Moving them into a part must not move their path.
pub(crate) use discovery::{
    Discovery,
    GitWorkingTree,
    discover,
};
/// What:
///  `pub(crate) use dot_git_entry::gitdir_from_git_file;` re-exports the
///  `gitdir:` line parser for the tests that compare it against Git;
///  only
///  those tests name it,
///  so the re-export exists only for them.
/// Why:
///  The parser's contract is what a `.git` file's own bytes mean.
#[cfg(test)]
pub(crate) use dot_git_entry::gitdir_from_git_file;
/// What:
///  `pub(crate) use object_format::index_hash_kind;` re-exports the
///  object-format reader the walker asks for before opening an index.
/// Why:
///  The walked tree's `config` file is the input it takes.
pub(crate) use object_format::index_hash_kind;
/// What:
///  `pub(crate) use path_normalization::lexically_normalized;` re-exports
///  the lexical normalizer for the tests that pin which `..` is settled and
///  which is left to the filesystem;
///  only those tests name it,
///  so the
///  re-export exists only for them.
/// Why:
///  The function's edge cases are the test's subject.
#[cfg(test)]
pub(crate) use path_normalization::lexically_normalized;

/// Registers the `tests` child module.
// What:     `#[cfg(test)] #[path = "git_working_tree_tests.rs"] mod tests;`
//           declares a test-only submodule whose code lives in the sibling
//           file, so it compiles only under `cargo nextest run` and still
//           reaches this file's private items through `super::`.
// Why:      Keeps the production file inside its line budget while the
//           fixtures stay beside the code they cover.
//
// In TS you'd write (pseudocode):
// ```ts
// // git-working-tree.unit.test.ts, run only by the test runner
// ```
#[cfg(test)]
#[path = "git_working_tree_tests.rs"]
mod tests;
