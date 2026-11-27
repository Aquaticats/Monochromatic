//! What:
//!     Building a [`RegexSet`] from pattern text:
//!  the accumulator that records each built rule,
//!  the threaded strict and lenient compiles,
//!  and the split-on-delimiter convenience.
//! Why:
//!      This file is the Rust module that groups the ruleset construction,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module ruleset_construction: see the impl blocks below.
//! ```

/// What:
///     Imports the ruleset type these functions build and the accumulator they fill.
/// Why:
///      The code below uses `RegexSet`,
///  `RuleSink` directly;
///  importing from `./super` keeps each
///          call site focused on the building logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegexSet, RuleSink } from "./super";
/// ```
use super::{
    RegexSet,
    RuleSink,
};

/// What:
///     Imports the empty-text filter the delimiter split drops with.
/// Why:
///      The code below uses `is_nonempty_text` directly;
///  importing from `./line_checks` keeps
///          each call site focused on the building logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { is_nonempty_text } from "./line_checks";
/// ```
use super::line_checks::is_nonempty_text;

/// What:
///     Imports one built rule,
///  and the fold that turns the literal-free nodes into the oracle
///          counting union.
/// Why:
///      The code below uses `BuiltRule`,
///  `build_seedless_union` directly;
///  importing from `crate/build`
///          keeps each call site focused on the building logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BuiltRule, build_seedless_union } from "crate/build";
/// ```
use crate::build::{
    BuiltRule,
    build_seedless_union,
};

/// What:
///     Imports the byte set the line-start fast reject starts life empty in.
/// Why:
///      The code below uses `ByteSet` directly;
///  importing from `crate/charset` keeps each
///          call site focused on the building logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ByteSet } from "crate/charset";
/// ```
use crate::charset::ByteSet;

/// What:
///     Imports the per-pattern back-end the over-cap fallback borrows.
/// Why:
///      The code below names `Engine` in the fallback vector's type;
///  importing from
///          `crate/engine` keeps each call site focused on the building logic instead of the
///          full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Engine } from "crate/engine";
/// ```
use crate::engine::Engine;

/// What:
///     Imports the error type a strict compile returns.
/// Why:
///      The code below uses `CompileError` directly;
///  importing from `crate/error` keeps each
///          call site focused on the building logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CompileError } from "crate/error";
/// ```
use crate::error::CompileError;

/// What:
///     Imports the set-level combined literal prefilter an assembled set starts with.
/// Why:
///      The code below uses `SetGate` directly;
///  importing from `crate/gate` keeps each call
///          site focused on the building logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SetGate } from "crate/gate";
/// ```
use crate::gate::SetGate;

/// What:
///     Imports the seedless-rule grouping into union DFAs.
/// Why:
///      The code below uses `group_seedless` directly;
///  importing from `crate/group` keeps
///          each call site focused on the building logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { group_seedless } from "crate/group";
/// ```
use crate::group::group_seedless;

/// What:
///     Imports the threaded per-rule builder.
/// Why:
///      The code below uses `build_rules` directly;
///  importing from `crate/parallel` keeps
///          each call site focused on the building logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { build_rules } from "crate/parallel";
/// ```
use crate::parallel::build_rules;

/// What:
///     Recording one built rule and finishing into a `RegexSet`.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl RuleSink {
    /// Records a built rule,
    ///  tracking its routing and oracle node by rule id.
    ///
    /// What:
    ///  appends the engine and,
    ///  by route,
    ///  its anchored DFA,
    ///  line-start DFA,
    ///  or
    /// seedless id and node,
    ///  plus the original-seedless node for the oracle.
    ///  Why:
    ///  one
    /// place keeps every parallel vector aligned with the rule id.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function push(built: BuiltRule): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn push(
        &mut self,
        built: BuiltRule,
    ) {
        let id = self.rules.len();
        if built.seedless {
            self.seedless_ids.push(id);
            if let Some(node) = built.node {
                self.seedless_nodes.push(node);
            }
        }
        if let Some(engine) = built.routing.line_start {
            self.line_start.push(engine);
            self.line_start_ids.push(id);
        }
        if let Some(node) = built.reference_node {
            self.reference_nodes.push(node);
        }
        self.rules.push(built.engine);
        self.anchored.push(built.routing.anchored);
    }

    /// Assembles the accumulated rules into a prepared `RegexSet`.
    ///
    /// What:
    ///  groups the truly-seedless nodes into union DFAs,
    ///  hands each union the state
    /// cap rejects back to its rules' own engines,
    ///  builds the oracle
    /// counting union over the original-seedless nodes,
    ///  stores everything,
    ///  then builds
    /// the gate.
    ///  Why:
    ///  the one place that wires the matching structures together,
    ///  and the
    /// place holding both the seedless nodes and the per-rule engines that cover them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function assemble(): RegexSet {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn assemble(self) -> RegexSet {
        let seedless_union = build_seedless_union(&self.reference_nodes);
        // What:    The per-rule engine of every seedless rule, in the order its node was recorded.
        // Why:     A union group the state cap rejects falls back to these engines, so the
        //          boolean fast path keeps covering exactly the rules `matches` reports instead
        //          of dropping the group.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const seedlessEngines = [];
        // for (const id of seedlessIds) { seedlessEngines.push(rules[id]); }
        // ```
        let mut seedless_engines: Vec<&Engine> = Vec::with_capacity(self.seedless_ids.len());
        for &id in &self.seedless_ids {
            seedless_engines.push(&self.rules[id]);
        }
        let seedless_groups = group_seedless(&self.seedless_nodes, &seedless_engines);
        let mut set = RegexSet {
            rules: self.rules,
            anchored: self.anchored,
            line_start: self.line_start,
            line_start_ids: self.line_start_ids,
            seedless_ids: self.seedless_ids,
            seedless_groups,
            seedless_union,
            gate: SetGate::default(),
            line_start_first: ByteSet::empty(),
        };
        set.prepare();
        return set;
    }
}

/// What:
///     Compiling patterns into a ruleset:
///  the strict build,
///  the lenient build,
///  and the split-on-delimiter convenience.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl RegexSet {
    /// Compiles a slice of patterns into a `RegexSet`.
    ///
    /// What:
    ///  builds a rule per pattern,
    ///  spread across the available cores,
    ///  and one union
    /// automaton over the literal-free ones;
    ///  on failure,
    ///  returns the lowest-index rule's
    /// error.
    ///  Why:
    ///  seeded rules are gated and literal-free rules share a single pass,
    ///  and
    /// per-rule construction is independent,
    ///  so threads shorten large ruleset compiles.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function new(patterns: S[]): RegexSet {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(patterns: &[S]) -> Result<RegexSet, CompileError> {
        // What:    `patterns.iter().map(|pattern| return pattern.as_ref())` borrows each input
        //          as a `&str` (a read-only string view); `.collect()` gathers the views into a
        //          `Vec<&str>` (a growable array; siblings `&[&str]` and `[&str; N]` are a
        //          borrowed or fixed-size array).
        // Why:     Worker threads share plain `&str` views safely, whatever caller type `S`
        //          is, so `S` needs no thread-safety bound. A `Vec` because the pattern count
        //          is only known at run time.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const texts: string[] = patterns.map((pattern) => String(pattern));
        // ```
        let mut texts: Vec<&str> = Vec::with_capacity(patterns.len());
        for pattern in patterns {
            texts.push(pattern.as_ref());
        }
        let mut sink = RuleSink::default();
        // What:    `build_rules(&texts, true)` builds every rule and returns `(index, result)`
        //          pairs sorted by index; `true` lets workers skip rules past the first failure.
        //          `outcome?` unwraps a built rule or returns its error from `new`.
        // Why:     Index order keeps rule ids dense, and the first `Err` met in that order is
        //          the lowest-index failure, the error the one-at-a-time loop returned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const [, outcome] of build_rules(texts, true)) {
        //   if (outcome instanceof CompileError) throw outcome;
        //   sink.push(outcome);
        // }
        // ```
        for (_index, outcome) in build_rules(&texts, true) {
            sink.push(outcome?);
        }
        return Ok(sink.assemble());
    }

    /// Rebuilds the combined gate and the line-start first-byte set.
    ///
    /// What:
    ///  builds the set-level literal matcher from each rule's seeds,
    ///  then unions
    /// every line-start rule's possible first bytes.
    ///  Why:
    ///  neither is serialized,
    ///  so both
    /// are rebuilt after compilation and after decode.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function prepare(): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(super) fn prepare(&mut self) {
        let per_rule: Vec<Option<Vec<Vec<u8>>>> = self.rules.iter().map(Engine::seeds).collect();
        self.gate = SetGate::build(&per_rule);
        let mut first = ByteSet::empty();
        for engine in &self.line_start {
            engine.mark_first_bytes(&mut first);
        }
        self.line_start_first = first;
    }

    /// Compiles patterns,
    ///  skipping any that fail,
    ///  with the kept input indices.
    ///
    /// What:
    ///  builds a rule per pattern across the available cores,
    ///  dropping ones that do
    /// not compile,
    ///  plus the union automaton;
    ///  returns the set and the kept original
    /// indices.
    ///  Why:
    ///  a real ruleset has rules this dialect cannot express,
    ///  so the rest are
    /// kept in one pass.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function compile_lenient(patterns: S[]): (RegexSet, Vec<usize>) {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn compile_lenient<S: AsRef<str>>(patterns: &[S]) -> (RegexSet, Vec<usize>) {
        // What: borrow each input as a `&str` view. Why: the same thread-shareable form `new` uses.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const texts: string[] = patterns.map((pattern) => String(pattern));
        // ```
        let mut texts: Vec<&str> = Vec::with_capacity(patterns.len());
        for pattern in patterns {
            texts.push(pattern.as_ref());
        }
        let mut sink = RuleSink::default();
        let mut kept: Vec<usize> = Vec::new();
        // What: `false` builds every rule, no early stop. Why: a lenient compile keeps each
        // rule that builds, in input order.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const [index, outcome] of build_rules(texts, false)) { ... }
        // ```
        for (index, outcome) in build_rules(&texts, false) {
            if let Ok(built) = outcome {
                sink.push(built);
                kept.push(index);
            }
        }
        return (sink.assemble(), kept);
    }

    /// Compiles a ruleset from one text,
    ///  split on a delimiter.
    ///
    /// What:
    ///  splits `text` on `delimiter`,
    ///  trims each rule,
    ///  drops empties,
    ///  and
    /// delegates to `new`.
    ///  Why:
    ///  a convenience for a file format whose rule boundary
    /// is a non-whitespace marker the caller chooses.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function from_ruleset(text: string, delimiter: string): RegexSet {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn from_ruleset(
        text: &str,
        delimiter: &str,
    ) -> Result<RegexSet, CompileError> {
        let parts: Vec<&str> = text
            .split(delimiter)
            .map(str::trim)
            .filter(is_nonempty_text)
            .collect();
        return RegexSet::new(&parts);
    }
}
