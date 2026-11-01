//! What:
//!  The per-worktree capture store:
//!  its location,
//!  its sequence file and its identity file,
//!       read as the incumbent reads them.
//! Why:
//!  Every commit transaction captures under a short per-worktree lock and takes the next
//!      sequence number,
//!  so sequence numbers order the captured disk states of one worktree.
//!      Recovery and pruning read the store;
//!  both wrappers share it
//!      (`src/policy-engine/commit-capture-order-store.ts`):
//!      `<git-dir>/cli-git-captures/{capture.lock/, worktree-id, sequence, landed/<oid>.json}`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const next = await readNextCaptureSequence(gitDir);
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// Trimming with the incumbent's `.trim()` semantics.
use super::js_text::trim_javascript;
/// The largest safe integer.
use super::json_record::MAX_SAFE_INTEGER;
/// The private-directory creation the registry uses.
use super::transaction_registry::ensure_transaction_root;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Capture store directory name inside the worktree's Git directory.
pub const CAPTURE_STORE_NAME: &str = "cli-git-captures";

/// Capture lock directory name inside the store.
pub const CAPTURE_LOCK_NAME: &str = "capture.lock";

/// Worktree identity filename inside the store.
pub const WORKTREE_ID_FILENAME: &str = "worktree-id";

/// Sequence filename inside the store.
pub const SEQUENCE_FILENAME: &str = "sequence";

/// What:
///  A malformed or unreadable capture-order state,
///  with its message.
/// Why:
///   The incumbent's `CaptureOrderRecordError` and `TypeError`s of the store;
///  recovery and
///       pruning report them and continue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CaptureOrderRecordError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureError(pub String);

/// What:
///  The message as display text.
/// Why:
///   Warnings quote it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for CaptureError {
    /// Writes the message.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.0.as_str());
    }
}

/// What:
///  The capture store directory of a worktree.
/// Why:
///   `captureStorePath`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// captureStorePath('/repo/.git') // '/repo/.git/cli-git-captures'
/// ```
pub fn capture_store_path(git_dir: &Path) -> PathBuf {
    return git_dir.join(CAPTURE_STORE_NAME);
}

/// What:
///  The value of a sequence file:
///  canonical decimal digits of a safe integer,
///  then one
///       newline.
/// Why:
///   `parseSequence`:
///  `String(Number.parseInt(digits, 10)) === digits` admits no sign,
///  no
///       leading zero and no whitespace;
///  anything else is a malformed store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseSequence({ text, path }): number; // throws TypeError
/// ```
pub fn parse_sequence(bytes: &[u8], path: &Path) -> Result<i64, CaptureError> {
    let Some(digits) = bytes.strip_suffix(b"\n") else {
        return Err(malformed_sequence(path));
    };
    if digits.is_empty() || (digits.len() > 1 && digits[0] == b'0') {
        return Err(malformed_sequence(path));
    }
    // `mut` allows accumulating the value digit by digit.
    let mut value: i64 = 0;
    for digit in digits {
        if !digit.is_ascii_digit() {
            return Err(malformed_sequence(path));
        }
        // Sixteen digits already exceed the safe range, so the value never overflows `i64`.
        value = value * 10 + i64::from(digit - b'0');
        if value > MAX_SAFE_INTEGER {
            return Err(malformed_sequence(path));
        }
    }
    return Ok(value);
}

/// What:
///  The failure for a malformed sequence file.
/// Why:
///   Every malformed spelling reports the same message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TypeError(`Capture sequence file is malformed: ${path}`)
/// ```
fn malformed_sequence(path: &Path) -> CaptureError {
    return CaptureError(format!(
        "Capture sequence file is malformed: {}",
        path.display()
    ));
}

/// What:
///  The next capture sequence number the store would allocate,
///  read without the lock;
///  an
///       absent store or sequence file reads as 1.
/// Why:
///   `readNextCaptureSequence`:
///  landings record it after their compare-and-swap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readNextCaptureSequence(gitDir: string): Promise<number>;
/// ```
pub fn read_next_capture_sequence(git_dir: &Path) -> Result<i64, CaptureError> {
    let path: PathBuf = capture_store_path(git_dir).join(SEQUENCE_FILENAME);
    match std::fs::read(&path) {
        Ok(bytes) => return Ok(parse_sequence(bytes.as_slice(), path.as_path())? + 1),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            debug(
                "readNextCaptureSequence",
                format!("no capture sequence yet at {}", path.display()).as_str(),
            );
            return Ok(1);
        }
        Err(error) => {
            return Err(CaptureError(format!(
                "reading {} failed: {error}",
                path.display()
            )));
        }
    }
}

/// What:
///  Create the store directory as a private,
///  non-linked directory when it is missing.
/// Why:
///   `ensureCaptureStore`,
///  through the registry's root creation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ensureCaptureStore(gitDir);
/// ```
pub fn ensure_capture_store(git_dir: &Path) -> Result<PathBuf, CaptureError> {
    let store: PathBuf = capture_store_path(git_dir);
    if let Err(error) = ensure_transaction_root(store.as_path()) {
        return Err(CaptureError(error.0));
    }
    return Ok(store);
}

/// What:
///  The text of a small regular store file read without following a link,
///  decoded as
///       UTF-8 with replacement characters.
/// Why:
///   `readStoreFile` reads with `'utf8'`,
///  which never fails on bytes.
///  A missing file is
///       `Ok(None)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readStoreFile(path: string): Promise<string>; // ENOENT propagates
/// ```
pub fn read_store_file(path: &Path) -> Result<Option<String>, CaptureError> {
    match super::private_storage::read_regular_file(path, STORE_FILE_LIMIT) {
        Ok(bytes) => return Ok(Some(String::from_utf8_lossy(bytes.as_slice()).into_owned())),
        Err(super::private_storage::ReadRefusal::Missing) => return Ok(None),
        Err(super::private_storage::ReadRefusal::NotRegular) => {
            return Err(CaptureError(format!(
                "Capture store entry is not a regular file: {}",
                path.display()
            )));
        }
        Err(super::private_storage::ReadRefusal::TooLarge) => {
            return Err(CaptureError(format!(
                "Capture store entry is larger than {STORE_FILE_LIMIT} bytes: {}",
                path.display()
            )));
        }
        Err(super::private_storage::ReadRefusal::Io(error)) => {
            return Err(CaptureError(format!(
                "reading {} failed: {error}",
                path.display()
            )));
        }
    }
}

/// Largest store file read:
///  records list captured paths,
///  so they can be large.
const STORE_FILE_LIMIT: u64 = 256 * 1024 * 1024;

/// What:
///  The store's worktree identity without creating one,
///  or nothing when absent.
/// Why:
///   `readWorktreeId`:
///  a record of another store generation is unusable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readWorktreeId(gitDir: string): Promise<string | typeof WORKTREE_ID_ABSENT>;
/// ```
pub fn read_worktree_id(git_dir: &Path) -> Result<Option<String>, CaptureError> {
    let path: PathBuf = capture_store_path(git_dir).join(WORKTREE_ID_FILENAME);
    match read_store_file(path.as_path())? {
        Some(text) => return Ok(Some(String::from(trim_javascript(text.as_str())))),
        None => {
            debug(
                "readWorktreeId",
                format!("no capture store identity in {}", git_dir.display()).as_str(),
            );
            return Ok(None);
        }
    }
}

/// Store controls stay out of the release executable.
#[cfg(test)]
#[path = "capture_store_tests.rs"]
mod tests;
