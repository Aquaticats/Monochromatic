//! What:
//!  The worktree entries that stand where one guarded form writes,
//!  whatever Git's status calls them.
//! Why:
//!  A forced tree write replaces an untracked file,
//!  turns it into a directory,
//!  and deletes the work inside a nested repository at that path,
//!  while the status the guard reads names only the worktree's own tracked changes.
//!  An ignored file is in neither reading,
//!  and an ignored file is where a person keeps their environment and their local secrets.
//!  The question is Git's own: the paths the form writes,
//!  then every entry of the worktree at, below or above one of them that Git did not put there.
//!  Git is asked once over the whole worktree rather than once per written path,
//!  because its own pathspec matching over the thousands of paths a whole-tree write names is
//!  what a guard on every `reset --hard` cannot pay;
//!  the answers are intersected with the written paths in this process instead.
//!  `--directory` collapses a directory the worktree holds whole into one entry (`nested/`),
//!  which bounds the listing to the worktree's own directories,
//!  and a nested repository is reported that way:
//!  naming that entry is what stops the command that would take it whole.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // pathsInTheWay(facts, { kind: 'tree', revision: 'other' }) === ['b.txt', 'nested']
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The written-path reading the question starts from,
///  and which entries at those paths Git writes over.
use crate::command_written_paths::{WrittenEntries, WrittenPaths};
/// The paths the form writes,
///  each with each directory on the way to it,
///  and one byte-exact path built from Git's bytes.
use crate::repository_facts_written_paths::{path_from_bytes, paths_the_tree_writes};
/// The trait that answers where the command runs.
use crate::repository_facts::RepositoryFacts;
/// The captured-query runner's output type.
use crate::git_metadata::MetadataOutput;
/// The queries over one set of paths,
///  and the reading of each answer.
use crate::rule_worktree_dirty::worktree_entries_from_answer;
/// The worktree an entry stands in,
///  read from the repository's identity.
use crate::worktree_identity::worktree_root;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Every path at or below a written path that the worktree holds and Git did not put there,
///  each as Git printed it,
///  with the entries below another entry dropped.
///  `Result<Vec<String>, String>` is "the paths,
///  or why Git could not give one".
/// Why:
///   Only the outermost entry is named:
///  a directory holds everything below it,
///  and naming a file inside it as well would describe a loss the message already covers.
///  A failure is a failure:
///  a reading that could not be made says nothing about what the command overwrites,
///  and the caller refuses rather than spending a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsInTheWay(facts, written): Promise<string[]>; // throws
/// ```
pub(super) fn paths_in_the_way(facts: &mut GitFacts, written: &WrittenPaths) -> Result<Vec<String>, String> {
    let mut asked: Vec<Vec<u8>> = paths_the_tree_writes(facts, written)?;
    if asked.is_empty() {
        return Ok(Vec::<String>::new());
    }
    // Sorted once, every worktree entry is placed among the written paths by two searches instead of by a
    //  comparison against each of them: a worktree holding thousands of untracked entries met twenty thousand
    //  written paths one at a time before this.
    asked.sort();
    asked.dedup();
    let mut found: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    // Two answers, one question: what is untracked and what is ignored, each asked once over the whole worktree
    //  because Git matches thousands of literal pathspecs far more slowly than it lists the entries itself.
    //  Every directory the worktree holds whole arrives as one entry with a trailing separator, so an entry here
    //  is either a path the form writes, one below it, or a directory that holds it.
    //  A form that writes over only the entries its ignore rules cover asks the ignored listing alone.
    for ignored in listings(written) {
        let output: MetadataOutput = facts.query(worktree_entry_query(facts.global_prefix.as_slice(), *ignored), "the files this command writes over")?;
        for entry in worktree_entries_from_answer(&output)? {
            if !stands_where_the_form_writes(asked.as_slice(), entry.as_slice()) {
                continue;
            }
            found.push(entry);
        }
    }
    let kept: Vec<Vec<u8>> = outermost_entries(found);
    if kept.is_empty() {
        return Ok(Vec::<String>::new());
    }
    // A repository with no worktree of its own has no entry to name, and the caller asked a worktree question.
    let location: crate::repository_location::RepositoryLocation = facts.location()?;
    let Some(root) = worktree_root(&location.identity) else {
        return Ok(Vec::<String>::new());
    };
    return Ok(entries_the_form_would_lose(kept, asked.as_slice(), root));
}

/// What:
///  The entries a form would lose,
///  each named as the file whose bytes go:
///  a file the form writes over, or a directory it replaces whole, is named as Git printed it,
///  and a directory the listing collapsed is named by the first entry inside it that the form
///  would replace,
///  which is a file where a directory must go,
///  a directory where a file must go,
///  or the written file itself.
///  `Vec<Vec<u8>>` is the entries and the paths,
///  `&Path` the worktree they stand in.
/// Why:
///  `--directory` collapses an untracked or ignored directory into one entry,
///  and the answer's reader drops the separator Git printed on it,
///  so the entry is asked of the worktree whether it is a directory.
///  Naming the directory would claim the command deletes it,
///  which it does not when the form only writes a file inside it,
///  and would hide which of the person's entries goes.
///  A directory holding nothing the form replaces loses nothing and is not named,
///  which is what keeps a tree adding files beside a person's own directory from being refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lost = (kept, asked, root) => kept.flatMap((entry) => isDirectory(join(root, entry))
///   ? asked.some((path) => path === entry) ? [entry] : asked.filter((path) => under(entry, path)).flatMap((path) => firstReplaced(root, entry, path))
///   : [entry]);
/// ```
fn entries_the_form_would_lose(kept: Vec<Vec<u8>>, asked: &[Vec<u8>], root: &Path) -> Vec<String> {
    let mut named: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for entry in kept {
        if !is_a_directory(root, entry.as_slice()) {
            named.push(entry);
            continue;
        }
        // A file the form writes at the directory's own path replaces the directory whole.
        if asked.iter().any(|path| return path.as_slice() == entry.as_slice()) {
            named.push(entry);
            continue;
        }
        for path in asked {
            if !holds(entry.as_slice(), path.as_slice()) {
                continue;
            }
            if let Some(replaced) = the_entry_the_form_replaces(root, entry.as_slice(), path.as_slice()) {
                named.push(replaced);
            }
        }
    }
    // The outermost of two named paths describes the same bytes once: a file inside a directory that
    //  is itself named is that directory's content.
    named.sort();
    named.dedup();
    let mut kept: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in named {
        if kept.iter().any(|outer| return holds(outer.as_slice(), path.as_slice())) {
            continue;
        }
        kept.push(path);
    }
    return kept.into_iter().map(|path| return String::from_utf8_lossy(path.as_slice()).into_owned()).collect();
}

/// What:
///  The first entry the form would replace on the way from `directory` down to `written`,
///  or nothing when the worktree holds nothing there.
///  `&[u8]` borrows the directory the listing collapsed, the written path and the entry found.
/// Why:
///  A tree creates the directories it needs:
///  a directory on the way is no loss,
///  while a file there is deleted to make room and the file's bytes go.
///  The written path itself is replaced whether it is a file or a directory,
///  and a component the worktree does not hold is one Git creates, so nothing is lost from it on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const replaced = (root, directory, written) => {
///   let current = directory;
///   for (const component of components) {
///     current = join(current, component);
///     const found = lstat(join(root, current));
///     if (!found) return undefined;
///     if (!found.isDirectory() || current === written) return current;
///   }
///   return undefined;
/// };
/// ```
fn the_entry_the_form_replaces(root: &Path, directory: &[u8], written: &[u8]) -> Option<Vec<u8>> {
    let inside: &[u8] = written.strip_prefix(directory)?.strip_prefix(b"/")?;
    let mut current: Vec<u8> = directory.to_vec();
    let components: Vec<&[u8]> = inside.split(|byte| return *byte == b'/').collect();
    for (index, component) in components.iter().enumerate() {
        current.push(b'/');
        current.extend(component.iter());
        let full: PathBuf = root.join(path_from_bytes(current.as_slice()).as_os_str());
        let Ok(metadata) = std::fs::symlink_metadata(full.as_path()) else {
            // Git creates this entry and everything below it, so no bytes of the person's go.
            return None;
        };
        let last: bool = index + 1 == components.len();
        if !metadata.file_type().is_dir() || last {
            return Some(current);
        }
    }
    return None;
}

/// What:
///  Whether the worktree holds a directory at one named path,
///  without following a symbolic link.
/// Why:
///   A link is an entry of its own whose bytes go with the link,
///  while the directory it names is not touched by a form that writes at the link's path;
///  and `ls-files` prints a collapsed directory without the separator that named it one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isADirectory = (root, path) => lstatSync(join(root, path)).isDirectory();
/// ```
fn is_a_directory(root: &Path, path: &[u8]) -> bool {
    let full: PathBuf = root.join(path_from_bytes(path).as_os_str());
    return std::fs::symlink_metadata(full.as_path()).is_ok_and(|metadata| return metadata.file_type().is_dir());
}

/// What:
///  Which of the two listings the form's written paths need,
///  the untracked one first and the ignored one after it.
///  `bool` is true for the listing of the entries Git's ignore rules cover.
/// Why:
///   A form that writes over every entry needs both;
///  a form that writes its tree over the ignored entries alone needs only the second,
///  and asking the first would refuse a command over a file Git keeps and name a loss that never happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listings(written: WrittenPaths): boolean[];
/// ```
fn listings(written: &WrittenPaths) -> &'static [bool] {
    return match written {
        WrittenPaths::Tree { entries: WrittenEntries::OnlyIgnored, .. }
        | WrittenPaths::ReadTree { entries: WrittenEntries::OnlyIgnored, .. } => &[true],
        _ => &[false, true],
    };
}

/// What:
///  Whether one worktree entry stands where the form writes:
///  at a written path,
///  below one,
///  or above one,
///  which is the case of a directory the tree needs a file inside of.
///  `&[Vec<u8>]` borrows the written paths, sorted and without duplicates.
/// Why:
///  A directory the tree needs a file inside of is found by the range of names that start with it and a
///  separator,
///  a path written below a directory by asking whether each of its ancestors is written,
///  and a name that merely starts the same letters names nothing either way.
///  Both searches walk a sorted list instead of every written path,
///  which is what keeps a worktree holding thousands of untracked entries cheap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stands = (asked, entry) => asked.includes(entry) || asked.some((p) => p.startsWith(entry + '/'))
///   || ancestors(entry).some((a) => asked.includes(a));
/// ```
fn stands_where_the_form_writes(asked: &[Vec<u8>], entry: &[u8]) -> bool {
    if asked.binary_search_by(|path| return path.as_slice().cmp(entry)).is_ok() {
        return true;
    }
    // The entry is a directory above a written path: some written path starts with it and a separator.
    let mut inside: Vec<u8> = entry.to_vec();
    inside.push(b'/');
    let from: usize = asked.partition_point(|path| return path.as_slice() < inside.as_slice());
    if asked.get(from).is_some_and(|path| return path.starts_with(inside.as_slice())) {
        return true;
    }
    // The entry is below a written path: one of its ancestors is written.
    let mut at: usize = 0;
    while let Some(found) = entry[at..].iter().position(|byte| return *byte == b'/') {
        let slash: usize = at + found;
        if asked.binary_search_by(|path| return path.as_slice().cmp(&entry[..slash])).is_ok() {
            return true;
        }
        at = slash + 1;
    }
    return false;
}

/// What:
///  The one listing over the whole worktree that names what Git did not put there,
///  either the entries its ignore rules do not cover or the ones they do,
///  with each directory the worktree holds whole collapsed into one entry.
///  `bool` asks for the ignored ones.
/// Why:
///   Asking over the written paths alone costs one large argument list per bounded start,
///  and Git's own pathspec matching over thousands of literal paths is what made a
///  `reset --hard` slow:
///  measured on a worktree of this repository's size, ten such starts took 1.9 s where these two take 0.12 s.
///  The answers are intersected with the written paths in this process,
///  which is one pass over each of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeEntryQuery(globalPrefix: string[], ignored: boolean): string[];
/// ```
fn worktree_entry_query(global_prefix: &[OsString], ignored: bool) -> Vec<OsString> {
    let selection: &[&str] = if ignored {
        &["--others", "--ignored", "--exclude-standard", "--directory"]
    } else {
        &["--others", "--exclude-standard", "--directory"]
    };
    return crate::rule_worktree_dirty::listing_arguments(global_prefix, selection);
}

/// What:
///  The entries that no other entry holds,
///  each as the bytes Git printed and in one stable order.
/// Why:
///   Naming both a directory and a file inside it describes a loss twice,
///  and the directory's own name covers it;
///  the shortest path first makes the answer the same on every run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outermostEntries(found: Buffer[]): Buffer[];
/// ```
fn outermost_entries(found: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut sorted: Vec<Vec<u8>> = found;
    sorted.sort();
    sorted.dedup();
    let mut kept: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in sorted {
        if kept.iter().any(|outer| return holds(outer.as_slice(), path.as_slice())) {
            continue;
        }
        kept.push(path);
    }
    return kept;
}

/// What:
///  Whether one entry is the other or holds it,
///  at a component boundary,
///  where a trailing separator names the directory it ends.
/// Why:
///  `a/b` and `a` are one entry's worth of saving,
///  while `ab` is a name of its own beside `a`;
///  and Git prints a directory the worktree holds whole as `a/`,
///  which holds `a/b` exactly as `a` would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holds = (outer, inner) =>
///   outer === inner || inner.startsWith(outer.replace(/\/$/, '') + '/');
/// ```
fn holds(outer: &[u8], inner: &[u8]) -> bool {
    if outer == inner {
        return true;
    }
    let holding: &[u8] = outer.strip_suffix(b"/").unwrap_or(outer);
    return !holding.is_empty() && inner.starts_with(holding) && inner.get(holding.len()) == Some(&b'/');
}

/// The paths the worktree holds where the form writes, and the answer reading.
#[cfg(test)]
#[path = "repository_facts_paths_in_the_way_tests.rs"]
mod tests;
