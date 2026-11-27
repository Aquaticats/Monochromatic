//! What:
//!     The profiling hooks of a [`RegexSet`]:
//!  matching paths that run one component of a line match on its own,
//!  so a benchmark can attribute per-line time to the gate,
//!  the anchored checks,
//!  the literal-free groups or the counting union.
//! Why:
//!      This file is the Rust module that groups the profiling hooks,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module ruleset_profiling: see the impl block below.
//! ```

/// What:
///     Imports the ruleset type these hooks measure.
/// Why:
///      The code below uses `RegexSet` directly;
///  importing from `./super` keeps each call
///          site focused on the measuring logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegexSet } from "./super";
/// ```
use super::RegexSet;

/// What:
///     Imports the per-line checks a partial match runs,
///  including the stub that rejects
///          every candidate.
/// Why:
///      The code below uses `AnchoredRuleCheck`,
///  `CheckedFull`,
///  `RejectEveryCandidate`,
///  `RuleCheck`,
///          `any_group_matches` directly;
///  importing from `./line_checks` keeps each call
///          site focused on the measuring logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   AnchoredRuleCheck,
///   CheckedFull,
///   RejectEveryCandidate,
///   RuleCheck,
///   any_group_matches,
/// } from "./line_checks";
/// ```
use super::line_checks::{
    AnchoredRuleCheck,
    CheckedFull,
    RejectEveryCandidate,
    RuleCheck,
    any_group_matches,
};

/// What:
///     Running one component of a line match on its own.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl RegexSet {
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
    pub fn gate_only_is_match(
        &self,
        line: &[u8],
    ) -> bool {
        let mut rule_check = RuleCheck {
            set: self,
            line,
            checked: CheckedFull::new(),
        };
        return self.gate.any_candidate(line, &mut rule_check);
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
    pub fn prefilter_only_is_match(
        &self,
        line: &[u8],
    ) -> bool {
        return self.gate.prefilter_present(line);
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
    pub fn candidates_only_is_match(
        &self,
        line: &[u8],
    ) -> bool {
        return self.gate.any_candidate(line, &mut RejectEveryCandidate);
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
    pub fn gate_anchored_only_is_match(
        &self,
        line: &[u8],
    ) -> bool {
        return self.gate.any_candidate(
            line,
            &mut AnchoredRuleCheck {
                set: self,
                line,
            },
        );
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
    pub fn seedless_only_is_match(
        &self,
        line: &[u8],
    ) -> bool {
        return any_group_matches(&self.seedless_groups, line);
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
    pub fn csa_only_is_match(
        &self,
        line: &[u8],
    ) -> bool {
        return match &self.seedless_union {
            Some(nfa) => nfa.is_match(line),
            None => false,
        };
    }
}
