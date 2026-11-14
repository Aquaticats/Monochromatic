//! What:
//!  The record of the commit an abort restores,
//!  read from the file Git's own implementation reads.
//! Why:
//!  A rebase records its original head in `rebase-merge/orig-head` or `rebase-apply/orig-head`,
//!  and a pick sequence in `sequencer/head`;
//!  a record that cannot be read,
//!  that is not a regular file,
//!  or that does not hold a commit id is a failure,
//!  because the refusal it earns must not read as clean.
//!  Every read here is bounded and refuses anything that is not a regular file:
//!  a pipe at that path held a `git rebase --abort` for good.
//!  It lives beside the reading that asks for it,
//!  so the provider's own file stays within its line bound.

use std::path::Path;

/// What:
///  How many bytes of a record the wrapper reads.
/// Why:
///   A record holds one commit id,
/// which is 40 hexadecimal bytes for SHA-1 and 64 for SHA-256;
///  nothing Git writes there is longer,
///  and a read that obeyed the file's own length would let whatever stands at that path decide
///  how much this command spends.
///  One page is room for either id,
/// its newline and any trailing blank the writer left.
pub const RECORD_BYTES_READ: u64 = 4096;

/// What:
///  The record of the commit an abort restores opened for reading,
///  or the system's own reason for not opening it.
/// Why:
///   A record that is a pipe,
/// a socket,
/// a device or a directory is not something Git writes,
///  and opening one without `O_NONBLOCK` waits for a writer that may never come:
///  a `git rebase --abort` whose `rebase-merge/orig-head` is a pipe hung there,
/// with nothing able to end it.
///  `O_NONBLOCK` returns at once for every kind of entry,
///  `O_NOFOLLOW` refuses a link some other process left at the name,
///  and the caller refuses whatever is not a regular file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const opened = openTheRecord(path); // never waits
/// ```
pub(super) fn open_the_record(path: &Path) -> std::io::Result<std::fs::File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        return std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(path);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return std::fs::File::open(path);
    }
}

/// What:
///  The text of a record of the commit an abort restores,
///  read from a regular file and never more than `RECORD_BYTES_READ` of it,
///  or why that cannot be read.
/// Why:
///   A record that is not a regular file,
///  one longer than any commit id,
///  one that cannot be opened,
///  and one whose bytes are not text are all failures with their own words,
///  because the person has to be told which path stands in the way and what is wrong with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = readTheRecord(path); // throws
/// ```
pub(super) fn read_the_record(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let file: std::fs::File = match open_the_record(path) {
        Ok(opened) => opened,
        Err(error) => {
            return Err(format!(
                "could not read the commit this abort restores: Git records it in {} and the file could not be opened ({error}).",
                path.display()
            ));
        }
    };
    let kind: std::io::Result<std::fs::Metadata> = file.metadata();
    let metadata: std::fs::Metadata = match kind {
        Ok(read) => read,
        Err(error) => {
            return Err(format!(
                "could not read the commit this abort restores: Git records it in {} and that path could not be read ({error}).",
                path.display()
            ));
        }
    };
    if !metadata.file_type().is_file() {
        return Err(format!(
            "could not read the commit this abort restores: Git records it in {}, which is not a regular file, so the wrapper did not read it.",
            path.display()
        ));
    }
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    let mut bounded = file.take(RECORD_BYTES_READ + 1);
    if let Err(error) = bounded.read_to_end(&mut bytes) {
        return Err(format!(
            "could not read the commit this abort restores: Git records it in {} and the file could not be read ({error}).",
            path.display()
        ));
    }
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > RECORD_BYTES_READ {
        return Err(format!(
            "could not read the commit this abort restores: Git records it in {} and the file holds more than {RECORD_BYTES_READ} bytes, so it holds no commit id.",
            path.display()
        ));
    }
    return match String::from_utf8(bytes) {
        Ok(text) => Ok(text),
        Err(error) => Err(format!(
            "could not read the commit this abort restores: Git records it in {} and the file is not text ({error}).",
            path.display()
        )),
    };
}
