//! What:
//!     Public API:
//!  a single-pattern `Regex` and a combined `RegexSet`.
//! Why:
//!      This file is the Rust module that groups the regex implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module regex: see exported functions and types below.
//! ```

/// What:
///     Imports the serde derives for persisting compiled matchers.
/// Why:
///      The code below uses `Deserialize`,
///  `Serialize` directly;
///  importing from `serde`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Deserialize, Serialize } from "serde";
/// ```
use serde::{Deserialize, Serialize};

/// What:
///     Imports the node algebra used by the rule sink's node lists.
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
///     Imports the byte set used for the line-start first-byte fast reject.
/// Why:
///      The code below uses `ByteSet` directly;
///  importing from `crate/charset` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ByteSet } from "crate/charset";
/// ```
use crate::charset::ByteSet;

/// What:
///     Imports the rule construction,
///  the seedless fold,
///  and the line-start matcher.
/// Why:
///      The code below uses `BuiltRule`,
///  `build_engine`,
///  `build_seedless_union`,
///          `line_start_match` directly;
///  importing from `crate/build` keeps each call site
///          focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   BuiltRule,
///   build_engine,
///   build_seedless_union,
///   line_start_match,
/// } from "crate/build";
/// ```
use crate::build::{BuiltRule, build_engine, build_seedless_union, line_start_match};

/// What:
///     Imports the threaded per-rule builder.
/// Why:
///      The code below uses `build_rules` directly;
///  importing from `crate/parallel` keeps
///          each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { build_rules } from "crate/parallel";
/// ```
use crate::parallel::build_rules;

/// What:
///     Imports the seedless-rule grouping into union DFAs.
/// Why:
///      The code below uses `group_seedless` directly;
///  importing from `crate/group` keeps
///          each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { group_seedless } from "crate/group";
/// ```
use crate::group::group_seedless;

/// What:
///     Imports the counting NFA type for the seedless-union oracle.
/// Why:
///      The code below uses `CountingNfa` directly;
///  importing from `crate/counting` keeps
///          each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CountingNfa } from "crate/counting";
/// ```
use crate::counting::CountingNfa;

/// What:
///     Imports the per-pattern back-end.
/// Why:
///      The code below uses `Engine` directly;
///  importing from `crate/engine` keeps each call
///          site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Engine } from "crate/engine";
/// ```
use crate::engine::Engine;

/// What:
///     Imports the error type.
/// Why:
///      The code below uses `CompileError` directly;
///  importing from `crate/error` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CompileError } from "crate/error";
/// ```
use crate::error::CompileError;

/// What:
///     Imports the RegexSet-level combined literal prefilter.
/// Why:
///      The code below uses `SetGate` directly;
///  importing from `crate/gate` keeps each call
///          site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SetGate } from "crate/gate";
/// ```
use crate::gate::SetGate;

/// What:
///     Imports the parser entry point.
/// Why:
///      The code below uses `parse` directly;
///  importing from `crate/parse` keeps each call
///          site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parse } from "crate/parse";
/// ```
use crate::parse::parse;

/// What:
///     Batched,
///  many-lines-at-once matching for `Regex` and `RegexSet`.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./batch";
/// ```
mod batch;

/// A compiled single pattern.
///
/// What:
///  wraps one back-end engine.
///  Why:
///  the reusable single-pattern face of the
/// engine,
///  used directly and as each member of a `RegexSet`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Regex = {
///   // fields documented in Rust above
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regex {
    /// What:
    ///     The compiled back-end for this pattern.
    /// Why:
    ///      `engine` stores the compiled back-end for this pattern,
    ///  so matcher code reads
    ///          that precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// engine: Engine;
    /// ```
    engine: Engine,
}

/// What:
///     Matching and (de)serialization for a single pattern.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl Regex {
    /// Reports whether the pattern matches some substring of `line`.
    ///
    /// What:
    ///  delegates to the back-end.
    ///  Why:
    ///  the public,
    ///  boolean match operation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn is_match(&self, line: &[u8]) -> bool {
        return self.engine.is_match(line)
    }

    /// Serializes the compiled pattern to bytes.
    ///
    /// What:
    ///  bincode-encodes the engine.
    ///  Why:
    ///  lets a caller persist a built matcher
    /// and reload it without recompiling.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function to_bytes(): number[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn to_bytes(&self) -> Result<Vec<u8>, CompileError> {
        return bincode::serialize(self).map_err(|e| return CompileError::Serialize {
            message: e.to_string(),
        })
    }

    /// Loads a compiled pattern from bytes,
    ///  validating it first.
    ///
    /// What:
    ///  decodes then runs structural validation.
    ///  Why:
    ///  a decoded engine is
    /// executed against untrusted input,
    ///  so it must be proven in-bounds first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function from_bytes(bytes: Uint8Array): Regex {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Regex, CompileError> {
        let mut regex: Regex = bincode::deserialize(bytes).map_err(|e| return CompileError::Invalid {
            message: e.to_string(),
        })?;
        regex.engine.validate()?;
        // What: rebuild the prefilter from the validated graph. Why: it is not
        // serialized, and matching depends on it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        regex.engine.prepare();
        return Ok(regex)
    }
}

/// Compiles one pattern into a `Regex`.
///
/// What:
///  parses then selects a back-end.
///  Why:
///  the primary entry for a single rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compile(pattern: string): Regex {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn compile(pattern: &str) -> Result<Regex, CompileError> {
    // Log the compile at debug; the pattern is small and this is the public boundary.
    tracing::debug!(pattern, "compile regex");
    // Parse, then build; `inspect_err` records the typed CompileError cause at each stage
    // before `?` propagates it (a rejected pattern is worth a warning).
    let node = parse(pattern)
        .inspect_err(|error| tracing::warn!(pattern, cause = %error, "regex parse failed"))?;
    let engine =
        build_engine(node).inspect_err(|error| tracing::warn!(pattern, cause = %error, "regex build failed"))?;
    return Ok(Regex { engine })
}

/// What:
///      `pub struct RegexSet { ... }` declares an exported Rust record type.
///           `pub` makes the type name public while its fields stay private.
///  The record
///           owns `Vec<Engine>` values (growable owned arrays,
///  not borrowed `&[Engine]`
///           slices or fixed `[Engine; N]` arrays),
///  optional anchored engines,
///  line-start
///           engines,
///  seedless groups,
///  and the rebuilt set-level gate.
/// Why:
///       The shared gate finds only rules whose required literal occurs,
///  while each
///           small owned engine keeps exact rule semantics without reintroducing
///           all-rules blowup.
///  Owned `Vec` storage is used instead of borrowed slices
///           because a compiled ruleset must outlive the input pattern list,
///  and fixed
///           arrays cannot represent a caller-chosen rule count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type RegexSet = {
///   // Private matcher state: exact engines, gates, line-start checks, and groups.
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexSet {
    /// What:
    ///     Per-rule substring engines,
    ///  indexed by rule id.
    /// Why:
    ///      `rules` stores per-rule substring engines,
    ///  indexed by rule id,
    ///  so matcher code
    ///          reads that precomputed state by name instead of recomputing or passing it
    ///          separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rules: Engine[];
    /// ```
    rules: Vec<Engine>,
    /// What:
    ///     Per-rule anchored DFA,
    ///  when the seed is the rule's leading literal,
    ///  else
    ///          `None`.
    /// Why:
    ///      `anchored` stores per-rule anchored DFA,
    ///  when the seed is the rule's leading
    ///          literal,
    ///  else `None`,
    ///  so matcher code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// anchored: Engine | null[];
    /// ```
    anchored: Vec<Option<Engine>>,
    /// What:
    ///     Anchored DFAs for `^`-anchored rules,
    ///  checked at every line start.
    /// Why:
    ///      `line_start` stores anchored DFAs for `^`-anchored rules,
    ///  checked at every line
    ///          start,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// line_start: Engine[];
    /// ```
    line_start: Vec<Engine>,
    /// What:
    ///     Rule ids paired with `line_start`,
    ///  for rule-id attribution.
    /// Why:
    ///      `line_start_ids` stores rule ids paired with `line_start`,
    ///  for rule-id
    ///          attribution,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// line_start_ids: number[];
    /// ```
    line_start_ids: Vec<usize>,
    /// Bytes that could begin a line-start rule match;
    ///  the per-line fast reject.
    ///
    /// What:
    ///  the union of every line-start rule's possible first bytes,
    ///  rebuilt on load
    /// (not serialized).
    ///  Why:
    ///  a line-start rule matches only at position zero,
    ///  so unless
    /// `line[0]` is in this set the anchored checks are skipped in one byte test.
    /// Why:
    ///      `line_start_first` stores the union of every line-start rule's possible first
    ///          bytes,
    ///  rebuilt on load (not serialized).
    ///  Why:
    ///  a line-start rule matches only at
    ///          position zero,
    ///  so unless `line[0]` is in this set the anchored checks are
    ///          skipped in one byte test,
    ///  so matcher code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// line_start_first: ByteSet;
    /// ```
    #[serde(skip)]
    line_start_first: ByteSet,
    /// What:
    ///     Ids of the literal-free rules,
    ///  run against every line for rule-id attribution.
    /// Why:
    ///      `seedless_ids` stores ids of the literal-free rules,
    ///  run against every line for
    ///          rule-id attribution,
    ///  so matcher code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seedless_ids: number[];
    /// ```
    seedless_ids: Vec<usize>,
    /// What:
    ///     Union DFAs over groups of the literal-free rules,
    ///  for the boolean fast path.
    /// Why:
    ///      `seedless_groups` stores union DFAs over groups of the literal-free rules,
    ///  for
    ///          the boolean fast path,
    ///  so matcher code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seedless_groups: Engine[];
    /// ```
    seedless_groups: Vec<Engine>,
    /// Single counting automaton over the literal-free rules (CsA measurement path).
    ///
    /// What:
    ///  one counting NFA over the alternation of every NFA-expressible seedless
    /// rule,
    ///  `None` when a seedless rule needs the product back-end.
    ///  Why:
    ///  lets the
    /// bench measure one counting pass against the unrolled DFA groups,
    ///  to see whether
    /// folding the literal-free rules into a single counting traversal beats them.
    /// Why:
    ///      `seedless_union` stores one counting NFA over the alternation of every
    ///          NFA-expressible seedless rule,
    ///  `None` when a seedless rule needs the product
    ///          back-end.
    ///  Why:
    ///  lets the bench measure one counting pass against the unrolled DFA
    ///          groups,
    ///  to see whether folding the literal-free rules into a single counting
    ///          traversal beats them,
    ///  so matcher code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seedless_union: CountingNfa | null;
    /// ```
    seedless_union: Option<CountingNfa>,
    /// What:
    ///     Combined required-literal gate over the seeded rules (never serialized).
    /// Why:
    ///      `gate` stores combined required-literal gate over the seeded rules (never
    ///          serialized),
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// gate: SetGate;
    /// ```
    #[serde(skip)]
    gate: SetGate,
}

/// Accumulates the per-rule matching structures while a ruleset is compiled.
///
/// What:
///  the parallel per-rule vectors plus the node lists for the seedless union DFAs
/// and the original-seedless oracle.
///  Why:
///  one sink keeps rule ids dense and aligned as
/// each built rule is recorded,
///  shared by the strict and lenient builders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuleSink = {
///   // fields documented in Rust above
/// };
/// ```
#[derive(Default)]
struct RuleSink {
    /// What:
    ///     Per-rule engines,
    ///  indexed by rule id.
    /// Why:
    ///      `rules` stores per-rule engines,
    ///  indexed by rule id,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rules: Engine[];
    /// ```
    rules: Vec<Engine>,
    /// What:
    ///     Per-rule anchored DFA at a leading-seed hit,
    ///  or `None`.
    /// Why:
    ///      `anchored` stores per-rule anchored DFA at a leading-seed hit,
    ///  or `None`,
    ///  so
    ///          matcher code reads that precomputed state by name instead of recomputing or
    ///          passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// anchored: Engine | null[];
    /// ```
    anchored: Vec<Option<Engine>>,
    /// What:
    ///     Anchored DFAs for `^`-anchored line-start rules.
    /// Why:
    ///      `line_start` stores anchored DFAs for `^`-anchored line-start rules,
    ///  so matcher
    ///          code reads that precomputed state by name instead of recomputing or passing it
    ///          separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// line_start: Engine[];
    /// ```
    line_start: Vec<Engine>,
    /// What:
    ///     Rule ids paired with `line_start`.
    /// Why:
    ///      `line_start_ids` stores rule ids paired with `line_start`,
    ///  so matcher code reads
    ///          that precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// line_start_ids: number[];
    /// ```
    line_start_ids: Vec<usize>,
    /// What:
    ///     Ids of the truly-seedless rules (handled by the union DFAs).
    /// Why:
    ///      `seedless_ids` stores ids of the truly-seedless rules (handled by the union
    ///          DFAs),
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seedless_ids: number[];
    /// ```
    seedless_ids: Vec<usize>,
    /// What:
    ///     Nodes of the truly-seedless rules,
    ///  combined into union DFAs.
    /// Why:
    ///      `seedless_nodes` stores nodes of the truly-seedless rules,
    ///  combined into union
    ///          DFAs,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seedless_nodes: Node[];
    /// ```
    seedless_nodes: Vec<Node>,
    /// What:
    ///     Nodes seedless at the default floor,
    ///  combined into the oracle counting union.
    /// Why:
    ///      `reference_nodes` stores nodes seedless at the default floor,
    ///  combined into the
    ///          oracle counting union,
    ///  so matcher code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reference_nodes: Node[];
    /// ```
    reference_nodes: Vec<Node>,
}

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
    fn push(&mut self, built: BuiltRule) {
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
    ///  builds the oracle
    /// counting union over the original-seedless nodes,
    ///  stores everything,
    ///  then builds
    /// the gate.
    ///  Why:
    ///  the one place that wires the matching structures together.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function assemble(): RegexSet {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn assemble(self) -> RegexSet {
        let seedless_union = build_seedless_union(&self.reference_nodes);
        let mut set = RegexSet {
            rules: self.rules,
            anchored: self.anchored,
            line_start: self.line_start,
            line_start_ids: self.line_start_ids,
            seedless_ids: self.seedless_ids,
            seedless_groups: group_seedless(self.seedless_nodes),
            seedless_union,
            gate: SetGate::default(),
            line_start_first: ByteSet::empty(),
        };
        set.prepare();
        return set
    }
}

/// A stack-only set of rule ids already fully checked on the current line.
///
/// What:
///  a 256-bit set over rule ids,
///  allocation-free.
///  Why:
///  a non-anchored rule's
/// whole-line check ignores the hit position,
///  so it need run only once per line even
/// when its seed occurs many times;
///  this records which have run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckedFull = {
///   // fields documented in Rust above
/// };
/// ```
struct CheckedFull {
    /// What:
    ///     Bit `r` set means rule `r` has already had its whole-line check this line.
    /// Why:
    ///      `bits` stores bit `r` set means rule `r` has already had its whole-line check
    ///          this line,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bits: number[];
    /// ```
    bits: [u64; 4],
}

/// What:
///     Construction and the first-seen test for the per-line dedup set.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl CheckedFull {
    /// Builds an empty set for one line.
    ///
    /// What:
    ///  all bits clear.
    ///  Why:
    ///  a fresh set per `is_match`/`matches` call.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function new(): CheckedFull {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn new() -> CheckedFull {
        return CheckedFull { bits: [0; 4] }
    }

    /// Reports whether `rule` is seen for the first time,
    ///  recording it.
    ///
    /// What:
    ///  true the first time a rule id is passed,
    ///  false after;
    ///  rule ids at or above
    /// the set's 256 capacity always report true.
    ///  Why:
    ///  the caller runs the whole-line
    /// check only on the first sighting;
    ///  beyond capacity it simply re-runs,
    ///  still sound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function first_time(rule: number): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn first_time(&mut self, rule: usize) -> bool {
        if rule >= 256 {
            return true;
        }
        let (word, bit) = (rule / 64, 1u64 << (rule % 64));
        let fresh = self.bits[word] & bit == 0;
        self.bits[word] |= bit;
        return fresh
    }
}

/// What:
///     Building,
///  matching,
///  and (de)serialization for a ruleset.
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
        let texts: Vec<&str> = patterns.iter().map(|pattern| return pattern.as_ref()).collect();
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
        return Ok(sink.assemble())
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
    fn prepare(&mut self) {
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
        let texts: Vec<&str> = patterns.iter().map(|pattern| return pattern.as_ref()).collect();
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
        return (sink.assemble(), kept)
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
    pub fn from_ruleset(text: &str, delimiter: &str) -> Result<RegexSet, CompileError> {
        let parts: Vec<&str> = text
            .split(delimiter)
            .map(|s| return s.trim())
            .filter(|s| return !s.is_empty())
            .collect();
        return RegexSet::new(&parts)
    }

    /// Reports whether any rule matches a substring of `line`.
    ///
    /// What:
    ///  the one gate pass (seeded rules whose literal occurs,
    ///  checked anchored or
    /// in full),
    ///  the `^`-anchored rules checked at every line start,
    ///  and any remaining
    /// truly-literal-free rules in a union pass.
    ///  Why:
    ///  the fold puts every rule in the
    /// gate or the cheap line-start check,
    ///  so for the shipped ruleset there is one pass.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn is_match(&self, line: &[u8]) -> bool {
        let mut checked = CheckedFull::new();
        if self
            .gate
            .any_candidate(line, |rule, pos| return self.matches_rule(line, rule, pos, &mut checked))
        {
            return true;
        }
        if self.line_start_candidate(line)
            && self.line_start.iter().any(|engine| return line_start_match(engine, line))
        {
            return true;
        }
        return self.seedless_groups.iter().any(|group| return group.is_match(line))
    }

    /// Reports whether `line` could begin a line-start rule match.
    ///
    /// What:
    ///  true when the first byte is one a line-start rule can begin with.
    ///  Why:
    ///  a
    /// line-start rule matches only at position zero,
    ///  so this one-byte test skips the
    /// anchored checks on almost every line (the deny-code markers begin few lines).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function line_start_candidate(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn line_start_candidate(&self, line: &[u8]) -> bool {
        return line.first().is_some_and(|&b| return self.line_start_first.contains(b))
    }

    /// Checks one seeded rule against `line`,
    ///  anchored at `pos` when possible.
    ///
    /// What:
    ///  runs the rule's anchored DFA over `line[pos..]` when the seed is the rule's
    /// leading literal;
    ///  otherwise runs the whole-line engine once per line (the result
    /// ignores `pos`),
    ///  skipping its now-redundant prefilter.
    ///  Why:
    ///  anchoring at the hit
    /// replaces the slow per-rule scan,
    ///  deduping a non-anchored rule avoids re-scanning
    /// the line for each repeat of its seed,
    ///  and the gate already proved the seed present.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function matches_rule(line: Uint8Array, rule: number, pos: number, checked: CheckedFull): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn matches_rule(&self, line: &[u8], rule: usize, pos: usize, checked: &mut CheckedFull) -> bool {
        match &self.anchored[rule] {
            Some(engine) => return engine.is_match(&line[pos..]),
            None => return checked.first_time(rule) && self.rules[rule].matches_only(line),
        }
    }

    /// Returns the ids of the rules that match `line`.
    ///
    /// What:
    ///  collects gate hits,
    ///  then the `^`-anchored line-start hits,
    ///  then any
    /// truly-literal-free hits.
    ///  Why:
    ///  each routing path attributes its own rule ids.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function matches(line: Uint8Array): Iterable<number> {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn matches(&self, line: &[u8]) -> impl Iterator<Item = usize> {
        let mut hits: Vec<usize> = Vec::new();
        let mut checked = CheckedFull::new();
        self.gate.for_each_candidate(line, |rule, pos| {
            if self.matches_rule(line, rule, pos, &mut checked) {
                hits.push(rule);
            }
        });
        if self.line_start_candidate(line) {
            for (engine, &id) in self.line_start.iter().zip(&self.line_start_ids) {
                if line_start_match(engine, line) {
                    hits.push(id);
                }
            }
        }
        for &id in &self.seedless_ids {
            if self.rules[id].is_match(line) {
                hits.push(id);
            }
        }
        hits.sort_unstable();
        hits.dedup();
        return hits.into_iter()
    }

    /// Profiling hook:
    ///  runs only the seeded-rule gate path.
    ///
    /// What:
    ///  the gate candidates,
    ///  skipping the literal-free rules.
    ///  Why:
    ///  lets the
    /// bench split per-line time between the gate and the literal-free scans.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function gate_only_is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn gate_only_is_match(&self, line: &[u8]) -> bool {
        let mut checked = CheckedFull::new();
        return self.gate
            .any_candidate(line, |rule, pos| return self.matches_rule(line, rule, pos, &mut checked))
    }

    /// Profiling hook:
    ///  runs only the seeded-literal prefilter,
    ///  no fallback.
    ///
    /// What:
    ///  the gate's prefilter presence test alone.
    ///  Why:
    ///  separates the prefilter's
    /// cost from the per-rule counting fallback it triggers on a hit.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function prefilter_only_is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn prefilter_only_is_match(&self, line: &[u8]) -> bool {
        return self.gate.prefilter_present(line)
    }

    /// Profiling hook:
    ///  prefilter plus aho-corasick enumeration,
    ///  no per-rule check.
    ///
    /// What:
    ///  the gate path with the per-rule predicate stubbed to never match.
    ///  Why:
    /// isolates the which-rule enumeration cost from the per-rule counting cost.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function candidates_only_is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn candidates_only_is_match(&self, line: &[u8]) -> bool {
        return self.gate.any_candidate(line, |_rule, _pos| return false)
    }

    /// Profiling hook:
    ///  the gate path but skipping rules without an anchored DFA.
    ///
    /// What:
    ///  runs only anchored per-rule checks,
    ///  treating counting-fallback rules as
    /// non-matching.
    ///  Why:
    ///  isolates the anchored-check cost from the slow counting
    /// fallback,
    ///  to see which dominates the gate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function gate_anchored_only_is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn gate_anchored_only_is_match(&self, line: &[u8]) -> bool {
        return self.gate.any_candidate(line, |rule, pos| match &self.anchored[rule] {
            Some(engine) => return engine.is_match(&line[pos..]),
            None => return false,
        })
    }

    /// Profiling hook:
    ///  runs only the literal-free group DFAs.
    ///
    /// What:
    ///  the seedless union engines,
    ///  skipping the gate.
    ///  Why:
    ///  the other half of the
    /// per-line time split.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function seedless_only_is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn seedless_only_is_match(&self, line: &[u8]) -> bool {
        return self.seedless_groups.iter().any(|group| return group.is_match(line))
    }

    /// Profiling hook:
    ///  runs only the single counting-union automaton.
    ///
    /// What:
    ///  the one counting NFA over every seedless rule,
    ///  skipping the gate and the
    /// unrolled DFA groups.
    ///  Why:
    ///  measures whether one counting pass over the
    /// literal-free rules beats the unrolled DFA groups it would replace.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function csa_only_is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn csa_only_is_match(&self, line: &[u8]) -> bool {
        return self.seedless_union.as_ref().is_some_and(|nfa| return nfa.is_match(line))
    }

    /// Returns the position count of the seedless counting union,
    ///  or zero when absent.
    ///
    /// What:
    ///  how many NFA positions the single counting pass carries.
    ///  Why:
    ///  a
    /// diagnostic for the per-byte cost of the counting union against the DFA groups.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function seedless_union_size(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn seedless_union_size(&self) -> usize {
        return self.seedless_union.as_ref().map_or(0, |nfa| return nfa.elements.len())
    }

    /// Returns how many seeded rules have an anchored DFA fast-check.
    ///
    /// What:
    ///  the count of rules whose seed is their leading literal.
    ///  Why:
    ///  a diagnostic
    /// for how much of the gate fallback avoids the slow counting scan.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function anchored_count(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn anchored_count(&self) -> usize {
        return self.anchored.iter().filter(|engine| return engine.is_some()).count()
    }

    /// Returns how many union DFAs the seedless rules collapsed into.
    ///
    /// What:
    ///  the group count.
    ///  Why:
    ///  a diagnostic for how well the literal-free rules
    /// combine;
    ///  zero means every literal-free rule folded into the one gate pass.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function seedless_group_count(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn seedless_group_count(&self) -> usize {
        return self.seedless_groups.len()
    }

    /// Returns how many `^`-anchored rules are checked at line starts.
    ///
    /// What:
    ///  the line-start rule count.
    ///  Why:
    ///  a diagnostic for the fold,
    ///  since these
    /// rules left the per-line scan for a cheap anchored check at line starts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function line_start_count(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn line_start_count(&self) -> usize {
        return self.line_start.len()
    }

    /// Returns how many rules have no required-literal prefilter.
    ///
    /// What:
    ///  counts engines whose seed set is empty.
    ///  Why:
    ///  a diagnostic for tuning
    /// the prefilter,
    ///  since seedless rules run against every line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function seedless_count(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn seedless_count(&self) -> usize {
        return self.rules.iter().filter(|engine| return engine.seeds().is_none()).count()
    }

    /// Returns the number of rules.
    ///
    /// What:
    ///  the rule count.
    ///  Why:
    ///  callers index `matches` results against rules.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function len(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn len(&self) -> usize {
        return self.rules.len()
    }

    /// Reports whether the set has no rules.
    ///
    /// What:
    ///  rule count is zero.
    ///  Why:
    ///  the conventional companion to `len`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_empty(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        return self.rules.is_empty()
    }

    /// Checks the decoded set's parallel-vector indices are mutually consistent.
    ///
    /// What:
    ///  `anchored` must be one per rule,
    ///  the line-start engines and ids must pair
    /// up,
    ///  and every attribution id (`seedless_ids`,
    ///  `line_start_ids`) must index a real
    /// rule.
    ///  Why:
    ///  matching indexes `anchored[rule]`/`rules[id]` from the gate and these
    /// id lists,
    ///  so a hostile or corrupt serialization with mismatched lengths would
    /// read out of bounds;
    ///  this rejects it before any match runs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function validate_structure(): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn validate_structure(&self) -> Result<(), CompileError> {
        let invalid = |message: &str| return CompileError::Invalid {
            message: message.to_string(),
        };
        let rules = self.rules.len();
        if self.anchored.len() != rules {
            return Err(invalid("anchored length does not match rule count"));
        }
        if self.line_start.len() != self.line_start_ids.len() {
            return Err(invalid("line-start engines and ids length mismatch"));
        }
        if self.seedless_ids.iter().chain(&self.line_start_ids).any(|&id| return id >= rules) {
            return Err(invalid("attribution id out of range"));
        }
        return Ok(())
    }

    /// Serializes the compiled ruleset to bytes.
    ///
    /// What:
    ///  bincode-encodes every rule engine.
    ///  Why:
    ///  the pre-serialized form the
    /// throughput benchmark loads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function to_bytes(): number[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn to_bytes(&self) -> Result<Vec<u8>, CompileError> {
        return bincode::serialize(self).map_err(|e| return CompileError::Serialize {
            message: e.to_string(),
        })
    }

    /// Loads a compiled ruleset from bytes,
    ///  validating every engine.
    ///
    /// What:
    ///  decodes then validates each rule engine.
    ///  Why:
    ///  every engine is
    /// executed,
    ///  so all must be proven in-bounds before use.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function from_bytes(bytes: Uint8Array): RegexSet {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<RegexSet, CompileError> {
        let mut set: RegexSet = bincode::deserialize(bytes).map_err(|e| return CompileError::Invalid {
            message: e.to_string(),
        })?;
        // What: prove the cross-vector indices are consistent before anything indexes
        // them. Why: the per-engine validation below cannot catch a decoded set whose
        // parallel vectors disagree (e.g. a `seedless_id` past `rules`, or `anchored`
        // shorter than `rules`), which would panic at match time on hostile bytes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        set.validate_structure()?;
        // What: validate the decoded graph, then rebuild its runtime prefilter. Why:
        // both must happen before any engine runs on untrusted input.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        for engine in set
            .rules
            .iter_mut()
            .chain(set.seedless_groups.iter_mut())
            .chain(set.line_start.iter_mut())
            .chain(set.anchored.iter_mut().flatten())
        {
            engine.validate()?;
            engine.prepare();
        }
        // What: validate the decoded counting union before it runs. Why: it is
        // executed against untrusted input like every other back-end.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        if let Some(nfa) = &set.seedless_union {
            nfa.validate()?;
        }
        // What: rebuild the combined gate from the prepared engines. Why: it is not
        // serialized, and the fast path depends on it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        set.prepare();
        return Ok(set)
    }
}

/// What:
///     Unit tests for the public matchers' internals,
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
#[path = "regex_tests.rs"]
mod tests;
