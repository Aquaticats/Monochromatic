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

/// What:
///  Explores like `explore`, and also checks a liveness bound: a state for which `waiting` is true may last at most `bound`
///  of the protocol's own steps. Steps of the environment (`outside:` and `environment:` labels) do not count and do not end a
///  wait; a waiting state that has no step of the protocol's own to take is a wait with nothing that can end it.
///  `fn(&S) -> bool` is a plain function value; the search state is a pair of the model state and how many own steps the
///  current wait has taken, so two visits with different wait lengths are different states.
/// Why:
///  The owner's rule "nothing should ever wait forever or indefinitely" needs a checkable form: for every wait the protocol
///  states, the bound in steps. A model gives `waiting` and the bound; a wait that outlives it is a violation with its trace.
///  This does not assume the environment helps (a hook returning, a foreign lock freeing): only the protocol's own steps count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exploreBounded<S>(initials: S[], successors: (s: S) => [string, S][], invariant: (s: S) => string | undefined,
///                            waiting: (s: S) => boolean, bound: number, limits: Limits): Report;
/// ```
pub fn explore_bounded<S: Clone + Eq + Hash>(
    initials: Vec<S>,
    successors: fn(&S) -> Vec<(String, S)>,
    invariant: fn(&S) -> Option<String>,
    waiting: fn(&S) -> bool,
    bound: usize,
    limits: Limits,
) -> Report {
    return explore_bounded_by(initials, successors, invariant, waiting, |label| { return !is_environment(label); }, bound, limits);
}

/// What:
///  `explore_bounded` with the steps that count toward the bound chosen by the model: `counts(label)` says whether a step is one of
///  the waiting process's own. A wait with no step of the protocol's own at all (`!is_environment`) is still reported.
///  `fn(&str) -> bool` is a plain function value, so a closure that captures nothing can be passed.
/// Why:
///  The bound of a wait is the waiting process's own clock: another lane's retries are not steps of this wait, but the waiting
///  process's own step (the bound reached, then the action) must always be available.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exploreBoundedBy<S>(..., counts: (label: string) => boolean, ...): Report;
/// ```
pub fn explore_bounded_by<S: Clone + Eq + Hash>(
    initials: Vec<S>,
    successors: fn(&S) -> Vec<(String, S)>,
    invariant: fn(&S) -> Option<String>,
    waiting: fn(&S) -> bool,
    counts: fn(&str) -> bool,
    bound: usize,
    limits: Limits,
) -> Report {
    let mut nodes: Vec<Node<(S, usize)>> = Vec::new();
    let mut seen: HashMap<(S, usize), usize> = HashMap::new();
    let mut exhausted: bool = true;
    let mut deepest: usize = 0;
    for initial in initials {
        let start: (S, usize) = (initial, 0);
        if !seen.contains_key(&start) {
            seen.insert(start.clone(), nodes.len());
            nodes.push(Node { state: start, parent: usize::MAX, label: String::new(), depth: 0 });
        }
    }
    let mut next: usize = 0;
    while next < nodes.len() {
        let at: usize = next;
        next += 1;
        let (state, count): (S, usize) = nodes[at].state.clone();
        let depth: usize = nodes[at].depth;
        if depth > deepest {
            deepest = depth;
        }
        let steps: Vec<(String, S)> = successors(&state);
        // The waiting process must itself have a step that counts (not merely some other process's step): the same definition of its own
        // progress that the counter uses.
        let own_step: bool = steps.iter().any(|step| { return !is_environment(step.0.as_str()) && counts(step.0.as_str()); });
        let mut message: Option<String> = invariant(&state);
        if message.is_none() && waiting(&state) && !own_step {
            message = Some("a wait with nothing of the protocol's own left that can end it".to_string());
        }
        if let Some(text) = message {
            let trace: Vec<String> = trace_to(&nodes, at);
            return Report { states: nodes.len(), deepest, exhausted: false, violation: Some(Violation { message: text, trace }) };
        }
        if depth >= limits.max_depth {
            exhausted = false;
            continue;
        }
        for (label, after) in steps {
            let new_count: usize = if !waiting(&after) {
                0
            } else if !counts(label.as_str()) {
                count
            } else {
                count + 1
            };
            if new_count > bound {
                let mut trace: Vec<String> = trace_to(&nodes, at);
                trace.push(label);
                let text: String = format!("a wait lasted more than {} steps of the protocol's own", bound);
                return Report { states: nodes.len() + 1, deepest, exhausted: false, violation: Some(Violation { message: text, trace }) };
            }
            let key: (S, usize) = (after, new_count);
            if seen.contains_key(&key) {
                continue;
            }
            if nodes.len() >= limits.max_states {
                exhausted = false;
                continue;
            }
            seen.insert(key.clone(), nodes.len());
            nodes.push(Node { state: key, parent: at, label, depth: depth + 1 });
        }
    }
    return Report { states: nodes.len(), deepest, exhausted, violation: None };
}

/// What:
///  Checks AG EF goal: from every reachable state that is not a goal, a path of the protocol's own steps
///  (steps that are not `outside:` or `environment:`) leads to a goal state. A reachable cycle of non-goal states with no such
///  exit, and a non-goal state with no own step, are both reported, with the trace to the first such state.
///  This is reachability of the goal from every reachable state, not weak-fair eventual completion: a model whose own steps can
///  loop for ever beside an available exit is green here (the exit is assumed to be taken eventually). It does not assume the
///  environment ever helps. When the state or depth cap omitted edges, a state with an omitted edge is treated as able to finish
///  (unknown is not a counterexample) and the report says `exhausted: false`; a violation is reported only where the coverage is complete.
/// Why:
///  `explore` calls its `terminal` check only at states with no own step, so a cycle of own steps that never completes (a restarter
///  that starts again and again against an owner it can never restart) is never examined. This is the general liveness check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exploreEventually<S>(initials: S[], successors: (s: S) => [string, S][], goal: (s: S) => boolean, limits: Limits): Report;
/// ```
pub fn explore_eventually<S: Clone + Eq + Hash>(
    initials: Vec<S>,
    successors: fn(&S) -> Vec<(String, S)>,
    goal: fn(&S) -> bool,
    limits: Limits,
) -> Report {
    let mut nodes: Vec<Node<S>> = Vec::new();
    let mut seen: HashMap<S, usize> = HashMap::new();
    let mut edges: Vec<Vec<usize>> = Vec::new();
    let mut omitted: Vec<bool> = Vec::new();
    let mut exhausted: bool = true;
    for initial in initials {
        if !seen.contains_key(&initial) {
            seen.insert(initial.clone(), nodes.len());
            nodes.push(Node { state: initial, parent: usize::MAX, label: String::new(), depth: 0 });
        }
    }
    let mut next: usize = 0;
    while next < nodes.len() {
        let at: usize = next;
        next += 1;
        let state: S = nodes[at].state.clone();
        let depth: usize = nodes[at].depth;
        let mut own: Vec<usize> = Vec::new();
        let mut cut: bool = false;
        for (label, after) in successors(&state) {
            let index: usize = match seen.get(&after) {
                Some(found) => *found,
                None => {
                    if nodes.len() >= limits.max_states || depth >= limits.max_depth {
                        exhausted = false;
                        cut = true;
                        continue;
                    }
                    seen.insert(after.clone(), nodes.len());
                    nodes.push(Node { state: after, parent: at, label: label.clone(), depth: depth + 1 });
                    nodes.len() - 1
                }
            };
            if !is_environment(label.as_str()) {
                own.push(index);
            }
        }
        edges.push(own);
        omitted.push(cut);
    }
    // Backward fixpoint: a state can finish if it is a goal or some own step leads to a state that can.
    let mut can_finish: Vec<bool> = nodes.iter().enumerate().map(|(index, node)| { return goal(&node.state) || omitted[index]; }).collect();
    let mut changed: bool = true;
    while changed {
        changed = false;
        for index in 0..edges.len() {
            if !can_finish[index] && edges[index].iter().any(|target| { return can_finish[*target]; }) {
                can_finish[index] = true;
                changed = true;
            }
        }
    }
    for index in 0..edges.len() {
        if !can_finish[index] {
            let trace: Vec<String> = trace_to(&nodes, index);
            let message: String = "a state from which the protocol's own steps can never complete".to_string();
            return Report { states: nodes.len(), deepest: nodes[index].depth, exhausted: false, violation: Some(Violation { message, trace }) };
        }
    }
    let deepest: usize = nodes.iter().map(|node| { return node.depth; }).max().unwrap_or(0);
    return Report { states: nodes.len(), deepest, exhausted, violation: None };
}
