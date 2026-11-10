//! Greedy combination of literal-free rules into a few union DFAs.
//!
//! What:
//!  the seedless (literal-free) rules run against every line,
//!  so [`group_seedless`]
//! collapses them from N separate DFA passes into a few combined union DFAs.
//!  Why:
//!  one
//! O(1)-per-byte pass over a union covers many rules at once;
//!  growing a group rule by
//! rule lets the smart `alt` constructor share structure across members,
//!  so compatible
//! rules fold together while the per-group state cap keeps the explosive ones apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module group: see exported functions and types below.
//! ```

/// What:
///     Imports the node algebra combined into each group.
/// Why:
///      The code below uses `Node` directly;
///  importing from `crate/ast/node` keeps each call
///          site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Node } from "crate/ast/node";
/// ```
use crate::ast::node::Node;

/// What:
///     Imports the constructors that wrap a group for unanchored search.
/// Why:
///      The code below uses `alt`,
///  `concat` directly;
///  importing from `crate/ast/smart` keeps
///          each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { alt, concat } from "crate/ast/smart";
/// ```
use crate::ast::smart::{alt, concat};

/// What:
///     Imports the DFA builder and minimizer for each group.
/// Why:
///      The code below uses `build_dfa_within`,
///  `minimize` directly;
///  importing from
///          `crate/dfa` keeps each call site focused on the matcher logic instead of the full
///          Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { build_dfa_within, minimize } from "crate/dfa";
/// ```
use crate::dfa::{build_dfa_within, minimize};

/// What:
///     Imports the engine wrapper each group DFA becomes.
/// Why:
///      The code below uses `Engine`,
///  `EngineKind` directly;
///  importing from `crate/engine`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Engine, EngineKind } from "crate/engine";
/// ```
use crate::engine::{Engine, EngineKind};

/// Largest combined DFA a group of seedless rules may form.
///
/// What:
///  a state ceiling on a group's union DFA;
///  a trial union exceeding it starts a
/// new group instead.
///  Why:
///  combining literal-free rules into one pass is the
/// throughput lever,
///  but `{n,m}` overlap blowup must cap,
///  so explosive rules separate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GROUP_DFA_CAP: number = 6_000;
/// ```
const GROUP_DFA_CAP: usize = 6_000;

/// Wraps a node with the `Σ*` prefix for unanchored substring search.
///
/// What:
///  prefixes `Top` so a nullable residual at any boundary means a substring
/// matched.
///  Why:
///  every group DFA matches anywhere in the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function search_wrap(node: Node): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn search_wrap(node: Node) -> Node {
    return concat(vec![Node::Top, node])
}

/// The table engine of a built automaton after minimization,
/// with no seeds.
fn minimized_engine(dfa: crate::dfa::table::Dfa) -> Engine {
    return Engine::new(EngineKind::Table(minimize(&dfa)), Vec::new());
}

/// Builds a node into a seedless Table engine,
///  or `None` past the cap.
///
/// What:
///  determinizes the search-wrapped node under the cap and minimizes it.
///  Why:
///  a
/// node that exceeds the cap is rejected so the caller can keep it separate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function group_engine(node: Node): Engine | null {
///   // Rust body below is the implementation.
/// }
/// ```
fn group_engine(node: Node) -> Option<Engine> {
    return build_dfa_within(search_wrap(node), GROUP_DFA_CAP)
        .ok()
        .map(minimized_engine)
}

/// One candidate union group:
///  the merged node its members form,
///  plus the input indices of those members.
///
/// What:
///  keeps the merged union the cap measures and the members it merged.
///  Why:
///  a
/// group the cap rejects must hand each member back to its own engine,
///  so the members
/// are needed exactly when the union is unusable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Group = {
///   // fields documented in Rust above
/// };
/// ```
struct Group {
    /// What:
    ///     The alternation of the group's members.
    /// Why:
    ///      `union` stores the alternation of the group's members,
    ///  the node whose DFA the
    ///          state cap bounds,
    ///  so the grouping code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// union: Node;
    /// ```
    union: Node,
    /// What:
    ///     Indices in the input node list of the rules this group merged.
    /// Why:
    ///      `members` stores indices in the input node list of the rules this group merged,
    ///  so a
    ///          union the cap rejects can be replaced rule by rule instead of dropped,
    ///  so
    ///          the grouping code reads that precomputed state by name instead of recomputing
    ///          or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// members: number[];
    /// ```
    members: Vec<usize>,
}

/// Folds each rule into the first group whose grown union still fits the cap.
///
/// What:
///  tries appending the rule to each existing group union and keeps the first
/// union that builds under the cap;
///  otherwise the rule opens a new group.
///  Why:
///  growing
/// a group lets `alt` share structure across members,
///  so far more rules combine than a
/// pairwise test would admit,
///  and each group keeps its members so the cap can never lose
/// one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function grow_groups(nodes: Node[]): Group[] {
///   // Rust body below is the implementation.
/// }
/// ```
fn grow_groups(nodes: &[Node]) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    'rule: for (index, node) in nodes.iter().enumerate() {
        for group in &mut groups {
            let trial = alt(vec![group.union.clone(), node.clone()]);
            if build_dfa_within(search_wrap(trial.clone()), GROUP_DFA_CAP).is_ok() {
                group.union = trial;
                group.members.push(index);
                continue 'rule;
            }
        }
        groups.push(Group { union: node.clone(), members: vec![index] });
    }
    return groups
}

/// Combines literal-free rule nodes into as few union DFAs as the cap allows.
///
/// What:
///  greedily grows groups,
///  then builds one Table engine per group,
///  falling back to
/// each member's own engine for a union the cap rejects.
///  Why:
///  the
/// literal-free rules dominate per-line cost,
///  so fewer passes over them is the main
/// throughput lever,
///  and the cap is a throughput budget,
///  so exceeding it may cost an extra
/// pass but never the rule's coverage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function group_seedless(nodes: Node[], per_rule: Engine[]): Engine[] {
///   // Rust body below is the implementation.
/// }
/// ```
///
/// `per_rule` holds each node's own engine in the same order as `nodes`.
///
/// @example
/// ```ignore
/// let nodes = vec![parse("[0-9]{15,16}").unwrap()];
/// let per_rule = vec![build_engine(nodes[0].clone()).unwrap()];
/// let engines = group_seedless(&nodes, &[&per_rule[0]]);
/// assert_eq!(engines.len(), 1);
/// ```
pub(crate) fn group_seedless(nodes: &[Node], per_rule: &[&Engine]) -> Vec<Engine> {
    let mut engines: Vec<Engine> = Vec::new();
    for group in grow_groups(nodes) {
        match group_engine(group.union) {
            Some(engine) => engines.push(engine),
            None => {
                tracing::warn!(
                    cap = GROUP_DFA_CAP,
                    members = group.members.len(),
                    "seedless union exceeds the group state cap; keeping each member's own engine",
                );
                for index in group.members {
                    engines.push(per_rule[index].clone());
                }
            }
        }
    }
    return engines
}

/// What:
///     Unit tests for seedless-rule grouping,
///  in a sidecar (max-lines exempt).
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./tests";
/// ```
#[cfg(test)]
#[path = "group_tests.rs"]
mod tests;
