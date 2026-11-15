//! What:
//!  The paths one guarded form writes into the worktree,
//!  asked of the tree or the index the form names.
//! Why:
//!  Refusing a form that would write over a file nothing else holds needs the written paths first,
//!  and only Git knows which paths a pathspec reaches in a tree:
//!  `git checkout other -- dir` writes every path of `other` below `dir`,
//!  which a glob or a directory name would have to be re-read to guess.
//!  Every directory on the way to a written path is asked about as well,
//!  because a tree creates the directories it needs,
//!  and a worktree that holds a file where one of them must go loses that file.
//!  A form that reads its pathspecs from a file names paths no token does, so the file is read
//!  here, the way Git reads it.
//!  A revision Git cannot resolve writes no path,
//!  which is the reading `git checkout -f <a path>` needs:
//!  Git writes that path from the index and no tree at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // pathsTheTreeWrites(facts, { kind: 'matching-tree', revision: 'other', pathspecs: ['b.txt'] }) === ['b.txt']
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The written-path reading this fact answers, and the pathspec target it may name.
use crate::command_written_paths::{PathspecTarget, WrittenPaths};
/// The interface the location is asked through,
///  whose `prefix` names the paths a form that names paths writes below.
use crate::repository_facts::RepositoryFacts;
/// The captured-query runner's output type,
///  and the NUL-separated reading of an answer.
use crate::git_metadata::MetadataOutput;
use crate::rule_worktree_dirty::nul_separated_paths;
/// The ceiling one metadata lookup of the worktree is read under,
///  and the bound on the bytes of a pathspec file.
use crate::wait_ceilings::{PATHSPEC_FILE_READ_LIMIT, WORKTREE_LOOKUP_CEILING};
/// The private directory the wrapper keeps short-lived files in,
///  which the copy it hands Git is written to.
use crate::scratch_location::prepare_repository_scratch;
/// The common directory of a location.
use crate::worktree_identity::common_directory;
/// The worktree a path is named from.
use crate::worktree_identity::worktree_root;
/// The caller's global options without the `-C` chain a query started elsewhere already reached.
use crate::global_arguments::without_directory_changes;
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Every path at which the form writes a file,
///  `Result<Vec<Vec<u8>>, String>` is "the paths,
///  or why Git could not give one".
///  `&Path` borrows the directory the command runs in, which every relative word of it is
///  read from.
/// Why:
///   A query that failed says nothing,
///  and a safety check must never read a failure as "nothing is written".
///  The directory travels with the reading because a command's relative paths mean one place
///  and one prefix:
///  the names a path form matches are read from the directory Git runs it in,
///  which is the top of the worktree for the command a `!` alias's shell runs,
///  and where the wrapper stands otherwise.
///  The directories on the way to a path are not listed:
///  Git creates the ones it needs,
///  so a directory there loses nothing,
///  while a file at one of those paths would lose its bytes and is found as the entry above the
///  written path
///  (`repository_facts_paths_in_the_way.rs` says how).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsTheTreeWrites(facts, written, runIn): Promise<Buffer[]>; // throws
/// ```
pub(super) fn paths_the_tree_writes(facts: &mut GitFacts, written: &WrittenPaths, run_in: &Path) -> Result<Vec<WrittenPlace>, String> {
    return match written {
        WrittenPaths::Tree { revision, guesses_a_remote_branch, .. } => {
            Ok(inside_places(tree_paths(facts, revision.as_deref(), &[], *guesses_a_remote_branch, run_in)?))
        }
        WrittenPaths::MatchingTree { revision, pathspecs } => {
            // A form that names paths writes the tree of the word before them, and Git resolves that word as a
            //  revision alone: `checkout -f topic -- b.txt` answers `invalid reference: topic` for a remote-only name.
            Ok(inside_places(tree_paths(facts, revision.as_deref(), pathspecs.as_slice(), false, run_in)?))
        }
        WrittenPaths::Index { pathspecs } => Ok(inside_places(index_paths(facts, pathspecs.as_slice(), run_in)?)),
        WrittenPaths::IndexUnderPrefix { pathspecs, prefix } => index_under_prefix_paths(facts, pathspecs, prefix, run_in),
        WrittenPaths::MoveInto { sources, destination } => move_paths(facts, sources.as_slice(), destination, run_in),
        WrittenPaths::ReadTree { revisions, prefix, .. } => read_tree_paths(facts, revisions.as_slice(), prefix.as_deref(), run_in),
        WrittenPaths::PathspecFile { target, file, nul_separated, entries: _, guesses_a_remote_branch } => {
            let pathspecs: Vec<OsString> = pathspecs_in_the_file(facts, file, *nul_separated, run_in)?;
            if !pathspecs.is_empty() {
                return match target {
                    PathspecTarget::Tree(revision) => {
                        Ok(inside_places(tree_paths(facts, revision.as_deref(), pathspecs.as_slice(), false, run_in)?))
                    }
                    PathspecTarget::Index => Ok(inside_places(index_paths(facts, pathspecs.as_slice(), run_in)?)),
                };
            }
            // Git reads a zero-byte file as "no pathspecs were given", so the form becomes the one without a path at
            //  all: a `checkout` of a tree writes that whole tree (by force, over every entry) and guesses the remote
            //  branch its name stands for, and a `restore` is Git's own refusal, which writes nothing.
            return match target {
                PathspecTarget::Tree(Some(revision)) => {
                    Ok(inside_places(tree_paths(facts, Some(revision.as_os_str()), &[], *guesses_a_remote_branch, run_in)?))
                }
                PathspecTarget::Tree(None) | PathspecTarget::Index => Ok(Vec::<WrittenPlace>::new()),
            };
        }
    };
}

/// What:
///  The paths a forced `git mv` writes,
///  read from the worktree the way Git reads them.
///  `&[OsString]` borrows the sources, `&OsStr` the destination,
///  `&Path` the directory Git runs the form in.
/// Why:
///  Git moves every source into a destination that is a directory
///  (`git mv -f a.txt b` writes `b/a.txt` and leaves `b/` and its other files alone),
///  renames its one source onto a destination that is not one,
///  and refuses the shapes it cannot do;
///  what a shape writes decides whether an entry in the way is really lost,
///  so the shapes Git refuses answer "nothing is written" and Git's own refusal answers the person
///  (each measured on this host with native Git 2.56.0, 2026-11-14):
///  several sources onto a non-directory (`fatal: destination 'x' is not a directory`),
///  a trailing slash on a destination that is not there (`destination directory does not exist`),
///  a directory source onto an existing file (`destination already exists`),
///  a source whose name inside the destination is a directory (`fatal: Cannot overwrite`),
///  and a missing source (`bad source`).
///  A metadata lookup that gives no answer stops the command,
///  because the shape it would write was not seen.
///  Relative words are read from the directory the command runs in,
///  which is the worktree's top for the command a `!` alias's shell runs,
///  and the names this answers are rooted there.
///  A source's trailing separators are dropped before it is looked up:
///  Git reads `a.txt/` and `a.txt//` as `a.txt` and renames it
///  (`git mv -f a.txt/ renamed.txt` exits 0 and writes `renamed.txt`, measured on this host with
///  native Git 2.56.0,
///  2026-11-14),
///  so a lookup of the spelling as typed met `ENOTDIR` and refused a rename Git runs.
///  A source with nothing left after its separators is the `bad source` Git refuses, which writes
///  nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function movePaths(facts, ['a.txt'], 'b', runIn): Promise<Buffer[]>;
/// ```
fn move_paths(
    facts: &mut GitFacts,
    sources: &[OsString],
    destination: &OsStr,
    run_in: &Path,
) -> Result<Vec<WrittenPlace>, String> {
    let reached: Named = named_entry(run_in.join(destination).as_path())?;
    // Which source is a directory of its own decides two of Git's refusals, and a source that is not there is
    //  Git's `bad source`, which writes nothing.
    let mut sources_are_directories: Vec<bool> = Vec::<bool>::with_capacity(sources.len());
    for source in sources {
        let named: &[u8] = without_trailing_separators(source.as_encoded_bytes());
        if named.is_empty() {
            return Ok(Vec::<WrittenPlace>::new());
        }
        match named_entry(run_in.join(path_from_bytes(named).as_os_str()).as_path())? {
            Named::Missing => return Ok(Vec::<WrittenPlace>::new()),
            Named::Directory => sources_are_directories.push(true),
            Named::Other => sources_are_directories.push(false),
        }
    }
    let mut written: Vec<WrittenPlace> = Vec::<WrittenPlace>::new();
    match reached {
        // A destination that is a directory is one Git moves into: the entry in the way is each source's name
        //  inside it, and the directory itself is not written.
        Named::Directory => {
            for source in sources {
                let mut target: Vec<u8> = destination.as_encoded_bytes().to_vec();
                if !target.ends_with(b"/") {
                    target.push(b'/');
                }
                target.extend(source_name(source));
                match named_entry(run_in.join(path_from_bytes(target.as_slice()).as_os_str()).as_path())? {
                    // Git refuses to write over a directory of that name (`Cannot overwrite`), and writes nothing.
                    Named::Directory => return Ok(Vec::<WrittenPlace>::new()),
                    Named::Missing | Named::Other => {
                        written.push(the_place_a_word_names(facts, run_in, target.as_slice())?);
                    }
                }
            }
        }
        // A destination Git cannot read as a directory means a rename onto that path; the shapes Git refuses there
        //  write nothing at all.
        Named::Missing | Named::Other => {
            let refused: bool = destination.as_encoded_bytes().ends_with(b"/")
                || sources.len() > 1
                || (matches!(reached, Named::Other) && sources_are_directories.first() == Some(&true));
            if !refused {
                written.push(the_place_a_word_names(facts, run_in, destination.as_encoded_bytes())?);
            }
        }
    }
    return Ok(written);
}

/// What:
///  What the worktree holds at one path:
///  a directory, something else, or nothing.
///  An `enum` is a closed set of named alternatives.
/// Why:
///  Git's answer for a destination turns on whether it is a directory
///  (a metadata lookup that does not follow a link decides it),
///  and a path that is not there is the shape a rename creates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Named = 'directory' | 'other' | 'missing';
/// ```
enum Named {
    /// A directory of the worktree.
    Directory,
    /// A file or a symbolic link.
    Other,
    /// Nothing at all.
    Missing,
}

/// What:
///  What stands at one path, read under a ceiling,
///  or a failure when the lookup gave no answer.
///  `&Path` borrows the path.
/// Why:
///  A metadata lookup on a mount whose server stopped answering waits in the kernel with no end,
///  and a shape the reading did not see is one it cannot judge:
///  no answer is a refusal naming the path, never "nothing is there".
///  `ENOTDIR` is a name with a component below a file, which no path stands at and Git refuses
///  itself (`bad source` for `git mv -f a.txt/. renamed`), so it is read as nothing there rather
///  than as a lookup that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedEntry(path): Named; // throws
/// ```
fn named_entry(path: &Path) -> Result<Named, String> {
    let Some(answer) =
        crate::metadata_of_a_path_within_a_ceiling::symlink_metadata_within(WORKTREE_LOOKUP_CEILING, path)
    else {
        return Err(format!(
            "the worktree entry `{}` did not answer within {} seconds, so the guard cannot tell what this command writes over",
            path.display(),
            WORKTREE_LOOKUP_CEILING.as_secs()
        ));
    };
    return match answer {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(Named::Directory),
        Ok(_) => Ok(Named::Other),
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound || error.kind() == std::io::ErrorKind::NotADirectory =>
        {
            Ok(Named::Missing)
        }
        Err(error) => Err(format!("Git could not read `{}`: {error}", path.display())),
    };
}

/// What:
///  One name with the trailing separators Git ignores dropped.
///  `&[u8]` borrows the name, `&[u8]` is the same bytes without them.
/// Why:
///  Git reads `a.txt/` as `a.txt` and renames it, and `dir//` as `dir`
///  (measured on this host with native Git 2.56.0,
///  2026-11-14),
///  while a lookup of the spelling as typed fails with `ENOTDIR` and refused the rename.
///  `a.txt/.` keeps its last component and is Git's own `bad source`, which the lookup answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutSeparators = (name) => name.replace(/\/+$/, '');
/// ```
fn without_trailing_separators(name: &[u8]) -> &[u8] {
    let mut trimmed: &[u8] = name;
    while let Some(shorter) = trimmed.strip_suffix(b"/") {
        trimmed = shorter;
    }
    return trimmed;
}

/// What:
///  The name a source takes inside a destination directory:
///  its last component, with the trailing separators Git ignores.
/// Why:
///  `git mv -f d1/x.txt b` writes `b/x.txt`, and `git mv -f a.txt/ newname` still names `a.txt`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (source) => source.replace(/\/+$/, '').split('/').pop();
/// ```
fn source_name(source: &OsStr) -> Vec<u8> {
    let mut trimmed: &[u8] = source.as_encoded_bytes();
    while let Some(shorter) = trimmed.strip_suffix(b"/") {
        trimmed = shorter;
    }
    return match trimmed.iter().rposition(|byte| return *byte == b'/') {
        Some(slash) => trimmed[slash + 1..].to_vec(),
        None => trimmed.to_vec(),
    };
}

/// What:
///  One file a guarded form writes, at the place Git will write it.
/// Why:
///  Git writes both kinds: an entry of this worktree,
///  spelled as its own listings spell it,
///  and a path anywhere else on the machine,
///  which an absolute word names,
///  a `..` that leaves the worktree reaches,
///  a `--prefix` glued to an index path as text produces,
///  or a link on the way turns one into.
///  A file the form writes outside the worktree is a person's file as much as one inside it,
///  so the guard looks at it where it lies,
///  and there is no third answer here:
///  "no path" is the answer that let Git write a file nobody watched
///  (the eighth review, 2026-11-15).
///  A word this reading cannot place is an `Err` naming it,
///  which the caller refuses on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrittenPlace = { kind: 'inside'; path: Buffer } | { kind: 'outside'; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum WrittenPlace {
    /// Below the worktree root, as Git's listings spell it:
    ///  relative,
    ///  no `.` or `..` component,
    ///  no doubled or trailing separator.
    Inside(Vec<u8>),
    /// A path that leaves the worktree, resolved as the operating system reads it.
    Outside(PathBuf),
}

/// What:
///  One word of a command line as the place Git will write,
///  or why this reading cannot tell.
///  `&[u8]` borrows the word, `&Path` the directory the command runs in.
/// Why:
///  The word is read the way the kernel reads it:
///  joined to the directory the command runs in,
///  every link on the way resolved as far as the path exists,
///  and every `.` and `..` applied there,
///  so an absolute path inside the worktree is that entry,
///  a `..` that leaves it is a path outside,
///  and a path through a link is the file the link reaches
///  (`git mv -f a.txt link-out/b.txt` writes the link's target, measured on this host with native
///  Git 2.56.0,
///  2026-11-15).
///  Both resolutions run in one job the command waits for only until the worktree lookup ceiling,
///  because a `canonicalize` on a mount whose server stopped answering never returns,
///  and a path it cannot read is an `Err` that names the word rather than a place it guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePlaceAWordNames(facts, runIn, '/abs/w/b.txt') === { kind: 'inside', path: 'b.txt' };
/// thePlaceAWordNames(facts, runIn, '../outside/b.txt') === { kind: 'outside', path: '/abs/outside/b.txt' };
/// ```
fn the_place_a_word_names(facts: &mut GitFacts, run_in: &Path, word: &[u8]) -> Result<WrittenPlace, String> {
    let location: crate::repository_location::RepositoryLocation = facts.location()?;
    let Some(root) = worktree_root(&location.identity) else {
        return Err(format!(
            "the guard could not place the path `{}` this command writes: this repository has no worktree of its own, so no directory reads that word",
            String::from_utf8_lossy(word)
        ));
    };
    let named: PathBuf = if word.starts_with(b"/") {
        PathBuf::from(path_from_bytes(word))
    } else {
        run_in.join(path_from_bytes(word))
    };
    let (resolved_root, resolved_named): (PathBuf, PathBuf) = match resolve_within_a_ceiling(root, named.as_path()) {
        Some(found) => found,
        None => {
            return Err(format!(
                "the guard could not place the path `{}` this command writes: the worktree did not answer within {} seconds, so the guard cannot tell which file that word names",
                String::from_utf8_lossy(word),
                WORKTREE_LOOKUP_CEILING.as_secs()
            ));
        }
    };
    return place_of(resolved_root.as_path(), resolved_named, word);
}

/// What:
///  Two paths resolved as the operating system reads them, as far as they exist,
///  in one job the command waits for only until the worktree lookup ceiling.
///  Nothing when the job gave no answer in that time.
///  `&Path` borrows the worktree root and the word's path.
/// Why:
///  A path is one place only as the kernel resolves it,
///  so both sides of the comparison resolve:
///  the root Git named,
///  and the word joined to the directory the command runs in.
///  The job is one thread because the ceiling covers the pair,
///  and a lookup that never answers must not hold the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const both = workWithin(CEILING, () => [resolve(root), resolve(joined)]);
/// ```
fn resolve_within_a_ceiling(root: &Path, named: &Path) -> Option<(PathBuf, PathBuf)> {
    let root_owned: PathBuf = root.to_path_buf();
    let named_owned: PathBuf = named.to_path_buf();
    return crate::work_awaited_within_a_ceiling::work_within(WORKTREE_LOOKUP_CEILING, move || {
        return (
            crate::repository_link_paths::resolve_as_far_as_it_exists(root_owned.as_path()),
            crate::repository_link_paths::resolve_as_far_as_it_exists(named_owned.as_path()),
        );
    });
}

/// What:
///  One resolved path as the place it is, below the resolved worktree root or outside it.
///  `&[u8]` borrows the word it came from, for the sentence of a refusal.
/// Why:
///  The comparison is between two resolved paths,
///  so a word that reaches an entry of the worktree by any route is that entry,
///  and everything else is a path outside it.
///  The root itself is no file a form writes,
///  so a word that resolves to it is an `Err` naming the word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// placeOf('/abs/w', '/abs/w/b.txt', '../b.txt') === { kind: 'inside', path: 'b.txt' };
/// ```
fn place_of(resolved_root: &Path, resolved: PathBuf, word: &[u8]) -> Result<WrittenPlace, String> {
    let Ok(relative) = resolved.strip_prefix(resolved_root) else {
        return Ok(WrittenPlace::Outside(resolved));
    };
    return match as_the_worktree_listing_spells_it(&[], relative.as_os_str().as_encoded_bytes()) {
        Some(spelled) => Ok(WrittenPlace::Inside(spelled)),
        None => Err(format!(
            "the guard could not place the path `{}` this command writes: it names the worktree's own top, which is no file",
            String::from_utf8_lossy(word)
        )),
    };
}

/// What:
///  One path as the worktree root and Git's own listings spell it,
///  from the prefix the form's words are read below and the path they name.
///  `&[u8]` borrows both,
///  `Option<Vec<u8>>` is the path or nothing when it leaves the worktree.
/// Why:
///  Git prints a path of the worktree one way only:
///  relative to the worktree root,
///  no `.` component,
///  no `..` component,
///  no doubled or trailing separator.
///  The entries this reading compares with are those printed paths,
///  so a word that names one of them in another spelling is the same entry and must be written the
///  same way, or the comparison misses a file the command would overwrite
///  (`mv -f a.txt ./b.txt` and `checkout-index -f --prefix=./out/ -a` wrote over an untracked
///  `b.txt` and `out/a.txt` while the reading named `./b.txt` and `./out/a.txt`).
///  A leading separator is refused here: an absolute path is the caller's question,
///  asked of the place a word names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// asTheWorktreeListingSpellsIt('sub/', '../a.txt') === undefined;
/// asTheWorktreeListingSpellsIt('', './a//b.txt') === 'a/b.txt';
/// ```
pub(super) fn as_the_worktree_listing_spells_it(prefix: &[u8], path: &[u8]) -> Option<Vec<u8>> {
    if path.starts_with(b"/") {
        return None;
    }
    let mut names: Vec<&[u8]> = Vec::<&[u8]>::new();
    for piece in [prefix, path] {
        for component in piece.split(|byte| return *byte == b'/') {
            match component {
                b"" | b"." => continue,
                // A parent component steps up one component already read, and one that steps above the worktree
                //  names nothing the worktree holds.
                b".." => {
                    names.pop()?;
                }
                name => names.push(name),
            }
        }
    }
    if names.is_empty() {
        return None;
    }
    let mut spelled: Vec<u8> = Vec::<u8>::with_capacity(names.iter().map(|name| return name.len() + 1).sum());
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            spelled.push(b'/');
        }
        spelled.extend_from_slice(name);
    }
    return Some(spelled);
}

/// What:
///  Answers Git printed as index paths, each as the place it names below the worktree root.
/// Why:
///  A listing prints the worktree's own paths,
///  and it never names a path outside the worktree,
///  so these are entries of the worktree and nothing is resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// insidePlaces(['a.txt', 'sub/b.txt']) === [{ kind: 'inside', path: 'a.txt' }, ...];
/// ```
fn inside_places(paths: Vec<Vec<u8>>) -> Vec<WrittenPlace> {
    return paths.into_iter().map(|path| return WrittenPlace::Inside(path)).collect();
}

/// What:
///  The bytes of the file at `path`,
///  read once and never past `limit` bytes,
///  which is the `limit + 1` it reads so that a file past it is recognised.
///  `&Path` borrows the file, and `u64` is the bound.
/// Why:
///  The read runs on a thread this process abandons at the read ceiling,
///  so every byte it can still take is memory this process keeps after the answer:
///  a bound is what keeps a file larger than memory from growing it without end.
///  The file is opened the way Git opens it rather than as a plain file only,
///  because a process substitution names a pipe
///  (`--pathspec-from-file=<(printf 'a.txt\n')`),
///  and Git reads it.
///  A file past the bound is `InvalidData` with no size in the error, so the refusal can
///  name the bound and the path in the person's own terms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = readAtMost(path, limit); // throws InvalidData when longer
/// ```
fn read_at_most(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    let file: std::fs::File = std::fs::File::open(path)?;
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(&file, limit + 1), &mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "larger than the guard reads"));
    }
    return Ok(bytes);
}

/// What:
///  The pathspecs the file a form names holds,
///  each as its own token,
///  `Result<Vec<OsString>, String>` is "the pathspecs, or why the file could not be read".
///  `&OsStr` borrows the file's name, `&Path` the directory a relative name is read from.
/// Why:
///  `--pathspec-from-file` names paths no token of the command does,
///  so a reading that ignores it asks Git about nothing and the file the command writes over is
///  never seen.
///  Git separates the entries with line ends, or with NUL bytes when `--pathspec-file-nul` is
///  written, and a line ending may carry the carriage return of a CRLF file.
///  A file that cannot be read is a failure, never an empty answer:
///  Git refuses the command for the same reason, and a guard must not read that as "nothing is
///  written".
///  A relative name means the directory the command runs in,
///  which is the worktree's top for the command a `!` alias's shell runs.
///  The bytes are read under a byte bound,
///  because the thread that reads them is abandoned when the ceiling passes and a read with no
///  bound went on filling this process's memory after the answer
///  (a sparse file larger than this host's memory raised the wrapper's resident memory to a
///  scope's whole gigabyte in the five seconds before the refusal,
///  measured on this host on 2026-11-14).
///  A file past the bound is refused by its size,
///  and a pipe is still read, which is what a process substitution names
///  (`--pathspec-from-file=<(printf 'a.txt\n')`):
///  refusing it would refuse a spelling Git runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathspecsInTheFile(file, nulSeparated, directory): string[]; // throws
/// ```
fn pathspecs_in_the_file(
    facts: &mut GitFacts,
    file: &OsStr,
    nul_separated: bool,
    directory: &Path,
) -> Result<Vec<OsString>, String> {
    // Git itself waits for a writer on a FIFO and for a mount that stopped answering
    //  (`timeout 10 git checkout --pathspec-from-file=<fifo>` was still waiting when the probe ended it,
    //  measured on this host with native Git 2.56.0, 2026-11-13),
    //  and the wrapper's own rule is that nothing waits forever, so the read runs on a thread this
    //  process stops waiting for at the read ceiling.
    let ceiling: std::time::Duration = crate::wait_ceilings::PATHSPEC_FILE_READ_CEILING;
    // `-` is standard input to Git, which this guard cannot read without consuming what Git needs.
    if file.as_encoded_bytes() == b"-" {
        return Err(String::from(
            "Git could not read the pathspecs `--pathspec-from-file` names, because `-` means standard input, which the guard cannot read before Git does",
        ));
    }
    let named: PathBuf = PathBuf::from(file);
    let absolute: PathBuf = if named.is_absolute() {
        named
    } else {
        directory.join(named.as_path())
    };
    let opened: PathBuf = absolute.clone();
    let Some(answer) = crate::work_awaited_within_a_ceiling::work_within(ceiling, move || {
        return read_at_most(opened.as_path(), PATHSPEC_FILE_READ_LIMIT);
    }) else {
        return Err(format!(
            "the file `{}` that `--pathspec-from-file` names gave no bytes within {} seconds, so the guard cannot tell which paths this command writes over",
            absolute.display(),
            ceiling.as_secs()
        ));
    };
    let bytes: Vec<u8> = match answer {
        Ok(found) => found,
        // A file past the byte bound says its size, which is what the person acts on.
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
            return Err(format!(
                "Git could not read the pathspecs `{}` names: it holds more than {PATHSPEC_FILE_READ_LIMIT} bytes, which is more than the guard reads, so the guard cannot tell which paths this command writes over",
                absolute.display()
            ));
        }
        Err(error) => {
            return Err(format!("Git could not read the pathspecs `{}` names: {error}", absolute.display()));
        }
    };
    // What:
    //  The bytes read here are the bytes Git is given.
    // Why:
    //  Git opens the file a second time,
    //  and a pipe has one writer:
    //  the guard consumed `a.txt\n` from a named pipe and forwarded the same name, and Git then waited
    //  for a second writer until the run was killed at its deadline
    //  (the seventh review, reproduced here).
    //  A file changed between the two reads is the other half:
    //  Git would act on names the guard never saw.
    hand_the_bytes_to_git(facts, bytes.as_slice(), file.as_encoded_bytes(), absolute.as_path())?;
    // A zero-byte file is no pathspec at all, which Git reads as "none were given"
    //  (`git checkout -f other --pathspec-from-file=<empty>` switches the branch by force,
    //  measured on this host with native Git 2.56.0, 2026-11-13);
    //  an empty entry inside a file is Git's own refusal
    //  (`fatal: empty string is not a valid pathspec`, measured in the same place),
    //  and neither may be read as "nothing is written by a form that names paths".
    if bytes.is_empty() {
        return Ok(Vec::<OsString>::new());
    }
    let separator: u8 = if nul_separated { 0 } else { b'\n' };
    let mut pathspecs: Vec<OsString> = Vec::<OsString>::new();
    let mut entries: Vec<&[u8]> = bytes.split(|byte| return *byte == separator).collect::<Vec<&[u8]>>();
    // A line-ended file ends with a separator, which names no entry of its own.
    if let Some(last) = entries.last()
        && last.is_empty()
    {
        entries.pop();
    }
    for entry in entries {
        let line: &[u8] = if nul_separated { entry } else { entry.strip_suffix(b"\r").unwrap_or(entry) };
        if line.is_empty() {
            return Err(format!(
                "the file `{}` that `--pathspec-from-file` names holds an empty pathspec, which Git refuses (`empty string is not a valid pathspec`), and the guard cannot tell which paths this command writes over",
                absolute.display()
            ));
        }
        // A line that begins with a double quote is a C-quoted string to Git, and one that does not is the name
        //  itself: the quoted form is what Git prints for a name it cannot show plainly.
        let unquoted: Vec<u8> = if !nul_separated && line.first() == Some(&b'"') {
            match unquote_c_style(line, absolute.as_path())? {
                Some(found) => found,
                None => line.to_vec(),
            }
        } else {
            line.to_vec()
        };
        pathspecs.push(path_from_bytes(unquoted.as_slice()));
    }
    return Ok(pathspecs);
}

/// What:
///  The instant this wrapper process began,
///  captured once,
///  in nanoseconds since the Unix epoch.
/// Why:
///  The copy the guard hands Git is named by the invocation that wrote it,
///  and two stages of one invocation must compute the same name while an earlier run must not:
///  a name carrying this value is never shared with a process that started another time,
///  so a later run cannot find a stale copy and hand Git bytes it never read.
///  A process that started in the same nanosecond as this one and read the same file name would
///  share the name, and that needs both to happen at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startedAt = (() => { let once; return () => once ??= BigInt(Date.now()) * 1_000_000n; })();
/// ```
pub(crate) fn this_invocation_started_at() -> u128 {
    static STARTED_AT: std::sync::OnceLock<u128> = std::sync::OnceLock::new();
    return *STARTED_AT.get_or_init(|| {
        return std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
    });
}

/// What:
///  The name of the copy one pathspec file's bytes are handed to Git under,
///  inside the wrapper's private scratch directory.
///  `&[u8]` borrows the file's name as the person wrote it.
/// Why:
///  Both stages of one invocation build this name from the same two things,
///  and neither reads the file:
///  the guard writes the copy when it reads the bytes,
///  and the transform substitutes the name in the words Git receives,
///  so Git reads the bytes the guard read.
///  The file's own name is hashed rather than embedded,
///  because a path may hold any bytes and a name that is too long is not a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copyName = (directory, written) => join(directory, `pathspec-from-file-${process.pid}-${startedAt()}-${fnv1a(written)}`);
/// ```
pub(crate) fn a_pathspec_copy_the_guard_hands_to_git(scratch: &Path, written: &[u8]) -> PathBuf {
    // FNV-1a, 64 bits: one pass, no allocation, and stable across processes.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in written {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    return scratch.join(format!(
        "pathspec-from-file-{}-{}-{hash:016x}",
        std::process::id(),
        this_invocation_started_at()
    ));
}

/// What:
///  The copy one pathspec file's bytes were handed to Git under,
///  when this invocation wrote one,
///  asked of the place itself under the worktree lookup ceiling.
///  `&[u8]` borrows the file's name as the person wrote it.
/// Why:
///  The transform substitutes the copy only when the guard read that file,
///  and the file's presence under this invocation's own name is that answer:
///  a command run with the guard's opt-out read nothing and finds no copy,
///  so it is forwarded exactly as the person typed it.
///  A lookup that gives no answer is no copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const there = (scratch, written) => workWithin(CEILING, () => stat(copyName(scratch, written))).ok ? path : undefined;
/// ```
pub(crate) fn the_pathspec_copy_that_is_there(scratch: &Path, written: &[u8]) -> Option<PathBuf> {
    let copy: PathBuf = a_pathspec_copy_the_guard_hands_to_git(scratch, written);
    let named: PathBuf = copy.clone();
    let found = crate::work_awaited_within_a_ceiling::work_within(WORKTREE_LOOKUP_CEILING, move || {
        return std::fs::symlink_metadata(named.as_path());
    });
    return match found {
        Some(Ok(_)) => Some(copy),
        Some(Err(_)) | None => None,
    };
}

/// What:
///  Write the bytes one pathspec file held into the copy Git will read,
///  so that the file is read once,
///  or refuse the command when the copy cannot be written.
///  `&mut GitFacts` asks the repository's common directory,
///  the first `&[u8]` borrows the bytes,
///  the second the file's name as the person wrote it
///  (the copy's name is built from it, so both stages agree without reading anything),
///  and `&Path` the file they came from, for the sentence.
/// Why:
///  A pipe has one writer,
///  and a file can change between two reads:
///  handing Git the bytes is what makes its reading of them the guard's reading.
///  A copy that cannot be written means Git would read the file again,
///  which is the wait and the divergence this exists to prevent,
///  so the command is refused and the sentence names the file and the opt-out runs it as Git runs it.
///  Both the directory and the write run on threads this process waits for only until the worktree
///  lookup ceiling,
///  because a `mkdir` or a write on a mount whose server stopped answering never returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function handTheBytesToGit(facts, bytes, file): void; // throws a refusal naming `file`
/// ```
fn hand_the_bytes_to_git(facts: &mut GitFacts, bytes: &[u8], written: &[u8], file: &Path) -> Result<(), String> {
    let location: crate::repository_location::RepositoryLocation = facts.location()?;
    let Some(common) = common_directory(&location.identity) else {
        return Err(format!(
            "the guard read the pathspecs `{}` names and could not name the place it gives Git the same bytes from, so it cannot tell which paths this command writes over",
            file.display()
        ));
    };
    let common_owned: PathBuf = common.to_path_buf();
    let prepared = crate::work_awaited_within_a_ceiling::work_within(WORKTREE_LOOKUP_CEILING, move || {
        return prepare_repository_scratch(common_owned.as_path());
    });
    let scratch: PathBuf = match prepared {
        Some(Ok(directory)) => directory,
        Some(Err(error)) => {
            return Err(format!(
                "the guard read the pathspecs `{}` names and could not make the place it gives Git the same bytes from ({error}), so it cannot tell which paths this command writes over",
                file.display()
            ));
        }
        None => {
            return Err(format!(
                "the guard read the pathspecs `{}` names and the place it gives Git the same bytes from did not answer within {} seconds, so it cannot tell which paths this command writes over",
                file.display(),
                WORKTREE_LOOKUP_CEILING.as_secs()
            ));
        }
    };
    let copy: PathBuf = a_pathspec_copy_the_guard_hands_to_git(scratch.as_path(), written);
    let handed: Vec<u8> = bytes.to_vec();
    let written_path: PathBuf = copy.clone();
    let answer = crate::work_awaited_within_a_ceiling::work_within(WORKTREE_LOOKUP_CEILING, move || {
        return std::fs::write(written_path.as_path(), handed);
    });
    return match answer {
        Some(Ok(())) => Ok(()),
        Some(Err(error)) => Err(format!(
            "the guard read the pathspecs `{}` names and could not write the copy it gives Git instead ({error}), so it cannot tell which paths this command writes over",
            file.display()
        )),
        None => Err(format!(
            "the guard read the pathspecs `{}` names and writing the copy it gives Git instead did not answer within {} seconds, so it cannot tell which paths this command writes over",
            file.display(),
            WORKTREE_LOOKUP_CEILING.as_secs()
        )),
    };
}

/// What:
///  The bytes of one C-quoted line of a pathspec file,
///  which is the name Git reads from it,
///  or nothing when the line does not end its quote.
///  `&[u8]` borrows the line, `&Path` the file, for the refusal's words.
/// Why:
///  Git unquotes a line that begins with a double quote (`parse_pathspec_file` in `pathspec.c`):
///  `\t`, `\n` and the other C escapes become their bytes,
///  an octal `\ooo` becomes one byte,
///  and anything after the closing quote is ignored
///  (`"plain.txt" junk` writes `plain.txt`,
///  measured on this host with native Git 2.56.0, 2026-11-13).
///  An escape Git does not know, or a quote it never closes, is Git's own refusal
///  (`fatal: line is badly quoted`), so the reading refuses too rather than asking about a path
///  Git never writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unquoted = (line) => line[0] === '"' ? unquoteCStyle(line) : line;
/// ```
fn unquote_c_style(line: &[u8], file: &Path) -> Result<Option<Vec<u8>>, String> {
    let mut found: Vec<u8> = Vec::<u8>::new();
    let mut at: usize = 1;
    while at < line.len() {
        let byte: u8 = line[at];
        at += 1;
        match byte {
            b'"' => return Ok(Some(found)),
            b'\\' => {
                let Some(escaped) = line.get(at) else {
                    return Err(badly_quoted(line, file));
                };
                at += 1;
                match *escaped {
                    b'a' => found.push(0x07),
                    b'b' => found.push(0x08),
                    b'f' => found.push(0x0c),
                    b'n' => found.push(b'\n'),
                    b'r' => found.push(b'\r'),
                    b't' => found.push(b'\t'),
                    b'v' => found.push(0x0b),
                    b'\\' => found.push(b'\\'),
                    b'"' => found.push(b'"'),
                    digit if (b'0'..=b'7').contains(&digit) => {
                        let mut value: u32 = u32::from(digit - b'0');
                        let mut taken: usize = 1;
                        while taken < 3
                            && let Some(next) = line.get(at)
                            && (b'0'..=b'7').contains(next)
                        {
                            value = value * 8 + u32::from(*next - b'0');
                            at += 1;
                            taken += 1;
                        }
                        if value > u32::from(u8::MAX) {
                            return Err(badly_quoted(line, file));
                        }
                        found.push(u8::try_from(value).unwrap_or(0));
                    }
                    _ => return Err(badly_quoted(line, file)),
                }
            }
            other => found.push(other),
        }
    }
    return Err(badly_quoted(line, file));
}

/// What:
///  The sentence for a quote Git would reject,
///  naming the file and the line.
/// Why:
///  A refusal a person reads has to say which file holds the line Git cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = (line, file) => `... line is badly quoted ... ${file}`;
/// ```
fn badly_quoted(line: &[u8], file: &Path) -> String {
    return format!(
        "the file `{}` that `--pathspec-from-file` names holds a line Git refuses as badly quoted (`{}`), so the guard cannot tell which paths this command writes over",
        file.display(),
        String::from_utf8_lossy(line)
    );
}

/// What:
///  One path as the operating system takes it,
///  from the bytes a file or Git printed.
/// Why:
///  A name that is not UTF-8 is still a name;
///  converting it through `String` would replace its bytes and look up a file that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fromBytes = (bytes) => bytes; // a Buffer is already the name
/// ```
pub(super) fn path_from_bytes(bytes: &[u8]) -> OsString {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return OsStr::from_bytes(bytes).to_os_string();
    }
    #[cfg(not(unix))]
    {
        return OsString::from(String::from_utf8_lossy(bytes).into_owned());
    }
}

/// What:
///  The paths of one revision's tree that the given pathspecs match,
///  or every path of it when there are none,
///  read from the name the command itself turns into a tree.
///  `bool` says whether `checkout` and `switch` are free to guess the remote branch a short name
///  stands for,
///  and `&Path` borrows the directory Git runs the command in.
/// Why:
///   `git ls-tree -r -z --name-only` prints each path of the tree,
///  and Git itself matches the pathspecs,
///  so a directory, a glob and a literal name reach exactly what the form reaches.
///  `-` is the previous branch only to `switch` and `checkout`,
///  so it is read as `@{-1}` before the tree is listed.
///  A name Git cannot turn into a tree is never read as "this form writes nothing":
///  `git ls-tree topic` fails for a remote-only branch while `git checkout -f topic` writes that
///  branch's tree,
///  so the name is resolved the way the command resolves it,
///  and a tree Git knows but cannot list is a refusal
///  (`name_git_may_still_reach` says how the two are told apart).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treePaths(facts, revision, pathspecs, guesses, runIn): Promise<Buffer[]>; // throws
/// ```
fn tree_paths(
    facts: &mut GitFacts,
    revision: Option<&OsStr>,
    pathspecs: &[OsString],
    guesses_a_remote_branch: bool,
    run_in: &Path,
) -> Result<Vec<Vec<u8>>, String> {
    // `-` is the previous branch only to `switch` and `checkout`, so it is read as the expression Git resolves
    //  wherever an object name is read; `@{-N}` needs no such reading.
    let named: OsString = match revision {
        Some(written) => match crate::command_discard::previous_branch_expression(written.as_encoded_bytes()) {
            Some(expression) => OsString::from(String::from_utf8_lossy(expression).into_owned()),
            None => written.to_os_string(),
        },
        None => OsString::from("HEAD"),
    };
    let Some(paths) = listed_paths_of(facts, named.as_os_str(), pathspecs, run_in)? else {
        return tree_of_a_name_git_may_still_reach(facts, named.as_os_str(), pathspecs, guesses_a_remote_branch, run_in);
    };
    return Ok(paths);
}

/// What:
///  The tree of a name `git ls-tree` did not list,
///  when Git may still reach one through it:
///  the local branch a `checkout` or a `switch` creates from a remote-tracking branch of that name.
/// Why:
///  `git checkout -f topic` with only `refs/remotes/origin/topic` writes that tree over the
///  worktree
///  (`Switched to a new branch 'topic'`, measured on this host with native Git 2.56.0,
///  2026-11-13),
///  so a guard that read the failed listing as "nothing is written" let the forced form overwrite
///  an untracked file.
///  The three answers this reading can give are all safe ones:
///  the tree's paths, so the entries in the way are found;
///  no path at all, only when Git itself refuses the name and no branch is there to guess; and
///  a refusal, when Git knows an object by the name but could not list its tree,
///  or when the guessed branch's tree could not be listed either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeOfANameGitMayStillReach(facts, 'topic', [], true): Promise<Buffer[]>; // throws
/// ```
fn tree_of_a_name_git_may_still_reach(
    facts: &mut GitFacts,
    named: &OsStr,
    pathspecs: &[OsString],
    guesses_a_remote_branch: bool,
    run_in: &Path,
) -> Result<Vec<Vec<u8>>, String> {
    if git_holds_an_object_named(facts, named)? {
        return Err(tree_reading_failure(named, "Git knows an object by that name and did not list its tree"));
    }
    // A form that names paths, a reset, and a restore never guess: Git refuses the name itself
    //  (`invalid reference`, `ambiguous argument`, `could not resolve`), so there is nothing to write.
    if !guesses_a_remote_branch || !pathspecs.is_empty() || !guessing_is_left_on_in_this_repository(facts)? {
        return Ok(Vec::<Vec<u8>>::new());
    }
    let Some(remote_branch) = remote_branch_a_checkout_would_create_from(facts, named)? else {
        return Ok(Vec::<Vec<u8>>::new());
    };
    return match listed_paths_of(facts, remote_branch.as_os_str(), &[], run_in)? {
        Some(paths) => Ok(paths),
        None => Err(tree_reading_failure(
            remote_branch.as_os_str(),
            "Git named that branch and did not list its tree",
        )),
    };
}

/// What:
///  The sentence a refusal prints when the guard could not read the tree a form writes,
///  naming the tree and what Git answered.
/// Why:
///  The guard cannot tell which of the person's files the command would write over,
///  so the command is not run;
///  the sentence says which name failed, that nothing ran, and that the option runs the command
///  as real Git runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cause = treeReadingFailure('topic', 'Git knows an object by that name and did not list its tree');
/// ```
fn tree_reading_failure(named: &OsStr, what_git_did: &str) -> String {
    return format!(
        "the guard could not read the tree `{}` names, so it cannot tell which of your files this command would write over ({what_git_did})",
        named.to_string_lossy()
    );
}

/// What:
///  The paths `git ls-tree -r -z --name-only` answers for one name and its pathspecs,
///  or nothing when Git exited nonzero.
///  `&Path` borrows the directory Git runs the command in.
/// Why:
///  A listing that ran and failed says the name did not resolve to a tree Git can list;
///  a listing that could not run at all is a failure of the query, never a killed read.
///  `git ls-tree` is scoped to the directory Git runs in unless `--full-tree` is written
///  (run in `sub` it printed `inner.txt` alone, where the tree holds `top.txt` and `sub/inner.txt`,
///  measured on this host with native Git 2.56.0, 2026-11-13),
///  and every guarded form acts on the whole worktree,
///  so the whole-tree reading asks with `--full-tree` and names paths the worktree root holds.
///  A form that names paths keeps Git's own reading of them,
///  which is relative to the directory the command runs in,
///  and that directory is where the query starts:
///  `-C` is left out and the directory carries the chain instead, because `-C` reaches the same
///  place and measures from the one Git was started in.
///  From a subdirectory, `checkout other -- b.txt` inside a `!` alias writes the top-level
///  `b.txt` while a query started in the caller's directory found `sub/b.txt` and let the write
///  through
///  (measured on this host with native Git 2.56.0, 2026-11-14).
///  Its answers are written below that directory as the worktree root sees it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listedPathsOf(facts, 'other', ['b.txt'], runIn): Promise<Buffer[] | undefined>;
/// ```
fn listed_paths_of(
    facts: &mut GitFacts,
    named: &OsStr,
    pathspecs: &[OsString],
    run_in: &Path,
) -> Result<Option<Vec<Vec<u8>>>, String> {
    let mut arguments: Vec<OsString> = without_directory_changes(facts.global_prefix.as_slice());
    for token in ["--no-optional-locks", "ls-tree", "-r", "-z", "--name-only"] {
        arguments.push(OsString::from(token));
    }
    if pathspecs.is_empty() {
        // `--full-tree` prints every path of the tree, whatever directory Git runs in.
        arguments.push(OsString::from("--full-tree"));
    }
    arguments.push(named.to_os_string());
    if !pathspecs.is_empty() {
        arguments.push(OsString::from("--"));
        arguments.extend(pathspecs.iter().cloned());
    }
    // A whole-tree reading names the worktree's own paths, and a path form's pathspecs mean the directory the
    //  command runs in, so that is where Git is started for both.
    let output: MetadataOutput = facts.query_in(run_in, arguments, "the tree this command writes")?;
    if !output.success {
        // A child a signal ended answered nothing at all; only a nonzero exit is an answer about the name.
        if output.code.is_none() {
            return Err(String::from(
                "Git was ended by a signal while the guard asked which paths this command writes, so it cannot tell what would be overwritten",
            ));
        }
        return Ok(None);
    }
    let mut paths: Vec<Vec<u8>> = nul_separated_paths(output.stdout.as_slice());
    if pathspecs.is_empty() || paths.is_empty() {
        return Ok(Some(paths));
    }
    // A path form's answers are relative to the directory Git runs in, and the entries in the way are named from the
    //  worktree root, so that directory's own prefix is written before each of them.
    //  An answer that cannot be spelled below the root is no entry of the worktree this reading can compare, and
    //  dropping it would be the empty answer the place type exists to make impossible.
    let location: crate::repository_location::RepositoryLocation = facts.location()?;
    let Some(root) = worktree_root(&location.identity) else {
        return Err(String::from(
            "the guard could not place the paths Git answered for this command: the repository has no worktree of its own",
        ));
    };
    let prefix: Vec<u8> = crate::command_run_directory::run_directory_prefix(root, run_in, location.prefix.as_slice());
    let mut rooted: Vec<Vec<u8>> = Vec::<Vec<u8>>::with_capacity(paths.len());
    for path in paths.drain(..) {
        match as_the_worktree_listing_spells_it(prefix.as_slice(), path.as_slice()) {
            Some(spelled) => rooted.push(spelled),
            None => {
                return Err(format!(
                    "the guard could not place the path `{}` Git answered for this command, which stands outside the worktree this reading compares entries with",
                    String::from_utf8_lossy(path.as_slice())
                ));
            }
        }
    }
    return Ok(Some(rooted));
}

/// What:
///  Whether Git holds an object the name resolves to,
///  which tells a name Git does not know from a tree it could not read.
/// Why:
///  `git ls-tree` exits nonzero for both,
///  and the two mean opposite things for a guard:
///  a name Git does not know is a form Git refuses itself, so nothing is written,
///  while an object Git knows and cannot list is a read that failed,
///  and what such a command writes is unknown.
///  `rev-parse --verify` resolves the name without reading the object,
///  so it answers for a tree whose bytes are missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const known = (facts, 'topic') => facts.query(['rev-parse', '--verify', '--quiet', '--end-of-options', 'topic']).success;
/// ```
fn git_holds_an_object_named(facts: &mut GitFacts, named: &OsStr) -> Result<bool, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "rev-parse", "--verify", "--quiet", "--end-of-options"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(named.to_os_string());
    let output: MetadataOutput = facts.query(arguments, "whether Git knows this revision")?;
    // `rev-parse --verify --quiet` answers through its exit status, so a child a signal ended answered nothing.
    if !output.success && output.code.is_none() {
        return Err(String::from(
            "Git was ended by a signal while the guard asked whether it knows the revision this command names, so it cannot tell what would be overwritten",
        ));
    }
    return Ok(output.success);
}

/// What:
///  Whether the repository leaves `checkout`'s guess of a remote branch on.
/// Why:
///  `checkout.guess=false` makes Git refuse the short name
///  (`error: pathspec 'topic' did not match any file(s) known to git`),
///  so the guard must not resolve a branch Git never touches:
///  a refusal over a name Git rejects is a command refused that Git would not have run.
///  An unset key, or an empty value, leaves the guess on;
///  a value Git does not read as a boolean is a configuration error of its own,
///  so the guard leaves the command to Git (which refuses with its own sentence about the value).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const on = (facts) => { const v = facts.query(['config', '--get', 'checkout.guess']); return !v.success || v.stdout.length === 0 || /^(true|yes|on|1)$/i.test(v.stdout.trim()); };
/// ```
fn guessing_is_left_on_in_this_repository(facts: &mut GitFacts) -> Result<bool, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "config", "--get", "checkout.guess"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the repository's answer on guessing a remote branch")?;
    if !output.success {
        // A child a signal ended answered nothing at all, which is no statement about the repository's setting.
        if output.code.is_none() {
            return Err(String::from(
                "Git was ended by a signal while the guard asked whether this repository leaves a remote branch to be guessed, so it cannot tell what this command would write over",
            ));
        }
        // `config --get` exits 1 for a key that is not set, and Git then guesses.
        //  Any other nonzero is a configuration Git objects to, and Git then refuses the command
        //  itself, so the guard resolves nothing either.
        return Ok(output.code == Some(1));
    }
    let value: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_ascii_lowercase();
    if value.is_empty() {
        return Ok(true);
    }
    return Ok(matches!(value.as_str(), "true" | "yes" | "on" | "1"));
}

/// What:
///  The remote-tracking ref a `checkout` or a `switch` would create the local branch from,
///  worked out the way Git works it out
///  (`unique_tracking_name` in `branch.c`).
/// Why:
///  One remote-tracking branch of that name is the branch Git creates;
///  several are an ambiguity Git refuses
///  (`hint: Branch name 'topic' appears in multiple remotes`, measured on this host with native Git
///  2.56.0, 2026-11-13)
///  unless `checkout.defaultRemote` names one of them,
///  and then that one is Git's answer.
///  The remotes are read from Git rather than split from the refname, because a remote's own name
///  may hold a slash:
///  with the remote `a/b`, `refs/remotes/a/b/topic` is the branch `topic` Git guesses from
///  (`Switched to a new branch 'topic'`, measured on this host).
///  A listing that failed is a failure of the query, never "no branch to guess".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function remoteBranchACheckoutWouldCreateFrom(facts, 'topic'): Promise<string | undefined>;
/// ```
fn remote_branch_a_checkout_would_create_from(facts: &mut GitFacts, named: &OsStr) -> Result<Option<OsString>, String> {
    let remotes: Vec<Vec<u8>> = remote_names(facts)?;
    if remotes.is_empty() {
        return Ok(None);
    }
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "for-each-ref", "--format=%(refname)", "refs/remotes/"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the remote-tracking branches this repository holds")?;
    if !output.success {
        return Err(String::from(
            "Git could not list the remote-tracking branches, so the guard cannot tell what this command would write over",
        ));
    }
    let mut candidates: Vec<(Vec<u8>, Vec<u8>)> = Vec::<(Vec<u8>, Vec<u8>)>::new();
    for line in output.stdout.split(|byte| return *byte == b'\n') {
        let Some(short) = line.strip_prefix(b"refs/remotes/") else {
            continue;
        };
        for remote in &remotes {
            let mut wanted: Vec<u8> = remote.clone();
            wanted.push(b'/');
            wanted.extend(named.as_encoded_bytes());
            if short == wanted.as_slice() {
                candidates.push((remote.clone(), line.to_vec()));
            }
        }
    }
    if candidates.is_empty() {
        return Ok(None);
    }
    if candidates.len() == 1 {
        return Ok(candidates.first().map(|(_, full)| return os_string_of(full.as_slice())));
    }
    let Some(default_remote) = configured_remote_to_guess_from(facts)? else {
        // Git refuses the ambiguous name itself, so the form writes nothing.
        return Ok(None);
    };
    return Ok(candidates
        .iter()
        .find(|(remote, _)| return remote.as_slice() == default_remote.as_encoded_bytes())
        .map(|(_, full)| return os_string_of(full.as_slice())));
}

/// What:
///  Every remote this repository has, as Git lists them.
/// Why:
///  A remote-tracking refname is `<remote>/<branch>`,
///  and a remote's own name may hold a slash,
///  so the split between the two is only knowable from the list of remotes.
///  A listing that failed is a failure, never "no remote".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remotes = (facts) => facts.query(['remote']).stdout.toString().split('\n').filter(Boolean);
/// ```
fn remote_names(facts: &mut GitFacts) -> Result<Vec<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "remote"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the remotes this repository has")?;
    if !output.success {
        return Err(String::from(
            "Git could not list the remotes, so the guard cannot tell what this command would write over",
        ));
    }
    let mut names: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for line in output.stdout.split(|byte| return *byte == b'\n') {
        if !line.is_empty() {
            names.push(line.to_vec());
        }
    }
    return Ok(names);
}

/// What:
///  The `checkout.defaultRemote` this repository sets,
///  or nothing when it sets none.
/// Why:
///  It decides which remote Git guesses from when several carry the branch,
///  and a value that does not name one of them leaves Git's own ambiguity refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remote = (facts) => { const v = facts.query(['config', '--get', 'checkout.defaultRemote']); return v.success ? v.stdout.trim() : undefined; };
/// ```
fn configured_remote_to_guess_from(facts: &mut GitFacts) -> Result<Option<OsString>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "config", "--get", "checkout.defaultRemote"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the remote this repository guesses a branch from")?;
    if !output.success {
        return Ok(None);
    }
    let trimmed: &[u8] = output.stdout.strip_suffix(b"\n").unwrap_or(output.stdout.as_slice());
    if trimmed.is_empty() {
        return Ok(None);
    }
    return Ok(Some(os_string_of(trimmed)));
}

/// What:
///  One name as the operating system takes it,
///  from the bytes Git printed.
/// Why:
///  A refname is bytes;
///  converting through `String` would replace a byte that is not UTF-8 and name another ref.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (bytes) => bytes; // a Buffer is already the name
/// ```
fn os_string_of(bytes: &[u8]) -> OsString {
    return path_from_bytes(bytes);
}

/// The walk over every spelling of a path a form writes, each row against real Git.
#[cfg(test)]
#[path = "repository_facts_written_paths_place_tests.rs"]
mod place_tests;

/// The names this reading takes from Git's answers,
/// and the paths they resolve to,
/// beside real-Git controls of each way a word becomes a tree.
#[cfg(test)]
#[path = "repository_facts_written_paths_tests.rs"]
mod tests;

/// What:
///  The paths of every tree a `git read-tree` form names,
///  each below the directory its `--prefix` writes it in.
///  `&[OsString]` borrows the names the form carries and the prefix it may name,
///  and `&Path` the directory Git runs the form in.
/// Why:
///   Every path a merge or a reset can write comes from one of the trees it names,
///  and `--prefix` moves each of them below a directory the form names,
///  which is where the worktree entry Git writes over really stands.
///  A revision here is read as Git reads a tree name:
///  `-` is no previous branch to `read-tree`, so it is not read as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTreePaths(facts, ['HEAD', 'other'], 'sub/', runIn) ==> ['sub/a.txt', 'sub/b.txt']
/// ```
fn read_tree_paths(
    facts: &mut GitFacts,
    revisions: &[OsString],
    prefix: Option<&OsStr>,
    run_in: &Path,
) -> Result<Vec<WrittenPlace>, String> {
    let root: PathBuf = worktree_root_of(facts)?;
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for revision in revisions {
        paths.extend(tree_paths(facts, Some(revision.as_os_str()), &[], false, run_in)?);
    }
    return the_places_under_a_prefix(root.as_path(), prefix, paths);
}

/// What:
///  The worktree root of this repository, or the sentence of a refusal when it has none.
/// Why:
///  Every place this reading answers is either relative to that root or compared with it,
///  so a repository without a worktree of its own has no place to answer,
///  and no form this reading guards runs there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = worktreeRootOf(facts); // '/abs/worktree'
/// ```
fn worktree_root_of(facts: &mut GitFacts) -> Result<PathBuf, String> {
    let location: crate::repository_location::RepositoryLocation = facts.location()?;
    return match worktree_root(&location.identity) {
        Some(root) => Ok(root.to_path_buf()),
        None => Err(String::from(
            "the guard could not place the paths this command writes: the repository has no worktree of its own, so no directory reads them",
        )),
    };
}

/// What:
///  The index's or a tree's paths as the places one prefix writes them at,
///  or those paths themselves as entries of the worktree when no prefix is named.
///  `&Path` borrows the worktree root the prefix is read from,
///  `Option<&OsStr>` the prefix as the form spelled it.
/// Why:
///  Git joins a prefix to each path as text and resolves the pair from the top of the working tree
///  (`setup_work_tree` moves there before any entry is written),
///  so `--prefix=out/` and `a.txt` write `out/a.txt`,
///  `--prefix=out2` and `sub/b.txt` write `out2sub/b.txt`,
///  and a prefix that names a directory outside the worktree writes a person's file there,
///  which this reading now names instead of dropping
///  (measured on this host with native Git 2.56.0, 2026-11-15).
///  The prefix is resolved once for all of its paths,
///  because every path is appended to it as text,
///  and that resolution is where a link on the way and a `..` are read the way the kernel reads them.
///  A trailing separator is kept as the form wrote it:
///  it decides whether the first path's name is a name inside the prefix's directory or continues it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePlacesUnderAPrefix(facts, root, 'out/', ['a.txt']) === [{ kind: 'inside', path: 'out/a.txt' }];
/// ```
fn the_places_under_a_prefix(
    root: &Path,
    prefix: Option<&OsStr>,
    paths: Vec<Vec<u8>>,
) -> Result<Vec<WrittenPlace>, String> {
    let Some(prefix) = prefix else {
        return Ok(inside_places(paths));
    };
    let written: Vec<u8> = prefix.as_encoded_bytes().to_vec();
    let named: PathBuf = if written.starts_with(b"/") {
        PathBuf::from(prefix)
    } else {
        root.join(prefix)
    };
    // One resolution of the prefix, under the same ceiling as a word's place.
    let resolved: PathBuf = match resolve_within_a_ceiling(root, named.as_path()) {
        Some((_resolved_root, resolved_prefix)) => resolved_prefix,
        None => {
            return Err(format!(
                "the guard could not place the prefix `{}` this command writes below: the worktree did not answer within {} seconds",
                String::from_utf8_lossy(written.as_slice()),
                WORKTREE_LOOKUP_CEILING.as_secs()
            ));
        }
    };
    let mut base: Vec<u8> = resolved.as_os_str().as_encoded_bytes().to_vec();
    if written.ends_with(b"/") && !base.ends_with(b"/") {
        base.push(b'/');
    }
    let resolved_root: PathBuf = match resolve_within_a_ceiling(root, root) {
        Some((resolved_root, _same)) => resolved_root,
        None => {
            return Err(format!(
                "the guard could not place the paths below `{}`: the worktree did not answer within {} seconds",
                String::from_utf8_lossy(written.as_slice()),
                WORKTREE_LOOKUP_CEILING.as_secs()
            ));
        }
    };
    let mut places: Vec<WrittenPlace> = Vec::<WrittenPlace>::with_capacity(paths.len());
    for path in paths {
        let mut glued: Vec<u8> = Vec::<u8>::with_capacity(base.len() + path.len());
        glued.extend_from_slice(base.as_slice());
        glued.extend_from_slice(path.as_slice());
        places.push(place_of(resolved_root.as_path(), PathBuf::from(path_from_bytes(glued.as_slice())), glued.as_slice())?);
    }
    return Ok(places);
}

/// What:
///  The paths the index holds that the given pathspecs match,
///  each below the directory a `git checkout-index --prefix` names.
///  `&OsString` borrows the pathspecs and the directory,
///  and `&Path` the directory Git runs the form in.
/// Why:
///  `git checkout-index --prefix=<directory>/ --all` writes the index's content there,
///  so the worktree entry a form would write over stands at that shifted path,
///  and no query of the index knows the directory's name.
///  Git joins the prefix to the path as text and resolves the pair from the top of the working
///  tree
///  (`setup_work_tree` moves there before any index entry is written),
///  so a relative prefix is read from the worktree root and an absolute one is its own path:
///  `git checkout-index -f --prefix=out/ -- b.txt` run in `sub/` writes `out/sub/b.txt`,
///  and `--prefix=out2` without a separator writes `out2sub/b.txt`
///  (measured on this host with native Git 2.56.0,
///  2026-11-14).
///  A prefix naming a directory outside the worktree writes a person's file there,
///  and this reading names that file where it lies
///  (the eighth review, 2026-11-15:
///  `git checkout-index -f --prefix=/abs/outside/ -a` wrote over a file outside the worktree without a
///  refusal).
///  The index paths it shifts are already the worktree root's own, so no directory prefix is
///  written before them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexUnderPrefixPaths(facts, pathspecs, prefix, runIn): Promise<WrittenPlace[]>; // throws
/// ```
fn index_under_prefix_paths(
    facts: &mut GitFacts,
    pathspecs: &[OsString],
    prefix: &OsStr,
    run_in: &Path,
) -> Result<Vec<WrittenPlace>, String> {
    let written: Vec<Vec<u8>> = index_paths(facts, pathspecs, run_in)?;
    if written.is_empty() {
        return Ok(Vec::<WrittenPlace>::new());
    }
    let root: PathBuf = worktree_root_of(facts)?;
    return the_places_under_a_prefix(root.as_path(), Some(prefix), written);
}

/// What:
///  The paths the index holds that the given pathspecs match,
///  or every path of it when there are none,
///  each as the worktree root names it.
///  `&Path` borrows the directory Git runs the command in.
/// Why:
///  A restore without a source writes the index's content over the worktree,
///  and a query that failed is a failure rather than an empty answer.
///  A pathspec of the form means the directory the command runs in, so the query starts there;
///  a listing with no pathspec is scoped to that directory unless `:(top)` is written,
///  and every guarded form acts on the whole worktree.
///  `--full-name` makes Git print every answer from the worktree root whatever directory the
///  query started in
///  (run in `sub/`, `ls-files --full-name -- b.txt` printed `sub/b.txt`, measured on this host with
///  native Git 2.56.0,
///  2026-11-14),
///  so the answers are already the names the entries in the way carry and nothing is written
///  before them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexPaths(facts, pathspecs: string[], runIn): Promise<Buffer[]>; // throws
/// ```
fn index_paths(facts: &mut GitFacts, pathspecs: &[OsString], run_in: &Path) -> Result<Vec<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = without_directory_changes(facts.global_prefix.as_slice());
    for token in ["--no-optional-locks", "ls-files", "-z", "--cached", "--full-name"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(OsString::from("--"));
    if pathspecs.is_empty() {
        // A listing without a pathspec is scoped to the directory Git runs in, and the paths it prints are the
        //  worktree's own, so the whole index is asked for from the top (`:(top)`).
        arguments.push(OsString::from(":(top)"));
    } else {
        arguments.extend(pathspecs.iter().cloned());
    }
    let output: MetadataOutput = facts.query_in(run_in, arguments, "the paths the index holds")?;
    if !output.success {
        let detail: String = String::from_utf8_lossy(output.stderr.as_slice())
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("Git could not read the paths the index holds: {detail}"));
    }
    return Ok(nul_separated_paths(output.stdout.as_slice()));
}
