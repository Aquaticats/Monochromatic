//! What:
//!  The tracked edits the index's own `assume-unchanged` mark hides from every status Git answers.
//! Why:
//!  `git status`,
//!  `git diff-files`,
//!  `git diff-index` and `git ls-files --modified` all read a marked entry as "do not compare the
//!  worktree copy",
//!  so an edited path that carries the mark is invisible to the question the guard asks,
//!  while `reset --hard`,
//!  a forced `checkout` or `switch`,
//!  a path `checkout` or `restore`,
//!  `read-tree --reset -u`,
//!  `checkout-index -f -a` and `rm -f` overwrite it anyway
//!  (`oneway_merge` with `CE_MATCH_IGNORE_VALID`).
//!  A marked edit is a tracked edit like any other,
//!  so it gets exactly what the same edit gets without the mark,
//!  which is what reading it here restores.
//!  `skip-worktree` is read only when the caller asks:
//!  Git keeps that file for a whole-tree write (`reset --hard`, `checkout --force`),
//!  and writes over it for a form that names paths
//!  (`checkout <tree> -- <path>`, `restore --source <tree> -- <path>`),
//!  measured on this host with native Git 2.56.0.
//!  The marks are listed once,
//!  and the worktree entries of those paths alone are compared,
//!  so a worktree that marks thousands of files costs one listing and one stat each,
//!  and only a stat that differs costs one more query.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // editsTheStatusHides(facts) === ['a.txt']  when a.txt is marked and its bytes differ from the index
//! ```

/// The provider whose Git,
///  global options and environment answer the queries.
use super::repository_facts::GitFacts;
/// The captured-query runner's output type,
///  and the read ceiling every query has.
use crate::git_metadata::MetadataOutput;
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// The mark `git ls-files -v` prints for an `assume-unchanged` entry:
/// its tag is the lower-case letter of the entry's state,
/// and the `s` that means both marks is an `assume-unchanged` entry as well.
/// Why:
///   `-v` lower-cases the tag of exactly the entries Git was told not to look at,
///  and every other entry keeps an upper-case tag.
///  The tag `s` is a path marked both `assume-unchanged` and `skip-worktree`:
///  Git's own status hides it for the skip-worktree mark, and the forms that write over a marked edit are
///  those that reach a skip-worktree entry, so reading it as an `assume-unchanged` edit alone is what made the
///  guard see an edit where Git kept the file (the eleventh review's seventh finding).
///  The caller's scope decides: the assume-unchanged reading takes the plain lower-case letters, and the
///  skip-worktree reading takes `S` and `s`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isMarked = (tag: string, skipWorktreeToo: boolean) => skipWorktreeToo ? tag === 'S' || tag === 's' : tag.length === 1 && tag === tag.toLowerCase() && tag !== 's';
/// ```
fn is_marked(tag: &[u8], skip_worktree_too: bool) -> bool {
    if tag.len() != 1 {
        return false;
    }
    return if skip_worktree_too {
        // Every mark Git hides from the status: an `assume-unchanged` entry (`h` and its kind) and a
        //  `skip-worktree` one (`S`, or `s` for both marks).
        tag[0].is_ascii_lowercase() || tag[0] == b'S'
    } else {
        // The `assume-unchanged` reading alone: a tag that says skip-worktree as well is left to the reading
        //  that asks about skip-worktree entries.
        tag[0].is_ascii_lowercase() && tag[0] != b's'
    };
}

/// What:
///  The paths whose index entry carries the `assume-unchanged` mark and whose worktree entry
///  no longer holds what the index records.
///  `Result<Vec<String>, String>` is "the paths,
///  or why Git could not give one".
/// Why:
///   A listing that failed says nothing,
///  and a safety check never reads a failure as "no marked edit";
///  a marked path with no worktree file is no edit of the person's,
///  because Git writes that file back.
///  The bytes of every marked file are compared, not its size, time or mode: a same-length edit with
///  its modification time put back passed a reading that trusted those, and the bytes are the only
///  proof (the seventh review's reproduction, measured on this host with native Git 2.56.0,
///  2026-11-15).
///  The stat answers only what kind of entry stands there, and the comparison is one Git process per
///  batch of names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function editsTheStatusHides(facts, worktreeRoot): Promise<string[]>; // throws
/// ```
pub(super) fn edits_the_status_hides(
    facts: &mut GitFacts,
    worktree_root: &Path,
    skip_worktree_too: bool,
) -> Result<Vec<String>, String> {
    let marked: Vec<Vec<u8>> = marked_paths(facts, skip_worktree_too)?;
    if marked.is_empty() {
        return Ok(Vec::<String>::new());
    }
    let mut edited: Vec<String> = Vec::<String>::new();
    let mut to_compare: Vec<MarkedEntry> = Vec::<MarkedEntry>::new();
    for entry in marked_entries(facts, marked.as_slice())? {
        let named_path = |held: &MarkedEntry| return String::from_utf8_lossy(held.path.as_slice()).into_owned();
        match (entry.kind, what_the_worktree_holds(worktree_root, &entry)?) {
            // A path that is gone is no edit of the person's: Git writes it back.
            (_, Held::NotThere) => {}
            // A gitlink is a submodule: its worktree is a directory of another repository, which this reading
            //  does not judge and the submodule reading does, so it is no edit here whatever stands there
            //  (the eleventh review's seventh finding: an `assume-unchanged` submodule was refused).
            (IndexKind::Submodule, _) => {}
            // The index holds a link and the worktree a file, or the other way round: a change of type is
            //  the person's row, whatever the bytes of either entry are
            //  (a link replaced by a file holding the link's target text hashes to the same object as the
            //  link, and the comparison of content alone read it as unchanged: the eleventh review's sixth
            //  finding).
            //  A repository that stores no link as a link (`core.symlinks=false`) writes the target's text into a
            //  plain file and reads it back the same way, so the two kinds are one entry there and its bytes are
            //  what tells an edit.
            (IndexKind::Link, Held::Regular { .. }) if symlinks_are_stored(facts, worktree_root)? => {
                edited.push(named_path(&entry))
            }
            (IndexKind::Link, Held::Regular { .. }) => to_compare.push(entry),
            (IndexKind::File { .. }, Held::Link) | (_, Held::Other) => edited.push(named_path(&entry)),
            (IndexKind::Link, Held::Link) => to_compare.push(entry),
            (IndexKind::File { executable }, Held::Regular { executable: now }) => {
                // A change of mode is the person's data like a change of bytes, and Git records an
                //  executable file by the owner's bit alone, so that bit is the one compared: `chmod 0655`
                //  on a `0755` file leaves Git's mode at `100644`, and a reading that counted group and
                //  other too read it as executable like the index and lost the change
                //  (the eleventh review's fourth finding, measured on this host with native Git 2.56.0,
                //  2026-11-16).
                //  A repository that records no file mode keeps the worktree's own bits, so the question is
                //  asked only where the bit differs and only of a repository that reads it
                //  (with `core.fileMode=false` the bit survived both a restore and a hard reset in the same
                //  measurement).
                if executable != now && file_mode_is_recorded(facts, worktree_root)? {
                    edited.push(named_path(&entry));
                    continue;
                }
                to_compare.push(entry);
            }
        }
    }
    if to_compare.is_empty() {
        return Ok(edited);
    }
    for (entry, object) in to_compare
        .iter()
        .zip(worktree_objects(facts, worktree_root, to_compare.as_slice())?.iter())
    {
        match object {
            Some(found) if found == &entry.object => {}
            // A hash that differs, and a name this process cannot spell, are both an edit.
            _ => edited.push(String::from_utf8_lossy(entry.path.as_slice()).into_owned()),
        }
    }
    return Ok(edited);
}

/// What:
///  What the index records a marked path as.
///  An `enum` is a closed set of named alternatives.
/// Why:
///  Git's comparison of a worktree entry with the index is by kind as well as by bytes:
///  a link replaced by a regular file is a change of type even when the file's bytes are the link's own
///  text, which hashes to the same object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexKind = { kind: 'file'; executable: boolean } | { kind: 'link' };
/// ```
#[derive(Clone, Copy, Debug)]
enum IndexKind {
    /// A regular file, executable by Git's owner bit or not.
    File {
        /// Whether the index's mode is `100755`, which is the owner's execute bit set.
        executable: bool,
    },
    /// A symbolic link, mode `120000`.
    Link,
    /// A submodule, mode `160000`: another repository's worktree, which this reading does not judge.
    Submodule,
}

/// What:
///  What the worktree holds at one marked entry's path.
///  An `enum` is a closed set of named alternatives.
/// Why:
///  The comparison needs a regular file to read;
///  a link is an entry of its own (Git writes the target's bytes, never the link), a directory where
///  the index holds a file is an edit, and a path that is gone is no edit of the person's because Git
///  writes that file back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Held = 'not-there' | 'not-a-file' | 'a-file';
/// ```
#[derive(Debug)]
enum Held {
    /// Nothing stands there.
    NotThere,
    /// A regular file.
    Regular {
        /// Whether the file carries Git's executable bit, which is the owner's.
        executable: bool,
    },
    /// A symbolic link.
    Link,
    /// A directory or another entry the index never records.
    Other,
}

/// What:
///  What the worktree holds at one marked entry's path, read under the worktree lookup ceiling.
///  `Result<Held, String>` is "the answer, or why it could not be read".
/// Why:
///  A metadata lookup on a mount whose server stopped answering waits in the kernel with no end,
///  so it runs on a thread this process waits for only until the ceiling;
///  no answer refuses the command rather than reading as "no edit", because a form that overwrites
///  the marked path would lose what the mark hides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatTheWorktreeHolds(facts, root, entry): Held; // throws
/// ```
fn what_the_worktree_holds(root: &Path, entry: &MarkedEntry) -> Result<Held, String> {
    let Some(relative) = relative_path(entry.path.as_slice()) else {
        // A name this process cannot spell is no name to read bytes from, so it is compared and fails;
        //  its kind and mode are the index's own, so neither question finds anything there.
        return Ok(match entry.kind {
            IndexKind::Link => Held::Link,
            IndexKind::Submodule => Held::Other,
            IndexKind::File { executable } => Held::Regular { executable },
        });
    };
    let location: PathBuf = root.join(relative.as_path());
    let Some(read) = crate::metadata_of_a_path_within_a_ceiling::symlink_metadata_within(crate::wait_ceilings::WORKTREE_LOOKUP_CEILING, location.as_path()) else {
        return Err(format!(
            "Git could not read `{}` within {} seconds, so the guard cannot tell whether the edit the index marks assume-unchanged is still there",
            relative.display(),
            crate::wait_ceilings::WORKTREE_LOOKUP_CEILING.as_secs()
        ));
    };
    let metadata: std::fs::Metadata = match read {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Held::NotThere),
        Err(error) => {
            return Err(format!(
                "Git could not read `{}`, so the guard cannot tell whether the edit the index marks assume-unchanged is still there: {error}",
                relative.display()
            ));
        }
    };
    if metadata.file_type().is_symlink() {
        return Ok(Held::Link);
    }
    if !metadata.file_type().is_file() {
        return Ok(Held::Other);
    }
    return Ok(Held::Regular { executable: carries_an_executable_bit(&metadata) });
}

/// What:
///  Whether the worktree entry carries an executable bit, the way Git records one.
///  `&std::fs::Metadata` borrows the entry's own metadata.
/// Why:
///  Git records a mode of `100755` for an executable file and `100644` for one that is not,
///  and the comparison against the index's mode is what tells an edit of the bit from an unchanged file.
///  A platform without the bit answers that nothing carries it, which matches what Git records there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // carriesAnExecutableBit(metadata) === (metadata.mode & 0o100) !== 0
/// ```
fn carries_an_executable_bit(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Git's `ce_mode_from_stat` looks at the owner's execute bit alone (`S_IXUSR`):
        //  a `0655` file is no executable to Git even though its group and other bits are set.
        return metadata.permissions().mode() & 0o100 != 0;
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        return false;
    }
}

/// What:
///  Whether this repository records the executable bit of a file,
///  which is `core.fileMode` and true unless the configuration says otherwise.
///  `Result<bool, String>` is "the answer, or why Git could not give one".
/// Why:
///  With `core.fileMode=false` Git keeps the worktree's own bits,
///  so a bit that differs from the index's is no loss there and refusing it would refuse a command
///  Git runs safely
///  (measured on this host with native Git 2.56.0, 2026-11-16: the bit survived a `restore` and a
///  hard reset with the setting false, and was removed with it true).
///  The question is asked only where a bit already differs, so a worktree with no mode edit pays no
///  process for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // fileModeIsRecorded(root) === (await git(['-C', root, 'config', '--type=bool', '--get', 'core.fileMode'])).stdout.trim() !== 'false'
/// ```
fn file_mode_is_recorded(facts: &mut GitFacts, worktree_root: &Path) -> Result<bool, String> {
    let mut arguments: Vec<OsString> = crate::global_arguments::without_directory_changes(facts.global_prefix.as_slice());
    for token in ["--no-optional-locks", "config", "--type=bool", "--get", "core.filemode"] {
        arguments.push(OsString::from(token));
    }
    // The setting belongs to the repository the command acts on, and `-C` moved that command to another
    //  directory: asking from the wrapper's own directory read another repository's setting, which let a
    //  mode edit through when that repository had the bit off and refused one when it had the bit on
    //  (the eleventh review's fourth finding, both halves).
    let output: MetadataOutput = facts.query_in(worktree_root, arguments, "whether this repository records the executable bit")?;
    // A setting that is not there is Git's own default, which records the bit.
    if !output.success {
        return Ok(true);
    }
    let answer: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    return Ok(answer != "false");
}

/// What:
///  Whether this repository stores a link as a link, which is `core.symlinks` and true unless the
///  configuration says otherwise.
///  `Result<bool, String>` is "the answer, or why Git could not give one".
/// Why:
///  With `core.symlinks=false` Git writes a link as a plain file holding the target's text, and its own
///  comparison sees the same kind on both sides, so a link in the index and a regular file in the worktree
///  is no edit there and refusing it refused a command Git runs safely
///  (the eleventh review's seventh finding, measured with an unmodified link stored as a plain file:
///  `git reset --hard` is a no-op and the guard refused it).
///  The question is asked only where the two kinds already differ, so a worktree with no such entry pays no
///  process for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // symlinksAreStored(root) === (await git(['-C', root, 'config', '--type=bool', '--get', 'core.symlinks'])).stdout.trim() !== 'false'
/// ```
fn symlinks_are_stored(facts: &mut GitFacts, worktree_root: &Path) -> Result<bool, String> {
    let mut arguments: Vec<OsString> = crate::global_arguments::without_directory_changes(facts.global_prefix.as_slice());
    for token in ["--no-optional-locks", "config", "--type=bool", "--get", "core.symlinks"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query_in(worktree_root, arguments, "whether this repository stores links as links")?;
    // A setting that is not there is Git's own default, which stores them.
    if !output.success {
        return Ok(true);
    }
    let answer: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    return Ok(answer != "false");
}

/// What:
///  The paths `git ls-files -v` reports with a mark that hides them from status.
/// Why:
///   One listing names every marked entry,
///  and the marks of the paths nothing edits cost nothing more.
///  A path marked both ways prints the tag `s`: `assume-unchanged` alone is what the tag's letter case
///  tells, and the skip-worktree mark is only read when the caller asked for it, so an entry Git hides from
///  the status for its skip-worktree mark alone is not read here as an `assume-unchanged` edit
///  (the eleventh review's seventh finding: `git reset --hard` keeps such a file in real Git).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markedPaths(facts, skipWorktreeToo): Promise<Buffer[]>; // throws
/// ```
fn marked_paths(facts: &mut GitFacts, skip_worktree_too: bool) -> Result<Vec<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    for token in ["--no-optional-locks", "--no-literal-pathspecs", "ls-files", "-v", "-z", "--full-name", "--"] {
        arguments.push(OsString::from(token));
    }
    // `:(top)` asks from the top of the worktree: a listing without a pathspec is scoped to the directory Git runs in,
    //  while an edit to a marked path anywhere in the worktree is one a discarding form would lose.
    arguments.push(OsString::from(":(top)"));
    let output: MetadataOutput = facts.query(arguments, "the paths the index marks assume-unchanged")?;
    if !output.success {
        return Err(failed(&output, "the paths the index marks assume-unchanged"));
    }
    let mut marked: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for entry in output.stdout.split(|byte| return *byte == 0) {
        // Each entry is `<tag> <path>`; a tag of one lower-case letter is the mark.
        let Some(space) = entry.iter().position(|byte| return *byte == b' ') else {
            continue;
        };
        if !is_marked(&entry[..space], skip_worktree_too) {
            continue;
        }
        marked.push(entry[space + 1..].to_vec());
    }
    return Ok(marked);
}

/// What:
///  One marked entry as the index records it:
///  its mode,
///  the object its bytes make,
///  and its path.
/// Why:
///   The object is what the worktree copy is compared with,
///  and the mode and stage are read only to know the entry is the index's own record of the path
///  at stage zero.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MarkedEntry = { mode: string; object: string; path: Buffer; };
/// ```
struct MarkedEntry {
    /// The object the index holds for the path.
    object: String,
    /// What the index records the path as:
    ///  a regular file with an executable bit or without one,
    ///  or a link.
    kind: IndexKind,
    /// The path,
    ///  as the index holds it.
    path: Vec<u8>,
}

/// What:
///  The index's own record of each marked path,
///  asked for those paths alone.
/// Why:
///   A query that failed says nothing about them and is a failure.
///  The listing that named these paths was asked from the worktree's top (`:(top)`),
///  so they are root-relative,
///  while a plain pathspec is matched from the directory Git runs in:
///  from a subdirectory `a.txt` reaches `sub/a.txt` or nothing,
///  and the marked entry in the way is then silently absent from this answer
///  (a `reset --hard` run in `sub` reverted an `assume-unchanged` edit at the top,
///  measured on this host with native Git 2.56.0,
///  2026-11-14).
///  Each path is therefore given back to Git rooted at the top,
///  and matched literally, which is the name the index holds.
///  The stat Git kept is not asked for: the bytes of every marked entry are compared, because equal
///  size, modification time and mode do not make equal content
///  (the seventh review's same-length edit with its time put back was invisible to a reading that
///  trusted the stat).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markedEntries(facts, marked: Buffer[]): Promise<MarkedEntry[]>; // throws
/// ```
fn marked_entries(facts: &mut GitFacts, marked: &[Vec<u8>]) -> Result<Vec<MarkedEntry>, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.clone();
    // The pathspecs below are the guard's own `:(top,literal)` magic, and a caller's
    //  `--literal-pathspecs`, or `GIT_LITERAL_PATHSPECS`, would turn that magic into a file name that
    //  matches nothing:
    //  the answer would be empty, the edit the mark hides would go unseen, and a restore would write
    //  over it (the ninth review's second finding).
    for token in ["--no-optional-locks", "--no-literal-pathspecs", "ls-files", "-v", "--stage", "-z", "--full-name", "--"] {
        arguments.push(OsString::from(token));
    }
    for path in marked {
        arguments.push(path_argument(rooted_at_the_top(path).as_slice()));
    }
    let output: MetadataOutput = facts.query(arguments, "the index's own record of the marked paths")?;
    if !output.success {
        return Err(failed(&output, "the index's own record of the marked paths"));
    }
    return Ok(parse_marked_entries(output.stdout.as_slice()));
}

/// What:
///  One worktree-relative path as the pathspec Git matches from the top of the worktree,
///  literally.
///  `&[u8]` borrows the path, `Vec<u8>` owns the pathspec.
/// Why:
///  `:(top,literal)` is Git's own spelling for "this name, from the top of the worktree,
///  with no pattern matched":
///  a path listed with `--full-name` and `:(top)` is relative to the worktree root,
///  and asking about it from a subdirectory reaches another file or nothing.
///  `literal` keeps a name holding `*`, `?` or `[` a name and no pattern.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rooted = (path) => Buffer.concat([Buffer.from(':(top,literal)'), path]);
/// ```
fn rooted_at_the_top(path: &[u8]) -> Vec<u8> {
    let mut pathspec: Vec<u8> = Vec::<u8>::with_capacity(TOP_LITERAL_MAGIC.len() + path.len());
    pathspec.extend_from_slice(TOP_LITERAL_MAGIC);
    pathspec.extend_from_slice(path);
    return pathspec;
}

/// Git's pathspec magic for a name that is matched literally from the top of the worktree.
/// Why:
///  One copy of the spelling keeps every such pathspec the same,
///  and it is Git's own syntax rather than a string built here.
const TOP_LITERAL_MAGIC: &[u8] = b":(top,literal)";

/// What:
///  The entries one `git ls-files -v --stage -z` answer holds.
/// Why:
///   Git writes each entry as `<tag> <mode> <object> <stage>\t<path>` and a NUL after it,
///  so a NUL is the boundary and a path holding a newline stays whole;
///  an entry that names no path, or one that is not at stage zero, is not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMarkedEntries(stdout: Buffer): MarkedEntry[];
/// ```
fn parse_marked_entries(stdout: &[u8]) -> Vec<MarkedEntry> {
    let mut entries: Vec<MarkedEntry> = Vec::<MarkedEntry>::new();
    for chunk in stdout.split(|byte| return *byte == 0) {
        let Some(entry) = entry_of(chunk) else {
            continue;
        };
        entries.push(entry);
    }
    return entries;
}

/// What:
///  One entry of that answer,
///  or nothing when the bytes are not one.
///  `&[u8]` borrows the chunk.
/// Why:
///   A chunk that names no path or no stage is not an entry,
///  and reading it as one would compare the wrong file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryOf(chunk: Buffer): MarkedEntry | undefined;
/// ```
fn entry_of(chunk: &[u8]) -> Option<MarkedEntry> {
    let tab: usize = chunk.iter().position(|byte| return *byte == b'\t')?;
    let fields: Vec<&[u8]> = chunk[..tab].split(|byte| return *byte == b' ').collect();
    let [_tag, mode, object, stage] = fields.as_slice() else {
        return None;
    };
    if *stage != b"0" {
        return None;
    }
    return Some(MarkedEntry {
        object: String::from_utf8_lossy(object).into_owned(),
        // Git writes the mode it keeps: `100755` is an executable file, `100644` is not one, `120000` is a
        //  link and `160000` a submodule.
        kind: match *mode {
            b"120000" => IndexKind::Link,
            b"160000" => IndexKind::Submodule,
            _ => IndexKind::File { executable: *mode == b"100755" },
        },
        path: chunk[tab + 1..].to_vec(),
    });
}

/// What:
///  The objects the marked worktree files' bytes make,
///  read through the clean filter of the path each stands at,
///  in as few Git processes as the arguments allow.
///  `&Path` borrows the worktree root the files stand below,
///  `Option<String>` is "the object, or nothing for a name this process cannot spell".
/// Why:
///  Git records the filtered object in the index,
///  so an unfiltered hash would differ from it for every file a filter touches.
///  `git hash-object` applies each file's own filter from the name it is given
///  (measured on this host with native Git 2.56.0, 2026-11-15:
///  `hash-object -- f.txt` and `hash-object --path=f.txt -- f.txt` print the same object,
///  and the same file named after another rule prints another),
///  so a batch of names run from the worktree root hashes exactly what one call per file did.
///  A batch keeps the command's cost to a few Git processes: one per `HASH_BATCH_BYTES` of names,
///  where one call per entry costs a start each
///  (1,000 marked paths: 8.5 ms as one call, 1,787 ms as 1,000 calls, measured on this host,
///  2026-11-15).
///  A query that failed says nothing about the bytes and is a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeObjects(facts, root, entries): (string | undefined)[]; // throws
/// ```
fn worktree_objects(facts: &mut GitFacts, root: &Path, entries: &[MarkedEntry]) -> Result<Vec<Option<String>>, String> {
    let mut objects: Vec<Option<String>> = vec![None; entries.len()];
    let mut batch: Vec<usize> = Vec::<usize>::new();
    let mut bytes: usize = 0;
    for (index, entry) in entries.iter().enumerate() {
        if relative_path(entry.path.as_slice()).is_none() {
            // A name this process cannot spell has no argument, and stays `None`, which the caller reads as an edit.
            continue;
        }
        bytes += entry.path.len() + 1;
        batch.push(index);
        if bytes >= HASH_BATCH_BYTES {
            read_the_objects(facts, root, entries, batch.as_slice(), &mut objects)?;
            batch.clear();
            bytes = 0;
        }
    }
    if !batch.is_empty() {
        read_the_objects(facts, root, entries, batch.as_slice(), &mut objects)?;
    }
    return Ok(objects);
}

/// How many bytes of path names one `hash-object` call takes.
/// Why:
///  A batch of names is one command line, and a command line has its own limit
///  (`MAX_ARG_STRLEN` and `ARG_MAX` on this system);
///  a hundred thousand bytes of names leaves room for the environment and for any name's own length,
///  and costs one Git start per batch (about 2 ms measured on this host, 2026-11-15).
const HASH_BATCH_BYTES: usize = 100_000;

/// What:
///  Fill in the objects of one batch of marked entries,
///  asking Git once from the worktree root.
///  `&mut [Option<String>]` is the answer being filled, one slot per entry.
/// Why:
///   The names are relative to the worktree root, because the filter Git applies is chosen by the name
///  it is given and a root-relative name matches the repository's own attribute rules;
///  the query runs from the root for the same reason.
///  A Git that failed, or answered a different number of objects, says nothing about the bytes, so it
///  is a failure rather than an edit or no edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTheObjects(facts, root, entries, batch, objects): void; // throws
/// ```
fn read_the_objects(
    facts: &mut GitFacts,
    root: &Path,
    entries: &[MarkedEntry],
    batch: &[usize],
    objects: &mut [Option<String>],
) -> Result<(), String> {
    let mut arguments: Vec<OsString> = crate::global_arguments::without_directory_changes(facts.global_prefix.as_slice());
    for token in ["--no-optional-locks", "hash-object", "--"] {
        arguments.push(OsString::from(token));
    }
    for index in batch {
        arguments.push(path_argument(entries[*index].path.as_slice()));
    }
    let output: MetadataOutput = facts.query_in(root, arguments, "the objects of the worktree copies")?;
    if !output.success {
        return Err(failed(&output, "the objects of the worktree copies"));
    }
    let found: Vec<&[u8]> = output.stdout.split(|byte| return *byte == b'\n').filter(|line| return !line.is_empty()).collect();
    if found.len() != batch.len() {
        return Err(format!(
            "Git answered {} objects for the {} marked paths the guard asked about, so the guard cannot tell which of them the edit the index hides is at",
            found.len(),
            batch.len()
        ));
    }
    for (slot, object) in batch.iter().zip(found.iter()) {
        objects[*slot] = Some(String::from_utf8_lossy(object).trim().to_string());
    }
    return Ok(());
}

/// What:
///  One repository path as the filesystem path below the worktree root,
///  or nothing when its bytes are not a name this process can spell.
/// Why:
///   The stat has to name the file the index names,
///  and a name that cannot be spelled here is one whose stat is no evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativePath(bytes: Buffer): string | undefined;
/// ```
#[cfg(unix)]
fn relative_path(bytes: &[u8]) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let text: OsString = OsString::from_vec(bytes.to_vec());
    return Some(PathBuf::from(text));
}

/// What:
///  One repository path as the filesystem path below the worktree root.
/// Why:
///  No gate compiles this branch (the crate does not build for Windows; README, Requirements).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativePath(bytes: Buffer): string | undefined;
/// ```
#[cfg(not(unix))]
fn relative_path(bytes: &[u8]) -> Option<PathBuf> {
    let text: String = String::from_utf8(bytes.to_vec()).ok()?;
    return Some(PathBuf::from(text));
}

/// What:
///  One path as an argument Git reads exactly as it is.
/// Why:
///   A path is bytes and not text,
///  so a name that is not UTF-8 reaches Git unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argument = (bytes: Buffer): string => bytes;
/// ```
#[cfg(unix)]
fn path_argument(bytes: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStringExt;
    return OsString::from_vec(bytes.to_vec());
}

/// What:
///  One path as an argument Git reads exactly as it is.
/// Why:
///  No gate compiles this branch (the crate does not build for Windows; README, Requirements).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argument = (bytes: Buffer): string => String(bytes);
/// ```
#[cfg(not(unix))]
fn path_argument(bytes: &[u8]) -> OsString {
    return OsString::from(String::from_utf8_lossy(bytes).into_owned());
}

/// What:
///  One failed query as this fact's own failure.
/// Why:
///  A reading that could not be made says nothing about the worktree,
///  and the words that stopped it are what the person acts on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `Git could not read ${fact}: ${firstStderrLine}`;
/// ```
fn failed(output: &MetadataOutput, fact: &str) -> String {
    let detail: String = String::from_utf8_lossy(output.stderr.as_slice())
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    return format!("Git could not read {fact}: {detail}");
}

/// Reading the marks and comparing what the worktree holds with what the index records.
#[cfg(test)]
#[path = "repository_facts_edits_the_status_hides_tests.rs"]
mod tests;
