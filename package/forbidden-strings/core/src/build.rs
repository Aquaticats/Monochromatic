//! Per-rule back-end selection and the seedless-rule fold into the gate.
//!
//! What:
//!  turns a parsed node into a [`BuiltRule`] (its engine plus how it is matched:
//! gate seeds,
//!  an anchored DFA at the hit,
//!  or a line-start check),
//!  and provides the
//! single-pattern engine builder,
//!  the oracle counting union,
//!  and the line-start
//! matcher.
//!  Why:
//!  keeping the construction logic here keeps the public-API module within
//! its line budget,
//!  and the fold is the lever that puts every rule in one pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module build: see exported functions and types below.
//! ```

/// What:
///     Imports the node algebra the builders read.
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
///     Imports the alternation and concatenation constructors.
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
///     Imports the NFA,
///  product,
///  and counting-NFA types for the back-end selection.
/// Why:
///      The code below uses `CountingNfa`,
///  `build_nfa`,
///  `build_product` directly;
///  importing
///          from `crate/counting` keeps each call site focused on the matcher logic instead of
///          the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CountingNfa, build_nfa, build_product } from "crate/counting";
/// ```
use crate::counting::{CountingNfa, build_nfa, build_product};

/// What:
///     Imports the seed extractors and leading-seed probes (default and weak floors).
/// Why:
///      The code below uses `leading_seeds`,
///  `leading_seeds_min`,
///  `seeds_from_node`,
///          `seeds_from_node_min` directly;
///  importing from `crate/counting` keeps each call site
///          focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   leading_seeds,
///   leading_seeds_min,
///   seeds_from_node,
///   seeds_from_node_min,
/// } from "crate/counting";
/// ```
use crate::counting::{leading_seeds, leading_seeds_min, seeds_from_node, seeds_from_node_min};

/// What:
///     Imports the DFA builder and minimizer for the general back-end.
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
///     Imports the per-pattern back-end and its kind.
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

/// Largest DFA the back-end selector makes before falling back to counting.
///
/// What:
///  a state ceiling for a seedless rule's DFA and the seeded last resort.
///  Why:
/// bounds build time against a pathological structure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ENGINE_DFA_CAP: number = 20_000;
/// ```
const ENGINE_DFA_CAP: usize = 20_000;

/// Shortest leading literal the fold will gate an otherwise-seedless rule on.
///
/// What:
///  a two-byte floor for a weak LEADING seed (anchored at the hit).
///  Why:
///  a rule
/// seedless at [`crate::counting`]'s default floor would otherwise need a second
/// per-line pass;
///  a two-byte leading literal (`SK`,
///  `s.`) gates it into the one gate
/// pass with a cheap anchored check that dies fast on the non-matches it over-flags.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WEAK_LEADING_SEED_LEN: number = 2;
/// ```
const WEAK_LEADING_SEED_LEN: usize = 2;

/// Shortest inner required literal the fold will gate an otherwise-seedless rule on.
///
/// What:
///  a one-byte floor for a weak INNER seed (the full engine runs on a hit).
///  Why:
/// facebook's only required literal is the one-byte alternation `|`/`%`,
///  which is rare
/// enough in code to gate on;
///  folding it in deletes the last second-pass rule.
///  A
/// common one-byte seed would over-flag,
///  but only a rule with no longer literal and no
/// line anchor reaches here,
///  and even then the gate is no worse than a second pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WEAK_INNER_SEED_LEN: number = 1;
/// ```
const WEAK_INNER_SEED_LEN: usize = 1;

/// Wraps a pattern node for unanchored search.
///
/// What:
///  prefixes the node with `Top` (sigma star),
///  giving `Σ*·R`.
///  Why:
///  a nullable
/// residual of `Σ*·R` at any boundary means `R` matched some substring ending there,
/// which is exactly substring search;
///  the counting back-end models the prefix itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function search_root(node: Node): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn search_root(node: Node) -> Node {
    return concat(vec![Node::Top, node])
}

/// Builds a bare anchored DFA engine for a rule,
///  or `None` past the cap.
///
/// What:
///  determinizes the node with no `Σ*` prefix and minimizes it.
///  Why:
///  anchored
/// there is no overlap blowup,
///  so even a literal-plus-counted rule (and `&`/`~`) is a
/// small linear DFA the gate runs at the hit position,
///  replacing the slow per-rule
/// counting scan that real code triggers on every keyword hit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anchored_engine(node: Node): Engine | null {
///   // Rust body below is the implementation.
/// }
/// ```
fn anchored_engine(node: &Node) -> Option<Engine> {
    return build_dfa_within(node.clone(), ENGINE_DFA_CAP)
        .ok()
        .map(|dfa| return Engine::new(EngineKind::Table(minimize(&dfa)), Vec::new()))
}

/// Builds the eager-DFA back-end for a seedless node,
///  or counting on overrun.
///
/// What:
///  determinizes under the cap;
///  on a cap overrun falls back to counting.
///  Why:
/// seedless rules want the fast DFA,
///  but a rare overrun must still be representable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_table_kind(node: Node): EngineKind {
///   // Rust body below is the implementation.
/// }
/// ```
fn build_table_kind(node: Node) -> Result<EngineKind, CompileError> {
    match build_dfa_within(search_root(node.clone()), ENGINE_DFA_CAP) {
        Ok(dfa) => return Ok(EngineKind::Table(minimize(&dfa))),
        Err(CompileError::StateCap { .. }) => return build_counting_kind(node),
        Err(other) => return Err(other),
    }
}

/// Builds a counting back-end for a node.
///
/// What:
///  a counting NFA for a node without `&`/`~`,
///  a synchronized product for one
/// with them,
///  else the full-cap DFA as a last resort.
///  Why:
///  these stay small where the
/// eager DFA explodes on bounded repetition.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_counting_kind(node: Node): EngineKind {
///   // Rust body below is the implementation.
/// }
/// ```
fn build_counting_kind(node: Node) -> Result<EngineKind, CompileError> {
    if let Some(nfa) = build_nfa(&node) {
        return Ok(EngineKind::Nfa(nfa));
    }
    if let Some(program) = build_product(&node) {
        return Ok(EngineKind::Product(program));
    }
    // What: a capped DFA as the last resort. Why: nothing else expressed this node,
    // and a capped build fails fast (the rule is dropped) rather than grinding.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    return Ok(EngineKind::Table(minimize(&build_dfa_within(
        search_root(node),
        ENGINE_DFA_CAP,
    )?)))
}

/// Compiles one parsed node into the engine best suited to it.
///
/// What:
///  a seedless rule gets the O(1)-per-byte DFA (no literal prefix,
///  so only linear
/// in any repetition bound),
///  a seeded rule the build-fast counting back-end.
///  Why:
///  fast
/// where it matters,
///  small where it must be,
///  both building quickly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_engine(node: Node): Engine {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn build_engine(node: Node) -> Result<Engine, CompileError> {
    let seeds = seeds_from_node(&node);
    let kind = if seeds.is_empty() {
        build_table_kind(node)?
    } else {
        build_counting_kind(node)?
    };
    return Ok(Engine::new(kind, seeds))
}

/// How a rule is matched after the fold:
///  gated,
///  anchored at the hit,
///  or line-start.
///
/// What:
///  the seeds the gate prefilters on (empty for a line-start or truly-seedless
/// rule),
///  an optional anchored DFA run at the hit,
///  and an optional line-start DFA run
/// at every line start.
///  Why:
///  every rule should ride the one gate pass,
///  so the fold
/// chooses,
///  per rule,
///  the cheapest sound way to fit it there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Routing = {
///   // fields documented in Rust above
/// };
/// ```
pub(crate) struct Routing {
    /// What:
    ///     Required-literal seeds the gate prefilters and attributes on;
    ///  empty when none.
    /// Why:
    ///      `seeds` stores required-literal seeds the gate prefilters and attributes on;
    ///          empty when none,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seeds: number[][];
    /// ```
    pub(crate) seeds: Vec<Vec<u8>>,
    /// What:
    ///     Anchored DFA run at a leading-seed hit,
    ///  when the rule has a leading literal.
    /// Why:
    ///      `anchored` stores anchored DFA run at a leading-seed hit,
    ///  when the rule has a
    ///          leading literal,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// anchored: Engine | null;
    /// ```
    pub(crate) anchored: Option<Engine>,
    /// What:
    ///     Anchored DFA run at every line start,
    ///  for a `^`-anchored otherwise-seedless
    ///          rule.
    /// Why:
    ///      `line_start` stores anchored DFA run at every line start,
    ///  for a `^`-anchored
    ///          otherwise-seedless rule,
    ///  so matcher code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// line_start: Engine | null;
    /// ```
    pub(crate) line_start: Option<Engine>,
}

/// One compiled rule plus how it is matched.
///
/// What:
///  the engine,
///  its routing (seeds / anchored / line-start),
///  whether it is still
/// truly literal-free,
///  and the node kept for a literal-free rule (to join a union DFA)
/// or for the original-seedless oracle.
///  Why:
///  the sink records each by rule id;
///  the
/// truly-seedless nodes form union DFAs,
///  the original-seedless nodes the oracle union.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BuiltRule = {
///   // fields documented in Rust above
/// };
/// ```
pub(crate) struct BuiltRule {
    /// What:
    ///     The compiled per-rule engine.
    /// Why:
    ///      `engine` stores the compiled per-rule engine,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// engine: Engine;
    /// ```
    pub(crate) engine: Engine,
    /// What:
    ///     How the rule is matched after the fold.
    /// Why:
    ///      `routing` stores how the rule is matched after the fold,
    ///  so matcher code reads
    ///          that precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// routing: Routing;
    /// ```
    pub(crate) routing: Routing,
    /// What:
    ///     Whether the rule still has no usable filter (gate,
    ///  anchor,
    ///  nor line-start).
    /// Why:
    ///      `seedless` stores whether the rule still has no usable filter (gate,
    ///  anchor,
    ///  nor
    ///          line-start),
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seedless: boolean;
    /// ```
    pub(crate) seedless: bool,
    /// What:
    ///     Parsed node,
    ///  kept for a truly-seedless rule so it can join a union DFA.
    /// Why:
    ///      `node` stores parsed node,
    ///  kept for a truly-seedless rule so it can join a union
    ///          DFA,
    ///  so matcher code reads that precomputed state by name instead of recomputing
    ///          or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// node: Node | null;
    /// ```
    pub(crate) node: Option<Node>,
    /// What:
    ///     Parsed node,
    ///  kept for any rule seedless at the default floor,
    ///  for the oracle.
    /// Why:
    ///      `reference_node` stores parsed node,
    ///  kept for any rule seedless at the default
    ///          floor,
    ///  for the oracle,
    ///  so matcher code reads that precomputed state by name
    ///          instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reference_node: Node | null;
    /// ```
    pub(crate) reference_node: Option<Node>,
}

/// Reports whether a node can match only at a line start (`^` at its head).
///
/// What:
///  true when the node is a concatenation whose first factor is `^`.
///  Why:
///  such a
/// rule's matches all begin at a line start,
///  so it is checked there instead of by a
/// per-line substring scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function starts_with_line_anchor(node: Node): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn starts_with_line_anchor(node: &Node) -> bool {
    return matches!(node, Node::Concat(parts) if parts.first() == Some(&Node::LineStart))
}

/// Routes a rule seedless at the default floor onto a weak seed.
///
/// What:
///  gate on a weak leading literal (anchored at the hit) when present,
///  else on
/// any weak inner required literal (the full engine runs on a hit).
///  Why:
///  a second
/// per-line pass caps the combined rate below regex,
///  so every literal-free rule is
/// folded into the gate;
///  each route is sound because the seed is a required literal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fold_seedless(node: Node): Routing {
///   // Rust body below is the implementation.
/// }
/// ```
fn fold_seedless(node: &Node) -> Routing {
    let weak_leading = leading_seeds_min(node, WEAK_LEADING_SEED_LEN);
    if !weak_leading.is_empty() {
        return Routing {
            anchored: anchored_engine(node),
            seeds: weak_leading,
            line_start: None,
        };
    }
    return Routing {
        seeds: seeds_from_node_min(node, WEAK_INNER_SEED_LEN),
        anchored: None,
        line_start: None,
    }
}

/// Chooses how a rule is matched:
///  line-start,
///  default seeds,
///  else the seedless fold.
///
/// What:
///  a `^`-anchored rule is checked at line starts (off the gate entirely);
///  else
/// gate on the leading literal when selective (anchored at the hit),
///  else on the best
/// inner literal,
///  else the seedless fold.
///  Why:
///  a marker rule's common short codes make
/// terrible gate seeds,
///  but it matches only at line starts,
///  so a cheap pos-zero check
/// beats flagging it as a substring;
///  anchoring at the hit replaces the slow counting
/// scan,
///  and folding deletes the second per-line pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function route_rule(node: Node): Routing {
///   // Rust body below is the implementation.
/// }
/// ```
fn route_rule(node: &Node) -> Routing {
    if starts_with_line_anchor(node) {
        return Routing {
            seeds: Vec::new(),
            anchored: None,
            line_start: anchored_engine(node),
        };
    }
    let leading = leading_seeds(node);
    if !leading.is_empty() {
        return Routing {
            anchored: anchored_engine(node),
            seeds: leading,
            line_start: None,
        };
    }
    let seeds = seeds_from_node(node);
    if !seeds.is_empty() {
        return Routing {
            seeds,
            anchored: None,
            line_start: None,
        };
    }
    return fold_seedless(node)
}

/// Compiles one parsed node into a rule,
///  folding literal-free rules into the gate.
///
/// What:
///  routes the rule (default seeds,
///  anchored,
///  line-start,
///  or last-resort
/// seedless),
///  then builds the back-end best suited to it.
///  Why:
///  a literal-free rule
/// that cannot be folded still falls back to a union DFA,
///  but the fold keeps that set
/// empty for this ruleset so there is one pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_rule(node: Node): BuiltRule {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn build_rule(node: Node) -> Result<BuiltRule, CompileError> {
    let was_seedless = leading_seeds(&node).is_empty() && seeds_from_node(&node).is_empty();
    let routing = route_rule(&node);
    let anchorable = routing.anchored.is_some();
    let seedless = routing.seeds.is_empty() && routing.line_start.is_none();
    // What:    An anchorable rule matches via its anchored DFA at the hit, so its own engine
    //          is unused and takes the build-fast counting back-end. A non-anchorable rule's
    //          engine IS its matcher, run over the whole line on every hit, so it takes the
    //          eager DFA for a fast per-byte check; `build_table_kind` falls back to counting
    //          if the DFA would explode. This keeps the inner-keyword rules off the slow
    //          counting scan they pay on common-keyword hits.
    // Why:     The surrounding function uses this step to keep the matcher behavior correct at
    //          this point.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let kind = if anchorable {
        build_counting_kind(node.clone())?
    } else {
        build_table_kind(node.clone())?
    };
    return Ok(BuiltRule {
        engine: Engine::new(kind, routing.seeds.clone()),
        routing,
        seedless,
        node: if seedless { Some(node.clone()) } else { None },
        reference_node: if was_seedless { Some(node) } else { None },
    })
}

/// Builds one counting automaton over every NFA-expressible seedless rule.
///
/// What:
///  the counting NFA of the alternation of the seedless nodes,
///  or `None` when
/// there are none or any needs the product back-end.
///  Why:
///  the oracle the bench checks
/// the folded `is_match` against (no literal-free rule's match may be missed).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_seedless_union(nodes: Node[]): CountingNfa | null {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn build_seedless_union(nodes: &[Node]) -> Option<CountingNfa> {
    if nodes.is_empty() {
        return None;
    }
    return build_nfa(&alt(nodes.to_vec()))
}

/// Reports whether a `^`-anchored engine matches at the start of `line`.
///
/// What:
///  runs the bare anchored DFA,
///  which matches only a prefix beginning at a line
/// start.
///  Why:
///  the engine's contract is one line per call,
///  so `^` is position zero;
/// the anchored DFA dies on the first non-matching byte,
///  making this an O(1) check on
/// almost every line instead of the whole-line newline scan a multi-line search needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function line_start_match(engine: Engine, line: Uint8Array): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn line_start_match(engine: &Engine, line: &[u8]) -> bool {
    return engine.is_match(line)
}

/// What:
///     Unit tests for rule routing,
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
#[path = "build_tests.rs"]
mod tests;
