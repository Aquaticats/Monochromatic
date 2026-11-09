//! What:
//!  The capture-order decision of every shared path of a replay,
//!  read from one walk over the landed history.
//! Why:
//!  The service remembers the stamps of the commits it landed for this worktree.
//!  When every commit that changed a path since the base is remembered with a stamp,
//!  and all are earlier than this capture,
//!  this capture holds their edits and its bytes land;
//!  when all are later,
//!  theirs hold this commit's edit and the landed bytes stay.
//!  Anything else (a commit the service does not know,
//!  a stamp that is missing,
//!  one earlier and one later) is not decided by order.
//!  Both the shared paths and the history since the base grow with the repository,
//!  so the history is walked once and each changed path is looked at as the walk meets it,
//!  rather than walked once per path (`AGENTS.md` `RG2`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const decisions = await captureOrderOfEach(site, captured, tip, shared); // Map<path, 'prepared' | 'landed' | 'unordered'>
//! ```

use crate::commit_transaction::{TransactionStop, failed};
use crate::replay_capture_order::CaptureOrderDecision;
use crate::replay_shared_paths::SharedPath;
use crate::service_protocol::LandedCapture;
use crate::service_replay::{Captured, MergeSite};
use crate::transaction_git::{GitRequest, run_git_checked};
use std::collections::{BTreeMap, HashMap};

/// What:
///  What the walk has seen of one candidate path so far.
/// Why:
///   The decision is the whole set of commits that changed the path,
///  so each one is folded in as the walk meets it:
///  whether any commit is one the service never landed,
///  whether all are earlier than this capture,
///  and whether all are later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OrderSoFar = { commits: number; unknown: boolean; allBefore: boolean; allAfter: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OrderSoFar {
    /// How many commits that changed the path the walk reported.
    commits: u64,
    /// Whether one of them is a commit this service never landed.
    unknown: bool,
    /// Every one of them was captured no later than this capture.
    all_before: bool,
    /// Every one of them was captured no earlier than this capture.
    all_after: bool,
}

/// What:
///  The state of a path nothing has been seen of yet.
/// Why:
///   A path the history never changed is not decided by order,
///  which its count of zero says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nothingSeen: OrderSoFar = { commits: 0, unknown: false, allBefore: true, allAfter: true };
/// ```
const NOTHING_SEEN: OrderSoFar = OrderSoFar { commits: 0, unknown: false, all_before: true, all_after: true };

/// What:
///  The decision the state of one path supports.
/// Why:
///   A path no commit changed,
///  a path one commit changed that the service never landed,
///  and a path changed both before and after this capture are not decided by order;
///  a set of commits all earlier than this capture means the captured bytes hold their edits,
///  and a set all later means the landed bytes hold this commit's edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decide = (state) => state.commits === 0 || state.unknown ? 'unordered' : state.allBefore ? 'prepared' : state.allAfter ? 'landed' : 'unordered';
/// ```
fn decide(state: OrderSoFar) -> CaptureOrderDecision {
    if state.commits == 0 || state.unknown {
        return CaptureOrderDecision::Unordered;
    }
    if state.all_before {
        return CaptureOrderDecision::Prepared;
    }
    if state.all_after {
        return CaptureOrderDecision::Landed;
    }
    return CaptureOrderDecision::Unordered;
}

/// What:
///  What the given commits say about one path,
///  folded into what was known of it.
/// Why:
///   One commit that this service never landed,
///  or a mix of earlier and later captures,
///  leaves the path undecided,
///  and the fold says so as it goes rather than collecting the set first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fold(state, commit, stamps, captured) { const stamp = stamps.get(commit); state.commits += 1; if (stamp === undefined) state.unknown = true; else if (stamp >= captured) state.allBefore = false; else if (stamp <= captured) state.allAfter = false; }
/// ```
fn fold(state: &mut OrderSoFar, commit: &str, stamps: &HashMap<&str, u64>, captured: u64) {
    state.commits += 1;
    let Some(stamp) = stamps.get(commit) else {
        state.unknown = true;
        return;
    };
    if *stamp >= captured {
        state.all_before = false;
    }
    if *stamp <= captured {
        state.all_after = false;
    }
}

/// What:
///  The capture stamp of each commit this service landed,
///  by commit.
/// Why:
///   Every changed path asks the same question of the same list,
///  which grows with the landed commits,
///  so one pass answers it for all of them (`AGENTS.md` `RG2`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stamps = new Map(captured.landed.map((entry) => [entry.commit, entry.captured]));
/// ```
fn stamps_by_commit(landed: &[LandedCapture]) -> HashMap<&str, u64> {
    let mut stamps: HashMap<&str, u64> = HashMap::with_capacity(landed.len());
    for entry in landed {
        crate::collected_elements_looked_at::note_looked_at(1);
        stamps.insert(entry.commit.as_str(), entry.captured);
    }
    return stamps;
}

/// What:
///  The arguments of the one walk:
///  the commits since the base and the paths each one changed.
/// Why:
///   `--raw -z` frames every change as one record then one path,
///  so a path that holds any byte or looks like anything else is read as the path it is;
///  `--no-renames` names both sides of a rename,
///  so a path that either name carried is one the walk reports;
///  `-m` reports a merge against each parent,
///  so a merge that holds one parent's bytes for a path and not the other's is a commit that changed it.
///  The per-path question followed history simplification,
/// which left such a merge out;
///  a walk that reports it can only hold a path back to subsumption and the merge,
///  except where no commit the per-path question reported changed the path at all,
///  and this capture there is later than every change of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['log', '-m', '-r', '--raw', '-z', '--no-renames', '--no-abbrev', '--format=%H', tip, `^${base}`]
/// ```
fn history_arguments<'a>(tip: &'a str, exclusion: &'a str) -> Vec<&'a str> {
    return vec!["log", "-m", "-r", "--raw", "-z", "--no-renames", "--no-abbrev", "--format=%H", tip, exclusion];
}

/// What:
///  Whether this byte ends one field of the walk's output.
/// Why:
///   The walk separates its records and paths by NUL,
///  and a path may hold any byte but that one,
///  so one pass over the bytes takes each field as Git printed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isFieldEnd = (byte) => byte === 0;
/// ```
fn is_field_end(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether one field of the walk's output is a change record.
/// Why:
///   A record names the two modes,
///  the two object names and the status before the path it belongs to,
///  and starts with `:`;
///  every other field in that position is the commit the records after it belong to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRawRecord = (field) => field[0] === ':';
/// ```
fn is_raw_record(field: &[u8]) -> bool {
    return field.starts_with(b":");
}

/// What:
///  The commits and paths of the one walk,
///  folded into the state of each candidate path.
///  `&mut BTreeMap<...>` is the state of the shared paths,
///  keyed by their bytes.
/// Why:
///   A change record is one field that starts with `:`
///  and the path after it;
///  every other non-empty field is the commit the records after it belong to.
///  Only the paths the caller asked about are looked up,
///  so the walk costs one question per change and one per candidate
///  rather than one query per candidate (`AGENTS.md` `RG2`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteHistory(bytes, stamps, captured, states) { let commit; let pathNext = false; for (const field of bytes.split(0)) { if (pathNext) { fold(states.get(field), commit, stamps, captured); pathNext = false; continue; } const plain = field.startsWith('\n') ? field.slice(1) : field; if (plain.length === 0) continue; if (plain[0] === ':') { pathNext = true; continue; } commit = plain.toString(); } }
/// ```
fn note_history(
    output: &[u8],
    stamps: &HashMap<&str, u64>,
    captured: u64,
    states: &mut BTreeMap<Vec<u8>, OrderSoFar>,
) -> Result<(), String> {
    let mut commit: Option<&str> = None;
    let mut path_next: bool = false;
    let mut rest: &[u8] = output;
    while !rest.is_empty() {
        let (field, after): (&[u8], &[u8]) = match rest.iter().position(is_field_end) {
            Some(at) => (&rest[..at], &rest[at + 1..]),
            None => (rest, &rest[rest.len()..]),
        };
        rest = after;
        if path_next {
            path_next = false;
            crate::collected_elements_looked_at::note_looked_at(1);
            if let Some(state) = states.get_mut(field) {
                let Some(known) = commit else {
                    state.unknown = true;
                    continue;
                };
                fold(state, known, stamps, captured);
            }
            continue;
        }
        // The line break Git prints before the first record of a commit belongs to that line break, not to the path.
        let plain: &[u8] = field.strip_prefix(b"\n").unwrap_or(field);
        if plain.is_empty() {
            continue;
        }
        if is_raw_record(plain) {
            path_next = true;
            continue;
        }
        commit = match std::str::from_utf8(plain) {
            Ok(text) => Some(text),
            Err(_) => return Err(String::from("git log named a commit with bytes that are not text")),
        };
    }
    return Ok(());
}

/// What:
///  The decision of every shared path,
///  from one walk over the history since the base.
/// Why:
///  The caller decides a whole conflict's paths at once,
///  and the history is the same for all of them,
///  so one walk answers them all (`AGENTS.md` `RG2`).
///  A capture with no stamp of its own is not decided by order at all,
///  and neither is a conflict with no path at all,
///  so both are answered without reading the history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const order = await captureOrderOfEach(site, captured, tip, shared);
/// ```
pub fn capture_order_of_each(
    site: &MergeSite<'_>,
    captured: &Captured,
    tip: &str,
    shared: &[SharedPath],
) -> Result<BTreeMap<Vec<u8>, CaptureOrderDecision>, TransactionStop> {
    let mut states: BTreeMap<Vec<u8>, OrderSoFar> = BTreeMap::new();
    for entry in shared {
        states.insert(entry.path.clone(), NOTHING_SEEN);
    }
    if captured.captured == 0 || states.is_empty() {
        return Ok(decisions(&states));
    }
    let stamps: HashMap<&str, u64> = stamps_by_commit(captured.landed.as_slice());
    let exclusion: String = format!("^{}", captured.base);
    let arguments: Vec<&str> = history_arguments(tip, exclusion.as_str());
    let request: GitRequest = GitRequest::new(site.cwd, arguments.as_slice());
    let listed = run_git_checked(site.context, &request)?;
    match note_history(listed.stdout.as_slice(), &stamps, captured.captured, &mut states) {
        Ok(()) => {}
        Err(reason) => return Err(failed(format!("reading the history of the paths the branch changed failed: {reason}"))),
    }
    return Ok(decisions(&states));
}

/// What:
///  The decision of each path the state was collected for.
/// Why:
///   The walk fills the states,
///  and the caller reads the same decisions whether the history was walked or not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decisions = (states) => new Map([...states].map(([path, state]) => [path, decide(state)]));
/// ```
fn decisions(states: &BTreeMap<Vec<u8>, OrderSoFar>) -> BTreeMap<Vec<u8>, CaptureOrderDecision> {
    let mut decided: BTreeMap<Vec<u8>, CaptureOrderDecision> = BTreeMap::new();
    for (path, state) in states {
        crate::collected_elements_looked_at::note_looked_at(1);
        decided.insert(path.clone(), decide(*state));
    }
    return decided;
}

/// The walk's controls weigh the pass they measure.
#[cfg(test)]
#[path = "service_replay_capture_order_tests.rs"]
mod tests;
