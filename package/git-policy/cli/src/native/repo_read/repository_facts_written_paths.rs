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
pub(super) fn paths_the_tree_writes(facts: &mut GitFacts, written: &WrittenPaths, run_in: &Path) -> Result<Vec<Vec<u8>>, String> {
    return match written {
        WrittenPaths::Tree { revision, guesses_a_remote_branch, .. } => {
            tree_paths(facts, revision.as_deref(), &[], *guesses_a_remote_branch, run_in)
        }
        WrittenPaths::MatchingTree { revision, pathspecs } => {
            // A form that names paths writes the tree of the word before them, and Git resolves that word as a
            //  revision alone: `checkout -f topic -- b.txt` answers `invalid reference: topic` for a remote-only name.
            tree_paths(facts, revision.as_deref(), pathspecs.as_slice(), false, run_in)
        }
        WrittenPaths::Index { pathspecs } => index_paths(facts, pathspecs.as_slice(), run_in),
        WrittenPaths::IndexUnderPrefix { pathspecs, prefix } => index_under_prefix_paths(facts, pathspecs, prefix, run_in),
        WrittenPaths::MoveInto { sources, destination } => move_paths(facts, sources.as_slice(), destination, run_in),
        WrittenPaths::ReadTree { revisions, prefix, .. } => read_tree_paths(facts, revisions.as_slice(), prefix.as_deref(), run_in),
        WrittenPaths::PathspecFile { target, file, nul_separated, entries: _, guesses_a_remote_branch } => {
            let pathspecs: Vec<OsString> = pathspecs_in_the_file(file, *nul_separated, run_in)?;
            if !pathspecs.is_empty() {
                return match target {
                    PathspecTarget::Tree(revision) => tree_paths(facts, revision.as_deref(), pathspecs.as_slice(), false, run_in),
                    PathspecTarget::Index => index_paths(facts, pathspecs.as_slice(), run_in),
                };
            }
            // Git reads a zero-byte file as "no pathspecs were given", so the form becomes the one without a path at
            //  all: a `checkout` of a tree writes that whole tree (by force, over every entry) and guesses the remote
            //  branch its name stands for, and a `restore` is Git's own refusal, which writes nothing.
            return match target {
                PathspecTarget::Tree(Some(revision)) => {
                    tree_paths(facts, Some(revision.as_os_str()), &[], *guesses_a_remote_branch, run_in)
                }
                PathspecTarget::Tree(None) | PathspecTarget::Index => Ok(Vec::<Vec<u8>>::new()),
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
) -> Result<Vec<Vec<u8>>, String> {
    let reached: Named = named_entry(run_in.join(destination).as_path())?;
    // Which source is a directory of its own decides two of Git's refusals, and a source that is not there is
    //  Git's `bad source`, which writes nothing.
    let mut sources_are_directories: Vec<bool> = Vec::<bool>::with_capacity(sources.len());
    for source in sources {
        match named_entry(run_in.join(source).as_path())? {
            Named::Missing => return Ok(Vec::<Vec<u8>>::new()),
            Named::Directory => sources_are_directories.push(true),
            Named::Other => sources_are_directories.push(false),
        }
    }
    let mut written: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
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
                    Named::Directory => return Ok(Vec::<Vec<u8>>::new()),
                    Named::Missing | Named::Other => {
                        written.push(rooted_at_the_run_directory(facts, run_in, target.as_slice())?);
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
                written.push(rooted_at_the_run_directory(facts, run_in, destination.as_encoded_bytes())?);
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
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Named::Missing),
        Err(error) => Err(format!("Git could not read `{}`: {error}", path.display())),
    };
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
///  One path a form writes as the worktree root spells it:
///  below the directory the command runs in.
///  `&Path` borrows that directory, `&[u8]` the path the form's own words name.
/// Why:
///  The entries in the way are named from the worktree root,
///  and a form run from a directory below it names its own paths from there,
///  which is not the caller's directory when Git runs the shell of a `!` alias at the top:
///  a `git mv` inside an alias that stood below the top wrote a file the guard looked for
///  one directory down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rooted = (facts, runIn, 'b/a.txt') => Buffer.concat([runPrefix(facts, runIn), 'b/a.txt']);
/// ```
fn rooted_at_the_run_directory(facts: &mut GitFacts, run_in: &Path, path: &[u8]) -> Result<Vec<u8>, String> {
    let location: crate::repository_location::RepositoryLocation = facts.location()?;
    let prefix: Vec<u8> = match worktree_root(&location.identity) {
        Some(root) => crate::command_run_directory::run_directory_prefix(root, run_in, location.prefix.as_slice()),
        // A repository with no worktree of its own has no directory a name is read from.
        None => location.prefix.clone(),
    };
    let mut rooted: Vec<u8> = prefix;
    rooted.extend_from_slice(path);
    return Ok(rooted);
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
fn pathspecs_in_the_file(file: &OsStr, nul_separated: bool, directory: &Path) -> Result<Vec<OsString>, String> {
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
    let mut rooted: Vec<Vec<u8>> = Vec::<Vec<u8>>::with_capacity(paths.len());
    for path in paths.drain(..) {
        rooted.push(rooted_at_the_run_directory(facts, run_in, path.as_slice())?);
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
) -> Result<Vec<Vec<u8>>, String> {
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for revision in revisions {
        for path in tree_paths(facts, Some(revision.as_os_str()), &[], false, run_in)? {
            paths.push(under_the_prefix(path, prefix));
        }
    }
    return Ok(paths);
}

/// What:
///  One path as the form writes it:
///  below the prefix when it names one,
///  and unchanged when it names none.
/// Why:
///  `read-tree --prefix=sub/ -u other` writes `other`'s `a.txt` at `sub/a.txt`,
///  so the entry in the way is the one below the prefix, not the tree's own path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const underThePrefix = (path, prefix) => prefix ? Buffer.concat([prefix, path]) : path;
/// ```
fn under_the_prefix(path: Vec<u8>, prefix: Option<&OsStr>) -> Vec<u8> {
    let Some(named) = prefix else {
        return path;
    };
    let mut shifted: Vec<u8> = named.as_encoded_bytes().to_vec();
    shifted.extend(path);
    return shifted;
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
///  Git joins the prefix to the path as text and resolves the pair from the directory it runs
///  in,
///  so a relative prefix is read below that directory and an absolute one is its own path:
///  a prefix naming a directory outside the worktree writes no entry of it,
///  and a command that writes nothing of the worktree is no reason to refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexUnderPrefixPaths(facts, pathspecs, prefix, runIn): Promise<Buffer[]>; // throws
/// ```
fn index_under_prefix_paths(
    facts: &mut GitFacts,
    pathspecs: &[OsString],
    prefix: &OsStr,
    run_in: &Path,
) -> Result<Vec<Vec<u8>>, String> {
    let written: Vec<Vec<u8>> = index_paths(facts, pathspecs, run_in)?;
    if written.is_empty() {
        return Ok(written);
    }
    let location: crate::repository_location::RepositoryLocation = facts.location()?;
    let Some(root) = worktree_root(&location.identity) else {
        return Ok(Vec::<Vec<u8>>::new());
    };
    let mut base: Vec<u8> = if prefix.as_encoded_bytes().starts_with(b"/") {
        Vec::<u8>::new()
    } else {
        run_in.as_os_str().as_encoded_bytes().to_vec()
    };
    if !base.is_empty() && !base.ends_with(b"/") {
        base.push(b'/');
    }
    base.extend_from_slice(prefix.as_encoded_bytes());
    let mut shifted: Vec<Vec<u8>> = Vec::<Vec<u8>>::with_capacity(written.len());
    for path in written {
        let mut absolute: Vec<u8> = base.clone();
        absolute.extend(path);
        // An entry outside the worktree is not one it holds, so nothing there is lost.
        if let Some(below) = below_the_worktree_root(root, absolute.as_slice()) {
            shifted.push(below);
        }
    }
    return Ok(shifted);
}

/// What:
///  One absolute path as the worktree root names it,
///  or nothing when it does not stand below that root.
///  `&Path` borrows the root, `&[u8]` the path's bytes.
/// Why:
///  The entries in the way are the worktree's own, named from its root,
///  and a path Git writes outside the worktree is an entry of no worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const below = (root, '/r/out/a.txt') => '/r/' === root + '/' ? 'out/a.txt' : undefined;
/// ```
fn below_the_worktree_root(root: &Path, absolute: &[u8]) -> Option<Vec<u8>> {
    let mut root_prefix: Vec<u8> = root.as_os_str().as_encoded_bytes().to_vec();
    if !root_prefix.ends_with(b"/") {
        root_prefix.push(b'/');
    }
    return absolute.strip_prefix(root_prefix.as_slice()).map(<[u8]>::to_vec);
}

/// What:
///  The paths the index holds that the given pathspecs match,
///  or every path of it when there are none.
///  `&Path` borrows the directory Git runs the command in.
/// Why:
///  A restore without a source writes the index's content over the worktree,
///  and a query that failed is a failure rather than an empty answer.
///  A pathspec of the form means the directory the command runs in,
///  so the query starts there and the answers are rooted there;
///  a listing with no pathspec is scoped to that directory unless `:(top)` is written,
///  and every guarded form acts on the whole worktree.
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
    let mut paths: Vec<Vec<u8>> = nul_separated_paths(output.stdout.as_slice());
    if pathspecs.is_empty() || paths.is_empty() {
        return Ok(paths);
    }
    // A path form's answers are relative to the directory Git runs in, and the entries in the way are named from the
    //  worktree root, so that directory's own prefix is written before each of them.
    let mut rooted: Vec<Vec<u8>> = Vec::<Vec<u8>>::with_capacity(paths.len());
    for path in paths.drain(..) {
        rooted.push(rooted_at_the_run_directory(facts, run_in, path.as_slice())?);
    }
    return Ok(rooted);
}
