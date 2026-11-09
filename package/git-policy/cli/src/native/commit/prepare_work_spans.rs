//! What:
//!  The named steps of a preparation's own work,
//! each with how often it ran,
//! the whole of its time and its longest run,
//! kept until the commit's timing record takes them.
//! Why:
//!  A record that names the `prepareMs` phase leaves a slow preparation unexplained:
//! the phase holds the one reading of the real index,
//! the walk of those bytes for the seed decision,
//! two copies of it into private indexes,
//! the writes of corrected entries,
//! and every policy by name,
//! and on 2026-11-09 the records could not say which of them spent a median 332 ms
//! (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
//! "What this could not determine").
//! The owner's answer to an unexplained slow commit was to log more:
//! a commit that cannot be explained afterwards is a defect.
//! The figures are taken from the monotonic clock around the step,
//! never from a Git child alone,
//! so a step that starts Git is stated here as the work it does and in `steps` as the child it started.
//! The names are literals of this crate (a policy's registry name included),
//! never a path,
//! a message,
//! an option value or anything a person typed.
//! The list is bounded by its number of names (`MAX_SPAN_NAMES`) and says how many names it dropped,
//! so a record never grows with the work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const spans = new Map<string, { calls: number; totalMs: number; longestMs: number }>();
//! export const noteSpan = (name: string, tookMs: number) => { const one = spans.get(name) ?? { calls: 0, totalMs: 0, longestMs: 0 }; spans.set(name, { calls: one.calls + 1, totalMs: one.totalMs + tookMs, longestMs: Math.max(one.longestMs, tookMs) }); };
//! ```

/// Escaping of a step name inside a JSON string.
use crate::json_record::quote;
/// A lock around the shared list.
use std::sync::Mutex;
/// A span of time.
use std::time::Duration;

/// The most distinct step names one record lists.
///
/// A preparation notes the index's stat check,
/// the index read,
/// the seed walk of those bytes,
/// the index copies,
/// the entry writes,
/// and one name per policy the pass evaluates,
/// per lifecycle point.
/// The registry holds ten policies (`policy_registry.rs`),
/// one commit evaluates them at its pre-forward point and again at its post-commit gate,
/// so thirty names at most,
/// and the work above adds fewer than ten;
/// 64 is the next power of two above that,
/// and the record names how many names went unlisted,
/// so a longer run is counted,
/// not lost silently.
pub const MAX_SPAN_NAMES: usize = 64;

/// What:
///  The name of the step that answers whether the real index still records its worktree's stat data.
/// Why:
///   The answer is read from the whole index and a `stat` of every entry it names,
/// which no Git child covers,
/// so a slow preparation must be able to name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INDEX_STAT_CHECK = 'index stat check';
/// ```
pub const INDEX_STAT_CHECK: &str = "index stat check";

/// The one reading of an index's bytes that every private copy of it and the seed decision are made from.
/// What the seed walks those bytes for is stated under `INDEX_SEED`.
pub const INDEX_READ: &str = "index read";

/// The walk of an index's bytes that decides whether they may seed a private index.
pub const INDEX_SEED: &str = "index seed";

/// The answer of whether the captured index already holds the base's tree,
/// read from the index and the tree rather than asked of Git.
pub const INDEX_EQUALS_BASE: &str = "index equals base";

/// What:
///  The name of the step that prepares a pass's candidate set:
/// the private index its listings read,
/// the listings,
/// and the content and attributes behind each path.
/// Why:
///   The first content policy of a pass pays for the whole set,
/// so a record that names only the policies blames whichever policy asked first
/// for work every content policy shares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CANDIDATES_PREPARED = 'candidates prepared';
/// ```
pub const CANDIDATES_PREPARED: &str = "candidates prepared";

/// The copying of an index into a private one.
pub const INDEX_COPY: &str = "index copy";

/// The writing of one corrected file's bytes as a blob.
pub const POLICY_BLOB_WRITE: &str = "policy blob write";

/// The pointing of one private index entry at a corrected blob.
pub const POLICY_ENTRY_WRITE: &str = "policy entry write";

/// What:
///  The name one policy's evaluation is noted under,
/// with the lifecycle point it evaluated.
/// Why:
///   A policy's registry name is the word its findings and its configuration use,
/// so a record that names it can be read beside the person's configuration and the tool's events,
/// and the lifecycle point tells a preparation's own pass from the post-commit gate's,
/// which spends `gateMs` and not `prepareMs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const policySpanName = (registryName: string, trigger: string) => `policy ${registryName} at ${trigger}`;
/// ```
pub fn policy_span_name(registry_name: &str, trigger: crate::policy_trigger::Trigger) -> String {
    return format!("policy {registry_name} at {}", crate::policy_trigger::trigger_name(trigger));
}

/// What:
///  One named step of the preparation's work,
/// as the record states it.
/// Why:
///   The count and the longest run are what tell a tail from a cost:
/// one policy that ran once for eleven seconds and one that ran sixty times for twenty milliseconds are the same total.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrepareSpan = { name: string; calls: number; total: number; longest: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrepareSpan {
    /// The step's name.
    pub name: String,
    /// How often it ran under this preparation.
    pub calls: usize,
    /// The whole of its time,
    /// summed over its runs.
    pub total: Duration,
    /// Its longest run.
    pub longest: Duration,
}

/// The names kept and how many did not fit.
struct Ledger {
    /// The names kept,
    /// in the order each was first noted.
    spans: Vec<PrepareSpan>,
    /// How many names were noted after the list was full.
    dropped: usize,
}

/// The steps of this process since a record last took them.
static SPANS: Mutex<Ledger> = Mutex::new(Ledger { spans: Vec::new(), dropped: 0 });

/// What:
///  Keeps one step that has just ended,
/// adding it to the name's entry or starting that entry.
/// Why:
///   Every site that measures a step calls this once,
/// whatever the step's outcome,
/// and a policy runs again in every convergence round under its own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteSpan('index copy', 12);
/// ```
pub fn note_span(name: &str, took: Duration) {
    // A poisoned lock means another thread panicked while holding it; the list is still a list of spans.
    let mut ledger = SPANS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    for span in &mut ledger.spans {
        if span.name == name {
            span.calls += 1;
            span.total += took;
            span.longest = span.longest.max(took);
            return;
        }
    }
    if ledger.spans.len() >= MAX_SPAN_NAMES {
        ledger.dropped += 1;
        return;
    }
    ledger.spans.push(PrepareSpan {
        name: name.to_string(),
        calls: 1,
        total: took,
        longest: took,
    });
}

/// What:
///  Every named step kept,
/// in the order each name was first noted,
/// with the number of names dropped;
/// the list is emptied.
/// Why:
///   One record takes the steps of the preparation it describes;
/// the next command starts from nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { spans, dropped } = drainSpans();
/// ```
pub fn drain_spans() -> (Vec<PrepareSpan>, usize) {
    let mut ledger = SPANS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    return (std::mem::take(&mut ledger.spans), std::mem::take(&mut ledger.dropped));
}

/// What:
///  One span as a JSON object:
/// `{"name":"index copy","calls":2,"totalMs":42,"longestMs":30}`.
/// Why:
///   `spans_json` joins them into the record's list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ name, calls, totalMs, longestMs })
/// ```
fn span_json(span: &PrepareSpan) -> String {
    return format!(
        "{{\"name\":{},\"calls\":{},\"totalMs\":{},\"longestMs\":{}}}",
        quote(span.name.as_str()),
        span.calls,
        span.total.as_millis(),
        span.longest.as_millis()
    );
}

/// What:
///  The spans as a JSON array,
/// in the order given.
/// Why:
///   The client's timing record carries it under `prepareSpans`,
/// with the number of names dropped under `prepareSpansDropped`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(spans)
/// ```
pub fn spans_json(spans: &[PrepareSpan]) -> String {
    let members: Vec<String> = spans.iter().map(span_json).collect();
    return format!("[{}]", members.join(","));
}

/// The ledger is kept,
/// aggregated by name,
/// bounded and written as controls in the test module beside it.
#[cfg(test)]
#[path = "prepare_work_spans_tests.rs"]
mod tests;
