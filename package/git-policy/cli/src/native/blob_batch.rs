//! What:
//!  Loading many blobs through one `git cat-file --batch`,
//!  optionally from a private object
//!       store,
//!  with strict parsing of the batch output.
//! Why:
//!  Post-landing completion reads the original and intended bytes of every added path;
//!  one
//!      batch replaces a Git start per object (`src/policy-engine/blob-batch.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const blobs = await loadBlobBatch({ gitPath, cwd, oids, objectDirectory, createError });
//! ```

/// Running real Git.
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// `HashMap` maps object IDs to bytes.
use std::collections::HashMap;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  One successful batch header:
///  the object ID and its byte length.
/// Why:
///   `parseBatchHeader`:
///  `<oid> blob <size>`,
///  exactly three space-separated fields,
///  a blob,
///       and a canonical decimal size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseBatchHeader({ header }): { oid: string; size: number };
/// ```
pub fn parse_batch_header(header: &str) -> Result<(String, usize), String> {
    let fields: Vec<&str> = header.split(' ').collect();
    if fields.len() != 3 {
        return Err(format!(
            "Git blob batch returned malformed header: {header}"
        ));
    }
    let (oid, kind, size_text) = (fields[0], fields[1], fields[2]);
    if kind != "blob" {
        return Err(format!("Git blob batch returned {kind} for {oid}."));
    }
    let canonical: bool = !size_text.is_empty()
        && size_text.bytes().all(is_ascii_digit)
        && (size_text == "0" || !size_text.starts_with('0'));
    let parsed: Option<i64> = if canonical {
        size_text.parse::<i64>().ok()
    } else {
        None
    };
    if let Some(found) = parsed
        && found <= super::json_record::MAX_SAFE_INTEGER
        && let Ok(size) = usize::try_from(found)
    {
        return Ok((String::from(oid), size));
    }
    return Err(format!(
        "Git blob batch returned invalid size for {oid}: {size_text}"
    ));
}

/// What:
///  Whether a byte is an ASCII digit.
/// Why:
///   A named predicate keeps the scan free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte >= 0x30 && byte <= 0x39
/// ```
fn is_ascii_digit(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
///  Parse ordered batch output for the requested objects,
///  never reading blob bytes as
///       delimiters.
/// Why:
///   `parseBatchOutput`:
///  every header names the requested object,
///  every content ends with a
///       newline,
///  and nothing trails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseBatchOutput({ output, requestedOids }): ReadonlyMap<string, Uint8Array>;
/// ```
pub fn parse_batch_output(
    output: &[u8],
    requested: &[String],
) -> Result<HashMap<String, Vec<u8>>, String> {
    // `mut` allows filling the map and advancing the cursor.
    let mut blobs: HashMap<String, Vec<u8>> = HashMap::new();
    let mut offset: usize = 0;
    for oid in requested {
        let Some(relative) = output[offset..].iter().position(is_newline) else {
            return Err(format!("Git blob batch omitted header for {oid}."));
        };
        let header_end: usize = offset + relative;
        let Ok(header) = std::str::from_utf8(&output[offset..header_end]) else {
            return Err(format!(
                "Git blob batch returned a header that is not UTF-8 for {oid}."
            ));
        };
        let (found, size) = parse_batch_header(header)?;
        if &found != oid {
            return Err(format!(
                "Git blob batch returned {found} while {oid} was requested."
            ));
        }
        let start: usize = header_end + 1;
        let end: usize = match start.checked_add(size) {
            Some(value) if value < output.len() && output[value] == b'\n' => value,
            _ => {
                return Err(format!(
                    "Git blob batch returned truncated content for {oid}."
                ));
            }
        };
        blobs.insert(oid.clone(), output[start..end].to_vec());
        offset = end + 1;
    }
    if offset != output.len() {
        return Err(String::from(
            "Git blob batch returned unexpected trailing bytes.",
        ));
    }
    return Ok(blobs);
}

/// What:
///  Whether a byte is a newline.
/// Why:
///   A named predicate keeps the search free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0x0a
/// ```
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Load every distinct requested blob through one batch,
///  optionally reading a private
///       object store (whose alternates still reach the real one).
/// Why:
///   `loadBlobBatch`;
///  request order is first-seen order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function loadBlobBatch({ gitPath, cwd, oids, objectDirectory }): Promise<ReadonlyMap<string, Uint8Array>>;
/// ```
pub fn load_blob_batch(
    context: &GitContext,
    cwd: &Path,
    oids: &[String],
    object_directory: Option<&Path>,
) -> Result<HashMap<String, Vec<u8>>, String> {
    // `mut` allows collecting distinct IDs in order.
    let mut requested: Vec<String> = Vec::new();
    for oid in oids {
        if !requested.contains(oid) {
            requested.push(oid.clone());
        }
    }
    if requested.is_empty() {
        return Ok(HashMap::new());
    }
    let mut input: String = requested.join("\n");
    input.push('\n');
    let mut request: GitRequest = GitRequest::new(cwd, &["cat-file", "--batch"]);
    request.input = Some(input.into_bytes());
    request.object_directory = object_directory.map(Path::to_path_buf);
    let output: GitOutput = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("git cat-file --batch could not start: {error}")),
    };
    if !output.succeeded() {
        return Err(format!(
            "git cat-file --batch failed: {}",
            output.error_text()
        ));
    }
    return parse_batch_output(output.stdout.as_slice(), requested.as_slice());
}

/// Batch controls stay out of the release executable.
#[cfg(test)]
#[path = "blob_batch_tests.rs"]
mod tests;
