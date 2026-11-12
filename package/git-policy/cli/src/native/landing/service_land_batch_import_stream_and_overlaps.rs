//! What:
//!  The `fast-import` stream that makes one commit per request of a batch,
//! the marks it exports,
//! and the
//!  tests of which requests of a batch touch the same paths.
//! Why:
//!  A landing writes the batch's commits with one `fast-import` run,
//! so a name that is not text is
//!  quoted as bytes rather than as a spelling two names can share,
//! and a later request that touches a path an
//!  earlier one also touches would replace that member's bytes in the stream,
//! which is why the two are
//!  compared here.
//! The stream and the comparisons are
//!  this file rather than `service_land_git_steps_and_import_stream.rs` to keep both under the
//!  file-size limit (`AGENTS.md`,
//! `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stream = importStream(scratch, old, batch, amendedParents); const commits = commitsFromMarks(text, count)
//! ```

use crate::service_protocol::CommitRequest;
use std::collections::{BTreeSet, HashMap};

/// A path in the quoted form `fast-import` reads,
/// so no byte of a name can end the line or the field.
/// Why:
///   A name is bytes,
/// so the quoted form is built as bytes:
/// `fast-import` reads a byte outside the printable range as the octal escape `\\ooo`,
/// and a byte that is text may be written as itself.
/// Writing the name as a Rust string would encode every byte above 0x7f as two,
/// which is another name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quoted = (path: Uint8Array) => '"' + [...path].map((b) => b === 0x22 || b === 0x5c || b < 0x20 || b === 0x7f ? '\\' + b.toString(8).padStart(3, '0') : String.fromCharCode(b)).join('') + '"'
/// ```
pub(crate) fn quoted(path: &[u8]) -> Vec<u8> {
    let mut text: Vec<u8> = vec![b'"'];
    for byte in path {
        if *byte == b'"' || *byte == b'\\' || *byte < b' ' || *byte == 0x7f {
            text.extend(format!("\\{byte:03o}").as_bytes());
        } else {
            text.push(*byte);
        }
    }
    text.push(b'"');
    return text;
}

/// Whether a byte ends a field of Git's `-z` output.
pub(super) fn ends_a_field(byte: &u8) -> bool {
    return *byte == b'\0';
}

/// Whether two repository paths touch the same file or one contains the other.
/// Why:
///   Both are the bytes Git listed,
/// so a name that is not text is compared as itself and never as a spelling that two names can share.
pub(crate) fn overlaps(path: &[u8], changed: &[u8]) -> bool {
    return path == changed || is_below(path, changed) || is_below(changed, path);
}

/// Whether `directory` holds `path`,
/// which is the same test as `path.startsWith(directory + '/')` on the bytes.
fn is_below(path: &[u8], directory: &[u8]) -> bool {
    return path.len() > directory.len() && path.starts_with(directory) && path[directory.len()] == b'/';
}

/// The stream that makes one commit per request on a scratch branch that starts at `old`,
/// or,
/// for an amend,
/// one commit
///  whose parents are `amended_parents` and whose whole tree is the request's tree.
pub(crate) fn import_stream(scratch: &str, old: &str, requests: &[&CommitRequest], amended_parents: &[String]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        bytes.extend(format!("commit {scratch}\nmark :{}\n", index + 1).as_bytes());
        if let Some(author) = &request.author {
            bytes.extend(format!("author {author}\n").as_bytes());
        }
        bytes.extend(format!("committer {}\ndata {}\n", request.committer, request.message.len()).as_bytes());
        bytes.extend(request.message.as_bytes());
        bytes.push(b'\n');
        if request.amend
            && let Some(tree) = request.tree.as_deref()
        {
            // An amend replaces the tip: its parents are the base's parents, and its tree is the one the client wrote.
            for (position, parent) in amended_parents.iter().enumerate() {
                bytes.extend(format!("{} {parent}\n", if position == 0 { "from" } else { "merge" }).as_bytes());
            }
            bytes.extend(format!("M 040000 {tree} \"\"\n").as_bytes());
            bytes.push(b'\n');
            continue;
        }
        if index == 0 {
            bytes.extend(format!("from {old}\n").as_bytes());
        }
        for change in &request.changes {
            let path: Vec<u8> = quoted(crate::path_bytes::path_bytes_of(change.path.as_path()));
            match &change.blob {
                Some(blob) => bytes.extend(format!("M {:o} {blob} ", change.mode).as_bytes()),
                None => bytes.extend(b"D "),
            }
            bytes.extend(path);
            bytes.push(b'\n');
        }
        bytes.push(b'\n');
    }
    bytes.extend(b"done\n");
    return bytes;
}

/// The object names `fast-import` recorded for the marks,
/// in request order.
pub(crate) fn commits_from_marks(text: &str, count: usize) -> Option<Vec<String>> {
    let mut commits: Vec<Option<String>> = vec![None; count];
    for line in text.lines() {
        let mut parts = line.split(' ');
        let (Some(mark), Some(name)) = (parts.next(), parts.next()) else { continue };
        let slot: usize = mark.trim_start_matches(':').parse::<usize>().ok()?;
        if slot >= 1 && slot <= count {
            commits[slot - 1] = Some(name.to_string());
        }
    }
    return commits.into_iter().collect();
}

/// What:
///  The identities of the requests of a batch.
/// Why:
///  The intent of a batch names the requests it lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// batch.map((request) => request.id)
/// ```
pub(crate) fn ids_of(batch: &[&CommitRequest]) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for request in batch {
        ids.push(request.id.clone());
    }
    return ids;
}

/// What:
///  Whether a change of the request touches a path that changed on the branch since the request's base.
/// Why:
///  A request whose paths changed meanwhile is sent back to be prepared again against the new tip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// request.changes.some((change) => changedSince?.some((name) => overlaps(change.path, name)))
/// ```
pub(crate) fn changes_overlap_names(request: &CommitRequest, names: Option<&BTreeSet<Vec<u8>>>) -> bool {
    let Some(changed) = names else {
        return false;
    };
    for change in &request.changes {
        for name in changed {
            if overlaps(crate::path_bytes::path_bytes_of(change.path.as_path()), name.as_slice()) {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  Whether a change of the request touches a path that a change of an earlier request of the batch also touches.
/// Why:
///  A later request would replace the earlier one's bytes in the import stream,
/// so it lands only when it captured later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// request.changes.some((c) => earlier.changes.some((m) => overlaps(c.path, m.path)))
/// ```
pub(crate) fn changes_overlap_members(request: &CommitRequest, earlier: &CommitRequest) -> bool {
    for change in &request.changes {
        for member in &earlier.changes {
            if overlaps(crate::path_bytes::path_bytes_of(change.path.as_path()), crate::path_bytes::path_bytes_of(member.path.as_path())) {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  The requests of a batch that land against the tip `old`,
///  with their positions,
///  from the paths each base changed since.
/// Why:
///  Both faces fold the same listing through this one function,
/// so a request is left out for the same reason whichever
///  face read the history:
/// a request whose paths changed on the branch since its base,
/// and one that an
///  earlier request of the batch overlaps unless it captured the worktree later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landing = (requests, old, changedSince) => requests.flatMap((r, i) => (conflictsSince(r) || shadowedByEarlier(r)) ? [] : [[i, r]]);
/// ```
pub(super) fn landing_of<'a>(requests: &'a [CommitRequest], old: &str, changed_since: &HashMap<&str, BTreeSet<Vec<u8>>>) -> Vec<(usize, &'a CommitRequest)> {
    let mut landing: Vec<(usize, &CommitRequest)> = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        let conflicts: bool = request.base != old && changes_overlap_names(request, changed_since.get(request.base.as_str()));
        // A member of this batch that an earlier member overlaps would replace that member's bytes in the stream below. It
        //  stays in the batch only when it captured the worktree (a batch has one) later than that member, so its bytes
        //  hold the member's edit; any other request is sent back and merged against the member's landed commit.
        let mut shadowed: bool = false;
        for (_earlier_index, earlier) in &landing {
            let later_capture: bool = earlier.captured != 0 && request.captured > earlier.captured;
            if !later_capture && changes_overlap_members(request, earlier) {
                shadowed = true;
                break;
            }
        }
        if !conflicts && !shadowed {
            landing.push((index, request));
        }
    }
    return landing;
}
