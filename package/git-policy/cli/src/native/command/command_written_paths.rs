//! What:
//!  Which tree paths one guarded form writes into the worktree,
//!  read from the form's own words.
//! Why:
//!  The guard has to know what a command would overwrite before it can save it:
//!  a status names the worktree's own changes,
//!  never the files Git did not put there,
//!  and an untracked or ignored file that stands at a path the form writes is lost
//!  whether or not anything tracked is dirty.
//!  A form that writes a tree writes every path of it,
//!  a form that names paths writes those of them the tree holds,
//!  and a restore without a source writes the index's own paths.
//!  Reading the words here keeps that question in one place,
//!  where the same option tables Git uses decide what the words mean.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writtenPaths('ResetHard', ['--hard', 'other']) === { kind: 'tree', revision: 'other' }
//! // writtenPaths('CheckoutPaths', ['other', '--', 'b.txt']) === { kind: 'matching-tree', revision: 'other', pathspecs: ['b.txt'] }
//! ```

use crate::command_checkout_table::{CHECKOUT_TABLE, FORCE, SWITCH_TABLE};
use crate::command_discard::{
    READ_TREE_DEBUG_UNPACK, READ_TREE_DRY_RUN, READ_TREE_MERGE, READ_TREE_PREFIX, READ_TREE_RESET, READ_TREE_TABLE,
    READ_TREE_UPDATE, restores_index_only, spells_long,
};
/// Git's own option grammar, for where a region's options end and its paths begin.
use crate::command_options::{Boundary, DEFAULT_MODE, ParsedOptions, parse_options};
/// The token indices that are positionals rather than options,
///  and the exact bytes of an option's value.
use crate::command_options_query::{is_enabled, positional_tokens, value_bytes};
use crate::command_reset::{RESET_TABLE, ResetMode, parse_reset_region};
/// The forms the linked-worktree policy guards.
use crate::rule_linked_worktree::GuardedCommand;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `OsStr`/`Path` are borrowed operating-system text and paths.
use std::path::Path;

/// What:
///  Which pathspecs a form reads from a file are matched against:
///  a tree, or the index.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   `git checkout <revision> --pathspec-from-file=<file>` writes the tree's paths that the file's
///  pathspecs match,
///  while a `checkout` with no revision and a `restore` with no source write the index's,
///  and the two readings ask Git different questions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathspecTarget = { kind: 'tree'; revision?: string } | { kind: 'index' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathspecTarget {
    /// The tree the form names,
    ///  or `HEAD` when it names none.
    Tree(Option<OsString>),
    /// The paths the index holds.
    Index,
}

/// What:
///  One word of a command as the operating system takes it,
///  from the bytes the person typed or Git printed.
/// Why:
///  A prefix,
///  a source and a file name are paths and revisions,
///  and Git holds any bytes in a name;
///  a conversion through `String` replaces every byte that is not UTF-8
///  and then names another file,
///  which is how a forced form wrote over a file the guard had not found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = (bytes) => Buffer.from(bytes); // already the name
/// ```
fn word_of(bytes: &[u8]) -> OsString {
    return crate::path_bytes::os_from_path_bytes(bytes);
}

/// What:
///  Which of the worktree's entries at a written path Git writes over:
///  every one, or only the ones its ignore rules cover.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A forced move of `HEAD` replaces or deletes any entry in its way,
///  untracked or ignored alike;
///  a `reset --merge` or `reset --keep` refuses the untracked one and writes over the ignored one
///  (measured on this host with native Git 2.56.0:
///  `Untracked working tree file 'b.txt' would be overwritten` and stop,
///  against an ignored `b.txt` replaced with exit 0),
///  so asking about the untracked entries there would copy a file Git keeps
///  and leave a notice that says the opposite.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrittenEntries = 'every-entry' | 'only-ignored';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WrittenEntries {
    /// Every entry at the path, whether Git's rules cover it or not.
    EveryEntry,
    /// Only the entries the worktree's ignore rules cover.
    OnlyIgnored,
}

/// What:
///  The paths one form writes, as the source of them.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A whole tree is written by a forced move of `HEAD`,
///  a named part of one by a path form,
///  and a restore without a source writes what the index holds;
///  each answer is a different query,
///  and the pathspecs alone decide which of those paths the form reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrittenPaths = { kind: 'tree'; revision?: string; entries: WrittenEntries }
///   | { kind: 'matching-tree'; revision?: string; pathspecs: string[] }
///   | { kind: 'index'; pathspecs: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WrittenPaths {
    /// Every path of the revision's tree,
    ///  `HEAD` when the form names none.
    Tree {
        /// The revision the form names,
        ///  or nothing for `HEAD`.
        revision: Option<OsString>,
        /// Which entries at those paths Git writes over.
        entries: WrittenEntries,
        /// What:
        ///  Whether the command line leaves `checkout` and `switch` free to turn the name into
        ///  the local branch a remote-tracking branch of that name stands for.
        /// Why:
        ///  `git ls-tree topic` fails while `git checkout -f topic` creates `topic` from
        ///  `origin/topic` and writes that tree over the worktree
        ///  (measured on this host with native Git 2.56.0,
        ///  2026-11-13),
        ///  so a reading that treats the failure as "nothing is written" lets the forced form
        ///  overwrite an untracked file it never looked at.
        ///  `reset`, `restore` and `read-tree` name no such branch
        ///  (`fatal: ambiguous argument 'topic'`,
        ///  `fatal: could not resolve 'topic'`,
        ///  `fatal: Not a valid object name topic`),
        ///  so they keep the flag false.
        guesses_a_remote_branch: bool,
    },
    /// The paths of the revision's tree that the form's own pathspecs match.
    MatchingTree {
        /// The revision the form names,
        ///  or nothing for `HEAD`.
        revision: Option<OsString>,
        /// The paths the form names,
        ///  which Git matches against the tree.
        pathspecs: Vec<OsString>,
    },
    /// The paths the index holds,
    ///  which a restore without a source writes over.
    Index {
        /// The paths the form names,
        ///  and nothing for every path of the index.
        pathspecs: Vec<OsString>,
    },
    /// The paths the index holds,
    ///  written below a directory the form names.
    IndexUnderPrefix {
        /// The paths the form names,
        ///  and nothing for every path of the index.
        pathspecs: Vec<OsString>,
        /// The directory they are written below.
        prefix: OsString,
    },
    /// What:
    ///  A forced `git mv`: the sources and the destination it names,
    ///  which only the worktree decides what Git writes.
    /// Why:
    ///  Git moves a source *into* a destination that is a directory (`git mv -f a.txt b` writes
    ///  `b/a.txt` and leaves `b` and everything else in it alone),
    ///  and renames onto a destination that is not one
    ///  (measured on this host with native Git 2.56.0, 2026-11-14:
    ///  one source into a directory, several sources into one, a source onto a file, a directory
    ///  onto a directory, a missing destination, a trailing slash, and the shapes Git refuses).
    ///  A reading that put the destination itself in the written set named the directory a form
    ///  replaces whole and refused a safe `git mv -f a.txt b`, which is the false refusal the third
    ///  review found.
    MoveInto {
        /// The sources, as the person spelled them.
        sources: Vec<OsString>,
        /// The destination, as the person spelled it.
        destination: OsString,
        /// The directory a relative path of the form is read from,
        ///  which is the shell's for a command a `!` alias runs
        ///  and the command's own otherwise.
        directory: Option<OsString>,
    },
    /// The paths of the tree or the index that the pathspecs of `--pathspec-from-file` match.
    PathspecFile {
        /// What the pathspecs are matched against.
        target: PathspecTarget,
        /// The file the form reads them from,
        ///  as the form spelled it.
        file: OsString,
        /// Whether the file separates them with NUL bytes instead of line ends.
        nul_separated: bool,
        /// The directory the file's name is relative to,
        ///  which is the shell's for a command a `!` alias runs
        ///  and the command's own otherwise.
        directory: Option<OsString>,
        /// What:
        ///  Which entries at the written paths Git writes over,
        ///  when the file holds no pathspec at all.
        /// Why:
        ///  Git reads a zero-byte file as "no pathspecs were given"
        ///  (`git checkout -f other --pathspec-from-file=<empty>` switches the branch by force,
        ///  measured on this host with native Git 2.56.0, 2026-11-13),
        ///  so the form becomes the whole-tree form and its forced or ordinary answer decides the
        ///  entries: a forced one writes over everything, an ordinary one keeps an untracked file
        ///  and writes over the ignored one.
        entries: WrittenEntries,
        /// What:
        ///  Whether the command line leaves `checkout` free to guess the remote branch the name stands
        ///  for, which it does for the whole-tree form an empty file leaves behind.
        guesses_a_remote_branch: bool,
    },
    /// The paths of every tree a `git read-tree -u` names,
    ///  each below the prefix the form writes them under.
    ReadTree {
        /// The trees the form names,
        ///  whose merged paths it writes into the worktree.
        revisions: Vec<OsString>,
        /// The directory the trees land in,
        ///  when the form writes one with `--prefix`.
        prefix: Option<OsString>,
        /// Which entries at those paths Git writes over.
        entries: WrittenEntries,
    },
}

impl WrittenPaths {
    /// What:
    ///  The word this answer is named by,
    ///  for a record of the question that was asked.
    /// Why:
    ///   A record of what the guard measured names the reading it made,
    ///  and the three readings are not the same question.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// written.kind === 'tree' | 'matching-tree' | 'index'
    /// ```
    pub fn kind(&self) -> &'static str {
        return match self {
            WrittenPaths::Tree { entries: WrittenEntries::EveryEntry, .. } => "tree",
            WrittenPaths::Tree { entries: WrittenEntries::OnlyIgnored, .. } => "tree-of-ignored-entries",
            WrittenPaths::MatchingTree { .. } => "matching-tree",
            WrittenPaths::Index { .. } => "index",
            WrittenPaths::IndexUnderPrefix { .. } => "index-under-a-prefix",
            WrittenPaths::MoveInto { .. } => "move-into",
            WrittenPaths::ReadTree { entries: WrittenEntries::EveryEntry, .. } => "read-tree-trees",
            WrittenPaths::ReadTree { entries: WrittenEntries::OnlyIgnored, .. } => "read-tree-trees-of-ignored-entries",
            WrittenPaths::PathspecFile { .. } => "pathspec-file",
        };
    }
}

/// What:
///  The tree paths one guarded form writes,
///  or nothing for a form that writes no file of a tree.
///  `&[OsString]` borrows the tokens after the subcommand.
/// Why:
///   A forced move of `HEAD` and a path form write files;
///  a `clean` deletes what Git's own dry run names,
///  and a `rm` and a `mv` reach only paths their index already holds,
///  so none of those is read here.
///  A hard reset writes over every entry at the tree's paths;
///  `reset --merge` and `reset --keep` write the tree too,
///  but refuse an untracked entry and write over an ignored one,
///  so their reading asks about the ignored entries alone.
///  A `read-tree -u` writes the paths of every tree it names,
///  below the directory `--prefix` puts them in,
///  and which entries it writes over depends on the mode it carries:
///  `--reset` writes over an untracked file as well,
///  while `-m` alone refuses one and writes over an ignored one
///  (`SPEC.md`, the discarding-commands section, holds both measurements).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writtenPaths(command: GuardedCommand, region: string[]): WrittenPaths | undefined;
/// ```
pub fn written_paths(command: GuardedCommand, region: &[OsString], shell_directory: Option<&Path>) -> Option<WrittenPaths> {
    match command {
        GuardedCommand::ResetHard => {
            // `--soft` and `--mixed` write no worktree file; `--merge` and `--keep` write the tree but refuse an
            //  untracked entry in its way, so only the entries their ignore rules cover are asked about.
            let parsed = parse_reset_region(region, &[]).ok()?;
            let revision: Option<OsString> = crate::command_discard::revision_before_paths(region, RESET_TABLE);
            return match parsed.mode {
                Some(ResetMode::Hard) => Some(WrittenPaths::Tree {
                    revision,
                    entries: WrittenEntries::EveryEntry,
                    // A reset resolves a revision and never the remote branch a short name stands for.
                    guesses_a_remote_branch: false,
                }),
                Some(ResetMode::Merge) | Some(ResetMode::Keep) => Some(WrittenPaths::Tree {
                    revision,
                    entries: WrittenEntries::OnlyIgnored,
                    guesses_a_remote_branch: false,
                }),
                _ => None,
            };
        }
        GuardedCommand::CheckoutPaths => return checkout_written_paths(region, shell_directory),
        GuardedCommand::SwitchForce => {
            return Some(WrittenPaths::Tree {
                revision: first_positional(region, SWITCH_TABLE),
                entries: WrittenEntries::EveryEntry,
                guesses_a_remote_branch: guess_is_not_switched_off(region),
            });
        }
        GuardedCommand::RestoreWorktree => return restore_written_paths(region, shell_directory),
        GuardedCommand::ReadTree => return read_tree_written_paths(region),
        GuardedCommand::Move => return move_written_paths(region, shell_directory),
        GuardedCommand::CheckoutIndex => return checkout_index_written_paths(region),
        _ => return None,
    }
}

/// What:
///  What a forced `git mv` names: its sources and its destination,
///  which the worktree resolves into the paths Git writes.
///  `&[OsString]` borrows the tokens after the subcommand,
///  and `Option<&Path>` the directory a `!` alias's shell runs in.
/// Why:
///   Git moves each source into a destination that is a directory
///  (`git mv -f a.txt b` writes `b/a.txt`, and leaves `b/` and its other files alone),
///  renames the one source onto a destination that is not one,
///  and refuses every other shape it cannot do:
///  several sources onto a non-directory (`destination 'dest.txt' is not a directory`),
///  a trailing slash on a destination that is not there (`destination directory does not exist`),
///  a directory onto an existing file (`destination already exists`),
///  a source onto a directory of its name inside the destination (`Cannot overwrite`),
///  and a directory into itself (`can not move directory into itself`).
///  Everything after `--` is a path,
///  and before it every token that is no option of `mv` is one,
///  none of which takes a value.
///  The last word is the destination and the words before it are the sources.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveWrittenPaths(region: string[]): WrittenPaths | undefined;
/// ```
fn move_written_paths(region: &[OsString], shell_directory: Option<&Path>) -> Option<WrittenPaths> {
    let mut words: Vec<&OsString> = Vec::<&OsString>::new();
    let mut after_dash_dash: bool = false;
    for token in region {
        let written: &[u8] = token.as_encoded_bytes();
        if !after_dash_dash && written == b"--" {
            after_dash_dash = true;
            continue;
        }
        if !after_dash_dash && written.len() > 1 && written[0] == b'-' {
            continue;
        }
        words.push(token);
    }
    let destination: &OsString = words.last()?;
    let mut sources: Vec<OsString> = Vec::<OsString>::with_capacity(words.len().saturating_sub(1));
    for source in words.iter().take(words.len().saturating_sub(1)) {
        sources.push((**source).clone());
    }
    if sources.is_empty() {
        return None;
    }
    return Some(WrittenPaths::MoveInto {
        sources,
        destination: destination.clone(),
        directory: shell_directory.map(|directory| return directory.as_os_str().to_os_string()),
    });
}

/// What:
///  The paths a forced `git checkout-index` writes:
///  the index's paths the form names, or all of them with `-a`/`--all`,
///  each below the directory `--prefix` names when it names one.
///  `&[OsString]` borrows the tokens after the subcommand.
/// Why:
///  `git checkout-index --force` writes over whatever stands at those paths,
///  and with `--prefix` it writes them below a directory no index query names,
///  which is where the worktree entry in the way is found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutIndexWrittenPaths(region: string[]): WrittenPaths | undefined;
/// ```
fn checkout_index_written_paths(region: &[OsString]) -> Option<WrittenPaths> {
    let names: &[&str] = crate::command_discard::CHECKOUT_INDEX_LONG_OPTIONS;
    let mut all: bool = false;
    let mut prefix: Option<OsString> = None;
    let mut pathspecs: Vec<OsString> = Vec::<OsString>::new();
    let mut at: usize = 0;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        at += 1;
        if written == b"--" {
            pathspecs.extend(region[at..].iter().cloned());
            break;
        }
        if written.len() > 1 && written[0] == b'-' && !written.starts_with(b"--") {
            if written.contains(&b'a') {
                all = true;
            }
            continue;
        }
        if spells_long(written, "all", names) {
            all = true;
            continue;
        }
        if spells_long(written, "prefix", names) {
            match written.iter().position(|byte| return *byte == b'=') {
                Some(equals) => prefix = Some(word_of(&written[equals + 1..])),
                None => prefix = region.get(at).cloned(),
            }
            continue;
        }
        if written.starts_with(b"--") {
            continue;
        }
        pathspecs.push(region[at - 1].clone());
    }
    let named: Vec<OsString> = if all { Vec::<OsString>::new() } else { pathspecs };
    return Some(match prefix {
        Some(directory) => WrittenPaths::IndexUnderPrefix { pathspecs: named, prefix: directory },
        None => WrittenPaths::Index { pathspecs: named },
    });
}

/// What:
///  The paths a `git read-tree` form writes into the worktree,
///  which are the paths of every tree it names,
///  each below the directory its `--prefix` puts it in.
/// Why:
///   Git writes no worktree file without `-u`,
///  and refuses `-u` unless the form also carries `-m`,
///  `--reset` or `--prefix`,
///  so only those forms write anything here.
///  Every path a merge can write comes from one of the trees it names,
///  so asking each of them is the whole set.
///  Measured on this host with native Git 2.56.0:
///  `--reset -u` writes over an untracked file at such a path and exits 0,
///  while `-m -u` and `--prefix` refuse an untracked one in Git's own words
///  and write over the one their ignore rules cover,
///  which is why only the reset form asks about the untracked entries too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTreeWrittenPaths(region: string[]): WrittenPaths | undefined;
/// ```
fn read_tree_written_paths(region: &[OsString]) -> Option<WrittenPaths> {
    let parsed: ParsedOptions = parse_options(region, READ_TREE_TABLE, DEFAULT_MODE, &[]).ok()?;
    // A dry run and `--debug-unpack` write no worktree file at all.
    if !is_enabled(&parsed, READ_TREE_UPDATE)
        || is_enabled(&parsed, READ_TREE_DRY_RUN)
        || is_enabled(&parsed, READ_TREE_DEBUG_UNPACK)
    {
        return None;
    }
    let reset: bool = is_enabled(&parsed, READ_TREE_RESET);
    // A region Git refuses writes nothing; Git says so itself.
    if !reset && !is_enabled(&parsed, READ_TREE_MERGE) && !is_enabled(&parsed, READ_TREE_PREFIX) {
        return None;
    }
    let mut revisions: Vec<OsString> = Vec::<OsString>::new();
    for index in positional_tokens(&parsed, region.len()) {
        if let Some(token) = region.get(index) {
            revisions.push(token.clone());
        }
    }
    if revisions.is_empty() {
        return None;
    }
    let mut prefix: Option<OsString> = None;
    for occurrence in &parsed.occurrences {
        if occurrence.id == READ_TREE_PREFIX
            && let Some(value) = occurrence.value
        {
            prefix = Some(word_of(value_bytes(region, value)));
        }
    }
    return Some(WrittenPaths::ReadTree {
        revisions,
        prefix,
        entries: if reset { WrittenEntries::EveryEntry } else { WrittenEntries::OnlyIgnored },
    });
}

/// What:
///  The paths a `git checkout` form writes.
/// Why:
///   A forced checkout with no path switches the whole tree;
///  a path form writes only the paths it names,
///  read from the revision before them or from the index when it names none,
///  and `git checkout <words> -- <paths>` is the spelling that always means paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutWrittenPaths(region: string[]): WrittenPaths | undefined; // throws-free
/// ```
fn checkout_written_paths(region: &[OsString], shell_directory: Option<&Path>) -> Option<WrittenPaths> {
    let parsed: ParsedOptions = parse_options(region, CHECKOUT_TABLE, DEFAULT_MODE, &[]).ok()?;
    // A form that reads its pathspecs from a file reaches paths no token names, so the file is where the
    //  written paths are read.
    if let Some((file, nul_separated)) = pathspec_file(region) {
        return Some(pathspec_file_written_paths(region, file, nul_separated, shell_directory, is_enabled(&parsed, FORCE)));
    }
    let names: Vec<&OsString> = positional_tokens(&parsed, region.len())
        .into_iter()
        .filter_map(|index| return region.get(index))
        .collect();
    // Everything after `--` is a path, and the one word before it that is no option is the revision Git reads.
    if let Boundary::DashDash(boundary) = parsed.boundary {
        let pathspecs: Vec<OsString> = region.get(boundary + 1..).unwrap_or(&[]).to_vec();
        let mut revision: Option<OsString> = None;
        for index in positional_tokens(&parsed, region.len()) {
            if index < boundary {
                revision = region.get(index).cloned();
                break;
            }
        }
        return Some(WrittenPaths::MatchingTree { revision, pathspecs });
    }
    if names.len() > 1 {
        let revision: Option<OsString> = Some(names[0].clone());
        return Some(WrittenPaths::MatchingTree {
            revision,
            pathspecs: names[1..].iter().map(|name| return (*name).clone()).collect(),
        });
    }
    if !is_enabled(&parsed, FORCE) {
        return None;
    }
    return Some(WrittenPaths::Tree {
        revision: names.first().map(|name| return (*name).clone()),
        entries: WrittenEntries::EveryEntry,
        guesses_a_remote_branch: guess_is_not_switched_off(region),
    });
}

/// What:
///  Whether a `checkout` or `switch` region leaves Git free to guess the remote branch a short
///  name stands for.
///  `&[OsString]` borrows the tokens after the subcommand.
/// Why:
///  `--no-guess` and `checkout.guess=false` both turn the guess off,
///  and Git then refuses the name itself
///  (`error: pathspec 'topic' did not match any file(s) known to git`, measured on this host with
///  native Git 2.56.0, 2026-11-13);
///  the guess the guard makes must follow the same switch,
///  or it would resolve a branch Git never touches.
///  `--guess` and `--gu` are the same option Git accepts by that prefix,
///  and nothing else in the two tables starts with `guess`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guesses = (region) => !region.some((word) => spellsLong(word, 'no-guess'));
/// ```
fn guess_is_not_switched_off(region: &[OsString]) -> bool {
    for token in region {
        let written: &[u8] = token.as_encoded_bytes();
        if written == b"--" {
            return true;
        }
        let Some(name) = written.strip_prefix(b"--") else {
            continue;
        };
        if b"guess".starts_with(name) || b"no-guess".starts_with(name) && name.len() > b"no-".len() {
            // `--guess` (any prefix) leaves the guess on; `--no-guess` (any prefix past `--no-`) turns it off.
            return !name.starts_with(b"no-");
        }
    }
    return true;
}

/// What:
///  The paths a `git restore` form writes over,
///  read from its `--source` and its own path list.
/// Why:
///  Without a source the index's content goes back over the worktree,
///  and with one the source tree's content does;
///  a form that restores only the index writes no worktree file at all.
///  The paths are the tokens after `--` when it is written,
///  and otherwise every token that is not an option or the value of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreWrittenPaths(region: string[]): WrittenPaths | undefined;
/// ```
fn restore_written_paths(region: &[OsString], shell_directory: Option<&Path>) -> Option<WrittenPaths> {
    if restores_index_only(region) {
        return None;
    }
    let source: Option<OsString> = restore_source(region);
    if let Some((file, nul_separated)) = pathspec_file(region) {
        return Some(WrittenPaths::PathspecFile {
            target: match source {
                Some(revision) => PathspecTarget::Tree(Some(revision)),
                None => PathspecTarget::Index,
            },
            file,
            nul_separated,
            directory: shell_directory.map(|directory| return directory.as_os_str().to_os_string()),
            // Git refuses a restore with no path at all: `fatal: you must specify path(s) to restore`
            //  (measured on this host with native Git 2.56.0, 2026-11-13), so only the refusal follows an empty file.
            entries: WrittenEntries::EveryEntry,
            guesses_a_remote_branch: false,
        });
    }
    let pathspecs: Vec<OsString> = restore_pathspecs(region);
    return Some(match source {
        Some(revision) => WrittenPaths::MatchingTree { revision: Some(revision), pathspecs },
        None => WrittenPaths::Index { pathspecs },
    });
}

/// What:
///  The revision a `git restore` form names as its source,
///  in any spelling Git's own table accepts.
/// Why:
///   `--source <tree>`, `--source=<tree>`, `-s <tree>` and `-s<tree>` are one option,
///  and a value that looks like a flag is that value (`restore --source -W -- f`).
///  An option written twice keeps the last value, as Git's own parser does,
///  so every occurrence is read and the last one is the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreSource(region: string[]): string | undefined;
/// ```
fn restore_source(region: &[OsString]) -> Option<OsString> {
    let mut at: usize = 0;
    let mut source: Option<OsString> = None;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        at += 1;
        if written == b"--" {
            break;
        }
        if let Some(value) = long_option_value(written, "source") {
            // Git's own option parser keeps the last value of an option written twice
            //  (`git restore -s main -s other -- f.txt` writes `other`'s content,
            //  measured on this host with native Git 2.56.0, 2026-11-13), so a reading that stopped at the first
            //  one asked about a tree Git never reads.
            source = Some(value);
            continue;
        }
        let long_without_value: bool =
            written.starts_with(b"--") && !written.contains(&b'=') && spells_long(written, "source", crate::command_discard::RESTORE_LONG_OPTIONS);
        if long_without_value {
            if let Some(value) = region.get(at) {
                source = Some(value.clone());
            }
            at += 1;
            continue;
        }
        if written.starts_with(b"--") {
            if !written.contains(&b'=') && option_takes_a_value(written) {
                at += 1;
            }
            continue;
        }
        if written.len() > 1 && written[0] == b'-' {
            let cluster: &[u8] = &written[1..];
            let position: usize = match cluster.iter().position(|byte| return *byte == b's') {
                Some(found) => found,
                None => {
                    continue;
                }
            };
            let value: &[u8] = &cluster[position + 1..];
            if !value.is_empty() {
                source = Some(word_of(value));
                continue;
            }
            if let Some(next) = region.get(at) {
                source = Some(next.clone());
            }
            at += 1;
        }
    }
    return source;
}

/// What:
///  The value a long option carries inside its own token (`--name=value`),
///  by any abbreviation Git accepts for `restore`.
/// Why:
///   Git accepts an unambiguous prefix of every long option,
///  and `--so`/`--sour` are `--source`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function longOptionValue(token: string, name: string): string | undefined; // the part after '='
/// ```
fn long_option_value(written: &[u8], name: &str) -> Option<OsString> {
    if !spells_long(written, name, crate::command_discard::RESTORE_LONG_OPTIONS) {
        return None;
    }
    let equals: usize = written.iter().position(|byte| return *byte == b'=')?;
    return Some(word_of(&written[equals + 1..]));
}

/// What:
///  Whether one long option of `git restore` takes the next token as its value.
/// Why:
///   `--source` and `--conflict` do,
///  and every other long option of this command does not,
///  so the token after one of them is a value and never a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const optionTakesAValue = (token) => ['source', 'conflict'].some((name) => spellsLong(token, name));
/// ```
fn option_takes_a_value(written: &[u8]) -> bool {
    let names: [&str; 2] = ["source", "conflict"];
    return names.iter().any(|name| return spells_long(written, name, crate::command_discard::RESTORE_LONG_OPTIONS));
}

/// What:
///  The paths a `git restore` form names,
///  which are the tokens after `--` when it is written and the plain words of the region otherwise.
/// Why:
///   Everything after `--` is a path to Git,
///  and before it every token that is not an option and not the value of one is a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restorePathspecs(region: string[]): string[];
/// ```
fn restore_pathspecs(region: &[OsString]) -> Vec<OsString> {
    let mut paths: Vec<OsString> = Vec::<OsString>::new();
    let mut at: usize = 0;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        at += 1;
        if written == b"--" {
            paths.extend(region[at..].iter().cloned());
            return paths;
        }
        if written.starts_with(b"--") {
            if !written.contains(&b'=') && option_takes_a_value(written) {
                at += 1;
            }
            continue;
        }
        if written.len() > 1 && written[0] == b'-' {
            // A short cluster that ends with `s` takes the next token as its value.
            if written[1..].ends_with(b"s") {
                at += 1;
            }
            continue;
        }
        paths.push(region[at - 1].clone());
    }
    return paths;
}

/// What:
///  The file a region reads its pathspecs from,
///  and whether it separates them with NUL bytes,
///  written in either spelling Git's own table accepts.
/// Why:
///  `--pathspec-from-file <file>`, `--pathspec-from-file=<file>` and the abbreviations Git accepts
///  are one option,
///  and `--pathspec-file-nul` decides how its entries are separated.
///  A file named here reaches paths no other word of the command names,
///  so a reading that ignores it asks about nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathspecFile = (region) => scan(region, ['pathspec-from-file', 'pathspec-file-nul']);
/// ```
fn pathspec_file(region: &[OsString]) -> Option<(OsString, bool)> {
    let names: [&str; 2] = ["pathspec-from-file", "pathspec-file-nul"];
    let mut file: Option<OsString> = None;
    let mut nul_separated: bool = false;
    let mut at: usize = 0;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        at += 1;
        if written == b"--" {
            break;
        }
        if spells_long(written, "pathspec-file-nul", &names) {
            nul_separated = true;
            continue;
        }
        if !spells_long(written, "pathspec-from-file", &names) {
            continue;
        }
        match written.iter().position(|byte| return *byte == b'=') {
            Some(equals) => file = Some(word_of(&written[equals + 1..])),
            None => file = region.get(at).cloned(),
        }
    }
    return file.map(|named| return (named, nul_separated));
}

/// What:
///  The written paths of a form that reads its pathspecs from a file,
///  matched against the tree it names or against the index when it names none.
/// Why:
///  A `checkout <revision> --pathspec-from-file=<file>` writes the tree's paths the file names,
///  and a form with no revision writes the index's,
///  which is the same pair the named-path forms ask about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fromFile = (region, file, nul, shell) => ({ kind: 'pathspec-file', target: revision(region) ?? 'index', file, nul, shell });
/// ```
fn pathspec_file_written_paths(
    region: &[OsString],
    file: OsString,
    nul_separated: bool,
    shell_directory: Option<&Path>,
    forced: bool,
) -> WrittenPaths {
    // The tree-ish is the last word that is neither an option nor the file's name.
    let read: Option<ParsedOptions> = parse_options(region, CHECKOUT_TABLE, DEFAULT_MODE, &[]).ok();
    let mut revision: Option<OsString> = None;
    if let Some(parsed) = read {
        for index in positional_tokens(&parsed, region.len()) {
            if let Some(token) = region.get(index)
                && token.as_encoded_bytes() != file.as_encoded_bytes()
            {
                revision = Some(token.clone());
            }
        }
    }
    return WrittenPaths::PathspecFile {
        target: match revision {
            Some(named) => PathspecTarget::Tree(Some(named)),
            None => PathspecTarget::Index,
        },
        file,
        nul_separated,
        directory: shell_directory.map(|directory| return directory.as_os_str().to_os_string()),
        entries: if forced { WrittenEntries::EveryEntry } else { WrittenEntries::OnlyIgnored },
        guesses_a_remote_branch: guess_is_not_switched_off(region),
    };
}

/// What:
///  The first positional word of a region,
///  which a `switch` reads as the branch or commit it moves to.
/// Why:
///   `git switch -f <target>` writes the target's whole tree,
///  and every option before it is not part of the target's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstPositional(region: string[], table: OptionSpec[]): string | undefined;
/// ```
fn first_positional(region: &[OsString], table: &[crate::command_options::OptionSpec]) -> Option<OsString> {
    let parsed: ParsedOptions = parse_options(region, table, DEFAULT_MODE, &[]).ok()?;
    let index: &usize = parsed.leading.first()?;
    return region.get(*index).cloned();
}

/// Reading the forms' words into the paths they write,
/// beside real-Git controls of each spelling.
#[cfg(test)]
#[path = "command_written_paths_tests.rs"]
mod tests;
