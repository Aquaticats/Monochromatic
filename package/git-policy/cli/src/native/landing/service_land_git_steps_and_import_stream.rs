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

use crate::landing_state_carried_by_its_task::{LandingState, on_the_landing_state};
use crate::service_land_follow::classify;
use crate::service_protocol::{CommitRequest, FailureCode};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, NO_REFLOG_SETTING, run_git};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
/// What:
///  Keeps the commit built for a batch that was withdrawn on a side ref,
/// and returns the line that tells the person where it is.
/// Why:
///   The owner's answer for a wait that reaches its bound is that nothing lands later by itself and the commit is kept for a retry;
///  the person's staged changes are untouched,
/// so running the commit again builds the same commit,
/// and the side ref lets them
///  recover this one if the worktree changed meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['update-ref', `refs/cli-git/kept/${seq}`, tip]); return `kept at refs/cli-git/kept/${seq}`
/// ```
pub(crate) fn keep_unlanded(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> String {
    let name: String = format!("refs/cli-git/kept/{seq}");
    let made = run(git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", name.as_str(), tip], None);
    if !matches!(made, Ok(output) if output.succeeded()) {
        return String::from("monochromatic-git-policy-cli: the commit built for you could not be kept on a side ref; your staged changes and your worktree are as they were, so run the commit again when the program above is done.\n");
    }
    return format!(
        "monochromatic-git-policy-cli: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
    );
}

pub(crate) fn run(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<GitOutput, FailureCode> {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.input = input;
    return match run_git(git, &request) {
        Ok(output) => Ok(output),
        Err(error) => {
            // A step ended at its ceiling is told apart from every other failure, and its sentence is kept for the answer.
            if crate::child_watch::is_ceiling(&error) {
                on_the_landing_state(error.to_string(), keep_the_ceiling_step);
                return Err(FailureCode::StepTooSlow);
            }
            Err(FailureCode::Unavailable)
        }
    };
}

/// Keeps the sentence of a step that was ended at its ceiling.
fn keep_the_ceiling_step(state: &LandingState, sentence: String) {
    *state.ceiling_step.borrow_mut() = Some(sentence);
}

/// Takes the sentence of the step that was ended at its ceiling out of the landing's state.
fn take_the_ceiling_step(state: &LandingState, (): ()) -> Option<String> {
    return state.ceiling_step.take();
}

/// The sentence of the step that was ended at its ceiling,
/// once;
/// the landing keeps it for the line the clients are told (the state `ceiling_step` of `landing_state_carried_by_its_task.rs`).
pub(crate) fn take_ceiling_step() -> Option<String> {
    return on_the_landing_state((), take_the_ceiling_step);
}

pub(crate) fn checked(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>) -> Result<String, FailureCode> {
    let output: GitOutput = run(git, repository, arguments, input)?;
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// A path in the quoted form `fast-import` reads,
/// so no byte of a name can end the line or the field.
pub(crate) fn quoted(path: &str) -> String {
    let mut text: String = String::from("\"");
    for character in path.chars() {
        match character {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            other => text.push(other),
        }
    }
    text.push('"');
    return text;
}

/// Whether two repository paths touch the same file or one contains the other.
pub(crate) fn overlaps(path: &str, changed: &str) -> bool {
    return path == changed || path.starts_with(&format!("{changed}/")) || changed.starts_with(&format!("{path}/"));
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
            match &change.blob {
                Some(blob) => bytes.extend(format!("M {:o} {blob} {}\n", change.mode, quoted(change.path.as_str())).as_bytes()),
                None => bytes.extend(format!("D {}\n", quoted(change.path.as_str())).as_bytes()),
            }
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
pub(crate) fn changes_overlap_names(request: &CommitRequest, names: Option<&BTreeSet<String>>) -> bool {
    let Some(changed) = names else {
        return false;
    };
    for change in &request.changes {
        for name in changed {
            if overlaps(change.path.as_str(), name.as_str()) {
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
            if overlaps(change.path.as_str(), member.path.as_str()) {
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
    let mut changed_since: HashMap<&str, BTreeSet<String>> = HashMap::new();
    let mut landing: Vec<(usize, &CommitRequest)> = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        if request.base != old && !changed_since.contains_key(request.base.as_str()) {
            let listing: String = checked(git, repository, &["diff-tree", "-r", "--name-only", "-z", request.base.as_str(), old], None)?;
            let mut names: BTreeSet<String> = BTreeSet::new();
            for name in listing.split('\0') {
                if !name.is_empty() {
                    names.insert(name.to_string());
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
