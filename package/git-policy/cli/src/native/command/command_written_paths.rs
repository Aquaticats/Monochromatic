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

use crate::command_checkout_index_table::{
    CHECKOUT_INDEX_ALL, CHECKOUT_INDEX_PREFIX, CHECKOUT_INDEX_STDIN, CHECKOUT_INDEX_TABLE,
};
use crate::command_checkout_table::{CHECKOUT_TABLE, FORCE, GUESS, PATHSPEC_FILE_NUL, PATHSPEC_FROM_FILE, SWITCH_TABLE};
use crate::command_discard::{
    READ_TREE_DEBUG_UNPACK, READ_TREE_DRY_RUN, READ_TREE_MERGE, READ_TREE_PREFIX, READ_TREE_RESET, READ_TREE_TABLE,
    READ_TREE_UPDATE,
};
/// Git's own option grammar, for where a region's options end and its paths begin.
use crate::command_options::{Boundary, DEFAULT_MODE, ParsedOptions, parse_options};
/// The token indices that are positionals rather than options,
///  the final state of one option, and the exact bytes of its value.
use crate::command_options_query::{is_enabled, last_occurrence, positional_tokens, value_bytes};
/// The command's own table of `git mv`, and its rows of the values a move reads.
use crate::command_mv_table::MV_TABLE;
/// The command's own table of `git restore`, and the rows of the values it reads.
use crate::command_restore_table::{
    RESTORE_PATHSPEC_FILE_NUL, RESTORE_PATHSPEC_FROM_FILE, RESTORE_SOURCE, RESTORE_STAGED, RESTORE_TABLE,
    RESTORE_WORKTREE,
};
use crate::command_reset::{RESET_TABLE, ResetMode, parse_reset_region};
/// The forms the linked-worktree policy guards.
use crate::rule_linked_worktree::GuardedCommand;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

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
    ///  The directory those words mean is not part of this answer:
    ///  it is where the reading starts, which the reading knows.
    MoveInto {
        /// The sources, as the person spelled them.
        sources: Vec<OsString>,
        /// The destination, as the person spelled it.
        destination: OsString,
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
pub fn written_paths(command: GuardedCommand, region: &[OsString]) -> Option<WrittenPaths> {
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
        GuardedCommand::CheckoutPaths => return checkout_written_paths(region),
        GuardedCommand::SwitchForce => {
            let parsed: ParsedOptions = parse_options(region, SWITCH_TABLE, DEFAULT_MODE, &[]).ok()?;
            let target: Option<OsString> = parsed.leading.first().and_then(|index| return region.get(*index).cloned());
            return Some(WrittenPaths::Tree {
                revision: target,
                entries: WrittenEntries::EveryEntry,
                guesses_a_remote_branch: guess_is_not_switched_off(&parsed),
            });
        }
        GuardedCommand::RestoreWorktree => return restore_written_paths(region),
        GuardedCommand::ReadTree => return read_tree_written_paths(region),
        GuardedCommand::Move => return move_written_paths(region),
        GuardedCommand::CheckoutIndex => return checkout_index_written_paths(region),
        _ => return None,
    }
}

/// What:
///  Whether a form that writes these paths writes over an edit a `skip-worktree` mark hides.
/// Why:
///  A mark says a file is outside the sparse definition,
///  so Git's own status never names an edit to it;
///  the guard asks about those edits exactly where Git replaces one,
///  and the written paths are what says where that is.
///  Measured on this host with native Git 2.56.0 (2026-11-15):
///  `git checkout other -- a.txt` and `git restore --source=other -- b.txt` replace the file,
///  and so does each of them with `--pathspec-from-file=<list>`
///  (the seventh review found that form missing);
///  `git mv -f a.txt b.txt` replaces the destination that way as well,
///  while `git checkout --force <branch>`, `git reset --hard`,
///  `git read-tree --reset -u`, `git checkout-index -f` and a restore of the index
///  keep the file the mark names (`checkout-index -f -a` printed the edit unchanged, and a restore
///  or checkout of a path without a tree answered `did not match any file(s) known to git`).
///  A form that writes no tree path at all writes over no marked edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const atRisk = (written) => written.kind === 'matching-tree' && written.revision
///   || written.kind === 'pathspec-file' && written.target.kind === 'tree' && written.target.revision
///   || written.kind === 'move-into';
/// ```
pub fn writes_over_an_edit_a_mark_hides(written: &WrittenPaths) -> bool {
    return match written {
        WrittenPaths::MatchingTree { revision: Some(_), .. } => true,
        WrittenPaths::PathspecFile { target: PathspecTarget::Tree(Some(_)), .. } => true,
        WrittenPaths::MoveInto { .. } => true,
        WrittenPaths::Tree { .. }
        | WrittenPaths::MatchingTree { revision: None, .. }
        | WrittenPaths::Index { .. }
        | WrittenPaths::IndexUnderPrefix { .. }
        | WrittenPaths::PathspecFile { .. }
        | WrittenPaths::ReadTree { .. } => false,
    };
}

/// What:
///  What a forced `git mv` names: its sources and its destination,
///  which the worktree resolves into the paths Git writes.
///  `&[OsString]` borrows the tokens after the subcommand.
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
fn move_written_paths(region: &[OsString]) -> Option<WrittenPaths> {
    // Every word that is no option is a source or the destination: no row of `mv` takes a value, and the table is
    //  what decides which words are options, so a `-k` or a `--sparse` of the person's own is none of them.
    let parsed: ParsedOptions = parse_options(region, MV_TABLE, DEFAULT_MODE, &[]).ok()?;
    let mut words: Vec<&OsString> = Vec::<&OsString>::new();
    for index in positional_tokens(&parsed, region.len()) {
        if let Some(token) = region.get(index) {
            words.push(token);
        }
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
    // The table decides which words are options and which tokens are their values: `--prefix` and `--stage` take
    //  one, so the token after them is never a path the command writes,
    //  and `-a` inside a cluster is the whole-index form (`git checkout-index -fq -a`, `-fa`).
    let parsed: ParsedOptions = parse_options(region, CHECKOUT_INDEX_TABLE, DEFAULT_MODE, &[]).ok()?;
    let all: bool = is_enabled(&parsed, CHECKOUT_INDEX_ALL);
    let stdin: bool = is_enabled(&parsed, CHECKOUT_INDEX_STDIN);
    let prefix: Option<OsString> = last_occurrence(&parsed, CHECKOUT_INDEX_PREFIX)
        .and_then(|occurrence| return occurrence.value)
        .map(|value| return word_of(value_bytes(region, value)));
    let mut pathspecs: Vec<OsString> = Vec::<OsString>::new();
    for index in positional_tokens(&parsed, region.len()) {
        if let Some(token) = region.get(index) {
            pathspecs.push(token.clone());
        }
    }
    // A form with neither `--all` nor a path has nothing to write: Git refuses it
    //  (`fatal: git checkout-index: nothing specified`), so the guard reads no write rather than the whole index.
    //  `--stdin` is the exception: Git reads its paths from the input, which the guard cannot read, so that form is
    //  judged over every path of the index (SPEC.md, the limits of this reading).
    if !all && pathspecs.is_empty() && !stdin {
        return None;
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
fn checkout_written_paths(region: &[OsString]) -> Option<WrittenPaths> {
    let parsed: ParsedOptions = parse_options(region, CHECKOUT_TABLE, DEFAULT_MODE, &[]).ok()?;
    // A form that reads its pathspecs from a file reaches paths no token names, so the file is where the
    //  written paths are read.
    if let Some((file, nul_separated)) = pathspec_file(&parsed, region, PATHSPEC_FROM_FILE, PATHSPEC_FILE_NUL) {
        return Some(pathspec_file_written_paths(&parsed, region, file, nul_separated, is_enabled(&parsed, FORCE)));
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
        guesses_a_remote_branch: guess_is_not_switched_off(&parsed),
    });
}

/// What:
///  Whether a `checkout` or `switch` region leaves Git free to guess the remote branch a short
///  name stands for.
///  `&ParsedOptions` borrows the tokenized region.
/// Why:
///  `--no-guess` and `checkout.guess=false` both turn the guess off,
///  and Git then refuses the name itself
///  (`error: pathspec 'topic' did not match any file(s) known to git`, measured on this host with
///  native Git 2.56.0, 2026-11-13);
///  the guess the guard makes must follow the same switch,
///  or it would resolve a branch Git never touches.
///  Git obeys the last of the two spellings, so `--no-guess --guess` guesses and `--guess --no-guess` does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guesses = (parsed) => parsed.last(GUESS)?.negated !== true;
/// ```
fn guess_is_not_switched_off(parsed: &ParsedOptions) -> bool {
    let mut guess: bool = true;
    if let Some(last) = last_occurrence(parsed, GUESS) {
        guess = !last.negated;
    }
    return guess;
}

/// What:
///  The paths a `git restore` form writes over,
///  read from its own option table.
/// Why:
///  Without a source the index's content goes back over the worktree,
///  and with one the source tree's content does;
///  a form that restores only the index writes no worktree file at all.
///  The paths are the tokens after `--` when it is written,
///  and otherwise every token that is not an option or the value of one,
///  which the table decides:
///  `git restore -U 3 -- a.txt` names a number and a path,
///  and `git restore -s -W --staged -- a.txt` names a revision called `-W`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreWrittenPaths(region: string[]): WrittenPaths | undefined;
/// ```
fn restore_written_paths(region: &[OsString]) -> Option<WrittenPaths> {
    let parsed: ParsedOptions = parse_options(region, RESTORE_TABLE, DEFAULT_MODE, &[]).ok()?;
    if is_enabled(&parsed, RESTORE_STAGED) && !is_enabled(&parsed, RESTORE_WORKTREE) {
        return None;
    }
    // `--source` takes a value in Git's own table, so the token after it is that value whatever it looks like, and an
    //  option written twice keeps the last one:
    //  `git restore -s main -s other -- f.txt` writes `other`'s content
    //  (measured on this host with native Git 2.56.0, 2026-11-13).
    let source: Option<OsString> = last_occurrence(&parsed, RESTORE_SOURCE)
        .and_then(|occurrence| return occurrence.value)
        .map(|value| return word_of(value_bytes(region, value)));
    if let Some((file, nul_separated)) = pathspec_file(&parsed, region, RESTORE_PATHSPEC_FROM_FILE, RESTORE_PATHSPEC_FILE_NUL) {
        return Some(WrittenPaths::PathspecFile {
            target: match source {
                Some(revision) => PathspecTarget::Tree(Some(revision)),
                None => PathspecTarget::Index,
            },
            file,
            nul_separated,
            // Git refuses a restore with no path at all: `fatal: you must specify path(s) to restore`
            //  (measured on this host with native Git 2.56.0, 2026-11-13), so only the refusal follows an empty file.
            entries: WrittenEntries::EveryEntry,
            guesses_a_remote_branch: false,
        });
    }
    let mut pathspecs: Vec<OsString> = Vec::<OsString>::new();
    for index in positional_tokens(&parsed, region.len()) {
        if let Some(token) = region.get(index) {
            pathspecs.push(token.clone());
        }
    }
    return Some(match source {
        Some(revision) => WrittenPaths::MatchingTree { revision: Some(revision), pathspecs },
        None => WrittenPaths::Index { pathspecs },
    });
}

/// What:
///  The file a region reads its pathspecs from,
///  and whether it separates them with NUL bytes,
///  read from the command's own option table.
///  `u16` names those two options' rows in that table.
/// Why:
///  `--pathspec-from-file <file>`, `--pathspec-from-file=<file>` and the abbreviations Git accepts
///  are one option,
///  and `--pathspec-file-nul` decides how its entries are separated.
///  A file named here reaches paths no other word of the command names,
///  so a reading that ignores it asks about nothing.
///  The table is what makes a value that looks like an option the file's name:
///  `git restore --source=other --pathspec-from-file --staged` reads the file named `--staged`
///  (the seventh review, reproduced here),
///  and a value that looks like a separator does not turn the separator on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathspecFile = (parsed, region, fromFile, fileNul) => ({ file: valueOf(parsed, fromFile), nul: enabled(parsed, fileNul) });
/// ```
fn pathspec_file(
    parsed: &ParsedOptions,
    region: &[OsString],
    from_file: u16,
    file_nul: u16,
) -> Option<(OsString, bool)> {
    let value = last_occurrence(parsed, from_file)?.value?;
    return Some((word_of(value_bytes(region, value)), is_enabled(parsed, file_nul)));
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
    parsed: &ParsedOptions,
    region: &[OsString],
    file: OsString,
    nul_separated: bool,
    forced: bool,
) -> WrittenPaths {
    // The tree-ish is the last word that is neither an option nor the file's name.
    //  The file's name is never among the positional words: `--pathspec-from-file` takes a value in Git's own table
    //  (`CHECKOUT_TABLE`), so a separated value is consumed as that option's value and an attached one is part of the
    //  option's own token.
    //  A filter that dropped every word equal to the file's name also dropped a revision spelled that way, and
    //  `git checkout -f topic --pathspec-from-file=topic` was then read as aimed at the index, whose paths do not hold
    //  `topic`'s new file
    //  (the untracked `b.txt` was overwritten, measured on this host with native Git 2.56.0, 2026-11-14).
    let mut revision: Option<OsString> = None;
    for index in positional_tokens(parsed, region.len()) {
        if let Some(token) = region.get(index) {
            revision = Some(token.clone());
        }
    }
    return WrittenPaths::PathspecFile {
        target: match revision {
            Some(named) => PathspecTarget::Tree(Some(named)),
            None => PathspecTarget::Index,
        },
        file,
        nul_separated,
        entries: if forced { WrittenEntries::EveryEntry } else { WrittenEntries::OnlyIgnored },
        guesses_a_remote_branch: guess_is_not_switched_off(parsed),
    };
}

/// Reading the forms' words into the paths they write,
/// beside real-Git controls of each spelling.
#[cfg(test)]
#[path = "command_written_paths_tests.rs"]
mod tests;

/// Every option a written-path reading consults, checked against real Git when it is given twice.
#[cfg(test)]
#[path = "command_repeated_options_tests.rs"]
mod repeated_options_tests;
