//! What:
//!  The person's own work in the paths a merge changes:
//! what the worktree held,
//! what the index held,
//! and the
//!  objects and ref that keep both from `git gc` while the merge runs.
//! Why:
//!  A merge replaces exactly these paths,
//! so their bytes must exist somewhere recovery can find them before the
//!  first one changes (`package/git-policy/cli/SPEC.md`,
//! "Merges and commits never fail").
//!  The record is written once and never rewritten,
//! so the capture travels with it,
//! and a capture is read back only
//!  through this file,
//! which is also what keeps the record's shape and the bytes it names from drifting apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Capture = { commit: string; reference: string; paths: CapturedPath[] };
//! ```

/// Records parsed as JSON.
use serde_json::{Map, Value};

/// What:
///  The worktree's file at one captured path:
/// its kind,
/// the bytes exactly as they stood,
/// and the bytes as Git records them for
///  the path.
/// Why:
///  The raw bytes are what a person sees with their editor,
/// so restoring them is the only way to give back
///  exactly what they had;
/// clean filters and line endings make the bytes Git records differ from those,
/// and the
///  merge is computed on the recorded form, as `git stash` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CapturedFile = { mode: string; raw: string; blob: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedFile {
    /// `100644`,
    /// `100755` or `120000`,
    /// as Git records that kind of entry.
    pub mode: String,
    /// The object of the bytes the file held,
    /// with no filter applied.
    pub raw: String,
    /// The object Git records for the file at its path,
    /// with the path's clean filters and line endings applied.
    pub blob: String,
}

/// What:
///  The index's entry at one captured path.
/// Why:
///  A staged edit has no other name:
/// the blob it points at is collected as soon as the merge's entry replaces it,
/// so the entry itself is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CapturedIndex = { mode: string; oid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedIndex {
    /// The mode of the index entry.
    pub mode: String,
    /// The object the index holds for the path.
    pub oid: String,
}

/// What:
///  One path the person had work in,
/// with each layer that held work.
/// Why:
///  The two layers are separate edits:
/// a staged edit is what a commit would record,
/// and the worktree file may hold
///  more on top of it,
/// so each is merged with the merge on its own and put back in its own place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CapturedPath = { path: Uint8Array; worktree?: CapturedFile; index?: CapturedIndex };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedPath {
    /// The path from the top of the worktree,
    /// exact bytes.
    pub path: Vec<u8>,
    /// The worktree's file,
    /// or nothing when nothing stood there.
    pub worktree: Option<CapturedFile>,
    /// The index's entry,
    /// or nothing when the index held none.
    pub index: Option<CapturedIndex>,
}

/// What:
///  Everything one merge keeps of the person's work.
/// Why:
///  The commit is the tree of slots the record's objects are named from,
/// and the reference is what keeps that commit from
///  `git gc`;
/// both are read only through this file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Capture = { commit: string; reference: string; paths: CapturedPath[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capture {
    /// The commit whose tree holds every captured object under a slot name.
    pub commit: String,
    /// The ref that keeps `commit` reachable,
    /// or nothing for a record read before its ref was written.
    pub reference: Option<String>,
    /// The person's work,
    /// path by path.
    pub paths: Vec<CapturedPath>,
}

/// The name of one slot in the capture tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `s${index}`
/// ```
pub fn slot_name(index: usize) -> String {
    return format!("s{index}");
}

/// One object the capture tree must hold,
/// as `git mktree` takes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Slot = { name: string; oid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Slot {
    /// The entry's name inside the tree.
    pub name: String,
    /// The object it names.
    pub oid: String,
}

/// What:
///  Every distinct object a capture names,
/// as slots of one flat tree.
/// Why:
///  A tree keeps the objects from being collected,
/// and a flat list of numbered names is a tree `git mktree`
///  makes without a scratch index,
/// while the paths themselves stay in the record where their bytes are exact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function slotsOf(capture: { paths: CapturedPath[] }): Slot[]; // deduplicated oids, s0, s1, ...
/// ```
pub fn slots(paths: &[CapturedPath]) -> Vec<Slot> {
    let mut found: Vec<Slot> = Vec::new();
    for entry in paths {
        let mut wanted: Vec<&str> = Vec::new();
        if let Some(file) = &entry.worktree {
            wanted.push(file.raw.as_str());
            wanted.push(file.blob.as_str());
        }
        if let Some(staged) = &entry.index {
            wanted.push(staged.oid.as_str());
        }
        for oid in wanted {
            if found.iter().any(|slot| { return slot.oid == oid; }) {
                continue;
            }
            found.push(Slot { name: slot_name(found.len()), oid: String::from(oid) });
        }
    }
    return found;
}

/// One JSON object for a captured file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fileJson = (file: CapturedFile) => ({ mode: file.mode, raw: file.raw, blob: file.blob });
/// ```
fn file_json(file: &CapturedFile) -> Value {
    let mut map: Map<String, Value> = Map::new();
    map.insert(String::from("mode"), Value::String(file.mode.clone()));
    map.insert(String::from("raw"), Value::String(file.raw.clone()));
    map.insert(String::from("blob"), Value::String(file.blob.clone()));
    return Value::Object(map);
}

/// One JSON object for a captured index entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const indexJson = (staged: CapturedIndex) => ({ mode: staged.mode, oid: staged.oid });
/// ```
fn index_json(staged: &CapturedIndex) -> Value {
    let mut map: Map<String, Value> = Map::new();
    map.insert(String::from("mode"), Value::String(staged.mode.clone()));
    map.insert(String::from("oid"), Value::String(staged.oid.clone()));
    return Value::Object(map);
}

/// What:
///  The capture as the record's `carried` field holds it.
/// Why:
///   Every text is quoted by the JSON writer,
/// so a path that holds a quote or a newline cannot corrupt the record,
/// and the
///  bytes of the path travel beside its text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encode(capture: Capture): string;
/// ```
pub fn encode(capture: &Capture) -> String {
    let mut map: Map<String, Value> = Map::new();
    map.insert(String::from("commit"), Value::String(capture.commit.clone()));
    if let Some(reference) = &capture.reference {
        map.insert(String::from("reference"), Value::String(reference.clone()));
    }
    let mut paths: Vec<Value> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let mut path: Map<String, Value> = Map::new();
        path.insert(String::from("path"), Value::String(String::from_utf8_lossy(entry.path.as_slice()).into_owned()));
        path.insert(String::from("pathHex"), Value::String(crate::path_bytes_in_records::hex_of_bytes(entry.path.as_slice())));
        if let Some(file) = &entry.worktree {
            path.insert(String::from("worktree"), file_json(file));
        }
        if let Some(staged) = &entry.index {
            path.insert(String::from("index"), index_json(staged));
        }
        paths.push(Value::Object(path));
    }
    map.insert(String::from("paths"), Value::Array(paths));
    return Value::Object(map).to_string();
}

/// The text of a JSON object's field,
/// when it is a non-empty string.
fn field<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    return map.get(key)?.as_str().filter(|text| { return !text.is_empty(); });
}

/// The object of one nested field.
fn object<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a Map<String, Value>> {
    return map.get(key)?.as_object();
}

/// What:
///  Read a capture back,
/// or nothing when any part of it is not the shape this file writes.
/// Why:
///   A malformed capture must fail closed rather than be guessed at:
/// the bytes it names are the person's only copy once the
///  checkout changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decode(value: unknown): Capture | undefined;
/// ```
pub fn decode(value: &Value) -> Option<Capture> {
    let map: &Map<String, Value> = value.as_object()?;
    let commit: &str = field(map, "commit")?;
    let mut paths: Vec<CapturedPath> = Vec::new();
    for item in map.get("paths")?.as_array()? {
        let entry: &Map<String, Value> = item.as_object()?;
        let decoded: std::path::PathBuf = crate::path_bytes_in_records::path_of_hex(field(entry, "pathHex")?)?;
        let path: Vec<u8> = crate::path_bytes::path_bytes_of(decoded.as_path()).to_vec();
        let worktree: Option<CapturedFile> = match object(entry, "worktree") {
            Some(file) => Some(CapturedFile {
                mode: String::from(field(file, "mode")?),
                raw: String::from(field(file, "raw")?),
                blob: String::from(field(file, "blob")?),
            }),
            None => None,
        };
        let index: Option<CapturedIndex> = match object(entry, "index") {
            Some(staged) => Some(CapturedIndex { mode: String::from(field(staged, "mode")?), oid: String::from(field(staged, "oid")?) }),
            None => None,
        };
        if worktree.is_none() && index.is_none() {
            return None;
        }
        paths.push(CapturedPath { path, worktree, index });
    }
    return Some(Capture {
        commit: String::from(commit),
        reference: field(map, "reference").map(String::from),
        paths,
    });
}

/// Reading and writing the capture a record holds.
#[cfg(test)]
#[path = "merge_isolation_carry_parts_tests.rs"]
mod tests;
