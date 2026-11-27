//! What:
//!     Matching one line against a [`RegexSet`]:
//!  the boolean answer,
//!  the rule ids,
//!  and the per-rule resolution both walk.
//! Why:
//!      This file is the Rust module that groups the line matching,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module ruleset_matching: see the impl block below.
//! ```

/// What:
///     Imports the ruleset type these functions match a line against.
/// Why:
///      The code below uses `RegexSet` directly;
///  importing from `./super` keeps each call
///          site focused on the matching logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegexSet } from "./super";
/// ```
use super::RegexSet;

/// What:
///     Imports the per-line checks and the first-seen set the two match paths use.
/// Why:
///      The code below uses `CheckedFull`,
///  `RuleCheck`,
///  `RuleCollector`,
///  `any_group_matches`,
///          `any_line_start_match` directly;
///  importing from `./line_checks` keeps each call site
///          focused on the matching logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   CheckedFull,
///   RuleCheck,
///   RuleCollector,
///   any_group_matches,
///   any_line_start_match,
/// } from "./line_checks";
/// ```
use super::line_checks::{
    CheckedFull,
    RuleCheck,
    RuleCollector,
    any_group_matches,
    any_line_start_match,
};

/// What:
///     Imports the anchored line-start match used by the rule-id path.
/// Why:
///      The code below uses `line_start_match` directly;
///  importing from `crate/build` keeps
///          each call site focused on the matching logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { line_start_match } from "crate/build";
/// ```
use crate::build::line_start_match;

/// What:
///     Matching a line and naming the rules that matched it.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl RegexSet {
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
    pub fn is_match(
        &self,
        line: &[u8],
    ) -> bool {
        let mut rule_check = RuleCheck {
            set: self,
            line,
            checked: CheckedFull::new(),
        };
        if self.gate.any_candidate(line, &mut rule_check) {
            return true;
        }
        if self.line_start_candidate(line) && any_line_start_match(&self.line_start, line) {
            return true;
        }
        return any_group_matches(&self.seedless_groups, line);
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
    pub(super) fn line_start_candidate(
        &self,
        line: &[u8],
    ) -> bool {
        return match line.first() {
            Some(&byte) => self.line_start_first.contains(byte),
            None => false,
        };
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
    pub(super) fn matches_rule(
        &self,
        line: &[u8],
        rule: usize,
        pos: usize,
        checked: &mut CheckedFull,
    ) -> bool {
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
    pub fn matches(
        &self,
        line: &[u8],
    ) -> impl Iterator<Item = usize> {
        let mut collector = RuleCollector {
            set: self,
            line,
            checked: CheckedFull::new(),
            hits: Vec::new(),
        };
        self.gate.for_each_candidate(line, &mut collector);
        let mut hits: Vec<usize> = collector.hits;
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
        return hits.into_iter();
    }
}
