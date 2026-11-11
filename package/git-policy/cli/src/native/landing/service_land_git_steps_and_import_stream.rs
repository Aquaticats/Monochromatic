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
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, value_of};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, NO_REFLOG_SETTING, run_git};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// How many times a side reference is written when the wait for a write ended without an answer.
///  Two:
///  a write a wait left running is often finished moments later,
///  so one write more after the reference is read costs a step and saves the commit.
///  A write Git itself refused is written no second time:
///  the same value at the same name is refused the same way.
const SIDE_REF_WRITE_ATTEMPTS: usize = 2;

/// What:
///  Writes the side reference `name` at `tip`,
///  and answers nothing when the reference names it,
///  or the words of the write that did not.
/// Why:
///   A side reference is what keeps a commit a reference lock held reachable
///  (`held_commit_record.rs`),
///  and the wrapper never ends such a write for being slow,
///  because cutting a write in place risks the person's data
///  (`transaction_git_step_bounds.rs`,
///  `is_side_ref_write`):
///  a wait that ends without an answer says the result is not known,
///  never that the write failed.
///  Reading that ending as a failed write told a commit that was made that it was not made
///  (2026-11-11,
///  the release gate of 0.6.1-alpha.11:
///  the write of `refs/worktree/cli-git-service/<batch>` was still moving after its wait of 127 seconds,
///  in quiet gaps under a second,
///  and the landing answered `unavailable`).
///  The write names one value for one name,
///  so it may be written again,
///  and the reference is the answer:
///  it is read,
///  never guessed at from a wait that ended.
///  `only_if_free` writes the reference only while its name names nothing,
///  and a name it already holds at `tip` is the answer either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let attempt = 0; attempt < 2; attempt++) { const made = await writeRef(name, tip, freeOnly); if (made.succeeded) return; if (await valueOf(name) === tip) return; if (made.answered) return wordsOf(made); } return words
/// ```
fn write_until_the_reference_names(git: &GitContext, repository: &Path, name: &str, tip: &str, only_if_free: bool) -> Option<String> {
    let mut refused: Option<String> = None;
    for _attempt in 0..SIDE_REF_WRITE_ATTEMPTS {
        let mut arguments: Vec<&str> = vec!["-c", NO_REFLOG_SETTING, "update-ref", "--no-deref", name, tip];
        if only_if_free {
            // The empty old value is Git's way to refuse when the name exists at all, so a name another command wrote is never
            //  replaced (`keep_on_a_side_ref_only_if_free`).
            arguments.push("");
        }
        let made: Result<GitOutput, FailureCode> = run(git, repository, arguments.as_slice(), None);
        // Git's own answer, as opposed to a wait that ended before the write did.
        let answered: bool = matches!(&made, Ok(_));
        if matches!(&made, Ok(output) if output.succeeded()) {
            return None;
        }
        // The reference says whether a write finished, whatever the wait for it said.
        if matches!(value_of(git, repository, name), RefValue::Names(named) if named == tip) {
            return None;
        }
        refused = words_of_a_refused_write(made);
        if answered {
            return refused;
        }
    }
    return refused;
}

/// What:
///  Writes `refs/cli-git/kept/<number>` at `tip`,
/// the ref the wrapper keeps a commit it did not land on,
/// and returns nothing when the reference names it or Git's own words when it does not.
/// Why:
///   One place names that ref and writes it,
/// so the side ref of a landing that gave up at a step and the side ref the restart writes for a batch its branch does
/// not show cannot drift apart;
/// the write follows no symbolic ref,
/// and it passes the setting that keeps Git from making a reflog for a ref that has none
/// (`SPEC.md`,
/// a setting that does not stop an append to a log another Git made).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnASideRef(git, repo, seq, tip)).toBeUndefined()
/// ```
pub(crate) fn keep_on_a_side_ref(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> Option<String> {
    let name: String = crate::service_scratch_ref_of_a_dropped_batch::kept_ref(seq);
    return write_until_the_reference_names(git, repository, name.as_str(), tip, false);
}

/// What:
///  Writes the per-worktree reference `refs/worktree/cli-git-service/<seq>` at `tip`,
///  the reference that keeps the objects of a batch reachable until its branch write,
///  and answers nothing when the reference names it or Git's own words when it does not.
/// Why:
///  This is the write that keeps a commit Git built reachable while its branch write meets a lock
///  (`service_land_objects_of_batch.rs`),
///  and it is the one the release gate of 0.6.1-alpha.11 read as a failed write when its wait ran out;
///  it is written here so that reading is answered by the reference
///  (`write_until_the_reference_names`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnTheScratchReference(git, repo, scratchRef(seq), tip)).toBeUndefined()
/// ```
pub(crate) fn keep_on_the_scratch_reference(git: &GitContext, repository: &Path, scratch: &str, tip: &str) -> Option<String> {
    return write_until_the_reference_names(git, repository, scratch, tip, false);
}

/// What:
///  Writes the reference `name` at `tip`
/// only while that name names nothing,
/// and answers nothing when the reference names `tip`
///  or Git's own words when it does not.
/// Why:
///   A command that removes a linked worktree keeps a held commit on a reference of the common Git directory
///  before Git removes the only other ones (`worktree_removal_held_commits.rs`),
///  and a name that another command already wrote must never be replaced:
///  replacing it would forget the commit that reference was made for,
///  which is the owner's rule "never discard user data" again.
///  The empty old value is Git's `--stdin`-free way to ask for exactly that:
///  `update-ref <ref> <new> ""` refuses when the name exists at all,
///  whether this command read it first or another one wrote it in between.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnASideRefOnlyIfFree(git, repo, name, tip)).toBeUndefined()
/// ```
pub(crate) fn keep_on_a_side_ref_only_if_free(git: &GitContext, repository: &Path, name: &str, tip: &str) -> Option<String> {
    return write_until_the_reference_names(git, repository, name, tip, true);
}

/// What:
///  Nothing when Git took a side-reference write,
/// and Git's own words,
/// or the reason it could not be run,
/// when it did not.
/// Why:
///  Every write of a side reference reads its answer here,
/// so a write Git refused and one that could not start
///  are the same shape for every caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wordsOfARefusedWrite(made) { return made.ok && made.value.succeeded ? undefined : (made.error ?? (made.value.stderr.trim() || `git update-ref ended with ${made.value.code}`)) }
/// ```
fn words_of_a_refused_write(made: Result<GitOutput, FailureCode>) -> Option<String> {
    return match made {
        Ok(output) if output.succeeded() => None,
        Ok(output) => {
            let words: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_string();
            if words.is_empty() {
                let status: String = match output.code {
                    Some(code) => code.to_string(),
                    None => String::from("no exit status"),
                };
                return Some(format!("git update-ref ended with {status}"));
            }
            Some(words)
        }
        Err(code) => Some(format!("Git could not be run ({code:?})")),
    };
}

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
/// await keepOnASideRef(git, repository, seq, tip); return `kept at refs/cli-git/kept/${seq}`
/// ```
pub(crate) fn keep_unlanded(git: &GitContext, repository: &Path, seq: u64, tip: &str) -> String {
    let name: String = crate::service_scratch_ref_of_a_dropped_batch::kept_ref(seq);
    if keep_on_a_side_ref(git, repository, seq, tip).is_some() {
        return String::from("monochromatic-git-policy-cli: the commit built for you could not be kept on a side ref; your staged changes and your worktree are as they were, so run the commit again when the program above is done.\n");
    }
    return format!(
        "monochromatic-git-policy-cli: the commit built for you is kept at {name} (see it with `git log {name}`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
    );
}

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
    return match run_git(git, &request) {
        Ok(output) => Ok(output),
        Err(error) => {
            // A step ended at its ceiling, or by the service's stop, is told apart from every other failure, and its sentence is kept for
            //  the answer; the commit it belongs to is kept on a side ref and not made (`land_batch_timed`).
            if crate::child_watch::is_ceiling(&error) || crate::child_watch::is_ended_by_stop(&error) {
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
