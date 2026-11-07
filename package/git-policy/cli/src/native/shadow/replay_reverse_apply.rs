//! What:
//!  The exact emulation of `git apply --reverse --check` for one text file:
//!       the subsumption test that asks whether a landed change already sits in the prepared bytes.
//! Why:
//!  Git's `apply.c` matching (`apply_one_fragment`, `find_pos`, `match_fragment`) with its defaults:
//!       no context reduction,
//!       no whitespace fixing,
//!       no overlapping hunks,
//!       and anchoring at the file's start and end.
//!       Replay must give the same answer as Git for the same bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! reverseApplies({ hunks, prepared }): boolean
//! ```

/// Hunks and lines from `replay_patch`.
use crate::replay_patch::{PatchHunk, PatchLine, split_keeping_newlines};

/// One image line:
///       its bytes and whether an earlier hunk wrote it,
///       so no later hunk may match it.
/// Why:
///   Git marks patched lines in the image and never matches them again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ImageLine = { text: string; patched: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct ImageLine {
    /// Line bytes.
    text: Vec<u8>,
    /// Written by an earlier hunk.
    patched: bool,
}

/// Whether one byte is not one of the white-space bytes that Git's quick hash leaves out. Used by `filter`.
fn is_not_hash_whitespace(byte: &u8) -> bool {
    return !matches!(*byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r');
}

/// Whether a hunk line is an addition or a removal and not context. `&PatchLine` borrows the line, as `rposition` hands it over.
fn is_change_line(line: &PatchLine) -> bool {
    return line.op != b' ';
}

/// The position `at` as a start in the image when the preimage matches there; nothing for a negative position or a mismatch.
fn position_that_matches(image: &[ImageLine], preimage: &[Vec<u8>], at: i64, match_beginning: bool, match_end: bool) -> Option<usize> {
    // A negative position has no `usize`, and `matches_at` rejects a position whose preimage runs past the image.
    let index: usize = usize::try_from(at).ok()?;
    if matches_at(image, preimage, index, match_beginning, match_end) {
        return Some(index);
    }
    return None;
}

/// What:
///  The bytes Git's quick line hash ignores:
///       spaces, tabs, newlines, vertical tabs, form feeds and carriage returns.
/// Why:
///   An incomplete last preimage line matches an image line with the same non-whitespace content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hashKey(text) // text without HASH_WHITESPACE
/// ```
fn hash_key(text: &[u8]) -> Vec<u8> {
    return text
        .iter()
        .copied()
        .filter(is_not_hash_whitespace)
        .collect();
}

/// What:
///  Whether one image line matches one preimage line,
///       as Git's quick hash check and `memcmp` together decide it.
/// Why:
///   A preimage's incomplete last line only has to be a prefix of its image line
///       whose non-whitespace content is equal,
///       unless the hunk is anchored at the end, where the line must be equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lineMatches({ line, wanted, incompleteLast, matchEnd });
/// ```
fn line_matches(line: &ImageLine, wanted: &[u8], incomplete_last: bool, match_end: bool) -> bool {
    if line.patched {
        return false;
    }
    if !incomplete_last || match_end {
        return line.text == wanted;
    }
    return line.text.starts_with(wanted) && hash_key(&line.text) == hash_key(wanted);
}

/// What:
///  Git's `match_fragment` without whitespace fixing:
///       whether the preimage matches the image starting at `at`.
/// Why:
///   The beginning and end anchors are checked before any line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// matchesAt({ image, preimage, at, matchBeginning, matchEnd });
/// ```
fn matches_at(
    image: &[ImageLine],
    preimage: &[Vec<u8>],
    at: usize,
    match_beginning: bool,
    match_end: bool,
) -> bool {
    let end: usize = preimage.len() + at;
    if end > image.len() || (match_end && end != image.len()) || (match_beginning && at != 0) {
        return false;
    }
    for (index, wanted) in preimage.iter().enumerate() {
        let incomplete_last: bool = index + 1 == preimage.len() && !wanted.ends_with(b"\n");
        if !line_matches(&image[at + index], wanted, incomplete_last, match_end) {
            return false;
        }
    }
    return true;
}

/// What:
///  Git's `find_pos`:
///       the expected line first,
///       then one line further forwards and backwards in turn.
/// Why:
///   A hunk whose context moved a few lines still applies where Git finds it,
///       so the subsumption answer matches Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// findPosition({ image, preimage, line, matchBeginning, matchEnd }); // first match or -1
/// ```
fn find_position(
    image: &[ImageLine],
    preimage: &[Vec<u8>],
    line: i64,
    match_beginning: bool,
    match_end: bool,
) -> Option<usize> {
    let length: i64 = i64::try_from(image.len()).unwrap_or(i64::MAX);
    let preimage_length: i64 = i64::try_from(preimage.len()).unwrap_or(i64::MAX);
    let anchored: i64 = if match_beginning {
        0
    } else if match_end {
        length - preimage_length
    } else {
        line
    };
    let first: i64 = anchored.max(0).min(length);
    // A negative position has no `usize`, and `matches_at` rejects a position whose preimage runs past the image.
    if let Some(found) = position_that_matches(image, preimage, first, match_beginning, match_end) {
        return Some(found);
    }
    // `mut` lets the search widen by one line per side until it passes both ends of the image.
    let mut distance: i64 = 1;
    while first + distance <= length || first - distance >= 0 {
        if let Some(found) = position_that_matches(image, preimage, first + distance, match_beginning, match_end) {
            return Some(found);
        }
        if let Some(found) = position_that_matches(image, preimage, first - distance, match_beginning, match_end) {
            return Some(found);
        }
        distance += 1;
    }
    return None;
}

/// What:
///  Apply one hunk in reverse to the image:
///       its new side (context and additions) must be found,
///       and its old side (context and removals) takes its place.
/// Why:
///   `reverse_patches` swaps the sides before Git searches,
///       so the search starts from the old side's line and anchors on the new side's start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reverseHunk({ image, hunk }); // image or HUNK_REJECTED
/// ```
fn reverse_hunk(image: &[ImageLine], hunk: &PatchHunk) -> Option<Vec<ImageLine>> {
    let mut preimage: Vec<Vec<u8>> = Vec::new();
    let mut postimage: Vec<ImageLine> = Vec::new();
    for line in &hunk.lines {
        if line.op != b'-' {
            preimage.push(line.text.clone());
        }
        if line.op != b'+' {
            postimage.push(ImageLine {
                text: line.text.clone(),
                patched: true,
            });
        }
    }
    let trailing: usize = match hunk.lines.iter().rposition(is_change_line) {
        Some(last_change) => hunk.lines.len() - 1 - last_change,
        None => hunk.lines.len(),
    };
    let line: i64 = if hunk.old_start == 0 { 0 } else { hunk.old_start - 1 };
    let at: usize = find_position(
        image,
        &preimage,
        line,
        hunk.new_start <= 1,
        trailing == 0,
    )?;
    // `mut` lets the reversed image be built in order.
    let mut rebuilt: Vec<ImageLine> = image[..at].to_vec();
    rebuilt.extend(postimage);
    rebuilt.extend_from_slice(&image[at + preimage.len()..]);
    return Some(rebuilt);
}

/// What:
///  Whether every hunk of a landed change applies in reverse to the prepared bytes,
///       in order, so the prepared bytes already contain the landed change.
/// Why:
///   Subsumption's strict check; the one-sided extension in `replay_containment` covers what this rejects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reverseApplies({ hunks, prepared: 'a\nB\nc\nd\n' });
/// ```
pub fn reverse_applies(hunks: &[PatchHunk], prepared: &[u8]) -> bool {
    // `mut` lets the image be replaced by each hunk's result.
    let mut image: Vec<ImageLine> = Vec::new();
    for text in split_keeping_newlines(prepared) {
        image.push(ImageLine {
            text,
            patched: false,
        });
    }
    for hunk in hunks {
        match reverse_hunk(&image, hunk) {
            Some(next) => image = next,
            None => return false,
        }
    }
    return true;
}

/// Batched Git work for the seeded differential control.
#[cfg(test)]
#[path = "replay_reverse_apply_test_support.rs"]
mod test_support_batch;

#[cfg(test)]
#[path = "replay_reverse_apply_tests.rs"]
mod tests;
