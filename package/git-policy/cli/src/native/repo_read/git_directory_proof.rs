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
pub fn prove_git_directory(git_dir: &Path, common_dir: &Path) -> Option<ProvenGitDirectory> {
    // What: Both folders are the ones the command's one resolver answered, resolved to real paths, and both are Git
    //       directories by Git's own test.
    // Why:  The common directory is never derived a second time here: with `GIT_COMMON_DIR` the resolver's answer is the
    //       one the records were written in, and a proof that read `commondir` itself would answer a folder the writer
    //       never used (a review of `4e9b9fff1d`, finding 1).
    let resolved: PathBuf = std::fs::canonicalize(git_dir).ok()?;
    let common: PathBuf = std::fs::canonicalize(common_dir).ok()?;
    if !is_a_git_directory(resolved.as_path(), common.as_path()) || !is_a_git_directory(common.as_path(), common.as_path()) {
        return None;
    }
    return Some(ProvenGitDirectory { git_dir: resolved, common_dir: common });
}

/// What:
///  Whether Git would find a repository at `directory`,
///  whose objects are in the store under `object_store`:
///  a `HEAD` this module reads whole,
///  and an object store with a reference folder,
///  or a `commondir` file.
/// Why:
///   This is `setup.c`'s own list (`is_git_directory`),
///  read from the files and never from a name,
///  so a folder of a person's that merely sits where a Git directory would cannot be proven a repository.
///  The `HEAD` reading is this module's own and refuses every spelling it cannot fully parse
///  (`head_names_a_reference_or_object`),
///  because Git later refuses such a folder itself and the pass must not open anything there first
///  (a review of `b0ea691cc1`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isGitDirectory = (dir, objectStore) => validHead(dir + '/HEAD', objectStore) && ((isDir(dir + '/objects') && isDir(dir + '/refs')) || isFile(dir + '/commondir'));
/// ```
pub fn is_a_git_directory(directory: &Path, object_store: &Path) -> bool {
    if !head_names_a_reference_or_object(directory.join("HEAD").as_path(), object_store) {
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
///  What a `HEAD` file names,
///  when its bytes are one of the two shapes a repository really has.
/// Why:
///   The question "is this a repository" is answered on these bytes alone,
///  so the shapes are spelled out rather than matched loosely:
///  a loose match accepted a symref carrying a NUL,
///  and Git refused the folder after the pass had already unlinked a file in it
///  (a review of `b0ea691cc1`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Head = { kind: 'symref'; name: string } | { kind: 'detached'; hex: string };
/// ```
enum Head {
    /// A `ref: <name>` line.
    Symref(String),
    /// A lone object name.
    Detached(String),
}

/// What:
///  Whether `HEAD` names a reference or an object that is really there,
///  which is the one file every Git directory has.
/// Why:
///   `HEAD` is either `ref: <name>` or an object's hexadecimal text,
///  and a folder with no such file is no repository to Git.
///  Everything else is refused:
///  the bytes must be one line,
///  the reference name must be a full name Git could store,
///  and a lone object name must be a lowercase hexadecimal name of this repository's length whose object the store
///  holds,
///  because Git refuses a folder whose detached `HEAD` names an object it does not have
///  (a review of `b0ea691cc1`,
/// finding 1).
///  A spelling Git accepts and this refuses costs the lock pass in that repository and nothing else,
///  so the reading errs towards refusing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const validHead = (path, objectStore) => { const head = parseHead(readText(path)); return head?.kind === 'symref' ? isFullRefname(head.name) : head?.kind === 'detached' && objectExists(objectStore, head.hex) };
/// ```
fn head_names_a_reference_or_object(path: &Path, object_store: &Path) -> bool {
    let bytes: Vec<u8> = match crate::plain_file::read_plain_file(path, HEAD_LIMIT) {
        Ok(read) => read,
        Err(_unreadable) => return false,
    };
    return match head_of(bytes.as_slice()) {
        Some(Head::Symref(name)) => is_a_full_refname(name.as_str()),
        Some(Head::Detached(hex)) => object_exists(object_store, hex.as_str()),
        None => false,
    };
}

/// What:
///  The shape `bytes` are,
///  or nothing when they are not exactly one of the two.
/// Why:
///   One line, an optional final newline,
///  no carriage return,
///  no second line:
///  anything else is text Git's own reader does not take as a name,
///  and a reader of this one must not take it either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const headOf = (bytes) => { const line = bytes.at(-1) === 0x0a ? bytes.subarray(0, -1) : bytes; if (line.includes(0x0a) || line.includes(0x0d)) return undefined; return line.startsWith('ref: ') ? { kind: 'symref', name: utf8(line.subarray(5)) } : { kind: 'detached', hex: utf8(line) } };
/// ```
fn head_of(bytes: &[u8]) -> Option<Head> {
    let line: &[u8] = match bytes.last() {
        Some(b'\n') => bytes.get(..bytes.len() - 1)?,
        _ => bytes,
    };
    if line.contains(&b'\n') || line.contains(&b'\r') {
        return None;
    }
    if let Some(named) = line.strip_prefix(b"ref: ") {
        let name: String = std::str::from_utf8(named).ok()?.to_owned();
        return Some(Head::Symref(name));
    }
    let hex: String = std::str::from_utf8(line).ok()?.to_owned();
    return Some(Head::Detached(hex));
}

/// What:
///  Whether `name` is a full reference name Git could store:
///  `refs/` and components of bytes Git allows,
///  none of them empty,
///  beginning with a dot,
///  or ending in `.lock`.
/// Why:
///   This is `check_refname_format`'s rule for a full name,
///  and it is what keeps the proof from accepting a name Git's own update would refuse:
///  a reference name cannot hold a space,
///  a control byte,
///  a NUL,
///  `..`,
///  `@{`,
///  or one of ``~^:?*[\``.
///  A one-level name such as `HEAD` is refused,
///  although Git accepts it there,
///  because Git's repository check accepts only `refs/`-prefixed text at this place
///  (a review of `b0ea691cc1`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isFullRefname = (name) => name.startsWith('refs/') && !/[\x00-\x20\x7f~^:?*[\\]/.test(name) && !name.includes('..') && !name.includes('@{') && !name.endsWith('/') && !name.endsWith('.') && name.split('/').every((part) => part && !part.startsWith('.') && !part.endsWith('.lock'));
/// ```
fn is_a_full_refname(name: &str) -> bool {
    if !name.starts_with("refs/") || name.ends_with('/') || name.ends_with('.') {
        return false;
    }
    if name.contains("..") || name.contains("@{") {
        return false;
    }
    for byte in name.bytes() {
        // What: Every byte Git allows in a reference name, and no other.
        // Why:  Git refuses a space, a control byte and 0x7f, and allows the rest, a name of another alphabet included;
        //       refusing a name Git allows would cost the pass in that repository and nothing else, but it would cost it
        //       on every command, so the rule follows Git's own.
        let allowed: bool = byte > 0x20 && byte != 0x7f;
        let special: bool = matches!(byte, b'~' | b'^' | b':' | b'?' | b'*' | b'[' | b'\\');
        if !allowed || special {
            return false;
        }
    }
    for component in name.split('/') {
        if component.is_empty() || component.starts_with('.') || component.ends_with(".lock") {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether the object store under `object_store` holds the object `hex` names.
/// Why:
///   Git refuses a folder whose detached `HEAD` names an object it cannot read
///  (`fatal: bad object HEAD`),
///  and the pass may not unlink anything there,
///  so the object is looked for before the folder is proven
///  (a review of `b0ea691cc1`,
/// finding 1).
///  A store this cannot open,
///  a name of another length,
///  and one that is not lowercase hexadecimal all answer no,
///  which keeps the lock and touches nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const objectExists = (objectStore, hex) => { const format = hex.length === 64 ? 'sha256' : hex.length === 40 ? 'sha1' : undefined; return format !== undefined && /^[0-9a-f]+$/.test(hex) && openOdb(objectStore, format)?.contains(fromHex(hex, format)) === true };
/// ```
fn object_exists(object_store: &Path, hex: &str) -> bool {
    let format: crate::repo_read_layout::ObjectFormat = match hex.len() {
        40 => crate::repo_read_layout::ObjectFormat::Sha1,
        64 => crate::repo_read_layout::ObjectFormat::Sha256,
        _ => return false,
    };
    if !hex.bytes().all(|byte| { return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte); }) {
        return false;
    }
    let Some(name) = crate::repo_read_object::from_hex(hex.as_bytes(), format) else {
        return false;
    };
    let Some(database) = crate::repo_read_odb::ObjectDatabase::open(object_store, None, format) else {
        return false;
    };
    return database.contains(name.as_slice());
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
