//! What:
//!  Reading one allow-list file from disk:
//!  the largest file this reader accepts,
//!  the checks a path must pass before it is read,
//!  and the bounded read the cap is checked by.
//! Why:
//!  The option names a file a person edits,
//!  so reading it must not block on a pipe or fill memory from a device,
//!  and a file that grows between the measurement and the read is refused rather than swallowed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const MAX_BYTES = 4 * 1024 * 1024;
//! function readAllowList(path: Path): AllowList;
//! function readAtMost(reader: ReadStream, path: Path): Uint8Array;
//! ```

/// Import the list the entries make and the reader that builds it from the text.
use crate::layout_allow_list::{
    AllowList,
    parse_allow_list,
};
/// Import the reader the cap is put on and native paths.
use std::{
    io::Read,
    path::Path,
};

/// What:
///  The largest list file this reader accepts:
///  four mebibytes.
/// Why:
///  This repository's largest lists are the git wrapper's,
///  measured on 2026-11-25 from the branch that builds them:
///  `folder-rule.txt` 57,956 bytes with 524 `uses` lines and 416 `missing-link` lines,
///  `file-rule.txt` 53,048 bytes with 544 `file-uses` lines and 42 `no-file-list` lines.
///  The list a loop needs is one line per use inside it,
///  and the dense shape of the alpha.15 release pass
///  (200 groups whose own files use one folder that uses a child of each of them,
///  60,100 uses in one loop)
///  needs 60,300 lines and 1,681,290 bytes,
///  measured on 2026-11-26 while that pass's findings were fixed:
///  one mebibyte refused the very list the rule asked the round to write.
///  Four mebibytes leave room for about 150,000 recorded uses
///  and still bound a file a person controls:
///  the bound is what keeps a mistyped path from reading a disk image,
///  and the read itself is bounded too,
///  so a file that grows between the check and the read is refused rather than swallowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_BYTES = 4 * 1024 * 1024;
/// ```
pub const MAX_BYTES: u64 = 4 * 1024 * 1024;

/// What:
///  Read one list file,
///  refusing a path that is not a regular file and a file over `MAX_BYTES`.
/// Why:
///  The option names a file a person edits;
///  reading it must not block on a pipe or fill memory from a device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readAllowList(path: Path): AllowList;
/// ```
pub fn read_allow_list(path: &Path) -> Result<AllowList, String> {
    let metadata: std::fs::Metadata = std::fs::metadata(path).map_err(|error| {
        return format!(
            "The allow-list file {} cannot be read: {error}.",
            path.display()
        );
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "The allow-list file {} is not a regular file.",
            path.display()
        ));
    }
    if metadata.len() > MAX_BYTES {
        return Err(format!(
            "The allow-list file {} is {} bytes, over the {MAX_BYTES} bytes this reader accepts.",
            path.display(),
            metadata.len()
        ));
    }
    let file: std::fs::File = std::fs::File::open(path).map_err(|error| {
        return format!(
            "The allow-list file {} cannot be read: {error}.",
            path.display()
        );
    })?;
    let bytes: Vec<u8> = read_at_most(file, path)?;
    let text: String = String::from_utf8(bytes).map_err(|error| {
        return format!(
            "The allow-list file {} cannot be read: {error}.",
            path.display()
        );
    })?;
    return parse_allow_list(text.as_str(), path);
}

/// What:
///  Read at most the list cap from one reader,
///  refusing anything longer.
/// Why:
///  The size of the file was measured before this read,
///  and a file that grows between the measurement and the read would get past that check,
///  so the read itself stops one byte past the cap and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readAtMost(reader: ReadStream, path: Path): Uint8Array;
/// ```
pub(crate) fn read_at_most<R: std::io::Read>(
    reader: R,
    path: &Path,
) -> Result<Vec<u8>, String> {
    // What: at most one byte more than the cap, so the length read is known even when it is over.
    // Why: reading the whole of a file that grew past the cap is what the bound exists to prevent.
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    reader
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            return format!(
                "The allow-list file {} cannot be read: {error}.",
                path.display()
            );
        })?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(format!(
            "The allow-list file {} is longer than the {MAX_BYTES} bytes this reader accepts.",
            path.display()
        ));
    }
    return Ok(bytes);
}
