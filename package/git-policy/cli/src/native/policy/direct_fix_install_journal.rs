//! What:
//!  The durable record of one correction a fix is installing:
//!  a directory inside the Git directory that
//!       holds the corrected bytes and a record of what the file was,
//!  so a command that finds it after a kill can
//!       tell what happened and put the person's bytes back.
//! Why:
//!  An install replaces a file in one step and then has to judge what it displaced.
//!  A kill
//!       between those two steps leaves the newest version of a file under a name nobody would look
//!       for,
//!  and the person's only copy has to be found and restored by a later command rather than
//!       named in a document.
//!  The record is written before the file is touched,
//!  so the bytes it holds are
//!       never the only evidence.
//!  Nothing here lives in the worktree:
//!  Git never lists it,
//!  never stages it,
//! and no
//!       `gitignore` rule of the repository can make it visible.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const journal = await openJournal(gitDir, destination, kept, replacement);
//! // const displaced = await exchange(journal.content, destination);
//! // await journal.recoverInterrupted(ownerIsDead);
//! ```

/// The corrected bytes,
///  and what a later command needs to know about them.
use crate::direct_fix_install_writes::Kept;
/// One extended attribute of the file a correction belongs to.
use crate::direct_fix_install_attributes::Attribute;
/// The unique name part shared with the private index directories.
use crate::candidate_private_index::unique_suffix;
/// The digest that names a state of a file without holding it.
use crate::subvolume_sha::sha256;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};

/// The name prefix of every journal directory,
///  inside the Git directory.
pub const JOURNAL_PREFIX: &str = "cli-git-direct-fix-";

/// The file inside a journal directory that holds the corrected bytes,
///  and then the bytes the
///       install displaced.
pub const CONTENT_FILE: &str = "content";

/// The file that records what the file was and what the correction is.
pub const RECORD_FILE: &str = "record";

/// What a record holds where the platform could not name the writer.
/// Why:
///   A writer nothing can name cannot be proven gone,
///  so its journal is kept rather than judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NO_OWNER_TOKEN = 'none';
/// ```
pub const NO_OWNER_TOKEN: &str = "none";

/// What:
///  What one correction was made of,
///  as a later command can read it.
/// Why:
///   A command that finds a journal after a kill must tell whether the corrected bytes are still in
///       place (so the displaced file holds the bytes to put back),
///  whether the install never reached the file
///       (so nothing needs restoring),
///  and whether someone has written since (so nothing may be undone).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JournalRecord = { destination: string; mode: number; uid: number; gid: number; owner: string; replacement: string; attributes: Attribute[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct JournalRecord {
    /// The file the correction belongs to.
    pub(crate) destination: PathBuf,
    /// The permission bits the file had.
    pub(crate) mode: u32,
    /// The owner the file had.
    pub(crate) uid: u32,
    /// The group the file had.
    pub(crate) gid: u32,
    /// The writer's owner token,
    ///  for the proof that it still runs;
    ///  nothing where the platform could not name the writer.
    pub(crate) owner: Option<String>,
    /// The SHA-256 of the corrected bytes,
    ///  in lower-case hexadecimal.
    pub(crate) replacement: String,
    /// The extended attributes the file carried,
    ///  in name order;
    ///  a sweep that compared only bytes,
    ///  bits and owner
    ///       would take a file the person gave an attribute for the fix's own.
    pub(crate) attributes: Vec<Attribute>,
}

/// The hexadecimal text of `bytes`,
///  in lower case.
fn to_hex(bytes: &[u8]) -> String {
    let mut text: String = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push_str(format!("{byte:02x}").as_str());
    }
    return text;
}

/// The bytes a hexadecimal text names,
///  or nothing when it is not one.
fn from_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len() / 2);
    let digits: &[u8] = text.as_bytes();
    let mut position: usize = 0;
    while position < digits.len() {
        let pair: [u8; 2] = [digits[position], digits[position + 1]];
        bytes.push(u8::from_str_radix(std::str::from_utf8(&pair).ok()?, 16).ok()?);
        position += 2;
    }
    return Some(bytes);
}

/// The pathname a hexadecimal text names,
///  or nothing when it is not one.
#[cfg(unix)]
fn path_of_hex(text: &str) -> Option<PathBuf> {
    /// `OsStringExt` builds OS text from raw bytes without decoding.
    use std::os::unix::ffi::OsStringExt;
    return Some(PathBuf::from(std::ffi::OsString::from_vec(from_hex(text)?)));
}

/// The pathname a hexadecimal text names,
///  on a system whose paths are text.
/// UNTESTED ON WINDOWS:
/// no gate compiles or runs this branch (the crate does not build for Windows;
/// README,
/// Requirements).
#[cfg(not(unix))]
fn path_of_hex(text: &str) -> Option<PathBuf> {
    return Some(PathBuf::from(String::from_utf8(from_hex(text)?).ok()?));
}

/// The bytes a pathname is made of,
///  on a system whose paths are raw bytes.
#[cfg(unix)]
fn bytes_of_path(path: &Path) -> Vec<u8> {
    /// `OsStrExt` lends a path's bytes.
    use std::os::unix::ffi::OsStrExt;
    return path.as_os_str().as_bytes().to_vec();
}

/// The bytes a pathname is made of,
///  on a system whose paths are text.
/// UNTESTED ON WINDOWS:
/// no gate compiles or runs this branch (the crate does not build for Windows;
/// README,
/// Requirements).
#[cfg(not(unix))]
fn bytes_of_path(path: &Path) -> Vec<u8> {
    return path.to_string_lossy().into_owned().into_bytes();
}

/// What:
///  The record of one correction as the text a later command reads.
/// Why:
///   One line per field keeps the record inspectable with `cat`,
///  and hexadecimal keeps a pathname of
///       any bytes and any digest unambiguous.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = `destination ${hex(path)}\nmode ${octal}\nuid ${uid}\ngid ${gid}\nowner ${token}\nreplacement ${sha}\n`;
/// ```
impl JournalRecord {
    /// What:
    ///  The text of this record,
    ///  one line per field and one line per extended attribute.
    /// Why:
    ///   Hexadecimal keeps a pathname of any bytes,
    ///  an attribute name and value and a digest unambiguous,
    ///  and one line per attribute keeps an empty value distinct from an absent attribute.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const text = `destination ${hex(path)}\nmode ${octal}\n...\n` + attributes.map((a) => `attribute ${hex(a.name)} ${hex(a.value)}\n`).join('');
    /// ```
    pub(crate) fn encode(&self) -> String {
        let mut text: String = format!(
            "destination {}\nmode {:o}\nuid {}\ngid {}\nowner {}\nreplacement {}\n",
            to_hex(bytes_of_path(self.destination.as_path()).as_slice()),
            self.mode,
            self.uid,
            self.gid,
            self.owner.as_deref().unwrap_or(NO_OWNER_TOKEN),
            self.replacement
        );
        for attribute in &self.attributes {
            text.push_str(
                format!(
                    "attribute {} {}\n",
                    to_hex(attribute.name.as_slice()),
                    to_hex(attribute.value.as_slice())
                )
                .as_str(),
            );
        }
        return text;
    }

    /// The record a text names,
    ///  or nothing when a field is missing or malformed.
    pub(crate) fn decode(text: &str) -> Option<JournalRecord> {
        let mut destination: Option<PathBuf> = None;
        let mut mode: Option<u32> = None;
        let mut uid: Option<u32> = None;
        let mut gid: Option<u32> = None;
        let mut owner: Option<Option<String>> = None;
        let mut replacement: Option<String> = None;
        let mut attributes: Vec<Attribute> = Vec::new();
        for line in text.lines() {
            let (key, value) = match line.split_once(' ') {
                Some(found) => found,
                None => continue,
            };
            match key {
                "destination" => destination = path_of_hex(value),
                "mode" => mode = u32::from_str_radix(value, 8).ok(),
                "uid" => uid = value.parse::<u32>().ok(),
                "gid" => gid = value.parse::<u32>().ok(),
                "owner" if value == NO_OWNER_TOKEN => owner = Some(None),
                "owner" => owner = Some(Some(String::from(value))),
                "replacement" => replacement = Some(String::from(value)),
                "attribute" => {
                    let (name, held) = value.split_once(' ')?;
                    attributes.push(Attribute {
                        name: from_hex(name)?,
                        value: from_hex(held)?,
                    });
                }
                _other => {}
            }
        }
        attributes.sort();
        return Some(JournalRecord {
            destination: destination?,
            mode: mode?,
            uid: uid?,
            gid: gid?,
            owner: owner?,
            replacement: replacement?,
            attributes,
        });
    }
}

/// What:
///  Make the journal directory for one correction and write its record,
///  before the file is touched.
///  `Result<PathBuf, JournalFailure>` is the directory,
///  or why nothing was written.
/// Why:
///   Everything a later command needs is on disk before the first file is replaced,
///  so a kill at
///       any later step leaves a record that says what the file was.
///  The directory is private (`0700`) and inside the
///       Git directory,
///       where no `gitignore` rule and no `git add` can reach it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdir(`${gitDir}/cli-git-direct-fix-${suffix}`, { mode: 0o700 }); write(record); write(content);
/// ```
pub(crate) fn open_journal(
    git_dir: &Path,
    destination: &Path,
    kept: &Kept,
    replacement: &[u8],
    owner: Option<&str>,
) -> Result<PathBuf, JournalFailure> {
    // What: The swap that installs the correction needs both names on one file system; a linked worktree whose Git
    //       directory lives on another disk has none, and a copy beside the person's file is what this design refuses.
    // Why:  Refusing before anything is written leaves the worktree untouched and says why, which is better than a copy
    //       Git could stage or a half-installed fix.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (deviceOf(gitDir) !== deviceOf(dirname(destination))) return { kind: 'other-file-system' };
    // ```
    let Some(parent) = destination.parent() else {
        return Err(JournalFailure::Unwritable {
            error: std::io::Error::from(std::io::ErrorKind::InvalidInput),
        });
    };
    if device_of(git_dir)? != device_of(parent)? {
        return Err(JournalFailure::OtherFileSystem);
    }
    let directory: PathBuf = git_dir.join(format!("{JOURNAL_PREFIX}{}", unique_suffix()));
    let created: std::io::Result<()> = private_directory(directory.as_path());
    if let Err(error) = created {
        return Err(JournalFailure::Unwritable { error });
    }
    let record: JournalRecord = JournalRecord {
        destination: destination.to_path_buf(),
        mode: kept.mode,
        uid: kept.uid,
        gid: kept.gid,
        owner: owner.map(String::from),
        replacement: digest_of(replacement),
        attributes: kept.attributes.clone(),
    };
    let written: std::io::Result<()> = std::fs::write(
        directory.join(RECORD_FILE),
        record.encode().as_bytes(),
    );
    if let Err(error) = written {
        let _ = std::fs::remove_dir_all(directory.as_path());
        return Err(JournalFailure::Unwritable { error });
    }
    return Ok(directory);
}

/// What:
///  Why a journal could not be made,
///  and what the person is told about it.
/// Why:
///   A fix that cannot record what it is about to do must not do it:
///  the record is what a later
///       command reads to put a file back.
///  A worktree whose Git directory is on another file system cannot
///       receive the swap at all,
///  and saying so is better than leaving a copy beside the person's file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JournalFailure = { kind: 'unwritable'; error: Error } | { kind: 'other-file-system'; gitDir: string };
/// ```
#[derive(Debug)]
pub(crate) enum JournalFailure {
    /// The journal directory or its record could not be written.
    Unwritable {
        /// The system's reason.
        error: std::io::Error,
    },
    /// The Git directory and the file are on different file systems.
    OtherFileSystem,
}

impl std::fmt::Display for JournalFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Unwritable { error } => write!(
                formatter,
                "its journal cannot be written inside the Git directory ({error})"
            ),
            Self::OtherFileSystem => write!(
                formatter,
                "the Git directory and the file are on different file systems, so the corrected bytes cannot be swapped in without leaving a copy of every corrected file beside it, which Git could stage; nothing was changed, and the fix works in a worktree whose Git directory shares the file system of its files"
            ),
        };
    }
}

/// What:
///  The directory a journal directory's record names,
///  or nothing when it cannot be read.
/// Why:
///   A record that cannot be read proves nothing,
///  and a journal is never removed on a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = readRecord(directory); // undefined when unreadable
/// ```
pub(crate) fn read_record(journal: &Path) -> Option<JournalRecord> {
    let record: PathBuf = journal.join(RECORD_FILE);
    // What: The type of the name first, never through a link, and the bytes only for a regular file.
    // Why:  Opening a FIFO no process writes to blocks for ever, and a record is a file a killed fix wrote.
    //
    // In TS you'd write (pseudocode):
    // if (!lstatSync(record).isFile()) return undefined; return decode(readFileSync(record, 'utf8'));
    let described: std::fs::Metadata = std::fs::symlink_metadata(record.as_path()).ok()?;
    if !described.file_type().is_file() {
        return None;
    }
    let text: String = std::fs::read_to_string(record.as_path()).ok()?;
    return JournalRecord::decode(text.as_str());
}

/// What:
///  The SHA-256 of `bytes`,
///  in lower-case hexadecimal.
/// Why:
///   A record keeps the correction's digest rather than its bytes,
///  and every comparison against it needs
///  the same spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const digestOf = (bytes) => createHash('sha256').update(bytes).digest('hex');
/// ```
pub(crate) fn digest_of(bytes: &[u8]) -> String {
    return to_hex(sha256(bytes).as_slice());
}

/// The corrected bytes' path inside a journal directory,
///  which holds the displaced file once the swap is made.
pub(crate) fn content_of(journal: &Path) -> PathBuf {
    return journal.join(CONTENT_FILE);
}

/// What:
///  Remove a journal directory and everything in it,
///  and say whether it is gone.
/// Why:
///   A removal that fails leaves the journal behind,
///  and a caller that reports success without naming it
///       lets a later command act on a fix behind the person's back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = (journal) => { try { rmSync(journal, { recursive: true }); return true } catch { return false } };
/// ```
pub(crate) fn remove(journal: &Path) -> bool {
    return std::fs::remove_dir_all(journal).is_ok();
}

/// What:
///  A directory only this user may read,
///  failing when it already exists.
/// Why:
///   The journal holds a file's bytes and their record;
///  a fresh directory also proves no other
///       process chose the same name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(directory, { mode: 0o700 }); // throws EEXIST
/// ```
fn private_directory(directory: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        /// `DirBuilderExt` adds `mode`,
        ///  the permissions a new directory is created with.
        use std::os::unix::fs::DirBuilderExt;
        return std::fs::DirBuilder::new().mode(0o700).create(directory);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return std::fs::DirBuilder::new().create(directory);
    }
}

/// What:
///  The identity of the file system `path` is on,
///  or why it cannot be read.
/// Why:
///  A swap needs both names on one file system,
///  and the record must know before it writes anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deviceOf = (path: string) => statSync(path).dev;
/// ```
fn device_of(path: &Path) -> Result<u64, JournalFailure> {
    #[cfg(unix)]
    {
        /// `MetadataExt` adds `dev`.
        use std::os::unix::fs::MetadataExt;
        let described: std::fs::Metadata = std::fs::metadata(path).map_err(unreadable)?;
        return Ok(described.dev());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let described: std::fs::Metadata = std::fs::metadata(path).map_err(unreadable)?;
        let _ = described;
        return Ok(0);
    }
}

/// What:
///  The refusal of a path whose file system identity cannot be read.
/// Why:
///   A journal that cannot tell which file system it is on must not be written,
///  because the swap that
///       follows might cross one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unreadable = (error: Error) => ({ kind: 'unwritable', error });
/// ```
fn unreadable(error: std::io::Error) -> JournalFailure {
    return JournalFailure::Unwritable { error };
}

/// What:
///  Make a journal's writer one no platform can name,
///  so a control can stand where a kill left the record.
/// Why:
///   A control cannot kill its own process and keep running;
///  the record's writer is what a sweep
///       judges,
///  so the control rewrites that one field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const markWriterGone = (journal) => rewriteOwner(journal, deadToken());
/// ```
pub fn mark_writer_gone(journal: &Path) {
    let record: PathBuf = journal.join(RECORD_FILE);
    // A name that is not a regular file is left alone: opening it could block, and this helper writes no record then.
    let Ok(described) = std::fs::symlink_metadata(record.as_path()) else {
        return;
    };
    if !described.file_type().is_file() {
        return;
    }
    let text: String = match std::fs::read_to_string(record.as_path()) {
        Ok(found) => found,
        Err(_unreadable) => return,
    };
    let dead: crate::worktree_copy_owner::CopyOwner = crate::worktree_copy_owner::CopyOwner {
        pid: 4_194_304,
        birth_identity: String::from("0"),
        pid_namespace: crate::pid_namespace::current_pid_namespace(),
    };
    let mut rewritten: String = String::new();
    for line in text.lines() {
        if line.starts_with("owner ") {
            rewritten.push_str(format!("owner {}\n", dead.token()).as_str());
        } else {
            rewritten.push_str(format!("{line}\n").as_str());
        }
    }
    let _written = std::fs::write(journal.join(RECORD_FILE), rewritten.as_bytes());
}

/// Journal controls stay out of the release executable.
#[cfg(test)]
#[path = "direct_fix_install_journal_tests.rs"]
mod tests;
