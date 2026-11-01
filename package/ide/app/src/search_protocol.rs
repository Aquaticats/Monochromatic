//! Decode ripgrep records without replacing native path bytes or silently discarding malformed matches.

/// Search results retain typed navigation metadata and bounded previews.
use crate::search::{MAX_PREVIEW_GRAPHEMES, SearchHit, SearchKind};
/// Parsing failures identify the ripgrep protocol surface and affected project.
use anyhow::{Context, Result, bail};
/// Ripgrep represents non-UTF-8 bytes with the standard padded base64 alphabet.
use base64::{Engine, engine::general_purpose::STANDARD};
/// Display previews use existing Helix grapheme boundaries rather than cutting combining sequences.
use helix_core::unicode::segmentation::UnicodeSegmentation;
/// Deserialize only needed fields while retaining envelope types for explicit metadata handling.
use serde::Deserialize;
/// Native paths preserve arbitrary Unix filename bytes;
///  components reject traversal-shaped results.
use std::{
    ffi::OsString,
    os::unix::ffi::OsStringExt,
    path::{Component, Path, PathBuf},
};

/// Ripgrep emits exactly one of text or base64 bytes for each path or line field.
#[derive(Deserialize)]
struct TextOrBytes {
    /// UTF-8 input retains its exact bytes after JSON decoding.
    text: Option<String>,
    /// Non-UTF-8 input is represented by base64,
    ///  not a lossy display string.
    bytes: Option<String>,
}

/// Decode the external tagged representation while rejecting ambiguous or missing payloads.
impl TextOrBytes {
    /// Own decoded bytes so a filename can outlive the JSON record buffer.
    fn decode(self) -> Result<Vec<u8>> {
        // What: match extracts the two valid optional-field combinations; other combinations are malformed.
        // Why: A record with both text and bytes cannot silently choose one path identity.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (text !== undefined && bytes === undefined) return utf8(text);
        // if (bytes !== undefined && text === undefined) return decodeBase64(bytes);
        // throw new Error('Expected exactly one payload');
        // ```
        match (self.text, self.bytes) {
            (Some(text), None) => {
                return Ok(text.into_bytes());
            }
            (None, Some(encoded)) => {
                return STANDARD
                    .decode(encoded)
                    .context("Invalid base64 bytes in ripgrep JSON");
            }
            _ => {
                bail!("Ripgrep JSON must contain exactly one text or bytes payload");
            }
        }
    }
}

/// What:
///  Serde selects a closed variant from type and decodes its data directly.
/// Why:
///  Metadata is skipped without allocation,
///  while duplicate match fields remain detectable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Event = { type: 'match'; data: ContentMatch } | { type: MetadataKind; data: unknown };
/// ```
#[derive(Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
enum Event {
    /// A matching line carries validated path and source metadata.
    Match(
        /// Validated match payload.
        Match,
    ),
    /// File-start metadata does not become a result.
    Begin(
        /// Unused file-start payload.
        serde::de::IgnoredAny,
    ),
    /// File-end metadata does not become a result.
    End(
        /// Unused file-end payload.
        serde::de::IgnoredAny,
    ),
    /// Context was not requested and does not become a content match.
    Context(
        /// Unused context-line payload.
        serde::de::IgnoredAny,
    ),
    /// Aggregate counters do not become a result.
    Summary(
        /// Unused aggregate-statistics payload.
        serde::de::IgnoredAny,
    ),
}

/// Match fields used by the reader;
///  other ripgrep offsets/statistics remain forward-compatible metadata.
#[derive(Deserialize)]
struct Match {
    /// Exact native path to the matching file.
    path: TextOrBytes,
    /// Source line bytes,
    ///  including their original terminator.
    lines: TextOrBytes,
    /// One-based line number supplied by normal non-multiline JSON search.
    line_number: usize,
}

/// Validate a native result lexically;
///  actual file activation separately resolves symlink containment.
fn path(root: &Path, bytes: Vec<u8>) -> Result<PathBuf> {
    if bytes.is_empty() || bytes.contains(&0) {
        bail!(
            "Ripgrep returned an empty or NUL-containing path for project {}",
            root.display()
        );
    }
    // What: OsString::from_vec retains Unix filename bytes instead of requiring UTF-8 String.
    // Why: Display labels must never become the identity used to open a search result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const resultPath = nativePathFromBytes(bytes);
    // ```
    let result = PathBuf::from(OsString::from_vec(bytes));
    let relative = match result.strip_prefix(root) {
        Ok(found) => found,
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!(
                "Ripgrep returned path {} outside project {}",
                result.display(),
                root.display()
            )));
        }
    };
    // A child path has only plain name components; `..`, roots, and prefixes name something else.
    let mut only_names = true;
    for part in relative.components() {
        if !matches!(part, Component::Normal(_)) {
            only_names = false;
            break;
        }
    }
    if relative.as_os_str().is_empty() || !only_names {
        bail!(
            "Ripgrep returned non-child path {} for project {}",
            result.display(),
            root.display()
        );
    }
    return Ok(result);
}

/// Decode a NUL-delimited filename payload after the stream reader has removed its delimiter.
pub fn filename(root: &Path, record: &[u8]) -> Result<PathBuf> {
    // Copy the bounded record into its owned native path rather than borrowing the reusable input buffer.
    return path(root, record.to_vec());
}

/// Decode one JSON record;
///  expected metadata has no content result,
///  while unknown or malformed events are errors.
pub fn content(root: &Path, record: &[u8]) -> Result<Option<SearchHit>> {
    let event: Event =
        serde_json::from_slice(record).context("Cannot decode ripgrep JSON record")?;
    let matched = match event {
        Event::Match(matched) => matched,
        Event::Begin(_) | Event::End(_) | Event::Context(_) | Event::Summary(_) => {
            return Ok(None);
        }
    };
    if matched.line_number == 0 {
        bail!(
            "Ripgrep returned a zero source line for project {}",
            root.display()
        );
    }
    let target = path(root, matched.path.decode()?)?;
    let line = matched.lines.decode()?;
    // Lossy conversion is display-only; the source is read and validated again when the result is opened.
    let readable = String::from_utf8_lossy(&line);
    let text = readable.trim_end();
    let cut = text.grapheme_indices(true).nth(MAX_PREVIEW_GRAPHEMES);
    let (preview, truncated) = if let Some((offset, _grapheme)) = cut {
        (text[..offset].to_string(), true)
    } else {
        (text.to_string(), false)
    };
    return Ok(Some(SearchHit {
        path: target,
        kind: SearchKind::Content {
            line: matched.line_number,
            preview,
            truncated,
        },
    }));
}
