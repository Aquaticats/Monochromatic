//! What:
//!  A small exhaustive schedule explorer: breadth-first search over every state a model can reach,
//!  with a cap on the number of states and on the depth, and the shortest trace to the first violation.
//! Why:
//!  The commits-never-fail design (`doc/planning/cli-git-commits-never-fail.md`) is checked by models of its protocol.
//!  A model is a start state, a function listing every step enabled in a state (a crash is always among them),
//!  and two checks.
//!  Nothing is random and nothing is a clock, so a violation is a schedule that can be read and replayed.
//!  No model-checking crate is used: none is in the offline registry cache or in `Cargo.lock`, and the schedules
//!  here are a handful of steps over a few processes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function explore<S>(initials: S[], successors: (s: S) => [string, S][], invariant: (s: S) => string | undefined,
//!                     terminal: (s: S) => string | undefined, limits: Limits): Report;
//! ```

// What:     `use` pulls names from the standard library into scope. `HashMap` is a key-to-value table
//           (siblings: `BTreeMap`, which keeps keys sorted and needs `Ord` instead of `Hash`).
//           `VecDeque` is not needed because the search walks a growing `Vec` with an index.
// Why:      The visited table maps a state to its position in the search order, so a repeated state is found at once.
//
// In TS you'd write (pseudocode):
// ```ts
// // A Map<string, number> keyed by a serialization of the state.
// ```
use std::collections::HashMap;
// What:     `Hash` is the trait (an interface) that a type implements to be usable as a `HashMap` key.
// Why:      Every model state is a key of the visited table.
//
// In TS you'd write (pseudocode):
// ```ts
// // interface Hashable { hash(): number }
// ```
use std::hash::Hash;

/// What:
///  `struct Limits { ... }` is a record of the two caps on a search.
///  Both fields are `usize`, the unsigned integer wide enough to index memory (siblings: `u32`, `u64`, `i64`).
/// Why:
///  `usize` and not `u64` because both numbers are compared with `Vec` lengths and `usize` needs no cast.
///  A search that reaches a cap is reported as not exhausted, and the green tests refuse such a report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Limits = { maxStates: number; maxDepth: number };
/// ```
pub struct Limits {
    /// The most distinct states the search may hold before it stops.
    pub max_states: usize,
    /// The most steps from a start state the search may take.
    pub max_depth: usize,
}

/// What:
///  `struct Violation { ... }` is a failed check: the message the check gave and the labels of the steps from a
///  start state to the failing state.
///  `String` is an owned, growable UTF-8 text (sibling: `&str`, a borrowed view);
///  `Vec<String>` is an owned growable list of them (siblings: `&[String]`, `[String; N]`).
/// Why:
///  A report outlives the search that made it, so it owns its text (`String`, not `&str`);
///  the trace length is not known beforehand (`Vec`, not an array).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Violation = { message: string; trace: string[] };
/// ```
#[derive(Clone, Debug)]
pub struct Violation {
    /// What the failed check said.
    pub message: String,
    /// The labels of the steps taken, first to last.
    pub trace: Vec<String>,
}

impl Violation {
    /// What:
    ///  Whether every wanted label occurs in the trace, in the wanted order (other steps may lie between).
    ///  `&self` borrows the violation read-only; `&[&str]` borrows a list of borrowed texts.
    /// Why:
    ///  A red control must show that the checker found the review's schedule and not some other failure.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// containsInOrder(wanted: string[]): boolean;
    /// ```
    pub fn contains_in_order(&self, wanted: &[&str]) -> bool {
        // What:     `let mut next: usize = 0;` makes a counter that can change (`mut`), typed as `usize`.
        // Why:      It marks how many wanted labels the walk has matched so far.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let next = 0;
        // ```
        let mut next: usize = 0;
        // What:     `for label in &self.trace` loops over borrowed entries of the trace; `&` lends them read-only.
        // Why:      Visit the steps in order without taking the list away from the violation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const label of this.trace) {
        // ```
        for label in &self.trace {
            // What:     `wanted.get(next)` is `Option<&&str>`: `Some(entry)` when the index exists, `None` past the end.
            //           `Some(entry) = ...` inside `if let` matches the present case and names the value `entry`.
            //           `*entry` reads through the borrow.
            // Why:      Compare this label with the next wanted one, if any wanted label is left.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const entry = wanted[next]; if (entry !== undefined && label === entry) next += 1;
            // ```
            // (`if let ... && cond` combines the match and the test in one condition.)
            if let Some(entry) = wanted.get(next)
                && label.as_str() == *entry
            {
                next += 1;
            }
        }
        // What:     the tail expression `next == wanted.len()` has no `;`, so it is the returned `bool`.
        // Why:      All wanted labels were matched exactly when the counter reached the list length.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return next === wanted.length;
        // ```
        return next == wanted.len();
    }
}

/// What:
///  `struct Report { ... }` is the result of a search.
///  `Option<Violation>` is either `Some(violation)` or `None` (TS: `Violation | undefined`).
/// Why:
///  A green test needs `exhausted` (no cap was hit) and the counts as well as the absence of a violation;
///  `Option` and not an empty violation because absence is the normal answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Report = { states: number; deepest: number; exhausted: boolean; violation: Violation | undefined };
/// ```
pub struct Report {
    /// How many distinct states the search saw.
    pub states: usize,
    /// The longest shortest path from a start state to any state seen.
    pub deepest: usize,
    /// True when every reachable state was visited: no cap was hit and the search was not stopped by a violation.
    pub exhausted: bool,
    /// The first violation found, shortest trace first.
    pub violation: Option<Violation>,
}

/// One state of the search with how it was reached.
struct Node<S> {
    /// The model state.
    state: S,
    /// The index of the node it was reached from, `usize::MAX` for a start state.
    parent: usize,
    /// The label of the step that reached it, empty for a start state.
    label: String,
    /// The number of steps from a start state.
    depth: usize,
}

/// What:
///  Builds the labels from a start state to node `at`, first to last.
///  `<S>` is a type parameter (TS: generics); `&[Node<S>]` borrows the node list read-only.
/// Why:
///  A violation reports the schedule, so the search keeps parent links and walks them back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function traceTo<S>(nodes: Node<S>[], at: number): string[];
/// ```
fn trace_to<S>(nodes: &[Node<S>], at: usize) -> Vec<String> {
    // What:     `Vec::new()` makes an empty growable list; the annotation says its entries are `String`.
    // Why:      Collect labels from the failing state back to the start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const labels: string[] = [];
    // ```
    let mut labels: Vec<String> = Vec::new();
    let mut here: usize = at;
    // What:     `loop { ... }` repeats until a `break`; `nodes[here]` indexes the list (a panic if out of range).
    // Why:      Follow parent links until the start state, whose parent is `usize::MAX`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (true) { const node = nodes[here]; ... }
    // ```
    loop {
        let node: &Node<S> = &nodes[here];
        if node.parent == usize::MAX {
            break;
        }
        // What:     `.clone()` makes an independent copy of the label text, because the node keeps its own.
        // Why:      The report owns its labels.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // labels.push(node.label); // strings are immutable values in TS, so no copy is spelled out
        // ```
        labels.push(node.label.clone());
        here = node.parent;
    }
    // What:     `.reverse()` flips the list in place (it was collected last-step-first).
    // Why:      A trace reads first step to last.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // labels.reverse();
    // ```
    labels.reverse();
    // What:     the tail expression `labels` moves the list out to the caller.
    // Why:      Hand over the finished trace.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return labels;
    // ```
    return labels;
}

/// What:
///  Whether a step label names a step of the environment: a death, a stall or a start that nobody in the protocol
///  causes. Such a label begins with `outside:` or `environment:`.
///  `label: &str` borrows the text; `.starts_with("...")` asks whether it begins with the given text.
/// Why:
///  A state is stuck for liveness when only the environment can move it. If a wedged owner could always be "rescued"
///  by an outside death, no wedge would ever look stuck, which assumes away the problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isEnvironment = (label: string) => label.startsWith('outside:') || label.startsWith('environment:');
/// ```
fn is_environment(label: &str) -> bool {
    return label.starts_with("outside:") || label.starts_with("environment:");
}

/// What:
///  Explores every state reachable from `initials`, breadth first.
///  `successors` lists `(label, next state)` for every step enabled in a state;
///  `invariant` returns `Some(message)` for a state that must never occur;
///  `terminal` returns `Some(message)` for a quiescent state that is not an acceptable end: no step is enabled except
///  steps of the environment (see `is_environment`), which are still taken.
///  `fn(&S) -> ...` is a function pointer (TS: a plain function value).
/// Why:
///  One generic type parameter is enough for every model, and function pointers keep each model a few plain
///  functions with its rule revision stored in the state.
///  Breadth first, so the trace of a violation is a shortest schedule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function explore<S>(initials: S[], successors: (s: S) => [string, S][], ...): Report;
/// ```
pub fn explore<S: Clone + Eq + Hash>(
    initials: Vec<S>,
    successors: fn(&S) -> Vec<(String, S)>,
    invariant: fn(&S) -> Option<String>,
    terminal: fn(&S) -> Option<String>,
    limits: Limits,
) -> Report {
    let mut nodes: Vec<Node<S>> = Vec::new();
    // What:     `HashMap<S, usize>` maps a state to its index in `nodes`.
    // Why:      Skip a state that was already queued.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const seen = new Map<S, number>();
    // ```
    let mut seen: HashMap<S, usize> = HashMap::new();
    let mut exhausted: bool = true;
    let mut deepest: usize = 0;
    for initial in initials {
        // What:     `seen.contains_key(&initial)` asks whether the key exists, borrowing it (`&`) without moving it.
        // Why:      Two equal start states are one state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (seen.has(initial)) continue;
        // ```
        if seen.contains_key(&initial) {
            continue;
        }
        seen.insert(initial.clone(), nodes.len());
        nodes.push(Node { state: initial, parent: usize::MAX, label: String::new(), depth: 0 });
    }
    let mut next: usize = 0;
    while next < nodes.len() {
        let at: usize = next;
        next += 1;
        // What:     `.clone()` copies the state so the node list can be extended while this state is read.
        // Why:      Rust forbids holding a borrow of `nodes` across `nodes.push`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const state = nodes[at].state;
        // ```
        let state: S = nodes[at].state.clone();
        let depth: usize = nodes[at].depth;
        if depth > deepest {
            deepest = depth;
        }
        // What:     `if let Some(message) = invariant(&state)` runs the check and enters the block when it said `Some`.
        // Why:      The first failing state in breadth-first order has the shortest trace.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const message = invariant(state); if (message !== undefined) { return {...}; }
        // ```
        if let Some(message) = invariant(&state) {
            let trace: Vec<String> = trace_to(&nodes, at);
            return Report {
                states: nodes.len(),
                deepest,
                exhausted: false,
                violation: Some(Violation { message, trace }),
            };
        }
        let steps: Vec<(String, S)> = successors(&state);
        // What:     `steps.iter().any(|step| ...)` asks whether some step satisfies the test; `|step| expr` is a
        //           closure (TS: `(step) => expr`); `step.0` is the first field of the pair, the label; `!` is `not`.
        // Why:      A state where only the environment can move is quiescent: the protocol itself cannot progress.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const acting = steps.some(([label]) => !isEnvironment(label));
        // ```
        let acting: bool = steps.iter().any(|step| { return !is_environment(step.0.as_str()); });
        if !acting && let Some(message) = terminal(&state) {
            let trace: Vec<String> = trace_to(&nodes, at);
            return Report {
                states: nodes.len(),
                deepest,
                exhausted: false,
                violation: Some(Violation { message, trace }),
            };
        }
        if steps.is_empty() {
            continue;
        }
        if depth >= limits.max_depth {
            // The state has steps the search will not take.
            exhausted = false;
            continue;
        }
        for (label, after) in steps {
            if seen.contains_key(&after) {
                continue;
            }
            if nodes.len() >= limits.max_states {
                // A new state does not fit under the cap.
                exhausted = false;
                continue;
            }
            seen.insert(after.clone(), nodes.len());
            nodes.push(Node { state: after, parent: at, label, depth: depth + 1 });
        }
    }
    // What:     `Report { ... }` as the tail expression builds and returns the result; `violation: None` is the
    //           absent case of `Option`.
    // Why:      No check failed; `exhausted` says whether that covers every reachable state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { states: nodes.length, deepest, exhausted, violation: undefined };
    // ```
    return Report { states: nodes.len(), deepest, exhausted, violation: None };
}

/// What:
///  Replays one named schedule from `start`: for each wanted label, takes the enabled step with that label.
///  Panics (stops the test with a message) when a wanted label is not enabled in the state reached.
///  `&[&str]` borrows a list of borrowed texts.
/// Why:
///  The review's counterexamples are literal schedules; a test can state one step by step and check the state
///  it reaches, besides what the exhaustive search finds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replay<S>(start: S, successors: (s: S) => [string, S][], labels: string[]): S;
/// ```
pub fn replay<S: Clone>(start: S, successors: fn(&S) -> Vec<(String, S)>, labels: &[&str]) -> S {
    let mut state: S = start;
    for wanted in labels {
        let mut taken: Option<S> = None;
        for (label, after) in successors(&state) {
            if label.as_str() == *wanted {
                taken = Some(after);
                break;
            }
        }
        // What:     `match taken { Some(after) => ..., None => panic!(...) }` splits the two cases of an `Option`.
        // Why:      A schedule that names a step that is not enabled is a broken test, so say which step.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (taken === undefined) throw new Error(`step not enabled: ${wanted}`);
        // state = taken;
        // ```
        match taken {
            Some(after) => state = after,
            None => panic!("the step is not enabled in the state reached: {}", wanted),
        }
    }
    return state;
}

/// What:
///  Appends one enabled step, `(label, after)`, to the list a model is building.
///  `found: &mut Vec<...>` is a mutable borrow: the caller lends the list for this call and may see it changed
///  (TS passes arrays by reference, so a TS function can already `found.push(...)` on its argument).
///  `label: &str` borrows the text; `.to_string()` inside makes the owned copy the list keeps.
/// Why:
///  Every model writes dozens of steps; one helper keeps each to a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pushStep<S>(found: [string, S][], label: string, after: S): void { found.push([label, after]); }
/// ```
pub fn push_step<S>(found: &mut Vec<(String, S)>, label: &str, after: S) {
    found.push((label.to_string(), after));
}
