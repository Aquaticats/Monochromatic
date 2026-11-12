//! What:
//!  The Git steps of a landing and the text it feeds Git:
//! running one step and telling a step ended at its ceiling from other
//!  failures,
//! keeping an unlanded commit on a side ref,
//! the `fast-import` stream of a batch and the marks it exports,
//! and the
//!  tests of which requests of a batch touch the same paths.
//! Why:
//!  `service_land.rs` holds the landing itself;
//! these steps depend on nothing in it,
//! so they are a file it uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await run(git, repository, args, input); importStream(scratch, old, batch, amendedParents); commitsFromMarks(text, count)
//! ```

use crate::service_land_follow::classify;
use crate::service_land_side_reference_write::{StepAnswer, answering, keep_unlanded, take_ceiling_step};
use crate::service_protocol::{CommitRequest, FailureCode};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, NO_REFLOG_SETTING};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

pub(crate) fn run(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    let words: Vec<std::ffi::OsString> = arguments.iter().map(std::ffi::OsString::from).collect();
    return run_words(git, repository, words.as_slice(), input);
}

/// What:
///  `run` for a command line that is not all UTF-8,
///  which the reflog message of a commit whose message is not UTF-8 needs.
/// Why:
///   The message is one argument of `git update-ref -m`,
/// so its bytes reach Git as they are and no text conversion can change them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(['update-ref', '-m', message, '--stdin'], { input: transaction })
/// ```
pub(crate) fn run_words(git: &GitContext, repository: &Path, arguments: &[std::ffi::OsString], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.input = input;
    return match answering(git, request) {
        StepAnswer::Answered(output) => Ok(output),
        // A step whose wait ended is left running, and nothing that reads this answer can tell: `Unavailable` is the code for a step
        //  the landing cannot resolve. A write that asks the reference itself reads `answering` instead
        //  (`write_until_the_reference_names`).
        StepAnswer::LeftRunning => Err(FailureCode::Unavailable),
        StepAnswer::Ended(code) => Err(code),
    };
}

pub(crate) fn checked(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<String, FailureCode> {
    let output: GitOutput = run(git, repository, arguments, input)?;
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// What:
///  The standard output of one Git step that must succeed,
/// as the bytes Git wrote.
/// Why:
///   `checked` answers with trimmed text,
/// which is right for an object name and wrong for a listing of paths:
/// a listing with a name that is not text has to reach the caller as the name Git wrote.
pub(crate) fn checked_bytes(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<Vec<u8>, FailureCode> {
    let output: GitOutput = run(git, repository, arguments, input)?;
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(output.stdout);
}

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
fn ends_a_field(byte: &u8) -> bool {
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
/// with their positions:
/// a request whose paths changed on the branch
///  since its base is left out,
/// and so is one that an earlier request of the batch overlaps unless it captured the worktree later.
/// Why:
///  The requests left out are sent back to be prepared again and merged against what landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// requests.map((r, i) => [i, r]).filter(([, r]) => !conflictsSince(r, old) && !shadowedByEarlier(r))
/// ```
pub(crate) fn requests_that_land<'a>(git: &GitContext, repository: &Path, requests: &'a [CommitRequest], old: &str) -> Result<Vec<(usize, &'a CommitRequest)>, FailureCode> {
    let mut changed_since: HashMap<&str, BTreeSet<Vec<u8>>> = HashMap::new();
    let mut landing: Vec<(usize, &CommitRequest)> = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        if request.base != old && !changed_since.contains_key(request.base.as_str()) {
            let listing: Vec<u8> = checked_bytes(git, repository, &["diff-tree", "-r", "--name-only", "-z", request.base.as_str(), old], None)?;
            let mut names: BTreeSet<Vec<u8>> = BTreeSet::new();
            for name in listing.split(ends_a_field) {
                if !name.is_empty() {
                    names.insert(name.to_vec());
                }
            }
            changed_since.insert(request.base.as_str(), names);
        }
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
    return Ok(landing);
}

/// What:
///  Tells the clients of a batch which step was ended at its ceiling,
/// and keeps the commit the batch built on a side ref
///  before the landing gives its scratch branch up.
/// Why:
///   A landing that ends at a ceiling lands nothing,
/// so the commit its import built is on the scratch branch
///  and no intent will finish it;
/// the marker names why the commit was not made,
/// and the message names the ref the commit is kept
///  at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepWhatAnEndedStepBuilt(context, announce, requests, seq)
/// ```
pub fn keep_what_an_ended_step_built(context: &crate::service_land_context::LandContext, announce: &dyn crate::service_index_lock::Announcer, requests: &[CommitRequest], seq: u64) {
    let Some(step) = take_ceiling_step() else { return };
    let Some(first) = requests.first() else { return };
    announce.announce(crate::service_index_lock::Announce::Waiting(format!("monochromatic-git-policy-cli: {step}, so this commit was not made.\n")));
    let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
    let scratch: String = crate::service_scratch_ref_of_a_dropped_batch::scratch_ref(seq);
    if let Ok(tip) = checked(&git, first.repository.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()], None) {
        announce.announce(crate::service_index_lock::Announce::Waiting(keep_unlanded(&git, first.repository.as_path(), seq, tip.as_str())));
        let _deleted = run(&git, first.repository.as_path(), &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch.as_str()], None);
    }
}
