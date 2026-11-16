//! What:
//!  The one proof that a folder is the Git directory of the command,
//!  resolved so that no component of its path is a symbolic link.
//! Why:
//!  The lock pass opens and unlinks files,
//!  and it may do that only inside a Git directory Git itself would accept:
//!  a folder an option named,
//!  or one a walk reached through a link,
//!  is no repository,
//!  and cleaning it deleted a person's file from a folder that is not one
//!  (a review of `551084db3c`,
//! findings 2 and 3).
//!  So the proof is taken once,
//!  before anything is judged,
//!  and handed to the pass as an argument:
//!  a caller with no proof has no pass.
//!  A link above the Git directory is resolved rather than refused,
//!  because that is where the repository really is;
//!  what the proof guarantees is that the path it gives has no link left in it,
//!  so every folder below it can be opened one component at a time without one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proven = proveGitDirectory(gitDir, commonDir); // realpath both, then Git's own signatures, or undefined
//! ```

/// Git's own test of a Git directory:
///  a `HEAD` that names a reference or an object,
///  and either an object store and a reference folder or a `commondir` file.
use std::path::{Path, PathBuf};

/// The longest `HEAD` this reads.
const HEAD_LIMIT: u64 = 1 << 16;

/// What:
///  A Git directory this command may have its locks cleared in,
///  and the common Git directory its records and shared references live in,
///  each with no link in its path.
/// Why:
///  Every root the pass derives is built from these two folders,
///  so a proof here is what makes a candidate reachable only inside a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProvenGitDirectory = { gitDir: string; commonDir: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenGitDirectory {
    /// The Git directory,
    ///  resolved so that no component is a link.
    git_dir: PathBuf,
    /// The common Git directory,
    ///  resolved the same way.
    common_dir: PathBuf,
}

/// What:
///  Prove that `git_dir` is a Git directory of this machine's Git,
///  or answer nothing when it is not one.
/// Why:
///   The folder is resolved first,
///  so a link anywhere in the path is followed once,
///  and from then on the path has none;
///  then Git's own signatures decide:
///  a `HEAD` that names a reference or an object,
///  and an object store with a reference folder,
///  or the `commondir` file a linked worktree's Git directory carries.
///  A folder that fails this is no repository,
///  and the pass and the record folder both stay out of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proven = (gitDir, commonDir) => { const real = realpathSync(gitDir); const shared = realpathSync(commonDir); return isGitDirectory(real) && isGitDirectory(shared) ? { gitDir: real, commonDir: shared } : undefined };
/// ```
pub fn prove_git_directory(git_dir: &Path, common_dir: &Path, object_store: Option<&Path>) -> Option<ProvenGitDirectory> {
    // What: Both folders are the ones the command's one resolver answered, resolved to real paths, and both are Git
    //       directories by Git's own test.
    // Why:  The common directory is never derived a second time here: with `GIT_COMMON_DIR` the resolver's answer is the
    //       one the records were written in, and a proof that read `commondir` itself would answer a folder the writer
    //       never used (a review of `4e9b9fff1d`, finding 1).
    let resolved: PathBuf = std::fs::canonicalize(git_dir).ok()?;
    let common: PathBuf = std::fs::canonicalize(common_dir).ok()?;
    // What: The object store is the one the command's resolver answered, `common/objects` when nothing moved it.
    // Why:  `GIT_OBJECT_DIRECTORY` puts the objects elsewhere, and a detached `HEAD` whose object lives only there is a
    //       repository Git works in: asking the wrong store would answer "no such object" and switch clearing off
    //       (a review of `faf314ba2f`,
    // finding 2).
    let objects: PathBuf = match object_store {
        Some(store) => std::fs::canonicalize(store).unwrap_or_else(|_unresolvable| { return store.to_path_buf(); }),
        None => common.join("objects"),
    };
    if !is_a_git_directory(resolved.as_path(), objects.as_path()) || !is_a_git_directory(common.as_path(), objects.as_path()) {
        return None;
    }
    return Some(ProvenGitDirectory { git_dir: resolved, common_dir: common });
}

/// What:
///  Whether Git would find a repository at `directory`,
///  whose objects are in `object_store`:
///  a `HEAD` Git's own repository check accepts,
///  and an object store with a reference folder,
///  or a `commondir` file.
/// Why:
///   This is `setup.c`'s own list (`is_git_directory`),
///  read from the files and never from a name,
///  so a folder of a person's that merely sits where a Git directory would cannot be proven a repository.
///  The `HEAD` reading is this module's own, and it answers three ways:
///  a reference,
///  a detached object name,
///  or no repository at all.
///  A detached name is looked up in the object store without opening anything that is not a regular file,
///  because a pipe at a pack's name held every command before Git started
///  (a review of `faf314ba2f`,
/// finding 1);
///  and where the store cannot be read that way the proof stands aside as `main` did,
///  because refusing a repository Git works in switches the clearing of a killed Git's lock off
///  (the same review,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isGitDirectory = (dir, objectStore) => headAnswer(dir + '/HEAD', objectStore) !== 'not-a-repository' && ((isDir(dir + '/objects') && isDir(dir + '/refs')) || isFile(dir + '/commondir'));
/// ```
pub fn is_a_git_directory(directory: &Path, object_store: &Path) -> bool {
    let head: HeadAnswer = head_answer(directory.join("HEAD").as_path(), object_store);
    if matches!(head, HeadAnswer::NotARepository) {
        return false;
    }
    // What: The object store and the reference folder are read as Git reads them, a link followed once at the last
    //       component.
    // Why:  This answers whether Git accepts the folder, and Git follows those two names;
    //       that no link is ever opened belongs to the walk of the roots
    //       (`abandoned_locks.rs`,
    // `folder_below_proven`), and that no root of the proof's own path is a link belongs to the resolution above.
    let shared: bool = directory.join("objects").is_dir() && directory.join("refs").is_dir();
    let linked: bool = matches!(std::fs::symlink_metadata(directory.join("commondir")).map(|metadata| { return metadata.file_type(); }), Ok(kind) if kind.is_file());
    return shared || linked;
}

/// What:
///  What a `HEAD` file says a folder is,
///  in the three answers Git's own repository check gives.
/// Why:
///  The first two are repositories and the third is none,
///  and the difference decides whether the pass runs,
///  whether a record is written,
///  and whether a lock a killed Git left is ever cleared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeadAnswer = { kind: 'reference' } | { kind: 'detached'; hex: string } | { kind: 'not-a-repository' };
/// ```
enum HeadAnswer {
    /// `ref: <a name under refs/>`, which Git's check accepts.
    Reference,
    /// A lone object name, whose object must be in the store.
    ///  The name is kept for the words a caller may say about it.
    Detached(#[expect(dead_code, reason = "the name is what a caller may report; the answer itself is the variant")] String),
    /// Everything else: no `ref: ` line, a name that does not start with `refs/`, or nothing Git reads.
    NotARepository,
}

/// What:
///  What `HEAD` says about the folder,
///  read through a link at `HEAD` itself,
///  from a regular file only,
///  and with the object looked up for a detached name.
/// Why:
///   Git's own repository check reads `HEAD` and asks two things:
///  a `ref: ` line names something under `refs/`,
///  or the file is an object name.
///  It accepts `ref: refs/heads/.invalid` (the reftable backend's `HEAD`) and a `HEAD` that is a symbolic link,
///  and this reading must not refuse repositories Git works in
///  (a review of `faf314ba2f`,
/// finding 2).
///  It refuses `ref: ` followed by anything else, which is the NUL case a review of `b0ea691cc1` named.
///  A detached name is an object name only for the two lengths this reader knows,
///  in lowercase,
///  and its object must not be proved absent;
///  a store this cannot read that way answers unknown,
///  and unknown is a repository,
///  which is what `main` took every folder for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const headAnswer = (path, objectStore) => { const bytes = readThroughLinks(path, 65536); if (bytes.startsWith('ref: ')) return bytes.slice(5).startsWith('refs/') ? 'reference' : 'not-a-repository'; const hex = text(bytes).trimEnd(); return /^[0-9a-f]{40}([0-9a-f]{24})?$/.test(hex) && objectAnswer(objectStore, hex) !== 'absent' ? { kind: 'detached', hex } : 'not-a-repository' };
/// ```
fn head_answer(path: &Path, object_store: &Path) -> HeadAnswer {
    let bytes: Vec<u8> = match crate::plain_file::read_plain_file_through_links(path, HEAD_LIMIT) {
        Ok(read) => read,
        // A name a pipe, a folder or nothing at all stands at is no `HEAD` Git reads either.
        Err(_unreadable) => return HeadAnswer::NotARepository,
    };
    if let Some(named) = bytes.strip_prefix(b"ref: ".as_slice()) {
        if named.starts_with(b"refs/".as_slice()) {
            return HeadAnswer::Reference;
        }
        return HeadAnswer::NotARepository;
    }
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
    let trimmed: &str = text.trim_end_matches('\n');
    let lowercase_hexadecimal: bool = matches!(trimmed.len(), 40 | 64) && trimmed.bytes().all(|byte| { return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte); });
    if !lowercase_hexadecimal {
        return HeadAnswer::NotARepository;
    }
    return match object_answer(object_store, trimmed) {
        ObjectAnswer::Absent => HeadAnswer::NotARepository,
        ObjectAnswer::Present | ObjectAnswer::Unknown => HeadAnswer::Detached(trimmed.to_owned()),
    };
}

/// What:
///  What the object store says about one object name,
///  read without opening anything that is not a regular file:
///  the loose object itself,
///  then the pack indexes of the store and of its alternates,
///  never a pack.
/// Why:
///   A pipe at `objects/pack/pack-x.idx`,
///  or a pack index whose `.pack` is a pipe,
///  held every command for good when this lookup opened the pack
///  (a review of `faf314ba2f`,
/// finding 1).
///  An index answers from its own bytes,
///  so the pack is never opened,
///  and an index this reader cannot read makes the answer unknown rather than absent:
///  absent is what a repository Git refuses looks like
///  (`fatal: bad object HEAD`),
///  and unknown is a repository this proof stands aside on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const objectAnswer = (store, hex) => { if (openPlainFile(join(store, hex.slice(0, 2), hex.slice(2)))) return 'present'; for (const root of [store, ...alternates]) for (const idx of packIndexes(root)) { const a = indexNamesTheObject(idx, hex); if (a === 'names') return 'present'; if (a === 'unreadable') unknown = true } return unknown ? 'unknown' : 'absent' };
/// ```
fn object_answer(object_store: &Path, hex: &str) -> ObjectAnswer {
    let format: crate::repo_read_layout::ObjectFormat = match hex.len() {
        40 => crate::repo_read_layout::ObjectFormat::Sha1,
        64 => crate::repo_read_layout::ObjectFormat::Sha256,
        _ => return ObjectAnswer::Absent,
    };
    let Some(name) = crate::repo_read_object::from_hex(hex.as_bytes(), format) else {
        return ObjectAnswer::Absent;
    };
    let mut roots: Vec<PathBuf> = vec![object_store.to_path_buf()];
    let mut unknown: bool = false;
    object_roots(object_store, 0, &mut roots, &mut unknown);
    for root in &roots {
        // What: The loose name first: it is one plain-file open, and a program that put a pipe at it is unknown.
        // Why:  A packed object is the usual case after a clone, but a loose one needs no pack at all.
        let loose: PathBuf = root.join(hex.get(..2).unwrap_or_default()).join(hex.get(2..).unwrap_or_default());
        match crate::plain_file::open_plain_file(loose.as_path()) {
            Ok(_found) => return ObjectAnswer::Present,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_not_a_regular_file) => unknown = true,
        }
        let pack: PathBuf = root.join("pack");
        match std::fs::read_dir(pack.as_path()) {
            Ok(listing) => {
                for entry in listing.flatten() {
                    let path: PathBuf = entry.path();
                    if path.extension() != Some(std::ffi::OsStr::new("idx")) {
                        continue;
                    }
                    match crate::repo_read_pack::index_names_the_object(path.as_path(), name.as_slice(), name.len()) {
                        crate::repo_read_pack::IndexAnswer::Names => return ObjectAnswer::Present,
                        crate::repo_read_pack::IndexAnswer::DoesNotName => {}
                        crate::repo_read_pack::IndexAnswer::Unreadable => unknown = true,
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_unreadable) => unknown = true,
        }
    }
    return match unknown {
        true => ObjectAnswer::Unknown,
        false => ObjectAnswer::Absent,
    };
}

/// What the object store's answer about one name is.
enum ObjectAnswer {
    /// A loose object or an index names it.
    Present,
    /// Every store read said no.
    Absent,
    /// A part of the store could not be read without opening something that is not a regular file.
    Unknown,
}

/// What:
///  Every object store this repository reads through `alternates` files,
///  under the same rules Git uses with one allowance:
///  an entry that cannot be resolved is skipped,
///  and text that cannot be parsed makes the answer unknown.
/// Why:
///   A stale or quoted entry is one Git warns about and works past,
///  so it may not turn the answer into absent
///  (a review of `faf314ba2f`,
/// finding 2);
///  a file this cannot read at all leaves the answer unknown,
///  which keeps the lock pass running as `main`'s did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alternates = (root, depth, roots, unknown) => { const text = readThroughLinks(join(root, 'info/alternates')); for (const line of text.lines()) { const target = canonicalize(resolve(root, stripQuotes(line))); target ? (roots.push(target), alternates(target, depth + 1, roots, unknown)) : (unparseable ? unknown = true : undefined) } };
/// ```
fn object_roots(root: &Path, depth: usize, roots: &mut Vec<PathBuf>, unknown: &mut bool) {
    /// The deepest alternates chain this follows.
    const ALTERNATES_DEPTH: usize = 5;
    /// The longest alternates file this reads.
    const ALTERNATES_LIMIT: u64 = 1 << 16;
    /// The most stores one lookup reads.
    const ROOTS_LIMIT: usize = 64;
    if depth >= ALTERNATES_DEPTH || roots.len() >= ROOTS_LIMIT {
        return;
    }
    let text: String = match crate::plain_file::read_plain_file_through_links(root.join("info/alternates").as_path(), ALTERNATES_LIMIT) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_not_text) => {
                *unknown = true;
                return;
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(_unreadable) => {
            *unknown = true;
            return;
        }
    };
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let unquoted: &str = trimmed.strip_prefix('"').and_then(|rest| { return rest.strip_suffix('"'); }).unwrap_or(trimmed);
        if unquoted.contains('\\') || unquoted.is_empty() {
            // Git reads a quoted entry with C escapes; a line this cannot parse is one Git warns about.
            continue;
        }
        let named: PathBuf = PathBuf::from(unquoted);
        let candidate: PathBuf = match named.is_absolute() {
            true => named,
            false => root.join(named),
        };
        let Ok(resolved) = std::fs::canonicalize(candidate.as_path()) else {
            // The clone it names is gone: Git warns and works on with the stores it does have.
            continue;
        };
        if roots.contains(&resolved) || roots.len() >= ROOTS_LIMIT {
            continue;
        }
        roots.push(resolved.clone());
        object_roots(resolved.as_path(), depth + 1, roots, unknown);
    }
}

/// What:
///  The Git directory and the common one, for a caller that proves nothing itself.
/// Why:
///   A control reads the two folders a proof gives,
///  and nothing else may read them from here:
///  the pass takes the proof, not the paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proven.path(); proven.common()
/// ```
impl ProvenGitDirectory {
    /// The Git directory, with no link in its path.
    pub fn path(&self) -> &Path {
        return self.git_dir.as_path();
    }

    /// The common Git directory, with no link in its path.
    pub fn common(&self) -> &Path {
        return self.common_dir.as_path();
    }
}
