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
///     Imports the back-end selection for a parsed pattern.
/// Why:
///      The code below uses `build_engine` directly;
///  importing from `crate/build` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { build_engine } from "crate/build";
/// ```
use crate::build::build_engine;
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
use serde::{
    Deserialize,
    Serialize,
};
/// Registers the per-line checks the gate walk and the line scans use.
mod line_checks;
/// What:
///     Building a ruleset from pattern text.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./ruleset_construction";
/// ```
mod ruleset_construction;
/// What:
///     The counts a built ruleset reports about what it holds.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./ruleset_diagnostics";
/// ```
mod ruleset_diagnostics;
/// What:
///     Matching one line against a ruleset.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./ruleset_matching";
/// ```
mod ruleset_matching;
/// What:
///     The profiling hooks that run one component of a line match on its own.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./ruleset_profiling";
/// ```
mod ruleset_profiling;
/// What:
///     Persisting a built ruleset to bytes and loading one back.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./ruleset_serialization";
/// ```
mod ruleset_serialization;
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
    pub fn is_match(
        &self,
        line: &[u8],
    ) -> bool {
        return self.engine.is_match(line);
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
        return bincode::serialize(self).map_err(CompileError::serialize_failed);
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
        let mut regex: Regex = bincode::deserialize(bytes).map_err(CompileError::undecodable)?;
        regex.engine.validate()?;
        // What: rebuild the prefilter from the validated graph. Why: it is not
        // serialized, and matching depends on it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        regex.engine.prepare();
        return Ok(regex);
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
    let node = match parse(pattern) {
        Ok(node) => node,
        Err(error) => {
            tracing::warn!(pattern, cause = %error, "regex parse failed");
            return Err(error);
        }
    };
    let engine = match build_engine(node) {
        Ok(engine) => engine,
        Err(error) => {
            tracing::warn!(pattern, cause = %error, "regex build failed");
            return Err(error);
        }
    };
    return Ok(Regex {
        engine,
    });
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
