//! What:
//!  The paths the guarded forms other than a reset,
//!  a checkout and a switch write,
//!  read from their own words.
//! Why:
//!  A `restore` writes its source tree's paths or the index's content,
//!  a `read-tree -u` writes the tree it is given last,
//!  a `mv --force` writes its destination,
//!  and a `checkout-index` writes the index's paths under its prefix;
//!  each one can replace a file Git did not put there,
//!  so each one's words are read before the refusal is decided.
//!  The readings live apart from the three common forms
//!  so each file stays within its line bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writtenPathsOfOtherForms('RestoreWorktree', ['--source=other', '--', 'b.txt'], root)
//! //   === { kind: 'tree', revision: 'other', pathspecs: ['b.txt'] }
//! ```

/// The answer type,
///  the word reader and the first positional word of a region.
use super::policy_linked_worktree_exempt_written_paths::{Written, read_words};
/// Git's own option grammar for a read-tree.
use crate::command_discard::READ_TREE_TABLE;
/// Whether a restore form writes the worktree at all.
use crate::command_discard::restores_index_only;
/// Git's option parser and where its options end.
use crate::command_options::{DEFAULT_MODE, ParsedOptions, parse_options};
/// The positional tokens of a parsed region.
use crate::command_options_query::positional_tokens;
/// The forms the linked-worktree policy guards.
use crate::rule_linked_worktree::GuardedCommand;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The paths one of the remaining guarded forms writes,
///  or nothing for a form that writes no untracked entry.
/// Why:
///   Every form here reaches a worktree path only through the words read below,
///  and a form that is none of them writes none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writtenPathsOfOtherForms(command, region, root): Written | undefined; // throws
/// ```
pub(super) fn written_paths_of_other_forms(
    command: GuardedCommand,
    region: &[OsString],
    worktree_root: &Path,
) -> Result<Option<Written>, String> {
    return match command {
        GuardedCommand::RestoreWorktree => restore_paths(region),
        GuardedCommand::ReadTree => read_tree_paths(region),
        GuardedCommand::Move => move_paths(region, worktree_root),
        GuardedCommand::CheckoutIndex => checkout_index_paths(region),
        _ => Ok(None),
    };
}

/// What:
///  The paths a `git restore` form writes:
///  the source tree's paths it names,
///  or the index's paths when it names no source,
///  and nothing when it restores the index alone.
/// Why:
///  A restore without `--worktree` writes no worktree file,
///  a source names the tree its content comes from,
///  and the paths after `--` are the ones it reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restorePaths(region: string[]): Written | undefined;
/// ```
fn restore_paths(region: &[OsString]) -> Result<Option<Written>, String> {
    if restores_index_only(region) {
        return Ok(None);
    }
    let pathspecs: Vec<OsString> = restore_pathspecs(region);
    return match restore_source(region) {
        Some(revision) => Ok(Some(Written::Tree {
            revision: Some(revision),
            pathspecs,
            prefix: Vec::<u8>::new(),
        })),
        None => Ok(Some(Written::Index {
            pathspecs,
            prefix: Vec::<u8>::new(),
        })),
    };
}

/// What:
///  The revision a `git restore` form names as its source,
///  in every spelling Git's own grammar accepts.
/// Why:
///   `--source <tree>`,
///  `--source=<tree>`,
///  `-s <tree>` and `-s<tree>` are one option,
///  and a value that looks like a flag is that value (`restore --source -W -- f`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreSource(region: string[]): string | undefined;
/// ```
fn restore_source(region: &[OsString]) -> Option<OsString> {
    let mut at: usize = 0;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        at += 1;
        if written == b"--" {
            return None;
        }
        if let Some(value) = long_value(written, "source", 4) {
            return Some(value);
        }
        if spells_long(written, "source", 4) {
            return region.get(at).cloned();
        }
        if written.starts_with(b"--") {
            if !written.contains(&b'=') && restore_long_takes_a_value(written) {
                at += 1;
            }
            continue;
        }
        if written.len() > 1 && written[0] == b'-' {
            let cluster: &[u8] = &written[1..];
            let position: Option<usize> = position_of(cluster, b's');
            let Some(found) = position else {
                continue;
            };
            let value: &[u8] = &cluster[found + 1..];
            if !value.is_empty() {
                return Some(OsString::from(String::from_utf8_lossy(value).into_owned()));
            }
            return region.get(at).cloned();
        }
    }
    return None;
}

/// What:
///  The paths a `git restore` form names,
///  which are the tokens after `--` when it is written and the plain words otherwise.
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
            if !written.contains(&b'=') && restore_long_takes_a_value(written) {
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
///  Whether one long option of `git restore` takes the next token as its value.
/// Why:
///   `--source` and `--conflict` do,
///  and every other long option of this command does not,
///  so the token after one of them is a value and never a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const restoreLongTakesAValue = (token) => spellsLong(token, 'source') || spellsLong(token, 'conflict');
/// ```
fn restore_long_takes_a_value(written: &[u8]) -> bool {
    return spells_long(written, "source", 4) || spells_long(written, "conflict", 3);
}

/// What:
///  The tree a `read-tree` form writes into the worktree,
///  under its `--prefix` when it names one.
/// Why:
///   `git read-tree --reset -u <tree>` writes the tree's paths,
///  `-m -u <base> <tree>` writes the tree it is given last,
///  `--prefix` puts every path below that directory,
///  and `--empty` names no tree at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTreePaths(region: string[]): Written | undefined; // throws
/// ```
fn read_tree_paths(region: &[OsString]) -> Result<Option<Written>, String> {
    // `-u` is what writes worktree files;
    // a read-tree without it replaces only the index,
    // which an exempt repository's form may do over its build step's own changes.
    if !read_tree_writes_the_worktree(region) {
        return Ok(None);
    }
    let parsed: ParsedOptions = read_words(parse_options(region, READ_TREE_TABLE, DEFAULT_MODE, &[]), "read-tree")?;
    let positions: Vec<usize> = positional_tokens(&parsed, region.len());
    let Some(last) = positions.last() else {
        return Ok(None);
    };
    return Ok(Some(Written::Tree {
        revision: Some(region[*last].clone()),
        pathspecs: Vec::<OsString>::new(),
        prefix: read_tree_prefix(region)?,
    }));
}

/// What:
///  Whether the region carries `-u`,
///  the one flag that makes a read-tree write the worktree.
/// Why:
///   `read-tree` without it replaces only the index,
///  so no untracked file stands in its way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readTreeWritesTheWorktree = (region) => region.some((one) => one.startsWith('-u') || one.includes('u'));
/// ```
fn read_tree_writes_the_worktree(region: &[OsString]) -> bool {
    for token in region {
        let written: &[u8] = token.as_encoded_bytes();
        if written.starts_with(b"--") || !written.starts_with(b"-") {
            continue;
        }
        if written[1..].contains(&b'u') {
            return true;
        }
    }
    return false;
}

/// What:
///  The directory a `read-tree` writes under,
///  or nothing.
/// Why:
///   `--prefix` takes its value in its own token or the next one,
///  and Git accepts every unambiguous abbreviation of the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTreePrefix(region: string[]): string; // throws
/// ```
fn read_tree_prefix(region: &[OsString]) -> Result<Vec<u8>, String> {
    let mut at: usize = 0;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        at += 1;
        if let Some(value) = long_value(written, "prefix", 3) {
            return Ok(value.as_encoded_bytes().to_vec());
        }
        if spells_long(written, "prefix", 3) {
            let Some(value) = region.get(at) else {
                return Err(String::from("the read-tree's --prefix names no directory"));
            };
            return Ok(value.as_encoded_bytes().to_vec());
        }
    }
    return Ok(Vec::<u8>::new());
}

/// What:
///  The destination a `git mv` writes,
///  one path per source when the destination is a directory.
/// Why:
///  `mv --force` replaces the destination,
///  and a destination that is a directory receives each source under its own name,
///  so the paths to look at are the ones Git would write:
///  the destination itself,
///  or the destination joined with each source's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function movePaths(region: string[], root: string): Written | undefined;
/// ```
fn move_paths(region: &[OsString], worktree_root: &Path) -> Result<Option<Written>, String> {
    let positions: Vec<usize> = plain_positionals(region);
    let Some(last) = positions.last() else {
        return Ok(None);
    };
    if positions.len() < 2 {
        return Ok(None);
    }
    let destination: &OsString = &region[*last];
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    if worktree_root.join(destination).is_dir() {
        for source in &positions[..positions.len() - 1] {
            let name: &[u8] = last_component(region[*source].as_encoded_bytes());
            let mut path: Vec<u8> = destination.as_encoded_bytes().to_vec();
            if !path.ends_with(b"/") {
                path.push(b'/');
            }
            path.extend_from_slice(name);
            paths.push(path);
        }
    } else {
        paths.push(destination.as_encoded_bytes().to_vec());
    }
    return Ok(Some(Written::Paths { paths }));
}

/// What:
///  The part of one path after its last separator.
/// Why:
///   A move into a directory writes each source under its own name,
///  which is that part.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lastComponent = (path) => path.slice(path.lastIndexOf('/') + 1);
/// ```
fn last_component(path: &[u8]) -> &[u8] {
    let mut start: usize = 0;
    for (index, byte) in path.iter().enumerate() {
        if *byte == b'/' {
            start = index + 1;
        }
    }
    return &path[start..];
}

/// What:
///  The paths a `checkout-index` form writes:
///  every path of the index,
///  or the paths it names,
///  each under its `--prefix`.
/// Why:
///  `--all` and `--stdin` write the whole index,
///  the paths after `--` are the ones it reaches,
///  and `--prefix` puts every one of them below that directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutIndexPaths(region: string[]): Written | undefined;
/// ```
fn checkout_index_paths(region: &[OsString]) -> Result<Option<Written>, String> {
    let words: CheckoutIndexWords = checkout_index_words(region)?;
    if words.pathspecs.is_empty() && !words.every {
        // No paths and no `--all`/`--stdin`: Git has nothing to check out and refuses the form itself.
        return Ok(None);
    }
    let pathspecs: Vec<OsString> = if words.every {
        Vec::<OsString>::new()
    } else {
        words.pathspecs
    };
    return Ok(Some(Written::Index {
        pathspecs,
        prefix: words.prefix,
    }));
}

/// What:
///  The prefix,
///  the `--all`/`--stdin` answer and the paths of one `checkout-index` region.
///  A `struct` is a record with named fields.
/// Why:
///  One pass over the words answers all three,
///  and the answer decides whether the form writes anything at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutIndexWords = { prefix: string; every: boolean; pathspecs: string[] };
/// ```
struct CheckoutIndexWords {
    /// The directory every written path goes under.
    prefix: Vec<u8>,
    /// Whether `--all` or `--stdin` writes the whole index.
    every: bool,
    /// The paths the form names.
    pathspecs: Vec<OsString>,
}

/// What:
///  The words of one `checkout-index` region.
/// Why:
///   `--prefix` takes its value in its own token or the next one,
///  `-a` and `--all` write the whole index,
///  and everything after `--` is a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutIndexWords(region: string[]): CheckoutIndexWords; // throws
/// ```
fn checkout_index_words(region: &[OsString]) -> Result<CheckoutIndexWords, String> {
    let mut words: CheckoutIndexWords = CheckoutIndexWords {
        prefix: Vec::<u8>::new(),
        every: false,
        pathspecs: Vec::<OsString>::new(),
    };
    let mut at: usize = 0;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        at += 1;
        if written == b"--" {
            words.pathspecs.extend(region[at..].iter().cloned());
            at = region.len();
            continue;
        }
        if let Some(value) = long_value(written, "prefix", 3) {
            words.prefix = value.as_encoded_bytes().to_vec();
            continue;
        }
        if spells_long(written, "prefix", 3) {
            let Some(value) = region.get(at) else {
                return Err(String::from("the checkout-index's --prefix names no directory"));
            };
            words.prefix = value.as_encoded_bytes().to_vec();
            at += 1;
            continue;
        }
        if spells_long(written, "all", 1) || spells_long(written, "stdin", 1) {
            words.every = true;
            continue;
        }
        if written.starts_with(b"--") {
            continue;
        }
        if written.len() > 1 && written[0] == b'-' {
            if written[1..].contains(&b'a') {
                words.every = true;
            }
            continue;
        }
        words.pathspecs.push(region[at - 1].clone());
    }
    return Ok(words);
}

/// What:
///  The positional words of a region whose options all take no value,
///  which is every option of `git mv`.
/// Why:
///  A destination is the last positional,
///  and `--` ends the options so a name that begins with `-` stays a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plainPositionals(region: string[]): number[];
/// ```
fn plain_positionals(region: &[OsString]) -> Vec<usize> {
    let mut positions: Vec<usize> = Vec::<usize>::new();
    let mut at: usize = 0;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        if written == b"--" {
            let mut rest: usize = at + 1;
            while rest < region.len() {
                positions.push(rest);
                rest += 1;
            }
            return positions;
        }
        if written.len() > 1 && written[0] == b'-' {
            at += 1;
            continue;
        }
        positions.push(at);
        at += 1;
    }
    return positions;
}

/// What:
///  The first position of one byte in a token,
///  or nothing.
/// Why:
///   A short cluster's `s` is where a restore's source value begins,
///  and a named function keeps that scan free of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const positionOf = (bytes, wanted) => bytes.indexOf(wanted);
/// ```
fn position_of(bytes: &[u8], wanted: u8) -> Option<usize> {
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == wanted {
            return Some(index);
        }
    }
    return None;
}

/// What:
///  The value a long option carries inside its own token (`--name=value`),
///  by any abbreviation Git accepts for it.
/// Why:
///   Git accepts an unambiguous prefix of every long option,
///  and `--sour=x` is `--source=x`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function longValue(token: Buffer, name: string, minimum: number): string | undefined;
/// ```
fn long_value(written: &[u8], name: &str, minimum: usize) -> Option<OsString> {
    let Some(equals) = position_of(written, b'=') else {
        return None;
    };
    if !spells_long(&written[..equals], name, minimum) {
        return None;
    }
    return Some(OsString::from(String::from_utf8_lossy(&written[equals + 1..]).into_owned()));
}

/// What:
///  Whether one token spells the long option `full`,
///  by its whole name or any unambiguous prefix Git accepts for it,
///  and never its `--no-` form.
/// Why:
///   `--source`,
///  `--sour` and `--so` are one option,
///  while `--no-source` is a different spelling Git refuses,
///  and a token carrying `=` is read by `long_value` instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spellsLong = (token, full, minimum) =>
///   token.startsWith('--') && !token.includes('=') && token.length >= minimum + 2 && full.startsWith(token.slice(2));
/// ```
pub(super) fn spells_long(written: &[u8], full: &str, minimum: usize) -> bool {
    let Some(name) = written.strip_prefix(b"--") else {
        return false;
    };
    if name.contains(&b'=') {
        return false;
    }
    if name.len() < minimum || name.len() > full.len() {
        return false;
    }
    return full.as_bytes().starts_with(name);
}
