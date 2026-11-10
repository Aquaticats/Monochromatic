//! What:
//!  What one layer of one path the merge changes ends as once the person's own work is carried
//!  onto the merge:
//!  the merge's side,
//!  the person's bytes,
//!  a word-level merge of the two,
//!  both sides between conflict markers,
//!  or one side kept whole with the reason it could not be merged.
//! Why:
//!  A person's unsaved and staged work are two layers of the same path,
//! and each is merged with the merge on its
//!  own so a staged edit stays staged and an unsaved one stays unsaved (`SPEC.md`,
//! "Merges and commits never fail",
//!  "Carry edits forward").
//! The text merge is the word-level pass the package already runs
//! (`merge_second_pass.rs`),
//!  reused rather than written again:
//! its answer decides whether a region merges,
//! and `word_merge::keep_both` gives the file the same
//!  merge with both sides kept where it does not.
//! A file the pass cannot take as text keeps the person's bytes whole:
//! a
//!  binary file has no markers to hold two sides in,
//! and half a picture is worse than the person's own copy beside the
//!  merge's version in the commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const end = decide(runner, path, { base, ours, theirs }); // { outcome: 'merges' | 'ours' | 'merged' | 'kept' | 'gone', mode }
//! ```

use crate::merge_second_pass::{Conflict, GitRunner, Left, Stage, Verdict, second_pass, unfit, write_blob};
use crate::word_merge::MergedPart;
use crate::merge_second_pass::AttributeSource;
use crate::word_merge::{WordMerge, keep_both, merge_by_word};

/// The labels the two sides carry between conflict markers.
///  A person reads them in their editor,
/// so each says whose version it is.
pub const OUR_LABEL: &str = "your changes";

/// The label of the merge's side between conflict markers.
pub const THEIR_LABEL: &str = "the merge";

/// The reason a path is named when the merge deletes it and the person changed it.
pub const MODIFY_DELETE: &str = "the merge deletes it";

/// The reason a path is named when the person deleted it and the merge changed it.
pub const DELETE_MODIFY: &str = "you deleted it, and the merge changes it";

/// The reason a path is named when both sides hold a kind of entry the text merge cannot take.
pub const NOT_TEXT: &str = "its changes cannot be merged as text";

/// One side of one layer:
/// the mode Git records and the object it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Side = { mode: string; oid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Side {
    /// `100644`,
    /// `100755` or `120000`.
    pub mode: String,
    /// The object name.
    pub oid: String,
}

/// The three sides of one layer of one path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Layer = { base: Side | null; ours: Side | null; theirs: Side | null };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Layer {
    /// What the path held before the merge.
    pub base: Option<Side>,
    /// What the person's layer held.
    pub ours: Option<Side>,
    /// What the merge makes of the path.
    pub theirs: Option<Side>,
}

/// What becomes of one layer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { kind: 'merges' } | { kind: 'ours' } | { kind: 'merged'; bytes: Uint8Array; conflict: boolean; parts: MergedPart[] } | { kind: 'kept'; why: string } | { kind: 'gone'; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// Nothing of the person's is in this layer for the merge to touch:
    /// the layer ends as the merge's side.
    Merges,
    /// The person's object ends the layer,
    /// byte for byte.
    Ours,
    /// New text ends the layer.
    Merged {
        /// The bytes in the object form Git records.
        bytes: Vec<u8>,
        /// Whether the bytes hold conflict markers.
        conflict: bool,
        /// The regions the word pass merged.
        parts: Vec<MergedPart>,
    },
    /// The person's object ends the layer,
    /// and the path is named with this reason.
    Kept(&'static str),
    /// Nothing stands at the path,
    /// and the path is named with this reason.
    Gone(&'static str),
}

/// How one layer ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Decision = { outcome: Outcome; mode: string | null };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decision {
    /// What the layer holds.
    pub outcome: Outcome,
    /// The mode it holds it with,
    /// or nothing when the path ends absent.
    pub mode: Option<String>,
}

/// Whether one side holds what the other does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const equal = (a, b) => (a === null && b === null) || (a !== null && b !== null && a.oid === b.oid);
/// ```
fn same_object(left: &Option<Side>, right: &Option<Side>) -> bool {
    return match (left, right) {
        (None, None) => true,
        (Some(one), Some(other)) => one.oid == other.oid,
        _ => false,
    };
}

/// The mode of one side,
/// or nothing when the side holds no entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modeOf = (side) => side === null ? null : side.mode;
/// ```
fn mode_of(side: &Option<Side>) -> Option<String> {
    return side.as_ref().map(|found| { return found.mode.clone(); });
}

/// What:
///  The mode one layer ends with,
/// as a three-way choice over the three modes.
/// Why:
///  A mode is one bit,
/// so a side equal to the base changed nothing,
/// and the side that did change it wins;
/// two sides that changed it agree,
/// and the person's is the one left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endMode(layer): string | null; // base === ours ? theirs : base === theirs ? ours : ours
/// ```
fn end_mode(layer: &Layer) -> Option<String> {
    let (Some(ours), Some(theirs)) = (&layer.ours, &layer.theirs) else {
        return mode_of(&layer.ours).or_else(|| { return mode_of(&layer.theirs); });
    };
    let base: Option<&str> = layer.base.as_ref().map(|side| { return side.mode.as_str(); });
    if base == Some(ours.mode.as_str()) {
        return Some(theirs.mode.clone());
    }
    return Some(ours.mode.clone());
}

/// What:
///  Read one object's bytes,
/// which a kept conflict needs to hold both of its sides.
/// Why:
///   The second pass reads the stages itself but keeps no bytes;
/// a region it cannot merge needs the three texts to write the
///  markers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function bytes(runner, oid: string): Promise<Uint8Array>;
/// ```
async fn bytes_of(runner: &dyn GitRunner, oid: &str) -> Result<Vec<u8>, String> {
    let reply = runner.run(&["cat-file", "blob", oid], None, None).await?;
    if reply.code != Some(0) {
        return Err(format!("git cat-file {oid} failed: {}", String::from_utf8_lossy(reply.stderr.as_slice()).trim()));
    }
    return Ok(reply.stdout);
}

/// What:
///  The stage a side is,
/// as the second pass reads it.
/// Why:
///  A missing side is what tells the pass the path was added,
/// deleted or kept on one side only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stage = (side) => side === null ? null : { mode: parseInt(side.mode, 8), oid: side.oid };
/// ```
fn stage(side: &Option<Side>) -> Option<Stage> {
    return side.as_ref().map(|found| {
        return Stage { mode: u32::from_str_radix(found.mode.as_str(), 8).unwrap_or(0), oid: found.oid.clone() };
    });
}

/// What:
///  What one layer of one path ends as.
/// Why:
///   The three sides decide most layers with no object read at all:
/// a side that equals the base is the
///  merge's to take,
/// and a side the merge leaves alone is the person's.
/// Only a path both sides changed reaches the
///  word pass,
/// whose gating (a plain file,
/// one mode,
/// text,
/// small,
/// no merge driver) decides whether a
///  region can be merged or both sides must stay between markers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decide(runner, path, layer): Decision; // throws when Git cannot be read
/// ```
pub async fn decide(runner: &dyn GitRunner, path: &[u8], layer: &Layer) -> Result<Decision, String> {
    if same_object(&layer.base, &layer.ours) {
        return Ok(Decision { outcome: Outcome::Merges, mode: end_mode(layer) });
    }
    if same_object(&layer.ours, &layer.theirs) || same_object(&layer.theirs, &layer.base) {
        return Ok(Decision { outcome: Outcome::Ours, mode: end_mode(layer) });
    }
    match (&layer.ours, &layer.theirs) {
        (Some(ours), None) => return Ok(Decision { outcome: Outcome::Kept(MODIFY_DELETE), mode: Some(ours.mode.clone()) }),
        (None, Some(_)) => return Ok(Decision { outcome: Outcome::Gone(DELETE_MODIFY), mode: None }),
        (Some(_), Some(_)) => {}
        (None, None) => return Ok(Decision { outcome: Outcome::Merges, mode: None }),
    }
    if layer.base.is_none() {
        // Both sides added the path, so there is no ancestor to merge from: the two texts are the whole of it, and the word
        //  pass still settles the regions that agree.
        let (Some(ours), Some(theirs)) = (&layer.ours, &layer.theirs) else {
            return Err(String::from("a path both sides added holds two sides"));
        };
        let ours_bytes: Vec<u8> = bytes_of(runner, ours.oid.as_str()).await?;
        let theirs_bytes: Vec<u8> = bytes_of(runner, theirs.oid.as_str()).await?;
        if unfit(ours_bytes.as_slice()).is_some() || unfit(theirs_bytes.as_slice()).is_some() {
            return Ok(Decision { outcome: Outcome::Kept(NOT_TEXT), mode: end_mode(layer) });
        }
        return match merge_by_word(b"", ours_bytes.as_slice(), theirs_bytes.as_slice()) {
            WordMerge::Resolved { merged, parts } => Ok(Decision { outcome: Outcome::Merged { bytes: merged, conflict: false, parts }, mode: end_mode(layer) }),
            WordMerge::Conflict { .. } => Ok(Decision {
                outcome: Outcome::Merged {
                    bytes: keep_both(b"", ours_bytes.as_slice(), theirs_bytes.as_slice(), (OUR_LABEL, THEIR_LABEL)),
                    conflict: true,
                    parts: Vec::new(),
                },
                mode: end_mode(layer),
            }),
        };
    }
    let conflict: Conflict = Conflict { path: path.to_vec(), base: stage(&layer.base), ours: stage(&layer.ours), theirs: stage(&layer.theirs) };
    let verdicts: Vec<Verdict> = second_pass(runner, &[conflict], AttributeSource::Invocation).await?;
    let Some(verdict) = verdicts.into_iter().next() else {
        return Err(String::from("the second pass answered nothing for a path it was given"));
    };
    match verdict {
        Verdict::Merged(file) => {
            return Ok(Decision { outcome: Outcome::Merged { bytes: file.bytes, conflict: false, parts: file.parts }, mode: end_mode(layer) });
        }
        Verdict::Left { why, .. } => {
            if !matches!(why, Left::Overlap(_)) {
                return Ok(Decision { outcome: Outcome::Kept(NOT_TEXT), mode: end_mode(layer) });
            }
            let (Some(base), Some(ours), Some(theirs)) = (&layer.base, &layer.ours, &layer.theirs) else {
                return Err(String::from("a conflict the pass resolved as an overlap holds every stage"));
            };
            let bytes: Vec<u8> = keep_both(
                bytes_of(runner, base.oid.as_str()).await?.as_slice(),
                bytes_of(runner, ours.oid.as_str()).await?.as_slice(),
                bytes_of(runner, theirs.oid.as_str()).await?.as_slice(),
                (OUR_LABEL, THEIR_LABEL),
            );
            return Ok(Decision { outcome: Outcome::Merged { bytes, conflict: true, parts: Vec::new() }, mode: end_mode(layer) });
        }
    }
}

/// What:
///  What one layer ends as without the word pass,
/// for controls that need the verdict of the equality rules alone.
/// Why:
///   The first four rules decide every layer whose sides agree or hold a missing side,
/// and they cost no
///  object read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideWithoutThePass(layer): Decision | null;
/// ```
pub fn decide_without_the_pass(layer: &Layer) -> Option<Decision> {
    if same_object(&layer.base, &layer.ours) {
        return Some(Decision { outcome: Outcome::Merges, mode: end_mode(layer) });
    }
    if same_object(&layer.ours, &layer.theirs) || same_object(&layer.theirs, &layer.base) {
        return Some(Decision { outcome: Outcome::Ours, mode: end_mode(layer) });
    }
    match (&layer.ours, &layer.theirs) {
        (Some(ours), None) => return Some(Decision { outcome: Outcome::Kept(MODIFY_DELETE), mode: Some(ours.mode.clone()) }),
        (None, Some(_)) => return Some(Decision { outcome: Outcome::Gone(DELETE_MODIFY), mode: None }),
        (None, None) => return Some(Decision { outcome: Outcome::Merges, mode: None }),
        (Some(_), Some(_)) => return None,
    }
}

/// What:
///  A text merge of three texts,
/// for controls that do not start Git.
/// Why:
///   The word pass's verdict and the file a person reads must come from one merge,
/// and a control can check both
///  without a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mergeText = (base, ours, theirs) => mergeByWord(base, ours, theirs);
/// ```
pub fn merge_text(base: &[u8], ours: &[u8], theirs: &[u8]) -> WordMerge {
    return merge_by_word(base, ours, theirs);
}

/// The object a merged layer's bytes are written as.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeLayerBytes(runner, bytes): Promise<string>;
/// ```
pub async fn write_layer_bytes(runner: &dyn GitRunner, bytes: &[u8]) -> Result<String, String> {
    return write_blob(runner, bytes).await;
}

/// The decisions of one layer of one path,
/// and the words each reason adds.
#[cfg(test)]
#[path = "merge_isolation_carry_layers_tests.rs"]
mod tests;
