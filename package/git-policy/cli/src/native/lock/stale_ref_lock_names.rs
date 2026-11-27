//! What:
//!  The names a lock and its pid file are written under,
//!  where the pid file of one lock sits,
//!  and the process number such a file names.
//! Why:
//!  The judgement of a reference lock,
//!  the walk that finds one,
//!  the proof that an abandoned lock is nobody's,
//!  and the sweep that acts on them
//!  read the same two suffixes and the same `pid <number>` text,
//!  so they name one file instead of each other's.
//!  A number in a pid file is a fact about the file,
//!  never a proof about a holder:
//!  a Git of a sibling PID namespace that shares this repository may have written that number.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const number = pidNamedBy(pidFileBeside('/r/.git/packed-refs.lock')); // '/r/.git/packed-refs~pid.lock'
//! ```

/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What every Git lock file's name ends with.
pub(crate) const LOCK_SUFFIX: &str = ".lock";

/// What the pid file of a lock ends with:
/// the lock's name without `.lock`,
/// then this.
pub(crate) const PID_LOCK_SUFFIX: &str = "~pid.lock";

/// What a pid file's text starts with,
/// before the number.
const PID_TEXT_PREFIX: &str = "pid ";

/// The most of a PID file this read takes:
/// a few bytes of `pid <number>` and a newline,
/// and far less than any file a program that meant to hold this reader would offer.
const PID_FILE_LIMIT: u64 = 4096;

/// The pid file Git writes beside a lock:
/// the lock's name without `.lock`,
/// then `~pid.lock`.
pub fn pid_file_beside(lock: &Path) -> PathBuf {
    let name: String = match lock.file_name() {
        Some(file_name) => file_name.to_string_lossy().into_owned(),
        None => String::new(),
    };
    let stem: &str = name.strip_suffix(LOCK_SUFFIX).unwrap_or(name.as_str());
    return lock.with_file_name(format!("{stem}{PID_LOCK_SUFFIX}"));
}

/// What:
///  The bytes of the PID file at `path`,
///  or nothing when the name holds no plain file this command reads.
/// Why:
///  The removal unlinks the PID file only when its contents are the ones it judged,
///  and this is the one read that gives them.
///  Opened the way every other file this judgement reads is opened:
/// without following a link,
/// without blocking on a
///  named pipe,
/// and never past a small bound.
/// A pipe left at a lock's PID-file name held every command for ever when
///  this read was `std::fs::read_to_string` (a review of `675771e759`,
/// finding 2),
/// and the pass that reads it runs on
///  every command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readPlainFile(path, PID_FILE_LIMIT) // throws on anything but a plain file this command reads
/// ```
pub(crate) fn pid_file_contents(path: &Path) -> Option<Vec<u8>> {
    return crate::plain_file::read_plain_file(path, PID_FILE_LIMIT).ok();
}

/// The number the bytes of a pid file name,
/// when their text is `pid <number>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(Buffer.from(bytes).toString('utf8').trim().replace(/^pid /, ''))
/// ```
pub(crate) fn pid_named_in(contents: &[u8]) -> Option<i32> {
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(contents);
    return text
        .trim()
        .strip_prefix(PID_TEXT_PREFIX)?
        .trim()
        .parse::<i32>()
        .ok();
}

/// The process number a pid file names,
/// when its text is `pid <number>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(readFileSync(path, 'utf8').trim().replace(/^pid /, ''))
/// ```
pub(crate) fn pid_named_by(path: &Path) -> Option<i32> {
    return pid_named_in(pid_file_contents(path)?.as_slice());
}
